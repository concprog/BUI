use std::collections::HashMap;

use super::model::types::{Id, Rect};

pub const SPRING_STIFFNESS: f64 = 210.0;
pub const SPRING_DAMPING: f64 = 29.0;
pub const SPRING_MAX_STEP: f64 = 0.032;
pub const SPRING_EPSILON: f64 = 0.0001;

pub const LAYOUT_MS: f64 = 460.0;
pub const PICKUP_MS: f64 = 280.0;
pub const APPEAR_MS: f64 = 220.0;
pub const DOCK_HIDE_MS: f64 = 300.0;
pub const DOCK_RESTORE_MS: f64 = 380.0;

#[must_use]
pub fn ease_out_quint(t: f64) -> f64 {
    1.0 - (1.0 - t.clamp(0.0, 1.0)).powi(5)
}

#[derive(Clone, Debug)]
pub struct RectSpring {
    pub value: Rect,
    pub target: Rect,
    pub velocity: Rect,
}

impl RectSpring {
    #[must_use]
    pub const fn new(initial: Rect) -> Self {
        Self {
            value: initial,
            target: initial,
            velocity: Rect::ZERO,
        }
    }

    #[must_use]
    pub fn moving(&self) -> bool {
        !self.value.approx_eq(&self.target, 1e-9) || !self.velocity.approx_eq(&Rect::ZERO, 1e-9)
    }

    pub fn step(&mut self, dt: f64) -> bool {
        let dt = if dt > 0.0 {
            dt.min(SPRING_MAX_STEP)
        } else {
            0.016
        };
        let mut error = 0.0;
        let (v, t, vel) = (&mut self.value, &self.target, &mut self.velocity);
        for (value, target, velocity) in [
            (&mut v.x, t.x, &mut vel.x),
            (&mut v.y, t.y, &mut vel.y),
            (&mut v.w, t.w, &mut vel.w),
            (&mut v.h, t.h, &mut vel.h),
        ] {
            *velocity += ((target - *value) * SPRING_STIFFNESS - *velocity * SPRING_DAMPING) * dt;
            *value += *velocity * dt;
            error += (target - *value).abs() + velocity.abs() * 0.1;
        }
        if error <= SPRING_EPSILON {
            self.finish();
            return false;
        }
        true
    }

    pub const fn finish(&mut self) {
        self.value = self.target;
        self.velocity = Rect::ZERO;
    }

    pub const fn jump(&mut self, rect: Rect) {
        self.target = rect;
        self.finish();
    }
}

#[derive(Clone, Debug)]
pub struct LayoutTween {
    from: HashMap<Id, Rect>,
    start: f64,
    progress: f64,
}

impl Default for LayoutTween {
    fn default() -> Self {
        Self {
            from: HashMap::new(),
            start: 0.0,
            progress: 1.0,
        }
    }
}

impl LayoutTween {
    pub fn begin(&mut self, from: HashMap<Id, Rect>, now: f64) {
        self.progress = if from.is_empty() { 1.0 } else { 0.0 };
        self.from = from;
        self.start = now;
    }

    pub fn step(&mut self, now: f64, duration: f64) -> bool {
        let elapsed = if duration > 0.0 {
            (now - self.start) / duration
        } else {
            1.0
        };
        self.progress = ease_out_quint(elapsed);
        if elapsed >= 1.0 {
            self.stop();
        }
        self.progress < 1.0
    }

    pub fn stop(&mut self) {
        self.progress = 1.0;
        self.from.clear();
    }

    #[must_use]
    pub fn active(&self) -> bool {
        self.progress < 1.0
    }

    #[must_use]
    pub fn apply(&self, id: &str, target: Rect) -> Rect {
        match self.from.get(id) {
            Some(from) if self.progress < 1.0 => from.lerp(&target, self.progress),
            _ => target,
        }
    }
}

#[must_use]
pub fn cubic_bezier(x1: f64, y1: f64, x2: f64, y2: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let sample = |a: f64, b: f64, t: f64| {
        ((1.0 - 3.0 * b + 3.0 * a) * t + (3.0 * b - 6.0 * a)) * t * t + 3.0 * a * t
    };
    let (mut lo, mut hi, mut t) = (0.0, 1.0, x);
    for _ in 0..24 {
        let v = sample(x1, x2, t);
        if (v - x).abs() < 1e-5 {
            break;
        }
        if v < x {
            lo = t;
        } else {
            hi = t;
        }
        t = f64::midpoint(lo, hi);
    }
    sample(y1, y2, t)
}

#[must_use]
pub fn dock_ease(t: f64) -> f64 {
    cubic_bezier(0.2, 0.75, 0.2, 1.0, t)
}

#[must_use]
pub fn rubber_band(x: f64, lo: f64, hi: f64, limit: f64) -> f64 {
    let band = |d: f64| (1.0 - 1.0 / (d * 0.5 / limit + 1.0)) * limit;
    if x < lo {
        lo - band(lo - x)
    } else if x > hi {
        hi + band(x - hi)
    } else {
        x
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rubber_band_resists_past_the_edges() {
        assert_eq!(rubber_band(0.5, 0.0, 1.0, 0.05), 0.5);
        let past = rubber_band(1.5, 0.0, 1.0, 0.05);
        assert!(past > 1.0 && past < 1.05);
        assert!(rubber_band(-9.0, 0.0, 1.0, 0.05) > -0.05);
    }

    #[test]
    fn spring_settles_on_its_target() {
        let mut s = RectSpring::new(Rect::UNIT);
        s.target = Rect::new(0.2, 0.2, 0.5, 0.5);
        let mut frames = 0;
        while s.step(1.0 / 60.0) {
            frames += 1;
            assert!(frames < 600, "spring never settled");
        }
        assert!(s.value.approx_eq(&s.target, 1e-9));
    }

    #[test]
    fn bezier_is_monotonic_and_pinned() {
        assert_eq!(dock_ease(0.0), 0.0);
        assert_eq!(dock_ease(1.0), 1.0);
        assert!(dock_ease(0.3) < dock_ease(0.6));
    }

    #[test]
    fn tween_reaches_the_target() {
        let mut t = LayoutTween::default();
        let id: Id = "a".into();
        t.begin([(id, Rect::ZERO)].into(), 0.0);
        assert!(t.active());
        assert!(!t.step(LAYOUT_MS + 1.0, LAYOUT_MS));
        assert_eq!(t.apply("a", Rect::UNIT), Rect::UNIT);
    }
}
