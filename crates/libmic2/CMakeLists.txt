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
    set(LIBMIC2_LIB_NAME "libmic2.so")
elseif(APPLE)
    set(LIBMIC2_LIB_NAME "libmic2.dylib")
else(UNIX)
    set(LIBMIC2_LIB_NAME "libmic2.so")
endif()

set(LIBMIC2_LIB "${CMAKE_CURRENT_BINARY_DIR}/${TARGET_DIR}/${LIBMIC2_LIB_NAME}")
set(LIBMIC2_STATIC_LIB "${CMAKE_CURRENT_BINARY_DIR}/${TARGET_DIR}/libmic2.a")
set(LIBMIC2_HEADER_PATH "${CMAKE_CURRENT_BINARY_DIR}/${TARGET_DIR}/mic2.h")

# Build libmic2
message(STATUS "executing CARGO_TARGET_DIR=${CMAKE_CURRENT_BINARY_DIR} ${CARGO_CMD}")
add_custom_target(libmic2_rs ALL
    COMMENT "Compiling C shared library libmic2"
    COMMAND CARGO_TARGET_DIR=${CMAKE_CURRENT_BINARY_DIR} ${CARGO_CMD}
    COMMAND cp ${LIBMIC2_LIB} ${CMAKE_CURRENT_BINARY_DIR}
    COMMAND cp ${LIBMIC2_STATIC_LIB} ${CMAKE_CURRENT_BINARY_DIR}
    COMMAND cp ${LIBMIC2_HEADER_PATH} ${CMAKE_CURRENT_BINARY_DIR}
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})

# Useful target properties for other Projects
set_target_properties(libmic2_rs PROPERTIES LIB_NAME ${LIBMIC2_LIB_NAME})
set_target_properties(libmic2_rs PROPERTIES BIN_LOCATION ${CMAKE_CURRENT_BINARY_DIR})

# Test libmic2
add_test(NAME libmic2_test
    COMMAND cargo test
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})
