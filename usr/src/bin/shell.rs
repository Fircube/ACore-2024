#![no_std]
#![no_main]
#![allow(clippy::println_empty_string)]

extern crate alloc;

#[macro_use]
extern crate usr_lib;

use usr_lib::{exec, fork, waitpid};

#[no_mangle]
pub fn main() {
    println!("shell start!");
    loop {
        print!(">> ");
        let str = usr_lib::console::Stdin::getshell();
        let fork_pid = fork();
        if fork_pid == 0 {
            if exec(str.as_str()) == -1 {
                println!("[user] exec {} failed", str);
                return;
            }
        } else {
            let mut exit_code: i32 = 0;
            waitpid(fork_pid as usize, &mut exit_code);
            println!("[usr] process with pid {} exit with code {}", fork_pid, exit_code);
        }
    }
}


// let c = getchar();
// match c {
//     LF | CR => {
//         println!("");
//         if !line.is_empty() {
//             line.push('\0');
//             let pid = fork();
//             if pid == 0 {
//                 // child process
//                 if exec(line.as_str()) == -1 {
//                     println!("Error when executing!");
//                     return -4;
//                 }
//                 unreachable!();
//             } else {
//                 let mut exit_code: i32 = 0;
//                 let exit_pid = waitpid(pid as usize, &mut exit_code);
//                 assert_eq!(pid, exit_pid);
//                 println!("Shell: Process {} exited with code {}", pid, exit_code);
//             }
//             line.clear();
//         }
//         print!(">> ");
//     }
//     BS | DL => {
//         if !line.is_empty() {
//             print!("{}", BS as char);
//             print!(" ");
//             print!("{}", BS as char);
//             line.pop();
//         }
//     }
//     _ => {
//         print!("{}", c as char);
//         line.push(c as char);
//     }
// }
