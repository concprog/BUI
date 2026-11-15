/* What an Android app gives the shell (bui_android.c).
 * SPDX-License-Identifier: GPL-2.0-or-later */
#ifndef BUI_APP_H
#define BUI_APP_H

#include "BUI_kit.h"

/* The root node of the UI, made once when the activity starts. */
BKNode *bui_app_build(void);

/* Registers the fonts the app draws with (the system's: nothing is embedded). */
void bui_app_fonts(void);

#endif
