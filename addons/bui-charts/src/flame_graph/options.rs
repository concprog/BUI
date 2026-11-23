use blenderui::{App, SharedString, Window};


use crate::format::{bytes, nanoseconds, short};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SampleUnit {
    #[default]
    Count,
    Nanoseconds,
    Bytes,
}

impl SampleUnit {
    pub(crate) const fn title(self) -> &'static str {
        match self {
            Self::Count => "Count",
            Self::Nanoseconds => "Time",
            Self::Bytes => "RAM",
        }
    }

    pub(crate) fn text(self, value: f64) -> SharedString {
        match self {
            Self::Count => short(value),
            Self::Nanoseconds => nanoseconds(value),
            Self::Bytes => bytes(value),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FlameColors {
    ByPackage,
    ByValue,
    Diff,
    DiffColorBlind,
}

impl FlameColors {
    pub(crate) const fn default_for(diff: bool) -> Self {
        if diff { Self::Diff } else { Self::ByPackage }
    }

    pub(crate) const fn is_diff(self) -> bool {
        matches!(self, Self::Diff | Self::DiffColorBlind)
    }

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::ByPackage => "By package name",
            Self::ByValue => "By value",
            Self::Diff => "Default (green to red)",
            Self::DiffColorBlind => "Color blind (blue to red)",
        }
    }

    pub(crate) const fn choices(diff: bool) -> [Self; 2] {
        if diff {
            [Self::Diff, Self::DiffColorBlind]
        } else {
            [Self::ByPackage, Self::ByValue]
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum LabelAlign {
    #[default]
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Pane {
    TopTable,
    FlameGraph,
    CallTree,
}

impl Pane {
    pub(crate) const ALL: [Self; 3] = [Self::TopTable, Self::FlameGraph, Self::CallTree];

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::TopTable => "Top Table",
            Self::FlameGraph => "Flame Graph",
            Self::CallTree => "Call Tree",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PaneLayout {
    Single(Pane),
    Split(Pane, Pane),
}

impl Default for PaneLayout {
    fn default() -> Self {
        Self::Split(Pane::TopTable, Pane::FlameGraph)
    }
}

impl PaneLayout {
    pub(crate) const fn panes(self) -> (Pane, Option<Pane>) {
        match self {
            Self::Single(pane) => (pane, None),
            Self::Split(left, right) => (left, Some(right)),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Selected<'a> {
    pub label: &'a str,
    pub total: f64,
    pub self_value: f64,
    pub level: u32,
}

#[derive(Clone, Copy)]
pub struct FrameAction {
    pub label: &'static str,
    pub run: fn(&Selected<'_>, &mut Window, &mut App),
}
