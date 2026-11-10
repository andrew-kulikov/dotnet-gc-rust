using System.Runtime.InteropServices;

namespace GraphFixtures;

// Values match InspectionStatus in inspection.rs.
enum InspectionStatus : uint
{
    NotRun,
    Reference,
    Null,
    InvalidObject,
    UnsupportedLayout,
    InvalidMetadata,
    CorruptDescriptor,
    InvalidTarget,
    UnknownTarget,
    ExpectationMismatch,
    BudgetExceeded,
}

[StructLayout(LayoutKind.Sequential, Pack = 4)]
struct InspectionObservation
{
    public InspectionStatus Status;
    public uint Size;
    public uint SlotOffset;
    public uint SlotCount;
    public uint NextNodeId;
}
