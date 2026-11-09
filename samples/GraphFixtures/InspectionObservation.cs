using System.Runtime.InteropServices;

namespace GraphFixtures;

[StructLayout(LayoutKind.Sequential, Pack = 4)]
struct InspectionObservation
{
    public uint Status;
    public uint Size;
    public uint SlotOffset;
    public uint SlotCount;
    public uint TargetId;
}
