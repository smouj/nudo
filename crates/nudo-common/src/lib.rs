//! Shared, dependency-free vocabulary used across the NUDO workspace.
//!
//! This crate holds the few values that must mean exactly one thing
//! everywhere: the language name, the toolchain version, the conventional file
//! names and the release channel. It has no dependencies, so any other crate
//! can depend on it without constraining the build graph.

use std::fmt;

/// The name of the language, spelled the way the brand spells it: `NUDO`.
///
/// In ordinary documentation sentences the project is written `Nudo`, and the
/// toolchain binary is `nudo`. No other variant is official.
pub const LANGUAGE_NAME: &str = "NUDO";

/// The toolchain binary name.
pub const TOOLCHAIN_NAME: &str = "nudo";

/// The source file extension for NUDO programs, without the leading dot.
///
/// `.nu` is deliberately **not** used: it is already associated with another
/// ecosystem, and a new language should not collide with it.
pub const SOURCE_EXTENSION: &str = "nudo";

/// The conventional manifest file name.
pub const MANIFEST_FILE: &str = "nudo.toml";

/// The conventional lockfile name.
pub const LOCKFILE: &str = "nudo.lock";

/// The version of the toolchain, taken from the workspace manifest.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The release channel of this toolchain.
pub const CHANNEL: Channel = Channel::PreAlpha;

/// A release channel.
///
/// NUDO is currently [`Channel::PreAlpha`]: the language is not stable, the
/// toolchain is incomplete, and nothing here is fit for production use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Channel {
    /// The language and the toolchain change without notice.
    PreAlpha,
    /// The language covers its first milestone set but carries no stability
    /// guarantees.
    Alpha,
    /// A stabilisation candidate: removing anything requires a NEP.
    Beta,
    /// Stable: breaking changes require a new edition.
    Stable,
}

impl Channel {
    /// The channel as lowercase text, as printed by `nudo --version`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Channel::PreAlpha => "pre-alpha",
            Channel::Alpha => "alpha",
            Channel::Beta => "beta",
            Channel::Stable => "stable",
        }
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Reports whether `path` names a NUDO source file.
///
/// The comparison is ASCII case-insensitive, because the same repository may
/// be checked out on case-insensitive file systems.
#[must_use]
pub fn has_source_extension(path: &str) -> bool {
    std::path::Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case(SOURCE_EXTENSION))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_source_extension() {
        assert!(has_source_extension("main.nudo"));
        assert!(has_source_extension("examples/00-hello-world/main.nudo"));
        assert!(has_source_extension("MAIN.NUDO"));
    }

    #[test]
    fn rejects_other_extensions() {
        assert!(!has_source_extension("main.nu"));
        assert!(!has_source_extension("main.rs"));
        assert!(!has_source_extension("nudo"));
        assert!(!has_source_extension("dir.nudo/main"));
    }

    #[test]
    fn channel_renders_as_text() {
        assert_eq!(Channel::PreAlpha.as_str(), "pre-alpha");
        assert_eq!(Channel::PreAlpha.to_string(), "pre-alpha");
    }

    #[test]
    fn version_is_present() {
        assert!(!VERSION.is_empty());
        assert!(VERSION.starts_with(char::is_numeric));
    }
}
