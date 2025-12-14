#[cfg(test)]
mod tests {
  use crate::layout::{FieldPart, Layout};

  #[test]
  fn test_parse_simple_layout() {
    let content = r#"
person
  name
  email

experience
  title
  company
"#;

    let layout = Layout::parse(content).unwrap();
    assert_eq!(layout.sections.len(), 2);

    assert_eq!(layout.sections[0].name, "person");
    assert_eq!(layout.sections[0].fields.len(), 2);
    assert_eq!(
      layout.sections[0].fields[0].parts,
      vec![FieldPart::Field("name".to_string())]
    );
    assert_eq!(
      layout.sections[0].fields[1].parts,
      vec![FieldPart::Field("email".to_string())]
    );

    assert_eq!(layout.sections[1].name, "experience");
    assert_eq!(layout.sections[1].fields.len(), 2);
    assert_eq!(
      layout.sections[1].fields[0].parts,
      vec![FieldPart::Field("title".to_string())]
    );
    assert_eq!(
      layout.sections[1].fields[1].parts,
      vec![FieldPart::Field("company".to_string())]
    );
  }

  #[test]
  fn test_parse_multi_part_field() {
    let content = r#"
experience
  start - end
"#;

    let layout = Layout::parse(content).unwrap();
    assert_eq!(layout.sections.len(), 1);
    assert_eq!(layout.sections[0].fields.len(), 1);
    assert_eq!(
      layout.sections[0].fields[0].parts,
      vec![
        FieldPart::Field("start".to_string()),
        FieldPart::Field("-".to_string()),
        FieldPart::Field("end".to_string())
      ]
    );
  }

  #[test]
  fn test_parse_empty_lines() {
    let content = r#"
person
  name

  email


experience
  title
"#;

    let layout = Layout::parse(content).unwrap();
    assert_eq!(layout.sections.len(), 2);
    assert_eq!(layout.sections[0].fields.len(), 2);
    assert_eq!(layout.sections[1].fields.len(), 1);
  }

  #[test]
  fn test_parse_section_without_fields() {
    let content = r#"
person
  name

summary

skills
"#;

    let layout = Layout::parse(content).unwrap();
    assert_eq!(layout.sections.len(), 3);
    assert_eq!(layout.sections[0].name, "person");
    assert_eq!(layout.sections[0].fields.len(), 1);
    assert_eq!(layout.sections[1].name, "summary");
    assert_eq!(layout.sections[1].fields.len(), 0);
    assert_eq!(layout.sections[2].name, "skills");
    assert_eq!(layout.sections[2].fields.len(), 0);
  }

  #[test]
  fn test_default_layout() {
    let layout = Layout::default();
    assert!(!layout.sections.is_empty());

    let section_names: Vec<_> =
      layout.sections.iter().map(|s| s.name.as_str()).collect();
    assert!(section_names.contains(&"person"));
    assert!(section_names.contains(&"experience"));
    assert!(section_names.contains(&"education"));
  }

  #[test]
  fn test_minimal_layout() {
    let content = r#"
person
  name
"#;

    let layout = Layout::parse(content).unwrap();
    assert_eq!(layout.sections.len(), 1);
    assert_eq!(layout.sections[0].name, "person");
    assert_eq!(layout.sections[0].fields.len(), 1);
    assert_eq!(
      layout.sections[0].fields[0].parts,
      vec![FieldPart::Field("name".to_string())]
    );
  }

  #[test]
  fn test_quoted_strings() {
    let content = r#"
person
  name "at" email
  "Location:" location
"#;

    let layout = Layout::parse(content).unwrap();
    assert_eq!(layout.sections.len(), 1);
    assert_eq!(layout.sections[0].fields.len(), 2);

    // First field: name "at" email
    assert_eq!(
      layout.sections[0].fields[0].parts,
      vec![
        FieldPart::Field("name".to_string()),
        FieldPart::Literal("at".to_string()),
        FieldPart::Field("email".to_string())
      ]
    );

    // Second field: "Location:" location
    assert_eq!(
      layout.sections[0].fields[1].parts,
      vec![
        FieldPart::Literal("Location:".to_string()),
        FieldPart::Field("location".to_string())
      ]
    );
  }

  #[test]
  fn test_quoted_with_spaces() {
    let content = r#"
experience
  start " - " end
"#;

    let layout = Layout::parse(content).unwrap();
    assert_eq!(
      layout.sections[0].fields[0].parts,
      vec![
        FieldPart::Field("start".to_string()),
        FieldPart::Literal(" - ".to_string()),
        FieldPart::Field("end".to_string())
      ]
    );
  }
}
