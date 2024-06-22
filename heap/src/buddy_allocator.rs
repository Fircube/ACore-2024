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
        // avoid unaligned access on some platforms
        let unit = size_of::<usize>();
        start = (start + unit - 1) & (!unit + 1);
        end &= !unit + 1;
        self.total += end - start;

        while start < end {
            let level = (end - start).trailing_zeros() as usize;
            self.free_list[level].push(start as *mut usize);
            start += 1 << level;
        }
        // while current_start < end {
        //     let lowbit = current_start & (!current_start + 1);
        //     let num = end - current_start;
        //     let size = min(lowbit, 1 << (usize::BITS as usize - num.leading_zeros() as usize - 1));
        //     total += size;
        //
        //     self.free_list[size.trailing_zeros() as usize].push(current_start as *mut usize);
        //     current_start += size;
        // }
        // assert_eq!(total, end - start, "Total is not equal to end - start!");
        // self.total += total;
    }



    // Alloc a range of memory from the heap satisfying `layout` requirements
    pub fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let size = max(
            layout.size().next_power_of_two(),
            max(layout.align(), size_of::<usize>()),
        );
        let class = size.trailing_zeros() as usize;
        for i in class..self.free_list.len() {
            // Find the first non-empty size class
            if !self.free_list[i].is_empty() {
                // Split buffers
                for j in (class + 1..i + 1).rev() {
                    if let Some(block) = self.free_list[j].pop() {
                        unsafe {
                            self.free_list[j - 1]
                                .push((block as usize + (1 << (j - 1))) as *mut usize);
                            self.free_list[j - 1].push(block);
                        }
                    } else {
                        panic!("buddy allocator: block split failed");
                    }
                }

                let result = core::ptr::NonNull::new(
                    self.free_list[class]
                        .pop()
                        .expect("current block should have free space now")
                        as *mut u8,
                );
                if let Some(result) = result {
                    self.user += layout.size();
                    self.allocated += size;
                    return result.as_ptr();
                } else {
                    panic!("buddy allocator: block split failed");
                }
            }
        }
        panic!("buddy allocator: out of memory");
    }

    // Dealloc a range of memory from the heap
    pub fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let size = max(
            layout.size().next_power_of_two(),
            max(layout.align(), size_of::<usize>()),
        );
        let class = size.trailing_zeros() as usize;

        unsafe {
            // Put back into free list
            self.free_list[class].push(ptr as *mut usize);

            // Merge free buddy lists
            let mut current_ptr = ptr as usize;
            let mut current_class = class;

            while current_class < self.free_list.len() - 1 {
                let buddy = current_ptr ^ (1 << current_class);
                let mut flag = false;
                for block in self.free_list[current_class].iter_mut() {
                    if block.value() == buddy {
                        block.pop();
                        flag = true;
                        break;
                    }
                }

                // Free buddy found
                if flag {
                    self.free_list[current_class].pop();
                    current_ptr = min(current_ptr, buddy);
                    current_class += 1;
                    self.free_list[current_class].push(current_ptr as *mut usize);
                } else {
                    break;
                }
            }
        }

        self.user -= layout.size();
        self.allocated -= size;
    }
}

