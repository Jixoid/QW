set(TARGET_NAME qaos_qw)



file(GLOB_RECURSE SOURCES "src/*.cc")
file(GLOB_RECURSE HEADERS "inc/*.hh")


# i18n Pack
add_custom_command(
  OUTPUT ${CMAKE_CURRENT_SOURCE_DIR}/i18n/main.pot
  COMMAND xgettext --keyword=_ --language=C++ --from-code=UTF-8 --output=${CMAKE_CURRENT_SOURCE_DIR}/i18n/main.pot ${SOURCES} ${HEADERS}
  WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
  DEPENDS ${SOURCES} ${HEADERS}
  COMMENT "POT şablon dosyası güncelleniyor..."
)

add_custom_target(update-pot
  DEPENDS "${CMAKE_CURRENT_SOURCE_DIR}/i18n/main.pot"
)
# end i18n


# llvm Pack

find_package(LLVM REQUIRED CONFIG)

include_directories(${LLVM_INCLUDE_DIRS})
add_definitions(${LLVM_DEFINITIONS})

set(LLVM_LINK_COMPONENTS core support native executionengine orcjit)

# end llvm


add_library(${TARGET_NAME}_shared SHARED ${SOURCES})
add_library(${TARGET_NAME}_static STATIC ${SOURCES})

add_dependencies(${TARGET_NAME}_shared update-pot)
add_dependencies(${TARGET_NAME}_static update-pot)


target_include_directories(${TARGET_NAME}_shared PUBLIC inc)

target_link_libraries(${TARGET_NAME}_shared PUBLIC
  qaos_basis
  
  LLVM
)

set_target_properties(${TARGET_NAME}_shared PROPERTIES 
  OUTPUT_NAME "${TARGET_NAME}"
)



target_include_directories(${TARGET_NAME}_static PUBLIC inc)

target_link_libraries(${TARGET_NAME}_static PUBLIC
  qaos_basis
  
  LLVM
)

set_target_properties(${TARGET_NAME}_static PROPERTIES 
  OUTPUT_NAME "${TARGET_NAME}"
)

