#![no_std]
#![no_main]
#![feature(macro_metavar_expr)]
#![feature(type_alias_impl_trait)]

use core::hint::black_box;
use core::net::Ipv4Addr;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_alloc::HEAP;
use esp_hal::clock::CpuClock;
use esp_hal::rng::Rng;
use esp_hal::system::software_reset;
use esp_hal::timer::timg::TimerGroup;

use crate::log::Serial;
use crate::storage::{Auth, Storage};
use crate::wifi::AP;

#[macro_use]
mod log;
mod bytes;
mod errors;
mod storage;
mod wifi;

extern crate alloc;

const AP_SSID: &'static str = "RGB-NEXT-LIFE";
const AP_PASSWORD: &'static str = "rgb!nextlife";
const GATEAWAY_ADDR: Ipv4Addr = Ipv4Addr::new(192, 168, 67, 1);

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

    let ap = AP::new(AP_SSID, AP_PASSWORD, GATEAWAY_ADDR, peripherals.WIFI);
    ap.start(&spawner).await;

    core::future::pending::<()>().await;
    loop {} // never happens
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    esp_println::println!("Panic! Reason: {:?}", info);
    software_reset()
}
