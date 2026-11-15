/* What an Android app gives the shell (bui_android.c).
 * SPDX-License-Identifier: GPL-2.0-or-later */
#ifndef BUI_APP_H
#define BUI_APP_H

#include "BUI_kit.h"

/* The root node of the UI, made once when the activity starts. */
BKNode *bui_app_build(void);

/* Registers the fonts the app draws with (the system's: nothing is embedded). */
void bui_app_fonts(void);

/* The app's event sink (BKHost.emit: clicks, value changes ... as the kit reports them). The app
 * sets it in bui_app_build; NULL when it handles nothing (the demo). */
extern void (*bui_app_emit)(void *data, BKNode *node, int kind, void *ref, double a, double b);

/* The activity's assets (the APK's assets/ folder), for apps that load scripts or data. */
struct AAssetManager;
struct AAssetManager *bui_android_assets(void);

#endif
