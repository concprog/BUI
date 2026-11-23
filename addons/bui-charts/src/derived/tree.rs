use blenderui::element::AnyElement;
use blenderui::{Hsla, SharedString};
use crate::component;
use blenderui::hooks::use_signal;

use super::{Sketch, XySpec, categories, series, xy_line, xy_view};
use crate::derive::{Build, Table, sync};
use crate::plot::{AxisConfig, LineStyle};
use crate::xy_chart::Style as XyStyle;
use crate::{ChartTheme, Dataset, Legend, Tooltip};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Linkage {
    Single,
    Complete,
    #[default]
    Average,
    Ward,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Merge {
    pub left: usize,
    pub right: usize,
    pub height: f64,
}

fn distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b)
        .map(|(a, b)| a - b)
        .filter(|delta| delta.is_finite())
        .map(|delta| delta * delta)
        .sum()
}

pub(crate) fn cluster(points: &[&[f64]], linkage: Linkage) -> Vec<Merge> {
    let count = points.len();
    let squared = linkage == Linkage::Ward;
    let mut gaps: Vec<Vec<f64>> = points
        .iter()
        .map(|a| {
            points
                .iter()
                .map(|b| {
                    let value = distance(a, b);
                    if squared { value } else { value.sqrt() }
                })
                .collect()
        })
        .collect();
    let mut sizes = vec![1f64; count];
    let mut ids: Vec<usize> = (0..count).collect();
    let mut active = vec![true; count];
    let mut merges = Vec::with_capacity(count.saturating_sub(1));
    for step in 0..count.saturating_sub(1) {
        let mut best = (0, 0, f64::INFINITY);
        for i in (0..count).filter(|i| active[*i]) {
            for j in (i + 1..count).filter(|j| active[*j]) {
                if gaps[i][j] < best.2 {
                    best = (i, j, gaps[i][j]);
                }
            }
        }
        let (i, j, gap) = best;
        merges.push(Merge {
            left: ids[i],
            right: ids[j],
            height: if squared { gap.max(0.).sqrt() } else { gap },
        });
        for k in (0..count).filter(|k| active[*k] && *k != i && *k != j) {
            let (a, b, size) = (gaps[i][k], gaps[j][k], sizes[k]);
            let next = match linkage {
                Linkage::Single => a.min(b),
                Linkage::Complete => a.max(b),
                Linkage::Average => sizes[i].mul_add(a, sizes[j] * b) / (sizes[i] + sizes[j]),
                Linkage::Ward => {
                    ((sizes[i] + size) * a + (sizes[j] + size) * b - size * gap)
                        / (sizes[i] + sizes[j] + size)
                }
            };
            gaps[i][k] = next;
            gaps[k][i] = next;
        }
        sizes[i] += sizes[j];
        active[j] = false;
        ids[i] = count + step;
    }
    merges
}

pub(crate) struct Tree {
    pub order: Vec<usize>,
    pub spots: Vec<(f64, f64)>,
    pub groups: Vec<Option<usize>>,
}

pub(crate) fn arrange(count: usize, merges: &[Merge], clusters: usize) -> Tree {
    let total = count + merges.len();
    let above = |node: usize| node >= count && node - count + clusters > merges.len();
    let mut order = Vec::with_capacity(count);
    let mut groups = vec![None; total];
    let mut next = 0;
    let mut stack: Vec<(usize, Option<usize>)> = total
        .checked_sub(1)
        .map(|root| (root, None))
        .into_iter()
        .collect();
    while let Some((node, inherited)) = stack.pop() {
        let group = inherited.or_else(|| {
            (!above(node)).then(|| {
                next += 1;
                next - 1
            })
        });
        groups[node] = group;
        if node < count {
            order.push(node);
        } else {
            let merge = merges[node - count];
            stack.push((merge.right, group));
            stack.push((merge.left, group));
        }
    }
    let mut spots = vec![(0., 0.); total];
    for (slot, leaf) in order.iter().enumerate() {
        spots[*leaf] = (slot as f64, 0.);
    }
    for (index, merge) in merges.iter().enumerate() {
        spots[count + index] = (
            f64::midpoint(spots[merge.left].0, spots[merge.right].0),
            merge.height,
        );
    }
    Tree {
        order,
        spots,
        groups,
    }
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn dendrogram<
    T: 'static,
    D: Dataset<T>,
    N: Fn(&T) -> &str + 'static,
    F: Fn(&T) -> &[f64] + 'static,
>(
    data: D,
    name: N,
    features: F,
    linkage: Option<Linkage>,
    clusters: Option<usize>,
    horizontal: Option<bool>,
    #[into] color: Option<Hsla>,
    value_axis: Option<AxisConfig>,
    class: Option<SharedString>,
) -> AnyElement {
    let table = use_signal(Table::default());
    let method = linkage.unwrap_or_default();
    let wanted = clusters.unwrap_or(1).max(1);
    let sideways = horizontal.unwrap_or(false);
    sync(
        table,
        &data,
        (method, wanted, sideways),
        |rows| {
            let count = rows.len();
            let groups = wanted.min(count.max(1));
            let points: Vec<&[f64]> = rows.iter().map(&features).collect();
            let merges = cluster(&points, method);
            let tree = arrange(count, &merges, groups);
            let width = 2 * (groups + 1);
            let place = |(x, y): (f64, f64)| {
                if sideways {
                    (y, (count - 1) as f64 - x)
                } else {
                    (x, y)
                }
            };
            let mut sketch = Sketch::new(width);
            for (index, merge) in merges.iter().enumerate() {
                let node = count + index;
                let column = 2 * tree.groups[node].unwrap_or(groups);
                let (left, right, top) = (
                    tree.spots[merge.left],
                    tree.spots[merge.right],
                    merge.height,
                );
                sketch.trace(
                    column,
                    [left, (left.0, top), (right.0, top), right].map(place),
                );
            }
            let mut names: Vec<SharedString> = tree
                .order
                .iter()
                .map(|leaf| SharedString::from(name(&rows[*leaf]).to_owned()))
                .collect();
            if sideways {
                names.reverse();
            }
            Build {
                rows: sketch.rows,
                names,
                ..Build::default()
            }
        });
    let colors = ChartTheme::colors();
    let names = table.with(|table| table.names.clone());
    let groups = wanted.min(names.len().max(1));
    let mut list: Vec<_> = (0..groups)
        .map(|group| {
            let tint = if groups == 1 {
                color.unwrap_or_else(|| colors.series(0))
            } else {
                colors.series(group)
            };
            series(
                format!("Cluster {}", group + 1),
                2 * group,
                2 * group + 1,
                Some(tint),
                xy_line(1.5, LineStyle::Solid),
            )
        })
        .collect();
    list.push(series(
        "Links",
        2 * groups,
        2 * groups + 1,
        Some(colors.neutral),
        xy_line(1.5, LineStyle::Solid),
    ));
    let height = value_axis.unwrap_or_else(|| AxisConfig::new().min(0.).label("Distance"));
    let leaves = categories(&names);
    let (x_axis, y_axis) = if sideways {
        (height, leaves)
    } else {
        (leaves, height)
    };
    xy_view(
        table,
        XySpec {
            series: list,
            style: XyStyle::default(),
            x_axis,
            y_axis,
            legend: Some(Legend::hidden()),
            tooltip: Some(Tooltip::hidden()),
            class,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closest_points_merge_first() {
        let points: [&[f64]; 4] = [&[0.], &[1.], &[10.], &[11.5]];
        let merges = cluster(&points, Linkage::Average);
        assert_eq!((merges[0].left, merges[0].right), (0, 1));
        assert!((merges[1].height - 1.5).abs() < 1e-9);
        assert!((merges[2].height - 10.25).abs() < 1e-9);
        let tree = arrange(4, &merges, 2);
        assert_eq!(tree.order, [0, 1, 2, 3]);
        assert_eq!(tree.groups[0], Some(0));
        assert_eq!(tree.groups[3], Some(1));
        assert_eq!(tree.groups[6], None);
    }
}
