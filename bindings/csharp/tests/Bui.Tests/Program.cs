// Headless tests of the C# binding: the components layer and the Forms layer.
// SPDX-License-Identifier: GPL-2.0-or-later
using System;
using System.Collections.Generic;
using System.Drawing;
using System.Runtime.CompilerServices;
using Bui;
using Forms = Bui.Forms;

var failures = new List<string>();

void Test(string name, Action body)
{
    try
    {
        body();
        Console.WriteLine($"ok   {name}");
    }
#pragma warning disable CA1031 // a test runner reports every failure
    catch (Exception e)
#pragma warning restore CA1031
    {
        failures.Add(name);
        Console.WriteLine($"FAIL {name}: {e}");
    }
}

static void Check(bool ok, string what)
{
    if (!ok)
    {
        throw new InvalidOperationException(what);
    }
}

static void Throws<T>(Action body, string what) where T : Exception
{
    try
    {
        body();
    }
    catch (T)
    {
        return;
    }
    throw new InvalidOperationException($"no {typeof(T).Name}: {what}");
}

// The center of a node, after the context laid it out.
static (int X, int Y) Center(Node n)
{
    var (x, y, w, h) = n.Bounds;
    return (x + w / 2, y + h / 2);
}

static void Click(Context ctx, Node n)
{
    var (x, y) = Center(n);
    ctx.Motion(x, y);
    ctx.Press(x, y);
    ctx.Release(x, y);
}

Test("library and header versions match", () =>
{
    Check(Kit.LibraryVersion == Kit.HeaderVersion, $"library {Kit.LibraryVersion}, header {Kit.HeaderVersion}");
    Check(Kit.Version == "0.5.1", Kit.Version);
});

Test("unknown node type and property throw", () =>
{
    Throws<ArgumentException>(() => _ = new Node("no-such-type"), "node type");
    using var b = new Bui.Button("x");
    Throws<ArgumentException>(() => b.Set("no-such-key", 1), "property");
});

Test("properties round-trip, non-ASCII too", () =>
{
    using var b = new Bui.Button("Grüße — 你好");
    Check(b.Text == "Grüße — 你好", b.Text);
    Check(b.Type == "button", b.Type);
    string longText = new('x', 1000); // past the stack buffer
    b.Text = longText;
    Check(b.Text == longText, "long text");
});

Test("button click raises Click", () =>
{
    var root = new Frame();
    var button = new Bui.Button("Press");
    button.Geometry = GeometryManager.Place;
    button.SetGeometry("x", 10);
    button.SetGeometry("y", 10);
    button.SetGeometry("width", 100);
    button.SetGeometry("height", 24);
    root.Add(button);
    using var ctx = new Context(root);
    ctx.Configure(200, 100);
    int clicks = 0;
    button.Click += (_, _) => clicks++;
    Click(ctx, button);
    Check(clicks == 1, $"clicks {clicks}");
    Check(ReferenceEquals(button.Parent, root), "parent");
    root.Dispose();
});

Test("a parent keeps its children alive", () =>
{
    var root = new Frame();
    using var ctx = new Context(root);
    int clicks = 0;
    AddButton(root, () => clicks++); // no managed reference stays
    GC.Collect();
    GC.WaitForPendingFinalizers();
    GC.Collect();
    ctx.Configure(200, 100);
    var child = root.Child(0);
    Check(child is Bui.Button, "the wrapper survived");
    Click(ctx, child!);
    Check(clicks == 1, $"clicks {clicks}");
    root.Dispose();
});

Test("a handler's exception is thrown from the input call", () =>
{
    var root = new Frame();
    var button = new Bui.Button("Boom");
    button.Geometry = GeometryManager.Pack;
    root.Add(button);
    using var ctx = new Context(root);
    ctx.Configure(200, 100);
    button.Click += (_, _) => throw new InvalidTimeZoneException("from the handler");
    var (x, y) = Center(button);
    ctx.Motion(x, y);
    ctx.Press(x, y);
    Throws<InvalidTimeZoneException>(() => ctx.Release(x, y), "rethrow");
    root.Dispose();
});

Test("forms: Controls.Add, Location and Click", () =>
{
    using var form = new Forms.Form { Text = "Test" };
    var button = new Forms.Button { Text = "OK", Location = new Point(20, 30), Size = new Size(80, 24) };
    object? sender = null;
    button.Click += (s, _) => sender = s;
    form.Controls.Add(button);
    Check(form.Controls.Count == 1 && ReferenceEquals(button.Parent, form), "added");
    form.Context.Configure(300, 300);
    Click(form.Context, form.Context.Root.Child(0)!);
    Check(ReferenceEquals(sender, button), "sender is the Forms button");
});

Test("forms: Dock fills the form", () =>
{
    using var form = new Forms.Form();
    var button = new Forms.Button { Text = "Fill", Dock = Forms.DockStyle.Fill };
    form.Controls.Add(button);
    form.Context.Configure(300, 200);
    var (_, _, w, h) = form.Context.Root.Child(0)!.Bounds;
    Check(w == 300 && h == 200, $"{w} x {h}");
});

Test("forms: Dispose removes a control", () =>
{
    using var form = new Forms.Form();
    var button = new Forms.Button();
    form.Controls.Add(button);
    button.Dispose();
    Check(form.Controls.Count == 0 && form.Context.Root.ChildCount == 0, "removed");
    Throws<ArgumentException>(() => form.Controls.Add(form), "a form inside itself");
});

// The bounds of the form's i-th control, after the context laid it out.
static (int X, int Y, int W, int H) BoundsOf(Forms.Form form, int i = 0) => form.Context.Root.Child(i)!.Bounds;

// A form with one button, laid out in a 300 x 200 window; the button's bounds.
static (int X, int Y, int W, int H) Placed(Forms.Button button)
{
    using var form = new Forms.Form();
    form.Controls.Add(button);
    form.Context.Configure(300, 200);
    var bounds = BoundsOf(form);
    form.Controls.Remove(button); // the button outlives the form
    return bounds;
}

Test("forms: the defaults match Windows Forms", () =>
{
    using var form = new Forms.Form();
    using var button = new Forms.Button();
    Check(form.Size == new Size(300, 300) && form.Text == "", "form");
    Check(button.Size == new Size(75, 23) && button.Location == Point.Empty, "button");
    Check(button.Dock == Forms.DockStyle.None && button.Parent is null && button.Text == "", "button state");
});

Test("forms: Location and Size place a control", () =>
{
    using var button = new Forms.Button { Location = new Point(7, 9), Size = new Size(60, 20) };
    Check(Placed(button) == (7, 9, 60, 20), "placed");
    button.Location = new Point(40, 50); // moved after it was added
    button.Size = new Size(90, 30);
    Check(Placed(button) == (40, 50, 90, 30), "moved");
});

Test("forms: every Dock style packs against its side", () =>
{
    using var button = new Forms.Button { Size = new Size(80, 25) };
    foreach (var (dock, want) in new[]
    {
        (Forms.DockStyle.Top, (0, 0, 300, 25)),
        (Forms.DockStyle.Bottom, (0, 175, 300, 25)),
        (Forms.DockStyle.Left, (0, 0, 80, 200)),
        (Forms.DockStyle.Right, (220, 0, 80, 200)),
        (Forms.DockStyle.Fill, (0, 0, 300, 200)),
    })
    {
        button.Dock = dock;
        var got = Placed(button);
        Check(got == want, $"{dock}: {got}");
    }
    button.Dock = Forms.DockStyle.None; // back to Location and Size
    button.Location = new Point(3, 4);
    Check(Placed(button) == (3, 4, 80, 25), "undocked");
});

Test("forms: docked controls share the form in the order they were added", () =>
{
    using var form = new Forms.Form();
    using var top = new Forms.Button { Dock = Forms.DockStyle.Top, Size = new Size(75, 30) };
    using var left = new Forms.Button { Dock = Forms.DockStyle.Left, Size = new Size(50, 23) };
    using var fill = new Forms.Button { Dock = Forms.DockStyle.Fill };
    form.Controls.Add(top);
    form.Controls.Add(left);
    form.Controls.Add(fill);
    form.Context.Configure(300, 200);
    Check(BoundsOf(form, 0) == (0, 0, 300, 30), $"top {BoundsOf(form, 0)}");
    Check(BoundsOf(form, 1) == (0, 30, 50, 170), $"left {BoundsOf(form, 1)}");
    Check(BoundsOf(form, 2) == (50, 30, 250, 170), $"fill {BoundsOf(form, 2)}");
});

Test("forms: Controls keeps order, indexes and enumerates", () =>
{
    using var form = new Forms.Form();
    using var a = new Forms.Button { Text = "a" };
    using var b = new Forms.Button { Text = "b" };
    form.Controls.Add(a);
    form.Controls.Add(b);
    Check(ReferenceEquals(form.Controls[0], a) && ReferenceEquals(form.Controls[1], b), "index");
    var seen = new List<Forms.Control>(form.Controls);
    Check(seen.Count == 2 && ReferenceEquals(seen[1], b), "enumeration");
    Throws<ArgumentOutOfRangeException>(() => _ = form.Controls[2], "index past the end");
    int untyped = 0;
    foreach (object _ in (System.Collections.IEnumerable)form.Controls)
    {
        untyped++;
    }
    Check(untyped == 2, "untyped enumeration");
});

Test("forms: a control added to another container moves", () =>
{
    using var one = new Forms.Form();
    using var two = new Forms.Form();
    using var button = new Forms.Button();
    one.Controls.Add(button);
    two.Controls.Add(button);
    Check(one.Controls.Count == 0 && one.Context.Root.ChildCount == 0, "left the first");
    Check(two.Controls.Count == 1 && ReferenceEquals(button.Parent, two), "joined the second");
    Check(two.Context.Root.ChildCount == 1, "node moved");
});

Test("forms: adding a control twice keeps one copy", () =>
{
    using var form = new Forms.Form();
    using var button = new Forms.Button();
    form.Controls.Add(button);
    form.Controls.Add(button);
    Check(form.Controls.Count == 1 && form.Context.Root.ChildCount == 1, "one copy");
});

Test("forms: Remove keeps the control usable, and ignores a stranger", () =>
{
    using var form = new Forms.Form();
    using var button = new Forms.Button { Text = "kept" };
    using var stranger = new Forms.Button();
    form.Controls.Add(button);
    form.Controls.Remove(stranger);
    Check(form.Controls.Count == 1, "stranger ignored");
    form.Controls.Remove(button);
    Check(form.Controls.Count == 0 && button.Parent is null && form.Context.Root.ChildCount == 0, "removed");
    form.Controls.Add(button);
    Check(button.Text == "kept" && form.Controls.Count == 1, "added again");
});

Test("forms: null arguments throw", () =>
{
    using var form = new Forms.Form();
    Throws<ArgumentNullException>(() => form.Controls.Add(null!), "Add");
    Throws<ArgumentNullException>(() => form.Controls.Remove(null!), "Remove");
    Throws<ArgumentNullException>(() => Forms.Application.Run(null!), "Run");
});

Test("forms: Button text, PerformClick and a Click override", () =>
{
    using var button = new Forms.Button { Text = "Grüße" };
    Check(button.Text == "Grüße", button.Text);
    object? sender = null;
    button.Click += (s, _) => sender = s;
    button.PerformClick();
    Check(ReferenceEquals(sender, button), "PerformClick");
    using var custom = new Counting();
    custom.PerformClick();
    Check(custom.Clicks == 1, "OnClick override");
});

Test("forms: a click reaches every Click handler once", () =>
{
    using var form = new Forms.Form();
    var button = new Forms.Button { Location = new Point(5, 5), Size = new Size(100, 30) };
    int first = 0, second = 0;
    button.Click += (_, _) => first++;
    button.Click += (_, _) => second++;
    form.Controls.Add(button);
    form.Context.Configure(200, 100);
    Click(form.Context, form.Context.Root.Child(0)!);
    Check(first == 1 && second == 1, $"{first}, {second}");
});

Test("forms: a Form's Text and Close work without a window", () =>
{
    using var form = new Forms.Form { Text = "Before" };
    form.Text = "After";
    form.Close(); // nothing to close
    Check(form.Text == "After", form.Text);
});

Test("forms: Dispose is safe twice and frees the children's place", () =>
{
    var form = new Forms.Form();
    var button = new Forms.Button();
    form.Controls.Add(button);
    button.Dispose();
    button.Dispose();
    form.Dispose();
    form.Dispose();
});

Console.WriteLine(failures.Count == 0 ? "all passed" : $"{failures.Count} failed");
return failures.Count == 0 ? 0 : 1;

[MethodImpl(MethodImplOptions.NoInlining)]
static void AddButton(Node root, Action onClick)
{
    var b = new Bui.Button("Kept");
    b.Geometry = GeometryManager.Pack;
    b.Click += (_, _) => onClick();
    root.Add(b);
}

// A button that counts its clicks by overriding OnClick.
sealed class Counting : Forms.Button
{
    public int Clicks { get; private set; }

    protected override void OnClick(EventArgs e)
    {
        Clicks++;
        base.OnClick(e);
    }
}
