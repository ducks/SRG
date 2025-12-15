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
}

impl Field {
  pub fn new(parts: Vec<FieldPart>) -> Self {
    Self { parts }
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

    for line in content.lines() {
      if line.trim().is_empty() {
        continue;
      }

      let indent_level = line.len() - line.trim_start().len();
      let trimmed = line.trim();

      if indent_level == 0 {
        if let Some(section) = current_section.take() {
          sections.push(section);
        }
        current_section = Some(Section {
          name: trimmed.to_string(),
          fields: Vec::new(),
        });
      } else if indent_level >= 2 {
        if let Some(ref mut section) = current_section {
          let parts = parse_field_parts(trimmed);
          section.fields.push(Field::new(parts));
        }
      }
    }

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
