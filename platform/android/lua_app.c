/* A Lua app: assets/main.lua builds the UI and returns its root node.
 *
 * The Lua binding (bindings/lua) runs unchanged; this file is what bui-lua's main.c is on a
 * desktop: it owns the Lua state, finds `require`d files in the APK's assets (bui.lua, your own
 * modules), and hands the shell the root node. Callbacks (on_click, on_change ...) reach Lua
 * through the binding's event dispatcher. A script error is shown on screen, not just logged.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

#include <android/asset_manager.h>
#include <android/log.h>
#include <lauxlib.h>
#include <lua.h>
#include <lualib.h>
#include <stdlib.h>
#include <string.h>

#include "bui_app.h"
#include "bui_internal.h"

int luaopen__bui(lua_State *L);

/* Load assets/<name> (dots in a module name are folders) as a chunk; 0 when there is none. */
static int load_asset(lua_State *L, const char *path)
{
  AAsset *a = AAssetManager_open(bui_android_assets(), path, AASSET_MODE_BUFFER);
  if (!a) {
    return 0;
  }
  const int status = luaL_loadbuffer(L, (const char *)AAsset_getBuffer(a), (size_t)AAsset_getLength(a),
                                     path);
  AAsset_close(a);
  return status == LUA_OK ? 1 : -1;
}

/* package.searchers entry: `require("a.b")` reads assets/a/b.lua */
static int asset_searcher(lua_State *L)
{
  char path[256];
  const char *name = luaL_checkstring(L, 1);
  snprintf(path, sizeof(path), "%s.lua", name);
  for (char *c = path; *c; c++) {
    *c = *c == '.' && strcmp(c, ".lua") != 0 ? '/' : *c;
  }
  const int found = load_asset(L, path);
  if (found > 0) {
    lua_pushstring(L, path);
    return 2;
  }
  if (found < 0) {
    return luaL_error(L, "error loading '%s': %s", path, lua_tostring(L, -1));
  }
  lua_pushfstring(L, "no asset '%s'", path);
  return 1;
}

static BKNode *error_page(const char *message)
{
  __android_log_print(ANDROID_LOG_ERROR, "bui", "lua: %s", message);
  BKNode *page = BK_node_new("column");
  BK_set_num(page, "padding", 12);
  BKNode *label = BK_node_new("label");
  BK_set_str(label, "text", message);
  BK_node_add(page, label, -1);
  return page;
}

BKNode *bui_app_build(void)
{
  lua_State *L = luaL_newstate();
  if (!L) {
    return error_page("lua: out of memory");
  }
  luaL_openlibs(L);
  luaL_requiref(L, "_bui", luaopen__bui, 1); /* what bui.lua requires first */
  lua_pop(L, 1);
  lua_getglobal(L, "package");
  lua_getfield(L, -1, "searchers");
  const int n = (int)luaL_len(L, -1);
  lua_pushcfunction(L, asset_searcher);
  lua_rawseti(L, -2, n + 1);
  lua_pop(L, 2);

  luabui_set_host_state(L);
  bui_app_emit = luabui_host_emit;

  const int loaded = load_asset(L, "main.lua");
  if (loaded <= 0) {
    return error_page(loaded < 0 ? lua_tostring(L, -1) : "assets/main.lua is missing");
  }
  if (lua_pcall(L, 0, 1, 0) != LUA_OK) {
    return error_page(lua_tostring(L, -1));
  }
  luabui_Node *root = (luabui_Node *)luaL_testudata(L, -1, LUABUI_NODE_MT);
  if (!root || !root->node) {
    return error_page("main.lua must return the root node (return bui.Node(...))");
  }
  lua_setfield(L, LUA_REGISTRYINDEX, "bui.android.root"); /* the script's node outlives this call */
  return root->node;
}
