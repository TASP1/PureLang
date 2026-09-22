//! WebAssembly Text Format (.wat) emitter for PureLang

use std::collections::HashMap;
use std::fmt::Write as _;

use crate::ast::*;

pub struct WasmCodegen {
    /// data segment bytes
    data: Vec<u8>,
    /// string → offset in data
    strings: HashMap<String, u32>,
    /// wat function body lines
    body: Vec<String>,
    /// local declarations for current function
    locals: Vec<String>,
    /// variable name → local index
    vars: HashMap<String, u32>,
    next_local: u32,
    label_id: u32,
    errors: Vec<String>,
}

impl WasmCodegen {
    pub fn new() -> Self {
        WasmCodegen {
            data: Vec::new(),
            strings: HashMap::new(),
            body: Vec::new(),
            locals: Vec::new(),
            vars: HashMap::new(),
            next_local: 0,
            label_id: 0,
            errors: Vec::new(),
        }
    }

    pub fn generate(&mut self, program: &Program) -> Result<String, Vec<String>> {
        // Reserve local 0 as scratch in main; we'll declare properly per function
        for item in &program.items {
            if let Item::Function { name, params, body } = item
                && name == "main"
            {
                self.emit_main(params, body);
            }
        }

        if !self.errors.is_empty() {
            return Err(std::mem::take(&mut self.errors));
        }

        Ok(self.finish_module())
    }

    fn intern(&mut self, s: &str) -> u32 {
        if let Some(&off) = self.strings.get(s) {
            return off;
        }
        let off = self.data.len() as u32;
        self.data.extend_from_slice(s.as_bytes());
        self.data.push(0); // null terminator
        self.strings.insert(s.to_string(), off);
        off
    }

    fn alloc_local(&mut self, name: &str, ty: &str) -> u32 {
        let idx = self.next_local;
        self.next_local += 1;
        self.locals.push(format!("(local ${} {})", name, ty));
        self.vars.insert(name.to_string(), idx);
        idx
    }

    fn fresh_local(&mut self, ty: &str) -> u32 {
        let idx = self.next_local;
        self.next_local += 1;
        self.locals.push(format!("(local $t{} {})", idx, ty));
        idx
    }

    fn emit_main(&mut self, _params: &[String], body: &Block) {
        self.vars.clear();
        self.locals.clear();
        self.body.clear();
        self.next_local = 0;
        self.label_id = 0;

        // scratch locals used by helpers
        self.alloc_local("_iov", "i32"); // iov base pointer on stack region
        self.alloc_local("_nwritten", "i32");
        self.alloc_local("_tmp", "i32");
        self.alloc_local("_tmp64", "i64");

        self.emit_block(body);

        // main returns void for WASI (_start will call it)
    }

    fn emit_block(&mut self, block: &Block) {
        for stmt in &block.statements {
            self.emit_stmt(stmt);
        }
    }

    fn emit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let { name, value, .. } | Stmt::Assign { name, value } => {
                let is_str = self.expr_is_string(value);
                if !self.vars.contains_key(name) {
                    if is_str {
                        self.alloc_local(name, "i32"); // pointer
                    } else {
                        self.alloc_local(name, "i64");
                    }
                }
                self.emit_expr(value);
                if is_str {
                    self.body.push(format!("    local.set ${}", name));
                } else {
                    // if expression left i32 (bool/cmp), extend — we use i64 for numbers
                    self.body.push(format!("    local.set ${}", name));
                }
            }
            Stmt::Print(expr) => {
                self.emit_print(expr);
            }
            Stmt::If {
                condition,
                then_block,
                else_block,
            } => {
                self.emit_expr(condition);
                // condition yields i32 (from comparisons) or i64 — convert
                self.body.push("    i32.wrap_i64".into()); // if i64
                // Actually comparisons emit i32. Track better below.
                // Use if/else structure
                self.body.push("    if".into());
                self.emit_block(then_block);
                if let Some(eb) = else_block {
                    self.body.push("    else".into());
                    self.emit_block(eb);
                }
                self.body.push("    end".into());
            }
            Stmt::For {
                var,
                iterable,
                body,
            } => {
                if let Expr::Range { start, end } = iterable {
                    if !self.vars.contains_key(var) {
                        self.alloc_local(var, "i64");
                    }
                    self.emit_expr(start);
                    self.body.push(format!("    local.set ${}", var));

                    let end_local = self.fresh_local("i64");
                    self.emit_expr(end);
                    self.body.push(format!("    local.set $t{}", end_local));

                    // block $break
                    //   loop $continue
                    //     ...
                    //   end
                    // end
                    self.body.push("    block $break".into());
                    self.body.push("      loop $continue".into());
                    self.body.push(format!("        local.get ${}", var));
                    self.body.push(format!("        local.get $t{}", end_local));
                    self.body.push("        i64.lt_s".into());
                    self.body.push("        i32.eqz".into());
                    self.body.push("        br_if $break".into());

                    self.emit_block(body);

                    // i = i + 1
                    self.body.push(format!("        local.get ${}", var));
                    self.body.push("        i64.const 1".into());
                    self.body.push("        i64.add".into());
                    self.body.push(format!("        local.set ${}", var));
                    self.body.push("        br $continue".into());
                    self.body.push("      end".into());
                    self.body.push("    end".into());
                } else {
                    self.errors
                        .push("wasm: only range for-loops supported".into());
                }
            }
            Stmt::Return(_) => {
                self.body.push("    return".into());
            }
            Stmt::Expr(expr) => {
                self.emit_expr(expr);
                self.body.push("    drop".into());
            }
        }
    }

    fn expr_is_string(&self, expr: &Expr) -> bool {
        match expr {
            Expr::String(_) => true,
            Expr::Ident(name) => {
                // heuristic: if we know it's a string local
                if let Some(&idx) = self.vars.get(name) {
                    // check local type from locals vec
                    self.locals
                        .get(idx as usize)
                        .map(|l| l.contains("i32"))
                        .unwrap_or(false)
                } else {
                    false
                }
            }
            Expr::Binary {
                op: BinaryOp::Add,
                left,
                right,
            } => self.expr_is_string(left) || self.expr_is_string(right),
            _ => false,
        }
    }

    fn emit_print(&mut self, expr: &Expr) {
        // Build a string in linear memory and call $print_cstr
        match expr {
            Expr::String(s) => {
                let off = self.intern(s);
                self.body.push(format!("    i32.const {}", off));
                self.body.push("    call $print_cstr".into());
            }
            Expr::Binary {
                left,
                op: BinaryOp::Add,
                right,
            } if matches!(left.as_ref(), Expr::String(_)) => {
                if let Expr::String(s) = left.as_ref() {
                    // print left string then right
                    let off = self.intern(s);
                    self.body.push(format!("    i32.const {}", off));
                    self.body.push("    call $print_cstr_nonl".into());
                    self.emit_expr(right);
                    if self.expr_is_string(right) {
                        self.body.push("    call $print_cstr".into());
                    } else {
                        self.body.push("    call $print_i64".into());
                    }
                }
            }
            _ => {
                if self.expr_is_string(expr) {
                    self.emit_expr(expr);
                    self.body.push("    call $print_cstr".into());
                } else {
                    self.emit_expr(expr);
                    self.body.push("    call $print_i64".into());
                }
            }
        }
    }

    /// Leaves value on stack: i64 for numbers, i32 ptr for strings
    fn emit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Number(n) => {
                self.body.push(format!("    i64.const {}", *n as i64));
            }
            Expr::Bool(b) => {
                self.body
                    .push(format!("    i64.const {}", if *b { 1 } else { 0 }));
            }
            Expr::String(s) => {
                let off = self.intern(s);
                self.body.push(format!("    i32.const {}", off));
            }
            Expr::Ident(name) => {
                self.body.push(format!("    local.get ${}", name));
            }
            Expr::Binary { left, op, right } => {
                if *op == BinaryOp::Add && (self.expr_is_string(left) || self.expr_is_string(right))
                {
                    // string concat not fully supported in wasm MVP — print path handles common case
                    // Fallback: just evaluate right
                    self.errors.push(
                        "wasm: string concatenation in expressions not fully supported; use print"
                            .into(),
                    );
                    self.emit_expr(right);
                    return;
                }
                self.emit_expr(left);
                self.emit_expr(right);
                match op {
                    BinaryOp::Add => self.body.push("    i64.add".into()),
                    BinaryOp::Sub => self.body.push("    i64.sub".into()),
                    BinaryOp::Mul => self.body.push("    i64.mul".into()),
                    BinaryOp::Div => self.body.push("    i64.div_s".into()),
                    BinaryOp::Eq => {
                        self.body.push("    i64.eq".into());
                        self.body.push("    i64.extend_i32_u".into());
                    }
                    BinaryOp::NotEq => {
                        self.body.push("    i64.ne".into());
                        self.body.push("    i64.extend_i32_u".into());
                    }
                    BinaryOp::Lt => {
                        self.body.push("    i64.lt_s".into());
                        self.body.push("    i64.extend_i32_u".into());
                    }
                    BinaryOp::Gt => {
                        self.body.push("    i64.gt_s".into());
                        self.body.push("    i64.extend_i32_u".into());
                    }
                    BinaryOp::LtEq => {
                        self.body.push("    i64.le_s".into());
                        self.body.push("    i64.extend_i32_u".into());
                    }
                    BinaryOp::GtEq => {
                        self.body.push("    i64.ge_s".into());
                        self.body.push("    i64.extend_i32_u".into());
                    }
                }
            }
            Expr::Unary { op, expr } => match op {
                UnaryOp::Neg => {
                    self.body.push("    i64.const 0".into());
                    self.emit_expr(expr);
                    self.body.push("    i64.sub".into());
                }
                UnaryOp::Not => {
                    self.emit_expr(expr);
                    self.body.push("    i64.eqz".into());
                    self.body.push("    i64.extend_i32_u".into());
                }
            },
            Expr::Range { start, .. } => self.emit_expr(start),
            Expr::List(_) | Expr::Call { .. } | Expr::Field { .. } => {
                self.errors.push("wasm: unsupported expression".into());
                self.body.push("    i64.const 0".into());
            }
        }
    }

    fn finish_module(&self) -> String {
        let mut out = String::new();
        out.push_str(";; PureLang → WebAssembly (WASI)\n");
        out.push_str("(module\n");
        // WASI fd_write
        out.push_str("  (import \"wasi_snapshot_preview1\" \"fd_write\"\n");
        out.push_str("    (func $fd_write (param i32 i32 i32 i32) (result i32)))\n\n");

        // memory + data
        let mem_pages = 2;
        let _ = writeln!(out, "  (memory (export \"memory\") {})", mem_pages);
        if !self.data.is_empty() {
            out.push_str("  (data (i32.const 0) \"");
            for b in &self.data {
                match *b {
                    b'\\' => out.push_str("\\\\"),
                    b'"' => out.push_str("\\\""),
                    c if (32..127).contains(&c) => out.push(c as char),
                    c => {
                        let _ = write!(out, "\\{:02x}", c);
                    }
                }
            }
            out.push_str("\")\n\n");
        }

        // iov buffer region starts at 64KB - 64
        // We'll use fixed addresses: iov at 65536-32, nwritten at 65536-16
        // Actually use low addresses after data: align data end
        let heap_base = self.data.len().div_ceil(16) * 16 + 16;
        let iov_addr = heap_base;
        let nwritten_addr = heap_base + 16;
        let numbuf_addr = heap_base + 32;

        // $print_cstr: (param $ptr i32) — print null-terminated string + newline via fd_write
        out.push_str("  (func $print_cstr (param $ptr i32)\n");
        out.push_str("    (local $len i32)\n");
        out.push_str("    ;; strlen\n");
        out.push_str("    (local.set $len (i32.const 0))\n");
        out.push_str("    (block $done\n");
        out.push_str("      (loop $loop\n");
        out.push_str(
            "        (i32.eqz (i32.load8_u (i32.add (local.get $ptr) (local.get $len))))\n",
        );
        out.push_str("        br_if $done\n");
        out.push_str("        (local.set $len (i32.add (local.get $len) (i32.const 1)))\n");
        out.push_str("        br $loop\n");
        out.push_str("      )\n");
        out.push_str("    )\n");
        // write string
        let _ = writeln!(
            out,
            "    (i32.store (i32.const {}) (local.get $ptr))",
            iov_addr
        );
        let _ = writeln!(
            out,
            "    (i32.store (i32.const {}) (local.get $len))",
            iov_addr + 4
        );
        let _ = writeln!(
            out,
            "    (call $fd_write (i32.const 1) (i32.const {}) (i32.const 1) (i32.const {}))",
            iov_addr, nwritten_addr
        );
        out.push_str("    drop\n");
        // newline
        let nl_off = {
            // embed newline in data at end — use a fixed const in memory
            // We'll put newline byte at heap_base+64
            0u32
        };
        let _ = nl_off;
        let nl_addr = heap_base + 64;
        let _ = writeln!(
            out,
            "    (i32.store8 (i32.const {}) (i32.const 10))",
            nl_addr
        );
        let _ = writeln!(
            out,
            "    (i32.store (i32.const {}) (i32.const {}))",
            iov_addr, nl_addr
        );
        let _ = writeln!(
            out,
            "    (i32.store (i32.const {}) (i32.const 1))",
            iov_addr + 4
        );
        let _ = writeln!(
            out,
            "    (call $fd_write (i32.const 1) (i32.const {}) (i32.const 1) (i32.const {}))",
            iov_addr, nwritten_addr
        );
        out.push_str("    drop\n");
        out.push_str("  )\n\n");

        // print without newline
        out.push_str("  (func $print_cstr_nonl (param $ptr i32)\n");
        out.push_str("    (local $len i32)\n");
        out.push_str("    (local.set $len (i32.const 0))\n");
        out.push_str("    (block $done\n");
        out.push_str("      (loop $loop\n");
        out.push_str(
            "        (i32.eqz (i32.load8_u (i32.add (local.get $ptr) (local.get $len))))\n",
        );
        out.push_str("        br_if $done\n");
        out.push_str("        (local.set $len (i32.add (local.get $len) (i32.const 1)))\n");
        out.push_str("        br $loop\n");
        out.push_str("      )\n");
        out.push_str("    )\n");
        let _ = writeln!(
            out,
            "    (i32.store (i32.const {}) (local.get $ptr))",
            iov_addr
        );
        let _ = writeln!(
            out,
            "    (i32.store (i32.const {}) (local.get $len))",
            iov_addr + 4
        );
        let _ = writeln!(
            out,
            "    (call $fd_write (i32.const 1) (i32.const {}) (i32.const 1) (i32.const {}))",
            iov_addr, nwritten_addr
        );
        out.push_str("    drop\n");
        out.push_str("  )\n\n");

        // print i64
        out.push_str("  (func $print_i64 (param $v i64)\n");
        out.push_str("    (local $buf i32) (local $p i32) (local $neg i32) (local $dig i64)\n");
        let _ = writeln!(out, "    (local.set $buf (i32.const {}))", numbuf_addr + 32);
        out.push_str("    (local.set $p (local.get $buf))\n");
        out.push_str("    (local.set $neg (i32.const 0))\n");
        out.push_str("    (if (i64.lt_s (local.get $v) (i64.const 0))\n");
        out.push_str("      (then\n");
        out.push_str("        (local.set $neg (i32.const 1))\n");
        out.push_str("        (local.set $v (i64.sub (i64.const 0) (local.get $v)))\n");
        out.push_str("      )\n");
        out.push_str("    )\n");
        out.push_str("    (if (i64.eq (local.get $v) (i64.const 0))\n");
        out.push_str("      (then\n");
        out.push_str("        (local.set $p (i32.sub (local.get $p) (i32.const 1)))\n");
        out.push_str("        (i32.store8 (local.get $p) (i32.const 48))\n");
        out.push_str("      )\n");
        out.push_str("      (else\n");
        out.push_str("        (loop $digits\n");
        out.push_str("          (local.set $dig (i64.rem_u (local.get $v) (i64.const 10)))\n");
        out.push_str("          (local.set $v (i64.div_u (local.get $v) (i64.const 10)))\n");
        out.push_str("          (local.set $p (i32.sub (local.get $p) (i32.const 1)))\n");
        out.push_str("          (i32.store8 (local.get $p) (i32.add (i32.wrap_i64 (local.get $dig)) (i32.const 48)))\n");
        out.push_str("          (br_if $digits (i64.ne (local.get $v) (i64.const 0)))\n");
        out.push_str("        )\n");
        out.push_str("      )\n");
        out.push_str("    )\n");
        out.push_str("    (if (local.get $neg) (then\n");
        out.push_str("      (local.set $p (i32.sub (local.get $p) (i32.const 1)))\n");
        out.push_str("      (i32.store8 (local.get $p) (i32.const 45))\n");
        out.push_str("    ))\n");
        // null terminate at buf
        let _ = writeln!(
            out,
            "    (i32.store8 (i32.const {}) (i32.const 0))",
            numbuf_addr + 32
        );
        out.push_str("    (call $print_cstr (local.get $p))\n");
        out.push_str("  )\n\n");

        // user main
        out.push_str("  (func $main\n");
        for loc in &self.locals {
            let _ = writeln!(out, "    {}", loc);
        }
        for line in &self.body {
            out.push_str(line);
            out.push('\n');
        }
        out.push_str("  )\n\n");

        // WASI _start
        out.push_str("  (func (export \"_start\")\n");
        out.push_str("    call $main\n");
        out.push_str("  )\n");
        out.push_str(")\n");
        out
    }
}
