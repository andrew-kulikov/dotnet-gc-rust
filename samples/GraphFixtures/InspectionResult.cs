using System.Runtime.InteropServices;

namespace GraphFixtures;

[StructLayout(LayoutKind.Sequential, Pack = 4)]
struct InspectionResult
{
    public DiagnosticResult Lifecycle;
    public InspectionObservation Observation;
}
