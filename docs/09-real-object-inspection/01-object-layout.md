# 1. Object layout

[Overview](README.md) · [Next: bounds and provenance](02-metadata-and-bounds.md)

The fixture declares a sealed `Node` with one `Node? Next` field. Labels are
stored separately. A nullable reference is still one pointer-sized slot; null
changes its value, not the layout. C# source order is not a general native ABI,
so these offsets are a narrow, source-derived expectation. The checkpoint
confirms the size and reference slot for this fixture on the pinned runtime.

The target is Windows x64, SDK `10.0.400`, runtime `10.0.11`, with the runtime
source pinned to `79d0c463f1b55624c874a11585f7e47731e8d675`. The object header,
reference slots, and allocation alignment are each 8 bytes; the minimum object
size is 24 bytes. See the pinned [object size definitions](https://github.com/dotnet/runtime/blob/79d0c463f1b55624c874a11585f7e47731e8d675/src/coreclr/vm/object.h#L86-L109)
and [BaseSize calculation](https://github.com/dotnet/runtime/blob/79d0c463f1b55624c874a11585f7e47731e8d675/src/coreclr/vm/methodtablebuilder.cpp#L11087-L11095).

```text
H = P - 8     ObjHeader: 4-byte padding and 4-byte sync-block value
P             MethodTable pointer M                         [P, P+8)
P + 8         Next: object reference or null                 [P+8, P+16)
P + 16        end of Node allocation
```

`P` is the managed object reference; `H` is the allocation start. `M` points to
separate loader metadata. For `Node`, `BaseSize = align_up(max(8 + 16, 24), 8)
= 24`, including the header. Thus its complete range is `[H, H+24)`, or
`[P-8, P+16)`. Using `P+24` as the end would permit an eight-byte overread.
ZeroGC allocates the header with the body and returns `H+8`; an allocation
record alone does not prove that the runtime has initialized the object.

The relevant [MethodTable prefix](https://github.com/dotnet/runtime/blob/79d0c463f1b55624c874a11585f7e47731e8d675/src/coreclr/vm/methodtable.h#L3891-L3905)
contains `uint32` flags at `M` and `uint32` BaseSize at `M+4`. The
[GC-side masks](https://github.com/dotnet/runtime/blob/79d0c463f1b55624c874a11585f7e47731e8d675/src/coreclr/gc/env/gcenv.object.h#L50-L55)
are `ContainsGCPointers = 0x01000000`, `Collectible = 0x00200000`, and
`HasComponentSize = 0x80000000`. The supported profile requires the first bit
and rejects the latter two. Other flag bits need not equal a fabricated constant.
Variable-size objects use component size and are outside this mission.

The object word must equal the trusted, untagged witness for this `Node` before
metadata is read. A size and flag match cannot identify a type. Unexpected tag
bits must be rejected rather than silently masked. See
[bounds and provenance](02-metadata-and-bounds.md) for the lifetime contract.
The reader checks the flags and reads `BaseSize`; it does not substitute 24 for
the runtime value. A mismatch is reported as invalid metadata or unsupported
layout rather than accepted as another object shape.
