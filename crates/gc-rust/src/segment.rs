use std::ffi::c_void;
use std::ptr;
use std::sync::Mutex;

/// Rust's C ABI representation of CoreCLR's `segment_info`.
///
/// Every size is a byte offset from `memory`. The descriptor itself is borrowed
/// only for the duration of `rust_gc_register_frozen_segment`; a real
/// implementation must copy the values it needs into Rust-owned metadata.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FrozenSegmentInfo {
    pub memory: *mut c_void,
    pub first_object_offset: usize,
    pub allocated_offset: usize,
    pub committed_offset: usize,
    pub reserved_offset: usize,
}

/// Opaque identity of Rust-owned frozen-segment metadata.
pub type FrozenSegmentHandle = *mut c_void;

#[derive(Debug)]
struct FrozenSegment {
    memory: usize,
    first_object: usize,
    allocated: usize,
    committed: usize,
    reserved: usize,
}

// A Box gives every returned handle a stable address even if this Vec grows.
// The global lock is intentionally simple for the study collector.
#[expect(
    clippy::vec_box,
    reason = "boxed addresses are returned as stable handles"
)]
static FROZEN_SEGMENTS: Mutex<Vec<Box<FrozenSegment>>> = Mutex::new(Vec::new());

impl FrozenSegment {
    fn from_info(info: FrozenSegmentInfo) -> Option<Self> {
        if info.memory.is_null()
            || info.reserved_offset == 0
            || info.first_object_offset > info.allocated_offset
            || info.allocated_offset > info.committed_offset
            || info.committed_offset > info.reserved_offset
        {
            return None;
        }

        let memory = info.memory as usize;
        let segment = Self {
            memory,
            first_object: memory.checked_add(info.first_object_offset)?,
            allocated: memory.checked_add(info.allocated_offset)?,
            committed: memory.checked_add(info.committed_offset)?,
            reserved: memory.checked_add(info.reserved_offset)?,
        };
        debug_assert!(segment.memory <= segment.first_object);
        debug_assert!(segment.first_object <= segment.allocated);
        debug_assert!(segment.allocated <= segment.committed);
        debug_assert!(segment.committed <= segment.reserved);
        Some(segment)
    }

    fn overlaps(&self, other: &Self) -> bool {
        self.memory < other.reserved && other.memory < self.reserved
    }
}

/// Registers externally owned frozen-segment memory and returns a stable handle
/// to Rust-owned metadata. Invalid and overlapping ranges return null.
///
/// # Safety
///
/// A non-null `segment_info` must point to an initialized `FrozenSegmentInfo`
/// that remains readable for the duration of this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_gc_register_frozen_segment(
    segment_info: *const FrozenSegmentInfo,
) -> FrozenSegmentHandle {
    println!("rust_gc_register_frozen_segment: {:?}", segment_info);

    if segment_info.is_null() {
        return ptr::null_mut();
    }

    // SAFETY: the caller guarantees that a non-null descriptor is initialized
    // and readable for this call. Copying it ensures no borrowed pointer escapes.
    let info = unsafe { *segment_info };
    let Some(mut segment) = FrozenSegment::from_info(info).map(Box::new) else {
        return ptr::null_mut();
    };

    let mut segments = FROZEN_SEGMENTS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if segments.iter().any(|existing| segment.overlaps(existing)) {
        return ptr::null_mut();
    }

    let handle = ptr::from_mut(segment.as_mut()).cast::<c_void>();
    segments.push(segment);
    handle
}


/// Updates the allocated and committed pointers of a previously registered frozen segment.
///
/// # Safety
///
/// The `segment` must be a valid handle returned by `rust_gc_register_frozen_segment`.
/// The `allocated` and `committed` pointers must be valid for the duration of this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_gc_update_frozen_segment(
    segment: FrozenSegmentHandle,
    allocated: *mut u8,
    committed: *mut u8,
) {
    println!("rust_gc_update_frozen_segment: {:?}", segment);

    if segment.is_null() {
        return;
    }

    let segment = segment.cast::<FrozenSegment>();
    // SAFETY: the caller guarantees that `segment` is a valid handle.
    unsafe {
        (*segment).allocated = allocated as usize;
        (*segment).committed = committed as usize;
    }
}

// Keep these Windows x64 checks in sync with rust_gc.h and shim.cpp.
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
const _: () = {
    assert!(size_of::<FrozenSegmentInfo>() == 40);
    assert!(align_of::<FrozenSegmentInfo>() == 8);
    assert!(std::mem::offset_of!(FrozenSegmentInfo, memory) == 0);
    assert!(std::mem::offset_of!(FrozenSegmentInfo, first_object_offset) == 8);
    assert!(std::mem::offset_of!(FrozenSegmentInfo, allocated_offset) == 16);
    assert!(std::mem::offset_of!(FrozenSegmentInfo, committed_offset) == 24);
    assert!(std::mem::offset_of!(FrozenSegmentInfo, reserved_offset) == 32);
};

#[cfg(test)]
mod tests {
    use super::*;

    fn remove_test_segment(handle: FrozenSegmentHandle) {
        let mut segments = FROZEN_SEGMENTS
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let position = segments
            .iter()
            .position(|segment| ptr::eq(segment.as_ref(), handle.cast::<FrozenSegment>()))
            .expect("test segment handle must be registered");
        segments.swap_remove(position);
    }

    #[test]
    fn registration_copies_metadata_and_returns_stable_handle() {
        let mut memory = [0_u8; 64];
        let info = FrozenSegmentInfo {
            memory: memory.as_mut_ptr().cast(),
            first_object_offset: 8,
            allocated_offset: 24,
            committed_offset: 32,
            reserved_offset: 64,
        };

        // SAFETY: `info` is initialized and remains live for this call.
        let handle = unsafe { rust_gc_register_frozen_segment(&info) };
        assert!(!handle.is_null());

        let segment = handle.cast::<FrozenSegment>();
        // SAFETY: a successful registration returns a live FrozenSegment and
        // this test does not remove it until after these reads.
        unsafe {
            assert_eq!((*segment).memory, memory.as_ptr() as usize);
            assert_eq!((*segment).first_object, memory.as_ptr() as usize + 8);
            assert_eq!((*segment).allocated, memory.as_ptr() as usize + 24);
            assert_eq!((*segment).committed, memory.as_ptr() as usize + 32);
            assert_eq!((*segment).reserved, memory.as_ptr() as usize + 64);
        }

        remove_test_segment(handle);
    }

    #[test]
    fn registration_rejects_invalid_descriptors() {
        let mut memory = [0_u8; 64];
        let valid = FrozenSegmentInfo {
            memory: memory.as_mut_ptr().cast(),
            first_object_offset: 8,
            allocated_offset: 24,
            committed_offset: 32,
            reserved_offset: 64,
        };

        let invalid = [
            FrozenSegmentInfo {
                memory: ptr::null_mut(),
                ..valid
            },
            FrozenSegmentInfo {
                first_object_offset: 25,
                ..valid
            },
            FrozenSegmentInfo {
                allocated_offset: 33,
                ..valid
            },
            FrozenSegmentInfo {
                committed_offset: 65,
                ..valid
            },
            FrozenSegmentInfo {
                reserved_offset: 0,
                ..valid
            },
            FrozenSegmentInfo {
                memory: ptr::without_provenance_mut(usize::MAX),
                ..valid
            },
        ];

        // SAFETY: null descriptors are explicitly accepted as invalid input.
        assert!(unsafe { rust_gc_register_frozen_segment(ptr::null()) }.is_null());
        for info in invalid {
            // SAFETY: every descriptor is initialized and live for this call.
            assert!(unsafe { rust_gc_register_frozen_segment(&info) }.is_null());
        }
    }

    #[test]
    fn registration_rejects_overlapping_reservations() {
        let mut memory = [0_u8; 128];
        let first = FrozenSegmentInfo {
            memory: memory.as_mut_ptr().cast(),
            first_object_offset: 8,
            allocated_offset: 24,
            committed_offset: 32,
            reserved_offset: 64,
        };
        let overlapping = FrozenSegmentInfo {
            // SAFETY: this points within `memory` and is not dereferenced.
            memory: unsafe { memory.as_mut_ptr().add(32) }.cast(),
            ..first
        };

        // SAFETY: both descriptors are initialized and live for their calls.
        let handle = unsafe { rust_gc_register_frozen_segment(&first) };
        assert!(!handle.is_null());
        // SAFETY: `overlapping` is initialized and live for this call.
        assert!(unsafe { rust_gc_register_frozen_segment(&overlapping) }.is_null());

        remove_test_segment(handle);
    }
}
