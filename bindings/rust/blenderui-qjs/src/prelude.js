// The globals a BUI app expects (console, timers, performance, process) and
// `__BUI_NATIVE__`: the same API as the Node.js addon (`NativeApp`), on the
// QuickJS host. Runs before the app's bundle. `host` is the Rust side.
// SPDX-License-Identifier: GPL-2.0-or-later
(function (host) {
  'use strict'
  const g = globalThis

  // ---- console ---------------------------------------------------------------
  function fmt (v, depth = 0) {
    if (typeof v === 'string') return v
    if (v instanceof Error) return v.stack ? `${v.name}: ${v.message}\n${v.stack}` : String(v)
    if (typeof v === 'function') return `[Function ${v.name || 'anonymous'}]`
    if (typeof v !== 'object' || v === null) return String(v)
    if (depth > 2) return Array.isArray(v) ? '[Array]' : '[Object]'
    try {
      if (Array.isArray(v)) return `[ ${v.map((x) => fmt(x, depth + 1)).join(', ')} ]`
      const parts = Object.keys(v).map((k) => `${k}: ${typeof v[k] === 'string' ? JSON.stringify(v[k]) : fmt(v[k], depth + 1)}`)
      return `{ ${parts.join(', ')} }`
    } catch (e) {
      return String(v)
    }
  }
  const log = (level) => (...args) => host.print(level, args.map((a) => fmt(a)).join(' '))
  g.console = { log: log(0), info: log(0), debug: log(0), warn: log(1), error: log(2), trace: log(2) }

  // ---- timers ----------------------------------------------------------------
  const now = () => host.now()
  const timers = new Map() // id -> { at, fn, args, every }
  let nextTimer = 1
  // the delay as browsers take it (a WebIDL long): not a number, negative, infinite or past
  // 2^31-1 ms is 0
  const MAX_DELAY = 0x7fffffff
  function addTimer (fn, ms, args, every) {
    const id = nextTimer++
    const d = Number(ms)
    const delay = d > 0 && d <= MAX_DELAY ? d : 0
    timers.set(id, { at: now() + delay, fn, args, every: every ? Math.max(1, delay) : 0 })
    return id
  }
  const clear = (id) => { timers.delete(id) }
  g.setTimeout = (fn, ms, ...args) => addTimer(fn, ms, args, false)
  g.setInterval = (fn, ms, ...args) => addTimer(fn, ms, args, true)
  g.setImmediate = (fn, ...args) => addTimer(fn, 0, args, false)
  g.clearTimeout = g.clearInterval = g.clearImmediate = clear
  g.queueMicrotask = (fn) => { Promise.resolve().then(fn) }
  g.performance = { now, timeOrigin: Date.now() - now() }

  // The timers due when a batch began, in deadline order. The host runs them one task at a
  // time, with the promise jobs after each (HTML's event loop).
  let due = []
  let head = 0

  /** Run the next due timer; true when more of this batch are left. */
  g.__bui_tick = () => {
    if (head === due.length) {
      const t = now()
      due = []
      head = 0
      for (const [id, tm] of timers) if (tm.at <= t) due.push(id)
      if (due.length === 0) return false
      due.sort((a, b) => timers.get(a).at - timers.get(b).at || a - b)
    }
    const id = due[head++]
    const tm = timers.get(id)
    if (tm) {
      if (tm.every) tm.at = now() + tm.every
      else timers.delete(id)
      try {
        tm.fn(...tm.args)
      } catch (e) {
        console.error(e)
      }
    }
    return head < due.length
  }

  /** The next deadline (ms), or -1 for no timer. */
  g.__bui_next = () => {
    let next = -1
    for (const tm of timers.values()) if (next < 0 || tm.at < next) next = tm.at
    return next
  }

  // ---- process ----------------------------------------------------------------
  const rss = () => host.rss()
  g.process = {
    argv: host.argv(),
    env: host.env(),
    platform: host.platform(),
    arch: host.arch(),
    versions: { quickjs: 'quickjs-ng', bui: '0.1.0' },
    exit: (code) => host.exit(code ?? 0),
    nextTick: (fn, ...args) => g.queueMicrotask(() => fn(...args)),
    cpuUsage: () => ({ user: 0, system: 0 }),
    memoryUsage: Object.assign(() => ({ rss: rss(), heapUsed: 0, heapTotal: 0, external: 0, arrayBuffers: 0 }), { rss }),
    on () { return g.process },
    cwd: () => '.',
  }

  // ---- the native API (as the Node.js addon has it) ----------------------------
  let onEvent = null
  const requests = new Map()
  let nextRequest = 1
  function request (send) {
    const id = nextRequest++
    return new Promise((resolve, reject) => {
      requests.set(id, { resolve, reject })
      send(id)
    })
  }
  g.__bui_resolve = (id, ok, value) => {
    const r = requests.get(id)
    if (!r) return
    requests.delete(id)
    if (ok) r.resolve(value)
    else r.reject(new Error(value))
  }
  g.__bui_dispatch = (e) => { if (onEvent !== null) onEvent(e) }
  g.__bui_busy = () => requests.size > 0

  class NativeApp {
    constructor (options, callback) {
      this._options = options || {}
      onEvent = callback
      if (this._options.slug != null) host.setSlug(!!this._options.slug)
    }

    get headless () { return host.headless() }
    openWindow (id, c) {
      c = c || {}
      host.openWindow(id, { ...c, width: c.width ?? 800, height: c.height ?? 600,
        gpu: c.gpu ?? this._options.gpu })
    }

    commit (window, ops, strings, animated, motion) { host.commit(window, ops, strings, !!animated, motion) }
    requestFrame (window) { host.requestFrame(window) }
    closeWindow (window) { host.closeWindow(window) }
    setSlug (on) { host.setSlug(!!on) }
    quit () { host.quit() }
    screenshot (window) { return request((id) => host.screenshot(window, id)) }
    bounds (window, node) { return request((id) => host.bounds(window, node, id)) }
    simulateClick (window, x, y) { host.simulateClick(window, x, y) }
  }

  g.__BUI_NATIVE__ = {
    NativeApp,
    processCpuCycles: () => host.cpuCycles(),
    timestampCounter: () => host.tsc(),
  }
})
