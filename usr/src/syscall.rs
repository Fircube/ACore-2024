use core::arch::asm;

const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
// const SYSCALL_GET_TIME: usize = 169;
// const SYSCALL_GETPID: usize = 172;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_WAITPID: usize = 260;

fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        asm!(
        "ecall",
        inlateout("x10") args[0] => ret,
        in("x11") args[1],
        in("x12") args[2],
        in("x17") id
        );
    }
    ret
}

/// 功能：从文件中读取一段内容到缓冲区。
///
/// 参数：fd 是待读取文件的文件描述符，切片 buffer 则给出缓冲区。
///
/// 返回值：如果出现了错误则返回 -1，否则返回实际读到的字节数。
///
/// syscall ID：63
pub fn sys_read(fd: usize, buffer: &mut [u8]) -> isize {
    syscall(SYSCALL_READ, [fd, buffer.as_mut_ptr() as usize, buffer.len()])
}

/// 功能：将内存中缓冲区中的数据写入文件。
///
/// 参数：`fd` 表示待写入文件的文件描述符；
///      `buf` 表示内存中缓冲区的起始地址；
///      `len` 表示内存中缓冲区的长度。
///
/// 返回值：返回成功写入的长度。
///
/// syscall ID：64
pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}

/// 功能：退出应用程序并将返回值告知系统。
///
/// 参数：`exit_code` 表示应用程序的返回值。
///
/// 返回值：该系统调用不应该返回。
///
/// syscall ID：93
pub fn sys_exit(exit_code: i32) -> ! {
    syscall(SYSCALL_EXIT, [exit_code as usize, 0, 0]);
    panic!("[usr] return from sys_exit()");
}

/// 功能：应用主动交出 CPU 所有权并切换到其他应用。
///
/// 返回值：总是返回 0。
///
/// syscall ID：124
pub fn sys_yield() -> isize {
    syscall(SYSCALL_YIELD, [0, 0, 0])
}

// pub fn sys_get_time() -> isize {
//     syscall(SYSCALL_GET_TIME, [0, 0, 0])
// }

// pub fn sys_getpid() -> isize {
//     syscall(SYSCALL_GETPID, [0, 0, 0])
// }

pub fn sys_fork() -> isize {
    syscall(SYSCALL_FORK, [0, 0, 0])
}

pub fn sys_exec(path: &str) -> isize {
    syscall(SYSCALL_EXEC, [path.as_ptr() as usize, 0, 0])
}

pub fn sys_waitpid(pid: isize, exit_code: *mut i32) -> isize {
    syscall(SYSCALL_WAITPID, [pid as usize, exit_code as usize, 0])
}
