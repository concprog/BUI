//! Charts for BUI's Rust binding (`blenderui`): time series, bar gauges, gauges, stats, heatmaps,
//! histograms, state timelines, candlesticks, flame graphs, traces, node graphs, XY charts, and
//! eighty more built on them. Each chart is a component for `rsx!`:
//!
//! ```ignore
//! use blenderui::prelude::*;
//! use bui_charts::*;
//!
//! #[component]
//! fn cpu() {
//!     let rows = use_signal(vec![(0., 12.), (1., 40.), (2., 31.)]);
//!     rsx! { <TimeSeries data={rows} x={|r: &(f64, f64)| r.0} class="h-64">
//!         <TrendSeries name="cpu" y={|r: &(f64, f64)| r.1} />
//!     </TimeSeries> }
//! }
//! ```

// lets `#[component]` (which names `::bui_charts::...`) expand inside this crate too
extern crate self as bui_charts;

macro_rules! with {
    ($($field:ident: $ty:ty),* $(,)?) => {$(
        #[must_use]
        pub fn $field(mut self, $field: $ty) -> Self {
            self.$field = $field.into();
            self
        }
    )*};
}

macro_rules! style {
    ($name:ident { $($field:ident: $ty:ty = $default:expr),* $(,)? }) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq)]
        #[allow(clippy::struct_field_names)]
        pub struct $name {
            $(pub $field: Option<$ty>,)*
        }

        impl $name {
            pub(crate) fn or(self, fallback: Self) -> Self {
                Self { $($field: self.$field.or(fallback.$field),)* }
            }

            $(pub(crate) fn $field(&self) -> $ty {
                self.$field.unwrap_or($default)
            })*

            #[allow(dead_code)]
            pub(crate) fn fingerprint<H: std::hash::Hasher>(&self, state: &mut H) {
                $(crate::bits::Bits::bits(&self.$field(), state);)*
            }
        }
    };
}

pub mod bar_gauge;
mod bits;
pub mod calc;
pub mod candlestick;
mod color;
pub mod data;
mod derive;
pub mod derived;
pub mod flame_graph;
mod format;
pub mod gauge;
mod geom;
pub mod heatmap;
pub mod histogram;
mod icon;
mod kit;
pub mod legend;
pub mod mappings;
mod names;
pub mod node_graph;
mod path;
pub mod plot;
#[doc(hidden)]
#[path = "private.rs"]
pub mod __private;
mod single_stat;
pub mod stat;
pub mod state_timeline;
pub mod status_history;
mod surface;
mod text;
pub mod theme;
pub mod thresholds;
mod time;
pub mod traces;
pub mod trend;
mod ui;
mod widgets;
pub mod xy_chart;

/// Chart components: generic over their rows, with typed children (see `bui-charts-macros`).
pub(crate) use bui_charts_macros::component;

pub use bar_gauge::*;
pub use calc::{Calc, CalcSet};
pub use candlestick::*;
pub use data::{Dataset, Frame, Revision, Schema};
pub use derived::category::*;
pub use derived::circular::*;
pub use derived::diagram::*;
pub use derived::distribution::*;
pub use derived::finance::*;
pub use derived::flow::*;
pub use derived::geo::*;
pub use derived::glyphs::*;
pub use derived::hybrid::*;
pub use derived::matrix::*;
pub use derived::profile::*;
pub use derived::quality::*;
pub use derived::science::*;
pub use derived::sets::*;
pub use derived::ternary::*;
pub use derived::thermo::*;
pub use derived::tiles::*;
pub use derived::tree::*;
pub use derived::vector::*;
pub use flame_graph::*;
pub use gauge::*;
pub use heatmap::*;
pub use histogram::*;
pub use legend::{Legend, LegendMode, LegendPlacement, LegendSort};
pub use mappings::{Special, StateValue, ValueMappings};
pub use node_graph::*;
pub use plot::{
    AxisColorMode, AxisConfig, AxisPlacement, HideFrom, LineStyle, ScaleKind, SelectedRange, SelectionMode,
    SortOrder, Tooltip, TooltipMode, VisibilityMode,
};
pub use stat::*;
pub use state_timeline::*;
pub use status_history::*;
pub use theme::{ChartColors, ChartTheme, Colors, Scheme};
pub use thresholds::{Thresholds, ThresholdsMode};
pub use traces::*;
pub use trend::*;
pub use xy_chart::*;
