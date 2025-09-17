use std::{iter::Peekable, str::CharIndices};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TokenKind {
    Identifier,
    Colon,
    String,
    Whitespace,
    NewLine,
    Hyphen, // - for array items
    Comment,
    Indent,
    Dedent,
    Pipe,       // | for literal multiline
    GreaterThan, // > for folded multiline
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Token<'g> {
    pub(crate) kind: TokenKind,
    pub(crate) text: &'g str,
    pub(crate) line: usize,
    pub(crate) column: usize,
}

impl<'g> Token<'g> {
    pub(crate) fn new(kind: TokenKind, text: &'g str, line: usize, column: usize) -> Self {
        Token {
            kind,
            text,
            line,
            column,
        }
    }
}

pub(crate) struct Lexer<'g> {
    source: &'g str,
    chars: Peekable<CharIndices<'g>>,
    current: usize,
    line: usize,
    column: usize,
    indent_stack: Vec<usize>,
}

impl<'g> Lexer<'g> {
    pub(crate) fn new(source: &'g str) -> Self {
        Lexer {
            source,
            chars: source.char_indices().peekable(),
            current: 0,
            line: 1,
            column: 1,
            indent_stack: vec![0],
        }
    }
    pub(crate) fn tokenize(&mut self) -> Vec<Token<'g>> {
        // Pre-allocate with estimated capacity based on source length
        let estimated_tokens = self.source.len() / 4; // Rough estimate: 1 token per 4 chars
        let mut tokens = Vec::with_capacity(estimated_tokens);
        let mut at_line_start = true;

        while let Some((start, c)) = self.chars.next() {
            let start_line = self.line;
            let start_column = self.column;

            match c {
                '\n' => {
                    tokens.push(Token::new(
                        TokenKind::NewLine,
                        "\n",
                        start_line,
                        start_column,
                    ));
                    self.line += 1;
                    self.column = 1;
                    at_line_start = true;
                    self.current = start + 1;
                }
                ' ' | '\t' if at_line_start => {
                    let (indent_level, end) = self.consume_indent(start);
                    if indent_level > 0 {
                        self.handle_indent_changes(
                            &mut tokens,
                            indent_level,
                            start_line,
                            start_column,
                        );
                    }
                    at_line_start = false;
                    self.current = end;
                    self.column += end - start;
                }
                ' ' | '\t' => {
                    tokens.push(Token::new(
                        TokenKind::Whitespace,
                        &self.source[start..start + 1],
                        start_line,
                        start_column,
                    ));
                    self.current += 1;
                    self.column += 1;
                }
                '#' => {
                    let end = self.consume_comment(start);
                    tokens.push(Token::new(
                        TokenKind::Comment,
                        &self.source[start..end],
                        start_line,
                        start_column,
                    ));
                    self.current = end;
                    self.column += end - start;
                    at_line_start = false;
                }
                '-' if self.peek_char() == Some(' ')
                    || self.peek_char() == Some('\t')
                    || self.peek_char() == Some('\n') =>
                {
                    tokens.push(Token::new(TokenKind::Hyphen, "-", start_line, start_column));
                    self.current += 1;
                    self.column += 1;
                    at_line_start = false;
                }
                '-' if self.peek_char() == Some('-') => {
                    self.chars.next();
                    self.chars.next();
                    tokens.push(Token::new(
                        TokenKind::Identifier,
                        "---",
                        start_line,
                        start_column,
                    ));
                    self.current = start + 3;
                    self.column += 3;
                    at_line_start = false;
                }
                ':' => {
                    tokens.push(Token::new(TokenKind::Colon, ":", start_line, start_column));
                    self.current += 1;
                    self.column += 1;
                    at_line_start = false;
                }
                '"' | '\'' => {
                    let end = self.consume_quoted_string(start, c);
                    tokens.push(Token::new(
                        TokenKind::String,
                        &self.source[start..end],
                        start_line,
                        start_column,
                    ));
                    self.current = end;
                    self.column += end - start;
                    at_line_start = false;
                }
                '0'..='9' => {
                    // Treat all unquoted values as identifiers
                    let end = self.consume_simple_value(start);
                    tokens.push(Token::new(
                        TokenKind::Identifier,
                        &self.source[start..end],
                        start_line,
                        start_column,
                    ));
                    self.current = end;
                    self.column += end - start;
                    at_line_start = false;
                }
                '-' if !matches!(self.peek_char(), Some(' ') | Some('\t') | Some('\n') | None) => {
                    // Minus followed by something - treat as identifier
                    let end = self.consume_simple_value(start);
                    tokens.push(Token::new(
                        TokenKind::Identifier,
                        &self.source[start..end],
                        start_line,
                        start_column,
                    ));
                    self.current = end;
                    self.column += end - start;
                    at_line_start = false;
                }
                '.' => {
                    // Dot followed by letters (like .inf, .nan) or numbers
                    let end = self.consume_simple_value(start);
                    tokens.push(Token::new(
                        TokenKind::Identifier,
                        &self.source[start..end],
                        start_line,
                        start_column,
                    ));
                    self.current = end;
                    self.column += end - start;
                    at_line_start = false;
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let end = self.consume_simple_value(start);
                    tokens.push(Token::new(
                        TokenKind::Identifier,
                        &self.source[start..end],
                        start_line,
                        start_column,
                    ));
                    self.current = end;
                    self.column += end - start;
                    at_line_start = false;
                }
                '~' => {
                    let end = self.consume_simple_value(start);
                    tokens.push(Token::new(
                        TokenKind::Identifier,
                        &self.source[start..end],
                        start_line,
                        start_column,
                    ));
                    self.current = end;
                    self.column += end - start;
                    at_line_start = false;
                }
                '|' => {
                    tokens.push(Token::new(
                        TokenKind::Pipe,
                        &self.source[start..start + 1],
                        start_line,
                        start_column,
                    ));
                    self.current = start + 1;
                    self.column += 1;
                    at_line_start = false;
                }
                '>' => {
                    tokens.push(Token::new(
                        TokenKind::GreaterThan,
                        &self.source[start..start + 1],
                        start_line,
                        start_column,
                    ));
                    self.current = start + 1;
                    self.column += 1;
                    at_line_start = false;
                }
                _ => {
                    self.current += 1;
                    self.column += 1;
                }
            }
        }

        // Handle remaining dedents at end of file
        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            tokens.push(Token::new(TokenKind::Dedent, "", self.line, self.column));
        }

        tokens
    }

    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().map(|(_, c)| *c)
    }

    fn consume_indent(&mut self, start: usize) -> (usize, usize) {
        let mut end = start;
        let mut indent_level = 0;

        while let Some(&(index, c)) = self.chars.peek() {
            match c {
                ' ' => {
                    indent_level += 1;
                    end = index + 1;
                    self.chars.next();
                }
                '\t' => {
                    indent_level += 4; // Count tab as 4 spaces
                    end = index + 1;
                    self.chars.next();
                }
                _ => break,
            }
        }
        (indent_level, end)
    }

    fn handle_indent_changes(
        &mut self,
        tokens: &mut Vec<Token<'g>>,
        new_indent: usize,
        line: usize,
        column: usize,
    ) {
        let current_indent = *self.indent_stack.last().unwrap();

        if new_indent > current_indent {
            self.indent_stack.push(new_indent);
            tokens.push(Token::new(TokenKind::Indent, "", line, column));
        } else if new_indent < current_indent {
            while self.indent_stack.len() > 1 && *self.indent_stack.last().unwrap() > new_indent {
                self.indent_stack.pop();
                tokens.push(Token::new(TokenKind::Dedent, "", line, column));
            }
        }
    }

    fn consume_comment(&mut self, start: usize) -> usize {
        let mut end = start;
        while let Some(&(index, c)) = self.chars.peek() {
            if c == '\n' {
                break;
            }
            self.chars.next();
            end = index + 1;
        }
        end
    }

    fn consume_quoted_string(&mut self, start: usize, quote: char) -> usize {
        let mut end = start + 1;
        let mut escaped = false;

        for (index, c) in self.chars.by_ref() {
            end = index + 1;
            if escaped {
                escaped = false;
                continue;
            }
            if c == '\\' {
                escaped = true;
            } else if c == quote {
                break;
            }
        }
        end
    }

    fn consume_simple_value(&mut self, start: usize) -> usize {
        let mut end = start + 1;

        // Consume any characters that could be part of an unquoted value
        while let Some(&(index, c)) = self.chars.peek() {
            // Stop at YAML structural characters
            if matches!(c, ':' | '#' | '\n' | '\r') {
                break;
            }

            // Handle whitespace - stop if followed by structural chars
            if matches!(c, ' ' | '\t') {
                let mut temp = self.chars.clone();
                temp.next(); // skip whitespace

                // Check what follows the whitespace
                match temp.peek() {
                    Some(&(_, ':' | '#' | '\n')) => break,
                    None => break, // End of input
                    _ => {}        // Continue, whitespace is part of value
                }
            }

            // Consume the character
            self.chars.next();
            end = index + 1;
        }

        // Trim trailing whitespace
        let value = &self.source[start..end];
        let trimmed = value.trim_end();
        start + trimmed.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions_sorted::assert_eq;

    #[test]
    fn test_basic_yaml() {
        let source = "key: value\nanother_key: 123";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Identifier);
        assert_eq!(tokens[0].text, "key");
        assert_eq!(tokens[1].kind, TokenKind::Colon);
        assert_eq!(tokens[2].kind, TokenKind::Whitespace);
        assert_eq!(tokens[3].kind, TokenKind::Identifier);
        assert_eq!(tokens[3].text, "value");
    }

    #[test]
    fn test_comments() {
        let source = "# This is a comment\nkey: value # inline comment";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Comment);
        assert_eq!(tokens[0].text, "# This is a comment");

        // Find inline comment
        let comment_token = tokens
            .iter()
            .find(|t| t.text == "# inline comment")
            .unwrap();
        assert_eq!(comment_token.kind, TokenKind::Comment);
    }

    #[test]
    fn test_arrays() {
        let source = "items:\n  - first\n  - second";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        // Check for hyphen tokens
        let hyphens: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind == TokenKind::Hyphen)
            .collect();
        assert_eq!(hyphens.len(), 2);
    }

    #[test]
    fn test_booleans_and_null() {
        let source = "bool1: true\nbool2: false\nempty: null";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        // All values are now identifiers
        let value_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| {
                t.kind == TokenKind::Identifier
                    && (t.text == "true" || t.text == "false" || t.text == "null")
            })
            .collect();
        assert_eq!(value_tokens.len(), 3);
    }

    #[test]
    fn test_numbers() {
        let source = "int: 42\nfloat: 3.14\nnegative: -17\nscientific: 1.2e-3";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        // All values are now identifiers
        let number_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| {
                t.kind == TokenKind::Identifier
                    && (t.text == "42" || t.text == "-17" || t.text == "3.14" || t.text == "1.2e-3")
            })
            .collect();
        assert_eq!(number_tokens.len(), 4);
    }

    #[test]
    fn test_strings() {
        let source = r#"single: 'hello world'
double: "hello world"
escaped: "quote \" here""#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        let string_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind == TokenKind::String)
            .collect();
        assert_eq!(string_tokens.len(), 3);
    }
}
