use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

mod build;

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

    // Build outputs
    build::build_resume(&doc, &args.out, &args.template)
        .context("Failed to build resume")?;

    println!("Resume built successfully:");
    println!("  HTML: {}/index.html", args.out.display());
    println!("  PDF:  {}/resume.pdf", args.out.display());

    Ok(())
}
