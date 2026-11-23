use super::options::GaugeShape;

pub(crate) const ARC_START: f32 = 250.;
pub(crate) const ARC_END: f32 = 110.;

pub(crate) fn to_rad(angle: f32) -> f32 {
    (angle - 90.).to_radians()
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Dimensions {
    pub radius: f32,
    pub center_x: f32,
    pub center_y: f32,
    pub bar_width: f32,
    pub bottom_y: f32,
    pub marker_radius: f32,
    pub marker_width: f32,
    pub label_size: f32,
    pub label_radius: f32,
}

#[allow(clippy::struct_excessive_bools)]
pub(crate) struct Options {
    pub glow: bool,
    pub rounded: bool,
    pub width_factor: f32,
    pub markers: bool,
    pub labels: bool,
}

pub(crate) const fn angles(shape: GaugeShape) -> (f32, f32) {
    match shape {
        GaugeShape::Gauge => (ARC_START, 360. - ARC_START + ARC_END),
        GaugeShape::Circle => (0., 360.),
    }
}

pub(crate) fn dimensions(
    width: f32,
    height: f32,
    shape: GaugeShape,
    options: &Options,
) -> Dimensions {
    let end = match shape {
        GaugeShape::Gauge => ARC_END,
        GaugeShape::Circle => 360.,
    };
    let y_max_angle = end.min(180.);
    let margin = if options.glow {
        0.02 * width.min(height)
    } else {
        0.
    };
    let mut max_w = width / 2. - margin;
    let below = to_rad(y_max_angle).sin();
    let mut max_h = (height - margin * 2.) / (1. + below);
    let max_radius = max_w.min(max_h);
    let limited_by_height = max_h <= max_w;
    let mut outer = max_radius;
    let bar_width = (options.width_factor * (max_radius / 3.)).max(2.);
    let endcap = if y_max_angle < 180. && options.rounded {
        bar_width * 0.5
    } else {
        0.
    };
    let radius_endcap = endcap * 0.5;
    outer -= radius_endcap;
    max_h -= radius_endcap;
    max_w -= radius_endcap;
    let (mut label_size, mut label_radius) = (0., 0.);
    if options.labels {
        label_radius = outer;
        label_size = (0.12 * outer.powf(0.92)).max(10.);
        let used = label_size * 1.2 + label_size / 3.;
        outer -= used;
        max_w -= used;
        max_h -= used;
        if limited_by_height && end < 180. {
            let amount = outer * 0.07;
            label_radius -= amount;
            max_h -= amount;
            max_w -= amount;
            outer -= amount;
        }
    }
    let marker_width = if options.markers {
        (0.2 * bar_width.powf(0.92)).clamp(4., 12.)
    } else {
        0.
    };
    let marker_spacing = (marker_width / 2.).clamp(2., 12.);
    let mut marker_radius = 0.;
    if marker_width > 0. {
        marker_radius = outer - marker_width / 2.;
        max_w -= marker_width + marker_spacing;
        max_h -= marker_width + marker_spacing;
        outer = max_w.min(max_h);
    }
    let radius = outer - bar_width / 2.;
    let below_center = max_radius * to_rad(y_max_angle).sin();
    let rest = height - below_center - margin * 2. - max_radius;
    let center_y = max_radius + margin + rest / 2. - (endcap - radius_endcap);
    Dimensions {
        radius,
        center_x: width / 2.,
        center_y,
        bar_width,
        bottom_y: center_y + below_center,
        marker_radius,
        marker_width,
        label_size,
        label_radius,
    }
}

pub(crate) fn value_angles(fraction: f32, neutral: Option<f32>, range: f32) -> (f32, f32) {
    let value = fraction.clamp(0., 1.) * range;
    let (start, length) = match neutral {
        Some(neutral) => {
            let neutral = neutral.clamp(0., 1.) * range;
            if neutral <= value {
                (neutral, value - neutral)
            } else {
                (value, neutral - value)
            }
        }
        None => (0., value),
    };
    (start, length.clamp(0., range))
}

pub(crate) fn segment_count(
    dims: &Dimensions,
    spacing: f32,
    count: usize,
    range: f32,
) -> (usize, f32) {
    let between = spacing * (range / 1.5 / count as f32).max(2.);
    let inner = dims.radius - dims.bar_width / 2.;
    let circumference = std::f32::consts::PI * inner * 2. * (range / 360.);
    let fitting = (circumference / (between + 3.)).floor().max(1.) as usize;
    (fitting.min(count), between)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gauge_arc_spans_220_degrees() {
        assert!((angles(GaugeShape::Gauge).1 - 220.).abs() < 1e-4);
        assert!((angles(GaugeShape::Circle).1 - 360.).abs() < 1e-4);
    }

    #[test]
    fn neutral_splits_the_bar() {
        assert_eq!(value_angles(0.75, None, 200.), (0., 150.));
        assert_eq!(value_angles(0.25, Some(0.5), 200.), (50., 50.));
        assert_eq!(value_angles(0.75, Some(0.5), 200.), (100., 50.));
    }

    #[test]
    fn circle_fits_the_smaller_side() {
        let options = Options {
            glow: false,
            rounded: false,
            width_factor: 0.5,
            markers: false,
            labels: false,
        };
        let dims = dimensions(200., 100., GaugeShape::Circle, &options);
        let bar = 0.5 * 50. / 3.;
        assert!((dims.radius - (50. - bar / 2.)).abs() < 1e-3);
        assert!((dims.center_y - 50.).abs() < 1e-3);
    }
}
