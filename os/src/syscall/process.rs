//! Process management syscalls
// extern crate libc;
use crate::{
    config::PAGE_SIZE,
    config::MAX_SYSCALL_NUM,
    task::TaskStatus,
    task::suspend_current_and_run_next,
    task::exit_current_and_run_next,
    task::change_program_brk,
    task::mmap,
    task::munmap,
    task::current_user_token,
    task::get_current_task_info,
    timer::get_time_us,
    mm::translated_mut_ptr
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
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let p_ts = translated_mut_ptr(current_user_token(), ts);
    *p_ts = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    let (status, syscall_times, start_time) = get_current_task_info();
    
    let time_now = get_time_us();
    let time_now_ms = ((time_now / 1_000_000) & 0xffff) * 1000 + (time_now % 1_000_000 ) / 1000;
    let time_start_ms = ((start_time / 1_000_000) & 0xffff) * 1000 + (start_time % 1_000_000 ) / 1000;
    let time = time_now_ms - time_start_ms;

    let pti = translated_mut_ptr(current_user_token(), ti);
    *pti = TaskInfo {
        status,
        syscall_times,
        time,
    };
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap");
    // 合法性检验
    if start %  PAGE_SIZE != 0 || port & !0x7 != 0 || port &0x7 == 0{
        return -1;
    }
    mmap(start, len, port)
}

// YOUR JOB: Implement munmap.
pub fn sys_unmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_unmap!");
   if start %  PAGE_SIZE != 0 {
        return -1;
    }
    munmap(start, len)
}

/// change data segment size
pub fn sys_brk(size: i32) -> isize {
    trace!("kernel: sys_brk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
