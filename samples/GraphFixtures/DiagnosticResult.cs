using System.Runtime.InteropServices;

namespace GraphFixtures;

[StructLayout(LayoutKind.Sequential, Pack = 4)]
struct DiagnosticResult
{
    public int Status;
    public uint RestartAttempts;
}
