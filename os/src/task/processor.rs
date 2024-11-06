//!Implementation of [`Processor`] and Intersection of control flow
//!
//! Here, the continuous operation of user apps in CPU is maintained,
//! the current running state of CPU is recorded,
//! and the replacement and transfer of control flow of different applications are executed.

use super::__switch;
use super::{fetch_task, TaskStatus};
use super::{TaskContext, TaskControlBlock};
use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use crate::config::{PAGE_SIZE, PAGE_SIZE_BITS};
use crate::mm::{MapPermission, PageTable, VirtAddr, VirtPageNum};
use alloc::sync::Arc;
use lazy_static::*;

/// Processor management structure
pub struct Processor {
    ///The task currently executing on the current processor
    current: Option<Arc<TaskControlBlock>>,

    ///The basic control flow of each core, helping to select and switch process
    idle_task_cx: TaskContext,
}

impl Processor {
    ///Create an empty Processor
    pub fn new() -> Self {
        Self {
            current: None,
            idle_task_cx: TaskContext::zero_init(),
        }
    }

    ///Get mutable reference to `idle_task_cx`
    fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_cx as *mut _
    }

    ///Get current task in moving semanteme
    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }

    ///Get current task in cloning semanteme
    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(Arc::clone)
    }

    /// Map area to current task
    fn map_area(&mut self, _start: usize, len: usize, _port: usize) -> isize{
        let mut inner = self.current.as_mut().unwrap().inner_exclusive_access();
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
        let page_table = PageTable::from_token(inner.get_user_token());
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
        inner.memory_set.insert_framed_area(_start.into(), (_start + len).into(), permission);
        0
    }

    /// unmap area in current task's pagetable
    fn unmap_area(&mut self, _start: usize, len: usize) -> isize{
        let mut inner = self.current.as_mut().unwrap().inner_exclusive_access();
        let len = ((len - 1 + PAGE_SIZE) / PAGE_SIZE) << PAGE_SIZE_BITS;
        let mut page_table = PageTable::from_token(inner.get_user_token());
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
        for area in &mut inner.memory_set.areas {
            if virt_page == area.vpn_range.get_start() && virt_end == area.vpn_range.get_end() {
                area.unmap(&mut page_table);
                return 0;
            }
        }
        -1
    }

    /// set priority
    fn set_prio(&mut self, _prio: usize) {
        let mut inner = self.current.as_mut().unwrap().inner_exclusive_access();
        inner.prio = _prio;
    }
}

lazy_static! {
    pub static ref PROCESSOR: UPSafeCell<Processor> = unsafe { UPSafeCell::new(Processor::new()) };
}

///The main part of process execution and scheduling
///Loop `fetch_task` to get the process that needs to run, and switch the process through `__switch`
pub fn run_tasks() {
    loop {
        let mut processor = PROCESSOR.exclusive_access();
        if let Some(task) = fetch_task() {
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            // access coming task TCB exclusively
            let mut task_inner = task.inner_exclusive_access();
            let next_task_cx_ptr = &task_inner.task_cx as *const TaskContext;
            task_inner.task_status = TaskStatus::Running;
            // release coming task_inner manually
            drop(task_inner);
            // release coming task TCB manually
            processor.current = Some(task);
            // release processor manually
            drop(processor);
            unsafe {
                __switch(idle_task_cx_ptr, next_task_cx_ptr);
            }
        } else {
            warn!("no tasks available in run_tasks");
        }
    }
}

/// Get current task through take, leaving a None in its place
pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().take_current()
}

/// Get a copy of the current task
pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().current()
}

/// Get the current user token(addr of page table)
pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    task.get_user_token()
}

///Get the mutable reference to trap context of current task
pub fn current_trap_cx() -> &'static mut TrapContext {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .get_trap_cx()
}

///Return to idle control flow for new scheduling
pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let mut processor = PROCESSOR.exclusive_access();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);
    unsafe {
        __switch(switched_task_cx_ptr, idle_task_cx_ptr);
    }
}

/// map area for current task
pub fn map_current_task(_start: usize, len: usize, _port: usize) -> isize{
    PROCESSOR.exclusive_access().map_area(_start, len, _port)
}

/// unmap area for current task
pub fn unmap_current_task(_start: usize, len: usize) -> isize{
    PROCESSOR.exclusive_access().unmap_area(_start, len)
}

/// Set current task's priority
pub fn set_current_task_priority(_prio: usize){
    PROCESSOR.exclusive_access().set_prio(_prio)
}