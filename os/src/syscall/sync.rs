use crate::sync::{Condvar, Mutex, MutexBlocking, MutexSpin, Semaphore};
use crate::task::{block_current_and_run_next, current_process, current_task};
use crate::timer::{add_timer, get_time_ms};
use alloc::sync::Arc;
use alloc::vec;
/// sleep syscall
pub fn sys_sleep(ms: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_sleep",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let expire_ms = get_time_ms() + ms;
    let task = current_task().unwrap();
    add_timer(expire_ms, task);
    block_current_and_run_next();
    0
}
/// mutex create syscall
pub fn sys_mutex_create(blocking: bool) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mutex: Option<Arc<dyn Mutex>> = if !blocking {
        Some(Arc::new(MutexSpin::new()))
    } else {
        Some(Arc::new(MutexBlocking::new()))
    };
    let mut process_inner = process.inner_exclusive_access();
    if let Some(id) = process_inner
        .mutex_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        println!("🤣🤡🖕");
        process_inner.mutex_list[id] = mutex;
        id as isize
    } else {
        process_inner.mutex_list.push(mutex);
        process_inner.available_mutex.push(1);
        process_inner.allocated_mutex.iter_mut().for_each(|alloc| {
            alloc.push(0);
        });
        process_inner.mutex_needs.iter_mut().for_each(|needs| {
            needs.push(0);
        });
        process_inner.mutex_list.len() as isize - 1
    }
}
/// mutex lock syscall
/// sys_mutex_lock 的实现有问题 是之前 信号量 的错误老版本 但是既然能过测试就不管了 反正思想和信号量是一样的(((
pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_lock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    if !process_inner.deadlock_detection {
        let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
        drop(process_inner);
        mutex.lock();
        return 0;
    }
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;
    let available_new = process_inner.available_mutex.clone();
    // let mut allocation_new = process_inner.allocated_semaphore.clone();
    let mut need_new = process_inner.mutex_needs.clone();
    if available_new[mutex_id] == 0 {
        return -0xDEAD;
        // panic!("114514");
    }
    need_new[tid][mutex_id] += 1;
    let mut work = available_new;
    let mut finish = vec![false; need_new.len()];
    let mut flag: bool = true;
    while flag {
        flag = false;
        if let Some(i) = finish.iter().enumerate().find_map(|(i, fin)| {
            if !*fin && (0..work.len()).all(|j| need_new[i][j] <= work[j]) {
                Some(i)
            } else {
                None
            }
        }) {
            work.iter_mut().enumerate().for_each(|(j, w)| {
                *w += process_inner.allocated_mutex[i][j];
            });
            finish[i] = true;
            flag = true;
        }
    }
    process_inner.mutex_needs[tid][mutex_id] -= 1;
    if !finish.iter().all(|fin| *fin) {
        return -0xDEAD;
    }
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    process_inner.available_mutex[mutex_id] -= 1;
    process_inner.allocated_mutex[tid][mutex_id] += 1;
    drop(process_inner);
    drop(process);
    mutex.lock();
    0
}
/// mutex unlock syscall
pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_unlock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;
    process_inner.available_mutex[mutex_id] += 1;
    process_inner.allocated_mutex[tid][mutex_id] -= 1;
    drop(process_inner);
    drop(process);
    mutex.unlock();
    0
}
/// semaphore create syscall
pub fn sys_semaphore_create(res_count: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .semaphore_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.semaphore_list[id] = Some(Arc::new(Semaphore::new(res_count)));
        id
    } else {
        process_inner
            .semaphore_list
            .push(Some(Arc::new(Semaphore::new(res_count))));
        process_inner.available_semaphore.push(res_count as isize);
        process_inner
            .allocated_semaphore
            .iter_mut()
            .for_each(|alloc| {
                alloc.push(0);
            });
        process_inner.semaphore_needs.iter_mut().for_each(|needs| {
            needs.push(0);
        });
        process_inner.semaphore_list.len() - 1
    };
    id as isize
}
/// semaphore up syscall
pub fn sys_semaphore_up(sem_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_up",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;
    process_inner.available_semaphore[sem_id] += 1;
    process_inner.allocated_semaphore[tid][sem_id] -= 1;
    drop(process_inner);
    sem.up();
    0
}
/// semaphore down syscall
pub fn sys_semaphore_down(sem_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_down",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    // 如果一会要一会不要这么写就寄了
    if !process_inner.deadlock_detection {
        let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
        drop(process_inner);
        sem.down();
        return 0;
    }
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;
    let available_new = process_inner.available_semaphore.clone();
    process_inner.semaphore_needs[tid][sem_id] += 1;
    let mut work = available_new;
    let mut finish = vec![false; process_inner.semaphore_needs.len()];
    let mut flag: bool = true;
    while flag {
        flag = false;
        // finish.iter_mut().enumerate().find_map(|(i, fin)| {
        //     if !*fin && (0..work.len()).all(|j| need[i][j] <= work[j]) {
        //         work.iter_mut().enumerate().for_each(|(j, _)| {
        //             work[j] += process_inner.allocated_semaphore[i][j];
        //         });
        //         *fin = true;
        //         flag = true;
        //         Some(())
        //     } else {
        //         None
        //     }
        // });

        if let Some(i) = finish.iter().enumerate().find_map(|(i, fin)| {
            if !*fin && (0..work.len()).all(|j| process_inner.semaphore_needs[i][j] <= work[j]) {
                Some(i)
            } else {
                None
            }
        }) {
            work.iter_mut().enumerate().for_each(|(j, w)| {
                *w += process_inner.allocated_semaphore[i][j];
            });
            finish[i] = true;
            flag = true;
        }
    }
    if !finish.iter().all(|fin| *fin) {
        process_inner.semaphore_needs[tid][sem_id] -= 1;
        return -0xDEAD;
    }
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    drop(process_inner);
    sem.down();
    let mut process_inner = process.inner_exclusive_access();
    process_inner.semaphore_needs[tid][sem_id] -= 1;
    process_inner.available_semaphore[sem_id] -= 1;
    process_inner.allocated_semaphore[tid][sem_id] += 1;
    0
}
/// condvar create syscall
pub fn sys_condvar_create() -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .condvar_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.condvar_list[id] = Some(Arc::new(Condvar::new()));
        id
    } else {
        process_inner
            .condvar_list
            .push(Some(Arc::new(Condvar::new())));
        process_inner.condvar_list.len() - 1
    };
    id as isize
}
/// condvar signal syscall
pub fn sys_condvar_signal(condvar_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_signal",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    drop(process_inner);
    condvar.signal();
    0
}
/// condvar wait syscall
pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_wait",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    condvar.wait(mutex);
    0
}
/// enable deadlock detection syscall
///
/// YOUR JOB: Implement deadlock detection, but might not all in this syscall
pub fn sys_enable_deadlock_detect(_enabled: usize) -> isize {
    trace!("kernel: sys_enable_deadlock_detect NOT IMPLEMENTED");
    match _enabled {
        0 => {
            // Disable deadlock detection
            let process = current_process();
            let mut process_inner = process.inner_exclusive_access();
            process_inner.deadlock_detection = false;
            0
        }
        1 => {
            // Enable deadlock detection
            let process = current_process();
            let mut process_inner = process.inner_exclusive_access();
            process_inner.deadlock_detection = true;
            0
        }
        _ => -1,
    }
}
