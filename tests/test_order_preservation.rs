#![deny(clippy::all)]

use yamp::{parse, emit, YamlNode, YamlObject, YamlValue};

#[test]
fn test_parse_preserves_key_order() {
    let yaml = r#"zoo: first
apple: second
middle: third
banana: fourth"#;

    let parsed = parse(yaml).expect("Failed to parse");

    let YamlValue::Object(obj) = &parsed.value else {
        panic!("Expected object");
    };

    // Collect keys in the order they appear
    let keys: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();

    // Keys should be in original order, NOT alphabetical
    assert_eq!(keys, vec!["zoo", "apple", "middle", "banana"]);
}

#[test]
fn test_emit_preserves_key_order() {
    // Manually construct object with specific order
    let mut obj = YamlObject::new();
    obj.insert("zoo".to_string(), YamlNode::from_value(YamlValue::String("1".to_string())));
    obj.insert("apple".to_string(), YamlNode::from_value(YamlValue::String("2".to_string())));
    obj.insert("middle".to_string(), YamlNode::from_value(YamlValue::String("3".to_string())));

    let node = YamlNode::from_value(YamlValue::Object(obj));
    let emitted = emit(&node);

    let lines: Vec<&str> = emitted.lines().collect();

    // Should emit in insertion order
    assert!(lines[0].starts_with("zoo:"));
    assert!(lines[1].starts_with("apple:"));
    assert!(lines[2].starts_with("middle:"));
}

#[test]
fn test_round_trip_preserves_order() {
    let yaml = r#"zebra: "1"
apple: "2"
middle: "3"
banana: "4""#;

    let parsed = parse(yaml).expect("Failed to parse");
    let emitted = emit(&parsed);

    // The emitted YAML should have the same key order
    let lines: Vec<&str> = emitted.lines().collect();
    assert!(lines[0].starts_with("zebra:"));
    assert!(lines[1].starts_with("apple:"));
    assert!(lines[2].starts_with("middle:"));
    assert!(lines[3].starts_with("banana:"));
}

#[test]
fn test_nested_objects_preserve_order() {
    let yaml = r#"outer_z: value
outer_a:
  inner_z: "1"
  inner_a: "2"
  inner_m: "3"
outer_m: value"#;

    let parsed = parse(yaml).expect("Failed to parse");

    let YamlValue::Object(outer) = &parsed.value else {
        panic!("Expected object");
    };

    let outer_keys: Vec<&str> = outer.keys().map(|k| k.as_str()).collect();
    assert_eq!(outer_keys, vec!["outer_z", "outer_a", "outer_m"]);

    // Check inner object order
    let inner_node = outer.get("outer_a").expect("outer_a not found");
    let YamlValue::Object(inner) = &inner_node.value else {
        panic!("Expected nested object");
    };

    let inner_keys: Vec<&str> = inner.keys().map(|k| k.as_str()).collect();
    assert_eq!(inner_keys, vec!["inner_z", "inner_a", "inner_m"]);
}