#pragma once

#include <Windows.h>

// IGCHeap implementation
extern "C" HRESULT rust_gc_loader_probe() noexcept;
extern "C" HRESULT rust_gc_initialize() noexcept;

// IGCHandleManager implementation
extern "C" bool rust_gc_handle_manager_initialize() noexcept;

