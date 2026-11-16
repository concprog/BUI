// System.Windows.Forms.Button over BUI's button.
// SPDX-License-Identifier: GPL-2.0-or-later
using System;
using System.Drawing;

namespace Bui.Forms;

/// <summary>A push button; <see cref="Click"/> is raised with this control as the sender.</summary>
public class Button : Control
{
    private readonly Bui.Button button;

    public Button()
        : this(new Bui.Button())
    {
    }

    private Button(Bui.Button button)
        : base(button, new Size(75, 23)) // the Windows Forms default
    {
        this.button = button;
        button.Click += (_, e) => OnClick(e);
    }

    public event EventHandler? Click;

    /// <summary>The label.</summary>
    public string Text
    {
        get => button.Text;
        set => button.Text = value;
    }

    /// <summary>Raises <see cref="Click"/>, as a press would.</summary>
    public void PerformClick() => OnClick(EventArgs.Empty);

    protected virtual void OnClick(EventArgs e) => Click?.Invoke(this, e);
}
