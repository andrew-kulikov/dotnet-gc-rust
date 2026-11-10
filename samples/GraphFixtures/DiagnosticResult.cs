using System.Runtime.InteropServices;

namespace GraphFixtures;

// Values match Status in suspend.rs.
enum DiagnosticStatus : int
{
    Ok,
    Rejected,
    SuspendFailed,
    OperationFailed,
    RestartFailed,
}

[StructLayout(LayoutKind.Sequential, Pack = 4)]
struct DiagnosticResult
{
    public DiagnosticStatus Status;
    public uint RestartAttempts;
}
