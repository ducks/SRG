use anyhow::{Context, Result};
use headless_chrome::types::PrintToPdfOptions;
use headless_chrome::Browser;
use jobl::JoblDocument;
use std::fs;
use std::path::Path;

use crate::layout::{FieldPart, Layout};

/// Build HTML and PDF resume from JOBL document
pub fn build_resume(
    doc: &JoblDocument,
    out_dir: &Path,
    template: &str,
    layout: &Layout,
) -> Result<()> {
    // Create output directory
    fs::create_dir_all(out_dir)
        .context("Failed to create output directory")?;

    // Generate HTML
    let html = generate_html(doc, template, layout)?;
    let html_path = out_dir.join("index.html");
    fs::write(&html_path, html)
        .context("Failed to write HTML file")?;

    // Generate PDF from HTML
    let pdf_path = out_dir.join("resume.pdf");
    generate_pdf(&html_path, &pdf_path)
        .context("Failed to generate PDF")?;

    Ok(())
}

/// Generate HTML from JOBL document
fn generate_html(
    doc: &JoblDocument,
    template: &str,
    layout: &Layout,
) -> Result<String> {
    match template {
        "minimal" => generate_minimal_html(doc, layout),
        _ => anyhow::bail!("Unknown template: {}", template),
    }
}

/// Generate HTML for testing (public for integration tests)
pub fn generate_test_html(
    doc: &JoblDocument,
    template: &str,
    layout: &Layout,
) -> Result<String> {
    generate_html(doc, template, layout)
}

/// Generate minimal HTML template
fn generate_minimal_html(
    doc: &JoblDocument,
    layout: &Layout,
) -> Result<String> {
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

    for section in &layout.sections {
        match section.name.as_str() {
            "person" => {
                render_person_section(&mut html, doc, section);
            }
            "summary" => {
                render_summary_section(&mut html, doc);
            }
            "skills" => {
                render_skills_section(&mut html, doc);
            }
            "experience" => {
                render_experience_section(&mut html, doc, section);
            }
            "projects" => {
                render_projects_section(&mut html, doc, section);
            }
            "education" => {
                render_education_section(&mut html, doc, section);
            }
            _ => {}
        }
    }

    html.push_str("  </main>\n");
    html.push_str("</body>\n");
    html.push_str("</html>\n");

    Ok(html)
}

fn render_person_section(
    html: &mut String,
    doc: &JoblDocument,
    section: &crate::layout::Section,
) {
    html.push_str("    <header id=\"person\" class=\"section section-person\">\n");

    for field in &section.fields {
        render_person_field(html, doc, field);
    }

    html.push_str("    </header>\n");
}

fn render_person_field(
    html: &mut String,
    doc: &JoblDocument,
    field: &crate::layout::Field,
) {
    // If field has single part that's a known field, render it specially
    if field.parts.len() == 1 {
        if let FieldPart::Field(name) = &field.parts[0] {
            match name.as_str() {
                "name" => {
                    html.push_str(
                        &format!("      <h1 class=\"person-name\">{}</h1>\n", doc.person.name),
                    );
                    return;
                }
                "headline" => {
                    if let Some(headline) = &doc.person.headline {
                        html.push_str(&format!(
                            "      <p class=\"person-headline\">{}</p>\n",
                            escape_html(headline)
                        ));
                    }
                    return;
                }
                "email" => {
                    if let Some(email) = &doc.person.email {
                        html.push_str(&format!(
                            "      <span class=\"person-email\">{}</span>\n",
                            escape_html(email)
                        ));
                    }
                    return;
                }
                "phone" => {
                    if let Some(phone) = &doc.person.phone {
                        html.push_str(&format!(
                            "      <span class=\"person-phone\">{}</span>\n",
                            escape_html(phone)
                        ));
                    }
                    return;
                }
                "location" => {
                    if let Some(location) = &doc.person.location {
                        html.push_str(&format!(
                            "      <span class=\"person-location\">{}</span>\n",
                            escape_html(location)
                        ));
                    }
                    return;
                }
                "website" => {
                    if let Some(website) = &doc.person.website {
                        html.push_str(&format!(
                            "      <a class=\"person-website\" href=\"{}\">{}</a>\n",
                            escape_html(website),
                            escape_html(website)
                        ));
                    }
                    return;
                }
                _ => {}
            }
        }
    }

    // Otherwise, render as inline mixed content
    html.push_str("      <p>");
    for part in &field.parts {
        match part {
            FieldPart::Literal(text) => {
                html.push_str(&escape_html(text));
            }
            FieldPart::Field(name) => {
                let value = get_person_field_value(doc, name);
                if let Some(v) = value {
                    html.push_str(&escape_html(&v));
                }
            }
        }
    }
    html.push_str("</p>\n");
}

fn get_person_field_value(doc: &JoblDocument, field: &str) -> Option<String> {
    match field {
        "name" => Some(doc.person.name.clone()),
        "headline" => doc.person.headline.clone(),
        "email" => doc.person.email.clone(),
        "phone" => doc.person.phone.clone(),
        "location" => doc.person.location.clone(),
        "website" => doc.person.website.clone(),
        _ => None,
    }
}

fn render_summary_section(html: &mut String, doc: &JoblDocument) {
    if let Some(summary) = &doc.person.summary {
        html.push_str("    <section id=\"summary\" class=\"section section-summary\">\n");
        html.push_str("      <h2>Summary</h2>\n");
        html.push_str(
            &format!("      <p class=\"summary-text\">{}</p>\n", escape_html(summary)),
        );
        html.push_str("    </section>\n");
    }
}

fn render_skills_section(html: &mut String, doc: &JoblDocument) {
    if let Some(skills) = &doc.skills {
        if !skills.is_empty() {
            html.push_str("    <section id=\"skills\" class=\"section section-skills\">\n");
            html.push_str("      <h2>Skills</h2>\n");
            for (category, items) in skills {
                html.push_str(&format!(
                    "      <p class=\"skills-category\"><strong class=\"skills-category-name\">{}:</strong> <span class=\"skills-items\">{}</span></p>\n",
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
}

fn render_experience_section(
    html: &mut String,
    doc: &JoblDocument,
    section: &crate::layout::Section,
) {
    if doc.experience.is_empty() {
        return;
    }

    html.push_str("    <section id=\"experience\" class=\"section section-experience\">\n");
    html.push_str("      <h2>Experience</h2>\n");

    for exp in &doc.experience {
        html.push_str("      <div class=\"experience-item\">\n");

        for field in &section.fields {
            render_experience_field(html, exp, field);
        }

        html.push_str("      </div>\n");
    }

    html.push_str("    </section>\n");
}

fn render_experience_field(
    html: &mut String,
    exp: &jobl::ExperienceItem,
    field: &crate::layout::Field,
) {
    if field.parts.is_empty() {
        return;
    }

    // Check for single-field special cases
    if field.parts.len() == 1 {
        if let FieldPart::Field(name) = &field.parts[0] {
            match name.as_str() {
                "title" => {
                    html.push_str(&format!(
                        "        <h3 class=\"experience-title\">{}</h3>\n",
                        escape_html(&exp.title)
                    ));
                    return;
                }
                "company" => {
                    html.push_str(&format!(
                        "        <p class=\"experience-company\">{}</p>\n",
                        escape_html(&exp.company)
                    ));
                    return;
                }
                "summary" => {
                    if let Some(summary) = &exp.summary {
                        html.push_str(&format!(
                            "        <p class=\"experience-summary\">{}</p>\n",
                            escape_html(summary)
                        ));
                    }
                    return;
                }
                "highlights" => {
                    if !exp.highlights.is_empty() {
                        html.push_str("        <ul class=\"experience-highlights\">\n");
                        for highlight in &exp.highlights {
                            html.push_str(&format!(
                                "          <li>{}</li>\n",
                                escape_html(highlight)
                            ));
                        }
                        html.push_str("        </ul>\n");
                    }
                    return;
                }
                _ => {}
            }
        }
    }

    // Render as inline mixed content
    html.push_str("        <p>");
    for part in &field.parts {
        match part {
            FieldPart::Literal(text) => {
                html.push_str(&escape_html(text));
            }
            FieldPart::Field(name) => {
                let value = get_experience_field_value(exp, name);
                if let Some(v) = value {
                    html.push_str(&escape_html(&v));
                }
            }
        }
    }
    html.push_str("</p>\n");
}

fn get_experience_field_value(
    exp: &jobl::ExperienceItem,
    field: &str,
) -> Option<String> {
    match field {
        "title" => Some(exp.title.clone()),
        "company" => Some(exp.company.clone()),
        "location" => exp.location.clone(),
        "start" => exp.start.clone(),
        "end" => exp.end.clone(),
        "summary" => exp.summary.clone(),
        _ => None,
    }
}

fn render_projects_section(
    html: &mut String,
    doc: &JoblDocument,
    section: &crate::layout::Section,
) {
    if doc.projects.is_empty() {
        return;
    }

    html.push_str("    <section id=\"projects\" class=\"section section-projects\">\n");
    html.push_str("      <h2>Projects</h2>\n");

    for proj in &doc.projects {
        html.push_str("      <div class=\"projects-item\">\n");

        for field in &section.fields {
            render_project_field(html, proj, field);
        }

        html.push_str("      </div>\n");
    }

    html.push_str("    </section>\n");
}

fn render_project_field(
    html: &mut String,
    proj: &jobl::ProjectItem,
    field: &crate::layout::Field,
) {
    if field.parts.is_empty() {
        return;
    }

    if field.parts.len() == 1 {
        if let FieldPart::Field(name) = &field.parts[0] {
            match name.as_str() {
                "name" => {
                    html.push_str(&format!(
                        "        <h3 class=\"projects-name\">{}</h3>\n",
                        escape_html(&proj.name)
                    ));
                    return;
                }
                "url" => {
                    if let Some(url) = &proj.url {
                        html.push_str(&format!(
                            "        <p class=\"projects-url\"><a href=\"{}\">{}</a></p>\n",
                            escape_html(url),
                            escape_html(url)
                        ));
                    }
                    return;
                }
                "summary" => {
                    if let Some(summary) = &proj.summary {
                        html.push_str(&format!(
                            "        <p class=\"projects-summary\">{}</p>\n",
                            escape_html(summary)
                        ));
                    }
                    return;
                }
                _ => {}
            }
        }
    }

    html.push_str("        <p>");
    for part in &field.parts {
        match part {
            FieldPart::Literal(text) => {
                html.push_str(&escape_html(text));
            }
            FieldPart::Field(name) => {
                let value = get_project_field_value(proj, name);
                if let Some(v) = value {
                    html.push_str(&escape_html(&v));
                }
            }
        }
    }
    html.push_str("</p>\n");
}

fn get_project_field_value(
    proj: &jobl::ProjectItem,
    field: &str,
) -> Option<String> {
    match field {
        "name" => Some(proj.name.clone()),
        "url" => proj.url.clone(),
        "summary" => proj.summary.clone(),
        _ => None,
    }
}

fn render_education_section(
    html: &mut String,
    doc: &JoblDocument,
    section: &crate::layout::Section,
) {
    if doc.education.is_empty() {
        return;
    }

    html.push_str("    <section id=\"education\" class=\"section section-education\">\n");
    html.push_str("      <h2>Education</h2>\n");

    for edu in &doc.education {
        html.push_str("      <div class=\"education-item\">\n");

        for field in &section.fields {
            render_education_field(html, edu, field);
        }

        html.push_str("      </div>\n");
    }

    html.push_str("    </section>\n");
}

fn render_education_field(
    html: &mut String,
    edu: &jobl::EducationItem,
    field: &crate::layout::Field,
) {
    if field.parts.is_empty() {
        return;
    }

    if field.parts.len() == 1 {
        if let FieldPart::Field(name) = &field.parts[0] {
            match name.as_str() {
                "degree" => {
                    html.push_str(&format!(
                        "        <h3 class=\"education-degree\">{}</h3>\n",
                        escape_html(&edu.degree)
                    ));
                    return;
                }
                "institution" => {
                    html.push_str(&format!(
                        "        <p class=\"education-institution\">{}</p>\n",
                        escape_html(&edu.institution)
                    ));
                    return;
                }
                "details" => {
                    if !edu.details.is_empty() {
                        html.push_str("        <ul class=\"education-details\">\n");
                        for detail in &edu.details {
                            html.push_str(&format!(
                                "          <li>{}</li>\n",
                                escape_html(detail)
                            ));
                        }
                        html.push_str("        </ul>\n");
                    }
                    return;
                }
                _ => {}
            }
        }
    }

    html.push_str("        <p>");
    for part in &field.parts {
        match part {
            FieldPart::Literal(text) => {
                html.push_str(&escape_html(text));
            }
            FieldPart::Field(name) => {
                let value = get_education_field_value(edu, name);
                if let Some(v) = value {
                    html.push_str(&escape_html(&v));
                }
            }
        }
    }
    html.push_str("</p>\n");
}

fn get_education_field_value(
    edu: &jobl::EducationItem,
    field: &str,
) -> Option<String> {
    match field {
        "degree" => Some(edu.degree.clone()),
        "institution" => Some(edu.institution.clone()),
        "location" => edu.location.clone(),
        "start" => edu.start.clone(),
        "end" => edu.end.clone(),
        _ => None,
    }
}

/// Generate PDF from HTML file using headless Chrome
fn generate_pdf(html_path: &Path, pdf_path: &Path) -> Result<()> {
    let browser = Browser::default()
        .context("Failed to launch Chrome browser")?;

    let tab = browser.new_tab()
        .context("Failed to create new browser tab")?;

    // Convert path to file:// URL
    let html_url = format!(
        "file://{}",
        html_path.canonicalize()
            .context("Failed to resolve HTML path")?
            .display()
    );

    tab.navigate_to(&html_url)
        .context("Failed to navigate to HTML file")?;

    tab.wait_until_navigated()
        .context("Failed to wait for page load")?;

    let pdf_data = tab.print_to_pdf(Some(PrintToPdfOptions {
        landscape: Some(false),
        display_header_footer: Some(false),
        print_background: Some(true),
        scale: Some(1.0),
        paper_width: Some(8.5),
        paper_height: Some(11.0),
        margin_top: Some(0.4),
        margin_bottom: Some(0.4),
        margin_left: Some(0.4),
        margin_right: Some(0.4),
        page_ranges: None,
        ignore_invalid_page_ranges: None,
        header_template: None,
        footer_template: None,
        prefer_css_page_size: Some(false),
        transfer_mode: None,
        generate_document_outline: None,
        generate_tagged_pdf: None,
    })).context("Failed to generate PDF")?;

    fs::write(pdf_path, pdf_data)
        .context("Failed to write PDF file")?;

    Ok(())
}

/// Escape HTML special characters
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
