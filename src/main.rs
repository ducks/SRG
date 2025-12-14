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

    /// Template name
    #[arg(short, long, default_value = "minimal")]
    template: String,

    /// Layout file (optional, uses default if not specified)
    #[arg(short, long, value_name = "FILE")]
    layout: Option<PathBuf>,
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

    // Load layout
    let layout = match &args.layout {
        Some(path) => layout::Layout::from_file(path)
            .context("Failed to load layout file")?,
        None => layout::Layout::default(),
    };

    // Build outputs
    build::build_resume(&doc, &args.out, &args.template, &layout)
        .context("Failed to build resume")?;

    println!("Resume built successfully:");
    println!("  HTML: {}/index.html", args.out.display());
    println!("  PDF:  {}/resume.pdf", args.out.display());

    Ok(())
}
