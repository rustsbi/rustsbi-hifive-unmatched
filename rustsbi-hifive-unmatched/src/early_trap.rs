use riscv::register::{
    mstatus::Mstatus,
    mtvec::{self, TrapMode},
    mscratch,
    mcause,
    mtval
};

#[inline]
pub fn init(hartid: usize) {
    let stack_base = unsafe { &super::SBI_STACK } as *const _ as usize;
    let stack = stack_base + (hartid + 1) * super::PER_HART_STACK_SIZE;
    mscratch::write(stack);
    let mut addr = early_trap_fail as usize;
    if addr & 0x2 != 0 {
        addr += 0x2; // 中断入口地址必须对齐到4个字节
    }
    unsafe { mtvec::write(addr, TrapMode::Direct) };
}

extern "C" fn rust_fail(ctx: &SupervisorContext) -> ! {
    crate::eprintln!("rustsbi: early init stage fail, context: {:x?}, mcause: {:?}, mtval: {:x}", ctx, mcause::read(), mtval::read());
    loop {}
}

#[derive(Debug)]
#[repr(C)]
pub struct SupervisorContext {
    pub ra: usize, // 0
    pub sp: usize,
    pub gp: usize,
    pub tp: usize,
    pub t0: usize,
    pub t1: usize,
    pub t2: usize,
    pub s0: usize,
    pub s1: usize,
    pub a0: usize,
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub a4: usize,
    pub a5: usize,
    pub a6: usize,
    pub a7: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,
    pub t3: usize,
    pub t4: usize,
    pub t5: usize,
    pub t6: usize,            // 30
    pub mstatus: Mstatus,     // 31
    pub mepc: usize,          // 32
}

#[naked]
#[link_section = ".text"]
pub unsafe extern "C" fn early_trap_fail() -> ! {
    asm!( // sp:特权级栈,mscratch:特权级上下文
        ".p2align 2",
        "csrrw  sp, mscratch, sp", // 新mscratch:特权级栈, 新sp:特权级上下文
        "addi   sp, sp, -33*8",
        "sd     ra, 0*8(sp)
        sd      gp, 2*8(sp)
        sd      tp, 3*8(sp)
        sd      t0, 4*8(sp)
        sd      t1, 5*8(sp)
        sd      t2, 6*8(sp)
        sd      s0, 7*8(sp)
        sd      s1, 8*8(sp)
        sd      a0, 9*8(sp)
        sd      a1, 10*8(sp)
        sd      a2, 11*8(sp)
        sd      a3, 12*8(sp)
        sd      a4, 13*8(sp)
        sd      a5, 14*8(sp)
        sd      a6, 15*8(sp)
        sd      a7, 16*8(sp)
        sd      s2, 17*8(sp)
        sd      s3, 18*8(sp)
        sd      s4, 19*8(sp)
        sd      s5, 20*8(sp)
        sd      s6, 21*8(sp)
        sd      s7, 22*8(sp)
        sd      s8, 23*8(sp)
        sd      s9, 24*8(sp)
        sd     s10, 25*8(sp)
        sd     s11, 26*8(sp)
        sd      t3, 27*8(sp)
        sd      t4, 28*8(sp)
        sd      t5, 29*8(sp)
        sd      t6, 30*8(sp)",
        "csrr   t0, mstatus
        sd      t0, 31*8(sp)",
        "csrr   t1, mepc
        sd      t1, 32*8(sp)",
        "csrr   t2, mscratch
        sd      t2, 1*8(sp)",
        "mv     a0, sp",
        "j      {fail}",
        fail = sym rust_fail,
        options(noreturn)
    )
}
