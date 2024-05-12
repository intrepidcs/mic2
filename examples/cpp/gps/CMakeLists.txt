set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

if (MSVC)
    # warning level 4
    add_compile_options(/W4)
else()
    # additional warnings
    add_compile_options(-Wall -Wextra -Wpedantic -Werror -Wconversion -Wsign-conversion -Wfloat-equal)
endif()

get_target_property(LIBMIC2_DIR libmic2_rs BIN_LOCATION)
get_target_property(LIBMIC2_LIB_FNAME libmic2_rs LIB_NAME)
get_target_property(LIBMIC2_CXX_SOURCE libmic2_rs CXX_SOURCE)

set(SOURCE src/main.cpp ${LIBMIC2_CXX_SOURCE})



add_executable(example_gps_cpp ${SOURCE})
add_dependencies(example_gps_cpp libmic2_rs)

target_link_libraries(example_gps_cpp ${LIBMIC2_DIR}/${LIBMIC2_LIB_FNAME})
target_include_directories(example_gps_cpp PUBLIC ${LIBMIC2_DIR})
