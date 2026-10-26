#![no_std]
#![no_main]
#![feature(macro_metavar_expr)]

use core::hint::black_box;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_backtrace as _;

use crate::log::Serial;
use crate::storage::{Auth, Storage};

#[macro_use]
mod log;
mod errors;
mod storage;
mod bytes;

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 66320);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    let (mut wifi_controller, interfaces) =
        esp_radio::wifi::new(peripherals.WIFI, Default::default())
            .expect("Failed to initialize Wi-Fi controller");

    Serial::init(peripherals.USB_DEVICE, &spawner).await;

    let read_buffer = &mut [0u8; 128];
    let write_buffer = &mut [0u8; 128];
    
    let mut storage = Storage::new(peripherals.FLASH, read_buffer, write_buffer);

    loop {
        Timer::after_millis(200).await;
        
        info!("Trying to update..");
        storage.update(Auth { ssid: b"test network", password: b"123" }).await.unwrap();
        info!("Done, now reading...");
        storage.read_slice(0x3F0000).await.unwrap();
        info!("Done");
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}