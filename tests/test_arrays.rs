#![deny(clippy::all)]

use yamp::{emit, parse, YamlNode, YamlObject, YamlValue};

#[test]
fn test_simple_array() {
    let yaml = "- item1\n- item2\n- item3";
    let result = parse(yaml).expect("Failed to parse YAML");

    if let YamlValue::Array(items) = &result.value {
        assert_eq!(items.len(), 3);
    }
}

#[test]
fn test_nested_arrays() {
    let yaml = "fruits:\n  - apple\n  - banana\n  - orange";
    let result = parse(yaml).expect("Failed to parse YAML");

    if let YamlValue::Object(map) = &result.value {
        let fruits_node = map.get("fruits").expect("fruits key not found");
        if let YamlValue::Array(items) = &fruits_node.value {
            assert_eq!(items.len(), 3);
        }
    }
}

#[test]
fn test_array_of_objects() {
    let yaml = "- name: Alice\n  age: 30\n- name: Bob\n  age: 25";
    let result = parse(yaml).expect("Failed to parse YAML");

    if let YamlValue::Array(items) = &result.value {
        assert_eq!(items.len(), 2);

        if let YamlValue::Object(obj) = &items[0].value {
            assert!(obj.contains_key("name"));
            assert!(obj.contains_key("age"));
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

    let result = parse(yaml).expect("Failed to parse YAML");

    if let YamlValue::Object(map) = &result.value {
        let features = map.get("features").expect("features key not found");
        if let YamlValue::Array(items) = &features.value {
            assert_eq!(items.len(), 2);
        }
    }
}

#[test]
fn test_manual_array_construction() {
    let items = vec![
        YamlNode::from_value(YamlValue::String("item1".to_string())),
        YamlNode::from_value(YamlValue::String("item2".to_string())),
    ];

    let mut root = YamlObject::new();
    root.insert(
        "list".to_string(),
        YamlNode::from_value(YamlValue::Array(items)),
    );

    let doc = YamlNode::from_value(YamlValue::Object(root));
    let yaml_string = emit(&doc);

    let reparsed = parse(&yaml_string).expect("Failed to parse emitted array");

    // Direct comparison with PartialEq!
    assert_eq!(doc.value, reparsed.value);
}
