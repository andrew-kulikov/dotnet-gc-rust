use crate::alloc::{GcAllocContext, GcAllocFlags};
use crate::object::Object;
use crate::runtime::{
    HResult, IGcToClr, S_OK, WriteBarrierOp, WriteBarrierParameters, gc_to_clr, install_gc_to_clr,
};
use std::alloc::{Layout, alloc_zeroed};
use std::ptr;

// Windows x64 CoreCLR: ObjHeader is a 4-byte alignment pad followed by the
// 4-byte sync-block value. Object* points just past it, at the method table.
const OBJECT_ALIGNMENT: usize = 8;
const OBJECT_HEADER_SIZE: usize = 8;
const MIN_OBJECT_SIZE: usize = 24;

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
    println!("rust_gc_alloc(context: {acontext:p}, size: {size}, flags: {flags:?})");

    let Some(layout) = object_layout(size) else {
        return Object::from_ptr(std::ptr::null_mut());
    };

    // SAFETY: layout has a non-zero size and valid alignment. Zeroing initializes
    // both ObjHeader words, the method-table slot, and all object fields.
    let base = unsafe { alloc_zeroed(layout) };
    if base.is_null() {
        return Object::from_ptr(std::ptr::null_mut());
    }

    // SAFETY: layout reserves at least MIN_OBJECT_SIZE bytes, so this offset
    // stays within the allocation and preserves 8-byte alignment.
    let object = unsafe { base.add(OBJECT_HEADER_SIZE) };
    println!("rust_gc_alloc() returning object at {object:p}");
    Object::from_ptr(object.cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_gc_set_finalization_run(obj: Object) {
    println!("rust_gc_set_finalization_run(obj: {obj:p})");
    // TODO: Implement finalization bit flipping logic. Currently, this is a no-op.
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_gc_shutdown() {
    println!("rust_gc_shutdown()");
    // CoreCLR just flushes GC log in this method
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
}
