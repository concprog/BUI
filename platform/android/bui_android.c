/* BUI on Android: a NativeActivity host for the kit (no Java, no Gradle).
 *
 * The kit draws into a software raster; this file is what kit_window_x11.c is on Linux: it feeds
 * touch, keys and the soft keyboard to the kit and puts the damaged part of each frame on the
 * screen.
 *
 *  - Damage driven: nothing is drawn, and the thread sleeps in ALooper until an event, a timer
 *    or the kit asks for a frame. While the kit animates, the buffer queue (unlockAndPost) paces
 *    the loop to the display's refresh.
 *  - Only the damaged rectangles are converted (B,G,R,A raster words to the window's R,G,B,X
 *    bytes) and locked, so an idle blinking caret costs a few hundred pixels, not a frame.
 *  - Touch: one finger is the pointer (a tap is a click, a drag drags), two fingers scroll, and
 *    the pointer leaves the screen when the last finger does (there is no hover on glass).
 *
 * The application is `bui_app_build` (the root node of its UI), see demo.c.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

#include <android/configuration.h>
#include <android/keycodes.h>
#include <android/log.h>
#include <android/looper.h>
#include <android/native_activity.h>
#include <android/native_window.h>
#include <jni.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include "BUI_kit.h"
#include "GPU_raster.h"
#include "android_native_app_glue.h"
#include "bui_app.h"

#define LOG(...) __android_log_print(ANDROID_LOG_INFO, "bui", __VA_ARGS__)
#define MAX_TIMERS 8
#define TOUCH_SLOP_DP 8.0f

void (*bui_app_emit)(void *data, BKNode *node, int kind, void *ref, double a, double b);
static struct android_app *g_app;

struct AAssetManager *bui_android_assets(void)
{
  return g_app ? g_app->activity->assetManager : NULL;
}

typedef struct Timer {
  double due_ms; /* CLOCK_MONOTONIC; 0 = unused */
  int id;
} Timer;

typedef struct Shell {
  struct android_app *app;
  BKContext *ctx;
  BuiRaster *raster;
  int w, h;         /* the content area (the window less the system bars) */
  int ox, oy;       /* its top left in the window */
  int sw, sh;       /* the window */
  int dirty;        /* the kit asked for a frame, or the surface changed */
  int full;         /* the whole surface must be sent (new surface, resize) */
  int ime_on;
  float scale;
  float density;    /* dp to px */
  Timer timers[MAX_TIMERS];
  char *clipboard;
  /* touch */
  int pointer_down; /* the primary finger is pressed in the kit */
  int scrolling;    /* two fingers (or a drag that is not the widget's) scroll */
  int draggable;    /* the finger went down on a widget that drags (sliders, text, scrollbars) */
  float last_x, last_y, down_x, down_y, scroll_x, scroll_y;
} Shell;

static double now_ms(void)
{
  struct timespec ts;
  clock_gettime(CLOCK_MONOTONIC, &ts);
  return (double)ts.tv_sec * 1000.0 + (double)ts.tv_nsec / 1.0e6;
}

/* -- host hooks -------------------------------------------------------------------------------- */

static void host_emit(void *data, BKNode *node, int kind, void *ref, double a, double b)
{
  if (bui_app_emit) {
    bui_app_emit(data, node, kind, ref, a, b);
  }
}

static void host_request(void *data)
{
  ((Shell *)data)->dirty = 1;
}

static void host_schedule(void *data, int ms, int id)
{
  Shell *s = (Shell *)data;
  Timer *slot = NULL;
  for (int i = 0; i < MAX_TIMERS; i++) {
    if (s->timers[i].due_ms == 0.0 || s->timers[i].id == id) {
      slot = &s->timers[i];
      if (slot->id == id) {
        break;
      }
    }
  }
  if (slot) {
    slot->id = id;
    slot->due_ms = now_ms() + (ms > 0 ? ms : 1);
  }
  ALooper_wake(s->app->looper); /* the wait ends in time for it */
}

static char *host_clipboard_get(void *data)
{
  const Shell *s = (Shell *)data;
  return s->clipboard ? strdup(s->clipboard) : NULL;
}

static void host_clipboard_set(void *data, const char *text)
{
  Shell *s = (Shell *)data;
  free(s->clipboard);
  s->clipboard = strdup(text ? text : "");
}

/* -- surface ----------------------------------------------------------------------------------- */

static void surface_changed(Shell *s)
{
  ANativeWindow *win = s->app->window;
  if (!win) {
    return;
  }
  s->sw = ANativeWindow_getWidth(win);
  s->sh = ANativeWindow_getHeight(win);
  ANativeWindow_setBuffersGeometry(win, s->sw, s->sh, 0x2 /* WINDOW_FORMAT_RGBX_8888 */);
  /* the activity's content rectangle: the window less the status and navigation bars */
  const ARect c = s->app->contentRect;
  const int ok = c.right > c.left && c.bottom > c.top && c.right <= s->sw && c.bottom <= s->sh;
  s->ox = ok ? c.left : 0;
  s->oy = ok ? c.top : 0;
  s->w = ok ? c.right - c.left : s->sw;
  s->h = ok ? c.bottom - c.top : s->sh;
  const float density = (float)AConfiguration_getDensity(s->app->config);
  s->density = (density > 0 ? density : 160.0f) / 160.0f;
  /* Blender's 1.0 is a 20 px widget; 1.2 dp-units gives a 24 dp row, a finger-sized target
   * once the kit's own padding is added */
  s->scale = s->density * 1.2f;
  BK_set_scale(s->scale);
  BK_configure(s->ctx, s->w, s->h);
  s->dirty = s->full = 1;
}

/* Swap B and R (the raster's 0xAARRGGBB words are B,G,R,A bytes) over one rectangle. */
static void convert(const uint32_t *src, int src_stride, uint32_t *dst, int dst_stride,
                    const int r[4])
{
  for (int y = r[1]; y < r[1] + r[3]; y++) {
    const uint32_t *in = src + (size_t)y * src_stride + r[0];
    uint32_t *out = dst + (size_t)y * dst_stride + r[0];
    for (int x = 0; x < r[2]; x++) {
      const uint32_t p = in[x];
      out[x] = (p & 0xFF00FF00u) | ((p & 0xFFu) << 16) | ((p >> 16) & 0xFFu) | 0xFF000000u;
    }
  }
}

static void draw(Shell *s)
{
  ANativeWindow *win = s->app->window;
  if (!win || s->w <= 0 || s->h <= 0) {
    return;
  }
  if (!s->raster) {
    s->raster = BUI_raster_create(s->w, s->h);
  }
  const int full = s->full || BUI_raster_width(s->raster) != s->w ||
                   BUI_raster_height(s->raster) != s->h;
  s->full = 0;
  s->dirty = 0;
  const int regions = BK_render(s->ctx, s->raster, s->w, s->h, full);
  if (!regions && !full) {
    return;
  }
  int rects_all[1][4] = {{0, 0, s->w, s->h}};
  const int(*rects)[4] = rects_all;
  int n = 1;
  ARect dirty = {s->ox, s->oy, s->ox + s->w, s->oy + s->h};
  if (!full) {
    n = BUI_raster_damage(s->raster, &rects);
    if (n <= 0) {
      return;
    }
    int x0 = s->w, y0 = s->h, x1 = 0, y1 = 0;
    for (int i = 0; i < n; i++) {
      x0 = rects[i][0] < x0 ? rects[i][0] : x0;
      y0 = rects[i][1] < y0 ? rects[i][1] : y0;
      x1 = rects[i][0] + rects[i][2] > x1 ? rects[i][0] + rects[i][2] : x1;
      y1 = rects[i][1] + rects[i][3] > y1 ? rects[i][1] + rects[i][3] : y1;
    }
    dirty.left = s->ox + x0;
    dirty.top = s->oy + y0;
    dirty.right = s->ox + x1;
    dirty.bottom = s->oy + y1;
  }
  ANativeWindow_Buffer buf;
  if (ANativeWindow_lock(win, &buf, &dirty) != 0) {
    s->full = s->dirty = 1; /* try again with the next event */
    return;
  }
  /* The buffer is recycled: outside `dirty` it holds an older frame, which locking brought up to
   * date; if the window handed back a larger rectangle, send that too. */
  if (buf.width != s->sw || buf.height != s->sh || buf.format != 0x2) {
    ANativeWindow_unlockAndPost(win);
    surface_changed(s);
    return;
  }
  const uint32_t *src = BUI_raster_pixels(s->raster);
  uint32_t *dst = (uint32_t *)buf.bits + (size_t)s->oy * buf.stride + s->ox;
  if (full) {
    convert(src, s->w, dst, buf.stride, rects_all[0]);
  }
  else { /* locking may have widened the rectangle: send all of it */
    const int back[4] = {dirty.left - s->ox, dirty.top - s->oy, dirty.right - dirty.left,
                         dirty.bottom - dirty.top};
    convert(src, s->w, dst, buf.stride, back);
  }
  ANativeWindow_unlockAndPost(win);
}

/* -- soft keyboard ------------------------------------------------------------------------------ */

static void sync_keyboard(Shell *s)
{
  BKImeState st;
  if (!BK_ime_state(s->ctx, &st)) {
    return;
  }
  if (st.enabled && !s->ime_on) {
    ANativeActivity_showSoftInput(s->app->activity, ANATIVEACTIVITY_SHOW_SOFT_INPUT_FORCED);
  }
  else if (!st.enabled && s->ime_on) {
    ANativeActivity_hideSoftInput(s->app->activity, 0);
  }
  s->ime_on = st.enabled;
}

/* -- input -------------------------------------------------------------------------------------- */

static int mods_of(int32_t meta)
{
  return ((meta & AMETA_SHIFT_ON) ? BK_SHIFT : 0) | ((meta & AMETA_CTRL_ON) ? BK_CTRL : 0) |
         ((meta & AMETA_ALT_ON) ? BK_ALT : 0);
}

/* The character a key press types (KeyEvent.getUnicodeChar through JNI: NativeActivity gives the
 * key code and meta state, the layout is the framework's). */
static int unicode_of(struct android_app *app, int32_t code, int32_t meta)
{
  JavaVM *vm = app->activity->vm;
  JNIEnv *env = NULL;
  int attached = 0;
  if ((*vm)->GetEnv(vm, (void **)&env, JNI_VERSION_1_6) == JNI_EDETACHED) {
    if ((*vm)->AttachCurrentThread(vm, &env, NULL) != JNI_OK) {
      return 0;
    }
    attached = 1;
  }
  int ch = 0;
  jclass cls = (*env)->FindClass(env, "android/view/KeyEvent");
  if (cls) {
    jmethodID ctor = (*env)->GetMethodID(env, cls, "<init>", "(II)V");
    jmethodID get = (*env)->GetMethodID(env, cls, "getUnicodeChar", "(I)I");
    if (ctor && get) {
      jobject ev = (*env)->NewObject(env, cls, ctor, 0 /* ACTION_DOWN */, code);
      if (ev) {
        ch = (*env)->CallIntMethod(env, ev, get, meta);
        (*env)->DeleteLocalRef(env, ev);
      }
    }
    (*env)->DeleteLocalRef(env, cls);
  }
  if ((*env)->ExceptionCheck(env)) {
    (*env)->ExceptionClear(env);
  }
  if (attached) {
    (*vm)->DetachCurrentThread(vm);
  }
  return ch;
}

static const char *keysym_of(int32_t code)
{
  switch (code) {
    case AKEYCODE_DEL: return "BackSpace";
    case AKEYCODE_FORWARD_DEL: return "Delete";
    case AKEYCODE_ENTER:
    case AKEYCODE_NUMPAD_ENTER: return "Return";
    case AKEYCODE_TAB: return "Tab";
    case AKEYCODE_ESCAPE: return "Escape";
    case AKEYCODE_DPAD_LEFT: return "Left";
    case AKEYCODE_DPAD_RIGHT: return "Right";
    case AKEYCODE_DPAD_UP: return "Up";
    case AKEYCODE_DPAD_DOWN: return "Down";
    case AKEYCODE_MOVE_HOME: return "Home";
    case AKEYCODE_MOVE_END: return "End";
    case AKEYCODE_PAGE_UP: return "Prior";
    case AKEYCODE_PAGE_DOWN: return "Next";
    case AKEYCODE_INSERT: return "Insert";
  }
  return NULL;
}

static int utf8_encode(int cp, char out[5])
{
  if (cp < 0x80) {
    out[0] = (char)cp;
    out[1] = 0;
  }
  else if (cp < 0x800) {
    out[0] = (char)(0xC0 | (cp >> 6));
    out[1] = (char)(0x80 | (cp & 0x3F));
    out[2] = 0;
  }
  else if (cp < 0x10000) {
    out[0] = (char)(0xE0 | (cp >> 12));
    out[1] = (char)(0x80 | ((cp >> 6) & 0x3F));
    out[2] = (char)(0x80 | (cp & 0x3F));
    out[3] = 0;
  }
  else {
    out[0] = (char)(0xF0 | (cp >> 18));
    out[1] = (char)(0x80 | ((cp >> 12) & 0x3F));
    out[2] = (char)(0x80 | ((cp >> 6) & 0x3F));
    out[3] = (char)(0x80 | (cp & 0x3F));
    out[4] = 0;
  }
  return cp;
}

static int32_t on_key(Shell *s, AInputEvent *e)
{
  if (AKeyEvent_getAction(e) == AKEY_EVENT_ACTION_UP) {
    return 0;
  }
  const int32_t code = AKeyEvent_getKeyCode(e), meta = AKeyEvent_getMetaState(e);
  const int mods = mods_of(meta);
  if (code == AKEYCODE_BACK) { /* back closes a popup or leaves a text field; else the app goes back */
    return BK_key(s->ctx, "", "Escape", 0) ? 1 : 0;
  }
  const char *ks = keysym_of(code);
  if (ks) {
    return BK_key(s->ctx, "", ks, mods);
  }
  const int cp = unicode_of(s->app, code, meta & ~(AMETA_CTRL_ON | AMETA_META_ON));
  if (cp > 0 && cp < 0x10FFFF && cp != 0x7F && (cp >= 32 || cp == '\t')) {
    char text[5];
    utf8_encode(cp, text);
    const char *sym = cp == ' ' ? "space" : cp == '-' ? "minus" : text;
    return BK_key(s->ctx, text, sym, mods);
  }
  return 0;
}

/* Widgets that use a drag themselves; a drag that starts anywhere else scrolls the page. */
static int drags(const BKNode *n)
{
  static const char *const types[] = {"slider", "number", "scrollbar", "text", "textarea", "panel",
                                      "nodeeditor", "canvas", "curve", "ramp", "picker", "normal",
                                      "swatch", "histogram", "eyedropper", "custom"};
  const char *t = n ? BK_node_type(n) : "";
  for (size_t i = 0; i < sizeof(types) / sizeof(types[0]); i++) {
    if (strcmp(t, types[i]) == 0) {
      return 1;
    }
  }
  return 0;
}

static void pointer_up(Shell *s, float x, float y)
{
  if (s->pointer_down) {
    BK_motion(s->ctx, (int)x, (int)y, 0);
    BK_release(s->ctx, (int)x, (int)y, 0);
    s->pointer_down = 0;
  }
  BK_motion(s->ctx, -1, -1, 0); /* no hover once the finger is gone */
}

static int32_t on_touch(Shell *s, AInputEvent *e)
{
  const int32_t action = AMotionEvent_getAction(e) & AMOTION_EVENT_ACTION_MASK;
  const size_t count = AMotionEvent_getPointerCount(e);
  const float x = AMotionEvent_getX(e, 0) - (float)s->ox, y = AMotionEvent_getY(e, 0) - (float)s->oy;
  const float slop = TOUCH_SLOP_DP * s->density;
  switch (action) {
    case AMOTION_EVENT_ACTION_DOWN:
      s->down_x = s->last_x = x;
      s->down_y = s->last_y = y;
      s->scrolling = 0;
      s->scroll_x = x;
      s->scroll_y = y;
      BK_motion(s->ctx, (int)x, (int)y, 0);
      s->draggable = drags(BK_hover(s->ctx));
      BK_press(s->ctx, (int)x, (int)y, 0);
      s->pointer_down = 1;
      break;
    case AMOTION_EVENT_ACTION_POINTER_DOWN:
      if (count >= 2 && !s->scrolling) { /* a second finger: scroll instead of drag */
        if (s->pointer_down) {
          BK_motion(s->ctx, -100, -100, 0); /* released outside: no click, no drag */
          BK_release(s->ctx, -100, -100, 0);
          s->pointer_down = 0;
        }
        s->scrolling = 1;
        s->scroll_x = (AMotionEvent_getX(e, 0) + AMotionEvent_getX(e, 1)) * 0.5f;
        s->scroll_y = (AMotionEvent_getY(e, 0) + AMotionEvent_getY(e, 1)) * 0.5f;
      }
      break;
    case AMOTION_EVENT_ACTION_MOVE:
      if (s->scrolling && count >= 2) {
        const float cx = (AMotionEvent_getX(e, 0) + AMotionEvent_getX(e, 1)) * 0.5f;
        const float cy = (AMotionEvent_getY(e, 0) + AMotionEvent_getY(e, 1)) * 0.5f;
        const float notch = (float)BK_unit() * 2.0f; /* px per wheel step */
        BK_scroll(s->ctx, (int)cx, (int)cy, (s->scroll_x - cx) / notch, (cy - s->scroll_y) / notch, 0);
        s->scroll_x = cx;
        s->scroll_y = cy;
      }
      else if (s->scrolling && count == 1) { /* a one-finger scroll */
        const float notch = (float)BK_unit() * 2.0f;
        BK_scroll(s->ctx, (int)x, (int)y, (s->scroll_x - x) / notch, (y - s->scroll_y) / notch, 0);
        s->scroll_x = x;
        s->scroll_y = y;
      }
      else if (s->pointer_down) {
        const float dx = x - s->down_x, dy = y - s->down_y;
        if (!s->draggable && dx * dx + dy * dy > slop * slop) { /* not a click: scroll */
          BK_motion(s->ctx, -100, -100, 0); /* the press ends outside: no click */
          BK_release(s->ctx, -100, -100, 0);
          s->pointer_down = 0;
          s->scrolling = 1;
          s->scroll_x = x;
          s->scroll_y = y;
        }
        else {
          BK_motion(s->ctx, (int)x, (int)y, BK_BUTTON1);
        }
      }
      break;
    case AMOTION_EVENT_ACTION_POINTER_UP:
      if (count <= 2) {
        s->scrolling = 0;
      }
      break;
    case AMOTION_EVENT_ACTION_UP:
      s->scrolling = 0;
      pointer_up(s, x, y);
      break;
    case AMOTION_EVENT_ACTION_CANCEL:
      s->scrolling = 0;
      pointer_up(s, -100.0f, -100.0f);
      break;
    default:
      return 0;
  }
  s->dirty = 1;
  return 1;
}

static int32_t on_input(struct android_app *app, AInputEvent *e)
{
  Shell *s = (Shell *)app->userData;
  if (!s->ctx) {
    return 0;
  }
  const int32_t type = AInputEvent_getType(e);
  const int32_t source = AInputEvent_getSource(e);
  if (type == AINPUT_EVENT_TYPE_MOTION && (source & AINPUT_SOURCE_CLASS_POINTER)) {
    return on_touch(s, e);
  }
  if (type == AINPUT_EVENT_TYPE_KEY) {
    return on_key(s, e);
  }
  return 0;
}

/* -- lifecycle ---------------------------------------------------------------------------------- */

static void on_cmd(struct android_app *app, int32_t cmd)
{
  Shell *s = (Shell *)app->userData;
  switch (cmd) {
    case APP_CMD_INIT_WINDOW:
    case APP_CMD_WINDOW_RESIZED:
    case APP_CMD_CONFIG_CHANGED:
    case APP_CMD_CONTENT_RECT_CHANGED:
      if (app->window) {
        surface_changed(s);
      }
      break;
    case APP_CMD_TERM_WINDOW:
      s->full = 1; /* the next surface starts empty */
      break;
    case APP_CMD_GAINED_FOCUS:
    case APP_CMD_RESUME:
      s->full = s->dirty = 1;
      break;
  }
}

/* Milliseconds the loop may sleep: forever when idle. */
static int wait_ms(Shell *s)
{
  if (s->dirty || BK_animating(s->ctx)) {
    return 0;
  }
  double next = 0.0;
  for (int i = 0; i < MAX_TIMERS; i++) {
    if (s->timers[i].due_ms != 0.0 && (next == 0.0 || s->timers[i].due_ms < next)) {
      next = s->timers[i].due_ms;
    }
  }
  if (next == 0.0) {
    return -1;
  }
  const double d = next - now_ms();
  return d <= 0.0 ? 0 : (int)d + 1;
}

static void fire_timers(Shell *s)
{
  const double t = now_ms();
  for (int i = 0; i < MAX_TIMERS; i++) {
    if (s->timers[i].due_ms != 0.0 && s->timers[i].due_ms <= t) {
      const int id = s->timers[i].id;
      s->timers[i].due_ms = 0.0;
      BK_timer(s->ctx, id);
    }
  }
}

void android_main(struct android_app *app)
{
  Shell s = {0};
  s.app = g_app = app;
  s.scale = 1.0f;
  app->userData = &s;
  app->onAppCmd = on_cmd;
  app->onInputEvent = on_input;

  BKNode *root = bui_app_build();
  s.ctx = BK_context_new(root);
  BKHost host = {0};
  host.data = &s;
  host.request = host_request;
  host.schedule = host_schedule;
  host.clipboard_get = host_clipboard_get;
  host.clipboard_set = host_clipboard_set;
  host.emit = host_emit;
  BK_context_set_host(s.ctx, &host);
  bui_app_fonts();

  while (!app->destroyRequested) {
    struct android_poll_source *src;
    int events;
    while (ALooper_pollOnce(wait_ms(&s), NULL, &events, (void **)&src) >= 0) {
      if (src) {
        src->process(app, src);
      }
      if (app->destroyRequested) {
        break;
      }
    }
    if (app->destroyRequested) {
      break;
    }
    fire_timers(&s);
    if (app->window && (s.dirty || BK_needs_render(s.ctx) || BK_animating(s.ctx))) {
      draw(&s);
      sync_keyboard(&s);
    }
  }
  if (s.raster) {
    BUI_raster_free(s.raster);
  }
  BK_context_free(s.ctx);
  free(s.clipboard);
}
