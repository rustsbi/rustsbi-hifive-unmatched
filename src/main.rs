#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(asm, asm_const, asm_sym)]
#![feature(generator_trait)]
#![feature(default_alloc_error_handler)]
#![no_std]
#![no_main]

mod runtime;

use core::panic::PanicInfo;

#[panic_handler]
fn on_panic(_pi: &PanicInfo) -> ! {
    loop {}
}

fn rust_main(hartid: usize, opaque: usize) -> ! {
    runtime::init();
    if hartid == 0 { 
        use fu740_hal::{pac, serial::Serial, prelude::*};
        use rustsbi::legacy_stdio::init_legacy_stdio_embedded_hal_fuse;
        let p = pac::Peripherals::take().unwrap();
        let clocks = p.PRCI.setup()
            .coreclk(1500.mhz()) // 1.5GHz todo: 根据device tree去配置
            .pclk(120.mhz())
            .freeze();
        let serial = Serial::new(p.UART0, 115200.bps(), &clocks);
        let (tx, rx) = serial.split();
        init_legacy_stdio_embedded_hal_fuse(tx, rx);
        // todo: u-boot spl是否已经设置了串口？
        rustsbi::println!("rustsbi: hello world!");
    }
    todo!()
}

const PER_HART_STACK_SIZE: usize = 4 * 4096; // 16KiB
const SBI_STACK_SIZE: usize = 5 * PER_HART_STACK_SIZE; // 5 harts
#[link_section = ".bss.uninit"]
static mut SBI_STACK: [u8; SBI_STACK_SIZE] = [0; SBI_STACK_SIZE];

#[naked]
#[link_section = ".text.entry"]
#[export_name = "_start"]
unsafe extern "C" fn entry() -> ! {
    asm!(
    // 1. set sp
    // sp = bootstack + (hartid + 1) * HART_STACK_SIZE
    "
    la      sp, {stack}
    li      t0, {per_hart_stack_size}
    csrr    t1, mhartid
    addi    t2, t1, 1
1:  add     sp, sp, t0
    addi    t2, t2, -1
    bnez    t2, 1b
    ",
    // 2. jump to rust_main (absolute address)
    "j      {rust_main}",
    per_hart_stack_size = const PER_HART_STACK_SIZE,
    stack = sym SBI_STACK,
    rust_main = sym rust_main,
    options(noreturn))
}
