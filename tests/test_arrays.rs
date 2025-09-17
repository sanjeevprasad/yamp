#![deny(clippy::all)]

use std::borrow::Cow;
use std::collections::BTreeMap;
use yamp::{YamlNode, YamlValue, emit, parse};

#[test]
fn test_simple_array() {
    let yaml = "- item1\n- item2\n- item3";
    let result = parse(yaml).expect("Failed to parse");

    if let YamlValue::Array(items) = &result.value {
        assert_eq!(items.len(), 3);
    }
}

#[test]
fn test_nested_arrays() {
    let yaml = "fruits:\n  - apple\n  - banana\n  - orange";
    let result = parse(yaml).expect("Failed to parse");

    if let YamlValue::Object(map) = &result.value {
        if let Some(fruits_node) = map.get(&Cow::Borrowed("fruits")) {
            if let YamlValue::Array(items) = &fruits_node.value {
                assert_eq!(items.len(), 3);
            }
        }
    }
}

#[test]
fn test_array_of_objects() {
    let yaml = "- name: Alice\n  age: 30\n- name: Bob\n  age: 25";
    let result = parse(yaml).expect("Failed to parse");

    if let YamlValue::Array(items) = &result.value {
        assert_eq!(items.len(), 2);

        if let YamlValue::Object(obj) = &items[0].value {
            assert!(obj.contains_key(&Cow::Borrowed("name")));
            assert!(obj.contains_key(&Cow::Borrowed("age")));
        }
    }
}

#[test]
fn test_array_of_objects_inline_format() {
    let yaml = r#"features:
  - enabled: false
    name: feature1
  - enabled: true
    name: feature2"#;

    let result = parse(yaml).expect("Failed to parse");

    if let YamlValue::Object(map) = &result.value {
        if let Some(features) = map.get(&Cow::Borrowed("features")) {
            if let YamlValue::Array(items) = &features.value {
                assert_eq!(items.len(), 2);
            }
        }
    }
}

#[test]
fn test_manual_array_construction() {
    let items = vec![
        YamlNode::from_value(YamlValue::String(Cow::Borrowed("item1"))),
        YamlNode::from_value(YamlValue::String(Cow::Borrowed("item2"))),
    ];

    let mut root = BTreeMap::new();
    root.insert(
        Cow::Borrowed("list"),
        YamlNode::from_value(YamlValue::Array(items)),
    );

    let doc = YamlNode::from_value(YamlValue::Object(root));
    let yaml_string = emit(&doc);

    let reparsed = parse(&yaml_string).expect("Failed to parse emitted array");

    // Direct comparison with PartialEq!
    assert_eq!(doc.value, reparsed.value);
}
