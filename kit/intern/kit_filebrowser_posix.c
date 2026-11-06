/* File browser platform parts for Linux, BSD and macOS: directory reading, home, time and
 * sleep. System folders are Blender's fsmenu_system_unix.cc / fsmenu_system_macos.mm
 * (blender/). Compiles to nothing on Windows (kit_filebrowser_win32.c).
 *
 * SPDX-License-Identifier: GPL-2.0-or-later */

#ifndef _WIN32

#  include <dirent.h>
#  include <stdio.h>
#  include <stdlib.h>
#  include <string.h>
#  include <sys/stat.h>
#  include <unistd.h>

#  include "kit_filebrowser_intern.h"

char fb_os_sep(void)
{
  return '/';
}

bool fb_os_is_sep(char c)
{
  return c == '/';
}

size_t fb_os_root_len(const char *p)
{
  (void)p;
  return 0;
}

bool fb_os_path_is_absolute(const char *p)
{
  return p[0] == '/';
}

bool fb_os_path_is_root(const char *p)
{
  return p[0] == '/' && p[1] == '\0';
}

bool fb_os_getcwd(char *buf, size_t size)
{
  return getcwd(buf, size) != NULL;
}

bool fb_os_mkdir(const char *p)
{
  return mkdir(p, 0777) == 0;
}

const char *fb_os_home(void)
{
  const char *h = getenv("HOME");
  return (h && h[0]) ? h : NULL;
}

void fb_os_localtime(time_t t, struct tm *r)
{
  localtime_r(&t, r);
}

void fb_os_sleep_ms(int ms)
{
  usleep((useconds_t)ms * 1000);
}

void fb_os_bookmarks_path(char *buf, size_t size)
{
  const char *xch = getenv("XDG_CONFIG_HOME");
  if (xch && xch[0]) {
    snprintf(buf, size, "%s/bui/bookmarks.txt", xch);
  }
  else if (fb_os_home()) {
    snprintf(buf, size, "%s/.config/bui/bookmarks.txt", fb_os_home());
  }
}

/* filelist_readjob_common.cc: stat follows links, lstat tells them apart. */
bool fb_os_read_dir(FBData *f)
{
  DIR *d = opendir(f->dir);
  if (!d) {
    return false;
  }
  struct dirent *de;
  while ((de = readdir(d)) != NULL) {
    if (strcmp(de->d_name, ".") == 0 || strcmp(de->d_name, "..") == 0) {
      continue;
    }
    char full[FB_PATH];
    fb_path_join(full, sizeof(full), f->dir, de->d_name);
    struct stat st, lst;
    const bool ok = stat(full, &st) == 0, lok = lstat(full, &lst) == 0;
    const struct stat *use = ok ? &st : &lst; /* a broken link reports itself */
    FBEntry *e = fb_entry_add(f, de->d_name);
    e->hidden = de->d_name[0] == '.';
    e->link = lok && S_ISLNK(lst.st_mode);
    e->kind = (ok && S_ISDIR(st.st_mode)) ? FB_KIND_DIR : FB_KIND_FILE;
    e->size = (ok || lok) ? (long long)use->st_size : 0;
    e->mtime = (ok || lok) ? use->st_mtime : 0;
  }
  closedir(d);
  return true;
}

#endif /* !_WIN32 */
