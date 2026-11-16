// NUL-terminated UTF-8 for the kit's const char * arguments.
// SPDX-License-Identifier: GPL-2.0-or-later
using System;
using System.Buffers;
using System.Runtime.InteropServices;
using System.Text;

namespace Bui;

/// <summary>
/// A string encoded as NUL-terminated UTF-8 in the caller's stack buffer, or in a
/// pooled array when it does not fit. Use with <c>stackalloc byte[Utf8Z.StackSize]</c>.
/// </summary>
internal unsafe ref struct Utf8Z
{
    public const int StackSize = 256;

    private readonly Span<byte> bytes;
    private byte[]? rented;

    public Utf8Z(ReadOnlySpan<char> text, Span<byte> scratch)
    {
        int max = Encoding.UTF8.GetMaxByteCount(text.Length) + 1;
        if (max > scratch.Length)
        {
            rented = ArrayPool<byte>.Shared.Rent(max);
            scratch = rented;
        }
        int n = Encoding.UTF8.GetBytes(text, scratch);
        scratch[n] = 0;
        bytes = scratch[..(n + 1)];
    }

    /// <summary>The bytes, NUL included; valid while this value lives.</summary>
    public readonly ref readonly byte GetPinnableReference() => ref bytes.GetPinnableReference();

    public void Dispose()
    {
        if (rented is not null)
        {
            ArrayPool<byte>.Shared.Return(rented);
            rented = null;
        }
    }

    /// <summary>A kit string (NULL: empty) as a managed string.</summary>
    public static string Read(sbyte* s) =>
        s is null ? string.Empty : Encoding.UTF8.GetString(MemoryMarshal.CreateReadOnlySpanFromNullTerminated((byte*)s));
}
