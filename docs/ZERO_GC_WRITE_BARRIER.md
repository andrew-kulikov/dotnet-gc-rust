# Card tables and write barriers: why our ZeroGC works

This is a learning guide to the current implementation, not a design for the
future collecting GC. It applies to Windows x64, workstation GC, and the pinned
.NET 10.0.11 runtime (commit `79d0c463f1b55624c874a11585f7e47731e8d675`).
NativeAOT is discussed below to compare source implementations; our smoke test
runs CoreCLR.

## 1. Why keep track of reference writes?

Imagine a normal generational GC and this code:

```csharp
oldObject.Child = new Node();
```

`oldObject` has been alive for a while and is in generation 2. The new `Node` is
in generation 0. We now have this connection:

```text
GC root → old object → Child field → young object
```

A generation 0 collection tries to avoid walking all the older generations.
However, it must discover that the old object's field keeps the young object
alive. Otherwise, it could incorrectly decide that nobody needs the young object.

That is why the runtime performs some extra work when a reference is written:
“A reference in this part of memory changed; check this area during collection.”
This extra work is called a **GC write barrier**.

The barrier does not prevent the write. It performs the write and, when necessary,
records the change. Do not confuse this with a CPU memory barrier: that concerns
the order in which memory operations are observed, whereas here we are tracking
references for the GC.

## 2. Objects and the table occupy separate memory areas

A card table is a separate, small array of flags. It is not another object heap.

Think of a book and a separate sheet listing pages that need to be read again.
The book holds the data; the sheet helps you find the relevant places quickly.

In the simple whole-byte marking path on our x64 platform, one table byte covers
2048 bytes of object memory:

```text
Object memory:
┌───────────────────┬───────────────────┬───────────────────┐
│ 2 KB              │ 2 KB              │ 2 KB              │
│                   │ oldObject.Child   │                   │
└─────────┬─────────┴─────────┬─────────┴─────────┬─────────┘
          │                   │                   │
          ▼                   ▼                   ▼
Table:   [00]                [FF]                [00]
         clean               dirty               clean
```

`FF` means that the area needs attention. It does not mean that all objects there
are alive, nor does it record exactly which field changed. The GC still needs to
examine real objects and references.

**We mark the area containing the field into which the reference was written.**
In this example, that means the address of `oldObject.Child`, not the address
of the new `Node`.

A terminology detail: bitwise variants exist where one bit describes 256 bytes,
and the eight bits of a table byte together describe 2 KB. The simple path sets
the entire byte to `FF`. This is why the source also uses the word `clump` for a
group of cards. Below, we focus on marking a whole byte.

## 3. Where do H and T come from?

For a numerical example, take one 6 KB area of object memory and three bytes of
table storage. All addresses here are made up and written in decimal.

- `H = 8192` is the starting address of the object memory area.
- `T = 50000` is the starting address of the separately allocated flag array.

These are two addresses, not two object heaps. `T` is not calculated from `H`:
the table's memory is allocated separately.

```text
Object memory                            Flag array

Addresses              Size              Address     Value
┌────────────────────────────┐           ┌──────────────────┐
│ 8192 … 10239          2 KB  │ ────────► │ 50000     00     │
├────────────────────────────┤           ├──────────────────┤
│ 10240 … 12287         2 KB  │ ────────► │ 50001     00     │
├────────────────────────────┤           ├──────────────────┤
│ 12288 … 14335         2 KB  │ ────────► │ 50002     00     │
└────────────────────────────┘           └──────────────────┘
▲                                        ▲
H                                        T
```

Suppose we write a reference at address `10400`. That address is in the second
area, so we need to put `FF` at address `50001`.

The most straightforward calculation has three steps:

```text
Distance from the area's beginning: 10400 - 8192 = 2208 bytes
Area number, counting from 0:       2208 / 2048 = 1
Address of the required byte:       50000 + 1 = 50001
```

Division here is integer division: the fractional part is discarded. The formula
is:

```text
byte address = T + (destination - H) / 2048
```

`destination` is the address of the location receiving the reference.

## 4. Why subtract H / 2048 from T?

We want to avoid subtracting the area's starting address on every reference
write. There are many writes, while the table's location changes much less often.

In our example, `H` is aligned to 2048 bytes: it is divisible by 2048 without a
remainder. Under that condition, we can rearrange the formula:

```text
T + (destination - H) / 2048

= T + destination / 2048 - H / 2048

= (T - H / 2048) + destination / 2048
```

We calculate the constant part in parentheses ahead of time:

```text
g_card_table = T - H / 2048
```

Then each write only needs:

```text
byte address = g_card_table + destination / 2048
```

In the code, division is written as `>> 11`, because `2048 = 2¹¹`:

```cpp
cardAddress = g_card_table + (destinationAddress >> 11);
```

Substituting our numbers:

```text
Ahead of time:
    H / 2048 = 8192 / 2048 = 4
    g_card_table = 50000 - 4 = 49996

When writing:
    destination / 2048 = 10400 / 2048 = 5
    cardAddress = 49996 + 5 = 50001
```

Why subtraction specifically? Dividing an absolute address numbers areas **from
address 0**. We need numbers **from the beginning of our object memory area**:

```text
Number from address 0:    0      1      2      3      4      5      6
Area starting address:   0    2048   4096   6144   8192  10240  12288
                                                     ▲
                                           Our area begins here

Number within our area:                              0      1      2
```

Subtracting 4 turns the absolute numbers `4, 5, 6` into the local numbers
`0, 1, 2` that we need. The runtime incorporates this adjustment into the table
base ahead of time.

Therefore, **`g_card_table` is an adjusted base for calculations, not necessarily
the actual starting address of the allocated array**. This adjustment is called
a bias.

```text
49996                         50000    50001    50002
  ▲                             ▲
g_card_table                    T
base used in calculations       actual array
```

Nothing needs to be read at address `49996`. Memory is accessed after adding the
index, for example at address `50001`. This diagram explains the runtime's
address arithmetic; it does not mean that ordinary Rust pointer operations may
arbitrarily go outside an allocation's bounds.

## 5. Two addresses and two checks

An assignment has a destination address and a reference value:

```text
a.Child = b
   │     │
   │     └── WHAT we write: the address of object B.
   └──────── WHERE we write: the address of the Child field in object A.
```

| Check | Address being checked | Purpose |
| --- | --- | --- |
| Heap bounds | Destination address | Does a write at this location need tracking? |
| Ephemeral bounds | The reference value being written | Could the reference point into the young generations? |

Here, ephemeral refers to the young generations. A real runtime may additionally
account for regions and their generations. These two checks are enough to
explain our configuration.

A checked write barrier checks whether the destination falls within the declared
heap bounds. An unchecked write barrier can skip that check: the caller already
assumes that it is writing into a heap object.

A simplified single-reference barrier:

```text
Write the reference into the field.
If the reference does not require tracking as a reference to a young object, finish.
Otherwise, mark the table byte corresponding to the field's address.
```

## 6. Why bulk copy works differently

For example, `Array.Copy` can copy a block of references. Checking each reference
individually is not always worthwhile. The runtime can copy the block and then
mark every destination area touched by the write.

The CoreCLR path relevant to our investigation looks like this:

```text
InlinedMemmoveGCRefsHelper
    │
    ├── Determine whether the destination is within the declared heap.
    ├── Copy the data, accounting for overlapping memory areas.
    └── If the destination is in the heap:
            InlinedSetCardsAfterBulkCopyHelper
```

`InlinedSetCardsAfterBulkCopyHelper` does not itself copy the data. It relies on
the caller to check the destination. It calculates the range of table bytes:

```text
first = startAddress / 2048
endExclusive = round_up(startAddress + byteCount, 2048) / 2048

For each index from first up to, but not including, endExclusive:
    If the byte at g_card_table + index is not already FF:
        write FF
```

For example, copying 32 bytes to address `10224` crosses the boundary between
two areas:

```text
First area: 8192 … 10239            Second area: 10240 … 12287
                  [16 bytes       |       16 bytes]
                           32 copied bytes

Flags:               FF                       FF
```

Checking the existing value reduces unnecessary memory writes and traffic
between processor caches. When card bundles are enabled, the helper also updates
a coarser table: one of its bytes describes 2 MB of object memory.

**The bulk helper does not check each copied reference against the ephemeral
bounds.** It conservatively marks the destination area. An unnecessary mark can
cause extra scanning; a missing necessary mark can break collection.

## 7. Why the previous configuration crashed

Our allocator allocates objects separately using `alloc_zeroed`. Instead of a
full table, we supplied the address of one static `u32`: four bytes.

The old parameters were:

| Parameter | Old value |
| --- | --- |
| `lowest_address` | `1` |
| `highest_address` | `usize::MAX` |
| `ephemeral_low` | `usize::MAX` |
| `ephemeral_high` | `usize::MAX` |
| `card_table` | Address of a four-byte variable |

Let us call `usize::MAX` simply `MAX`. On x64, it is `0xFFFFFFFFFFFFFFFF`: a
limiting number for comparisons, not an allocated object address.

The interval `[low, high)` contains addresses satisfying `address >= low` and
`address < high`. The upper bound is excluded.

The interval `[1, MAX)` included almost every ordinary process address. The
interval `[MAX, MAX)` was empty: no number can simultaneously be at least `MAX`
and less than `MAX`.

Single-reference barriers could exit on the ephemeral check without accessing
the table. But for bulk copy, the destination was considered part of the heap:

```text
Destination falls within [1, MAX)
              ↓
Copy data and mark cards
              ↓
Small variable's address + (destination address >> 11)
              ↓
Access outside the four-byte variable
```

Even for the made-up address `30000`, the index is `14`: already outside four
bytes. With real process addresses, the offset is much larger.

During the investigation, we reproduced an access violation exit code
`0xC0000005` with this message:

```text
Fatal error.
Internal CLR error. (0x80131506)
   at System.Diagnostics.Tracing.EventSource.InitializeDefaultEventSources()
```

EventSource is where the problem surfaced during runtime startup. Its name in
the managed stack does not by itself identify a bug in EventSource's implementation.

## 8. Why the minimal change helped

We changed `lowest_address` from `1` to `MAX`. Both intervals are now empty:

```text
lowest_address  = MAX
highest_address = MAX

ephemeral_low   = MAX
ephemeral_high  = MAX
```

### The real memory did not disappear

The bounds provide information to the runtime. They do not allocate or free
memory. Objects still occupy their Rust allocations.

```text
Real memory:           object A             object B
                          │                    ▲
                          └──── Child ─────────┘

Declared range
used by the checks:    [MAX, MAX) — empty
```

We intentionally supply bounds that do not describe the objects' real locations,
in order to disable write tracking in this learning configuration.

### Bulk copy no longer calls the card-marking helper

The destination check is equivalent to:

```cpp
notInHeap = destination < lowest_address || destination >= highest_address;
```

For destination `30000`, after the change:

```text
30000 < MAX → yes → notInHeap = true
```

The data is still copied. However, the condition for calling the card-marking
function is false:

```text
Copy data → destination is outside the declared heap → finish
```

The placeholder table is neither read nor written.

### The unchecked barrier stops at a different check

Not every single-reference barrier checks heap bounds. The current x64 pre-grow
variant approximately does the following:

```text
Write reference at destination.
If reference < ephemeral_low, finish.
Otherwise, access the card table.
```

With `ephemeral_low = MAX`, every valid Windows user-mode object address, as well
as null, is below the lower bound. The write happens, but tracking is skipped.

Choosing the maximum bound matters here. This variant only checks the lower
ephemeral bound. Arbitrary equal low/high values, such as `1000` and `1000`,
would not stop it for reference `20000`.

### Both settings are needed together

```text
                         Reference write or copy
                                    │
                 ┌──────────────────┴────────────────────┐
                 ▼                                       ▼
      Destination check present              Destination check skipped
      For example, bulk copy                 For example, unchecked store
                 │                                       │
                 ▼                                       ▼
      Empty heap bounds                      Ephemeral lower bound = MAX
      exclude the destination                excludes real references
                 │                                       │
                 └──────────────────┬────────────────────┘
                                    ▼
                         The table is not accessed
```

An empty ephemeral interval alone did not protect bulk copy: that was the bug.
Empty heap bounds alone do not protect paths that skip the heap check.

## 9. How the parameters reach the runtime from Rust

`StompWriteBarrier` is a configuration operation. It is not called for every field
write. During initialization, the following happens:

```text
Rust: rust_gc_initialize
    ↓ constructs WriteBarrierParameters
C-compatible callback table
    ↓
C++: StompWriteBarrier_Invoke
    ↓ performs a virtual call
CoreCLR: IGCToCLR::StompWriteBarrier
    ↓
Installs bounds and configures the barrier's machine code
```

After that, managed code uses the runtime barriers with those installed values.
It does not call back into our Rust code on every reference write.

The C++ shim owns the C++ virtual-call ABI. Rust stores a context and a
C-compatible callback rather than reconstructing the `IGCToCLR` vtable.

The current additional parameters are: no region table, `region_shr = 0`,
bitwise region barriers disabled, no write-watch table, and
`requires_upper_bounds_check = false`. The static `u32` address remains a non-null
placeholder for the card table and card-bundle table. Avoiding a crash does not
turn that placeholder into a real table.

## 10. What about NativeAOT instead of JIT?

NativeAOT compiles code ahead of time, but it does not remove GC or the need to
track reference changes. The pinned sources contain an analogous bulk path:

```text
CoreCLR:
InlinedMemmoveGCRefsHelper → InlinedSetCardsAfterBulkCopyHelper

NativeAOT:
RhBulkMoveWithWriteBarrier → InlinedBulkWriteBarrier
```

Both paths check whether the destination belongs to the declared heap, copy the
data, and, when necessary, mark a range of cards without checking every reference
against the ephemeral bounds. Switching to NativeAOT would not by itself fix the
old placeholder table combined with broad heap bounds.

| Detail | CoreCLR with JIT | NativeAOT |
| --- | --- | --- |
| Application code generation | At runtime; precompiled code can also be present | Ahead of time |
| Example single-reference helper | `JIT_WriteBarrier` | `RhpAssignRef` |
| Configuration in the x64 path examined | Selects a specialized barrier and patches embedded addresses/bounds | Reads runtime global values in the assembly helper examined |
| GC metadata needed for ordinary collection | Still needed | Still needed |

This compares specific source implementations. It does not establish that our
standalone shim supports NativeAOT: the current target and smoke test are CoreCLR.

## 11. What was verified, and where this shortcut ends

During the investigation, the original configuration reproduced the fatal error.
After changing the bounds, the program passed EventSource initialization, printed
`Hello, World!`, and stopped at an explicit abort in the still-unimplemented
`GetExtraWorkForFinalization`. That is a result from that development stage, not a
promise that this method will always be the next stopping point.

The smoke script is no longer tied to a particular method name. It accepts a
normal exit or a diagnosed unimplemented-method abort; initialization failures
and unexpected crashes should fail the check.

ZeroGC retains allocated objects forever: it neither moves them nor frees them
in a collection. Therefore, there is currently no young-generation collection
that could miss a reference from an old object because a card was not marked.

The current solution's limits are:

- The pinned CoreCLR, Windows x64, workstation GC; the shim rejects Server GC.
- Region and software write-watch mechanisms remain disabled.
- There is no collection consuming these marks.
- The synthetic bounds must not be treated as an accurate description of the
  real managed heap.
- A successful `Hello, World!` does not establish support for all runtime features.

Before adding features that need real address classification, or a collecting
GC, this configuration must be reconsidered. A non-generational GC may have
different write-tracking needs, but must still align its behavior with the
runtime contract.

Mission 13 plans to replace scattered allocations with a reserved area. This
requires real bounds, suitable card metadata with the correct bias, agreed
semantics for generations or their absence, and updates through `StompResize`
when the relevant bounds or tables change. Tests must exercise real card marking
and reading before a collector relies on those marks.

## 12. Where to look in the code

The links below point into the local submodule, pinned to the commit listed at
the beginning of this document:

- [Our configuration and allocator: gc_heap.rs](../crates/gc-rust/src/gc_heap.rs).
- [C ABI callback: runtime.rs](../crates/gc-rust/src/runtime.rs).
- [C++ shim and StompWriteBarrier_Invoke](../native-shim/src/shim.cpp).
- [CoreCLR: InlinedMemmoveGCRefsHelper](../external/dotnet-runtime/src/coreclr/classlibnative/bcltype/arraynative.inl).
- [CoreCLR: InlinedSetCardsAfterBulkCopyHelper](../external/dotnet-runtime/src/coreclr/vm/gchelpers.inl).
- [CoreCLR: JIT_WriteBarrier_PreGrow64](../external/dotnet-runtime/src/coreclr/vm/amd64/JitHelpers_FastWriteBarriers.asm).
- [CoreCLR: StompWriteBarrier parameter installation](../external/dotnet-runtime/src/coreclr/vm/gcenv.ee.cpp).
- [NativeAOT: RhBulkMoveWithWriteBarrier](../external/dotnet-runtime/src/coreclr/nativeaot/Runtime/GCMemoryHelpers.cpp).
- [NativeAOT: InlinedBulkWriteBarrier](../external/dotnet-runtime/src/coreclr/nativeaot/Runtime/GCMemoryHelpers.inl).
- [NativeAOT x64: RhpAssignRef](../external/dotnet-runtime/src/coreclr/runtime/amd64/WriteBarriers.asm).
- [Mission 13: a real reserved area](../tasks/13-reserved-managed-heap.md).
