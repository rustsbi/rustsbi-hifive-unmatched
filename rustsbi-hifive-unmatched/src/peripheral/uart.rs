use core::convert::Infallible;
use embedded_hal::serial::{Read, Write};
use fu740_hal::pac;

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
    pub unsafe fn preloaded_uart0() -> Self {
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
                (&*self.inner)
                    .txdata
                    .write_with_zero(|w| w.data().bits(byte));
            }
            Ok(())
        }
    }

    #[inline]
    fn flush(&mut self) -> nb::Result<(), Infallible> {
        Ok(()) // todo: 观察水标
               // if unsafe { &*self.inner }.ip.read().txwm().bit_is_set() {
               //     // FIFO count is below the receive watermark (1)
               //     Ok(())
               // } else {
               //     Err(nb::Error::WouldBlock)
               // }
    }
}
