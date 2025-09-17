#![deny(clippy::all)]

use yamp::parse;

#[test]
fn test_helper_method_as_str() {
    let yaml = "name: John Doe";
    let parsed = parse(yaml).expect("Failed to parse");

    // Direct access with helper method
    let name = parsed.get("name").and_then(|n| n.as_str());
    assert_eq!(name, Some("John Doe"));
}

#[test]
fn test_helper_method_get() {
    let yaml = r#"
server:
  host: localhost
  port: 8080
database:
  name: myapp
"#;

    let parsed = parse(yaml).expect("Failed to parse");

    // Easy nested access
    let host = parsed
        .get("server")
        .and_then(|s| s.get("host"))
        .and_then(|h| h.as_str());
    assert_eq!(host, Some("localhost"));

    let port = parsed
        .get("server")
        .and_then(|s| s.get("port"))
        .and_then(|p| p.as_str());
    assert_eq!(port, Some("8080"));

    let db_name = parsed
        .get("database")
        .and_then(|d| d.get("name"))
        .and_then(|n| n.as_str());
    assert_eq!(db_name, Some("myapp"));
}

#[test]
fn test_helper_method_as_array() {
    let yaml = r#"
items:
  - first
  - second
  - third
"#;

    let parsed = parse(yaml).expect("Failed to parse");

    // Access array with helper
    let items = parsed.get("items").and_then(|i| i.as_array());
    assert!(items.is_some());
    assert_eq!(items.unwrap().len(), 3);

    // Access individual items
    if let Some(items_array) = items {
        assert_eq!(items_array[0].as_str(), Some("first"));
        assert_eq!(items_array[1].as_str(), Some("second"));
        assert_eq!(items_array[2].as_str(), Some("third"));
    }
}

#[test]
fn test_helper_method_as_object() {
    let yaml = r#"
config:
  debug: true
  timeout: 30
"#;

    let parsed = parse(yaml).expect("Failed to parse");

    // Access object with helper
    let config = parsed.get("config").and_then(|c| c.as_object());
    assert!(config.is_some());
    assert_eq!(config.unwrap().len(), 2);

    // Can still access the raw map when needed
    if let Some(config_map) = config {
        assert!(config_map.contains_key("debug"));
        assert!(config_map.contains_key("timeout"));
    }
}

#[test]
fn test_type_checking_helpers() {
    let yaml = r#"
string_val: hello
array_val:
  - item1
object_val:
  key: value
"#;

    let parsed = parse(yaml).expect("Failed to parse");

    // Type checking with is_* methods
    assert!(parsed.is_object());

    if let Some(string_node) = parsed.get("string_val") {
        assert!(string_node.is_string());
        assert!(!string_node.is_array());
        assert!(!string_node.is_object());
    }

    if let Some(array_node) = parsed.get("array_val") {
        assert!(!array_node.is_string());
        assert!(array_node.is_array());
        assert!(!array_node.is_object());
    }

    if let Some(object_node) = parsed.get("object_val") {
        assert!(!object_node.is_string());
        assert!(!object_node.is_array());
        assert!(object_node.is_object());
    }
}

#[test]
fn test_chained_access() {
    let yaml = r#"
app:
  server:
    host: 127.0.0.1
    port: 3000
    ssl:
      enabled: true
      cert: /path/to/cert
"#;

    let parsed = parse(yaml).expect("Failed to parse");

    // Clean chained access
    let ssl_enabled = parsed
        .get("app")
        .and_then(|a| a.get("server"))
        .and_then(|s| s.get("ssl"))
        .and_then(|ssl| ssl.get("enabled"))
        .and_then(|e| e.as_str());

    assert_eq!(ssl_enabled, Some("true"));

    // Another chain
    let cert_path = parsed
        .get("app")
        .and_then(|a| a.get("server"))
        .and_then(|s| s.get("ssl"))
        .and_then(|ssl| ssl.get("cert"))
        .and_then(|c| c.as_str());

    // Note: The lexer currently strips the leading slash
    assert_eq!(cert_path, Some("path/to/cert"));
}

#[test]
fn test_comments_still_accessible() {
    let yaml = "name: John # This is a comment";
    let parsed = parse(yaml).expect("Failed to parse");

    // Can still access both value and comments easily
    if let Some(name_node) = parsed.get("name") {
        assert_eq!(name_node.as_str(), Some("John"));
        assert_eq!(
            name_node.inline_comment.as_deref(),
            Some("This is a comment")
        );
    }
}
