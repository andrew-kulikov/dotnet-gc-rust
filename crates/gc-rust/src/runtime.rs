use std::ffi::c_void;
use std::sync::OnceLock;

pub type HResult = u32;

pub const S_OK: HResult = 0;

/// Reasons for which the runtime might suspend the execution environment (EE).
/// See `SUSPEND_REASON` in the pinned `gcinterface.h`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuspendReason {
    SuspendForGc = 1,
    SuspendForGcPrep = 6,
}

/// CoreCLR operation used to configure or update its managed write barrier.
///
/// Keep the discriminants synchronized with `WriteBarrierOp` in the pinned
/// `gcinterface.h`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WriteBarrierOp {
    StompResize = 0,
    StompEphemeral = 1,
    Initialize = 2,
    SwitchToWriteWatch = 3,
    SwitchToNonWriteWatch = 4,
}

/// CoreCLR's native write-barrier request, mirrored for the pinned Windows x64
/// ABI.
///
/// The C++ shim verifies this layout against CoreCLR's
/// `WriteBarrierParameters` and forwards the same record without copying it.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct WriteBarrierParameters {
    pub operation: WriteBarrierOp,
    pub is_runtime_suspended: bool,
    pub requires_upper_bounds_check: bool,
    pub card_table: *mut u32,
    pub card_bundle_table: *mut u32,
    pub lowest_address: *mut u8,
    pub highest_address: *mut u8,
    pub ephemeral_low: *mut u8,
    pub ephemeral_high: *mut u8,
    pub write_watch_table: *mut u8,
    pub region_to_generation_table: *mut u8,
    pub region_shr: u8,
    pub region_use_bitwise_write_barrier: bool,
}

type StompWriteBarrierFunc =
    unsafe extern "C" fn(context: *mut c_void, parameters: *mut WriteBarrierParameters);
type SuspendEeFunc = unsafe extern "C" fn(context: *mut c_void, reason: SuspendReason);
type RestartEeFunc = unsafe extern "C" fn(context: *mut c_void, finished_gc: bool);

/// C ABI callback table through which Rust asks CoreCLR to perform EE work.
///
/// The pinned C++ shim supplies a live context and all callbacks. Rust copies
/// this table at initialization; there is no separate validated representation.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GcToClr {
    context: *mut c_void,
    stomp_write_barrier: StompWriteBarrierFunc,
    suspend_ee: SuspendEeFunc,
    restart_ee: RestartEeFunc,
}

// CoreCLR owns `context` for the process lifetime and its IGCToCLR methods are
// invoked from GC/runtime threads according to each method's contract. The
// callback table is immutable after installation.
unsafe impl Send for GcToClr {}
// SAFETY: see the process-lifetime and immutability guarantees above.
unsafe impl Sync for GcToClr {}

static GC_TO_CLR: OnceLock<GcToClr> = OnceLock::new();

impl GcToClr {
    /// Asks CoreCLR to install write-barrier state.
    ///
    /// # Safety
    ///
    /// The callbacks and context must be live, and `parameters` must describe
    /// valid barrier state with the suspension required by CoreCLR.
    pub unsafe fn stomp_write_barrier(&self, parameters: &mut WriteBarrierParameters) {
        // SAFETY: the caller guarantees valid runtime state and parameters.
        unsafe { (self.stomp_write_barrier)(self.context, parameters) }
    }

    /// Suspends the EE on a thread allowed to initiate GC suspension.
    ///
    /// # Safety
    ///
    /// The runtime must be initialized, with no suspension already owned by
    /// this caller. The caller must restart the EE on the same thread.
    pub unsafe fn suspend_ee(&self, reason: SuspendReason) {
        // SAFETY: the caller guarantees the runtime's suspension preconditions.
        unsafe { (self.suspend_ee)(self.context, reason) }
    }

    /// Resumes the EE after a matching suspension.
    ///
    /// # Safety
    ///
    /// The caller must own the suspension on this thread. `finished_gc` must
    /// indicate whether a collection actually completed.
    pub unsafe fn restart_ee(&self, finished_gc: bool) {
        // SAFETY: the caller owns the suspension and supplies its completion state.
        unsafe { (self.restart_ee)(self.context, finished_gc) }
    }
}

/// Copies the shim's temporary table during heap initialization.
pub(crate) fn install_gc_to_clr(source: &GcToClr) -> &'static GcToClr {
    GC_TO_CLR.get_or_init(|| *source)
}

pub(crate) fn gc_to_clr() -> &'static GcToClr {
    GC_TO_CLR.get().expect("GC callbacks are not initialized")
}

// Basic Windows x64 layout checks, mirrored in the pinned C++ shim.
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
const _: () = {
    assert!(size_of::<bool>() == 1);
    assert!(size_of::<WriteBarrierOp>() == 4);
    assert!(size_of::<WriteBarrierParameters>() == 80);
    assert!(align_of::<WriteBarrierParameters>() == 8);
    assert!(size_of::<SuspendReason>() == 4);
    assert!(size_of::<GcToClr>() == 32);
    assert!(align_of::<GcToClr>() == 8);
};

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Calls {
        operation: Option<WriteBarrierOp>,
        reason: Option<SuspendReason>,
        finished_gc: Option<bool>,
    }

    unsafe extern "C" fn record_stomp(
        context: *mut c_void,
        parameters: *mut WriteBarrierParameters,
    ) {
        // SAFETY: the test supplies exclusive access to both records.
        unsafe { (*context.cast::<Calls>()).operation = Some((*parameters).operation) };
    }

    unsafe extern "C" fn record_suspend(context: *mut c_void, reason: SuspendReason) {
        // SAFETY: the test supplies exclusive access to its context.
        unsafe { (*context.cast::<Calls>()).reason = Some(reason) };
    }

    unsafe extern "C" fn record_restart(context: *mut c_void, finished_gc: bool) {
        // SAFETY: the test supplies exclusive access to its context.
        unsafe { (*context.cast::<Calls>()).finished_gc = Some(finished_gc) };
    }

    fn parameters() -> WriteBarrierParameters {
        WriteBarrierParameters {
            operation: WriteBarrierOp::Initialize,
            is_runtime_suspended: true,
            requires_upper_bounds_check: false,
            card_table: std::ptr::null_mut(),
            card_bundle_table: std::ptr::null_mut(),
            lowest_address: std::ptr::null_mut(),
            highest_address: std::ptr::null_mut(),
            ephemeral_low: std::ptr::null_mut(),
            ephemeral_high: std::ptr::null_mut(),
            write_watch_table: std::ptr::null_mut(),
            region_to_generation_table: std::ptr::null_mut(),
            region_shr: 0,
            region_use_bitwise_write_barrier: false,
        }
    }

    #[test]
    fn dispatches_callbacks_with_context_and_arguments() {
        let mut calls = Calls::default();
        let gc_to_clr = GcToClr {
            context: (&raw mut calls).cast(),
            stomp_write_barrier: record_stomp,
            suspend_ee: record_suspend,
            restart_ee: record_restart,
        };

        // SAFETY: these callbacks only record arguments in the live test context.
        unsafe {
            gc_to_clr.stomp_write_barrier(&mut parameters());
            gc_to_clr.suspend_ee(SuspendReason::SuspendForGc);
            gc_to_clr.restart_ee(false);
        }

        assert_eq!(calls.operation, Some(WriteBarrierOp::Initialize));
        assert_eq!(calls.reason, Some(SuspendReason::SuspendForGc));
        assert_eq!(calls.finished_gc, Some(false));
    }
}
