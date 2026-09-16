use std::alloc::{Layout, alloc};
use std::ptr;

use crate::object::Object;

/// Address of a stable slot whose value is an [`Object`].
///
/// CoreCLR reads an `OBJECTHANDLE` by dereferencing this slot, so this cannot be
/// an index or an element that may move when a `Vec` grows.
pub(crate) type ObjectHandle = *mut Object;

#[repr(C)]
struct HandleRecord {
    // Must remain the first field: ObjectHandle points at this exact slot.
    object: Object,
    handle_type: u32,
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_gc_handle_store_create_handle_of_type(
    object: Object,
    handle_type: u32,
) -> ObjectHandle {
    println!("rust_gc_handle_store_create_handle_of_type: {object:?} with type: {handle_type}");

    let layout = Layout::new::<HandleRecord>();

    // SAFETY: `layout` describes a non-zero-sized `HandleRecord`. On success,
    // `alloc` returns suitably aligned storage owned by this Rust library.
    let record = unsafe { alloc(layout).cast::<HandleRecord>() };
    if record.is_null() {
        return ptr::null_mut();
    }

    // SAFETY: `record` points to writable storage for one `HandleRecord` and
    // has not been initialized yet.
    unsafe {
        record.write(HandleRecord {
            object,
            handle_type,
        });
    }

    // `repr(C)` guarantees that the first field is at offset zero. Therefore
    // this is both the HandleRecord address and the address of its Object slot.
    record.cast::<Object>()
}

#[cfg(test)]
pub(crate) unsafe fn destroy_test_handle(handle: ObjectHandle) {
    unsafe {
        std::alloc::dealloc(handle.cast::<u8>(), Layout::new::<HandleRecord>());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::c_void;

    #[test]
    fn created_handle_is_a_stable_object_slot() {
        let mut object_storage = 0_u8;
        let object = Object::from_ptr(ptr::from_mut(&mut object_storage).cast::<c_void>());

        let handle = rust_gc_handle_store_create_handle_of_type(object, 2);

        assert!(!handle.is_null());
        // SAFETY: the function returned a live slot initialized with `object`.
        assert_eq!(unsafe { *handle }, object);

        let record = handle.cast::<HandleRecord>();
        // SAFETY: `handle` is the address returned for this HandleRecord and
        // the record remains allocated for the duration of the test.
        assert_eq!(unsafe { (*record).handle_type }, 2);

        // There is intentionally no public destroy operation yet. Tests can
        // release private records because CoreCLR never observes these handles.
        unsafe {
            destroy_test_handle(handle);
        }
    }
}
