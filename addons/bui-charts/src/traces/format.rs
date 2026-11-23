use std::fmt::Write as _;
use web_time::{SystemTime, UNIX_EPOCH};

use blenderui::SharedString;

use crate::time;

const MICROS: [(f64, &str); 6] = [
    (86_400_000_000., "d"),
    (3_600_000_000., "h"),
    (60_000_000., "m"),
    (1_000_000., "s"),
    (1_000., "ms"),
    (1., "μs"),
];

fn rounded(value: f64) -> String {
    let value = (value * 100.).round() / 100.;
    let mut text = format!("{value:.2}");
    while text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    text
}

pub(crate) fn duration(micros: f64) -> SharedString {
    let position = MICROS
        .iter()
        .position(|(unit, _)| micros >= *unit)
        .unwrap_or(MICROS.len() - 1);
    let (unit, label) = MICROS[position];
    if position >= 3 {
        return format!("{}{label}", rounded(micros / unit)).into();
    }
    let (next, next_label) = MICROS[position + 1];
    let mut primary = (micros / unit).floor();
    let mut secondary = ((micros - primary * unit) / next).round();
    if secondary * next >= unit {
        primary += 1.;
        secondary = 0.;
    }
    let mut text = format!("{primary}{label}");
    if secondary > 0. {
        let _ = write!(text, " {secondary}{next_label}");
    }
    text.into()
}

pub(crate) fn clock(micros: u64) -> SharedString {
    let p = time::parts(micros as f64 / 1_000., 0);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
        p.year, p.month, p.day, p.hour, p.minute, p.second, p.milli
    )
    .into()
}

pub(crate) fn time_of_day(micros: u64) -> SharedString {
    let p = time::parts(micros as f64 / 1_000., 0);
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        p.hour, p.minute, p.second, p.milli
    )
    .into()
}

pub(crate) fn ago(micros: u64) -> SharedString {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_micros() as u64);
    let seconds = now.saturating_sub(micros) / 1_000_000;
    let (value, unit) = match seconds {
        0..60 => return "a few seconds ago".into(),
        60..3_600 => (seconds / 60, "minute"),
        3_600..86_400 => (seconds / 3_600, "hour"),
        86_400..2_592_000 => (seconds / 86_400, "day"),
        2_592_000..31_536_000 => (seconds / 2_592_000, "month"),
        _ => (seconds / 31_536_000, "year"),
    };
    let plural = if value == 1 { "" } else { "s" };
    format!("{value} {unit}{plural} ago").into()
}

pub(crate) struct Category {
    pub label: &'static str,
    prefixes: &'static [&'static str],
    resource: bool,
}

const CATEGORIES: [Category; 18] = [
    Category {
        label: "Frontend",
        prefixes: &["browser", "device", "session", "gf.feo11y"],
        resource: true,
    },
    Category {
        label: "Service",
        prefixes: &["service"],
        resource: true,
    },
    Category {
        label: "Deployment",
        prefixes: &["deployment"],
        resource: true,
    },
    Category {
        label: "Database",
        prefixes: &["db"],
        resource: true,
    },
    Category {
        label: "Process",
        prefixes: &["process"],
        resource: true,
    },
    Category {
        label: "Runtime",
        prefixes: &["jvm", "nodejs", "go", "dotnet"],
        resource: true,
    },
    Category {
        label: "Container",
        prefixes: &["container"],
        resource: true,
    },
    Category {
        label: "Kubernetes",
        prefixes: &["k8s"],
        resource: true,
    },
    Category {
        label: "Host / OS",
        prefixes: &["host", "system", "os"],
        resource: true,
    },
    Category {
        label: "Cloud",
        prefixes: &["cloud", "aws", "gcp", "azure", "google"],
        resource: true,
    },
    Category {
        label: "Telemetry SDK",
        prefixes: &["telemetry"],
        resource: true,
    },
    Category {
        label: "HTTP",
        prefixes: &["http"],
        resource: false,
    },
    Category {
        label: "URL",
        prefixes: &["url"],
        resource: false,
    },
    Category {
        label: "Error",
        prefixes: &["error"],
        resource: false,
    },
    Category {
        label: "Exception",
        prefixes: &["exception"],
        resource: false,
    },
    Category {
        label: "Messaging",
        prefixes: &["messaging"],
        resource: false,
    },
    Category {
        label: "Network",
        prefixes: &["network", "net", "server", "client"],
        resource: false,
    },
    Category {
        label: "RPC",
        prefixes: &["rpc"],
        resource: false,
    },
];

const MATCH_ORDER: [usize; 18] = [1, 7, 8, 6, 9, 2, 4, 5, 0, 10, 11, 12, 16, 3, 15, 17, 13, 14];

pub(crate) const OTHER: &str = "Other";

pub(crate) fn category(key: &str) -> Option<usize> {
    let key = key.to_ascii_lowercase();
    MATCH_ORDER.into_iter().find(|index| {
        let category = &CATEGORIES[*index];
        let skip_google = category.label == "Runtime" && key.starts_with("google");
        !skip_google
            && category.prefixes.iter().any(|prefix| {
                key.strip_prefix(prefix)
                    .is_some_and(|rest| rest.starts_with('.') || rest.starts_with('_'))
            })
    })
}

pub(crate) fn category_label(index: Option<usize>) -> &'static str {
    index.map_or(OTHER, |index| CATEGORIES[index].label)
}

pub(crate) type Rank = (u8, usize, &'static str);

pub(crate) fn category_rank(index: Option<usize>, resource: bool) -> Rank {
    const SPAN_FIRST: [usize; 2] = [11, 12];
    match index {
        None => (2, 0, ""),
        Some(index) if resource && CATEGORIES[index].resource => (0, index, ""),
        Some(index) if !resource && SPAN_FIRST.contains(&index) => (0, index, ""),
        Some(index) => (1, 0, CATEGORIES[index].label),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_durations_like_grafana() {
        assert_eq!(duration(0.), "0μs");
        assert_eq!(duration(850.), "850μs");
        assert_eq!(duration(1_234.), "1.23ms");
        assert_eq!(duration(1_500_000.), "1.5s");
        assert_eq!(duration(3_900_000_000.), "1h 5m");
        assert_eq!(duration(119_700_000.), "2m");
        assert_eq!(duration(60_000_000.), "1m");
    }

    #[test]
    fn formats_utc_dates() {
        assert_eq!(clock(1_700_000_000_123_000), "2023-11-14 22:13:20.123");
    }

    #[test]
    fn categorises_attribute_keys() {
        assert_eq!(category_label(category("http.method")), "HTTP");
        assert_eq!(category_label(category("k8s.pod.name")), "Kubernetes");
        assert_eq!(category_label(category("google.cloud")), "Cloud");
        assert_eq!(category_label(category("custom")), OTHER);
    }
}
