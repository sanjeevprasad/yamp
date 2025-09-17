#![deny(clippy::all)]

use std::borrow::Cow;
use std::collections::BTreeMap;
use yamp::{YamlNode, YamlValue, emit, parse};

#[test]
fn test_simple_construction() {
    let mut root = BTreeMap::new();

    root.insert(
        Cow::Borrowed("name"),
        YamlNode::from_value(YamlValue::String(Cow::Borrowed("test"))),
    );

    root.insert(
        Cow::Borrowed("version"),
        YamlNode::from_value(YamlValue::String(Cow::Borrowed("1"))),
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
    let mut root = BTreeMap::new();

    // Add string with comment
    let mut name_node = YamlNode::from_value(YamlValue::String(Cow::Borrowed("MyApp")));
    name_node.inline_comment = Some(Cow::Borrowed("Application name"));
    root.insert(Cow::Borrowed("app"), name_node);

    // Add nested object
    let mut config = BTreeMap::new();
    config.insert(
        Cow::Borrowed("debug"),
        YamlNode::from_value(YamlValue::String(Cow::Borrowed("true"))),
    );
    config.insert(
        Cow::Borrowed("timeout"),
        YamlNode::from_value(YamlValue::String(Cow::Borrowed("30"))),
    );

    // Add array (move items into config to work around parser limitation)
    let items = vec![
        YamlNode::from_value(YamlValue::String(Cow::Borrowed("item1"))),
        YamlNode::from_value(YamlValue::String(Cow::Borrowed("item2"))),
    ];

    config.insert(
        Cow::Borrowed("items"),
        YamlNode::from_value(YamlValue::Array(items)),
    );

    root.insert(
        Cow::Borrowed("config"),
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
    let mut root = BTreeMap::new();

    // Create array of objects
    let mut user1 = BTreeMap::new();
    user1.insert(
        Cow::Borrowed("name"),
        YamlNode::from_value(YamlValue::String(Cow::Borrowed("Alice"))),
    );
    user1.insert(
        Cow::Borrowed("age"),
        YamlNode::from_value(YamlValue::String(Cow::Borrowed("30"))),
    );

    let mut user2 = BTreeMap::new();
    user2.insert(
        Cow::Borrowed("name"),
        YamlNode::from_value(YamlValue::String(Cow::Borrowed("Bob"))),
    );
    user2.insert(
        Cow::Borrowed("age"),
        YamlNode::from_value(YamlValue::String(Cow::Borrowed("25"))),
    );

    let users = vec![
        YamlNode::from_value(YamlValue::Object(user1)),
        YamlNode::from_value(YamlValue::Object(user2)),
    ];

    root.insert(
        Cow::Borrowed("users"),
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
    let node1 = YamlNode::from_value(YamlValue::String(Cow::Borrowed("hello")));
    let node2 = YamlNode::from_value(YamlValue::String(Cow::Borrowed("hello")));
    let node3 = YamlNode::from_value(YamlValue::String(Cow::Borrowed("world")));

    // Direct equality comparison works!
    assert_eq!(node1, node2);
    assert_ne!(node1, node3);

    // For complex structures
    let mut map1 = BTreeMap::new();
    map1.insert(
        Cow::Borrowed("key"),
        YamlNode::from_value(YamlValue::String(Cow::Borrowed("value"))),
    );
    let complex1 = YamlNode::from_value(YamlValue::Object(map1));

    let mut map2 = BTreeMap::new();
    map2.insert(
        Cow::Borrowed("key"),
        YamlNode::from_value(YamlValue::String(Cow::Borrowed("value"))),
    );
    let complex2 = YamlNode::from_value(YamlValue::Object(map2));

    // These are equal even though they were constructed separately
    assert_eq!(complex1, complex2);

    // YamlValue also implements PartialEq
    assert_eq!(complex1.value, complex2.value);
}
