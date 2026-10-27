using System.Runtime;
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
            {
                throw new Exception("Pinned address was null.");
            }
            Marshal.WriteByte(address, 0x5A);
            if (buffer[0] != 0x5A)
            {
                throw new Exception("Pinned write did not reach the buffer.");
            }
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
    case "dependent":
        CheckDependentHandles();
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

static void CheckDependentHandles()
{
    var target = new object();
    var dependent = new object();
    var replacement = new object();
    var handle = new DependentHandle(target, dependent);
    try
    {
        CheckDependentPair(handle, target, dependent);
        handle.Dependent = replacement;
        CheckDependentPair(handle, target, replacement);
        handle.Dependent = null;
        CheckDependentPair(handle, target, null);
        handle.Dependent = replacement;
        handle.Target = null;
        CheckDependentPair(handle, null, null);
        handle.Dependent = dependent;
        CheckDependentPair(handle, null, null);
    }
    finally
    {
        handle.Dispose();
    }
    if (handle.IsAllocated)
    {
        throw new Exception("Dependent handle was not released.");
    }
    handle.Dispose();

    foreach (var pair in new (object? Target, object? Dependent)[]
    {
        (null, dependent), (target, null), (null, null), (target, target),
    })
    {
        var nullable = new DependentHandle(pair.Target, pair.Dependent);
        try
        {
            CheckDependentPair(nullable, pair.Target, pair.Target is null ? null : pair.Dependent);
        }
        finally
        {
            nullable.Dispose();
        }
    }
    GC.KeepAlive(target);
    GC.KeepAlive(dependent);
    GC.KeepAlive(replacement);
}

static void CheckDependentPair(DependentHandle handle, object? target, object? dependent)
{
    var pair = handle.TargetAndDependent;
    if (!handle.IsAllocated || !ReferenceEquals(handle.Target, target) ||
        !ReferenceEquals(handle.Dependent, dependent) ||
        !ReferenceEquals(pair.Target, target) || !ReferenceEquals(pair.Dependent, dependent))
    {
        throw new Exception("Dependent handle returned an unexpected pair.");
    }
}

sealed class Finalizable
{
    ~Finalizable() { }
}
