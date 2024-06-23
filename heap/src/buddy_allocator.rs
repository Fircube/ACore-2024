use super::linked_list::LinkedList;
use core::cmp::{max, min};
use core::mem::size_of;
use alloc::alloc::Layout;


pub struct BuddyAllocator {
    free_list: [LinkedList; 32],

    user: usize,
    allocated: usize,
    total: usize,
}

impl BuddyAllocator {
    // Create a new heap
    pub unsafe fn new(start: usize, end: usize) -> Self {
        let mut new_allocator = Self::empty();
        new_allocator.add_to_heap(start, end);
        new_allocator
    }

    // Add a range of memory [start, start+size) to the heap
    pub unsafe fn init(&mut self, start: usize, size: usize) {
        self.add_to_heap(start, start + size);
    }

    // Create an empty heap
    pub const fn empty() -> Self {
        Self {
            free_list: [LinkedList::new(); 32],
            user: 0,
            allocated: 0,
            total: 0,
        }
    }

    // Add a range of memory [start, end) to the heap
    pub unsafe fn add_to_heap(&mut self, mut start: usize, mut end: usize) {
        let unit = size_of::<usize>();
        start = (start + unit - 1) & (!unit + 1);
        end &= !unit + 1;
        self.total += end - start;

        while start < end {
            let level = (end - start).trailing_zeros() as usize;
            self.free_list[level].push(start as *mut usize);
            start += 1 << level;
        }
    }

    // Alloc a range of memory from the heap satisfying `layout` requirements
    pub fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let size = max(
            layout.size().next_power_of_two(),
            max(layout.align(), size_of::<usize>()),
        );
        let level = size.trailing_zeros() as usize;
        for i in level..self.free_list.len() {
            if !self.free_list[i].is_empty() {
                self.split(level, i);
                let result = self.free_list[level]
                    .pop()
                    .expect("[buddy_allocator] current block should have free space now");

                self.user += layout.size();
                self.allocated += size;
                return result as *mut u8;
            }
        }
        panic!("[buddy allocator] out of memory when alloc size {}", size);
    }

    // Dealloc a range of memory from the heap
    pub fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let size = max(
            layout.size().next_power_of_two(),
            max(layout.align(), size_of::<usize>()),
        );
        let level = size.trailing_zeros() as usize;
        self.merge(level, ptr);
    }

    fn split(&mut self, start: usize, end: usize) {
        for i in (start..end).rev() {
            let ptr = self.free_list[i + 1]
                .pop()
                .expect("[buddy_allocator] current block should have free space now");

            // split into 2 parts
            unsafe {
                self.free_list[i].push((ptr as usize + (1 << i)) as *mut usize);
                self.free_list[i].push(ptr);
            }
        }
    }

    fn merge(&mut self, start: usize, ptr: *mut u8) {
        let mut curr = ptr as usize;
        for i in start..self.free_list.len() {
            let buddy = curr ^ (1 << i);
            let target = self.free_list[i]
                .iter_mut()
                .find(|node| node.ptr() as usize == buddy);

            if let Some(node) = target {
                node.pop();
                curr = min(curr, buddy);
            } else {
                unsafe {
                    self.free_list[i].push(curr as *mut usize);
                }
                break;
            }
        }
    }
}

