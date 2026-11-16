// Static text.
// SPDX-License-Identifier: GPL-2.0-or-later
namespace Bui;

/// <summary>A "label".</summary>
public class Label : Node
{
    public Label(string text = "")
        : base("label") => Text = text;

    /// <summary>The text.</summary>
    public string Text
    {
        get => GetString("text");
        set => Set("text", value);
    }
}
