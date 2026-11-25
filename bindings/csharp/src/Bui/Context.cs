// One input and render context per window (BKContext).
// SPDX-License-Identifier: GPL-2.0-or-later
using System;
using System.Collections.Generic;
using System.Runtime.CompilerServices;
using System.Runtime.ExceptionServices;
using System.Runtime.InteropServices;
using Bui.Interop;
using static Bui.Interop.Native;

namespace Bui;

/// <summary>
/// The kit's context for a root node: input in, events out. Handlers run inside
/// kit calls; an exception from one is held and thrown when that call returns
/// (or when <see cref="Window.Run"/> stops), never across native frames.
/// </summary>
public sealed unsafe class Context : IDisposable
{
    private static ExceptionDispatchInfo? pending;

    private static int active; // native callbacks on the stack (UI thread)
    private static readonly Queue<Action> Deferred = new(); // frees waiting for the callbacks to return

    private readonly Node root;
    private BKContext* handle;
    private bool attached; // a window uses the handle
    private bool disposed;

    /// <summary>A context showing <paramref name="root"/>; it keeps the root alive.</summary>
    public Context(Node root)
    {
        ArgumentNullException.ThrowIfNull(root);
        this.root = root;
        handle = BK_context_new(root.Handle);
        if (handle is null)
        {
            throw new InvalidOperationException("the kit could not create a context");
        }
        BKHost host = default;
        host.emit = &Emit;
        BK_context_set_host(handle, &host);
    }

    /// <summary>The root node.</summary>
    public Node Root => root;

    internal BKContext* Handle => !disposed && handle is not null ? handle : throw new ObjectDisposedException(nameof(Context));

    /// <summary>True while a native callback is on the stack: the kit objects it uses must stay allocated.</summary>
    internal static bool InCallback => active > 0;

    internal static void EnterCallback() => active++;

    internal static void LeaveCallback() => active--;

    /// <summary>True while a window uses this context: one window per context.</summary>
    internal bool HasWindow => attached;

    internal void Attach() => attached = true;

    /// <summary>The window stops using this context: a context disposed earlier is freed now.</summary>
    internal void Detach()
    {
        attached = false;
        if (disposed)
        {
            Free();
        }
    }

    /// <summary>Runs <paramref name="release"/> now, or after the outermost native callback has returned.</summary>
    internal static void ReleaseWhenSafe(Action release)
    {
        if (InCallback)
        {
            Deferred.Enqueue(release);
        }
        else
        {
            release();
        }
    }

    /// <summary>Runs the releases that waited for a native callback, unless one is still on the stack.</summary>
    internal static void RunDeferred()
    {
        while (!InCallback && Deferred.TryDequeue(out var release))
        {
            release();
        }
    }

    /// <summary>Lays the tree out at <paramref name="width"/> x <paramref name="height"/>; true when the size changed.</summary>
    public bool Configure(int width, int height) => Done(BK_configure(Handle, width, height));

    /// <summary>Renders into <paramref name="raster"/>: only damaged regions unless <paramref name="full"/>.
    /// Returns the number of regions redrawn (0: nothing changed).</summary>
    public int Render(Raster raster, bool full = false)
    {
        ArgumentNullException.ThrowIfNull(raster);
        int n = BK_render(Handle, raster.Handle, raster.Width, raster.Height, full ? 1 : 0);
        ThrowPending();
        return n;
    }

    /// <summary>Pointer press at window pixels; true when something changed.</summary>
    public bool Press(int x, int y, int mods = 0) => Done(BK_press(Handle, x, y, mods));

    /// <summary>Pointer release at window pixels; true when something changed.</summary>
    public bool Release(int x, int y, int mods = 0) => Done(BK_release(Handle, x, y, mods));

    /// <summary>Pointer motion at window pixels; true when something changed.</summary>
    public bool Motion(int x, int y, int mods = 0) => Done(BK_motion(Handle, x, y, mods));

    /// <summary>Disposes the context. While its <see cref="Window"/> still uses it, or from inside
    /// a handler, the native context is freed later, once that is safe.</summary>
    public void Dispose()
    {
        if (disposed)
        {
            return;
        }
        disposed = true;
        if (!attached)
        {
            ReleaseWhenSafe(Free);
        }
    }

    private void Free()
    {
        BK_context_free(handle);
        handle = null;
    }

    /// <summary>Throws the exception a handler raised during the last kit call, if any.</summary>
    internal static void ThrowPending()
    {
        RunDeferred();
        Node.FreeCollected();
        ExceptionDispatchInfo? e = pending;
        pending = null;
        e?.Throw();
    }

    internal static bool HasPending => pending is not null;

    /// <summary>Keeps the first exception of a handler until the kit call returns.</summary>
    internal static void Hold(Exception e) => pending ??= ExceptionDispatchInfo.Capture(e);

    private static bool Done(int changed)
    {
        ThrowPending();
        return changed != 0;
    }

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static void Emit(void* data, BKNode* node, int kind, void* r, double a, double b)
    {
        EnterCallback();
        try
        {
            if (node is null)
            {
                return;
            }
            void* wrapper = BK_node_ref(node);
            Node? n = KitRef.From<Node>(wrapper is not null ? wrapper : BK_node_borrowed_ref(node));
            n?.Dispatch(kind, a, b);
        }
#pragma warning disable CA1031 // any handler exception is held and rethrown on the managed side
        catch (Exception e)
#pragma warning restore CA1031
        {
            Hold(e);
        }
        finally
        {
            LeaveCallback();
        }
    }
}
