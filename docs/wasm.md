# BUI in a web page (WebAssembly)

A blenderui app (Rust) can run in a browser: it is compiled to `wasm32-wasip1`, draws with BUI's software
raster, and a small JavaScript module puts each frame on a `<canvas>` and passes the page's pointer, wheel
and key events to it. It is Blender's UI drawn by the same C kit as on a desktop, so the pixels are the
same; there is no DOM and no browser CSS involved.

```
python platform/wasm/build.py examples/wasm/counter --serve      # then open http://localhost:8000
```

## Writing an app

An app is a Rust `cdylib` crate. Its UI is the code it would be on a desktop; the only addition is
`blenderui_web::export!`, which opens the window with the canvas's size and defines what the page calls:

```rust
use blenderui::prelude::*;

struct Counter { count: i32 }

impl Render for Counter {
    fn render(&mut self, _w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        body().styled().m_0().css("height", "100vh").p_4()
            .child(h1().child(format!("Count: {}", self.count)))
            .child(button().child("Add one").on_click(cx.listener(|this, _, _, cx| {
                this.count += 1;
                cx.notify();
            })))
    }
}

blenderui_web::export!(|cx, options| {
    cx.open_window(options, |_, cx| cx.new(|_| Counter { count: 0 })).unwrap();
});
```

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
blenderui = { version = "0.5", default-features = false, features = ["png", "svg", "freetype", "anim", "video"] }
blenderui-web = "0.5"
```

(`video` is in the list because the kit's video code needs its audio code to link; nothing plays.) Give the
root `height: 100vh` (the window is not a scrolling document), and `m_0()` to drop the body's 8 px margin.

## Building

`python platform/wasm/build.py CRATE` builds the crate and writes `CRATE/dist/` (or `--out DIR`):
`app.wasm`, `bui-web.js` and an `index.html` that mounts it. `--serve [PORT]` then serves it.

It needs:

* Rust with the target: `rustup target add wasm32-wasip1` (the script adds it),
* LLVM's clang 17 or newer with the `wasm32` backend, and CMake. Apple's clang has no `wasm32` backend:
  install LLVM and set `WASI_CLANG` to its `bin` folder,
* `make` or Ninja,
* the WASI sysroot (wasi-libc and libc++): `python platform/wasm/build.py --fetch-sysroot` downloads
  wasi-sdk's once into `~/.bui/wasi-sysroot` (0.2 GB download, 94 MB kept), or point `WASI_SYSROOT` at one.

`wasm-opt` (Binaryen), when it is installed, shrinks the module further.

The C kit is built for WebAssembly by CMake through `platform/wasm/wasi.cmake`, driven by
`blenderui-sys`'s build script, so `cargo build --target wasm32-wasip1` works in your own project when
`WASI_SYSROOT`, `CC_wasm32_wasip1`, `CXX_wasm32_wasip1` and `AR_wasm32_wasip1` are set (the script sets them).

## How it works

* The module has no `_start`: it is a library. `export!` defines `bui_start`, `bui_frame`, `bui_resize`,
  `bui_pointer_*`, `bui_wheel` and `bui_key`, and `bui_pixels`, `bui_width`, `bui_height` for the frame.
* `blenderui::Application::start` keeps a headless application alive between calls (`Application::run`
  would run its own loop and return when the windows close). The window is off-screen; the host asks
  `App::frame_if_needed` for a frame on each animation frame.
* A frame is drawn only when something changed or something animates; otherwise `bui_frame` returns 0 and
  the page does nothing. The frame is RGBA bytes in the module's memory, which the page puts on the canvas
  with `putImageData`.
* `bui-web.js` is the whole page side: a small WASI (preview 1) shim (a clock, random bytes, stdout and
  stderr as `console`), the canvas, the frame loop and the events. It follows `devicePixelRatio` (a retina
  canvas draws twice the pixels) and the canvas's CSS size.

```js
import { mount } from './bui-web.js'
const app = await mount(document.querySelector('canvas'), './app.wasm')
// app.stop()
```

## Size

The counter in `examples/wasm/counter` is a 4.2 MB module (1.6 MB gzipped), release build with
`opt-level = "s"` and fat LTO. It instantiates in about 55 ms under Node.js, starts in about 50 ms, and
draws its first frame in about 11 ms; a changed frame took 3 ms.

## What it does not do

* **One thread.** The kit's worker threads and the audio code do not exist. Loading images was not tried.
* **Software raster only.** No GPU renderers.
* **Input:** the primary pointer button, the wheel and the keyboard. No right or middle button, touch
  gestures, IME composition, clipboard, or mouse cursor shapes yet.
* **Fonts:** only the embedded Inter; there are no system fonts.
* **No files:** there is no file system, so no file browser and no file-backed lists.
* **Expressions:** WebAssembly has no floating-point exception flags, so a number field's `1/0` is infinity
  (a desktop reports an error).
* **A malformed font traps** instead of being rejected (FreeType's `setjmp` recovery is not available; the
  embedded font is fine).

## Speed

`node examples/wasm/bench.mjs app.wasm [width height]` times full redraws. The counter at 1280 x 800 takes about
3 to 5 ms a frame under Node.js on a laptop; copying the kit's pixels to the canvas's RGBA bytes took half of the
10 ms it took before that loop was written for the compiler to vectorize. Compiling the Rust side with
`simd128` made no measurable difference, so the build does not use it.

## Tested

Node.js 26 (`examples/wasm/smoke.mjs`: the first frame, no frame when idle, a click, a resize, a retina
resize) and Microsoft Edge 157 (Chromium) driven by Playwright on Windows: the page renders, three real
mouse clicks counted, typing and Backspace in a text field, no console output, sharp at a device pixel
ratio of 2. Firefox and Safari have not been tried. The build was run on Windows with LLVM 22; the script is
written for Linux and macOS too but was not run there.
