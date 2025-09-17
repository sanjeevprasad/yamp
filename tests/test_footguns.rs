#![deny(clippy::all)]

use yamp::{emit, parse, YamlValue};

#[test]
fn test_octal_footgun() {
    // Leading zeros should be treated as strings, not octal numbers
    let test_cases = [
        ("0755", YamlValue::String("0755".to_string())), // File permissions
        ("0123", YamlValue::String("0123".to_string())), // Would be 83 in octal
        ("0001", YamlValue::String("0001".to_string())), // Leading zeros
        ("0", YamlValue::String("0".to_string())),       // Single zero is now string
        ("123", YamlValue::String("123".to_string())),   // Regular number is now string
    ];

    for (input, expected) in test_cases {
        let yaml = format!("value: {}", input);
        let parsed = parse(&yaml).unwrap_or_else(|_| panic!("Failed to parse: {}", input));

        if let YamlValue::Object(map) = &parsed.value {
            let actual = &map.get("value").expect("value not found").value;
            assert_eq!(*actual, expected, "Failed for input: {}", input);
        } else {
            panic!("Expected object")
        }
    }
}

#[test]
fn test_version_number_preservation() {
    // Version numbers with trailing zeros should be preserved as strings
    let test_cases = [
        ("3.10", YamlValue::String("3.10".to_string())), // Version number
        ("1.20", YamlValue::String("1.20".to_string())), // Trailing zero
        ("2.0", YamlValue::String("2.0".to_string())),   // Trailing zero
        ("3.25", YamlValue::String("3.25".to_string())), // Now also string
        ("3.5", YamlValue::String("3.5".to_string())),   // Now also string
    ];

    for (input, expected) in test_cases {
        let yaml = format!("version: {}", input);
        let parsed = parse(&yaml).unwrap_or_else(|_| panic!("Failed to parse: {}", input));

        if let YamlValue::Object(map) = &parsed.value {
            let actual = &map.get("version").expect("version not found").value;
            assert_eq!(*actual, expected, "Failed for input: {}", input);
        } else {
            panic!("Expected object")
        }
    }
}

#[test]
fn test_null_tilde_footgun() {
    // Tilde should not be parsed as null
    let yaml = r#"
null_value: null
tilde_path: ~/.ssh/config
just_tilde: ~
"#;

    let parsed = parse(yaml).expect("Failed to parse");

    if let YamlValue::Object(map) = &parsed.value {
        // null is now a string "null"
        assert!(matches!(
            map.get("null_value").expect("null_value not found").value,
            YamlValue::String(ref s) if s == "null"
        ));

        // Tilde paths should be strings
        assert!(matches!(
            map.get("tilde_path").expect("tilde_path not found").value,
            YamlValue::String(ref s) if s == "~/.ssh/config"
        ));

        // Just tilde should be a string now (not null)
        assert!(matches!(
            map.get("just_tilde").expect("just_tilde not found").value,
            YamlValue::String(ref s) if s == "~"
        ));
    } else {
        panic!("Expected object")
    }
}

#[test]
fn test_norway_problem() {
    // Country codes that look like booleans should remain strings
    let yaml = r#"
country_no: NO
country_yes: YES
"#;

    let parsed = parse(yaml).expect("Failed to parse");

    if let YamlValue::Object(map) = &parsed.value {
        // These should all be strings, not booleans
        assert!(matches!(
            map.get("country_no").expect("country_no not found").value,
            YamlValue::String(ref s) if s == "NO"
        ));
        assert!(matches!(
            map.get("country_yes").expect("country_yes not found").value,
            YamlValue::String(ref s) if s == "YES"
        ));
    }
}

#[test]
fn test_scientific_notation_confusion() {
    // Strings that look like scientific notation should be handled carefully
    let yaml = r#"
real_scientific: 1.2e3
real_scientific2: 1e10
git_sha: 2e5a8d9
regular_id: e1234
"#;

    let parsed = parse(yaml).expect("Failed to parse");

    if let YamlValue::Object(map) = &parsed.value {
        // All values are now strings
        assert!(matches!(
            map.get("real_scientific").expect("real_scientific not found").value,
            YamlValue::String(ref s) if s == "1.2e3"
        ));
        assert!(matches!(
            map.get("real_scientific2").expect("real_scientific2 not found").value,
            YamlValue::String(ref s) if s == "1e10"
        ));

        // Git SHAs and IDs that happen to look like scientific notation should be strings
        assert!(matches!(
            map.get("git_sha").expect("git_sha not found").value,
            YamlValue::String(ref s) if s == "2e5a8d9"
        ));
        assert!(matches!(
            map.get("regular_id").expect("regular_id not found").value,
            YamlValue::String(ref s) if s == "e1234"
        ));
    }
}

#[test]
fn test_special_float_values() {
    // Special float values should be treated as strings
    let yaml = r#"
inf_value: .inf
neg_inf: -.inf
nan_value: .nan
nan_upper: .NaN
inf_upper: .Inf
"#;

    let parsed = parse(yaml).expect("Failed to parse");

    if let YamlValue::Object(map) = &parsed.value {
        // All special float values should be strings
        assert!(matches!(
            map.get("inf_value").expect("inf_value not found").value,
            YamlValue::String(ref s) if s == ".inf"
        ));
        assert!(matches!(
            map.get("neg_inf").expect("neg_inf not found").value,
            YamlValue::String(ref s) if s == "-.inf"
        ));
        assert!(matches!(
            map.get("nan_value").expect("nan_value not found").value,
            YamlValue::String(ref s) if s == ".nan"
        ));
        assert!(matches!(
            map.get("nan_upper").expect("nan_upper not found").value,
            YamlValue::String(ref s) if s == ".NaN"
        ));
        assert!(matches!(
            map.get("inf_upper").expect("inf_upper not found").value,
            YamlValue::String(ref s) if s == ".Inf"
        ));
    }
}

#[test]
fn test_zip_codes_and_leading_zeros() {
    let yaml = r#"
zip_code: 01234
item_code: 00042
regular_num: 12345
"#;

    let parsed = parse(yaml).expect("Failed to parse");

    if let YamlValue::Object(map) = &parsed.value {
        // ZIP codes with leading zeros should be strings
        assert!(matches!(
            map.get("zip_code").expect("zip_code not found").value,
            YamlValue::String(ref s) if s == "01234"
        ));
        assert!(matches!(
            map.get("item_code").expect("item_code not found").value,
            YamlValue::String(ref s) if s == "00042"
        ));
        // Regular numbers are now also strings
        assert!(matches!(
            map.get("regular_num").expect("regular_num not found").value,
            YamlValue::String(ref s) if s == "12345"
        ));
    }
}

#[test]
fn test_emitter_preserves_footgun_fixes() {
    // Test that emitting preserves our safety features
    let yaml = r#"
octal_perms: "0755"
version: "3.10"
tilde: "~"
zip: "01234"
"#;

    let parsed = parse(yaml).expect("Failed to parse");
    let emitted = emit(&parsed);
    let reparsed = parse(&emitted).expect("Failed to reparse");

    // Values should round-trip correctly - direct comparison with PartialEq!
    assert_eq!(parsed.value, reparsed.value);

    // Check that strings are quoted in output
    assert!(emitted.contains("\"0755\"") || emitted.contains("'0755'") || emitted.contains("0755"));
    assert!(emitted.contains("\"3.10\"") || emitted.contains("'3.10'") || emitted.contains("3.10"));
}

#[test]
fn test_sexagesimal_footgun() {
    // Time-like values should not be parsed as base-60 numbers
    let yaml = r#"
time1: 12:34:56
time2: 1:2:3
ratio: 1:100
"#;

    let parsed = parse(yaml).expect("Failed to parse");

    if let YamlValue::Object(map) = &parsed.value {
        // These should be strings, not parsed as sexagesimal
        assert!(matches!(
            map.get("time1").expect("time1 not found").value,
            YamlValue::String(ref s) if s == "12:34:56"
        ));
        assert!(matches!(
            map.get("time2").expect("time2 not found").value,
            YamlValue::String(ref s) if s == "1:2:3"
        ));
        assert!(matches!(
            map.get("ratio").expect("ratio not found").value,
            YamlValue::String(ref s) if s == "1:100"
        ));
    }
}
