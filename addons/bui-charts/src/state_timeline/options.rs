use std::hash::{Hash, Hasher};

use blenderui::SharedString;

use crate::Thresholds;
use crate::bar_gauge::GaugeColor;
use crate::mappings::{StateValue, ValueMappings};
use crate::plot::HideFrom;
use crate::trend::NullSpan;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ValueAlign {
    #[default]
    Left,
    Center,
    Right,
}

style!(Style {
    fill_opacity: f32 = 0.7,
    line_width: f32 = 0.,
    span_nulls: NullSpan = NullSpan::Never,
    insert_nulls: f64 = f64::INFINITY,
    hide_from: HideFrom = HideFrom {
        legend: false,
        tooltip: false,
        viz: false
    },
});

pub(crate) enum Reader<T> {
    Row(Box<dyn Fn(&T) -> StateValue>),
    Field(fn(&T, usize) -> f64, usize),
}

impl<T> Reader<T> {
    pub(crate) fn read(&self, row: &T) -> StateValue {
        match self {
            Self::Row(read) => read(row),
            Self::Field(read, field) => {
                let value = read(row, *field);
                if value.is_nan() {
                    StateValue::Null
                } else {
                    StateValue::Number(value)
                }
            }
        }
    }

    pub(crate) fn fingerprint<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Row(_) => usize::MAX.hash(state),
            Self::Field(_, field) => field.hash(state),
        }
    }
}

pub struct StateLane<T> {
    pub(crate) name: Option<SharedString>,
    pub(crate) value: Reader<T>,
    pub(crate) mappings: Option<&'static ValueMappings>,
    pub(crate) color: Option<GaugeColor>,
    pub(crate) thresholds: Option<Thresholds>,
    pub(crate) style: Style,
}

#[crate::component]
#[allow(clippy::too_many_arguments)]
pub fn state_row<T: 'static, V: Into<StateValue>, F: Fn(&T) -> V + 'static>(
    name: Option<SharedString>,
    value: F,
    mappings: Option<&'static ValueMappings>,
    color: Option<GaugeColor>,
    thresholds: Option<Thresholds>,
    fill_opacity: Option<f32>,
    line_width: Option<f32>,
    span_nulls: Option<NullSpan>,
    insert_nulls: Option<f64>,
    hide_from: Option<HideFrom>,
) -> StateLane<T> {
    StateLane {
        name,
        value: Reader::Row(Box::new(move |row| value(row).into())),
        mappings,
        color,
        thresholds,
        style: Style {
            fill_opacity,
            line_width,
            span_nulls,
            insert_nulls,
            hide_from,
        },
    }
}
