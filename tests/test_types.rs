#![deny(clippy::all)]

use yamp::{parse, YamlValue};

#[test]
fn test_strings() {
    let yaml = r#"
simple: hello
quoted: "hello world"
single_quoted: 'hello world'
"#;

    let result = parse(yaml).expect("Failed to parse YAML");

    if let YamlValue::Object(map) = &result.value {
        assert!(matches!(
            map.get("simple").expect("simple key not found").value,
            YamlValue::String(ref s) if s == "hello"
        ));
        assert!(matches!(
            map.get("quoted").expect("quoted key not found").value,
            YamlValue::String(ref s) if s == "hello world"
        ));
        assert!(matches!(
            map.get("single_quoted").expect("single_quoted key not found").value,
            YamlValue::String(ref s) if s == "hello world"
        ));
    }
}

#[test]
fn test_boolean_like_strings() {
    // All boolean-like values are now strings
    let yaml = r#"
bool1: true
bool2: false
bool3: yes
bool4: no
bool5: on
bool6: off
"#;

    let result = parse(yaml).expect("Failed to parse YAML");

    if let YamlValue::Object(map) = &result.value {
        assert_eq!(map.get("bool1").and_then(|n| n.as_str()), Some("true"));
        assert_eq!(map.get("bool2").and_then(|n| n.as_str()), Some("false"));
        assert_eq!(map.get("bool3").and_then(|n| n.as_str()), Some("yes"));
        assert_eq!(map.get("bool4").and_then(|n| n.as_str()), Some("no"));
        assert_eq!(map.get("bool5").and_then(|n| n.as_str()), Some("on"));
        assert_eq!(map.get("bool6").and_then(|n| n.as_str()), Some("off"));
    }
}

#[test]
fn test_null_strings() {
    // null is now a string
    let yaml = r#"
null1: null
"#;

    let result = parse(yaml).expect("Failed to parse YAML");

    if let YamlValue::Object(map) = &result.value {
        assert_eq!(map.get("null1").and_then(|n| n.as_str()), Some("null"));
    }
}

#[test]
fn test_number_strings() {
    // All numbers are now strings
    let yaml = r#"
positive: 42
negative: -17
zero: 0
"#;

    let result = parse(yaml).expect("Failed to parse YAML");

    if let YamlValue::Object(map) = &result.value {
        assert_eq!(map.get("positive").and_then(|n| n.as_str()), Some("42"));
        assert_eq!(map.get("negative").and_then(|n| n.as_str()), Some("-17"));
        assert_eq!(map.get("zero").and_then(|n| n.as_str()), Some("0"));
    }
}

#[test]
fn test_float_strings() {
    // All floats are now strings
    let yaml = r#"
pi: 3.14999
negative: -2.5
scientific: 1.2e-3
"#;

    let result = parse(yaml).expect("Failed to parse YAML");

    if let YamlValue::Object(map) = &result.value {
        assert_eq!(map.get("pi").and_then(|n| n.as_str()), Some("3.14999"));
        assert_eq!(map.get("negative").and_then(|n| n.as_str()), Some("-2.5"));
        assert_eq!(
            map.get("scientific").and_then(|n| n.as_str()),
            Some("1.2e-3")
        );
    }
}
