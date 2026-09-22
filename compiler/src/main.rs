//! PureLang Compiler (purec) - Phase 1: Lexer + Parser

mod ast;
mod lexer;
mod parser;
mod token;

use std::env;
use std::fs;
use std::process;

use ast::*;
use lexer::Lexer;
use parser::Parser;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("PureLang Compiler (purec) v0.2.0");
        eprintln!();
        eprintln!("Usage:");
        eprintln!("  purec <file.pure>          Parse a PureLang source file");
        eprintln!("  purec --tokens <file>      Show tokens only");
        eprintln!("  purec --version            Show version");
        eprintln!();
        eprintln!("Example:");
        eprintln!("  purec examples/hello.pure");
        process::exit(1);
    }

    if args[1] == "--version" || args[1] == "-V" {
        println!("purec 0.2.0 (PureLang compiler - lexer + parser)");
        return;
    }

    let tokens_only = args[1] == "--tokens";
    let filename = if tokens_only {
        if args.len() < 3 {
            eprintln!("Error: --tokens requires a file path");
            process::exit(1);
        }
        &args[2]
    } else {
        &args[1]
    };

    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", filename, e);
            process::exit(1);
        }
    };

    // Lex
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    if tokens_only {
        println!("=== PureLang Tokens ===");
        println!("File: {}", filename);
        println!("----------------------");
        for (i, token) in tokens.iter().enumerate() {
            if matches!(token, token::Token::Eof) {
                println!("{:3}: EOF", i);
            } else {
                println!("{:3}: {:?}", i, token);
            }
        }
        println!("----------------------");
        println!("Total tokens: {}", tokens.len());
        return;
    }

    // Parse
    println!("=== PureLang Parser ===");
    println!("File: {}", filename);
    println!("----------------------");

    let mut parser = Parser::new(tokens);
    match parser.parse_program() {
        Ok(program) => {
            print_program(&program, 0);
            println!("----------------------");
            println!("Parse successful ✓");
        }
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}

// ---------- Pretty printer ----------

fn indent(level: usize) -> String {
    "  ".repeat(level)
}

fn print_program(program: &Program, level: usize) {
    println!("{}Program", indent(level));
    for item in &program.items {
        print_item(item, level + 1);
    }
}

fn print_item(item: &Item, level: usize) {
    match item {
        Item::Function { name, params, body } => {
            let params_str = params.join(", ");
            println!("{}Fn {}({})", indent(level), name, params_str);
            print_block(body, level + 1);
        }
        Item::Struct { name, fields } => {
            println!("{}Struct {}", indent(level), name);
            for f in fields {
                println!("{}Field {}", indent(level + 1), f);
            }
        }
    }
}

fn print_block(block: &Block, level: usize) {
    println!("{}Block", indent(level));
    for stmt in &block.statements {
        print_stmt(stmt, level + 1);
    }
}

fn print_stmt(stmt: &Stmt, level: usize) {
    match stmt {
        Stmt::Let {
            mutable,
            name,
            value,
        } => {
            let mut_str = if *mutable { "mut " } else { "" };
            println!("{}Let {}{}", indent(level), mut_str, name);
            print_expr(value, level + 1);
        }
        Stmt::Assign { name, value } => {
            println!("{}Assign {}", indent(level), name);
            print_expr(value, level + 1);
        }
        Stmt::Print(expr) => {
            println!("{}Print", indent(level));
            print_expr(expr, level + 1);
        }
        Stmt::If {
            condition,
            then_block,
            else_block,
        } => {
            println!("{}If", indent(level));
            print_expr(condition, level + 1);
            println!("{}Then", indent(level + 1));
            print_block(then_block, level + 2);
            if let Some(else_b) = else_block {
                println!("{}Else", indent(level + 1));
                print_block(else_b, level + 2);
            }
        }
        Stmt::For {
            var,
            iterable,
            body,
        } => {
            println!("{}For {}", indent(level), var);
            print_expr(iterable, level + 1);
            print_block(body, level + 1);
        }
        Stmt::Return(None) => {
            println!("{}Return", indent(level));
        }
        Stmt::Return(Some(expr)) => {
            println!("{}Return", indent(level));
            print_expr(expr, level + 1);
        }
        Stmt::Expr(expr) => {
            println!("{}ExprStmt", indent(level));
            print_expr(expr, level + 1);
        }
    }
}

fn print_expr(expr: &Expr, level: usize) {
    match expr {
        Expr::Number(n) => println!("{}Number({})", indent(level), n),
        Expr::String(s) => println!("{}String(\"{}\")", indent(level), s),
        Expr::Bool(b) => println!("{}Bool({})", indent(level), b),
        Expr::Ident(name) => println!("{}Ident({})", indent(level), name),
        Expr::Binary { left, op, right } => {
            println!("{}Binary({})", indent(level), op);
            print_expr(left, level + 1);
            print_expr(right, level + 1);
        }
        Expr::Unary { op, expr } => {
            println!("{}Unary({:?})", indent(level), op);
            print_expr(expr, level + 1);
        }
        Expr::Call { callee, args } => {
            println!("{}Call", indent(level));
            print_expr(callee, level + 1);
            for arg in args {
                print_expr(arg, level + 1);
            }
        }
        Expr::Range { start, end } => {
            println!("{}Range", indent(level));
            print_expr(start, level + 1);
            print_expr(end, level + 1);
        }
        Expr::List(elements) => {
            println!("{}List", indent(level));
            for e in elements {
                print_expr(e, level + 1);
            }
        }
        Expr::Field { object, field } => {
            println!("{}Field({})", indent(level), field);
            print_expr(object, level + 1);
        }
    }
}
