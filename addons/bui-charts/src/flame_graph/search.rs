use blenderui::SharedString;
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use regex::{Regex, RegexBuilder};

const REGEX_CHARS: &[char] = &[
    '.', '*', '+', '?', '^', '$', '{', '}', '(', ')', '|', '[', ']', '\\',
];

pub(crate) fn mark(query: &str, labels: &[SharedString], found: &mut Vec<bool>) {
    found.clear();
    found.resize(labels.len(), false);
    let mut matcher: Option<Matcher> = None;
    let mut buffer = Vec::new();
    for term in query.split(',').filter(|term| !term.is_empty()) {
        let by_regex = Regex::new(term).is_ok_and(|regex| {
            let mut any = false;
            for (slot, label) in found.iter_mut().zip(labels) {
                if regex.is_match(label) {
                    *slot = true;
                    any = true;
                }
            }
            any
        });
        if by_regex {
            continue;
        }
        let pattern = Pattern::parse(term, CaseMatching::Smart, Normalization::Smart);
        let matcher = matcher.get_or_insert_with(|| Matcher::new(Config::DEFAULT));
        for (slot, label) in found.iter_mut().zip(labels) {
            if pattern
                .score(Utf32Str::new(label, &mut buffer), matcher)
                .is_some()
            {
                *slot = true;
            }
        }
    }
}

pub(crate) enum Finder {
    Regex(Regex),
    Text(String),
    Invalid,
}

impl Finder {
    pub(crate) fn new(query: &str) -> Self {
        if query.contains(REGEX_CHARS) {
            RegexBuilder::new(query)
                .case_insensitive(true)
                .build()
                .map_or(Self::Invalid, Self::Regex)
        } else {
            Self::Text(query.to_lowercase())
        }
    }

    pub(crate) fn is_match(&self, label: &str) -> bool {
        match self {
            Self::Regex(regex) => regex.is_match(label),
            Self::Text(text) => contains_folded(label, text),
            Self::Invalid => false,
        }
    }

    pub(crate) const fn is_invalid(&self) -> bool {
        matches!(self, Self::Invalid)
    }
}

fn contains_folded(haystack: &str, needle: &str) -> bool {
    haystack.char_indices().any(|(start, _)| {
        let mut rest = haystack[start..].chars().flat_map(char::to_lowercase);
        needle.chars().all(|expected| rest.next() == Some(expected))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels() -> Vec<SharedString> {
        ["runtime.mallocgc", "net/http.serve", "main.parse"]
            .map(SharedString::Borrowed)
            .into()
    }

    #[test]
    fn regex_terms_then_fuzzy_fallback() {
        let labels = labels();
        let mut found = Vec::new();
        mark("^main", &labels, &mut found);
        assert_eq!(found, [false, false, true]);
        mark("mlcgc", &labels, &mut found);
        assert_eq!(found, [true, false, false]);
        mark("serve,parse", &labels, &mut found);
        assert_eq!(found, [false, true, true]);
    }

    #[test]
    fn call_tree_search_is_case_insensitive() {
        assert!(Finder::new("PARSE").is_match("main.parse"));
        assert!(Finder::new("^net.*serve$").is_match("net/http.serve"));
        assert!(Finder::new("(").is_invalid());
    }
}
