use std::hash::{Hash, Hasher};
use std::ops::RangeInclusive;

use blenderui::{Hsla, SharedString};

use super::style::{PointShape, Show, Style};
use crate::data::Accessor;
use crate::names::Affixes;
use crate::plot::{HideFrom, LineStyle};
use crate::{Schema, Scheme};

pub(crate) enum Paint<T> {
    Fixed(Hsla),
    By(Accessor<T>, Scheme),
}

pub struct Color<T>(pub(crate) Paint<T>);

impl<T> Color<T> {
    pub fn fixed(color: impl Into<Hsla>) -> Self {
        Self(Paint::Fixed(color.into()))
    }

    pub fn by(value: impl Fn(&T) -> f64 + 'static, scheme: Scheme) -> Self {
        Self(Paint::By(Accessor::row(value), scheme))
    }
}

impl<T> From<Hsla> for Color<T> {
    fn from(color: Hsla) -> Self {
        Self::fixed(color)
    }
}

#[derive(Debug, Default)]
pub(crate) struct Fields {
    pub x: Option<SharedString>,
    pub y: Option<SharedString>,
    pub size: Option<SharedString>,
    pub color: Option<SharedString>,
}

pub struct XYSeries<T> {
    pub(crate) name: Option<SharedString>,
    pub(crate) x: Accessor<T>,
    pub(crate) y: Accessor<T>,
    pub(crate) size: Option<Accessor<T>>,
    pub(crate) color: Option<Color<T>>,
    pub(crate) style: Style,
    pub(crate) fields: Fields,
}

impl<T> XYSeries<T> {
    pub(crate) fn new(
        name: SharedString,
        x: Accessor<T>,
        y: Accessor<T>,
        color: Option<Hsla>,
        style: Style,
    ) -> Self {
        Self {
            name: Some(name),
            x,
            y,
            size: None,
            color: color.map(Color::fixed),
            style,
            fields: Fields::default(),
        }
    }

    pub(crate) fn sized(mut self, size: Accessor<T>) -> Self {
        self.size = Some(size);
        self
    }

    pub(crate) fn tinted(mut self, value: Accessor<T>, scheme: Scheme) -> Self {
        self.color = Some(Color(Paint::By(value, scheme)));
        self
    }

    pub(crate) const fn tint(&self) -> Option<(&Accessor<T>, &Scheme)> {
        match &self.color {
            Some(Color(Paint::By(value, scheme))) => Some((value, scheme)),
            _ => None,
        }
    }

    pub(crate) fn fingerprint<H: Hasher>(&self, state: &mut H) {
        let style = &self.style;
        let (smallest, largest) = style.size_range();
        (
            style.hide_from().viz,
            style.point_size().to_bits(),
            smallest.to_bits(),
            largest.to_bits(),
        )
            .hash(state);
        self.x.fingerprint(state);
        self.y.fingerprint(state);
        self.size.is_some().hash(state);
        if let Some(size) = &self.size {
            size.fingerprint(state);
        }
        if let Some((value, scheme)) = self.tint() {
            value.fingerprint(state);
            scheme.hash(state);
        }
    }
}

#[crate::component]
pub fn series<
    T: 'static,
    X: Fn(&T) -> f64 + 'static,
    Y: Fn(&T) -> f64 + 'static,
    S: Fn(&T) -> f64 + 'static,
>(
    name: Option<SharedString>,
    x: X,
    y: Y,
    size: Option<S>,
    #[into] color: Option<Color<T>>,
    show: Option<Show>,
    point_size: Option<f32>,
    size_range: Option<RangeInclusive<f32>>,
    point_shape: Option<PointShape>,
    point_stroke_width: Option<f32>,
    fill_opacity: Option<f32>,
    line_width: Option<f32>,
    line_style: Option<LineStyle>,
    hide_from: Option<HideFrom>,
) -> XYSeries<T> {
    XYSeries {
        name,
        x: Accessor::row(x),
        y: Accessor::row(y),
        size: size.map(Accessor::row),
        color,
        style: Style {
            show,
            point_size,
            size_range: size_range.map(RangeInclusive::into_inner),
            point_shape,
            point_stroke_width,
            fill_opacity,
            line_width,
            line_style,
            hide_from,
        },
        fields: Fields::default(),
    }
}

pub(crate) fn auto<T>(schema: Schema<'_, T>) -> Vec<XYSeries<T>> {
    let Schema {
        names,
        read,
        size,
        color,
    } = schema;
    let ys = || {
        (1..names.len())
            .filter(|field| Some(*field) != size && Some(*field) != color.map(|(field, _)| field))
    };
    let affixes = Affixes::of(ys().map(|y| names[y].as_ref()));
    let field = |index: usize| Accessor::Field(read, index);
    let name = |index: Option<usize>| index.map(|index| names[index].clone());
    ys().map(|y| XYSeries {
        name: Some(affixes.strip(&names[y])),
        x: field(0),
        y: field(y),
        size: size.map(field),
        color: color.map(|(index, scheme)| Color(Paint::By(field(index), scheme.clone()))),
        style: Style::default(),
        fields: Fields {
            x: name(Some(0)),
            y: name(Some(y)),
            size: name(size),
            color: name(color.map(|(index, _)| index)),
        },
    })
    .collect()
}
