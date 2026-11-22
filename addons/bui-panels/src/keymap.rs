use std::collections::{HashMap, HashSet};

use blenderui::prelude::{Keystroke, Modifiers};

#[must_use]
pub const fn is_mac() -> bool {
    cfg!(target_os = "macos")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Command {
    FrameToggle,
    NavigationBack,
    NavigationForward,
    NavigationOverview,
    NavigationStepOut,
    NavigationLeft,
    NavigationRight,
    NavigationUp,
    NavigationDown,
    PanelNext,
    PanelPrevious,
    TabNext,
    TabPrevious,
    ViewClose,
    ViewReopen,
    PanelFloat,
    PanelHide,
    ShortcutsToggle,
}

impl Command {
    pub const ALL: [Self; 18] = [
        Self::NavigationLeft,
        Self::NavigationRight,
        Self::NavigationUp,
        Self::NavigationDown,
        Self::NavigationOverview,
        Self::NavigationBack,
        Self::NavigationForward,
        Self::NavigationStepOut,
        Self::FrameToggle,
        Self::PanelNext,
        Self::PanelPrevious,
        Self::TabNext,
        Self::TabPrevious,
        Self::ViewClose,
        Self::ViewReopen,
        Self::PanelFloat,
        Self::PanelHide,
        Self::ShortcutsToggle,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::FrameToggle => "Frame panel",
            Self::NavigationBack => "Back",
            Self::NavigationForward => "Forward",
            Self::NavigationOverview => "Overview",
            Self::NavigationStepOut => "Step out",
            Self::NavigationLeft => "Panel to the left",
            Self::NavigationRight => "Panel to the right",
            Self::NavigationUp => "Panel above",
            Self::NavigationDown => "Panel below",
            Self::PanelNext => "Next panel",
            Self::PanelPrevious => "Previous panel",
            Self::TabNext => "Next tab",
            Self::TabPrevious => "Previous tab",
            Self::ViewClose => "Close tab",
            Self::ViewReopen => "Reopen closed tab",
            Self::PanelFloat => "Float panel",
            Self::PanelHide => "Hide panel",
            Self::ShortcutsToggle => "Shortcuts",
        }
    }
}

pub type Keymap = HashMap<Command, Option<String>>;

#[must_use]
pub fn default_keymap() -> Keymap {
    use Command::{
        FrameToggle, NavigationBack, NavigationDown, NavigationForward, NavigationLeft,
        NavigationOverview, NavigationRight, NavigationStepOut, NavigationUp, PanelFloat,
        PanelHide, PanelNext, PanelPrevious, ShortcutsToggle, TabNext, TabPrevious, ViewClose,
        ViewReopen,
    };
    let some = |s: &str| Some(s.to_string());
    HashMap::from([
        (FrameToggle, some("Mod+Shift+Enter")),
        (NavigationBack, some("Mod+Alt+ArrowLeft")),
        (NavigationForward, some("Mod+Alt+ArrowRight")),
        (NavigationOverview, some("Mod+Alt+ArrowUp,Mod+0")),
        (NavigationStepOut, some("Escape")),
        (NavigationLeft, some("Alt+Shift+ArrowLeft")),
        (NavigationRight, some("Alt+Shift+ArrowRight")),
        (NavigationUp, some("Alt+Shift+ArrowUp")),
        (NavigationDown, some("Alt+Shift+ArrowDown")),
        (ShortcutsToggle, some("Mod+Shift+/,Mod+?,Mod+Shift+?")),
        (PanelNext, some("F6")),
        (PanelPrevious, some("Shift+F6")),
        (TabNext, some("Mod+Tab")),
        (TabPrevious, some("Mod+Shift+Tab")),
        (ViewClose, some("Mod+W")),
        (ViewReopen, some("Mod+Shift+T")),
        (PanelFloat, None),
        (PanelHide, None),
    ])
}

#[must_use]
pub fn merged_keymap(overrides: &Keymap) -> Keymap {
    let mut map = default_keymap();
    map.extend(overrides.iter().map(|(k, v)| (*k, v.clone())));
    map
}

/// A key's canonical name: BUI's keysyms (`Return`, `Left`, `ISO_Left_Tab`, `slash` ...) and the
/// DOM names a keymap is written with (`Enter`, `ArrowLeft` ...) meet here. Other names come back
/// as they are (compare them ignoring ASCII case).
fn canonical(name: &str) -> &str {
    const ALIASES: [(&str, &str); 22] = [
        ("arrowleft", "left"),
        ("arrowright", "right"),
        ("arrowup", "up"),
        ("arrowdown", "down"),
        ("return", "enter"),
        ("kp_enter", "enter"),
        ("iso_left_tab", "tab"),
        ("esc", "escape"),
        (" ", "space"),
        ("slash", "/"),
        ("question", "?"),
        ("plus", "+"),
        ("minus", "-"),
        ("shift_l", "shift"),
        ("shift_r", "shift"),
        ("control_l", "control"),
        ("control_r", "control"),
        ("alt_l", "alt"),
        ("alt_r", "alt"),
        ("super_l", "meta"),
        ("super_r", "meta"),
        ("meta_l", "meta"),
    ];
    ALIASES
        .iter()
        .find(|(alias, _)| alias.eq_ignore_ascii_case(name))
        .map_or(name, |(_, key)| key)
}

#[must_use]
pub fn normalize_key(name: &str) -> String {
    canonical(name).to_ascii_lowercase()
}

fn same_key(a: &str, b: &str) -> bool {
    canonical(a).eq_ignore_ascii_case(canonical(b))
}

#[derive(Clone, Debug, Default, PartialEq)]
#[allow(clippy::struct_excessive_bools)] // one flag per modifier a combo names
struct Combo {
    key: String,
    modifier: bool,
    ctrl: bool,
    meta: bool,
    alt: bool,
    shift: bool,
}

fn parse_combo(combo: &str) -> Combo {
    let mut parts: Vec<&str> = combo.split('+').map(str::trim).collect();
    let key = match parts.pop() {
        Some("") | None => "+".to_string(),
        Some(k) => normalize_key(k),
    };
    let has = |name: &str| parts.iter().any(|p| p.eq_ignore_ascii_case(name));
    Combo {
        key,
        modifier: has("Mod"),
        ctrl: has("Ctrl") || has("Control"),
        meta: has("Meta") || has("Cmd"),
        alt: has("Alt") || has("Option"),
        shift: has("Shift"),
    }
}

#[must_use]
pub fn matches(keystroke: &Keystroke, combo: &str) -> bool {
    combo.split(',').any(|part| matches_single(keystroke, part.trim()))
}

fn matches_single(keystroke: &Keystroke, combo: &str) -> bool {
    parse_combo(combo).matches(keystroke)
}

impl Combo {
    /// BUI reports Cmd as Control on macOS (and no Super elsewhere): `Mod`, `Ctrl` and, on macOS,
    /// `Cmd` / `Meta` all mean the Control bit; elsewhere a `Meta` combo never matches.
    fn matches(&self, keystroke: &Keystroke) -> bool {
        if self.meta && !is_mac() {
            return false;
        }
        let m = &keystroke.modifiers;
        m.control == (self.ctrl || self.modifier || self.meta)
            && m.alt == self.alt
            && m.shift == self.shift
            && same_key(&keystroke.key, &self.key)
    }
}

#[derive(Default)]
pub(crate) struct Bindings(Vec<(Command, Vec<Combo>)>);

impl Bindings {
    pub(crate) fn new(map: &Keymap) -> Self {
        let mut list: Vec<(Command, Vec<Combo>)> = map
            .iter()
            .filter_map(|(command, combo)| {
                let combos = combo
                    .as_deref()?
                    .split(',')
                    .map(|part| parse_combo(part.trim()))
                    .collect();
                Some((*command, combos))
            })
            .collect();
        list.sort_by_key(|(command, _)| Command::ALL.iter().position(|c| c == command));
        Self(list)
    }

    pub(crate) fn matching<'a>(
        &'a self,
        keystroke: &'a Keystroke,
    ) -> impl Iterator<Item = Command> + 'a {
        self.0
            .iter()
            .filter(|(_, combos)| combos.iter().any(|c| c.matches(keystroke)))
            .map(|(command, _)| *command)
    }
}

#[must_use]
pub fn format_combo(combo: &str) -> String {
    let first = combo.split(',').next().unwrap_or(combo).trim();
    let c = parse_combo(first);
    let mac = is_mac();
    let key = match c.key.as_str() {
        "left" => "←".to_string(),
        "right" => "→".to_string(),
        "up" => "↑".to_string(),
        "down" => "↓".to_string(),
        "enter" if mac => "↩".to_string(),
        "enter" => "Enter".to_string(),
        k if k.chars().count() == 1 => k.to_uppercase(),
        k => k.to_string(),
    };
    if mac {
        return format!(
            "{}{}{}{}{key}",
            if c.ctrl { "⌃" } else { "" },
            if c.alt { "⌥" } else { "" },
            if c.shift { "⇧" } else { "" },
            if c.modifier || c.meta { "⌘" } else { "" },
        );
    }
    [
        (c.modifier || c.ctrl).then_some("Ctrl"),
        c.meta.then_some("Meta"),
        c.alt.then_some("Alt"),
        c.shift.then_some("Shift"),
        Some(key.as_str()),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join("+")
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GestureKeys {
    pub pan: Vec<String>,
    pub scale: Vec<String>,
    pub rect: Vec<String>,
    pub step: Vec<String>,
    pub touch_scale: Vec<String>,
}

#[must_use]
pub fn default_gesture_keys() -> GestureKeys {
    let list = |items: &[&str]| items.iter().map(std::string::ToString::to_string).collect();
    GestureKeys {
        pan: list(&[]),
        scale: list(&["space"]),
        rect: list(&["Mod+Alt+Shift", "Mod+Shift"]),
        step: list(&["Mod+Alt"]),
        touch_scale: list(&["Ctrl"]),
    }
}

#[derive(Clone, Debug)]
#[allow(clippy::struct_excessive_bools)] // one flag per modifier a chord names
pub(crate) struct Chord {
    ctrl: bool,
    meta: bool,
    alt: bool,
    shift: bool,
    keys: Vec<String>,
}

fn parse_chord(combo: &str) -> Chord {
    let mac = is_mac();
    let mut chord = Chord {
        ctrl: false,
        meta: false,
        alt: false,
        shift: false,
        keys: Vec::new(),
    };
    for part in combo.split('+').map(str::trim).filter(|p| !p.is_empty()) {
        match part.to_ascii_lowercase().as_str() {
            "mod" => {
                if mac {
                    chord.meta = true;
                } else {
                    chord.ctrl = true;
                }
            }
            "ctrl" | "control" => chord.ctrl = true,
            "meta" | "cmd" => chord.meta = true,
            "alt" | "option" => chord.alt = true,
            "shift" => chord.shift = true,
            other => chord.keys.push(normalize_key(other)),
        }
    }
    chord
}

impl Chord {
    fn matches(&self, modifiers: Modifiers, held: &HashSet<String>) -> bool {
        (!self.meta || is_mac())
            && modifiers.control == (self.ctrl || self.meta)
            && modifiers.alt == self.alt
            && modifiers.shift == self.shift
            && self.keys.iter().all(|k| held.contains(k))
    }
}

#[must_use]
pub fn matches_chord(modifiers: Modifiers, combos: &[String], held: &HashSet<String>) -> bool {
    combos
        .iter()
        .any(|combo| parse_chord(combo).matches(modifiers, held))
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Chords {
    pub pan: Vec<Chord>,
    pub scale: Vec<Chord>,
    pub rect: Vec<Chord>,
    pub step: Vec<Chord>,
    pub touch_scale: Vec<Chord>,
}

impl Chords {
    pub(crate) fn new(keys: &GestureKeys) -> Self {
        let parse = |combos: &[String]| combos.iter().map(|c| parse_chord(c)).collect();
        Self {
            pan: parse(&keys.pan),
            scale: parse(&keys.scale),
            rect: parse(&keys.rect),
            step: parse(&keys.step),
            touch_scale: parse(&keys.touch_scale),
        }
    }
}

pub(crate) fn any_chord(modifiers: Modifiers, chords: &[Chord], held: &HashSet<String>) -> bool {
    chords.iter().any(|c| c.matches(modifiers, held))
}

#[must_use]
pub fn is_chord_key(keys: &GestureKeys, key: &str) -> bool {
    [&keys.pan, &keys.scale, &keys.rect]
        .into_iter()
        .flatten()
        .any(|combo| parse_chord(combo).keys.iter().any(|k| k == key))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ks(key: &str, modifiers: Modifiers) -> Keystroke {
        Keystroke {
            modifiers,
            key: key.into(),
            key_char: None,
        }
    }

    #[test]
    fn combos_match_by_physical_key_and_modifiers() {
        let m = Modifiers {
            shift: true,
            control: true,
            alt: true,
        };
        assert!(matches(&ks("enter", Modifiers::default()), "Enter"));
        assert!(!matches(&ks("enter", m), "Enter"));
        let mod_alt = Modifiers {
            control: true,
            alt: true,
            ..Modifiers::default()
        };
        assert!(matches(&ks("left", mod_alt), "Mod+Alt+ArrowLeft"));
        assert!(matches(&ks("up", mod_alt), "Mod+Alt+ArrowUp,Mod+0"));
        let just_mod = Modifiers {
            control: true,
            ..Modifiers::default()
        };
        assert!(matches(&ks("0", just_mod), "Mod+Alt+ArrowUp,Mod+0"));
        assert!(matches(&ks("tab", just_mod), "Mod+Tab"));
    }

    #[test]
    fn bui_keysyms_match_dom_names() {
        let shift = Modifiers {
            shift: true,
            ..Modifiers::default()
        };
        let ctrl_shift = Modifiers {
            control: true,
            ..shift
        };
        assert!(matches(&ks("Return", Modifiers::default()), "Enter"));
        assert!(matches(&ks("Left", ctrl_shift), "Ctrl+Shift+ArrowLeft"));
        assert!(matches(&ks("ISO_Left_Tab", ctrl_shift), "Mod+Shift+Tab"));
        assert!(matches(&ks("slash", ctrl_shift), "Mod+Shift+/"));
        assert!(matches(&ks("F6", shift), "Shift+F6"));
        assert!(matches(&ks("Escape", Modifiers::default()), "Escape"));
        assert_eq!(normalize_key("Shift_L"), "shift");
    }

    #[test]
    fn chords_need_their_letter_held() {
        let combos = vec!["space".to_string()];
        let mut held = HashSet::new();
        let m = Modifiers::default();
        assert!(!matches_chord(m, &combos, &held));
        held.insert("space".to_string());
        assert!(matches_chord(m, &combos, &held));
    }
}
