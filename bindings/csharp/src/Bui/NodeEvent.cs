// Events the kit delivers through BKHost.emit.
// SPDX-License-Identifier: GPL-2.0-or-later
using static Bui.Interop.Native;

namespace Bui;

/// <summary>Callback kinds (BK_EV_*).</summary>
public enum EventKind : uint
{
    Change = BK_EV_CHANGE,
    Click = BK_EV_CLICK,
    Call = BK_EV_CALL,
    Rename = BK_EV_RENAME,
    Activate = BK_EV_ACTIVATE,
    Toggle = BK_EV_TOGGLE,
    Browse = BK_EV_BROWSE,
    Press = BK_EV_PRESS,
    Release = BK_EV_RELEASE,
    DoubleClick = BK_EV_DOUBLE,
    Motion = BK_EV_MOTION,
    Enter = BK_EV_ENTER,
    Leave = BK_EV_LEAVE,
    Key = BK_EV_KEY,
    Wheel = BK_EV_WHEEL,
    Submit = BK_EV_SUBMIT,
    View = BK_EV_VIEW,
    Scroll = BK_EV_SCROLL,
    Configure = BK_EV_CONFIGURE,
    KeyRelease = BK_EV_KEYRELEASE,
}

/// <summary>One event of a node; <c>A</c> and <c>B</c> as BUI_kit.h documents per kind.</summary>
public readonly record struct NodeEvent(EventKind Kind, double A, double B);

/// <summary>Geometry managers of a frame's children (Tk's pack / grid / place).</summary>
public enum GeometryManager : uint
{
    None = BK_GEOM_NONE,
    Pack = BK_GEOM_PACK,
    Grid = BK_GEOM_GRID,
    Place = BK_GEOM_PLACE,
}
