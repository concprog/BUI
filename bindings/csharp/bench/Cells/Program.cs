// The cells benchmark of bindings/rust/bench (bui-bench), from C#: a 1000x800
// window filled with N rounded, numbered cells, measured the same way
// (common/measure.rs): a 3 s warm-up, then a 10 s window of process CPU time
// (every thread), frames and resident memory, printed as one JSON line.
//   dotnet run -c Release -- --cells 3000 --mode anim [--warmup 3] [--seconds 10]
// anim: every frame changes every cell's colour and number (paced to vsync).
// one: every frame, only cell 0 changes.
// The cells are kit canvas items (a rounded rect and its label each); a frame
// replaces the items that changed, and the kit redraws only those.
// SPDX-License-Identifier: GPL-2.0-or-later
using System;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.Runtime.InteropServices;
using System.Threading;
using Bui;
using Bui.Interop;
using static Bui.Interop.Native;

var launched = Stopwatch.StartNew();
string Arg(string key, string fallback)
{
    int i = Array.IndexOf(args, key);
    return i >= 0 && i + 1 < args.Length ? args[i + 1] : fallback;
}
int cells = int.Parse(Arg("--cells", "100"), CultureInfo.InvariantCulture);
string mode = Arg("--mode", "anim");
double warmup = double.Parse(Arg("--warmup", "3"), CultureInfo.InvariantCulture);
double seconds = double.Parse(Arg("--seconds", "10"), CultureInfo.InvariantCulture);
bool one = mode == "one";
if (mode is not ("anim" or "one"))
{
    throw new ArgumentException($"--mode {mode}: anim or one");
}

const int W = 1000, H = 800;
const float Gap = 2;
// measure.rs Config::grid
int cols = Math.Max(1, (int)MathF.Ceiling(MathF.Sqrt(cells * (float)W / H)));
int rows = (cells + cols - 1) / cols;
float cw = MathF.Floor((W - Gap) / cols - Gap);
float hh = (H - Gap) / rows - Gap;
float ch = MathF.Floor(hh);
float font = MathF.Floor(Math.Clamp(hh * 0.55f, 8, 14));

var grid = new CellGrid(cols, cw, ch, font, Gap);
var canvas = new Canvas();
for (int i = 0; i < cells; i++)
{
    grid.Put(canvas, i, 0);
}
using var context = new Context(canvas);
using var window = new Window(context, "BUI bench (C#)", W, H);

long frames = 0;
long t = 0;
window.Frame += (_, _) =>
{
    t++;
    if (one)
    {
        grid.Put(canvas, 0, t);
    }
    else
    {
        for (int i = 0; i < cells; i++)
        {
            grid.Put(canvas, i, t);
        }
    }
    Interlocked.Increment(ref frames);
    window.RequestFrame();
};

// measure.rs start(): first frame, warm-up, then the measured window
new Thread(() =>
{
    double first = 0;
    while (Interlocked.Read(ref frames) == 0 && launched.Elapsed.TotalSeconds < 30)
    {
        Thread.Sleep(1);
    }
    first = launched.Elapsed.TotalMilliseconds;
    Thread.Sleep(TimeSpan.FromSeconds(warmup));
    var w0 = Stopwatch.StartNew();
    long c0 = Probe.CpuNanoseconds(), f0 = Interlocked.Read(ref frames);
    long peak = 0;
    double sum = 0;
    int samples = 0;
    while (w0.Elapsed.TotalSeconds < seconds)
    {
        Thread.Sleep(100);
        long ws = Probe.Status("VmRSS:");
        peak = Math.Max(peak, ws);
        sum += ws;
        samples++;
    }
    double wall = w0.Elapsed.TotalSeconds;
    long n = Interlocked.Read(ref frames) - f0;
    double cpu = (Probe.CpuNanoseconds() - c0) / 1e9;
    static double Mb(double b) => b / (1024.0 * 1024.0);
    Console.WriteLine(string.Create(CultureInfo.InvariantCulture,
        $"{{\"app\":\"blenderui-csharp\",\"cells\":{cells},\"mode\":\"{mode}\",\"fps\":{n / wall:0.0}," +
        $"\"cpu_pct\":{cpu / wall * 100:0.0},\"cpu_ms_per_frame\":{(n > 0 ? cpu * 1000 / n : 0):0.000}," +
        $"\"ws_mb\":{Mb(sum / Math.Max(1, samples)):0.0},\"peak_ws_mb\":{Mb(peak):0.0}," +
        $"\"private_mb\":{Mb(Probe.Status("RssAnon:")):0.0},\"first_frame_ms\":{first:0}}}"));
    Environment.Exit(0);
}) { IsBackground = true }.Start();

window.Show();
window.RequestFrame();
Window.Run();

/// <summary>The grid's items: cell i is a rounded rect (id 2i+1) and its label (id 2i+2).</summary>
internal sealed unsafe class CellGrid
{
    private readonly int cols;
    private readonly float cw, ch, gap;
    private readonly float[] coords;    // pinned: x0 y0 x1 y1 for the rect, cx cy for the text
    private readonly byte[] labels;     // pinned: "0\0" .. "999\0", 4 bytes each
    private BKCanvasItem rect, text;

    public CellGrid(int cols, float cw, float ch, float font, float gap)
    {
        this.cols = cols;
        this.cw = cw;
        this.ch = ch;
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

    public void Put(Canvas canvas, int i, long t)
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
            text.text = (sbyte*)(l + (int)((i * 13 + t) % 1000) * 4);
            canvas.SetItem(in rect);
            canvas.SetItem(in text);
        }
    }

    // measure.rs cell_color: HSV with h from i and t, s 0.55, v 0.75, rounded to bytes
    private static void Color(int i, long t, ref BKCanvasItem._fill_e__FixedBuffer rgba)
    {
        float h = ((i * 7 + t * 3) % 360) / 60f, s = 0.55f, v = 0.75f, c = v * s;
        float x = c * (1 - MathF.Abs(h % 2 - 1)), m = v - c;
        (float r, float g, float b) = ((int)h) switch
        {
            0 => (c, x, 0f),
            1 => (x, c, 0f),
            2 => (0f, c, x),
            3 => (0f, x, c),
            4 => (x, 0f, c),
            _ => (c, 0f, x),
        };
        rgba[0] = MathF.Round((r + m) * 255) / 255;
        rgba[1] = MathF.Round((g + m) * 255) / 255;
        rgba[2] = MathF.Round((b + m) * 255) / 255;
        rgba[3] = 1;
    }
}

/// <summary>measure.rs on Linux: CLOCK_PROCESS_CPUTIME_ID (every thread, exact) and /proc/self/status.</summary>
internal static unsafe partial class Probe
{
    private const int ClockProcessCputimeId = 2;

    [LibraryImport("libc", EntryPoint = "clock_gettime")]
    private static partial int ClockGetTime(int clock, long* ts);

    public static long CpuNanoseconds()
    {
        long* ts = stackalloc long[2];
        _ = ClockGetTime(ClockProcessCputimeId, ts);
        return ts[0] * 1_000_000_000 + ts[1];
    }

    /// <summary>A "Key: N kB" line of /proc/self/status, in bytes.</summary>
    public static long Status(string key)
    {
        foreach (string line in File.ReadLines("/proc/self/status"))
        {
            if (line.StartsWith(key, StringComparison.Ordinal))
            {
                return long.Parse(line.AsSpan(key.Length).Trim().TrimEnd("kB").Trim(), CultureInfo.InvariantCulture) * 1024;
            }
        }
        return 0;
    }
}
