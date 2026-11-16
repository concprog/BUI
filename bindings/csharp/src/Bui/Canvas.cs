// Tk Canvas / QPainter-style retained items (BKCanvasItem).
// SPDX-License-Identifier: GPL-2.0-or-later
using Bui.Interop;
using static Bui.Interop.Native;

namespace Bui;

/// <summary>A "canvas": items (rects, ovals, lines, text, images ...) kept by id;
/// only items that change are redrawn.</summary>
public sealed unsafe class Canvas : Node
{
    public Canvas()
        : base("canvas")
    {
    }

    /// <summary>Inserts the item, or replaces the one with its id. The kit copies it.</summary>
    public void SetItem(in BKCanvasItem item)
    {
        fixed (BKCanvasItem* p = &item)
        {
            BK_canvas_set_item(Handle, p);
        }
    }

    /// <summary>Removes item <paramref name="id"/> (all items for id &lt; 0).</summary>
    public void Delete(int id) => BK_canvas_delete(Handle, id);

    /// <summary>The number of items.</summary>
    public int Count => BK_canvas_count(Handle);
}
