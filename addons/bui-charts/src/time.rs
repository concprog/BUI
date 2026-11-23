use std::fmt::Write as _;

use blenderui::SharedString;

const SECOND: f64 = 1_000.;
const MINUTE: f64 = 60. * SECOND;
const HOUR: f64 = 60. * MINUTE;
const DAY: f64 = 24. * HOUR;
const MONTH: f64 = 30. * DAY;
const YEAR: f64 = 365. * DAY;

pub(crate) const STEPS: [f64; 37] = [
    1.,
    2.,
    5.,
    10.,
    20.,
    50.,
    100.,
    200.,
    500.,
    SECOND,
    2. * SECOND,
    5. * SECOND,
    10. * SECOND,
    15. * SECOND,
    30. * SECOND,
    MINUTE,
    2. * MINUTE,
    5. * MINUTE,
    10. * MINUTE,
    15. * MINUTE,
    30. * MINUTE,
    HOUR,
    2. * HOUR,
    3. * HOUR,
    4. * HOUR,
    6. * HOUR,
    8. * HOUR,
    12. * HOUR,
    DAY,
    2. * DAY,
    3. * DAY,
    7. * DAY,
    14. * DAY,
    MONTH,
    3. * MONTH,
    6. * MONTH,
    YEAR,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Parts {
    pub year: i64,
    pub month: i64,
    pub day: i64,
    pub hour: i64,
    pub minute: i64,
    pub second: i64,
    pub milli: i64,
}

const fn civil(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + if month <= 2 { 1 } else { 0 };
    (year, month, day)
}

pub(crate) fn parts(millis: f64, offset_minutes: i32) -> Parts {
    let millis = millis.round() as i64 + i64::from(offset_minutes) * 60_000;
    let days = millis.div_euclid(86_400_000);
    let rest = millis.rem_euclid(86_400_000);
    let (year, month, day) = civil(days);
    Parts {
        year,
        month,
        day,
        hour: rest / 3_600_000,
        minute: rest / 60_000 % 60,
        second: rest / 1_000 % 60,
        milli: rest % 1_000,
    }
}

pub(crate) fn tick(millis: f64, step: f64, span: f64, offset: i32) -> SharedString {
    let p = parts(millis, offset);
    let text = if step < SECOND {
        format!(
            "{:02}:{:02}:{:02}.{:03}",
            p.hour, p.minute, p.second, p.milli
        )
    } else if step < MINUTE {
        format!("{:02}:{:02}:{:02}", p.hour, p.minute, p.second)
    } else if step < DAY && span > DAY && p.hour == 0 && p.minute == 0 {
        format!("{:02}/{:02}", p.month, p.day)
    } else if step < DAY && span > DAY {
        format!("{:02}/{:02} {:02}:{:02}", p.month, p.day, p.hour, p.minute)
    } else if step < DAY {
        format!("{:02}:{:02}", p.hour, p.minute)
    } else if step < MONTH {
        format!("{:02}/{:02}", p.month, p.day)
    } else if step < YEAR {
        format!("{}-{:02}", p.year, p.month)
    } else {
        format!("{}", p.year)
    };
    text.into()
}

pub(crate) fn full(millis: f64, offset: i32) -> SharedString {
    let p = parts(millis, offset);
    format!(
        "{}-{:02}-{:02} {:02}:{:02}:{:02}",
        p.year, p.month, p.day, p.hour, p.minute, p.second
    )
    .into()
}

pub(crate) fn step(span: f64, count: f32) -> f64 {
    let wanted = span / f64::from(count.max(1.));
    STEPS
        .into_iter()
        .find(|step| *step >= wanted)
        .unwrap_or_else(|| (wanted / YEAR).ceil() * YEAR)
}

pub(crate) fn first(min: f64, step: f64, offset: i32) -> f64 {
    let shift = f64::from(offset) * MINUTE;
    ((min + shift) / step).ceil().mul_add(step, -shift)
}

pub(crate) fn span(millis: f64) -> SharedString {
    let millis = millis.max(0.);
    let units = [(DAY, "d"), (HOUR, "h"), (MINUTE, "m"), (SECOND, "s")];
    let mut rest = millis;
    let mut out = String::new();
    for (unit, label) in units {
        let whole = (rest / unit).floor();
        if whole > 0. {
            if !out.is_empty() {
                out.push(' ');
            }
            let _ = write!(out, "{whole}{label}");
            rest -= whole * unit;
        }
    }
    if out.is_empty() {
        out = format!("{}ms", millis.round());
    } else if rest >= 1. && millis < MINUTE {
        let _ = write!(out, " {}ms", rest.round());
    }
    out.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_epoch_millis_into_utc_parts() {
        let p = parts(1_700_000_000_123., 0);
        assert_eq!(
            (p.year, p.month, p.day, p.hour, p.minute, p.second, p.milli),
            (2023, 11, 14, 22, 13, 20, 123)
        );
        assert_eq!(parts(1_700_000_000_123., 120).hour, 0);
    }

    #[test]
    fn formats_ticks_by_step() {
        let t = 1_700_000_000_000.;
        assert_eq!(tick(t, 5. * MINUTE, HOUR, 0), "22:13");
        assert_eq!(tick(t, 10. * SECOND, MINUTE, 0), "22:13:20");
        assert_eq!(tick(t, DAY, 10. * DAY, 0), "11/14");
        assert_eq!(tick(t, MONTH, YEAR, 0), "2023-11");
    }

    #[test]
    fn picks_round_steps_and_aligns_them() {
        assert!((step(HOUR, 6.) - 10. * MINUTE).abs() < f64::EPSILON);
        assert!((first(61_000., MINUTE, 0) - 120_000.).abs() < f64::EPSILON);
        assert_eq!(span(3_900_000.), "1h 5m");
        assert_eq!(span(1_500.), "1s 500ms");
    }
}
