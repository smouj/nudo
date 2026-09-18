//! The five states, and the glyphs that stand for them.
//!
//! One place, so that no command writes a symbol by hand. The Unicode set is the
//! one in [`docs/tooling/terminal-ux.md`][spec]; the ASCII set is what a
//! terminal that cannot show it gets, and either way the word is printed too —
//! a state is never carried by a glyph alone.
//!
//! [spec]: https://github.com/smouj/nudo/blob/main/docs/tooling/terminal-ux.md

/// The state of a stage, a file or a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Status {
    /// Not started.
    Pending,
    /// Running now.
    Active,
    /// Finished, nothing to report.
    Success,
    /// Finished, with something to look at.
    Warning,
    /// Failed.
    Error,
}

impl Status {
    /// The word for this state. Always printed, in every mode.
    #[must_use]
    pub(crate) const fn word(self) -> &'static str {
        match self {
            Status::Pending => "pending",
            Status::Active => "running",
            Status::Success => "ok",
            Status::Warning => "warning",
            Status::Error => "error",
        }
    }

    /// The theme role this state is painted with.
    #[must_use]
    pub(crate) const fn role(self) -> crate::terminal::Role {
        match self {
            Status::Pending => crate::terminal::Role::Dim,
            Status::Active => crate::terminal::Role::Nudo,
            Status::Success => crate::terminal::Role::Ok,
            Status::Warning => crate::terminal::Role::Warn,
            Status::Error => crate::terminal::Role::Err,
        }
    }
}

/// The glyph set a terminal can show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Symbols {
    unicode: bool,
}

impl Symbols {
    /// The Unicode set, or the ASCII one.
    #[must_use]
    pub(crate) const fn new(unicode: bool) -> Self {
        Symbols { unicode }
    }

    /// The glyph for a state.
    #[must_use]
    pub(crate) const fn glyph(self, status: Status) -> &'static str {
        match (self.unicode, status) {
            (true, Status::Pending) => "○",
            (true, Status::Active) => "●",
            (true, Status::Success) => "✓",
            (true, Status::Warning) => "!",
            (true, Status::Error) => "×",
            (false, Status::Pending) => ".",
            (false, Status::Active) => ">",
            (false, Status::Success) => "+",
            (false, Status::Warning) => "!",
            (false, Status::Error) => "x",
        }
    }

    /// The rule used between pipeline stages when they fit on one line.
    #[must_use]
    pub(crate) const fn connection(self) -> &'static str {
        if self.unicode { "──" } else { "--" }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_state_has_a_glyph_in_both_sets() {
        for status in [
            Status::Pending,
            Status::Active,
            Status::Success,
            Status::Warning,
            Status::Error,
        ] {
            let unicode = Symbols::new(true).glyph(status);
            let ascii = Symbols::new(false).glyph(status);
            assert!(!unicode.is_empty());
            assert!(!ascii.is_empty());
            assert!(
                ascii.is_ascii(),
                "{status:?} has a non-ASCII fallback: {ascii:?}"
            );
        }
    }

    #[test]
    fn the_two_sets_agree_on_the_states_that_have_one_glyph() {
        assert_eq!(Symbols::new(true).glyph(Status::Warning), "!");
        assert_eq!(Symbols::new(false).glyph(Status::Warning), "!");
    }

    #[test]
    fn every_state_has_a_word_that_is_not_its_glyph() {
        for status in [Status::Success, Status::Error] {
            assert_ne!(status.word(), Symbols::new(true).glyph(status));
            assert!(status.word().is_ascii());
        }
    }
}
