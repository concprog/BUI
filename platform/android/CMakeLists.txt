# BUI on Android: libbui_app.so, the kit + a NativeActivity host (bui_android.c) + an app (demo.c).
# Built through the NDK's toolchain file; platform/android/build.py drives it and makes the APK.
#
# SPDX-License-Identifier: GPL-2.0-or-later

set(_glue "${ANDROID_NDK}/sources/android/native_app_glue")

add_library(bui_app SHARED bui_android.c fonts.c demo.c ${_glue}/android_native_app_glue.c)
target_include_directories(bui_app PRIVATE ${_glue} ${CMAKE_CURRENT_SOURCE_DIR})
target_link_libraries(bui_app PRIVATE bui::kit android log)
# one exported symbol (ANativeActivity_onCreate, from the glue), no unwind tables, no build ids,
# unused sections gone, 16 KB pages (Android 15+)
target_compile_options(bui_app PRIVATE -fvisibility=hidden)
target_link_options(bui_app PRIVATE -Wl,--gc-sections -Wl,--exclude-libs,ALL -Wl,--build-id=none
                    -Wl,-z,max-page-size=16384 -s)
# the glue's entry point must stay exported
set_source_files_properties(${_glue}/android_native_app_glue.c PROPERTIES COMPILE_OPTIONS
                            -fvisibility=default)
