cmake_minimum_required(VERSION 3.28)
project(QW C CXX)


set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(CMAKE_C_STANDARD 23)
set(CMAKE_C_STANDARD_REQUIRED ON)

set(CMAKE_CXX_COMPILER clang++)
set(CMAKE_C_COMPILER clang)

add_compile_options(-gdwarf-4)


# Debug
add_compile_definitions($<$<CONFIG:Debug>:_DBG> $<$<CONFIG:RelWithDebInfo>:_DBG>)
add_compile_definitions(_QW_use_internal_diagnostic)


# Sanitizer
add_compile_options($<$<CONFIG:Debug>:-fsanitize=address,leak,undefined>)
add_link_options($<$<CONFIG:Debug>:-fsanitize=address,leak,undefined>)


# LTO
include(CheckIPOSupported)
check_ipo_supported(RESULT lto_supported OUTPUT lto_error)

if(lto_supported)
  set(CMAKE_INTERPROCEDURAL_OPTIMIZATION_RELEASE ON)
  set(CMAKE_INTERPROCEDURAL_OPTIMIZATION_RELWITHDEBINFO ON)
else()
  message(WARNING "LTO is not supported: ${lto_error}")
endif()


file(GLOB childs RELATIVE "${CMAKE_CURRENT_SOURCE_DIR}/pkgs" "${CMAKE_CURRENT_SOURCE_DIR}/pkgs/*")
foreach(child ${childs})
  if(
    IS_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/pkgs/${child}" AND
    EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/pkgs/${child}/CMakeLists.txt"
  )
    
    message(STATUS "Sub Project Added: ${child}")
    add_subdirectory("${CMAKE_CURRENT_SOURCE_DIR}/pkgs/${child}")
      
  endif()
endforeach()


# Resources
file(GLOB_RECURSE ABS_RES_FILES "${CMAKE_CURRENT_SOURCE_DIR}/qw/res/*")
set(RES_OBJECTS "")

if (NOT EMSCRIPTEN)
  foreach(ABS_RES_FILE ${ABS_RES_FILES})
    
    file(RELATIVE_PATH RES_FILE "${CMAKE_CURRENT_SOURCE_DIR}/qw" "${ABS_RES_FILE}")
    
    set(OBJ_FILE "${CMAKE_CURRENT_BINARY_DIR}/${RES_FILE}.o")

    if(WIN32 OR MINGW)
      set(LD_FLAGS "-r") 
    else()
      set(LD_FLAGS "-r" "-m" "elf_x86_64")
    endif()
    
    add_custom_command(
      OUTPUT ${OBJ_FILE}
      COMMAND ${CMAKE_LINKER} ${LD_FLAGS} -b binary -o ${OBJ_FILE} ${RES_FILE}
      WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/qw"
      DEPENDS "${ABS_RES_FILE}"
      COMMENT "Embedding: ${RES_FILE}"
    )
    
    list(APPEND RES_OBJECTS ${OBJ_FILE})
  endforeach()
endif()


# find
file(GLOB_RECURSE SOURCES "${CMAKE_CURRENT_SOURCE_DIR}/qw/*.cc")
file(GLOB_RECURSE HEADERS "${CMAKE_CURRENT_SOURCE_DIR}/qw/*.hh")


# i18n
add_custom_command(
  OUTPUT ${CMAKE_CURRENT_SOURCE_DIR}/qw/i18n/main.pot
  COMMAND xgettext --keyword=_ --language=C++ --from-code=UTF-8 --output=${CMAKE_CURRENT_SOURCE_DIR}/qw/i18n/main.pot ${SOURCES} ${HEADERS}
  WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
  DEPENDS ${SOURCES} ${HEADERS}
  COMMENT "Updating the POT template file."
)

add_custom_target(update-pot
  DEPENDS "${CMAKE_CURRENT_SOURCE_DIR}/qw/i18n/main.pot"
)


# llvm Pack
find_package(LLVM REQUIRED CONFIG)

include_directories(${LLVM_INCLUDE_DIRS})
add_definitions(${LLVM_DEFINITIONS})

set(LLVM_LINK_COMPONENTS core support native executionengine orcjit)


# Engine
add_library(qaos_qw_shared SHARED ${SOURCES} ${RES_OBJECTS})
target_include_directories(qaos_qw_shared PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}")
add_dependencies(qaos_qw_shared update-pot)
target_link_libraries(qaos_qw_shared PUBLIC
  LLVM
)


add_library(qaos_qw_static STATIC ${SOURCES} ${RES_OBJECTS})
target_include_directories(qaos_qw_static PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}")
add_dependencies(qaos_qw_static update-pot)
target_link_libraries(qaos_qw_static PUBLIC
  LLVM
)



# Test App
add_subdirectory(app)
