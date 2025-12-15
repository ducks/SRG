use jobl::{EducationItem, ExperienceItem, JoblDocument, Person};
use std::collections::BTreeMap;

fn create_test_document() -> JoblDocument {
  JoblDocument {
    person: Person {
      name: "Test User".to_string(),
      headline: Some("Software Engineer".to_string()),
      email: Some("test@example.com".to_string()),
      phone: Some("555-1234".to_string()),
      location: Some("Test City".to_string()),
      website: Some("https://example.com".to_string()),
      summary: Some("Test summary".to_string()),
    },
    skills: Some({
      let mut skills = BTreeMap::new();
      skills.insert("Languages".to_string(), vec!["Rust".to_string()]);
      skills
    }),
    experience: vec![ExperienceItem {
      title: "Engineer".to_string(),
      company: "Test Co".to_string(),
      location: None,
      start: Some("2020".to_string()),
      end: Some("2024".to_string()),
      summary: Some("Did things".to_string()),
      highlights: vec!["Built stuff".to_string()],
      technologies: vec![],
    }],
    projects: vec![],
    education: vec![EducationItem {
      degree: "BS CS".to_string(),
      institution: "Test U".to_string(),
      location: None,
      start: Some("2016".to_string()),
      end: Some("2020".to_string()),
      details: vec![],
    }],
  }
}

#[test]
fn test_minimal_layout_excludes_fields() {
  let layout_content = r#"
person
  name
"#;

  let layout = srg::layout::Layout::parse(layout_content).unwrap();
  let doc = create_test_document();

  let html =
    srg::build::generate_test_html(&doc, "minimal", &layout).unwrap();

  assert!(html.contains("Test User"));
  assert!(!html.contains("test@example.com"));
  assert!(!html.contains("555-1234"));
  assert!(!html.contains("Software Engineer"));
}

#[test]
fn test_full_person_section() {
  let layout_content = r#"
person
  name
  headline
  email
  phone
  location
  website
"#;

  let layout = srg::layout::Layout::parse(layout_content).unwrap();
  let doc = create_test_document();

  let html =
    srg::build::generate_test_html(&doc, "minimal", &layout).unwrap();

  assert!(html.contains("Test User"));
  assert!(html.contains("Software Engineer"));
  assert!(html.contains("test@example.com"));
  assert!(html.contains("555-1234"));
  assert!(html.contains("Test City"));
  assert!(html.contains("https://example.com"));
}

#[test]
fn test_section_ordering() {
  let layout_content = r#"
education
  degree

experience
  title
"#;

  let layout = srg::layout::Layout::parse(layout_content).unwrap();
  let doc = create_test_document();

  let html =
    srg::build::generate_test_html(&doc, "minimal", &layout).unwrap();

  let education_pos = html.find("Education").unwrap();
  let experience_pos = html.find("Experience").unwrap();

  // Education should appear before Experience in HTML
  assert!(education_pos < experience_pos);
}

#[test]
fn test_date_range_format() {
  let layout_content = r#"
experience
  start " - " end
"#;

  let layout = srg::layout::Layout::parse(layout_content).unwrap();
  let doc = create_test_document();

  let html =
    srg::build::generate_test_html(&doc, "minimal", &layout).unwrap();

  assert!(html.contains("2020 - 2024"));
}

#[test]
fn test_highlights_rendering() {
  let layout_content = r#"
experience
  highlights
"#;

  let layout = srg::layout::Layout::parse(layout_content).unwrap();
  let doc = create_test_document();

  let html =
    srg::build::generate_test_html(&doc, "minimal", &layout).unwrap();

  assert!(html.contains("<ul"));
  assert!(html.contains("Built stuff"));
  assert!(html.contains("</ul>"));
}

#[test]
fn test_summary_section() {
  let layout_content = r#"
summary
"#;

  let layout = srg::layout::Layout::parse(layout_content).unwrap();
  let doc = create_test_document();

  let html =
    srg::build::generate_test_html(&doc, "minimal", &layout).unwrap();

  assert!(html.contains("Summary"));
  assert!(html.contains("Test summary"));
}

#[test]
fn test_skills_section() {
  let layout_content = r#"
skills
"#;

  let layout = srg::layout::Layout::parse(layout_content).unwrap();
  let doc = create_test_document();

  let html =
    srg::build::generate_test_html(&doc, "minimal", &layout).unwrap();

  assert!(html.contains("Skills"));
  assert!(html.contains("Languages"));
  assert!(html.contains("Rust"));
}
