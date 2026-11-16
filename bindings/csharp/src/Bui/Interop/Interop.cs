// Support for the generated Native.g.cs.
// SPDX-License-Identifier: GPL-2.0-or-later
using System;
using System.Diagnostics;
using System.Runtime.CompilerServices;

// Every kit signature is blittable: no marshalling stubs at all.
[assembly: DisableRuntimeMarshalling]

namespace Bui.Interop;

/// <summary>The C type of a generated declaration, as written in BUI_kit.h.</summary>
[AttributeUsage(AttributeTargets.All, AllowMultiple = false, Inherited = true)]
[Conditional("DEBUG")]
internal sealed class NativeTypeNameAttribute(string name) : Attribute
{
    public string Name { get; } = name;
}
