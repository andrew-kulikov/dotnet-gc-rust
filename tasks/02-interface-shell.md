# Mission 02 - Let initialization succeed

## Where you are

CoreCLR reaches `GC_Initialize`, crosses into Rust, and receives an intentional
failure. The shim has no `IGCHeap` or handle-manager objects.

## The problem

Returning `S_OK` without valid, stable interface objects would let CoreCLR call
garbage pointers. Implementing every interface method as if the final collector
already existed would be equally misleading. The useful next checkpoint is to
accept initialization, continue just far enough to observe the next runtime
request, and fail there by name.

## Observe first

From the pinned headers, inventory the pure virtual methods of `IGCHeap`,
`IGCHandleManager`, and `IGCHandleStore`. Group them as:

- required to construct and return the interface;
- observed during the current startup trace;
- not yet observed;
- explicitly outside the supported Windows x64 workstation configuration.

Do not infer that an unobserved method is optional forever.

## Your challenge

- [x] Create C++ interface objects with process-long stable addresses and exact
  signatures from the pinned headers.
- [x] Store the supplied `IGCToCLR` pointer in the adapter without exposing its
  C++ layout to Rust.
- [x] Move any mutable collector state behind an opaque Rust-owned C ABI handle;
  do not duplicate heap policy or object state in C++.
- [x] Make each unsupported method terminate through one diagnostic path that
  includes the method name. A stub must not return a plausible success value.
- [x] Return the stable `IGCHeap` and `IGCHandleManager` pointers from
  `GC_Initialize` and return `S_OK`.
- [x] Reject unsupported architecture at configure or compile time and reject
  Server GC configuration with a useful startup message.

## Checkpoint

Launch `LoaderSmoke` with the custom GC. The old message
`GC initialization failed` must disappear. The process should then stop at a
deterministic diagnostic naming the first operation that the shell does not yet
support.

Record that method and the call sequence; they are input to mission 03.

## Recorded interface inventory and trace

All pure virtual methods of `IGCHeap` and `IGCHandleManager` must have exact C++
overrides before those two objects can be constructed. The compiler verifies
that requirement, and `static_assert` verifies that both adapter classes are
concrete. Mission 02 does not construct an `IGCHandleStore`.

Observed with the pinned runtime:

1. `GC_Initialize` stores `IGCToCLR`, rejects unsupported configuration, and
   returns the stable heap and handle-manager objects with `S_OK`.
2. `IGCHeap::ControlEvents` and `IGCHeap::ControlPrivateEvents` are accepted as
   intentional no-ops while CoreCLR publishes the heap.
3. `IGCHandleManager::Initialize` succeeds through the Rust C ABI.
4. `IGCHandleManager::GetGlobalHandleStore` reaches the shared unsupported-call
   diagnostic and terminates. This is the first Mission 03 operation.

Not yet observed from `IGCHeap`:

`Alloc`, `CancelFullGCNotification`, `CollectionCount`, `DiagDescrGenerations`,
`DiagGetGCSettings`, `DiagScanDependentHandles`, `DiagScanFinalizeQueue`,
`DiagScanHandles`, `DiagTraceGCSegments`, `DiagWalkFinalizeQueue`, `DiagWalkHeap`,
`DiagWalkHeapWithACHandling`, `DiagWalkObject`, `DiagWalkObject2`,
`DiagWalkSurvivorsWithType`, `EnableNoGCRegionCallback`, `EndNoGCRegion`,
`EnumerateConfigurationValues`, `FixAllocContext`, `GarbageCollect`,
`GetCondemnedGeneration`, `GetContainingObject`, `GetCurrentObjSize`,
`GetExtraWorkForFinalization`, `GetGcCount`, `GetGcLatencyMode`,
`GetGenerationBudget`, `GetGenerationWithRange`, `GetLastGCDuration`,
`GetLastGCGenerationSize`, `GetLastGCPercentTimeInGC`, `GetLastGCStartTime`,
`GetLOHCompactionMode`, `GetLOHThreshold`, `GetMaxGeneration`, `GetMemoryInfo`,
`GetMemoryLoad`, `GetNextFinalizable`, `GetNow`, `GetNumberOfFinalizable`,
`GetTotalAllocatedBytes`, `GetTotalBytesInUse`, `GetTotalPauseDuration`,
`GetValidSegmentSize`, `Initialize`, `IsConcurrentGCEnabled`,
`IsConcurrentGCInProgress`, `IsEphemeral`, `IsGCInProgressHelper`,
`IsHeapPointer`, `IsInFrozenSegment`, `IsLargeObject`, `IsPromoted`,
`IsThreadUsingAllocationContextHeap`, `IsValidGen0MaxSize`,
`IsValidSegmentSize`, `NextObj`, `NullBridgeObjectsWeakRefs`, `PublishObject`,
`RefreshMemoryLimit`, `RegisterForFinalization`,
`RegisterForFullGCNotification`, `RegisterFrozenSegment`, `ResetWaitForGCEvent`,
`RuntimeStructuresValid`, `SetFinalizationRun`, `SetGCInProgress`,
`SetGcLatencyMode`, `SetLOHCompactionMode`, `SetReservedVMLimit`,
`SetSuspensionPending`, `SetWaitForGCEvent`, `SetYieldProcessorScalingFactor`,
`Shutdown`, `StartNoGCRegion`, `StressHeap`, `TemporaryDisableConcurrentGC`,
`TemporaryEnableConcurrentGC`, `UnregisterFrozenSegment`, `UpdateFrozenSegment`,
`ValidateObjectMember`, `WaitForFullGCApproach`, `WaitForFullGCComplete`,
`WaitUntilConcurrentGCComplete`, `WaitUntilConcurrentGCCompleteAsync`,
`WaitUntilGCComplete`, and `WhichGeneration`.

Not yet observed from `IGCHandleManager`:

`CreateDuplicateHandle`, `CreateGlobalHandleOfType`, `CreateHandleStore`,
`DestroyHandleOfType`, `DestroyHandleOfUnknownType`, `DestroyHandleStore`,
`GetDependentHandleSecondary`, `GetExtraInfoFromHandle`, `HandleFetchType`,
`InterlockedCompareExchangeObjectInHandle`, `SetDependentHandleSecondary`,
`SetExtraInfoForHandle`, `Shutdown`, `StoreObjectInHandle`,
`StoreObjectInHandleIfNull`, and `TraceRefCountedHandles`.

Every `IGCHandleStore` method is not yet observed: `Uproot`, `ContainsHandle`,
both `CreateHandleOfType` overloads, `CreateHandleWithExtraInfo`, and
`CreateDependentHandle`.

No pure virtual method is classified as configuration-only by the pinned
Windows x64 interface. Unsupported architecture is rejected before compilation,
and Server GC is rejected in `GC_Initialize`; unobserved methods remain required
ABI surface and are not assumed optional.

## Allowed shortcuts

- Most interface methods may fail fast.
- No managed allocation needs to succeed.
- Interface objects may live until process exit.
- The real CoreCLR `LoaderSmoke` run is the ABI integration test. A separate C++
  test executable is intentionally omitted for this educational checkpoint.

## Known debt

The shell proves ABI shape and lifetime only. It is not a collector, and its
method inventory is not yet a supported-feature matrix.

There is no mutable collector state in this shell. When such state is introduced,
it must be Rust-owned and exposed to C++ only as an opaque C ABI handle.

## What this unlocks

Mission 03 can implement only the operations demonstrated by the startup trace
and aim for the first managed `Main` execution.

## Hints

Use compile-time version assertions and let the C++ compiler verify overrides.
Keep failure reporting native and safe on allocation-sensitive paths.
