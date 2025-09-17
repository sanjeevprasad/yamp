#![deny(clippy::all)]

use yamp::{emit, parse, YamlValue};

#[test]
fn test_emit_non_standard_booleans_as_strings() {
    let yaml = r#"
test_yes: yes
test_no: no
test_on: on
test_off: off
test_true: true
test_false: false
"#;

    let parsed = parse(yaml).expect("Failed to parse YAML");

    // Verify parsing - all values are now strings
    if let YamlValue::Object(map) = &parsed.value {
        assert_eq!(map.get("test_yes").and_then(|n| n.as_str()), Some("yes"));
        assert_eq!(map.get("test_no").and_then(|n| n.as_str()), Some("no"));
        assert_eq!(map.get("test_on").and_then(|n| n.as_str()), Some("on"));
        assert_eq!(map.get("test_off").and_then(|n| n.as_str()), Some("off"));
        assert_eq!(map.get("test_true").and_then(|n| n.as_str()), Some("true"));
        assert_eq!(
            map.get("test_false").and_then(|n| n.as_str()),
            Some("false")
        );
    }

    // Verify emission
    let emitted = emit(&parsed);

    // Non-standard booleans should remain as strings
    assert!(emitted.contains("test_yes: yes"));
    assert!(emitted.contains("test_no: no"));
    assert!(emitted.contains("test_on: on"));
    assert!(emitted.contains("test_off: off"));
    // true/false/null are quoted to preserve them as strings
    assert!(emitted.contains("test_true: \"true\""));
    assert!(emitted.contains("test_false: \"false\""));
}

#[test]
fn test_roundtrip_boolean_strings() {
    let yaml = "bool_string: yes\ntrue_bool: true";

    let parsed = parse(yaml).expect("Failed to parse YAML");
    let emitted = emit(&parsed);
    let reparsed = parse(&emitted).expect("Failed to reparse emitted YAML");

    // Verify the roundtrip preserves values - all are strings now
    if let YamlValue::Object(original_map) = &parsed.value {
        if let YamlValue::Object(reparsed_map) = &reparsed.value {
            // yes should remain a string
            assert_eq!(
                original_map.get("bool_string").and_then(|n| n.as_str()),
                Some("yes")
            );
            assert_eq!(
                reparsed_map.get("bool_string").and_then(|n| n.as_str()),
                Some("yes")
            );

            // true is now also a string
            assert_eq!(
                original_map.get("true_bool").and_then(|n| n.as_str()),
                Some("true")
            );
            assert_eq!(
                reparsed_map.get("true_bool").and_then(|n| n.as_str()),
                Some("true")
            );
        }
    }
}
