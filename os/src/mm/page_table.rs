use alloc::vec;
use alloc::vec::Vec;
use core::arch::asm;
use bitflags::*;
use core::cmp::PartialEq;
use crate::mm::frame_allocator::*;
use crate::mm::memory_set::KERNEL_SPACE;
use super::address::*;

bitflags! {
    pub struct PTEFlags: u8 {
        const V = 1 << 0; // Valid
        const R = 1 << 1; // Read
        const W = 1 << 2; // Write
        const X = 1 << 3; // Execute
        const U = 1 << 4; // User's access
        const G = 1 << 5; // Unknown
        const A = 1 << 6; // Whether accessed since last refresh
        const D = 1 << 7; // Whether written since last refresh
    }
}

// 自动实现 Copy 和 Clone 特性， 这个类型以值语义赋值/传参的时候不会发生所有权转移。
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize,
}

impl PartialEq for PTEFlags {
    fn eq(&self, other: &Self) -> bool {
        self.bits() == other.bits()
    }
}

// |64    54|53  28|27  19|18  10|9 8|7|6|5|4|3|2|1|0|
// |Reserved|PPN[2]|PPN[1]|PPN[0]|RSW|D|A|G|U|X|W|R|V|
impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PageTableEntry {
            bits: ppn.0 << 10 | flags.bits() as usize,
        }
    }
    pub fn empty() -> Self {
        PageTableEntry {
            bits: 0,
        }
    }
    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & ((1usize << 44) - 1)).into()
    }
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }
    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
    pub fn readable(&self) -> bool {
        (self.flags() & PTEFlags::R) != PTEFlags::empty()
    }
    pub fn writable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }
    pub fn executable(&self) -> bool {
        (self.flags() & PTEFlags::X) != PTEFlags::empty()
    }
}

pub struct PageTable {
    level0_ppn: PhysPageNum,
    frames: Vec<FrameTracker>,
}

impl PageTable {
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();
        PageTable {
            level0_ppn: frame.ppn,
            frames: vec![frame],
        }
    }
    fn create_pte(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idx = vpn.indexes();
        let mut ppn = self.level0_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for i in 0..3 {
            let pte = &mut ppn.get_pte_array()[idx[i]];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                let frame = frame_alloc().unwrap();
                // set valid
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.push(frame);
            }
            ppn = pte.ppn();
        }
        result
    }
    fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idx = vpn.indexes();
        let mut ppn = self.level0_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for i in 0..3 {
            let pte = &mut ppn.get_pte_array()[idx[i]];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                return None;
            }
            ppn = pte.ppn();
        }
        result
    }
    pub fn insert(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags){
        let pte = self.create_pte(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn {:?} is invalid before mapping", vpn);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }
    pub fn erase(&mut self, vpn: VirtPageNum){
        let pte = self.find_pte(vpn).unwrap();
        assert!(pte.is_valid(), "vpn {:?} is invalid before unmapping", vpn);
        *pte = PageTableEntry::empty();
    }
    pub fn query_pt(satp: usize) -> Self {
        Self {
            level0_ppn: PhysPageNum::from(satp & ((1usize << 44) - 1)),
            frames: Vec::new(),
        }
    }
    pub fn query_pte(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn)
            .map(|pte| {pte.clone()})
    }
    #[allow(unused)]
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.create_pte(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }
    #[allow(unused)]
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(pte.is_valid(), "vpn {:?} is invalid before unmapping", vpn);
        *pte = PageTableEntry::empty();
    }

    //
    pub fn from_token(satp: usize) -> Self {
        Self {
            level0_ppn: PhysPageNum::from(satp & ((1usize << 44) - 1)),
            frames: Vec::new(),
        }
    }
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn)
            .map(|pte| {pte.clone()})
    }
    pub fn token(&self) -> usize {
        8usize << 60 | self.level0_ppn.0
    }
    pub fn to_satp(&self) -> usize {
        8usize << 60 | self.level0_ppn.0
    }
}

pub fn activate_page_table() {
    let kernel_space = KERNEL_SPACE.exclusive_access();
    let page_table = kernel_space.page_table();
    let satp = page_table.to_satp();
    riscv::register::satp::write(satp);
    unsafe {
        asm!("sfence.vma");
    }
}