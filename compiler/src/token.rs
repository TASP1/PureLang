//! Token definitions for PureLang

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Number(f64),
    String(String),
    Identifier(String),

    // Keywords
    Fn,
    Mut,
    If,
    Else,
    For,
    While,
    Break,
    Continue,
    In,
    Return,
    Struct,
    Enum,
    Match,
    Mod,
    Pub,
    Trait,
    Impl,
    Print,
    True,
    False,

    // Operators & symbols
    Plus,
    Minus,
    Star,
    Slash,
    Assign,       // =
    Equal,        // ==
    NotEqual,     // !=
    Less,         // <
    Greater,      // >
    LessEqual,    // <=
    GreaterEqual, // >=
    FatArrow,     // =>
    DotDot,       // ..
    Dot,          // .
    Comma,
    Colon,
    Question, // ?

    // Delimiters
    LParen,   // (
    RParen,   // )
    LBrace,   // {
    RBrace,   // }
    LBracket, // [
    RBracket, // ]

    // Special
    Newline,
    Eof,
}

impl Token {
    pub fn keyword_or_ident(s: &str) -> Token {
        match s {
            "fn" => Token::Fn,
            "mut" => Token::Mut,
            "if" => Token::If,
            "else" => Token::Else,
            "for" => Token::For,
            "while" => Token::While,
            "break" => Token::Break,
            "continue" => Token::Continue,
            "in" => Token::In,
            "return" => Token::Return,
            "struct" => Token::Struct,
            "enum" => Token::Enum,
            "match" => Token::Match,
            "mod" => Token::Mod,
            "pub" => Token::Pub,
            "trait" => Token::Trait,
            "impl" => Token::Impl,
            "print" => Token::Print,
            "true" => Token::True,
            "false" => Token::False,
            _ => Token::Identifier(s.to_string()),
        }
    }
}

/// Token with source location (1-based line)
#[derive(Debug, Clone, PartialEq)]
pub struct Spanned {
    pub token: Token,
    pub line: u32,
}

impl Spanned {
    pub fn new(token: Token, line: u32) -> Self {
        Spanned { token, line }
    }
}
