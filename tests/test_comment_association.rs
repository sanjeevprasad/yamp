#![deny(clippy::all)]

use yamp::{emit, parse, YamlValue};

#[test]
fn test_leading_comments_for_simple_values() {
    let yaml = r#"# Leading comment for name
name: John Doe

# Leading comment for age
age: 30"#;

    let parsed = parse(yaml).expect("Failed to parse YAML");

    let YamlValue::Object(map) = &parsed.value else {
        panic!("Expected object, got {:?}", parsed.value);
    };

    // Check that name has the leading comment
    let name_node = map.get("name").expect("name not found");
    assert_eq!(
        name_node.leading_comment.as_deref(),
        Some("Leading comment for name"),
        "Leading comment for 'name' not captured"
    );
    assert_eq!(name_node.as_str(), Some("John Doe"));

    // Check that age has the leading comment
    let age_node = map.get("age").expect("age not found");
    assert_eq!(
        age_node.leading_comment.as_deref(),
        Some("Leading comment for age"),
        "Leading comment for 'age' not captured"
    );
    assert_eq!(age_node.as_str(), Some("30"));
}

#[test]
fn test_leading_and_inline_comments_together() {
    let yaml = r#"# Leading comment for server
server: production  # Inline comment for server

# Leading comment for port
port: 8080  # Inline comment for port"#;

    let parsed = parse(yaml).expect("Failed to parse YAML");

    let YamlValue::Object(map) = &parsed.value else {
        panic!("Expected object, got {:?}", parsed.value);
    };

    // Check server node has both comments
    let server_node = map.get("server").expect("server not found");
    assert_eq!(
        server_node.leading_comment.as_deref(),
        Some("Leading comment for server"),
        "Leading comment for 'server' not captured"
    );
    assert_eq!(
        server_node.inline_comment.as_deref(),
        Some("Inline comment for server"),
        "Inline comment for 'server' not captured"
    );
    assert_eq!(server_node.as_str(), Some("production"));

    // Check port node has both comments
    let port_node = map.get("port").expect("port not found");
    assert_eq!(
        port_node.leading_comment.as_deref(),
        Some("Leading comment for port"),
        "Leading comment for 'port' not captured"
    );
    assert_eq!(
        port_node.inline_comment.as_deref(),
        Some("Inline comment for port"),
        "Inline comment for 'port' not captured"
    );
    assert_eq!(port_node.as_str(), Some("8080"));
}

#[test]
fn test_leading_comments_in_nested_objects() {
    let yaml = r#"# Leading comment for database section
database:
  # Leading comment for host
  host: localhost
  # Leading comment for port
  port: 5432
  # Leading comment for credentials
  credentials:
    # Leading comment for username
    username: admin
    # Leading comment for password
    password: secret"#;

    let parsed = parse(yaml).expect("Failed to parse YAML");

    let YamlValue::Object(map) = &parsed.value else {
        panic!("Expected object, got {:?}", parsed.value);
    };

    // Check database has leading comment
    let db_node = map.get("database").expect("database not found");
    assert_eq!(
        db_node.leading_comment.as_deref(),
        Some("Leading comment for database section"),
        "Leading comment for 'database' not captured"
    );

    let YamlValue::Object(db_map) = &db_node.value else {
        panic!("Expected object for database, got {:?}", db_node.value);
    };
    // Check nested host has leading comment
    let host_node = db_map.get("host").expect("host not found");
    assert_eq!(
        host_node.leading_comment.as_deref(),
        Some("Leading comment for host"),
        "Leading comment for nested 'host' not captured"
    );
    assert_eq!(host_node.as_str(), Some("localhost"));

    // Check nested port has leading comment
    let port_node = db_map.get("port").expect("port not found");
    assert_eq!(
        port_node.leading_comment.as_deref(),
        Some("Leading comment for port"),
        "Leading comment for nested 'port' not captured"
    );
    assert_eq!(port_node.as_str(), Some("5432"));

    // Check nested credentials object has leading comment
    let creds_node = db_map.get("credentials").expect("credentials not found");
    assert_eq!(
        creds_node.leading_comment.as_deref(),
        Some("Leading comment for credentials"),
        "Leading comment for nested 'credentials' not captured"
    );

    let YamlValue::Object(creds_map) = &creds_node.value else {
                panic!("Expected object for credentials, got {:?}", creds_node.value);
            };
    // Check deeply nested username has leading comment
    let user_node = creds_map.get("username").expect("username not found");
    assert_eq!(
        user_node.leading_comment.as_deref(),
        Some("Leading comment for username"),
        "Leading comment for deeply nested 'username' not captured"
    );
    assert_eq!(user_node.as_str(), Some("admin"));

    // Check deeply nested password has leading comment
    let pass_node = creds_map.get("password").expect("password not found");
    assert_eq!(
        pass_node.leading_comment.as_deref(),
        Some("Leading comment for password"),
        "Leading comment for deeply nested 'password' not captured"
    );
    assert_eq!(pass_node.as_str(), Some("secret"));
}

#[test]
fn test_leading_comments_in_arrays() {
    let yaml = r#"# Leading comment for items array
items:
  # Leading comment for first item
  - apple
  # Leading comment for second item
  - banana
  # Leading comment for third item
  - cherry"#;

    let parsed = parse(yaml).expect("Failed to parse YAML");

    let YamlValue::Object(map) = &parsed.value else {
        panic!("Expected object, got {:?}", parsed.value);
    };

    // Check array has leading comment
    let items_node = map.get("items").expect("items not found");
    assert_eq!(
        items_node.leading_comment.as_deref(),
        Some("Leading comment for items array"),
        "Leading comment for 'items' array not captured"
    );

    let YamlValue::Array(items) = &items_node.value else {
        panic!("Expected array for items, got {:?}", items_node.value);
    };
    assert_eq!(items.len(), 3);

    // Check first item has leading comment
    assert_eq!(
        items[0].leading_comment.as_deref(),
        Some("Leading comment for first item"),
        "Leading comment for first array item not captured"
    );
    assert_eq!(items[0].as_str(), Some("apple"));

    // Check second item has leading comment
    assert_eq!(
        items[1].leading_comment.as_deref(),
        Some("Leading comment for second item"),
        "Leading comment for second array item not captured"
    );
    assert_eq!(items[1].as_str(), Some("banana"));

    // Check third item has leading comment
    assert_eq!(
        items[2].leading_comment.as_deref(),
        Some("Leading comment for third item"),
        "Leading comment for third array item not captured"
    );
    assert_eq!(items[2].as_str(), Some("cherry"));
}

#[test]
fn test_comments_preserved_after_round_trip() {
    let yaml = r#"# Application configuration file
# This is a multi-line leading comment

# Leading comment for name
name: MyApp  # Application name

# Leading comment for version
version: 1.0.0  # Semantic version

# Server configuration section
server:
  # Server host
  host: localhost  # Can be IP or hostname
  # Server port
  port: 8080  # Default port

# Feature flags
features:
  # Enable new UI
  - ui_v2
  # Enable analytics
  - analytics"#;

    let parsed = parse(yaml).expect("Failed to parse YAML");

    let emitted = emit(&parsed);

    // The emitted YAML should contain all the comments with proper # prefix
    assert!(
        emitted.contains("# Leading comment for name"),
        "Leading comment for 'name' not preserved in emission"
    );
    assert!(
        emitted.contains("# Application name"),
        "Inline comment for 'name' not preserved in emission"
    );
    assert!(
        emitted.contains("# Server host"),
        "Leading comment for nested 'host' not preserved in emission"
    );
    assert!(
        emitted.contains("# Can be IP or hostname"),
        "Inline comment for nested 'host' not preserved in emission"
    );

    // Re-parse the emitted YAML
    let reparsed = parse(&emitted).expect("Failed to re-parse emitted YAML");

    // Check that comments survived the round-trip
    let YamlValue::Object(map) = &parsed.value else {
        panic!("Expected object in original, got {:?}", parsed.value);
    };
    let YamlValue::Object(reparsed_map) = &reparsed.value else {
        panic!("Expected object in reparsed, got {:?}", reparsed.value);
    };

    // Check name comments survived
    let name_node = map.get("name").expect("name not found in original");
    let reparsed_name = reparsed_map
        .get("name")
        .expect("name not found in reparsed");

    assert_eq!(
        name_node.leading_comment, reparsed_name.leading_comment,
        "Leading comment for 'name' not preserved after round-trip"
    );
    assert_eq!(
        name_node.inline_comment, reparsed_name.inline_comment,
        "Inline comment for 'name' not preserved after round-trip"
    );

    // Check nested server comments survived
    let YamlValue::Object(server_map) = &map.get("server").expect("server not found").value else {
        panic!("Expected object for server in original");
    };
    let YamlValue::Object(reparsed_server) = &reparsed_map.get("server").expect("server not found in reparsed").value else {
        panic!("Expected object for server in reparsed");
    };

    let host_node = server_map.get("host").expect("host not found");
    let reparsed_host = reparsed_server
        .get("host")
        .expect("host not found in reparsed");

    assert_eq!(
        host_node.leading_comment, reparsed_host.leading_comment,
        "Leading comment for nested 'host' not preserved after round-trip"
    );
    assert_eq!(
        host_node.inline_comment, reparsed_host.inline_comment,
        "Inline comment for nested 'host' not preserved after round-trip"
    );
}

#[test]
fn test_multiple_consecutive_comments_preserved() {
    // Test that ALL consecutive comment lines before a key are preserved
    let yaml = r#"# First comment line
# Second comment line
# Third comment line
name: John Doe

# Single comment for age
age: 30

# Block comment for server
# Multiple lines here
# Last line of block
server: production"#;

    let parsed = parse(yaml).expect("Failed to parse YAML");

    let YamlValue::Object(map) = &parsed.value else {
        panic!("Expected object, got {:?}", parsed.value);
    };

    let name_node = map.get("name").expect("name not found");
    // Should capture ALL consecutive comment lines
    assert_eq!(
        name_node.leading_comment.as_deref(),
        Some("First comment line\nSecond comment line\nThird comment line"),
        "Should capture all consecutive comments for 'name'"
    );

    let age_node = map.get("age").expect("age not found");
    // Should capture single comment
    assert_eq!(
        age_node.leading_comment.as_deref(),
        Some("Single comment for age"),
        "Should capture single comment for 'age'"
    );

    let server_node = map.get("server").expect("server not found");
    // Should capture all lines in the comment block
    assert_eq!(
        server_node.leading_comment.as_deref(),
        Some("Block comment for server\nMultiple lines here\nLast line of block"),
        "Should capture all consecutive comments for 'server'"
    );
}

#[test]
fn test_minimal_comment_after_array() {
    let yaml = r#"features:
  - item
# Comment for name
name: value"#;

    let parsed = parse(yaml).expect("Failed to parse");

    // Check if the comment was captured at all
    let YamlValue::Object(original_map) = &parsed.value else {
        panic!("Expected object");
    };

    println!("Parsed keys: {:?}", original_map.keys().collect::<Vec<_>>());
    for (key, value) in original_map {
        println!("{}: comment = {:?}", key, value.leading_comment);
    }

    let original_name = original_map.get("name").expect("name not found");

    // This should pass if the original parsing works
    assert_eq!(
        original_name.leading_comment,
        Some("Comment for name".to_string()),
        "Original parsing should capture the comment"
    );
}

#[test]
fn test_comment_after_array() {
    let yaml = r#"items:
  - apple
  - banana
# Comment for next field
next: value"#;

    println!("=== ORIGINAL ===");
    println!("{}", yaml);

    let parsed = parse(yaml).expect("Failed to parse");
    println!("=== PARSED ===");
    println!("{:#?}", parsed);

    let emitted = emit(&parsed);
    println!("=== EMITTED ===");
    println!("{}", emitted);

    let reparsed = parse(&emitted).expect("Failed to re-parse");
    println!("=== REPARSED ===");
    println!("{:#?}", reparsed);

    // Check that comment is preserved
    let YamlValue::Object(map) = &parsed.value else {
        panic!("Expected object");
    };
    let YamlValue::Object(reparsed_map) = &reparsed.value else {
        panic!("Expected object");
    };

    let original_next = map.get("next").expect("next not found");
    let reparsed_next = reparsed_map
        .get("next")
        .expect("next not found in reparsed");

    assert_eq!(
        original_next.leading_comment, reparsed_next.leading_comment,
        "Comment not preserved for field after array"
    );
}

#[test]
fn test_debug_dedent_issue() {
    let yaml = r#"features:
  - ui_v2
  - analytics
# Server config
server:
  # Host config
  host: localhost
  port: 8080"#;

    let parsed = parse(yaml).expect("Failed to parse YAML");
    let emitted = emit(&parsed);

    println!("Emitted: '{}'", emitted);

    // This should not fail
    let _reparsed = parse(&emitted).expect("Failed to re-parse emitted YAML");
}

#[test]
fn test_comment_at_end_of_file() {
    let yaml = r#"name: John Doe
age: 30
# This is a comment at the end of the file"#;

    let parsed = parse(yaml).expect("Failed to parse YAML with trailing comment");

    // The file should parse successfully even with a trailing comment
    let YamlValue::Object(map) = &parsed.value else {
        panic!("Expected object, got {:?}", parsed.value);
    };

    assert_eq!(map.get("name").and_then(|n| n.as_str()), Some("John Doe"));
    assert_eq!(map.get("age").and_then(|n| n.as_str()), Some("30"));
}

#[test]
fn test_inline_comment_at_end_of_file() {
    let yaml = r#"name: John Doe
age: 30  # Final value with comment"#;

    let parsed = parse(yaml).expect("Failed to parse YAML with inline comment on last line");

    let YamlValue::Object(map) = &parsed.value else {
        panic!("Expected object, got {:?}", parsed.value);
    };

    let age_node = map.get("age").expect("age not found");
    assert_eq!(age_node.as_str(), Some("30"));
    assert_eq!(
        age_node.inline_comment.as_deref(),
        Some("Final value with comment"),
        "Inline comment on last line should be captured"
    );
}

#[test]
fn test_multiple_trailing_comments() {
    let yaml = r#"name: John Doe
age: 30
# Comment 1 at end
# Comment 2 at end
# Comment 3 at end"#;

    // Should parse successfully even with multiple trailing comments
    let parsed = parse(yaml).expect("Failed to parse YAML with multiple trailing comments");

    let YamlValue::Object(map) = &parsed.value else {
        panic!("Expected object, got {:?}", parsed.value);
    };

    assert_eq!(map.len(), 2);
    assert!(map.contains_key("name"));
    assert!(map.contains_key("age"));
}

#[test]
fn test_empty_file_with_only_comments() {
    let yaml = r#"# This file only has comments
# No actual YAML content
# Just comments"#;

    // Should handle files with only comments gracefully
    let result = parse(yaml);

    // This might be an empty object or an error, depending on implementation
    // But it shouldn't panic
    if let Ok(parsed) = result {
        // If it succeeds, it should probably be an empty object or string
        match parsed.value {
            YamlValue::Object(map) => assert_eq!(map.len(), 0),
            YamlValue::String(s) => assert!(s.is_empty() || s.starts_with('#')),
            YamlValue::Array(arr) => assert_eq!(arr.len(), 0),
        }
    }
    // It's also acceptable to return an error for a file with no content
}
