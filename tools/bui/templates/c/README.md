# {{title}}

A [BUI](https://bui.preetham.top) app in C, built with CMake (BUI is fetched at release `{{tag}}`).

```sh
bui dev            # configure, build (Debug, in build/) and run it
bui build          # an optimized release build, in build-release/bin
bui doctor         # check the toolchain, and fix what is wrong
bui update         # move BUI to the newest release
```

`clang-format -i src/*.c` formats (see `.clang-format`); `clang-tidy src/main.c -p build` lints (see `.clang-tidy`).
