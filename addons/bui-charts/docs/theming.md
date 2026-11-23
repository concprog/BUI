# Theming

Charts take colours from two places:

| Colours | Source | Examples |
| --- | --- | --- |
| **Data colours** | `ChartTheme`, loaded from [`themes/catppuccin.json`](../themes/catppuccin.json) | series palette, colour schemes, fallback, hover, good / neutral / bad |
| **UI colours** | the Blender kit theme (`blenderui::set_theme`, `blenderui::load_theme`) | axis text, grid lines, borders, tooltip and legend surfaces |

The data colours live in a JSON file in the layout of gpui-kit's theme sets (the same shape BUI's own theme files use), so you can edit or replace it without touching any code. A chart reads the light or dark variant to match the kit theme's mode (`blenderui::theme_name()`), so switching the app theme switches the chart colours with it.

The UI colours come from the active Blender theme: `region_bg` (background), `text` (axis text, and mixed with the background for muted text and grid lines), `tooltip.inner` / `tooltip.text` (tooltip cards) and `regular.inner_sel` (primary). Themes loaded with `blenderui::load_theme` work the same way.

## The bundled theme: Catppuccin

`themes/catppuccin.json` holds **Latte** (light) and **Mocha** (dark):

| Key | Latte | Mocha |
| --- | --- | --- |
| palette order | blue, peach, green, red, mauve, teal, yellow, pink, sapphire, maroon, lavender, flamingo | same order |
| `fallback` | overlay `#9ca0b0` | overlay `#6c7086` |
| `hover` | base `#eff1f5` | text `#cdd6f4` |
| `good` / `bad` / `neutral` | green / red / overlay | green / red / overlay |

The schemes `green-yellow-red`, `red-yellow-green`, `blue-yellow-red`, `yellow-red`, `blue-purple`, `yellow-blue`, `blues`, `reds`, `greens`, `purples` and `oranges` match the `Scheme` variants with the same names. The single-hue schemes (`blues`, `reds`, and so on) fade from the theme's `surface` colour, so low values blend into the background.

## File format

```json
{
  "name": "My charts",
  "themes": [
    {
      "name": "My charts light",
      "mode": "light",
      "colors": {
        "brand": "#1e66f5",
        "warn": "#df8e1d",
        "bad": "#d20f39",
        "muted": "#9ca0b0",
        "paper": "#eff1f5"
      },
      "palette": ["brand", "warn", "bad", "#40a02b"],
      "schemes": {
        "health": ["brand", "warn", "bad"],
        "heat": ["#04a5e5", "bad"]
      },
      "fallback": "muted",
      "good": "#40a02b",
      "bad": "bad",
      "neutral": "muted",
      "hover": "paper"
    }
  ]
}
```

| Field | Required | Meaning |
| --- | --- | --- |
| `themes[].mode` | yes | `"light"` or `"dark"`. If a mode is missing, the first theme in the file is used for it. |
| `themes[].colors` | no | names you can use in the other fields |
| `themes[].palette` | yes | series colours, used in order and repeated from the start when they run out. Traces picks one per service from this list, and State Timeline one per text state that has no value mapping. |
| `themes[].schemes` | no | named gradients with two or more stops, for `Scheme::Named("…")` or the built-in variants |
| `themes[].fallback` | yes | colour for values that are `NaN` or have no colour |
| `themes[].hover` | yes | hover marker colour; drawn at 40% opacity |
| `themes[].good` | no | colour for "fine" or "improved": the default base threshold of the bar gauge and the trend chart, and frames that shrank in a flame graph diff. Default `#73bf69`. |
| `themes[].bad` | no | colour for "too high" or "regressed": the default threshold at 80 of the bar gauge and the trend chart, and frames that grew in a flame graph diff. Default `#f2495c`. |
| `themes[].neutral` | no | colour for "unchanged" in a flame graph diff. Defaults to `fallback`. |

A colour can be a name from `colors`, or any colour a blenderui theme accepts: `#rrggbb`, `#rrggbbaa`, or a Tailwind name such as `blue`, `blue-500` or `red-600/50`.

## Loading a theme

```rust
use blenderui::prelude::*;

fn main() {
    Application::new().run(|cx: &mut App| {
        bui_charts::ChartTheme::load(include_str!("../assets/charts.json")).expect("valid chart theme");
        // cx.open_window(..)
    });
}
```

`ChartTheme::load` checks every colour and applies the theme to every chart in the app from its next render (loading it after windows are open, call `cx.notify_all()` to re-render them now), and it returns an error that names any colour it can't read. If you never load one, the bundled Catppuccin theme is used. To check a theme file without applying it, for example in a test, call `ChartTheme::parse(json)`.

## Reading theme colours in your own components

```rust
let colors = bui_charts::ChartTheme::colors(); // a `Colors`: derefs to `ChartColors`
let first_series = colors.series(0);
let health = colors.schemes.get("health");
```

## Light and dark

The mode is the kit theme's. Switching it repaints every window; charts compute their colours while
rendering, so re-render too:

```rust
blenderui::set_theme("light"); // or "dark", or a theme added with blenderui::load_theme
cx.notify_all();
```

## Which colour setting to use

| You want | Use |
| --- | --- |
| your brand colours on every chart | a theme file with your own `palette`, loaded with `ChartTheme::load` |
| one series in a specific colour | `color={rgb(..)}` on that `<Series>` |
| a gradient you reuse across charts | add it to `schemes` in your theme file, then use `Scheme::Named("…")` |
| a one-off gradient or threshold list | `Scheme::gradient(..)` / `Scheme::thresholds(..)` on the series |
| different colours in light and dark mode | one theme per `mode` in the same file |

Load your own file rather than editing the bundled `themes/catppuccin.json`, so updating bui-charts never overwrites your colours.
