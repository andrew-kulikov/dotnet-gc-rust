# 2. Bounds and metadata provenance

[Overview](README.md) · [Previous: layout](01-object-layout.md) · [Next: GCDesc](03-gcdesc.md)

A valid read needs four independent facts: a live object, stable contents during
the read, an owned object range, and valid metadata with a sufficient lifetime.
The fixture supplies a live normal handle through the native call;
suspension excludes managed mutations, and the caller must exclude native
writers. A normal handle is not a pinned handle: resolve it only inside the
suspension window and discard borrowed addresses before restart. A fabricated,
foreign, freed, or concurrently modified handle is outside the unsafe contract;
checking its numerical value cannot make dereferencing it safe.

## Object storage

The ZeroGC ledger records allocation ranges `(H, E)` and retains allocations for
the process lifetime. Before reading an object word, the reader must check
that `P` is non-null and 8-byte aligned, `P - 8` does not underflow, and that
it equals an allocation start. The range must contain the header and the full
word `[P, P+8)`, with all end calculations checked. After reading a trusted
BaseSize `S`, require `S == 24`, `H+S <= E`, and no overflow. Slot bounds
must fit both the logical object end and allocation end. A range proves neither
reachability nor initialization, and an interior address is not an object start.

`snapshot_object_ranges` copies at most 262,144 ranges with fallible allocation
before suspension and releases the ledger mutex before calling the runtime.
Budget exhaustion or allocation failure returns `BudgetExceeded` with no
restart attempt. Fixture objects already exist at snapshot time. Because this
ZeroGC neither moves nor frees allocations, the captured ranges remain valid
for the call. New ranges are outside that snapshot. A numerical range does not
create Rust pointer provenance: derive reads from the live pointer supplied by the runtime or the
original allocation, never from a fabricated integer pointer. Do not form Rust
references to managed objects. Explain readable bounds, initialization,
provenance, and absence of races at each unsafe read.

## Metadata witness

A MethodTable lives in loader memory, not the ZeroGC ledger. The
fixture obtains `typeof(Node).TypeHandle.Value` before native entry and passes
it as a trusted witness. For the pinned runtime, [RuntimeTypeHandle.Value](https://github.com/dotnet/runtime/blob/79d0c463f1b55624c874a11585f7e47731e8d675/src/coreclr/System.Private.CoreLib/src/System/RuntimeHandles.cs#L88-L108)
uses the native handle, and the VM [TypeHandle representation](https://github.com/dotnet/runtime/blob/79d0c463f1b55624c874a11585f7e47731e8d675/src/coreclr/vm/typehandle.h#L116-L177)
represents this ordinary class by its MethodTable. This is not a contract for
arbitrary `Type` values. The fixture requires the type and assembly to be
non-collectible, keeps the `Type` alive until return, and performs type lookup
before suspension. The [builder](https://github.com/dotnet/runtime/blob/79d0c463f1b55624c874a11585f7e47731e8d675/src/coreclr/vm/methodtablebuilder.cpp#L10687-L10765)
allocates the descriptor with the table on the loader heap. Unloadable types
and arbitrary caller-supplied pointers are unsupported.

Read the object word only after validating its range, compare it with the
untagged witness, and reject a mismatch without reading the foreign address.
For this known type, the read span is `[M-24, M+8)`: one descriptor
and the table prefix. This span must be justified by the fixture and pinned
builder, not by an untrusted count. A malformed bounded buffer can be rejected;
physical corruption of the runtime loader heap may still cause a native fault.
The reader copies this fixed 32-byte span to a local array before decoding; it
never uses a descriptor count to enlarge the span. The witness and initialized
span are an unsafe caller contract, not the result of a numeric validity test.
