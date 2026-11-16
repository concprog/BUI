// A CPU pixel buffer the kit renders into, without a window.
// SPDX-License-Identifier: GPL-2.0-or-later
using System;
using Bui.Interop;
using static Bui.Interop.Native;

namespace Bui;

/// <summary>A BuiRaster: <see cref="Context.Render"/> draws into it headless (tests, benchmarks, other hosts).</summary>
public sealed unsafe class Raster : IDisposable
{
    private BuiRaster* handle;

    public Raster(int width, int height)
    {
        ArgumentOutOfRangeException.ThrowIfNegativeOrZero(width);
        ArgumentOutOfRangeException.ThrowIfNegativeOrZero(height);
        handle = BUI_raster_create(width, height);
        if (handle is null)
        {
            throw new InvalidOperationException("the kit could not create a raster");
        }
        Width = width;
        Height = height;
    }

    public int Width { get; }

    public int Height { get; }

    /// <summary>The pixels, row by row, premultiplied 0xAARRGGBB; valid until the next render or dispose.</summary>
    public ReadOnlySpan<uint> Pixels => new(BUI_raster_pixels(Handle), Width * Height);

    internal BuiRaster* Handle => handle is not null ? handle : throw new ObjectDisposedException(nameof(Raster));

    public void Dispose()
    {
        if (handle is null)
        {
            return;
        }
        BUI_raster_free(handle);
        handle = null;
    }
}
