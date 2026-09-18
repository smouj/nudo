//! The terminal presentation layer.
//!
//! Everything here is **presentation**: it turns data the toolchain already
//! produced into something a person can read. No stage of the compiler knows
//! this module exists, and nothing in it may change what a command does, what it
//! reports, or what it exits with.
//!
//! The rules live in [`docs/tooling/terminal-ux.md`][spec]; this module is their
//! implementation. Read that document before changing anything here, and keep
//! the two in step.
//!
//! [spec]: https://github.com/smouj/nudo/blob/main/docs/tooling/terminal-ux.md

pub(crate) mod layout;
/// `NudoPulse` is implemented and gated, and deliberately not wired to a
/// command yet: no command has a genuinely indeterminate phase, and animating a
/// step that has already finished would be a lie. See the specification.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) mod pulse;
pub(crate) mod report;
pub(crate) mod symbols;
pub(crate) mod theme;

use crate::ColorChoice;

pub(crate) use layout::{Section, Stage, render_rule, render_sections, render_stages};
pub(crate) use symbols::{Status, Symbols};
pub(crate) use theme::{Role, Theme};

/// How the command's output is presented.
///
/// The mode is chosen from the environment, never from what the command *means*:
/// the same `nudo check` produces the same result in all three, and only the
/// words around it change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    /// A real terminal: structure, colour where the terminal has it, and the
    /// possibility of motion.
    Interactive,
    /// A pipe, a redirect, `--plain`, or a terminal that cannot do colour: one
    /// line per event, nothing to reprocess.
    Plain,
    /// CI: the same as plain, plus a deterministic block a log can be diffed
    /// against, ending in `PASS` or `FAIL`.
    Ci,
}

impl Mode {
    /// The mode this environment calls for.
    ///
    /// `--plain` and `--ci` win over detection, because an explicit request is
    /// better evidence than a guess: a shell can be a TTY and still want lines.
    #[must_use]
    pub(crate) fn select(environment: &Environment) -> Self {
        if environment.force_ci {
            return Mode::Ci;
        }
        if environment.force_plain || environment.no_color {
            return Mode::Plain;
        }
        if !environment.is_tty {
            return Mode::Plain;
        }
        if environment.ci {
            return Mode::Ci;
        }
        Mode::Interactive
    }

    /// Whether this mode may redraw in place.
    #[must_use]
    pub(crate) const fn allows_motion(self) -> bool {
        matches!(self, Mode::Interactive)
    }
}

/// What the environment can do, decided once.
///
/// Every answer comes from this one value, so that detection is not repeated —
/// and so that a test can ask "what would this look like in CI, in a pipe, in a
/// narrow terminal?" without a terminal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Environment {
    /// Whether the stream is a terminal.
    pub(crate) is_tty: bool,
    /// Whether `NO_COLOR` is set, which means the user asked for no colour.
    pub(crate) no_color: bool,
    /// Whether `TERM` is `dumb`.
    pub(crate) dumb_terminal: bool,
    /// Whether `CI` is set.
    pub(crate) ci: bool,
    /// `--plain`.
    pub(crate) force_plain: bool,
    /// `--ci`.
    pub(crate) force_ci: bool,
    /// `--color`.
    pub(crate) color_choice: ColorChoice,
    /// The terminal width in columns, when it can be discovered.
    pub(crate) width: Option<usize>,
    /// Whether the terminal can represent the Unicode symbols.
    pub(crate) unicode: bool,
}

impl Default for Environment {
    fn default() -> Self {
        Environment {
            is_tty: false,
            no_color: false,
            dumb_terminal: false,
            ci: false,
            force_plain: false,
            force_ci: false,
            color_choice: ColorChoice::Auto,
            width: None,
            unicode: false,
        }
    }
}

impl Environment {
    /// Reads the environment this process is actually in.
    #[must_use]
    pub(crate) fn detect(color_choice: ColorChoice) -> Self {
        let term = std::env::var("TERM").unwrap_or_default();
        let locale = std::env::var("LC_ALL")
            .or_else(|_| std::env::var("LC_CTYPE"))
            .or_else(|_| std::env::var("LANG"))
            .unwrap_or_default();
        Environment {
            is_tty: std::io::IsTerminal::is_terminal(&std::io::stdout()),
            no_color: std::env::var_os("NO_COLOR").is_some(),
            dumb_terminal: term == "dumb",
            ci: std::env::var_os("CI").is_some(),
            force_plain: false,
            force_ci: false,
            color_choice,
            width: terminal_width(),
            unicode: terminal_utf8(&locale),
        }
    }

    /// The width to compose for: at least 40 columns, as the specification
    /// promises, and at most 100 so that a very wide terminal does not produce
    /// lines a reader has to scan across.
    #[must_use]
    pub(crate) fn width(&self) -> usize {
        self.width
            .unwrap_or(DEFAULT_WIDTH)
            .clamp(MINIMUM_WIDTH, MAXIMUM_WIDTH)
    }
}

/// What the terminal can do, once the environment and the mode are known.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Capabilities {
    /// The presentation mode.
    pub(crate) mode: Mode,
    /// Whether colour may be emitted.
    pub(crate) color: bool,
    /// Whether the Unicode symbols may be used.
    pub(crate) unicode: bool,
    /// The width to compose for, in columns.
    pub(crate) width: usize,
    /// Whether motion (the pulse) is allowed.
    pub(crate) motion: bool,
}

impl Capabilities {
    /// Decides what this environment and mode can do.
    #[must_use]
    pub(crate) fn from(environment: &Environment) -> Self {
        let mode = Mode::select(environment);
        let color = match environment.color_choice {
            ColorChoice::Always => true,
            ColorChoice::Never => false,
            ColorChoice::Auto => {
                environment.is_tty && !environment.no_color && !environment.dumb_terminal
            }
        };
        let unicode = environment.unicode && !environment.dumb_terminal;
        let motion = mode.allows_motion() && color && !environment.ci && !environment.no_color;
        Capabilities {
            mode,
            color,
            unicode,
            width: environment.width(),
            motion,
        }
    }

    /// The symbols this terminal can show.
    #[must_use]
    pub(crate) fn symbols(&self) -> Symbols {
        Symbols::new(self.unicode)
    }

    /// The theme this terminal can show.
    #[must_use]
    pub(crate) fn theme(&self) -> Theme {
        Theme::new(self.color)
    }
}

/// The width assumed when the terminal will not say.
pub(crate) const DEFAULT_WIDTH: usize = 80;
/// The narrowest width NUDO composes for.
pub(crate) const MINIMUM_WIDTH: usize = 40;
/// The widest width NUDO composes for.
pub(crate) const MAXIMUM_WIDTH: usize = 100;

/// The terminal's width, when it can be read without a dependency.
///
/// `COLUMNS` is honoured, because that is what a shell, a script or a test can
/// set to say how wide the window is. There is no `ioctl` fallback: the
/// workspace forbids `unsafe`, a syscall needs it, and a terminal that will not
/// say its width gets [`DEFAULT_WIDTH`] — which is why nothing in this layer
/// depends on a fixed column.
fn terminal_width() -> Option<usize> {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|columns| *columns > 0)
}

/// Whether the terminal's locale can represent the Unicode symbols.
fn terminal_utf8(locale: &str) -> bool {
    let locale = locale.to_ascii_lowercase();
    locale.contains("utf-8") || locale.contains("utf8")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn environment() -> Environment {
        Environment {
            unicode: true,
            ..Environment::default()
        }
    }

    #[test]
    fn a_pipe_is_plain_even_when_it_could_colour() {
        let environment = environment();
        assert_eq!(Mode::select(&environment), Mode::Plain);
        let capabilities = Capabilities::from(&environment);
        assert!(!capabilities.color, "a pipe is not a terminal");
        assert!(!capabilities.motion);
    }

    #[test]
    fn a_terminal_is_interactive_and_may_animate() {
        let environment = Environment {
            is_tty: true,
            ..environment()
        };
        let capabilities = Capabilities::from(&environment);
        assert_eq!(capabilities.mode, Mode::Interactive);
        assert!(capabilities.color);
        assert!(capabilities.motion);
        assert!(capabilities.unicode);
    }

    #[test]
    fn no_color_turns_colour_off_and_asks_for_plain_lines() {
        let environment = Environment {
            is_tty: true,
            no_color: true,
            ..environment()
        };
        let capabilities = Capabilities::from(&environment);
        assert!(!capabilities.color);
        assert!(!capabilities.motion);
        assert_eq!(capabilities.mode, Mode::Plain);
    }

    #[test]
    fn a_dumb_terminal_gets_no_colour_and_no_unicode() {
        let environment = Environment {
            is_tty: true,
            dumb_terminal: true,
            ..environment()
        };
        let capabilities = Capabilities::from(&environment);
        assert!(!capabilities.color);
        assert!(!capabilities.unicode);
    }

    #[test]
    fn ci_is_deterministic_and_still() {
        let environment = Environment {
            is_tty: true,
            ci: true,
            ..environment()
        };
        let capabilities = Capabilities::from(&environment);
        assert_eq!(capabilities.mode, Mode::Ci);
        assert!(!capabilities.motion, "CI must not animate");
    }

    #[test]
    fn an_explicit_request_beats_detection() {
        let forced_plain = Environment {
            is_tty: true,
            force_plain: true,
            ..environment()
        };
        assert_eq!(Mode::select(&forced_plain), Mode::Plain);

        let forced_ci = Environment {
            is_tty: true,
            force_ci: true,
            ..environment()
        };
        assert_eq!(Mode::select(&forced_ci), Mode::Ci);
    }

    #[test]
    fn color_choice_overrides_the_terminal() {
        let always = Environment {
            color_choice: ColorChoice::Always,
            ..environment()
        };
        assert!(Capabilities::from(&always).color);

        let never = Environment {
            is_tty: true,
            color_choice: ColorChoice::Never,
            ..environment()
        };
        assert!(!Capabilities::from(&never).color);
    }

    #[test]
    fn width_is_clamped_to_what_the_specification_promises() {
        let narrow = Environment {
            width: Some(10),
            ..environment()
        };
        assert_eq!(Capabilities::from(&narrow).width, MINIMUM_WIDTH);

        let wide = Environment {
            width: Some(500),
            ..environment()
        };
        assert_eq!(Capabilities::from(&wide).width, MAXIMUM_WIDTH);

        assert_eq!(Capabilities::from(&environment()).width, DEFAULT_WIDTH);
    }

    #[test]
    fn a_locale_without_utf8_gets_the_ascii_symbols() {
        assert!(terminal_utf8("en_US.UTF-8"));
        assert!(terminal_utf8("es_ES.utf8"));
        assert!(!terminal_utf8("C"));
        assert!(!terminal_utf8("en_US.ISO-8859-1"));
    }
}
