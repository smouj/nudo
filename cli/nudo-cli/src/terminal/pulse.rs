//! `NudoPulse`: NUDO's own indeterminate-progress animation.
//!
//! A node travelling along a connection, not a generic spinner:
//!
//! ```text
//! ●────  ─●───  ──●──  ───●─  ────●  ───●─  ──●──  ─●───
//! ```
//!
//! Three rules make it safe to have at all:
//!
//! * **The frames are a pure function of a tick.** Anything that can be computed
//!   can be tested, and a test never has to wait for an animation.
//! * **It needs permission.** `Pulse::allowed` is the only gate, and outside a
//!   real terminal it says no.
//! * **Its absence changes nothing.** The frames go to a stream that is already
//!   showing progress; the command's result, diagnostics and exit code are
//!   decided elsewhere. A piped run and an animated run are the same run.
//!
//! It is not wired to `nudo check`: checking a local file has no indeterminate
//! phase, and animating a step that is already finished would be a lie the
//! specification forbids. It exists, it is gated, and the first genuinely
//! long-running command will use it.

use super::{Capabilities, Role, Symbols, Theme};

/// The default number of cells in the connection.
pub(crate) const DEFAULT_SPAN: usize = 5;

/// The travelling-node animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Pulse {
    span: usize,
    symbols: Symbols,
}

impl Pulse {
    /// A pulse with a connection `span` cells wide.
    #[must_use]
    pub(crate) fn new(span: usize, symbols: Symbols) -> Self {
        Pulse {
            span: span.max(3),
            symbols,
        }
    }

    /// A pulse long enough to be visible and short enough to be quiet.
    #[must_use]
    pub(crate) fn default_for(symbols: Symbols) -> Self {
        Pulse::new(DEFAULT_SPAN, symbols)
    }

    /// The number of cells in the connection.
    #[must_use]
    pub(crate) const fn span(&self) -> usize {
        self.span
    }

    /// The number of frames before the animation repeats.
    #[must_use]
    pub(crate) const fn cycle(&self) -> usize {
        // Ping-pong: the node visits both ends once per cycle without pausing on
        // them, so `0 ..= span` forward and `span-1 ..= 1` back.
        self.span * 2 - 2
    }

    /// Which cell the node occupies at `tick`.
    #[must_use]
    pub(crate) const fn position(&self, tick: usize) -> usize {
        let cycle = self.cycle();
        let step = tick % cycle;
        if step < self.span { step } else { cycle - step }
    }

    /// The frame at `tick`, with no colour.
    #[must_use]
    pub(crate) fn frame(&self, tick: usize) -> String {
        let node = self.position(tick);
        let connection = self.symbols.connection();
        let cell = connection.chars().next().unwrap_or('-');
        let mut frame = String::with_capacity(self.span * cell.len_utf8());
        for index in 0..self.span {
            if index == node {
                frame.push_str(self.symbols.glyph(super::Status::Active));
            } else {
                frame.push(cell);
            }
        }
        frame
    }

    /// The frame at `tick`, painted for this terminal.
    #[must_use]
    pub(crate) fn painted_frame(&self, tick: usize, theme: Theme) -> String {
        theme.paint(Role::Nudo, &self.frame(tick))
    }

    /// Whether this terminal may animate at all.
    ///
    /// The single gate. Everything that wants to animate asks here first, so
    /// that "no motion in a pipe or in CI" is one decision in one place.
    #[must_use]
    pub(crate) const fn allowed(capabilities: &Capabilities) -> bool {
        capabilities.motion
    }

    /// The `ticks` frames of this pulse, which is what a test compares.
    #[must_use]
    pub(crate) fn frames(&self, ticks: usize) -> Vec<String> {
        (0..ticks).map(|tick| self.frame(tick)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::{Capabilities, Environment, Mode};

    #[test]
    fn the_node_travels_and_comes_back() {
        let pulse = Pulse::new(5, Symbols::new(true));
        assert_eq!(pulse.frame(0), "●────");
        assert_eq!(pulse.frame(1), "─●───");
        assert_eq!(pulse.frame(2), "──●──");
        assert_eq!(pulse.frame(3), "───●─");
        assert_eq!(pulse.frame(4), "────●");
        assert_eq!(pulse.frame(5), "───●─");
        assert_eq!(pulse.frame(6), "──●──");
        assert_eq!(pulse.frame(7), "─●───");
    }

    #[test]
    fn the_animation_repeats_exactly_once_per_cycle() {
        let pulse = Pulse::new(5, Symbols::new(true));
        assert_eq!(pulse.cycle(), 8);
        assert_eq!(pulse.frame(0), pulse.frame(pulse.cycle()));
        assert_eq!(pulse.frame(3), pulse.frame(pulse.cycle() + 3));
    }

    #[test]
    fn every_frame_is_the_same_width() {
        let pulse = Pulse::new(6, Symbols::new(true));
        let widths: Vec<usize> = pulse
            .frames(pulse.cycle() + 1)
            .iter()
            .map(|frame| frame.chars().count())
            .collect();
        assert!(widths.iter().all(|width| *width == 6), "{widths:?}");
    }

    #[test]
    fn the_ascii_fallback_stays_ascii() {
        let pulse = Pulse::new(5, Symbols::new(false));
        for frame in pulse.frames(pulse.cycle()) {
            assert!(frame.is_ascii(), "{frame:?}");
        }
        assert_eq!(pulse.frame(0), ">----");
    }

    #[test]
    fn a_span_too_small_to_travel_is_widened() {
        let pulse = Pulse::new(1, Symbols::new(true));
        assert_eq!(pulse.span(), 3);
        assert!(pulse.cycle() >= 4);
    }

    #[test]
    fn motion_is_allowed_only_where_a_terminal_can_show_it() {
        let interactive = Environment {
            is_tty: true,
            unicode: true,
            ..Environment::default()
        };
        assert!(Pulse::allowed(&Capabilities::from(&interactive)));

        let piped = Environment {
            unicode: true,
            ..Environment::default()
        };
        assert!(!Pulse::allowed(&Capabilities::from(&piped)));

        let ci = Environment {
            is_tty: true,
            ci: true,
            unicode: true,
            ..Environment::default()
        };
        assert!(!Pulse::allowed(&Capabilities::from(&ci)));

        let no_color = Environment {
            is_tty: true,
            no_color: true,
            unicode: true,
            ..Environment::default()
        };
        assert!(!Pulse::allowed(&Capabilities::from(&no_color)));

        let forced_ci = Environment {
            is_tty: true,
            force_ci: true,
            unicode: true,
            ..Environment::default()
        };
        assert!(!Pulse::allowed(&Capabilities::from(&forced_ci)));
    }

    #[test]
    fn a_colorless_frame_has_no_escape_bytes() {
        let pulse = Pulse::default_for(Symbols::new(true));
        let painted = pulse.painted_frame(0, Theme::new(false));
        assert_eq!(painted, pulse.frame(0));
        assert!(!painted.contains('\x1b'));
    }

    #[test]
    fn the_mode_decides_whether_motion_is_even_possible() {
        let interactive = Environment {
            is_tty: true,
            unicode: true,
            ..Environment::default()
        };
        assert_eq!(Capabilities::from(&interactive).mode, Mode::Interactive);
        assert!(Capabilities::from(&interactive).motion);
    }
}
