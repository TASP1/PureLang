//! PureLang Compiler (purec) - Lexer + Parser + Type Checker + LLVM Codegen

mod ast;
mod checker;
mod codegen;
mod lexer;
mod parser;
mod token;
mod types;
mod wasm;

use std::env;
use std::fs;
use std::path::Path;
use std::process::{self, Command};

use ast::*;
use checker::TypeChecker;
use codegen::Codegen;
use lexer::Lexer;
use parser::Parser;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    if args[1] == "--version" || args[1] == "-V" {
        println!("purec 0.7.0 (PureLang — lexer + parser + typecheck + llvm + wasm)");
        return;
    }

    let mut mode = "check";
    let mut filename: Option<&str> = None;
    let mut output: Option<&str> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--tokens" => {
                mode = "tokens";
                i += 1;
                if i < args.len() {
                    filename = Some(&args[i]);
                }
            }
            "--ast" => {
                mode = "ast";
                i += 1;
                if i < args.len() {
                    filename = Some(&args[i]);
                }
            }
            "--emit-ir" => {
                mode = "ir";
                i += 1;
                if i < args.len() && !args[i].starts_with('-') {
                    filename = Some(&args[i]);
                }
            }
            "-o" | "--output" => {
                i += 1;
                if i < args.len() {
                    output = Some(&args[i]);
                }
            }
            "--compile" | "-c" => {
                mode = "compile";
            }
            "--emit-wasm" => {
                mode = "wasm";
                i += 1;
                if i < args.len() && !args[i].starts_with('-') {
                    filename = Some(&args[i]);
                }
            }
            s if s.starts_with('-') => {
                eprintln!("Unknown option: {}", s);
                process::exit(1);
            }
            s => {
                filename = Some(s);
                if mode == "check" {
                    // default stays check unless -c/--compile
                }
            }
        }
        i += 1;
    }

    // If -o given without explicit mode, compile
    if output.is_some() && mode == "check" {
        mode = "compile";
    }

    let filename = match filename {
        Some(f) => f,
        None => {
            eprintln!("Error: no input file");
            print_usage();
            process::exit(1);
        }
    };

    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", filename, e);
            process::exit(1);
        }
    };

    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    if mode == "tokens" {
        println!("=== PureLang Tokens ===");
        for (i, token) in tokens.iter().enumerate() {
            if matches!(token, token::Token::Eof) {
                println!("{:3}: EOF", i);
            } else {
                println!("{:3}: {:?}", i, token);
            }
        }
        return;
    }

    let mut parser = Parser::new(tokens);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    if mode == "ast" {
        println!("=== PureLang AST ===");
        print_program(&program, 0);
        return;
    }

    // Type check
    let mut checker = TypeChecker::new();
    if let Err(errors) = checker.check_program(&program) {
        for err in &errors {
            eprintln!("{}", err);
        }
        eprintln!("{} error(s) found", errors.len());
        process::exit(1);
    }

    if mode == "check" {
        println!("=== PureLang Type Checker ===");
        println!("File: {}", filename);
        println!("Type check passed ✓");
        return;
    }

    if mode == "wasm" {
        let mut wg = wasm::WasmCodegen::new();
        match wg.generate(&program) {
            Ok(wat) => {
                if let Some(out) = output {
                    fs::write(out, &wat).unwrap_or_else(|e| {
                        eprintln!("Failed to write {}: {}", out, e);
                        process::exit(1);
                    });
                    println!("Wrote WebAssembly to {}", out);
                } else {
                    let base = Path::new(filename)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("out");
                    let wat_path = format!("{}.wat", base);
                    fs::write(&wat_path, &wat).unwrap_or_else(|e| {
                        eprintln!("Failed to write {}: {}", wat_path, e);
                        process::exit(1);
                    });
                    println!("=== PureLang WASM ===");
                    println!("File: {}", filename);
                    println!("Type check passed ✓");
                    println!("WebAssembly → {}", wat_path);
                    println!("Run with: wasmtime {}", wat_path);
                }
            }
            Err(errors) => {
                for e in errors {
                    eprintln!("WASM error: {}", e);
                }
                process::exit(1);
            }
        }
        return;
    }

    // LLVM Codegen
    let mut cg = Codegen::new();
    let ir = match cg.generate(&program) {
        Ok(ir) => ir,
        Err(errors) => {
            for e in errors {
                eprintln!("Codegen error: {}", e);
            }
            process::exit(1);
        }
    };

    if mode == "ir" {
        if let Some(out) = output {
            fs::write(out, &ir).unwrap_or_else(|e| {
                eprintln!("Failed to write {}: {}", out, e);
                process::exit(1);
            });
            println!("Wrote LLVM IR to {}", out);
        } else {
            print!("{}", ir);
        }
        return;
    }

    // mode == compile
    let base = Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("out");
    let ir_path = format!("{}.ll", base);
    let bin_path = output.unwrap_or(base).to_string();

    fs::write(&ir_path, &ir).unwrap_or_else(|e| {
        eprintln!("Failed to write {}: {}", ir_path, e);
        process::exit(1);
    });

    // Compile IR with clang
    let status = Command::new("clang")
        .args(["-O2", "-o", &bin_path, &ir_path])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("=== PureLang Compiler ===");
            println!("File: {}", filename);
            println!("Type check passed ✓");
            println!("LLVM IR → {}", ir_path);
            println!("Native binary → {}", bin_path);
            println!("Compile successful ✓");
        }
        Ok(s) => {
            eprintln!("clang failed with status {}", s);
            process::exit(1);
        }
        Err(e) => {
            eprintln!("Failed to run clang: {}", e);
            eprintln!("Install clang/LLVM, or use --emit-ir to only generate IR.");
            process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("PureLang Compiler (purec) v0.7.0");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  purec <file.pure>              Type-check");
    eprintln!("  purec --compile <file.pure>    Compile to native binary");
    eprintln!("  purec -o <out> <file.pure>     Compile to named binary");
    eprintln!("  purec --emit-ir <file.pure>    Print LLVM IR");
    eprintln!("  purec --emit-wasm <file.pure>  Emit WebAssembly (.wat)");
    eprintln!("  purec --ast <file.pure>        Show AST");
    eprintln!("  purec --tokens <file.pure>     Show tokens");
    eprintln!("  purec --version");
}

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
            println!("{}Fn {}({})", indent(level), name, params.join(", "));
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
            let m = if *mutable { "mut " } else { "" };
            println!("{}Let {}{}", indent(level), m, name);
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
            print_block(then_block, level + 1);
            if let Some(e) = else_block {
                print_block(e, level + 1);
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
        Stmt::Return(None) => println!("{}Return", indent(level)),
        Stmt::Return(Some(e)) => {
            println!("{}Return", indent(level));
            print_expr(e, level + 1);
        }
        Stmt::Expr(e) => {
            println!("{}ExprStmt", indent(level));
            print_expr(e, level + 1);
        }
    }
}

fn print_expr(expr: &Expr, level: usize) {
    match expr {
        Expr::Number(n) => println!("{}Number({})", indent(level), n),
        Expr::String(s) => println!("{}String(\"{}\")", indent(level), s),
        Expr::Bool(b) => println!("{}Bool({})", indent(level), b),
        Expr::Ident(n) => println!("{}Ident({})", indent(level), n),
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
            for a in args {
                print_expr(a, level + 1);
            }
        }
        Expr::Range { start, end } => {
            println!("{}Range", indent(level));
            print_expr(start, level + 1);
            print_expr(end, level + 1);
        }
        Expr::List(els) => {
            println!("{}List", indent(level));
            for e in els {
                print_expr(e, level + 1);
            }
        }
        Expr::Field { object, field } => {
            println!("{}Field({})", indent(level), field);
            print_expr(object, level + 1);
        }
        Expr::Index { object, index } => {
            println!("{}Index", indent(level));
            print_expr(object, level + 1);
            print_expr(index, level + 1);
        }
    }
}
