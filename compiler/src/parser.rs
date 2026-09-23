//! Recursive-descent parser for PureLang

use crate::ast::*;
use crate::token::{Spanned, Token};

pub struct Parser {
    tokens: Vec<Spanned>,
    pos: usize,
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub line: u32,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.line > 0 {
            write!(f, "Parse error (line {}): {}", self.line, self.message)
        } else {
            write!(f, "Parse error: {}", self.message)
        }
    }
}

impl std::error::Error for ParseError {}

impl Parser {
    pub fn new(tokens: Vec<Spanned>) -> Self {
        // Filter out Newline tokens — they are insignificant for parsing
        let tokens: Vec<Spanned> = tokens
            .into_iter()
            .filter(|t| !matches!(t.token, Token::Newline))
            .collect();
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .map(|s| &s.token)
            .unwrap_or(&Token::Eof)
    }

    fn peek_line(&self) -> u32 {
        self.tokens.get(self.pos).map(|s| s.line).unwrap_or(0)
    }

    fn advance(&mut self) -> Token {
        let tok = self.peek().clone();
        if !matches!(tok, Token::Eof) {
            self.pos += 1;
        }
        tok
    }

    fn err(&self, message: impl Into<String>) -> ParseError {
        ParseError {
            message: message.into(),
            line: self.peek_line(),
        }
    }

    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        let tok = self.advance();
        if tok == expected {
            Ok(())
        } else {
            Err(self.err(format!("Expected {:?}, found {:?}", expected, tok)))
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        match self.advance() {
            Token::Identifier(name) => Ok(name),
            other => Err(self.err(format!("Expected identifier, found {:?}", other))),
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
        let is_pub = if matches!(self.peek(), Token::Pub) {
            self.advance();
            true
        } else {
            false
        };
        match self.peek() {
            Token::Fn => self.parse_function(is_pub),
            Token::Struct => self.parse_struct(is_pub),
            Token::Enum => self.parse_enum(is_pub),
            Token::Mod => self.parse_module(is_pub),
            Token::Trait => self.parse_trait(is_pub),
            Token::Impl => self.parse_impl(),
            other => Err(self.err(format!(
                "Expected top-level item (fn, struct, enum, mod, trait, impl), found {:?}",
                other
            ))),
        }
    }

    fn parse_function(&mut self, is_pub: bool) -> Result<Item, ParseError> {
        self.expect(Token::Fn)?;
        // Support both `fn name(...)` and method form `fn Type.name(...)`
        let first = self.expect_ident()?;
        let (receiver, name) = if matches!(self.peek(), Token::Dot) {
            self.advance(); // consume '.'
            let method = self.expect_ident()?;
            (Some(first), method)
        } else {
            (None, first)
        };
        // Optional generics: fn id[T](...) or fn id<T>(...)
        let mut type_params = Vec::new();
        if matches!(self.peek(), Token::LBracket) {
            self.advance();
            while !matches!(self.peek(), Token::RBracket | Token::Eof) {
                type_params.push(self.expect_ident()?);
                if matches!(self.peek(), Token::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect(Token::RBracket)?;
        } else if matches!(self.peek(), Token::Less) {
            self.advance();
            while !matches!(self.peek(), Token::Greater | Token::Eof) {
                type_params.push(self.expect_ident()?);
                if matches!(self.peek(), Token::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect(Token::Greater)?;
        }
        self.expect(Token::LParen)?;

        let mut params = Vec::new();
        if !matches!(self.peek(), Token::RParen) {
            loop {
                let pname = self.expect_ident()?;
                let ty_annotation = if matches!(self.peek(), Token::Colon) {
                    self.advance();
                    Some(self.expect_ident()?)
                } else {
                    None
                };
                params.push(Param {
                    name: pname,
                    ty_annotation,
                });
                if matches!(self.peek(), Token::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect(Token::RParen)?;
        let body = self.parse_block()?;
        Ok(Item::Function {
            receiver,
            name,
            type_params,
            params,
            body,
            is_pub,
        })
    }

    fn parse_struct(&mut self, is_pub: bool) -> Result<Item, ParseError> {
        self.expect(Token::Struct)?;
        let name = self.expect_ident()?;
        self.expect(Token::LBrace)?;

        let mut fields = Vec::new();
        while !matches!(self.peek(), Token::RBrace) {
            fields.push(self.expect_ident()?);
            if matches!(self.peek(), Token::Comma) {
                self.advance();
            }
        }
        self.expect(Token::RBrace)?;
        Ok(Item::Struct {
            name,
            fields,
            is_pub,
        })
    }

    fn parse_enum(&mut self, is_pub: bool) -> Result<Item, ParseError> {
        self.expect(Token::Enum)?;
        let name = self.expect_ident()?;
        self.expect(Token::LBrace)?;

        let mut variants = Vec::new();
        while !matches!(self.peek(), Token::RBrace) {
            let vname = self.expect_ident()?;
            let mut fields = Vec::new();
            // Optional payload: Variant(field) or Variant(a, b)
            if matches!(self.peek(), Token::LParen) {
                self.advance();
                if !matches!(self.peek(), Token::RParen) {
                    loop {
                        fields.push(self.expect_ident()?);
                        if matches!(self.peek(), Token::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                self.expect(Token::RParen)?;
            }
            variants.push(EnumVariant {
                name: vname,
                fields,
            });
            if matches!(self.peek(), Token::Comma) {
                self.advance();
            }
        }
        self.expect(Token::RBrace)?;
        Ok(Item::Enum {
            name,
            variants,
            is_pub,
        })
    }

    fn parse_module(&mut self, is_pub: bool) -> Result<Item, ParseError> {
        self.expect(Token::Mod)?;
        let name = self.expect_ident()?;
        self.expect(Token::LBrace)?;
        let mut items = Vec::new();
        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            items.push(self.parse_item()?);
        }
        self.expect(Token::RBrace)?;
        Ok(Item::Module {
            name,
            items,
            is_pub,
        })
    }

    fn parse_trait(&mut self, is_pub: bool) -> Result<Item, ParseError> {
        self.expect(Token::Trait)?;
        let name = self.expect_ident()?;
        self.expect(Token::LBrace)?;
        let mut methods = Vec::new();
        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            // optional pub inside trait
            if matches!(self.peek(), Token::Pub) {
                self.advance();
            }
            self.expect(Token::Fn)?;
            let mname = self.expect_ident()?;
            self.expect(Token::LParen)?;
            let mut params = Vec::new();
            if !matches!(self.peek(), Token::RParen) {
                loop {
                    let pname = self.expect_ident()?;
                    let ty_annotation = if matches!(self.peek(), Token::Colon) {
                        self.advance();
                        Some(self.expect_ident()?)
                    } else {
                        None
                    };
                    params.push(Param {
                        name: pname,
                        ty_annotation,
                    });
                    if matches!(self.peek(), Token::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
            self.expect(Token::RParen)?;
            // optional empty body or just signature
            if matches!(self.peek(), Token::LBrace) {
                let _ = self.parse_block()?;
            }
            methods.push(TraitMethod {
                name: mname,
                params,
            });
        }
        self.expect(Token::RBrace)?;
        Ok(Item::Trait {
            name,
            methods,
            is_pub,
        })
    }

    fn parse_impl(&mut self) -> Result<Item, ParseError> {
        self.expect(Token::Impl)?;
        // `impl Trait for Type { ... }` or `impl Type { ... }`
        let first = self.expect_ident()?;
        let (trait_name, type_name) = if matches!(self.peek(), Token::For) {
            self.advance();
            let ty = self.expect_ident()?;
            (Some(first), ty)
        } else {
            (None, first)
        };
        self.expect(Token::LBrace)?;
        let mut methods = Vec::new();
        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            if matches!(self.peek(), Token::Pub) {
                self.advance();
            }
            // Methods in impl are functions with receiver = type_name
            let mut item = self.parse_function(true)?;
            if let Item::Function {
                ref mut receiver, ..
            } = item
            {
                if receiver.is_none() {
                    // Treat first param as self of type_name when defining bare fn in impl
                    *receiver = Some(type_name.clone());
                }
            }
            methods.push(item);
        }
        self.expect(Token::RBrace)?;
        Ok(Item::Impl {
            trait_name,
            type_name,
            methods,
        })
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

    fn parse_stmt(&mut self) -> Result<StmtNode, ParseError> {
        let line = self.peek_line();
        let stmt = self.parse_stmt_inner()?;
        Ok(StmtNode { line, stmt })
    }

    fn parse_stmt_inner(&mut self) -> Result<Stmt, ParseError> {
        match self.peek() {
            Token::Print => {
                self.advance();
                let expr = self.parse_expr()?;
                Ok(Stmt::Print(expr))
            }
            Token::If => self.parse_if(),
            Token::For => self.parse_for(),
            Token::While => self.parse_while(),
            Token::Break => {
                self.advance();
                Ok(Stmt::Break)
            }
            Token::Continue => {
                self.advance();
                Ok(Stmt::Continue)
            }
            Token::Match => self.parse_match(),
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

    fn parse_while(&mut self) -> Result<Stmt, ParseError> {
        self.expect(Token::While)?;
        let condition = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::While { condition, body })
    }

    fn parse_match(&mut self) -> Result<Stmt, ParseError> {
        self.expect(Token::Match)?;
        let expr = self.parse_expr()?;
        self.expect(Token::LBrace)?;
        let mut arms = Vec::new();
        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            // Pattern: Enum.Variant or Enum.Variant(bind)
            let enum_name = self.expect_ident()?;
            self.expect(Token::Dot)?;
            let variant = self.expect_ident()?;
            let binding = if matches!(self.peek(), Token::LParen) {
                self.advance();
                let b = self.expect_ident()?;
                self.expect(Token::RParen)?;
                Some(b)
            } else {
                None
            };
            self.expect(Token::FatArrow)?;
            // Body can be a block `{ ... }` or a single statement expression treated as block
            let body = if matches!(self.peek(), Token::LBrace) {
                self.parse_block()?
            } else {
                // single expression / print / return as a one-statement block
                let stmt = self.parse_stmt()?;
                Block {
                    statements: vec![stmt],
                }
            };
            arms.push(MatchArm {
                pattern: Pattern::Variant {
                    enum_name,
                    variant,
                    binding,
                },
                body,
            });
            if matches!(self.peek(), Token::Comma) {
                self.advance();
            }
        }
        self.expect(Token::RBrace)?;
        Ok(Stmt::Match { expr, arms })
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
                Token::LBracket => {
                    self.advance();
                    let index = self.parse_expr()?;
                    self.expect(Token::RBracket)?;
                    expr = Expr::Index {
                        object: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                Token::Question => {
                    self.advance();
                    expr = Expr::Try(Box::new(expr));
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
            other => Err(self.err(format!("Unexpected token in expression: {:?}", other))),
        }
    }
}
