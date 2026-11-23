use blenderui::{Hsla, SharedString};

use crate::mappings::StateValue;

#[derive(Clone, Debug, PartialEq)]
pub struct GraphNode {
    pub(crate) id: SharedString,
    pub(crate) title: SharedString,
    pub(crate) sub_title: SharedString,
    pub(crate) main_stat: StateValue,
    pub(crate) secondary_stat: StateValue,
    pub(crate) arcs: Vec<f64>,
    pub(crate) color: Option<Hsla>,
    pub(crate) radius: Option<f32>,
    pub(crate) highlighted: bool,
    pub(crate) details: Vec<(SharedString, SharedString)>,
}

impl GraphNode {
    #[must_use]
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();
        Self {
            title: id.clone(),
            id,
            sub_title: SharedString::default(),
            main_stat: StateValue::Null,
            secondary_stat: StateValue::Null,
            arcs: Vec::new(),
            color: None,
            radius: None,
            highlighted: false,
            details: Vec::new(),
        }
    }

    #[must_use]
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self
    }

    #[must_use]
    pub fn sub_title(mut self, sub_title: impl Into<SharedString>) -> Self {
        self.sub_title = sub_title.into();
        self
    }

    #[must_use]
    pub fn main_stat(mut self, value: impl Into<StateValue>) -> Self {
        self.main_stat = value.into();
        self
    }

    #[must_use]
    pub fn secondary_stat(mut self, value: impl Into<StateValue>) -> Self {
        self.secondary_stat = value.into();
        self
    }

    #[must_use]
    pub fn arcs(mut self, arcs: impl Into<Vec<f64>>) -> Self {
        self.arcs = arcs.into();
        self
    }

    #[must_use]
    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }

    #[must_use]
    pub const fn radius(mut self, radius: f32) -> Self {
        self.radius = Some(radius);
        self
    }

    #[must_use]
    pub const fn highlighted(mut self, highlighted: bool) -> Self {
        self.highlighted = highlighted;
        self
    }

    #[must_use]
    pub fn detail(mut self, name: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
        self.details.push((name.into(), value.into()));
        self
    }

    #[must_use]
    pub const fn id(&self) -> &SharedString {
        &self.id
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphEdge {
    pub(crate) id: SharedString,
    pub(crate) source: SharedString,
    pub(crate) target: SharedString,
    pub(crate) main_stat: StateValue,
    pub(crate) secondary_stat: StateValue,
    pub(crate) thickness: f32,
    pub(crate) color: Option<Hsla>,
    pub(crate) dashed: bool,
    pub(crate) highlighted: bool,
    pub(crate) details: Vec<(SharedString, SharedString)>,
}

impl GraphEdge {
    #[must_use]
    pub fn new(source: impl Into<SharedString>, target: impl Into<SharedString>) -> Self {
        let (source, target) = (source.into(), target.into());
        Self {
            id: format!("{source}→{target}").into(),
            source,
            target,
            main_stat: StateValue::Null,
            secondary_stat: StateValue::Null,
            thickness: 1.,
            color: None,
            dashed: false,
            highlighted: false,
            details: Vec::new(),
        }
    }

    #[must_use]
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = id.into();
        self
    }

    #[must_use]
    pub fn main_stat(mut self, value: impl Into<StateValue>) -> Self {
        self.main_stat = value.into();
        self
    }

    #[must_use]
    pub fn secondary_stat(mut self, value: impl Into<StateValue>) -> Self {
        self.secondary_stat = value.into();
        self
    }

    #[must_use]
    pub const fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        self
    }

    #[must_use]
    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }

    #[must_use]
    pub const fn dashed(mut self, dashed: bool) -> Self {
        self.dashed = dashed;
        self
    }

    #[must_use]
    pub const fn highlighted(mut self, highlighted: bool) -> Self {
        self.highlighted = highlighted;
        self
    }

    #[must_use]
    pub fn detail(mut self, name: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
        self.details.push((name.into(), value.into()));
        self
    }

    #[must_use]
    pub const fn source(&self) -> &SharedString {
        &self.source
    }

    #[must_use]
    pub const fn target(&self) -> &SharedString {
        &self.target
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArcSection {
    pub(crate) name: SharedString,
    pub(crate) color: Hsla,
}

impl ArcSection {
    #[must_use]
    pub fn new(name: impl Into<SharedString>, color: impl Into<Hsla>) -> Self {
        Self {
            name: name.into(),
            color: color.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum NodeLayout {
    #[default]
    Layered,
    Force,
    Grid,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ZoomMode {
    #[default]
    Cooperative,
    Greedy,
}

type Format = fn(f64) -> blenderui::SharedString;

#[derive(Clone, Copy, Default)]
pub struct StatFormats {
    pub(crate) node_main: Option<Format>,
    pub(crate) node_secondary: Option<Format>,
    pub(crate) edge_main: Option<Format>,
    pub(crate) edge_secondary: Option<Format>,
}

impl StatFormats {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            node_main: None,
            node_secondary: None,
            edge_main: None,
            edge_secondary: None,
        }
    }

    #[must_use]
    pub const fn node_main(mut self, format: Format) -> Self {
        self.node_main = Some(format);
        self
    }

    #[must_use]
    pub const fn node_secondary(mut self, format: Format) -> Self {
        self.node_secondary = Some(format);
        self
    }

    #[must_use]
    pub const fn edge_main(mut self, format: Format) -> Self {
        self.edge_main = Some(format);
        self
    }

    #[must_use]
    pub const fn edge_secondary(mut self, format: Format) -> Self {
        self.edge_secondary = Some(format);
        self
    }
}
