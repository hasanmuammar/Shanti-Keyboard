cmake_minimum_required(VERSION 3.18...3.29)
list(APPEND CMAKE_MODULE_PATH "${CMAKE_SOURCE_DIR}/cmake")

file(STRINGS "${CMAKE_SOURCE_DIR}/version.txt" VERSION_STRING LIMIT_COUNT 1 REGEX "^.+")
project("Shanti-Keyboard" VERSION ${VERSION_STRING})
set(APP_NAME "Shanti-Keyboard")

if(NOT CMAKE_SYSTEM_NAME STREQUAL "Linux")
    message(FATAL_ERROR "This fork currently supports Linux only.")
endif()

set(OS_LINUX 1)

add_definitions(-DPROJECT_VERSION="${PROJECT_VERSION}")
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

add_subdirectory(cmake/corrosion-rs)
# The IBus engine is Qt-free; the GTK4/libadwaita GUI is built as the separate
# Rust `shanti-gui` crate and installed by the Flatpak manifest as `shanti-gui`.
# The legacy Qt5 settings GUI (src/frontend + src/shared) is retired.

# Rust crates used by the IBus engine (shanti-config, shanti-paths); imported
# as CMake targets so the engine can link them directly.
add_subdirectory(crates)

option(BUILD_RUST_GUI_SHELL "Build the Rust GUI migration shell" OFF)
if(BUILD_RUST_GUI_SHELL)
    find_program(CARGO_EXECUTABLE cargo REQUIRED)
    set(RUST_GUI_TARGET_DIR "${CMAKE_BINARY_DIR}/rust-gui-target")
    set(RUST_GUI_BINARY "${RUST_GUI_TARGET_DIR}/release/shanti-gui-rust")
    add_custom_command(
        OUTPUT "${RUST_GUI_BINARY}"
        COMMAND ${CMAKE_COMMAND} -E env
            "CARGO_TARGET_DIR=${RUST_GUI_TARGET_DIR}"
            "SHANTI_VERSION=${PROJECT_VERSION}"
            "${CARGO_EXECUTABLE}" build --release --manifest-path
            "${CMAKE_SOURCE_DIR}/crates/shanti-gui/Cargo.toml"
        DEPENDS
            crates/shanti-gui/Cargo.toml
            crates/shanti-gui/Cargo.lock
            crates/shanti-gui/src/main.rs
            crates/shanti-paths/Cargo.toml
            crates/shanti-paths/src/lib.rs
        WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
        VERBATIM
    )
    add_custom_target(shanti-gui-rust ALL DEPENDS "${RUST_GUI_BINARY}")
    install(PROGRAMS "${RUST_GUI_BINARY}" DESTINATION "${BIN_DIR}")
endif()

if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release)
endif()

if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    add_definitions(-DDEBUG)
endif()

if(CMAKE_SOURCE_DIR STREQUAL CMAKE_BINARY_DIR)
    message(FATAL_ERROR "In-source builds are not allowed.\nPlease create a build directory and configure CMake from outside the source tree.")
endif()

if(NOT DEFINED BIN_DIR)
    set(BIN_DIR "${CMAKE_INSTALL_PREFIX}/bin")
endif()
if(NOT DEFINED DATADIR)
    set(DATADIR "${CMAKE_INSTALL_PREFIX}/share")
endif()
if(NOT DEFINED LIBEXECDIR)
    set(LIBEXECDIR "${CMAKE_INSTALL_PREFIX}/libexec")
endif()

set(PROJECT_DATADIR "${DATADIR}/shanti-keyboard")
add_definitions(-DPROJECT_DATADIR="${PROJECT_DATADIR}")
add_definitions(-DBIN_DIR="${BIN_DIR}")
add_definitions(-DLIBEXECDIR="${LIBEXECDIR}")

include(FindPkgConfig)
find_package(PkgConfig REQUIRED)

option(ENABLE_IBUS "Enable IBus support" ON)
if(NOT ENABLE_IBUS)
    message(FATAL_ERROR "This branch supports IBus only.")
endif()

pkg_check_modules(ZSTD REQUIRED libzstd)

pkg_check_modules(IBUS REQUIRED ibus-1.0)

add_subdirectory(src/engine)
add_subdirectory(src/platform)
