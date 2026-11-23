//! The gallery's table of charts and examples, shared by `main.rs` and `tests/gallery.rs` (which
//! renders every example headless). Each example module is a sibling of this file at the crate root.

use blenderui::prelude::{AnyElement, Rgba, rgb};

#[macro_export]
macro_rules! examples {
    ($($tag:ident: $title:literal => $description:literal),* $(,)?) => {
        pub const EXAMPLES: &[$crate::registry::Example] = &[$($crate::registry::Example {
            title: $title,
            description: $description,
            // the component itself, not `rsx! { <$tag /> }`: a name such as `small` or `summary` is
            // an HTML tag there
            render: || ::blenderui::element::IntoElement::into_any_element(::blenderui::hooks::build($tag::__builder())),
        }),*];
    };
}

pub struct Example {
    pub title: &'static str,
    pub description: &'static str,
    pub render: fn() -> AnyElement,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    Grid(u16),
    Page,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Group {
    Dashboards,
    Base,
    Charts,
}

pub struct Chart {
    pub name: &'static str,
    pub examples: &'static [Example],
    pub layout: Layout,
    pub group: Group,
}

macro_rules! charts {
    ($($group:ident: $name:literal => $examples:expr, $layout:expr;)*) => {
        &[$($crate::registry::Chart { name: $name, examples: $examples, layout: $layout, group: $crate::registry::Group::$group }),*]
    };
}

pub const TWO: Layout = Layout::Grid(2);
pub const ONE: Layout = Layout::Grid(1);

pub const CHARTS: &[Chart] = charts! {
    Dashboards: "Checkout service" => crate::dashboard::EXAMPLES, Layout::Page;
    Dashboards: "Trading desk" => crate::dashboards::TRADING, Layout::Page;
    Dashboards: "Factory floor" => crate::dashboards::FACTORY, Layout::Page;
    Dashboards: "Operations (nested)" => crate::fractal::EXAMPLES, Layout::Page;
    Base: "XY Chart" => crate::xy::EXAMPLES, TWO;
    Base: "Trend" => crate::trend::EXAMPLES, TWO;
    Base: "Candlestick" => crate::candlestick::EXAMPLES, TWO;
    Base: "Heatmap" => crate::heatmap::EXAMPLES, TWO;
    Base: "State Timeline" => crate::state_timeline::EXAMPLES, TWO;
    Base: "Flame Graph" => crate::flame_graph::EXAMPLES, ONE;
    Base: "Bar Gauge" => crate::bar_gauge::EXAMPLES, TWO;
    Charts: "Time Series" => crate::time_series::EXAMPLES, TWO;
    Charts: "Histogram" => crate::histogram::EXAMPLES, TWO;
    Charts: "Status History" => crate::status_history::EXAMPLES, TWO;
    Charts: "Stat" => crate::stat::EXAMPLES, TWO;
    Charts: "Gauge" => crate::gauge::EXAMPLES, TWO;
    Charts: "HoneyComb" => crate::diagram::honey_comb::EXAMPLES, TWO;
    Charts: "Traces" => crate::traces::EXAMPLES, ONE;
    Charts: "Node Graph" => crate::node_graph::EXAMPLES, TWO;
    Charts: "Heikin-Ashi" => crate::finance::heikin_ashi::EXAMPLES, TWO;
    Charts: "Renko" => crate::finance::renko::EXAMPLES, TWO;
    Charts: "Kagi" => crate::finance::kagi::EXAMPLES, TWO;
    Charts: "Point & Figure" => crate::finance::point_figure::EXAMPLES, TWO;
    Charts: "Pareto" => crate::quality::pareto::EXAMPLES, TWO;
    Charts: "Ishikawa" => crate::diagram::ishikawa::EXAMPLES, TWO;
    Charts: "Control Chart" => crate::quality::control_chart::EXAMPLES, TWO;
    Charts: "Bland-Altman" => crate::quality::bland_altman::EXAMPLES, TWO;
    Charts: "Lollipop" => crate::category::lollipop::EXAMPLES, TWO;
    Charts: "Dumbbell" => crate::category::dumbbell::EXAMPLES, TWO;
    Charts: "Slope" => crate::category::slope::EXAMPLES, TWO;
    Charts: "Bump" => crate::category::bump::EXAMPLES, TWO;
    Charts: "Back-to-Back" => crate::category::back_to_back::EXAMPLES, TWO;
    Charts: "Waterfall" => crate::category::waterfall::EXAMPLES, TWO;
    Charts: "Bullet" => crate::category::bullet::EXAMPLES, TWO;
    Charts: "Funnel" => crate::category::funnel::EXAMPLES, TWO;
    Charts: "Mekko" => crate::diagram::mekko::EXAMPLES, TWO;
    Charts: "Forest" => crate::category::forest::EXAMPLES, TWO;
    Charts: "Barcode" => crate::distribution::barcode::EXAMPLES, TWO;
    Charts: "Beeswarm" => crate::distribution::beeswarm::EXAMPLES, TWO;
    Charts: "Ridgeline" => crate::distribution::ridgeline::EXAMPLES, TWO;
    Charts: "Horizon" => crate::distribution::horizon::EXAMPLES, TWO;
    Charts: "Spectrogram" => crate::matrix::spectrogram::EXAMPLES, TWO;
    Charts: "Attention Map" => crate::matrix::attention_map::EXAMPLES, TWO;
    Charts: "Waffle" => crate::matrix::waffle::EXAMPLES, TWO;
    Charts: "Venn" => crate::diagram::venn::EXAMPLES, TWO;
    Charts: "UpSet" => crate::diagram::up_set::EXAMPLES, TWO;
    Charts: "Warped Waffle" => crate::diagram::warped_waffle::EXAMPLES, TWO;
    Charts: "Word Cloud" => crate::diagram::word_cloud::EXAMPLES, TWO;
    Charts: "Gantt" => crate::flow::gantt::EXAMPLES, TWO;
    Charts: "Sankey" => crate::flow::sankey::EXAMPLES, TWO;
    Charts: "Sankey Matrix" => crate::hybrid::sankey_matrix::EXAMPLES, TWO;
    Charts: "Treemap Force" => crate::hybrid::treemap_force::EXAMPLES, TWO;
    Charts: "Chord Diagram" => crate::diagram::chord_diagram::EXAMPLES, TWO;
    Charts: "Circos" => crate::diagram::circos::EXAMPLES, TWO;
    Charts: "Arc Diagram" => crate::diagram::arc_diagram::EXAMPLES, TWO;
    Charts: "Alluvial" => crate::flow::alluvial::EXAMPLES, ONE;
    Charts: "Sankey Funnel" => crate::flow::sankey_funnel::EXAMPLES, ONE;
    Charts: "Dendrogram" => crate::flow::dendrogram::EXAMPLES, TWO;
    Charts: "Icicle" => crate::flow::icicle::EXAMPLES, ONE;
    Charts: "Sunburst" => crate::diagram::sunburst::EXAMPLES, TWO;
    Charts: "Spider" => crate::flow::spider::EXAMPLES, TWO;
    Charts: "Nightingale Rose" => crate::flow::nightingale_rose::EXAMPLES, TWO;
    Charts: "Parallel Coordinates" => crate::flow::parallel_coordinates::EXAMPLES, TWO;
    Charts: "Oscilloscope" => crate::science::oscilloscope::EXAMPLES, TWO;
    Charts: "Efficient Frontier" => crate::science::efficient_frontier::EXAMPLES, TWO;
    Charts: "Phase Portrait" => crate::science::phase_portrait::EXAMPLES, TWO;
    Charts: "Poincaré Map" => crate::science::poincare_map::EXAMPLES, TWO;
    Charts: "Bifurcation" => crate::science::bifurcation::EXAMPLES, TWO;
    Charts: "H-R Diagram" => crate::science::hr_diagram::EXAMPLES, TWO;
    Charts: "Ramachandran" => crate::science::ramachandran::EXAMPLES, TWO;
    Charts: "Harker" => crate::science::harker::EXAMPLES, ONE;
    Charts: "Root Locus" => crate::science::root_locus::EXAMPLES, TWO;
    Charts: "Radiation Pattern" => crate::science::radiation_pattern::EXAMPLES, TWO;
    Charts: "Quiver" => crate::science::quiver::EXAMPLES, TWO;
    Charts: "Hodograph" => crate::science::hodograph::EXAMPLES, TWO;
    Charts: "Smith Chart" => crate::science::smith_chart::EXAMPLES, TWO;
    Charts: "Eye Diagram" => crate::science::eye_diagram::EXAMPLES, TWO;
    Charts: "Sequence Logo" => crate::diagram::sequence_logo::EXAMPLES, TWO;
    Charts: "Ternary" => crate::earth::ternary::EXAMPLES, TWO;
    Charts: "Ternary Contour" => crate::earth::ternary_contour::EXAMPLES, TWO;
    Charts: "Piper" => crate::earth::piper::EXAMPLES, ONE;
    Charts: "Stüve" => crate::earth::stuve::EXAMPLES, TWO;
    Charts: "Tephigram" => crate::earth::tephigram::EXAMPLES, TWO;
    Charts: "Choropleth" => crate::earth::choropleth::EXAMPLES, TWO;
};

pub const GROUPS: [(Group, &str); 3] = [(Group::Dashboards, "Dashboards"), (Group::Base, "Base"), (Group::Charts, "Charts")];


/// The kit theme's page colors (the charts take theirs from the same theme).
#[must_use]
pub fn page() -> (Rgba, Rgba, Rgba) {
    let color = |key: &str, fallback: u32| {
        blenderui::theme_color(key).map_or_else(|| rgb(fallback), |[r, g, b, a]| Rgba { r, g, b, a })
    };
    let (background, foreground) = (color("region_bg", 0x303030), color("text", 0xe6e6e6));
    let muted = Rgba { a: 0.6, ..foreground };
    (background, foreground, muted)
}
