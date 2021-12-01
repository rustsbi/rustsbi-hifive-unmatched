#![no_std]
#![no_main]
#![feature(naked_functions, asm, asm_const, asm_sym)]
#![feature(generator_trait)]
#![feature(default_alloc_error_handler)]
#![feature(ptr_metadata)]

extern crate alloc;

mod runtime;
mod peripheral;
mod early_trap;
mod execute;
mod hart_csr_utils;
// #[allow(unused)] // use this in the future
// mod device_tree;

use core::panic::PanicInfo;
use rustsbi::println;

#[panic_handler]
fn on_panic(info: &PanicInfo) -> ! {
    let hart_id = riscv::register::mhartid::read();
    eprintln!("[rustsbi-panic] hart {} {}", hart_id, info); // [rustsbi-panic] hart 0 panicked at xxx
    loop {}
}

fn rust_main(hart_id: usize, opaque: usize) -> ! {
    let clint = peripheral::Clint::new(0x2000000 as *mut u8);
    if hart_id == 0 {
        init_bss();
        let uart = unsafe { peripheral::Uart::preloaded_uart0() };
        init_stdout(uart);
        early_trap::init(hart_id);
        init_heap(); // 必须先加载堆内存，才能使用rustsbi框架
        init_stdio(uart);
        init_clint(clint);
        println!("[rustsbi] RustSBI version {}", rustsbi::VERSION);
        println!("{}", rustsbi::LOGO);
        println!(
            "[rustsbi] Implementation: RustSBI-HiFive-Unleashed Version {}",
            env!("CARGO_PKG_VERSION")
        );
        hart_csr_utils::print_hart_csrs();
        println!("[rustsbi] enter supervisor 0x80200000, opaque register {:#x}", opaque);
        for target_hart_id in 0..=4 {
            if target_hart_id != 0 {
                clint.send_soft(target_hart_id);
            }
        }
    } else { // 不是初始化核，先暂停
        pause(clint);
    }
    delegate_interrupt_exception();
    runtime::init();
    execute::execute_supervisor(0x80200000, hart_id, opaque);
}

fn init_bss() {
    extern "C" {
        static mut ebss: u32;
        static mut sbss: u32;
        static mut edata: u32;
        static mut sdata: u32;
        static sidata: u32;
    }
    unsafe {
        r0::zero_bss(&mut sbss, &mut ebss);
        r0::init_data(&mut sdata, &mut edata, &sidata);
    } 
}

fn init_stdio(uart: peripheral::Uart) {
    use rustsbi::legacy_stdio::init_legacy_stdio_embedded_hal;
    init_legacy_stdio_embedded_hal(uart);
}

fn init_clint(clint: peripheral::Clint) {
    rustsbi::init_ipi(clint);
    rustsbi::init_timer(clint);
}

fn delegate_interrupt_exception() {
    use riscv::register::{medeleg, mideleg, mie};
    unsafe {
        mideleg::set_sext();
        mideleg::set_stimer();
        mideleg::set_ssoft();
        mideleg::set_uext();
        mideleg::set_utimer();
        mideleg::set_usoft();
        medeleg::set_instruction_misaligned();
        medeleg::set_breakpoint();
        medeleg::set_user_env_call();
        medeleg::set_instruction_page_fault();
        medeleg::set_load_page_fault();
        medeleg::set_store_page_fault();
        medeleg::set_instruction_fault();
        medeleg::set_load_fault();
        medeleg::set_store_fault();
        mie::set_mext();
        // 不打开mie::set_mtimer
        mie::set_msoft();
    }
}

pub fn pause(clint: peripheral::Clint) {
    use riscv::asm::wfi;
    use riscv::register::{mhartid, mie, mip};
    unsafe {
        let hartid = mhartid::read();
        clint.clear_soft(hartid); // Clear IPI
        mip::clear_msoft(); // clear machine software interrupt flag
        let prev_msoft = mie::read().msoft();
        mie::set_msoft(); // Start listening for software interrupts
        loop {
            wfi();
            if mip::read().msoft() {
                break;
            }
        }
        if !prev_msoft {
            mie::clear_msoft(); // Stop listening for software interrupts
        }
        clint.clear_soft(hartid); // Clear IPI
    }
}

const SBI_HEAP_SIZE: usize = 64 * 1024; // 64KiB
#[link_section = ".bss.uninit"]
static mut HEAP_SPACE: [u8; SBI_HEAP_SIZE] = [0; SBI_HEAP_SIZE];

use buddy_system_allocator::LockedHeap;

use crate::peripheral::uart::init_stdout;
#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::<32>::empty();

#[inline] fn init_heap() {
    unsafe {
        HEAP_ALLOCATOR.lock().init(HEAP_SPACE.as_ptr() as usize, SBI_HEAP_SIZE);
    }
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
    // 1. clear all registers
    "li x1, 0
    li x2, 0
    li x3, 0
    li x4, 0
    li x5, 0
    li x6, 0
    li x7, 0
    li x8, 0
    li x9, 0",
    // no x10 and x11: x10 is a0 and x11 is a1, they are passed to 
    // main function as arguments
    "li x12, 0
    li x13, 0
    li x14, 0
    li x15, 0
    li x16, 0
    li x17, 0
    li x18, 0
    li x19, 0
    li x20, 0
    li x21, 0
    li x22, 0
    li x23, 0
    li x24, 0
    li x25, 0
    li x26, 0
    li x27, 0
    li x28, 0
    li x29, 0
    li x30, 0
    li x31, 0",
    // 2. set sp
    // sp = bootstack + (hart_id + 1) * HART_STACK_SIZE
    "
    la      sp, {stack}
    li      t0, {per_hart_stack_size}
    csrr    t1, mhartid
    addi    t2, t1, 1
1:  add     sp, sp, t0
    addi    t2, t2, -1
    bnez    t2, 1b
    ",
    // 3. jump to main function (absolute address)
    "j      {rust_main}",
    per_hart_stack_size = const PER_HART_STACK_SIZE,
    stack = sym SBI_STACK,
    rust_main = sym rust_main,
    options(noreturn))
}
