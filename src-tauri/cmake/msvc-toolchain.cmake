# Settings for the C libraries that build scripts build with CMake through the
# `cmake` crate: libFLAC (libflac-sys) and libopus (opusic-sys). CMake reads this
# file because `.cargo/config.toml` names it in
# CMAKE_TOOLCHAIN_FILE_x86_64_pc_windows_msvc (decision-10); that file sits in
# the repository root, so every cargo call uses it (TASK-37).
#
# 1. The static C runtime, to match `+crt-static` in the same `.cargo/config.toml`.
#    libFLAC asks for CMake policies up to 3.31, so CMP0091 is NEW there and the
#    runtime comes from this variable, not from the -MT flag the `cmake` crate
#    passes; without it libFLAC gets the DLL runtime and the exe does not link.
#    Rust never links the debug runtime, so there is no "Debug" variant here.
set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded")

# 2. Optimisation. For the Visual Studio generator the `cmake` crate replaces
#    CMAKE_C_FLAGS_<CONFIG> with flags that have no /O option, so the library
#    would be built unoptimised: libFLAC then adds 700 KB to the exe instead of
#    170 KB and encodes three times slower. These are CMake's own defaults.
#    Cargo's `opt-level = "s"` maps to MinSizeRel, other optimised levels to
#    Release, or to RelWithDebInfo when the profile keeps debug info: the dev
#    profile, where Cargo.toml raises opt-level for libflac-sys and opusic-sys.
set(CMAKE_C_FLAGS_MINSIZEREL "${CMAKE_C_FLAGS_MINSIZEREL} /O1 /Ob1 /DNDEBUG")
set(CMAKE_C_FLAGS_RELEASE "${CMAKE_C_FLAGS_RELEASE} /O2 /Ob2 /DNDEBUG")
set(CMAKE_C_FLAGS_RELWITHDEBINFO "${CMAKE_C_FLAGS_RELWITHDEBINFO} /O2 /Ob1 /DNDEBUG")
