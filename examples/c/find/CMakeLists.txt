if (MSVC)
    # warning level 4
    add_compile_options(/W4)
else()
    # additional warnings
    add_compile_options(-Wall -Wextra -Wpedantic -Werror -Wconversion -Wsign-conversion -Wfloat-equal)
endif()

set(SOURCE src/main.c)

get_target_property(LIBMIC2_DIR libmic2_rs BIN_LOCATION)
get_target_property(LIBMIC2_LIB_FNAME libmic2_rs LIB_NAME)

add_executable(example_find_c ${SOURCE})
add_dependencies(example_find_c libmic2_rs)

target_link_libraries(example_find_c ${LIBMIC2_DIR}/${LIBMIC2_LIB_FNAME})
target_include_directories(example_find_c PUBLIC ${LIBMIC2_DIR})
