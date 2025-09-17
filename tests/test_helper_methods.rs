#![deny(clippy::all)]

//! Tests specifically for YamlNode helper methods to ensure API stability

use std::borrow::Cow;
use std::collections::BTreeMap;
use yamp::{parse, YamlNode, YamlValue};

#[test]
fn test_as_str_method() {
    // Test with actual string
    let string_node = YamlNode::from_value(YamlValue::String(Cow::Borrowed("hello")));
    assert_eq!(string_node.as_str(), Some("hello"));

    // Test with non-string values
    let array_node = YamlNode::from_value(YamlValue::Array(vec![]));
    assert_eq!(array_node.as_str(), None);

    let object_node = YamlNode::from_value(YamlValue::Object(BTreeMap::new()));
    assert_eq!(object_node.as_str(), None);
}

#[test]
fn test_as_object_method() {
    // Test with actual object
    let mut map = BTreeMap::new();
    map.insert(
        Cow::Borrowed("key"),
        YamlNode::from_value(YamlValue::String(Cow::Borrowed("value"))),
    );
    let object_node = YamlNode::from_value(YamlValue::Object(map));
    assert!(object_node.as_object().is_some());
    assert_eq!(object_node.as_object().unwrap().len(), 1);

    // Test with non-object values
    let string_node = YamlNode::from_value(YamlValue::String(Cow::Borrowed("hello")));
    assert_eq!(string_node.as_object(), None);

    let array_node = YamlNode::from_value(YamlValue::Array(vec![]));
    assert_eq!(array_node.as_object(), None);
}

#[test]
fn test_as_array_method() {
    // Test with actual array
    let items = vec![
        YamlNode::from_value(YamlValue::String(Cow::Borrowed("item1"))),
        YamlNode::from_value(YamlValue::String(Cow::Borrowed("item2"))),
    ];
    let array_node = YamlNode::from_value(YamlValue::Array(items));
    assert!(array_node.as_array().is_some());
    assert_eq!(array_node.as_array().unwrap().len(), 2);
    assert_eq!(array_node.as_array().unwrap()[0].as_str(), Some("item1"));

    // Test with non-array values
    let string_node = YamlNode::from_value(YamlValue::String(Cow::Borrowed("hello")));
    assert_eq!(string_node.as_array(), None);

    let object_node = YamlNode::from_value(YamlValue::Object(BTreeMap::new()));
    assert_eq!(object_node.as_array(), None);
}

#[test]
fn test_get_method() {
    let mut map = BTreeMap::new();
    map.insert(
        Cow::Borrowed("name"),
        YamlNode::from_value(YamlValue::String(Cow::Borrowed("John"))),
    );
    map.insert(
        Cow::Borrowed("age"),
        YamlNode::from_value(YamlValue::String(Cow::Borrowed("30"))),
    );

    let object_node = YamlNode::from_value(YamlValue::Object(map));

    // Test successful get
    assert_eq!(object_node.get("name").and_then(|n| n.as_str()), Some("John"));
    assert_eq!(object_node.get("age").and_then(|n| n.as_str()), Some("30"));

    // Test missing key
    assert!(object_node.get("missing").is_none());

    // Test get on non-object
    let string_node = YamlNode::from_value(YamlValue::String(Cow::Borrowed("hello")));
    assert!(string_node.get("anything").is_none());
}

#[test]
fn test_get_with_different_key_types() {
    // Test that get works with different Cow variants in the map
    let mut map = BTreeMap::new();
    map.insert(
        Cow::Owned("owned_key".to_string()),
        YamlNode::from_value(YamlValue::String(Cow::Borrowed("owned_value"))),
    );
    map.insert(
        Cow::Borrowed("borrowed_key"),
        YamlNode::from_value(YamlValue::String(Cow::Borrowed("borrowed_value"))),
    );

    let object_node = YamlNode::from_value(YamlValue::Object(map));

    // Both should work regardless of Cow variant
    assert_eq!(object_node.get("owned_key").and_then(|n| n.as_str()), Some("owned_value"));
    assert_eq!(object_node.get("borrowed_key").and_then(|n| n.as_str()), Some("borrowed_value"));
}

#[test]
fn test_is_string_method() {
    let string_node = YamlNode::from_value(YamlValue::String(Cow::Borrowed("hello")));
    assert!(string_node.is_string());
    assert!(!string_node.is_array());
    assert!(!string_node.is_object());
}

#[test]
fn test_is_array_method() {
    let array_node = YamlNode::from_value(YamlValue::Array(vec![]));
    assert!(!array_node.is_string());
    assert!(array_node.is_array());
    assert!(!array_node.is_object());
}

#[test]
fn test_is_object_method() {
    let object_node = YamlNode::from_value(YamlValue::Object(BTreeMap::new()));
    assert!(!object_node.is_string());
    assert!(!object_node.is_array());
    assert!(object_node.is_object());
}

#[test]
fn test_nested_navigation() {
    // Create a nested structure
    let mut inner_map = BTreeMap::new();
    inner_map.insert(
        Cow::Borrowed("host"),
        YamlNode::from_value(YamlValue::String(Cow::Borrowed("localhost"))),
    );
    inner_map.insert(
        Cow::Borrowed("port"),
        YamlNode::from_value(YamlValue::String(Cow::Borrowed("8080"))),
    );

    let mut outer_map = BTreeMap::new();
    outer_map.insert(
        Cow::Borrowed("server"),
        YamlNode::from_value(YamlValue::Object(inner_map)),
    );

    let root = YamlNode::from_value(YamlValue::Object(outer_map));

    // Test nested navigation
    let port = root
        .get("server")
        .and_then(|s| s.get("port"))
        .and_then(|p| p.as_str());
    assert_eq!(port, Some("8080"));

    // Test partial navigation
    assert!(root.get("server").is_some());
    assert!(root.get("server").unwrap().is_object());

    // Test failed navigation
    let missing = root
        .get("client")
        .and_then(|s| s.get("port"))
        .and_then(|p| p.as_str());
    assert_eq!(missing, None);
}

#[test]
fn test_helper_methods_preserve_comments() {
    // Create a node with comments
    let mut node = YamlNode::from_value(YamlValue::String(Cow::Borrowed("value")));
    node.leading_comment = Some(Cow::Borrowed("Leading comment"));
    node.inline_comment = Some(Cow::Borrowed("Inline comment"));

    // Helper methods should work without affecting comments
    assert_eq!(node.as_str(), Some("value"));
    assert!(node.is_string());

    // Comments should still be accessible
    assert_eq!(node.leading_comment.as_deref(), Some("Leading comment"));
    assert_eq!(node.inline_comment.as_deref(), Some("Inline comment"));
}

#[test]
fn test_real_world_usage() {
    let yaml = r#"
app:
  name: MyApp
  version: 1.0.0
  features:
    - logging
    - metrics
  config:
    debug: false
    timeout: 30
"#;

    let parsed = parse(yaml).expect("Failed to parse");

    // Test various helper method combinations
    assert_eq!(
        parsed.get("app").and_then(|a| a.get("name")).and_then(|n| n.as_str()),
        Some("MyApp")
    );

    assert_eq!(
        parsed.get("app").and_then(|a| a.get("version")).and_then(|v| v.as_str()),
        Some("1.0.0")
    );

    // Test array access
    let features = parsed
        .get("app")
        .and_then(|a| a.get("features"))
        .and_then(|f| f.as_array());
    assert!(features.is_some());
    assert_eq!(features.unwrap().len(), 2);
    assert_eq!(features.unwrap()[0].as_str(), Some("logging"));

    // Test nested config
    assert_eq!(
        parsed
            .get("app")
            .and_then(|a| a.get("config"))
            .and_then(|c| c.get("debug"))
            .and_then(|d| d.as_str()),
        Some("false")
    );
}

#[test]
fn test_empty_values() {
    // Test helper methods with empty values
    let empty_string = YamlNode::from_value(YamlValue::String(Cow::Borrowed("")));
    assert_eq!(empty_string.as_str(), Some(""));

    let empty_array = YamlNode::from_value(YamlValue::Array(vec![]));
    assert_eq!(empty_array.as_array().map(|a| a.len()), Some(0));

    let empty_object = YamlNode::from_value(YamlValue::Object(BTreeMap::new()));
    assert_eq!(empty_object.as_object().map(|o| o.len()), Some(0));
}