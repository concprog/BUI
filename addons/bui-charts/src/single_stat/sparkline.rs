use blenderui::{Hsla, Painter};

use crate::geom::{Bounds, Point, point};
use crate::path::Path;

use crate::trend::shade::{self, Shade};
use crate::trend::{Interpolation, trace};

pub(crate) struct Sparkline<'a> {
    pub xs: &'a [f64],
    pub ys: &'a [f64],
    pub line: Hsla,
    pub fill: Option<Shade>,
    pub smooth: bool,
}

fn extent(values: impl Iterator<Item = f64>) -> Option<(f64, f64)> {
    values
        .filter(|value| value.is_finite())
        .fold(None, |range, value| match range {
            None => Some((value, value)),
            Some((low, high)) => Some((low.min(value), high.max(value))),
        })
}

impl Sparkline<'_> {
    pub(crate) fn paint(&self, bounds: Bounds, painter: &mut Painter) {
        let Some((left, right)) = extent(self.xs.iter().copied()) else {
            return;
        };
        let Some((low, high)) = extent(self.ys.iter().copied()) else {
            return;
        };
        let (low, high) = if high > low {
            (low, high)
        } else if low == 0. {
            (0., 1.)
        } else {
            (low - low.abs() * 0.1, high + high.abs() * 0.1)
        };
        let width = if right > left { right - left } else { 1. };
        let points: Vec<Point> = self
            .xs
            .iter()
            .zip(self.ys)
            .filter(|(x, y)| x.is_finite() && y.is_finite())
            .map(|(x, y)| {
                point(
                    bounds.left() + bounds.size.width * ((x - left) / width) as f32,
                    bounds.bottom() - bounds.size.height * ((y - low) / (high - low)) as f32,
                )
            })
            .collect();
        let (Some(first), Some(last)) = (points.first(), points.last()) else {
            return;
        };
        let interpolation = if self.smooth {
            Interpolation::Smooth
        } else {
            Interpolation::Linear
        };
        if let Some(fill) = &self.fill {
            let mut area = Path::with_capacity(points.len() + 3);
            area.move_to(point(first.x, bounds.bottom()));
            area.line_to(*first);
            trace(&mut area, &points, interpolation);
            area.line_to(point(last.x, bounds.bottom()));
            area.close();
            shade::paint(&area, fill, bounds, painter);
        }
        let mut line = Path::with_capacity(points.len());
        line.move_to(*first);
        trace(&mut line, &points, interpolation);
        line.stroke(painter, self.line, 1.);
    }
}
