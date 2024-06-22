use core::arch::{asm, global_asm};
use riscv::register::mtvec::TrapMode;
use riscv::register::{scause, stvec};

use crate::{syscall::syscall};
use crate::config::{TRAMPOLINE, TRAP_CONTEXT};
use crate::task::processor::{current_trap_cx, current_user_satp};


pub mod context;
global_asm!(include_str!("trap.s"));

#[no_mangle]
pub fn trap_handler() {
    let trap = scause::read().cause();
    match trap {
        scause::Trap::Interrupt(intp) => match intp {
            scause::Interrupt::SupervisorSoft => todo!(),
            scause::Interrupt::SupervisorTimer => todo!(),
            scause::Interrupt::SupervisorExternal => todo!(),
            scause::Interrupt::UserSoft => todo!(),
            scause::Interrupt::UserTimer => todo!(),
            scause::Interrupt::UserExternal => todo!(),
            scause::Interrupt::Unknown => todo!(),
            scause::Interrupt::VirtualSupervisorSoft => todo!(),
            scause::Interrupt::VirtualSupervisorTimer => todo!(),
            scause::Interrupt::VirtualSupervisorExternal => todo!(),
        },
        scause::Trap::Exception(excp) => match excp {
            scause::Exception::UserEnvCall => {
                // jump to next instruction anyway
                let mut cx = current_trap_cx();
                cx.sepc += 4;
                // get system call return value
                let result = syscall(cx.regs[17], [cx.regs[10], cx.regs[11], cx.regs[12]]);
                // cx is changed during sys_exec, so we have to call it again
                cx = current_trap_cx();
                cx.regs[10] = result as usize;
            }
            scause::Exception::InstructionMisaligned => todo!(),
            scause::Exception::InstructionFault => todo!(),
            scause::Exception::IllegalInstruction => todo!(),
            scause::Exception::Breakpoint => todo!(),
            scause::Exception::LoadFault => todo!(),
            scause::Exception::StoreMisaligned => todo!(),
            scause::Exception::StoreFault => todo!(),
            scause::Exception::InstructionPageFault => todo!(),
            scause::Exception::LoadPageFault => todo!(),
            scause::Exception::StorePageFault => todo!(),
            scause::Exception::Unknown => todo!(),
            scause::Exception::VirtualSupervisorEnvCall => todo!(),
            scause::Exception::InstructionGuestPageFault => todo!(),
            scause::Exception::LoadGuestPageFault => todo!(),
            scause::Exception::VirtualInstruction => todo!(),
            scause::Exception::StoreGuestPageFault => todo!(),
        },
    }
    trap_return();
}

fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE as usize, TrapMode::Direct);
    }
}
#[no_mangle]
/// set the new addr of __restore asm function in TRAMPOLINE page,
/// set the reg a0 = trap_cx_ptr, reg a1 = phy addr of usr page table,
/// finally, jump to new addr of __restore asm function
pub fn trap_return() -> ! {
    set_user_trap_entry();
    let trap_cx_ptr = TRAP_CONTEXT;
    let user_satp = current_user_satp();
    extern "C" {
        fn __usertrap();
        fn __userret();
    }
    let restore_va = __userret as usize - __usertrap as usize + TRAMPOLINE;
    unsafe {
        asm!(
        "fence.i",
        "jr {restore_va}",
        restore_va = in(reg) restore_va,
        in("a0") trap_cx_ptr,
        in("a1") user_satp,
        options(noreturn)
        )
    }
}