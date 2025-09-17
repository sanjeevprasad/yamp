#![deny(clippy::all)]

use std::borrow::Cow;
use yamp::{YamlValue, parse};

#[test]
fn test_strings() {
    let yaml = r#"
simple: hello
quoted: "hello world"
single_quoted: 'hello world'
"#;

    let result = parse(yaml).expect("Failed to parse");

    if let YamlValue::Object(map) = &result.value {
        assert!(matches!(
            map.get(&Cow::Borrowed("simple")).unwrap().value,
            YamlValue::String(ref s) if s == "hello"
        ));
        assert!(matches!(
            map.get(&Cow::Borrowed("quoted")).unwrap().value,
            YamlValue::String(ref s) if s == "hello world"
        ));
        assert!(matches!(
            map.get(&Cow::Borrowed("single_quoted")).unwrap().value,
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

    let result = parse(yaml).expect("Failed to parse");

    if let YamlValue::Object(map) = &result.value {
        assert!(
            matches!(map.get(&Cow::Borrowed("bool1")).unwrap().value, YamlValue::String(ref s) if s == "true")
        );
        assert!(
            matches!(map.get(&Cow::Borrowed("bool2")).unwrap().value, YamlValue::String(ref s) if s == "false")
        );
        assert!(
            matches!(map.get(&Cow::Borrowed("bool3")).unwrap().value, YamlValue::String(ref s) if s == "yes")
        );
        assert!(
            matches!(map.get(&Cow::Borrowed("bool4")).unwrap().value, YamlValue::String(ref s) if s == "no")
        );
        assert!(
            matches!(map.get(&Cow::Borrowed("bool5")).unwrap().value, YamlValue::String(ref s) if s == "on")
        );
        assert!(
            matches!(map.get(&Cow::Borrowed("bool6")).unwrap().value, YamlValue::String(ref s) if s == "off")
        );
    }
}

#[test]
fn test_null_strings() {
    // null is now a string
    let yaml = r#"
null1: null
"#;

    let result = parse(yaml).expect("Failed to parse");

    if let YamlValue::Object(map) = &result.value {
        assert!(
            matches!(map.get(&Cow::Borrowed("null1")).unwrap().value, YamlValue::String(ref s) if s == "null")
        );
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

    let result = parse(yaml).expect("Failed to parse");

    if let YamlValue::Object(map) = &result.value {
        assert!(
            matches!(map.get(&Cow::Borrowed("positive")).unwrap().value, YamlValue::String(ref s) if s == "42")
        );
        assert!(
            matches!(map.get(&Cow::Borrowed("negative")).unwrap().value, YamlValue::String(ref s) if s == "-17")
        );
        assert!(
            matches!(map.get(&Cow::Borrowed("zero")).unwrap().value, YamlValue::String(ref s) if s == "0")
        );
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

    let result = parse(yaml).expect("Failed to parse");

    if let YamlValue::Object(map) = &result.value {
        assert!(
            matches!(map.get(&Cow::Borrowed("pi")).unwrap().value, YamlValue::String(ref s) if s == "3.14999")
        );
        assert!(
            matches!(map.get(&Cow::Borrowed("negative")).unwrap().value, YamlValue::String(ref s) if s == "-2.5")
        );
        assert!(
            matches!(map.get(&Cow::Borrowed("scientific")).unwrap().value, YamlValue::String(ref s) if s == "1.2e-3")
        );
    }
}
