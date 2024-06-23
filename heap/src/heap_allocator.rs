use core::alloc::GlobalAlloc;
use crate::buddy_allocator::BuddyAllocator;
use crate::temp_mut::TempMut;


pub struct LockedBuddyHeap {
    pub allocator: TempMut<BuddyAllocator>,
}

impl LockedBuddyHeap {
    pub const fn new(unit: usize) -> Self {
        Self {
            allocator: TempMut::new(BuddyAllocator::empty(unit)),
        }
    }

    pub unsafe fn add_to_heap(&self, start: usize, end: usize) {
        self.allocator.inner.borrow_mut().add_to_heap(start, end);
    }
}

unsafe impl GlobalAlloc for LockedBuddyHeap {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.allocator.inner.borrow_mut().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        self.allocator.inner.borrow_mut().dealloc(ptr, layout);
    }
}

