mod context;

// use crate::batch::run_next_app;
// use crate::syscall::syscall;
use core::arch::global_asm;
// use riscv::register::{
//     mtvec::TrapMode,
//     scause::{self, Exception, Trap},
//     stval, stvec,
// };
// pub use context::TrapContext;
// use crate::println;

global_asm!(include_str!("trap.s"));

// pub fn init() {
//     extern "C" {
//         fn __alltraps();
//     }
//     unsafe {
//         stvec::write(__alltraps as usize, TrapMode::Direct);
//     }
// }
//
// // 分发和处理
// #[no_mangle]
// pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
//     let scause = scause::read();
//     let stval = stval::read();
//     match scause.cause() {
//         Trap::Exception(Exception::UserEnvCall) => {
//             cx.sepc += 4; // next instruction
//             cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
//         }
//         Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
//             println!("[kernel] PageFault in application, kernel killed it.");
//             // run_next_app();
//         }
//         Trap::Exception(Exception::IllegalInstruction) => {
//             println!("[kernel] IllegalInstruction in application, kernel killed it.");
//             // run_next_app();
//         }
//         _ => {
//             panic!(
//                 "Unsupported trap {:?}, stval = {:#x}!",
//                 scause.cause(),
//                 stval
//             );
//         }
//     }
//     cx
// }
//
