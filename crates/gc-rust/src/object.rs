use std::ffi::c_void;
use std::fmt;

/// Opaque managed-object address received from CoreCLR.
///
/// CoreCLR owns the managed object. Rust only passes this address around and
/// must not interpret the managed object's C++ layout through the pointer.
/// Formatting never dereferences the address: `{object}` and `{object:p}` print
/// the managed-object address, while `{object:?}` labels it as an `Object`.
/// This does not invoke managed `ToString()` or inspect fields.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Object(*mut c_void);

impl Object {
    /// Wrap an opaque address without dereferencing it or taking ownership.
    pub const fn from_ptr(address: *mut c_void) -> Self {
        Self(address)
    }

    /// Return the opaque managed-object address.
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }

    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

impl fmt::Pointer for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.0, f)
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(self, f)
    }
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Object").field(&self.0).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatting_uses_object_address_without_reading_object_memory() {
        // Neither null nor a dangling pointer may be dereferenced by formatting.
        for address in [std::ptr::null_mut(), std::ptr::dangling_mut::<u8>().cast()] {
            let object = Object::from_ptr(address);
            assert_eq!(object.as_ptr(), address);
            assert_eq!(object.is_null(), address.is_null());
            assert_eq!(format!("{object}"), format!("{address:p}"));
            assert_eq!(format!("{object:p}"), format!("{address:p}"));
            assert_eq!(format!("{object:?}"), format!("Object({address:p})"));
        }
    }

    #[test]
    fn object_has_native_pointer_layout() {
        assert_eq!(size_of::<Object>(), size_of::<*mut c_void>());
        assert_eq!(align_of::<Object>(), align_of::<*mut c_void>());
    }
}
