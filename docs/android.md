# BUI on Android

`platform/android` runs the kit on a phone: one `NativeActivity` (no Java, no Gradle), the software
raster, touch and the soft keyboard. Nothing outside `platform/android` and the `if(ANDROID)` blocks
of the top-level `CMakeLists.txt` and `core/CMakeLists.txt` changed, so no other platform builds
differently.

```
python platform/android/build.py                  # arm64-v8a + x86_64 -> build-android/bui.apk
python platform/android/build.py --abi arm64-v8a  # a phone only
python platform/android/build.py --install        # adb install -r, then start it
```

Needs the Android SDK (`ANDROID_HOME`; build-tools, platform 34) and an NDK (the newest in the SDK, or
`ANDROID_NDK_HOME`), CMake and a JDK for `keytool`. The APK is signed with `~/.android/debug.keystore`.

## Your app

`bui_app.h` is the whole interface: `bui_app_build()` returns the root node of the UI and
`bui_app_fonts()` registers fonts. `demo.c` is a scrolling page of widgets; replace it with your own
(`platform/android/CMakeLists.txt` lists the sources).

## What it does

- **Damage driven.** Nothing is drawn and the thread sleeps in `ALooper` until a touch, key, timer or
  frame request. Each frame converts and locks only the damaged rectangles (B,G,R words to the
  window's R,G,B bytes). Idle cost: 0 CPU ticks in 10 s on the emulator.
- **Touch.** One finger is the pointer (tap = click). A drag that starts on a widget that drags
  (sliders, number fields, text, scrollbars, panels, canvases) drags it; anywhere else it scrolls
  the page. Two fingers always scroll. The pointer leaves the screen with the finger.
- **Keyboard.** The soft keyboard opens while a text field or area is being edited. Keys arrive as key
  events (`KeyEvent.getUnicodeChar`), so typing, Backspace, arrows and Back (Escape) work. Predictive
  and composing IMEs (`BK_ime_event`) need an `InputConnection` and are not wired up.
- **System bars.** The UI is drawn inside the activity's content rectangle, and the soft keyboard
  resizes it (`adjustResize`).
- **Scale.** `AConfiguration_getDensity` / 160 * 1.2 is the kit's scale (Blender's 1.0 is a 20 px widget).
- **Fonts.** Inter is left out of the build (`BUI_NO_INTER`, 0.8 MB); Roboto from `/system/fonts` is used.

## Size

Built with `-O2` and LTO, `-fvisibility=hidden`, `--gc-sections`, stripped, and with the GPU
renderers, Dear ImGui, the file browser and every image format but PNG and SVG off
(`features.json`): `libbui_app.so` is 2.5 MB (arm64-v8a) and an APK of one ABI is about 1.1 MB (the
library is deflated and extracted at install). `-Os` saved only 4%, so speed was kept. About 0.45 MB of
the library is the icon set.

## Not done

- No GPU renderer (the software raster draws; OpenGL ES would reuse `gpu_gl.c`, untested).
- No long-press as right-click, no pinch, no predictive keyboard or IME composition, no clipboard
  beyond the app's own, no cutout or gesture-bar insets beyond the content rectangle.
- Tested on an x86_64 emulator (API 33) only; the arm64 build compiles and links but was not run.
