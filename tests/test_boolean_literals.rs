#![deny(clippy::all)]

use std::borrow::Cow;
use yamp::{YamlValue, parse};

#[test]
fn test_all_values_are_strings() {
    // Test that all values including "true" and "false" are parsed as strings
    let test_cases = [
        ("true", YamlValue::String(Cow::Borrowed("true"))),
        ("false", YamlValue::String(Cow::Borrowed("false"))),
    ];

    for (yaml, expected) in test_cases {
        let parsed = parse(yaml).unwrap_or_else(|_| panic!("Failed to parse '{}'", yaml));
        assert_eq!(parsed.value, expected, "Failed for input: {}", yaml);
    }
}

#[test]
fn test_boolean_like_values_parse_as_strings() {
    // Test that all boolean-like values are parsed as strings
    let boolean_like_values = [
        "yes", "no", "on", "off", "YES", "NO", "ON", "OFF", "True", "False", "TRUE", "FALSE",
        "true", "false",
    ];

    for value in boolean_like_values {
        let parsed = parse(value).unwrap_or_else(|_| panic!("Failed to parse '{}'", value));

        match parsed.value {
            YamlValue::String(s) => {
                assert_eq!(
                    s.as_ref(),
                    value,
                    "String value should match input for: {}",
                    value
                );
            }
            YamlValue::Object(_) | YamlValue::Array(_) => panic!(
                "Expected '{}' to be parsed as a string, got: {:?}",
                value, parsed.value
            ),
        }
    }
}

#[test]
fn test_boolean_like_values_in_objects() {
    let yaml = r#"
yes_key: yes
no_key: no
on_key: on
off_key: off
true_key: true
false_key: false
"#;

    let parsed = parse(yaml).expect("Failed to parse YAML with boolean-like values");

    if let YamlValue::Object(map) = &parsed.value {
        // All values are strings now
        assert_eq!(
            map.get(&Cow::Borrowed("yes_key")).unwrap().value,
            YamlValue::String(Cow::Borrowed("yes"))
        );
        assert_eq!(
            map.get(&Cow::Borrowed("no_key")).unwrap().value,
            YamlValue::String(Cow::Borrowed("no"))
        );
        assert_eq!(
            map.get(&Cow::Borrowed("on_key")).unwrap().value,
            YamlValue::String(Cow::Borrowed("on"))
        );
        assert_eq!(
            map.get(&Cow::Borrowed("off_key")).unwrap().value,
            YamlValue::String(Cow::Borrowed("off"))
        );
        assert_eq!(
            map.get(&Cow::Borrowed("true_key")).unwrap().value,
            YamlValue::String(Cow::Borrowed("true"))
        );
        assert_eq!(
            map.get(&Cow::Borrowed("false_key")).unwrap().value,
            YamlValue::String(Cow::Borrowed("false"))
        );
    } else {
        panic!("Expected object at root");
    }
}

#[test]
fn test_boolean_like_values_in_arrays() {
    let yaml = r#"
- yes
- no
- on
- off
- true
- false
"#;

    let parsed = parse(yaml).expect("Failed to parse array with boolean-like values");

    if let YamlValue::Array(items) = &parsed.value {
        assert_eq!(items.len(), 6);

        // All values are strings
        assert_eq!(items[0].value, YamlValue::String(Cow::Borrowed("yes")));
        assert_eq!(items[1].value, YamlValue::String(Cow::Borrowed("no")));
        assert_eq!(items[2].value, YamlValue::String(Cow::Borrowed("on")));
        assert_eq!(items[3].value, YamlValue::String(Cow::Borrowed("off")));
        assert_eq!(items[4].value, YamlValue::String(Cow::Borrowed("true")));
        assert_eq!(items[5].value, YamlValue::String(Cow::Borrowed("false")));
    } else {
        panic!("Expected array at root");
    }
}

#[test]
fn test_case_sensitive_strings() {
    // Test that all case variants are strings
    let case_variants = [
        "True", "False", "TRUE", "FALSE", "tRuE", "fAlSe", "true", "false",
    ];

    for value in case_variants {
        let parsed = parse(value).unwrap_or_else(|_| panic!("Failed to parse '{}'", value));

        // All values are strings
        match parsed.value {
            YamlValue::String(s) => {
                assert_eq!(s.as_ref(), value);
            }
            YamlValue::Object(_) | YamlValue::Array(_) => panic!(
                "Expected '{}' to be parsed as a string, got: {:?}",
                value, parsed.value
            ),
        }
    }
}
