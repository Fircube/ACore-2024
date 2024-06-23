use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
use crate::println;
use crate::sync::up::UPSafeCell;
use crate::task::processor::current_trap_cx;
use crate::task::task::TaskControlBlock;

pub struct TaskManager {
    ready_tasks_queue: VecDeque<Arc<TaskControlBlock>>,
}

// A simple FIFO scheduler.
impl TaskManager {
    pub fn new() -> Self {
        Self {
            ready_tasks_queue: VecDeque::new(),
        }
    }
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_tasks_queue.push_back(task);
    }
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.ready_tasks_queue.pop_front()
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

pub fn add_task(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.exclusive_access().add(task);
}
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.exclusive_access().fetch()
}
