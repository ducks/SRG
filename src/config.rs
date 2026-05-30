//! Optional `srg.toml` config file loaded alongside the resume.
//!
//! The config lets a resume repo declare its presentation choices
//! (theme, layout overrides, output dir) without baking them into
//! the JOBL file or relying on CLI flags. JOBL stays the content
//! source of truth; `srg.toml` stays the presentation source of
//! truth.
//!
//! Lookup is conservative: srg looks for a file literally named
//! `srg.toml` in the same directory as the input JOBL file. If
//! it's absent the program continues with CLI defaults. If it's
//! present but malformed we fail loudly so the user knows.
//!
//! CLI flags always override config values. The precedence chain is:
//!
//!   CLI flag  >  srg.toml  >  built-in default
//!
//! Keeping CLI strictly highest means a release pipeline can pass
//! `--theme classic` to preview an alternate look without editing
//! the resume repo.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// On-disk shape of `srg.toml`. Every field is optional. Unknown
/// fields are rejected so typos surface immediately rather than
/// silently doing nothing.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Theme name (e.g. "minimal", "jake", "classic"). Maps to a
    /// directory under `src/layouts/` built into the binary.
    pub theme: Option<String>,

    /// Path to a custom layout file. Overrides the theme's layout
    /// when set. Relative paths are resolved against the directory
    /// containing `srg.toml`.
    pub layout: Option<PathBuf>,

    /// Path to a custom CSS file appended to the theme's CSS (or
    /// used standalone if no theme is set). Relative paths are
    /// resolved against the directory containing `srg.toml`.
    pub css: Option<PathBuf>,

    /// Output directory for the rendered HTML + PDF. Relative
    /// paths resolve against the directory containing `srg.toml`,
    /// not the current working directory, so `srg.toml` files are
    /// portable.
    pub out: Option<PathBuf>,
}

impl Config {
    /// Look for `srg.toml` next to the JOBL input file. Returns
    /// `Ok(None)` if the file isn't present, an error if it's
    /// present but malformed.
    pub fn load_for(input_path: &Path) -> Result<Option<LoadedConfig>> {
        let dir = input_path.parent().unwrap_or_else(|| Path::new("."));
        let candidate = dir.join("srg.toml");

        if !candidate.exists() {
            return Ok(None);
        }

        let body = std::fs::read_to_string(&candidate)
            .with_context(|| format!("reading {}", candidate.display()))?;
        let config: Config = toml::from_str(&body)
            .with_context(|| format!("parsing {}", candidate.display()))?;

        Ok(Some(LoadedConfig {
            config,
            base_dir: dir.to_path_buf(),
        }))
    }
}

/// A config plus the directory it was loaded from. The base
/// directory is needed to resolve relative paths so consumers don't
/// have to track it separately.
#[derive(Debug)]
pub struct LoadedConfig {
    pub config: Config,
    pub base_dir: PathBuf,
}

impl LoadedConfig {
    /// Resolve an optional relative path against the config's
    /// base directory. Absolute paths pass through unchanged.
    pub fn resolve(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.base_dir.join(path)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_returns_none_when_file_missing() {
        let dir = tempfile::TempDir::new().unwrap();
        let jobl_path = dir.path().join("resume.jobl");
        std::fs::write(&jobl_path, "").unwrap();

        let loaded = Config::load_for(&jobl_path).unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn load_parses_a_valid_config() {
        let dir = tempfile::TempDir::new().unwrap();
        let jobl_path = dir.path().join("resume.jobl");
        std::fs::write(&jobl_path, "").unwrap();
        std::fs::write(
            dir.path().join("srg.toml"),
            r#"
theme = "jake"
out = "build"
"#,
        )
        .unwrap();

        let loaded = Config::load_for(&jobl_path).unwrap().unwrap();
        assert_eq!(loaded.config.theme.as_deref(), Some("jake"));
        assert_eq!(loaded.config.out, Some(PathBuf::from("build")));
    }

    #[test]
    fn load_rejects_unknown_fields() {
        let dir = tempfile::TempDir::new().unwrap();
        let jobl_path = dir.path().join("resume.jobl");
        std::fs::write(&jobl_path, "").unwrap();
        std::fs::write(
            dir.path().join("srg.toml"),
            r#"
theme = "jake"
mistyped_field = "oops"
"#,
        )
        .unwrap();

        let err = Config::load_for(&jobl_path).unwrap_err();
        assert!(err.to_string().contains("srg.toml"));
    }

    #[test]
    fn resolve_keeps_absolute_paths_unchanged() {
        let loaded = LoadedConfig {
            config: Config::default(),
            base_dir: PathBuf::from("/some/dir"),
        };
        let abs = PathBuf::from("/etc/passwd");
        assert_eq!(loaded.resolve(&abs), abs);
    }

    #[test]
    fn resolve_joins_relative_paths_to_base_dir() {
        let loaded = LoadedConfig {
            config: Config::default(),
            base_dir: PathBuf::from("/some/dir"),
        };
        let rel = PathBuf::from("build");
        assert_eq!(loaded.resolve(&rel), PathBuf::from("/some/dir/build"));
    }
}
