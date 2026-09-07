use crate::alloc::{GcAllocContext, GcAllocFlags};
use crate::object::Object;
use std::alloc::{Layout, alloc_zeroed};

// Windows x64 CoreCLR: ObjHeader is a 4-byte alignment pad followed by the
// 4-byte sync-block value. Object* points just past it, at the method table.
const OBJECT_ALIGNMENT: usize = 8;
const OBJECT_HEADER_SIZE: usize = 8;
const MIN_OBJECT_SIZE: usize = 24;

type HResult = u32;

const S_OK: HResult = 0;

#[unsafe(no_mangle)]
pub extern "C" fn rust_gc_loader_probe() -> HResult {
    println!("rust_gc_loader_probe() called");
    S_OK
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_gc_initialize() -> HResult {
    println!("rust_gc_initialize() called");
    S_OK
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
    println!("rust_gc_alloc(context: {acontext:p}, size: {size}, flags: {flags:?}) called");

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

    #[test]
    fn loader_probe_succeeds() {
        assert_eq!(rust_gc_loader_probe(), S_OK);
    }

    #[test]
    fn loader_initialize_dummy_ok() {
        assert_eq!(rust_gc_initialize(), S_OK);
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
