#include "common.h"
#include "gcenv.h"

#include "rust_gc.h"

static_assert(sizeof(void*) == 8, "The native shim requires a 64-bit process");
static_assert(GC_INTERFACE_MAJOR_VERSION == 5, "Unexpected CoreCLR GC interface major version");
static_assert(GC_INTERFACE_MINOR_VERSION == 5, "Unexpected CoreCLR GC interface minor version");

namespace
{
constexpr char ShimName[] = "dotnet-gc-rust";
constexpr char InitializationMessage[] =
    "dotnet-gc-rust: native shim reached Rust; initialization is intentionally unsupported\r\n";

void WriteInitializationDiagnostic() noexcept
{
    HANDLE standardError = GetStdHandle(STD_ERROR_HANDLE);
    if ((standardError == nullptr) || (standardError == INVALID_HANDLE_VALUE))
    {
        return;
    }

    DWORD bytesWritten = 0;
    WriteFile(
        standardError,
        InitializationMessage,
        static_cast<DWORD>(sizeof(InitializationMessage) - 1),
        &bytesWritten,
        nullptr);
}
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
    if (gcHeap != nullptr)
    {
        *gcHeap = nullptr;
    }
    if (gcHandleManager != nullptr)
    {
        *gcHandleManager = nullptr;
    }
    if (gcDacVars != nullptr)
    {
        *gcDacVars = {};
    }

    const std::int32_t rustResult = gc_rust_loader_probe();
    if (rustResult == 0)
    {
        WriteInitializationDiagnostic();
    }

    return E_FAIL;
}
