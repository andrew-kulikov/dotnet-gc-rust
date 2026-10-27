using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

if (args.Length != 4 && (args.Length != 5 || args[4] is not ("--suspend" or "--suspend-repeat")))
{
    throw new ArgumentException("Expected a scenario name, edges, roots, live labels, and optional --suspend or --suspend-repeat.");
}

var scenarioName = args[0];
var links = args[1].Split(',').Select(link => link.Split(':')).ToArray();
var labels = links.Select(link => link[0]).ToArray();
// Labels stay in a sidecar array so each Node has only its Next reference.
var nodes = labels.Select(_ => new Node()).ToArray();
var expectedLive = args[3];

Node ResolveNode(string label)
{
    return nodes[Array.IndexOf(labels, label)];
}

for (int i = 0; i < nodes.Length; i++)
{
    var targetLabel = links[i][1];
    nodes[i].Next = targetLabel == "-" ? null : ResolveNode(targetLabel);
}

var roots = args[2] == "-" ? [] : args[2].Split(',').Select(ResolveNode).ToArray();

Observe(scenarioName, labels, nodes, roots, expectedLive, args.Length == 5 ? args[4] : null);
Log($"Managed execution continued after '{scenarioName}'.");

// Retain even disconnected nodes: fixture roots are only inputs to reachability.
GC.KeepAlive(roots);
GC.KeepAlive(nodes);

static void Log(string message)
{
    Console.WriteLine($"dotnet: {message}");
}

// A stable control point for later native diagnostics. No address is exported.
[MethodImpl(MethodImplOptions.NoInlining)]
static void Observe(string name, string[] labels, Node[] nodes, Node[] roots, string expectedLive, string? suspension)
{
    string GetLabel(Node? node)
    {
        return node is null
            ? "-"
            : labels[Array.IndexOf(nodes, node)];
    }

    var edges = nodes.Select((node, i) => $"{labels[i]}:{GetLabel(node.Next)}")
        .Aggregate((list, edge) => $"{list},{edge}");
    var rootLabels = roots.Length == 0 ? "-" : roots.Select(root => GetLabel(root))
        .Aggregate((list, label) => $"{list},{label}");

    var reachable = new HashSet<Node>();
    foreach (var root in roots)
    {
        Node? node = root;
        // Stop at visited nodes to handle cycles, duplicate roots, and shared tails.
        while (node is not null && reachable.Add(node))
        {
            node = node.Next;
        }
    }

    var liveLabels = reachable.Select(node => GetLabel(node)).Order(StringComparer.Ordinal);
    var actualLive = reachable.Count == 0
        ? "-"
        : liveLabels.Aggregate((list, label) => $"{list},{label}");
    if (actualLive != expectedLive)
    {
        throw new InvalidOperationException($"Expected reachable nodes: {expectedLive}\nObserved reachable nodes: {actualLive}");
    }

    if (suspension == "--suspend-repeat")
    {
        RepeatSuspension(nodes[0]);
    }
    else if (suspension == "--suspend")
    {
        var result = RustGcNative.SuspendEE();
        if (result.Status != 0 || result.RestartAttempts != 1)
        {
            throw new InvalidOperationException("The suspension failed or did not restart managed execution exactly once.");
        }
        // Format only after restart; never call managed code inside the window.
        Log("Managed execution was restarted once.");
    }
    Log($"Graph '{name}':");
    Log($"Edges: {edges}");
    Log($"Roots: {rootLabels}");
    Log($"Reachable nodes: {actualLive}");
    GC.KeepAlive(roots);
    GC.KeepAlive(nodes);
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

sealed class Node
{
    public Node? Next;
}

sealed class BoundedMutator
{
    public int Progress;
    public int Stop;
    private readonly Node first = new();
    private readonly Node second = new();

    public void Run()
    {
        long start = System.Diagnostics.Stopwatch.GetTimestamp();
        // Preallocated graph, bounded lifetime, no lock needed by the owner.
        while (Volatile.Read(ref Stop) == 0 &&
               System.Diagnostics.Stopwatch.GetElapsedTime(start) < TimeSpan.FromSeconds(10))
        {
            first.Next = first.Next is null ? second : null;
            Interlocked.Increment(ref Progress);
        }
        GC.KeepAlive(first);
        GC.KeepAlive(second);
    }

    public void WaitForProgress(int previous)
    {
        long start = System.Diagnostics.Stopwatch.GetTimestamp();
        while (Volatile.Read(ref Progress) == previous)
        {
            if (System.Diagnostics.Stopwatch.GetElapsedTime(start) > TimeSpan.FromSeconds(5))
            {
                throw new InvalidOperationException("Worker made no progress in time.");
            }
            Thread.Yield();
        }
    }
}

[StructLayout(LayoutKind.Sequential, Pack = 4)]
struct DiagnosticResult
{
    public int Status;
    public uint RestartAttempts;
}

static partial class RustGcNative
{
    // Ordinary P/Invoke: suspension must not use SuppressGCTransition.
    [LibraryImport("gc_rust", EntryPoint = "rust_gc_suspend_ee")]
    public static partial DiagnosticResult SuspendEE();

    [LibraryImport("gc_rust", EntryPoint = "rust_gc_suspend_with_handle")]
    public static partial DiagnosticResult SuspendWithHandle(nint handle, uint failOperation);
}
