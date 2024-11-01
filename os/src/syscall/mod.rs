//! Implementation of syscalls
//!
//! The single entry point to all system calls, [`syscall()`], is called
//! whenever userspace wishes to perform a system call using the `ecall`
//! instruction. In this case, the processor raises an 'Environment call from
//! U-mode' exception, which is handled as one of the cases in
//! [`crate::trap::trap_handler`].
//!
//! For clarity, each single syscall is implemented as its own function, named
//! `sys_` then the name of the syscall. You can find functions like this in
//! submodules, and you should also implement syscalls this way.
//! 

/// total syscall count
pub const SYSCALL_CNT: usize = 8;

/// write syscall
pub const SYSCALL_WRITE: usize = 64;
/// exit syscall
pub const SYSCALL_EXIT: usize = 93;
/// yield syscall
pub const SYSCALL_YIELD: usize = 124;
/// gettime syscall
pub const SYSCALL_GET_TIME: usize = 169;
/// sbrk syscall
pub const SYSCALL_SBRK: usize = 214;
/// munmap syscall
pub const SYSCALL_MUNMAP: usize = 215;
/// mmap syscall
pub const SYSCALL_MMAP: usize = 222;
/// taskinfo syscall
pub const SYSCALL_TASK_INFO: usize = 410;

mod fs;
mod process;

use fs::*;
use process::*;
pub use process::TaskInfo;
use crate::task::add_syscall_time;
/// handle syscall exception with `syscall_id` and other arguments
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    if [SYSCALL_WRITE, SYSCALL_EXIT, SYSCALL_YIELD, SYSCALL_GET_TIME, SYSCALL_SBRK, SYSCALL_MUNMAP, SYSCALL_MMAP, SYSCALL_TASK_INFO].contains(&syscall_id) {
        match syscall_id {
            SYSCALL_WRITE => add_syscall_time(0),
            SYSCALL_EXIT => add_syscall_time(1),
            SYSCALL_YIELD => add_syscall_time(2),
            SYSCALL_GET_TIME => add_syscall_time(3),
            SYSCALL_SBRK => add_syscall_time(4),
            SYSCALL_MUNMAP => add_syscall_time(5),
            SYSCALL_MMAP => add_syscall_time(6),
            SYSCALL_TASK_INFO => add_syscall_time(7),
            _ => panic!("Unsupported syscall_id: {}", syscall_id),
        }
    }
    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(args[0] as *mut TimeVal, args[1]),
        SYSCALL_TASK_INFO => sys_task_info(args[0] as *mut TaskInfo),
        SYSCALL_MMAP => sys_mmap(args[0], args[1], args[2]),
        SYSCALL_MUNMAP => sys_munmap(args[0], args[1]),
        SYSCALL_SBRK => sys_sbrk(args[0] as i32),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
