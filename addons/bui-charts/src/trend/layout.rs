use crate::geom::{Bounds, Point, Size, point, size};
use crate::surface::Arrange;

use super::model::{Data, Model};
use crate::plot::{
    AxisConfig, AxisPlacement, LABEL_HEIGHT, Labels, PADDING, Scale, X_TICK_SPACING,
    Y_TICK_SPACING, gutter,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Hover {
    pub index: usize,
    pub series: Option<usize>,
    pub at: Point,
    pub area: Size,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct Geometry {
    pub plot: Bounds,
    pub area: Size,
    pub x: Option<Scale>,
    pub vertical: bool,
}

impl Geometry {
    pub(crate) fn at(&self, x: f64) -> Option<Point> {
        let fraction = self.x?.fraction(x);
        let plot = self.plot;
        Some(if self.vertical {
            point(
                plot.origin.x + PADDING,
                plot.size.height * fraction + plot.origin.y,
            )
        } else {
            point(
                plot.size.width * fraction + plot.origin.x,
                plot.origin.y + PADDING,
            )
        })
    }
}

pub(crate) struct AxisLabels {
    pub axis: usize,
    pub edge: f32,
    pub labels: Labels,
}

pub(crate) struct TimeRow {
    pub edge: f32,
    pub labels: Labels,
}

pub(crate) struct Layout {
    pub model: Model,
    pub bounds: Bounds,
    pub plot: Bounds,
    pub logical: Bounds,
    pub x_rows: Vec<TimeRow>,
    pub y_labels: Vec<AxisLabels>,
}

fn zones(model: &Model) -> impl Iterator<Item = AxisConfig> + '_ {
    let first = std::iter::once(model.x_axis.clone());
    let rest = model.zones.iter().skip(1).map(|offset| AxisConfig {
        utc_offset: *offset,
        label: None,
        ..model.x_axis.clone()
    });
    first.chain(rest)
}

pub(crate) fn unrotate(
    plot: Bounds,
    logical: Bounds,
    vertical: bool,
    at: Point,
) -> Point {
    if vertical {
        point(
            logical.left() + (at.y - plot.top()),
            logical.bottom() - (at.x - plot.left()),
        )
    } else {
        at
    }
}

impl Layout {
    pub(crate) fn new(model: Model, bounds: Bounds) -> Self {
        if model.vertical {
            Self::vertical(model, bounds)
        } else {
            Self::horizontal(model, bounds)
        }
    }

    fn horizontal(model: Model, bounds: Bounds) -> Self {
        let x_axis = &model.x_axis;
        let x_on_top = x_axis.placement == AxisPlacement::Top;
        let rows = model.zones.len().max(1);
        let row_height = gutter(&AxisConfig {
            label: None,
            ..x_axis.clone()
        });
        let x_gutter = gutter(x_axis) + row_height * (rows - 1) as f32;
        let titled = model
            .axes
            .iter()
            .any(|axis| axis.config.visible() && axis.config.label.is_some());
        let top =
            PADDING + if titled { LABEL_HEIGHT } else { 0. } + if x_on_top { x_gutter } else { 0. };
        let bottom = PADDING + if x_on_top { 0. } else { x_gutter };
        let height = (bounds.size.height - top - bottom).max(0.);

        let mut y_labels: Vec<AxisLabels> = Vec::with_capacity(model.axes.len());
        let (mut left, mut right) = (0., 0.);
        for (index, axis) in model.axes.iter().enumerate() {
            if !axis.config.visible() {
                continue;
            }
            let labels = Labels::new(
                &axis.scale,
                &axis.config,
                height,
                Y_TICK_SPACING,
                |fraction| top + height * (1. - fraction),
            );
            let width = labels.width(&axis.config);
            let side = if axis.right { &mut right } else { &mut left };
            y_labels.push(AxisLabels {
                axis: index,
                edge: *side,
                labels,
            });
            *side += width;
        }
        let (left, right) = (PADDING + left, PADDING + right);
        let width = (bounds.size.width - left - right).max(0.);
        for labels in &mut y_labels {
            labels.edge = if model.axes[labels.axis].right {
                left + width + labels.edge
            } else {
                left - labels.edge
            };
        }
        let x_rows = zones(&model)
            .enumerate()
            .map(|(row, config)| TimeRow {
                edge: if x_on_top {
                    top - row_height * (row + 1) as f32
                } else {
                    top + height + row_height * row as f32
                },
                labels: Labels::new(&model.x, &config, width, X_TICK_SPACING, |fraction| {
                    left + width * fraction
                }),
            })
            .collect();
        let plot = Bounds::new(
            bounds.origin + point(left, top),
            size(width, height),
        );
        Self {
            plot,
            logical: plot,
            bounds,
            x_rows,
            y_labels,
            model,
        }
    }

    fn vertical(model: Model, bounds: Bounds) -> Self {
        let (mut top, mut bottom) = (PADDING, PADDING);
        let mut rows: Vec<(usize, bool, f32)> = Vec::new();
        for (index, axis) in model.axes.iter().enumerate() {
            if !axis.config.visible() {
                continue;
            }
            let size = gutter(&axis.config);
            if axis.right {
                rows.push((index, true, top));
                top += size;
            } else {
                rows.push((index, false, bottom));
                bottom += size;
            }
        }
        let height = (bounds.size.height - top - bottom).max(0.);
        let mut x_rows: Vec<TimeRow> = zones(&model)
            .map(|config| TimeRow {
                edge: 0.,
                labels: Labels::new(&model.x, &config, height, Y_TICK_SPACING, |fraction| {
                    top + height * fraction
                }),
            })
            .collect();
        let mut left = PADDING;
        for row in &mut x_rows {
            left += row.labels.width(&model.x_axis);
            row.edge = left;
        }
        let width = (bounds.size.width - left - PADDING).max(0.);
        let y_labels = rows
            .into_iter()
            .map(|(index, on_top, offset)| {
                let axis = &model.axes[index];
                AxisLabels {
                    axis: index,
                    edge: if on_top {
                        offset
                    } else {
                        top + height + offset - PADDING
                    },
                    labels: Labels::new(
                        &axis.scale,
                        &axis.config,
                        width,
                        X_TICK_SPACING,
                        |fraction| left + width * fraction,
                    ),
                }
            })
            .collect();
        let plot = Bounds::new(
            bounds.origin + point(left, top),
            size(width, height),
        );
        Self {
            logical: Bounds::new(plot.origin, size(plot.size.height, plot.size.width)),
            plot,
            bounds,
            x_rows,
            y_labels,
            model,
        }
    }

    pub(crate) const fn vertical_axis(&self) -> bool {
        self.model.vertical
    }

    pub(crate) fn geometry(&self) -> Geometry {
        Geometry {
            plot: Bounds::new(self.plot.origin - self.bounds.origin, self.plot.size),
            area: self.bounds.size,
            x: Some(self.model.x),
            vertical: self.model.vertical,
        }
    }

    pub(crate) fn x(&self, value: f64) -> f32 {
        self.logical.left() + self.logical.size.width * self.model.x.fraction(value)
    }

    pub(crate) fn y(&self, axis: usize, value: f64) -> f32 {
        self.logical.bottom()
            - self.logical.size.height * self.model.axes[axis].scale.fraction(value)
    }

    pub(crate) fn screen(&self, at: Point) -> Point {
        if self.model.vertical {
            point(
                self.plot.left() + (self.logical.bottom() - at.y),
                self.plot.top() + (at.x - self.logical.left()),
            )
        } else {
            at
        }
    }

    pub(crate) fn logical_point(&self, at: Point) -> Point {
        unrotate(self.plot, self.logical, self.model.vertical, at)
    }

    pub(crate) fn screen_bounds(&self, area: Bounds) -> Bounds {
        let (a, b) = (self.screen(area.origin), self.screen(area.bottom_right()));
        Bounds::from_corners(
            point(a.x.min(b.x), a.y.min(b.y)),
            point(a.x.max(b.x), a.y.max(b.y)),
        )
    }

    pub(crate) fn nearest(&self, cursor: f32, xs: &[f64]) -> Option<usize> {
        let split = xs.partition_point(|x| self.x(*x) < cursor);
        let distance = |index: usize| {
            xs.get(index)
                .filter(|x| x.is_finite())
                .map(|x| (self.x(*x) - cursor).abs())
        };
        [split.checked_sub(1), Some(split)]
            .into_iter()
            .flatten()
            .filter_map(|index| Some((index, distance(index)?)))
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(index, _)| index)
    }

    pub(crate) fn hover(&self, cursor: Point, data: &Data, single: bool) -> Option<Hover> {
        if !self.plot.contains(cursor) {
            return None;
        }
        let logical = self.logical_point(cursor);
        let index = if data.ends.is_empty() {
            self.nearest(logical.x, &data.xs)?
        } else {
            let index = data
                .xs
                .partition_point(|x| self.x(*x) <= logical.x)
                .checked_sub(1)?;
            (logical.x < self.x(data.ends[index])).then_some(index)?
        };
        let series = single
            .then(|| {
                self.model
                    .visible()
                    .filter(|series| !series.style.hide_from().tooltip)
                    .filter_map(|series| {
                        let value = *data.lines[series.index].values.get(index)?;
                        value.is_finite().then(|| {
                            let gap = (self.y(series.axis, value) - logical.y).abs();
                            (series.index, gap)
                        })
                    })
                    .min_by(|a, b| a.1.total_cmp(&b.1))
                    .map(|(index, _)| index)
            })
            .flatten();
        Some(Hover {
            index,
            series,
            at: cursor - self.bounds.origin,
            area: self.bounds.size,
        })
    }
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
