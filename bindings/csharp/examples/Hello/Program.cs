// A Windows Forms-style window drawn with Blender's widgets.
// SPDX-License-Identifier: GPL-2.0-or-later
using System;
using System.Drawing;
using Bui.Forms;

using var form = new Form { Text = "Hello from C#", Size = new Size(320, 120) };
var button = new Button { Text = "Click me", Location = new Point(110, 40), Size = new Size(100, 24) };
int clicks = 0;
button.Click += (_, _) => button.Text = $"Clicked {++clicks}x";
form.Controls.Add(button);

if (Environment.GetEnvironmentVariable("BUI_EXAMPLE_SMOKE") is not null)
{
    // CI: open, then close right away
    form.Show();
    form.Close();
}
Application.Run(form);
