// Blender's push button.
// SPDX-License-Identifier: GPL-2.0-or-later
using System;

namespace Bui;

/// <summary>A "button": raises <see cref="Click"/> when pressed.</summary>
public class Button : Node
{
    public Button(string text = "")
        : base("button") => Text = text;

    /// <summary>Raised when the button is pressed (BK_EV_CLICK).</summary>
    public event EventHandler? Click;

    /// <summary>The label.</summary>
    public string Text
    {
        get => GetString("text");
        set => Set("text", value);
    }

    protected override void OnEvent(NodeEvent e)
    {
        base.OnEvent(e);
        if (e.Kind == EventKind.Click)
        {
            Click?.Invoke(this, EventArgs.Empty);
        }
    }
}
