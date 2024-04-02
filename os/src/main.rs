#![no_std] // use core
#![no_main] // no initialization of std
#![feature(panic_info_message)]

mod lang_items;
mod io;

// 在 Rust 代码中直接插入汇编指令
use core::arch::global_asm;
use core::mem::MaybeUninit;
global_asm!(include_str!("entry.asm"));
const SERIAL_PORT_BASE_ADDRESS: usize = 0x1000_0000;

// avoid confusing names
#[no_mangle]
pub fn rust_main(){
    let mut serial_port = unsafe { io::uart::MMIOPort::new(SERIAL_PORT_BASE_ADDRESS) };
    serial_port.init();
    unsafe {
        io::uart::UART = MaybeUninit::new(io::uart::MMIOPort::new(SERIAL_PORT_BASE_ADDRESS));
        io::uart::UART.assume_init_mut().init();
    }
    clear_bss();
    println!("Hello, world!");

    // panic!("Shutdown machine!");
}

// initialize .bss section
fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| {
        unsafe { (a as *mut u8).write_volatile(0) }
    });
}


