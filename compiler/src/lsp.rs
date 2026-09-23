//! Minimal Language Server Protocol (stdio, JSON-RPC)

use std::io::{self, BufRead, Write};
use std::path::Path;

use crate::checker::TypeChecker;
use crate::lexer::Lexer;
use crate::parser::Parser;

pub fn run() {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut stdout = io::stdout();
    let mut open_files: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    loop {
        let msg = match read_message(&mut stdin) {
            Ok(Some(m)) => m,
            Ok(None) => break,
            Err(e) => {
                eprintln!("LSP read error: {}", e);
                break;
            }
        };
        let method = json_str_field(&msg, "method").unwrap_or_default();
        let id = json_raw_field(&msg, "id");

        match method.as_str() {
            "initialize" => {
                let result = r#"{"capabilities":{"textDocumentSync":1,"hoverProvider":true,"diagnosticProvider":{}}}"#;
                respond(&mut stdout, id.as_deref(), result);
            }
            "initialized" | "workspace/didChangeConfiguration" => {}
            "shutdown" => {
                respond(&mut stdout, id.as_deref(), "null");
            }
            "exit" => break,
            "textDocument/didOpen" => {
                if let Some((uri, text)) = parse_did_open(&msg) {
                    open_files.insert(uri.clone(), text.clone());
                    publish_diagnostics(&mut stdout, &uri, &text);
                }
            }
            "textDocument/didChange" => {
                if let Some((uri, text)) = parse_did_change(&msg) {
                    open_files.insert(uri.clone(), text.clone());
                    publish_diagnostics(&mut stdout, &uri, &text);
                }
            }
            "textDocument/didClose" => {
                if let Some(uri) = json_nested_str(&msg, &["params", "textDocument", "uri"]) {
                    open_files.remove(&uri);
                }
            }
            "textDocument/hover" => {
                let result = r#"{"contents":{"kind":"markdown","value":"**PureLang**\n\nType-check with purec. Hover detail expands in future LSP versions."}}"#;
                respond(&mut stdout, id.as_deref(), result);
            }
            "" if id.is_some() => {
                // response to our request — ignore
            }
            _ => {
                if id.is_some() {
                    // Method not found
                    let err = r#"{"code":-32601,"message":"Method not found"}"#;
                    respond_error(&mut stdout, id.as_deref(), err);
                }
            }
        }
    }
}

fn read_message(stdin: &mut impl BufRead) -> io::Result<Option<String>> {
    let mut content_length = None;
    loop {
        let mut line = String::new();
        let n = stdin.read_line(&mut line)?;
        if n == 0 {
            return Ok(None);
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some(rest) = line.strip_prefix("Content-Length:") {
            content_length = Some(rest.trim().parse::<usize>().unwrap_or(0));
        }
    }
    let len = match content_length {
        Some(l) if l > 0 => l,
        _ => return Ok(None),
    };
    let mut buf = vec![0u8; len];
    stdin.read_exact(&mut buf)?;
    Ok(Some(String::from_utf8_lossy(&buf).into_owned()))
}

fn write_message(stdout: &mut impl Write, body: &str) {
    let _ = write!(stdout, "Content-Length: {}\r\n\r\n{}", body.len(), body);
    let _ = stdout.flush();
}

fn respond(stdout: &mut impl Write, id: Option<&str>, result: &str) {
    let id = id.unwrap_or("null");
    let body = format!(r#"{{"jsonrpc":"2.0","id":{},"result":{}}}"#, id, result);
    write_message(stdout, &body);
}

fn respond_error(stdout: &mut impl Write, id: Option<&str>, error: &str) {
    let id = id.unwrap_or("null");
    let body = format!(r#"{{"jsonrpc":"2.0","id":{},"error":{}}}"#, id, error);
    write_message(stdout, &body);
}

fn notify(stdout: &mut impl Write, method: &str, params: &str) {
    let body = format!(
        r#"{{"jsonrpc":"2.0","method":"{}","params":{}}}"#,
        method, params
    );
    write_message(stdout, &body);
}

fn publish_diagnostics(stdout: &mut impl Write, uri: &str, text: &str) {
    let diags = analyze(text);
    let mut arr = String::from("[");
    for (i, d) in diags.iter().enumerate() {
        if i > 0 {
            arr.push(',');
        }
        arr.push_str(&format!(
            r#"{{"range":{{"start":{{"line":{},"character":0}},"end":{{"line":{},"character":200}}}},"severity":1,"source":"purec","message":{}}}"#,
            d.0,
            d.0,
            json_escape(&d.1)
        ));
    }
    arr.push(']');
    let params = format!(r#"{{"uri":{},"diagnostics":{}}}"#, json_escape(uri), arr);
    notify(stdout, "textDocument/publishDiagnostics", &params);
}

struct Diag(u32, String); // line, message

fn analyze(source: &str) -> Vec<Diag> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => {
            return vec![Diag(0, e.to_string())];
        }
    };
    let mut checker = TypeChecker::new();
    match checker.check_program(&program) {
        Ok(()) => Vec::new(),
        Err(errors) => errors.into_iter().map(|e| Diag(0, e.to_string())).collect(),
    }
}

fn json_escape(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn json_str_field(msg: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\"", key);
    let idx = msg.find(&pattern)?;
    let after = &msg[idx + pattern.len()..];
    let after = after.trim_start();
    let after = after.strip_prefix(':')?.trim_start();
    if let Some(rest) = after.strip_prefix('"') {
        let mut s = String::new();
        let mut chars = rest.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                if let Some(n) = chars.next() {
                    s.push(n);
                }
            } else if c == '"' {
                break;
            } else {
                s.push(c);
            }
        }
        return Some(s);
    }
    None
}

fn json_raw_field(msg: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\"", key);
    let idx = msg.find(&pattern)?;
    let after = &msg[idx + pattern.len()..];
    let after = after.trim_start().strip_prefix(':')?.trim_start();
    if after.starts_with('"') {
        return json_str_field(msg, key);
    }
    // number or null
    let end = after.find([',', '}', ']']).unwrap_or(after.len());
    Some(after[..end].trim().to_string())
}

fn json_nested_str(msg: &str, keys: &[&str]) -> Option<String> {
    // naive: search last key as string field
    json_str_field(msg, keys.last()?)
}

fn parse_did_open(msg: &str) -> Option<(String, String)> {
    let uri = json_nested_str(msg, &["params", "textDocument", "uri"])?;
    // text is in textDocument.text
    let text = extract_text_field(msg)?;
    let _ = Path::new(&uri);
    Some((uri, text))
}

fn parse_did_change(msg: &str) -> Option<(String, String)> {
    let uri = json_nested_str(msg, &["params", "textDocument", "uri"])?;
    // Full document sync: contentChanges[0].text
    let text = extract_text_field(msg)?;
    Some((uri, text))
}

fn extract_text_field(msg: &str) -> Option<String> {
    // Find "text": "..." with possible escapes — last occurrence often the doc body
    let key = "\"text\"";
    let mut last = None;
    let mut search = msg;
    let mut offset = 0;
    while let Some(idx) = search.find(key) {
        let abs = offset + idx;
        let after = &msg[abs + key.len()..];
        let after = after.trim_start();
        if let Some(after) = after.strip_prefix(':') {
            let after = after.trim_start();
            if after.starts_with('"') {
                last = Some(abs);
            }
        }
        offset = abs + key.len();
        search = &msg[offset..];
    }
    let abs = last?;
    let after = &msg[abs + key.len()..];
    let after = after.trim_start().strip_prefix(':')?.trim_start();
    let rest = after.strip_prefix('"')?;
    let mut s = String::new();
    let mut chars = rest.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => s.push('\n'),
                Some('t') => s.push('\t'),
                Some('r') => s.push('\r'),
                Some('"') => s.push('"'),
                Some('\\') => s.push('\\'),
                Some(o) => s.push(o),
                None => {}
            }
        } else if c == '"' {
            break;
        } else {
            s.push(c);
        }
    }
    Some(s)
}
