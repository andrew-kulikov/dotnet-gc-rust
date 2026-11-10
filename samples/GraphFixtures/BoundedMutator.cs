using System;
using System.Threading;

namespace GraphFixtures;

sealed class BoundedMutator
{
    private int progress;
    private int stop;
    private readonly Node first = new();
    private readonly Node second = new();

    public void Run()
    {
        long start = System.Diagnostics.Stopwatch.GetTimestamp();
        // Preallocated graph, bounded lifetime, no lock needed by the owner.
        while (Volatile.Read(ref stop) == 0 &&
               System.Diagnostics.Stopwatch.GetElapsedTime(start) < TimeSpan.FromSeconds(10))
        {
            first.Next = first.Next is null ? second : null;
            Interlocked.Increment(ref progress);
        }
        GC.KeepAlive(first);
        GC.KeepAlive(second);
    }

    public void RequestStop() => Volatile.Write(ref stop, 1);

    public void WaitForProgress()
    {
        int previous = Volatile.Read(ref progress);
        long start = System.Diagnostics.Stopwatch.GetTimestamp();
        while (Volatile.Read(ref progress) == previous)
        {
            if (System.Diagnostics.Stopwatch.GetElapsedTime(start) > TimeSpan.FromSeconds(5))
            {
                throw new InvalidOperationException("Worker made no progress in time.");
            }
            Thread.Yield();
        }
    }
}
