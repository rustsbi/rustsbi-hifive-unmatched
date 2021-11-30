#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(asm, asm_const, asm_sym)]
#![feature(generator_trait)]
#![feature(default_alloc_error_handler)]
#![no_std]
#![no_main]

mod runtime;
mod peripheral;

use core::panic::PanicInfo;

#[panic_handler]
fn on_panic(panic_info: &PanicInfo) -> ! {
    if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
        println!("panic occurred: {}", s);
    } else {
        println!("panic occurred");
    }
    println!("panic occurred: {}", panic_info);
    loop {}
}

fn rust_main(hartid: usize, opaque: usize) -> ! {
    if hartid == 0 {
        init_bss();
    }
    runtime::init();
    if hartid == 0 {
        let uart = unsafe { peripheral::Uart::prev_bootloading_step() };
        init_stdout(uart);
        println!("rustsbi: hello world (1)!");
        init_heap(); // 必须先加载堆内存，才能使用rustsbi框架
        init_stdio();
        println!("rustsbi: hello world!");
        println!("rustsbi: hello world! {:x} {:x}", hartid, opaque);
    }
    todo!()
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

fn init_stdio() {
    use rustsbi::legacy_stdio::init_legacy_stdio_embedded_hal;
    let uart = unsafe { peripheral::Uart::prev_bootloading_step() };
    init_legacy_stdio_embedded_hal(uart);
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
