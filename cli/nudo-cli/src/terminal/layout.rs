//! Composition: rules, key/value sections, and the pipeline.
//!
//! Two ideas, and both are about *not* assuming a terminal:
//!
//! * a **section** is a title and aligned key/value rows, padded to the widest
//!   key in that section and never to a fixed column;
//! * a **line of stages** is a composition, not a format: if it fits on one
//!   line it is drawn on one line, and if it does not, the same stages are drawn
//!   one per line. Same information, different composition.
//!
//! Everything here returns a `String`. Nothing writes to a stream, which is what
//! makes a whole report testable as a snapshot.

use super::{Symbols, Theme};

/// A titled group of key/value rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Section {
    /// The section's title, printed as written in upper case.
    pub(crate) title: &'static str,
    /// The rows: a label and a value. An empty value prints the label alone.
    pub(crate) rows: Vec<(String, String)>,
}

impl Section {
    /// A section from its title and rows.
    #[must_use]
    pub(crate) fn new(title: &'static str, rows: Vec<(String, String)>) -> Self {
        Section { title, rows }
    }
}

/// One stage of the work a command did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Stage {
    /// A short identifier, used in the one-line form (`SOURCE`, `LEX`, …).
    pub(crate) id: &'static str,
    /// The label used when the stages are stacked.
    pub(crate) label: &'static str,
    /// What happened.
    pub(crate) status: super::Status,
    /// Anything worth saying beside it, such as a count.
    pub(crate) detail: Option<String>,
}

impl Stage {
    /// A stage with no detail.
    #[must_use]
    pub(crate) fn new(id: &'static str, label: &'static str, status: super::Status) -> Self {
        Stage {
            id,
            label,
            status,
            detail: None,
        }
    }

    /// The same stage, with a detail.
    #[must_use]
    pub(crate) fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

/// A dim horizontal rule, as wide as the report.
#[must_use]
pub(crate) fn render_rule(width: usize, theme: Theme) -> String {
    theme.paint(super::Role::Dim, &"─".repeat(width.max(1)))
}

/// Renders sections, aligned to their own widest key.
#[must_use]
pub(crate) fn render_sections(sections: &[Section], width: usize, theme: Theme) -> String {
    let mut out = String::new();
    for (index, section) in sections.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        out.push_str(&theme.paint(super::Role::Plain, section.title));
        out.push('\n');
        let widest = section
            .rows
            .iter()
            .map(|(key, _)| display_width(key))
            .max()
            .unwrap_or(0);
        for (key, value) in &section.rows {
            if value.is_empty() {
                out.push_str(&format!("  {key}\n"));
                continue;
            }
            let line_width = 2 + widest + 2 + display_width(value);
            if line_width > width {
                // Not enough room to align: the value goes on its own line
                // rather than being cut or pushed off the edge.
                out.push_str(&format!("  {key}\n"));
                out.push_str(&format!("    {value}\n"));
                continue;
            }
            let padding = " ".repeat(widest.saturating_sub(display_width(key)));
            out.push_str(&format!("  {key}{padding}  {value}\n"));
        }
    }
    out
}

/// Renders the stages of one command, on one line when they fit.
///
/// The one-line form needs its own width: `✓ SOURCE ── ✓ LEX ── ● PARSE`. When
/// that does not fit, the same stages are stacked, which is not a degraded
/// rendering — it is the same information for a smaller window.
#[must_use]
pub(crate) fn render_stages(
    stages: &[Stage],
    width: usize,
    symbols: Symbols,
    theme: Theme,
) -> String {
    if stages.is_empty() {
        return String::new();
    }
    let one_line = one_line_width(stages, symbols);
    if one_line <= width {
        let mut out = String::new();
        for (index, stage) in stages.iter().enumerate() {
            if index > 0 {
                out.push(' ');
                out.push_str(&theme.paint(super::Role::Dim, symbols.connection()));
                out.push(' ');
            }
            out.push_str(&painted_glyph(stage.status, symbols, theme));
            out.push(' ');
            out.push_str(&theme.paint(super::Role::Plain, stage.id));
            if let Some(detail) = &stage.detail {
                out.push(' ');
                out.push_str(&theme.paint(super::Role::Dim, detail));
            }
        }
        out.push('\n');
        return out;
    }

    let mut out = String::new();
    // Padding is only needed to line up details. Without them, trailing spaces
    // would be noise in every line of a narrow report — and noise in a diff.
    let aligns_details = stages.iter().any(|stage| stage.detail.is_some());
    let widest = stages
        .iter()
        .map(|stage| display_width(stage.label))
        .max()
        .unwrap_or(0);
    for stage in stages {
        let padding = if aligns_details && stage.detail.is_some() {
            " ".repeat(widest.saturating_sub(display_width(stage.label)))
        } else {
            // A line without a detail gets no padding: a trailing space is a
            // byte nobody asked for, and it shows up in every diff.
            String::new()
        };
        out.push_str(&format!(
            "  {} {}{padding}",
            painted_glyph(stage.status, symbols, theme),
            stage.label,
        ));
        if let Some(detail) = &stage.detail {
            out.push(' ');
            out.push_str(&theme.paint(super::Role::Dim, detail));
        }
        out.push('\n');
    }
    out
}

/// The width the one-line form of these stages would occupy.
fn one_line_width(stages: &[Stage], symbols: Symbols) -> usize {
    let mut width = 0;
    for (index, stage) in stages.iter().enumerate() {
        if index > 0 {
            width += 1 + display_width(symbols.connection()) + 1;
        }
        width += display_width(symbols.glyph(stage.status)) + 1 + display_width(stage.id);
        if let Some(detail) = &stage.detail {
            width += 1 + display_width(detail);
        }
    }
    width
}

fn painted_glyph(status: super::Status, symbols: Symbols, theme: Theme) -> String {
    theme.paint(status.role(), symbols.glyph(status))
}

/// The width of a string in columns.
///
/// Only the symbols this crate prints are considered: they are all one column
/// wide, so counting characters is right. A general Unicode width table is a
/// dependency, and this layer does not need one to draw its own eight glyphs.
fn display_width(text: &str) -> usize {
    text.chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::Status;

    fn stages() -> Vec<Stage> {
        vec![
            Stage::new("SOURCE", "source", Status::Success),
            Stage::new("LEX", "lex", Status::Success),
            Stage::new("PARSE", "parse", Status::Active),
            Stage::new("HIR", "hir", Status::Pending),
        ]
    }

    #[test]
    fn a_wide_terminal_draws_the_stages_on_one_line() {
        let rendered = render_stages(&stages(), 80, Symbols::new(true), Theme::new(false));
        assert_eq!(
            rendered, "✓ SOURCE ── ✓ LEX ── ● PARSE ── ○ HIR\n",
            "{rendered}"
        );
    }

    #[test]
    fn a_narrow_terminal_stacks_the_same_stages() {
        // 34 columns of stages do not fit in 30, and a window is not widened to
        // suit a format. The same stages are stacked instead.
        let rendered = render_stages(&stages(), 30, Symbols::new(true), Theme::new(false));
        assert_eq!(
            rendered, "  ✓ source\n  ✓ lex\n  ● parse\n  ○ hir\n",
            "{rendered}"
        );
    }

    #[test]
    fn the_ascii_symbols_are_used_when_unicode_is_unavailable() {
        let rendered = render_stages(&stages(), 80, Symbols::new(false), Theme::new(false));
        assert_eq!(
            rendered, "+ SOURCE -- + LEX -- > PARSE -- . HIR\n",
            "{rendered}"
        );
    }

    #[test]
    fn a_section_aligns_on_its_own_widest_key() {
        let section = Section::new(
            "SUMMARY",
            vec![
                ("errors".to_string(), "0".to_string()),
                ("warnings".to_string(), "0".to_string()),
            ],
        );
        let rendered = render_sections(&[section], 80, Theme::new(false));
        assert_eq!(
            rendered, "SUMMARY\n  errors    0\n  warnings  0\n",
            "{rendered}"
        );
    }

    #[test]
    fn a_value_that_does_not_fit_goes_on_its_own_line() {
        let section = Section::new(
            "FILE",
            vec![(
                "path".to_string(),
                "examples/03-types/main.nudo".to_string(),
            )],
        );
        let rendered = render_sections(&[section], 24, Theme::new(false));
        assert_eq!(
            rendered, "FILE\n  path\n    examples/03-types/main.nudo\n",
            "{rendered}"
        );
    }

    #[test]
    fn a_row_with_no_value_prints_its_label_alone() {
        let section = Section::new("PIPELINE", vec![("source".to_string(), String::new())]);
        let rendered = render_sections(&[section], 80, Theme::new(false));
        assert_eq!(rendered, "PIPELINE\n  source\n", "{rendered}");
    }

    #[test]
    fn no_escape_bytes_reach_a_colorless_rendering() {
        let rendered = render_sections(
            &[Section::new(
                "SUMMARY",
                vec![("errors".to_string(), "1".to_string())],
            )],
            80,
            Theme::new(false),
        ) + &render_stages(&stages(), 80, Symbols::new(true), Theme::new(false));
        assert!(!rendered.contains('\x1b'));
    }

    #[test]
    fn a_rule_is_as_wide_as_asked_and_carries_no_escape_in_a_pipe() {
        let rule = render_rule(10, Theme::new(false));
        assert_eq!(rule, "─".repeat(10));
        assert!(!rule.contains('\x1b'));
    }
}
