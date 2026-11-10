using System;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.CompilerServices;

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

        Log.Info("Managed execution continued after Observe.");

        Node ResolveNode(string label)
        {
            return nodes[Array.IndexOf(labels, label)];
        }
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
            if (result.Status != DiagnosticStatus.Ok || result.RestartAttempts != 1)
            {
                throw new InvalidOperationException("The suspension failed or did not restart managed execution exactly once.");
            }
            Log.Info("Managed execution was restarted once.");
        }
        else if (args.DiagnosticMode == DiagnosticModeType.SuspendRepeat)
        {
            Diagnostics.RepeatSuspension(nodes[0]);
        }
        else if (args.DiagnosticMode == DiagnosticModeType.InspectNode)
        {
            Diagnostics.InspectFixture();
        }

        Log.Info("Edges:");
        foreach (var node in nodes)
        {
            Log.Info($"{GetLabel(node)} -> {GetLabel(node.Next)}");
        }
        Log.Info("Roots:");
        foreach (var node in roots)
        {
            Log.Info(GetLabel(node));
        }
        Log.Info($"Reachable nodes: {string.Join(',', liveLabels)}");
        GC.KeepAlive(roots);
        GC.KeepAlive(nodes);

        string GetLabel(Node? node)
        {
            return node is null
                ? "-"
                : labels[Array.IndexOf(nodes, node)];
        }
    }

}
