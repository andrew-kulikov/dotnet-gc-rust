using System;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Threading;

namespace GraphFixtures;

public static class Program
{
    public static void Main(string[] args)
    {
        var parsedArgs = Args.Parse(args);

        var labels = parsedArgs.Links.Select(link => link.From).ToArray();
        // Labels stay in a sidecar array so each Node has only its Next reference.
        var nodes = labels.Select(_ => new Node()).ToArray();
        var roots = parsedArgs.Roots.Select(ResolveNode).ToArray();

        for (int i = 0; i < nodes.Length; i++)
        {
            var targetLabel = parsedArgs.Links[i].To;
            nodes[i].Next = targetLabel == "-" ? null : ResolveNode(targetLabel);
        }


        Observe(parsedArgs, labels, nodes, roots);

        Log($"Managed execution continued after Observe.");

        Node ResolveNode(string label)
        {
            return nodes[Array.IndexOf(labels, label)];
        }
    }

    private static void Log(string message)
    {
        Console.WriteLine($"dotnet: {message}");
    }

    [MethodImpl(MethodImplOptions.NoInlining)]
    static void Observe(Args args, string[] labels, Node[] nodes, Node[] roots)
    {
        var reachable = new HashSet<Node>();
        foreach (var root in roots)
        {
            Node? node = root;
            while (node is not null && reachable.Add(node))
            {
                node = node.Next;
            }
        }

        var liveLabels = new HashSet<string>(reachable.Select(node => GetLabel(node)));
        if (!liveLabels.SetEquals(args.ExpectedLive))
        {
            throw new InvalidOperationException($"Expected: {string.Join(',', args.ExpectedLive)}. Actual: {string.Join(',', liveLabels)}");
        }

        if (args.DiagnosticMode == DiagnosticModeType.Suspend)
        {
            var result = RustGcNative.SuspendEE();
            if (result.Status != 0 || result.RestartAttempts != 1)
            {
                throw new InvalidOperationException("The suspension failed or did not restart managed execution exactly once.");
            }
            Log("Managed execution was restarted once.");
        }
        else if (args.DiagnosticMode == DiagnosticModeType.SuspendRepeat)
        {
            RepeatSuspension(nodes[0]);
        }
        else if (args.DiagnosticMode == DiagnosticModeType.InspectNode)
        {
            InspectFixture();
        }

        Log($"Edges:");
        foreach (var node in nodes)
        {
            Log($"{GetLabel(node)} -> {GetLabel(node.Next)}");
        }
        Log($"Roots:");
        foreach (var node in roots)
        {
            Log(GetLabel(node));
        }
        Log($"Reachable nodes: {string.Join(',', liveLabels)}");
        GC.KeepAlive(roots);
        GC.KeepAlive(nodes);

        string GetLabel(Node? node)
        {
            return node is null
                ? "-"
                : labels[Array.IndexOf(nodes, node)];
        }
    }

    static void InspectFixture()
    {
        Type nodeType = typeof(Node);
        nint witness = nodeType.TypeHandle.Value;
        var source = new Node();
        var candidate = new Node();
        source.Next = candidate;
        var sourceHandle = GCHandle.Alloc(source, GCHandleType.Normal);
        GCHandle candidateHandle = default;
        try
        {
            candidateHandle = GCHandle.Alloc(candidate, GCHandleType.Normal);
            var mutator = new BoundedMutator();
            var worker = new Thread(mutator.Run);
            worker.Start();
            try
            {
                for (int cycle = 1; cycle <= 8; cycle++)
                {
                    mutator.WaitForProgress(Volatile.Read(ref mutator.Progress));
                    Log($"Inspection worker made progress before cycle {cycle}.");
                    bool hasTarget = cycle % 2 == 1;
                    source.Next = hasTarget ? candidate : null;
                    bool mismatch = cycle == 4;
                    uint actual = hasTarget ? 1u : 0u;
                    CheckInspection(sourceHandle, candidateHandle, witness,
                        expected: mismatch ? 1u : actual, actual: actual, mismatch: mismatch);
                    mutator.WaitForProgress(Volatile.Read(ref mutator.Progress));
                    Log($"Inspection worker made progress after cycle {cycle}.");
                }
            }
            finally
            {
                Volatile.Write(ref mutator.Stop, 1);
                if (!worker.Join(TimeSpan.FromSeconds(5)))
                {
                    throw new InvalidOperationException("Inspection worker did not stop in time.");
                }
            }
            Log("Inspection worker stopped.");
            if (!ReferenceEquals(sourceHandle.Target, source) || !ReferenceEquals(candidateHandle.Target, candidate) ||
                GC.CollectionCount(0) != 0)
            {
                throw new InvalidOperationException("Inspection changed a handle target or completed a collection.");
            }
        }
        finally
        {
            if (candidateHandle.IsAllocated)
            {
                candidateHandle.Free();
            }
            sourceHandle.Free();
            GC.KeepAlive(nodeType);
            GC.KeepAlive(source);
            GC.KeepAlive(candidate);
        }
        if (sourceHandle.IsAllocated || candidateHandle.IsAllocated)
        {
            throw new InvalidOperationException("Inspection managed handles were not freed.");
        }
        Log("Inspection managed handles freed.");
    }

    static void CheckInspection(GCHandle source, GCHandle candidate, nint witness, uint expected, uint actual, bool mismatch)
    {
        var result = RustGcNative.InspectNode(GCHandle.ToIntPtr(source), GCHandle.ToIntPtr(candidate), witness, expected);
        uint status = mismatch ? 9u : actual == 0 ? 2u : 1u;
        if (result.Lifecycle.Status != (mismatch ? 3 : 0) || result.Lifecycle.RestartAttempts != 1 ||
            result.Observation.Status != status || result.Observation.Size != 24 ||
            result.Observation.SlotOffset != 8 || result.Observation.SlotCount != 1 ||
            result.Observation.TargetId != actual)
        {
            throw new InvalidOperationException($"Unexpected inspection: lifecycle={result.Lifecycle.Status}, restart={result.Lifecycle.RestartAttempts}, status={result.Observation.Status}, size={result.Observation.Size}, offset={result.Observation.SlotOffset}, count={result.Observation.SlotCount}, target={result.Observation.TargetId}.");
        }
        Log($"Inspection: size=24 slot=8 count=1 target={(actual == 0 ? "null" : "B")} status={(mismatch ? "mismatch" : "ok")} restart=1.");
    }

    static void RepeatSuspension(Node target)
    {
        // Allocate before native entry. The token is a handle, never an object address.
        var handle = GCHandle.Alloc(target, GCHandleType.Normal);
        try
        {
            CheckHandleCycle(handle, target, failOperation: false);
            Log("The handle was checked and still points to the expected object.");
            Log("Managed execution was restarted once.");
            var mutator = new BoundedMutator();
            var worker = new Thread(mutator.Run);
            worker.Start();
            try
            {
                for (int cycle = 1; cycle <= 8; cycle++)
                {
                    mutator.WaitForProgress(Volatile.Read(ref mutator.Progress));
                    Log($"Worker made progress before cycle {cycle}.");
                    // The fourth cycle fails after resolving the handle, before restart.
                    CheckHandleCycle(handle, target, failOperation: cycle == 4);
                    if (cycle == 4)
                    {
                        Log($"Cycle {cycle} reported the expected operation failure.");
                    }
                    else
                    {
                        Log($"Cycle {cycle} completed.");
                    }
                    Log("Managed execution was restarted once.");
                    mutator.WaitForProgress(Volatile.Read(ref mutator.Progress));
                    Log($"Worker made progress after cycle {cycle}.");
                }
            }
            finally
            {
                Volatile.Write(ref mutator.Stop, 1);
                if (!worker.Join(TimeSpan.FromSeconds(5)))
                {
                    throw new InvalidOperationException("Worker did not stop in time.");
                }
            }
            Log("Worker stopped.");
            if (GC.CollectionCount(0) != 0)
            {
                throw new InvalidOperationException("The suspension unexpectedly completed a garbage collection.");
            }
        }
        finally
        {
            handle.Free();
        }
        if (handle.IsAllocated)
        {
            throw new InvalidOperationException("Managed handle was not released.");
        }
        Log("Handle released.");
    }

    static void CheckHandleCycle(GCHandle handle, Node target, bool failOperation)
    {
        if (!ReferenceEquals(handle.Target, target))
        {
            throw new InvalidOperationException("Handle target changed before suspension.");
        }
        var result = RustGcNative.SuspendWithHandle(GCHandle.ToIntPtr(handle), failOperation ? 1u : 0u);
        if (result.Status != (failOperation ? 3 : 0) || result.RestartAttempts != 1)
        {
            throw new InvalidOperationException("The handle check failed or managed execution was not restarted exactly once.");
        }
        if (!ReferenceEquals(handle.Target, target))
        {
            throw new InvalidOperationException("Handle target changed after restart.");
        }
    }
}
