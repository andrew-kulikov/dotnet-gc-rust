using System.Runtime.InteropServices;

if (args.Length == 0)
{
    Console.WriteLine("Hello, World!");
    return;
}

if (args.Length != 1)
{
    throw new ArgumentException("Expected one scenario name.");
}

Console.WriteLine("scenario starting");

switch (args[0])
{
    case "small":
        AllocateArrays(512, 128);
        break;
    case "large":
        var large = new byte[128 * 1024];
        large[^1] = 1;
        GC.KeepAlive(large);
        break;
    case "threads":
        var first = new Thread(() => AllocateArrays(256, 128));
        var second = new Thread(() => AllocateArrays(256, 128));
        first.Start();
        second.Start();
        first.Join();
        second.Join();
        break;
    case "pin":
        var buffer = new byte[128];
        var handle = GCHandle.Alloc(buffer, GCHandleType.Pinned);
        try
        {
            var address = handle.AddrOfPinnedObject();
            if (address == IntPtr.Zero)
                throw new Exception("Pinned address was null.");
            Marshal.WriteByte(address, 0x5A);
            if (buffer[0] != 0x5A)
                throw new Exception("Pinned write did not reach the buffer.");
        }
        finally
        {
            handle.Free();
        }
        GC.KeepAlive(buffer);
        break;
    case "finalize":
        var finalizable = new Finalizable();
        GC.ReRegisterForFinalize(finalizable);
        GC.KeepAlive(finalizable);
        break;
    case "exhaust":
        AllocateArrays(4096, 1024);
        break;
    default:
        throw new ArgumentException($"Unknown scenario: {args[0]}");
}

Console.WriteLine($"scenario {args[0]} ok");

static void AllocateArrays(int count, int length)
{
    for (int i = 0; i < count; i++)
    {
        var array = new byte[length];
        array[0] = (byte)i;
        GC.KeepAlive(array);
    }
}

sealed class Finalizable
{
    ~Finalizable() { }
}
