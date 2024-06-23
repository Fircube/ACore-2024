#![no_main]
#![no_std]

use usr_lib::fork;

#[macro_use]
extern crate usr_lib;

#[no_mangle]
fn main() {
    if fork() == 0 {
        println!("This is children!");
    } else {
        println!("This is parent!");
    }
}
