mod options;

use blenderui::{AnyElement, App, Hsla, SharedString, Window};
use crate::component;

pub(crate) use options::Candles;
pub use options::{CandleMode, CandleStyle, ColorStrategy};

use crate::data::Accessor;
use crate::plot::{AxisConfig, AxisPlacement, HideFrom, SelectedRange, Tooltip};
use crate::trend::{Annotations, Chart, Extras, Part, Style, TrendField, VizOrientation, chart};
use crate::{ChartTheme, Dataset, Legend};

pub(crate) const VOLUME_ALPHA: f32 = 0.5;
const NAMES: [(Part, &str, &[&str]); 5] = [
    (Part::Open, "Open", &["open", "o"]),
    (Part::High, "High", &["high", "h", "max"]),
    (Part::Low, "Low", &["low", "l", "min"]),
    (Part::Close, "Close", &["close", "c"]),
    (Part::Volume, "Volume", &["volume", "v"]),
];

fn price_style() -> Style {
    Style {
        hide_from: Some(HideFrom {
            legend: true,
            tooltip: false,
            viz: false,
        }),
        ..Style::default()
    }
}

pub(crate) fn part<T>(
    part: Part,
    y: Accessor<T>,
    mode: CandleMode,
    color: Hsla,
) -> Option<TrendField<T>> {
    let wanted = match part {
        Part::High | Part::Low => mode.price(),
        Part::Volume => mode.volume(),
        Part::Open | Part::Close => true,
    };
    let (_, name, _) = NAMES.iter().find(|(kind, _, _)| *kind == part)?;
    wanted.then(|| {
        let mut field = TrendField::new(
            SharedString::Borrowed(name),
            y,
            Some(color),
            price_style(),
        );
        if part == Part::Volume && mode.price() {
            field.y_axis = Some(AxisConfig::new().placement(AxisPlacement::Hidden));
        }
        field.part = Some(part);
        field
    })
}

#[component]
#[allow(
    clippy::needless_pass_by_value,
    clippy::too_many_arguments,
    clippy::too_many_lines
)]
pub fn candlestick<
    T: 'static,
    D: Dataset<T>,
    X: Fn(&T) -> f64 + 'static,
    O: Fn(&T) -> f64 + 'static,
    H: Fn(&T) -> f64 + 'static,
    L: Fn(&T) -> f64 + 'static,
    C: Fn(&T) -> f64 + 'static,
    V: Fn(&T) -> f64 + 'static,
    S: Fn(SelectedRange, &mut Window, &mut App) + 'static,
>(
    data: D,
    x: Option<X>,
    x_field: Option<&'static str>,
    open: Option<O>,
    high: Option<H>,
    low: Option<L>,
    close: Option<C>,
    volume: Option<V>,
    mode: Option<CandleMode>,
    candle_style: Option<CandleStyle>,
    color_strategy: Option<ColorStrategy>,
    #[into] up: Option<Hsla>,
    #[into] down: Option<Hsla>,
    #[into] flat: Option<Hsla>,
    include_all_fields: Option<bool>,
    x_axis: Option<AxisConfig>,
    y_axis: Option<AxisConfig>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    #[into] annotations: Option<Annotations>,
    orientation: Option<VizOrientation>,
    timezones: Option<&'static [i32]>,
    keyboard: Option<bool>,
    on_select: Option<S>,
    class: Option<SharedString>,
    children: Children<TrendField<T>>,
) -> AnyElement {
    let overlays = children;
    let mode = mode.unwrap_or_default();
    let colors = ChartTheme::colors();
    let look = Candles {
        mode,
        style: candle_style.unwrap_or_default(),
        strategy: color_strategy.unwrap_or_default(),
        up: up.unwrap_or(colors.good),
        down: down.unwrap_or(colors.bad),
        flat: flat.unwrap_or(colors.neutral),
        volume_alpha: VOLUME_ALPHA,
    };
    let tint = look.flat;
    let mut fields: Vec<TrendField<T>> = [
        (Part::Open, open.map(Accessor::row)),
        (Part::High, high.map(Accessor::row)),
        (Part::Low, low.map(Accessor::row)),
        (Part::Close, close.map(Accessor::row)),
        (Part::Volume, volume.map(Accessor::row)),
    ]
    .into_iter()
    .filter_map(|(kind, y)| part(kind, y?, mode, tint))
    .collect();
    let mut x = x.map(Accessor::row);
    data.schema(|schema| {
    if let Some(schema) = schema {
        let x_index = x_field
            .and_then(|name| schema.names.iter().position(|field| field.as_ref() == name))
            .unwrap_or(0);
        if x.is_none() {
            x = Some(Accessor::Field(schema.read, x_index));
        }
        if fields.is_empty() {
            let mut used = vec![x_index];
            for (kind, _, aliases) in NAMES {
                let found = schema.names.iter().position(|name| {
                    aliases
                        .iter()
                        .any(|alias| name.as_ref().eq_ignore_ascii_case(alias))
                });
                if let Some(index) = found
                    && let Some(field) = part(kind, Accessor::Field(schema.read, index), mode, tint)
                {
                    used.push(index);
                    fields.push(field);
                }
            }
            if include_all_fields.unwrap_or(false) {
                fields.extend(
                    (0..schema.names.len())
                        .filter(|index| !used.contains(index))
                        .map(|index| {
                            TrendField::new(
                                schema.names[index].clone(),
                                Accessor::Field(schema.read, index),
                                None,
                                Style::default(),
                            )
                        }),
                );
            }
        }
    }
    });
    fields.extend(overlays);
    let zones = timezones.unwrap_or(&[]);
    let mut x_axis = x_axis.unwrap_or_else(|| AxisConfig::new().time());
    if let Some(first) = zones.first() {
        x_axis.utc_offset = *first;
    }
    let mut extras = Extras {
        annotations,
        exemplars: None,
        exemplar_color: None,
        orientation,
        keyboard,
    };
    let mut defaults = extras.defaults(x_axis, y_axis.unwrap_or_default(), None, None, zones);
    defaults.candles = Some(look);
    chart(
        Chart {
            data,
            x,
            x_field: None,
            ends: None,
            fields,
            style: Style::default(),
            defaults,
            annotations: extras.annotations,
            compare: None,
            keyboard: extras.keyboard.unwrap_or(true),
            legend,
            tooltip: tooltip.or_else(|| Some(Tooltip::multi())),
            class,
        },
        on_select)
}
