//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::mm::MapPermission;
use crate::sync::UPSafeCell;
use alloc::sync::Arc;
use alloc::vec::Vec;
use lazy_static::*;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: Vec<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: Vec::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push(task);
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        if self.ready_queue.is_empty() {
            None
        } else {
            self.ready_queue
                .sort_unstable_by_key(|task| task.inner_exclusive_access().stride);
            Some(self.ready_queue.remove(0))
        }
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}

/// Allocate memory for a task with the given vpn range and permissions.
pub fn alloc(
    task: Arc<TaskControlBlock>,
    start_va: usize,
    end_va: usize,
    perm: MapPermission,
) -> isize {
    // let mut inner = TASK_MANAGER.exclusive_access();
    // let memory_set = &mut inner.tasks[task_pid].memory_set;
    let memory_set = &mut task.inner_exclusive_access().memory_set;
    let result = memory_set.insert_framed_area(start_va.into(), end_va.into(), perm);
    if result.is_ok() {
        0
    } else {
        -1
    }
}

/// Deallocate memory for a task with the given start virtual address and length.
pub fn dealloc(task: Arc<TaskControlBlock>, start_va: usize, len: usize) -> isize {
    // let mut memory_set = task.inner_exclusive_access().memory_set;
    // let result = memory_set.shrink_from(start_va.into(), (start_va + len).into());
    let memory_set = &mut task.inner_exclusive_access().memory_set;
    let result = memory_set.shrink_from(start_va.into(), (start_va + len).into());
    if result.is_ok() {
        0
    } else {
        -1
    }
}
