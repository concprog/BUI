// The kit's native window host (Win32, X11 / Wayland, Cocoa).
// SPDX-License-Identifier: GPL-2.0-or-later
using System;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using Bui.Interop;
using static Bui.Interop.Native;

namespace Bui;

/// <summary>A native window that drives a <see cref="Context"/>: input, render and present stay in C.</summary>
public sealed unsafe class Window : IDisposable
{
    private readonly Context context;
    private BKWindow* handle;

    /// <summary>A window showing <paramref name="context"/>, in logical pixels.</summary>
    /// <exception cref="PlatformNotSupportedException">No window host here (no display, or not the main thread on macOS).</exception>
    public Window(Context context, string title, int width, int height)
    {
        ArgumentNullException.ThrowIfNull(context);
        if (BK_window_supported() == 0)
        {
            throw new PlatformNotSupportedException("the kit has no window host here");
        }
        this.context = context;
        using var t = new Utf8Z(title, stackalloc byte[Utf8Z.StackSize]);
        fixed (byte* p = t)
        {
            handle = BK_window_new(context.Handle, (sbyte*)p, width, height);
        }
        if (handle is null)
        {
            throw new InvalidOperationException("the kit could not open a window");
        }
    }

    /// <summary>The context this window drives.</summary>
    public Context Context => context;

    /// <summary>The window title.</summary>
    public string Title
    {
        set
        {
            using var t = new Utf8Z(value, stackalloc byte[Utf8Z.StackSize]);
            fixed (byte* p = t)
            {
                BK_window_set_title(Handle, (sbyte*)p);
            }
        }
    }

    /// <summary>The client size in logical pixels.</summary>
    public (int Width, int Height) Size
    {
        get
        {
            int w, h;
            BK_window_get_size(Handle, &w, &h);
            return (w, h);
        }
        set => BK_window_set_size(Handle, value.Width, value.Height);
    }

    /// <summary>True until the window is closed.</summary>
    public bool IsOpen => handle is not null && BK_window_is_open(handle) != 0;

    private BKWindow* Handle => handle is not null ? handle : throw new ObjectDisposedException(nameof(Window));

    public void Show() => BK_window_show(Handle);

    public void Close() => BK_window_close(Handle);

    public void Dispose()
    {
        if (handle is null)
        {
            return;
        }
        BK_window_free(handle);
        handle = null;
    }

    /// <summary>Runs the message loop until every window is closed. An exception
    /// from a handler stops the loop and is thrown here.</summary>
    public static void Run()
    {
        BK_window_set_stop_check(&StopCheck);
        BK_window_run();
        Context.ThrowPending();
    }

    /// <summary>Stops <see cref="Run"/>.</summary>
    public static void Quit() => BK_window_quit();

    // polled after each message: free collected nodes, stop on a handler's exception
    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static int StopCheck()
    {
        if (Context.HasPending)
        {
            return 1;
        }
        Node.FreeCollected();
        return 0;
    }
}
