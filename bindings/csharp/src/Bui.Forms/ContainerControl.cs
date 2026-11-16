// Controls that hold other controls.
// SPDX-License-Identifier: GPL-2.0-or-later
using System;
using System.Collections;
using System.Collections.Generic;
using System.Drawing;
using Bui;

namespace Bui.Forms;

/// <summary>A control with child controls, over a BUI frame.</summary>
public abstract class ContainerControl : Control
{
    private protected ContainerControl(Frame frame, Size defaultSize)
        : base(frame, defaultSize) => Controls = new ControlCollection(this);

    /// <summary>The child controls, in the order they were added.</summary>
    public ControlCollection Controls { get; }
}

/// <summary>The children of a <see cref="ContainerControl"/>.</summary>
public sealed class ControlCollection : IReadOnlyList<Control>
{
    private readonly ContainerControl owner;
    private readonly List<Control> items = [];

    internal ControlCollection(ContainerControl owner) => this.owner = owner;

    public int Count => items.Count;

    public Control this[int index] => items[index];

    /// <summary>Adds <paramref name="control"/>; a control in another container moves here.</summary>
    public void Add(Control control)
    {
        ArgumentNullException.ThrowIfNull(control);
        if (ReferenceEquals(control, owner))
        {
            throw new ArgumentException("a control cannot contain itself", nameof(control));
        }
        control.Parent?.Controls.items.Remove(control);
        owner.Node.Add(control.Node);
        items.Add(control);
        control.Parent = owner;
    }

    /// <summary>Removes <paramref name="control"/>; it stays usable.</summary>
    public void Remove(Control control)
    {
        ArgumentNullException.ThrowIfNull(control);
        if (items.Remove(control))
        {
            owner.Node.Remove(control.Node);
            control.Parent = null;
        }
    }

    public IEnumerator<Control> GetEnumerator() => items.GetEnumerator();

    IEnumerator IEnumerable.GetEnumerator() => GetEnumerator();
}
