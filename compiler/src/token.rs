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
    In,
    Return,
    Struct,
    Enum,
    Match,
    Mod,
    Pub,
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
            "in" => Token::In,
            "return" => Token::Return,
            "struct" => Token::Struct,
            "enum" => Token::Enum,
            "match" => Token::Match,
            "mod" => Token::Mod,
            "pub" => Token::Pub,
            "print" => Token::Print,
            "true" => Token::True,
            "false" => Token::False,
            _ => Token::Identifier(s.to_string()),
        }
    }
}
