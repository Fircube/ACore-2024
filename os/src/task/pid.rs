use crate::sync::up::UPSafeCell;
use alloc::vec::Vec;
use lazy_static::*;

pub struct PidAllocator {
    current: usize,
    recycled: Vec<usize>,
}

impl PidAllocator {
    pub fn new() -> Self {
        PidAllocator {
            current: 0,
            recycled: Vec::new(),
        }
    }
    pub fn alloc(&mut self) -> PidTracker {
        if let Some(pid) = self.recycled.pop() {
            PidTracker(pid)
        } else {
            self.current += 1;
            PidTracker(self.current - 1)
        }
    }
    pub fn dealloc(&mut self, pid: usize) {
        assert!(pid < self.current);
        assert!(
            !self.recycled.iter().any(|ppid| *ppid == pid),
            "pid {} has been deallocated!",
            pid
        );
        self.recycled.push(pid);
    }
}

lazy_static! {
    pub static ref PID_ALLOCATOR: UPSafeCell<PidAllocator> = UPSafeCell::new(PidAllocator::new()) ;
}

pub struct PidTracker(pub usize);

impl Drop for PidTracker {
    fn drop(&mut self) {
        //println!("drop pid {}", self.0);
        pid_dealloc(self);
    }
}

pub fn pid_alloc() -> PidTracker {
    PID_ALLOCATOR.exclusive_access().alloc()
}

pub fn pid_dealloc(pid: &PidTracker) {
    PID_ALLOCATOR.exclusive_access().dealloc(pid.0);
}