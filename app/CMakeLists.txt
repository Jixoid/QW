set(TARGET_NAME test_qw)

file(GLOB_RECURSE SOURCES "*.cc")

add_executable(${TARGET_NAME} ${SOURCES})

target_link_libraries(${TARGET_NAME} PRIVATE 
  qaos_qw_static
  CLI11::CLI11
)

target_link_options(${TARGET_NAME} PUBLIC -rdynamic)
