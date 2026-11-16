# libbui_kit as a shared library for the C# binding: the static kit (and the
# gpu and core libraries it links) linked whole, so every BK_* symbol is
# exported. .NET loads it by the name "bui_kit" (libbui_kit.so / .dylib).
#
# SPDX-License-Identifier: GPL-2.0-or-later

if(MSVC)
  # MSVC exports only __declspec(dllexport) symbols, and BUI_kit.h has no
  # export macro yet
  message(STATUS "BUI: the C# binding's shared kit is not built with MSVC yet")
  return()
endif()
if(CMAKE_VERSION VERSION_LESS 3.24) # $<LINK_LIBRARY:WHOLE_ARCHIVE>
  message(STATUS "BUI: the C# binding's shared kit needs CMake 3.24 or later")
  return()
endif()

add_library(bui_kit_shared SHARED ${BUI_SOURCE_DIR}/kit/include/BUI_kit.h)
set_target_properties(bui_kit_shared PROPERTIES
  OUTPUT_NAME bui_kit
  LINKER_LANGUAGE CXX
  LIBRARY_OUTPUT_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
  RUNTIME_OUTPUT_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR})
target_link_libraries(bui_kit_shared PRIVATE "$<LINK_LIBRARY:WHOLE_ARCHIVE,bui_kit>")
if(WIN32)
  # MinGW: export every symbol of the archives
  target_link_options(bui_kit_shared PRIVATE -Wl,--export-all-symbols)
endif()
