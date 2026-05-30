//! Theme registry — auto-generated at compile time by `build.rs`.
//!
//! Every directory under `src/layouts/` containing both
//! `layout.resume` and `style.css` is registered as a theme.
//! Any files under `<theme>/fonts/` are bundled as font assets
//! and exposed via `fonts_for`. Adding a new theme is a matter
//! of dropping the directory into `src/layouts/` and rebuilding.
//! No registration code to edit.

include!(concat!(env!("OUT_DIR"), "/themes.rs"));
