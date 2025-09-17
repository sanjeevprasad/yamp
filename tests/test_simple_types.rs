#![deny(clippy::all)]

use std::borrow::Cow;
use std::collections::BTreeMap;
use yamp::{YamlNode, YamlValue, emit, parse};

#[test]
fn test_basic_string_parsing() {
    let test_cases = [
        ("hello", "hello"),
        ("123", "123"),
        ("3.14", "3.14"),
        ("true", "true"),
        ("false", "false"),
        ("null", "null"),
        ("yes", "yes"),
        ("no", "no"),
        ("NO", "NO"),
        ("3.10", "3.10"),
        ("0755", "0755"),
        ("~/.ssh/config", "~/.ssh/config"),
        ("12:34:56", "12:34:56"),
    ];

    for (input, expected) in test_cases {
        let yaml = format!("value: {}", input);
        let parsed = parse(&yaml).expect("Failed to parse");

        let map = match &parsed.value {
            YamlValue::Object(m) => m,
            YamlValue::String(_) | YamlValue::Array(_) => panic!("Expected YamlValue::Object, got {:?}", parsed.value),
        };
        let value_node = map.get(&Cow::Borrowed("value"))
            .expect("Key 'value' not found in map");
        let s = match &value_node.value {
            YamlValue::String(s) => s,
            YamlValue::Object(_) | YamlValue::Array(_) => panic!("Expected YamlValue::String for input '{}', got {:?}", input, value_node.value),
        };
        assert_eq!(s.as_ref(), expected, "Failed for input: {}", input);
    }
}

#[test]
fn test_array_parsing() {
    let yaml = r#"
items:
  - first
  - second
  - 123
  - true
  - 3.14
"#;

    let parsed = parse(yaml).expect("Failed to parse");

    let map = match &parsed.value {
        YamlValue::Object(m) => m,
        _ => panic!("Expected YamlValue::Object, got {:?}", parsed.value),
    };
    let items_node = map.get(&Cow::Borrowed("items"))
        .expect("Key 'items' not found in map");
    let items = match &items_node.value {
        YamlValue::Array(arr) => arr,
        YamlValue::String(_) | YamlValue::Object(_) => panic!("Expected YamlValue::Array for items, got {:?}", items_node.value),
    };
    assert_eq!(items.len(), 5);

    let expected = ["first", "second", "123", "true", "3.14"];
    for (i, expected_val) in expected.iter().enumerate() {
        let s = match &items[i].value {
            YamlValue::String(s) => s,
            YamlValue::Object(_) | YamlValue::Array(_) => panic!("Expected YamlValue::String at index {}, got {:?}", i, items[i].value),
        };
        assert_eq!(s.as_ref(), *expected_val);
    }
}

#[test]
fn test_nested_objects() {
    let yaml = r#"
root:
  child1:
    value: test
    number: 42
  child2:
    flag: true
    version: 3.10
"#;

    let parsed = parse(yaml).expect("Failed to parse");

    let root_map = match &parsed.value {
        YamlValue::Object(m) => m,
        _ => panic!("Expected YamlValue::Object, got {:?}", parsed.value),
    };
    let root_node = root_map.get(&Cow::Borrowed("root"))
        .expect("Key 'root' not found in map");
    let root_obj = match &root_node.value {
        YamlValue::Object(obj) => obj,
        _ => panic!("Expected YamlValue::Object for root, got {:?}", root_node.value),
    };
    assert_eq!(root_obj.len(), 2);

    // Check child1
    let child1_node = root_obj.get(&Cow::Borrowed("child1"))
        .expect("Key 'child1' not found in root");
    let child1 = match &child1_node.value {
        YamlValue::Object(obj) => obj,
        _ => panic!("Expected YamlValue::Object for child1, got {:?}", child1_node.value),
    };
    let value_node = child1.get(&Cow::Borrowed("value"))
        .expect("Key 'value' not found in child1");
    let s = match &value_node.value {
        YamlValue::String(s) => s,
        _ => panic!("Expected YamlValue::String for child1.value, got {:?}", value_node.value),
    };
    assert_eq!(s.as_ref(), "test");
    let number_node = child1.get(&Cow::Borrowed("number"))
        .expect("Key 'number' not found in child1");
    let s = match &number_node.value {
        YamlValue::String(s) => s,
        _ => panic!("Expected YamlValue::String for child1.number, got {:?}", number_node.value),
    };
    assert_eq!(s.as_ref(), "42");

    // Check child2
    let child2_node = root_obj.get(&Cow::Borrowed("child2"))
        .expect("Key 'child2' not found in root");
    let child2 = match &child2_node.value {
        YamlValue::Object(obj) => obj,
        _ => panic!("Expected YamlValue::Object for child2, got {:?}", child2_node.value),
    };
    let flag_node = child2.get(&Cow::Borrowed("flag"))
        .expect("Key 'flag' not found in child2");
    let s = match &flag_node.value {
        YamlValue::String(s) => s,
        _ => panic!("Expected YamlValue::String for child2.flag, got {:?}", flag_node.value),
    };
    assert_eq!(s.as_ref(), "true");
    let version_node = child2.get(&Cow::Borrowed("version"))
        .expect("Key 'version' not found in child2");
    let s = match &version_node.value {
        YamlValue::String(s) => s,
        _ => panic!("Expected YamlValue::String for child2.version, got {:?}", version_node.value),
    };
    assert_eq!(s.as_ref(), "3.10");
}

#[test]
fn test_round_trip() {
    let yaml = r#"name: John Doe
age: 30
active: true
version: 3.10
permissions: 0755
items:
  - first
  - second
nested:
  key1: value1
  key2: value2"#;

    let parsed = parse(yaml).expect("Failed to parse");
    let emitted = emit(&parsed);
    let reparsed = parse(&emitted).expect("Failed to reparse");

    // Direct comparison now works with PartialEq!
    assert_eq!(parsed.value, reparsed.value, "Round-trip failed");
}

#[test]
fn test_quoted_strings() {
    let yaml = r#"
single: 'hello world'
double: "hello world"
unquoted: hello world
"#;

    let parsed = parse(yaml).expect("Failed to parse");

    let map = match &parsed.value {
        YamlValue::Object(m) => m,
        _ => panic!("Expected YamlValue::Object, got {:?}", parsed.value),
    };
    let single_node = map.get(&Cow::Borrowed("single"))
        .expect("Key 'single' not found in map");
    let s = match &single_node.value {
        YamlValue::String(s) => s,
        _ => panic!("Expected YamlValue::String for single, got {:?}", single_node.value),
    };
    assert_eq!(s.as_ref(), "hello world");
    let double_node = map.get(&Cow::Borrowed("double"))
        .expect("Key 'double' not found in map");
    let s = match &double_node.value {
        YamlValue::String(s) => s,
        _ => panic!("Expected YamlValue::String for double, got {:?}", double_node.value),
    };
    assert_eq!(s.as_ref(), "hello world");
    let unquoted_node = map.get(&Cow::Borrowed("unquoted"))
        .expect("Key 'unquoted' not found in map");
    let s = match &unquoted_node.value {
        YamlValue::String(s) => s,
        _ => panic!("Expected YamlValue::String for unquoted, got {:?}", unquoted_node.value),
    };
    assert_eq!(s.as_ref(), "hello world");
}

#[test]
fn test_manual_construction() {
    // Test manual construction of YAML values
    let mut map = BTreeMap::new();
    map.insert(
        Cow::Borrowed("name"),
        YamlNode::from_value(YamlValue::String(Cow::Borrowed("Test"))),
    );
    map.insert(
        Cow::Borrowed("count"),
        YamlNode::from_value(YamlValue::String(Cow::Borrowed("42"))),
    );

    let root = YamlNode::from_value(YamlValue::Object(map));
    let emitted = emit(&root);

    assert!(emitted.contains("name:"));
    assert!(emitted.contains("Test"));
    assert!(emitted.contains("count:"));
    assert!(emitted.contains("42"));
}
