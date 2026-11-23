use blenderui::{Hsla, SharedString};
use regex::Regex;

use crate::format::trimmed;

#[derive(Clone, Debug, Default, PartialEq)]
pub enum StateValue {
    #[default]
    Null,
    Number(f64),
    Text(SharedString),
    Bool(bool),
}

impl StateValue {
    pub(crate) const fn number(&self) -> Option<f64> {
        match self {
            Self::Number(value) if value.is_finite() => Some(*value),
            _ => None,
        }
    }

    pub(crate) const fn is_null(&self) -> bool {
        matches!(self, Self::Null) || matches!(self, Self::Number(value) if value.is_nan())
    }

    pub(crate) fn text(&self) -> SharedString {
        match self {
            Self::Null => SharedString::default(),
            Self::Number(value) => trimmed(*value),
            Self::Text(text) => text.clone(),
            Self::Bool(value) => SharedString::Borrowed(if *value { "true" } else { "false" }),
        }
    }
}

macro_rules! state_value {
    ($($from:ty => $variant:ident($convert:expr)),* $(,)?) => {$(
        impl From<$from> for StateValue {
            fn from(value: $from) -> Self {
                Self::$variant($convert(value))
            }
        }
    )*};
}

state_value! {
    f64 => Number(std::convert::identity),
    f32 => Number(f64::from),
    i32 => Number(f64::from),
    u32 => Number(f64::from),
    bool => Bool(std::convert::identity),
    &'static str => Text(SharedString::Borrowed),
    String => Text(SharedString::from),
    SharedString => Text(std::convert::identity),
}

impl<T: Into<Self>> From<Option<T>> for StateValue {
    fn from(value: Option<T>) -> Self {
        value.map_or(Self::Null, Into::into)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Special {
    Null,
    NaN,
    NullOrNaN,
    True,
    False,
    Empty,
}

#[derive(Clone, Debug)]
enum Rule {
    Value(SharedString),
    Range(f64, f64),
    Regex(Regex),
    Special(Special),
}

#[derive(Clone, Debug)]
struct Mapping {
    rule: Rule,
    text: SharedString,
    color: Hsla,
}

#[derive(Clone, Debug, Default)]
pub struct ValueMappings(Vec<Mapping>);

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Mapped {
    pub index: usize,
    pub text: SharedString,
    pub color: Hsla,
}

impl ValueMappings {
    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    fn push(mut self, rule: Rule, text: impl Into<SharedString>, color: impl Into<Hsla>) -> Self {
        self.0.push(Mapping {
            rule,
            text: text.into(),
            color: color.into(),
        });
        self
    }

    #[must_use]
    pub fn value(
        self,
        value: impl Into<SharedString>,
        text: impl Into<SharedString>,
        color: impl Into<Hsla>,
    ) -> Self {
        self.push(Rule::Value(value.into()), text, color)
    }

    #[must_use]
    pub fn range(
        self,
        from: f64,
        to: f64,
        text: impl Into<SharedString>,
        color: impl Into<Hsla>,
    ) -> Self {
        self.push(Rule::Range(from, to), text, color)
    }

    #[must_use]
    pub fn regex(
        self,
        pattern: &str,
        text: impl Into<SharedString>,
        color: impl Into<Hsla>,
    ) -> Self {
        match Regex::new(pattern) {
            Ok(regex) => self.push(Rule::Regex(regex), text, color),
            Err(_) => self,
        }
    }

    #[must_use]
    pub fn special(
        self,
        special: Special,
        text: impl Into<SharedString>,
        color: impl Into<Hsla>,
    ) -> Self {
        self.push(Rule::Special(special), text, color)
    }

    pub(crate) const fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn entry(&self, index: usize) -> (SharedString, Hsla) {
        let mapping = &self.0[index];
        (mapping.text.clone(), mapping.color)
    }

    pub(crate) fn find(&self, value: &StateValue) -> Option<Mapped> {
        let text = value.text();
        self.0.iter().enumerate().find_map(|(index, mapping)| {
            let hit = match &mapping.rule {
                Rule::Value(wanted) => !value.is_null() && *wanted == text,
                Rule::Range(from, to) => value
                    .number()
                    .is_some_and(|number| *from <= number && number <= *to),
                Rule::Regex(regex) => !value.is_null() && regex.is_match(&text),
                Rule::Special(special) => match special {
                    Special::Null => matches!(value, StateValue::Null),
                    Special::NaN => matches!(value, StateValue::Number(number) if number.is_nan()),
                    Special::NullOrNaN => value.is_null(),
                    Special::True => matches!(value, StateValue::Bool(true)),
                    Special::False => matches!(value, StateValue::Bool(false)),
                    Special::Empty => matches!(value, StateValue::Text(text) if text.is_empty()),
                },
            };
            hit.then(|| Mapped {
                index,
                text: if mapping.text.is_empty() {
                    text.clone()
                } else {
                    mapping.text.clone()
                },
                color: mapping.color,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use blenderui::{green, red, yellow};

    use super::*;

    #[test]
    fn first_matching_mapping_wins() {
        let mappings = ValueMappings::new()
            .value("1", "Up", green())
            .range(0., 50., "Low", yellow())
            .regex("^err", "Error", red())
            .special(Special::Null, "No data", red());
        let text = |value: StateValue| mappings.find(&value).map(|mapped| mapped.text);
        assert_eq!(text(1f64.into()), Some("Up".into()));
        assert_eq!(text(20f64.into()), Some("Low".into()));
        assert_eq!(text("error: disk".into()), Some("Error".into()));
        assert_eq!(text(StateValue::Null), Some("No data".into()));
        assert_eq!(text(80f64.into()), None);
    }
}
