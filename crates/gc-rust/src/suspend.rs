use crate::handle_store::{ObjectHandle, check_diagnostic_handle};
use crate::runtime::{SuspendReason, gc_to_clr};
use crate::util::log_info;
use std::sync::atomic::{AtomicU8, Ordering};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Running,
    Suspending,
    Suspended,
    Restarting,
    Failed,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Status {
    Ok,
    Rejected,
    SuspendFailed,
    OperationFailed,
    RestartFailed,
}

struct Lifecycle {
    state: AtomicU8,
}

impl Lifecycle {
    const fn new() -> Self {
        Self {
            state: AtomicU8::new(State::Running as u8),
        }
    }

    fn set(&self, state: State) {
        self.state.store(state as u8, Ordering::Release);
    }

    fn run(
        &self,
        suspend: impl FnOnce() -> Result<(), ()>,
        operation: impl FnOnce() -> Result<(), ()>,
        restart: impl FnOnce() -> Result<(), ()>,
    ) -> Status {
        if self
            .state
            .compare_exchange(
                State::Running as u8,
                State::Suspending as u8,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_err()
        {
            return Status::Rejected;
        }
        if suspend().is_err() {
            self.set(State::Failed);
            return Status::SuspendFailed;
        }
        self.set(State::Suspended);
        // Private and stack-bound: the suspension never escapes this call.
        // Drop also covers a Rust operation panic before explicit restart.
        let mut guard = RestartGuard {
            lifecycle: self,
            restart: Some(restart),
        };
        let operation_result = operation();
        if guard.restart().is_err() {
            Status::RestartFailed
        } else if operation_result.is_err() {
            Status::OperationFailed
        } else {
            Status::Ok
        }
    }
}

struct RestartGuard<'a, R: FnOnce() -> Result<(), ()>> {
    lifecycle: &'a Lifecycle,
    restart: Option<R>,
}

impl<R: FnOnce() -> Result<(), ()>> RestartGuard<'_, R> {
    fn restart(&mut self) -> Result<(), ()> {
        // Consume before calling out, so failure cannot trigger a second attempt.
        let restart = self.restart.take().expect("restart already attempted");
        self.lifecycle.set(State::Restarting);
        let result = restart();
        self.lifecycle.set(if result.is_ok() {
            State::Running
        } else {
            State::Failed
        });
        result
    }
}

impl<R: FnOnce() -> Result<(), ()>> Drop for RestartGuard<'_, R> {
    fn drop(&mut self) {
        if self.restart.is_some() {
            let _ = self.restart();
        }
    }
}

static LIFECYCLE: Lifecycle = Lifecycle::new();

/// Fixed-size copied observations; contains no managed references.
#[repr(C)]
pub struct DiagnosticResult {
    status: Status,
    restart_attempts: u32,
}

const _: () = {
    assert!(size_of::<DiagnosticResult>() == 8);
    assert!(align_of::<DiagnosticResult>() == 4);
    assert!(std::mem::offset_of!(DiagnosticResult, restart_attempts) == 4);
};

/// One empty diagnostic cycle. No managed allocation, logging, or sleeping
/// occurs between suspension and restart. An unexpected Rust panic at this
/// extern-C boundary terminates the process instead of unwinding across FFI.
///
/// # Safety
/// Call only through ordinary GC-transitioning P/Invoke after GC initialization,
/// with no managed monitor or native lock held that suspension may need.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_gc_suspend_ee() -> DiagnosticResult {
    log_info!("rust_gc_suspend_ee()");
    run_diagnostic(|| Ok(()))
}

/// Bounded handle-handoff probe; nonzero `fail_operation` injects an error after
/// handle resolution, before restart. The result never contains an object pointer.
/// An unexpected panic aborts at this extern-C boundary; it cannot unwind to C#.
///
/// # Safety
/// All `rust_gc_suspend_ee` preconditions apply. `handle` must be a live normal
/// GCHandle allocated by this ZeroGC instance, exclusively owned by the caller
/// and kept unchanged until return. Stock-GC handles are unsupported.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_gc_suspend_with_handle(
    handle: ObjectHandle,
    fail_operation: u32,
) -> DiagnosticResult {
    log_info!(
        "rust_gc_suspend_with_handle([{:?}], fail_operation: {})",
        handle,
        fail_operation
    );
    run_diagnostic(|| {
        // SAFETY: the caller owns the live store handle and cannot change/free it
        // during this synchronous call. Lifecycle invokes this only while stopped.
        unsafe { check_diagnostic_handle(handle) }?;
        if fail_operation == 0 { Ok(()) } else { Err(()) }
    })
}

fn run_diagnostic(operation: impl FnOnce() -> Result<(), ()>) -> DiagnosticResult {
    let callbacks = gc_to_clr();
    let mut restart_attempts = 0;
    // CoreCLR methods return void. Only injected callbacks return Err; an abort
    // or partially completed runtime call is not a recoverable failure here.
    let status = LIFECYCLE.run(
        || {
            // SAFETY: the caller guarantees runtime initialization and an eligible
            // thread. Lifecycle admits one owner; required GC callbacks exist.
            unsafe { callbacks.suspend_ee(SuspendReason::SuspendForGcPrep) };
            Ok(())
        },
        operation,
        || {
            restart_attempts += 1;
            // SAFETY: the local guard owns a successful suspension on this
            // thread and attempts restart once, after the bounded operation.
            // This diagnostic completes no collection.
            unsafe { callbacks.restart_ee(false) };
            Ok(())
        },
    );
    DiagnosticResult {
        status,
        restart_attempts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        cell::Cell,
        panic::{AssertUnwindSafe, catch_unwind},
    };

    #[test]
    fn repeated_cycles_restart_once_and_reject_nested_entry() {
        let lifecycle = Lifecycle::new();
        let suspends = Cell::new(0);
        let restarts = Cell::new(0);
        for _ in 0..3 {
            assert_eq!(
                lifecycle.run(
                    || {
                        suspends.set(suspends.get() + 1);
                        Ok(())
                    },
                    || {
                        assert_eq!(
                            lifecycle.state.load(Ordering::Acquire),
                            State::Suspended as u8
                        );
                        assert_eq!(
                            lifecycle.run(
                                || panic!("nested suspend must not run"),
                                || panic!("nested operation must not run"),
                                || panic!("nested restart must not run"),
                            ),
                            Status::Rejected
                        );
                        Ok(())
                    },
                    || {
                        restarts.set(restarts.get() + 1);
                        Ok(())
                    }
                ),
                Status::Ok
            );
        }
        assert_eq!((suspends.get(), restarts.get()), (3, 3));
    }

    #[test]
    fn operation_error_and_panic_each_restart_once() {
        let lifecycle = Lifecycle::new();
        let restarts = Cell::new(0);
        let restart = || {
            restarts.set(restarts.get() + 1);
            Ok(())
        };
        assert_eq!(
            lifecycle.run(|| Ok(()), || Err(()), restart),
            Status::OperationFailed
        );
        let panic = catch_unwind(AssertUnwindSafe(|| {
            lifecycle.run(|| Ok(()), || panic!("injected operation panic"), restart);
        }));
        assert!(panic.is_err());
        assert_eq!(restarts.get(), 2);
        assert_eq!(
            lifecycle.state.load(Ordering::Acquire),
            State::Running as u8
        );
    }

    #[test]
    fn suspend_error_does_not_restart_or_run_operation() {
        let lifecycle = Lifecycle::new();
        assert_eq!(
            lifecycle.run(
                || Err(()),
                || panic!("operation must not run"),
                || panic!("restart must not run")
            ),
            Status::SuspendFailed
        );
        assert_eq!(
            lifecycle.run(|| Ok(()), || Ok(()), || Ok(())),
            Status::Rejected
        );
    }

    #[test]
    fn restart_error_is_reported_without_retry_and_blocks_next_cycle() {
        for operation_result in [Ok(()), Err(())] {
            let lifecycle = Lifecycle::new();
            let restarts = Cell::new(0);
            assert_eq!(
                lifecycle.run(
                    || Ok(()),
                    || operation_result,
                    || {
                        restarts.set(restarts.get() + 1);
                        Err(())
                    }
                ),
                Status::RestartFailed
            );
            assert_eq!(lifecycle.state.load(Ordering::Acquire), State::Failed as u8);
            assert_eq!(
                lifecycle.run(|| Ok(()), || Ok(()), || Ok(())),
                Status::Rejected
            );
            assert_eq!(restarts.get(), 1);
        }
    }

    #[test]
    fn every_non_running_state_rejects_entry() {
        let lifecycle = Lifecycle::new();
        for state in [
            State::Suspending,
            State::Suspended,
            State::Restarting,
            State::Failed,
        ] {
            lifecycle.set(state);
            assert_eq!(
                lifecycle.run(
                    || panic!("suspend must not run"),
                    || panic!("operation must not run"),
                    || panic!("restart must not run"),
                ),
                Status::Rejected
            );
        }
    }
}
