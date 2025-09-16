use std::{borrow::Cow, iter::Peekable, str::CharIndices};

#[derive(Debug, PartialEq, Eq)]
enum TokenKind {
    Identifier,
    Colon,
    Integer,
    Float,
    String,
    Boolean,
    Whitespace,
    NewLine,
    ObjectStart,
    ObjectEnd,
    ArrayStart,
    ArrayEnd,
    ArrayItem,
    Comma,
    Comment,
    Indent,
}

#[derive(Debug, PartialEq, Eq)]

struct Token<'g> {
    kind: TokenKind,
    text: &'g str,
}

impl<'g> Token<'g> {
    pub fn new(kind: TokenKind, text: &'g str) -> Self {
        Token { kind, text }
    }
}

struct Lexer<'g> {
    source: &'g str,
    chars: Peekable<CharIndices<'g>>,
    current: usize,
}

impl<'g> Lexer<'g> {
    pub fn new(source: &'g str) -> Self {
        Lexer {
            source,
            chars: source.char_indices().peekable(),
            current: 0,
        }
    }
    pub fn tokenize(&mut self) -> Vec<Token<'g>> {
        let mut tokens = Vec::new();
        while let Some((start, c)) = self.chars.next() {
            match c {
                'a'..='z' | 'A'..='Z' | '_' => {
                    let end = self.consume_identifier(start);
                    tokens.push(Token::new(TokenKind::Identifier, &self.source[start..end]));
                    self.current = end;
                }
                ':' => {
                    tokens.push(Token::new(TokenKind::Colon, ":"));
                    self.current += 1;
                }
                '0'..='9' => {
                    let end = self.consume_number(start);
                    tokens.push(Token::new(TokenKind::Integer, &self.source[start..end]));
                    self.current = end;
                }
                '"' => {
                    let end = self.consume_string(start);
                    tokens.push(Token::new(TokenKind::String, &self.source[start..end]));
                    self.current = end;
                }
                ' ' | '\t' => {
                    tokens.push(Token {
                        kind: TokenKind::Whitespace,
                        text: &self.source[start..start + 1],
                    });
                    self.current += 1;
                }
                '\n' => {
                    tokens.push(Token {
                        kind: TokenKind::NewLine,
                        text: "\n",
                    });
                }
                _ => continue, // Ignore other characters for now
            }
        }
        tokens
    }

    pub fn consume_identifier(&mut self, start: usize) -> usize {
        let mut end = start + 1;
        while let Some(&(index, c)) = self.chars.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.chars.next(); // Consume the character
                end = index + 1;
            } else {
                break; // Stop at the first non-identifier character
            }
        }
        end
    }

    pub fn consume_number(&mut self, start: usize) -> usize {
        let mut end = start + 1;
        while let Some(&(index, c)) = self.chars.peek() {
            if c.is_ascii_digit() || c == '.' {
                self.chars.next(); // Consume the character
                end = index + 1;
            } else {
                break; // Stop at the first non-number character
            }
        }
        end
    }

    pub fn consume_string(&mut self, start: usize) -> usize {
        let mut end = start + 1;
        while let Some(&(index, c)) = self.chars.peek() {
            if c != '"' {
                self.chars.next(); // Consume the character
                end = index + 1;
            } else {
                self.chars.next(); // Consume the closing quote
                break; // Stop at the closing quote
            }
        }
        end
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions_sorted::assert_eq;

    #[test]
    fn test_tokenize() {
        let source = concat!("key: value\n", "another_key: 123");
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let expected = vec![
            Token::new(TokenKind::Identifier, "key"),
            Token::new(TokenKind::Colon, ":"),
            Token::new(TokenKind::Whitespace, " "),
            Token::new(TokenKind::Identifier, "value"),
            Token::new(TokenKind::NewLine, "\n"),
            Token::new(TokenKind::Identifier, "another_key"),
            Token::new(TokenKind::Colon, ":"),
            Token::new(TokenKind::Whitespace, " "),
            Token::new(TokenKind::Integer, "123"),
        ];
        assert_eq!(tokens, expected);
        assert_eq!(lexer.current, source.len());
    }
}
