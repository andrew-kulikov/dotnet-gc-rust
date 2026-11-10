using System.Runtime.InteropServices;

namespace GraphFixtures;

static partial class RustGcNative
{
    // Ordinary P/Invoke: suspension must not use SuppressGCTransition.
    [LibraryImport("gc_rust", EntryPoint = "rust_gc_suspend_ee")]
    public static partial DiagnosticResult SuspendEE();

    [LibraryImport("gc_rust", EntryPoint = "rust_gc_suspend_with_handle")]
    public static partial DiagnosticResult SuspendWithHandle(nint handle, uint failOperation);

    [LibraryImport("gc_rust", EntryPoint = "rust_gc_inspect_node")]
    public static partial InspectionResult InspectNode(nint nodeToInspectHandle, nint knownNextNodeHandle, nint nodeMethodTable, uint expectedNextNodeId);
}
