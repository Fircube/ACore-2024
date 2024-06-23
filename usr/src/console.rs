use alloc::string::String;
use core::fmt::{self, Write};
use crate::read;
use crate::syscall::sys_write;

const STDIN: usize = 0;
const STDOUT: usize = 1;

const BS: char = 8 as char;
const DL: char = 127 as char;

pub struct Stdin;

pub struct Stdout;

impl Stdin {
    pub fn getchar() -> char {
        let mut c = [0u8; 1];
        read(STDIN, &mut c);
        c[0] as char
    }

    pub fn getline() -> String {
        let mut str = String::new();
        loop {
            let c = Self::getchar();
            if c == '\n' || c == '\r' {
                break;
            }
            str.push(c);
        }
        str
    }

    pub fn getshell() -> String {
        let mut str = String::new();
        let mut c = Self::getchar();
        while c != '\n' && c != '\r' {
            if c == DL || c == BS {
                if !str.is_empty() {
                    str.pop();
                }
            } else {
                str.push(c);
            }
            c = Self::getchar();
        }
        str
    }
}

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        sys_write(STDOUT, s.as_bytes());
        Ok(())
    }
}


pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}