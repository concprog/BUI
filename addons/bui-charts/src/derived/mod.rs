//! The charts derived from the base ones: each turns its rows into a table (`crate::derive`:
//! `sync(table, &data, settings, |rows| Build { .. })`) and shows it with a base chart, or lays out its
//! own shapes on a [`scene`]. Helpers for them, as in zopra-charts but without `window` / `cx`:
//!
//! - `trend_view(table: Signal<Table>, TrendSpec) -> AnyElement` (a trend over the table, x = column 0)
//! - `xy_view / xy_equal(table, XySpec) / xy_locked(table, XySpec, ratio: f64) -> AnyElement`
//! - `field(name, column, color, Style) -> TrendField<Row>`, `series(name, x, y, color, xy Style) -> XYSeries<Row>`
//! - styles: `bars(width)`, `lines(width, smooth)`, `dots(size)`, `xy_points(size)`, `xy_line(width, LineStyle)`
//! - `categories(&[SharedString]) -> AxisConfig`, `percent`, `magnitude`, `mean`, `deviation`, `nice`,
//!   `padded`, `Sketch` (polylines as NaN-separated rows)
//! - `card(Tip) -> AnyElement`: a tooltip card (`Tip { at: geom::Point, area: geom::Size, title, accent, rows }`,
//!   positions local to the chart)
//!
//! Colors come from `ChartTheme::colors()`, interface colors from `crate::ui::theme()`.

pub mod category;
pub mod circular;
pub mod diagram;
pub mod distribution;
pub mod finance;
pub mod flow;
pub mod geo;
pub mod glyphs;
pub mod hybrid;
pub mod matrix;
pub mod profile;
pub mod quality;
pub(crate) mod sankey;
pub(crate) mod scene;
pub mod science;
pub mod sets;
pub mod ternary;
pub mod thermo;
pub mod tiles;
pub mod tree;
pub mod vector;
use blenderui::element::{AnyElement, IntoElement, ParentElement};
use blenderui::hooks::Signal;
use blenderui::html::div;
use blenderui::{App, Hsla, SharedString, Styled, Window};

use crate::derive::{Row, Rows, Table, column};
use crate::geom::{Point, Size};
use crate::plot::{AxisConfig, HideFrom, LineStyle, SelectedRange, Tooltip, corner};
use crate::trend::{
    Chart, Defaults, DrawStyle, Interpolation, PointsMode, Style, TrendField, chart,
};
use crate::xy_chart::{Plot, Show, XYSeries, plot};
use crate::{Legend, xy_chart};

pub(crate) const HIDDEN: HideFrom = HideFrom {
    legend: true,
    tooltip: true,
    viz: false,
};

pub(crate) const QUIET: HideFrom = HideFrom {
    legend: true,
    tooltip: false,
    viz: false,
};

pub(crate) struct TrendSpec {
    pub fields: Vec<TrendField<Row>>,
    pub style: Style,
    pub defaults: Defaults,
    pub legend: Option<Legend>,
    pub tooltip: Option<Tooltip>,
    pub class: Option<SharedString>,
}

pub(crate) fn trend_view(
    table: Signal<Table>,
    spec: TrendSpec
) -> AnyElement {
    chart(
        Chart {
            data: Rows(table),
            x: Some(column(0)),
            x_field: None,
            ends: None,
            fields: spec.fields,
            style: spec.style,
            defaults: spec.defaults,
            annotations: None,
            compare: None,
            keyboard: true,
            legend: spec.legend,
            tooltip: spec.tooltip,
            class: spec.class,
        },
        None::<fn(SelectedRange, &mut Window, &mut App)>)
}

pub(crate) struct XySpec {
    pub series: Vec<XYSeries<Row>>,
    pub style: xy_chart::Style,
    pub x_axis: AxisConfig,
    pub y_axis: AxisConfig,
    pub legend: Option<Legend>,
    pub tooltip: Option<Tooltip>,
    pub class: Option<SharedString>,
}

pub(crate) fn xy_view(
    table: Signal<Table>,
    spec: XySpec
) -> AnyElement {
    xy_plot(table, spec, None)
}

pub(crate) fn xy_equal(
    table: Signal<Table>,
    spec: XySpec
) -> AnyElement {
    xy_plot(table, spec, Some(1.))
}

pub(crate) fn xy_locked(
    table: Signal<Table>,
    spec: XySpec,
    ratio: f64
) -> AnyElement {
    xy_plot(table, spec, Some(ratio))
}

fn xy_plot(
    table: Signal<Table>,
    spec: XySpec,
    equal: Option<f64>
) -> AnyElement {
    plot(
        Plot {
            data: Rows(table),
            series: spec.series,
            style: spec.style,
            x_axis: Some(spec.x_axis),
            y_axis: Some(spec.y_axis),
            legend: spec.legend,
            tooltip: spec.tooltip,
            class: spec.class,
            equal,
        })
}

pub(crate) fn field(
    name: impl Into<SharedString>,
    index: usize,
    color: Option<Hsla>,
    style: Style,
) -> TrendField<Row> {
    TrendField::new(name.into(), column(index), color, style)
}

pub(crate) fn series(
    name: impl Into<SharedString>,
    x: usize,
    y: usize,
    color: Option<Hsla>,
    style: xy_chart::Style,
) -> XYSeries<Row> {
    XYSeries::new(name.into(), column(x), column(y), color, style)
}

pub(crate) fn categories(names: &[SharedString]) -> AxisConfig {
    AxisConfig::new()
        .categories(names.iter().cloned())
        .min(-0.5)
        .max(names.len() as f64 - 0.5)
}

pub(crate) fn bars(width: f32) -> Style {
    Style {
        draw_style: Some(DrawStyle::Bars),
        fill_opacity: Some(1.),
        line_width: Some(0.),
        bar_width_factor: Some(width),
        show_points: Some(PointsMode::Never),
        ..Style::default()
    }
}

pub(crate) fn lines(width: f32, smooth: bool) -> Style {
    Style {
        draw_style: Some(DrawStyle::Line),
        line_width: Some(width),
        interpolation: Some(if smooth {
            Interpolation::Smooth
        } else {
            Interpolation::Linear
        }),
        ..Style::default()
    }
}

pub(crate) fn dots(size: f32) -> Style {
    Style {
        draw_style: Some(DrawStyle::Points),
        point_size: Some(size),
        show_points: Some(PointsMode::Always),
        ..Style::default()
    }
}

pub(crate) fn percent(value: f64) -> SharedString {
    format!("{}%", crate::format::trimmed(value)).into()
}

pub(crate) fn magnitude(value: f64) -> SharedString {
    crate::format::trimmed(value.abs())
}

pub(crate) fn mean(values: &[f64]) -> f64 {
    let finite: Vec<f64> = values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .collect();
    if finite.is_empty() {
        return f64::NAN;
    }
    finite.iter().sum::<f64>() / finite.len() as f64
}

pub(crate) fn deviation(values: &[f64]) -> f64 {
    let center = mean(values);
    let finite: Vec<f64> = values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .collect();
    if finite.len() < 2 {
        return 0.;
    }
    (finite
        .iter()
        .map(|value| (value - center).powi(2))
        .sum::<f64>()
        / (finite.len() - 1) as f64)
        .sqrt()
}

pub(crate) fn xy_points(size: f32) -> xy_chart::Style {
    xy_chart::Style {
        point_size: Some(size),
        ..xy_chart::Style::default()
    }
}

pub(crate) fn xy_line(width: f32, dash: LineStyle) -> xy_chart::Style {
    xy_chart::Style {
        show: Some(Show::Lines),
        line_width: Some(width),
        line_style: Some(dash),
        ..xy_chart::Style::default()
    }
}

pub(crate) fn padded(width: usize, cells: &[(usize, f64)]) -> Row {
    let mut out = vec![f64::NAN; width];
    for (index, value) in cells {
        out[*index] = *value;
    }
    out.into_boxed_slice()
}

pub(crate) struct Sketch {
    pub rows: Vec<Row>,
    width: usize,
}

impl Sketch {
    pub(crate) const fn new(width: usize) -> Self {
        Self {
            rows: Vec::new(),
            width,
        }
    }

    pub(crate) fn point(&mut self, x: usize, at: (f64, f64)) {
        self.rows
            .push(padded(self.width, &[(x, at.0), (x + 1, at.1)]));
    }

    pub(crate) fn lift(&mut self) {
        self.rows.push(padded(self.width, &[]));
    }

    pub(crate) fn trace(&mut self, x: usize, points: impl IntoIterator<Item = (f64, f64)>) {
        for at in points {
            self.point(x, at);
        }
        self.lift();
    }

    pub(crate) fn arrow(&mut self, x: usize, from: (f64, f64), to: (f64, f64), head: f64) {
        let (dx, dy) = (to.0 - from.0, to.1 - from.1);
        let length = dx.hypot(dy);
        if !(length > 0. && length.is_finite()) {
            return;
        }
        let (ux, uy) = (dx / length * head, dy / length * head);
        let barb = |sign: f64| (to.0 - ux - sign * uy * 0.6, to.1 - uy + sign * ux * 0.6);
        self.trace(x, [from, to]);
        self.trace(x, [barb(1.), to, barb(-1.)]);
    }

    pub(crate) fn ellipse(
        &mut self,
        x: usize,
        center: (f64, f64),
        radii: (f64, f64),
        steps: usize,
    ) {
        self.trace(
            x,
            (0..=steps).map(|step| {
                let angle = std::f64::consts::TAU * step as f64 / steps as f64;
                (
                    radii.0.mul_add(angle.cos(), center.0),
                    radii.1.mul_add(angle.sin(), center.1),
                )
            }),
        );
    }

    pub(crate) fn clipped(
        &mut self,
        x: usize,
        points: impl IntoIterator<Item = (f64, f64)>,
        keep: impl Fn((f64, f64)) -> bool,
    ) {
        let mut open = false;
        for at in points {
            if keep(at) {
                self.point(x, at);
                open = true;
            } else if open {
                self.lift();
                open = false;
            }
        }
        if open {
            self.lift();
        }
    }
}

pub(crate) fn nice(raw: f64) -> f64 {
    if !(raw.is_finite() && raw > 0.) {
        return 1.;
    }
    let power = 10f64.powf(raw.log10().floor());
    let scaled = raw / power;
    power
        * if scaled <= 1. {
            1.
        } else if scaled <= 2. {
            2.
        } else if scaled <= 5. {
            5.
        } else {
            10.
        }
}

pub(crate) struct Tip {
    pub at: Point,
    pub area: Size,
    pub title: SharedString,
    pub accent: Option<Hsla>,
    pub rows: Vec<(Option<Hsla>, SharedString, SharedString)>,
}

fn swatch(color: Hsla) -> blenderui::element::Div {
    div().class("size-[10px] rounded-full flex-none").bg(color)
}

/// A tooltip card beside the pointer (the pointer passes through it).
pub(crate) fn card(tip: Tip) -> AnyElement {
    let theme = crate::ui::theme();
    let Tip { at, area, title, accent, rows } = tip;
    corner(crate::trend::card(), at, area)
        .bg(theme.popover)
        .text_color(theme.popover_foreground)
        .border_color(theme.border)
        .child(div().class("flex items-center gap-2 font-semibold").child(accent.map(swatch)).child(title))
        .children(rows.into_iter().map(move |(color, name, value)| {
            div()
                .class("flex items-center justify-between gap-4")
                .child(
                    div()
                        .class("flex items-center gap-2")
                        .text_color(theme.popover_muted)
                        .child(color.map(swatch))
                        .child(name),
                )
                .child(div().class("font-medium").child(value))
        }))
        .into_any_element()
}
