#![no_main]
#![no_std]

use usr_lib::{exec, fork};

#[macro_use]
extern crate usr_lib;

#[no_mangle]
fn main() {
    if fork() == 0 {
        exec("hello_world\0");
    } else {
        println!("!");
    }
}
