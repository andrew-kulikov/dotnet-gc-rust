use crate::alloc::{GcAllocContext, GcAllocFlags};
use crate::object::Object;
use crate::runtime::{
    HResult, IGcToClr, S_OK, WriteBarrierOp, WriteBarrierParameters, gc_to_clr, install_gc_to_clr,
};
use std::alloc::{Layout, alloc_zeroed};
use std::ptr;
use std::sync::{LazyLock, Mutex, OnceLock};

// Windows x64 CoreCLR: ObjHeader is a 4-byte alignment pad followed by the
// 4-byte sync-block value. Object* points just past it, at the method table.
const OBJECT_ALIGNMENT: usize = 8;
const OBJECT_HEADER_SIZE: usize = 8;
const MIN_OBJECT_SIZE: usize = 24;

const DEFAULT_LIMIT_BYTES: usize = 64 * 1024 * 1024;
static LIMIT_BYTES: OnceLock<usize> = OnceLock::new();
static LEDGER: LazyLock<Mutex<Ledger>> = LazyLock::new(|| Mutex::new(Ledger::default()));

#[derive(Default)]
struct Ledger {
    requests: usize,
    successful: usize,
    requested_bytes: usize,
    owned_bytes: usize,
    // Diagnostic metadata only. A collection must never use this as liveness data.
    ranges: Vec<(usize, usize)>,
}

impl Ledger {
    fn allocate(&mut self, size: usize, limit: usize) -> Object {
        self.requests = self.requests.saturating_add(1);
        self.requested_bytes = self.requested_bytes.saturating_add(size);
        let Some(layout) = object_layout(size) else {
            eprintln!("ZeroGC: invalid allocation size: {size}");
            return Object::from_ptr(ptr::null_mut());
        };
        let Some(next_owned) = self.owned_bytes.checked_add(layout.size()) else {
            eprintln!("ZeroGC: allocation accounting overflow");
            return Object::from_ptr(ptr::null_mut());
        };
        if next_owned > limit {
            eprintln!(
                "ZeroGC: allocation limit exceeded: owned={} requested={} limit={limit}",
                self.owned_bytes,
                layout.size()
            );
            return Object::from_ptr(ptr::null_mut());
        }
        if self.ranges.try_reserve(1).is_err() {
            eprintln!("ZeroGC: allocation registry exhausted");
            return Object::from_ptr(ptr::null_mut());
        }

        // SAFETY: layout is nonzero and aligned; the returned block is retained
        // for the process lifetime and never handed to a second allocation.
        let base = unsafe { alloc_zeroed(layout) };
        if base.is_null() {
            eprintln!("ZeroGC: native allocation failed");
            return Object::from_ptr(ptr::null_mut());
        }
        let start = base as usize;
        let Some(end) = start.checked_add(layout.size()) else {
            // SAFETY: base was just allocated with this layout.
            unsafe { std::alloc::dealloc(base, layout) };
            eprintln!("ZeroGC: native address overflow");
            return Object::from_ptr(ptr::null_mut());
        };
        self.ranges.push((start, end));
        self.successful += 1;
        self.owned_bytes = next_owned;

        // SAFETY: every valid object layout contains its 8-byte header.
        Object::from_ptr(unsafe { base.add(OBJECT_HEADER_SIZE) }.cast())
    }

    fn report(&mut self, limit: usize) {
        self.ranges.sort_unstable();
        let disjoint = self.ranges.windows(2).all(|pair| pair[0].1 <= pair[1].0);
        let gaps = self
            .ranges
            .windows(2)
            .filter(|pair| pair[0].1 < pair[1].0)
            .count();
        let registry_bytes = self.ranges.capacity() * size_of::<(usize, usize)>();
        eprintln!(
            "ZeroGC counters: requests={} successful={} requested_bytes={} owned_bytes={} limit={} ranges={} registry_bytes={} disjoint={} gaps={}",
            self.requests,
            self.successful,
            self.requested_bytes,
            self.owned_bytes,
            limit,
            self.ranges.len(),
            registry_bytes,
            disjoint,
            gaps
        );
        for (start, end) in self.ranges.iter().take(2) {
            eprintln!("ZeroGC native range: [{start:#x}, {end:#x})");
        }
        eprintln!("ZeroGC heap walk: unavailable; objects are separate native allocations");
    }
}

// With the empty heap and ephemeral ranges below, barrier helpers never index
// this placeholder. CoreCLR still requires a non-null card-table address when
// its write barrier is initialized.
static mut INERT_CARD_TABLE: u32 = 0;

#[unsafe(no_mangle)]
pub extern "C" fn rust_gc_loader_probe() -> HResult {
    println!("rust_gc_loader_probe()");
    S_OK
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_gc_get_max_generation() -> u32 {
    // For now only the ephemeral generation (generation 0) exists.
    // TODO: Change in case more generations are added.
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_gc_collection_count(_generation: i32, _get_bgc_fgc_coutn: i32) -> i32 {
    // Zero GC does not perform any collections, so the count is always zero (for now).
    // TODO: Implement actual collection counting when more generations are added.
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_gc_is_gc_in_progress_helper(_b_consider_gc_start: bool) -> bool {
    // Zero GC never performs any collections, so GC is never in progress.
    // TODO: Implement actual GC progress tracking when more generations are added.
    false
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_gc_initialize(gc_to_clr_source: *const IGcToClr) -> HResult {
    println!("rust_gc_initialize()");

    let limit = match std::env::var("ZERO_GC_LIMIT_BYTES") {
        Ok(value) => match value.parse::<usize>() {
            Ok(limit) if limit > 0 => limit,
            _ => {
                eprintln!("ZeroGC: ZERO_GC_LIMIT_BYTES must be a positive integer");
                DEFAULT_LIMIT_BYTES
            }
        },
        Err(std::env::VarError::NotPresent) => DEFAULT_LIMIT_BYTES,
        Err(std::env::VarError::NotUnicode(_)) => {
            eprintln!("ZeroGC: ZERO_GC_LIMIT_BYTES must be a positive integer");
            DEFAULT_LIMIT_BYTES
        }
    };
    let _ = LIMIT_BYTES.set(limit);

    // SAFETY: CoreCLR supplies a readable callback table whose context has
    // process lifetime. Installation copies the table rather than retaining
    // this pointer to the shim's temporary record.
    if let Err(error) = unsafe { install_gc_to_clr(gc_to_clr_source) } {
        return error;
    }
    let gc_to_clr = gc_to_clr().expect("callback table was just installed");

    // Study-only ZeroGC: retain all objects forever and disable card marking.
    // An empty heap range makes checked stores and bulk reference copies skip
    // the card table. An empty ephemeral range also makes unchecked object
    // stores skip it (the x64 pre-grow barrier checks only ephemeral_low).
    // Empty ephemeral bounds ALONE are insufficient: bulk copies index the
    // card table based on the destination's heap membership, without checking
    // the source references against the ephemeral range.
    // These sentinel addresses are comparison values, never dereferenced.
    // Before adding collection or heap-membership-dependent features, replace
    // this setup with real heap bounds and correctly biased card/bundle tables.
    let lowest_address: *mut u8 = ptr::without_provenance_mut(usize::MAX);
    let highest_address: *mut u8 = ptr::without_provenance_mut(usize::MAX);
    let mut parameters = WriteBarrierParameters {
        operation: WriteBarrierOp::Initialize,
        is_runtime_suspended: true,
        requires_upper_bounds_check: false,
        card_table: &raw mut INERT_CARD_TABLE,
        card_bundle_table: &raw mut INERT_CARD_TABLE,
        lowest_address,
        highest_address,
        ephemeral_low: highest_address,
        ephemeral_high: highest_address,
        write_watch_table: ptr::null_mut(),
        region_to_generation_table: ptr::null_mut(),
        region_shr: 0,
        region_use_bitwise_write_barrier: false,
    };

    // SAFETY: CoreCLR supplied this callback object and Initialize runs while
    // the runtime is suspended. Table storage has process lifetime; the empty
    // ranges prevent indexing it and their sentinel bounds are never accessed.
    unsafe { gc_to_clr.stomp_write_barrier(&mut parameters) }
}

/// Dummy Windows x64 allocator. Storage is zeroed and retained forever.
///
/// `size` already includes ObjHeader. CoreCLR installs the method table after
/// this returns. Allocation contexts, finalization, and heap classification
/// flags are not implemented; all objects use the same non-moving allocation.
#[unsafe(no_mangle)]
pub extern "C" fn rust_gc_alloc(
    acontext: *mut GcAllocContext,
    size: usize,
    flags: GcAllocFlags,
) -> Object {
    let _ = (acontext, flags);
    let mut ledger = LEDGER.lock().unwrap_or_else(|poison| poison.into_inner());
    ledger.allocate(size, *LIMIT_BYTES.get().unwrap_or(&DEFAULT_LIMIT_BYTES))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_gc_set_finalization_run(obj: Object) {
    println!("rust_gc_set_finalization_run(obj: {obj:p})");
    // TODO: Implement finalization bit flipping logic. Currently, this is a no-op.
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_gc_shutdown() {
    rust_gc_report();
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_gc_report() {
    let mut ledger = LEDGER.lock().unwrap_or_else(|poison| poison.into_inner());
    ledger.report(*LIMIT_BYTES.get().unwrap_or(&DEFAULT_LIMIT_BYTES));
}

fn object_layout(size: usize) -> Option<Layout> {
    if size < MIN_OBJECT_SIZE {
        return None;
    }
    // Layout rejects sizes that would overflow when rounded to alignment.
    Some(
        Layout::from_size_align(size, OBJECT_ALIGNMENT)
            .ok()?
            .pad_to_align(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::E_POINTER;

    #[test]
    fn loader_probe_succeeds() {
        assert_eq!(rust_gc_loader_probe(), S_OK);
    }

    #[test]
    fn loader_initialize_dummy_ok() {
        // The actual initialization path is covered with a fake IGCToCLR in
        // runtime.rs; a missing runtime callback is rejected deterministically.
        assert_eq!(unsafe { rust_gc_initialize(ptr::null_mut()) }, E_POINTER);
    }

    #[test]
    fn allocation_has_zeroed_header_and_aligned_writable_body() {
        for size in [24, 25, 32, 85_001] {
            let object = rust_gc_alloc(std::ptr::null_mut(), size, GcAllocFlags::NONE);
            assert!(!object.is_null());
            let address = object.as_ptr().cast::<u8>();
            assert_eq!(address as usize % OBJECT_ALIGNMENT, 0);

            // SAFETY: this test owns the live allocation. The header precedes
            // Object* by 8 bytes, and the entire requested size is readable.
            unsafe {
                let base = address.sub(OBJECT_HEADER_SIZE);
                let bytes = std::slice::from_raw_parts(base, size);
                assert!(bytes.iter().all(|byte| *byte == 0));
                // Simulate the runtime installing a method table and writing
                // the last byte of the object's requested body.
                address.cast::<usize>().write(0x1234);
                address.add(size - OBJECT_HEADER_SIZE - 1).write(0xAB);
                assert_eq!(address.cast::<usize>().read(), 0x1234);
                assert_eq!(base.cast::<u64>().read(), 0);
                // Production deliberately retains storage; only tests free it.
                std::alloc::dealloc(base, object_layout(size).unwrap());
            }
        }
    }

    #[test]
    fn invalid_allocation_sizes_return_null() {
        for size in [0, MIN_OBJECT_SIZE - 1, isize::MAX as usize, usize::MAX] {
            assert!(rust_gc_alloc(std::ptr::null_mut(), size, GcAllocFlags::NONE).is_null());
        }
    }

    #[test]
    fn ledger_reconciles_success_rejection_and_disjoint_ranges() {
        let mut ledger = Ledger::default();
        let first = ledger.allocate(25, 64);
        let second = ledger.allocate(25, 64);
        assert!(!first.is_null() && !second.is_null());
        assert!(ledger.allocate(24, 64).is_null());
        assert!(ledger.allocate(usize::MAX, 64).is_null());
        assert_eq!(ledger.requests, 4);
        assert_eq!(ledger.successful, 2);
        assert_eq!(ledger.requested_bytes, usize::MAX);
        assert_eq!(ledger.owned_bytes, 64);
        assert_eq!(ledger.ranges.len(), 2);
        let (a, b) = (ledger.ranges[0], ledger.ranges[1]);
        assert!(a.1 <= b.0 || b.1 <= a.0);

        for object in [first, second] {
            // SAFETY: these allocations belong only to this test and use the
            // recorded object layout. Production never frees managed storage.
            unsafe {
                std::alloc::dealloc(
                    object.as_ptr().cast::<u8>().sub(OBJECT_HEADER_SIZE),
                    object_layout(25).unwrap(),
                );
            }
        }
    }

    #[test]
    fn ledger_rejects_accounting_overflow_without_changing_owned_bytes() {
        let mut ledger = Ledger {
            owned_bytes: usize::MAX - 8,
            ..Ledger::default()
        };
        assert!(ledger.allocate(24, usize::MAX).is_null());
        assert_eq!(ledger.requests, 1);
        assert_eq!(ledger.successful, 0);
        assert_eq!(ledger.owned_bytes, usize::MAX - 8);
    }
}
