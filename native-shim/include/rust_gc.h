#pragma once

#include <cstddef>
#include <cstdint>

#include <Windows.h>

using RustGCObject = void*;
using RustGCFrozenSegmentHandle = void*;

struct gc_alloc_context;
struct segment_info;


// IGCHeap implementation
extern "C" HRESULT rust_gc_loader_probe() noexcept;
extern "C" HRESULT rust_gc_initialize() noexcept;
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

// IGCHandleStore implementation
using RustGCObjectHandle = RustGCObject*;

extern "C" RustGCObjectHandle rust_gc_handle_store_create_handle_of_type(
    RustGCObject object,
    std::uint32_t type) noexcept;

// IGCHandleManager implementation
extern "C" bool rust_gc_handle_manager_initialize() noexcept;
