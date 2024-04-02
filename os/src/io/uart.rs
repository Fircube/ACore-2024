use core::sync::atomic::{AtomicPtr, Ordering};
// use volatile::Volatile;
use core::mem::MaybeUninit;
use bitflags::bitflags;

macro_rules! wait_for {
    ($cond:expr) => {
        while !$cond {
            core::hint::spin_loop();
        }
    };
}

#[derive(Debug)]
pub struct MMIOPort {
    // receiver buffer & transmitter holding
    rbr_thr: AtomicPtr<u8>,
    // interrupt enable
    ier: AtomicPtr<u8>,
    // interrupt identification & FIFO control
    iir_fcr: AtomicPtr<u8>,
    // line control
    lcr: AtomicPtr<u8>,
    // modem control
    mcr: AtomicPtr<u8>,
    // line status
    lsr: AtomicPtr<u8>,
    // modem status
    msr: AtomicPtr<u8>,
    // scratch
    scr: AtomicPtr<u8>,
}

bitflags! {
    // Line status flags
    struct LineStsFlags: u8 {
        const INPUT_FULL = 1;
        const OUTPUT_EMPTY = 1 << 5;
    }
}

impl MMIOPort {
    // Creates a new UART interface on the given memory mapped address.
    // The caller must ensure that the given base address really points to a serial port device.
    pub unsafe fn new(base: usize) -> Self {
        let base_pointer = base as *mut u8;
        Self {
            rbr_thr: AtomicPtr::new(base_pointer),
            ier: AtomicPtr::new(base_pointer.add(1)),
            iir_fcr: AtomicPtr::new(base_pointer.add(2)),
            lcr: AtomicPtr::new(base_pointer.add(3)),
            mcr: AtomicPtr::new(base_pointer.add(4)),
            lsr: AtomicPtr::new(base_pointer.add(5)),
            msr: AtomicPtr::new(base_pointer.add(6)),
            scr: AtomicPtr::new(base_pointer.add(7)),
        }
    }

    // Initialize the serial port.
    pub fn init(&mut self) {
        let rbr_thr = self.rbr_thr.load(Ordering::Relaxed);
        let ier = self.ier.load(Ordering::Relaxed);
        let iir_fcr = self.iir_fcr.load(Ordering::Relaxed);
        let lcr = self.lcr.load(Ordering::Relaxed);
        let mcr = self.mcr.load(Ordering::Relaxed);
        let _lsr = self.lsr.load(Ordering::Relaxed);
        let _msr = self.msr.load(Ordering::Relaxed);
        let _scr = self.scr.load(Ordering::Relaxed);
        unsafe {
            // disable interrupts.
            ier.write(0x00);

            // special mode to set baud rate.
            lcr.write(0x80);

            // LSB for baud rate of 38.4K.
            rbr_thr.write(0x03);

            // MSB for baud rate of 38.4K.
            ier.write(0x00);

            // leave set-baud mode,
            // and set word length to 8 bits, no parity.
            lcr.write(0x03);

            // reset and enable FIFOs.
            iir_fcr.write(0xC7);

            // Mark data terminal ready, signal request to send
            // and enable auxilliary output #2 (used as interrupt line for CPU)
            mcr.write(0x0B);

            // enable transmit and receive interrupts.
            ier.write(0x01);
        }
    }

    fn line_status(&mut self) -> LineStsFlags {
        unsafe { LineStsFlags::from_bits_truncate(*self.lsr.load(Ordering::Relaxed)) }
    }

    // Sends a byte on the serial port.
    pub fn send(&mut self, data: u8) {
        let rbr_thr = self.rbr_thr.load(Ordering::Relaxed);
        unsafe{
            match data {
                8 | 0x7F => {
                    wait_for!(self.line_status().contains(LineStsFlags::OUTPUT_EMPTY));
                    rbr_thr.write(8);
                    wait_for!(self.line_status().contains(LineStsFlags::OUTPUT_EMPTY));
                    rbr_thr.write(b' ');
                    wait_for!(self.line_status().contains(LineStsFlags::OUTPUT_EMPTY));
                    rbr_thr.write(8)
                }
                _ => {
                    wait_for!(self.line_status().contains(LineStsFlags::OUTPUT_EMPTY));
                    rbr_thr.write(data);
                }
            }
        }
    }

    // Receives a byte on the serial port.
    pub fn receive(&mut self) -> u8 {
        let rbr_thr = self.rbr_thr.load(Ordering::Relaxed);
        unsafe {
            wait_for!(self.line_status().contains(LineStsFlags::INPUT_FULL));
            rbr_thr.read()
        }
    }
}

pub static mut UART:MaybeUninit<MMIOPort> = MaybeUninit::uninit();