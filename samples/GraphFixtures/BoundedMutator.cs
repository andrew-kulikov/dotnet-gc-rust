using System;
using System.Threading;

namespace GraphFixtures;

sealed class BoundedMutator
{
    public int Progress;
    public int Stop;
    private readonly Node first = new();
    private readonly Node second = new();

    public void Run()
    {
        long start = System.Diagnostics.Stopwatch.GetTimestamp();
        // Preallocated graph, bounded lifetime, no lock needed by the owner.
        while (Volatile.Read(ref Stop) == 0 &&
               System.Diagnostics.Stopwatch.GetElapsedTime(start) < TimeSpan.FromSeconds(10))
        {
            first.Next = first.Next is null ? second : null;
            Interlocked.Increment(ref Progress);
        }
        GC.KeepAlive(first);
        GC.KeepAlive(second);
    }

    public void WaitForProgress(int previous)
    {
        long start = System.Diagnostics.Stopwatch.GetTimestamp();
        while (Volatile.Read(ref Progress) == previous)
        {
            if (System.Diagnostics.Stopwatch.GetElapsedTime(start) > TimeSpan.FromSeconds(5))
            {
                throw new InvalidOperationException("Worker made no progress in time.");
            }
            Thread.Yield();
        }
    }
}
