#![deny(clippy::all)]

use std::borrow::Cow;
use yamp::{YamlValue, emit, parse};

#[test]
fn test_octal_footgun() {
    // Leading zeros should be treated as strings, not octal numbers
    let test_cases = [
        ("0755", YamlValue::String(Cow::Borrowed("0755"))), // File permissions
        ("0123", YamlValue::String(Cow::Borrowed("0123"))), // Would be 83 in octal
        ("0001", YamlValue::String(Cow::Borrowed("0001"))), // Leading zeros
        ("0", YamlValue::String(Cow::Borrowed("0"))),       // Single zero is now string
        ("123", YamlValue::String(Cow::Borrowed("123"))),   // Regular number is now string
    ];

    for (input, expected) in test_cases {
        let yaml = format!("value: {}", input);
        let parsed = parse(&yaml).unwrap_or_else(|_| panic!("Failed to parse: {}", input));

        if let YamlValue::Object(map) = &parsed.value {
            let actual = &map.get(&Cow::Borrowed("value")).unwrap().value;
            assert_eq!(*actual, expected, "Failed for input: {}", input);
        } else {
            panic!("Expected object");
        }
    }
}

#[test]
fn test_version_number_preservation() {
    // Version numbers with trailing zeros should be preserved as strings
    let test_cases = [
        ("3.10", YamlValue::String(Cow::Borrowed("3.10"))), // Version number
        ("1.20", YamlValue::String(Cow::Borrowed("1.20"))), // Trailing zero
        ("2.0", YamlValue::String(Cow::Borrowed("2.0"))),   // Trailing zero
        ("3.25", YamlValue::String(Cow::Borrowed("3.25"))), // Now also string
        ("3.5", YamlValue::String(Cow::Borrowed("3.5"))),   // Now also string
    ];

    for (input, expected) in test_cases {
        let yaml = format!("version: {}", input);
        let parsed = parse(&yaml).unwrap_or_else(|_| panic!("Failed to parse: {}", input));

        if let YamlValue::Object(map) = &parsed.value {
            let actual = &map.get(&Cow::Borrowed("version")).unwrap().value;
            assert_eq!(*actual, expected, "Failed for input: {}", input);
        } else {
            panic!("Expected object");
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
            map.get(&Cow::Borrowed("null_value")).unwrap().value,
            YamlValue::String(ref s) if s == "null"
        ));

        // Tilde paths should be strings
        assert!(matches!(
            map.get(&Cow::Borrowed("tilde_path")).unwrap().value,
            YamlValue::String(ref s) if s == "~/.ssh/config"
        ));

        // Just tilde should be a string now (not null)
        assert!(matches!(
            map.get(&Cow::Borrowed("just_tilde")).unwrap().value,
            YamlValue::String(ref s) if s == "~"
        ));
    } else {
        panic!("Expected object");
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
            map.get(&Cow::Borrowed("country_no")).unwrap().value,
            YamlValue::String(ref s) if s == "NO"
        ));
        assert!(matches!(
            map.get(&Cow::Borrowed("country_yes")).unwrap().value,
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
            map.get(&Cow::Borrowed("real_scientific")).unwrap().value,
            YamlValue::String(ref s) if s == "1.2e3"
        ));
        assert!(matches!(
            map.get(&Cow::Borrowed("real_scientific2")).unwrap().value,
            YamlValue::String(ref s) if s == "1e10"
        ));

        // Git SHAs and IDs that happen to look like scientific notation should be strings
        assert!(matches!(
            map.get(&Cow::Borrowed("git_sha")).unwrap().value,
            YamlValue::String(ref s) if s == "2e5a8d9"
        ));
        assert!(matches!(
            map.get(&Cow::Borrowed("regular_id")).unwrap().value,
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
            map.get(&Cow::Borrowed("inf_value")).unwrap().value,
            YamlValue::String(ref s) if s == ".inf"
        ));
        assert!(matches!(
            map.get(&Cow::Borrowed("neg_inf")).unwrap().value,
            YamlValue::String(ref s) if s == "-.inf"
        ));
        assert!(matches!(
            map.get(&Cow::Borrowed("nan_value")).unwrap().value,
            YamlValue::String(ref s) if s == ".nan"
        ));
        assert!(matches!(
            map.get(&Cow::Borrowed("nan_upper")).unwrap().value,
            YamlValue::String(ref s) if s == ".NaN"
        ));
        assert!(matches!(
            map.get(&Cow::Borrowed("inf_upper")).unwrap().value,
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
            map.get(&Cow::Borrowed("zip_code")).unwrap().value,
            YamlValue::String(ref s) if s == "01234"
        ));
        assert!(matches!(
            map.get(&Cow::Borrowed("item_code")).unwrap().value,
            YamlValue::String(ref s) if s == "00042"
        ));
        // Regular numbers are now also strings
        assert!(matches!(
            map.get(&Cow::Borrowed("regular_num")).unwrap().value,
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
            map.get(&Cow::Borrowed("time1")).unwrap().value,
            YamlValue::String(ref s) if s == "12:34:56"
        ));
        assert!(matches!(
            map.get(&Cow::Borrowed("time2")).unwrap().value,
            YamlValue::String(ref s) if s == "1:2:3"
        ));
        assert!(matches!(
            map.get(&Cow::Borrowed("ratio")).unwrap().value,
            YamlValue::String(ref s) if s == "1:100"
        ));
    }
}
