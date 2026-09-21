//! PureLang Compiler (purec) - Phase 1: Lexer

mod token;
mod lexer;

use std::env;
use std::fs;
use std::process;

use lexer::Lexer;
use token::Token;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("PureLang Compiler (purec) v0.1.0");
        eprintln!();
        eprintln!("Usage:");
        eprintln!("  purec <file.pure>          Tokenize a PureLang source file");
        eprintln!("  purec --version            Show version");
        eprintln!();
        eprintln!("Example:");
        eprintln!("  purec examples/hello.pure");
        process::exit(1);
    }

    if args[1] == "--version" || args[1] == "-V" {
        println!("purec 0.1.0 (PureLang compiler - lexer stage)");
        return;
    }

    let filename = &args[1];
    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", filename, e);
            process::exit(1);
        }
    };

    println!("=== PureLang Lexer ===");
    println!("File: {}", filename);
    println!("----------------------");

    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    for (i, token) in tokens.iter().enumerate() {
        if matches!(token, Token::Eof) {
            println!("{:3}: EOF", i);
        } else {
            println!("{:3}: {:?}", i, token);
        }
    }

    println!("----------------------");
    println!("Total tokens: {}", tokens.len());
}
