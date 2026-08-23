# Loader-boundary decision

Status: accepted by Mission 01.

CoreCLR exposes the standalone-GC contract as C++ virtual interfaces. Rust does
not implement or reconstruct those compiler-specific vtables. The native C++
shim is the only component that owns this ABI adaptation: it implements the
pinned CoreCLR interfaces and forwards each operation through a narrow C ABI.
It must not own collector state or choose collection policy.

Rust owns collector state, its lifetime, and all allocation and collection
policy. Values crossing the C ABI use explicitly sized C-compatible types;
ownership and validity are defined by that API rather than by C++ or Rust object
layout.

No language unwind may cross the boundary. Rust FFI entry points must contain
panics and translate recoverable failures into an explicit result; if a panic
cannot be contained safely, the process aborts. C++ ABI entry points must not
allow an exception to escape: they translate recoverable failures into the
declared result, or terminate for an unrecoverable failure. CoreCLR must never
observe a Rust panic or a C++ exception.

This keeps the shim replaceable and confines the unstable C++ ABI to the one
component compiled against the pinned runtime headers.
