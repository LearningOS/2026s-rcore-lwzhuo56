//! Semaphore

use crate::sync::UPSafeCell;
use crate::task::{block_current_and_run_next, current_task, wakeup_task, TaskControlBlock};
use alloc::{collections::VecDeque, sync::Arc};

/// semaphore structure
pub struct Semaphore {
    /// semaphore inner
    pub inner: UPSafeCell<SemaphoreInner>,
}

pub struct SemaphoreInner {
    pub count: isize,
    pub wait_queue: VecDeque<Arc<TaskControlBlock>>,
    pub holders: alloc::vec::Vec<usize>,
}

impl Semaphore {
    /// Create a new semaphore
    pub fn new(res_count: usize) -> Self {
        trace!("kernel: Semaphore::new");
        Self {
            inner: unsafe {
                UPSafeCell::new(SemaphoreInner {
                    count: res_count as isize,
                    wait_queue: VecDeque::new(),
                    holders: alloc::vec::Vec::new(),
                })
            },
        }
    }

    /// up operation of semaphore
    pub fn up(&self) {
        trace!("kernel: Semaphore::up");
        let mut inner = self.inner.exclusive_access();
        let current_tid = current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .map(|res| res.tid);
        if let Some(tid) = current_tid {
            if let Some(index) = inner.holders.iter().rposition(|holder| *holder == tid) {
                inner.holders.remove(index);
            }
        }
        inner.count += 1;
        if inner.count <= 0 {
            if let Some(task) = inner.wait_queue.pop_front() {
                wakeup_task(task);
            }
        }
    }

    /// down operation of semaphore
    pub fn down(&self) {
        trace!("kernel: Semaphore::down");
        let mut inner = self.inner.exclusive_access();
        let current = current_task().unwrap();
        let current_tid = current
            .inner_exclusive_access()
            .res
            .as_ref()
            .map(|res| res.tid);
        inner.count -= 1;
        if inner.count < 0 {
            inner.wait_queue.push_back(current);
            drop(inner);
            block_current_and_run_next();
            if let Some(tid) = current_tid {
                self.inner.exclusive_access().holders.push(tid);
            }
        } else if let Some(tid) = current_tid {
            inner.holders.push(tid);
        }
    }

    /// Return the number of available units.
    pub fn get_count(&self) -> isize {
        self.inner.exclusive_access().count
    }

    /// Return the tids holding semaphore units.
    pub fn get_holders(&self) -> alloc::vec::Vec<usize> {
        self.inner.exclusive_access().holders.clone()
    }

    /// Return the tids waiting for semaphore units.
    pub fn get_wait_queue_tids(&self) -> alloc::vec::Vec<usize> {
        self.inner
            .exclusive_access()
            .wait_queue
            .iter()
            .filter_map(|task| {
                task.inner_exclusive_access()
                    .res
                    .as_ref()
                    .map(|res| res.tid)
            })
            .collect()
    }
}
