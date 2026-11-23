//! Plane geometry in logical pixels (`f32`), named as in gpui (`point`, `size`, `Bounds`): blenderui's
//! own `Point` / `Bounds` are plain records without arithmetic, which a chart's layout is made of.
//! Converts to and from blenderui's at the painter and at events.

use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

use blenderui::{Pixels, geometry};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Size {
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Bounds {
    pub origin: Point,
    pub size: Size,
}

pub(crate) const fn point(x: f32, y: f32) -> Point {
    Point { x, y }
}

pub(crate) const fn size(width: f32, height: f32) -> Size {
    Size { width, height }
}

impl Point {
    pub(crate) const fn tuple(self) -> (f32, f32) {
        (self.x, self.y)
    }

    pub(crate) fn distance(self, other: Self) -> f32 {
        (self.x - other.x).hypot(self.y - other.y)
    }
}

impl Add for Point {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        point(self.x + other.x, self.y + other.y)
    }
}

impl AddAssign for Point {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Sub for Point {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        point(self.x - other.x, self.y - other.y)
    }
}

impl SubAssign for Point {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl Mul<f32> for Point {
    type Output = Self;
    fn mul(self, k: f32) -> Self {
        point(self.x * k, self.y * k)
    }
}

impl Div<f32> for Point {
    type Output = Self;
    fn div(self, k: f32) -> Self {
        point(self.x / k, self.y / k)
    }
}

impl Neg for Point {
    type Output = Self;
    fn neg(self) -> Self {
        point(-self.x, -self.y)
    }
}

impl Mul<f32> for Size {
    type Output = Self;
    fn mul(self, k: f32) -> Self {
        size(self.width * k, self.height * k)
    }
}

impl Bounds {
    pub(crate) const fn new(origin: Point, size: Size) -> Self {
        Self { origin, size }
    }

    pub(crate) fn from_corners(a: Point, b: Point) -> Self {
        let (left, right) = (a.x.min(b.x), a.x.max(b.x));
        let (top, bottom) = (a.y.min(b.y), a.y.max(b.y));
        Self::new(point(left, top), size(right - left, bottom - top))
    }

    pub(crate) fn centered_at(center: Point, size: Size) -> Self {
        Self::new(point(center.x - size.width / 2., center.y - size.height / 2.), size)
    }

    pub(crate) const fn left(&self) -> f32 {
        self.origin.x
    }

    pub(crate) fn right(&self) -> f32 {
        self.origin.x + self.size.width
    }

    pub(crate) const fn top(&self) -> f32 {
        self.origin.y
    }

    pub(crate) fn bottom(&self) -> f32 {
        self.origin.y + self.size.height
    }

    pub(crate) fn center(&self) -> Point {
        point(self.origin.x + self.size.width / 2., self.origin.y + self.size.height / 2.)
    }

    pub(crate) fn bottom_right(&self) -> Point {
        point(self.right(), self.bottom())
    }

    pub(crate) fn contains(&self, at: Point) -> bool {
        at.x >= self.left() && at.x < self.right() && at.y >= self.top() && at.y < self.bottom()
    }

    pub(crate) fn intersect(&self, other: &Self) -> Self {
        let left = self.left().max(other.left());
        let top = self.top().max(other.top());
        let right = self.right().min(other.right()).max(left);
        let bottom = self.bottom().min(other.bottom()).max(top);
        Self::new(point(left, top), size(right - left, bottom - top))
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.size.width <= 0. || self.size.height <= 0.
    }
}

impl From<geometry::Bounds<f32>> for Bounds {
    fn from(b: geometry::Bounds<f32>) -> Self {
        Self::new(point(b.origin.x, b.origin.y), size(b.size.width, b.size.height))
    }
}

impl From<Bounds> for geometry::Bounds<f32> {
    fn from(b: Bounds) -> Self {
        Self { origin: geometry::point(b.origin.x, b.origin.y), size: geometry::size(b.size.width, b.size.height) }
    }
}

impl From<geometry::Point<Pixels>> for Point {
    fn from(p: geometry::Point<Pixels>) -> Self {
        point(p.x.0, p.y.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corners_and_edges() {
        let b = Bounds::from_corners(point(30., 40.), point(10., 20.));
        assert_eq!((b.left(), b.top(), b.right(), b.bottom()), (10., 20., 30., 40.));
        assert_eq!(b.center(), point(20., 30.));
        assert!(b.contains(point(10., 20.)) && !b.contains(point(30., 40.)));
        assert_eq!(b.intersect(&Bounds::new(point(25., 0.), size(100., 25.))).size, size(5., 5.));
    }
}
