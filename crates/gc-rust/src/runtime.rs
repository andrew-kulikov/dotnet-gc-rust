use std::ffi::c_void;
use std::sync::OnceLock;

pub type HResult = u32;

pub const S_OK: HResult = 0;
pub const E_POINTER: HResult = 0x8000_4003;
pub const E_UNEXPECTED: HResult = 0x8000_FFFF;

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
    unsafe extern "C" fn(context: *mut c_void, parameters: *mut WriteBarrierParameters) -> HResult;

/// C ABI callback table through which Rust asks CoreCLR to perform EE work.
///
/// The context and callbacks are created by the C++ shim. Add callbacks only
/// when the collector needs them; unneeded `IGCToCLR` methods intentionally do
/// not occupy ABI slots here. Future candidates include `SuspendEE`,
/// `RestartEE`, `GcScanRoots`, `GcEnumAllocContexts`, finalization notification,
/// configuration access, diagnostics, and event reporting.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct IGcToClr {
    context: *mut c_void,
    stomp_write_barrier: Option<StompWriteBarrierFunc>,
}

#[derive(Clone, Copy)]
pub(crate) struct GcToClr {
    context: *mut c_void,
    stomp_write_barrier: StompWriteBarrierFunc,
}

// CoreCLR owns `context` for the process lifetime and its IGCToCLR methods are
// invoked from GC/runtime threads according to each method's contract. The
// callback table is immutable after installation.
unsafe impl Send for GcToClr {}
// SAFETY: see the process-lifetime and immutability guarantees above.
unsafe impl Sync for GcToClr {}

static GC_TO_CLR: OnceLock<GcToClr> = OnceLock::new();

impl GcToClr {
    /// Asks the C++ shim to validate and install write-barrier state.
    ///
    /// # Safety
    ///
    /// The context must have been supplied by the shim and must remain live for
    /// the duration of the process.
    pub unsafe fn stomp_write_barrier(&self, parameters: &mut WriteBarrierParameters) -> HResult {
        // SAFETY: the caller guarantees the callback and context came from the
        // shim; `parameters` remains exclusively borrowed until it returns.
        unsafe { (self.stomp_write_barrier)(self.context, parameters) }
    }
}

/// Copies and installs CoreCLR's process-lifetime callback table exactly once.
///
/// Reinstalling the identical table is harmless; attempting to replace it with
/// a different runtime object or callback is rejected.
///
/// # Safety
///
/// `source` must either be null or point to a readable `IGcToClr` supplied by
/// the native shim. Its context must remain valid for the process lifetime.
pub(crate) unsafe fn install_gc_to_clr(source: *const IGcToClr) -> Result<(), HResult> {
    // SAFETY: this function has the same source-pointer contract as the helper.
    unsafe { install_gc_to_clr_in(&GC_TO_CLR, source) }.map(|_| ())
}

/// Returns the process-wide callback table after successful installation.
pub(crate) fn gc_to_clr() -> Option<&'static GcToClr> {
    GC_TO_CLR.get()
}

unsafe fn install_gc_to_clr_in(
    destination: &OnceLock<GcToClr>,
    source: *const IGcToClr,
) -> Result<&GcToClr, HResult> {
    // SAFETY: the caller guarantees that any non-null source is readable.
    let Some(source) = (unsafe { source.as_ref() }) else {
        return Err(E_POINTER);
    };
    let Some(stomp_write_barrier) = source.stomp_write_barrier else {
        return Err(E_POINTER);
    };
    if source.context.is_null() {
        return Err(E_POINTER);
    }

    let candidate = GcToClr {
        context: source.context,
        stomp_write_barrier,
    };
    if destination.set(candidate).is_ok() {
        return Ok(destination.get().expect("OnceLock was just initialized"));
    }

    let installed = destination.get().expect("OnceLock rejected initialization");
    if installed.context == candidate.context
        && std::ptr::fn_addr_eq(installed.stomp_write_barrier, candidate.stomp_write_barrier)
    {
        Ok(installed)
    } else {
        Err(E_UNEXPECTED)
    }
}

// Keep these Windows x64 checks synchronized with the bridge declarations in
// rust_gc.h. These are C ABI layouts rather than C++ class/vtable layouts.
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
const _: () = {
    assert!(size_of::<bool>() == 1);
    assert!(size_of::<WriteBarrierOp>() == 4);
    assert!(size_of::<WriteBarrierParameters>() == 80);
    assert!(align_of::<WriteBarrierParameters>() == 8);
    assert!(std::mem::offset_of!(WriteBarrierParameters, operation) == 0);
    assert!(std::mem::offset_of!(WriteBarrierParameters, is_runtime_suspended) == 4);
    assert!(std::mem::offset_of!(WriteBarrierParameters, requires_upper_bounds_check) == 5);
    assert!(std::mem::offset_of!(WriteBarrierParameters, card_table) == 8);
    assert!(std::mem::offset_of!(WriteBarrierParameters, card_bundle_table) == 16);
    assert!(std::mem::offset_of!(WriteBarrierParameters, lowest_address) == 24);
    assert!(std::mem::offset_of!(WriteBarrierParameters, highest_address) == 32);
    assert!(std::mem::offset_of!(WriteBarrierParameters, ephemeral_low) == 40);
    assert!(std::mem::offset_of!(WriteBarrierParameters, ephemeral_high) == 48);
    assert!(std::mem::offset_of!(WriteBarrierParameters, write_watch_table) == 56);
    assert!(std::mem::offset_of!(WriteBarrierParameters, region_to_generation_table) == 64);
    assert!(std::mem::offset_of!(WriteBarrierParameters, region_shr) == 72);
    assert!(std::mem::offset_of!(WriteBarrierParameters, region_use_bitwise_write_barrier) == 73);
    assert!(size_of::<IGcToClr>() == 16);
    assert!(align_of::<IGcToClr>() == 8);
    assert!(std::mem::offset_of!(IGcToClr, context) == 0);
    assert!(std::mem::offset_of!(IGcToClr, stomp_write_barrier) == 8);
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    static CALLED: AtomicBool = AtomicBool::new(false);
    static OPERATION: AtomicUsize = AtomicUsize::new(usize::MAX);

    unsafe extern "C" fn record_stomp(
        _context: *mut c_void,
        parameters: *mut WriteBarrierParameters,
    ) -> HResult {
        // SAFETY: the test passes a live initialized parameter record.
        let parameters = unsafe { &*parameters };
        OPERATION.store(parameters.operation as usize, Ordering::Relaxed);
        CALLED.store(true, Ordering::Release);
        S_OK
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
    fn dispatches_stomp_write_barrier_through_c_callback() {
        CALLED.store(false, Ordering::Relaxed);
        OPERATION.store(usize::MAX, Ordering::Relaxed);

        let source = IGcToClr {
            context: std::ptr::dangling_mut::<u8>().cast(),
            stomp_write_barrier: Some(record_stomp),
        };
        let destination = OnceLock::new();

        // SAFETY: the source remains readable for installation, and the
        // callback does not dereference its non-null test context.
        let gc_to_clr = unsafe { install_gc_to_clr_in(&destination, &raw const source) }.unwrap();
        let result = unsafe { gc_to_clr.stomp_write_barrier(&mut parameters()) };

        assert_eq!(result, S_OK);
        assert!(CALLED.load(Ordering::Acquire));
        assert_eq!(
            OPERATION.load(Ordering::Relaxed),
            WriteBarrierOp::Initialize as usize
        );
    }

    #[test]
    fn rejects_missing_callback_or_context() {
        let destination = OnceLock::new();
        let missing_callback = IGcToClr {
            context: std::ptr::dangling_mut::<u8>().cast(),
            stomp_write_barrier: None,
        };
        let missing_context = IGcToClr {
            context: std::ptr::null_mut(),
            stomp_write_barrier: Some(record_stomp),
        };

        // SAFETY: both source records remain readable during installation.
        assert!(matches!(
            unsafe { install_gc_to_clr_in(&destination, &raw const missing_callback) },
            Err(E_POINTER)
        ));
        // SAFETY: both source records remain readable during installation.
        assert!(matches!(
            unsafe { install_gc_to_clr_in(&destination, &raw const missing_context) },
            Err(E_POINTER)
        ));
    }

    #[test]
    fn accepts_identical_reinstall_and_rejects_replacement() {
        let destination = OnceLock::new();
        let source = IGcToClr {
            context: std::ptr::dangling_mut::<u8>().cast(),
            stomp_write_barrier: Some(record_stomp),
        };
        let replacement = IGcToClr {
            context: std::ptr::dangling_mut::<u16>().cast(),
            stomp_write_barrier: Some(record_stomp),
        };

        // SAFETY: all source records remain readable during installation.
        assert!(unsafe { install_gc_to_clr_in(&destination, &raw const source) }.is_ok());
        // SAFETY: all source records remain readable during installation.
        assert!(unsafe { install_gc_to_clr_in(&destination, &raw const source) }.is_ok());
        // SAFETY: all source records remain readable during installation.
        assert!(matches!(
            unsafe { install_gc_to_clr_in(&destination, &raw const replacement) },
            Err(E_UNEXPECTED)
        ));
    }
}
