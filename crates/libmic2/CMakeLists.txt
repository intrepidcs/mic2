# Setup Cargo commands

if (CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(CARGO_CMD cargo build)
    set(TARGET_DIR "debug")
else ()
    set(CARGO_CMD cargo build --release --quiet)
    set(TARGET_DIR "release")
endif ()

# Platform specific library name
if(WIN32)
    set(LIBMIC2_LIB_NAME "mic2.dll")
    set(LIBMIC2_STATIC_LIB_NAME "mic2.lib")
elseif(APPLE)
    set(LIBMIC2_LIB_NAME "libmic2.dylib")
    set(LIBMIC2_STATIC_LIB_NAME "libmic2.a")
else(UNIX)
    set(LIBMIC2_LIB_NAME "libmic2.so")
    set(LIBMIC2_STATIC_LIB_NAME "libmic2.a")
endif()

set(LIBMIC2_LIB "${CMAKE_CURRENT_BINARY_DIR}/${TARGET_DIR}/${LIBMIC2_LIB_NAME}")
set(LIBMIC2_STATIC_LIB "${CMAKE_CURRENT_BINARY_DIR}/${TARGET_DIR}/${LIBMIC2_STATIC_LIB_NAME}")
set(LIBMIC2_HEADER_PATH "${CMAKE_CURRENT_BINARY_DIR}/${TARGET_DIR}/mic2.h")
set(LIBMIC2_CXX_HEADER_PATH "${CMAKE_SOURCE_DIR}/crates/libmic2/src/mic2.hpp")
set(LIBMIC2_CXX_SRC_PATH "${CMAKE_SOURCE_DIR}/crates/libmic2/src/mic2.cpp")


# Build libmic2
message(STATUS "executing CARGO_TARGET_DIR=${CMAKE_CURRENT_BINARY_DIR} ${CARGO_CMD}")
add_custom_target(libmic2_rs ALL
    COMMENT "Compiling C shared library libmic2"
    COMMAND ${CMAKE_COMMAND} -E env "CARGO_TARGET_DIR=${CMAKE_CURRENT_BINARY_DIR}" -- ${CARGO_CMD}
    COMMAND ${CMAKE_COMMAND} -E copy ${LIBMIC2_LIB} ${CMAKE_CURRENT_BINARY_DIR}
    COMMAND ${CMAKE_COMMAND} -E copy ${LIBMIC2_STATIC_LIB} ${CMAKE_CURRENT_BINARY_DIR}
    COMMAND ${CMAKE_COMMAND} -E copy ${LIBMIC2_HEADER_PATH} ${CMAKE_CURRENT_BINARY_DIR}
    COMMAND ${CMAKE_COMMAND} -E copy ${LIBMIC2_CXX_HEADER_PATH} ${CMAKE_CURRENT_BINARY_DIR}
    COMMAND ${CMAKE_COMMAND} -E copy ${LIBMIC2_CXX_SRC_PATH} ${CMAKE_CURRENT_BINARY_DIR}
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})

# Useful target properties for other Projects
set_target_properties(libmic2_rs PROPERTIES LIB_NAME ${LIBMIC2_LIB_NAME})
set_target_properties(libmic2_rs PROPERTIES BIN_LOCATION ${CMAKE_CURRENT_BINARY_DIR})
set_target_properties(libmic2_rs PROPERTIES CXX_SOURCE "${LIBMIC2_CXX_SRC_PATH}")

# Test libmic2
add_test(NAME libmic2_test
    COMMAND cargo test
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})
