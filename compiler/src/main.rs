//! PureLang Compiler (purec) - Lexer + Parser + Type Checker + LLVM Codegen

mod ast;
mod checker;
mod codegen;
mod fmt;
mod lexer;
mod lsp;
mod parser;
mod pkg;
mod platforms;
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
        println!("purec 0.26.0 (PureLang — multi-platform, LSP, package manager)");
        return;
    }

    if args[1] == "pkg" {
        pkg::run(&args[2..]);
        return;
    }
    if args[1] == "--lsp" || args[1] == "lsp" {
        lsp::run();
        return;
    }
    if args[1] == "--list-platforms" {
        println!("{}", platforms::list_platforms());
        return;
    }

    let mut mode = "check";
    let mut filename: Option<&str> = None;
    let mut output: Option<&str> = None;
    let mut target: Option<String> = None;
    let mut opt_level = "2".to_string();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--tokens" => {
                mode = "tokens";
            }
            "--ast" => {
                mode = "ast";
            }
            "--emit-ir" => {
                mode = "ir";
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
            "--target" => {
                i += 1;
                if i < args.len() {
                    target = Some(args[i].clone());
                }
            }
            "--platform" => {
                i += 1;
                if i < args.len() {
                    if let Some(spec) = platforms::resolve_platform(&args[i]) {
                        target = Some(spec.triple.to_string());
                        eprintln!("platform: {} ({}) — {}", spec.name, spec.triple, spec.notes);
                    } else {
                        eprintln!("Unknown platform: {}", args[i]);
                        eprintln!("Available: {}", platforms::list_platforms());
                        process::exit(1);
                    }
                }
            }
            "--opt" | "-O" => {
                i += 1;
                if i < args.len() {
                    opt_level = args[i].clone();
                }
            }
            "--emit-wasm" => {
                mode = "wasm";
                // Do not advance i here — the loop advances once per arg.
                // Filename is picked up by the non-option arm.
            }
            "--fmt" | "--format" => {
                mode = "fmt";
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
        for (i, st) in tokens.iter().enumerate() {
            if matches!(st.token, token::Token::Eof) {
                println!("{:3}: EOF (line {})", i, st.line);
            } else {
                println!("{:3}: {:?} (line {})", i, st.token, st.line);
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

    if mode == "fmt" {
        let formatted = fmt::format_program(&program);
        if let Some(out) = output {
            fs::write(out, &formatted).unwrap_or_else(|e| {
                eprintln!("Failed to write {}: {}", out, e);
                process::exit(1);
            });
            println!("Formatted → {}", out);
        } else {
            print!("{}", formatted);
        }
        return;
    }

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
    let triple = target.clone().unwrap_or_else(codegen::host_triple);
    let mut cg = Codegen::with_target(&triple);
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

    // Compile IR with clang (platform-aware)
    let triple = target.unwrap_or_else(codegen::host_triple);
    let opt_flag = format!("-O{}", opt_level);
    let mut cmd = Command::new("clang");
    cmd.arg(&opt_flag);
    // Math library: libm on Unix; on Windows MSVC math is in the CRT
    if !triple.contains("windows") {
        cmd.arg("-lm");
        cmd.arg("-lpthread");
    }
    // Explicit target when cross-compiling or for consistency
    cmd.arg(format!("--target={}", triple));
    if triple.contains("android") {
        cmd.args(["-shared", "-fPIC"]);
    }
    if triple.contains("ios") {
        // Object-friendly; full link needs xcrun on macOS
        cmd.arg("-c");
    }
    if let Ok(sysroot) = std::env::var("PUREC_SYSROOT") {
        cmd.arg(format!("--sysroot={}", sysroot));
    } else if let Ok(ndk) = std::env::var("ANDROID_NDK") {
        if triple.contains("android") {
            // Common NDK llvm sysroot layout (linux host)
            let candidates = [
                format!("{}/toolchains/llvm/prebuilt/linux-x86_64/sysroot", ndk),
                format!("{}/toolchains/llvm/prebuilt/darwin-x86_64/sysroot", ndk),
                format!("{}/toolchains/llvm/prebuilt/windows-x86_64/sysroot", ndk),
            ];
            for c in &candidates {
                if std::path::Path::new(c).exists() {
                    cmd.arg(format!("--sysroot={}", c));
                    break;
                }
            }
        }
    }
    cmd.arg("-o").arg(&bin_path).arg(&ir_path);
    // Link PureLang runtime (maps + UI)
    let rt_candidates = [
        Path::new("runtime/purelang_rt.c").to_path_buf(),
        Path::new("../runtime/purelang_rt.c").to_path_buf(),
        Path::new("../../runtime/purelang_rt.c").to_path_buf(),
        // When running from compiler/ directory after cargo build
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../runtime/purelang_rt.c"),
    ];
    let mut linked_rt = false;
    for c in &rt_candidates {
        if c.exists() {
            cmd.arg(c);
            linked_rt = true;
            break;
        }
    }
    if !linked_rt {
        eprintln!("warning: purelang_rt.c not found — map/ui builtins need runtime/purelang_rt.c");
    }

    let status = cmd.status();
    match status {
        Ok(s) if s.success() => {
            println!("=== PureLang Compiler ===");
            println!("File: {}", filename);
            println!("Target: {}", triple);
            println!("Opt: -O{}", opt_level);
            println!("Type check passed ✓");
            println!("LLVM IR → {}", ir_path);
            println!("Native binary → {}", bin_path);
            println!("Compile successful ✓");
        }
        Ok(s) => {
            eprintln!("clang failed with status {}", s);
            eprintln!("Target was: {}", triple);
            process::exit(1);
        }
        Err(e) => {
            eprintln!("Failed to run clang: {}", e);
            eprintln!("Install clang/LLVM for your platform:");
            eprintln!("  Linux:   sudo apt install clang");
            eprintln!("  macOS:   xcode-select --install  (or brew install llvm)");
            eprintln!(
                "  Windows: install LLVM from https://llvm.org/ or use winget install LLVM.LLVM"
            );
            eprintln!("Or use --emit-ir / --emit-wasm without a native link step.");
            process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("PureLang Compiler (purec) v0.19.0");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  purec <file.pure>                 Type-check");
    eprintln!("  purec --compile <file.pure>       Compile to native binary");
    eprintln!("  purec -o <out> <file.pure>        Compile to named binary");
    eprintln!("  purec --target <triple> ...       Target triple (default: host)");
    eprintln!("  purec --opt <0|1|2|3|s> ...       Optimization level (default: 2)");
    eprintln!("  purec --emit-ir <file.pure>       Print LLVM IR");
    eprintln!("  purec --emit-wasm <file.pure>     Emit WebAssembly (.wat / WASI)");
    eprintln!("  purec --fmt <file.pure>           Format source (pretty-print)");
    eprintln!("  purec --platform <name> ...       android|ios|linux|macos|windows|console|...");
    eprintln!("  purec --list-platforms            List platform presets");
    eprintln!("  purec --lsp                       Run language server (stdio)");
    eprintln!("  purec pkg <init|add|list|build>   Package manager");
    eprintln!("  purec --ast <file.pure>           Show AST");
    eprintln!("  purec --tokens <file.pure>        Show tokens");
    eprintln!("  purec --version");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  purec --compile -o hello hello.pure");
    eprintln!("  purec --target x86_64-pc-windows-msvc -o app.exe app.pure");
    eprintln!("  purec --emit-wasm -o app.wat app.pure");
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
        Item::Function {
            receiver,
            name,
            type_params,
            params,
            body,
            is_pub,
        } => {
            let vis = if *is_pub { "pub " } else { "" };
            let gens = if type_params.is_empty() {
                String::new()
            } else {
                format!("[{}]", type_params.join(", "))
            };
            let ps: Vec<String> = params
                .iter()
                .map(|p| match &p.ty_annotation {
                    Some(ty) => format!("{}: {}", p.name, ty),
                    None => p.name.clone(),
                })
                .collect();
            if let Some(recv) = receiver {
                println!(
                    "{}{}Fn {}.{}{}({})",
                    indent(level),
                    vis,
                    recv,
                    name,
                    gens,
                    ps.join(", ")
                );
            } else {
                println!(
                    "{}{}Fn {}{}({})",
                    indent(level),
                    vis,
                    name,
                    gens,
                    ps.join(", ")
                );
            }
            print_block(body, level + 1);
        }
        Item::Struct {
            name,
            fields,
            is_pub,
        } => {
            let vis = if *is_pub { "pub " } else { "" };
            println!("{}{}Struct {}", indent(level), vis, name);
            for f in fields {
                println!("{}Field {}", indent(level + 1), f);
            }
        }
        Item::Enum {
            name,
            variants,
            is_pub,
        } => {
            let vis = if *is_pub { "pub " } else { "" };
            println!("{}{}Enum {}", indent(level), vis, name);
            for v in variants {
                if v.fields.is_empty() {
                    println!("{}Variant {}", indent(level + 1), v.name);
                } else {
                    println!(
                        "{}Variant {}({})",
                        indent(level + 1),
                        v.name,
                        v.fields.join(", ")
                    );
                }
            }
        }
        Item::Module {
            name,
            items,
            is_pub,
        } => {
            let vis = if *is_pub { "pub " } else { "" };
            println!("{}{}Mod {}", indent(level), vis, name);
            for it in items {
                print_item(it, level + 1);
            }
        }
        Item::Trait {
            name,
            methods,
            is_pub,
        } => {
            let vis = if *is_pub { "pub " } else { "" };
            println!("{}{}Trait {}", indent(level), vis, name);
            for m in methods {
                println!("{}fn {}", indent(level + 1), m.name);
            }
        }
        Item::Impl {
            trait_name,
            type_name,
            methods,
        } => {
            if let Some(tr) = trait_name {
                println!("{}Impl {} for {}", indent(level), tr, type_name);
            } else {
                println!("{}Impl {}", indent(level), type_name);
            }
            for m in methods {
                print_item(m, level + 1);
            }
        }
    }
}

fn print_block(block: &Block, level: usize) {
    println!("{}Block", indent(level));
    for node in &block.statements {
        print_stmt(&node.stmt, level + 1);
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
        Stmt::While { condition, body } => {
            println!("{}While", indent(level));
            print_expr(condition, level + 1);
            print_block(body, level + 1);
        }
        Stmt::Break => println!("{}Break", indent(level)),
        Stmt::Continue => println!("{}Continue", indent(level)),
        Stmt::Return(None) => println!("{}Return", indent(level)),
        Stmt::Return(Some(e)) => {
            println!("{}Return", indent(level));
            print_expr(e, level + 1);
        }
        Stmt::Match { expr, arms } => {
            println!("{}Match", indent(level));
            print_expr(expr, level + 1);
            for arm in arms {
                match &arm.pattern {
                    Pattern::Variant {
                        enum_name,
                        variant,
                        binding,
                    } => {
                        if let Some(b) = binding {
                            println!("{}Arm {}.{}({})", indent(level + 1), enum_name, variant, b);
                        } else {
                            println!("{}Arm {}.{}", indent(level + 1), enum_name, variant);
                        }
                    }
                    Pattern::Wildcard => println!("{}Arm _", indent(level + 1)),
                }
                print_block(&arm.body, level + 2);
            }
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
        Expr::Try(e) => {
            println!("{}Try", indent(level));
            print_expr(e, level + 1);
        }
    }
}
