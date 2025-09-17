#![deny(clippy::all)]

use yamp::{parse, YamlValue};

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
        assert!(map.contains_key("server"));
    }

    if let YamlValue::Object(map) = &result2.value {
        assert!(map.contains_key("database"));
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

    let map = match &result.value {
        YamlValue::Object(m) => m,
        YamlValue::String(_) | YamlValue::Array(_) => {
            panic!("Expected YamlValue::Object at root, got {:?}", result.value)
        }
    };

    let l1 = map.get("level1").expect("Key 'level1' not found in map");
    let l1_map = match &l1.value {
        YamlValue::Object(m) => m,
        YamlValue::String(_) | YamlValue::Array(_) => {
            panic!("Expected YamlValue::Object for level1, got {:?}", l1.value)
        }
    };

    let l2 = l1_map
        .get("level2")
        .expect("Key 'level2' not found in level1");
    let l2_map = match &l2.value {
        YamlValue::Object(m) => m,
        YamlValue::String(_) | YamlValue::Array(_) => {
            panic!("Expected YamlValue::Object for level2, got {:?}", l2.value)
        }
    };

    let l3 = l2_map
        .get("level3")
        .expect("Key 'level3' not found in level2");
    let l3_map = match &l3.value {
        YamlValue::Object(m) => m,
        YamlValue::String(_) | YamlValue::Array(_) => {
            panic!("Expected YamlValue::Object for level3, got {:?}", l3.value)
        }
    };

    let val = l3_map
        .get("value")
        .expect("Key 'value' not found in level3");
    let s = match &val.value {
        YamlValue::String(s) => s,
        YamlValue::Object(_) | YamlValue::Array(_) => {
            panic!("Expected YamlValue::String for value, got {:?}", val.value)
        }
    };
    assert_eq!(s.as_str(), "deep");
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

    let map = match &result.value {
        YamlValue::Object(m) => m,
        YamlValue::String(_) | YamlValue::Array(_) => {
            panic!("Expected YamlValue::Object at root, got {:?}", result.value)
        }
    };

    let config = map.get("config").expect("Key 'config' not found in map");
    let config_map = match &config.value {
        YamlValue::Object(m) => m,
        YamlValue::String(_) | YamlValue::Array(_) => panic!(
            "Expected YamlValue::Object for config, got {:?}",
            config.value
        ),
    };
    assert!(config_map.contains_key("features"));

    // Verify the array structure
    let features = config_map
        .get("features")
        .expect("Key 'features' not found in config");
    let features_arr = match &features.value {
        YamlValue::Array(arr) => arr,
        YamlValue::String(_) | YamlValue::Object(_) => panic!(
            "Expected YamlValue::Array for features, got {:?}",
            features.value
        ),
    };
    assert_eq!(features_arr.len(), 2);
}
