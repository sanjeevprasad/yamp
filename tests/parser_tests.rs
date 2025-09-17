#![deny(clippy::all)]

use yamp::{parse, YamlValue};

#[test]
fn test_simple_key_value() {
    let yaml = "key: value";
    let result = parse(yaml).expect("Failed to parse simple key-value");

    let map = match &result.value {
        YamlValue::Object(m) => m,
        YamlValue::String(_) | YamlValue::Array(_) => {
            panic!("Expected object at root, got {:?}", result.value)
        }
    };
    assert_eq!(map.len(), 1);
    assert!(map.contains_key("key"));

    let key_node = map.get("key").expect("key not found");
    let s = match &key_node.value {
        YamlValue::String(s) => s,
        YamlValue::Object(_) | YamlValue::Array(_) => {
            panic!("Expected string value, got {:?}", key_node.value)
        }
    };
    assert_eq!(s.as_str(), "value")
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
        assert_eq!(map.get("string").and_then(|n| n.as_str()), Some("hello"));

        // Check integer - now a string
        assert_eq!(map.get("integer").and_then(|n| n.as_str()), Some("42"));

        // Check float - now a string
        assert_eq!(map.get("float").and_then(|n| n.as_str()), Some("3.15"));

        // Check booleans - now strings
        assert_eq!(
            map.get("boolean_true").and_then(|n| n.as_str()),
            Some("true")
        );
        assert_eq!(
            map.get("boolean_false").and_then(|n| n.as_str()),
            Some("false")
        );

        // Check null - now a string
        assert_eq!(map.get("null_value").and_then(|n| n.as_str()), Some("null"));
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

    if let YamlValue::Object(map) = &result.value {
        let db_node = map.get("database").expect("database not found");
        if let YamlValue::Object(db_map) = &db_node.value {
            assert!(db_map.contains_key("host"));
            assert!(db_map.contains_key("port"));

            let creds_node = db_map.get("credentials").expect("credentials not found");
            if let YamlValue::Object(creds_map) = &creds_node.value {
                assert!(creds_map.contains_key("username"));
                assert!(creds_map.contains_key("password"));
            }
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

    if let YamlValue::Object(map) = &result.value {
        let users_node = map.get("users").expect("users not found");
        if let YamlValue::Array(users) = &users_node.value {
            assert_eq!(users.len(), 2);

            // Check first user
            if let YamlValue::Object(user1) = &users[0].value {
                assert_eq!(user1.get("name").and_then(|n| n.as_str()), Some("Alice"));
                assert_eq!(user1.get("age").and_then(|n| n.as_str()), Some("30"));
            }

            // Check second user
            if let YamlValue::Object(user2) = &users[1].value {
                assert_eq!(user2.get("name").and_then(|n| n.as_str()), Some("Bob"));
                assert_eq!(user2.get("age").and_then(|n| n.as_str()), Some("25"));
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

    let YamlValue::Object(map) = &result.value else {
        panic!("Expected object, got {:?}", result.value);
    };

    // All boolean values are now strings
    assert_eq!(map.get("bool1").and_then(|n| n.as_str()), Some("true"));
    assert_eq!(map.get("bool2").and_then(|n| n.as_str()), Some("false"));

    // yes/no/on/off should be strings
    assert_eq!(map.get("string3").and_then(|n| n.as_str()), Some("yes"));
    assert_eq!(map.get("string4").and_then(|n| n.as_str()), Some("no"));
    assert_eq!(map.get("string5").and_then(|n| n.as_str()), Some("on"));
    assert_eq!(map.get("string6").and_then(|n| n.as_str()), Some("off"));
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
        assert_eq!(
            map.get("key_with_colon").and_then(|n| n.as_str()),
            Some("value: with colon")
        );
        assert_eq!(
            map.get("key_with_hash").and_then(|n| n.as_str()),
            Some("value # with hash")
        );
        assert_eq!(
            map.get("key_with_quotes").and_then(|n| n.as_str()),
            Some("value \\\"with\\\" quotes")
        );
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
        assert_eq!(map.get("positive_int").and_then(|n| n.as_str()), Some("42"));
        assert_eq!(
            map.get("negative_int").and_then(|n| n.as_str()),
            Some("-17")
        );
        assert_eq!(
            map.get("positive_float").and_then(|n| n.as_str()),
            Some("3.15")
        );
        assert_eq!(
            map.get("negative_float").and_then(|n| n.as_str()),
            Some("-2.5")
        );
        assert_eq!(
            map.get("scientific").and_then(|n| n.as_str()),
            Some("1.2e-3")
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

    if let YamlValue::Object(map) = &result.value {
        let features_node = map.get("features").expect("features not found");
        if let YamlValue::Array(features) = &features_node.value {
            assert_eq!(features.len(), 2);

            // First feature
            if let YamlValue::Object(f1) = &features[0].value {
                assert_eq!(f1.len(), 2);
                assert_eq!(f1.get("enabled").and_then(|n| n.as_str()), Some("false"));
                assert_eq!(f1.get("name").and_then(|n| n.as_str()), Some("feature1"));
            }

            // Second feature
            if let YamlValue::Object(f2) = &features[1].value {
                assert_eq!(f2.len(), 3);
                assert_eq!(f2.get("enabled").and_then(|n| n.as_str()), Some("true"));
                assert_eq!(f2.get("name").and_then(|n| n.as_str()), Some("feature2"));
                assert_eq!(f2.get("priority").and_then(|n| n.as_str()), Some("high"));
            }
        }
    }
}
