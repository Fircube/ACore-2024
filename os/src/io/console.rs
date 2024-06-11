use super::uart::UART;
use core::fmt::{self, Write};

pub struct Stdout;

impl Stdout {
    pub fn putchar(&self, c: char) {
        UART.send(c as u8);
    }

    pub fn print(&mut self, args: fmt::Arguments) {
        self.write_fmt(args).unwrap();
    }
}

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.putchar(c);
        }
        Ok(())
    }
}


#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::io::stdout().print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::io::stdout().print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}