// Runs a blenderui app (compiled to WebAssembly with blenderui-web) in a <canvas>.
//
//   import { mount } from './bui-web.js'
//   const app = await mount(document.querySelector('canvas'), 'counter.wasm')
//   app.stop()
//
// The module draws off-screen: this calls into it on every animation frame, puts the frame it returns
// on the canvas, and passes it the page's pointer, wheel and key events. `load` is the same without
// a canvas (Node.js, tests).
//
// SPDX-License-Identifier: GPL-2.0-or-later

const ENOSYS = 52
const EBADF = 8

/** The part of WASI (preview 1) a blenderui module uses: a clock, random bytes, stdout and stderr as the
 *  console, and "not supported" for files, which a page has none of. */
function wasiImports (module, memory) {
  const view = () => new DataView(memory.buffer)
  const text = new TextDecoder()
  const lines = { 1: '', 2: '' }
  const known = {
    random_get (ptr, len) {
      const bytes = new Uint8Array(memory.buffer, ptr, len)
      for (let i = 0; i < len; i += 65536) crypto.getRandomValues(bytes.subarray(i, Math.min(len, i + 65536)))
      return 0
    },
    environ_sizes_get (count, size) {
      view().setUint32(count, 0, true)
      view().setUint32(size, 0, true)
      return 0
    },
    environ_get: () => 0,
    clock_time_get (id, _precision, out) {
      // 0: wall clock, anything else: monotonic
      const ms = id === 0 ? Date.now() : performance.now()
      view().setBigUint64(out, BigInt(Math.round(ms * 1e6)), true)
      return 0
    },
    fd_write (fd, iovs, count, written) {
      const v = view()
      let total = 0
      let out = ''
      for (let i = 0; i < count; i++) {
        const ptr = v.getUint32(iovs + i * 8, true)
        const len = v.getUint32(iovs + i * 8 + 4, true)
        out += text.decode(new Uint8Array(memory.buffer, ptr, len), { stream: true })
        total += len
      }
      v.setUint32(written, total, true)
      if (fd === 1 || fd === 2) {
        // whole lines to the console
        lines[fd] += out
        const parts = lines[fd].split('\n')
        lines[fd] = parts.pop()
        for (const line of parts) (fd === 1 ? console.log : console.error)(line)
      }
      return 0
    },
    fd_prestat_get: () => EBADF, // no preopened directories
    proc_exit (code) {
      throw new Error(`the module exited with code ${code}`)
    },
    sched_yield: () => 0,
  }
  const imports = {}
  for (const { module: ns, name, kind } of WebAssembly.Module.imports(module)) {
    if (ns !== 'wasi_snapshot_preview1' || kind !== 'function') continue
    imports[name] = known[name] ?? (() => ENOSYS)
  }
  return { wasi_snapshot_preview1: imports }
}

/** Instantiates a module (a URL, a Response or bytes). Returns its exports. */
export async function load (source) {
  const bytes = typeof source === 'string' || source instanceof URL ? await fetch(source) : source
  const module = bytes instanceof Response
    ? await WebAssembly.compileStreaming(bytes)
    : await WebAssembly.compile(bytes)
  const memory = { buffer: null }
  const imports = wasiImports(module, memory)
  const instance = await WebAssembly.instantiate(module, imports)
  Object.defineProperty(memory, 'buffer', { get: () => instance.exports.memory.buffer })
  return instance.exports
}

// KeyboardEvent.key -> the X11 key names the kit takes
const KEYSYMS = {
  Enter: 'Return', Backspace: 'BackSpace', Delete: 'Delete', Tab: 'Tab', Escape: 'Escape',
  ArrowLeft: 'Left', ArrowRight: 'Right', ArrowUp: 'Up', ArrowDown: 'Down',
  Home: 'Home', End: 'End', PageUp: 'Prior', PageDown: 'Next', Insert: 'Insert', ' ': 'space',
}

/** Writes a string into the module's memory for `bui_key`; the module frees it. */
function putString (exports, s) {
  const bytes = new TextEncoder().encode(s)
  const ptr = exports.bui_alloc(bytes.length)
  new Uint8Array(exports.memory.buffer, ptr, bytes.length).set(bytes)
  return [ptr, bytes.length]
}

/**
 * Starts the app in `canvas`. `source` is the .wasm (a URL or bytes). Options: `fit` (default true) makes
 * the canvas follow its CSS size; otherwise it keeps `width` x `height`. Returns `{ exports, stop }`.
 */
export async function mount (canvas, source, { fit = true, width = 640, height = 480 } = {}) {
  const exports = await load(source)
  const ctx = canvas.getContext('2d')
  canvas.tabIndex = canvas.tabIndex >= 0 ? canvas.tabIndex : 0 // it can take the keyboard
  canvas.style.touchAction = 'none'
  const css = () => (fit ? [Math.max(1, canvas.clientWidth || width), Math.max(1, canvas.clientHeight || height)] : [width, height])
  const scale = () => window.devicePixelRatio || 1

  const [w0, h0] = css()
  canvas.width = Math.round(w0 * scale())
  canvas.height = Math.round(h0 * scale())
  if (!exports.bui_start(w0, h0, scale())) throw new Error('the app did not open a window')

  let running = true
  let frameId = 0
  const draw = () => {
    if (!running) return
    if (exports.bui_frame()) {
      const [w, h] = [exports.bui_width(), exports.bui_height()]
      if (canvas.width !== w || canvas.height !== h) {
        canvas.width = w
        canvas.height = h
      }
      ctx.putImageData(new ImageData(new Uint8ClampedArray(exports.memory.buffer, exports.bui_pixels(), w * h * 4), w, h), 0, 0)
    }
    frameId = requestAnimationFrame(draw)
  }
  frameId = requestAnimationFrame(draw)

  const at = (e) => [e.offsetX, e.offsetY]
  const on = (type, fn, options) => {
    canvas.addEventListener(type, fn, options)
    return () => canvas.removeEventListener(type, fn, options)
  }
  const off = [
    on('pointermove', (e) => exports.bui_pointer_move(...at(e))),
    on('pointerdown', (e) => {
      canvas.focus()
      canvas.setPointerCapture(e.pointerId)
      exports.bui_pointer_move(...at(e))
      exports.bui_pointer_down(...at(e))
    }),
    on('pointerup', (e) => exports.bui_pointer_up(...at(e))),
    on('wheel', (e) => {
      e.preventDefault()
      const lines = Math.sign(e.deltaY) * Math.max(1, Math.round(Math.abs(e.deltaY) / 40))
      exports.bui_wheel(...at(e), lines)
    }, { passive: false }),
    on('keydown', (e) => {
      const text = e.key.length === 1 && !e.ctrlKey && !e.metaKey ? e.key : ''
      const keysym = KEYSYMS[e.key] ?? (e.key.length === 1 ? e.key : null)
      if (keysym === null) return
      const [tp, tl] = putString(exports, text)
      const [kp, kl] = putString(exports, keysym)
      exports.bui_key(tp, tl, kp, kl, e.ctrlKey || e.metaKey ? 1 : 0, e.shiftKey ? 1 : 0)
      e.preventDefault()
    }),
  ]
  const resize = () => {
    const [w, h] = css()
    exports.bui_resize(w, h, scale())
  }
  let observer = null
  if (fit && typeof ResizeObserver !== 'undefined') {
    observer = new ResizeObserver(resize)
    observer.observe(canvas)
  }

  return {
    exports,
    resize,
    stop () {
      running = false
      cancelAnimationFrame(frameId)
      observer?.disconnect()
      for (const f of off) f()
    },
  }
}
