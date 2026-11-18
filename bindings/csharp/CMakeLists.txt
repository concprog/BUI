# libbui_kit as a shared library for the C# binding: the static kit (and the
# gpu and core libraries it links) in one library that exports every function
# the binding imports. .NET loads it by the name "bui_kit" (bui_kit.dll,
# libbui_kit.dylib, libbui_kit.so).
#
# SPDX-License-Identifier: GPL-2.0-or-later

if(CMAKE_VERSION VERSION_LESS 3.24) # $<LINK_LIBRARY:WHOLE_ARCHIVE>
  message(STATUS "BUI: the C# binding's shared kit needs CMake 3.24 or later")
  return()
endif()

# Visual Studio links a library only when it compiles something: one empty source
file(CONFIGURE OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/bui_kit_shared.c
  CONTENT "/* the shared kit: all of its code comes from the static libraries */\ntypedef int bui_kit_shared_unit;\n")
add_library(bui_kit_shared SHARED ${BUI_SOURCE_DIR}/kit/include/BUI_kit.h
  ${CMAKE_CURRENT_BINARY_DIR}/bui_kit_shared.c)
# $<0:> keeps multi-config generators (Visual Studio, Xcode) from adding a
# Release/ or Debug/ directory: the projects find the library in one place
set_target_properties(bui_kit_shared PROPERTIES
  OUTPUT_NAME bui_kit
  LINKER_LANGUAGE CXX
  LIBRARY_OUTPUT_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}$<0:>
  RUNTIME_OUTPUT_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}$<0:>)

if(MSVC)
  # MSVC exports only what it is told to. The export list is the set of
  # functions the binding imports (src/Bui/Interop/Native.g.cs, generated
  # from BUI_kit.h): the linker takes each one from the static libraries.
  set(imports ${CMAKE_CURRENT_SOURCE_DIR}/src/Bui/Interop/Native.g.cs)
  set_property(DIRECTORY APPEND PROPERTY CMAKE_CONFIGURE_DEPENDS ${imports})
  file(READ ${imports} src)
  string(REGEX MATCHALL "public static extern [^(;]+ [A-Za-z_][A-Za-z0-9_]*\\(" decls "${src}")
  set(def "EXPORTS\n")
  foreach(d IN LISTS decls)
    string(REGEX REPLACE ".* ([A-Za-z_][A-Za-z0-9_]*)\\($" "\\1" name "${d}")
    string(APPEND def "  ${name}\n")
  endforeach()
  file(CONFIGURE OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/bui_kit.def CONTENT "${def}")
  target_sources(bui_kit_shared PRIVATE ${CMAKE_CURRENT_BINARY_DIR}/bui_kit.def)
  target_link_libraries(bui_kit_shared PRIVATE bui_kit)
else()
  target_link_libraries(bui_kit_shared PRIVATE "$<LINK_LIBRARY:WHOLE_ARCHIVE,bui_kit>")
  if(WIN32)
    # MinGW: export every symbol of the archives
    target_link_options(bui_kit_shared PRIVATE -Wl,--export-all-symbols)
  endif()
endif()
