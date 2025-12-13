use anyhow::{Context, Result};
use jobl::JoblDocument;
use std::fs;
use std::path::Path;

/// Build HTML and PDF resume from JOBL document
pub fn build_resume(
    doc: &JoblDocument,
    out_dir: &Path,
    template: &str,
) -> Result<()> {
    // Create output directory
    fs::create_dir_all(out_dir)
        .context("Failed to create output directory")?;

    // Generate HTML
    let html = generate_html(doc, template)?;
    let html_path = out_dir.join("index.html");
    fs::write(&html_path, html)
        .context("Failed to write HTML file")?;

    // Generate PDF (placeholder for now)
    let pdf_path = out_dir.join("resume.pdf");
    fs::write(&pdf_path, b"PDF generation not implemented yet")
        .context("Failed to write PDF file")?;

    Ok(())
}

/// Generate HTML from JOBL document
fn generate_html(doc: &JoblDocument, template: &str) -> Result<String> {
    match template {
        "minimal" => generate_minimal_html(doc),
        _ => anyhow::bail!("Unknown template: {}", template),
    }
}

/// Generate minimal HTML template
fn generate_minimal_html(doc: &JoblDocument) -> Result<String> {
    let mut html = String::new();

    html.push_str("<!DOCTYPE html>\n");
    html.push_str("<html lang=\"en\">\n");
    html.push_str("<head>\n");
    html.push_str("  <meta charset=\"UTF-8\">\n");
    html.push_str(
        "  <meta name=\"viewport\" content=\"width=device-width, \
         initial-scale=1.0\">\n",
    );
    html.push_str(&format!("  <title>{}</title>\n", doc.person.name));
    html.push_str("  <style>\n");
    html.push_str(include_str!("templates/minimal.css"));
    html.push_str("  </style>\n");
    html.push_str("</head>\n");
    html.push_str("<body>\n");
    html.push_str("  <main>\n");

    // Header
    html.push_str("    <header>\n");
    html.push_str(&format!("      <h1>{}</h1>\n", doc.person.name));
    if let Some(headline) = &doc.person.headline {
        html.push_str(
            &format!("      <p class=\"headline\">{}</p>\n", headline),
        );
    }
    html.push_str("      <div class=\"contact\">\n");
    if let Some(email) = &doc.person.email {
        html.push_str(
            &format!("        <span>{}</span>\n", escape_html(email)),
        );
    }
    if let Some(phone) = &doc.person.phone {
        html.push_str(
            &format!("        <span>{}</span>\n", escape_html(phone)),
        );
    }
    if let Some(location) = &doc.person.location {
        html.push_str(
            &format!("        <span>{}</span>\n", escape_html(location)),
        );
    }
    if let Some(website) = &doc.person.website {
        html.push_str(&format!(
            "        <a href=\"{}\">{}</a>\n",
            escape_html(website),
            escape_html(website)
        ));
    }
    html.push_str("      </div>\n");
    html.push_str("    </header>\n");

    // Summary
    if let Some(summary) = &doc.person.summary {
        html.push_str("    <section>\n");
        html.push_str("      <h2>Summary</h2>\n");
        html.push_str(
            &format!("      <p>{}</p>\n", escape_html(summary)),
        );
        html.push_str("    </section>\n");
    }

    // Skills
    if let Some(skills) = &doc.skills {
        if !skills.is_empty() {
            html.push_str("    <section>\n");
            html.push_str("      <h2>Skills</h2>\n");
            for (category, items) in skills {
                html.push_str(&format!(
                    "      <p><strong>{}:</strong> {}</p>\n",
                    escape_html(category),
                    items
                        .iter()
                        .map(|s| escape_html(s))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            html.push_str("    </section>\n");
        }
    }

    // Experience
    if !doc.experience.is_empty() {
        html.push_str("    <section>\n");
        html.push_str("      <h2>Experience</h2>\n");
        for exp in &doc.experience {
            html.push_str("      <div class=\"entry\">\n");
            html.push_str(&format!(
                "        <h3>{}</h3>\n",
                escape_html(&exp.title)
            ));
            html.push_str(&format!(
                "        <p class=\"company\">{}</p>\n",
                escape_html(&exp.company)
            ));
            if exp.start.is_some() || exp.end.is_some() {
                let start = exp.start.as_deref().unwrap_or("");
                let end = exp.end.as_deref().unwrap_or("");
                html.push_str(&format!(
                    "        <p class=\"dates\">{} - {}</p>\n",
                    escape_html(start),
                    escape_html(end)
                ));
            }
            if let Some(summary) = &exp.summary {
                html.push_str(
                    &format!("        <p>{}</p>\n", escape_html(summary)),
                );
            }
            if !exp.highlights.is_empty() {
                html.push_str("        <ul>\n");
                for highlight in &exp.highlights {
                    html.push_str(&format!(
                        "          <li>{}</li>\n",
                        escape_html(highlight)
                    ));
                }
                html.push_str("        </ul>\n");
            }
            html.push_str("      </div>\n");
        }
        html.push_str("    </section>\n");
    }

    // Projects
    if !doc.projects.is_empty() {
        html.push_str("    <section>\n");
        html.push_str("      <h2>Projects</h2>\n");
        for proj in &doc.projects {
            html.push_str("      <div class=\"entry\">\n");
            html.push_str(&format!(
                "        <h3>{}</h3>\n",
                escape_html(&proj.name)
            ));
            if let Some(url) = &proj.url {
                html.push_str(&format!(
                    "        <p><a href=\"{}\">{}</a></p>\n",
                    escape_html(url),
                    escape_html(url)
                ));
            }
            if let Some(summary) = &proj.summary {
                html.push_str(
                    &format!("        <p>{}</p>\n", escape_html(summary)),
                );
            }
            html.push_str("      </div>\n");
        }
        html.push_str("    </section>\n");
    }

    // Education
    if !doc.education.is_empty() {
        html.push_str("    <section>\n");
        html.push_str("      <h2>Education</h2>\n");
        for edu in &doc.education {
            html.push_str("      <div class=\"entry\">\n");
            html.push_str(&format!(
                "        <h3>{}</h3>\n",
                escape_html(&edu.degree)
            ));
            html.push_str(&format!(
                "        <p class=\"company\">{}</p>\n",
                escape_html(&edu.institution)
            ));
            if edu.start.is_some() || edu.end.is_some() {
                let start = edu.start.as_deref().unwrap_or("");
                let end = edu.end.as_deref().unwrap_or("");
                html.push_str(&format!(
                    "        <p class=\"dates\">{} - {}</p>\n",
                    escape_html(start),
                    escape_html(end)
                ));
            }
            if !edu.details.is_empty() {
                html.push_str("        <ul>\n");
                for detail in &edu.details {
                    html.push_str(&format!(
                        "          <li>{}</li>\n",
                        escape_html(detail)
                    ));
                }
                html.push_str("        </ul>\n");
            }
            html.push_str("      </div>\n");
        }
        html.push_str("    </section>\n");
    }

    html.push_str("  </main>\n");
    html.push_str("</body>\n");
    html.push_str("</html>\n");

    Ok(html)
}

/// Escape HTML special characters
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
