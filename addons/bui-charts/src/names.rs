use blenderui::SharedString;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Affixes {
    prefix: usize,
    suffix: usize,
}

impl Affixes {
    pub(crate) fn of<'a>(names: impl Iterator<Item = &'a str>) -> Self {
        let mut names = names;
        let Some(first) = names.next() else {
            return Self::default();
        };
        let words = |name: &'a str| name.split_whitespace();
        let mut shortest = words(first).count();
        let (mut prefix, mut suffix) = (shortest, shortest);
        for name in names {
            shortest = shortest.min(words(name).count());
            prefix = prefix.min(
                words(first)
                    .zip(words(name))
                    .take_while(|(a, b)| a == b)
                    .count(),
            );
            suffix = suffix.min(
                words(first)
                    .rev()
                    .zip(words(name).rev())
                    .take_while(|(a, b)| a == b)
                    .count(),
            );
        }
        let prefix = prefix.min(shortest);
        Self {
            prefix,
            suffix: suffix.min(shortest - prefix),
        }
    }

    pub(crate) fn strip(self, name: &SharedString) -> SharedString {
        let count = name.split_whitespace().count();
        let kept = count.saturating_sub(self.prefix + self.suffix);
        if kept == 0 || kept == count {
            return name.clone();
        }
        join(name.split_whitespace().skip(self.prefix).take(kept))
    }

    pub(crate) fn common(self, name: &str) -> Option<SharedString> {
        let count = name.split_whitespace().count();
        let words = name.split_whitespace();
        let tail = count - self.suffix.min(count);
        let label = words
            .enumerate()
            .filter(|(index, _)| *index < self.prefix || *index >= tail)
            .map(|(_, word)| word);
        let label = join(label);
        (!label.is_empty()).then_some(label)
    }
}

fn join<'a>(words: impl Iterator<Item = &'a str>) -> SharedString {
    let mut joined = String::new();
    for word in words {
        if !joined.is_empty() {
            joined.push(' ');
        }
        joined.push_str(word);
    }
    joined.into()
}

pub(crate) fn common_label<'a>(
    names: impl Iterator<Item = &'a SharedString> + Clone,
) -> Option<SharedString> {
    let mut all = names.clone();
    let first = all.next()?;
    if all.all(|name| name == first) {
        return Some(first.clone());
    }
    Affixes::of(names.map(SharedString::as_ref)).common(first)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shared(names: &[&'static str]) -> Vec<SharedString> {
        names
            .iter()
            .map(|name| SharedString::Borrowed(name))
            .collect()
    }

    #[test]
    fn strips_shared_words() {
        let names = shared(&["cpu load host-a", "cpu load host-b"]);
        let affixes = Affixes::of(names.iter().map(SharedString::as_ref));
        assert_eq!(affixes.strip(&names[0]), "host-a");
        assert_eq!(affixes.strip(&names[1]), "host-b");
        assert_eq!(common_label(names.iter()), Some("cpu load".into()));
    }

    #[test]
    fn keeps_names_that_would_vanish() {
        let names = shared(&["Value", "Value"]);
        let affixes = Affixes::of(names.iter().map(SharedString::as_ref));
        assert_eq!(affixes.strip(&names[0]), "Value");
        assert_eq!(common_label(names.iter()), Some("Value".into()));
    }

    #[test]
    fn keeps_both_ends_of_the_label() {
        let names = shared(&["a x b", "a y b"]);
        assert_eq!(common_label(names.iter()), Some("a b".into()));
    }
}
