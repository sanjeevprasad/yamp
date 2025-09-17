#![deny(clippy::all)]

use yamp::{emit, parse, YamlValue};

#[test]
fn test_inline_comments() {
    let yaml = "key: value # This is a comment";
    let result = parse(yaml).expect("Failed to parse");

    if let YamlValue::Object(map) = &result.value {
        let node = map.get("key").expect("key not found");
        assert_eq!(node.inline_comment, Some("This is a comment".to_string()));
    }
}

#[test]
fn test_multiple_inline_comments() {
    let yaml = r#"# Header comment
key: value # inline comment
age: 30"#;
    let result = parse(yaml).expect("Failed to parse");

    if let YamlValue::Object(map) = &result.value {
        let node = map.get("key").expect("key not found");
        assert_eq!(node.inline_comment, Some("inline comment".to_string()));
    }
}

#[test]
fn test_comments_preserved_in_emit() {
    let yaml = "name: John # Name field\nage: 30";
    let parsed = parse(yaml).expect("Failed to parse");
    let emitted = emit(&parsed);

    // Comments should be preserved in the output
    assert!(emitted.contains("John"));
    assert!(emitted.contains("30"));
}
