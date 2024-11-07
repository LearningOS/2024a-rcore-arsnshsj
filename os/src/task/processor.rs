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
use crate::config::MAX_SYSCALL_NUM;
use crate::mm::MapPermission;
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

    ///
    pub fn get_current_task_syscall_time(&self) -> [u32;MAX_SYSCALL_NUM] {
        let task = self.current().unwrap();
        let inner = task.inner_exclusive_access();
        inner.syscall_times
    }
    ///
    pub fn get_current_task_start_time(&self) -> usize {
        let task = self.current().unwrap();
        let inner = task.inner_exclusive_access();
        inner.start_time
    }
    ///
    pub fn update_syscall_times(&mut self, syscall_id: usize) {
        let task = self.current().unwrap();
        let mut inner = task.inner_exclusive_access();
        inner.syscall_times[syscall_id] += 1;
    }
    ///
    pub fn get_current_task_status(&mut self) -> TaskStatus{
        let task = self.current().unwrap();
        let inner = task.inner_exclusive_access();
        inner.task_status
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
///get start time
pub fn get_start_time() -> usize{
    PROCESSOR.exclusive_access().get_current_task_start_time()
}
/// get syscall times
pub fn get_syscall_time() -> [u32;MAX_SYSCALL_NUM]{ 
    PROCESSOR.exclusive_access().get_current_task_syscall_time()
}

/// get status
pub fn get_status() -> TaskStatus{ 
    PROCESSOR.exclusive_access().get_current_task_status()
}

///
pub fn update_syscall_times(syscall_id: usize) {
    PROCESSOR.exclusive_access().update_syscall_times(syscall_id);
}

///
pub fn task_mmap(_start: usize, _len: usize, port: usize) -> isize{
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    let mut _port = MapPermission::U;
    if port & (1 << 0) != 0 {
        _port |= MapPermission::R;
    }
    if port & (1 << 1) != 0 {
        _port |= MapPermission::W;
    }
    if port & (1 << 2) != 0 {
        _port |= MapPermission::X;
    }
    inner.memory_set.mmap(_start, _len, _port)
}

///
pub fn task_munmap(_start: usize, _len: usize) -> isize{
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    inner.memory_set.unmmap(_start, _len)
}