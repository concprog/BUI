use std::{collections::BTreeMap, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type Id = Arc<str>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    pub const UNIT: Self = Self {
        x: 0.0,
        y: 0.0,
        w: 1.0,
        h: 1.0,
    };
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        w: 0.0,
        h: 0.0,
    };

    #[must_use]
    pub const fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { x, y, w, h }
    }
    #[must_use]
    pub fn right(&self) -> f64 {
        self.x + self.w
    }
    #[must_use]
    pub fn bottom(&self) -> f64 {
        self.y + self.h
    }
    #[must_use]
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && x <= self.right() && y >= self.y && y <= self.bottom()
    }
    #[must_use]
    pub const fn lo(&self, axis: Axis) -> f64 {
        match axis {
            Axis::X => self.x,
            Axis::Y => self.y,
        }
    }
    #[must_use]
    pub const fn len(&self, axis: Axis) -> f64 {
        match axis {
            Axis::X => self.w,
            Axis::Y => self.h,
        }
    }
    #[must_use]
    pub fn inset(&self, d: f64) -> Self {
        Self {
            x: self.x + d,
            y: self.y + d,
            w: (self.w - d * 2.0).max(0.0),
            h: (self.h - d * 2.0).max(0.0),
        }
    }
    #[must_use]
    pub fn lerp(&self, to: &Self, t: f64) -> Self {
        Self {
            x: self.x + (to.x - self.x) * t,
            y: self.y + (to.y - self.y) * t,
            w: self.w + (to.w - self.w) * t,
            h: self.h + (to.h - self.h) * t,
        }
    }
    #[must_use]
    pub fn approx_eq(&self, o: &Self, eps: f64) -> bool {
        (self.x - o.x).abs() < eps
            && (self.y - o.y).abs() < eps
            && (self.w - o.w).abs() < eps
            && (self.h - o.h).abs() < eps
    }
    #[must_use]
    pub fn center(&self) -> (f64, f64) {
        (self.x + self.w / 2.0, self.y + self.h / 2.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Axis {
    X,
    Y,
}

impl Axis {
    #[must_use]
    pub const fn cross(self) -> Self {
        match self {
            Self::X => Self::Y,
            Self::Y => Self::X,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Edge {
    Left,
    Right,
    Top,
    Bottom,
}

impl Edge {
    #[must_use]
    pub const fn axis(self) -> Axis {
        match self {
            Self::Left | Self::Right => Axis::X,
            Self::Top | Self::Bottom => Axis::Y,
        }
    }
    #[must_use]
    pub const fn before(self) -> bool {
        matches!(self, Self::Left | Self::Top)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SplitNode {
    pub id: Id,
    pub axis: Axis,
    pub weights: Vec<f64>,
    pub children: Vec<Node>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanelNode {
    pub id: Id,
    pub views: Vec<Id>,
    pub selected: Id,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StageNode {
    pub id: Id,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub child: Option<Box<Node>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Node {
    Split(SplitNode),
    Panel(PanelNode),
    Stage(StageNode),
}

impl Node {
    #[must_use]
    pub const fn id(&self) -> &Id {
        match self {
            Self::Split(n) => &n.id,
            Self::Panel(n) => &n.id,
            Self::Stage(n) => &n.id,
        }
    }
    #[must_use]
    pub const fn is_stage(&self) -> bool {
        matches!(self, Self::Stage(_))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FloatingLayer {
    Stage,
    Overlay,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FloatingPanel {
    pub panel: PanelNode,
    pub rect: Rect,
    pub z: i64,
    pub layer: FloatingLayer,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum RestoreTarget {
    Floating { rect: Rect, layer: FloatingLayer },
    Docked { beside: Id, edge: Edge, share: f64 },
    Tab { panel: Id },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HiddenPanel {
    pub panel: PanelNode,
    pub restore: RestoreTarget,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewRecord {
    #[serde(rename = "type")]
    pub ty: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Framing {
    pub id: Id,
    pub name: String,
    pub frame: Vec<Id>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavigationState {
    #[serde(default)]
    pub frame: Vec<Id>,
    #[serde(default)]
    pub framings: Vec<Framing>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayoutDocument {
    pub schema: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub root: Option<Node>,
    #[serde(default)]
    pub floating: Vec<FloatingPanel>,
    #[serde(default)]
    pub hidden: Vec<HiddenPanel>,
    #[serde(default)]
    pub views: BTreeMap<Id, ViewRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub navigation: Option<NavigationState>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Placement {
    Stage,
    Float,
    Side,
    Tab,
    Beside {
        beside: Id,
        edge: Edge,
        share: Option<f64>,
    },
    Into {
        panel: Id,
        index: Option<usize>,
    },
    FloatAt {
        rect: Rect,
        layer: Option<FloatingLayer>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Allow {
    pub stage: bool,
    pub side: bool,
    pub floating: bool,
}

impl Default for Allow {
    fn default() -> Self {
        Self {
            stage: true,
            side: true,
            floating: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabbarMode {
    #[default]
    Always,
    Auto,
    Never,
    Overlay,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Scaling {
    #[default]
    Interactive,
    Inert,
    Off,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GestureOwner {
    #[default]
    Content,
    Workspace,
    Exclusive,
}

#[derive(Clone, Debug, Default)]
pub struct ViewRules {
    pub placement: Option<Placement>,
    pub allow: Allow,
    pub singleton: bool,
    pub closable: Option<bool>,
    pub tabbar: TabbarMode,
    pub min_size: Option<(f64, f64)>,
    pub scaling: Scaling,
    pub gestures: GestureOwner,
}

impl ViewRules {
    #[must_use]
    pub fn is_closable(&self) -> bool {
        self.closable.unwrap_or(true)
    }
}
