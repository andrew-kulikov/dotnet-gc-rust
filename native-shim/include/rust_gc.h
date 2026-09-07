#pragma once

#include <cstddef>
#include <cstdint>

#include <Windows.h>

using RustGCObject = void*;
struct gc_alloc_context;

// IGCHeap implementation
extern "C" HRESULT rust_gc_loader_probe() noexcept;
extern "C" HRESULT rust_gc_initialize() noexcept;
extern "C" RustGCObject rust_gc_alloc(
    gc_alloc_context* acontext,
    std::size_t size,
    std::uint32_t flags) noexcept;

// IGCHandleStore implementation
using RustGCObjectHandle = RustGCObject*;

extern "C" RustGCObjectHandle rust_gc_handle_store_create_handle_of_type(
    RustGCObject object,
    std::uint32_t type) noexcept;

// IGCHandleManager implementation
extern "C" bool rust_gc_handle_manager_initialize() noexcept;
