// The kit's native window host (Win32, X11 / Wayland, Cocoa).
// SPDX-License-Identifier: GPL-2.0-or-later
using System;
using System.Collections.Generic;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using Bui.Interop;
using static Bui.Interop.Native;

namespace Bui;

/// <summary>A native window that drives a <see cref="Context"/>: input, render and present stay in C.</summary>
public sealed unsafe class Window : IDisposable
{
    private readonly Context context;
    private static readonly Queue<Window> Disposed = new(); // freed at the next safe point (UI thread)

    private BKWindow* handle;
    private GCHandle self; // BKWindowCallbacks.data, while the window lives
    private bool disposed;

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
        self = GCHandle.Alloc(this);
        BKWindowCallbacks cb = default;
        cb.data = (void*)GCHandle.ToIntPtr(self);
        cb.on_frame = &OnFrame;
        BK_window_set_callbacks(handle, &cb);
        context.Attach();
    }

    /// <summary>Raised right before a frame asked for with <see cref="RequestFrame"/> renders:
    /// the place to update the UI for exactly that frame.</summary>
    public event EventHandler? Frame;

    /// <summary>Asks for a <see cref="Frame"/>; with <paramref name="vsync"/>, frames are paced to the display.</summary>
    public void RequestFrame(bool vsync = true) => BK_window_request_frame(Handle, vsync ? 1 : 0);

    /// <summary>Frames presented so far.</summary>
    public int Frames => BK_window_frames(Handle);

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
    public bool IsOpen => !disposed && handle is not null && BK_window_is_open(handle) != 0;

    private BKWindow* Handle => !disposed && handle is not null ? handle : throw new ObjectDisposedException(nameof(Window));

    public void Show() => BK_window_show(Handle);

    public void Close() => BK_window_close(Handle);

    /// <summary>Disposes the window. From inside a handler the native window is still
    /// on the stack, so it is freed once the native call has returned.</summary>
    public void Dispose()
    {
        if (disposed)
        {
            return;
        }
        disposed = true;
        if (Context.InCallback)
        {
            Disposed.Enqueue(this);
            return;
        }
        Release();
    }

    /// <summary>Frees the windows disposed inside a callback, unless one is still on the stack.</summary>
    internal static void FreeDisposed()
    {
        while (!Context.InCallback && Disposed.TryDequeue(out var w))
        {
            w.Release();
        }
    }

    private void Release()
    {
        BK_window_free(handle);
        handle = null;
        self.Free();
        context.Detach();
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

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static void OnFrame(void* data, BKWindow* win)
    {
        Context.EnterCallback();
        try
        {
            var w = (Window?)GCHandle.FromIntPtr((nint)data).Target;
            if (w is { disposed: false })
            {
                w.Frame?.Invoke(w, EventArgs.Empty);
            }
        }
#pragma warning disable CA1031 // held and rethrown from Run, never across native frames
        catch (Exception e)
#pragma warning restore CA1031
        {
            Context.Hold(e);
        }
        finally
        {
            Context.LeaveCallback();
        }
    }

    // polled after each message: free collected nodes, stop on a handler's exception
    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static int StopCheck()
    {
        if (Context.HasPending)
        {
            return 1;
        }
        Window.FreeDisposed();
        Node.FreeCollected();
        return 0;
    }
}
