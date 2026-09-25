//! Very simple lexer for PureLang (easy syntax)

use crate::token::Token;

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: u32,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            input: source.chars().collect(),
            position: 0,
            line: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek();
        self.position += 1;
        ch
    }

    fn skip_whitespace_except_newline(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self) -> Token {
        let start = self.position;
        // Integer part
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }
        // Optional fractional part — only a single '.' followed by digits.
        // Do NOT consume ".." (range operator).
        if self.peek() == Some('.') {
            let next = self.input.get(self.position + 1).copied();
            if next == Some('.') {
                // It's the start of a range operator — leave the dots alone.
            } else if next.map(|c| c.is_ascii_digit()).unwrap_or(false) {
                self.advance(); // consume '.'
                while let Some(ch) = self.peek() {
                    if ch.is_ascii_digit() {
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
        }
        let number_str: String = self.input[start..self.position].iter().collect();
        let value: f64 = number_str.parse().unwrap_or(0.0);
        Token::Number(value)
    }

    fn read_string(&mut self) -> Token {
        self.advance(); // skip opening "
        let mut result = String::new();
        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.advance(); // skip closing "
                break;
            }
            if ch == '\\' {
                self.advance();
                match self.peek() {
                    Some('n') => {
                        self.advance();
                        result.push('\n');
                    }
                    Some('t') => {
                        self.advance();
                        result.push('\t');
                    }
                    Some('r') => {
                        self.advance();
                        result.push('\r');
                    }
                    Some('\\') => {
                        self.advance();
                        result.push('\\');
                    }
                    Some('"') => {
                        self.advance();
                        result.push('"');
                    }
                    Some(other) => {
                        self.advance();
                        result.push(other);
                    }
                    None => break,
                }
            } else {
                result.push(ch);
                self.advance();
            }
        }
        Token::String(result)
    }

    fn read_identifier(&mut self) -> Token {
        let start = self.position;
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }
        let ident: String = self.input[start..self.position].iter().collect();
        Token::keyword_or_ident(&ident)
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_except_newline();

        match self.peek() {
            None => Token::Eof,
            Some('\n') => {
                self.advance();
                self.line += 1;
                Token::Newline
            }
            Some('"') => self.read_string(),
            Some(ch) if ch.is_ascii_digit() => self.read_number(),
            Some(ch) if ch.is_alphabetic() || ch == '_' => self.read_identifier(),
            Some('+') => {
                self.advance();
                Token::Plus
            }
            Some('-') => {
                self.advance();
                Token::Minus
            }
            Some('*') => {
                self.advance();
                Token::Star
            }
            Some('/') => {
                self.advance();
                // simple line comment support
                if self.peek() == Some('/') {
                    while let Some(ch) = self.peek() {
                        if ch == '\n' {
                            break;
                        }
                        self.advance();
                    }
                    return self.next_token();
                }
                Token::Slash
            }
            Some('=') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token::Equal
                } else if self.peek() == Some('>') {
                    self.advance();
                    Token::FatArrow
                } else {
                    Token::Assign
                }
            }
            Some('!') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token::NotEqual
                } else {
                    // for now treat lone ! as identifier start or error
                    Token::Identifier("!".to_string())
                }
            }
            Some('<') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token::LessEqual
                } else {
                    Token::Less
                }
            }
            Some('>') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token::GreaterEqual
                } else {
                    Token::Greater
                }
            }
            Some('.') => {
                self.advance();
                if self.peek() == Some('.') {
                    self.advance();
                    Token::DotDot
                } else {
                    Token::Dot
                }
            }
            Some(',') => {
                self.advance();
                Token::Comma
            }
            Some(':') => {
                self.advance();
                Token::Colon
            }
            Some('?') => {
                self.advance();
                Token::Question
            }
            Some('(') => {
                self.advance();
                Token::LParen
            }
            Some(')') => {
                self.advance();
                Token::RParen
            }
            Some('{') => {
                self.advance();
                Token::LBrace
            }
            Some('}') => {
                self.advance();
                Token::RBrace
            }
            Some('[') => {
                self.advance();
                Token::LBracket
            }
            Some(']') => {
                self.advance();
                Token::RBracket
            }
            Some(ch) => {
                self.advance();
                Token::Identifier(ch.to_string()) // fallback
            }
        }
    }

    pub fn tokenize(&mut self) -> Vec<crate::token::Spanned> {
        let mut tokens = Vec::new();
        loop {
            let line_before = self.line;
            let tok = self.next_token();
            let is_eof = matches!(tok, Token::Eof);
            // For newline, line already incremented; report the previous line
            let line = if matches!(tok, Token::Newline) {
                line_before
            } else {
                self.line
            };
            tokens.push(crate::token::Spanned::new(tok, line));
            if is_eof {
                break;
            }
        }
        tokens
    }
}
