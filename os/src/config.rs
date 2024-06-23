pub const SERIAL_PORT_BASE_ADDRESS: usize = 0x1000_0000;
pub const KERNEL_STACK_SIZE: usize = 0x2000;
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;
pub const KERNEL_HEAP_UNIT: usize = PAGE_SIZE;
pub const SERIAL_PORT_MAP_SIZE: usize = 0x6;

pub const USER_STACK_SIZE: usize = 0x2000;

pub const TIMER_INTERVAL: usize = 100_0000;

pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 0xc; // 12

pub const SLAB_NODE_NUM: usize = KERNEL_HEAP_SIZE / PAGE_SIZE;

pub const PA_WIDTH_SV39: usize = 56;
pub const VA_WIDTH_SV39: usize = 39;
pub const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_SIZE_BITS;
pub const VPN_WIDTH_SV39: usize = VA_WIDTH_SV39 - PAGE_SIZE_BITS;

pub const MEMORY_END: usize = 0x88000000;

pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;