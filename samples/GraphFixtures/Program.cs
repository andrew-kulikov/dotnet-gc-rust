using System.Runtime.CompilerServices;

if (args.Length != 4)
    throw new ArgumentException("Expected a scenario name, edges, roots, and live labels.");

var links = args[1].Split(',').Select(link => link.Split(':')).ToArray();
var labels = links.Select(link => link[0]).ToArray();
var nodes = labels.Select(_ => new Node()).ToArray();
Node Resolve(string label) => nodes[Array.IndexOf(labels, label)];
for (int i = 0; i < nodes.Length; i++)
    nodes[i].Next = links[i][1] == "-" ? null : Resolve(links[i][1]);
var roots = args[2] == "-" ? [] : args[2].Split(',').Select(Resolve).ToArray();

Console.WriteLine($"fixture {args[0]} constructed");
Observe(args[0], labels, nodes, roots, args[3]);
Console.WriteLine($"fixture {args[0]} continuation");
GC.KeepAlive(roots);
GC.KeepAlive(nodes);

// A stable control point for later native diagnostics. No address is exported.
[MethodImpl(MethodImplOptions.NoInlining)]
static void Observe(string name, string[] labels, Node[] nodes, Node[] roots, string expected)
{
    string Label(Node? node) => node is null ? "-" : labels[Array.IndexOf(nodes, node)];
    var edges = string.Join(',', nodes.Select((node, i) => $"{labels[i]}:{Label(node.Next)}"));
    var rootLabels = roots.Length == 0 ? "-" : string.Join(',', roots.Select(root => Label(root)));
    var reachable = new HashSet<Node>();
    foreach (var root in roots)
        for (Node? node = root; node is not null && reachable.Add(node); node = node.Next) { }
    var live = string.Join(',', reachable.Select(node => Label(node)).Order(StringComparer.Ordinal));
    if (live.Length == 0)
        live = "-";
    if (live != expected)
        throw new InvalidOperationException($"Expected {expected}, observed {live}.");

    Console.WriteLine($"fixture {name} observation declared-only raw-handoff=unsupported");
    Console.WriteLine($"graph {name} {edges} {rootLabels} {live}");
    GC.KeepAlive(roots);
    GC.KeepAlive(nodes);
}

sealed class Node
{
    public Node? Next;
}
