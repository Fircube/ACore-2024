use crate::mm::page_table::translated_byte_buffer;
use crate::{print, println};
use crate::io::console::{Stdin, Stdout};
use crate::io::stdin;
use crate::task::processor::current_user_satp;
use crate::task::suspend_and_run_next;

const FD_STDIN: usize = 0;
const FD_STDOUT: usize = 1;

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDIN => {
            let mut buffers = translated_byte_buffer(current_user_satp(), buf, len);
            buffers.iter_mut().for_each(|b| (**b)[0] = Stdin.getchar());
            len as isize
            // assert_eq!(len, 1, "Only support len = 1 in sys_read!");
            // let mut c: usize;
            // loop {
            //     c = stdin().getchar() as usize;
            //     if c == 0 {
            //         suspend_and_run_next();
            //         continue;
            //     } else {
            //         break;
            //     }
            // }
            // let ch = c as u8;
            // unsafe {
            //     buffers[0].as_mut_ptr().write_volatile(ch);
            // }
            // 1
        }
        _ => {
            panic!("[syscall] Unsupported fd in sys_read!");
        }
    }
}


pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let buffers = translated_byte_buffer(current_user_satp(), buf, len);
            for buffer in buffers {
                print!("{}", core::str::from_utf8(buffer).unwrap());
            }
            len as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}

