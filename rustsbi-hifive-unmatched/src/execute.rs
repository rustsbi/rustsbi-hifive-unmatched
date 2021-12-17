use crate::feature;
use crate::runtime::{MachineTrap, Runtime, SupervisorContext};
use core::{
    ops::{Generator, GeneratorState},
    pin::Pin,
};
use riscv::register::scause::{Exception, Trap};
use riscv::register::{mie, mip};

pub fn execute_supervisor(supervisor_mepc: usize, hart_id: usize, opaque: usize) {
    let mut rt = Runtime::new_sbi_supervisor(supervisor_mepc, hart_id, opaque);
    loop {
        match Pin::new(&mut rt).resume(()) {
            GeneratorState::Yielded(MachineTrap::SbiCall()) => {
                let ctx = rt.context_mut();
                let param = [ctx.a0, ctx.a1, ctx.a2, ctx.a3, ctx.a4, ctx.a5];
                let ans = rustsbi::ecall(ctx.a7, ctx.a6, param);
                ctx.a0 = ans.error;
                ctx.a1 = ans.value;
                ctx.mepc = ctx.mepc.wrapping_add(4);
            }
            GeneratorState::Yielded(MachineTrap::IllegalInstruction()) => {
                let ctx = rt.context_mut();
                // FIXME: get_vaddr_u32这个过程可能出错。
                let ins = unsafe { get_vaddr_u32(ctx.mepc) } as usize;
                if !emulate_illegal_instruction(ctx, ins) {
                    unsafe {
                        if feature::should_transfer_trap(ctx) {
                            feature::do_transfer_trap(
                                ctx,
                                Trap::Exception(Exception::IllegalInstruction),
                            )
                        } else {
                            fail_illegal_instruction(ctx, ins)
                        }
                    }
                }
            }
            GeneratorState::Yielded(MachineTrap::MachineTimer()) => unsafe {
                mip::set_stimer();
                mie::clear_mtimer();
            },
            GeneratorState::Yielded(MachineTrap::MachineSoft()) => todo!(),
            GeneratorState::Complete(()) => break,
        }
    }
}

#[inline]
unsafe fn get_vaddr_u32(vaddr: usize) -> u32 {
    let mut ans: u32;
    core::arch::asm!("
        li      {tmp}, (1 << 17)
        csrrs   {tmp}, mstatus, {tmp}
        lwu     {ans}, 0({vaddr})
        csrw    mstatus, {tmp}
        ",
        tmp = out(reg) _,
        vaddr = in(reg) vaddr,
        ans = lateout(reg) ans
    );
    ans
}

fn emulate_illegal_instruction(ctx: &mut SupervisorContext, ins: usize) -> bool {
    if feature::emulate_rdtime(ctx, ins) {
        return true;
    }
    false
}

// 真·非法指令异常，是M层出现的
fn fail_illegal_instruction(ctx: &mut SupervisorContext, ins: usize) -> ! {
    #[cfg(target_pointer_width = "64")]
    panic!("invalid instruction from machine level, mepc: {:016x?}, instruction: {:016x?}, context: {:016x?}", ctx.mepc, ins, ctx);
    #[cfg(target_pointer_width = "32")]
    panic!("invalid instruction from machine level, mepc: {:08x?}, instruction: {:08x?}, context: {:08x?}", ctx.mepc, ins, ctx);
}
