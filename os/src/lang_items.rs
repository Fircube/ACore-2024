// use crate::sbi::shutdown;
use core::panic::PanicInfo;
use crate::println;

#[panic_handler] // 编译指导属性 与 panic! 宏配合使用
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "[kernel]Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        println!("[kernel]Panicked: {}", info.message().unwrap());
    }
    // shutdown(true)
    loop {}
}