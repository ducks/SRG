use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

#[cfg(test)]
#[path = "layout_tests.rs"]
mod layout_tests;

#[derive(Debug, Clone)]
pub struct Layout {
  pub sections: Vec<Section>,
}

#[derive(Debug, Clone)]
pub struct Section {
  pub name: String,
  pub fields: Vec<FieldOrContainer>,
}

#[derive(Debug, Clone)]
pub enum FieldOrContainer {
  Field(Field),
  Container(Container),
}

#[derive(Debug, Clone)]
pub struct Container {
  pub class_name: String,
  pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldPart {
  Field(String),
  Literal(String),
}

#[derive(Debug, Clone)]
pub struct Field {
  pub parts: Vec<FieldPart>,
  pub class_name: Option<String>,
}

impl Field {
  pub fn new(parts: Vec<FieldPart>) -> Self {
    Self { parts, class_name: None }
  }

  pub fn with_class(parts: Vec<FieldPart>, class_name: String) -> Self {
    Self { parts, class_name: Some(class_name) }
  }
}

fn parse_field_parts(line: &str) -> Vec<FieldPart> {
  let mut parts = Vec::new();
  let mut chars = line.chars().peekable();
  let mut current = String::new();
  let mut in_quote = false;

  while let Some(ch) = chars.next() {
    match ch {
      '"' => {
        if in_quote {
          // End of quoted string
          parts.push(FieldPart::Literal(current.clone()));
          current.clear();
          in_quote = false;
        } else {
          // Start of quoted string
          if !current.is_empty() {
            parts.push(FieldPart::Field(current.clone()));
            current.clear();
          }
          in_quote = true;
        }
      }
      ' ' if !in_quote => {
        // Whitespace outside quotes - end current field
        if !current.is_empty() {
          parts.push(FieldPart::Field(current.clone()));
          current.clear();
        }
      }
      _ => {
        current.push(ch);
      }
    }
  }

  // Handle any remaining content
  if !current.is_empty() {
    if in_quote {
      // Unclosed quote - treat as literal
      parts.push(FieldPart::Literal(current));
    } else {
      parts.push(FieldPart::Field(current));
    }
  }

  parts
}

impl Layout {
  pub fn from_file(path: &Path) -> Result<Self> {
    let content =
      fs::read_to_string(path).context("Failed to read layout file")?;
    Self::parse(&content)
  }

  pub fn from_theme(theme: &str) -> Result<Self> {
    let content = match theme {
      "minimal" => include_str!("layouts/minimal/layout.resume"),
      "jake" => include_str!("layouts/jake/layout.resume"),
      _ => anyhow::bail!("Unknown theme: {}", theme),
    };
    Self::parse(content)
  }

  pub fn parse(content: &str) -> Result<Self> {
    let mut sections = Vec::new();
    let mut current_section: Option<Section> = None;
    let mut current_container: Option<Container> = None;

    for line in content.lines() {
      if line.trim().is_empty() {
        continue;
      }

      let indent_level = line.len() - line.trim_start().len();
      let trimmed = line.trim();

      if indent_level == 0 {
        // Close any open container
        if let (Some(container), Some(ref mut section)) = (current_container.take(), current_section.as_mut()) {
          section.fields.push(FieldOrContainer::Container(container));
        }

        // Close any open section
        if let Some(section) = current_section.take() {
          sections.push(section);
        }

        current_section = Some(Section {
          name: trimmed.to_string(),
          fields: Vec::new(),
        });
      } else if indent_level == 2 {
        // Close any open container first
        if let (Some(container), Some(ref mut section)) = (current_container.take(), current_section.as_mut()) {
          section.fields.push(FieldOrContainer::Container(container));
        }

        if let Some(ref mut section) = current_section {
          // Check if this is a container definition (ends with :)
          if trimmed.ends_with(':') {
            let container_name = trimmed.trim_end_matches(':').trim();
            if !container_name.is_empty() && !container_name.contains('"') && !container_name.contains(' ') {
              current_container = Some(Container {
                class_name: container_name.to_string(),
                fields: Vec::new(),
              });
              continue;
            }
          }

          // Check for custom class syntax: "class-name: field definition"
          if let Some(colon_pos) = trimmed.find(':') {
            let before_colon = &trimmed[..colon_pos].trim();
            let after_colon = &trimmed[colon_pos + 1..].trim();

            // Check if before_colon looks like a class name (no quotes or special chars)
            if !before_colon.is_empty() && !before_colon.contains('"') && !before_colon.contains(' ') && !after_colon.is_empty() {
              let parts = parse_field_parts(after_colon);
              section.fields.push(FieldOrContainer::Field(Field::with_class(parts, before_colon.to_string())));
              continue;
            }
          }

          // Default: regular field
          let parts = parse_field_parts(trimmed);
          section.fields.push(FieldOrContainer::Field(Field::new(parts)));
        }
      } else if indent_level >= 4 {
        // Add to current container if one exists, otherwise to section
        if let Some(ref mut container) = current_container {
          // Check for custom class syntax
          if let Some(colon_pos) = trimmed.find(':') {
            let before_colon = &trimmed[..colon_pos].trim();
            let after_colon = &trimmed[colon_pos + 1..].trim();

            if !before_colon.is_empty() && !before_colon.contains('"') && !before_colon.contains(' ') && !after_colon.is_empty() {
              let parts = parse_field_parts(after_colon);
              container.fields.push(Field::with_class(parts, before_colon.to_string()));
              continue;
            }
          }

          let parts = parse_field_parts(trimmed);
          container.fields.push(Field::new(parts));
        } else if let Some(ref mut section) = current_section {
          // Treat as regular field if no container
          let parts = parse_field_parts(trimmed);
          section.fields.push(FieldOrContainer::Field(Field::new(parts)));
        }
      }
    }

    // Close any remaining open container
    if let (Some(container), Some(ref mut section)) = (current_container.take(), current_section.as_mut()) {
      section.fields.push(FieldOrContainer::Container(container));
    }

    // Close any remaining open section
    if let Some(section) = current_section {
      sections.push(section);
    }

    Ok(Layout { sections })
  }

  pub fn default() -> Self {
    Self::parse(include_str!("layouts/minimal/layout.resume"))
      .expect("Default layout should be valid")
  }
}
