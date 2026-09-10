# Implementing frozen-segment registration in Rust

`IGCHeap::RegisterFrozenSegment` receives a temporary `segment_info` owned by
CoreCLR. Because this project supports one pinned Windows x64 runtime, the C++
shim passes that pointer directly to Rust. Its `#[repr(C)]` representation is
`FrozenSegmentInfo`. Independent layout assertions in C++ and Rust keep the
five-field ABI synchronized. The descriptor pointer is valid only during the
call.

The fields describe one externally owned memory reservation:

- `memory` is the base of the reservation, before the first object.
- `first_object_offset` is the first possible object address relative to the
  base.
- `allocated_offset` is the end of the object-containing range.
- `committed_offset` is the end of committed memory.
- `reserved_offset` is the end of reserved memory.

The basic study implementation of `rust_gc_register_frozen_segment` now:

1. Reject a null descriptor or null `memory`.
2. Copies the descriptor immediately; it never retains the incoming descriptor
   pointer.
3. Validate
   `first_object_offset <= allocated_offset <= committed_offset <= reserved_offset`.
   Decide explicitly whether an empty object range is supported.
4. Convert the base pointer to an integer address and use checked integer
   addition to calculate the range endpoints. This validates overflow without
   creating out-of-bounds Rust pointers.
5. Allocates stable Rust-owned `Box<FrozenSegment>` metadata and inserts its
   non-overlapping reservation range into a mutex-protected registry. The
   backing segment memory remains externally owned.
6. Returns the boxed record's stable address as the opaque handle.
7. Leaves the registry unchanged and returns null for invalid or overlapping
   input.

This intentionally uses one global lock and retains registrations until process
exit because `UnregisterFrozenSegment` is not implemented yet. Replace that
process-lifetime retention when the unregister operation is added.

The follow-on methods must share the same handle lifecycle:

- `UnregisterFrozenSegment` validates/removes the exact registered record and
  releases only Rust metadata.
- `IsInFrozenSegment` queries whether the object address is in a registered
  half-open object range, normally
  `[base + first_object_offset, base + allocated_offset)`.
- `UpdateFrozenSegment` finds the record by handle and updates its allocated
  and committed endpoints only after checking ordering, reservation bounds,
  overflow, and whatever synchronization the heap uses for readers.

Do not return `memory` itself as the handle: the handle identifies registration
metadata and must support exact unregister/update operations. Also do not use
the normal managed-heap ownership rules for this memory. CoreCLR supplies and
owns frozen-segment storage; the collector only indexes and classifies it.
