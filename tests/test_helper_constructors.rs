#![deny(clippy::all)]

use yamp::{emit, parse, YamlNode, YamlObject, YamlValue};

#[test]
fn test_from_string() {
    let node1: YamlNode = "hello world".into();
    let node2: YamlNode = String::from("hello world").into();

    assert_eq!(node1.as_str(), Some("hello world"));
    assert_eq!(node2.as_str(), Some("hello world"));
}

#[test]
fn test_from_array() {
    let items: Vec<YamlNode> = vec!["item1".into(), "item2".into()];
    let node: YamlNode = items.into();

    if let YamlValue::Array(arr) = &node.value {
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_str(), Some("item1"));
        assert_eq!(arr[1].as_str(), Some("item2"));
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_from_object() {
    let obj = YamlObject::new().with("name", "John").with("age", "30");

    let node: YamlNode = obj.into();

    assert_eq!(node.get("name").and_then(|n| n.as_str()), Some("John"));
    assert_eq!(node.get("age").and_then(|n| n.as_str()), Some("30"));
}

#[test]
fn test_from_numeric_types() {
    let int_node: YamlNode = 42i32.into();
    let float_node: YamlNode = 1.234f64.into();
    let bool_node: YamlNode = true.into();

    assert_eq!(int_node.as_str(), Some("42"));
    assert_eq!(float_node.as_str(), Some("1.234"));
    assert_eq!(bool_node.as_str(), Some("true"));
}

#[test]
fn test_from_option() {
    let some_node: YamlNode = Some("value").into();
    let none_node: YamlNode = None::<String>.into();

    assert_eq!(some_node.as_str(), Some("value"));
    assert_eq!(none_node.as_str(), Some("null"));
}

#[test]
fn test_comment_builders() {
    let node = YamlNode::from("test")
        .with_leading_comment("This is a leading comment")
        .with_inline_comment("This is inline");

    assert_eq!(
        node.leading_comment,
        Some("This is a leading comment".to_string())
    );
    assert_eq!(node.inline_comment, Some("This is inline".to_string()));
}

#[test]
fn test_object_builder_chaining() {
    let obj = YamlObject::new()
        .with("field1", "value1")
        .with("field2", "value2")
        .with_string("field3", "value3");

    assert_eq!(obj.get("field1").and_then(|n| n.as_str()), Some("value1"));
    assert_eq!(obj.get("field2").and_then(|n| n.as_str()), Some("value2"));
    assert_eq!(obj.get("field3").and_then(|n| n.as_str()), Some("value3"));
}

#[test]
fn test_nested_structure_with_builders() {
    let inner_obj = YamlObject::new()
        .with_string("host", "localhost")
        .with_string("port", "8080");

    let root_obj = YamlObject::new()
        .with_string("name", "MyApp")
        .with("server", inner_obj)
        .with(
            "tags",
            YamlNode::from(vec![YamlNode::from("web"), YamlNode::from("api")]),
        );

    let doc: YamlNode = root_obj.into();

    // Test the structure
    assert_eq!(doc.get("name").and_then(|n| n.as_str()), Some("MyApp"));

    let server = doc.get("server").expect("server not found");
    assert_eq!(
        server.get("host").and_then(|n| n.as_str()),
        Some("localhost")
    );
    assert_eq!(server.get("port").and_then(|n| n.as_str()), Some("8080"));

    let tags = doc
        .get("tags")
        .and_then(|n| n.as_array())
        .expect("tags not found");
    assert_eq!(tags.len(), 2);
    assert_eq!(tags[0].as_str(), Some("web"));
    assert_eq!(tags[1].as_str(), Some("api"));
}

#[test]
fn test_trailing_comment_captured() {
    let yaml = r#"name: John
age: 30
# This is a trailing comment at the bottom"#;

    let parsed = parse(yaml).expect("Failed to parse");

    // Check that trailing comment was captured in inline_comment
    assert_eq!(
        parsed.inline_comment,
        Some("This is a trailing comment at the bottom".to_string())
    );
}

#[test]
fn test_trailing_comment_emitted() {
    let node: YamlNode = YamlObject::new()
        .with_string("name", "John")
        .with_string("age", "30")
        .into();
    let node = node.with_inline_comment("This is a trailing comment");

    let emitted = emit(&node);

    assert!(emitted.contains("# This is a trailing comment"));
}
