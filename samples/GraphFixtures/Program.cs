using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

if (args.Length != 4 && (args.Length != 5 || args[4] != "--suspend"))
{
    throw new ArgumentException("Expected a scenario name, edges, roots, live labels, and optional --suspend.");
}

var scenarioName = args[0];
var links = args[1].Split(',').Select(link => link.Split(':')).ToArray();
var labels = links.Select(link => link[0]).ToArray();
// Labels stay in a sidecar array so each Node has only its Next reference.
var nodes = labels.Select(_ => new Node()).ToArray();
var expectedLive = args[3];

Node ResolveNode(string label) => nodes[Array.IndexOf(labels, label)];

for (int i = 0; i < nodes.Length; i++)
{
    var targetLabel = links[i][1];
    nodes[i].Next = targetLabel == "-" ? null : ResolveNode(targetLabel);
}

var roots = args[2] == "-" ? [] : args[2].Split(',').Select(ResolveNode).ToArray();

Log($"fixture '{scenarioName}' constructed");
Observe(scenarioName, labels, nodes, roots, expectedLive, args.Length == 5);
Log($"fixture '{scenarioName}' continuation");

// Retain even disconnected nodes: fixture roots are only inputs to reachability.
GC.KeepAlive(roots);
GC.KeepAlive(nodes);

static void Log(string message)
{
    Console.WriteLine($"dotnet: {message}");
}

// A stable control point for later native diagnostics. No address is exported.
[MethodImpl(MethodImplOptions.NoInlining)]
static void Observe(string name, string[] labels, Node[] nodes, Node[] roots, string expectedLive, bool suspend)
{
    string GetLabel(Node? node) => node is null ? "-" : labels[Array.IndexOf(nodes, node)];

    var edges = string.Join(',', nodes.Select((node, i) => $"{labels[i]}:{GetLabel(node.Next)}"));
    var rootLabels = roots.Length == 0 ? "-" : string.Join(',', roots.Select(root => GetLabel(root)));

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
    var actualLive = reachable.Count == 0 ? "-" : string.Join(',', liveLabels);
    if (actualLive != expectedLive)
    {
        throw new InvalidOperationException($"Expected {expectedLive}, observed {actualLive}.");
    }

    Log($"fixture '{name}' observation declared-only raw-handoff=unsupported");
    if (suspend)
    {
        var result = RustGcNative.SuspendEE();
        if (result.Status != 0 || result.RestartAttempts != 1)
        {
            throw new InvalidOperationException($"Empty diagnostic cycle failed: status={result.Status} restart-attempts={result.RestartAttempts}");
        }
        // Format only after restart; never call managed code inside the window.
        Log($"fixture '{name}' diagnostic suspension completed restart-attempts=1");
    }
    // Interpolated handlers use ArrayPool/ConditionalWeakTable on this runtime;
    // ZeroGC does not implement the dependent handles needed by that path.
    Log(string.Join(' ', "graph", "'" + name + "'", edges, rootLabels, actualLive));
    GC.KeepAlive(roots);
    GC.KeepAlive(nodes);
}

sealed class Node
{
    public Node? Next;
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
}
