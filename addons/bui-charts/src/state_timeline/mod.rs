mod model;
mod options;
mod paint;
mod view;

use blenderui::element::{AnyElement, Div, FluentBuilder, IntoElement, ParentElement};
use blenderui::hooks::use_signal;
use blenderui::html::div;
use blenderui::{App, MouseButton, SharedString, Styled, Window, canvas};

pub(crate) use options::{Reader, Style};
pub use options::{StateLane, StateRow, StateRowProps, ValueAlign, state_row};

use crate::bar_gauge::GaugeColor;
use crate::data::Accessor;
use crate::legend::Legend;
use crate::mappings::ValueMappings;
use crate::plot::{
    AxisConfig, Drag, HideFrom, Scale, SelectSlot, SelectedRange, SortOrder, Tooltip, TooltipMode,
    VisibilityMode, Zoom,
};
use crate::trend::NullSpan;
use crate::surface::use_surface;
use crate::{ChartTheme, Dataset, Scheme, Thresholds, component, time};
use model::{Cache, Defaults};
use paint::{Hover, Layout, Palette, Plan, Pointer, Settings, paint};
use view::{TipRow, pager, state_legend, state_tip};

const PER_PAGE: usize = 20;
const ROW_HEIGHT: f32 = 0.9;

#[derive(Clone, Copy)]
pub(crate) enum Cells {
    Merged { merge: bool, align: ValueAlign },
    Samples { width: f32 },
}

pub(crate) struct Timeline<T, D, X> {
    pub data: D,
    pub x: Option<X>,
    pub x_field: Option<&'static str>,
    pub cells: Cells,
    pub show_value: Option<VisibilityMode>,
    pub row_height: Option<f32>,
    pub per_page: Option<usize>,
    pub mappings: Option<&'static ValueMappings>,
    pub color: Option<GaugeColor>,
    pub thresholds: Option<Thresholds>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub style: Style,
    pub x_axis: Option<AxisConfig>,
    pub y_axis: Option<AxisConfig>,
    pub legend: Option<Legend>,
    pub tooltip: Option<Tooltip>,
    pub class: Option<SharedString>,
    pub lanes: Vec<StateLane<T>>,
}

#[component]
pub fn state_timeline<
    T: 'static,
    D: Dataset<T>,
    X: Fn(&T) -> f64 + 'static,
    S: Fn(SelectedRange, &mut Window, &mut App) + 'static,
>(
    data: D,
    x: Option<X>,
    x_field: Option<&'static str>,
    merge_values: Option<bool>,
    show_value: Option<VisibilityMode>,
    align_value: Option<ValueAlign>,
    row_height: Option<f32>,
    per_page: Option<usize>,
    mappings: Option<&'static ValueMappings>,
    color: Option<GaugeColor>,
    thresholds: Option<Thresholds>,
    min: Option<f64>,
    max: Option<f64>,
    fill_opacity: Option<f32>,
    line_width: Option<f32>,
    span_nulls: Option<NullSpan>,
    insert_nulls: Option<f64>,
    hide_from: Option<HideFrom>,
    x_axis: Option<AxisConfig>,
    y_axis: Option<AxisConfig>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    on_select: Option<S>,
    class: Option<SharedString>,
    children: Children<StateLane<T>>,
) -> AnyElement {
    let lanes = children;
    timeline(
        Timeline {
            data,
            x,
            x_field,
            cells: Cells::Merged {
                merge: merge_values.unwrap_or(true),
                align: align_value.unwrap_or_default(),
            },
            show_value,
            row_height,
            per_page,
            mappings,
            color,
            thresholds,
            min,
            max,
            style: Style {
                fill_opacity,
                line_width,
                span_nulls,
                insert_nulls,
                hide_from,
            },
            x_axis,
            y_axis,
            legend,
            tooltip,
            class,
            lanes,
        },
        on_select)
}

#[allow(clippy::too_many_lines)]
pub(crate) fn timeline<
    T: 'static,
    D: Dataset<T>,
    X: Fn(&T) -> f64 + 'static,
    S: Fn(SelectedRange, &mut Window, &mut App) + 'static,
>(
    options: Timeline<T, D, X>,
    on_select: Option<S>,
) -> AnyElement {
    let cache = use_signal(Cache::default());
    let surface = use_surface::<Layout>();
    let hover = use_signal(None::<Hover>);
    let drag = use_signal(None::<Drag>);
    let zoom = use_signal(Zoom::default());
    let page = use_signal(0usize);
    let select = use_signal(SelectSlot::default());
    select.update_untracked(|slot| slot.0 = on_select.map(|f| Box::new(f) as Box<_>));

    let Timeline {
        data,
        x,
        x_field,
        cells,
        show_value,
        row_height,
        per_page,
        mappings,
        color,
        thresholds,
        min,
        max,
        style: defaults,
        x_axis,
        y_axis,
        legend,
        tooltip,
        class,
        mut lanes,
    } = options;
    let samples = matches!(cells, Cells::Samples { .. });
    let x = data.schema(|schema| {
        let x_index = schema.map_or(0, |schema| {
            x_field.and_then(|name| schema.names.iter().position(|field| field.as_ref() == name)).unwrap_or(0)
        });
        if lanes.is_empty()
            && let Some(schema) = schema
        {
            lanes = (0..schema.names.len())
                .filter(|index| *index != x_index)
                .map(|index| StateLane {
                    name: Some(schema.names[index].clone()),
                    value: Reader::Field(schema.read, index),
                    mappings: None,
                    color: None,
                    thresholds: None,
                    style: Style::default(),
                })
                .collect();
        }
        x.map(Accessor::row).or_else(|| schema.map(|schema| Accessor::Field(schema.read, x_index)))
    });
    for lane in &mut lanes {
        lane.style = lane.style.or(defaults);
    }
    let theme = crate::ui::theme();
    let palette = Palette {
        text: theme.muted_foreground,
        ink: theme.foreground,
        grid: theme.chart_grid,
        border: theme.border,
        accent: theme.primary,
    };
    let Some(x) = x.filter(|_| !lanes.is_empty()) else {
        return div()
            .class("flex size-full items-center justify-center text-sm")
            .text_color(palette.text)
            .when_some(class, Div::class)
            .child("No data")
            .into_any_element();
    };

    let x_axis = x_axis.unwrap_or_default();
    let color = color.unwrap_or(GaugeColor::Scheme(Scheme::GreenYellowRed));
    let defaults = Defaults {
        mappings,
        color: &color,
        thresholds: thresholds.as_ref(),
        min,
        max,
        merge: matches!(cells, Cells::Merged { merge: true, .. }),
        samples,
        end: x_axis.max,
    };
    let key = data
        .revision()
        .map(|revision| model::fingerprint(revision, ChartTheme::revision(), &x, &lanes, &defaults));
    if !cache.with(|cache| cache.fresh(key)) {
        let colors = ChartTheme::colors();
        cache.update_untracked(|cache| data.rows(|rows| cache.refresh(key, rows, &x, &lanes, &defaults, &colors)));
    }

    let per_page = per_page.unwrap_or(PER_PAGE).max(1);
    let (count, range) = cache.with(|cache| (cache.lanes.len(), cache.range));
    let pages = count.div_ceil(per_page).max(1);
    let current = page.get().min(pages - 1);
    let visible = current * per_page..((current + 1) * per_page).min(count);
    let mut scale_axis = x_axis.clone();
    if let Some((low, high)) = zoom.get().x {
        scale_axis.min = Some(low);
        scale_axis.max = Some(high);
    }
    let x_scale = Scale::exact(&scale_axis, Some(range));
    let tooltip = tooltip.unwrap_or_default();
    let legend = legend.unwrap_or_default();
    let tip = hover.get().filter(|_| tooltip.visible()).and_then(|hover| {
        cache.with(|cache| {
            let row = |lane: &model::Lane| {
                let segment = lane.at(hover.x)?;
                Some(TipRow {
                    lane: lane.name.clone(),
                    color: segment.color,
                    value: segment.text.clone(),
                    start: if samples { SharedString::default() } else { x_axis.text(segment.start, None) },
                    length: if samples {
                        SharedString::default()
                    } else if x_axis.time {
                        time::span(segment.end - segment.start)
                    } else {
                        crate::format::trimmed(segment.end - segment.start)
                    },
                })
            };
            let mut rows: Vec<TipRow> = if tooltip.mode == TooltipMode::Multi {
                cache.lanes.iter().filter(|lane| lane.tooltip).filter_map(row).collect()
            } else {
                cache.lanes.get(hover.lane).filter(|lane| lane.tooltip).and_then(row).into_iter().collect()
            };
            if tooltip.sort != SortOrder::None {
                rows.sort_by(|a, b| a.value.cmp(&b.value));
                if tooltip.sort == SortOrder::Descending {
                    rows.reverse();
                }
            }
            let at = if samples {
                cache
                    .lanes
                    .iter()
                    .find_map(|lane| lane.at(hover.x))
                    .map_or(hover.x, |segment| f64::midpoint(segment.start, segment.end))
            } else {
                hover.x
            };
            (!rows.is_empty()).then(|| (hover, x_axis.text(at, None), rows))
        })
    });
    let card = tip.map(|(hover, title, rows)| state_tip(tooltip, hover, title, rows));
    let legend_view = legend.visible().then(|| state_legend(cache, legend.on_right()));
    let pager = (pages > 1).then(|| pager(page, pages));
    let settings = Settings {
        x: x_scale,
        x_axis,
        y_axis: y_axis.unwrap_or_default(),
        page: visible,
        row_height: row_height.unwrap_or(ROW_HEIGHT),
        col_width: match cells {
            Cells::Samples { width } => width,
            Cells::Merged { .. } => 1.,
        },
        show_value: show_value.unwrap_or_default(),
        align: match cells {
            Cells::Merged { align, .. } => align,
            Cells::Samples { .. } => ValueAlign::Center,
        },
    };
    surface.set(Plan { settings, cache });
    let pointer = Pointer { surface, cache, hover, drag, zoom, select, multi: tooltip.mode == TooltipMode::Multi };

    div()
        .class("flex size-full min-w-0 min-h-0")
        .map(|el| if legend.on_right() { el.flex_row() } else { el.flex_col() })
        .when_some(class, Div::class)
        .child(
            div()
                .class("flex flex-col flex-1 min-w-0 min-h-0")
                .child(
                    div()
                        .class("relative flex-1 min-h-0")
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
                        .child(card),
                )
                .child(pager),
        )
        .child(legend_view)
        .into_any_element()
}
