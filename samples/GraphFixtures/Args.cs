using System;
using System.Collections.Generic;
using System.Linq;

namespace GraphFixtures;

public class Args
{
    public required string ScenarioName { get; init; }
    public required LinkDesc[] Links { get; init; }
    public required string[] Roots { get; init; }
    public required HashSet<string> ExpectedLive { get; init; }
    public DiagnosticModeType? DiagnosticMode { get; private set; }

    public static Args Parse(string[] args)
    {
        if (args.Length != 4 && args.Length != 5)
        {
            throw new ArgumentException("Expected a scenario name, edges, roots, live labels, and an optional diagnostic mode");
        }

        var res = new Args
        {
            ScenarioName = args[0],
            Links = args[1]
                .Split(',')
                .Select(link =>
                {
                    var parts = link.Split(':');
                    return new LinkDesc { From = parts[0], To = parts[1] };
                })
                .ToArray(),
            Roots = args[2] == "-" ? [] : args[2].Split(','),
            ExpectedLive = args[3] == "-"
                ? []
                : [.. args[3].Split(',')],
        };

        if (args.Length == 5)
        {
            res.DiagnosticMode = args[4] switch
            {
                "--suspend" => DiagnosticModeType.Suspend,
                "--suspend-repeat" => DiagnosticModeType.SuspendRepeat,
                "--inspect" => DiagnosticModeType.InspectNode,
                _ => throw new ArgumentException($"Unexpected diagnostic type: {args[4]}")
            };
        }

        return res;
    }
}

public class LinkDesc
{
    public required string From { get; set; }
    public required string To { get; set; }
}

public enum DiagnosticModeType
{
    Suspend,
    SuspendRepeat,
    InspectNode
}