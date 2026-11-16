// Blender's collapsible panel.
// SPDX-License-Identifier: GPL-2.0-or-later
namespace Bui;

/// <summary>A "panel": a header with a title, and a <see cref="Body"/> for the content.
/// Nodes added to the panel itself go in its header.</summary>
public class Panel : Node
{
    public Panel(string title)
        : base("panel")
    {
        Set("text", title);
        Body = Child(0);
    }

    /// <summary>Where the content goes (the kit's own body node).</summary>
    public Node Body { get; }

    /// <summary>Folded to its header.</summary>
    public bool Collapsed
    {
        get => GetNumber("collapsed") != 0;
        set => Set("collapsed", value ? 1 : 0);
    }
}
