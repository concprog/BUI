// The base of every control: a BUI node placed like a Windows Forms control.
// SPDX-License-Identifier: GPL-2.0-or-later
using System;
using System.Drawing;
using Bui;

namespace Bui.Forms;

/// <summary>Where a control docks in its container (System.Windows.Forms.DockStyle).</summary>
public enum DockStyle
{
    None,
    Top,
    Bottom,
    Left,
    Right,
    Fill,
}

/// <summary>
/// A control over one BUI node. <see cref="Location"/> and <see cref="Size"/>
/// place it absolutely (the kit's place manager); <see cref="Dock"/> other than
/// None packs it against a side of its container instead.
/// </summary>
public abstract class Control : IDisposable
{
    private Point location;
    private Size size;
    private DockStyle dock;

    private protected Control(Node node, Size defaultSize)
    {
        Node = node;
        size = defaultSize;
        ApplyLayout();
    }

    /// <summary>The container holding this control, or null.</summary>
    public ContainerControl? Parent { get; internal set; }

    /// <summary>Top-left corner in the container, in pixels.</summary>
    public Point Location
    {
        get => location;
        set
        {
            location = value;
            ApplyLayout();
        }
    }

    /// <summary>Size in pixels.</summary>
    public Size Size
    {
        get => size;
        set
        {
            size = value;
            ApplyLayout();
        }
    }

    /// <summary>Docking in the container; None uses <see cref="Location"/>.</summary>
    public DockStyle Dock
    {
        get => dock;
        set
        {
            dock = value;
            ApplyLayout();
        }
    }

    /// <summary>The BUI node behind this control.</summary>
    internal Node Node { get; }

    public void Dispose()
    {
        Dispose(true);
        GC.SuppressFinalize(this);
    }

    protected virtual void Dispose(bool disposing)
    {
        if (disposing)
        {
            Parent?.Controls.Remove(this);
            Node.Dispose();
        }
    }

    // Tk's pack keys: side 0 top, 1 bottom, 2 left, 3 right; fill 1 x, 2 y, 3 both
    private void ApplyLayout()
    {
        if (dock == DockStyle.None)
        {
            Node.Geometry = GeometryManager.Place;
            Node.SetGeometry("x", location.X);
            Node.SetGeometry("y", location.Y);
            Node.SetGeometry("width", size.Width);
            Node.SetGeometry("height", size.Height);
            return;
        }
        Node.Geometry = GeometryManager.Pack;
        (int side, int fill) = dock switch
        {
            DockStyle.Top => (0, 1),
            DockStyle.Bottom => (1, 1),
            DockStyle.Left => (2, 2),
            DockStyle.Right => (3, 2),
            _ => (0, 3),
        };
        Node.SetGeometry("side", side);
        Node.SetGeometry("fill", fill);
        Node.SetGeometry("expand", dock == DockStyle.Fill ? 1 : 0);
        Node.SetGeometry("reqwidth", size.Width);
        Node.SetGeometry("reqheight", size.Height);
    }
}
