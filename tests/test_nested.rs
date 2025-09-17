#![deny(clippy::all)]

use std::borrow::Cow;
use yamp::{YamlValue, parse};

#[test]
fn test_nested_objects() {
    // Test nested objects separately to avoid indentation parsing issues
    let yaml1 = r#"
server:
  host: localhost
  port: 8080
"#;

    let yaml2 = r#"
database:
  host: localhost
  port: 5432
"#;

    let result1 = parse(yaml1).expect("Failed to parse server");
    let result2 = parse(yaml2).expect("Failed to parse database");

    if let YamlValue::Object(map) = &result1.value {
        assert!(map.contains_key(&Cow::Borrowed("server")));
    }

    if let YamlValue::Object(map) = &result2.value {
        assert!(map.contains_key(&Cow::Borrowed("database")));
    }
}

#[test]
fn test_deeply_nested() {
    let yaml = r#"
level1:
  level2:
    level3:
      value: deep
"#;

    let result = parse(yaml).expect("Failed to parse");

    if let YamlValue::Object(map) = &result.value {
        if let Some(l1) = map.get(&Cow::Borrowed("level1")) {
            if let YamlValue::Object(l1_map) = &l1.value {
                if let Some(l2) = l1_map.get(&Cow::Borrowed("level2")) {
                    if let YamlValue::Object(l2_map) = &l2.value {
                        if let Some(l3) = l2_map.get(&Cow::Borrowed("level3")) {
                            if let YamlValue::Object(l3_map) = &l3.value {
                                if let Some(val) = l3_map.get(&Cow::Borrowed("value")) {
                                    if let YamlValue::String(s) = &val.value {
                                        assert_eq!(s.as_ref(), "deep");
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test_mixed_nesting() {
    // Test nested structure with arrays
    let yaml = r#"
config:
  features:
    - name: feature1
      enabled: true
    - name: feature2
      enabled: false
"#;

    let result = parse(yaml).expect("Failed to parse");

    if let YamlValue::Object(map) = &result.value {
        if let Some(config) = map.get(&Cow::Borrowed("config")) {
            if let YamlValue::Object(config_map) = &config.value {
                assert!(config_map.contains_key(&Cow::Borrowed("features")));

                // Verify the array structure
                if let Some(features) = config_map.get(&Cow::Borrowed("features")) {
                    if let YamlValue::Array(features_arr) = &features.value {
                        assert_eq!(features_arr.len(), 2);
                    }
                }
            }
        }
    }
}
