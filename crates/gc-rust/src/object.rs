use std::ffi::c_void;
use std::fmt;

/// Opaque managed-object address received from CoreCLR.
///
/// CoreCLR owns the managed object. This wrapper provides no object-layout
/// access; the separate diagnostic reader has a narrow, explicit unsafe contract.
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

    #[test]
    fn synthetic_node_descriptor_describes_one_reference_slot() {
        // CoreCLR x64 stores a positive GCDesc immediately before MethodTable:
        // adjusted series size, offset from Object*, then series count.
        let mut descriptor = [0_u8; 24];
        descriptor[0..8].copy_from_slice(&(-16_i64).to_le_bytes());
        descriptor[8..16].copy_from_slice(&8_u64.to_le_bytes());
        descriptor[16..24].copy_from_slice(&1_i64.to_le_bytes());

        let word = |offset| u64::from_le_bytes(descriptor[offset..offset + 8].try_into().unwrap());
        let object_size = 24_i64; // Includes the 8-byte header before Object*.
        let adjustment = word(0) as i64;
        let offset = word(8);
        let count = word(16) as i64;
        let reference_bytes = object_size.checked_add(adjustment).unwrap();

        assert_eq!(count, 1);
        assert_eq!(reference_bytes, 8);
        assert_eq!(offset..offset + reference_bytes as u64, 8..16);
        assert_eq!(object_size - 8, 16); // End of object relative to Object*.
    }
}
