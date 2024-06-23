use crate::task::{exit_and_run_next, suspend_and_run_next};
use crate::task::manager::add_task;
use alloc::sync::Arc;
use crate::mm::page_table::{translated_refmut, translated_str};
use crate::println;
use crate::task::loader::get_app_data_by_name;
use crate::task::processor::{curr_task, current_user_satp};

pub fn sys_exit(exit_code: i32) -> ! {
    exit_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_and_run_next();
    0
}

pub fn sys_getpid() -> isize {
    curr_task().unwrap().pid.0 as isize
}

pub fn sys_fork() -> isize {
    let curr_task = curr_task().unwrap();
    let new_task = curr_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.regs[10] = 0;
    // add new task to scheduler
    add_task(new_task);
    println!("[syscall] fork a pid {} process", new_pid);
    new_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    let token = current_user_satp();
    let path = translated_str(token, path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        println!("[syscall] exec");
        let task = curr_task().unwrap();
        task.exec(data);
        0
    } else {
        println!("[syscall] fail to exec {}", path);
        -1
    }
}

pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let task = curr_task().unwrap();
    // find a child process

    // ---- access current TCB exclusively
    let mut inner = task.inner_exclusive_access();
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
        // ---- release current PCB
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        // ++++ temporarily access child PCB lock exclusively
        p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        // ++++ release child PCB
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after removing from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily access child TCB exclusively
        let exit_code = child.inner_exclusive_access().exit_code;
        // ++++ release child PCB
        *translated_refmut(inner.usr_mem.satp(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB lock automatically
}
