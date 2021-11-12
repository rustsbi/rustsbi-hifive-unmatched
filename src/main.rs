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

use riscv::register::mhartid;

fn rust_main() -> ! {
    runtime::init();
    let hartid = mhartid::read();
    if hartid == 0 { 
        use rustsbi::legacy_stdio::init_legacy_stdio_embedded_hal;
        let p = fu740_hal::pac::Peripherals::take().unwrap();
        // todo: u-boot spl是否已经设置了串口？
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
