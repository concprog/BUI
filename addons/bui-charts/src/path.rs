//! Paths drawn on blenderui's [`Painter`]: contours of straight segments (curves flattened as they
//! are added), filled even-odd as one polygon or stroked as polylines, solid or dashed. gpui's
//! `PathBuilder` / `paint_path`, made of what the painter draws (`polygon`, `line`, `with_clip`).

use blenderui::{Hsla, Painter};

use crate::geom::{Bounds, Point, point};
use crate::plot::LineStyle;

/// The longest straight piece a flattened curve is made of.
const CURVE_STEP: f32 = 4.;
const MAX_CURVE_STEPS: usize = 64;
/// Dash patterns longer than this are cut.
const MAX_DASHES: usize = 8;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Path {
    points: Vec<(f32, f32)>,
    /// where each contour starts in `points`
    starts: Vec<usize>,
}

impl Path {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn with_capacity(points: usize) -> Self {
        Self { points: Vec::with_capacity(points), starts: Vec::with_capacity(1) }
    }

    /// A closed polygon through `points`.
    pub(crate) fn polygon(points: &[Point]) -> Self {
        let mut path = Self::with_capacity(points.len() + 1);
        path.add_polygon(points, true);
        path
    }

    /// An open polyline through `points`.
    pub(crate) fn polyline(points: impl IntoIterator<Item = Point>) -> Self {
        let mut path = Self::new();
        for at in points {
            path.line_to(at);
        }
        path
    }

    pub(crate) const fn is_empty(&self) -> bool {
        self.points.len() < 2
    }

    fn last(&self) -> Option<Point> {
        self.points.last().map(|&(x, y)| point(x, y))
    }

    pub(crate) fn move_to(&mut self, to: Point) {
        // a contour of a lone point is only moved
        if let Some(&start) = self.starts.last()
            && start + 1 == self.points.len()
        {
            self.points[start] = to.tuple();
            return;
        }
        self.starts.push(self.points.len());
        self.points.push(to.tuple());
    }

    pub(crate) fn line_to(&mut self, to: Point) {
        if self.starts.is_empty() {
            self.starts.push(0);
        }
        self.points.push(to.tuple());
    }

    /// A cubic curve to `to` with control points `a` and `b`.
    pub(crate) fn cubic_to(&mut self, to: Point, a: Point, b: Point) {
        let from = self.last().unwrap_or(a);
        let steps = steps(from.distance(a) + a.distance(b) + b.distance(to));
        for step in 1..=steps {
            let t = step as f32 / steps as f32;
            let u = 1. - t;
            self.line_to(from * (u * u * u) + a * (3. * u * u * t) + b * (3. * u * t * t) + to * (t * t * t));
        }
    }

    /// Back to where the contour started (a stroke draws the closing edge; a fill closes anyway).
    pub(crate) fn close(&mut self) {
        if let Some(&start) = self.starts.last()
            && self.points.len() > start + 1
        {
            self.points.push(self.points[start]);
        }
    }

    pub(crate) fn add_polygon(&mut self, points: &[Point], closed: bool) {
        let Some((first, rest)) = points.split_first() else { return };
        self.move_to(*first);
        for at in rest {
            self.line_to(*at);
        }
        if closed {
            self.close();
        }
    }

    /// Every point moved by `f` (gpui's `PathBuilder::transform`).
    pub(crate) fn map(&mut self, f: impl Fn(Point) -> Point) {
        for xy in &mut self.points {
            *xy = f(point(xy.0, xy.1)).tuple();
        }
    }

    pub(crate) fn translate(&mut self, by: Point) {
        for xy in &mut self.points {
            xy.0 += by.x;
            xy.1 += by.y;
        }
    }

    fn contours(&self) -> impl Iterator<Item = &[(f32, f32)]> {
        self.starts.iter().enumerate().map(|(index, &start)| {
            let end = self.starts.get(index + 1).copied().unwrap_or(self.points.len());
            &self.points[start..end]
        })
    }

    /// Fills the contours even-odd (holes stay empty).
    pub(crate) fn fill(&self, painter: &mut Painter, color: Hsla) {
        if color.is_transparent() {
            return;
        }
        match self.starts.len() {
            0 => {}
            1 => {
                if self.points.len() >= 3 {
                    painter.polygon(&self.points, color);
                }
            }
            _ => {
                // one polygon: each contour closed, joined to the first one's start by an edge
                // walked there and back, which even-odd filling does not see
                let anchor = self.points[0];
                let mut joined = Vec::with_capacity(self.points.len() + self.starts.len() * 2);
                for contour in self.contours().filter(|contour| contour.len() >= 3) {
                    joined.extend_from_slice(contour);
                    joined.push(contour[0]);
                    joined.push(anchor);
                }
                if joined.len() >= 3 {
                    painter.polygon(&joined, color);
                }
            }
        }
    }

    /// Fills the part of the path inside `clip`.
    pub(crate) fn fill_clipped(&self, painter: &mut Painter, clip: Bounds, color: Hsla) {
        if !clip.is_empty() {
            painter.with_clip(clip.into(), |painter| self.fill(painter, color));
        }
    }

    pub(crate) fn stroke(&self, painter: &mut Painter, color: Hsla, width: f32) {
        if color.is_transparent() || width <= 0. {
            return;
        }
        let translucent = color.a < 1. && width > 2.;
        let mut outline = Vec::new();
        for contour in self.contours().filter(|contour| contour.len() >= 2) {
            if translucent {
                // the kit draws a polyline segment by segment, so where wide translucent segments
                // overlap at a joint the color doubles: such a stroke is filled as one outline
                ribbon(contour, width / 2., &mut outline);
                painter.polygon(&outline, color);
            } else {
                painter.line(contour, color, width);
            }
        }
    }

    /// A stroke in `style`: solid, or dashes / dots along each contour.
    pub(crate) fn stroke_styled(&self, painter: &mut Painter, color: Hsla, width: f32, style: LineStyle) {
        let dot;
        let pattern: &[f32] = match style {
            LineStyle::Solid => return self.stroke(painter, color, width),
            LineStyle::Dash(dashes) => &dashes[..dashes.len().min(MAX_DASHES)],
            LineStyle::Dot(gap) => {
                dot = [width.max(1.), gap];
                &dot
            }
        };
        self.stroke_dashed(painter, color, width, pattern);
    }

    /// A stroke of dashes: `pattern` alternates drawn and skipped lengths.
    pub(crate) fn stroke_dashed(&self, painter: &mut Painter, color: Hsla, width: f32, pattern: &[f32]) {
        let total: f32 = pattern.iter().sum();
        if pattern.is_empty() || total <= 0. || color.is_transparent() || width <= 0. {
            return self.stroke(painter, color, width);
        }
        let mut dash: Vec<(f32, f32)> = Vec::with_capacity(8);
        for contour in self.contours() {
            let (mut index, mut left, mut on) = (0, pattern[0], true);
            dash.clear();
            for pair in contour.windows(2) {
                let (mut from, to) = (point(pair[0].0, pair[0].1), point(pair[1].0, pair[1].1));
                let mut length = from.distance(to);
                while length > 0. {
                    let step = left.min(length);
                    let next = from + (to - from) * (step / length);
                    if on {
                        if dash.is_empty() {
                            dash.push(from.tuple());
                        }
                        dash.push(next.tuple());
                    }
                    left -= step;
                    length -= step;
                    from = next;
                    if left <= 0. {
                        if on && dash.len() >= 2 {
                            painter.line(&dash, color, width);
                        }
                        dash.clear();
                        on = !on;
                        index = (index + 1) % pattern.len();
                        left = pattern[index];
                    }
                }
            }
            if on && dash.len() >= 2 {
                painter.line(&dash, color, width);
            }
        }
    }
}

/// The outline of a polyline `half` wide on each side: down its left edge and back up its right one,
/// the corners mitred (a sharp corner's miter is limited to twice the width).
fn ribbon(line: &[(f32, f32)], half: f32, out: &mut Vec<(f32, f32)>) {
    out.clear();
    let normal = |a: (f32, f32), b: (f32, f32)| {
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let length = dx.hypot(dy).max(f32::EPSILON);
        (-dy / length, dx / length)
    };
    let count = line.len();
    let offset = |index: usize| {
        let before = (index > 0).then(|| normal(line[index - 1], line[index]));
        let after = (index + 1 < count).then(|| normal(line[index], line[index + 1]));
        match (before, after) {
            (Some(a), Some(b)) => {
                let (x, y) = (a.0 + b.0, a.1 + b.1);
                let dot = (x * a.0 + y * a.1).max(0.25);
                (x / dot, y / dot)
            }
            (Some(n), None) | (None, Some(n)) => n,
            (None, None) => (0., 0.),
        }
    };
    for (index, &(x, y)) in line.iter().enumerate() {
        let (nx, ny) = offset(index);
        out.push((nx.mul_add(half, x), ny.mul_add(half, y)));
    }
    for (index, &(x, y)) in line.iter().enumerate().rev() {
        let (nx, ny) = offset(index);
        out.push((nx.mul_add(-half, x), ny.mul_add(-half, y)));
    }
}

fn steps(length: f32) -> usize {
    ((length / CURVE_STEP).ceil() as usize).clamp(2, MAX_CURVE_STEPS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contours_split_at_moves() {
        let mut path = Path::new();
        path.move_to(point(0., 0.));
        path.line_to(point(10., 0.));
        path.line_to(point(10., 10.));
        path.close();
        path.move_to(point(20., 20.));
        path.line_to(point(30., 20.));
        let contours: Vec<_> = path.contours().map(<[(f32, f32)]>::len).collect();
        assert_eq!(contours, [4, 2]);
    }

    #[test]
    fn curves_end_where_they_are_sent() {
        let mut path = Path::new();
        path.move_to(point(0., 0.));
        path.cubic_to(point(100., 0.), point(30., 50.), point(70., 50.));
        assert_eq!(path.last(), Some(point(100., 0.)));
        assert!(path.points.len() > 10);
    }
}
