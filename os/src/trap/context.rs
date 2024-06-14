use riscv::register::sstatus::{self, Sstatus, SPP};

#[repr(C)]
pub struct TrapContext {
    pub regs: [usize; 32],
    pub sstatus: Sstatus,
    pub sepc: usize,
    pub kernel_satp: usize,
    pub kernel_sp: usize,
    pub trap_handler: usize,
}

// impl TrapContext {
//     pub fn set_sp(&mut self, sp: usize) {
//         self.x[2] = sp;
//     }
//     pub fn app_init_context(entry: usize, sp: usize) -> Self {
//         let mut sstatus = sstatus::read();
//         sstatus.set_spp(SPP::User);
//         let mut cx = Self {
//             regs: [0; 32],
//             sstatus,
//             sepc: entry,
//         };
//         cx.set_sp(sp);
//         cx
//     }
// }