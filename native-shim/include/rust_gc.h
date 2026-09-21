#pragma once

#include <cstddef>
#include <cstdint>

#include <Windows.h>

using RustGCObject = void*;
using RustGCFrozenSegmentHandle = void*;

struct gc_alloc_context;
struct segment_info;
struct WriteBarrierParameters;

// IGCToCLR interface bridge
extern "C"
{
    typedef HRESULT StompWriteBarrier_Func(
        void* context,
        WriteBarrierParameters* parameters) noexcept;
}

struct RustGCToCLR
{
    void* context;
    StompWriteBarrier_Func* stomp_write_barrier;
};

// IGCHeap implementation
extern "C" HRESULT rust_gc_loader_probe() noexcept;
extern "C" HRESULT rust_gc_initialize(const RustGCToCLR* gc_to_clr) noexcept;
extern "C" RustGCObject rust_gc_alloc(
    gc_alloc_context* acontext,
    std::size_t size,
    std::uint32_t flags) noexcept;
extern "C" RustGCFrozenSegmentHandle rust_gc_register_frozen_segment(
    const segment_info* segment_info) noexcept;
extern "C" void rust_gc_update_frozen_segment(
    RustGCFrozenSegmentHandle seg,
    uint8_t* allocated,
    uint8_t* committed) noexcept;
extern "C" void rust_gc_set_finalization_run(RustGCObject obj) noexcept;
extern "C" unsigned rust_gc_get_max_generation() noexcept;
extern "C" int rust_gc_collection_count(int generation, int get_bgc_fgc_coutn) noexcept;

// IGCHandleStore implementation
using RustGCObjectHandle = RustGCObject*;

extern "C" RustGCObjectHandle rust_gc_handle_store_create_handle_of_type(
    RustGCObject object,
    std::uint32_t type) noexcept;

// IGCHandleManager implementation
extern "C" bool rust_gc_handle_manager_initialize() noexcept;
extern "C" void rust_gc_handle_manager_store_object_in_handle(
    RustGCObjectHandle handle,
    RustGCObject object) noexcept;
extern "C" RustGCObject rust_gc_handle_manager_interlocked_compare_exchange_object_in_handle(
    RustGCObjectHandle handle,
    RustGCObject object,
    RustGCObject comparand_object) noexcept;
