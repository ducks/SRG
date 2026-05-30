use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

pub mod build;
pub mod config;
pub mod layout;
pub mod themes;

/// Static Resume Generator - Build HTML and PDF resumes from JOBL files
#[derive(Parser, Debug)]
#[command(name = "srg")]
#[command(about = "Static Resume Generator", long_about = None)]
struct Args {
    /// Input JOBL file
    #[arg(short, long, value_name = "FILE")]
    input: PathBuf,

    /// Output directory. Overrides `out` in srg.toml. Defaults to "dist".
    #[arg(short, long, value_name = "DIR")]
    out: Option<PathBuf>,

    /// Theme name. Overrides `theme` in srg.toml.
    #[arg(short, long)]
    theme: Option<String>,

    /// Custom layout file. Overrides `layout` in srg.toml.
    #[arg(short, long, value_name = "FILE")]
    layout: Option<PathBuf>,

    /// Custom CSS file. Overrides `css` in srg.toml.
    #[arg(short, long, value_name = "FILE")]
    css: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Parse and validate JOBL file.
    let doc = jobl::parse_file(&args.input).map_err(|errors| {
        eprintln!("Validation errors in {}:", args.input.display());
        for err in &errors {
            eprintln!("  - {}", err);
        }
        anyhow::anyhow!("Failed to parse JOBL file")
    })?;

    // Load srg.toml from the JOBL file's directory if present. Missing
    // is OK; malformed is fatal.
    let loaded = config::Config::load_for(&args.input)?;

    // Resolve each setting with the precedence:
    //   CLI flag  >  srg.toml  >  built-in default
    // The closure resolves relative paths in srg.toml against the
    // directory the config was loaded from so the config stays
    // portable across working directories.
    let resolve = |p: PathBuf| -> PathBuf {
        match &loaded {
            Some(l) => l.resolve(&p),
            None => p,
        }
    };

    let theme = args
        .theme
        .clone()
        .or_else(|| loaded.as_ref().and_then(|l| l.config.theme.clone()));

    let layout_path = args
        .layout
        .clone()
        .or_else(|| loaded.as_ref().and_then(|l| l.config.layout.clone()).map(resolve));

    let css_path = args
        .css
        .clone()
        .or_else(|| loaded.as_ref().and_then(|l| l.config.css.clone()).map(resolve));

    let out_dir = args
        .out
        .clone()
        .or_else(|| {
            loaded
                .as_ref()
                .and_then(|l| l.config.out.clone())
                .map(resolve)
        })
        .unwrap_or_else(|| PathBuf::from("dist"));

    // Default to the "minimal" theme only when nothing else was
    // chosen. A custom CSS by itself implies "no theme, just this CSS,"
    // which matches the original behavior.
    let theme = theme.or_else(|| if css_path.is_none() { Some("minimal".into()) } else { None });

    // Load layout — either from a custom file or from the theme.
    let layout = match layout_path.as_deref() {
        Some(path) => layout::Layout::from_file(path).context("Failed to load layout file")?,
        None => match theme.as_deref() {
            Some(theme_name) => layout::Layout::from_theme(theme_name)
                .context("Failed to load theme layout")?,
            None => layout::Layout::default(),
        },
    };

    build::build_resume(&doc, &out_dir, theme.as_deref(), &layout, css_path.as_deref())
        .context("Failed to build resume")?;

    println!("Resume built successfully:");
    println!("  HTML: {}/index.html", out_dir.display());
    println!("  PDF:  {}/resume.pdf", out_dir.display());

    Ok(())
}
