//! # YAMP - Yet Another Minimal Parser
//!
//! A lightweight, efficient YAML parser that treats all scalar values as strings,
//! avoiding YAML's numerous type-related pitfalls.
//!
//! ## Features
//!
//! - All scalar values are strings (no implicit type conversion)
//! - Supports basic YAML structures (objects, arrays, scalars)
//! - Preserves comments during parsing
//! - Supports multiline strings (literal `|` and folded `>`)
//! - Zero dependencies
//! - Predictable, secure behavior
//!
//! ## Example
//!
//! ```rust
//! use yamp::{parse, emit, YamlValue};
//! use std::borrow::Cow;
//!
//! let yaml = "name: John\nage: 30";
//! let parsed = parse(yaml).expect("Failed to parse");
//!
//! // Using the new helper methods for cleaner access
//! if let Some(name) = parsed.get("name").and_then(|n| n.as_str()) {
//!     assert_eq!(name, "John");
//! }
//!
//! // Or using the traditional approach
//! if let YamlValue::Object(map) = &parsed.value {
//!     let age = &map.get(&Cow::Borrowed("age")).unwrap().value;
//!     // Note: age is a string "30", not a number
//!     assert_eq!(age, &YamlValue::String(Cow::Borrowed("30")));
//! }
//!
//! let output = emit(&parsed);
//! ```

#![deny(clippy::all)]
mod emitter;
mod lexer;
mod parser;

pub use parser::{YamlNode, YamlValue};

use emitter::Emitter;
use parser::Parser;

/// Parse a YAML string into a `YamlNode`.
///
/// All scalar values are parsed as strings. No type inference is performed.
///
/// # Example
///
/// ```rust
/// use yamp::parse;
///
/// let yaml = "name: John\nage: 30";
/// let parsed = parse(yaml).expect("Failed to parse");
/// ```
pub fn parse(yaml: &str) -> Result<YamlNode<'_>, String> {
    let mut parser = Parser::new(yaml);
    parser.parse()
}

/// Emit a `YamlNode` back to a YAML string.
///
/// Preserves comments and automatically uses multiline string format
/// for values containing newlines.
///
/// # Example
///
/// ```rust
/// use yamp::{parse, emit};
///
/// let yaml = "name: John";
/// let parsed = parse(yaml).expect("Failed to parse");
/// let output = emit(&parsed);
/// assert!(output.contains("name: John"));
/// ```
pub fn emit(node: &YamlNode<'_>) -> String {
    let mut emitter = Emitter::new();
    emitter.emit(node)
}
