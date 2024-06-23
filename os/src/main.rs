#![no_std] // use core
#![no_main] // no initialization of std
#![feature(panic_info_message, asm_const)]

// #[macro_use]
extern crate alloc;

// #[macro_use]
// extern crate bitflags;

mod config;
mod io;
mod lang_items;
mod mm;
mod sync;
mod syscall;
mod task;
mod time;
mod trap;

// 在 Rust 代码中直接插入汇编指令
use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.s"));

use config::*;
use core::arch::asm;
use mm::heap_allocator::HEAP_ALLOCATOR;
use io::uart::UART;
use riscv::register::*;
use mm::frame_allocator::init_frame_allocator;
use mm::memory_set::activate_page_table;
use crate::task::{INITPROC, loader};
use crate::task::processor::run_tasks;
use crate::time::init_timer;

// avoid confusing names
#[no_mangle]
pub fn rust_main() {
    init_trap();
    println!("[kernel] From m mode to s mode");
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

    println!("[kernel] init task");
    task::add_initproc();

    loader::list_apps();

    println!("[kernel] run tasks");
    run_tasks();
    println!("[kernel] tasks finished");
}

#[no_mangle]
unsafe fn rust_m2s_mode() {
    mstatus::set_mpp(mstatus::MPP::Supervisor);
    mepc::write(rust_main as usize);

    satp::write(0);// disable paging

    pmpaddr0::write(0x3fffffffffffffusize);
    pmpcfg0::write(0xf);

    asm!("csrr tp, mhartid");

    init_timer();

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

fn init_trap() {
    unsafe {
        sie::set_sext(); // SEIE
        sie::set_stimer(); // STIE
        sie::set_ssoft(); // SSIE
    }
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


