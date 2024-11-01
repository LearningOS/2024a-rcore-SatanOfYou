//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the operating system.
//!
//! Be careful when you see `__switch` ASM function in `switch.S`. Control flow around this function
//! might not be what you expect.

mod context;
mod switch;
#[allow(clippy::module_inception)]
mod task;

use crate::config::{PAGE_SIZE, PAGE_SIZE_BITS};
use crate::loader::{get_app_data, get_num_app};
use crate::sync::UPSafeCell;
use crate::timer::get_time_ms;
use crate::syscall::{TaskInfo, SYSCALL_EXIT, SYSCALL_GET_TIME, SYSCALL_MMAP, SYSCALL_MUNMAP, SYSCALL_SBRK, SYSCALL_TASK_INFO, SYSCALL_WRITE, SYSCALL_YIELD};
use crate::trap::TrapContext;
use alloc::vec::Vec;
use lazy_static::*;
use switch::__switch;
pub use task::{TaskControlBlock, TaskStatus};
use crate::mm::{MapPermission, PageTable, VirtAddr, VirtPageNum};
pub use context::TaskContext;

/// The task manager, where all the tasks are managed.
///
/// Functions implemented on `TaskManager` deals with all task state transitions
/// and task context switching. For convenience, you can find wrappers around it
/// in the module level.
///
/// Most of `TaskManager` are hidden behind the field `inner`, to defer
/// borrowing checks to runtime. You can see examples on how to use `inner` in
/// existing functions on `TaskManager`.
pub struct TaskManager {
    /// total number of tasks
    num_app: usize,
    /// use inner value to get mutable access
    inner: UPSafeCell<TaskManagerInner>,
}

/// The task manager inner in 'UPSafeCell'
struct TaskManagerInner {
    /// task list
    tasks: Vec<TaskControlBlock>,
    /// id of current `Running` task
    current_task: usize,
}

lazy_static! {
    /// a `TaskManager` global instance through lazy_static!
    pub static ref TASK_MANAGER: TaskManager = {
        println!("init TASK_MANAGER");
        let num_app = get_num_app();
        println!("num_app = {}", num_app);
        let mut tasks: Vec<TaskControlBlock> = Vec::new();
        for i in 0..num_app {
            tasks.push(TaskControlBlock::new(get_app_data(i), i));
        }
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                })
            },
        }
    };
}

impl TaskManager {
    /// Run the first task in task list.
    ///
    /// Generally, the first task in task list is an idle task (we call it zero process later).
    /// But in ch4, we load apps statically, so the first task is a real app.
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let next_task = &mut inner.tasks[0];
        next_task.task_status = TaskStatus::Running;
        next_task.start_time = get_time_ms();
        let next_task_cx_ptr = &next_task.task_cx as *const TaskContext;
        drop(inner);
        let mut _unused = TaskContext::zero_init();
        // before this, we should drop local variables that must be dropped manually
        unsafe {
            __switch(&mut _unused as *mut _, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }

    /// Change the status of current `Running` task into `Ready`.
    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let cur = inner.current_task;
        inner.tasks[cur].task_status = TaskStatus::Ready;
    }

    /// Change the status of current `Running` task into `Exited`.
    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let cur = inner.current_task;
        inner.tasks[cur].task_status = TaskStatus::Exited;
    }

    /// Find next task to run and return task id.
    ///
    /// In this case, we only return the first `Ready` task in task list.
    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    /// Get the current 'Running' task's token.
    fn get_current_token(&self) -> usize {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].get_user_token()
    }

    /// Get the current 'Running' task's trap contexts.
    fn get_current_trap_cx(&self) -> &'static mut TrapContext {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].get_trap_cx()
    }

    /// Change the current 'Running' task's program break
    pub fn change_current_program_brk(&self, size: i32) -> Option<usize> {
        let mut inner = self.inner.exclusive_access();
        let cur = inner.current_task;
        inner.tasks[cur].change_program_brk(size)
    }

    /// Switch current `Running` task to the task we have found,
    /// or there is no `Ready` task and we can exit with all applications completed
    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            if inner.tasks[next].start_time == 0 {
                inner.tasks[next].start_time = get_time_ms();
            }
            inner.current_task = next;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            drop(inner);
            // before this, we should drop local variables that must be dropped manually
            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
            // go back to user mode
        } else {
            panic!("All applications completed!");
        }
    }

    /// Add syscall times
    fn add_syscall_times(&self, syscall_idx: usize) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].syscall_times[syscall_idx] += 1;
    }

    /// dump current task info
    fn dump_info(&self, task: &mut TaskInfo) {
        let inner = self.inner.exclusive_access();
        let syscall_arr = [SYSCALL_WRITE, SYSCALL_EXIT, SYSCALL_YIELD, SYSCALL_GET_TIME, SYSCALL_SBRK, SYSCALL_MUNMAP, SYSCALL_MMAP, SYSCALL_TASK_INFO];
        let current = inner.current_task;
        task.time = get_time_ms() - inner.tasks[current].start_time;
        for i in 0..syscall_arr.len() {
            task.syscall_times[syscall_arr[i]] = inner.tasks[current].syscall_times[i];
        }
        task.status = inner.tasks[current].task_status;
    }

    /// map area in current task's pagetable
    fn map_area(&self, _start: usize, len: usize, _port: usize) -> isize{
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        let len = ((len - 1 + PAGE_SIZE) / PAGE_SIZE) << PAGE_SIZE_BITS;
        let mut permission = MapPermission::U;
        if _port & 0x1 != 0 {
            permission = (permission) | MapPermission::R;
        }
        if _port & 0x2 != 0 {
            permission = (permission) | MapPermission::W;
        }
        if _port & 0x4 != 0 {
            permission = (permission) | MapPermission::X;
        }
        let page_table = PageTable::from_token(inner.tasks[current].get_user_token());
        let mut cnt = 0;
        loop {
            if cnt >= len {
                break;
            }
            let pte = page_table.find_pte(VirtAddr(_start + cnt).into());
            if pte.is_some() && pte.unwrap().is_valid() {
                println!("map length 0x{:x} at address 0x{:x} remap, return -1", len, _start + cnt);
                return -1;
            }
            cnt += PAGE_SIZE;
        }
        drop(page_table);
        inner.tasks[current].memory_set.insert_framed_area(_start.into(), (_start + len).into(), permission);
        println!("map virtual address from 0x{:x} to 0x{:x}", _start, _start + len);
        0
    }

    /// unmap area in current task's pagetable
    fn unmap_area(&self, _start: usize, len: usize) -> isize{
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        let len = ((len - 1 + PAGE_SIZE) / PAGE_SIZE) << PAGE_SIZE_BITS;
        let mut page_table = PageTable::from_token(inner.tasks[current].get_user_token());
        let mut cnt = 0;
        loop {
            if cnt >= len {
                break;
            }
            if page_table.find_pte(VirtAddr(_start + cnt).into()).is_none() {
                println!("address 0x{:x} not map, return -1", _start + cnt);
                return -1;
            }
            cnt += PAGE_SIZE;
        }
        let virt_page: VirtPageNum = VirtAddr(_start).into();
        let virt_end: VirtPageNum = VirtAddr(_start + len).into();
        // page_table.unmap(VirtAddr(_start + cnt).into());
        for area in &mut inner.tasks[current].memory_set.areas {
            if virt_page == area.vpn_range.get_start() && virt_end == area.vpn_range.get_end() {
                area.unmap(&mut page_table);
                return 0;
            }
        }
        -1
    }
}

/// Run the first task in task list.
pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

/// Switch current `Running` task to the task we have found,
/// or there is no `Ready` task and we can exit with all applications completed
fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

/// Change the status of current `Running` task into `Ready`.
fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

/// Change the status of current `Running` task into `Exited`.
fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

/// Suspend the current 'Running' task and run the next task in task list.
pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

/// Exit the current 'Running' task and run the next task in task list.
pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

/// Get the current 'Running' task's token.
pub fn current_user_token() -> usize {
    TASK_MANAGER.get_current_token()
}

/// Get the current 'Running' task's trap contexts.
pub fn current_trap_cx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_cx()
}

/// Change the current 'Running' task's program break
pub fn change_program_brk(size: i32) -> Option<usize> {
    TASK_MANAGER.change_current_program_brk(size)
}

/// Add syscall times
pub fn add_syscall_time(syscall_idx: usize) {
    TASK_MANAGER.add_syscall_times(syscall_idx);
}

/// Dump current task info 
pub fn dump_task_info(task: &mut TaskInfo) {
    TASK_MANAGER.dump_info(task);
}

/// map area for current task
pub fn map_current_task(_start: usize, len: usize, _port: usize) -> isize{
    TASK_MANAGER.map_area(_start, len, _port)
}

/// unmap area for current task
pub fn unmap_current_task(_start: usize, len: usize) -> isize{
    TASK_MANAGER.unmap_area(_start, len)
}