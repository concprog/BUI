/* C side of package bui: the kit's callbacks forward to exported Go functions,
 * each window, context and callback named by a small integer id rather than a
 * pointer into Go memory (cgo's pointer rules).
 *
 * SPDX-License-Identifier: GPL-2.0-or-later */
#pragma once

#include <stdint.h>
#include <stdlib.h>

#include "BUI_kit.h"

void bui_go_set_host(BKContext *ctx, uintptr_t id);
void bui_go_set_ref_hooks(void);
void bui_go_window_callbacks(BKWindow *win, uintptr_t id);
int bui_go_wake_init(void);
/* The kit's `void *ref`s for Go are small integer ids. Go never holds one as
 * a pointer: the runtime aborts on a stack holding a pointer below 4096. They
 * cross as uintptr_t and become pointers only here. */
static inline void bui_go_bind(BKContext *ctx, const char *combo, uintptr_t id)
{
  BK_bind(ctx, combo, (void *)id);
}

static inline void bui_go_set_borrowed_ref(BKNode *node, uintptr_t id)
{
  BK_node_set_borrowed_ref(node, (void *)id);
}

static inline void bui_go_menu_ref(BKMenuSpec *item, uintptr_t id)
{
  item->ref = (void *)id;
}

/* The text a key event's ref points to. */
static inline const char *bui_go_text(uintptr_t ref)
{
  return (const char *)ref;
}
