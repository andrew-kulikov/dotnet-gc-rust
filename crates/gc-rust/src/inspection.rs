//! Narrow, bounded diagnostics for the GraphFixtures Node on CoreCLR x64.

use crate::object::Object;

const HEADER_SIZE: usize = 8;
const WORD_SIZE: usize = 8;
const CONTAINS_POINTERS: u32 = 0x0100_0000;
const COLLECTIBLE: u32 = 0x0020_0000;
const COMPONENT_SIZE: u32 = 0x8000_0000;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InspectionStatus {
    NotRun = 0,
    Reference = 1,
    Null = 2,
    InvalidObject = 3,
    UnsupportedLayout = 4,
    InvalidMetadata = 5,
    CorruptDescriptor = 6,
    InvalidTarget = 7,
    UnknownTarget = 8,
    ExpectationMismatch = 9,
    BudgetExceeded = 10,
}

#[derive(Debug, PartialEq, Eq)]
struct NodeLayout {
    size: usize,
    slot: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct Observation {
    pub status: InspectionStatus,
    pub size: u32,
    pub slot_offset: u32,
    pub slot_count: u32,
    pub next_node_id: u32,
}

const _: () = {
    assert!(size_of::<Observation>() == 20);
    assert!(align_of::<Observation>() == 4);
    assert!(std::mem::offset_of!(Observation, next_node_id) == 16);
};

impl Observation {
    pub(crate) fn empty(status: InspectionStatus) -> Self {
        Self {
            status,
            size: 0,
            slot_offset: 0,
            slot_count: 0,
            next_node_id: 0,
        }
    }
}

/// Read `node_to_inspect.Next` and identify it against `known_next_node`.
/// ID 1 is the known node; ID 0 is null. No address is returned.
///
/// # Safety
/// Both objects must come from live, exclusively owned normal handles,
/// and `node_method_table` must be the live noncollectible Node MethodTable
/// supplied by the controlled fixture. Its `[M-24,M+8)` span must be initialized and
/// readable. The snapshot must cover these retained ZeroGC allocations. The EE
/// is suspended, no native writer may race, and no borrowed pointer may survive
/// restart. Live reference slots must contain null or initialized CLR objects;
/// arbitrary heap corruption is outside this contract.
pub(crate) unsafe fn inspect_node(
    ranges: &[(usize, usize)],
    node_to_inspect: Object,
    known_next_node: Object,
    node_method_table: *const u8,
    expected_next_node_id: u32,
) -> Observation {
    use InspectionStatus::*;
    let operation = || -> Result<Observation, InspectionStatus> {
        let node_address = node_to_inspect.as_ptr().cast::<u8>();
        let range = object_range(ranges, node_address.addr()).map_err(|_| InvalidObject)?;
        // SAFETY: the exact owned range includes the initialized object word;
        // the caller guarantees the object stays live and cannot be written.
        let actual_method_table = unsafe { node_address.cast::<*const u8>().read() };
        // Equality alone cannot establish provenance: the caller supplies a
        // trusted Node MethodTable. Never read metadata through the object's table.
        if node_method_table.is_null()
            || !node_method_table.addr().is_multiple_of(WORD_SIZE)
            || actual_method_table != node_method_table
        {
            return Err(UnsupportedLayout);
        }
        let mut metadata = [0_u8; 32];
        // SAFETY: the caller guarantees that the trusted MethodTable's metadata span
        // is initialized, readable, and unchanged during this inspection.
        unsafe {
            std::ptr::copy_nonoverlapping(node_method_table.sub(24), metadata.as_mut_ptr(), 32)
        };
        let layout = decode_node(&metadata[24..32], &metadata[..24])?;
        object_extent(range, layout.size).map_err(|_| InvalidObject)?;
        // SAFETY: the aligned slot is within the initialized object extent.
        // A CLR reference has pointer layout, as does the transparent Object
        // wrapper; suspension and the caller exclude concurrent writers.
        let next_node = unsafe { node_address.add(layout.slot).cast::<Object>().read() };
        let (status, next_node_id) = if next_node.is_null() {
            (Null, 0)
        } else {
            let next_range =
                object_range(ranges, next_node.as_ptr().addr()).map_err(|_| InvalidTarget)?;
            object_extent(next_range, layout.size).map_err(|_| InvalidTarget)?;
            // SAFETY: the next node is an initialized CLR object by the caller's
            // contract; its exact owned range includes this aligned object word.
            let next_method_table = unsafe { next_node.as_ptr().cast::<*const u8>().read() };
            if next_method_table != node_method_table {
                return Err(InvalidTarget);
            }
            if next_node != known_next_node {
                return Err(UnknownTarget);
            }
            (Reference, 1)
        };
        Ok(Observation {
            status: if next_node_id == expected_next_node_id {
                status
            } else {
                ExpectationMismatch
            },
            size: layout.size as u32,
            slot_offset: layout.slot as u32,
            slot_count: 1,
            next_node_id,
        })
    };
    operation().unwrap_or_else(Observation::empty)
}

fn read_word(bytes: &[u8], offset: usize) -> Option<u64> {
    Some(u64::from_le_bytes(
        bytes.get(offset..offset.checked_add(8)?)?.try_into().ok()?,
    ))
}

fn decode_node(prefix: &[u8], descriptor: &[u8]) -> Result<NodeLayout, InspectionStatus> {
    use InspectionStatus::{CorruptDescriptor, InvalidMetadata, UnsupportedLayout};
    let word = read_word(prefix, 0).ok_or(InvalidMetadata)?;
    let flags = word as u32;
    let size = (word >> 32) as usize;
    if flags & (COLLECTIBLE | COMPONENT_SIZE) != 0 {
        return Err(UnsupportedLayout);
    }
    if flags & CONTAINS_POINTERS == 0 || size != 24 {
        return Err(InvalidMetadata);
    }
    // CoreCLR stores a signed series count immediately before the MethodTable.
    // This fixture supports exactly one positive series, not repeating layouts.
    let count = read_word(descriptor, 16).ok_or(CorruptDescriptor)? as i64;
    if count == 0 {
        return Err(CorruptDescriptor);
    }
    if count != 1 {
        return Err(UnsupportedLayout);
    }
    let adjustment = read_word(descriptor, 0).ok_or(CorruptDescriptor)? as i64;
    let offset = read_word(descriptor, 8).ok_or(CorruptDescriptor)?;
    // A series stores its byte length minus the object's base size.
    let length = i128::from(adjustment) + size as i128;
    if length <= 0 || length > (size - HEADER_SIZE) as i128 || length % 8 != 0 {
        return Err(CorruptDescriptor);
    }
    let offset = usize::try_from(offset).map_err(|_| CorruptDescriptor)?;
    let length = usize::try_from(length).map_err(|_| CorruptDescriptor)?;
    let end = offset.checked_add(length).ok_or(CorruptDescriptor)?;
    if offset < WORD_SIZE || !offset.is_multiple_of(WORD_SIZE) || end > size - HEADER_SIZE {
        return Err(CorruptDescriptor);
    }
    if offset != WORD_SIZE || length != WORD_SIZE {
        return Err(UnsupportedLayout);
    }
    Ok(NodeLayout { size, slot: offset })
}

/// Find an exact allocation start and prove the object word is in bounds.
fn object_range(ranges: &[(usize, usize)], object: usize) -> Result<(usize, usize), ()> {
    if object == 0 || !object.is_multiple_of(WORD_SIZE) {
        return Err(());
    }
    let start = object.checked_sub(HEADER_SIZE).ok_or(())?;
    let prefix_end = object.checked_add(WORD_SIZE).ok_or(())?;
    ranges
        .iter()
        .copied()
        .find(|&(base, end)| base == start && prefix_end <= end)
        .ok_or(())
}

fn object_extent(range: (usize, usize), size: usize) -> Result<(), ()> {
    if size < 24 || !size.is_multiple_of(WORD_SIZE) {
        return Err(());
    }
    let end = range.0.checked_add(size).ok_or(())?;
    if end > range.1 { Err(()) } else { Ok(()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounds_require_an_exact_start_and_complete_ranges() {
        let ranges = [(0x1000, 0x1018), (0x2000, 0x200f)];
        assert_eq!(object_range(&ranges, 0x1008), Ok((0x1000, 0x1018)));
        for address in [0, 0x1001, 0x1010, 0x3008, 0x2008, usize::MAX - 7] {
            assert_eq!(object_range(&ranges, address), Err(()));
        }
        assert_eq!(object_extent((0x1000, 0x1018), 24), Ok(()));
        for size in [0, 16, 25, 32, usize::MAX] {
            assert_eq!(object_extent((0x1000, 0x1018), size), Err(()));
        }
        assert_eq!(object_extent((usize::MAX - 15, usize::MAX), 24), Err(()));
    }

    fn node_metadata() -> ([u8; 8], [u8; 24]) {
        let mut prefix = [0; 8];
        prefix[0..4].copy_from_slice(&CONTAINS_POINTERS.to_le_bytes());
        prefix[4..8].copy_from_slice(&24_u32.to_le_bytes());
        let mut descriptor = [0; 24];
        descriptor[0..8].copy_from_slice(&(-16_i64).to_le_bytes());
        descriptor[8..16].copy_from_slice(&8_u64.to_le_bytes());
        descriptor[16..24].copy_from_slice(&1_i64.to_le_bytes());
        (prefix, descriptor)
    }

    #[test]
    fn decodes_one_series_and_rejects_truncated_words() {
        let (prefix, descriptor) = node_metadata();
        assert_eq!(
            decode_node(&prefix, &descriptor),
            Ok(NodeLayout { size: 24, slot: 8 })
        );
        for end in 0..prefix.len() {
            assert_eq!(
                decode_node(&prefix[..end], &descriptor),
                Err(InspectionStatus::InvalidMetadata)
            );
        }
        for end in 0..descriptor.len() {
            assert_eq!(
                decode_node(&prefix, &descriptor[..end]),
                Err(InspectionStatus::CorruptDescriptor)
            );
        }
    }

    #[test]
    fn rejects_unsupported_or_corrupt_metadata_in_bounded_work() {
        use InspectionStatus::{CorruptDescriptor, InvalidMetadata, UnsupportedLayout};
        let (prefix, descriptor) = node_metadata();
        for (count, error) in [
            (0_i64, CorruptDescriptor),
            (-1, UnsupportedLayout),
            (i64::MIN, UnsupportedLayout),
            (i64::MAX, UnsupportedLayout),
        ] {
            let mut bad = descriptor;
            bad[16..24].copy_from_slice(&count.to_le_bytes());
            assert_eq!(decode_node(&prefix, &bad), Err(error));
        }
        for adjustment in [-24_i64, -25, -15, 0, i64::MAX, i64::MIN] {
            let mut bad = descriptor;
            bad[0..8].copy_from_slice(&adjustment.to_le_bytes());
            assert_eq!(decode_node(&prefix, &bad), Err(CorruptDescriptor));
        }
        for offset in [0_u64, 1, 9, 16, u64::MAX - 7, u64::MAX] {
            let mut bad = descriptor;
            bad[8..16].copy_from_slice(&offset.to_le_bytes());
            assert_eq!(decode_node(&prefix, &bad), Err(CorruptDescriptor));
        }
        for (flags, size, error) in [
            (0, 24_u32, InvalidMetadata),
            (CONTAINS_POINTERS | COMPONENT_SIZE, 24, UnsupportedLayout),
            (CONTAINS_POINTERS | COLLECTIBLE, 24, UnsupportedLayout),
            (CONTAINS_POINTERS, 32, InvalidMetadata),
        ] {
            let mut bad = prefix;
            bad[0..4].copy_from_slice(&flags.to_le_bytes());
            bad[4..8].copy_from_slice(&size.to_le_bytes());
            assert_eq!(decode_node(&bad, &descriptor), Err(error));
        }
    }

    #[repr(C, align(8))]
    struct Metadata([u8; 32]);

    #[repr(C)]
    struct TestObject {
        header: u64,
        table: *const u8,
        next: *const u8,
    }

    fn test_range(object: &TestObject) -> (usize, usize) {
        let start = std::ptr::from_ref(object).addr();
        (start, start + size_of::<TestObject>())
    }

    fn test_object_pointer(object: &TestObject) -> *const u8 {
        // Derive from the whole object so the pointer also covers `next`.
        std::ptr::from_ref(object).cast::<u8>().wrapping_add(8)
    }

    #[test]
    fn reads_real_buffer_slots_and_distinguishes_target_outcomes() {
        let (prefix, descriptor) = node_metadata();
        let mut metadata = Metadata([0; 32]);
        metadata.0[..24].copy_from_slice(&descriptor);
        metadata.0[24..].copy_from_slice(&prefix);
        // SAFETY: this points into the same initialized, aligned metadata object.
        let node_method_table = unsafe { metadata.0.as_ptr().add(24) };
        let known_next_node = TestObject {
            header: 0,
            table: node_method_table,
            next: std::ptr::null(),
        };
        let unknown = TestObject {
            header: 0,
            table: node_method_table,
            next: std::ptr::null(),
        };
        let foreign = TestObject {
            header: 0,
            table: std::ptr::null(),
            next: std::ptr::null(),
        };
        let unregistered = TestObject {
            header: 0,
            table: node_method_table,
            next: std::ptr::null(),
        };
        let known_next_node_ptr = test_object_pointer(&known_next_node);
        // SAFETY: this is an interior byte of the known next node, not read.
        let interior = unsafe { known_next_node_ptr.add(1) };
        // SAFETY: this points at the next field inside the same object, not read.
        let aligned_interior = unsafe { known_next_node_ptr.add(8) };
        for (table, next_node, expected_next_node_id, status, id) in [
            (
                node_method_table,
                known_next_node_ptr,
                1,
                InspectionStatus::Reference,
                1,
            ),
            (
                node_method_table,
                known_next_node_ptr,
                0,
                InspectionStatus::ExpectationMismatch,
                1,
            ),
            (
                node_method_table,
                std::ptr::null(),
                0,
                InspectionStatus::Null,
                0,
            ),
            (
                node_method_table,
                test_object_pointer(&unknown),
                1,
                InspectionStatus::UnknownTarget,
                0,
            ),
            (
                node_method_table,
                test_object_pointer(&foreign),
                1,
                InspectionStatus::InvalidTarget,
                0,
            ),
            (
                node_method_table,
                interior,
                1,
                InspectionStatus::InvalidTarget,
                0,
            ),
            (
                node_method_table,
                aligned_interior,
                1,
                InspectionStatus::InvalidTarget,
                0,
            ),
            (
                node_method_table,
                test_object_pointer(&unregistered),
                1,
                InspectionStatus::InvalidTarget,
                0,
            ),
            (
                // A foreign table is never dereferenced; only the trusted
                // Node MethodTable may be used to read metadata.
                std::ptr::without_provenance(8),
                known_next_node_ptr,
                1,
                InspectionStatus::UnsupportedLayout,
                0,
            ),
        ] {
            let node_to_inspect = TestObject {
                header: 0,
                table,
                next: next_node,
            };
            let ranges = [
                test_range(&node_to_inspect),
                test_range(&known_next_node),
                test_range(&unknown),
                test_range(&foreign),
            ];
            let node_to_inspect_ptr = test_object_pointer(&node_to_inspect);
            // SAFETY: all pointers are derived from initialized owned test objects;
            // the metadata view is within `metadata`, and no thread mutates them.
            let result = unsafe {
                inspect_node(
                    &ranges,
                    Object::from_ptr(node_to_inspect_ptr.cast_mut().cast()),
                    Object::from_ptr(known_next_node_ptr.cast_mut().cast()),
                    node_method_table,
                    expected_next_node_id,
                )
            };
            assert_eq!((result.status, result.next_node_id), (status, id));
            if matches!(
                status,
                InspectionStatus::Reference
                    | InspectionStatus::Null
                    | InspectionStatus::ExpectationMismatch
            ) {
                assert_eq!(
                    (result.size, result.slot_offset, result.slot_count),
                    (24, 8, 1)
                );
            }
        }
    }
}
