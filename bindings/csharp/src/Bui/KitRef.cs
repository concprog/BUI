// Host objects the kit holds by reference (nodes, menu callbacks, tree items).
// SPDX-License-Identifier: GPL-2.0-or-later
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using static Bui.Interop.Native;

namespace Bui;

/// <summary>
/// An object the kit can reference. Its address for the kit is a weak GC handle;
/// while the kit retains it (BK_set_ref_hooks) a strong handle keeps it alive, so
/// a node in a tree lives as long as its parent holds it, as in the Python binding.
/// The kit is a UI-thread API, so the counts need no locks.
/// </summary>
public abstract unsafe class KitRef
{
    private readonly GCHandle self;
    private GCHandle keep;
    private int kitRefs;

    static KitRef() => BK_set_ref_hooks(&Retain, &Release);

    protected KitRef() => self = GCHandle.Alloc(this, GCHandleType.Weak);

    /// <summary>The value the kit stores for this object.</summary>
    internal void* Ref => (void*)GCHandle.ToIntPtr(self);

    /// <summary>Non-zero while the kit holds this object.</summary>
    internal bool HeldByKit => kitRefs > 0;

    /// <summary>The live object behind a kit reference, or null.</summary>
    internal static T? From<T>(void* r) where T : KitRef =>
        r is null ? null : GCHandle.FromIntPtr((nint)r).Target as T;

    /// <summary>Releases the weak handle: the kit no longer stores <see cref="Ref"/>.</summary>
    private protected void FreeRef()
    {
        if (self.IsAllocated)
        {
            self.Free();
        }
    }

    /// <summary>The weak handle of a collected object, freed after its native part.</summary>
    private protected nint RefHandle => GCHandle.ToIntPtr(self);

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static void Retain(void* r)
    {
        KitRef? k = From<KitRef>(r);
        if (k is not null && k.kitRefs++ == 0)
        {
            k.keep = GCHandle.Alloc(k);
        }
    }

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static void Release(void* r)
    {
        KitRef? k = From<KitRef>(r);
        if (k is not null && --k.kitRefs == 0)
        {
            k.keep.Free();
        }
    }
}
