use crate::handle_store::{ObjectHandle, check_diagnostic_handle, resolve_diagnostic_handle};
use crate::inspection::{InspectionStatus, Observation, inspect_node};
use crate::runtime::{SuspendReason, gc_to_clr};
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
#[derive(Debug)]
pub struct DiagnosticResult {
    status: Status,
    restart_attempts: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct InspectionResult {
    lifecycle: DiagnosticResult,
    observation: Observation,
}

const _: () = {
    assert!(size_of::<InspectionResult>() == 28);
    assert!(align_of::<InspectionResult>() == 4);
    assert!(std::mem::offset_of!(InspectionResult, observation) == 8);
};

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
#[tracing::instrument(level = "info", ret)]
pub unsafe extern "C" fn rust_gc_suspend_ee() -> DiagnosticResult {
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
// Only the outer boundary is instrumented: span callbacks run before suspend
// and after restart. The stopped operation itself must not call the subscriber.
#[unsafe(no_mangle)]
#[tracing::instrument(level = "info", ret)]
pub unsafe extern "C" fn rust_gc_suspend_with_handle(
    handle: ObjectHandle,
    fail_operation: u32,
) -> DiagnosticResult {
    run_diagnostic(|| {
        // SAFETY: the caller owns the live store handle and cannot change/free it
        // during this synchronous call. Lifecycle invokes this only while stopped.
        unsafe { check_diagnostic_handle(handle) }?;
        if fail_operation == 0 { Ok(()) } else { Err(()) }
    })
}

/// Inspect one Node while suspended, returning only copied scalar observations.
///
/// # Safety
/// The suspension preconditions of `rust_gc_suspend_ee` apply. Both handles
/// must be live normal handles from this ZeroGC, exclusively owned and unchanged
/// through return. `witness` must be the controlled fixture's live,
/// noncollectible Node MethodTable, with its initialized descriptor and prefix
/// readable for the entire call. The fixture has already created the objects,
/// retains the type and objects until return, and excludes native writers.
/// A non-null live slot must contain an initialized CLR object; arbitrary heap
/// corruption and fabricated pointers or handles are outside this contract.
#[unsafe(no_mangle)]
#[tracing::instrument(level = "info", skip_all, ret)]
pub unsafe extern "C" fn rust_gc_inspect_node(
    source: ObjectHandle,
    candidate: ObjectHandle,
    witness: *const u8,
    expected: u32,
) -> InspectionResult {
    let ranges = match crate::gc_heap::snapshot_object_ranges() {
        Ok(ranges) => ranges,
        Err(()) => {
            return InspectionResult {
                lifecycle: DiagnosticResult {
                    status: Status::OperationFailed,
                    restart_attempts: 0,
                },
                observation: Observation::empty(InspectionStatus::BudgetExceeded),
            };
        }
    };
    let mut observation = Observation::empty(InspectionStatus::NotRun);
    let lifecycle = run_diagnostic(|| {
        // SAFETY: the FFI caller owns the handles and prevents slot changes;
        // this callback runs only after a successful suspension.
        let source = unsafe { resolve_diagnostic_handle(source) }.map_err(|_| {
            observation.status = InspectionStatus::InvalidObject;
        })?;
        // SAFETY: same handle ownership and stopped-window contract as above.
        let candidate = unsafe { resolve_diagnostic_handle(candidate) }.map_err(|_| {
            observation.status = InspectionStatus::InvalidTarget;
        })?;
        // SAFETY: the FFI contract establishes the witness and live object
        // provenance. The pre-suspend snapshot bounds all supported reads.
        observation = unsafe {
            inspect_node(
                &ranges,
                source.as_ptr().cast(),
                candidate.as_ptr().cast(),
                witness,
                expected,
            )
        };
        match observation.status {
            InspectionStatus::Reference | InspectionStatus::Null => Ok(()),
            _ => Err(()),
        }
    });
    InspectionResult {
        lifecycle,
        observation,
    }
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
    fn decoder_failure_runs_restart_and_restart_failure_takes_priority() {
        #[repr(C, align(8))]
        struct Metadata([u8; 32]);
        #[repr(C)]
        struct TestObject {
            header: u64,
            table: *const u8,
            next: *const u8,
        }

        let mut metadata = Metadata([0; 32]); // count zero is corrupt
        metadata.0[24..28].copy_from_slice(&0x0100_0000_u32.to_le_bytes());
        metadata.0[28..32].copy_from_slice(&24_u32.to_le_bytes());
        // SAFETY: offset 24 lies within the initialized metadata object.
        let witness = unsafe { metadata.0.as_ptr().add(24) };
        let object = TestObject {
            header: 0,
            table: witness,
            next: std::ptr::null(),
        };
        let start = std::ptr::from_ref(&object).addr();
        let ranges = [(start, start + size_of::<TestObject>())];
        let source = std::ptr::from_ref(&object).cast::<u8>().wrapping_add(8);
        for restart_fails in [false, true] {
            let lifecycle = Lifecycle::new();
            let attempts = Cell::new(0);
            let detail = Cell::new(InspectionStatus::NotRun);
            let status = lifecycle.run(
                || Ok(()),
                || {
                    assert_eq!(
                        lifecycle.run(|| panic!("nested suspend"), || Ok(()), || Ok(())),
                        Status::Rejected
                    );
                    // SAFETY: all bytes and pointers belong to initialized test
                    // objects, retained without writers for the duration.
                    let observation = unsafe { inspect_node(&ranges, source, source, witness, 0) };
                    detail.set(observation.status);
                    Err(())
                },
                || {
                    attempts.set(attempts.get() + 1);
                    if restart_fails { Err(()) } else { Ok(()) }
                },
            );
            assert_eq!(detail.get(), InspectionStatus::CorruptDescriptor);
            assert_eq!(attempts.get(), 1);
            assert_eq!(
                status,
                if restart_fails {
                    Status::RestartFailed
                } else {
                    Status::OperationFailed
                }
            );
            if restart_fails {
                assert_eq!(
                    lifecycle.run(|| Ok(()), || Ok(()), || Ok(())),
                    Status::Rejected
                );
            }
        }
    }

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
