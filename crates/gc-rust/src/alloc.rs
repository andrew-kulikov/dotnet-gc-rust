use std::{ffi::c_void, fmt::Debug};

/// CoreCLR's `gc_alloc_context` from gc/gcinterface.h.
///
/// Owned by the runtime's thread. Access through an incoming pointer requires
/// the caller to guarantee its validity and exclusive access before mutation.
#[repr(C)]
#[derive(Debug)]
pub struct GcAllocContext {
    pub alloc_ptr: *mut u8,
    pub alloc_limit: *mut u8,
    pub alloc_bytes: i64,
    pub alloc_bytes_uoh: i64,
    pub gc_reserved_1: *mut c_void,
    pub gc_reserved_2: *mut c_void,
    pub alloc_count: i32,
}

/// CoreCLR's `GC_ALLOC_FLAGS` bitmask, passed to `IGCHeap::Alloc` as uint32_t.
/// A transparent integer wrapper permits combinations and unknown flag bits.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct GcAllocFlags(pub u32);

impl GcAllocFlags {
    pub const NONE: Self = Self(0);
    pub const FINALIZE: Self = Self(1);
    pub const CONTAINS_REF: Self = Self(2);
    pub const ALIGN8_BIAS: Self = Self(4);
    pub const ALIGN8: Self = Self(8);
    pub const ZEROING_OPTIONAL: Self = Self(16);
    pub const LARGE_OBJECT_HEAP: Self = Self(32);
    pub const PINNED_OBJECT_HEAP: Self = Self(64);
    pub const USER_OLD_HEAP: Self = Self(32 | 64);

    pub const fn contains(self, flags: Self) -> bool {
        self.0 & flags.0 == flags.0
    }
}

impl Debug for GcAllocFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GcAllocFlags(")?;
        let mut first = true;
        for (name, flag) in [
            ("FINALIZE", GcAllocFlags::FINALIZE),
            ("CONTAINS_REF", GcAllocFlags::CONTAINS_REF),
            ("ALIGN8_BIAS", GcAllocFlags::ALIGN8_BIAS),
            ("ALIGN8", GcAllocFlags::ALIGN8),
            ("ZEROING_OPTIONAL", GcAllocFlags::ZEROING_OPTIONAL),
            ("LARGE_OBJECT_HEAP", GcAllocFlags::LARGE_OBJECT_HEAP),
            ("PINNED_OBJECT_HEAP", GcAllocFlags::PINNED_OBJECT_HEAP),
        ] {
            if self.contains(flag) {
                if first {
                    write!(f, "{name}")?;
                    first = false;
                } else {
                    write!(f, " | {name}")?;
                }
            }
        }
        write!(f, ")")?;
        Ok(())
    }
}

// Keep these Windows x64 ABI checks in sync with the assertions in shim.cpp.
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
const _: () = {
    assert!(size_of::<GcAllocContext>() == 56);
    assert!(align_of::<GcAllocContext>() == 8);
    assert!(std::mem::offset_of!(GcAllocContext, alloc_ptr) == 0);
    assert!(std::mem::offset_of!(GcAllocContext, alloc_limit) == 8);
    assert!(std::mem::offset_of!(GcAllocContext, alloc_bytes) == 16);
    assert!(std::mem::offset_of!(GcAllocContext, alloc_bytes_uoh) == 24);
    assert!(std::mem::offset_of!(GcAllocContext, gc_reserved_1) == 32);
    assert!(std::mem::offset_of!(GcAllocContext, gc_reserved_2) == 40);
    assert!(std::mem::offset_of!(GcAllocContext, alloc_count) == 48);
};
