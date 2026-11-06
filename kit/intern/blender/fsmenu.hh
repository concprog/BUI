/* BUI: stands in for editors/space_file/fsmenu.hh, the FSMenu enums of ED_fileselect.hh and
 * the icon ids of UI_resources.hh, for the fsmenu_system_*.cc copies in this folder.
 * FSMenu is the file browser's FBData; fsmenu_insert_entry adds to its place lists
 * (fb_place_add, kit_fsmenu.c), which keep Blender's category order.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later */
#pragma once

extern "C" {
#include "kit_filebrowser_intern.h"
}

#define N_(msgid) msgid

/* The icons fsmenu_read_system uses, by their names in Blender's icon set. */
#define BK_FSMENU_ICONS(X) \
  X(COMMUNITY) \
  X(DESKTOP) \
  X(DISC) \
  X(DISK_DRIVE) \
  X(DOCUMENTS) \
  X(EXTERNAL_DRIVE) \
  X(FILE_FOLDER) \
  X(FILE_FONT) \
  X(FILE_IMAGE) \
  X(FILE_MOVIE) \
  X(FILE_SOUND) \
  X(HOME) \
  X(IMPORT) \
  X(INTERNET) \
  X(NETWORK_DRIVE) \
  X(USB_DRIVE)
#define BK_FSMENU_ICON_ID(name) ICON_##name,
#define BK_FSMENU_ICON_NAME(name) #name,
enum : int { BK_FSMENU_ICONS(BK_FSMENU_ICON_ID) };
using BIFIconID = int;

namespace blender {

struct FSMenu;

enum FSMenuCategory {
  FS_CATEGORY_SYSTEM,
  FS_CATEGORY_SYSTEM_BOOKMARKS,
  FS_CATEGORY_BOOKMARKS,
  FS_CATEGORY_RECENT,
  /** For internal use, a list of known paths that are used to match paths to icons and names. */
  FS_CATEGORY_OTHER,
};
static_assert(FS_CATEGORY_SYSTEM == int(FB_CAT_VOLUMES) &&
                  FS_CATEGORY_SYSTEM_BOOKMARKS == int(FB_CAT_SYSTEM) &&
                  FS_CATEGORY_OTHER == int(FB_CAT_OTHER),
              "FBCategory follows FSMenuCategory");

enum FSMenuInsert {
  FS_INSERT_SORTED = (1 << 0),
  FS_INSERT_SAVE = (1 << 1),
  /** moves the item to the front of the list when its not already there */
  FS_INSERT_FIRST = (1 << 2),
  /** just append to preserve delivered order */
  FS_INSERT_LAST = (1 << 3),
};
static_assert(FS_INSERT_SORTED == int(FB_INSERT_SORTED) &&
                  FS_INSERT_FIRST == int(FB_INSERT_FIRST) && FS_INSERT_LAST == int(FB_INSERT_LAST),
              "FBInsert follows FSMenuInsert");

inline void fsmenu_insert_entry(FSMenu *fsmenu,
                                FSMenuCategory category,
                                const char *path,
                                const char *name,
                                int icon,
                                FSMenuInsert flag)
{
  static const char *const icon_names[] = {BK_FSMENU_ICONS(BK_FSMENU_ICON_NAME)};
  fb_place_add(reinterpret_cast<FBData *>(fsmenu),
               FBCategory(category),
               path,
               name,
               icon_names[icon],
               FBInsert(flag));
}

void fsmenu_read_system(FSMenu *fsmenu, int read_bookmarks);

}  // namespace blender
