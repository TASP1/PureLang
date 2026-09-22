//! Recursive-descent parser for PureLang

use crate::ast::*;
use crate::token::Token;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error: {}", self.message)
    }
}

impl std::error::Error for ParseError {}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        // Filter out Newline tokens — they are insignificant for parsing
        let tokens: Vec<Token> = tokens
            .into_iter()
            .filter(|t| !matches!(t, Token::Newline))
            .collect();
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let tok = self.peek().clone();
        if !matches!(tok, Token::Eof) {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        let tok = self.advance();
        if tok == expected {
            Ok(())
        } else {
            Err(ParseError {
                message: format!("Expected {:?}, found {:?}", expected, tok),
            })
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        match self.advance() {
            Token::Identifier(name) => Ok(name),
            other => Err(ParseError {
                message: format!("Expected identifier, found {:?}", other),
            }),
        }
    }

    // ---------- Entry point ----------

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut items = Vec::new();
        while !matches!(self.peek(), Token::Eof) {
            items.push(self.parse_item()?);
        }
        Ok(Program { items })
    }

    fn parse_item(&mut self) -> Result<Item, ParseError> {
        match self.peek() {
            Token::Fn => self.parse_function(),
            Token::Struct => self.parse_struct(),
            other => Err(ParseError {
                message: format!("Expected top-level item (fn or struct), found {:?}", other),
            }),
        }
    }

    fn parse_function(&mut self) -> Result<Item, ParseError> {
        self.expect(Token::Fn)?;
        let name = self.expect_ident()?;
        self.expect(Token::LParen)?;

        let mut params = Vec::new();
        if !matches!(self.peek(), Token::RParen) {
            loop {
                params.push(self.expect_ident()?);
                if matches!(self.peek(), Token::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect(Token::RParen)?;
        let body = self.parse_block()?;
        Ok(Item::Function { name, params, body })
    }

    fn parse_struct(&mut self) -> Result<Item, ParseError> {
        self.expect(Token::Struct)?;
        let name = self.expect_ident()?;
        self.expect(Token::LBrace)?;

        let mut fields = Vec::new();
        while !matches!(self.peek(), Token::RBrace) {
            fields.push(self.expect_ident()?);
            // optional comma
            if matches!(self.peek(), Token::Comma) {
                self.advance();
            }
        }
        self.expect(Token::RBrace)?;
        Ok(Item::Struct { name, fields })
    }

    fn parse_block(&mut self) -> Result<Block, ParseError> {
        self.expect(Token::LBrace)?;
        let mut statements = Vec::new();
        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            statements.push(self.parse_stmt()?);
        }
        self.expect(Token::RBrace)?;
        Ok(Block { statements })
    }

    // ---------- Statements ----------

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.peek() {
            Token::Print => {
                self.advance();
                let expr = self.parse_expr()?;
                Ok(Stmt::Print(expr))
            }
            Token::If => self.parse_if(),
            Token::For => self.parse_for(),
            Token::Return => {
                self.advance();
                // optional expression
                if matches!(self.peek(), Token::RBrace | Token::Eof) {
                    Ok(Stmt::Return(None))
                } else {
                    let expr = self.parse_expr()?;
                    Ok(Stmt::Return(Some(expr)))
                }
            }
            Token::Mut => {
                self.advance();
                let name = self.expect_ident()?;
                self.expect(Token::Assign)?;
                let value = self.parse_expr()?;
                Ok(Stmt::Let {
                    mutable: true,
                    name,
                    value,
                })
            }
            Token::Identifier(_) => {
                // Could be: assignment, let (immutable), or expression statement
                // Look ahead: ident = expr  → Let/Assign
                // otherwise treat as expression
                let name = self.expect_ident()?;
                if matches!(self.peek(), Token::Assign) {
                    self.advance();
                    let value = self.parse_expr()?;
                    // Treat first occurrence as Let (immutable). Later semantic analysis
                    // can distinguish reassignment.
                    Ok(Stmt::Let {
                        mutable: false,
                        name,
                        value,
                    })
                } else {
                    // Put the identifier back into an expression and continue
                    // We already consumed the ident, so build from there.
                    let mut expr = Expr::Ident(name);
                    // handle possible call or field access that started with this ident
                    expr = self.parse_postfix(expr)?;
                    // then continue with binary operators
                    expr = self.parse_binary_rest(expr, 0)?;
                    Ok(Stmt::Expr(expr))
                }
            }
            _ => {
                let expr = self.parse_expr()?;
                Ok(Stmt::Expr(expr))
            }
        }
    }

    fn parse_if(&mut self) -> Result<Stmt, ParseError> {
        self.expect(Token::If)?;
        let condition = self.parse_expr()?;
        let then_block = self.parse_block()?;
        let else_block = if matches!(self.peek(), Token::Else) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };
        Ok(Stmt::If {
            condition,
            then_block,
            else_block,
        })
    }

    fn parse_for(&mut self) -> Result<Stmt, ParseError> {
        self.expect(Token::For)?;
        let var = self.expect_ident()?;
        self.expect(Token::In)?;
        let iterable = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::For {
            var,
            iterable,
            body,
        })
    }

    // ---------- Expressions (Pratt-style precedence) ----------

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_binary(0)
    }

    fn parse_binary(&mut self, min_prec: u8) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;
        left = self.parse_binary_rest(left, min_prec)?;
        Ok(left)
    }

    fn parse_binary_rest(&mut self, mut left: Expr, min_prec: u8) -> Result<Expr, ParseError> {
        loop {
            let (op, prec) = match self.peek() {
                Token::Plus => (BinaryOp::Add, 10),
                Token::Minus => (BinaryOp::Sub, 10),
                Token::Star => (BinaryOp::Mul, 20),
                Token::Slash => (BinaryOp::Div, 20),
                Token::Equal => (BinaryOp::Eq, 5),
                Token::NotEqual => (BinaryOp::NotEq, 5),
                Token::Less => (BinaryOp::Lt, 5),
                Token::Greater => (BinaryOp::Gt, 5),
                Token::LessEqual => (BinaryOp::LtEq, 5),
                Token::GreaterEqual => (BinaryOp::GtEq, 5),
                Token::DotDot => {
                    // Range has lower precedence
                    self.advance();
                    let right = self.parse_binary(0)?;
                    left = Expr::Range {
                        start: Box::new(left),
                        end: Box::new(right),
                    };
                    continue;
                }
                _ => break,
            };

            if prec < min_prec {
                break;
            }
            self.advance(); // consume operator
            let right = self.parse_binary(prec + 1)?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            Token::Minus => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                })
            }
            _ => self.parse_postfix_expr(),
        }
    }

    fn parse_postfix_expr(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;
        expr = self.parse_postfix(expr)?;
        Ok(expr)
    }

    fn parse_postfix(&mut self, mut expr: Expr) -> Result<Expr, ParseError> {
        loop {
            match self.peek() {
                Token::LParen => {
                    // Function call
                    self.advance();
                    let mut args = Vec::new();
                    if !matches!(self.peek(), Token::RParen) {
                        loop {
                            args.push(self.parse_expr()?);
                            if matches!(self.peek(), Token::Comma) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    self.expect(Token::RParen)?;
                    expr = Expr::Call {
                        callee: Box::new(expr),
                        args,
                    };
                }
                Token::Dot => {
                    self.advance();
                    let field = self.expect_ident()?;
                    expr = Expr::Field {
                        object: Box::new(expr),
                        field,
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.advance() {
            Token::Number(n) => Ok(Expr::Number(n)),
            Token::String(s) => Ok(Expr::String(s)),
            Token::True => Ok(Expr::Bool(true)),
            Token::False => Ok(Expr::Bool(false)),
            Token::Identifier(name) => Ok(Expr::Ident(name)),
            Token::LParen => {
                let expr = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            Token::LBracket => {
                let mut elements = Vec::new();
                if !matches!(self.peek(), Token::RBracket) {
                    loop {
                        elements.push(self.parse_expr()?);
                        if matches!(self.peek(), Token::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                self.expect(Token::RBracket)?;
                Ok(Expr::List(elements))
            }
            other => Err(ParseError {
                message: format!("Unexpected token in expression: {:?}", other),
            }),
        }
    }
}
