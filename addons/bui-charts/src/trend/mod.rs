mod layout;
mod marks;
mod model;
mod options;
mod paint;
mod series;
pub(crate) mod shade;
mod tooltip;

use std::collections::BTreeSet;
use std::mem;

use blenderui::element::{AnyElement, Div, FluentBuilder, IntoElement, ParentElement};
use blenderui::hooks::use_signal;
use blenderui::html::div;
use blenderui::{App, Hsla, KeyDownEvent, MouseButton, SharedString, Styled, Window, canvas};

pub use marks::{Annotation, Annotations, VizOrientation};
pub use options::{
    BarAlignment, DrawStyle, GradientMode, Interpolation, NullSpan, PointsMode, Stacking,
    ThresholdsStyle, ValueTransform,
};
pub use series::{TrendField, TrendSeries, TrendSeriesProps, trend_series};

use crate::component;
use crate::data::Accessor;
use crate::legend::legend_panel;
use crate::plot::{
    AxisConfig, Drag, HideFrom, LineStyle, PathLayer, SelectSlot, SelectedRange, Tooltip, TooltipMode, Zoom,
};
use crate::surface::{Handle, use_surface};
use crate::{ChartTheme, Dataset, Legend, Scheme, Thresholds, time};
use layout::{Hover, Layout};
use marks::{Mark, MarkHover};
pub(crate) use model::Defaults;
use model::{Cache, Data, Model, UNSORTED};
pub(crate) use options::{Part, Style};
pub(crate) use paint::{inside, trace};
use paint::{Palette, Pointer, paint};
pub(crate) use tooltip::card;
use tooltip::{mark_tip, trend_tip};

pub(crate) struct Chart<T, D> {
    pub data: D,
    pub x: Option<Accessor<T>>,
    pub x_field: Option<&'static str>,
    pub ends: Option<Accessor<T>>,
    pub fields: Vec<TrendField<T>>,
    pub style: Style,
    pub defaults: Defaults,
    pub annotations: Option<Annotations>,
    pub compare: Option<f64>,
    pub keyboard: bool,
    pub legend: Option<Legend>,
    pub tooltip: Option<Tooltip>,
    pub class: Option<SharedString>,
}

pub(crate) struct Extras {
    pub annotations: Option<Annotations>,
    pub exemplars: Option<Vec<(f64, f64)>>,
    pub exemplar_color: Option<Hsla>,
    pub orientation: Option<VizOrientation>,
    pub keyboard: Option<bool>,
}

#[allow(clippy::too_many_arguments)]
pub(crate) const fn style(
    draw_style: Option<DrawStyle>,
    interpolation: Option<Interpolation>,
    line_width: Option<f32>,
    line_style: Option<LineStyle>,
    fill_opacity: Option<f32>,
    gradient: Option<GradientMode>,
    fill_below_to: Option<&'static str>,
    show_points: Option<PointsMode>,
    point_size: Option<f32>,
    show_values: Option<bool>,
    span_nulls: Option<NullSpan>,
    insert_nulls: Option<f64>,
    bar_alignment: Option<BarAlignment>,
    bar_width_factor: Option<f32>,
    bar_max_width: Option<f32>,
    stacking: Option<Stacking>,
    transform: Option<ValueTransform>,
    thresholds_style: Option<ThresholdsStyle>,
    hide_from: Option<HideFrom>,
) -> Style {
    Style {
        draw_style,
        interpolation,
        line_width,
        line_style,
        fill_opacity,
        gradient,
        fill_below_to,
        show_points,
        point_size,
        show_values,
        span_nulls,
        insert_nulls,
        bar_alignment,
        bar_width_factor,
        bar_max_width,
        stacking,
        transform,
        thresholds_style,
        hide_from,
    }
}

impl Extras {
    pub(crate) fn defaults(
        &mut self,
        x_axis: AxisConfig,
        y_axis: AxisConfig,
        thresholds: Option<Thresholds>,
        scheme: Option<Scheme>,
        zones: &'static [i32],
    ) -> Defaults {
        Defaults {
            x_axis,
            y_axis,
            thresholds,
            scheme,
            vertical: self.orientation == Some(VizOrientation::Vertical),
            zones,
            notes: Vec::new(),
            exemplars: self.exemplars.take().unwrap_or_default(),
            exemplar_color: self.exemplar_color,
            candles: None,
        }
    }
}

#[component]
pub fn trend<
    T: 'static,
    D: Dataset<T>,
    X: Fn(&T) -> f64 + 'static,
    S: Fn(SelectedRange, &mut Window, &mut App) + 'static,
>(
    data: D,
    x: Option<X>,
    x_field: Option<&'static str>,
    x_axis: Option<AxisConfig>,
    y_axis: Option<AxisConfig>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    thresholds: Option<Thresholds>,
    scheme: Option<Scheme>,
    draw_style: Option<DrawStyle>,
    interpolation: Option<Interpolation>,
    line_width: Option<f32>,
    line_style: Option<LineStyle>,
    fill_opacity: Option<f32>,
    gradient: Option<GradientMode>,
    fill_below_to: Option<&'static str>,
    show_points: Option<PointsMode>,
    point_size: Option<f32>,
    show_values: Option<bool>,
    span_nulls: Option<NullSpan>,
    insert_nulls: Option<f64>,
    bar_alignment: Option<BarAlignment>,
    bar_width_factor: Option<f32>,
    bar_max_width: Option<f32>,
    stacking: Option<Stacking>,
    transform: Option<ValueTransform>,
    thresholds_style: Option<ThresholdsStyle>,
    hide_from: Option<HideFrom>,
    #[into] annotations: Option<Annotations>,
    exemplars: Option<Vec<(f64, f64)>>,
    #[into] exemplar_color: Option<Hsla>,
    orientation: Option<VizOrientation>,
    keyboard: Option<bool>,
    on_select: Option<S>,
    class: Option<SharedString>,
    children: Children<TrendField<T>>,
) -> AnyElement {
    let fields = children;
    let mut extras = Extras {
        annotations,
        exemplars,
        exemplar_color,
        orientation,
        keyboard,
    };
    let defaults = extras.defaults(
        x_axis.unwrap_or_default(),
        y_axis.unwrap_or_default(),
        thresholds,
        scheme,
        &[],
    );
    chart(
        Chart {
            data,
            x: x.map(Accessor::row),
            x_field,
            ends: None,
            fields,
            style: style(
                draw_style,
                interpolation,
                line_width,
                line_style,
                fill_opacity,
                gradient,
                fill_below_to,
                show_points,
                point_size,
                show_values,
                span_nulls,
                insert_nulls,
                bar_alignment,
                bar_width_factor,
                bar_max_width,
                stacking,
                transform,
                thresholds_style,
                hide_from,
            ),
            defaults,
            annotations: extras.annotations,
            compare: None,
            keyboard: extras.keyboard.unwrap_or(true),
            legend,
            tooltip,
            class,
        },
        on_select,
    )
}

#[component]
pub fn time_series<
    T: 'static,
    D: Dataset<T>,
    X: Fn(&T) -> f64 + 'static,
    S: Fn(SelectedRange, &mut Window, &mut App) + 'static,
>(
    data: D,
    x: Option<X>,
    x_field: Option<&'static str>,
    x_axis: Option<AxisConfig>,
    y_axis: Option<AxisConfig>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    thresholds: Option<Thresholds>,
    scheme: Option<Scheme>,
    draw_style: Option<DrawStyle>,
    interpolation: Option<Interpolation>,
    line_width: Option<f32>,
    line_style: Option<LineStyle>,
    fill_opacity: Option<f32>,
    gradient: Option<GradientMode>,
    fill_below_to: Option<&'static str>,
    show_points: Option<PointsMode>,
    point_size: Option<f32>,
    show_values: Option<bool>,
    span_nulls: Option<NullSpan>,
    insert_nulls: Option<f64>,
    bar_alignment: Option<BarAlignment>,
    bar_width_factor: Option<f32>,
    bar_max_width: Option<f32>,
    stacking: Option<Stacking>,
    transform: Option<ValueTransform>,
    thresholds_style: Option<ThresholdsStyle>,
    hide_from: Option<HideFrom>,
    #[into] annotations: Option<Annotations>,
    exemplars: Option<Vec<(f64, f64)>>,
    #[into] exemplar_color: Option<Hsla>,
    orientation: Option<VizOrientation>,
    timezones: Option<&'static [i32]>,
    time_compare: Option<f64>,
    keyboard: Option<bool>,
    on_select: Option<S>,
    class: Option<SharedString>,
    children: Children<TrendField<T>>,
) -> AnyElement {
    let fields = children;
    let zones = timezones.unwrap_or(&[]);
    let mut x_axis = x_axis.unwrap_or_else(|| AxisConfig::new().time());
    x_axis.time = true;
    if let Some(first) = zones.first() {
        x_axis.utc_offset = *first;
    }
    let mut extras = Extras {
        annotations,
        exemplars,
        exemplar_color,
        orientation,
        keyboard,
    };
    let defaults = extras.defaults(
        x_axis,
        y_axis.unwrap_or_default(),
        thresholds,
        scheme,
        zones,
    );
    chart(
        Chart {
            data,
            x: x.map(Accessor::row),
            x_field,
            ends: None,
            fields,
            style: style(
                draw_style,
                interpolation,
                line_width,
                line_style,
                fill_opacity,
                gradient,
                fill_below_to,
                show_points,
                point_size,
                show_values,
                span_nulls,
                insert_nulls,
                bar_alignment,
                bar_width_factor,
                bar_max_width,
                stacking,
                transform,
                thresholds_style,
                hide_from,
            ),
            defaults,
            annotations: extras.annotations,
            compare: time_compare.filter(|shift| *shift != 0.),
            keyboard: extras.keyboard.unwrap_or(true),
            legend,
            tooltip,
            class,
        },
        on_select,
    )
}

const fn missing<T>(_: &T, _: usize) -> f64 {
    f64::NAN
}

fn compared<T>(fields: &mut Vec<TrendField<T>>, shift: f64) {
    let colors = ChartTheme::colors();
    let ago = time::span(shift.abs());
    let count = fields.len();
    for index in 0..count {
        let field = &fields[index];
        if field.style.hide_from().viz {
            continue;
        }
        let name: SharedString = match &field.name {
            Some(name) => format!("{name} ({ago} ago)").into(),
            None => format!("Series {} ({ago} ago)", index + 1).into(),
        };
        let color = field.color.unwrap_or_else(|| colors.series(index));
        let style = Style {
            line_style: Some(LineStyle::DASH),
            fill_opacity: Some(0.),
            stacking: Some(Stacking::None),
            show_points: Some(PointsMode::Never),
            fill_below_to: None,
            ..field.style
        };
        let mut twin = TrendField::new(name, Accessor::Field(missing, 0), Some(color), style);
        twin.y_axis.clone_from(&field.y_axis);
        twin.compare = Some(index);
        fields.push(twin);
    }
}

#[allow(clippy::too_many_lines)]
pub(crate) fn chart<T: 'static, D: Dataset<T>, S: Fn(SelectedRange, &mut Window, &mut App) + 'static>(
    options: Chart<T, D>,
    on_select: Option<S>,
) -> AnyElement {
    let hover = use_signal(None::<Hover>);
    let hidden = use_signal(BTreeSet::<usize>::new());
    let cache = use_signal(Cache::default());
    let surface = use_surface::<Layout>();
    let drag = use_signal(None::<Drag>);
    let zoom = use_signal(Zoom::default());
    let select = use_signal(SelectSlot::default());
    let marks = use_signal(None::<MarkHover>);
    let paths = use_signal(PathLayer::default());
    select.update_untracked(|slot| slot.0 = on_select.map(|f| Box::new(f) as Box<_>));

    let Chart {
        data,
        mut x,
        x_field,
        ends,
        mut fields,
        style: defaults,
        defaults: mut chart_defaults,
        annotations,
        compare,
        keyboard,
        legend,
        tooltip,
        class,
    } = options;
    let legend = legend.unwrap_or_default();
    let tooltip = tooltip.unwrap_or_default();
    if fields.is_empty() || x.is_none() {
        let auto = data.schema(|schema| schema.and_then(|schema| series::auto(schema, x_field)));
        if let Some((auto_x, auto_fields)) = auto {
            x = x.or(Some(auto_x));
            if fields.is_empty() {
                fields = auto_fields;
            }
        }
    }
    let theme = crate::ui::theme();
    let palette = Palette {
        text: theme.muted_foreground,
        grid: theme.chart_grid,
        border: theme.border,
        cursor: theme.muted_foreground.opacity(0.5),
        background: theme.background,
    };
    let Some(x) = x.filter(|_| !fields.is_empty()) else {
        return message("No data", palette, class);
    };
    for field in &mut fields {
        field.style = field.style.or(defaults);
    }
    if let Some(shift) = compare {
        compared(&mut fields, shift);
    }
    chart_defaults.notes = annotations.map(Annotations::into_notes).unwrap_or_default();
    if let Some((low, high)) = zoom.get().x {
        chart_defaults.x_axis.min = Some(low);
        chart_defaults.x_axis.max = Some(high);
    }

    let key = data.revision().map(|revision| {
        let key = hidden
            .with(|hidden| model::fingerprint(revision, ChartTheme::revision(), &x, ends.as_ref(), &fields, hidden));
        key ^ compare.map_or(0, f64::to_bits)
    });
    if !cache.with(|cache| cache.fresh(key)) {
        let mut fresh = Data::default();
        cache.update_untracked(|cache| mem::swap(&mut cache.data, &mut fresh));
        hidden.with(|hidden| {
            data.rows(|rows| {
                model::refresh(&mut fresh, rows, &x, ends.as_ref(), &fields, hidden, compare.unwrap_or(0.));
            });
        });
        cache.update_untracked(|cache| cache.store(key, fresh));
    }
    if !cache.with(|cache| cache.data.ascending) {
        return message(UNSORTED, palette, class);
    }

    let colors = ChartTheme::colors();
    let (model, tip, items, note) = cache.with(|cache| {
        let data = &cache.data;
        let mut model = hidden.with(|hidden| Model::new(fields, data, chart_defaults, hidden, &colors));
        model.layer = model.fingerprint(key);
        let note = marks.get().and_then(|hovered| note(&model, hovered));
        let tip = hover
            .get()
            .filter(|_| tooltip.visible())
            .and_then(|hit| Some((hit, model.tooltip(data, hit.index, hit.series, tooltip)?)));
        let items = legend.visible().then(|| model.legend_items(legend, data)).filter(|items| !items.is_empty());
        (model, tip, items, note)
    });
    let count = model.count;
    surface.set(model);
    let pointer = Pointer {
        surface,
        cache,
        hover,
        drag,
        zoom,
        select,
        marks,
        paths,
        single: tooltip.mode == TooltipMode::Single,
    };
    let note_card = note.map(|(hovered, title, rows)| mark_tip(hovered, title, rows));
    let card = tip.map(|(hit, data)| trend_tip(tooltip, hit, data));
    let panel = items.map(|items| legend_panel(legend, items, hidden, count));

    div()
        .class("flex size-full min-w-0 min-h-0")
        .map(|el| if legend.on_right() { el.flex_row() } else { el.flex_col() })
        .when_some(class, Div::class)
        .child(
            div()
                .class("relative flex-1 min-w-0 min-h-0")
                .when(keyboard, |el| el.on_key_down(move |event, _, cx| step(event, pointer, cx)))
                .child(plot(pointer, palette))
                .child(card)
                .child(note_card),
        )
        .child(panel)
        .into_any_element()
}

/// The plot's canvas: it paints the layout and takes the pointer.
fn plot(pointer: Pointer, palette: Palette) -> Div {
    let surface: Handle<Layout> = pointer.surface;
    canvas(move |bounds, painter| surface.paint(bounds.into(), |layout| paint(layout, pointer, palette, painter)))
        .class("absolute inset-0 size-full")
        .on_mouse_move(move |event, _, _| paint::on_move(pointer, event))
        .on_hover(move |inside, _, _| {
            if !inside {
                paint::on_leave(pointer);
            }
        })
        .on_mouse_down(MouseButton::Left, move |event, _, _| paint::on_down(pointer, event))
        .on_mouse_up(MouseButton::Left, move |_, window, cx| paint::on_up(pointer, window, cx))
        .on_scroll_wheel(move |event, _, cx| paint::on_wheel(pointer, event, cx))
}

/// The title and lines of a hovered annotation or exemplar.
fn note(model: &Model, hovered: MarkHover) -> Option<(MarkHover, SharedString, Vec<SharedString>)> {
    let (title, rows) = match hovered.mark {
        Mark::Note(index) => {
            let note = model.notes.get(index)?;
            let when = match note.end {
                Some(end) => {
                    format!("{} – {}", model.x_axis.text(note.time, None), model.x_axis.text(end, None)).into()
                }
                None => model.x_axis.text(note.time, None),
            };
            (note.title.clone(), vec![when, note.text.clone()])
        }
        Mark::Exemplar(index) => {
            let (at, value) = *model.exemplars.get(index)?;
            let axis = &model.axes.first()?.config;
            (SharedString::Borrowed("Exemplar"), vec![model.x_axis.text(at, None), axis.text(value, None)])
        }
    };
    Some((hovered, title, rows))
}

fn message(text: &'static str, palette: Palette, class: Option<SharedString>) -> AnyElement {
    div()
        .class("flex size-full items-center justify-center text-sm")
        .text_color(palette.text)
        .when_some(class, Div::class)
        .child(text)
        .into_any_element()
}

/// Arrow keys step the hover from point to point (Shift: ten at a time), Escape clears it.
fn step(event: &KeyDownEvent, pointer: Pointer, cx: &mut App) {
    let delta: isize = match event.keystroke.key.to_ascii_lowercase().as_str() {
        "left" | "up" => -1,
        "right" | "down" => 1,
        "escape" => {
            pointer.hover.set(None);
            return;
        }
        _ => return,
    };
    let delta = if event.keystroke.modifiers.shift { delta * 10 } else { delta };
    let current = pointer.hover.get();
    let next = pointer.cache.with(|cache| {
        let data = &cache.data;
        let len = data.xs.len();
        if len == 0 {
            return None;
        }
        let index = current.map_or(0, |hover| hover.index.saturating_add_signed(delta).min(len - 1));
        let geometry = pointer.surface.with(Layout::geometry)?;
        let at = geometry.at(data.center(index))?;
        Some(Hover { index, series: current.and_then(|hover| hover.series), at, area: geometry.area })
    });
    if let Some(next) = next {
        pointer.hover.set(Some(next));
        cx.prevent_default();
    }
}
