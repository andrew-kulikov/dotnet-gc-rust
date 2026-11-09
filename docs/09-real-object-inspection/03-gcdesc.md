# 3. GCDesc

[Overview](README.md) · [Previous: provenance](02-metadata-and-bounds.md) · [Next: inspection](04-suspended-inspection.md)

`GCDesc` describes reference locations for a type; it does not store the value
of `Next`. The pinned [descriptor definition](https://github.com/dotnet/runtime/blob/79d0c463f1b55624c874a11585f7e47731e8d675/src/coreclr/gc/gcdesc.h)
and [runtime traversal](https://github.com/dotnet/runtime/blob/79d0c463f1b55624c874a11585f7e47731e8d675/src/coreclr/gc/gc.cpp#L27387-L27449)
place a positive series immediately before `M` on x64:

```text
M - 24   8 bytes  adjusted series length
M - 16   8 bytes  series start offset from P
M -  8   8 bytes  signed series count
M        8 bytes  MethodTable flags and BaseSize
```

For a positive count `n`, the descriptor occupies `8 + 16*n` bytes; the
runtime traverses from the highest series towards lower addresses. Mission 09
supports exactly one series and must not use the count to establish its own
readable bounds. A negative count is a different repeating form and remains
unsupported. The fixture's expected values, derived from the pinned
[builder](https://github.com/dotnet/runtime/blob/79d0c463f1b55624c874a11585f7e47731e8d675/src/coreclr/vm/methodtablebuilder.cpp#L11925-L11977),
are:

```text
BaseSize = 24, count = 1, offset = 8, reference length = 8
adjustment = 8 - 24 = -16 = 0xffff_ffff_ffff_fff0
slot = [P+8, P+16), object end = P+16
```

The stored size is a `size_t` bit pattern. For this narrow profile, interpret
it as signed two's complement, extend to `i128`, and add the object size.
An unsigned checked addition would incorrectly reject `-16`; blind wrapping
could conceal malformed input. Validate a positive, aligned length and checked
`offset + length` within `S - 8`; then require exactly offset 8 and length 8.
The supported decoder must read actual metadata rather than return constants.

The decoder checks a bounded metadata view, flags and BaseSize,
then count, then the one series. Count zero is corrupt for this reference-bearing
profile; negative and positive counts above one are unsupported. A truncated
word, invalid alignment, zero or excessive length, offset into the MethodTable
word, or offset/end overflow must fail explicitly before reading the slot.
Null is a successful value of the existing slot, not an absent descriptor.
Tests must use owned byte buffers rather than corrupt the running managed heap.

[inspection.rs](../../crates/gc-rust/src/inspection.rs) implements this decoder
over byte slices; its tests cover truncated spans, malformed sizes and offsets,
and extreme counts. The separate step 1 example in
[object.rs](../../crates/gc-rust/src/object.rs) illustrates only the arithmetic.
The live checkpoint subsequently observed the same size and slot on the pinned
runtime; the synthetic cases do not claim to corrupt or inspect the live heap.
