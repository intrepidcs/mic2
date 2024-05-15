cmake_minimum_required(VERSION 3.25)
project(libmic2 LANGUAGES C CXX)

enable_testing()

add_subdirectory(crates/libmic2)

option(BUILD_EXAMPLES "Build examples" OFF)


# Build Examples if specified
if (${BUILD_EXAMPLES})
    message(STATUS "Building examples")
    add_subdirectory(examples)
endif()
