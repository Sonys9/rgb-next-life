#![no_std]
#![no_main]
#![feature(macro_metavar_expr)]

use core::hint::black_box;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::rng::Rng;
use esp_hal::system::software_reset;
use esp_hal::timer::timg::TimerGroup;
use esp_alloc::HEAP;

use crate::log::Serial;
use crate::storage::{Auth, Storage};

#[macro_use]
mod log;
mod errors;
mod storage;
mod bytes;
mod wifi;

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[unsafe(link_section = ".dram2_uninit")] size: 66320);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let rng = Rng::new();
    // let sw_interrupt =
    //     esp_hal::interrupt::software::SoftwareInterrupt::new(peripherals.FROM_CPU_INTR0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    // let (mut wifi_controller, interfaces) =
    //     esp_radio::wifi::new(peripherals.WIFI, Default::default())
    //         .expect("Failed to initialize Wi-Fi controller");

    Serial::init(peripherals.USB_DEVICE, &spawner).await;

    let read_buffer = &mut [0u8; 128];
    let write_buffer = &mut [0u8; 128];
    let mut storage = Storage::new(peripherals.FLASH, read_buffer, write_buffer);
    
    

    loop {}
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    esp_println::println!("Panic! Reason: {:?}", info);
    software_reset()
}