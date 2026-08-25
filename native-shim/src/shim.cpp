#include <cstdio>
#include <cstdlib>
#include <type_traits>

#include "common.h"
#include "gcenv.h"

#include "rust_gc.h"

static_assert(sizeof(void*) == 8, "The native shim requires a 64-bit process");
static_assert(GC_INTERFACE_MAJOR_VERSION == 5, "Unexpected CoreCLR GC interface major version");
static_assert(GC_INTERFACE_MINOR_VERSION == 5, "Unexpected CoreCLR GC interface minor version");

namespace
{
// ----------------------------------------------------------------------
// Constants and helpers for the native shim.
// ----------------------------------------------------------------------
constexpr char ShimName[] = "dotnet-gc-rust";

void WriteInitializationDiagnostic() noexcept
{
    std::fprintf(stderr, "dotnet-gc-rust: native shim reached Rust\n");
    std::fflush(stderr);
}

// ----------------------------------------------------------------------
// IGCHeap interface implementation for the native shim.
// ----------------------------------------------------------------------
template<typename ReturnType>
[[noreturn]] ReturnType AbortUnimplemented(const char* method) noexcept
{
    std::fprintf(
        stderr,
        "dotnet-gc-rust: unimplemented IGCHeap method called: %s\n",
        method);

    std::fflush(stderr);
    std::abort();
}

#define ABORTING_OVERRIDE(returnType, name, parameters) \
    returnType name parameters noexcept override        \
    {                                                   \
        return AbortUnimplemented<returnType>(#name);   \
    }

class RustGCHeap final : public IGCHeap
{
public:
    ABORTING_OVERRIDE(bool, IsValidSegmentSize, (size_t size))
    ABORTING_OVERRIDE(bool, IsValidGen0MaxSize, (size_t size))
    ABORTING_OVERRIDE(size_t, GetValidSegmentSize, (bool large_seg))
    ABORTING_OVERRIDE(void, SetReservedVMLimit, (size_t vmlimit))

    ABORTING_OVERRIDE(void, WaitUntilConcurrentGCComplete, ())
    ABORTING_OVERRIDE(bool, IsConcurrentGCInProgress, ())
    ABORTING_OVERRIDE(void, TemporaryEnableConcurrentGC, ())
    ABORTING_OVERRIDE(void, TemporaryDisableConcurrentGC, ())
    ABORTING_OVERRIDE(bool, IsConcurrentGCEnabled, ())
    ABORTING_OVERRIDE(
        HRESULT,
        WaitUntilConcurrentGCCompleteAsync,
        (int millisecondsTimeout))

    ABORTING_OVERRIDE(size_t, GetNumberOfFinalizable, ())
    ABORTING_OVERRIDE(Object*, GetNextFinalizable, ())

    ABORTING_OVERRIDE(
        void,
        GetMemoryInfo,
        (
            uint64_t* highMemLoadThresholdBytes,
            uint64_t* totalAvailableMemoryBytes,
            uint64_t* lastRecordedMemLoadBytes,
            uint64_t* lastRecordedHeapSizeBytes,
            uint64_t* lastRecordedFragmentationBytes,
            uint64_t* totalCommittedBytes,
            uint64_t* promotedBytes,
            uint64_t* pinnedObjectCount,
            uint64_t* finalizationPendingCount,
            uint64_t* index,
            uint32_t* generation,
            uint32_t* pauseTimePct,
            bool* isCompaction,
            bool* isConcurrent,
            uint64_t* genInfoRaw,
            uint64_t* pauseInfoRaw,
            int kind))

    ABORTING_OVERRIDE(uint32_t, GetMemoryLoad, ())
    ABORTING_OVERRIDE(int, GetGcLatencyMode, ())
    ABORTING_OVERRIDE(int, SetGcLatencyMode, (int newLatencyMode))
    ABORTING_OVERRIDE(int, GetLOHCompactionMode, ())
    ABORTING_OVERRIDE(
        void,
        SetLOHCompactionMode,
        (int newLOHCompactionMode))

    ABORTING_OVERRIDE(
        bool,
        RegisterForFullGCNotification,
        (uint32_t gen2Percentage, uint32_t lohPercentage))
    ABORTING_OVERRIDE(bool, CancelFullGCNotification, ())
    ABORTING_OVERRIDE(
        int,
        WaitForFullGCApproach,
        (int millisecondsTimeout))
    ABORTING_OVERRIDE(
        int,
        WaitForFullGCComplete,
        (int millisecondsTimeout))

    ABORTING_OVERRIDE(unsigned, WhichGeneration, (Object* obj))
    ABORTING_OVERRIDE(
        int,
        CollectionCount,
        (int generation, int get_bgc_fgc_coutn))
    ABORTING_OVERRIDE(
        int,
        StartNoGCRegion,
        (
            uint64_t totalSize,
            bool lohSizeKnown,
            uint64_t lohSize,
            bool disallowFullBlockingGC))
    ABORTING_OVERRIDE(int, EndNoGCRegion, ())
    ABORTING_OVERRIDE(size_t, GetTotalBytesInUse, ())
    ABORTING_OVERRIDE(uint64_t, GetTotalAllocatedBytes, ())
    ABORTING_OVERRIDE(
        HRESULT,
        GarbageCollect,
        (int generation, bool low_memory_p, int mode))
    ABORTING_OVERRIDE(unsigned, GetMaxGeneration, ())
    ABORTING_OVERRIDE(void, SetFinalizationRun, (Object* obj))
    ABORTING_OVERRIDE(
        bool,
        RegisterForFinalization,
        (int gen, Object* obj))
    ABORTING_OVERRIDE(int, GetLastGCPercentTimeInGC, ())
    ABORTING_OVERRIDE(
        size_t,
        GetLastGCGenerationSize,
        (int gen))

    ABORTING_OVERRIDE(HRESULT, Initialize, ())
    ABORTING_OVERRIDE(bool, IsPromoted, (Object* object))
    ABORTING_OVERRIDE(
        bool,
        IsHeapPointer,
        (void* object, bool small_heap_only))
    ABORTING_OVERRIDE(unsigned, GetCondemnedGeneration, ())
    ABORTING_OVERRIDE(
        bool,
        IsGCInProgressHelper,
        (bool bConsiderGCStart))
    ABORTING_OVERRIDE(unsigned, GetGcCount, ())
    ABORTING_OVERRIDE(
        bool,
        IsThreadUsingAllocationContextHeap,
        (gc_alloc_context* acontext, int thread_number))
    ABORTING_OVERRIDE(bool, IsEphemeral, (Object* object))
    ABORTING_OVERRIDE(
        uint32_t,
        WaitUntilGCComplete,
        (bool bConsiderGCStart))
    ABORTING_OVERRIDE(
        void,
        FixAllocContext,
        (gc_alloc_context* acontext, void* arg, void* heap))
    ABORTING_OVERRIDE(size_t, GetCurrentObjSize, ())
    ABORTING_OVERRIDE(void, SetGCInProgress, (bool fInProgress))
    ABORTING_OVERRIDE(bool, RuntimeStructuresValid, ())
    ABORTING_OVERRIDE(
        void,
        SetSuspensionPending,
        (bool fSuspensionPending))
    ABORTING_OVERRIDE(
        void,
        SetYieldProcessorScalingFactor,
        (float yieldProcessorScalingFactor))
    ABORTING_OVERRIDE(void, Shutdown, ())

    ABORTING_OVERRIDE(
        size_t,
        GetLastGCStartTime,
        (int generation))
    ABORTING_OVERRIDE(
        size_t,
        GetLastGCDuration,
        (int generation))
    ABORTING_OVERRIDE(size_t, GetNow, ())

    ABORTING_OVERRIDE(
        Object*,
        Alloc,
        (gc_alloc_context* acontext, size_t size, uint32_t flags))
    ABORTING_OVERRIDE(void, PublishObject, (uint8_t* obj))
    ABORTING_OVERRIDE(void, SetWaitForGCEvent, ())
    ABORTING_OVERRIDE(void, ResetWaitForGCEvent, ())

    ABORTING_OVERRIDE(bool, IsLargeObject, (Object* pObj))
    ABORTING_OVERRIDE(void, ValidateObjectMember, (Object* obj))
    ABORTING_OVERRIDE(Object*, NextObj, (Object* object))
    ABORTING_OVERRIDE(
        Object*,
        GetContainingObject,
        (void* pInteriorPtr, bool fCollectedGenOnly))

    ABORTING_OVERRIDE(
        void,
        DiagWalkObject,
        (Object* obj, walk_fn fn, void* context))
    ABORTING_OVERRIDE(
        void,
        DiagWalkObject2,
        (Object* obj, walk_fn2 fn, void* context))
    ABORTING_OVERRIDE(
        void,
        DiagWalkHeap,
        (
            walk_fn fn,
            void* context,
            int gen_number,
            bool walk_large_object_heap_p))
    ABORTING_OVERRIDE(
        void,
        DiagWalkSurvivorsWithType,
        (
            void* gc_context,
            record_surv_fn fn,
            void* diag_context,
            walk_surv_type type,
            int gen_number))
    ABORTING_OVERRIDE(
        void,
        DiagWalkFinalizeQueue,
        (void* gc_context, fq_walk_fn fn))
    ABORTING_OVERRIDE(
        void,
        DiagScanFinalizeQueue,
        (fq_scan_fn fn, ScanContext* context))
    ABORTING_OVERRIDE(
        void,
        DiagScanHandles,
        (handle_scan_fn fn, int gen_number, ScanContext* context))
    ABORTING_OVERRIDE(
        void,
        DiagScanDependentHandles,
        (handle_scan_fn fn, int gen_number, ScanContext* context))
    ABORTING_OVERRIDE(
        void,
        DiagDescrGenerations,
        (gen_walk_fn fn, void* context))
    ABORTING_OVERRIDE(void, DiagTraceGCSegments, ())
    ABORTING_OVERRIDE(
        void,
        DiagGetGCSettings,
        (EtwGCSettingsInfo* settings))

    ABORTING_OVERRIDE(
        bool,
        StressHeap,
        (gc_alloc_context* acontext))

    ABORTING_OVERRIDE(
        segment_handle,
        RegisterFrozenSegment,
        (segment_info* pseginfo))
    ABORTING_OVERRIDE(
        void,
        UnregisterFrozenSegment,
        (segment_handle seg))
    ABORTING_OVERRIDE(
        bool,
        IsInFrozenSegment,
        (Object* object))

    ABORTING_OVERRIDE(
        void,
        ControlEvents,
        (GCEventKeyword keyword, GCEventLevel level))
    ABORTING_OVERRIDE(
        void,
        ControlPrivateEvents,
        (GCEventKeyword keyword, GCEventLevel level))
    ABORTING_OVERRIDE(
        unsigned int,
        GetGenerationWithRange,
        (
            Object* object,
            uint8_t** ppStart,
            uint8_t** ppAllocated,
            uint8_t** ppReserved))

    ABORTING_OVERRIDE(int64_t, GetTotalPauseDuration, ())
    ABORTING_OVERRIDE(
        void,
        EnumerateConfigurationValues,
        (
            void* context,
            ConfigurationValueFunc configurationValueFunc))
    ABORTING_OVERRIDE(
        void,
        UpdateFrozenSegment,
        (
            segment_handle seg,
            uint8_t* allocated,
            uint8_t* committed))
    ABORTING_OVERRIDE(int, RefreshMemoryLimit, ())
    ABORTING_OVERRIDE(
        enable_no_gc_region_callback_status,
        EnableNoGCRegionCallback,
        (
            NoGCRegionCallbackFinalizerWorkItem* callback,
            uint64_t callback_threshold))
    ABORTING_OVERRIDE(
        FinalizerWorkItem*,
        GetExtraWorkForFinalization,
        ())
    ABORTING_OVERRIDE(
        uint64_t,
        GetGenerationBudget,
        (int generation))
    ABORTING_OVERRIDE(size_t, GetLOHThreshold, ())
    ABORTING_OVERRIDE(
        void,
        DiagWalkHeapWithACHandling,
        (
            walk_fn fn,
            void* context,
            int gen_number,
            bool walk_large_object_heap_p))
    ABORTING_OVERRIDE(
        void,
        NullBridgeObjectsWeakRefs,
        (size_t length, void* unreachableObjectHandles))
};

static_assert(
    !std::is_abstract_v<RustGCHeap>,
    "RustGCHeap must implement every IGCHeap method");

RustGCHeap GlobalRustGCHeap;
// End of IGCHeap interface 

// ----------------------------------------------------------------------
// IGCHandleManager interface implementation for the native shim.
// ----------------------------------------------------------------------
class RustGCHandleManager final : public IGCHandleManager
{
public:
    ABORTING_OVERRIDE(bool, Initialize, ());
    ABORTING_OVERRIDE(void, Shutdown, ());
    ABORTING_OVERRIDE(IGCHandleStore*, GetGlobalHandleStore, ());
    ABORTING_OVERRIDE(IGCHandleStore*, CreateHandleStore, ());
    ABORTING_OVERRIDE(void, DestroyHandleStore, (IGCHandleStore* store));
    ABORTING_OVERRIDE(OBJECTHANDLE, CreateGlobalHandleOfType, (Object* object, HandleType type));
    ABORTING_OVERRIDE(OBJECTHANDLE, CreateDuplicateHandle, (OBJECTHANDLE handle));
    ABORTING_OVERRIDE(void, DestroyHandleOfType, (OBJECTHANDLE handle, HandleType type));
    ABORTING_OVERRIDE(void, DestroyHandleOfUnknownType, (OBJECTHANDLE handle));
    ABORTING_OVERRIDE(void, SetExtraInfoForHandle, (OBJECTHANDLE handle, HandleType type, void* pExtraInfo));
    ABORTING_OVERRIDE(void*, GetExtraInfoFromHandle, (OBJECTHANDLE handle));
    ABORTING_OVERRIDE(void, StoreObjectInHandle, (OBJECTHANDLE handle, Object* object));
    ABORTING_OVERRIDE(bool, StoreObjectInHandleIfNull, (OBJECTHANDLE handle, Object* object));
    ABORTING_OVERRIDE(void, SetDependentHandleSecondary, (OBJECTHANDLE handle, Object* object));
    ABORTING_OVERRIDE(Object*, GetDependentHandleSecondary, (OBJECTHANDLE handle));
    ABORTING_OVERRIDE(Object*, InterlockedCompareExchangeObjectInHandle, (OBJECTHANDLE handle, Object* object, Object* comparandObject));
    ABORTING_OVERRIDE(HandleType, HandleFetchType, (OBJECTHANDLE handle));
    ABORTING_OVERRIDE(void, TraceRefCountedHandles, (HANDLESCANPROC callback, uintptr_t param1, uintptr_t param2));
};

static_assert(
    !std::is_abstract_v<RustGCHandleManager>,
    "RustGCHandleManager must implement every IGCHandleManager method");

RustGCHandleManager GlobalRustGCHandleManager;
// End of IGCHandleManager interface implementation for the native shim.
} // namespace

extern "C" __declspec(dllexport) void LOCALGC_CALLCONV GC_VersionInfo(VersionInfo* versionInfo) noexcept
{
    if (versionInfo == nullptr)
    {
        return;
    }

    versionInfo->MajorVersion = GC_INTERFACE_MAJOR_VERSION;
    versionInfo->MinorVersion = GC_INTERFACE_MINOR_VERSION;
    versionInfo->BuildVersion = 0;
    versionInfo->Name = ShimName;
}

extern "C" __declspec(dllexport) HRESULT LOCALGC_CALLCONV GC_Initialize(
    IGCToCLR*,
    IGCHeap** gcHeap,
    IGCHandleManager** gcHandleManager,
    GcDacVars* gcDacVars) noexcept
{
    *gcHeap = &GlobalRustGCHeap;
    *gcHandleManager = &GlobalRustGCHandleManager;
    // TODO: Implement GcDacVars support in the native shim.

    const std::int32_t rustResult = gc_rust_loader_probe();
    if (rustResult == 0)
    {
        WriteInitializationDiagnostic();
    }

    return S_OK;
}
