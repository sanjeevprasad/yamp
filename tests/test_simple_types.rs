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

        if let YamlValue::Object(map) = &parsed.value {
            if let YamlValue::String(s) = &map.get(&Cow::Borrowed("value")).unwrap().value {
                assert_eq!(s.as_ref(), expected, "Failed for input: {}", input);
            } else {
                panic!("Expected string value for: {}", input);
            }
        } else {
            panic!("Expected object");
        }
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

    if let YamlValue::Object(map) = &parsed.value {
        if let YamlValue::Array(items) = &map.get(&Cow::Borrowed("items")).unwrap().value {
            assert_eq!(items.len(), 5);

            let expected = ["first", "second", "123", "true", "3.14"];
            for (i, expected_val) in expected.iter().enumerate() {
                if let YamlValue::String(s) = &items[i].value {
                    assert_eq!(s.as_ref(), *expected_val);
                } else {
                    panic!("Expected string at index {}", i);
                }
            }
        } else {
            panic!("Expected array");
        }
    } else {
        panic!("Expected object");
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

    if let YamlValue::Object(root_map) = &parsed.value {
        if let YamlValue::Object(root_obj) = &root_map.get(&Cow::Borrowed("root")).unwrap().value {
            assert_eq!(root_obj.len(), 2);

            // Check child1
            if let YamlValue::Object(child1) = &root_obj.get(&Cow::Borrowed("child1")).unwrap().value {
                if let YamlValue::String(s) = &child1.get(&Cow::Borrowed("value")).unwrap().value {
                    assert_eq!(s.as_ref(), "test");
                }
                if let YamlValue::String(s) = &child1.get(&Cow::Borrowed("number")).unwrap().value {
                    assert_eq!(s.as_ref(), "42");
                }
            }

            // Check child2
            if let YamlValue::Object(child2) = &root_obj.get(&Cow::Borrowed("child2")).unwrap().value {
                if let YamlValue::String(s) = &child2.get(&Cow::Borrowed("flag")).unwrap().value {
                    assert_eq!(s.as_ref(), "true");
                }
                if let YamlValue::String(s) = &child2.get(&Cow::Borrowed("version")).unwrap().value {
                    assert_eq!(s.as_ref(), "3.10");
                }
            }
        }
    }
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

    if let YamlValue::Object(map) = &parsed.value {
        if let YamlValue::String(s) = &map.get(&Cow::Borrowed("single")).unwrap().value {
            assert_eq!(s.as_ref(), "hello world");
        }
        if let YamlValue::String(s) = &map.get(&Cow::Borrowed("double")).unwrap().value {
            assert_eq!(s.as_ref(), "hello world");
        }
        if let YamlValue::String(s) = &map.get(&Cow::Borrowed("unquoted")).unwrap().value {
            assert_eq!(s.as_ref(), "hello world");
        }
    }
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
