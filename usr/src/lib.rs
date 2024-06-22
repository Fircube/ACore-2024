#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]

#[macro_use]
pub mod console;
mod lang_items;
mod syscall;
mod config;

use syscall::*;

use heap::heap_allocator::*;
use crate::config::USER_HEAP_SIZE;

static mut USER_HEAP: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

#[global_allocator]
static HEAP: LockedHeap = unsafe { LockedHeap::new() };

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    unsafe{
        HEAP.allocator.inner.borrow_mut().add_to_heap(USER_HEAP.as_ptr() as usize, USER_HEAP_SIZE);
    }
    exit(main());
    panic!("unreachable after sys_exit!");
}

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!");
}
pub fn read(fd: usize, buf: &mut [u8]) -> isize { sys_read(fd, buf) }
pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}
pub fn exit(exit_code: i32) -> isize {
    sys_exit(exit_code)
}
pub fn yield_() -> isize { sys_yield() }