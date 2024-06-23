mod heap_allocator;
mod kmem_cache;
mod slub_allocator;

use crate::config::{KERNEL_HEAP_SIZE, PAGE_SIZE,  SLAB_NODE_NUM};
use crate::heap::heap_allocator::UPHeapAllocator;
use crate::heap::kmem_cache::SlabNode;

#[link_section = ".data.heap"]
static mut KERNEL_HEAP: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];
static mut SLABNODES: [SlabNode; SLAB_NODE_NUM] = [SlabNode::empty(); SLAB_NODE_NUM];

#[global_allocator]
static HEAP: UPHeapAllocator = UPHeapAllocator::empty();

pub fn init_heap() {
    unsafe {
        let start = KERNEL_HEAP.as_ptr() as usize;
        let end = start + KERNEL_HEAP_SIZE;
        HEAP.init(start, end);
        for (i, slab) in SLABNODES.iter_mut().enumerate() {
            slab.phy_addr = start + i * PAGE_SIZE;
        }
    }
}


