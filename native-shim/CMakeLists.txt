# Require a modern CMake version with reliable imported-target and MSVC runtime
# selection support.
cmake_minimum_required(VERSION 3.25)

# Reject an unsupported Visual Studio platform before project() asks MSBuild to
# initialize that toolchain. This keeps the failure deterministic even when the
# corresponding ARM64 or x86 build tools are not installed.
if(NOT CMAKE_GENERATOR_PLATFORM STREQUAL "x64")
    message(FATAL_ERROR
        "The native shim currently supports Windows x64 only; selected platform: "
        "'${CMAKE_GENERATOR_PLATFORM}'"
    )
endif()

# Create a C++-only project. Cargo builds the Rust crate separately before CMake
# is configured.
project(dotnet_gc_shim LANGUAGES CXX)

# Stop at configure time on platforms that the shim does not support yet.
if(NOT WIN32)
    message(FATAL_ERROR "The native shim currently supports Windows only")
endif()

# The pinned C++ ABI and build definitions below are specific to MSVC x64.
# A pointer-size check alone would incorrectly accept Windows ARM64.
if(NOT MSVC)
    message(FATAL_ERROR "The native shim currently supports the MSVC toolchain only")
endif()
if(NOT CMAKE_SIZEOF_VOID_P EQUAL 8)
    message(FATAL_ERROR "The native shim requires a 64-bit x64 target")
endif()

# Expose the runtime checkout as an overridable CMake cache path. By default it
# points at the pinned Git submodule in this repository.
set(
    DOTNET_RUNTIME_ROOT
    "${CMAKE_CURRENT_SOURCE_DIR}/../external/dotnet-runtime"
    CACHE PATH
    "Path to the pinned dotnet/runtime source tree"
)

# Tell CMake where Cargo placed gc_rust.dll and its MSVC import library. The
# Python build script changes this path for Debug versus Release builds.
set(
    RUST_OUTPUT_DIR
    "${CMAKE_CURRENT_SOURCE_DIR}/../target/debug"
    CACHE PATH
    "Directory containing the gc-rust Cargo build outputs"
)

# Validate the runtime dependency early so a missing submodule produces an
# actionable error instead of a compiler include failure later.
set(GC_INTERFACE_HEADER "${DOTNET_RUNTIME_ROOT}/src/coreclr/gc/gcinterface.h")
if(NOT EXISTS "${GC_INTERFACE_HEADER}")
    message(FATAL_ERROR
        "CoreCLR GC headers are missing. Run: python scripts/build.py bootstrap"
    )
endif()

# A Windows DLL is linked through its .dll.lib import library and loaded from
# its .dll at runtime. Cargo produces both files for the gc-rust cdylib.
set(RUST_DLL "${RUST_OUTPUT_DIR}/gc_rust.dll")
set(RUST_IMPORT_LIBRARY "${RUST_OUTPUT_DIR}/gc_rust.dll.lib")

# CMake does not invoke Cargo itself, so fail clearly if the Python script (or a
# manual Cargo build) has not produced the Rust library first.
if(NOT EXISTS "${RUST_IMPORT_LIBRARY}")
    message(FATAL_ERROR
        "The gc-rust import library is missing. Run: cargo build -p gc-rust"
    )
endif()

# Model Cargo's already-built DLL as an imported CMake target. IMPORTED means
# CMake links and copies the library but does not compile it; GLOBAL makes the
# target visible throughout this CMake project.
add_library(gc_rust SHARED IMPORTED GLOBAL)
set_target_properties(gc_rust PROPERTIES
    # MSVC uses this import library while linking dotnet_gc_shim.dll.
    IMPORTED_IMPLIB "${RUST_IMPORT_LIBRARY}"
    # This is the runtime DLL that must be available when CoreCLR loads the shim.
    IMPORTED_LOCATION "${RUST_DLL}"
)

# Compile shim.cpp into the DLL that CoreCLR loads through DOTNET_GCPath.
add_library(dotnet_gc_shim SHARED src/shim.cpp)

# Request C++20 without applying a compiler-specific language-standard flag.
target_compile_features(dotnet_gc_shim PRIVATE cxx_std_20)

# Match the platform and standalone-GC preprocessor environment expected by the
# pinned CoreCLR headers. PRIVATE keeps these definitions local to this target.
target_compile_definitions(dotnet_gc_shim PRIVATE
    # Select the standalone GC form of CoreCLR's native interface headers.
    BUILD_AS_STANDALONE
    FEATURE_STANDALONE_GC
    # Describe the compiler host on which this DLL is being built.
    HOST_64BIT
    HOST_AMD64
    HOST_WINDOWS
    # Keep windows.h small and prevent its min/max macros from breaking C++ code.
    NOMINMAX
    # Describe the runtime target for which the shim is being compiled.
    TARGET_64BIT
    TARGET_AMD64
    TARGET_WINDOWS
    WIN32_LEAN_AND_MEAN
    # Stack walking depends on DbgHelp and is intentionally debug-only.
    $<$<CONFIG:Debug>:DOTNET_GC_RUST_ENABLE_STACK_TRACE>
)

# Enable strong MSVC warnings and require standards-conforming language behavior.
target_compile_options(dotnet_gc_shim PRIVATE /W4 /permissive-)

# Add the original CoreCLR directory structure directly. SYSTEM marks these as
# upstream headers so warnings in dotnet/runtime do not become project warnings.
target_include_directories(dotnet_gc_shim SYSTEM PRIVATE
    "${DOTNET_RUNTIME_ROOT}/src/coreclr/gc"
    "${DOTNET_RUNTIME_ROOT}/src/coreclr/gc/env"
    "${DOTNET_RUNTIME_ROOT}/src/coreclr/inc"
    "${DOTNET_RUNTIME_ROOT}/src/native"
)

# Add the project-owned header containing the narrow C ABI exposed by Rust.
target_include_directories(dotnet_gc_shim PRIVATE
    "${CMAKE_CURRENT_SOURCE_DIR}/include"
)

# Record the native shim's dependency on the imported Rust DLL. This adds the
# Rust import library to the MSVC link command.
target_link_libraries(dotnet_gc_shim PRIVATE
    gc_rust
    $<$<CONFIG:Debug>:dbghelp>
)

# Configure the produced DLL and put all supported configurations in one clear
# staging directory. Configuration-specific paths prevent Visual Studio's
# multi-config generator from appending an extra Debug or Release directory.
set_target_properties(dotnet_gc_shim PROPERTIES
    # Use the redistributable release CRT even in Debug builds. The debug CRT is
    # normally unavailable when the sample process loads this DLL.
    MSVC_RUNTIME_LIBRARY "MultiThreadedDLL"
    # Produce dotnet_gc_shim.dll rather than CMake's target-derived default name.
    OUTPUT_NAME "dotnet_gc_shim"
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/stage"
    RUNTIME_OUTPUT_DIRECTORY_DEBUG "${CMAKE_BINARY_DIR}/stage"
    RUNTIME_OUTPUT_DIRECTORY_RELEASE "${CMAKE_BINARY_DIR}/stage"
)

# After linking the shim, place gc_rust.dll beside it. Windows must be able to
# resolve this dependent DLL when CoreCLR loads dotnet_gc_shim.dll.
add_custom_command(TARGET dotnet_gc_shim POST_BUILD
    # Use CMake's portable file helper and avoid rewriting an identical DLL.
    COMMAND "${CMAKE_COMMAND}" -E copy_if_different
        "${RUST_DLL}"
        "$<TARGET_FILE_DIR:dotnet_gc_shim>"
    # Ask CMake to preserve argument boundaries and escape paths correctly.
    VERBATIM
)
