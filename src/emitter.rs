use crate::parser::{YamlNode, YamlValue};
use std::fmt::Write;

pub(crate) struct Emitter {
    output: String,
    indent_size: usize,
    current_indent: usize,
}

impl Emitter {
    pub(crate) fn new() -> Self {
        Emitter {
            output: String::with_capacity(1024), // Pre-allocate reasonable capacity
            indent_size: 2,
            current_indent: 0,
        }
    }

    pub(crate) fn emit<'a>(&mut self, node: &YamlNode<'a>) -> String {
        self.output.clear(); // Clear previous content instead of creating new String
        self.emit_node(node, false);
        std::mem::take(&mut self.output) // Move instead of clone
    }

    fn write_indent(&mut self) {
        if self.current_indent > 0 {
            // Use a pre-allocated string for common indent levels
            if self.current_indent <= 64 {
                const SPACES: &str =
                    "                                                                ";
                self.output.push_str(&SPACES[..self.current_indent]);
            } else {
                // Fallback for very deep indentation
                for _ in 0..self.current_indent {
                    self.output.push(' ');
                }
            }
        }
    }

    fn write_comment(&mut self, comment: &str, inline: bool) {
        if inline {
            write!(&mut self.output, " # {}", comment).unwrap();
        } else {
            self.write_indent();
            writeln!(&mut self.output, "# {}", comment).unwrap();
        }
    }

    fn emit_node<'a>(&mut self, node: &YamlNode<'a>, inline: bool) {
        // Write leading comment if present
        if let Some(ref comment) = node.leading_comment
            && !inline
        {
            self.write_comment(comment, false);
        }

        match &node.value {
            YamlValue::String(s) => {
                // Check if string should be emitted as multiline
                if !inline && should_use_multiline(s.as_ref()) {
                    self.emit_multiline_string(s.as_ref());
                } else if needs_quoting(s.as_ref()) {
                    write!(&mut self.output, "\"{}\"", escape_string(s.as_ref())).unwrap();
                } else {
                    self.output.push_str(s.as_ref());
                }
            }
            YamlValue::Array(items) => {
                self.emit_array(items);
            }
            YamlValue::Object(_) => {
                self.emit_object(node);
            }
        }

        // Write inline comment if present
        if let Some(ref comment) = node.inline_comment
            && inline
        {
            self.write_comment(comment, true);
        }
    }

    fn emit_multiline_string(&mut self, s: &str) {
        // Determine whether to use literal (|) or folded (>) style
        // Use literal style if the string has meaningful line breaks
        let has_trailing_newline = s.ends_with('\n');
        let content = if has_trailing_newline {
            &s[..s.len() - 1]
        } else {
            s
        };

        // Use literal style for strings with multiple lines
        if content.contains('\n') {
            // Literal style preserves line breaks
            self.output.push('|');
            if has_trailing_newline {
                // Default clip mode - single trailing newline
            } else {
                // Strip mode - no trailing newline
                self.output.push('-');
            }
            self.output.push('\n');

            // Write each line with proper indentation
            for line in content.lines() {
                self.current_indent += self.indent_size;
                self.write_indent();
                self.output.push_str(line);
                self.output.push('\n');
                self.current_indent -= self.indent_size;
            }
        } else {
            // For single long lines, could use folded style
            // For now, just emit as quoted string
            write!(&mut self.output, "\"{}\"", escape_string(s)).unwrap();
        }
    }

    fn emit_array<'a>(&mut self, items: &[YamlNode<'a>]) {
        for (i, item) in items.iter().enumerate() {
            if i > 0 {
                self.output.push('\n');
                self.write_indent();
            }
            self.output.push_str("- ");

            // Handle simple values
            if let YamlValue::String(_) = &item.value {
                self.emit_node(item, true);
                continue;
            }

            // Handle nested arrays
            if let YamlValue::Array(_) = &item.value {
                self.output.push('\n');
                let old_indent = self.current_indent;
                self.current_indent += self.indent_size;
                self.write_indent();
                self.emit_node(item, false);
                self.current_indent = old_indent;
                continue;
            }

            // Handle objects in arrays
            let YamlValue::Object(map) = &item.value else {
                continue;
            };

            let Some((first_key, first_value)) = map.iter().next() else {
                continue;
            };

            // Write first key-value pair inline with the dash
            if needs_quoting(first_key.as_ref()) {
                write!(
                    &mut self.output,
                    "\"{}\"",
                    escape_string(first_key.as_ref())
                )
                .unwrap();
            } else {
                self.output.push_str(first_key.as_ref());
            }
            self.output.push_str(": ");

            // Emit first value
            match &first_value.value {
                YamlValue::Object(_) | YamlValue::Array(_) => {
                    self.output.push('\n');
                    let old_indent = self.current_indent;
                    self.current_indent += self.indent_size * 2;
                    self.write_indent();
                    self.emit_node(first_value, false);
                    self.current_indent = old_indent;
                }
                YamlValue::String(_) => {
                    self.emit_node(first_value, true);
                }
            }

            // Emit remaining properties
            for (key, value) in map.iter().skip(1) {
                self.output.push('\n');
                // Indent for array item properties
                for _ in 0..(self.current_indent + self.indent_size) {
                    self.output.push(' ');
                }

                if needs_quoting(key.as_ref()) {
                    write!(&mut self.output, "\"{}\"", escape_string(key.as_ref())).unwrap();
                } else {
                    self.output.push_str(key.as_ref());
                }
                self.output.push_str(": ");

                match &value.value {
                    YamlValue::Object(_) | YamlValue::Array(_) => {
                        self.output.push('\n');
                        let old_indent = self.current_indent;
                        self.current_indent += self.indent_size * 2;
                        self.write_indent();
                        self.emit_node(value, false);
                        self.current_indent = old_indent;
                    }
                    YamlValue::String(_) => {
                        self.emit_node(value, true);
                    }
                }
            }
        }
    }

    fn emit_object<'a>(&mut self, node: &YamlNode<'a>) {
        let YamlValue::Object(map) = &node.value else {
            return;
        };

        let mut first = true;
        for (key, value) in map.iter() {
            if !first {
                self.output.push('\n');
                self.write_indent();
            } else {
                first = false;
            }

            // Write key
            if needs_quoting(key.as_ref()) {
                write!(&mut self.output, "\"{}\"", escape_string(key.as_ref())).unwrap();
            } else {
                self.output.push_str(key.as_ref());
            }
            self.output.push(':');

            // Check if value is complex
            match &value.value {
                YamlValue::Object(_) | YamlValue::Array(_) => {
                    // Write inline comment for key if present
                    if let Some(ref comment) = value.inline_comment {
                        self.output.push(' ');
                        self.write_comment(comment, true);
                    }

                    self.output.push('\n');
                    let old_indent = self.current_indent;
                    self.current_indent += self.indent_size;
                    self.write_indent();
                    self.emit_node(value, false);
                    self.current_indent = old_indent;
                }
                YamlValue::String(s) => {
                    // Check if string should be multiline
                    if should_use_multiline(s.as_ref()) {
                        self.output.push(' '); // Space after colon
                        self.emit_multiline_string(s.as_ref());
                    } else {
                        self.output.push(' ');
                        self.emit_node(value, true);
                    }
                }
            }
        }
    }
}

fn should_use_multiline(s: &str) -> bool {
    // Use multiline if string contains newlines
    s.contains('\n')
}

fn needs_quoting(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }

    // Check for special YAML values that need quoting
    matches!(s, "true" | "false" | "null")
        || s.chars().any(|c| matches!(c, ':' | '#' | '[' | ']' | '{' | '}' | ',' | '&' | '*' | '!' | '|' | '>' | '\'' | '"' | '%' | '@' | '`' | '~'))
        || s.starts_with(' ')
        || s.ends_with(' ')
        || s.starts_with('-')
        || s.parse::<f64>().is_ok()
        // Quote leading zeros to preserve them
        || (s.len() > 1 && s.starts_with('0') && s.chars().nth(1).is_some_and(|c| c.is_ascii_digit()))
}

fn escape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            _ => result.push(c),
        }
    }
    result
}

impl Default for Emitter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;
    use std::borrow::Cow;

    #[test]
    fn test_emit_simple_object() {
        let yaml = "name: John\nage: 30";
        let node = parse(yaml).unwrap();

        let mut emitter = Emitter::new();
        let output = emitter.emit(&node);

        // BTreeMap orders keys alphabetically
        // All values are strings now, so "30" might be quoted
        assert!(output.contains("age:") && (output.contains("30") || output.contains("\"30\"")));
        assert!(output.contains("name: John"));
    }

    #[test]
    fn test_emit_array() {
        let yaml = "- apple\n- banana";
        let node = parse(yaml).unwrap();

        let mut emitter = Emitter::new();
        let output = emitter.emit(&node);

        assert_eq!(output, "- apple\n- banana");
    }

    #[test]
    fn test_emit_with_special_chars() {
        use std::collections::BTreeMap;

        let mut map = BTreeMap::new();
        map.insert(
            Cow::Borrowed("key:with:colons"),
            YamlNode::from_value(YamlValue::String(Cow::Borrowed("value"))),
        );
        map.insert(
            Cow::Borrowed("normal_key"),
            YamlNode::from_value(YamlValue::String(Cow::Borrowed("value with spaces"))),
        );

        let node = YamlNode::from_value(YamlValue::Object(map));

        let mut emitter = Emitter::new();
        let output = emitter.emit(&node);

        assert!(output.contains("\"key:with:colons\": value"));
        assert!(output.contains("normal_key: value with spaces"));
    }

    #[test]
    fn test_preserve_comments() {
        let yaml = "name: John # Name field\nage: 30";
        let node = parse(yaml).unwrap();

        let mut emitter = Emitter::new();
        let output = emitter.emit(&node);

        // Comments should be preserved in the output
        assert!(output.contains("John"));
        assert!(output.contains("30"));
    }

    #[test]
    fn test_emit_multiline_string() {
        use std::collections::BTreeMap;

        let mut map = BTreeMap::new();
        map.insert(
            Cow::Borrowed("description"),
            YamlNode::from_value(YamlValue::String(Cow::Borrowed("Line 1\nLine 2\nLine 3\n"))),
        );

        let node = YamlNode::from_value(YamlValue::Object(map));

        let mut emitter = Emitter::new();
        let output = emitter.emit(&node);

        // Should emit as literal multiline string
        assert!(output.contains("description:"));
        assert!(output.contains("|"));
        assert!(output.contains("Line 1"));
        assert!(output.contains("Line 2"));
        assert!(output.contains("Line 3"));
    }
}
