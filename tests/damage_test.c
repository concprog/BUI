/* Damage: after a layout change (text that grows, a child added or removed, a node hidden,
 * a panel collapsed, a tab switched) or a property that keeps a node's size (an icon, a font
 * scale) the partial frame equals the same frame drawn in full, and only the changed part is
 * redrawn, as is a z-order change that moves no rect. A change handler may free its node from
 * inside a setter.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "BUI_kit.h"
#include "GPU_raster.h"

enum { W = 320, H = 480 };

static int g_fail = 0;
#define CHECK(c) \
  do { \
    if (!(c)) { \
      printf("FAIL %s:%d: %s\n", __FILE__, __LINE__, #c); \
      g_fail = 1; \
    } \
  } while (0)

static uint32_t g_partial[W * H];

/* Renders the pending damage, then the whole window: the pixels that differ. */
static int missed(BKContext *ctx, BuiRaster *r, int *regions)
{
  *regions = BK_render(ctx, r, W, H, 0);
  memcpy(g_partial, BUI_raster_pixels(r), sizeof(g_partial));
  BK_render(ctx, r, W, H, 1);
  const uint32_t *px = BUI_raster_pixels(r);
  int n = 0;
  for (int i = 0; i < W * H; i++) {
    n += g_partial[i] != px[i];
  }
  return n;
}

static BKNode *add(BKNode *parent, const char *type, const char *text)
{
  BKNode *n = BK_node_new(type);
  if (text) {
    BK_set_str(n, "text", text);
  }
  BK_node_add(parent, n, -1);
  return n;
}

/* A change handler that frees its node, as a host's handler may. */
static BKNode *g_free_on_change;

static void emit(void *data, BKNode *node, int kind, void *ref, double a, double b)
{
  (void)data, (void)ref, (void)a, (void)b;
  if (kind == BK_EV_CHANGE && node == g_free_on_change) {
    g_free_on_change = NULL;
    BK_node_free(node);
  }
}

#define STEP(what) \
  do { \
    int regions; \
    const int m = missed(ctx, r, &regions); \
    if (m) { \
      printf("FAIL %s: %d pixels missed\n", what, m); \
      g_fail = 1; \
    } \
    CHECK(regions > 0); \
  } while (0)

int main(void)
{
  BK_set_animations(0);
  BKNode *col = BK_node_new("column");
  BKNode *label = add(col, "label", "Short");
  BKNode *button = add(col, "button", "Button");
  BKNode *ramp = add(col, "ramp", NULL); /* stop markers at both ends, under the band */
  BKNode *ramp2 = add(col, "ramp", NULL);
  add(col, "number", "Number");
  BKNode *panel = add(col, "panel", "Panel");
  add(panel, "checkbox", "Inside");
  add(panel, "slider", "Slider");
  BKNode *tabs = add(col, "tabs", NULL);
  add(add(tabs, "column", "One"), "button", "First tab");
  add(add(tabs, "column", "Two"), "text", "Second tab");
  add(col, "button", "Last");
  BKNode *num = add(add(col, "row", NULL), "number", NULL);
  BK_set_num(num, "weight", 0); /* the row sizes it to its text */
  /* two buttons stacked in one place: a z-order change moves no rect */
  BKNode *stack = add(col, "frame", NULL);
  BKNode *under = add(stack, "button", "Under");
  BKNode *over = add(stack, "button", "Over the other");
  BK_geom_set(stack, "reqheight", 30); /* placed children size no frame */
  for (BKNode *b = under; b; b = b == under ? over : NULL) {
    BK_geom_set_manager(b, BK_GEOM_PLACE);
    BK_geom_set(b, "width", 140);
    BK_geom_set(b, "height", 24);
  }
  BKContext *ctx = BK_context_new(col);
  BK_context_set_host(ctx, &(BKHost){.emit = emit});
  BuiRaster *r = BUI_raster_create(W, H);
  BK_render(ctx, r, W, H, 1);
  CHECK(BK_render(ctx, r, W, H, 0) == 0); /* nothing to redraw */

  BK_set_str(label, "text", "A label that is much longer than before");
  STEP("text grows");
  BKNode *extra = add(col, "button", "Added");
  STEP("child added");
  BK_node_remove(col, extra);
  BK_node_free(extra); /* a removed node is the caller's */
  STEP("child removed");
  BK_set_num(button, "visible", 0);
  STEP("hidden");
  BK_set_num(button, "visible", 1);
  STEP("shown");
  BK_set_num(panel, "collapsed", 1);
  STEP("panel collapsed");
  BK_set_num(panel, "collapsed", 0);
  STEP("panel opened");
  BK_set_num(tabs, "current", 1);
  STEP("tab switched");
  BK_set_num(button, "icon", BK_icon_id("ADD"));
  STEP("icon");
  BK_set_num(button, "font_scale", 1.5);
  STEP("font scale");
  g_free_on_change = ramp;
  BK_set_num(ramp, "add", 0.5); /* the handler frees the ramp before the setter returns */
  CHECK(g_free_on_change == NULL);
  STEP("freed by its change handler");
  int rr[4];
  BK_node_rect(ramp2, rr);
  g_free_on_change = ramp2;
  BK_press(ctx, rr[0] + rr[2] / 2, rr[1] + 2, BK_CTRL | BK_BUTTON1); /* Ctrl+click adds a stop */
  CHECK(g_free_on_change == NULL);
  BK_release(ctx, rr[0] + rr[2] / 2, rr[1] + 2, 0);
  STEP("freed by the change of a Ctrl+click");
  int nr[4], nr2[4];
  BK_node_rect(num, nr);
  BK_set_num(num, "value", 123456789.0); /* a programmatic write, no event */
  STEP("number value grows");
  BK_node_rect(num, nr2);
  CHECK(nr2[2] > nr[2]);
  int ur[4], vr[4];
  BK_node_rect(under, ur);
  BK_node_rect(over, vr);
  CHECK(ur[2] > 0 && ur[3] > 0 && memcmp(ur, vr, sizeof(ur)) == 0);
  CHECK(BK_node_move(stack, under, -1));
  STEP("z-order swapped");

  BUI_raster_free(r);
  BK_context_free(ctx);
  BK_node_free(col);
  if (!g_fail) {
    printf("damage_test: ok\n");
  }
  return g_fail;
}
