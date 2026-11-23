// A window drawn with Blender's widgets.
// SPDX-License-Identifier: GPL-2.0-or-later
using Bui;

var root = new Frame();
var button = new Button("Click me") { Geometry = GeometryManager.Pack };
int clicks = 0;
button.Click += (_, _) => button.Text = $"Clicked {++clicks}x";
root.Add(button);

using var context = new Context(root);
using var window = new Window(context, "Hello from C#", 320, 120);
window.Show();
Window.Run();
