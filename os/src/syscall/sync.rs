use crate::sync::{Condvar, Mutex, MutexBlocking, MutexSpin, Semaphore};
use crate::task::{block_current_and_run_next, current_process, current_task};
use crate::timer::{add_timer, get_time_ms};
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
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
        process_inner.mutex_list[id] = mutex;
        id as isize
    } else {
        process_inner.mutex_list.push(mutex);
        process_inner.mutex_list.len() as isize - 1
    }
}
/// mutex lock syscall
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
    if process
        .inner_exclusive_access()
        .deadlock_detect_enabled
    {
        let locked = process.inner_exclusive_access().mutex_list[mutex_id]
            .as_ref()
            .unwrap()
            .get_holder()
            .is_some();
        if locked && mutex_deadlock_detect(mutex_id) {
            return -0xdead;
        }
    }
    let process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
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
    let process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
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
    let process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
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
    if process
        .inner_exclusive_access()
        .deadlock_detect_enabled
    {
        let (would_block, target) = {
            let process_inner = process.inner_exclusive_access();
            let sem = process_inner.semaphore_list[sem_id].as_ref().unwrap();
            (
                sem.get_count() <= 0,
                Arc::as_ptr(sem) as *const Semaphore,
            )
        };
        if would_block && semaphore_deadlock_detect(target) {
            return -0xdead;
        }
    }
    let process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    drop(process_inner);
    sem.down();
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
pub fn sys_enable_deadlock_detect(enabled: usize) -> isize {
    trace!("kernel: sys_enable_deadlock_detect {}", enabled);
    if enabled > 1 {
        return -1;
    }
    current_process()
        .inner_exclusive_access()
        .deadlock_detect_enabled = enabled == 1;
    0
}

fn current_tid() -> usize {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid
}

fn mutex_deadlock_detect(mutex_id: usize) -> bool {
    let process = current_process();
    let inner = process.inner_exclusive_access();
    let my_tid = current_tid();
    let Some(mutex) = inner.mutex_list[mutex_id].as_ref() else {
        return false;
    };
    let mut holder = match mutex.get_holder() {
        Some(tid) => tid,
        None => return false,
    };
    if holder == my_tid {
        return true;
    }

    let mut visited = vec![holder];
    loop {
        let next = inner
            .mutex_list
            .iter()
            .filter_map(|mutex| mutex.as_ref())
            .find_map(|mutex| {
                if mutex.get_wait_queue_tids().contains(&holder) {
                    mutex.get_holder()
                } else {
                    None
                }
            });
        let Some(next) = next else {
            return false;
        };
        if next == my_tid {
            return true;
        }
        if !visited.contains(&next) {
            visited.push(next);
            holder = next;
        } else {
            return false;
        }
    }
}

fn semaphore_deadlock_detect(target: *const Semaphore) -> bool {
    let process = current_process();
    let inner = process.inner_exclusive_access();
    let my_tid = current_tid();
    let semaphores: Vec<&Arc<Semaphore>> = inner
        .semaphore_list
        .iter()
        .filter_map(|semaphore| semaphore.as_ref())
        .collect();
    if semaphores.is_empty() {
        return false;
    }

    let tids: Vec<usize> = inner
        .tasks
        .iter()
        .filter_map(|task| {
            task.as_ref().and_then(|task| {
                task.inner_exclusive_access()
                    .res
                    .as_ref()
                    .map(|res| res.tid)
            })
        })
        .collect();
    let n = tids.len();
    let m = semaphores.len();
    let mut available = vec![0isize; m];
    let mut allocation = vec![vec![0isize; m]; n];
    let mut need = vec![vec![0isize; m]; n];

    for (index, semaphore) in semaphores.iter().enumerate() {
        available[index] = semaphore.get_count().max(0);
        for tid in semaphore.get_holders() {
            if let Some(task_index) = tids.iter().position(|candidate| *candidate == tid) {
                allocation[task_index][index] += 1;
            }
        }
        for tid in semaphore.get_wait_queue_tids() {
            if let Some(task_index) = tids.iter().position(|candidate| *candidate == tid) {
                need[task_index][index] += 1;
            }
        }
    }

    if let Some(task_index) = tids.iter().position(|tid| *tid == my_tid) {
        let semaphore_index = semaphores
            .iter()
            .position(|semaphore| Arc::as_ptr(semaphore) as *const Semaphore == target)
            .unwrap_or(0);
        need[task_index][semaphore_index] += 1;
    }

    let mut work = available;
    let mut finished = vec![false; n];
    loop {
        let mut progressed = false;
        for task_index in 0..n {
            if finished[task_index] {
                continue;
            }
            if (0..m).all(|resource| need[task_index][resource] <= work[resource]) {
                for resource in 0..m {
                    work[resource] += allocation[task_index][resource];
                }
                finished[task_index] = true;
                progressed = true;
            }
        }
        if !progressed {
            break;
        }
    }
    finished.iter().any(|finished| !finished)
}
