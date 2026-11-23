//! The interface colors charts draw their frame with (text, grid, tooltips, legends), taken from the
//! active kit theme, Blender's: they follow `blenderui::set_theme` and themes loaded with
//! `blenderui::load_theme`. Named as gpui-kit's theme fields the charts used before. The series
//! colors are the chart theme's ([`crate::ChartTheme`]).

use std::cell::Cell;

use blenderui::{Hsla, rgb};

use crate::color::mix;
use crate::kit;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Theme {
    pub background: Hsla,
    pub foreground: Hsla,
    pub muted_foreground: Hsla,
    /// a track or empty well (gauges, bars)
    pub muted: Hsla,
    pub border: Hsla,
    /// a pressed or selected control
    pub accent: Hsla,
    pub popover: Hsla,
    pub popover_foreground: Hsla,
    /// secondary text on a popover (its own surface: the page's muted color can vanish on it)
    pub popover_muted: Hsla,
    pub primary: Hsla,
    pub secondary: Hsla,
    pub chart_grid: Hsla,
    pub success: Hsla,
    pub warning: Hsla,
    pub danger: Hsla,
    pub info: Hsla,
    pub dark: bool,
}

thread_local! {
    static CACHED: Cell<Option<(i32, Theme)>> = const { Cell::new(None) };
}

fn color(key: &std::ffi::CStr, fallback: u32) -> Hsla {
    kit::theme_color(key).unwrap_or_else(|| rgb(fallback))
}

fn read() -> Theme {
    let dark = kit::dark();
    let background = color(c"region_bg", if dark { 0x303030 } else { 0xb3b3b3 });
    let foreground = color(c"text", if dark { 0xe6e6e6 } else { 0x1a1a1a });
    let popover = Hsla { a: 1., ..color(c"tooltip.inner", 0x191919) };
    let popover_foreground = color(c"tooltip.text", 0xe6e6e6);
    Theme {
        background,
        foreground,
        muted_foreground: mix(foreground, background, 0.38),
        muted: mix(background, foreground, 0.12),
        border: foreground.opacity(0.16),
        accent: mix(background, foreground, 0.14),
        popover,
        popover_foreground,
        popover_muted: mix(popover_foreground, popover, 0.4),
        primary: color(c"regular.inner_sel", 0x4772b3),
        secondary: color(c"regular.inner", 0x545454),
        chart_grid: foreground.opacity(0.1),
        success: rgb(0x40a02b),
        warning: rgb(0xdf8e1d),
        danger: rgb(0xd20f39),
        info: rgb(0x1e66f5),
        dark,
    }
}

/// The interface colors of the active kit theme (read again only when the theme changes).
pub(crate) fn theme() -> Theme {
    let generation = kit::theme_generation();
    CACHED.with(|cell| match cell.get() {
        Some((seen, theme)) if seen == generation => theme,
        _ => {
            let theme = read();
            cell.set(Some((generation, theme)));
            theme
        }
    })
}
