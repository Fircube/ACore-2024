use core::arch::{asm, global_asm};
use riscv::register::mtvec::TrapMode;
use riscv::register::{scause, sip, stval, stvec};

use crate::{println, syscall::syscall};
use crate::config::{TRAMPOLINE, TRAP_CONTEXT};
use crate::task::processor::{current_trap_cx, current_user_satp};
use crate::task::{exit_and_run_next, suspend_and_run_next};


pub mod context;
global_asm!(include_str!("trap.s"));

#[no_mangle]
pub fn trap_handler() {
    let trap = scause::read().cause();
    let scause = scause::read();
    let stval = stval::read();
    match trap {
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
            scause::Exception::LoadFault
            | scause::Exception::LoadPageFault
            | scause::Exception::StoreFault
            | scause::Exception::StorePageFault
            | scause::Exception::InstructionFault
            | scause::Exception::InstructionPageFault => {
                println!(
                    "[kernel] {:?} in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.",
                    scause.cause(),
                    stval,
                    current_trap_cx().sepc,
                );
                // page fault exit code
                exit_and_run_next(-2);
            }
            scause::Exception::IllegalInstruction => {
                println!("[kernel] IllegalInstruction in application, kernel killed it.");
                // illegal instruction exit code
                exit_and_run_next(-3);
            }
            _ => {
                panic!(
                    "Unsupported exception {:?}, stval = {:#x}!",
                    scause.cause(),
                    stval
                );
            }
        },
        scause::Trap::Interrupt(intp) => match intp {
            scause::Interrupt::SupervisorSoft => {
                let sip = sip::read().bits();
                unsafe {
                    asm! {"csrw sip, {sip}", sip = in(reg) sip ^ 2};
                }
                suspend_and_run_next();
            }
            _ => {
                panic!(
                    "Unsupported interrupt {:?}, stval = {:#x}!",
                    scause.cause(),
                    stval
                );
            }
        },
    }
    trap_return();
}

fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE, TrapMode::Direct);
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