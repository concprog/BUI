use blenderui::SharedString;
use blenderui::style::TextAlign;

use crate::geom::{Bounds, Point, point, size};

use super::options::JustifyMode;
use crate::text;

pub(crate) const LINE_HEIGHT: f32 = 1.2;
const MAX_TITLE_SIZE: f32 = 30.;
const VALUE_TO_PERCENT: f32 = 2.5;
const MIN_PERCENT_SIZE: f32 = 12.;
const ROW_GAP: f32 = 6.;
const BASELINE: f32 = 0.9;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    WideText,
    WideChart,
    StackedChart,
    StackedText,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct Sizes {
    pub title: Option<f32>,
    pub value: Option<f32>,
    pub percent: Option<f32>,
}

pub(crate) struct Fit<'a> {
    pub title: Option<&'a SharedString>,
    pub value: &'a SharedString,
    pub percent: Option<&'a SharedString>,
    pub chart: bool,
    pub hidden: bool,
    pub wide: bool,
    pub justify: JustifyMode,
    pub sizes: Sizes,
}

pub(crate) struct Piece {
    pub origin: Point,
    pub font: f32,
    pub align: TextAlign,
}

pub(crate) struct Placed {
    pub title: Option<Piece>,
    pub value: Option<Piece>,
    pub percent: Option<Piece>,
    pub chart: Option<Bounds>,
}

struct Fonts {
    title: f32,
    value: f32,
    percent: f32,
    chart: f32,
}

fn fit(text: &SharedString, width: f32, height: f32, max: Option<f32>) -> f32 {
    let size = text::fit(text, width, height, LINE_HEIGHT);
    max.map_or(size, |max| size.min(max))
}

fn fonts(
    kind: Kind,
    height: f32,
    inner: (f32, f32),
    fit_in: &Fit<'_>) -> Fonts {
    let (max_width, max_height) = inner;
    let title = fit_in.title.filter(|_| fit_in.sizes.title.is_none());
    let value = fit_in.sizes.value.is_none().then_some(fit_in.value);
    let percent = fit_in.percent.is_some();
    let mut fonts = Fonts {
        title: fit_in.sizes.title.unwrap_or(0.),
        value: fit_in.sizes.value.unwrap_or(0.),
        percent: 0.,
        chart: 0.,
    };
    match kind {
        Kind::WideText => {
            let share = if title.is_some() { 0.3 } else { 1. };
            let tall = if percent {
                max_height * 0.75
            } else {
                max_height
            };
            if let Some(value) = value {
                fonts.value = fit(value, max_width * share, tall, None);
            }
            if let Some(title) = title {
                fonts.title = fit(
                    title,
                    max_width * 0.6,
                    max_height,
                    Some(MAX_TITLE_SIZE))
                .min(fonts.value * 0.7);
            }
        }
        Kind::WideChart => {
            fonts.chart = height * 0.5;
            if let Some(title) = title {
                fonts.title = fit(
                    title,
                    max_width * 0.6,
                    max_height * 0.4,
                    Some(MAX_TITLE_SIZE));
            }
            if let Some(value) = value {
                fonts.value = fit(value, max_width * 0.4, max_height * 0.5, None);
            }
        }
        Kind::StackedChart => {
            let chart = height * 0.25;
            let mut title_height = 0.;
            if let Some(title) = title {
                fonts.title = fit(
                    title,
                    max_width,
                    height * 0.15,
                    Some(MAX_TITLE_SIZE));
                title_height = fonts.title * LINE_HEIGHT;
            }
            if let Some(value) = value {
                fonts.value = fit(
                    value,
                    max_width,
                    max_height - chart - title_height,
                    None);
            }
            if title.is_some() {
                fonts.title = fonts.title.min(fonts.value * 0.7);
            }
            fonts.chart = height - fonts.title * LINE_HEIGHT - fonts.value * LINE_HEIGHT;
        }
        Kind::StackedText => {
            let mut title_height = 0.;
            if let Some(title) = title {
                fonts.title = fit(
                    title,
                    max_width,
                    height * 0.15,
                    Some(MAX_TITLE_SIZE));
                title_height = fonts.title * LINE_HEIGHT;
            }
            if let Some(value) = value {
                let tall = max_height - title_height;
                let tall = if percent { tall * 0.75 } else { tall };
                fonts.value = fit(value, max_width, tall, None);
            }
            if title.is_some() {
                fonts.title = fonts.title.min(fonts.value * 0.7);
            }
        }
    }
    fonts.percent = fit_in
        .sizes
        .percent
        .unwrap_or_else(|| (fonts.value / VALUE_TO_PERCENT).max(MIN_PERCENT_SIZE));
    if fit_in.hidden
        && let Some(text) = fit_in.percent
    {
        fonts.percent = fit(text, max_width * 0.8, max_height * 0.8, None);
    }
    fonts
}

struct Item<'a> {
    text: &'a SharedString,
    font: f32,
    slot: usize,
}

pub(crate) fn place(
    bounds: Bounds,
    shown: [Option<&SharedString>; 3],
    fit_in: &Fit<'_>) -> Placed {
    let (width, height) = (bounds.size.width, bounds.size.height);
    let wide = fit_in.wide && height > 0. && width / height > 2.5;
    let kind = match (wide, fit_in.chart) {
        (true, true) if height > 50. => Kind::WideChart,
        (true, _) => Kind::WideText,
        (false, true) if height > 100. => Kind::StackedChart,
        (false, _) => Kind::StackedText,
    };
    let padding = if fit_in.hidden {
        2.
    } else if height > 100. {
        12.
    } else {
        8.
    };
    let inner = (
        (width - padding * 2.).max(0.),
        (height - padding * 2.).max(0.),
    );
    let fonts = fonts(kind, height, inner, fit_in);
    let centered =
        fit_in.justify == JustifyMode::Center || fit_in.title.is_none_or(|title| title.is_empty());
    let [title, value, percent] = shown;
    let items: Vec<Item<'_>> = [
        title.map(|text| Item {
            text,
            font: fonts.title,
            slot: 0,
        }),
        value.map(|text| Item {
            text,
            font: fonts.value,
            slot: 1,
        }),
        percent.map(|text| Item {
            text,
            font: fonts.percent,
            slot: 2,
        }),
    ]
    .into_iter()
    .flatten()
    .filter(|item| item.font > 0.)
    .collect();
    let left = bounds.left() + padding;
    let top = bounds.top() + padding;
    let mut pieces: [Option<Piece>; 3] = [None, None, None];
    let row = matches!(kind, Kind::WideText | Kind::WideChart);
    if row {
        let widths: Vec<f32> = items
            .iter()
            .map(|item| text::measure(item.text, item.font))
            .collect();
        let total: f32 = widths.iter().sum();
        let count = items.len();
        let (mut x, gap) = if centered {
            let gaps = ROW_GAP * count.saturating_sub(1) as f32;
            (left + (inner.0 - total - gaps) / 2., ROW_GAP)
        } else if count > 1 {
            (left, (inner.0 - total) / (count - 1) as f32)
        } else {
            (left, 0.)
        };
        let value_box = items
            .iter()
            .find(|item| item.slot == 1)
            .map(|item| item.font * LINE_HEIGHT);
        let row_height = items
            .iter()
            .map(|item| item.font * LINE_HEIGHT)
            .fold(0., f32::max);
        let row_top = if kind == Kind::WideText || centered {
            top + (inner.1 - row_height) / 2.
        } else {
            top
        };
        let value_top = row_top + (row_height - value_box.unwrap_or(row_height)) / 2.;
        for (item, width) in items.iter().zip(widths) {
            let box_height = item.font * LINE_HEIGHT;
            let y = if item.slot == 2 && value_box.is_some() {
                value_top
                    + (LINE_HEIGHT - 1.) / 2. * fonts.value
                    + BASELINE * (fonts.value - item.font)
            } else {
                row_top + (row_height - box_height) / 2. + item.font * (LINE_HEIGHT - 1.) / 2.
            };
            pieces[item.slot] = Some(Piece {
                origin: point(x, y),
                font: item.font,
                align: TextAlign::Left,
            });
            x += width + gap;
        }
    } else {
        let spacing = |item: &Item<'_>| {
            if item.slot == 2 && item.font > MIN_PERCENT_SIZE {
                -item.font / 4.
            } else {
                0.
            }
        };
        let total: f32 = items
            .iter()
            .map(|item| item.font * LINE_HEIGHT + spacing(item))
            .sum();
        let mut y = if kind == Kind::StackedText {
            top + (inner.1 - total) / 2.
        } else {
            top
        };
        let (x, align) = if centered {
            (left + inner.0 / 2., TextAlign::Center)
        } else {
            (left, TextAlign::Left)
        };
        for item in &items {
            y += spacing(item);
            pieces[item.slot] = Some(Piece {
                origin: point(x, y + item.font * (LINE_HEIGHT - 1.) / 2.),
                font: item.font,
                align,
            });
            y += item.font * LINE_HEIGHT;
        }
    }
    let chart =
        (fonts.chart > 0. && matches!(kind, Kind::WideChart | Kind::StackedChart)).then(|| {
            Bounds::new(
                point(bounds.left(), bounds.bottom() - fonts.chart),
                size(bounds.size.width, fonts.chart),
            )
        });
    let [title, value, percent] = pieces;
    Placed {
        title,
        value,
        percent,
        chart,
    }
}
