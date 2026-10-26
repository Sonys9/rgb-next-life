use esp_hal::system::software_reset;
use esp_println::println;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("\x1b[31m==============================\x1b[0m");
    println!("\x1b[31m🚨 Panic!\x1b[0m");
    println!("❓ Reason: {:?}", info.message());
    if let Some(location) = info.location() {
        println!(
            "📍 Location: in \x1b[33m{}\x1b[0m at line \x1b[33m{}\x1b[0m, column \x1b[33m{}\x1b[0m",
            location.file(),
            location.line(),
            location.column()
        );
    } else {
        println!("📍 Location: \x1b[33mUnknown\x1b[0m");
    };
    println!("\x1b[31m==============================\x1b[0m");
    software_reset()
}
