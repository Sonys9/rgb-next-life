#![no_std]
#![no_main]
#![feature(macro_metavar_expr)]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]

use core::hint::black_box;
use core::net::Ipv4Addr;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_alloc::HEAP;
use esp_hal::clock::CpuClock;
use esp_hal::peripherals::WIFI;
use esp_hal::rng::Rng;
use esp_hal::system::software_reset;
use esp_hal::timer::timg::TimerGroup;
use esp_radio::wifi;

use crate::log::Serial;
use crate::network::{Auth, Http};
use crate::storage::Storage;

#[macro_use]
mod log;
mod bytes;
mod errors;
mod network;
mod panic;
mod storage;

extern crate alloc;

const AP_SSID: &'static str = "RGB-NEXT-LIFE";
const AP_PASSWORD: &'static str = "rgb!nextlife";
const GATEAWAY_ADDR: Ipv4Addr = Ipv4Addr::new(192, 168, 67, 1);

static mut HTTP: Option<Http> = None;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[unsafe(link_section = ".dram2_uninit")] size: 66320);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    Serial::init(peripherals.USB_DEVICE, &spawner).await;

    let read_buffer = &mut [0u8; 128];
    let write_buffer = &mut [0u8; 128];
    let mut storage = Storage::new(peripherals.FLASH, read_buffer, write_buffer);

    let auth = match storage.read().await {
        Err(errors::Error::Parse) => get_auth(storage, unsafe { peripherals.WIFI.clone_unchecked() }, &spawner).await,
        Ok(auth) if auth.ssid.is_empty() && auth.password.is_empty() => get_auth(storage, unsafe { peripherals.WIFI.clone_unchecked() }, &spawner).await,
        Ok(auth) => auth,
        Err(e) => panic!("Failed to read the storage: {}", e)
    };
    let amc = auth.auth_method.to_amc(&auth.password).unwrap();
    info!("got auth data: {:?} ({:?})", auth, amc);
    // auth.auth_method.0 = auth.password;

    // let wifi = network::WiFi::new(AP_SSID, AP_PASSWORD, GATEAWAY_ADDR, peripherals.WIFI);
    

    core::future::pending::<()>().await;
    loop {} // never happens
}

#[allow(static_mut_refs)]
async fn get_auth<'a>(mut storage: Storage<'a>, wifi_interface: WIFI<'static>, spawner: &Spawner) -> Auth {
    let wifi = network::WiFi::new(AP_SSID, AP_PASSWORD, GATEAWAY_ADDR, wifi_interface);
    let stack = wifi.access_point(&spawner).await;
    let http = Http::new(stack);
    let http_ref: &'static Http = unsafe { 
        HTTP = Some(http);
        HTTP.as_ref().unwrap()
    };
    let auth = http_ref.start(&spawner).await;
    unsafe { HTTP = None };
    drop(wifi);
    storage.update(&auth).expect("Failed to update the storage");
    auth
}