#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn on_panic(_pi: &PanicInfo) -> ! {
    loop {}
}
