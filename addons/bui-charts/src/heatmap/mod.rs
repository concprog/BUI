mod d3_data;
mod grid;
mod model;
mod options;
mod paint;
mod schemes;
mod view;

use blenderui::element::{AnyElement, Div, FluentBuilder, IntoElement, ParentElement};
use blenderui::hooks::use_signal;
use blenderui::html::div;
use blenderui::{App, Hsla, MouseButton, SharedString, Styled, Window, canvas, rgba};

pub use options::{
    BucketLayout, BucketScale, Buckets, Calculation, HeatmapColor, HeatmapLegend, HeatmapTooltip, OpacityScale,
};
pub use schemes::HeatmapScheme;

use crate::component;
use crate::data::Accessor;
use crate::legend::LegendPlacement;
use crate::plot::{AxisConfig, Drag, ScaleKind, SelectSlot, SelectedRange, SelectionMode, VisibilityMode, Zoom};
use crate::surface::use_surface;
use crate::{ChartTheme, Dataset};
use model::{Cache, Format, Model, Source};
use paint::{Hover, Layout, Palette, Plan, Pointer, paint};
use view::{TipData, color_bar, heatmap_tip};

const HIDE_BELOW: f64 = 1e-9;
const EXEMPLAR: u32 = 0xff00_ffb3;

#[component]
#[allow(clippy::too_many_lines)]
pub fn heatmap<
    T: 'static,
    D: Dataset<T>,
    X: Fn(&T) -> f64 + 'static,
    Y: Fn(&T) -> f64 + 'static,
    V: Fn(&T) -> f64 + 'static,
    XE: Fn(&T) -> f64 + 'static,
    YE: Fn(&T) -> f64 + 'static,
    F: Fn(f64) -> SharedString + 'static,
    S: Fn(SelectedRange, &mut Window, &mut App) + 'static,
>(
    data: D,
    x: Option<X>,
    y: Option<Y>,
    value: Option<V>,
    x_end: Option<XE>,
    y_end: Option<YE>,
    calculate: Option<Calculation>,
    x_field: Option<&'static str>,
    bucket_layout: Option<BucketLayout>,
    bucket_scale: Option<ScaleKind>,
    color: Option<HeatmapColor>,
    color_min: Option<f64>,
    color_max: Option<f64>,
    cell_gap: Option<f32>,
    cell_radius: Option<f32>,
    show_value: Option<VisibilityMode>,
    cell_format: Option<F>,
    cell_decimals: Option<usize>,
    hide_le: Option<f64>,
    hide_ge: Option<f64>,
    x_axis: Option<AxisConfig>,
    y_axis: Option<AxisConfig>,
    tooltip: Option<HeatmapTooltip>,
    legend: Option<HeatmapLegend>,
    exemplars: Option<Vec<(f64, f64)>>,
    #[into] exemplar_color: Option<Hsla>,
    selection: Option<SelectionMode>,
    on_select: Option<S>,
    class: Option<SharedString>,
) -> AnyElement {
    let cache = use_signal(Cache::default());
    let surface = use_surface::<Layout>();
    let hover = use_signal(None::<Hover>);
    let drag = use_signal(None::<Drag>);
    let zoom = use_signal(Zoom::default());
    let format = use_signal(Format(None));
    let select = use_signal(SelectSlot::default());
    format.update_untracked(|slot| slot.0 = cell_format.map(|f| Box::new(f) as Box<_>));
    select.update_untracked(|slot| slot.0 = on_select.map(|f| Box::new(f) as Box<_>));

    let x_axis = x_axis.unwrap_or_default();
    let mut y_axis = y_axis.unwrap_or_default();
    let source = data.schema(|schema| {
        let field = |name: Option<&str>, fallback: usize| {
            schema.and_then(|schema| {
                let index = name
                    .map_or(Some(fallback), |name| schema.names.iter().position(|field| field.as_ref() == name))?;
                (index < schema.names.len()).then_some((index, schema.read))
            })
        };
        let x = x.map(Accessor::row).or_else(|| field(x_field, 0).map(|(index, read)| Accessor::Field(read, index)));
        match (x, calculate, value) {
            (Some(x), Some(calc), _) => y
                .map(Accessor::row)
                .or_else(|| field(None, 1).map(|(index, read)| Accessor::Field(read, index)))
                .map(|y| Source::Samples { x, y, calc }),
            (Some(x), None, Some(value)) => y.map(|y| Source::Cells {
                x,
                y: Accessor::row(y),
                value: Accessor::row(value),
                x_end: x_end.map(Accessor::row),
                y_end: y_end.map(Accessor::row),
            }),
            (Some(x), None, None) => schema.map(|schema| {
                let skip = field(x_field, 0).map_or(0, |(index, _)| index);
                let fields: Vec<usize> = (0..schema.names.len()).filter(|index| *index != skip).collect();
                Source::Buckets {
                    x,
                    names: fields.iter().map(|index| schema.names[*index].clone()).collect(),
                    fields,
                    read: schema.read,
                    layout: bucket_layout.unwrap_or_default(),
                    numeric: bucket_scale.is_some(),
                }
            }),
            _ => None,
        }
    });
    if let Some(kind) = bucket_scale {
        y_axis.scale = kind;
    }

    let theme = crate::ui::theme();
    let palette = Palette { text: theme.muted_foreground, border: theme.border, accent: theme.primary };
    let Some(source) = source else {
        return div()
            .class("flex size-full items-center justify-center text-sm")
            .text_color(palette.text)
            .when_some(class, Div::class)
            .child("No data")
            .into_any_element();
    };
    let revision = data.revision();
    let color = color.unwrap_or_default();
    let colors = ChartTheme::colors();
    cache.update_untracked(|cache| {
        data.rows(|rows| cache.refresh(revision, rows, &source, x_axis.time));
        cache.paint(&color, theme.dark, ChartTheme::revision(), &colors);
    });

    let hide_le = Some(hide_le.unwrap_or(HIDE_BELOW)).filter(|limit| limit.is_finite());
    let hide_ge = hide_ge.filter(|limit| limit.is_finite());
    let hidden = |value: f64| hide_le.is_some_and(|limit| value <= limit) || hide_ge.is_some_and(|limit| value >= limit);
    let tooltip = tooltip.unwrap_or_default();
    let legend = legend.unwrap_or_default();
    let hovered = hover.get();
    let (model, tip, ends, marker) = cache.with(|cache| {
        let grid = &cache.grid;
        let range = grid.range(hidden).unwrap_or((0., 1.));
        let color_range = (color_min.unwrap_or(range.0), color_max.unwrap_or(range.1));
        let (x_scale, y_scale) = Model::scales(grid, &x_axis, &y_axis, zoom.get());
        let model = Model {
            x: x_scale,
            y: y_scale,
            x_axis,
            y_axis,
            color_range,
            hide_le,
            hide_ge,
            gap: cell_gap.unwrap_or(1.).clamp(0., 25.),
            radius: cell_radius.unwrap_or(0.).max(0.),
            show_value: show_value.unwrap_or_default(),
            decimals: cell_decimals,
            exemplars: exemplars.unwrap_or_default(),
            exemplar_color: exemplar_color.unwrap_or_else(|| rgba(EXEMPLAR)),
            selection: selection.unwrap_or_default(),
        };
        let value_at = hovered.map(|hover| grid.value(hover.column, hover.row));
        let position = |value: f64| {
            let span = color_range.1 - color_range.0;
            (span > 0.).then(|| ((value - color_range.0) / span).clamp(0., 1.) as f32)
        };
        let tip = format.with(|format| {
            hovered.filter(|_| tooltip.visible).zip(value_at).filter(|(_, value)| !model.hidden(*value)).map(
                |(hover, value)| {
                    let column = grid.column(hover.column);
                    let peak = column.iter().copied().filter(|value| value.is_finite()).fold(0., f64::max);
                    let x_text: SharedString = format!(
                        "{} – {}",
                        model.x_axis.text(grid.x_edges[hover.column], None),
                        model.x_axis.text(grid.x_edges[hover.column + 1], None)
                    )
                    .into();
                    let y_text = grid.labels.get(hover.row).cloned().unwrap_or_else(|| {
                        format!(
                            "{} – {}",
                            model.y_axis.text(grid.y_edges[hover.row], None),
                            model.y_axis.text(grid.y_edges[hover.row + 1], None)
                        )
                        .into()
                    });
                    let data = TipData {
                        x: x_text,
                        y: y_text,
                        value: model.cell_text(value, format),
                        color: model.tone(value, &cache.palette).unwrap_or(palette.text),
                        histogram: column
                            .iter()
                            .enumerate()
                            .map(|(row, value)| {
                                let height =
                                    if peak > 0. && value.is_finite() { (value / peak) as f32 } else { 0. };
                                (height, row == hover.row)
                            })
                            .collect(),
                        position: position(value),
                    };
                    (hover, data)
                },
            )
        });
        let ends = format.with(|format| (model.cell_text(color_range.0, format), model.cell_text(color_range.1, format)));
        let marker = value_at.filter(|value| value.is_finite()).and_then(position);
        (model, tip, ends, marker)
    });
    let tip = tip.map(|(hover, data)| heatmap_tip(tooltip, hover, data, cache));
    let bar = legend.visible.then(|| {
        let (low, high) = ends;
        color_bar(cache, low, high, marker, legend.placement == LegendPlacement::Right, legend.width)
    });
    surface.set(Plan { model, cache });
    let pointer = Pointer { surface, cache, hover, drag, zoom, format, select };
    let on_right = legend.placement == LegendPlacement::Right;

    div()
        .class("flex size-full min-w-0 min-h-0")
        .map(|el| if on_right { el.flex_row() } else { el.flex_col() })
        .when_some(class, Div::class)
        .child(
            div()
                .class("relative flex-1 min-w-0 min-h-0")
                .child(
                    canvas(move |bounds, painter| {
                        surface.paint(bounds.into(), |layout| paint(layout, pointer, palette, painter));
                    })
                    .class("absolute inset-0 size-full")
                    .on_mouse_move(move |event, _, _| paint::on_move(pointer, event))
                    .on_hover(move |inside, _, _| {
                        if !inside {
                            paint::on_leave(pointer);
                        }
                    })
                    .on_mouse_down(MouseButton::Left, move |event, _, _| paint::on_down(pointer, event))
                    .on_mouse_up(MouseButton::Left, move |_, window, cx| paint::on_up(pointer, window, cx))
                    .on_scroll_wheel(move |event, _, cx| paint::on_wheel(pointer, event, cx)),
                )
                .child(tip),
        )
        .child(bar)
        .into_any_element()
}
