// A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
// all the tasks in the whole operating system.
//
// A single global instance of [`Processor`] called `PROCESSOR` monitors running
// task(s) for each core.
//
// A single global instance of [`PidAllocator`] called `PID_ALLOCATOR` allocates
// pid for user apps.
mod context;
mod pid;
mod task;
pub(crate) mod loader;
pub(crate) mod manager;
pub(crate) mod processor;
mod kernel_stack;
mod switch;

use alloc::sync::Arc;
use lazy_static::lazy_static;
pub use context::TaskContext;
use crate::println;
use crate::task::loader::get_app_data_by_name;
use crate::task::manager::add_task;
use crate::task::processor::{schedule, take_current_task};
use crate::task::task::{TaskControlBlock, TaskStatus};

// Suspend the current 'Running' task and run the next task in task list.
pub fn suspend_and_run_next() {
    // There must be an application running.
    let task = take_current_task().unwrap();

    // ---- access current TCB exclusively
    let mut task_inner = task.inner_exclusive_access();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    // Change status to Ready
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);
    // ---- release current PCB

    // push back to ready queue.
    add_task(task);
    // jump to scheduling cycle
    schedule(task_cx_ptr);
}


pub const IDLE_PID: usize = 0;

// Exit the current 'Running' task and run the next task in task list.
pub fn exit_and_run_next(exit_code: i32) {
    // take from Processor
    let task = take_current_task().unwrap();

    let pid = task.getpid();
    if pid == IDLE_PID {
        println!(
            "[kernel] Idle process exit with exit_code {} ...",
            exit_code
        );
        if exit_code != 0 {
            //crate::sbi::shutdown(255); //255 == -1 for err hint
            // shutdown(true)
        } else {
            //crate::sbi::shutdown(0); //0 for success hint
            // shutdown(false)
        }
    }

    // **** access current TCB exclusively
    let mut inner = task.inner_exclusive_access();
    // Change status to Zombie
    inner.task_status = TaskStatus::Zombie;
    // Record exit code
    inner.exit_code = exit_code;
    // do not move to its parent but under initproc
    // ++++++ access initproc TCB exclusively
    {
        let mut initproc_inner = INITPROC.inner_exclusive_access();
        for child in inner.children.iter() {
            child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }
    // ++++++ release parent PCB

    inner.children.clear();
    // deallocate user space
    inner.usr_mem.clear();
    drop(inner);
    // **** release current PCB
    // drop task manually to maintain rc correctly
    drop(task);
    // we do not have to save task context
    let mut _unused = TaskContext::init();
    schedule(&mut _unused as *mut _);
}

lazy_static! {
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new(TaskControlBlock::new(
        get_app_data_by_name("initproc").unwrap()
    ));
}

pub fn add_initproc() {
    add_task(INITPROC.clone());
}
