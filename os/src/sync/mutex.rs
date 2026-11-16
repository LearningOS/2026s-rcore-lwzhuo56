//! Mutex (spin-like and blocking(sleep))

use super::UPSafeCell;
use crate::task::TaskControlBlock;
use crate::task::{block_current_and_run_next, suspend_current_and_run_next};
use crate::task::{current_task, wakeup_task};
use alloc::{collections::VecDeque, sync::Arc};

/// Mutex trait
pub trait Mutex: Sync + Send {
    /// Lock the mutex
    fn lock(&self);
    /// Unlock the mutex
    fn unlock(&self);
    /// Return the tid currently holding the mutex.
    fn get_holder(&self) -> Option<usize>;
    /// Return tids waiting for the mutex.
    fn get_wait_queue_tids(&self) -> alloc::vec::Vec<usize>;
}

/// Spinlock Mutex struct
pub struct MutexSpin {
    locked: UPSafeCell<bool>,
}

impl MutexSpin {
    /// Create a new spinlock mutex
    pub fn new() -> Self {
        Self {
            locked: unsafe { UPSafeCell::new(false) },
        }
    }
}

impl Mutex for MutexSpin {
    /// Lock the spinlock mutex
    fn lock(&self) {
        trace!("kernel: MutexSpin::lock");
        loop {
            let mut locked = self.locked.exclusive_access();
            if *locked {
                drop(locked);
                suspend_current_and_run_next();
                continue;
            } else {
                *locked = true;
                return;
            }
        }
    }

    fn unlock(&self) {
        trace!("kernel: MutexSpin::unlock");
        let mut locked = self.locked.exclusive_access();
        *locked = false;
    }

    fn get_holder(&self) -> Option<usize> {
        None
    }

    fn get_wait_queue_tids(&self) -> alloc::vec::Vec<usize> {
        alloc::vec::Vec::new()
    }
}

/// Blocking Mutex struct
pub struct MutexBlocking {
    inner: UPSafeCell<MutexBlockingInner>,
}

pub struct MutexBlockingInner {
    locked: bool,
    wait_queue: VecDeque<Arc<TaskControlBlock>>,
    holder: Option<usize>,
}

impl MutexBlocking {
    /// Create a new blocking mutex
    pub fn new() -> Self {
        trace!("kernel: MutexBlocking::new");
        Self {
            inner: unsafe {
                UPSafeCell::new(MutexBlockingInner {
                    locked: false,
                    wait_queue: VecDeque::new(),
                    holder: None,
                })
            },
        }
    }
}

impl Mutex for MutexBlocking {
    /// lock the blocking mutex
    fn lock(&self) {
        trace!("kernel: MutexBlocking::lock");
        let mut mutex_inner = self.inner.exclusive_access();
        if mutex_inner.locked {
            mutex_inner.wait_queue.push_back(current_task().unwrap());
            drop(mutex_inner);
            block_current_and_run_next();
        } else {
            mutex_inner.locked = true;
            mutex_inner.holder = current_task()
                .unwrap()
                .inner_exclusive_access()
                .res
                .as_ref()
                .map(|res| res.tid);
        }
    }

    /// unlock the blocking mutex
    fn unlock(&self) {
        trace!("kernel: MutexBlocking::unlock");
        let mut mutex_inner = self.inner.exclusive_access();
        assert!(mutex_inner.locked);
        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
            mutex_inner.holder = waking_task
                .inner_exclusive_access()
                .res
                .as_ref()
                .map(|res| res.tid);
            wakeup_task(waking_task);
        } else {
            mutex_inner.locked = false;
            mutex_inner.holder = None;
        }
    }

    fn get_holder(&self) -> Option<usize> {
        self.inner.exclusive_access().holder
    }

    fn get_wait_queue_tids(&self) -> alloc::vec::Vec<usize> {
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
