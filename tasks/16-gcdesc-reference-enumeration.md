# Mission 16 - Enumerate real outgoing references

## Where you are

The collector can walk supported managed objects and determine their byte sizes.
It cannot yet distinguish reference fields from non-reference payload.

## The problem

Tracing every pointer-looking word would retain garbage and may dereference
invalid values. CoreCLR method tables carry `GCDesc` metadata describing the
actual managed reference series. The encoding has multiple forms and must be
validated before walking memory.

## Observe first

Start with a class containing one reference and compare its expected field with
the pinned `GCDesc` representation. Add fixtures only when the current decoder
passes the simpler case.

## Your challenge

- [ ] Expose a callback-style operation that visits reference slots of one
  already validated object without allocating in the hot path.
- [ ] Decode the positive forms required by simple classes and inheritance, then
  add reference arrays, nested value types, arrays of structs containing
  references, and required negative forms incrementally.
- [ ] Validate series counts, signed values, offsets, multiplication, object
  bounds, and guaranteed decoder progress before visiting a slot.
- [ ] Adapt visited slots to the model tracer's reference boundary without
  sharing the model's teaching layout.
- [ ] Compare every supported fixture with an independent expectation or
  diagnostic tool; do not use the production decoder as its own oracle.
- [ ] Fuzz or generate malformed descriptors and require bounded failure.

## Checkpoint

Reference enumeration exactly matches all claimed managed fixtures, visits no
non-reference field, allocates no collection-time side list, and rejects corrupt
descriptors without walking outside the object.

## Allowed shortcuts

- Add descriptor forms only when a fixture or pinned runtime startup requires
  them, provided unsupported forms fail safely.
- DAC may help diagnostics but cannot be required for correctness.

## Known debt

Outgoing edges are known, but no trustworthy runtime root set or stop-the-world
window exists yet.

## What this unlocks

Mission 17 can establish a safe suspension lifecycle before any real heap bytes
are mutated by collection.

## Hints

Keep “decode descriptor” and “visit validated slots” separable. Ensure every loop
has a proven bound or strictly advancing cursor.
