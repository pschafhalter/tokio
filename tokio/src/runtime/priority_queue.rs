//! Run-queue structures to support a work-stealing scheduler

use crate::loom::cell::UnsafeCell;
use crate::loom::sync::atomic::{AtomicBool, AtomicU16, AtomicU32, AtomicUsize};
use crate::loom::sync::{Arc, Mutex};
use crate::runtime::task;

use std::collections::BinaryHeap;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ptr::{self, NonNull};
use std::sync::atomic::Ordering::{AcqRel, Acquire, Relaxed, Release};

/// Producer handle. May only be used from a single thread.
pub(super) struct Local<T: 'static> {
    inner: Arc<Inner<T>>,
}

/// Consumer handle. May be used from many threads.
pub(super) struct Steal<T: 'static>(Arc<Inner<T>>);

pub(super) struct Inject<T: 'static> {
    is_closed: AtomicBool,
    tasks: Arc<Mutex<BinaryHeap<task::Notified<T>>>>,
}

pub(super) struct Inner<T: 'static> {
    tasks: Arc<Mutex<BinaryHeap<task::Notified<T>>>>,
}

#[cfg(not(loom))]
const LOCAL_QUEUE_CAPACITY: usize = 256;

// Shrink the size of the local queue when using loom. This shouldn't impact
// logic, but allows loom to test more edge cases in a reasonable a mount of
// time.
#[cfg(loom)]
const LOCAL_QUEUE_CAPACITY: usize = 4;

pub(super) fn local<T: 'static>() -> (Steal<T>, Local<T>) {
    let inner = Arc::new(Inner {
        tasks: Arc::new(Mutex::new(BinaryHeap::new())),
    });

    let local = Local {
        inner: inner.clone(),
    };

    let remote = Steal(inner);

    (remote, local)
}

impl<T> Clone for Steal<T> {
    fn clone(&self) -> Steal<T> {
        Steal(self.0.clone())
    }
}

impl<T> Drop for Local<T> {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            assert!(self.pop().is_none(), "queue not empty");
        }
    }
}

impl<T> Local<T> {
    /// Returns true if the queue has entries that can be stealed.
    pub(super) fn is_stealable(&self) -> bool {
        !self.inner.is_empty()
    }

    pub(super) fn push_back(&mut self, mut task: task::Notified<T>, inject: &Inject<T>)
    where
        T: crate::runtime::task::Schedule,
    {
        let mut tasks = self.inner.tasks.lock();
        if tasks.len() < LOCAL_QUEUE_CAPACITY {
            tasks.push(task);
        } else {
            inject.push(task).ok();
        }
    }

    pub(super) fn pop(&mut self) -> Option<task::Notified<T>> {
        self.inner.tasks.lock().pop()
    }
}

impl<T> Steal<T> {
    pub(super) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Steals half the tasks from self and place them into `dst`.
    pub(super) fn steal_into(&self, dst: &mut Local<T>) -> Option<task::Notified<T>> {
        let mut src_tasks = self.0.tasks.lock();
        let mut dst_tasks = dst.inner.tasks.lock();

        let num_to_transfer = src_tasks.len() / 2;
        if dst_tasks.len() + num_to_transfer > LOCAL_QUEUE_CAPACITY {
            // For simplicity, abort.
            return None;
        }
        let res = src_tasks.pop();
        for _ in 1..num_to_transfer {
            dst_tasks.push(src_tasks.pop().unwrap());
        }
        res
    }
}

impl<T> Inner<T> {
    fn is_empty(&self) -> bool {
        self.tasks.lock().is_empty()
    }
}

impl<T: 'static> Inject<T> {
    pub(super) fn new() -> Self {
        Self {
            is_closed: AtomicBool::new(false),
            tasks: Arc::new(Mutex::new(BinaryHeap::with_capacity(LOCAL_QUEUE_CAPACITY))),
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.tasks.lock().is_empty()
    }

    pub(super) fn close(&self) -> bool {
        self.is_closed.store(true, Release);
        true
    }

    pub(super) fn is_closed(&self) -> bool {
        self.is_closed.load(Acquire)
    }

    pub(super) fn len(&self) -> usize {
        self.tasks.lock().len()
    }

    /// Pushes a value into the queue.
    ///
    /// Returns `Err(task)` if pushing fails due to the queue being shutdown.
    /// The caller is expected to call `shutdown()` on the task **if and only
    /// if** it is a newly spawned task.
    pub(super) fn push(&self, task: task::Notified<T>) -> Result<(), task::Notified<T>>
    where
        T: crate::runtime::task::Schedule,
    {
        if self.is_closed() {
            Err(task)
        } else {
            self.tasks.lock().push(task);
            Ok(())
        }
    }

    // pub(super) fn push_batch(
    //     &self,
    //     batch_head: task::Notified<T>,
    //     batch_tail: task::Notified<T>,
    //     num: usize,
    // ) {
    //     let tasks = self.tasks.lock();
    //     tasks.reserve(num);
    // }

    pub(super) fn pop(&self) -> Option<task::Notified<T>> {
        self.tasks.lock().pop()
    }
}

impl<T: 'static> Drop for Inject<T> {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            assert!(self.pop().is_none(), "queue not empty");
        }
    }
}
