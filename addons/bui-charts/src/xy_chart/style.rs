use crate::plot::{HideFrom, LineStyle};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Show {
    #[default]
    Points,
    Lines,
    PointsAndLines,
}

impl Show {
    pub(crate) fn points(self) -> bool {
        self != Self::Lines
    }

    pub(crate) fn lines(self) -> bool {
        self != Self::Points
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PointShape {
    #[default]
    Circle,
    Square,
}

style!(Style {
    show: Show = Show::Points,
    point_size: f32 = 5.,
    size_range: (f32, f32) = (5., 100.),
    point_shape: PointShape = PointShape::Circle,
    point_stroke_width: f32 = 1.,
    fill_opacity: f32 = 0.5,
    line_width: f32 = 2.,
    line_style: LineStyle = LineStyle::Solid,
    hide_from: HideFrom = HideFrom {
        legend: false,
        tooltip: false,
        viz: false
    },
});
