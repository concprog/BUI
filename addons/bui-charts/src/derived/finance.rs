use blenderui::{AnyElement, Hsla, SharedString};
use crate::component;
use blenderui::hooks::use_signal;

use super::{Sketch, TrendSpec, XySpec, field, lines, series, trend_view, xy_line, xy_locked};
use crate::candlestick::{CandleMode, CandleStyle, Candles, ColorStrategy, VOLUME_ALPHA, part};
use crate::derive::{Build, Table, column, row, sync};
use crate::plot::{AxisConfig, LineStyle, Tooltip};
use crate::trend::{Defaults, Interpolation, Part, Style, TrendField};
use crate::xy_chart::Style as XyStyle;
use crate::{ChartTheme, Dataset, Legend};

fn look(
    mode: CandleMode,
    style: Option<CandleStyle>,
    up: Option<Hsla>,
    down: Option<Hsla>,
) -> Candles {
    let colors = ChartTheme::colors();
    Candles {
        mode,
        style: style.unwrap_or_default(),
        strategy: ColorStrategy::OpenClose,
        up: up.unwrap_or(colors.good),
        down: down.unwrap_or(colors.bad),
        flat: colors.neutral,
        volume_alpha: VOLUME_ALPHA,
    }
}

fn parts(look: &Candles, volume: bool) -> Vec<TrendField<crate::derive::Row>> {
    let mut parts = vec![
        (Part::Open, 1),
        (Part::High, 2),
        (Part::Low, 3),
        (Part::Close, 4),
    ];
    if volume {
        parts.push((Part::Volume, 5));
    }
    parts
        .into_iter()
        .filter_map(|(kind, index)| part(kind, column(index), look.mode, look.flat))
        .collect()
}

#[component]
pub fn heikin_ashi<
    T: 'static,
    D: Dataset<T>,
    X: Fn(&T) -> f64 + 'static,
    O: Fn(&T) -> f64 + 'static,
    H: Fn(&T) -> f64 + 'static,
    L: Fn(&T) -> f64 + 'static,
    C: Fn(&T) -> f64 + 'static,
    V: Fn(&T) -> f64 + 'static,
>(
    data: D,
    x: X,
    open: O,
    high: H,
    low: L,
    close: C,
    volume: Option<V>,
    mode: Option<CandleMode>,
    candle_style: Option<CandleStyle>,
    #[into] up: Option<Hsla>,
    #[into] down: Option<Hsla>,
    x_axis: Option<AxisConfig>,
    y_axis: Option<AxisConfig>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let table = use_signal(Table::default());
    sync(
        table,
        &data,
        volume.is_some(),
        |rows| {
            let mut previous: Option<(f64, f64)> = None;
            Build {
                rows: rows
                    .iter()
                    .map(|item| {
                        let (first, top, bottom, last) =
                            (open(item), high(item), low(item), close(item));
                        let next_close = (first + top + bottom + last) / 4.;
                        let next_open = previous.map_or_else(
                            || f64::midpoint(first, last),
                            |(before, after)| f64::midpoint(before, after),
                        );
                        previous = Some((next_open, next_close));
                        row([
                            x(item),
                            next_open,
                            top.max(next_open).max(next_close),
                            bottom.min(next_open).min(next_close),
                            next_close,
                            volume.as_ref().map_or(f64::NAN, |volume| volume(item)),
                        ])
                    })
                    .collect(),
                ..Build::default()
            }
        });
    let look = look(
        if volume.is_some() {
            mode.unwrap_or_default()
        } else {
            CandleMode::Candles
        },
        candle_style,
        up,
        down);
    let mut defaults = Defaults::new(
        x_axis.unwrap_or_else(|| AxisConfig::new().time()),
        y_axis.unwrap_or_default(),
    );
    defaults.candles = Some(look);
    trend_view(
        table,
        TrendSpec {
            fields: parts(&look, volume.is_some()),
            style: Style::default(),
            defaults,
            legend,
            tooltip: tooltip.or_else(|| Some(Tooltip::multi())),
            class,
        })
}

fn round_size(value: f64) -> f64 {
    if value <= 0. || !value.is_finite() {
        return 1.;
    }
    let power = 10f64.powf(value.log10().floor() - 1.);
    (value / power).round() * power
}

pub(crate) fn renko_bricks(closes: &[f64], size: f64) -> Vec<(f64, f64)> {
    let mut bricks: Vec<(f64, f64)> = Vec::new();
    let Some(start) = closes.iter().copied().find(|value| value.is_finite()) else {
        return bricks;
    };
    let base = (start / size).floor() * size;
    let (mut low, mut high) = (base, base + size);
    for &price in closes.iter().filter(|value| value.is_finite()) {
        while price >= high + size {
            bricks.push((high, high + size));
            low = high;
            high += size;
        }
        while price <= low - size {
            bricks.push((low, low - size));
            high = low;
            low -= size;
        }
    }
    bricks
}

#[component]
pub fn renko<T: 'static, D: Dataset<T>, C: Fn(&T) -> f64 + 'static>(
    data: D,
    close: C,
    box_size: Option<f64>,
    #[into] up: Option<Hsla>,
    #[into] down: Option<Hsla>,
    y_axis: Option<AxisConfig>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let table = use_signal(Table::default());
    sync(
        table,
        &data,
        box_size.map(f64::to_bits),
        |rows| {
            let closes: Vec<f64> = rows.iter().map(&close).collect();
            let size = box_size
                .filter(|size| *size > 0.)
                .unwrap_or_else(|| default_box(&closes));
            Build {
                rows: renko_bricks(&closes, size)
                    .into_iter()
                    .enumerate()
                    .map(|(index, (from, to))| {
                        row([index as f64, from, from.max(to), from.min(to), to, f64::NAN])
                    })
                    .collect(),
                ..Build::default()
            }
        });
    let look = look(CandleMode::Candles, None, up, down);
    let mut defaults = Defaults::new(
        AxisConfig::new().decimals(0).label("Brick"),
        y_axis.unwrap_or_default(),
    );
    defaults.candles = Some(look);
    trend_view(
        table,
        TrendSpec {
            fields: parts(&look, false),
            style: Style::default(),
            defaults,
            legend: Some(Legend::hidden()),
            tooltip: tooltip.or_else(|| Some(Tooltip::multi())),
            class,
        })
}

pub(crate) fn kagi_lines(closes: &[f64], reversal: f64) -> (Vec<f64>, Vec<bool>) {
    let mut prices = closes.iter().copied().filter(|value| value.is_finite());
    let Some(first) = prices.next() else {
        return (Vec::new(), Vec::new());
    };
    let mut levels = vec![first];
    let mut direction = 0.;
    for price in prices {
        let last = levels[levels.len() - 1];
        if direction == 0. {
            if (price - first).abs() >= reversal {
                direction = (price - first).signum();
                levels.push(price);
            }
            continue;
        }
        if (price - last) * direction > 0. {
            let end = levels.len() - 1;
            levels[end] = price;
        } else if (last - price).abs() >= reversal {
            direction = -direction;
            levels.push(price);
        }
    }
    let mut yang = vec![false; levels.len()];
    let (mut shoulder, mut waist) = (f64::NEG_INFINITY, f64::INFINITY);
    let mut state = levels.get(1).is_some_and(|second| *second > first);
    for index in 1..levels.len() {
        let (from, to) = (levels[index - 1], levels[index]);
        if to > from {
            if to > shoulder {
                state = true;
            }
            shoulder = to;
        } else {
            if to < waist {
                state = false;
            }
            waist = to;
        }
        yang[index] = state;
    }
    (levels, yang)
}

#[component]
pub fn kagi<T: 'static, D: Dataset<T>, C: Fn(&T) -> f64 + 'static>(
    data: D,
    close: C,
    reversal: Option<f64>,
    #[into] yang: Option<Hsla>,
    #[into] yin: Option<Hsla>,
    y_axis: Option<AxisConfig>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let table = use_signal(Table::default());
    sync(
        table,
        &data,
        reversal.map(f64::to_bits),
        |rows| {
            let closes: Vec<f64> = rows.iter().map(&close).collect();
            let amount = reversal
                .filter(|amount| *amount > 0.)
                .unwrap_or_else(|| round_size(super::mean(&closes).abs() * 0.02));
            let (levels, thick) = kagi_lines(&closes, amount);
            Build {
                rows: levels
                    .iter()
                    .enumerate()
                    .map(|(index, level)| {
                        let joined = thick[index] || thick.get(index + 1).copied().unwrap_or(false);
                        row([index as f64, *level, if joined { *level } else { f64::NAN }])
                    })
                    .collect(),
                ..Build::default()
            }
        });
    let colors = ChartTheme::colors();
    let step = |width: f32| Style {
        interpolation: Some(Interpolation::StepAfter),
        ..lines(width, false)
    };
    let fields = vec![
        field("Yin", 1, Some(yin.unwrap_or(colors.bad)), step(1.5)),
        field("Yang", 2, Some(yang.unwrap_or(colors.good)), step(3.5)),
    ];
    trend_view(
        table,
        TrendSpec {
            fields,
            style: Style::default(),
            defaults: Defaults::new(
                AxisConfig::new().decimals(0).label("Line"),
                y_axis.unwrap_or_default(),
            ),
            legend: Some(Legend::list()),
            tooltip,
            class,
        })
}

const GLYPH: f64 = 0.35;

fn default_box(closes: &[f64]) -> f64 {
    let finite = || closes.iter().copied().filter(|value| value.is_finite());
    let span = finite().fold(f64::NEG_INFINITY, f64::max) - finite().fold(f64::INFINITY, f64::min);
    round_size(span / 25.)
}

pub(crate) fn figure_columns(closes: &[f64], size: f64, reversal: usize) -> Vec<(bool, i64, i64)> {
    let mut boxes = closes
        .iter()
        .filter(|value| value.is_finite())
        .map(|price| (price / size).floor() as i64);
    let Some(start) = boxes.next() else {
        return Vec::new();
    };
    let turn = i64::try_from(reversal.max(1)).unwrap_or(i64::MAX);
    let mut columns: Vec<(bool, i64, i64)> = Vec::new();
    for level in boxes {
        match columns.last_mut() {
            None if level > start => columns.push((true, start, level)),
            None if level < start => columns.push((false, level, start)),
            None => {}
            Some((true, _, high)) => {
                if level > *high {
                    *high = level;
                } else if level <= *high - turn {
                    let top = *high - 1;
                    columns.push((false, level, top));
                }
            }
            Some((false, low, _)) => {
                if level < *low {
                    *low = level;
                } else if level >= *low + turn {
                    let bottom = *low + 1;
                    columns.push((true, bottom, level));
                }
            }
        }
    }
    columns
}

#[component]
pub fn point_figure<T: 'static, D: Dataset<T>, C: Fn(&T) -> f64 + 'static>(
    data: D,
    close: C,
    box_size: Option<f64>,
    reversal: Option<usize>,
    #[into] up: Option<Hsla>,
    #[into] down: Option<Hsla>,
    y_axis: Option<AxisConfig>,
    class: Option<SharedString>,
) -> AnyElement {
    let table = use_signal(Table::default());
    let step = use_signal(1.0_f64);
    let turn = reversal.unwrap_or(3);
    let mut built = None;
    sync(
        table,
        &data,
        (box_size.map(f64::to_bits), turn),
        |rows| {
            let closes: Vec<f64> = rows.iter().map(&close).collect();
            let size = box_size
                .filter(|size| *size > 0.)
                .unwrap_or_else(|| default_box(&closes));
            built = Some(size);
            let mut sketch = Sketch::new(4);
            for (index, (rising, low, high)) in
                figure_columns(&closes, size, turn).into_iter().enumerate()
            {
                let x = index as f64;
                for level in low..=high {
                    let y = (level as f64 + 0.5) * size;
                    let (dx, dy) = (GLYPH, size * GLYPH);
                    if rising {
                        sketch.trace(0, [(x - dx, y - dy), (x + dx, y + dy)]);
                        sketch.trace(0, [(x - dx, y + dy), (x + dx, y - dy)]);
                    } else {
                        sketch.ellipse(2, (x, y), (dx, dy), 20);
                    }
                }
            }
            Build {
                rows: sketch.rows,
                ..Build::default()
            }
        });
    let (first, last) = table.with(|table| {
        table
            .rows
            .iter()
            .flat_map(|row| [row[0], row[2]])
            .filter(|value| value.is_finite())
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), value| (low.min(value), high.max(value)))
    });
    if let Some(size) = built {
        step.update_untracked(|step| *step = size);
    }
    let ratio = step.get();
    let columns = AxisConfig::new().decimals(0).label("Column");
    let columns = if first <= last {
        columns.min(first - 0.2).max(last + 0.2)
    } else {
        columns
    };
    let colors = ChartTheme::colors();
    xy_locked(
        table,
        XySpec {
            series: vec![
                series(
                    "X",
                    0,
                    1,
                    Some(up.unwrap_or(colors.good)),
                    xy_line(2., LineStyle::Solid),
                ),
                series(
                    "O",
                    2,
                    3,
                    Some(down.unwrap_or(colors.bad)),
                    xy_line(2., LineStyle::Solid),
                ),
            ],
            style: XyStyle::default(),
            x_axis: columns,
            y_axis: y_axis.unwrap_or_default(),
            legend: Some(Legend::hidden()),
            tooltip: Some(Tooltip::hidden()),
            class,
        },
        ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renko_adds_a_brick_per_box() {
        let bricks = renko_bricks(&[10., 12.5, 14.2, 11.9, 9.5], 1.);
        assert_eq!(bricks[0], (11., 12.));
        assert!(bricks.iter().any(|(from, to)| to < from));
    }

    #[test]
    fn kagi_reverses_only_past_the_amount() {
        let (levels, yang) = kagi_lines(&[10., 11., 12., 11.5, 10., 13.], 1.);
        assert_eq!(levels, [10., 12., 10., 13.]);
        assert!(yang[1] && !yang[2] && yang[3]);
    }

    #[test]
    fn point_figure_turns_after_the_reversal() {
        let columns = figure_columns(&[10., 13.2, 12.4, 9.1, 9.8, 12.6], 1., 3);
        assert_eq!(columns, [(true, 10, 13), (false, 9, 12), (true, 10, 12)]);
    }
}
