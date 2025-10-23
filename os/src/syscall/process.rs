//! Process management syscalls
use crate::{
    mm::{translated_byte_buffer, write_user_buffer, MapPermission},
    task::{
        alloc, change_program_brk, current_user_token, dealloc, exit_current_and_run_next,
        get_current_task_id, get_syscall_count, suspend_current_and_run_next,
    },
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
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
    let us = get_time_us();
    unsafe {
        let _ = write_user_buffer(
            current_user_token(),
            _ts as *mut u8,
            // 你就说能不能用嘛()
            &mut core::mem::transmute::<TimeVal, [u8; core::mem::size_of::<TimeVal>()]>(TimeVal {
                sec: us / 1_000_000,
                usec: us % 1_000_000,
            }),
        );
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    match _trace_request {
        0 => {
            // let val = core::ptr::read_volatile(target_ptr);
            if let Some(val) = translated_byte_buffer(current_user_token(), _id as *const u8, 1) {
                return val[0][0] as isize;
            }
            -1
        }
        1 => {
            let target_ptr: *mut u8 = _id as *mut u8;
            let new_val: u8 = _data as u8;
            let res = write_user_buffer(
                current_user_token(),
                target_ptr,
                core::slice::from_ref(&new_val),
            );
            if res.is_err() {
                return -1;
            }
            0
        }
        2 => get_syscall_count(get_current_task_id(), _id) as isize,
        _ => return -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    if _port & !0x7 != 0 || _port & 0x7 == 0 || _start & 0xfff != 0 {
        return -1;
    }
    // println!("{}", (_port << 1 | 0x10) as u8);
    return alloc(
        get_current_task_id(),
        _start,
        _start + _len,
        MapPermission::from_bits_truncate((_port << 1 | 0x10) as u8),
    );
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    if _start & 0xfff != 0 {
        return -1;
    }
    return dealloc(get_current_task_id(), _start, _len);
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
