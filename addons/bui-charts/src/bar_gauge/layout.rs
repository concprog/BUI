use blenderui::SharedString;
use blenderui::style::TextAlign;

use crate::geom::{Bounds, Point, Size, point, size};

use super::options::{BarDisplay, BarOrientation, NamePlacement, Sizing};
use crate::text;

const MIN_VALUE_HEIGHT: f32 = 18.;
const MAX_VALUE_HEIGHT: f32 = 50.;
const MAX_VALUE_WIDTH: f32 = 150.;
const TITLE_LINE_HEIGHT: f32 = 1.5;
const VALUE_PADDING: f32 = 10.;
const OVERFLOW_PADDING: f32 = 15.;
const VERTICAL_TITLE_SIZE: f32 = 14.;

pub(crate) fn is_vertical(orientation: BarOrientation, area: Size) -> bool {
    match orientation {
        BarOrientation::Auto => area.width > area.height,
        BarOrientation::Vertical => true,
        BarOrientation::Horizontal => false,
    }
}

pub(crate) const fn spacing(display: BarDisplay) -> f32 {
    match display {
        BarDisplay::Lcd => 2.,
        BarDisplay::Gradient | BarDisplay::Basic => 10.,
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Slots {
    origin: Point,
    vertical: bool,
    pub size: Size,
    step: f32,
    pub overflow: bool,
}

impl Slots {
    pub(crate) fn new(
        bounds: Bounds,
        count: usize,
        vertical: bool,
        spacing: f32,
        sizing: Sizing,
    ) -> Self {
        let (min_width, min_height, max_height) = sizing.limits();
        let n = count.max(1) as f32;
        let (width, height) = (bounds.size.width, bounds.size.height);
        let size = if vertical {
            size(
                spacing.mul_add(1. / n - 1., width / n).max(min_width),
                height,
            )
        } else {
            let fitted = (height + spacing) / n - spacing;
            size(
                width,
                fitted.max(min_height).min(max_height.max(min_height)),
            )
        };
        let extent = if vertical {
            size.width
        } else {
            size.height
        };
        let total = extent.mul_add(n, spacing * (n - 1.));
        let overflow = total > if vertical { width } else { height } + 0.5;
        Self {
            origin: bounds.origin,
            vertical,
            size,
            step: extent + spacing,
            overflow,
        }
    }

    pub(crate) fn at(&self, index: usize) -> Bounds {
        let offset = self.step * index as f32;
        let origin = if self.vertical {
            point(offset, 0.)
        } else {
            point(0., offset)
        };
        Bounds::new(self.origin + origin, self.size)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Region {
    pub bounds: Bounds,
    pub font: f32,
    pub align: TextAlign,
}

impl Region {
    pub(crate) fn origin(&self, offset: Point) -> Point {
        let bounds = self.bounds;
        let x = match self.align {
            TextAlign::Left | TextAlign::Justify => bounds.left(),
            TextAlign::Center => bounds.center().x,
            TextAlign::Right => bounds.right() - VALUE_PADDING,
        };
        offset + point(x, bounds.top() + (bounds.size.height - self.font) / 2.)
    }
}

pub(crate) struct Parts {
    pub title: Option<Region>,
    pub value: Option<Region>,
    pub track: Bounds,
}

pub(crate) struct Labels {
    pub title: Option<f32>,
    pub value: Option<f32>,
    pub placement: NamePlacement,
}

pub(crate) fn parts(
    area: Size,
    vertical: bool,
    labels: &Labels,
    title: Option<&SharedString>,
    value: Option<&SharedString>,
    overflow: bool,
) -> Parts {
    let title =
        title.filter(|title| !title.is_empty() && labels.placement != NamePlacement::Hidden);
    if vertical {
        vertical_parts(area, labels, title, value)
    } else {
        horizontal_parts(area, labels, title, value, overflow)
    }
}

const fn region(x: f32, y: f32, width: f32, height: f32, font: f32, align: TextAlign) -> Region {
    Region {
        bounds: Bounds::new(
            point(x, y),
            size(width.max(0.), height.max(0.)),
        ),
        font,
        align,
    }
}

fn vertical_parts(
    area: Size,
    labels: &Labels,
    title: Option<&SharedString>,
    value: Option<&SharedString>,
) -> Parts {
    let (width, height) = (area.width, area.height);
    let title_font = labels.title.unwrap_or(VERTICAL_TITLE_SIZE);
    let title_height = title.map_or(0., |_| title_font * TITLE_LINE_HEIGHT);
    let value_height = value.map_or(0., |_| {
        labels
            .value
            .unwrap_or_else(|| (height * 0.1).clamp(MIN_VALUE_HEIGHT, MAX_VALUE_HEIGHT))
    });
    Parts {
        title: title.map(|_| {
            region(
                0.,
                height - title_height,
                width,
                title_height,
                title_font,
                TextAlign::Center,
            )
        }),
        value: value.map(|text| {
            let font = labels
                .value
                .unwrap_or_else(|| text::fit(text, width, value_height, 1.));
            region(0., 0., width, value_height, font, TextAlign::Center)
        }),
        track: region(
            0.,
            value_height,
            width,
            height - title_height - value_height,
            0.,
            TextAlign::Left,
        )
        .bounds,
    }
}

fn horizontal_parts(
    area: Size,
    labels: &Labels,
    title: Option<&SharedString>,
    value: Option<&SharedString>,
    overflow: bool,
) -> Parts {
    let (width, height) = (area.width, area.height);
    let above = (height > 40. && labels.placement == NamePlacement::Auto)
        || labels.placement == NamePlacement::Top;
    let (title_width, title_height, title_font) = match title {
        None => (0., 0., 0.),
        Some(_) if above => {
            let auto = (height * 0.45).clamp(17., MAX_VALUE_HEIGHT);
            let font = labels.title.unwrap_or_else(|| auto / TITLE_LINE_HEIGHT);
            (
                0.,
                labels.title.map_or(auto, |font| font * TITLE_LINE_HEIGHT),
                font,
            )
        }
        Some(text) => {
            let font = labels
                .title
                .unwrap_or_else(|| (height * 0.6).max(MIN_VALUE_HEIGHT) / TITLE_LINE_HEIGHT);
            let width = (text::measure(text, font) + OVERFLOW_PADDING).min(width * 0.4);
            (width, 0., font)
        }
    };
    let value_width = value.map_or(0., |text| {
        let measured = text::measure(
            text,
            labels.value.unwrap_or_else(|| title_font.max(12.))
        );
        (width * 0.2)
            .min(MAX_VALUE_WIDTH)
            .max(VALUE_PADDING.mul_add(2., measured))
    });
    let body = height - title_height;
    let padding = if overflow { OVERFLOW_PADDING } else { 0. };
    Parts {
        title: title.map(|_| {
            if above {
                region(0., 0., width, title_height, title_font, TextAlign::Left)
            } else {
                region(0., 0., title_width, height, title_font, TextAlign::Right)
            }
        }),
        value: value.map(|text| {
            let font = labels.value.unwrap_or_else(|| {
                text::fit(
                    text,
                    VALUE_PADDING.mul_add(-2., value_width),
                    body,
                    1.
                )
            });
            region(
                width - value_width - padding,
                title_height,
                value_width,
                body,
                font,
                TextAlign::Right,
            )
        }),
        track: region(
            title_width,
            title_height,
            width - value_width - title_width - padding,
            body,
            0.,
            TextAlign::Left,
        )
        .bounds,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_share_the_height_and_stay_within_limits() {
        let bounds = Bounds::new(point(0., 0.), size(400., 230.));
        let slots = Slots::new(bounds, 3, false, 10., Sizing::Auto);
        assert!((slots.size.height - 70.).abs() < 1e-3);
        assert!((slots.at(2).top() - 160.).abs() < 1e-3);
        assert!(!slots.overflow);

        let crowded = Slots::new(bounds, 40, false, 10., Sizing::Auto);
        assert!((crowded.size.height - 16.).abs() < 1e-3);
        assert!(crowded.overflow);
    }

    #[test]
    fn columns_fill_the_width() {
        let bounds = Bounds::new(point(0., 0.), size(320., 200.));
        let slots = Slots::new(bounds, 4, true, 10., Sizing::Auto);
        let right = slots.at(3).right();
        assert!((right - 320.).abs() < 1e-3);
    }
}
