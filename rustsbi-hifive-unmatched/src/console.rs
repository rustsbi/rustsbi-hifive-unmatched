use core::fmt;
use crate::util::AmoMutex;

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    static STDIO_LOCK: AmoMutex<()> = AmoMutex::new(());
    let line_lock = STDIO_LOCK.lock();
    crate::peripheral::uart::_print(args);
    drop(line_lock);
}

macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::peripheral::uart::_print(core::format_args!($($arg)*));
    });
}

macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::peripheral::uart::_print(core::format_args!(core::concat!($fmt, "\r\n") $(, $($arg)+)?));
    }
}

pub(crate) use {print, println};
