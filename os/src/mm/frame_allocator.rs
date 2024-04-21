use super::address::*;
use alloc::vec::Vec;
use crate::config::*;
use crate::sync::up::UPSafeCell;
use lazy_static::lazy_static;

trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
}

pub struct StackFrameAllocator {
    //空闲内存的起始物理页号
    begin: usize,
    //空闲内存的结束物理页号
    end: usize,
    recycled: Vec<usize>,
}

impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            begin: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }
    fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else {
            if self.begin == self.end {
                None
            } else {
                self.begin += 1;
                Some((self.begin - 1).into())
            }
        }
    }
    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.0;
        // validity check
        if ppn >= self.begin || self.recycled
            .iter()
            .find(|&v| { *v == ppn })
            .is_some() {
            panic!("Frame ppn={:#x} has not been allocated!", ppn);
        }
        // recycle
        self.recycled.push(ppn);
    }
}

impl StackFrameAllocator {
    pub fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
        self.begin = l.0;
        self.end = r.0;
    }
}


type FrameAllocatorImpl = StackFrameAllocator;
lazy_static! {
    pub static ref FRAME_ALLOCATOR: UPSafeCell<FrameAllocatorImpl> = unsafe {
        UPSafeCell::new(FrameAllocatorImpl::new())
    };
}

pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }
    FRAME_ALLOCATOR
        .exclusive_access()
        .init(PhysAddr::from(ekernel as usize).ceil(), PhysAddr::from(MEMORY_END).floor());
}

pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR
        .exclusive_access()
        .alloc()
        .map(|ppn| FrameTracker::new(ppn))
}

fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR
        .exclusive_access()
        .dealloc(ppn);
}

pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

impl FrameTracker {
    pub fn new(ppn: PhysPageNum) -> Self {
        let bytes_array = ppn.get_bytes_array();
        for i in bytes_array {
            *i = 0;
        }
        Self { ppn }
    }
}


impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}

// #[allow(unused)]
// pub fn frame_allocator_test() {
//     let mut v: Vec<FrameTracker> = Vec::new();
//     for i in 0..5 {
//         let frame = frame_alloc().unwrap();
//         println!("{:?}", frame);
//         v.push(frame);
//     }
//     v.clear();
//     for i in 0..5 {
//         let frame = frame_alloc().unwrap();
//         println!("{:?}", frame);
//         v.push(frame);
//     }
//     drop(v);
//     println!("frame_allocator_test passed!");
// }