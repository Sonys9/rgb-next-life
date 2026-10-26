use core::{
    mem::{self, transmute},
    net::{Ipv4Addr, SocketAddrV4},
};

use alloc::string::ToString;
use embassy_executor::Spawner;
use embassy_net::{Ipv4Cidr, Runner, Stack, StackResources, StaticConfigV4};
use embassy_time::{Duration, Timer};
use esp_hal::{peripherals::WIFI, rng::Rng};
use esp_radio::wifi::{
    self, Config, ControllerConfig, Interface, Password, Ssid, WifiController,
    ap::{self, AccessPointConfig},
};

use edge_dhcp::{
    io::{self, DEFAULT_SERVER_PORT},
    server::{Server, ServerOptions},
};
use edge_nal::UdpBind;
use edge_nal_embassy::{Udp, UdpBuffers};

use crate::mk_static;

pub struct AP {
    controller: WifiController<'static>,
    gateaway: Ipv4Addr,
}

impl AP {
    pub fn new<'a>(
        ssid: &'a str,
        password: &'a str,
        gateaway: Ipv4Addr,
        wifi: WIFI<'static>,
    ) -> Self {
        let controller_config =
            ControllerConfig::default().with_initial_config(wifi::Config::AccessPoint(
                AccessPointConfig::default()
                    .with_ssid(ssid.try_into().unwrap())
                    .with_authentication(wifi::AuthenticationMethodConfig::Wpa2Personal(
                        password.try_into().unwrap(),
                    )),
            ));
        let controller = WifiController::new(wifi, controller_config).unwrap();
        Self {
            controller,
            gateaway,
        }
    }

    pub async fn start<'a>(&self, spawner: &Spawner) -> Stack<'a> {
        let rng = Rng::new();
        let seed = (rng.random() as u64) << 32 | rng.random() as u64;
        let (stack, runner) = embassy_net::new(
            Interface::access_point(),
            embassy_net::Config::ipv4_static(StaticConfigV4 {
                address: Ipv4Cidr::new(self.gateaway, 24),
                gateway: None,
                dns_servers: Default::default(),
            }),
            mk_static!(StackResources::<3>, StackResources::<3>::new()),
            seed,
        );
        spawner.spawn(net_task(runner).unwrap());
        spawner.spawn(run_dhcp(self.gateaway, stack).unwrap());

        let controller: WifiController<'static> = unsafe { mem::transmute_copy(&self.controller) };
        spawner.spawn(connection(controller).unwrap());

        stack.wait_config_up().await;
        stack
            .config_v4()
            .inspect(|config| info!("config ipv4: {:?}", config));
        stack
    }
}

#[embassy_executor::task]
async fn run_dhcp(gateaway: Ipv4Addr, stack: Stack<'static>) {
    let mut buf = [0u8; 1500];

    let mut gw_buf = [Ipv4Addr::UNSPECIFIED];

    let buffers = UdpBuffers::<3, 1024, 1024, 10>::new();
    let unbound_socket = Udp::new(stack, &buffers);
    let mut bound_socket = unbound_socket
        .bind(core::net::SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::UNSPECIFIED,
            DEFAULT_SERVER_PORT,
        )))
        .await
        .unwrap();

    loop {
        _ = io::server::run(
            &mut Server::<_, 64>::new_with_et(gateaway),
            &ServerOptions::new(gateaway, Some(&mut gw_buf)),
            &mut bound_socket,
            &mut buf,
        )
        .await
        .inspect_err(|e| warn!("DHCP server error: {e:?}"));
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[embassy_executor::task]
async fn connection(controller: WifiController<'static>) {
    info!("Spawned a connection handler");
    loop {
        let ev = controller
            .wait_for_access_point_connected_event_async()
            .await;
        match ev {
            Ok(esp_radio::wifi::ap::EventInfo::Connected(info)) => {
                info!("+ {:?}", info);
            }
            Ok(esp_radio::wifi::ap::EventInfo::Disconnected(info)) => {
                info!("- {:?}", info);
            }
            _ => (),
        }
        Timer::after(Duration::from_millis(5000)).await
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, Interface>) {
    runner.run().await
}
