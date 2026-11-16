#!/usr/bin/env bash
# Generate src/Bui.Interop/Native.g.cs from kit/include/BUI_kit.h.
# Needs ClangSharpPInvokeGenerator on PATH (or in $CLANGSHARP) and clang for
# its builtin headers (stddef.h, stdint.h).
#
# SPDX-License-Identifier: GPL-2.0-or-later
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")"
gen="${CLANGSHARP:-ClangSharpPInvokeGenerator}"
"$gen" @generate.rsp --include-directory "$(clang -print-resource-dir)/include"
