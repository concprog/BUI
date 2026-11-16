// The cells benchmark from C#, headless: a 1000x800 grid of N rounded,
// numbered cells (the grid, colours and labels of bindings/node/bench/cells.js
// and bindings/rust/bench/common/measure.rs) drawn as kit canvas items.
//   dotnet run -c Release -- [cells=3000] [one|all] [--frames=300]
// one: cell 0 changes colour and label each frame; all: every cell does.
// Prints the C# cost per frame (building and handing the changed items to the
// kit), the kit's damage-only render, and managed bytes allocated per frame.
// SPDX-License-Identifier: GPL-2.0-or-later
using System;
using System.Diagnostics;
using System.Globalization;
using Bui;
using Bui.Interop;
using static Bui.Interop.Native;

int cells = 3000;
bool all = false;
int frames = 300;
foreach (string a in args)
{
    if (int.TryParse(a, NumberStyles.None, CultureInfo.InvariantCulture, out int n))
    {
        cells = n;
    }
    else if (a == "all")
    {
        all = true;
    }
    else if (a.StartsWith("--frames=", StringComparison.Ordinal))
    {
        frames = int.Parse(a.AsSpan(9), CultureInfo.InvariantCulture);
    }
}

const int W = 1000, H = 800, Gap = 2, Warmup = 30;
int cols = Math.Max(1, (int)Math.Ceiling(Math.Sqrt(cells * (double)W / H)));
int rows = (cells + cols - 1) / cols;
float cw = MathF.Floor((W - Gap) / (float)cols - Gap);
float ch = MathF.Floor((H - Gap) / (float)rows - Gap);
float font = MathF.Floor(Math.Min(14, Math.Max(8, ((H - Gap) / (float)rows - Gap) * 0.55f)));

var bench = new CellGrid(cols, cw, ch, font, Gap);
var canvas = new Canvas();
using var context = new Context(canvas);
using var raster = new Raster(W, H);

for (int i = 0; i < cells; i++)
{
    bench.Put(canvas, i, 0);
}
context.Render(raster, full: true);
// the grid is really there: every item, and cell 0's colour at its corner
if (canvas.Count != 2 * cells)
{
    throw new InvalidOperationException($"{canvas.Count} items, not {2 * cells}");
}
uint px = Pixel(raster, Gap + 1, Gap + (int)ch / 2);
Console.Error.WriteLine($"items {canvas.Count}, cell 0 pixel #{px & 0xFFFFFF:x6}, background #{Pixel(raster, W - 1, H - 1) & 0xFFFFFF:x6}");

long tCs = 0, tRender = 0, bytes = 0;
for (int f = 1; f <= Warmup + frames; f++)
{
    bool measured = f > Warmup;
    long b0 = GC.GetAllocatedBytesForCurrentThread();
    long t0 = Stopwatch.GetTimestamp();
    if (all)
    {
        for (int i = 0; i < cells; i++)
        {
            bench.Put(canvas, i, f);
        }
    }
    else
    {
        bench.Put(canvas, 0, f);
    }
    long t1 = Stopwatch.GetTimestamp();
    context.Render(raster);
    long t2 = Stopwatch.GetTimestamp();
    if (measured)
    {
        tCs += t1 - t0;
        tRender += t2 - t1;
        bytes += GC.GetAllocatedBytesForCurrentThread() - b0;
    }
}

double ms(long ticks) => ticks * 1000.0 / Stopwatch.Frequency / frames;
Console.WriteLine(string.Create(CultureInfo.InvariantCulture,
    $"{{\"app\":\"blenderui-csharp\",\"runtime\":\".NET {Environment.Version}\",\"cells\":{cells},\"mode\":\"{(all ? "all" : "one")}\"," +
    $"\"csharp_ms_per_frame\":{ms(tCs):0.000},\"render_ms_per_frame\":{ms(tRender):0.000}," +
    $"\"total_ms_per_frame\":{ms(tCs + tRender):0.000},\"alloc_bytes_per_frame\":{bytes / frames}}}"));
canvas.Dispose();

static uint Pixel(Raster r, int x, int y) => r.Pixels[y * r.Width + x];

/// <summary>The grid's items: cell i is a rounded rect (id 2i+1) and its label (id 2i+2).</summary>
internal sealed unsafe class CellGrid
{
    private readonly int cols;
    private readonly float cw, ch, font, gap;
    private readonly float[] coords;    // pinned: x0 y0 x1 y1 for the rect, cx cy for the text
    private readonly byte[] labels;     // pinned: "0\0" .. "999\0", 4 bytes each
    private BKCanvasItem rect, text;

    public CellGrid(int cols, float cw, float ch, float font, float gap)
    {
        this.cols = cols;
        this.cw = cw;
        this.ch = ch;
        this.font = font;
        this.gap = gap;
        coords = GC.AllocateArray<float>(6, pinned: true);
        labels = GC.AllocateArray<byte>(1000 * 4, pinned: true);
        for (int i = 0; i < 1000; i++)
        {
            i.TryFormat(labels.AsSpan(i * 4, 3), out _, default, CultureInfo.InvariantCulture);
        }
        rect.type = (int)BK_ITEM_ROUNDRECT;
        rect.ncoords = 4;
        rect.radius = 3;
        text.type = (int)BK_ITEM_TEXT;
        text.ncoords = 2;
        text.size = font;
        text.fill[0] = text.fill[1] = text.fill[2] = text.fill[3] = 1; // #fff
    }

    public void Put(Canvas canvas, int i, int t)
    {
        float x = gap + (i % cols) * (cw + gap);
        float y = gap + (i / cols) * (ch + gap);
        fixed (float* c = coords)
        fixed (byte* l = labels)
        {
            c[0] = x;
            c[1] = y;
            c[2] = x + cw;
            c[3] = y + ch;
            c[4] = x + cw / 2;
            c[5] = y + ch / 2;
            rect.id = 2 * i + 1;
            rect.coords = c;
            Color(i, t, ref rect.fill);
            text.id = 2 * i + 2;
            text.coords = c + 4;
            text.text = (sbyte*)(l + ((i * 13 + t) % 1000) * 4);
            canvas.SetItem(in rect);
            canvas.SetItem(in text);
        }
    }

    // HSV (h from i and t, s 0.55, v 0.75) as in cells.js
    private static void Color(int i, int t, ref BKCanvasItem._fill_e__FixedBuffer rgba)
    {
        float h = ((i * 7 + t * 3) % 360) / 60f, s = 0.55f, v = 0.75f, c = v * s;
        float x = c * (1 - MathF.Abs(h % 2 - 1)), m = v - c;
        (float r, float g, float b) = ((int)h % 6) switch
        {
            0 => (c, x, 0f),
            1 => (x, c, 0f),
            2 => (0f, c, x),
            3 => (0f, x, c),
            4 => (x, 0f, c),
            _ => (c, 0f, x),
        };
        rgba[0] = r + m;
        rgba[1] = g + m;
        rgba[2] = b + m;
        rgba[3] = 1;
    }
}
