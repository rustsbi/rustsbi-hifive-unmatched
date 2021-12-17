use crate::peripheral::Uart;
use crate::util::AmoMutex;
use core::fmt;
use embedded_hal::serial::Write;

static STDOUT: AmoMutex<Option<Uart>> = AmoMutex::new(None);

pub fn init_stdout(uart: Uart) {
    let mut lock = STDOUT.lock();
    *lock = Some(uart);
    drop(lock);
}

impl fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.as_bytes() {
            nb::block!(self.write(*byte)).ok(); // todo: 为了极致性能，未来添加水标设置
        }
        nb::block!(self.flush()).ok(); // todo: 这行会影响输出
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    let lock = STDOUT.lock();
    if let Some(mut stdout) = *lock {
        stdout.write_fmt(args).unwrap();
    }
    drop(lock);
}

#[doc(hidden)]
pub fn _eprint(args: fmt::Arguments) {
    use fmt::Write;
    let lock = STDOUT.lock();
    if let Some(mut stdout) = *lock {
        stdout.write_fmt(args).unwrap();
    }
    drop(lock);
}

#[allow(unused)]
macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::console::_print(core::format_args!($($arg)*))
    });
}

#[allow(unused)]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::_print(core::format_args!(core::concat!($fmt, "\r\n") $(, $($arg)+)?))
    }
}

#[allow(unused)]
macro_rules! eprintln {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::_eprint(core::format_args!(core::concat!($fmt, "\r\n") $(, $($arg)+)?))
    }
}

#[allow(unused)]
pub(crate) use {eprintln, print, println};
