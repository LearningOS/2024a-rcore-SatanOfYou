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


/// write syscall
pub const SYSCALL_WRITE: usize = 64;
/// exit syscall
pub const SYSCALL_EXIT: usize = 93;
/// yield syscall
pub const SYSCALL_YIELD: usize = 124;
/// gettime syscall
pub const SYSCALL_GET_TIME: usize = 169;
/// taskinfo syscall
pub const SYSCALL_TASK_INFO: usize = 410;

mod fs;
mod process;


pub use process::TaskInfo;

use crate::task::add_syscall_cnt;
use fs::*;
use process::*;
/// handle syscall exception with `syscall_id` and other arguments
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    if [SYSCALL_WRITE, SYSCALL_EXIT, SYSCALL_YIELD, SYSCALL_GET_TIME, SYSCALL_TASK_INFO].contains(&syscall_id) {
        match syscall_id {
            SYSCALL_WRITE => add_syscall_cnt(0),
            SYSCALL_EXIT => add_syscall_cnt(1),
            SYSCALL_YIELD => add_syscall_cnt(2),
            SYSCALL_GET_TIME => add_syscall_cnt(3),
            SYSCALL_TASK_INFO => add_syscall_cnt(4),
            _ => panic!("Unsupported syscall_id: {}", syscall_id),
        }
    }
    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(args[0] as *mut TimeVal, args[1]),
        SYSCALL_TASK_INFO => sys_task_info(args[0] as *mut TaskInfo),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
