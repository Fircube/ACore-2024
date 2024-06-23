use core::alloc::GlobalAlloc;
use core::ops::Deref;
use heap::buddy_allocator::BuddyAllocator;
use crate::config::{KERNEL_HEAP_SIZE, KERNEL_HEAP_UNIT};
use crate::sync::up::UPSafeCell;

static mut KERNEL_HEAP: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

#[global_allocator]
pub static HEAP_ALLOCATOR: UPHeapAllocator = UPHeapAllocator::new();

pub struct UPHeapAllocator {
    heap: UPSafeCell<BuddyAllocator>,
}

impl UPHeapAllocator {
    pub const fn new() -> Self {
        Self {
            heap: UPSafeCell::new(BuddyAllocator::empty(KERNEL_HEAP_UNIT)),
        }
    }

    pub fn init(&self) {
        unsafe {
            let start = KERNEL_HEAP.as_ptr() as usize;
            self.heap.exclusive_access().init(start, KERNEL_HEAP_SIZE);
        }
    }
}

impl Deref for UPHeapAllocator {
    type Target = UPSafeCell<BuddyAllocator>;

    fn deref(&self) -> &Self::Target {
        &self.heap
    }
}

unsafe impl GlobalAlloc for UPHeapAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.heap.exclusive_access().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        self.heap.exclusive_access().dealloc(ptr, layout)
    }
}
