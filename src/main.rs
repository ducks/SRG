use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

pub mod build;
pub mod layout;

/// Static Resume Generator - Build HTML and PDF resumes from JOBL files
#[derive(Parser, Debug)]
#[command(name = "srg")]
#[command(about = "Static Resume Generator", long_about = None)]
struct Args {
    /// Input JOBL file
    #[arg(short, long, value_name = "FILE")]
    input: PathBuf,

    /// Output directory
    #[arg(short, long, value_name = "DIR", default_value = "dist")]
    out: PathBuf,

    /// Theme name (includes both layout and CSS)
    #[arg(short, long)]
    theme: Option<String>,

    /// Custom layout file (optional, overrides theme layout)
    #[arg(short, long, value_name = "FILE")]
    layout: Option<PathBuf>,

    /// Custom CSS file (optional, loaded after theme CSS or standalone if no theme)
    #[arg(short, long, value_name = "FILE")]
    css: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Parse and validate JOBL file
    let doc = jobl::parse_file(&args.input)
        .map_err(|errors| {
            eprintln!("Validation errors in {}:", args.input.display());
            for err in &errors {
                eprintln!("  - {}", err);
            }
            anyhow::anyhow!("Failed to parse JOBL file")
        })?;

    // Determine theme (default to "minimal" if neither theme nor CSS specified)
    let theme = args.theme.as_deref().or(if args.css.is_none() {
        Some("minimal")
    } else {
        None
    });

    // Load layout - either from custom file or from theme
    let layout = match &args.layout {
        Some(path) => layout::Layout::from_file(path)
            .context("Failed to load layout file")?,
        None => {
            if let Some(theme_name) = theme {
                layout::Layout::from_theme(theme_name)
                    .context("Failed to load theme layout")?
            } else {
                layout::Layout::default()
            }
        }
    };

    // Build outputs
    build::build_resume(&doc, &args.out, theme, &layout, args.css.as_deref())
        .context("Failed to build resume")?;

    println!("Resume built successfully:");
    println!("  HTML: {}/index.html", args.out.display());
    println!("  PDF:  {}/resume.pdf", args.out.display());

    Ok(())
}
