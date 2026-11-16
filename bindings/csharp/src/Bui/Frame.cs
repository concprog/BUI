// A container whose children use a geometry manager (pack / grid / place).
// SPDX-License-Identifier: GPL-2.0-or-later
namespace Bui;

/// <summary>A "frame": children placed by <see cref="Node.Geometry"/>.</summary>
public class Frame : Node
{
    public Frame()
        : base("frame")
    {
    }
}
