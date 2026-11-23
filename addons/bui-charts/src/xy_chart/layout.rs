use crate::geom::{Bounds, Point, Size, point, size};
use crate::surface::Arrange;

use super::model::{Columns, Model};
use crate::plot::{
    AxisPlacement, LABEL_HEIGHT, Labels, PADDING, X_TICK_SPACING, Y_TICK_SPACING, gutter,
};

const CELL: f32 = 32.;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Hit {
    pub series: usize,
    pub index: usize,
    pub at: Point,
    pub area: Size,
}

pub(crate) struct Layout {
    pub model: Model,
    pub bounds: Bounds,
    pub plot: Bounds,
    pub x_labels: Labels,
    pub y_labels: Labels,
    grid: Option<HitGrid>,
}

impl Layout {
    pub(crate) fn new(model: Model, bounds: Bounds) -> Self {
        let (x_axis, y_axis) = (&model.x_axis, &model.y_axis);
        let x_on_top = x_axis.placement == AxisPlacement::Top;
        let y_on_right = y_axis.placement == AxisPlacement::Right;
        let x_gutter = gutter(x_axis);
        let y_title = if y_axis.visible() && y_axis.label.is_some() {
            LABEL_HEIGHT
        } else {
            0.
        };

        let mut top = PADDING + y_title + if x_on_top { x_gutter } else { 0. };
        let bottom = PADDING + if x_on_top { 0. } else { x_gutter };
        let mut height = (bounds.size.height - top - bottom).max(0.);
        let mut y_labels = Labels::new(&model.y, y_axis, height, Y_TICK_SPACING, |fraction| {
            top + height * (1. - fraction)
        });

        let y_gutter = y_labels.width(y_axis);
        let mut left = PADDING + if y_on_right { 0. } else { y_gutter };
        let right = PADDING + if y_on_right { y_gutter } else { 0. };
        let mut width = (bounds.size.width - left - right).max(0.);
        if let Some(ratio) = model.equal {
            let (x_span, y_span) = (span(&model.x), span(&model.y) / ratio);
            if x_span > 0. && y_span > 0. && width > 0. && height > 0. {
                let unit = (x_span / f64::from(width)).max(y_span / f64::from(height));
                let (fitted_width, fitted_height) =
                    ((x_span / unit) as f32, (y_span / unit) as f32);
                left += (width - fitted_width) / 2.;
                top += (height - fitted_height) / 2.;
                (width, height) = (fitted_width, fitted_height);
                y_labels = Labels::new(&model.y, y_axis, height, Y_TICK_SPACING, |fraction| {
                    top + height * (1. - fraction)
                });
            }
        }
        let x_labels = Labels::new(&model.x, x_axis, width, X_TICK_SPACING, |fraction| {
            left + width * fraction
        });

        Self {
            plot: Bounds::new(
                bounds.origin + point(left, top),
                size(width, height),
            ),
            bounds,
            x_labels,
            y_labels,
            model,
            grid: None,
        }
    }

    pub(crate) fn position(&self, x: f64, y: f64) -> Point {
        point(
            self.plot.left() + self.plot.size.width * self.model.x.fraction(x),
            self.plot.bottom() - self.plot.size.height * self.model.y.fraction(y),
        )
    }

    pub(crate) fn shows(&self, at: Point, diameter: f32) -> bool {
        let margin = diameter / 2.;
        at.x >= self.plot.left() - margin
            && at.x <= self.plot.right() + margin
            && at.y >= self.plot.top() - margin
            && at.y <= self.plot.bottom() + margin
    }

    pub(crate) fn points<'a>(
        &'a self,
        data: &'a Columns,
    ) -> impl Iterator<Item = (usize, Point, f32)> + 'a {
        data.xs
            .iter()
            .zip(&data.ys)
            .enumerate()
            .filter_map(move |(index, (x, y))| {
                let (at, diameter) = (self.position(*x, *y), data.diameter(index));
                self.shows(at, diameter).then_some((index, at, diameter))
            })
    }

    pub(crate) fn hit(&mut self, cursor: Point, columns: &[Columns]) -> Option<Hit> {
        let grid = self
            .grid
            .take()
            .unwrap_or_else(|| HitGrid::new(self, columns));
        let found = grid
            .candidates(cursor)
            .iter()
            .filter_map(|&(series, index)| {
                let (series, index) = (series as usize, index as usize);
                let plotted = &self.model.series[series];
                let data = &columns[plotted.index];
                let at = self.position(data.xs[index], data.ys[index]);
                let radius =
                    f32::midpoint(data.diameter(index), plotted.style.point_stroke_width());
                let offset = at - cursor;
                let distance = offset.x.hypot(offset.y);
                (distance <= radius).then_some((plotted.index, index, at, distance))
            })
            .min_by(|a, b| a.3.total_cmp(&b.3));
        self.grid = Some(grid);
        let (series, index, at, _) = found?;
        Some(Hit {
            series,
            index,
            at: at - self.bounds.origin,
            area: self.bounds.size,
        })
    }
}

struct HitGrid {
    origin: Point,
    columns: usize,
    rows: usize,
    starts: Vec<u32>,
    entries: Vec<(u32, u32)>,
}

impl HitGrid {
    fn new(layout: &Layout, columns: &[Columns]) -> Self {
        let size = layout.bounds.size;
        let cells = |length: f32| (length / CELL).ceil().max(1.) as usize;
        let mut grid = Self {
            origin: layout.bounds.origin,
            columns: cells(size.width),
            rows: cells(size.height),
            starts: Vec::new(),
            entries: Vec::new(),
        };
        grid.starts = vec![0; grid.columns * grid.rows + 1];
        grid.visit(layout, columns, |grid, cell, _| grid.starts[cell] += 1);
        let mut total = 0;
        for start in &mut grid.starts {
            total += *start;
            *start = total;
        }
        grid.entries = vec![(0, 0); total as usize];
        grid.visit(layout, columns, |grid, cell, entry| {
            grid.starts[cell] -= 1;
            grid.entries[grid.starts[cell] as usize] = entry;
        });
        grid
    }

    fn visit(
        &mut self,
        layout: &Layout,
        columns: &[Columns],
        mut each: impl FnMut(&mut Self, usize, (u32, u32)),
    ) {
        for (series, plotted) in layout.model.series.iter().enumerate() {
            if plotted.hidden || !plotted.style.show().points() {
                continue;
            }
            let stroke = plotted.style.point_stroke_width();
            for (index, at, diameter) in layout.points(&columns[plotted.index]) {
                let radius = f32::midpoint(diameter, stroke);
                let (from, to) = (
                    self.cell(at - point(radius, radius)),
                    self.cell(at + point(radius, radius)),
                );
                for row in from.1..=to.1 {
                    for column in from.0..=to.0 {
                        each(
                            self,
                            row * self.columns + column,
                            (series as u32, index as u32),
                        );
                    }
                }
            }
        }
    }

    fn cell(&self, at: Point) -> (usize, usize) {
        let offset = at - self.origin;
        let clamp = |value: f32, count: usize| {
            ((value / CELL).floor().max(0.) as usize).min(count - 1)
        };
        (clamp(offset.x, self.columns), clamp(offset.y, self.rows))
    }

    fn candidates(&self, at: Point) -> &[(u32, u32)] {
        let (column, row) = self.cell(at);
        let cell = row * self.columns + column;
        &self.entries[self.starts[cell] as usize..self.starts[cell + 1] as usize]
    }
}

fn span(scale: &crate::plot::Scale) -> f64 {
    let (min, max) = scale.range();
    max - min
}

impl Arrange for Layout {
    type Model = Model;

    fn arrange(model: Model, bounds: Bounds) -> Self {
        Self::new(model, bounds)
    }

    fn into_model(self) -> Model {
        self.model
    }

    fn bounds(&self) -> Bounds {
        self.bounds
    }
}
