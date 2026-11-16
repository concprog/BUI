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
