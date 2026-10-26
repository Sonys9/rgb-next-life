use core::net::{Ipv4Addr, SocketAddrV4};

use alloc::string::ToString;
use embassy_executor::Spawner;
use embassy_net::{Ipv4Cidr, Runner, Stack, StackResources, StaticConfigV4};
use embassy_time::{Duration, Timer};
use esp_hal::{peripherals::WIFI, rng::Rng};
use esp_radio::wifi::{self, ControllerConfig, Interface, Password, Ssid, WifiController, ap::{self, AccessPointConfig}};

use edge_dhcp::{
    io::{self, DEFAULT_SERVER_PORT},
    server::{Server, ServerOptions},
};
use edge_nal::UdpBind;
use edge_nal_embassy::{Udp, UdpBuffers};

const AP_SSID: &'static str = "RGB-NEXT-LIFE";
const AP_PASSWORD: &'static str = "rgb!nextlife";
const GATEAWAY_ADDR: Ipv4Addr = Ipv4Addr::new(192, 168, 67, 1);

macro_rules! mk_static {
    ($t:ty, $val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        STATIC_CELL.init($val)
    }};
}

pub async fn start_ap(wifi: WIFI<'static>, spawner: &Spawner) {
    let ap_config = wifi::Config::AccessPoint(
        AccessPointConfig::default()
            .with_ssid(AP_SSID.try_into().unwrap())
            .with_authentication(wifi::AuthenticationMethodConfig::Wpa2Personal(AP_PASSWORD.try_into().unwrap()))
        );
    let device = Interface::access_point();
    let controller = ControllerConfig::default().with_initial_config(ap_config);

    let config = embassy_net::Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(GATEAWAY_ADDR, 24),
        gateway: Some(GATEAWAY_ADDR),
        dns_servers: Default::default()
    });

    let rng = Rng::new();
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;
    
    let controller = WifiController::new(wifi, controller).unwrap();
    let (stack, runner) = embassy_net::new(device, config, mk_static!(StackResources::<3>, StackResources::<3>::new()), seed);
    spawner.spawn(connection(controller).unwrap());
    spawner.spawn(net_task(runner).unwrap());
    spawner.spawn(run_dhcp(stack).unwrap());
    
    stack.wait_config_up().await;
    stack.config_v4().inspect(|config| info!("config ipv4: {:?}", config));

    loop {}
}

#[embassy_executor::task]
async fn run_dhcp(stack: Stack<'static>) {

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
            &mut Server::<_, 64>::new_with_et(GATEAWAY_ADDR),
            &ServerOptions::new(GATEAWAY_ADDR, Some(&mut gw_buf)),
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
    info!("start connection task");
    loop {
        let ev = controller
            .wait_for_access_point_connected_event_async()
            .await;
        match ev {
            Ok(esp_radio::wifi::ap::EventInfo::Connected(info)) => {
                info!("Station connected: {:?}", info);
            }
            Ok(esp_radio::wifi::ap::EventInfo::Disconnected(info)) => {
                info!("Station disconnected: {:?}", info);
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