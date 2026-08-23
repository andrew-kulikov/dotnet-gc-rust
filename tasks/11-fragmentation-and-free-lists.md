# Mission 11 - Make fragmentation choose the next data structure

## Where you are

The model reuses free records through a correct but linear first-fit walk. It
does not merge neighbors or maintain a free-space index.

## The problem

Do not add sophisticated free lists because a roadmap names them. First produce
two observable failures: excessive search work and a request that cannot use
adjacent free records until they are coalesced.

## Observe first

Create deterministic workloads with alternating sizes and deaths. Record records
inspected per allocation, total free bytes, largest free block, and a request
that fails despite enough aggregate adjacent space.

## Your challenge

- [ ] Coalesce neighboring free records during sweep without crossing a region
  boundary.
- [ ] Define the smallest encodable free remainder and consume a whole block when
  splitting would create an invalid fragment.
- [ ] Add a rebuildable free-space index only after retaining the linear allocator
  as a correctness oracle. Start with the simplest buckets that improve the
  measured workload.
- [ ] Ensure every reusable record appears exactly once in the index and no
  allocated record appears there.
- [ ] Compare indexed allocation with linear first-fit on exact fit, split,
  too-small remainder, two-sided coalescing, and repeated reuse.
- [ ] Report search work and fragmentation before and after the change; keep the
  result even if buckets do not improve a workload.

## Checkpoint

The fragmentation scenario succeeds after coalescing, the verifier can rebuild
and compare the index from heap bytes, and randomized indexed results remain
equivalent to the linear oracle.

## Allowed shortcuts

- Bucket boundaries may be simple and manually chosen.
- Intrusive or lock-free lists are not required.
- If measurement does not justify multiple buckets, document the result and keep
  a simpler index.

## Known debt

Model free records are not yet CoreCLR-compatible free objects. Native
synchronization and allocation contexts may change the useful indexing policy.

## What this unlocks

The model now separates authoritative heap representation from replaceable
allocation policy. Mission 12 can complete dependent-handle semantics before the
project returns to native memory.

## Hints

Write the heap verifier before the index. Count unusable fragments separately
from reusable free bytes.
