//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::mm::MapPermission;
use crate::sync::UPSafeCell;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue.pop_front()
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
