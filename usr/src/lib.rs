#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]

extern crate alloc;

#[macro_use]
pub mod console;
mod lang_items;
mod syscall;
mod config;

use syscall::*;

use heap::heap_allocator::*;
use crate::config::{USER_HEAP_SIZE, USER_HEAP_UNIT};

static mut USER_HEAP: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

#[global_allocator]
static HEAP: LockedBuddyHeap = unsafe { LockedBuddyHeap::new(USER_HEAP_UNIT) };

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    unsafe {
        let start = USER_HEAP.as_ptr() as usize;
        let end = start + USER_HEAP_SIZE;
        HEAP.add_to_heap(start, end);
    }
    exit(main());
    panic!("unreachable after sys_exit!");
}

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("[usr] Cannot find main!");
}

pub fn read(fd: usize, buf: &mut [u8]) -> isize { sys_read(fd, buf) }

pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}

pub fn exit(exit_code: i32) -> isize {
    sys_exit(exit_code)
}

pub fn yield_() -> isize { sys_yield() }

pub fn fork() -> isize {
    sys_fork()
}

pub fn exec(path: &str) -> isize {
    sys_exec(path)
}

pub fn wait(exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(-1, exit_code as *mut _) {
            -2 => {
                yield_();
            }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}

pub fn waitpid(pid: usize, exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(pid as isize, exit_code as *mut _) {
            -2 => {
                yield_();
            }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}