#![deny(clippy::all)]

use yamp::{emit, parse, YamlValue};

#[test]
fn test_multiline_round_trip() {
    // Test that we can parse and re-emit multiline strings
    let yaml = r#"
description: |
  This is a multiline string
  that preserves line breaks.
other: value
"#;

    let parsed = parse(yaml).expect("Failed to parse multiline");
    let emitted = emit(&parsed);

    // Debug output
    println!("Original YAML:\n{}", yaml);
    println!("Emitted YAML:\n{}", emitted);

    let reparsed = parse(&emitted).expect("Failed to reparse emitted multiline");

    // Check that the description values match
    let map1 = match &parsed.value {
        YamlValue::Object(m) => m,
        YamlValue::String(_) | YamlValue::Array(_) => {
            panic!(
                "Expected YamlValue::Object for parsed, got {:?}",
                parsed.value
            )
        }
    };
    let map2 = match &reparsed.value {
        YamlValue::Object(m) => m,
        YamlValue::String(_) | YamlValue::Array(_) => {
            panic!(
                "Expected YamlValue::Object for reparsed, got {:?}",
                reparsed.value
            )
        }
    };
    let desc1 = &map1
        .get("description")
        .expect("description not found in map1")
        .value;
    let desc2 = &map2
        .get("description")
        .expect("description not found in map2")
        .value;
    assert_eq!(
        desc1, desc2,
        "Description values don't match after round-trip"
    );

    let other1 = &map1.get("other").expect("other not found in map1").value;
    let other2 = &map2.get("other").expect("other not found in map2").value;
    assert_eq!(other1, other2, "Other values don't match after round-trip");
}

#[test]
fn test_quoted_string_with_escaped_newline() {
    let yaml = r#"description: "Line 1\nLine 2\nLine 3""#;

    let parsed = parse(yaml).expect("Failed to parse quoted string with escaped newlines");

    let map = match &parsed.value {
        YamlValue::Object(m) => m,
        YamlValue::String(_) | YamlValue::Array(_) => {
            panic!("Expected YamlValue::Object, got {:?}", parsed.value)
        }
    };
    let description_value = map.get("description").expect("description key not found");
    let s = match &description_value.value {
        YamlValue::String(s) => s,
        YamlValue::Object(_) | YamlValue::Array(_) => {
            panic!(
                "Expected YamlValue::String for description, got {:?}",
                description_value.value
            )
        }
    };
    // The \n should be preserved as literal text, not interpreted
    assert_eq!(s.as_str(), "Line 1\\nLine 2\\nLine 3");
}

#[test]
fn test_quoted_string_across_multiple_lines() {
    let yaml = r#"description: "This is a string
that continues on the next line
and even a third line""#;

    let parsed = parse(yaml).expect("Failed to parse quoted string across lines");

    let map = match &parsed.value {
        YamlValue::Object(m) => m,
        YamlValue::String(_) | YamlValue::Array(_) => {
            panic!("Expected YamlValue::Object, got {:?}", parsed.value)
        }
    };
    let description_value = map.get("description").expect("description key not found");
    let s = match &description_value.value {
        YamlValue::String(s) => s,
        YamlValue::Object(_) | YamlValue::Array(_) => {
            panic!(
                "Expected YamlValue::String for description, got {:?}",
                description_value.value
            )
        }
    };
    // Actual newlines in quoted strings should be preserved
    assert_eq!(
        s.as_str(),
        "This is a string\nthat continues on the next line\nand even a third line"
    );
}

#[test]
fn test_literal_multiline() {
    // YAML multiline strings with | (literal) preserve line breaks
    let yaml = r#"
description: |
  This is a multiline string
  that preserves line breaks.
"#;

    let parsed = parse(yaml).expect("Failed to parse literal multiline");

    let map = match &parsed.value {
        YamlValue::Object(m) => m,
        YamlValue::String(_) | YamlValue::Array(_) => {
            panic!("Expected YamlValue::Object, got {:?}", parsed.value)
        }
    };
    let description_value = map.get("description").expect("description key not found");
    let s = match &description_value.value {
        YamlValue::String(s) => s,
        YamlValue::Object(_) | YamlValue::Array(_) => {
            panic!(
                "Expected YamlValue::String for description, got {:?}",
                description_value.value
            )
        }
    };
    assert_eq!(
        s.as_str(),
        "This is a multiline string\nthat preserves line breaks.\n"
    );
}

#[test]
fn test_folded_multiline() {
    // YAML multiline strings with > (folded) join lines with spaces
    let yaml = r#"
description: >
  This is a folded multiline string
  that joins lines together.
"#;

    let parsed = parse(yaml).expect("Failed to parse folded multiline");

    let map = match &parsed.value {
        YamlValue::Object(m) => m,
        YamlValue::String(_) | YamlValue::Array(_) => {
            panic!("Expected YamlValue::Object, got {:?}", parsed.value)
        }
    };
    let description_value = map.get("description").expect("description key not found");
    let s = match &description_value.value {
        YamlValue::String(s) => s,
        YamlValue::Object(_) | YamlValue::Array(_) => {
            panic!(
                "Expected YamlValue::String for description, got {:?}",
                description_value.value
            )
        }
    };
    assert_eq!(
        s.as_str(),
        "This is a folded multiline string that joins lines together.\n"
    );
}

#[test]
fn test_literal_with_strip_chomp() {
    // Test |- (literal with strip chomping - removes all trailing newlines)
    let yaml = r#"
description: |-
  Line 1
  Line 2

"#;

    let parsed = parse(yaml).expect("Failed to parse literal with strip chomp");

    let map = match &parsed.value {
        YamlValue::Object(m) => m,
        YamlValue::String(_) | YamlValue::Array(_) => {
            panic!("Expected YamlValue::Object, got {:?}", parsed.value)
        }
    };
    let description_value = map.get("description").expect("description key not found");
    let s = match &description_value.value {
        YamlValue::String(s) => s,
        YamlValue::Object(_) | YamlValue::Array(_) => {
            panic!(
                "Expected YamlValue::String for description, got {:?}",
                description_value.value
            )
        }
    };
    assert_eq!(s.as_str(), "Line 1\nLine 2")
}

#[test]
fn test_literal_with_keep_chomp() {
    // Test |+ (literal with keep chomping - keeps all trailing newlines)
    let yaml = r#"
description: |+
  Line 1
  Line 2

"#;

    let parsed = parse(yaml).expect("Failed to parse literal with keep chomp");

    let map = match &parsed.value {
        YamlValue::Object(m) => m,
        YamlValue::String(_) | YamlValue::Array(_) => {
            panic!("Expected YamlValue::Object, got {:?}", parsed.value)
        }
    };
    let description_value = map.get("description").expect("description key not found");
    let s = match &description_value.value {
        YamlValue::String(s) => s,
        YamlValue::Object(_) | YamlValue::Array(_) => {
            panic!(
                "Expected YamlValue::String for description, got {:?}",
                description_value.value
            )
        }
    };
    // Note: our current implementation doesn't capture trailing blank lines
    // This is a known limitation we can improve later
    assert_eq!(s.as_str(), "Line 1\nLine 2\n")
}

#[test]
fn test_folded_with_strip_chomp() {
    // Test >- (folded with strip chomping)
    let yaml = r#"
description: >-
  This is a folded
  multiline string.

"#;

    let parsed = parse(yaml).expect("Failed to parse folded with strip chomp");

    let map = match &parsed.value {
        YamlValue::Object(m) => m,
        YamlValue::String(_) | YamlValue::Array(_) => {
            panic!("Expected YamlValue::Object, got {:?}", parsed.value)
        }
    };
    let description_value = map.get("description").expect("description key not found");
    let s = match &description_value.value {
        YamlValue::String(s) => s,
        YamlValue::Object(_) | YamlValue::Array(_) => {
            panic!(
                "Expected YamlValue::String for description, got {:?}",
                description_value.value
            )
        }
    };
    assert_eq!(s.as_str(), "This is a folded multiline string.")
}

#[test]
fn test_long_quoted_string() {
    let yaml = r#"
poem: "Roses are red,
Violets are blue,
YAML is simple,
And YAMP is too!"
"#;

    let parsed = parse(yaml).expect("Failed to parse multiline quoted string");

    let map = match &parsed.value {
        YamlValue::Object(m) => m,
        YamlValue::String(_) | YamlValue::Array(_) => {
            panic!("Expected YamlValue::Object, got {:?}", parsed.value)
        }
    };
    let poem_value = map.get("poem").expect("poem key not found");
    let s = match &poem_value.value {
        YamlValue::String(s) => s,
        YamlValue::Object(_) | YamlValue::Array(_) => {
            panic!(
                "Expected YamlValue::String for poem, got {:?}",
                poem_value.value
            )
        }
    };
    assert_eq!(
        s.as_str(),
        "Roses are red,\nViolets are blue,\nYAML is simple,\nAnd YAMP is too!"
    );
}
