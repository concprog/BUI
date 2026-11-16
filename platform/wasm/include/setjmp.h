/* <setjmp.h> for BUI's WebAssembly build (platform/wasm/wasi.cmake puts this folder before the sysroot).
 *
 * wasi-libc's own header refuses to compile without WebAssembly exception handling, which BUI does not
 * use. FreeType includes it to recover from a malformed font in the middle of a parse (`ft_setjmp`); the
 * fonts a BUI page uses are its own, so there recovery is not needed. `setjmp` returns 0 (the normal
 * path) and `longjmp` traps: a font that would have been rejected stops the program instead.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later */
#ifndef BUI_WASM_SETJMP_H
#define BUI_WASM_SETJMP_H

#include <stdlib.h>

typedef int jmp_buf[1];

#define setjmp(env) ((void)(env), 0)

static inline void longjmp(jmp_buf env, int value)
{
  (void)env;
  (void)value;
  abort();
}

#endif
