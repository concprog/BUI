use crate::geom::{Bounds, point, size};

use crate::bar_gauge::BarOrientation;

fn child_size(width: f32, height: f32, count: f32) -> f32 {
    let parts = (count * width / height).sqrt().ceil();
    if (parts * height / width).floor() * parts < count {
        height / (parts * height / width).ceil()
    } else {
        width / parts
    }
}

struct Grid {
    width: f32,
    height: f32,
    last_width: f32,
    columns: usize,
}

fn grid(width: f32, height: f32, spacing: f32, count: usize) -> Grid {
    let n = count as f32;
    let square = child_size(width, height, n).max(child_size(height, width, n));
    let columns = ((width / square).floor() as usize).max(1);
    let rows = count.div_ceil(columns);
    let columns = count.div_ceil(rows);
    let last = (columns - (columns * rows - count)) as f32;
    let share = |total: f32, parts: f32| total / parts - spacing + spacing / parts;
    Grid {
        width: share(width, columns as f32),
        height: share(height, rows as f32),
        last_width: share(width, last),
        columns,
    }
}

pub(crate) fn cells(
    bounds: Bounds,
    count: usize,
    orientation: BarOrientation,
    spacing: f32,
    minimum: (f32, f32),
) -> Vec<Bounds> {
    let (width, height) = (bounds.size.width, bounds.size.height);
    if count == 0 || width <= 0. || height <= 0. {
        return Vec::new();
    }
    let at = |x: f32, y: f32, w: f32, h: f32| {
        Bounds::new(
            bounds.origin + point(x, y),
            size(w.max(0.), h.max(0.)),
        )
    };
    let n = count as f32;
    match orientation {
        BarOrientation::Auto => {
            let grid = grid(width, height, spacing, count);
            let rows = count.div_ceil(grid.columns);
            (0..count)
                .map(|index| {
                    let (row, column) = (index / grid.columns, index % grid.columns);
                    let item = if row == rows - 1 {
                        grid.last_width
                    } else {
                        grid.width
                    };
                    at(
                        column as f32 * (item + spacing),
                        row as f32 * (grid.height + spacing),
                        item,
                        grid.height,
                    )
                })
                .collect()
        }
        BarOrientation::Horizontal => {
            let item = ((height + spacing) / n - spacing).max(minimum.1);
            (0..count)
                .map(|index| at(0., index as f32 * (item + spacing), width, item))
                .collect()
        }
        BarOrientation::Vertical => {
            let item = (width / n - spacing + spacing / n).max(minimum.0);
            (0..count)
                .map(|index| at(index as f32 * (item + spacing), 0., item, height))
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_matches_grafana() {
        let four = grid(400., 400., 0., 4);
        assert_eq!(four.columns, 2);
        assert!((four.width - 200.).abs() < 1e-3 && (four.height - 200.).abs() < 1e-3);
        let wide = grid(900., 100., 0., 3);
        assert_eq!(wide.columns, 3);
        let five = grid(300., 200., 0., 5);
        assert_eq!(five.columns, 3);
        assert!((five.last_width - 150.).abs() < 1e-3);
    }
}
