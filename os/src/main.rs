#![no_std] // use core
#![no_main] // no initialization of std
#![feature(panic_info_message)]

extern crate alloc;

mod lang_items;
mod io;
mod heap;
mod mm;
mod config;

// 在 Rust 代码中直接插入汇编指令
use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));

use core::arch::asm;
use config::*;
use io::uart::UART;
use riscv::register::*;


// avoid confusing names
#[no_mangle]
pub fn rust_main() {
    clear_bss();
    UART.init();
    println!("Hello, world!");
    // panic!("Shutdown machine!");
}

unsafe fn rust_m2s_mode(){
    mstatus::set_mpp(mstatus::MPP::Supervisor);
    mepc::write(rust_main as usize);

    satp::write(0);// disable paging

    medeleg::write(0xffff);
    mideleg::write(0xffff);
    // sstatus::set_sie();
    sie::set_sext(); // SEIE
    sie::set_stimer(); // STIE
    sie::set_ssoft(); // SSIE

    pmpaddr0::write(0x3fffffffffffffusize);
    pmpcfg0::write(0xf);


    init_timer();

    asm!(
        "mret",
        options(noreturn),
    );
}

pub fn set_time(hartid: usize, time: usize) {
    unsafe {
        let mtimecmp = (0x02004000 + 8 * hartid) as *mut usize;
        *mtimecmp = time;
    }
}

pub fn get_time() -> usize {
    unsafe {
        let mtime = 0x0200bff8 as *const usize;
        *mtime
    }
}

pub unsafe fn init_timer() {
    let hartid = mhartid::read();

    set_time(hartid, get_time() + TIMER_INTERVAL);

    // mscratch::write(...);

    extern "C" {
        fn _timer_int_handle();
    }
    mtvec::write(_timer_int_handle as usize, mtvec::TrapMode::Direct);

    mstatus::set_mie();

    mie::set_mtimer(); // MTIP
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


