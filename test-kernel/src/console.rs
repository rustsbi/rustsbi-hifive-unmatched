use crate::sbi;
use crate::util::AmoMutex;
use core::fmt::{self, Write};

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut buffer = [0u8; 4];
        for c in s.chars() {
            for code_point in c.encode_utf8(&mut buffer).as_bytes().iter() {
                sbi::console_putchar(*code_point as usize);
            }
        }
        Ok(())
    }
}

#[allow(unused)]
pub fn print(args: fmt::Arguments) {
    STDOUT.lock().write_fmt(args).unwrap();
}

static STDOUT: AmoMutex<Stdout> = AmoMutex::new(Stdout);

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
