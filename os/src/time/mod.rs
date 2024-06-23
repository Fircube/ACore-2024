use core::arch::global_asm;
use crate::config::TIMER_INTERVAL;
use riscv::register::{mhartid, mie, mscratch, mstatus, mtvec, time};

global_asm!(include_str!("interrupt.s"));

pub fn set_time(hartid: usize, time: usize) {
    unsafe {
        let mtimecmp = (0x02004000 + 8 * hartid) as *mut usize;
        *mtimecmp = time;
    }
}

pub fn get_time() -> usize {
    unsafe {
        let mtime = 0x0200bff8 as *const usize;
        *mtime
    }
}

#[link_section = ".bss.stack"]
static mut TIMER_SCRATCH: [[usize; 5]; 8] = [[0; 5]; 8];

pub unsafe fn init_timer() {
    let hartid = mhartid::read();

    set_time(hartid, get_time() + TIMER_INTERVAL);

    let mscratch = &mut TIMER_SCRATCH[hartid];
    mscratch[3] = 0x02004000 + 8 * hartid;
    mscratch[4] = TIMER_INTERVAL;

    mscratch::write(mscratch as *mut usize as usize);

    extern "C" {
        fn _timer_int_handle();
    }
    mtvec::write(_timer_int_handle as usize, mtvec::TrapMode::Direct);

    mstatus::set_mie();

    mie::set_mtimer(); // MTIP
}