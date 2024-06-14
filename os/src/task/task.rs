use crate::mm::memory_set::MemorySet;

use super::pid::{pid_alloc, PidTracker};

pub struct Task {
    pid: PidTracker,
    memory_set: MemorySet,
}

impl Task {
    pub fn from_elf(elf_data: &[u8]) -> Self {
        Self {
            pid: pid_alloc(),
            memory_set: MemorySet::from_elf(elf_data).0,
        }
    }
}