//! Process management syscalls
use crate::{
    config::MAX_SYSCALL_NUM, mm::{translated_byte_buffer, VirtAddr}, task::{
        change_program_brk, current_user_token, dump_task_info, exit_current_and_run_next, map_current_task, suspend_current_and_run_next, TaskStatus,unmap_current_task
    }, timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    pub status: TaskStatus,
    /// The numbers of syscall called by task
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    pub time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let buffers = translated_byte_buffer(current_user_token(), _ts as *const u8, core::mem::size_of::<TimeVal>());
    let time_now = get_time_us();
    let temp = TimeVal {
        sec: time_now / 1_000_000,
        usec: time_now % 1_000_000,
    };
    unsafe {
        let src = core::slice::from_raw_parts(
            &temp as *const TimeVal as *const u8,
            core::mem::size_of::<TimeVal>(),
        );
        // 将 src 的内容复制到 buffers 指向的内存区域
        let mut count = 0;
        for buffer in buffers {
            for j in 0..buffer.len() {
                buffer[j] = src[count];
                count += 1;
            }
        }
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
    let mut temp = TaskInfo {
        status: TaskStatus::Ready,
        syscall_times: [0; MAX_SYSCALL_NUM],
        time: 0
    }; 
    dump_task_info(&mut temp);
    let buffers = translated_byte_buffer(current_user_token(), _ti as *const u8, core::mem::size_of::<TaskInfo>());
    unsafe {
        let src = core::slice::from_raw_parts(
            &temp as *const TaskInfo as *const u8,
            core::mem::size_of::<TaskInfo>(),
        );
        // 将 src 的内容复制到 buffers 指向的内存区域
        let mut count = 0;
        for buffer in buffers {
            for j in 0..buffer.len() {
                buffer[j] = src[count];
                count += 1;
            }
        }
    }
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    let va: VirtAddr = _start.into();
    if va.page_offset() != 0 ||(_port & !0x7 != 0) || (_port & 0x7 == 0){
        return -1;
    }
    map_current_task(_start, _len, _port)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    let va: VirtAddr = _start.into();
    if va.page_offset() != 0 {
        return -1;
    }
    unmap_current_task(_start, _len)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
