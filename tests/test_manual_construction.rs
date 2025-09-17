#![deny(clippy::all)]

use yamp::{emit, parse, YamlNode, YamlObject, YamlValue};

#[test]
fn test_simple_construction() {
    let mut root = YamlObject::new();

    root.insert(
        "name".to_string(),
        YamlNode::from_value(YamlValue::String("test".to_string())),
    );

    root.insert(
        "version".to_string(),
        YamlNode::from_value(YamlValue::String("1".to_string())),
    );

    let doc = YamlNode::from_value(YamlValue::Object(root));
    let yaml_string = emit(&doc);

    // Should be able to parse what we emitted
    let parsed = parse(&yaml_string).expect("Failed to parse");

    // With PartialEq, we can now directly compare values!
    assert_eq!(doc.value, parsed.value);
}

#[test]
fn test_complex_construction() {
    let mut root = YamlObject::new();

    // Add string with comment
    let mut name_node = YamlNode::from_value(YamlValue::String("MyApp".to_string()));
    name_node.inline_comment = Some("Application name".to_string());
    root.insert("app".to_string(), name_node);

    // Add nested object
    let mut config = YamlObject::new();
    config.insert(
        "debug".to_string(),
        YamlNode::from_value(YamlValue::String("true".to_string())),
    );
    config.insert(
        "timeout".to_string(),
        YamlNode::from_value(YamlValue::String("30".to_string())),
    );

    // Add array (move items into config to work around parser limitation)
    let items = vec![
        YamlNode::from_value(YamlValue::String("item1".to_string())),
        YamlNode::from_value(YamlValue::String("item2".to_string())),
    ];

    config.insert(
        "items".to_string(),
        YamlNode::from_value(YamlValue::Array(items)),
    );

    root.insert(
        "config".to_string(),
        YamlNode::from_value(YamlValue::Object(config)),
    );

    let doc = YamlNode::from_value(YamlValue::Object(root));
    let yaml_string = emit(&doc);

    // Should be able to parse what we emitted
    let parsed = parse(&yaml_string).expect("Failed to parse complex construction");
    assert_eq!(doc.value, parsed.value);
}

#[test]
fn test_array_of_objects_construction() {
    let mut root = YamlObject::new();

    // Create array of objects
    let mut user1 = YamlObject::new();
    user1.insert(
        "name".to_string(),
        YamlNode::from_value(YamlValue::String("Alice".to_string())),
    );
    user1.insert(
        "age".to_string(),
        YamlNode::from_value(YamlValue::String("30".to_string())),
    );

    let mut user2 = YamlObject::new();
    user2.insert(
        "name".to_string(),
        YamlNode::from_value(YamlValue::String("Bob".to_string())),
    );
    user2.insert(
        "age".to_string(),
        YamlNode::from_value(YamlValue::String("25".to_string())),
    );

    let users = vec![
        YamlNode::from_value(YamlValue::Object(user1)),
        YamlNode::from_value(YamlValue::Object(user2)),
    ];

    root.insert(
        "users".to_string(),
        YamlNode::from_value(YamlValue::Array(users)),
    );

    let doc = YamlNode::from_value(YamlValue::Object(root));
    let yaml_string = emit(&doc);

    // Should parse successfully
    let parsed = parse(&yaml_string).expect("Failed to parse array of objects");
    assert_eq!(doc.value, parsed.value);
}

#[test]
fn test_direct_equality_with_partialeq() {
    // Now that YamlNode implements PartialEq, we can directly compare nodes
    let node1 = YamlNode::from_value(YamlValue::String("hello".to_string()));
    let node2 = YamlNode::from_value(YamlValue::String("hello".to_string()));
    let node3 = YamlNode::from_value(YamlValue::String("world".to_string()));

    // Direct equality comparison works!
    assert_eq!(node1, node2);
    assert_ne!(node1, node3);

    // For complex structures
    let mut map1 = YamlObject::new();
    map1.insert(
        "key".to_string(),
        YamlNode::from_value(YamlValue::String("value".to_string())),
    );
    let complex1 = YamlNode::from_value(YamlValue::Object(map1));

    let mut map2 = YamlObject::new();
    map2.insert(
        "key".to_string(),
        YamlNode::from_value(YamlValue::String("value".to_string())),
    );
    let complex2 = YamlNode::from_value(YamlValue::Object(map2));

    // These are equal even though they were constructed separately
    assert_eq!(complex1, complex2);

    // YamlValue also implements PartialEq
    assert_eq!(complex1.value, complex2.value);
}
