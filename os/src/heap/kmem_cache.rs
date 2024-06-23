use core::ops::{Deref, DerefMut};

use heap::linked_list::LinkedList;

use crate::config::{PAGE_SIZE, PAGE_SIZE_BITS};
use crate::heap::{KERNEL_HEAP, SLABNODES};

#[derive(Clone, Copy)]
pub struct SlabNode {
    pub phy_addr: usize,

    pub rank: usize,
    pub free_pieces: LinkedList,
    pub uses: usize,

    pub prev: Option<SlabPtr>,
    pub next: Option<SlabPtr>,
}

impl SlabNode {
    pub const fn empty() -> Self {
        Self {
            phy_addr: 0,
            rank: 0,
            free_pieces: LinkedList::new(),
            uses: 0,
            prev: None,
            next: None,
        }
    }

    pub fn new(&mut self) {
        let size = 1 << self.rank;
        for i in (0..PAGE_SIZE / size).rev() {
            unsafe { self.free_pieces.push((self.phy_addr + i * size) as *mut usize) };
        }
    }

    pub fn take_piece(&mut self) -> Option<*mut usize> {
        self.uses += 1;
        self.free_pieces.pop()
    }

    pub fn release_piece(&mut self, ptr: *mut usize) {
        self.uses -= 1;
        unsafe {
            self.free_pieces.push(ptr);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SlabPtr {
    ptr: *mut SlabNode,
}

impl Deref for SlabPtr {
    type Target = SlabNode;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl DerefMut for SlabPtr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}

pub fn map_slab(ptr: usize) -> Option<SlabPtr> {
    unsafe {
        let start = KERNEL_HEAP.as_ptr() as usize;
        let idx = (ptr - start) >> PAGE_SIZE_BITS;
        let ptr = SlabPtr{
            ptr: &mut SLABNODES[idx]
        };
        Some(ptr)
    }
}

