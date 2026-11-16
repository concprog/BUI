using System;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Bui.Interop
{
    public partial struct BKNode
    {
    }

    public partial struct BKContext
    {
    }

    public partial struct BKTreeItem
    {
    }

    public unsafe partial struct BKHost
    {
        public void* data;

        [NativeTypeName("void (*)(void *)")]
        public delegate* unmanaged[Cdecl]<void*, void> request;

        [NativeTypeName("void (*)(void *, int)")]
        public delegate* unmanaged[Cdecl]<void*, int, void> set_cursor;

        [NativeTypeName("void (*)(void *, int, int)")]
        public delegate* unmanaged[Cdecl]<void*, int, int, void> warp;

        [NativeTypeName("char *(*)(void *)")]
        public delegate* unmanaged[Cdecl]<void*, sbyte*> clipboard_get;

        [NativeTypeName("void (*)(void *, const char *)")]
        public delegate* unmanaged[Cdecl]<void*, sbyte*, void> clipboard_set;

        [NativeTypeName("void (*)(void *, int, int)")]
        public delegate* unmanaged[Cdecl]<void*, int, int, void> schedule;

        [NativeTypeName("void (*)(void *, BKNode *, int, void *, double, double)")]
        public delegate* unmanaged[Cdecl]<void*, BKNode*, int, void*, double, double, void> emit;
    }

    public unsafe partial struct BKMenuSpec
    {
        [NativeTypeName("const char *")]
        public sbyte* text;

        public int icon;

        [NativeTypeName("const char *")]
        public sbyte* shortcut;

        public int @checked;

        public int enabled;

        public void* @ref;

        [NativeTypeName("const struct BKMenuSpec *")]
        public BKMenuSpec* sub;

        public int nsub;

        [NativeTypeName("void (*)(void *)")]
        public delegate* unmanaged[Cdecl]<void*, void> fn;

        public void* data;
    }

    public partial struct BKImBuf
    {
    }

    public partial struct BKImageInfo
    {
        public int format;

        public int width;

        public int height;

        public int frames;
    }

    public partial struct BKAnim
    {
    }

    public partial struct BKVideo
    {
    }

    public partial struct BKImageStats
    {
        [NativeTypeName("size_t")]
        public nuint bytes;

        [NativeTypeName("size_t")]
        public nuint budget;

        public int entries;

        public int pending;

        public int decoded;

        public int failed;

        public int cancelled;

        public int evicted;

        public double decode_ms;
    }

    public unsafe partial struct BKCanvasItem
    {
        public int id;

        public int type;

        [NativeTypeName("const float *")]
        public float* coords;

        public int ncoords;

        [NativeTypeName("float[4]")]
        public _fill_e__FixedBuffer fill;

        [NativeTypeName("float[4]")]
        public _outline_e__FixedBuffer outline;

        public float width;

        public float radius;

        public float size;

        public float start;

        public float extent;

        [NativeTypeName("const char *")]
        public sbyte* text;

        public int anchor;

        [NativeTypeName("const unsigned char *")]
        public byte* rgba;

        public int iw;

        public int ih;

        public int hidden;

        public int text_flags;

        public int clip_on;

        [NativeTypeName("float[4]")]
        public _clip_e__FixedBuffer clip;

        [NativeTypeName("const char *")]
        public sbyte* family;

        [NativeTypeName("const BKImBuf *")]
        public BKImBuf* imbuf;

        [NativeTypeName("const char *")]
        public sbyte* src;

        public int fit;

        public int frame;

        public BKVideo* video;

        public int no_hit;

        public int effect;

        [NativeTypeName("int[8]")]
        public _effect_inputs_e__FixedBuffer effect_inputs;

        [NativeTypeName("const float *")]
        public float* effect_floats;

        public int n_effect_floats;

        [InlineArray(4)]
        public partial struct _fill_e__FixedBuffer
        {
            public float e0;
        }

        [InlineArray(4)]
        public partial struct _outline_e__FixedBuffer
        {
            public float e0;
        }

        [InlineArray(4)]
        public partial struct _clip_e__FixedBuffer
        {
            public float e0;
        }

        [InlineArray(8)]
        public partial struct _effect_inputs_e__FixedBuffer
        {
            public int e0;
        }
    }

    public unsafe partial struct BKCustomEvent
    {
        public int type;

        public float x;

        public float y;

        public int button;

        public int mods;

        public int clicks;

        public float wheel_x;

        public float wheel_y;

        [NativeTypeName("const char *")]
        public sbyte* text;

        [NativeTypeName("const char *")]
        public sbyte* keysym;
    }

    public unsafe partial struct BKCustomClass
    {
        [NativeTypeName("int (*)(void *, BKNode *, const BKCustomEvent *)")]
        public delegate* unmanaged[Cdecl]<void*, BKNode*, BKCustomEvent*, int> @event;

        [NativeTypeName("void (*)(void *, BKNode *)")]
        public delegate* unmanaged[Cdecl]<void*, BKNode*, void> frame;

        [NativeTypeName("void (*)(void *, BKNode *, BuiRaster *, const int *)")]
        public delegate* unmanaged[Cdecl]<void*, BKNode*, void*, int, void> draw;

        [NativeTypeName("void (*)(void *, BKNode *)")]
        public delegate* unmanaged[Cdecl]<void*, BKNode*, void> free;

        public int natural_w;

        public int natural_h;
    }

    public partial struct BKTextSpan
    {
        public int start;

        public int end;

        [NativeTypeName("float[4]")]
        public _fg_e__FixedBuffer fg;

        [NativeTypeName("float[4]")]
        public _bg_e__FixedBuffer bg;

        public int has_fg;

        public int has_bg;

        [InlineArray(4)]
        public partial struct _fg_e__FixedBuffer
        {
            public float e0;
        }

        [InlineArray(4)]
        public partial struct _bg_e__FixedBuffer
        {
            public float e0;
        }
    }

    public unsafe partial struct BKImeEvent
    {
        [NativeTypeName("const char *")]
        public sbyte* result;

        [NativeTypeName("const char *")]
        public sbyte* composite;

        public int cursor_pos;

        public int sel_start;

        public int sel_end;
    }

    public partial struct BKImeState
    {
        public int enabled;

        public int restart;

        public int x;

        public int y;
    }

    public partial struct BKWindow
    {
    }

    public unsafe partial struct BKWindowCallbacks
    {
        public void* data;

        [NativeTypeName("void (*)(void *, BKWindow *, int)")]
        public delegate* unmanaged[Cdecl]<void*, BKWindow*, int, void> on_timer;

        [NativeTypeName("int (*)(void *, BKWindow *)")]
        public delegate* unmanaged[Cdecl]<void*, BKWindow*, int> on_close;

        [NativeTypeName("void (*)(void *, BKWindow *)")]
        public delegate* unmanaged[Cdecl]<void*, BKWindow*, void> on_frame;

        [NativeTypeName("void (*)(void *, BKWindow *, int)")]
        public delegate* unmanaged[Cdecl]<void*, BKWindow*, int, void> on_focus;
    }

    public unsafe partial struct BKWidgetDraw
    {
        public int kind;

        public int state;

        public int corners;

        public int align;

        [NativeTypeName("float[4]")]
        public _rect_e__FixedBuffer rect;

        [NativeTypeName("float[4]")]
        public _clip_e__FixedBuffer clip;

        [NativeTypeName("float[4]")]
        public _text_rect_e__FixedBuffer text_rect;

        public float value;

        [NativeTypeName("float[4]")]
        public _color_e__FixedBuffer color;

        [NativeTypeName("float[4]")]
        public _aux_e__FixedBuffer aux;

        [NativeTypeName("const char *")]
        public sbyte* text;

        [NativeTypeName("const char *")]
        public sbyte* text2;

        [InlineArray(4)]
        public partial struct _rect_e__FixedBuffer
        {
            public float e0;
        }

        [InlineArray(4)]
        public partial struct _clip_e__FixedBuffer
        {
            public float e0;
        }

        [InlineArray(4)]
        public partial struct _text_rect_e__FixedBuffer
        {
            public float e0;
        }

        [InlineArray(4)]
        public partial struct _color_e__FixedBuffer
        {
            public float e0;
        }

        [InlineArray(4)]
        public partial struct _aux_e__FixedBuffer
        {
            public float e0;
        }
    }

    public static unsafe partial class Native
    {
        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_version();

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        [return: NativeTypeName("const char *")]
        public static extern sbyte* BK_version_string();

        public const uint BK_SHIFT = 1;
        public const uint BK_CTRL = 2;
        public const uint BK_ALT = 4;
        public const uint BK_BUTTON1 = 8;
        public const uint BK_BUTTON2 = 16;
        public const uint BK_BUTTON3 = 32;

        public const uint BK_CURSOR_DEFAULT = 0;
        public const uint BK_CURSOR_EW = 1;
        public const uint BK_CURSOR_NS = 2;
        public const uint BK_CURSOR_TEXT = 3;
        public const uint BK_CURSOR_CROSSHAIR = 4;
        public const uint BK_CURSOR_HAND = 5;
        public const uint BK_CURSOR_MOVE = 6;
        public const uint BK_CURSOR_NONE = 7;
        public const uint BK_CURSOR_NESW = 8;
        public const uint BK_CURSOR_NWSE = 9;
        public const uint BK_CURSOR_NOT_ALLOWED = 10;
        public const uint BK_CURSOR_WAIT = 11;
        public const uint BK_CURSOR_PROGRESS = 12;

        public const uint BK_EV_CHANGE = 1;
        public const uint BK_EV_CLICK = 2;
        public const uint BK_EV_CALL = 3;
        public const uint BK_EV_RENAME = 4;
        public const uint BK_EV_ACTIVATE = 5;
        public const uint BK_EV_TOGGLE = 6;
        public const uint BK_EV_BROWSE = 7;
        public const uint BK_EV_PRESS = 8;
        public const uint BK_EV_RELEASE = 9;
        public const uint BK_EV_DOUBLE = 10;
        public const uint BK_EV_MOTION = 11;
        public const uint BK_EV_ENTER = 12;
        public const uint BK_EV_LEAVE = 13;
        public const uint BK_EV_KEY = 14;
        public const uint BK_EV_WHEEL = 15;
        public const uint BK_EV_SUBMIT = 16;
        public const uint BK_EV_VIEW = 17;
        public const uint BK_EV_SCROLL = 18;
        public const uint BK_EV_CONFIGURE = 19;
        public const uint BK_EV_KEYRELEASE = 20;

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_set_ref_hooks([NativeTypeName("void (*)(void *)")] delegate* unmanaged[Cdecl]<void*, void> retain, [NativeTypeName("void (*)(void *)")] delegate* unmanaged[Cdecl]<void*, void> release);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_set_scale(float scale);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern float BK_scale();

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_set_text_shadow(int on);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_text_shadow();

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_unit();

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_set_font([NativeTypeName("const char *")] sbyte* family);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_set_font_size(float points);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern float BK_font_size_points();

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_set_theme([NativeTypeName("const char *")] sbyte* name);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        [return: NativeTypeName("const char *")]
        public static extern sbyte* BK_theme_name();

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_theme_color([NativeTypeName("const char *")] sbyte* key, [NativeTypeName("float[4]")] float* r_rgba);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_theme_load_json([NativeTypeName("const char *")] sbyte* json);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_theme_load_file([NativeTypeName("const char *")] sbyte* path);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        [return: NativeTypeName("const char *")]
        public static extern sbyte* BK_theme_error();

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_theme_ignored();

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_theme_count();

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        [return: NativeTypeName("const char *")]
        public static extern sbyte* BK_theme_at(int index);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_theme_set_color([NativeTypeName("const char *")] sbyte* key, [NativeTypeName("const float[4]")] float* rgba);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_theme_set_num([NativeTypeName("const char *")] sbyte* key, double value);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_theme_num([NativeTypeName("const char *")] sbyte* key, double* r_value);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_theme_generation();

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        [return: NativeTypeName("const char *")]
        public static extern sbyte* BK_theme_mode();

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_icon_id([NativeTypeName("const char *")] sbyte* name);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_icon_count();

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        [return: NativeTypeName("const char *")]
        public static extern sbyte* BK_icon_name(int id);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_icon_list_count();

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        [return: NativeTypeName("const char *")]
        public static extern sbyte* BK_icon_list_name(int i);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern BKNode* BK_node_new([NativeTypeName("const char *")] sbyte* type);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_node_free(BKNode* node);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        [return: NativeTypeName("const char *")]
        public static extern sbyte* BK_node_type([NativeTypeName("const BKNode *")] BKNode* node);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_node_set_ref(BKNode* node, void* @ref);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void* BK_node_ref([NativeTypeName("const BKNode *")] BKNode* node);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_node_set_borrowed_ref(BKNode* node, void* @ref);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void* BK_node_borrowed_ref([NativeTypeName("const BKNode *")] BKNode* node);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_node_add(BKNode* parent, BKNode* child, int index);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_node_remove(BKNode* parent, BKNode* child);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_node_child_count([NativeTypeName("const BKNode *")] BKNode* node);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern BKNode* BK_node_child([NativeTypeName("const BKNode *")] BKNode* node, int i);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern BKNode* BK_node_parent([NativeTypeName("const BKNode *")] BKNode* node);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_node_rect([NativeTypeName("const BKNode *")] BKNode* node, [NativeTypeName("int[4]")] int* r_rect);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_node_natural(BKNode* node, [NativeTypeName("int[2]")] int* r_size);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_node_invalidate(BKNode* node);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_node_relayout(BKNode* node);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_set_num(BKNode* node, [NativeTypeName("const char *")] sbyte* key, double value);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_set_str(BKNode* node, [NativeTypeName("const char *")] sbyte* key, [NativeTypeName("const char *")] sbyte* value);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_set_color(BKNode* node, [NativeTypeName("const char *")] sbyte* key, [NativeTypeName("const float[4]")] float* rgba);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_set_strs(BKNode* node, [NativeTypeName("const char *")] sbyte* key, [NativeTypeName("const char *const *")] sbyte** values, int n);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern double BK_get_num(BKNode* node, [NativeTypeName("const char *")] sbyte* key);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        [return: NativeTypeName("const char *")]
        public static extern sbyte* BK_get_str(BKNode* node, [NativeTypeName("const char *")] sbyte* key);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_get_color(BKNode* node, [NativeTypeName("const char *")] sbyte* key, [NativeTypeName("float[4]")] float* r_rgba);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_strs_count(BKNode* node, [NativeTypeName("const char *")] sbyte* key);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        [return: NativeTypeName("const char *")]
        public static extern sbyte* BK_strs_get(BKNode* node, [NativeTypeName("const char *")] sbyte* key, int i);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_link_fields([NativeTypeName("BKNode *const *")] BKNode** fields, int n);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_ramp_count(BKNode* ramp);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_ramp_get(BKNode* ramp, int i, float* r_pos, [NativeTypeName("float[4]")] float* r_rgba);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_ramp_set(BKNode* ramp, [NativeTypeName("const float *")] float* pos, [NativeTypeName("const float (*)[4]")] float** rgba, int n);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_ramp_active(BKNode* ramp);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_ramp_evaluate(BKNode* ramp, float t, [NativeTypeName("float[4]")] float* r_rgba);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_curve_count(BKNode* curve, int channel);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_curve_get(BKNode* curve, int channel, int i, [NativeTypeName("float[2]")] float* r_xy);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_curve_set(BKNode* curve, int channel, [NativeTypeName("const float (*)[2]")] float** xy, int n);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern float BK_curve_evaluate(BKNode* curve, int channel, float x);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_ramp_set_active_color(BKNode* ramp, [NativeTypeName("const float[4]")] float* rgba);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_histogram_set(BKNode* hist, [NativeTypeName("const float *")] float* bins, int n, int channels);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_gridview_set_colors(BKNode* grid, [NativeTypeName("const float (*)[4]")] float** colors, int n);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern BKTreeItem* BK_tree_item_new([NativeTypeName("const char *")] sbyte* label, int icon);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_tree_item_free(BKTreeItem* item);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_tree_item_set_ref(BKTreeItem* item, void* @ref);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void* BK_tree_item_ref([NativeTypeName("const BKTreeItem *")] BKTreeItem* item);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern BKTreeItem* BK_tree_item_parent([NativeTypeName("const BKTreeItem *")] BKTreeItem* item);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_tree_item_add(BKTreeItem* parent, BKTreeItem* child);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_tree_item_clear(BKTreeItem* parent);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_tree_item_set(BKTreeItem* item, [NativeTypeName("const char *")] sbyte* key, double value);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern double BK_tree_item_get([NativeTypeName("const BKTreeItem *")] BKTreeItem* item, [NativeTypeName("const char *")] sbyte* key);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_tree_item_set_label(BKTreeItem* item, [NativeTypeName("const char *")] sbyte* label);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        [return: NativeTypeName("const char *")]
        public static extern sbyte* BK_tree_item_label([NativeTypeName("const BKTreeItem *")] BKTreeItem* item);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_tree_item_child_count([NativeTypeName("const BKTreeItem *")] BKTreeItem* item);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern BKTreeItem* BK_tree_item_child([NativeTypeName("const BKTreeItem *")] BKTreeItem* item, int i);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_tree_set_roots(BKNode* tree, [NativeTypeName("BKTreeItem *const *")] BKTreeItem** roots, int n);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern BKTreeItem* BK_tree_selected(BKNode* tree);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_tree_select(BKNode* tree, BKTreeItem* item);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_tree_row_count(BKNode* tree);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern BKTreeItem* BK_tree_row(BKNode* tree, int row);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_tree_row_rect(BKNode* tree, int row, [NativeTypeName("int[4]")] int* r_rect);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_menubutton_set_items(BKNode* button, [NativeTypeName("const BKMenuSpec *")] BKMenuSpec* items, int n);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_open_menu(BKContext* ctx, BKNode* owner, [NativeTypeName("const BKMenuSpec *")] BKMenuSpec* items, int n, [NativeTypeName("const char *")] sbyte* title, int at_x, int at_y);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern BKNode* BK_hit_at(BKContext* ctx, int x, int y);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_open_pie(BKContext* ctx, [NativeTypeName("const BKMenuSpec *")] BKMenuSpec* items, int n, [NativeTypeName("const char *")] sbyte* title, int at_x, int at_y);

        public const uint BK_GEOM_NONE = 0;
        public const uint BK_GEOM_PACK = 1;
        public const uint BK_GEOM_GRID = 2;
        public const uint BK_GEOM_PLACE = 3;

        public const uint BK_ANCHOR_N = 1;
        public const uint BK_ANCHOR_S = 2;
        public const uint BK_ANCHOR_W = 4;
        public const uint BK_ANCHOR_E = 8;

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_geom_set_manager(BKNode* child, int manager);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_geom_manager([NativeTypeName("const BKNode *")] BKNode* child);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_geom_set(BKNode* child, [NativeTypeName("const char *")] sbyte* key, double value);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern double BK_geom_get([NativeTypeName("const BKNode *")] BKNode* child, [NativeTypeName("const char *")] sbyte* key);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_frame_grid_weight(BKNode* frame, int column_axis, int index, double weight, int minsize);

        public const uint BK_IMAGE_NONE = 0;
        public const uint BK_IMAGE_PNG = 1;
        public const uint BK_IMAGE_JPEG = 2;
        public const uint BK_IMAGE_WEBP = 3;
        public const uint BK_IMAGE_GIF = 4;
        public const uint BK_IMAGE_BMP = 5;
        public const uint BK_IMAGE_TGA = 6;
        public const uint BK_IMAGE_QOI = 7;
        public const uint BK_IMAGE_HDR = 8;
        public const uint BK_IMAGE_PNM = 9;
        public const uint BK_IMAGE_SVG = 10;
        public const uint BK_IMAGE_VIDEO = 11;

        public const uint BK_FIT_FILL = 0;
        public const uint BK_FIT_CONTAIN = 1;
        public const uint BK_FIT_COVER = 2;
        public const uint BK_FIT_NONE = 3;
        public const uint BK_FIT_SCALE_DOWN = 4;

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_image_probe([NativeTypeName("const void *")] void* data, [NativeTypeName("size_t")] nuint len, BKImageInfo* r_info);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_image_probe_file([NativeTypeName("const char *")] sbyte* path, BKImageInfo* r_info);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        [return: NativeTypeName("const char *")]
        public static extern sbyte* BK_image_format_name(int format);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern BKImBuf* BK_imbuf_decode([NativeTypeName("const void *")] void* data, [NativeTypeName("size_t")] nuint len, int max_w, int max_h);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern BKImBuf* BK_imbuf_load([NativeTypeName("const char *")] sbyte* path, int max_w, int max_h);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern BKImBuf* BK_imbuf_from_rgba([NativeTypeName("const unsigned char *")] byte* rgba, int w, int h, int stride);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern BKImBuf* BK_imbuf_from_argb([NativeTypeName("const uint32_t *")] uint* argb, int w, int h);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_imbuf_free(BKImBuf* ib);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_imbuf_size([NativeTypeName("const BKImBuf *")] BKImBuf* ib, int* r_w, int* r_h);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_imbuf_format([NativeTypeName("const BKImBuf *")] BKImBuf* ib);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        [return: NativeTypeName("const uint32_t *")]
        public static extern uint* BK_imbuf_pixels([NativeTypeName("const BKImBuf *")] BKImBuf* ib, int frame);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_imbuf_delay([NativeTypeName("const BKImBuf *")] BKImBuf* ib, int frame);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_imbuf_loops([NativeTypeName("const BKImBuf *")] BKImBuf* ib);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_imbuf_rgba([NativeTypeName("const BKImBuf *")] BKImBuf* ib, int frame, [NativeTypeName("unsigned char *")] byte* dst);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern BKImBuf* BK_imbuf_decode_frame([NativeTypeName("const void *")] void* data, [NativeTypeName("size_t")] nuint len, int frame, int max_w, int max_h);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_imbuf_texture([NativeTypeName("const BKImBuf *")] BKImBuf* ib, int frame);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_imbuf_texture_update([NativeTypeName("const BKImBuf *")] BKImBuf* ib, int frame, int tex);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern BKAnim* BK_anim_open([NativeTypeName("const void *")] void* data, [NativeTypeName("size_t")] nuint len, int max_w, int max_h);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern BKImBuf* BK_anim_next(BKAnim* anim, int* r_delay_ms);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_anim_info([NativeTypeName("const BKAnim *")] BKAnim* anim, int* r_w, int* r_h, int* r_loops);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_anim_free(BKAnim* anim);

        public const uint BK_VIDEO_LOADING = 0;
        public const uint BK_VIDEO_PLAYING = 1;
        public const uint BK_VIDEO_PAUSED = 2;
        public const uint BK_VIDEO_ENDED = 3;
        public const uint BK_VIDEO_ERROR = 4;

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern BKVideo* BK_video_new([NativeTypeName("const char *")] sbyte* src);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_video_free(BKVideo* v);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_video_play(BKVideo* v);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_video_pause(BKVideo* v);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_video_seek(BKVideo* v, double seconds);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_video_set_rate(BKVideo* v, double rate);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_video_set_loop(BKVideo* v, int loop);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_video_set_poster(BKVideo* v, [NativeTypeName("const char *")] sbyte* src);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_video_set_controls(BKVideo* v, int on);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_video_set_muted(BKVideo* v, int muted);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_video_muted([NativeTypeName("const BKVideo *")] BKVideo* v);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_video_set_audio_effects(BKVideo* v, int on);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_video_audio_effects([NativeTypeName("const BKVideo *")] BKVideo* v);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_video_set_volume(BKVideo* v, double volume);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern double BK_video_volume([NativeTypeName("const BKVideo *")] BKVideo* v);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_video_has_audio([NativeTypeName("const BKVideo *")] BKVideo* v);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_video_state([NativeTypeName("const BKVideo *")] BKVideo* v);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern double BK_video_time([NativeTypeName("const BKVideo *")] BKVideo* v);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern double BK_video_duration([NativeTypeName("const BKVideo *")] BKVideo* v);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_video_size([NativeTypeName("const BKVideo *")] BKVideo* v, int* r_w, int* r_h);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_video_set_listener(BKVideo* v, [NativeTypeName("void (*)(BKVideo *, int, void *)")] delegate* unmanaged[Cdecl]<BKVideo*, int, void*, void> fn, void* data);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_video_texture(BKVideo* v, int tex, int max_w, int max_h);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_node_set_video(BKNode* node, BKVideo* v, int fit);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        [return: NativeTypeName("const char *")]
        public static extern sbyte* BK_video_backends();

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_image_set_budget([NativeTypeName("size_t")] nuint bytes);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_image_set_threads(int n);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_image_set_notify([NativeTypeName("void (*)(void *)")] delegate* unmanaged[Cdecl]<void*, void> notify, void* data);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_image_poll();

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_image_stats(BKImageStats* r_stats);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_image_info([NativeTypeName("const char *")] sbyte* src, BKImageInfo* r_info);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_image_prefetch([NativeTypeName("const char *")] sbyte* src, int w, int h, int fit);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_image_clear_cache();

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_image_exit();

        public const uint BK_ITEM_RECT = 0;
        public const uint BK_ITEM_OVAL = 1;
        public const uint BK_ITEM_LINE = 2;
        public const uint BK_ITEM_POLYGON = 3;
        public const uint BK_ITEM_TEXT = 4;
        public const uint BK_ITEM_ARC = 5;
        public const uint BK_ITEM_IMAGE = 6;
        public const uint BK_ITEM_ROUNDRECT = 7;
        public const uint BK_ITEM_EFFECT = 8;

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_canvas_set_item(BKNode* canvas, [NativeTypeName("const BKCanvasItem *")] BKCanvasItem* item);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_canvas_add_item(BKNode* canvas, [NativeTypeName("const BKCanvasItem *")] BKCanvasItem* item);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_canvas_put_item(BKNode* canvas, int index, [NativeTypeName("const BKCanvasItem *")] BKCanvasItem* item);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_canvas_truncate(BKNode* canvas, int count);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_canvas_delete(BKNode* canvas, int id);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_canvas_raise(BKNode* canvas, int id, int to_top);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_canvas_count(BKNode* canvas);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_node_set_image(BKNode* node, [NativeTypeName("const unsigned char *")] byte* rgba, int w, int h);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_node_set_imbuf(BKNode* node, [NativeTypeName("const BKImBuf *")] BKImBuf* ib);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_node_set_image_source(BKNode* node, [NativeTypeName("const char *")] sbyte* src, int fit);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_node_set_image_frame(BKNode* node, int frame);

        public const uint BK_CUSTOM_PRESS = 1;
        public const uint BK_CUSTOM_RELEASE = 2;
        public const uint BK_CUSTOM_MOTION = 3;
        public const uint BK_CUSTOM_LEAVE = 4;
        public const uint BK_CUSTOM_WHEEL = 5;
        public const uint BK_CUSTOM_KEY = 6;
        public const uint BK_CUSTOM_KEYUP = 7;
        public const uint BK_CUSTOM_FOCUS = 8;
        public const uint BK_CUSTOM_BLUR = 9;

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_custom_set(BKNode* node, [NativeTypeName("const BKCustomClass *")] BKCustomClass* cls, void* data);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void* BK_custom_data(BKNode* node);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_custom_request_frame(BKNode* node, int ms);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_custom_set_cursor(BKNode* node, int cursor);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_custom_has_focus(BKNode* node);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern BKContext* BK_node_context([NativeTypeName("const BKNode *")] BKNode* node);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_textarea_insert(BKNode* n, int offset, [NativeTypeName("const char *")] sbyte* text);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_textarea_delete(BKNode* n, int start, int end);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_textarea_caret(BKNode* n);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_textarea_set_caret(BKNode* n, int offset);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_textarea_selection(BKNode* n, int* r_start, int* r_end);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_textarea_set_selection(BKNode* n, int start, int end);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_textarea_set_spans(BKNode* n, [NativeTypeName("const BKTextSpan *")] BKTextSpan* spans, int count);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_textarea_index_at(BKNode* n, int x, int y);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_textarea_see(BKNode* n, int offset);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_tree_item_set_values(BKTreeItem* item, [NativeTypeName("const char *const *")] sbyte** values, int n);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_undo(BKContext* ctx);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_redo(BKContext* ctx);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_undo_depth(BKContext* ctx, int* r_redo);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_animating(BKContext* ctx);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_set_animations(int enabled);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern BKContext* BK_context_new(BKNode* root);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_context_free(BKContext* ctx);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_context_set_host(BKContext* ctx, [NativeTypeName("const BKHost *")] BKHost* host);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_context_set_header(BKContext* ctx, BKNode* header);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern BKNode* BK_context_root(BKContext* ctx);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_render(BKContext* ctx, [NativeTypeName("BuiRaster *")] void* r, int w, int h, int full);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_configure(BKContext* ctx, int w, int h);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_needs_render(BKContext* ctx);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_press(BKContext* ctx, int x, int y, int mods);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_release(BKContext* ctx, int x, int y, int mods);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_motion(BKContext* ctx, int x, int y, int mods);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_wheel(BKContext* ctx, int x, int y, int delta, int mods);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_scroll(BKContext* ctx, int x, int y, float dx, float dy, int mods);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_key(BKContext* ctx, [NativeTypeName("const char *")] sbyte* text, [NativeTypeName("const char *")] sbyte* keysym, int mods);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_context_menu(BKContext* ctx, int x, int y, int mods);

        public const uint BK_IME_COMPOSITION_START = 0;
        public const uint BK_IME_COMPOSITION = 1;
        public const uint BK_IME_COMPOSITION_END = 2;

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_ime_event(BKContext* ctx, int type, [NativeTypeName("const BKImeEvent *")] BKImeEvent* e);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_ime_state(BKContext* ctx, BKImeState* state);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_ime_composing(BKContext* ctx);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_key_release(BKContext* ctx, [NativeTypeName("const char *")] sbyte* keysym, int mods);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_button(BKContext* ctx, int x, int y, int button, int release, int mods);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_timer(BKContext* ctx, int timer);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_bind(BKContext* ctx, [NativeTypeName("const char *")] sbyte* combo, void* @ref);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_close_popups(BKContext* ctx);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_set_pointer(BKContext* ctx, int x, int y);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_cursor(BKContext* ctx);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern BKNode* BK_hover(BKContext* ctx);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern BKNode* BK_focus(BKContext* ctx);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_focus_node(BKContext* ctx, BKNode* n);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        [return: NativeTypeName("char *")]
        public static extern sbyte* BK_clipboard_get(BKContext* ctx);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_clipboard_set(BKContext* ctx, [NativeTypeName("const char *")] sbyte* text);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_modal_active(BKContext* ctx);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_tooltip_visible(BKContext* ctx);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_popup_count(BKContext* ctx);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern BKNode* BK_popup(BKContext* ctx, int i);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern BKNode* BK_popup_owner(BKContext* ctx, int i);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_menu_row_count(BKNode* menu);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        [return: NativeTypeName("const char *")]
        public static extern sbyte* BK_menu_row_text(BKNode* menu, int i);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_menu_row_rect(BKNode* menu, int i, [NativeTypeName("int[4]")] int* r_rect);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern BKNode* BK_picker_part(BKNode* picker, [NativeTypeName("const char *")] sbyte* name);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_window_supported();

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern BKWindow* BK_window_new(BKContext* ctx, [NativeTypeName("const char *")] sbyte* title, int width, int height);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_window_free(BKWindow* win);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_window_set_callbacks(BKWindow* win, [NativeTypeName("const BKWindowCallbacks *")] BKWindowCallbacks* cb);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_window_show(BKWindow* win);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_window_close(BKWindow* win);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_window_is_open(BKWindow* win);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_window_set_title(BKWindow* win, [NativeTypeName("const char *")] sbyte* title);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_window_set_size(BKWindow* win, int width, int height);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_window_get_size(BKWindow* win, int* r_width, int* r_height);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        [return: NativeTypeName("BuiRaster *")]
        public static extern void* BK_window_raster(BKWindow* win);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_window_set_gpu(BKWindow* win, int on);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_window_gpu(BKWindow* win);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_window_frames(BKWindow* win);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_window_render_now(BKWindow* win);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_window_timer(BKWindow* win, int ms);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_window_request_frame(BKWindow* win, int vsync);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_window_cancel_timer(BKWindow* win, int id);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void* BK_window_handle(BKWindow* win);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_window_wm(BKWindow* win, [NativeTypeName("const char *")] sbyte* key, double a, double b);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_window_wm_get(BKWindow* win, [NativeTypeName("const char *")] sbyte* key, double* r_a, double* r_b);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_window_run();

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_window_pump();

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_window_quit();

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_window_set_stop_check([NativeTypeName("int (*)()")] delegate* unmanaged[Cdecl]<int> stop);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_window_wake_init([NativeTypeName("void (*)()")] delegate* unmanaged[Cdecl]<void> cb);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_window_wake();

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_window_set_keep_alive(int on);

        public const uint BK_WIDGET_BUTTON = 1;
        public const uint BK_WIDGET_TOGGLE = 2;
        public const uint BK_WIDGET_CHECKBOX = 3;
        public const uint BK_WIDGET_RADIO = 4;
        public const uint BK_WIDGET_NUMBER = 5;
        public const uint BK_WIDGET_SLIDER = 6;
        public const uint BK_WIDGET_TEXT = 7;
        public const uint BK_WIDGET_MENU = 8;
        public const uint BK_WIDGET_PULLDOWN = 9;
        public const uint BK_WIDGET_MENU_ITEM = 10;
        public const uint BK_WIDGET_LIST_ITEM = 11;
        public const uint BK_WIDGET_SWATCH = 12;
        public const uint BK_WIDGET_PROGRESS = 13;
        public const uint BK_WIDGET_TAB = 14;
        public const uint BK_WIDGET_PANEL_HEADER = 15;
        public const uint BK_WIDGET_DISCLOSURE = 16;
        public const uint BK_WIDGET_MENU_BACK = 17;
        public const uint BK_WIDGET_TOOLTIP_BACK = 18;
        public const uint BK_WIDGET_SEPARATOR = 19;
        public const uint BK_WIDGET_LABEL = 20;
        public const uint BK_WIDGET_ARROW = 21;
        public const uint BK_WIDGET_BOX = 22;

        public const uint BK_WS_HOVER = 1 << 0;
        public const uint BK_WS_PRESSED = 1 << 1;
        public const uint BK_WS_SELECTED = 1 << 2;
        public const uint BK_WS_DISABLED = 1 << 3;
        public const uint BK_WS_EDITING = 1 << 4;
        public const uint BK_WS_HOVER_LEFT = 1 << 5;
        public const uint BK_WS_HOVER_RIGHT = 1 << 6;
        public const uint BK_WS_MIXED = 1 << 7;
        public const uint BK_WS_FLAT = 1 << 8;
        public const uint BK_WS_OPEN = 1 << 9;

        public const uint BK_CORNERS_ALL = 15;

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_draw_widget([NativeTypeName("BuiRaster *")] void* r, [NativeTypeName("const BKWidgetDraw *")] BKWidgetDraw* w);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern float BK_text_width([NativeTypeName("const char *")] sbyte* s, int len);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_eval_number([NativeTypeName("const char *")] sbyte* expr, double* r_value);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern int BK_file_dialog([NativeTypeName("const char *")] sbyte* title, int mode, [NativeTypeName("const char *")] sbyte* dir, [NativeTypeName("const char *")] sbyte* file, [NativeTypeName("const char *const *")] sbyte** filters, int nfilters, int multiple, [NativeTypeName("char ***")] sbyte*** r_paths);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_file_dialog_free([NativeTypeName("char **")] sbyte** paths);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_recent_searches_enable(int enabled);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_recent_searches_clear();

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_recent_searches_load([NativeTypeName("const char *")] sbyte* path);

        [DllImport("bui_kit", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void BK_recent_searches_save([NativeTypeName("const char *")] sbyte* path);

        [NativeTypeName("#define BUI_KIT_VERSION_MAJOR 0")]
        public const int BUI_KIT_VERSION_MAJOR = 0;

        [NativeTypeName("#define BUI_KIT_VERSION_MINOR 5")]
        public const int BUI_KIT_VERSION_MINOR = 5;

        [NativeTypeName("#define BUI_KIT_VERSION_PATCH 1")]
        public const int BUI_KIT_VERSION_PATCH = 1;

        [NativeTypeName("#define BUI_KIT_VERSION_STRING \"0.5.1\"")]
        public static ReadOnlySpan<byte> BUI_KIT_VERSION_STRING => "0.5.1"u8;

        [NativeTypeName("#define BUI_KIT_VERSION (BUI_KIT_VERSION_MAJOR * 10000 + BUI_KIT_VERSION_MINOR * 100 + BUI_KIT_VERSION_PATCH)")]
        public const int BUI_KIT_VERSION = (0 * 10000 + 5 * 100 + 1);

        [NativeTypeName("#define BK_EFFECT_MAX_INPUTS BUI_EFFECT_MAX_INPUTS")]
        public const int BK_EFFECT_MAX_INPUTS = 8;

        [NativeTypeName("#define BK_EFFECT_MAX_FLOATS BUI_EFFECT_MAX_FLOATS")]
        public const int BK_EFFECT_MAX_FLOATS = 256;

        [NativeTypeName("#define BK_EFFECT_MAX_SOURCE BUI_EFFECT_MAX_SOURCE")]
        public const int BK_EFFECT_MAX_SOURCE = 65536;
    }
}
