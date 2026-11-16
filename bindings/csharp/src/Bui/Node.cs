// A node of the kit's tree: the base of every component.
// SPDX-License-Identifier: GPL-2.0-or-later
using System;
using System.Collections.Concurrent;
using System.Runtime.InteropServices;
using Bui.Interop;
using static Bui.Interop.Native;

namespace Bui;

/// <summary>
/// A kit node (BKNode). It owns its native node: a parent only holds it
/// (BK_node_add retains it, the parent's free releases it). The native node is
/// freed by <see cref="Dispose()"/>, or on the UI thread after the node is
/// collected while no parent holds it.
/// </summary>
public unsafe class Node : KitRef, IDisposable
{
    // native nodes of collected wrappers, freed on the UI thread
    private static readonly ConcurrentQueue<(nint Node, nint Ref)> Collected = new();

    private BKNode* handle;

    // false for a borrowed wrapper of a node the kit built (a panel body): the kit frees it
    private readonly bool owned = true;

    /// <summary>A node of a kit type ("button", "frame", "label", ...).</summary>
    /// <exception cref="ArgumentException">The kit has no such type.</exception>
    public Node(string type)
    {
        ArgumentNullException.ThrowIfNull(type);
        using var t = new Utf8Z(type, stackalloc byte[Utf8Z.StackSize]);
        fixed (byte* p = t)
        {
            handle = BK_node_new((sbyte*)p);
        }
        if (handle is null)
        {
            FreeRef();
            throw new ArgumentException($"no node type '{type}'", nameof(type));
        }
        BK_node_set_ref(handle, Ref);
    }

    // A borrowed wrapper: the kit retains it until it frees the node.
    private Node(BKNode* kitOwned)
    {
        handle = kitOwned;
        owned = false;
        BK_node_set_borrowed_ref(handle, Ref);
    }

    ~Node()
    {
        if (owned && handle is not null)
        {
            Collected.Enqueue(((nint)handle, RefHandle));
        }
    }

    /// <summary>Raised for every event the kit delivers to this node.</summary>
    public event EventHandler<NodeEvent>? Event;

    /// <summary>The kit type name.</summary>
    public string Type => Utf8Z.Read(BK_node_type(Handle));

    /// <summary>The parent node, or null.</summary>
    public Node? Parent
    {
        get
        {
            BKNode* p = BK_node_parent(Handle);
            return p is null ? null : Wrap(p);
        }
    }

    /// <summary>Number of child nodes, the kit's own parts included.</summary>
    public int ChildCount => BK_node_child_count(Handle);

    /// <summary>Child <paramref name="index"/>. A part the kit built itself (a panel's body)
    /// gets a borrowed wrapper, kept for the node's lifetime.</summary>
    public Node Child(int index)
    {
        ArgumentOutOfRangeException.ThrowIfNegative(index);
        ArgumentOutOfRangeException.ThrowIfGreaterThanOrEqual(index, ChildCount);
        return Wrap(BK_node_child(Handle, index));
    }

    /// <summary>The native node; throws after <see cref="Dispose()"/>.</summary>
    internal BKNode* Handle => handle is not null ? handle : throw new ObjectDisposedException(GetType().Name);

    /// <summary>Position and size in window pixels after the last layout.</summary>
    public (int X, int Y, int Width, int Height) Bounds
    {
        get
        {
            int* r = stackalloc int[4];
            BK_node_rect(Handle, r);
            return (r[0], r[1], r[2], r[3]);
        }
    }

    /// <summary>Appends <paramref name="child"/>, or inserts it at <paramref name="index"/>. A child with another parent moves.</summary>
    public void Add(Node child, int index = -1)
    {
        ArgumentNullException.ThrowIfNull(child);
        if (BK_node_add(Handle, child.Handle, index) == 0)
        {
            throw new ArgumentException("a node cannot be its own child", nameof(child));
        }
    }

    /// <summary>Detaches <paramref name="child"/>; it stays usable.</summary>
    public void Remove(Node child)
    {
        ArgumentNullException.ThrowIfNull(child);
        BK_node_remove(Handle, child.Handle);
    }

    /// <summary>Sets a number property. Throws for a key this node does not have.</summary>
    public void Set(string key, double value)
    {
        using var k = new Utf8Z(key, stackalloc byte[Utf8Z.StackSize]);
        fixed (byte* p = k)
        {
            Check(BK_set_num(Handle, (sbyte*)p, value), key);
        }
    }

    /// <summary>Sets a string property. Throws for a key this node does not have.</summary>
    public void Set(string key, string value)
    {
        ArgumentNullException.ThrowIfNull(value);
        using var k = new Utf8Z(key, stackalloc byte[Utf8Z.StackSize]);
        using var v = new Utf8Z(value, stackalloc byte[Utf8Z.StackSize]);
        fixed (byte* pk = k, pv = v)
        {
            Check(BK_set_str(Handle, (sbyte*)pk, (sbyte*)pv), key);
        }
    }

    /// <summary>Sets a string-list property ("items", "headers", ...). Throws for a key this node does not have.</summary>
    public void Set(string key, params string[] values)
    {
        ArgumentNullException.ThrowIfNull(values);
        // all strings NUL-terminated in one pooled buffer, pointers into it
        int size = 0;
        foreach (string v in values)
        {
            size += System.Text.Encoding.UTF8.GetMaxByteCount(v.Length) + 1;
        }
        byte[] buffer = System.Buffers.ArrayPool<byte>.Shared.Rent(Math.Max(size, 1));
        nint[]? rentedPtrs = values.Length > 64 ? System.Buffers.ArrayPool<nint>.Shared.Rent(values.Length) : null;
        Span<nint> offsets = rentedPtrs is null ? stackalloc nint[values.Length] : rentedPtrs;
        try
        {
            int at = 0;
            for (int i = 0; i < values.Length; i++)
            {
                offsets[i] = at;
                at += System.Text.Encoding.UTF8.GetBytes(values[i], buffer.AsSpan(at));
                buffer[at++] = 0;
            }
            using var k = new Utf8Z(key, stackalloc byte[Utf8Z.StackSize]);
            fixed (byte* pk = k, pb = buffer)
            {
                for (int i = 0; i < values.Length; i++)
                {
                    offsets[i] += (nint)pb;
                }
                fixed (nint* pv = offsets)
                {
                    Check(BK_set_strs(Handle, (sbyte*)pk, (sbyte**)pv, values.Length), key);
                }
            }
        }
        finally
        {
            System.Buffers.ArrayPool<byte>.Shared.Return(buffer);
            if (rentedPtrs is not null)
            {
                System.Buffers.ArrayPool<nint>.Shared.Return(rentedPtrs);
            }
        }
    }

    /// <summary>A number property (0 for an unknown key).</summary>
    public double GetNumber(string key)
    {
        using var k = new Utf8Z(key, stackalloc byte[Utf8Z.StackSize]);
        fixed (byte* p = k)
        {
            return BK_get_num(Handle, (sbyte*)p);
        }
    }

    /// <summary>A string property ("" for an unknown key).</summary>
    public string GetString(string key)
    {
        using var k = new Utf8Z(key, stackalloc byte[Utf8Z.StackSize]);
        fixed (byte* p = k)
        {
            return Utf8Z.Read(BK_get_str(Handle, (sbyte*)p));
        }
    }

    /// <summary>How the parent frame places this node.</summary>
    public GeometryManager Geometry
    {
        get => (GeometryManager)BK_geom_manager(Handle);
        set => BK_geom_set_manager(Handle, (int)value);
    }

    /// <summary>Sets a key of the geometry manager ("side", "fill", "x", "width", ...).</summary>
    public void SetGeometry(string key, double value)
    {
        using var k = new Utf8Z(key, stackalloc byte[Utf8Z.StackSize]);
        fixed (byte* p = k)
        {
            BK_geom_set(Handle, (sbyte*)p, value);
        }
    }

    /// <summary>Redraw after a change the kit cannot see.</summary>
    public void Invalidate() => BK_node_invalidate(Handle);

    /// <summary>Frees the native node; a parent lets go of it first.</summary>
    public void Dispose()
    {
        Dispose(true);
        GC.SuppressFinalize(this);
    }

    protected virtual void Dispose(bool disposing)
    {
        if (handle is null || !owned)
        {
            return;
        }
        BKNode* parent = BK_node_parent(handle);
        if (parent is not null)
        {
            BK_node_remove(parent, handle);
        }
        BK_node_free(handle);
        handle = null;
        FreeRef();
    }

    /// <summary>Delivers a kit event; components override it to raise typed events.</summary>
    protected virtual void OnEvent(NodeEvent e) => Event?.Invoke(this, e);

    internal void Dispatch(int kind, double a, double b) => OnEvent(new NodeEvent((EventKind)kind, a, b));

    /// <summary>Frees the native nodes of collected wrappers (UI thread). They have no
    /// parent: a parent's retain keeps its children's wrappers alive.</summary>
    internal static void FreeCollected()
    {
        while (Collected.TryDequeue(out var c))
        {
            BK_node_free((BKNode*)c.Node);
            GCHandle.FromIntPtr(c.Ref).Free();
        }
    }

    /// <summary>The wrapper of a native node: its own, its borrowed one, or a new borrowed one.</summary>
    private static Node Wrap(BKNode* n) =>
        From<Node>(BK_node_ref(n)) ?? From<Node>(BK_node_borrowed_ref(n)) ?? new Node(n);

    private static void Check(int ok, string key)
    {
        if (ok == 0)
        {
            throw new ArgumentException($"unknown property '{key}'", nameof(key));
        }
    }
}
