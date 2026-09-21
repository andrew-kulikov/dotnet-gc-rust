use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};

use crate::handle_store::ObjectHandle;
use crate::object::Object;

#[unsafe(no_mangle)]
pub extern "C" fn rust_gc_handle_manager_initialize() -> bool {
    println!("rust_gc_handle_manager_initialize() called");
    true
}

/// Atomically replace the object in `handle` when it equals `comparand_object`.
///
/// The object previously stored in the handle is returned whether or not the
/// exchange succeeds. `SeqCst` matches the full ordering provided by
/// CoreCLR's native `Interlocked::CompareExchangePointer` implementation.
///
/// # Safety
///
/// `handle` must point to a live, pointer-aligned object slot created by the
/// handle store. No non-atomic access to the slot may race with this operation.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_gc_handle_manager_interlocked_compare_exchange_object_in_handle(
    handle: ObjectHandle,
    object: Object,
    comparand_object: Object,
) -> Object {
    println!(
        "rust_gc_handle_manager_interlocked_compare_exchange_object_in_handle(handle: {handle:p}, object: {object:p}, comparand_object: {comparand_object:p})"
    );
    // Object is transparent over `*mut c_void`, and handle records are aligned
    // for an Object. AtomicPtr::from_ptr is the standard atomic view over an
    // existing pointer slot; the caller upholds its lifetime and race rules.
    // ZeroGC has no collection-time handle metadata to update; a collecting GC
    // will need to add its handle write barrier alongside this operation.
    let atomic = unsafe { AtomicPtr::<c_void>::from_ptr(handle.cast::<*mut c_void>()) };
    let previous = atomic
        .compare_exchange(
            comparand_object.as_ptr(),
            object.as_ptr(),
            Ordering::SeqCst,
            Ordering::SeqCst,
        )
        .unwrap_or_else(|current| current);

    Object::from_ptr(previous)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_gc_handle_manager_store_object_in_handle(
    handle: ObjectHandle,
    object: Object,
) {
    println!(
        "rust_gc_handle_manager_store_object_in_handle(handle: {handle:p}, object: {object:p})"
    );

    // CoreCLR publishes handle values with release semantics so writes that
    // initialized the object are visible before another thread observes the
    // handle. ZeroGC never collects, so it has no handle write barrier or
    // collection metadata to update here.
    let atomic = unsafe { AtomicPtr::<c_void>::from_ptr(handle.cast::<*mut c_void>()) };
    atomic.store(object.as_ptr(), Ordering::Release);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handle_store::{destroy_test_handle, rust_gc_handle_store_create_handle_of_type};
    use std::ptr;

    fn object_for(storage: &mut u8) -> Object {
        Object::from_ptr(ptr::from_mut(storage).cast::<c_void>())
    }

    #[test]
    fn handle_manager_initialize_succeeds() {
        assert!(rust_gc_handle_manager_initialize());
    }

    #[test]
    fn compare_exchange_replaces_matching_object_and_returns_previous_object() {
        let mut original_storage = 0_u8;
        let mut replacement_storage = 0_u8;
        let original = object_for(&mut original_storage);
        let replacement = object_for(&mut replacement_storage);
        let handle = rust_gc_handle_store_create_handle_of_type(original, 2);

        let previous = unsafe {
            rust_gc_handle_manager_interlocked_compare_exchange_object_in_handle(
                handle,
                replacement,
                original,
            )
        };

        assert_eq!(previous, original);
        assert_eq!(unsafe { *handle }, replacement);
        unsafe { destroy_test_handle(handle) };
    }

    #[test]
    fn compare_exchange_preserves_nonmatching_object_and_returns_it() {
        let mut original_storage = 0_u8;
        let mut replacement_storage = 0_u8;
        let mut comparand_storage = 0_u8;
        let original = object_for(&mut original_storage);
        let replacement = object_for(&mut replacement_storage);
        let nonmatching_comparand = object_for(&mut comparand_storage);
        let handle = rust_gc_handle_store_create_handle_of_type(original, 2);

        let previous = unsafe {
            rust_gc_handle_manager_interlocked_compare_exchange_object_in_handle(
                handle,
                replacement,
                nonmatching_comparand,
            )
        };

        assert_eq!(previous, original);
        assert_eq!(unsafe { *handle }, original);
        unsafe { destroy_test_handle(handle) };
    }

    #[test]
    fn compare_exchange_supports_null_object_values() {
        let mut original_storage = 0_u8;
        let original = object_for(&mut original_storage);
        let null = Object::from_ptr(ptr::null_mut());
        let handle = rust_gc_handle_store_create_handle_of_type(original, 2);

        let previous = unsafe {
            rust_gc_handle_manager_interlocked_compare_exchange_object_in_handle(
                handle, null, original,
            )
        };

        assert_eq!(previous, original);
        assert!(unsafe { *handle }.is_null());
        unsafe { destroy_test_handle(handle) };
    }

    #[test]
    fn store_object_in_handle_replaces_the_object() {
        let mut original_storage = 0_u8;
        let mut replacement_storage = 0_u8;
        let original = object_for(&mut original_storage);
        let replacement = object_for(&mut replacement_storage);
        let handle = rust_gc_handle_store_create_handle_of_type(original, 2);

        unsafe { rust_gc_handle_manager_store_object_in_handle(handle, replacement) };

        assert_eq!(unsafe { *handle }, replacement);
        unsafe { destroy_test_handle(handle) };
    }
}
