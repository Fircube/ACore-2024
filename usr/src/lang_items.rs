use core::panic::PanicInfo;
use crate::println;

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "[usr]Panicked at {}:{}, {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        println!("[usr]Panicked: {}", info.message().unwrap());
    }
    loop {}
}