#![deny(clippy::all)]

use std::borrow::Cow;
use yamp::{YamlValue, emit, parse};

#[test]
fn test_inline_comments() {
    let yaml = "key: value # This is a comment";
    let result = parse(yaml).expect("Failed to parse");

    if let YamlValue::Object(map) = &result.value {
        let node = map.get(&Cow::Borrowed("key")).unwrap();
        assert_eq!(
            node.inline_comment,
            Some(Cow::Borrowed("This is a comment"))
        );
    }
}

#[test]
fn test_multiple_inline_comments() {
    let yaml = "# Header comment\nkey: value # inline comment\nage: 30";
    let result = parse(yaml).expect("Failed to parse");

    if let YamlValue::Object(map) = &result.value {
        let node = map.get(&Cow::Borrowed("key")).unwrap();
        assert_eq!(node.inline_comment, Some(Cow::Borrowed("inline comment")));
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
