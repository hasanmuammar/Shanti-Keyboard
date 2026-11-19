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
# Qt5 is only needed by the settings GUI (src/frontend + src/shared); the
# IBus engine is Qt-free since rewrite phase 2.
option(BUILD_FRONTEND "Build the Qt5 settings GUI" ON)
if(BUILD_FRONTEND)
    find_package(Qt5 COMPONENTS Widgets Network Svg DBus REQUIRED)
endif()

# Rust crates used by the IBus engine (shanti-config, shanti-paths); imported
# as CMake targets so the engine can link them directly.
add_subdirectory(crates)

if(BUILD_FRONTEND)
    message(STATUS "Qt5 framework version: ${Qt5Core_VERSION}")
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
if(BUILD_FRONTEND)
    add_subdirectory(src/shared)
    add_subdirectory(src/frontend)
endif()
