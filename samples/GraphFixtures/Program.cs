using System.Runtime.CompilerServices;

if (args.Length != 4)
{
    throw new ArgumentException("Expected a scenario name, edges, roots, and live labels.");
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

Console.WriteLine($"fixture {scenarioName} constructed");
Observe(scenarioName, labels, nodes, roots, expectedLive);
Console.WriteLine($"fixture {scenarioName} continuation");

// Retain even disconnected nodes: fixture roots are only inputs to reachability.
GC.KeepAlive(roots);
GC.KeepAlive(nodes);

// A stable control point for later native diagnostics. No address is exported.
[MethodImpl(MethodImplOptions.NoInlining)]
static void Observe(string name, string[] labels, Node[] nodes, Node[] roots, string expectedLive)
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

    Console.WriteLine($"fixture {name} observation declared-only raw-handoff=unsupported");
    // Interpolated handlers use ArrayPool/ConditionalWeakTable on this runtime;
    // ZeroGC does not implement the dependent handles needed by that path.
    Console.WriteLine(string.Join(' ', "graph", name, edges, rootLabels, actualLive));
    GC.KeepAlive(roots);
    GC.KeepAlive(nodes);
}

sealed class Node
{
    public Node? Next;
}
