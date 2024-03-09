#![no_std]
#![no_main]

mod lang_items;

// 在 Rust 代码中直接插入汇编指令
use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));

fn main() {
    // println!("Hello, world!");
}
