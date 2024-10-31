//! Process management syscalls
use core::mem::size_of;
use crate::{
    config::{MAX_SYSCALL_NUM, PAGE_SIZE}, mm::translated_byte_buffer, task::{
        change_program_brk, current_user_token, exit_current_and_run_next, 
        get_current_task_start_time, get_current_task_syscall_time, 
        suspend_current_and_run_next, task_mmap, TaskStatus}, timer::{get_time_ms, get_time_us},
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
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
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
    let time = get_time_us();
    let time = TimeVal {
        sec: time / 1_000_000,
        usec: time % 1_000_000,
    };
    let time_val_size = size_of::<TimeVal>();
    let buffers = translated_byte_buffer
        (current_user_token(), _ts as *const u8, time_val_size);

    unsafe {
        for buffer in buffers {
            core::ptr::copy(
                &time as *const TimeVal as *const u8, 
                buffer.as_mut_ptr(), 
                time_val_size,
            );
        }

    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
    let info = TaskInfo {
        status : TaskStatus::Running,
        syscall_times : get_current_task_syscall_time(),
        time : get_time_ms() - get_current_task_start_time(),
    };
    
    let len = size_of::<TaskInfo>();
    let buffers = translated_byte_buffer
        (current_user_token(), _ti as *const u8, len);
    unsafe {
        for buffer in buffers {
            core::ptr::copy(
                &info as *const TaskInfo as *const u8,
                buffer.as_mut_ptr(),
                len);
        }
    }
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    if _start % PAGE_SIZE != 0 {
        return -1;
    }
    if _port & !0x7 != 0 {
        return -1;
    }
    if _port & 0x7 == 0 {
        return -1;
    }
    task_mmap(_start, _len, _port)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    -1
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
