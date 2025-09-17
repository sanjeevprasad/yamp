use std::collections::BTreeMap;

/// Represents a YAML value with associated comments
///
/// Comments are stored as follows:
/// - `leading_comment`: Comments that appear before this node
/// - `inline_comment`: Comments that appear on the same line as this node,
///   or for root-level nodes, trailing comments at the end of the document
#[derive(Debug, Clone, PartialEq)]
pub struct YamlNode {
    pub value: YamlValue,
    pub leading_comment: Option<String>,
    pub inline_comment: Option<String>,
}

/// Order-preserving YAML object structure
#[derive(Debug, Clone, PartialEq)]
pub struct YamlObject {
    pairs: Vec<(String, YamlNode)>,
}

/// Represents different YAML value types
#[derive(Debug, Clone, PartialEq)]
pub enum YamlValue {
    String(String),
    Array(Vec<YamlNode>),
    Object(YamlObject),
}

impl YamlNode {
    pub fn from_value(value: YamlValue) -> Self {
        YamlNode {
            value,
            leading_comment: None,
            inline_comment: None,
        }
    }

    pub fn with_leading_comment<S: Into<String>>(mut self, comment: S) -> Self {
        self.leading_comment = Some(comment.into());
        self
    }

    pub fn with_inline_comment<S: Into<String>>(mut self, comment: S) -> Self {
        self.inline_comment = Some(comment.into());
        self
    }


    pub fn as_str(&self) -> Option<&str> {
        match &self.value {
            YamlValue::String(s) => Some(s.as_ref()),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<YamlNode>> {
        match &self.value {
            YamlValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&YamlObject> {
        match &self.value {
            YamlValue::Object(obj) => Some(obj),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&YamlNode> {
        match &self.value {
            YamlValue::Object(obj) => obj.get(key),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut YamlNode> {
        match &mut self.value {
            YamlValue::Object(obj) => obj.get_mut(key),
            _ => None,
        }
    }

    pub fn is_string(&self) -> bool {
        matches!(&self.value, YamlValue::String(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(&self.value, YamlValue::Array(_))
    }

    pub fn is_object(&self) -> bool {
        matches!(&self.value, YamlValue::Object(_))
    }
}

// Internal methods for YamlNode
impl YamlNode {
    pub(crate) fn with_comments(
        value: YamlValue,
        leading: Option<String>,
        inline: Option<String>,
    ) -> Self {
        YamlNode {
            value,
            leading_comment: leading,
            inline_comment: inline,
        }
    }
}

impl YamlObject {
    pub fn new() -> Self {
        YamlObject { pairs: Vec::new() }
    }

    pub fn with<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<YamlNode>,
    {
        self.insert(key.into(), value.into());
        self
    }

    pub fn with_string<K, V>(self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.with(key, value.into())
    }

    pub fn insert(&mut self, key: String, value: YamlNode) -> Option<YamlNode> {
        // Check if key exists
        for (k, v) in &mut self.pairs {
            if k == &key {
                return Some(std::mem::replace(v, value));
            }
        }
        // Key doesn't exist, append
        self.pairs.push((key, value));
        None
    }

    pub fn get(&self, key: &str) -> Option<&YamlNode> {
        self.pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut YamlNode> {
        self.pairs
            .iter_mut()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.pairs.iter().any(|(k, _)| k == key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.pairs.iter().map(|(k, _)| k)
    }

    pub fn values(&self) -> impl Iterator<Item = &YamlNode> {
        self.pairs.iter().map(|(_, v)| v)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &YamlNode)> {
        self.pairs.iter().map(|(k, v)| (k, v))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&String, &mut YamlNode)> {
        self.pairs.iter_mut().map(|(k, v)| (&*k, v))
    }

    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }
}

impl Default for YamlObject {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for YamlObject {
    type Item = (String, YamlNode);
    type IntoIter = std::vec::IntoIter<(String, YamlNode)>;

    fn into_iter(self) -> Self::IntoIter {
        self.pairs.into_iter()
    }
}

impl<'a> IntoIterator for &'a YamlObject {
    type Item = (&'a String, &'a YamlNode);
    type IntoIter = std::iter::Map<
        std::slice::Iter<'a, (String, YamlNode)>,
        fn(&'a (String, YamlNode)) -> (&'a String, &'a YamlNode),
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.pairs.iter().map(|(k, v)| (k, v))
    }
}

// From trait implementations
impl From<String> for YamlNode {
    fn from(s: String) -> Self {
        YamlNode::from_value(YamlValue::String(s))
    }
}

impl From<&str> for YamlNode {
    fn from(s: &str) -> Self {
        YamlNode::from_value(YamlValue::String(s.to_string()))
    }
}

impl From<bool> for YamlNode {
    fn from(b: bool) -> Self {
        YamlNode::from_value(YamlValue::String(b.to_string()))
    }
}

impl From<i32> for YamlNode {
    fn from(n: i32) -> Self {
        YamlNode::from_value(YamlValue::String(n.to_string()))
    }
}

impl From<i64> for YamlNode {
    fn from(n: i64) -> Self {
        YamlNode::from_value(YamlValue::String(n.to_string()))
    }
}

impl From<f32> for YamlNode {
    fn from(n: f32) -> Self {
        YamlNode::from_value(YamlValue::String(n.to_string()))
    }
}

impl From<f64> for YamlNode {
    fn from(n: f64) -> Self {
        YamlNode::from_value(YamlValue::String(n.to_string()))
    }
}

impl<T> From<Vec<T>> for YamlNode
where
    T: Into<YamlNode>,
{
    fn from(v: Vec<T>) -> Self {
        let items: Vec<YamlNode> = v.into_iter().map(Into::into).collect();
        YamlNode::from_value(YamlValue::Array(items))
    }
}

impl<T> From<Option<T>> for YamlNode
where
    T: Into<YamlNode>,
{
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(val) => val.into(),
            None => YamlNode::from_value(YamlValue::String("null".to_string())),
        }
    }
}

impl From<YamlObject> for YamlNode {
    fn from(obj: YamlObject) -> Self {
        YamlNode::from_value(YamlValue::Object(obj))
    }
}

impl From<BTreeMap<String, YamlNode>> for YamlObject {
    fn from(map: BTreeMap<String, YamlNode>) -> Self {
        let mut obj = YamlObject::new();
        for (k, v) in map {
            obj.insert(k, v);
        }
        obj
    }
}
