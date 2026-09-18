//! The colour roles, and the only place ANSI escapes are written.
//!
//! Colour is secondary to structure: every distinction this module can make is
//! also made by a word, in every mode. When colour is off, painting is the
//! identity function, so a renderer never has to ask whether to paint.

/// What a piece of text means, not what colour it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Role {
    /// NUDO blue: the active, current or selected thing.
    Nudo,
    /// Success.
    Ok,
    /// Warning.
    Warn,
    /// Error.
    Err,
    /// Metadata: counts, paths, timing.
    Dim,
    /// Ordinary text.
    Plain,
}

impl Role {
    /// The ANSI sequence that starts this role, or an empty string.
    #[must_use]
    pub(crate) const fn ansi(self) -> &'static str {
        match self {
            Role::Nudo => "\x1b[34m",
            Role::Ok => "\x1b[32m",
            Role::Warn => "\x1b[33m",
            Role::Err => "\x1b[31m",
            Role::Dim => "\x1b[2m",
            Role::Plain => "",
        }
    }
}

/// Paints text for one terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Theme {
    color: bool,
}

impl Theme {
    /// A theme that paints, or one that does not.
    #[must_use]
    pub(crate) const fn new(color: bool) -> Self {
        Theme { color }
    }

    /// Wraps `text` in this role, if the terminal can show it.
    ///
    /// The reset is only emitted when something was set: a pipe must not receive
    /// an escape byte, not even a balanced pair.
    #[must_use]
    pub(crate) fn paint(self, role: Role, text: &str) -> String {
        if !self.color || role == Role::Plain {
            return text.to_string();
        }
        format!("{}{}\x1b[0m", role.ansi(), text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_colorless_theme_emits_no_escape_bytes() {
        let theme = Theme::new(false);
        for role in [Role::Nudo, Role::Ok, Role::Warn, Role::Err, Role::Dim] {
            let painted = theme.paint(role, "text");
            assert_eq!(painted, "text");
            assert!(!painted.contains('\x1b'));
        }
    }

    #[test]
    fn a_colored_theme_resets_what_it_sets() {
        let theme = Theme::new(true);
        let painted = theme.paint(Role::Err, "error");
        assert!(painted.starts_with("\x1b[31m"));
        assert!(painted.ends_with("\x1b[0m"));
        assert!(painted.contains("error"));
    }

    #[test]
    fn plain_text_is_never_wrapped_even_in_color() {
        let theme = Theme::new(true);
        assert_eq!(theme.paint(Role::Plain, "text"), "text");
    }
}
