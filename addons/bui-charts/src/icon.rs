//! The small line icons the charts' controls use (Lucide's shapes, on its 24-unit grid), drawn on
//! the painter in the color given: blenderui has no icon set of its own to draw from.

use blenderui::element::{AnyElement, IntoElement};
use blenderui::{Hsla, Painter, Styled, canvas, px};

use crate::geom::{Bounds, point};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum IconName {
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    Check,
    ChevronDown,
    ChevronLeft,
    ChevronRight,
    ChevronUp,
    ChevronsUpDown,
    CircleAlert,
    CircleCheck,
    Close,
    Copy,
    EllipsisVertical,
    ExternalLink,
    Maximize,
    Minimize,
    Minus,
    Plus,
    Search,
    TriangleAlert,
}

/// One stroke of an icon, in grid units.
enum Mark {
    Line(&'static [(f32, f32)]),
    Closed(&'static [(f32, f32)]),
    Circle(f32, f32, f32),
    Dot(f32, f32),
    Frame(f32, f32, f32, f32),
}

impl IconName {
    const fn marks(self) -> &'static [Mark] {
        use Mark::{Circle, Closed, Dot, Frame, Line};
        match self {
            Self::ArrowDown => &[Line(&[(12., 5.), (12., 19.)]), Line(&[(19., 12.), (12., 19.), (5., 12.)])],
            Self::ArrowUp => &[Line(&[(12., 19.), (12., 5.)]), Line(&[(5., 12.), (12., 5.), (19., 12.)])],
            Self::ArrowLeft => &[Line(&[(19., 12.), (5., 12.)]), Line(&[(12., 19.), (5., 12.), (12., 5.)])],
            Self::ArrowRight => &[Line(&[(5., 12.), (19., 12.)]), Line(&[(12., 5.), (19., 12.), (12., 19.)])],
            Self::Check => &[Line(&[(20., 6.), (9., 17.), (4., 12.)])],
            Self::ChevronDown => &[Line(&[(6., 9.), (12., 15.), (18., 9.)])],
            Self::ChevronUp => &[Line(&[(18., 15.), (12., 9.), (6., 15.)])],
            Self::ChevronLeft => &[Line(&[(15., 18.), (9., 12.), (15., 6.)])],
            Self::ChevronRight => &[Line(&[(9., 18.), (15., 12.), (9., 6.)])],
            Self::ChevronsUpDown => &[Line(&[(7., 15.), (12., 20.), (17., 15.)]), Line(&[(7., 9.), (12., 4.), (17., 9.)])],
            Self::CircleAlert => &[Circle(12., 12., 10.), Line(&[(12., 8.), (12., 12.)]), Dot(12., 16.)],
            Self::CircleCheck => &[Circle(12., 12., 10.), Line(&[(9., 12.), (11., 14.), (15., 10.)])],
            Self::Close => &[Line(&[(18., 6.), (6., 18.)]), Line(&[(6., 6.), (18., 18.)])],
            Self::Copy => &[Frame(8., 8., 14., 14.), Line(&[(4., 16.), (2., 16.), (2., 2.), (16., 2.), (16., 4.)])],
            Self::EllipsisVertical => &[Dot(12., 5.), Dot(12., 12.), Dot(12., 19.)],
            Self::ExternalLink => &[
                Line(&[(15., 3.), (21., 3.), (21., 9.)]),
                Line(&[(10., 14.), (21., 3.)]),
                Line(&[(18., 13.), (18., 21.), (3., 21.), (3., 6.), (11., 6.)]),
            ],
            Self::Maximize => &[
                Line(&[(8., 3.), (3., 3.), (3., 8.)]),
                Line(&[(21., 8.), (21., 3.), (16., 3.)]),
                Line(&[(3., 16.), (3., 21.), (8., 21.)]),
                Line(&[(16., 21.), (21., 21.), (21., 16.)]),
            ],
            Self::Minimize => &[
                Line(&[(8., 3.), (8., 8.), (3., 8.)]),
                Line(&[(21., 8.), (16., 8.), (16., 3.)]),
                Line(&[(3., 16.), (8., 16.), (8., 21.)]),
                Line(&[(16., 21.), (16., 16.), (21., 16.)]),
            ],
            Self::Minus => &[Line(&[(5., 12.), (19., 12.)])],
            Self::Plus => &[Line(&[(5., 12.), (19., 12.)]), Line(&[(12., 5.), (12., 19.)])],
            Self::Search => &[Circle(11., 11., 8.), Line(&[(21., 21.), (16.65, 16.65)])],
            Self::TriangleAlert => &[
                Closed(&[(12., 3.), (22., 20.), (2., 20.)]),
                Line(&[(12., 9.), (12., 13.)]),
                Dot(12., 17.),
            ],
        }
    }

    fn paint(self, bounds: Bounds, color: Hsla, painter: &mut Painter) {
        let side = bounds.size.width.min(bounds.size.height);
        let scale = side / 24.;
        let origin = bounds.center() - point(side / 2., side / 2.);
        let width = (2. * scale).max(1.);
        let at = |(x, y): (f32, f32)| (origin.x + x * scale, origin.y + y * scale);
        let mut points: Vec<(f32, f32)> = Vec::with_capacity(6);
        for mark in self.marks() {
            match *mark {
                Mark::Line(line) | Mark::Closed(line) => {
                    points.clear();
                    points.extend(line.iter().copied().map(at));
                    if matches!(mark, Mark::Closed(_)) {
                        points.push(at(line[0]));
                    }
                    painter.line(&points, color, width);
                }
                Mark::Circle(x, y, r) => {
                    let (x, y) = at((x - r, y - r));
                    let d = r * 2. * scale;
                    painter.oval(Bounds::new(point(x, y), crate::geom::size(d, d)).into(), Hsla::TRANSPARENT, color, width);
                }
                Mark::Dot(x, y) => {
                    let (x, y) = at((x, y));
                    let r = width * 0.75;
                    painter.oval(Bounds::new(point(x - r, y - r), crate::geom::size(r * 2., r * 2.)).into(), color, Hsla::TRANSPARENT, 0.);
                }
                Mark::Frame(x, y, w, h) => {
                    let (x, y) = at((x, y));
                    painter.quad(Bounds::new(point(x, y), crate::geom::size(w * scale, h * scale)).into(), 2. * scale, Hsla::TRANSPARENT, color, width);
                }
            }
        }
    }
}

/// An icon element: `icon(IconName::Close, color)`, 14 px square.
pub(crate) const fn icon(name: IconName, color: Hsla) -> Icon {
    Icon { name, color, size: 14. }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Icon {
    name: IconName,
    color: Hsla,
    size: f32,
}

impl Icon {
    #[must_use]
    pub(crate) const fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
}

impl IntoElement for Icon {
    fn into_any_element(self) -> AnyElement {
        let Self { name, color, size } = self;
        canvas(move |bounds, painter| name.paint(bounds.into(), color, painter))
            .w(px(size))
            .h(px(size))
            .flex_none()
            .into_any_element()
    }
}
