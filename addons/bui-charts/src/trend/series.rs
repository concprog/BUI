use std::hash::{Hash, Hasher};

use blenderui::{Hsla, SharedString};

use super::options::{
    BarAlignment, DrawStyle, GradientMode, Interpolation, NullSpan, Part, PointsMode, Stacking,
    Style, ThresholdsStyle, ValueTransform,
};
use crate::data::Accessor;
use crate::names::Affixes;
use crate::plot::{AxisConfig, HideFrom, LineStyle};
use crate::{Schema, Scheme, Thresholds};

pub struct TrendField<T> {
    pub(crate) name: Option<SharedString>,
    pub(crate) y: Accessor<T>,
    pub(crate) color: Option<Hsla>,
    pub(crate) y_axis: Option<AxisConfig>,
    pub(crate) thresholds: Option<Thresholds>,
    pub(crate) scheme: Option<Scheme>,
    pub(crate) style: Style,
    pub(crate) compare: Option<usize>,
    pub(crate) part: Option<Part>,
    pub(crate) floor: Option<Accessor<T>>,
}

impl<T> TrendField<T> {
    pub(crate) fn with_floor(mut self, floor: Accessor<T>) -> Self {
        self.floor = Some(floor);
        self
    }

    pub(crate) fn with_axis(mut self, axis: AxisConfig) -> Self {
        self.y_axis = Some(axis);
        self
    }

    pub(crate) const fn new(
        name: SharedString,
        y: Accessor<T>,
        color: Option<Hsla>,
        style: Style,
    ) -> Self {
        Self {
            name: Some(name),
            y,
            color,
            y_axis: None,
            thresholds: None,
            scheme: None,
            style,
            compare: None,
            part: None,
            floor: None,
        }
    }

    pub(crate) fn fingerprint<H: Hasher>(&self, state: &mut H) {
        let style = &self.style;
        (
            style.hide_from().viz,
            style.stacking(),
            style.transform(),
            self.compare,
            self.part,
            self.floor.is_some(),
            self.y_axis.as_ref().and_then(|axis| axis.label.as_ref()),
        )
            .hash(state);
        self.y.fingerprint(state);
    }
}

#[crate::component]
#[allow(clippy::too_many_arguments)]
pub fn trend_series<T: 'static, Y: Fn(&T) -> f64 + 'static>(
    name: Option<SharedString>,
    y: Y,
    #[into] color: Option<Hsla>,
    y_axis: Option<AxisConfig>,
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
) -> TrendField<T> {
    TrendField {
        name,
        y: Accessor::row(y),
        color,
        y_axis,
        thresholds,
        scheme,
        style: Style {
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
        },
        compare: None,
        part: None,
        floor: None,
    }
}

pub(crate) fn auto<T>(
    schema: Schema<'_, T>,
    x_field: Option<&str>,
) -> Option<(Accessor<T>, Vec<TrendField<T>>)> {
    let Schema { names, read, .. } = schema;
    let x = match x_field {
        Some(name) => names.iter().position(|field| field.as_ref() == name)?,
        None => 0,
    };
    let ys = || (0..names.len()).filter(move |field| *field != x);
    let affixes = Affixes::of(ys().map(|y| names[y].as_ref()));
    let fields = ys()
        .map(|y| TrendField {
            name: Some(affixes.strip(&names[y])),
            y: Accessor::Field(read, y),
            color: None,
            y_axis: None,
            thresholds: None,
            scheme: None,
            style: Style::default(),
            compare: None,
            part: None,
            floor: None,
        })
        .collect();
    Some((Accessor::Field(read, x), fields))
}
