// Blender's widgets from C#: panels, fields, sliders, toggles, menus and a
// progress bar, wired together through kit events.
// SPDX-License-Identifier: GPL-2.0-or-later
using System;
using Bui;

var root = new Node("column");
root.Set("padding", 12);
root.Set("spacing", 8);

var status = new Label("Change anything below");
root.Add(status);

// -- Object panel: number fields, a slider, a dropdown ----------------------------
var objectPanel = Panel(root, "Object");

var count = Field(objectPanel, "number", "Count");
count.Set("min", 0);
count.Set("max", 100);
count.Set("value", 12);

var opacity = Field(objectPanel, "slider", "Opacity");
opacity.Set("min", 0);
opacity.Set("max", 1);
opacity.Set("value", 0.4);

var shading = Field(objectPanel, "dropdown", "Shading");
shading.Set("items", "Solid", "Wireframe", "Material", "Rendered");

var name = Field(objectPanel, "text", "Name");
name.Set("placeholder", "Type a name, then Return");

// -- Options panel: checkboxes and toggles --------------------------------------
var optionsPanel = Panel(root, "Options");
var autosave = Field(optionsPanel, "checkbox", "Auto Save");
autosave.Set("text", "Auto Save");
var snap = Field(optionsPanel, "toggle", "Snap");
snap.Set("text", "Snap");

// -- Render panel: buttons drive a progress bar ---------------------------------
var renderPanel = Panel(root, "Render");
var progress = new Node("progress");
progress.Set("factor", 0.4);
renderPanel.Add(progress);

var buttons = new Node("row");
renderPanel.Add(buttons);
var render = new Button("Render");
var reset = new Button("Reset");
buttons.Add(render);
buttons.Add(reset);

// -- events -------------------------------------------------------------------------
opacity.Event += (_, e) =>
{
    if (e.Kind == EventKind.Change)
    {
        double v = opacity.GetNumber("value");
        progress.Set("factor", v);
        status.Text = $"Opacity {v:0.00}";
    }
};
count.Event += (_, e) =>
{
    if (e.Kind == EventKind.Change)
    {
        status.Text = $"Count {count.GetNumber("value"):0}";
    }
};
shading.Event += (_, e) =>
{
    if (e.Kind == EventKind.Change)
    {
        status.Text = $"Shading: {new[] { "Solid", "Wireframe", "Material", "Rendered" }[(int)shading.GetNumber("value")]}";
    }
};
name.Event += (_, e) =>
{
    if (e.Kind == EventKind.Submit)
    {
        status.Text = $"Hello, {name.GetString("value")}";
    }
};
autosave.Event += (_, e) =>
{
    if (e.Kind == EventKind.Change)
    {
        bool on = autosave.GetNumber("value") != 0;
        reset.Set("enabled", on ? 0 : 1);
        status.Text = on ? "Auto Save on: Reset is locked" : "Auto Save off";
    }
};
snap.Event += (_, e) =>
{
    if (e.Kind == EventKind.Change)
    {
        status.Text = snap.GetNumber("value") != 0 ? "Snapping on" : "Snapping off";
    }
};
render.Click += (_, _) =>
{
    double f = Math.Min(1, progress.GetNumber("factor") + 0.1);
    progress.Set("factor", f);
    opacity.Set("value", f);
    status.Text = f >= 1 ? "Render done" : $"Rendering {f:P0}";
};
reset.Click += (_, _) =>
{
    progress.Set("factor", 0);
    opacity.Set("value", 0);
    status.Text = "Reset";
};

using var context = new Context(root);
using var window = new Window(context, "BUI widgets from C#", 360, 460);
window.Show();
Window.Run();

// A collapsible panel in `parent`; returns its body.
static Node Panel(Node parent, string title)
{
    var panel = new Panel(title);
    parent.Add(panel);
    return panel.Body;
}

// A widget of `type` labelled `label`, added to `parent`.
static Node Field(Node parent, string type, string label)
{
    var field = new Node(type);
    field.Set("label", label);
    parent.Add(field);
    return field;
}
