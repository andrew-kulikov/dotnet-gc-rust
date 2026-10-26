//! EE-visible state for ZeroGC's diagnostic suspensions. See the pinned
//! gc/gcee.cpp: GCHeap::{IsGCInProgressHelper,WaitUntilGCComplete,SetSuspensionPending}.
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Condvar, Mutex};

struct SuspensionState {
    in_progress: AtomicBool,
    pending: AtomicUsize,
    signaled: Mutex<bool>,
    completed: Condvar,
}

impl SuspensionState {
    const fn new() -> Self {
        Self {
            in_progress: AtomicBool::new(false),
            pending: AtomicUsize::new(0),
            signaled: Mutex::new(true),
            completed: Condvar::new(),
        }
    }

    fn signal(&self, value: bool) {
        let mut signaled = self.signaled.lock().unwrap_or_else(|e| e.into_inner());
        *signaled = value;
        if value {
            self.completed.notify_all();
        }
    }

    fn wait(&self) {
        if self.in_progress.load(Ordering::Acquire) {
            let signaled = self.signaled.lock().unwrap_or_else(|e| e.into_inner());
            // The predicate and signal share a mutex: a restart between the
            // first load and wait is remembered. Spurious wakes recheck it.
            drop(
                self.completed
                    .wait_while(signaled, |signaled| !*signaled)
                    .unwrap_or_else(|e| e.into_inner()),
            );
        }
    }
}

static STATE: SuspensionState = SuspensionState::new();

#[unsafe(no_mangle)]
pub extern "C" fn rust_gc_set_gc_in_progress(value: bool) {
    STATE.in_progress.store(value, Ordering::Release);
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_gc_is_gc_in_progress_helper(_consider_gc_start: bool) -> bool {
    // In gcee.cpp, consider_gc_start includes gc_started, NOT suspension pending.
    // ZeroGC never starts a collection, so both values query only EE progress.
    STATE.in_progress.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_gc_set_suspension_pending(value: bool) {
    let result = STATE
        .pending
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
            if value {
                count.checked_add(1)
            } else {
                count.checked_sub(1)
            }
        });
    if result.is_err() {
        // An unbalanced runtime callback is unsupported, not silent success.
        std::process::abort();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_gc_set_wait_for_gc_event() {
    STATE.signal(true);
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_gc_reset_wait_for_gc_event() {
    STATE.signal(false);
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_gc_wait_until_gc_complete(_consider_gc_start: bool) -> u32 {
    // No separate gc_started phase exists in ZeroGC. Return CoreCLR's S_OK
    // only after the event (if GC is in progress); never return a timeout.
    STATE.wait();
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        sync::{Arc, mpsc},
        thread,
        time::Duration,
    };

    #[test]
    fn wait_blocks_until_signal_and_signal_is_remembered() {
        let state = Arc::new(SuspensionState::new());
        state.in_progress.store(true, Ordering::Release);
        state.signal(false);
        let (tx, rx) = mpsc::channel();
        let waiters: Vec<_> = (0..2)
            .map(|_| {
                let state = Arc::clone(&state);
                let tx = tx.clone();
                thread::spawn(move || {
                    tx.send("started").unwrap();
                    state.wait();
                    tx.send("finished").unwrap();
                })
            })
            .collect();
        for _ in 0..2 {
            assert_eq!(rx.recv_timeout(Duration::from_secs(5)).unwrap(), "started");
        }
        assert!(matches!(
            rx.recv_timeout(Duration::from_millis(30)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));
        state.signal(true);
        for _ in 0..2 {
            assert_eq!(rx.recv_timeout(Duration::from_secs(5)).unwrap(), "finished");
        }
        for waiter in waiters {
            waiter.join().unwrap();
        }
        // A signal before wait is remembered even while in_progress is true.
        state.wait();
    }
}
