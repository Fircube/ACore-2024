use core::alloc::{GlobalAlloc, Layout};
use crate::config::PAGE_SIZE_BITS;
use crate::heap::HEAP;
use crate::heap::kmem_cache::{map_slab, SlabPtr};
use crate::sync::up::UPSafeCell;

pub struct SlubAllocator {
    kmem_caches: [KmemPtr; PAGE_SIZE_BITS],
}

pub struct UPSlabHeap {
    allocator: UPSafeCell<SlubAllocator>,
}

impl SlubAllocator {
    pub const fn empty() -> Self {
        Self { kmem_caches: [KmemPtr::empty(); PAGE_SIZE_BITS] }
    }

    pub fn init(&mut self) {
        for (i, cache) in self.kmem_caches.iter_mut().enumerate() {
            cache.rank = i;
        }
    }

    pub fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let size = layout.size().next_power_of_two();
        let rank = size.trailing_zeros() as usize;
        self.kmem_caches[rank].alloc() as *mut u8
    }

    pub fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let size = layout.size().next_power_of_two();
        let rank = size.trailing_zeros() as usize;
        self.kmem_caches[rank].dealloc(ptr as usize);
    }
}

impl UPSlabHeap {
    pub const fn empty() -> Self {
        Self {
            allocator: UPSafeCell::new(SlubAllocator::empty()),
        }
    }

    pub fn init(&self) {
        self.allocator.exclusive_access().init();
    }
}

unsafe impl GlobalAlloc for UPSlabHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.allocator.exclusive_access().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.allocator.exclusive_access().dealloc(ptr, layout);
    }
}


#[derive(Clone, Copy)]
pub struct KmemPtr {
    pub rank: usize,
    curr: Option<SlabPtr>,
    next: Option<SlabPtr>,
}

impl KmemPtr {
    pub const fn empty() -> KmemPtr {
        KmemPtr {
            rank: 0,
            curr: None,
            next: None,
        }
    }

    pub fn alloc(&mut self) -> usize {
        // locate the first not full slab
        if self.curr.is_none() {
            if let Some(mut Slab) = self.next {
                if let Some(mut next) = Slab.next {
                    next.prev = None;
                    self.next = Some(next)
                } else {
                    self.next = None
                };
                Slab.next = None;
                self.curr = Some(Slab)
            } else {
                let ptr = HEAP
                    .buddy_allocator
                    .exclusive_access()
                    .alloc(Layout::array::<u8>(1 << self.rank).unwrap());
                let mut slab = map_slab(ptr as usize).unwrap();
                slab.rank = self.rank;
                slab.new();
                self.curr = Some(slab)
            }
        }

        // take the first free piece of the slab
        let mut slab = self.curr.unwrap();
        let ptr = slab.take_piece().unwrap() as usize;
        if slab.free_pieces.is_empty() {
            self.curr = None;
        }
        ptr
    }

    pub fn dealloc(&mut self, ptr: usize) {
        let mut slab = map_slab(ptr).unwrap();

        // the slab is full
        if slab.free_pieces.is_empty() {
            if let Some(mut next) = self.next {
                next.prev = Some(slab);
                slab.next = Some(next);
                slab.prev = None;
            }
            self.next = Some(slab);
        }

        // free a piece
        slab.release_piece(ptr as *mut usize);

        if slab.uses == 0 {
            if Some(slab) != self.curr {
                if let Some(mut prev) = slab.prev {
                    // not beginning
                    prev.next = slab.next;
                    if let Some(mut next) = slab.next {
                        next.prev = Some(prev);
                    }
                } else {
                    // beginning
                    if let Some(mut next) = slab.next {
                        next.prev = None;
                    }
                    self.next = None;
                }
                slab.prev = None;
                slab.next = None;
                unsafe {
                    HEAP.buddy_allocator.exclusive_access().dealloc(
                        slab.phy_addr as *mut u8,
                        Layout::array::<u8>(1 << self.rank).unwrap(),
                    );
                }
            }
        }
    }
}
