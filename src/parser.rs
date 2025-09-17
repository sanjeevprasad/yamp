use crate::lexer::{Lexer, Token, TokenKind};

#[derive(Debug, Clone, Copy)]
enum ChompMode {
    Strip, // - remove trailing newlines
    Clip,  // default - single newline
    Keep,  // + keep all trailing newlines
}

/// A YAML object that preserves insertion order
#[derive(Debug, Clone, PartialEq)]
pub struct YamlObject {
    pairs: Vec<(String, YamlNode)>,
}

impl YamlObject {
    pub fn new() -> Self {
        YamlObject { pairs: Vec::new() }
    }

    pub fn insert(&mut self, key: String, value: YamlNode) {
        // Check if key exists and update, otherwise append
        for (k, v) in &mut self.pairs {
            if k == &key {
                *v = value;
                return;
            }
        }
        self.pairs.push((key, value));
    }

    pub fn get(&self, key: &str) -> Option<&YamlNode> {
        self.pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut YamlNode> {
        self.pairs.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &YamlNode)> {
        self.pairs.iter().map(|(k, v)| (k, v))
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.pairs.iter().map(|(k, _)| k)
    }

    pub fn values(&self) -> impl Iterator<Item = &YamlNode> {
        self.pairs.iter().map(|(_, v)| v)
    }

    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.pairs.iter().any(|(k, _)| k == key)
    }
}

impl Default for YamlObject {
    fn default() -> Self {
        Self::new()
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

impl IntoIterator for YamlObject {
    type Item = (String, YamlNode);
    type IntoIter = std::vec::IntoIter<(String, YamlNode)>;

    fn into_iter(self) -> Self::IntoIter {
        self.pairs.into_iter()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum YamlValue {
    String(String),
    Array(Vec<YamlNode>),
    Object(YamlObject),
}

#[derive(Debug, Clone, PartialEq)]
pub struct YamlNode {
    pub value: YamlValue,
    pub leading_comment: Option<String>,
    pub inline_comment: Option<String>,
}

impl YamlNode {
    pub(crate) fn new(value: YamlValue) -> Self {
        YamlNode {
            value,
            leading_comment: None,
            inline_comment: None,
        }
    }

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

    // Public constructor for external use
    pub fn from_value(value: YamlValue) -> Self {
        YamlNode {
            value,
            leading_comment: None,
            inline_comment: None,
        }
    }

    // Helper methods for ergonomic value access

    /// Returns the string value if this node contains a string
    pub fn as_str(&self) -> Option<&str> {
        match &self.value {
            YamlValue::String(s) => Some(s.as_ref()),
            YamlValue::Array(_) | YamlValue::Object(_) => None,
        }
    }

    /// Returns the object if this node contains an object
    pub fn as_object(&self) -> Option<&YamlObject> {
        match &self.value {
            YamlValue::Object(obj) => Some(obj),
            YamlValue::String(_) | YamlValue::Array(_) => None,
        }
    }

    /// Returns the array items if this node contains an array
    pub fn as_array(&self) -> Option<&[YamlNode]> {
        match &self.value {
            YamlValue::Array(items) => Some(items),
            YamlValue::String(_) | YamlValue::Object(_) => None,
        }
    }

    /// Gets a child node by key if this node is an object
    pub fn get(&self, key: &str) -> Option<&YamlNode> {
        match &self.value {
            YamlValue::Object(obj) => obj.get(key),
            YamlValue::String(_) | YamlValue::Array(_) => None,
        }
    }

    /// Returns true if this node is a string
    pub fn is_string(&self) -> bool {
        matches!(&self.value, YamlValue::String(_))
    }

    /// Returns true if this node is an object
    pub fn is_object(&self) -> bool {
        matches!(&self.value, YamlValue::Object(_))
    }

    /// Returns true if this node is an array
    pub fn is_array(&self) -> bool {
        matches!(&self.value, YamlValue::Array(_))
    }
}

pub(crate) struct Parser<'g> {
    tokens: Vec<Token<'g>>,
    current: usize,
}

impl<'g> Parser<'g> {
    pub(crate) fn new(source: &'g str) -> Self {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        Parser { tokens, current: 0 }
    }

    fn collect_consecutive_comments(&mut self) -> Option<String> {
        let mut leading_comments: Vec<String> = Vec::new();

        // First, look backward to find any comments that should be associated with this position
        // This handles cases where comments were already passed during array/object parsing
        let mut check_position = self.current;
        let mut found_non_comment_content = false;

        while check_position > 0 {
            check_position -= 1;
            let token = &self.tokens[check_position];

            match token.kind {
                TokenKind::Comment => {
                    // Check if this comment is on the same line as some other content (inline comment)
                    // Look at the token before this comment
                    let is_inline_comment = if check_position > 0 {
                        let prev_token = &self.tokens[check_position - 1];
                        prev_token.kind != TokenKind::NewLine && prev_token.kind != TokenKind::Indent && prev_token.kind != TokenKind::Dedent
                    } else {
                        false
                    };

                    // Skip inline comments - they shouldn't be leading comments for subsequent keys
                    if is_inline_comment {
                        continue;
                    }

                    // If we haven't found any significant non-comment content yet, this comment belongs to current position
                    if !found_non_comment_content {
                        leading_comments.insert(0, token.text.trim_start_matches('#').trim().to_string());
                    } else {
                        // We found a comment but there's content between it and current position
                        // Check if there are only whitespace/newlines between this comment and current position
                        let mut valid_comment = true;

                        for i in (check_position + 1)..self.current {
                            if i < self.tokens.len() {
                                match self.tokens[i].kind {
                                    TokenKind::Whitespace | TokenKind::Indent | TokenKind::Dedent | TokenKind::NewLine => continue,
                                    TokenKind::Identifier
                                    | TokenKind::Colon
                                    | TokenKind::String
                                    | TokenKind::Hyphen
                                    | TokenKind::Comment
                                    | TokenKind::Pipe
                                    | TokenKind::GreaterThan => {
                                        valid_comment = false; // Non-whitespace content between comment and current position
                                        break;
                                    }
                                }
                            }
                        }

                        if valid_comment {
                            leading_comments.insert(0, token.text.trim_start_matches('#').trim().to_string());
                        } else {
                            break; // Stop looking backward if we hit a non-associable comment
                        }
                    }
                }
                TokenKind::Whitespace | TokenKind::NewLine | TokenKind::Indent | TokenKind::Dedent => {
                    continue; // Keep looking backward through whitespace
                }
                TokenKind::Identifier
                | TokenKind::Colon
                | TokenKind::String
                | TokenKind::Hyphen
                | TokenKind::Pipe
                | TokenKind::GreaterThan => {
                    found_non_comment_content = true;
                    // Don't break yet - continue looking for more comments
                }
            }
        }

        // Now look forward from current position for any additional comments
        while let Some(token) = self.current_token() {
            match token.kind {
                TokenKind::Whitespace | TokenKind::NewLine | TokenKind::Indent | TokenKind::Dedent => {
                    self.advance();
                    continue;
                }
                TokenKind::Comment => {
                    leading_comments.push(token.text.trim_start_matches('#').trim().to_string());
                    self.advance();

                    // Skip whitespace after comment
                    while let Some(next_token) = self.current_token() {
                        match next_token.kind {
                            TokenKind::Whitespace | TokenKind::Indent | TokenKind::Dedent => {
                                self.advance();
                                continue;
                            }
                            TokenKind::NewLine => {
                                self.advance();
                                break; // Single newline after comment is okay
                            }
                            TokenKind::Identifier
                            | TokenKind::Colon
                            | TokenKind::String
                            | TokenKind::Hyphen
                            | TokenKind::Comment
                            | TokenKind::Pipe
                            | TokenKind::GreaterThan => break,
                        }
                    }
                }
                TokenKind::Identifier
                | TokenKind::Colon
                | TokenKind::String
                | TokenKind::Hyphen
                | TokenKind::Pipe
                | TokenKind::GreaterThan => break,
            }
        }

        if leading_comments.is_empty() {
            None
        } else {
            Some(leading_comments.join("\n"))
        }
    }

    pub(crate) fn parse(&mut self) -> Result<YamlNode, String> {
        // Don't skip comments at the root level - parse_value will handle them
        let result = self.parse_value(0)?;
        Ok(result)
    }

    fn current_token(&self) -> Option<&Token<'g>> {
        self.tokens.get(self.current)
    }

    fn advance(&mut self) -> Option<&Token<'g>> {
        if self.current < self.tokens.len() {
            let token = &self.tokens[self.current];
            self.current += 1;
            Some(token)
        } else {
            None
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(token) = self.current_token() {
            if token.kind != TokenKind::Whitespace {
                break;
            }
            self.advance();
        }
    }

    fn skip_whitespace_and_newlines(&mut self) {
        while let Some(token) = self.current_token() {
            match token.kind {
                TokenKind::Whitespace
                | TokenKind::NewLine
                | TokenKind::Indent
                | TokenKind::Dedent => {
                    self.advance();
                    continue;
                }
                TokenKind::Identifier
                | TokenKind::Colon
                | TokenKind::String
                | TokenKind::Hyphen
                | TokenKind::Comment
                | TokenKind::Pipe
                | TokenKind::GreaterThan => break,
            }
        }
    }

    fn collect_comment(&mut self) -> Option<String> {
        self.skip_whitespace();
        let token = self.current_token()?;
        if token.kind != TokenKind::Comment {
            return None;
        }
        let comment = token.text.trim_start_matches('#').trim();
        self.advance();
        Some(comment.to_string())
    }

    fn parse_value(&mut self, min_indent: usize) -> Result<YamlNode, String> {
        // Skip only whitespace initially, not comments
        self.skip_whitespace();

        // Skip newlines but stop at comments
        while let Some(token) = self.current_token() {
            if token.kind != TokenKind::NewLine
                && token.kind != TokenKind::Indent
                && token.kind != TokenKind::Dedent
            {
                break;
            }
            self.advance();
        }

        // Collect leading comment(s) - preserve only consecutive comments (no blank lines)
        let mut leading_comment = self.collect_consecutive_comments();

        let token = self
            .current_token()
            .ok_or_else(|| "Unexpected end of input".to_string())?;

        let node = match token.kind {
            TokenKind::Hyphen => {
                // Pass the leading comment to parse_array for the first item
                // Take ownership of the comment to avoid cloning
                let value = self.parse_array(min_indent, leading_comment.take())?;
                YamlNode::new(value)
            }
            TokenKind::Identifier => {
                let text = token.text;
                self.advance();

                self.skip_whitespace();
                if let Some(next) = self.current_token() {
                    if next.kind == TokenKind::Colon {
                        self.current -= 1; // Back up
                                           // Pass the leading comment to parse_object for the first key
                        let obj_node = self.parse_object(min_indent, leading_comment)?;
                        return Ok(obj_node);
                    }
                }

                // It's a scalar value - always treat as string
                YamlNode::new(YamlValue::String(text.to_string()))
            }
            TokenKind::String => {
                let text = token.text;
                let content = if text.starts_with('"') || text.starts_with('\'') {
                    &text[1..text.len() - 1]
                } else {
                    text
                };
                self.advance();
                YamlNode::new(YamlValue::String(content.to_string()))
            }
            TokenKind::Whitespace
            | TokenKind::NewLine
            | TokenKind::Colon
            | TokenKind::Indent
            | TokenKind::Dedent
            | TokenKind::Pipe
            | TokenKind::GreaterThan => {
                return Err(format!("Unexpected token: {:?}", token.kind));
            }
            TokenKind::Comment => {
                // This shouldn't happen as we handle comments above
                return Err("Unexpected comment token".to_string());
            }
        };

        let inline_comment = self.collect_comment();

        Ok(YamlNode::with_comments(
            node.value,
            leading_comment,
            inline_comment,
        ))
    }

    fn parse_inline_value(&mut self) -> Result<YamlNode, String> {
        // Collect tokens until we hit a newline or comment
        let start_token = self
            .current_token()
            .ok_or_else(|| "Expected value".to_string())?;

        // Check for special single-token values first
        match start_token.kind {
            TokenKind::String => {
                let text = start_token.text;
                let content = if text.starts_with('"') || text.starts_with('\'') {
                    &text[1..text.len() - 1]
                } else {
                    text
                };
                self.advance();
                let inline_comment = self.collect_comment();
                return Ok(YamlNode::with_comments(
                    YamlValue::String(content.to_string()),
                    None,
                    inline_comment,
                ));
            }
            TokenKind::Identifier
            | TokenKind::Colon
            | TokenKind::Whitespace
            | TokenKind::NewLine
            | TokenKind::Hyphen
            | TokenKind::Comment
            | TokenKind::Indent
            | TokenKind::Dedent
            | TokenKind::Pipe
            | TokenKind::GreaterThan => {}
        }

        // Otherwise collect all tokens until newline or comment
        let mut value_parts = Vec::with_capacity(4); // Most values are 1-4 tokens
        let mut single_token_text: Option<&'g str> = None;

        while let Some(token) = self.current_token() {
            match token.kind {
                TokenKind::NewLine | TokenKind::Comment => break,
                TokenKind::Whitespace => {
                    value_parts.push(" ");
                    self.advance();
                }
                TokenKind::Identifier
                | TokenKind::Colon
                | TokenKind::String
                | TokenKind::Hyphen
                | TokenKind::Indent
                | TokenKind::Dedent
                | TokenKind::Pipe
                | TokenKind::GreaterThan => {
                    if value_parts.is_empty() && single_token_text.is_none() {
                        single_token_text = Some(token.text);
                    }
                    value_parts.push(token.text);
                    self.advance();
                }
            }
        }

        // Trim trailing whitespace from value_parts
        while value_parts.last() == Some(&" ") {
            value_parts.pop();
        }

        // Everything is a string now
        let value = if let Some(text) = single_token_text.filter(|_| value_parts.len() == 1) {
            YamlValue::String(text.to_string())
        } else {
            // For multi-token values, join them
            let value_str = value_parts.join("");
            YamlValue::String(value_str)
        };

        let inline_comment = self.collect_comment();

        Ok(YamlNode::with_comments(value, None, inline_comment))
    }

    fn parse_array(
        &mut self,
        min_indent: usize,
        mut initial_leading_comment: Option<String>,
    ) -> Result<YamlValue, String> {
        let mut items = Vec::new();
        let mut first_item = true;

        while let Some(_token) = self.current_token() {
            // Handle any leading comments before the array item
            let leading_comment: Option<String>;

            // Use initial comment for first item if provided
            if first_item {
                leading_comment = initial_leading_comment.take();
                first_item = false;
            } else {
                first_item = false;
                leading_comment = self.collect_consecutive_comments();
            }

            // After handling comments, check if we have a hyphen
            let Some(token) = self.current_token() else {
                break;
            };
            if token.kind != TokenKind::Hyphen {
                break;
            }

            self.advance(); // consume hyphen
            self.skip_whitespace();

            let mut item = self.parse_value(min_indent)?;

            // Apply leading comment to the item if we collected one
            // The comment before the hyphen takes precedence
            if leading_comment.is_some() {
                item.leading_comment = leading_comment;
            }

            items.push(item);

            self.skip_whitespace();
            let Some(token) = self.current_token() else {
                break;
            };

            match token.kind {
                TokenKind::NewLine => {
                    self.advance();
                    self.skip_whitespace_and_newlines();
                }
                TokenKind::Hyphen => {
                    // Continue loop to process next array item
                }
                TokenKind::Identifier
                | TokenKind::Colon
                | TokenKind::String
                | TokenKind::Whitespace
                | TokenKind::Comment
                | TokenKind::Indent
                | TokenKind::Dedent
                | TokenKind::Pipe
                | TokenKind::GreaterThan => break,
            }
        }

        Ok(YamlValue::Array(items))
    }

    fn parse_multiline_string(
        &mut self,
        base_indent: usize,
        is_literal: bool,
    ) -> Result<YamlNode, String> {
        // Skip any remaining whitespace and comments on the same line
        self.skip_whitespace();

        // Handle optional chomping indicator (-, +, or none)
        let mut chomp_mode = ChompMode::Clip; // default
        if let Some(token) = self.current_token() {
            match token.text {
                "-" => {
                    chomp_mode = ChompMode::Strip;
                    self.advance();
                }
                "+" => {
                    chomp_mode = ChompMode::Keep;
                    self.advance();
                }
                _ => {}
            }
        }

        // Skip to next line
        while let Some(token) = self.current_token() {
            if token.kind == TokenKind::NewLine {
                self.advance();
                break;
            }
            // Skip any other tokens (comments, etc.)
            self.advance();
        }

        let mut lines: Vec<String> = Vec::new();
        let mut content_indent = None;

        // Collect all lines that are more indented than base_indent
        while let Some(token) = self.current_token() {
            // Check if we've dedented back to or past the base level
            if token.kind == TokenKind::Dedent {
                // Check the next non-whitespace token's column
                let mut peek_index = self.current + 1;
                while peek_index < self.tokens.len() {
                    let peek_token = &self.tokens[peek_index];
                    if peek_token.kind != TokenKind::Whitespace
                        && peek_token.kind != TokenKind::Indent
                        && peek_token.kind != TokenKind::Dedent
                    {
                        break;
                    }
                    peek_index += 1;
                }
                if peek_index < self.tokens.len() && self.tokens[peek_index].column <= base_indent {
                    break;
                }
            }

            // Skip whitespace but track indentation, handle newlines
            match token.kind {
                TokenKind::Whitespace | TokenKind::Indent => {
                    self.advance();
                    continue;
                }
                TokenKind::NewLine => {
                    lines.push(String::new());
                    self.advance();
                    continue;
                }
                TokenKind::Identifier
                | TokenKind::Colon
                | TokenKind::String
                | TokenKind::Hyphen
                | TokenKind::Comment
                | TokenKind::Dedent
                | TokenKind::Pipe
                | TokenKind::GreaterThan => {}
            }

            // Check indentation
            if token.column <= base_indent {
                break;
            }

            // Set content indent from first content line
            if content_indent.is_none() {
                content_indent = Some(token.column);
            }

            // Collect the line
            let _line_start = self.current;
            let mut line_text = String::new();

            while let Some(token) = self.current_token() {
                if token.kind == TokenKind::NewLine {
                    break;
                }

                // For literal mode, preserve everything
                // For folded mode, we'll process later
                line_text.push_str(token.text);
                self.advance();
            }

            lines.push(line_text);

            if let Some(token) = self.current_token() {
                if token.kind == TokenKind::NewLine {
                    self.advance();
                }
            }
        }

        // Process the lines based on mode
        let result = if is_literal {
            // Literal mode: preserve line breaks
            let mut result = lines
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join("\n");

            // Apply chomping
            match chomp_mode {
                ChompMode::Strip => {
                    // Remove all trailing newlines
                    while result.ends_with('\n') {
                        result.pop();
                    }
                }
                ChompMode::Clip => {
                    // Keep single trailing newline (default)
                    while result.ends_with("\n\n") {
                        result.pop();
                    }
                    if !result.ends_with('\n') && !result.is_empty() {
                        result.push('\n');
                    }
                }
                ChompMode::Keep => {
                    // Keep all trailing newlines
                    result.push('\n');
                }
            }

            result
        } else {
            // Folded mode: fold lines together
            let mut result = String::new();
            let mut prev_empty = false;

            for (i, line) in lines.iter().enumerate() {
                if line.is_empty() {
                    if !prev_empty && i > 0 {
                        result.push('\n');
                    }
                    prev_empty = true;
                } else {
                    if i > 0 && !prev_empty {
                        result.push(' ');
                    }
                    result.push_str(line.trim_start());
                    prev_empty = false;
                }
            }

            // Apply chomping
            match chomp_mode {
                ChompMode::Strip => {
                    while result.ends_with('\n') || result.ends_with(' ') {
                        result.pop();
                    }
                }
                ChompMode::Clip => {
                    while result.ends_with('\n') || result.ends_with(' ') {
                        result.pop();
                    }
                    // Add single trailing newline for Clip mode
                    if !result.is_empty() {
                        result.push('\n');
                    }
                }
                ChompMode::Keep => {
                    // Keep trailing whitespace
                    if !result.is_empty() && !result.ends_with('\n') {
                        result.push('\n');
                    }
                }
            }

            result
        };

        Ok(YamlNode::new(YamlValue::String(result)))
    }

    fn parse_object(
        &mut self,
        min_indent: usize,
        mut initial_leading_comment: Option<String>,
    ) -> Result<YamlNode, String> {
        let mut object = YamlObject::new();
        let mut first_key = true;

        while let Some(_token) = self.current_token() {
            // Handle any leading comments before the key - always collect consistently
            let mut leading_comment = self.collect_consecutive_comments();


            // For the first key, prefer the initial comment if provided and no comment was collected
            if first_key {
                if leading_comment.is_none() && initial_leading_comment.is_some() {
                    leading_comment = initial_leading_comment.take();
                }
                first_key = false;
            }

            // After handling comments, check if we have a key
            let Some(token) = self.current_token() else {
                break;
            };
            if token.kind != TokenKind::Identifier {
                break;
            }

            let token = self.current_token().unwrap(); // Safe because we just checked

            // Check if this key is at the right indentation level
            // If we're in a nested object, keys should be more indented than min_indent
            if min_indent > 0 && token.column <= min_indent {
                break;
            }

            let key_column = token.column;
            let key = token.text.to_string();
            self.advance();

            self.skip_whitespace();

            // Early return if no colon found
            let Some(token) = self.current_token() else {
                return Err("Expected colon after key".to_string());
            };
            if token.kind != TokenKind::Colon {
                return Err(format!("Expected colon after key, got {:?}", token.kind));
            }
            self.advance();

            self.skip_whitespace();

            // Skip whitespace after colon
            self.skip_whitespace();

            // Collect the value - could be multiple tokens on the same line
            let Some(token) = self.current_token() else {
                return Err("Expected value after colon".to_string());
            };

            let mut value = match token.kind {
                TokenKind::Pipe => {
                    // Literal multiline string indicator
                    self.advance(); // consume |
                    self.parse_multiline_string(key_column, true)?
                }
                TokenKind::GreaterThan => {
                    // Folded multiline string indicator
                    self.advance(); // consume >
                    self.parse_multiline_string(key_column, false)?
                }
                TokenKind::NewLine | TokenKind::Indent => {
                    // Value is on next line
                    self.skip_whitespace_and_newlines();
                    // Use key_column as the new min_indent for nested values
                    self.parse_value(key_column)?
                }
                TokenKind::Identifier
                | TokenKind::Colon
                | TokenKind::String
                | TokenKind::Whitespace
                | TokenKind::Hyphen
                | TokenKind::Comment
                | TokenKind::Dedent => {
                    // Value is on same line - collect until newline
                    self.parse_inline_value()?
                }
            };

            // Apply leading comment to the value node if we collected one
            // The comment before the key takes precedence over any comment in the value
            if leading_comment.is_some() {
                value.leading_comment = leading_comment;
            }

            object.insert(key, value);

            self.skip_whitespace();
            if let Some(token) = self.current_token() {
                if token.kind == TokenKind::NewLine {
                    self.advance();
                    self.skip_whitespace_and_newlines();
                }
            }

            // Check if we've dedented or reached end
            let Some(token) = self.current_token() else {
                break;
            };
            if token.kind == TokenKind::Dedent {
                self.advance();
                break;
            }
        }

        Ok(YamlNode::new(YamlValue::Object(object)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_object() {
        let yaml = "name: John\nage: 30";
        let mut parser = Parser::new(yaml);
        let result = parser.parse().unwrap();

        if let YamlValue::Object(map) = &result.value {
            assert_eq!(map.len(), 2);

            let name_node = map.get("name").unwrap();
            assert_eq!(name_node.value, YamlValue::String("John".to_string()));

            let age_node = map.get("age").unwrap();
            assert_eq!(age_node.value, YamlValue::String("30".to_string()));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_parse_array() {
        let yaml = "- apple\n- banana\n- cherry";
        let mut parser = Parser::new(yaml);
        let result = parser.parse().unwrap();

        if let YamlValue::Array(items) = &result.value {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0].value, YamlValue::String("apple".to_string()));
            assert_eq!(items[1].value, YamlValue::String("banana".to_string()));
            assert_eq!(items[2].value, YamlValue::String("cherry".to_string()));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_parse_with_comments() {
        let yaml = "name: John # inline comment\nage: 30";
        let mut parser = Parser::new(yaml);
        let result = parser.parse().unwrap();

        if let YamlValue::Object(map) = &result.value {
            let name_node = map.get("name").unwrap();
            assert_eq!(name_node.inline_comment, Some("inline comment".to_string()));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_parse_mixed_types() {
        let yaml = "enabled: true\ncount: 42\nratio: 2.5\nempty: null";
        let mut parser = Parser::new(yaml);
        let result = parser.parse().unwrap();

        if let YamlValue::Object(map) = &result.value {
            assert_eq!(
                map.get("enabled").unwrap().value,
                YamlValue::String("true".to_string())
            );
            assert_eq!(
                map.get("count").unwrap().value,
                YamlValue::String("42".to_string())
            );
            assert_eq!(
                map.get("ratio").unwrap().value,
                YamlValue::String("2.5".to_string())
            );
            assert_eq!(
                map.get("empty").unwrap().value,
                YamlValue::String("null".to_string())
            );
        } else {
            panic!("Expected object");
        }
    }
}
