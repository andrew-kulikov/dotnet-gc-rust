use std::alloc::{Layout, alloc};
use std::ffi::c_void;
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

use crate::object::Object;
use crate::util::log_trace;

/// Address of a stable slot whose value is an [`Object`].
///
/// CoreCLR reads an `OBJECTHANDLE` by dereferencing this slot, so this cannot be
/// an index or an element that may move when a `Vec` grows.
pub(crate) type ObjectHandle = *mut Object;

// Pinned CoreCLR gcinterface.h; also asserted in the C++ bridge.
const DEPENDENT_HANDLE: u32 = 6;

#[repr(C)]
struct HandleRecord {
    // Must remain the first field: ObjectHandle points at this exact slot.
    object: Object,
    handle_type: u32,
    secondary: AtomicPtr<c_void>,
}

/// Check one owned normal handle without exporting its borrowed object address.
///
/// # Safety
/// A non-null, aligned `handle` must be an untagged live handle from this store.
/// The caller must own the handle exclusively, keep it alive, and suspend the EE
/// for this call. No native thread may mutate the slot concurrently.
pub(crate) unsafe fn check_diagnostic_handle(handle: ObjectHandle) -> Result<(), ()> {
    if handle.is_null() || !handle.is_aligned() {
        return Err(());
    }
    // SAFETY: the caller supplies a live store record; Object is its first field.
    // GCHandleType.Normal maps to pinned CoreCLR's HNDTYPE_STRONG (2). The normal
    // token is untagged; GCHandle only sets the low bit for pinned handles.
    let record = unsafe { &*handle.cast::<HandleRecord>() };
    if record.handle_type != 2 || record.object.is_null() {
        return Err(());
    }
    // The opaque object value is borrowed only here, before the owner restarts.
    // No object memory is read, and only success/failure leaves this scope.
    Ok(())
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_gc_handle_store_create_handle_of_type(
    object: Object,
    handle_type: u32,
) -> ObjectHandle {
    log_trace!("rust_gc_handle_store_create_handle_of_type: {object:?} with type: {handle_type}");
    create_handle(object, handle_type, Object::from_ptr(ptr::null_mut()))
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_gc_handle_store_create_dependent_handle(
    primary: Object,
    secondary: Object,
) -> ObjectHandle {
    create_handle(primary, DEPENDENT_HANDLE, secondary)
}

fn create_handle(object: Object, handle_type: u32, secondary: Object) -> ObjectHandle {
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
            secondary: AtomicPtr::new(secondary.as_ptr()),
        });
    }

    // `repr(C)` guarantees that the first field is at offset zero. Therefore
    // this is both the HandleRecord address and the address of its Object slot.
    record.cast::<Object>()
}

/// # Safety
/// `handle` must be a live dependent record from this store, with no concurrent
/// destruction. The returned reference must not outlive that record.
unsafe fn dependent_secondary<'a>(handle: ObjectHandle) -> &'a AtomicPtr<c_void> {
    let record = handle.cast::<HandleRecord>();
    // SAFETY: the caller guarantees a live record. The kind is immutable. Avoid
    // borrowing the entire record: CoreCLR may concurrently clear its primary.
    if unsafe { ptr::addr_of!((*record).handle_type).read() } != DEPENDENT_HANDLE {
        std::process::abort();
    }
    // SAFETY: this initialized atomic field lives as long as the owned record.
    unsafe { &*ptr::addr_of!((*record).secondary) }
}

/// # Safety
/// `handle` must be a live dependent handle, with no concurrent destruction.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_gc_handle_manager_get_dependent_handle_secondary(
    handle: ObjectHandle,
) -> Object {
    // SAFETY: the caller keeps this dependent record alive until return.
    let secondary = unsafe { dependent_secondary(handle) };
    // Return the stored secondary even if the primary was cleared meanwhile:
    // comdependenthandle.cpp already read the primary and masks null targets.
    Object::from_ptr(secondary.load(Ordering::Acquire))
}

/// # Safety
/// `handle` must be a live dependent handle, with no concurrent destruction.
/// `object` must be null or a valid managed object supplied by CoreCLR.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_gc_handle_manager_set_dependent_handle_secondary(
    handle: ObjectHandle,
    object: Object,
) {
    // SAFETY: the caller keeps this dependent record alive until return.
    let secondary = unsafe { dependent_secondary(handle) };
    // ZeroGC never collects or moves objects. No collection-time barrier is
    // needed; release publishes initialization to concurrent secondary readers.
    secondary.store(object.as_ptr(), Ordering::Release);
}

/// # Safety
/// `handle` must be an exclusively owned live dependent handle from this store.
/// CoreCLR must have ended all access to it; destroy it exactly once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_gc_handle_manager_destroy_dependent_handle(handle: ObjectHandle) {
    let record = handle.cast::<HandleRecord>();
    // SAFETY: the caller owns the live record and no other access can race.
    if unsafe { ptr::addr_of!((*record).handle_type).read() } != DEPENDENT_HANDLE {
        std::process::abort();
    }
    // SAFETY: create_handle allocated this exact layout in the global allocator.
    // HandleRecord has no fields requiring Drop, and the caller ends its lifetime.
    unsafe { std::alloc::dealloc(record.cast(), Layout::new::<HandleRecord>()) };
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
    fn dependent_handle_preserves_primary_and_stores_secondary_updates() {
        use crate::handle_manager::rust_gc_handle_manager_store_object_in_handle;

        let mut primary_storage = 0_u8;
        let mut secondary_storage = 0_u8;
        let mut replacement_storage = 0_u8;
        let primary = Object::from_ptr(ptr::from_mut(&mut primary_storage).cast());
        let secondary = Object::from_ptr(ptr::from_mut(&mut secondary_storage).cast());
        let replacement = Object::from_ptr(ptr::from_mut(&mut replacement_storage).cast());
        let null = Object::from_ptr(ptr::null_mut());
        let handle = rust_gc_handle_store_create_dependent_handle(primary, secondary);
        assert!(!handle.is_null());

        // SAFETY: the test exclusively owns this live record and all targets.
        // No runtime or other thread can access or destroy it.
        unsafe {
            assert_eq!(*handle, primary);
            assert_eq!(
                rust_gc_handle_manager_get_dependent_handle_secondary(handle),
                secondary
            );
            for value in [replacement, null, secondary] {
                rust_gc_handle_manager_set_dependent_handle_secondary(handle, value);
                assert_eq!(
                    rust_gc_handle_manager_get_dependent_handle_secondary(handle),
                    value
                );
                assert_eq!(*handle, primary);
            }
            rust_gc_handle_manager_store_object_in_handle(handle, null);
            // The EE masks a null primary. The GC-side getter must preserve the
            // secondary for an EE reader that obtained the previous primary.
            assert_eq!(
                rust_gc_handle_manager_get_dependent_handle_secondary(handle),
                secondary
            );
            rust_gc_handle_manager_destroy_dependent_handle(handle);
        }
    }

    #[test]
    fn dependent_handle_accepts_null_targets_and_aliases() {
        let mut storage = 0_u8;
        let object = Object::from_ptr(ptr::from_mut(&mut storage).cast());
        let null = Object::from_ptr(ptr::null_mut());
        for (primary, secondary) in [
            (null, object),
            (object, null),
            (null, null),
            (object, object),
        ] {
            let handle = rust_gc_handle_store_create_dependent_handle(primary, secondary);
            assert!(!handle.is_null());
            // SAFETY: exclusively owned live record; destruction occurs once,
            // after the last access. Managed target storage outlives this loop.
            unsafe {
                assert_eq!(*handle, primary);
                assert_eq!(
                    (*handle.cast::<HandleRecord>()).handle_type,
                    DEPENDENT_HANDLE
                );
                assert_eq!(
                    rust_gc_handle_manager_get_dependent_handle_secondary(handle),
                    secondary
                );
                rust_gc_handle_manager_destroy_dependent_handle(handle);
            }
        }
    }

    #[test]
    fn diagnostic_accepts_only_non_null_normal_targets() {
        let mut object_storage = 0_u8;
        let object = Object::from_ptr(ptr::from_mut(&mut object_storage).cast::<c_void>());
        for (target, kind, expected) in [
            (object, 2, Ok(())),
            (Object::from_ptr(ptr::null_mut()), 2, Err(())),
            (object, 0, Err(())),
            (object, 3, Err(())),
            (object, DEPENDENT_HANDLE, Err(())),
        ] {
            let handle = rust_gc_handle_store_create_handle_of_type(target, kind);
            assert!(!handle.is_null());
            // SAFETY: exclusively owned live record; no runtime is active in this
            // test and no other thread can access the slot or move the target.
            assert_eq!(unsafe { check_diagnostic_handle(handle) }, expected);
            unsafe { destroy_test_handle(handle) };
        }
        // SAFETY: null is rejected before any access to a record.
        assert_eq!(unsafe { check_diagnostic_handle(ptr::null_mut()) }, Err(()));
    }

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
