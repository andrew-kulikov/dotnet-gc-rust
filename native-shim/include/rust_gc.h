#pragma once

#include <cstdint>

#include <Windows.h>

// IGCHeap implementation
extern "C" HRESULT rust_gc_loader_probe() noexcept;
extern "C" HRESULT rust_gc_initialize() noexcept;

// IGCHandleStore implementation
using RustGCObject = void*;
using RustGCObjectHandle = RustGCObject*;

extern "C" RustGCObjectHandle rust_gc_handle_store_create_handle_of_type(
    RustGCObject object,
    std::uint32_t type) noexcept;

// IGCHandleManager implementation
extern "C" bool rust_gc_handle_manager_initialize() noexcept;
