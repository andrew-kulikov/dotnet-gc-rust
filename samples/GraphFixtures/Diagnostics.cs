using System;
using System.Runtime.InteropServices;
using System.Threading;

namespace GraphFixtures;

static class Diagnostics
{
    // Logical IDs used by this inspection, not object addresses.
    private const uint NullNodeId = 0;
    private const uint KnownNextNodeId = 1;

    public static void InspectFixture()
    {
        Type nodeType = typeof(Node);
        // The runtime's MethodTable for Node lets Rust validate the object's type.
        nint nodeMethodTable = nodeType.TypeHandle.Value;
        var nodeToInspect = new Node();
        var knownNextNode = new Node();
        var nodeToInspectHandle = GCHandle.Alloc(nodeToInspect, GCHandleType.Normal);
        GCHandle knownNextNodeHandle = default;
        try
        {
            knownNextNodeHandle = GCHandle.Alloc(knownNextNode, GCHandleType.Normal);
            RunWithWorker(mutator =>
            {
                void Inspect(Node? nextNode, bool expectMismatch = false)
                {
                    mutator.WaitForProgress();

                    nodeToInspect.Next = nextNode;
                    uint configuredNextNodeId = nextNode is null
                        ? NullNodeId
                        : KnownNextNodeId;
                    // The mismatch case asks Rust to expect B when Next is null.
                    uint expectedNextNodeId = expectMismatch
                        ? KnownNextNodeId
                        : configuredNextNodeId;

                    var result = RustGcNative.InspectNode(
                        GCHandle.ToIntPtr(nodeToInspectHandle),
                        GCHandle.ToIntPtr(knownNextNodeHandle),
                        nodeMethodTable,
                        expectedNextNodeId);

                    var observation = result.Observation;
                    var lifecycle = result.Lifecycle;
                    var expectedStatus = expectMismatch
                        ? InspectionStatus.ExpectationMismatch
                        : nextNode is null
                            ? InspectionStatus.Null
                            : InspectionStatus.Reference;
                    var expectedLifecycle = expectMismatch ? DiagnosticStatus.OperationFailed : DiagnosticStatus.Ok;
                    string details = $"nextId={observation.NextNodeId}, status={observation.Status}, size={observation.Size}, slot={observation.SlotOffset}";

                    if (lifecycle.Status != expectedLifecycle
                        || lifecycle.RestartAttempts != 1
                        || observation.Status != expectedStatus
                        || observation.NextNodeId != configuredNextNodeId
                        || observation.Size != 24
                        || observation.SlotOffset != 8
                        || observation.SlotCount != 1)
                    {
                        throw new InvalidOperationException($"Unexpected inspection: {details}, slots={observation.SlotCount}, lifecycle={lifecycle.Status}, restarts={lifecycle.RestartAttempts}");
                    }
                    Log.Info($"Inspection: {details}");

                    mutator.WaitForProgress();
                }

                Inspect(knownNextNode);
                Inspect(null);
                Inspect(knownNextNode);
                Inspect(null, expectMismatch: true);
                Inspect(knownNextNode);
                Inspect(null);
                Inspect(knownNextNode);
                Inspect(null);
            });
            if (!ReferenceEquals(nodeToInspectHandle.Target, nodeToInspect) || !ReferenceEquals(knownNextNodeHandle.Target, knownNextNode) ||
                GC.CollectionCount(0) != 0)
            {
                throw new InvalidOperationException("Inspection changed a handle target or completed a collection.");
            }
        }
        finally
        {
            if (knownNextNodeHandle.IsAllocated)
            {
                knownNextNodeHandle.Free();
            }
            nodeToInspectHandle.Free();
            // Keep the type alive while Rust uses its MethodTable.
            GC.KeepAlive(nodeType);
        }
        Log.Info("Inspection managed handles freed.");
    }

    public static void RepeatSuspension(Node target)
    {
        // Allocate before native entry. The token is a handle, never an object address.
        var handle = GCHandle.Alloc(target, GCHandleType.Normal);
        try
        {
            CheckHandleCycle(handle, target, failOperation: false);
            Log.Info("The handle was checked and still points to the expected object.");
            Log.Info("Managed execution was restarted once.");
            RunWithWorker(mutator =>
            {
                void CheckCycle(bool failOperation = false)
                {
                    mutator.WaitForProgress();
                    CheckHandleCycle(handle, target, failOperation);
                    mutator.WaitForProgress();
                }

                CheckCycle();
                CheckCycle();
                CheckCycle();
                CheckCycle(failOperation: true);
                CheckCycle();
                CheckCycle();
                CheckCycle();
                CheckCycle();
            });
            if (GC.CollectionCount(0) != 0)
            {
                throw new InvalidOperationException("The suspension unexpectedly completed a garbage collection.");
            }
        }
        finally
        {
            handle.Free();
        }
        Log.Info("Handle released.");
    }

    // Both diagnostics exercise the same suspend/restart lifecycle while a
    // second managed thread is running. Keep the operation itself separate.
    static void RunWithWorker(Action<BoundedMutator> runCycles)
    {
        var mutator = new BoundedMutator();
        var worker = new Thread(mutator.Run);
        worker.Start();
        try
        {
            runCycles(mutator);
        }
        finally
        {
            mutator.RequestStop();
            if (!worker.Join(TimeSpan.FromSeconds(5)))
            {
                throw new InvalidOperationException("Worker did not stop in time.");
            }
        }
        Log.Info("Worker made progress before and after each cycle and stopped.");
    }

    static void CheckHandleCycle(GCHandle handle, Node target, bool failOperation)
    {
        if (!ReferenceEquals(handle.Target, target))
        {
            throw new InvalidOperationException("Handle target changed before suspension.");
        }
        var result = RustGcNative.SuspendWithHandle(GCHandle.ToIntPtr(handle), failOperation ? 1u : 0u);
        if (result.Status != (failOperation ? DiagnosticStatus.OperationFailed : DiagnosticStatus.Ok) || result.RestartAttempts != 1)
        {
            throw new InvalidOperationException("The handle check failed or managed execution was not restarted exactly once.");
        }
        if (!ReferenceEquals(handle.Target, target))
        {
            throw new InvalidOperationException("Handle target changed after restart.");
        }
    }
}
