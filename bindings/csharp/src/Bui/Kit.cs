// Library-wide settings and the version.
// SPDX-License-Identifier: GPL-2.0-or-later
using static Bui.Interop.Native;

namespace Bui;

/// <summary>The kit library: its version and global look.</summary>
public static unsafe class Kit
{
    /// <summary>The version of the loaded library ("MAJOR.MINOR.PATCH").</summary>
    public static string Version => Utf8Z.Read(BK_version_string());

    /// <summary>The version this binding was generated from (BUI_KIT_VERSION).</summary>
    public const int HeaderVersion = BUI_KIT_VERSION;

    /// <summary>The loaded library's version as one number (0.5.1 is 501).</summary>
    public static int LibraryVersion => BK_version();

    /// <summary>Blender's resolution scale (1: 20 px widgets).</summary>
    public static float Scale
    {
        get => BK_scale();
        set => BK_set_scale(value);
    }
}
