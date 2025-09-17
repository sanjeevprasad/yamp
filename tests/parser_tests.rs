#![deny(clippy::all)]

use std::borrow::Cow;
use yamp::{YamlValue, parse};

#[test]
fn test_simple_key_value() {
    let yaml = "key: value";
    let result = parse(yaml).expect("Failed to parse simple key-value");

    if let YamlValue::Object(map) = &result.value {
        assert_eq!(map.len(), 1);
        assert!(map.contains_key(&Cow::Borrowed("key")));
        if let YamlValue::String(s) = &map.get(&Cow::Borrowed("key")).unwrap().value {
            assert_eq!(s.as_ref(), "value");
        } else {
            panic!("Expected string value");
        }
    } else {
        panic!("Expected object at root");
    }
}

#[test]
fn test_all_value_formats_parse_as_strings() {
    let yaml = r#"
string: hello
integer: 42
float: 3.15
boolean_true: true
boolean_false: false
null_value: null
"#;

    let result = parse(yaml).expect("Failed to parse multiple types");

    if let YamlValue::Object(map) = &result.value {
        // Check string
        if let Some(node) = map.get(&Cow::Borrowed("string")) {
            assert!(matches!(node.value, YamlValue::String(ref s) if s == "hello"));
        }

        // Check integer - now a string
        if let Some(node) = map.get(&Cow::Borrowed("integer")) {
            assert!(matches!(node.value, YamlValue::String(ref s) if s == "42"));
        }

        // Check float - now a string
        if let Some(node) = map.get(&Cow::Borrowed("float")) {
            assert!(matches!(node.value, YamlValue::String(ref s) if s == "3.15"));
        }

        // Check booleans - now strings
        if let Some(node) = map.get(&Cow::Borrowed("boolean_true")) {
            assert!(matches!(node.value, YamlValue::String(ref s) if s == "true"));
        }
        if let Some(node) = map.get(&Cow::Borrowed("boolean_false")) {
            assert!(matches!(node.value, YamlValue::String(ref s) if s == "false"));
        }

        // Check null - now a string
        if let Some(node) = map.get(&Cow::Borrowed("null_value")) {
            assert!(matches!(node.value, YamlValue::String(ref s) if s == "null"));
        }
    }
}

#[test]
fn test_nested_objects() {
    let yaml = r#"
database:
  host: localhost
  port: 5432
  credentials:
    username: admin
    password: secret
"#;

    let result = parse(yaml).expect("Failed to parse nested objects");

    if let YamlValue::Object(map) = &result.value
        && let Some(db_node) = map.get(&Cow::Borrowed("database"))
        && let YamlValue::Object(db_map) = &db_node.value
    {
        assert!(db_map.contains_key(&Cow::Borrowed("host")));
        assert!(db_map.contains_key(&Cow::Borrowed("port")));

        if let Some(creds_node) = db_map.get(&Cow::Borrowed("credentials"))
            && let YamlValue::Object(creds_map) = &creds_node.value
        {
            assert!(creds_map.contains_key(&Cow::Borrowed("username")));
            assert!(creds_map.contains_key(&Cow::Borrowed("password")));
        }
    }
}

#[test]
fn test_array_of_objects() {
    let yaml = r#"
users:
  - name: Alice
    age: 30
  - name: Bob
    age: 25
"#;

    let result = parse(yaml).expect("Failed to parse array of objects");

    if let YamlValue::Object(map) = &result.value
        && let Some(users_node) = map.get(&Cow::Borrowed("users"))
        && let YamlValue::Array(users) = &users_node.value
    {
        assert_eq!(users.len(), 2);

        // Check first user
        if let YamlValue::Object(user1) = &users[0].value {
            if let Some(name_node) = user1.get(&Cow::Borrowed("name")) {
                assert!(matches!(name_node.value, YamlValue::String(ref s) if s == "Alice"));
            }
            if let Some(age_node) = user1.get(&Cow::Borrowed("age")) {
                assert!(matches!(age_node.value, YamlValue::String(ref s) if s == "30"));
            }
        }

        // Check second user
        if let YamlValue::Object(user2) = &users[1].value {
            if let Some(name_node) = user2.get(&Cow::Borrowed("name")) {
                assert!(matches!(name_node.value, YamlValue::String(ref s) if s == "Bob"));
            }
            if let Some(age_node) = user2.get(&Cow::Borrowed("age")) {
                assert!(matches!(age_node.value, YamlValue::String(ref s) if s == "25"));
            }
        }
    }
}

#[test]
fn test_boolean_variants() {
    let yaml = r#"
bool1: true
bool2: false
string3: yes
string4: no
string5: on
string6: off
"#;

    let result = parse(yaml).expect("Failed to parse boolean variants");

    if let YamlValue::Object(map) = &result.value {
        // All boolean values are now strings
        assert!(
            matches!(map.get(&Cow::Borrowed("bool1")).unwrap().value, YamlValue::String(ref s) if s == "true")
        );
        assert!(
            matches!(map.get(&Cow::Borrowed("bool2")).unwrap().value, YamlValue::String(ref s) if s == "false")
        );

        // yes/no/on/off should be strings
        assert!(
            matches!(map.get(&Cow::Borrowed("string3")).unwrap().value, YamlValue::String(ref s) if s == "yes")
        );
        assert!(
            matches!(map.get(&Cow::Borrowed("string4")).unwrap().value, YamlValue::String(ref s) if s == "no")
        );
        assert!(
            matches!(map.get(&Cow::Borrowed("string5")).unwrap().value, YamlValue::String(ref s) if s == "on")
        );
        assert!(
            matches!(map.get(&Cow::Borrowed("string6")).unwrap().value, YamlValue::String(ref s) if s == "off")
        );
    }
}

#[test]
fn test_special_characters_in_strings() {
    let yaml = r#"
key_with_colon: "value: with colon"
key_with_hash: "value # with hash"
key_with_quotes: "value \"with\" quotes"
"#;

    let result = parse(yaml).expect("Failed to parse special characters");

    if let YamlValue::Object(map) = &result.value {
        if let Some(node) = map.get(&Cow::Borrowed("key_with_colon")) {
            assert!(matches!(node.value, YamlValue::String(ref s) if s == "value: with colon"));
        }
        if let Some(node) = map.get(&Cow::Borrowed("key_with_hash")) {
            assert!(matches!(node.value, YamlValue::String(ref s) if s == "value # with hash"));
        }
        if let Some(node) = map.get(&Cow::Borrowed("key_with_quotes")) {
            assert!(
                matches!(node.value, YamlValue::String(ref s) if s == "value \\\"with\\\" quotes")
            );
        }
    }
}

#[test]
fn test_numbers() {
    let yaml = r#"
positive_int: 42
negative_int: -17
positive_float: 3.15
negative_float: -2.5
scientific: 1.2e-3
"#;

    let result = parse(yaml).expect("Failed to parse numbers");

    if let YamlValue::Object(map) = &result.value {
        assert!(
            matches!(map.get(&Cow::Borrowed("positive_int")).unwrap().value, YamlValue::String(ref s) if s == "42")
        );
        assert!(
            matches!(map.get(&Cow::Borrowed("negative_int")).unwrap().value, YamlValue::String(ref s) if s == "-17")
        );
        assert!(
            matches!(map.get(&Cow::Borrowed("positive_float")).unwrap().value, YamlValue::String(ref s) if s == "3.15")
        );
        assert!(
            matches!(map.get(&Cow::Borrowed("negative_float")).unwrap().value, YamlValue::String(ref s) if s == "-2.5")
        );
        assert!(
            matches!(map.get(&Cow::Borrowed("scientific")).unwrap().value, YamlValue::String(ref s) if s == "1.2e-3")
        );
    }
}

#[test]
fn test_array_of_objects_inline_style() {
    // Test the specific format where array objects have properties on multiple lines
    let yaml = r#"
features:
  - enabled: false
    name: feature1
  - enabled: true
    name: feature2
    priority: high
"#;

    let result = parse(yaml).expect("Failed to parse array with inline object format");

    if let YamlValue::Object(map) = &result.value
        && let Some(features_node) = map.get(&Cow::Borrowed("features"))
        && let YamlValue::Array(features) = &features_node.value
    {
        assert_eq!(features.len(), 2);

        // First feature
        if let YamlValue::Object(f1) = &features[0].value {
            assert_eq!(f1.len(), 2);
            assert!(
                matches!(f1.get(&Cow::Borrowed("enabled")).unwrap().value, YamlValue::String(ref s) if s == "false")
            );
            assert!(
                matches!(f1.get(&Cow::Borrowed("name")).unwrap().value, YamlValue::String(ref s) if s == "feature1")
            );
        }

        // Second feature
        if let YamlValue::Object(f2) = &features[1].value {
            assert_eq!(f2.len(), 3);
            assert!(
                matches!(f2.get(&Cow::Borrowed("enabled")).unwrap().value, YamlValue::String(ref s) if s == "true")
            );
            assert!(
                matches!(f2.get(&Cow::Borrowed("name")).unwrap().value, YamlValue::String(ref s) if s == "feature2")
            );
            assert!(
                matches!(f2.get(&Cow::Borrowed("priority")).unwrap().value, YamlValue::String(ref s) if s == "high")
            );
        }
    }
}
