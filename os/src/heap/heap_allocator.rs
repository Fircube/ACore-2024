use core::alloc::GlobalAlloc;

use heap::buddy_allocator::BuddyAllocator;

use crate::config::KERNEL_HEAP_UNIT;
use crate::sync::up::UPSafeCell;

use super::slub_allocator::SlubAllocator;

pub struct UPHeapAllocator {
    pub buddy_allocator: UPSafeCell<BuddyAllocator>,
    pub slub_allocator: UPSafeCell<SlubAllocator>,
}

impl UPHeapAllocator {
    pub const fn empty() -> Self {
        Self {
            buddy_allocator: UPSafeCell::new(BuddyAllocator::empty(KERNEL_HEAP_UNIT)),
            slub_allocator: UPSafeCell::new(SlubAllocator::empty()),
        }
    }

    pub unsafe fn init(&self, start: usize, end: usize) {
        self.buddy_allocator.exclusive_access().add_to_heap(start, end);
        self.slub_allocator.exclusive_access().init();
    }
}

unsafe impl GlobalAlloc for UPHeapAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        if layout.size() < 4096 {
            self.slub_allocator.exclusive_access().alloc(layout)
        } else {
            self.buddy_allocator.exclusive_access().alloc(layout)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        if layout.size() < 4096 {
            self.slub_allocator.exclusive_access().dealloc(ptr, layout)
        } else {
            self.buddy_allocator.exclusive_access().dealloc(ptr, layout)
        }
    }
}
