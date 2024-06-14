#![no_std] // use core
#![no_main] // no initialization of std
#![feature(panic_info_message, asm_const)]
// #![feature(alloc_error_handler)]

// #[macro_use]
extern crate alloc;

// #[macro_use]
// extern crate bitflags;

mod config;
mod io;
mod lang_items;
mod mm;
mod sync;
// mod syscall;
mod task;
mod trap;

// 在 Rust 代码中直接插入汇编指令
use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));

use config::*;
use core::arch::asm;
use mm::heap_allocator::HEAP_ALLOCATOR;
use io::uart::UART;
use riscv::register::*;
use mm::frame_allocator::init_frame_allocator;
use mm::page_table::activate_page_table;


// avoid confusing names
#[no_mangle]
pub fn rust_main() {
    clear_bss();
    println!("[kernel] .bss cleared");
    UART.init();
    println!("[kernel] UART initialized");
    HEAP_ALLOCATOR.init();
    println!("[kernel] heap initialized");
    init_frame_allocator();
    println!("[kernel] frame allocator initialized");
    activate_page_table();
    println!("[kernel] page table activated");
    // panic!("Shutdown machine!");
}

#[no_mangle]
unsafe fn rust_m2s_mode() {
    mstatus::set_mpp(mstatus::MPP::Supervisor);
    mepc::write(rust_main as usize);

    satp::write(0);// disable paging

    sie::set_sext(); // SEIE
    sie::set_stimer(); // STIE
    sie::set_ssoft(); // SSIE

    pmpaddr0::write(0x3fffffffffffffusize);
    pmpcfg0::write(0xf);

    // init_timer();

    asm!(
        "li t0, {medeleg}",
        "li t1, {mideleg}",
        "csrw medeleg, t0",
        "csrw mideleg, t1",
        "mret",
        medeleg = const 0xffff,
        mideleg = const 0xffff,
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


