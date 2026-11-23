use blenderui::Hsla;

#[derive(Clone, Copy)]
pub(crate) struct Tone {
    pub text: Hsla,
    pub muted: Hsla,
    pub border: Hsla,
    pub strong: Hsla,
    pub accent: Hsla,
    pub secondary: Hsla,
    pub background: Hsla,
    pub popover: Hsla,
    pub popover_text: Hsla,
    pub danger: Hsla,
    pub warning: Hsla,
    pub info: Hsla,
    pub success: Hsla,
    pub primary: Hsla,
}

impl Tone {
    pub(crate) fn of() -> Self {
        let theme = crate::ui::theme();
        Self {
            text: theme.foreground,
            muted: theme.muted_foreground,
            border: theme.border,
            strong: theme.muted_foreground.opacity(0.6),
            accent: theme.accent,
            secondary: theme.muted,
            background: theme.background,
            popover: theme.popover,
            popover_text: theme.popover_foreground,
            danger: theme.danger,
            warning: theme.warning,
            info: theme.info,
            success: theme.success,
            primary: theme.primary,
        }
    }
}
