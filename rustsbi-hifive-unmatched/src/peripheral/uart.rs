use fu740_hal::pac;
use embedded_hal::serial::{Read, Write};
use core::convert::Infallible;

// UART that is initialized by prior steps of bootloading
#[derive(Clone, Copy)]
pub struct Uart {
    inner: *const pac::uart0::RegisterBlock,
}

// UART外设是可以跨上下文共享的
unsafe impl Send for Uart {}
unsafe impl Sync for Uart {}

impl Uart {
    #[inline]
    pub unsafe fn prev_bootloading_step() -> Self {
        let inner = pac::UART0::ptr();
        Self { inner }
    }
}

// Ref: fu740-hal

impl Read<u8> for Uart {
    type Error = Infallible;

    #[inline]
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        let rxdata = unsafe { &*self.inner }.rxdata.read();

        if rxdata.empty().bit_is_set() {
            Err(nb::Error::WouldBlock)
        } else {
            Ok(rxdata.data().bits() as u8)
        }
    }
}

impl Write<u8> for Uart {
    type Error = Infallible;

    #[inline]
    fn write(&mut self, byte: u8) -> nb::Result<(), Infallible> {
        let txdata = unsafe { &*self.inner }.txdata.read();

        if txdata.full().bit_is_set() {
            Err(nb::Error::WouldBlock)
        } else {
            unsafe {
                (&*self.inner).txdata.write_with_zero(|w| w.data().bits(byte));
            }
            Ok(())
        }
    }

    #[inline]
    fn flush(&mut self) -> nb::Result<(), Infallible> {
        if unsafe { &*self.inner }.ip.read().txwm().bit_is_set() {
            // FIFO count is below the receive watermark (1)
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

use core::fmt;

static mut STDOUT: Option<Uart> = None;

pub fn init_stdout(uart: Uart) {
    unsafe { STDOUT = Some(uart) };
}

impl fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut uart = unsafe { STDOUT.unwrap() };
        for byte in s.as_bytes() {
            nb::block!(uart.write(*byte)).ok(); // todo: 为了极致性能，未来添加水标设置
        }
        nb::block!(uart.flush()).ok();
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    unsafe { STDOUT.unwrap() }.write_fmt(args).unwrap();
}

/// Prints to the legacy debug console.
///
/// This is only supported when there exists legacy extension; 
/// otherwise platform caller should use an early kernel input/output device
/// declared in platform specific hardware.
#[macro_export(local_inner_macros)]
macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::peripheral::uart::_print(core::format_args!($($arg)*));
    });
}

/// Prints to the legacy debug console, with a newline.
///
/// This is only supported when there exists legacy extension; 
/// otherwise platform caller should use an early kernel input/output device
/// declared in platform specific hardware.
#[macro_export(local_inner_macros)]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::peripheral::uart::_print(core::format_args!(core::concat!($fmt, "\r\n") $(, $($arg)+)?));
    }
}
