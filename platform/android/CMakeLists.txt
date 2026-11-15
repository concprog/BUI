# BUI on Android: libbui_app.so, the kit + a NativeActivity host (bui_android.c) + an app.
# Built through the NDK's toolchain file; platform/android/build.py drives it and makes the APK.
#
# BUI_ANDROID_APP picks the app: demo (demo.c, C), lua (lua_app.c: assets/main.lua) or rust
# (a Rust staticlib that defines bui_app_build; BUI_ANDROID_RUST_LIB is its path).
#
# SPDX-License-Identifier: GPL-2.0-or-later

set(BUI_ANDROID_APP "demo" CACHE STRING "demo, lua or rust")
set(BUI_ANDROID_RUST_LIB "" CACHE FILEPATH "libXXX.a of the Rust app (BUI_ANDROID_APP=rust)")
set(_glue "${ANDROID_NDK}/sources/android/native_app_glue")

add_library(bui_app SHARED bui_android.c fonts.c ${_glue}/android_native_app_glue.c)
target_include_directories(bui_app PRIVATE ${_glue} ${CMAKE_CURRENT_SOURCE_DIR})
target_link_libraries(bui_app PRIVATE bui::kit android log)

if(BUI_ANDROID_APP STREQUAL "demo")
  target_sources(bui_app PRIVATE demo.c)
elseif(BUI_ANDROID_APP STREQUAL "lua")
  if(NOT TARGET bui_lua)
    message(FATAL_ERROR "BUI_ANDROID_APP=lua needs -DBUI_BUILD_LUA_BINDINGS=ON (it builds Lua 5.4)")
  endif()
  set(_lua ${BUI_SOURCE_DIR}/bindings/lua/src)
  target_sources(bui_app PRIVATE lua_app.c ${_lua}/bui_lua.c ${_lua}/bui_node.c ${_lua}/bui_context.c
                 ${_lua}/bui_window.c ${_lua}/bui_image.c ${_lua}/bui_anim.c)
  target_include_directories(bui_app PRIVATE ${_lua})
  target_link_libraries(bui_app PRIVATE bui_lua bui::gpu bui::core bui_api)
elseif(BUI_ANDROID_APP STREQUAL "rust")
  if(NOT BUI_ANDROID_RUST_LIB OR NOT EXISTS "${BUI_ANDROID_RUST_LIB}")
    message(FATAL_ERROR "BUI_ANDROID_APP=rust needs BUI_ANDROID_RUST_LIB (the app's staticlib)")
  endif()
  target_link_libraries(bui_app PRIVATE -Wl,--whole-archive "${BUI_ANDROID_RUST_LIB}" -Wl,--no-whole-archive)
else()
  message(FATAL_ERROR "BUI_ANDROID_APP must be demo, lua or rust")
endif()

# one exported symbol (ANativeActivity_onCreate, from the glue), no build ids, unused sections
# gone, 16 KB pages (Android 15+)
target_compile_options(bui_app PRIVATE -fvisibility=hidden)
target_link_options(bui_app PRIVATE -Wl,--gc-sections -Wl,--exclude-libs,ALL -Wl,--build-id=none
                    -Wl,-z,max-page-size=16384 -s)
# the glue's entry point must stay exported
set_source_files_properties(${_glue}/android_native_app_glue.c PROPERTIES COMPILE_OPTIONS
                            -fvisibility=default)
