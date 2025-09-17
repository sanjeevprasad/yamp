#![deny(clippy::all)]

use std::borrow::Cow;
use yamp::{YamlValue, emit, parse};

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

    let parsed = parse(yaml).expect("Failed to parse");

    // Verify parsing - all values are now strings
    if let YamlValue::Object(map) = &parsed.value {
        assert!(
            matches!(map.get(&Cow::Borrowed("test_yes")).unwrap().value, YamlValue::String(ref s) if s == "yes")
        );
        assert!(
            matches!(map.get(&Cow::Borrowed("test_no")).unwrap().value, YamlValue::String(ref s) if s == "no")
        );
        assert!(
            matches!(map.get(&Cow::Borrowed("test_on")).unwrap().value, YamlValue::String(ref s) if s == "on")
        );
        assert!(
            matches!(map.get(&Cow::Borrowed("test_off")).unwrap().value, YamlValue::String(ref s) if s == "off")
        );
        assert!(
            matches!(map.get(&Cow::Borrowed("test_true")).unwrap().value, YamlValue::String(ref s) if s == "true")
        );
        assert!(
            matches!(map.get(&Cow::Borrowed("test_false")).unwrap().value, YamlValue::String(ref s) if s == "false")
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

    let parsed = parse(yaml).expect("Failed to parse");
    let emitted = emit(&parsed);
    let reparsed = parse(&emitted).expect("Failed to reparse");

    // Verify the roundtrip preserves values - all are strings now
    if let YamlValue::Object(original_map) = &parsed.value {
        if let YamlValue::Object(reparsed_map) = &reparsed.value {
            // yes should remain a string
            assert!(
                matches!(original_map.get(&Cow::Borrowed("bool_string")).unwrap().value, YamlValue::String(ref s) if s == "yes")
            );
            assert!(
                matches!(reparsed_map.get(&Cow::Borrowed("bool_string")).unwrap().value, YamlValue::String(ref s) if s == "yes")
            );

            // true is now also a string
            assert!(
                matches!(original_map.get(&Cow::Borrowed("true_bool")).unwrap().value, YamlValue::String(ref s) if s == "true")
            );
            assert!(
                matches!(reparsed_map.get(&Cow::Borrowed("true_bool")).unwrap().value, YamlValue::String(ref s) if s == "true")
            );
        }
    }
}
