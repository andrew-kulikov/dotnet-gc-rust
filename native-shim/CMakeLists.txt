cmake_minimum_required(VERSION 3.25)

project(dotnet_gc_shim LANGUAGES CXX)

if(NOT WIN32)
    message(FATAL_ERROR "The native shim currently supports Windows only")
endif()

if(NOT CMAKE_SIZEOF_VOID_P EQUAL 8)
    message(FATAL_ERROR "The native shim currently supports x64 only")
endif()

set(
    DOTNET_RUNTIME_ROOT
    "${CMAKE_CURRENT_SOURCE_DIR}/../external/dotnet-runtime"
    CACHE PATH
    "Path to the pinned dotnet/runtime source tree"
)
set(
    RUST_OUTPUT_DIR
    "${CMAKE_CURRENT_SOURCE_DIR}/../target/debug"
    CACHE PATH
    "Directory containing the gc-ffi Cargo build outputs"
)

set(GC_INTERFACE_HEADER "${DOTNET_RUNTIME_ROOT}/src/coreclr/gc/gcinterface.h")
if(NOT EXISTS "${GC_INTERFACE_HEADER}")
    message(FATAL_ERROR
        "CoreCLR GC headers are missing. Run: python scripts/build.py bootstrap"
    )
endif()

set(RUST_DLL "${RUST_OUTPUT_DIR}/gc_ffi.dll")
set(RUST_IMPORT_LIBRARY "${RUST_OUTPUT_DIR}/gc_ffi.dll.lib")
if(NOT EXISTS "${RUST_IMPORT_LIBRARY}")
    message(FATAL_ERROR
        "The gc-ffi import library is missing. Run: cargo build -p gc-ffi"
    )
endif()

add_library(gc_ffi SHARED IMPORTED GLOBAL)
set_target_properties(gc_ffi PROPERTIES
    IMPORTED_IMPLIB "${RUST_IMPORT_LIBRARY}"
    IMPORTED_LOCATION "${RUST_DLL}"
)

add_library(dotnet_gc_shim SHARED src/shim.cpp)
target_compile_features(dotnet_gc_shim PRIVATE cxx_std_20)
target_compile_definitions(dotnet_gc_shim PRIVATE
    BUILD_AS_STANDALONE
    FEATURE_STANDALONE_GC
    HOST_64BIT
    HOST_AMD64
    HOST_WINDOWS
    NOMINMAX
    TARGET_64BIT
    TARGET_AMD64
    TARGET_WINDOWS
    WIN32_LEAN_AND_MEAN
)
target_compile_options(dotnet_gc_shim PRIVATE /W4 /permissive-)
target_include_directories(dotnet_gc_shim SYSTEM PRIVATE
    "${DOTNET_RUNTIME_ROOT}/src/coreclr/gc"
    "${DOTNET_RUNTIME_ROOT}/src/coreclr/gc/env"
    "${DOTNET_RUNTIME_ROOT}/src/coreclr/inc"
    "${DOTNET_RUNTIME_ROOT}/src/native"
)
target_include_directories(dotnet_gc_shim PRIVATE
    "${CMAKE_CURRENT_SOURCE_DIR}/include"
)
target_link_libraries(dotnet_gc_shim PRIVATE gc_ffi)

set_target_properties(dotnet_gc_shim PROPERTIES
    MSVC_RUNTIME_LIBRARY "MultiThreadedDLL"
    OUTPUT_NAME "dotnet_gc_shim"
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/stage"
    RUNTIME_OUTPUT_DIRECTORY_DEBUG "${CMAKE_BINARY_DIR}/stage"
    RUNTIME_OUTPUT_DIRECTORY_RELEASE "${CMAKE_BINARY_DIR}/stage"
)

add_custom_command(TARGET dotnet_gc_shim POST_BUILD
    COMMAND "${CMAKE_COMMAND}" -E copy_if_different
        "${RUST_DLL}"
        "$<TARGET_FILE_DIR:dotnet_gc_shim>"
    VERBATIM
)
