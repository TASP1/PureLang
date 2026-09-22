//! LLVM IR code generator for PureLang (opaque-pointer text emission)

use std::collections::HashMap;
use std::fmt::Write as _;

use crate::ast::*;

pub struct Codegen {
    preamble: String,
    strings_ir: String,
    types_ir: String,
    body: String,
    strings: HashMap<String, String>,
    string_counter: usize,
    temp: usize,
    label: usize,
    vars: HashMap<String, (String, VarKind)>,
    /// function name → param count (all i64 for now)
    functions: HashMap<String, usize>,
    /// struct name → field names
    structs: HashMap<String, Vec<String>>,
    current_is_main: bool,
    errors: Vec<String>,
}

#[derive(Clone, Copy, PartialEq)]
enum VarKind {
    Number,
    String,
    /// Pointer to a struct value on the stack
    Struct,
    /// Pointer to heap list: [i64 len][i64 elems...]
    List,
}

impl Codegen {
    pub fn new() -> Self {
        Codegen {
            preamble: String::new(),
            strings_ir: String::new(),
            types_ir: String::new(),
            body: String::new(),
            strings: HashMap::new(),
            string_counter: 0,
            temp: 0,
            label: 0,
            vars: HashMap::new(),
            functions: HashMap::new(),
            structs: HashMap::new(),
            current_is_main: true,
            errors: Vec::new(),
        }
    }

    pub fn generate(&mut self, program: &Program) -> Result<String, Vec<String>> {
        self.emit_preamble();

        // Register structs & functions first
        for item in &program.items {
            match item {
                Item::Struct { name, fields } => {
                    self.structs.insert(name.clone(), fields.clone());
                    // %Point = type { i64, i64, ... }
                    let fields_ir = fields.iter().map(|_| "i64").collect::<Vec<_>>().join(", ");
                    let _ = writeln!(self.types_ir, "%{} = type {{ {} }}", name, fields_ir);
                }
                Item::Function {
                    receiver,
                    name,
                    params,
                    ..
                } => {
                    // Methods are emitted as StructName_methodname
                    let full_name = if let Some(recv) = receiver {
                        format!("{}_{}", recv, name)
                    } else {
                        name.clone()
                    };
                    self.functions.insert(full_name, params.len());
                }
            }
        }

        let mut funcs = String::new();
        for item in &program.items {
            if let Item::Function {
                receiver,
                name,
                params,
                body,
            } = item
            {
                let full_name = if let Some(recv) = receiver {
                    format!("{}_{}", recv, name)
                } else {
                    name.clone()
                };
                self.emit_function(&full_name, params, body);
                funcs.push_str(&self.body);
            }
        }

        if !self.errors.is_empty() {
            return Err(std::mem::take(&mut self.errors));
        }

        let mut out = String::new();
        out.push_str(&self.preamble);
        out.push_str(&self.types_ir);
        out.push('\n');
        out.push_str(&self.strings_ir);
        out.push('\n');
        out.push_str(&funcs);
        Ok(out)
    }

    fn emit_preamble(&mut self) {
        self.preamble.push_str("; PureLang → LLVM IR\n");
        self.preamble
            .push_str("target triple = \"x86_64-unknown-linux-gnu\"\n\n");
        self.preamble.push_str("declare i32 @printf(ptr, ...)\n");
        self.preamble.push_str("declare ptr @malloc(i64)\n");
        self.preamble.push_str("declare ptr @strcpy(ptr, ptr)\n");
        self.preamble.push_str("declare ptr @strcat(ptr, ptr)\n");
        self.preamble.push_str("declare i64 @strlen(ptr)\n");
        self.preamble
            .push_str("declare i32 @snprintf(ptr, i64, ptr, ...)\n");
        self.preamble.push_str("declare void @free(ptr)\n\n");
        self.preamble.push_str(
            "@.fmt_str = private unnamed_addr constant [4 x i8] c\"%s\\0A\\00\", align 1\n",
        );
        self.preamble.push_str(
            "@.fmt_i64 = private unnamed_addr constant [6 x i8] c\"%lld\\0A\\00\", align 1\n",
        );
        self.preamble.push_str(
            "@.fmt_concat_sn = private unnamed_addr constant [7 x i8] c\"%s%lld\\00\", align 1\n",
        );
        self.preamble.push('\n');
    }

    fn intern_string(&mut self, s: &str) -> String {
        if let Some(name) = self.strings.get(s) {
            return name.clone();
        }
        let idx = self.string_counter;
        self.string_counter += 1;
        let name = format!("@.str.{}", idx);
        let mut escaped = String::new();
        for b in s.bytes() {
            match b {
                b'\n' => escaped.push_str("\\0A"),
                b'\r' => escaped.push_str("\\0D"),
                b'\t' => escaped.push_str("\\09"),
                b'"' => escaped.push_str("\\22"),
                b'\\' => escaped.push_str("\\5C"),
                c if (32..127).contains(&c) => escaped.push(c as char),
                c => escaped.push_str(&format!("\\{:02X}", c)),
            }
        }
        let len = s.len() + 1;
        let _ = writeln!(
            self.strings_ir,
            "{} = private unnamed_addr constant [{} x i8] c\"{}\\00\", align 1",
            name, len, escaped
        );
        self.strings.insert(s.to_string(), name.clone());
        name
    }

    fn fresh(&mut self) -> String {
        let t = self.temp;
        self.temp += 1;
        format!("%t{}", t)
    }

    fn fresh_label(&mut self, prefix: &str) -> String {
        let l = self.label;
        self.label += 1;
        format!("{}.{}", prefix, l)
    }

    fn emit_function(&mut self, name: &str, params: &[String], body: &Block) {
        self.vars.clear();
        self.temp = 0;
        self.label = 0;
        self.body.clear();

        let is_main = name == "main";
        self.current_is_main = is_main;
        // Detect method: name like "Point_distance" and first param is receiver (struct ptr)
        let is_method = name.contains('_') && !params.is_empty();
        if is_main {
            self.body.push_str("define i32 @main() {\nentry:\n");
        } else if is_method {
            // First arg is ptr (self), rest i64
            let mut param_list = vec!["ptr %arg0".to_string()];
            for i in 1..params.len() {
                param_list.push(format!("i64 %arg{}", i));
            }
            let _ = writeln!(
                self.body,
                "define i64 @{}({}) {{",
                name,
                param_list.join(", ")
            );
            self.body.push_str("entry:\n");
            // Store self (incoming ptr) into a stack slot so Ident load works uniformly
            if let Some(first) = params.first() {
                let ptr = format!("%{}.addr", first);
                let _ = writeln!(self.body, "  {} = alloca ptr, align 8", ptr);
                let _ = writeln!(self.body, "  store ptr %arg0, ptr {}, align 8", ptr);
                self.vars
                    .insert(first.clone(), (ptr, VarKind::Struct));
            }
            for (i, p) in params.iter().enumerate().skip(1) {
                let ptr = format!("%{}.addr", p);
                let _ = writeln!(self.body, "  {} = alloca i64, align 8", ptr);
                let _ = writeln!(self.body, "  store i64 %arg{}, ptr {}, align 8", i, ptr);
                self.vars.insert(p.clone(), (ptr, VarKind::Number));
            }
        } else {
            let param_list = params
                .iter()
                .enumerate()
                .map(|(i, _)| format!("i64 %arg{}", i))
                .collect::<Vec<_>>()
                .join(", ");
            let _ = writeln!(self.body, "define i64 @{}({}) {{", name, param_list);
            self.body.push_str("entry:\n");
            for (i, p) in params.iter().enumerate() {
                let ptr = format!("%{}.addr", p);
                let _ = writeln!(self.body, "  {} = alloca i64, align 8", ptr);
                let _ = writeln!(self.body, "  store i64 %arg{}, ptr {}, align 8", i, ptr);
                self.vars.insert(p.clone(), (ptr, VarKind::Number));
            }
        }

        self.emit_block(body);

        if is_main {
            self.body.push_str("  ret i32 0\n}\n\n");
        } else {
            // default return 0 if no explicit return
            self.body.push_str("  ret i64 0\n}\n\n");
        }
    }

    fn emit_block(&mut self, block: &Block) {
        for stmt in &block.statements {
            self.emit_stmt(stmt);
        }
    }

    fn emit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let {
                mutable: _,
                name,
                value,
            } => {
                let (val, kind) = self.emit_expr(value);
                if let Some((ptr, _)) = self.vars.get(name).cloned() {
                    match kind {
                        VarKind::Number => {
                            let _ =
                                writeln!(self.body, "  store i64 {}, ptr {}, align 8", val, ptr);
                        }
                        VarKind::String | VarKind::Struct | VarKind::List => {
                            let _ =
                                writeln!(self.body, "  store ptr {}, ptr {}, align 8", val, ptr);
                        }
                    }
                } else {
                    let ptr = format!("%{}.addr", name);
                    match kind {
                        VarKind::Number => {
                            let _ = writeln!(self.body, "  {} = alloca i64, align 8", ptr);
                            let _ =
                                writeln!(self.body, "  store i64 {}, ptr {}, align 8", val, ptr);
                        }
                        VarKind::String | VarKind::Struct | VarKind::List => {
                            let _ = writeln!(self.body, "  {} = alloca ptr, align 8", ptr);
                            let _ =
                                writeln!(self.body, "  store ptr {}, ptr {}, align 8", val, ptr);
                        }
                    }
                    self.vars.insert(name.clone(), (ptr, kind));
                }
            }
            Stmt::Assign { name, value } => {
                let (val, kind) = self.emit_expr(value);
                if let Some((ptr, _)) = self.vars.get(name).cloned() {
                    match kind {
                        VarKind::Number => {
                            let _ =
                                writeln!(self.body, "  store i64 {}, ptr {}, align 8", val, ptr);
                        }
                        VarKind::String | VarKind::Struct | VarKind::List => {
                            let _ =
                                writeln!(self.body, "  store ptr {}, ptr {}, align 8", val, ptr);
                        }
                    }
                } else {
                    self.errors
                        .push(format!("codegen: assign to unknown '{}'", name));
                }
            }
            Stmt::Print(expr) => self.emit_print(expr),
            Stmt::If {
                condition,
                then_block,
                else_block,
            } => {
                let (cond_val, _) = self.emit_expr(condition);
                let then_l = self.fresh_label("then");
                let else_l = self.fresh_label("else");
                let end_l = self.fresh_label("endif");
                if else_block.is_some() {
                    let _ = writeln!(
                        self.body,
                        "  br i1 {}, label %{}, label %{}",
                        cond_val, then_l, else_l
                    );
                } else {
                    let _ = writeln!(
                        self.body,
                        "  br i1 {}, label %{}, label %{}",
                        cond_val, then_l, end_l
                    );
                }
                let _ = writeln!(self.body, "{}:", then_l);
                self.emit_block(then_block);
                let _ = writeln!(self.body, "  br label %{}", end_l);
                if let Some(eb) = else_block {
                    let _ = writeln!(self.body, "{}:", else_l);
                    self.emit_block(eb);
                    let _ = writeln!(self.body, "  br label %{}", end_l);
                }
                let _ = writeln!(self.body, "{}:", end_l);
            }
            Stmt::For {
                var,
                iterable,
                body,
            } => {
                if let Expr::Range { start, end } = iterable {
                    let (start_v, _) = self.emit_expr(start);
                    let (end_v, _) = self.emit_expr(end);
                    let ivar = format!("%{}.addr", var);
                    let _ = writeln!(self.body, "  {} = alloca i64, align 8", ivar);
                    let _ = writeln!(self.body, "  store i64 {}, ptr {}, align 8", start_v, ivar);
                    self.vars
                        .insert(var.clone(), (ivar.clone(), VarKind::Number));

                    let loop_cond = self.fresh_label("loop.cond");
                    let loop_body = self.fresh_label("loop.body");
                    let loop_end = self.fresh_label("loop.end");
                    let _ = writeln!(self.body, "  br label %{}", loop_cond);
                    let _ = writeln!(self.body, "{}:", loop_cond);
                    let cur = self.fresh();
                    let _ = writeln!(self.body, "  {} = load i64, ptr {}, align 8", cur, ivar);
                    let cmp = self.fresh();
                    let _ = writeln!(self.body, "  {} = icmp slt i64 {}, {}", cmp, cur, end_v);
                    let _ = writeln!(
                        self.body,
                        "  br i1 {}, label %{}, label %{}",
                        cmp, loop_body, loop_end
                    );
                    let _ = writeln!(self.body, "{}:", loop_body);
                    self.emit_block(body);
                    let cur2 = self.fresh();
                    let _ = writeln!(self.body, "  {} = load i64, ptr {}, align 8", cur2, ivar);
                    let next = self.fresh();
                    let _ = writeln!(self.body, "  {} = add i64 {}, 1", next, cur2);
                    let _ = writeln!(self.body, "  store i64 {}, ptr {}, align 8", next, ivar);
                    let _ = writeln!(self.body, "  br label %{}", loop_cond);
                    let _ = writeln!(self.body, "{}:", loop_end);
                } else {
                    // for x in list { ... } — iterate by index
                    let (list_ptr, lkind) = self.emit_expr(iterable);
                    if lkind != VarKind::List {
                        self.errors
                            .push("codegen: for-loop expects Range or List".into());
                    }
                    let len = self.fresh();
                    let _ = writeln!(self.body, "  {} = load i64, ptr {}, align 8", len, list_ptr);
                    let idx = format!("%idx.{}", var);
                    let _ = writeln!(self.body, "  {} = alloca i64, align 8", idx);
                    let _ = writeln!(self.body, "  store i64 0, ptr {}, align 8", idx);

                    let vptr = format!("%{}.addr", var);
                    let _ = writeln!(self.body, "  {} = alloca i64, align 8", vptr);
                    self.vars
                        .insert(var.clone(), (vptr.clone(), VarKind::Number));

                    let loop_cond = self.fresh_label("forlist.cond");
                    let loop_body = self.fresh_label("forlist.body");
                    let loop_end = self.fresh_label("forlist.end");
                    let _ = writeln!(self.body, "  br label %{}", loop_cond);
                    let _ = writeln!(self.body, "{}:", loop_cond);
                    let cur = self.fresh();
                    let _ = writeln!(self.body, "  {} = load i64, ptr {}, align 8", cur, idx);
                    let cmp = self.fresh();
                    let _ = writeln!(self.body, "  {} = icmp slt i64 {}, {}", cmp, cur, len);
                    let _ = writeln!(
                        self.body,
                        "  br i1 {}, label %{}, label %{}",
                        cmp, loop_body, loop_end
                    );
                    let _ = writeln!(self.body, "{}:", loop_body);
                    // load element list[cur]
                    let off = self.fresh();
                    let _ = writeln!(self.body, "  {} = add i64 {}, 1", off, cur);
                    let ep = self.fresh();
                    let _ = writeln!(
                        self.body,
                        "  {} = getelementptr inbounds i64, ptr {}, i64 {}",
                        ep, list_ptr, off
                    );
                    let elem = self.fresh();
                    let _ = writeln!(self.body, "  {} = load i64, ptr {}, align 8", elem, ep);
                    let _ = writeln!(self.body, "  store i64 {}, ptr {}, align 8", elem, vptr);

                    self.emit_block(body);

                    let cur2 = self.fresh();
                    let _ = writeln!(self.body, "  {} = load i64, ptr {}, align 8", cur2, idx);
                    let next = self.fresh();
                    let _ = writeln!(self.body, "  {} = add i64 {}, 1", next, cur2);
                    let _ = writeln!(self.body, "  store i64 {}, ptr {}, align 8", next, idx);
                    let _ = writeln!(self.body, "  br label %{}", loop_cond);
                    let _ = writeln!(self.body, "{}:", loop_end);
                }
            }
            Stmt::Return(None) => {
                if self.current_is_main {
                    let _ = writeln!(self.body, "  ret i32 0");
                } else {
                    let _ = writeln!(self.body, "  ret i64 0");
                }
                let cont = self.fresh_label("after.ret");
                let _ = writeln!(self.body, "{}:", cont);
            }
            Stmt::Return(Some(expr)) => {
                let (v, kind) = self.emit_expr(expr);
                if self.current_is_main {
                    if kind == VarKind::Number {
                        let t = self.fresh();
                        let _ = writeln!(self.body, "  {} = trunc i64 {} to i32", t, v);
                        let _ = writeln!(self.body, "  ret i32 {}", t);
                    } else {
                        let _ = writeln!(self.body, "  ret i32 0");
                    }
                } else {
                    match kind {
                        VarKind::Number => {
                            let _ = writeln!(self.body, "  ret i64 {}", v);
                        }
                        _ => {
                            let _ = writeln!(self.body, "  ret i64 0");
                        }
                    }
                }
                let cont = self.fresh_label("after.ret");
                let _ = writeln!(self.body, "{}:", cont);
            }
            Stmt::Expr(expr) => {
                let _ = self.emit_expr(expr);
            }
        }
    }

    fn emit_print(&mut self, expr: &Expr) {
        if let Expr::Binary {
            left,
            op: BinaryOp::Add,
            right,
        } = expr
            && let Expr::String(s) = left.as_ref()
        {
            let (rval, rkind) = self.emit_expr(right);
            if rkind == VarKind::Number {
                let g = self.intern_string(s);
                let len = s.len() + 1;
                let sptr = self.fresh();
                let _ = writeln!(
                    self.body,
                    "  {} = getelementptr inbounds [{} x i8], ptr {}, i64 0, i64 0",
                    sptr, len, g
                );
                let fmt = self.fresh();
                let _ = writeln!(
                    self.body,
                    "  {} = getelementptr inbounds [7 x i8], ptr @.fmt_concat_sn, i64 0, i64 0",
                    fmt
                );
                let buf = self.fresh();
                let _ = writeln!(self.body, "  {} = call ptr @malloc(i64 256)", buf);
                let _ = writeln!(
                    self.body,
                    "  call i32 (ptr, i64, ptr, ...) @snprintf(ptr {}, i64 256, ptr {}, ptr {}, i64 {})",
                    buf, fmt, sptr, rval
                );
                let fmt_out = self.fresh();
                let _ = writeln!(
                    self.body,
                    "  {} = getelementptr inbounds [4 x i8], ptr @.fmt_str, i64 0, i64 0",
                    fmt_out
                );
                let _ = writeln!(
                    self.body,
                    "  call i32 (ptr, ...) @printf(ptr {}, ptr {})",
                    fmt_out, buf
                );
                let _ = writeln!(self.body, "  call void @free(ptr {})", buf);
                return;
            }
        }

        let (val, kind) = self.emit_expr(expr);
        match kind {
            VarKind::String => {
                let fmt = self.fresh();
                let _ = writeln!(
                    self.body,
                    "  {} = getelementptr inbounds [4 x i8], ptr @.fmt_str, i64 0, i64 0",
                    fmt
                );
                let _ = writeln!(
                    self.body,
                    "  call i32 (ptr, ...) @printf(ptr {}, ptr {})",
                    fmt, val
                );
            }
            VarKind::Number => {
                let fmt = self.fresh();
                let _ = writeln!(
                    self.body,
                    "  {} = getelementptr inbounds [6 x i8], ptr @.fmt_i64, i64 0, i64 0",
                    fmt
                );
                let _ = writeln!(
                    self.body,
                    "  call i32 (ptr, ...) @printf(ptr {}, i64 {})",
                    fmt, val
                );
            }
            VarKind::Struct | VarKind::List => {
                let fmt = self.fresh();
                let _ = writeln!(
                    self.body,
                    "  {} = getelementptr inbounds [6 x i8], ptr @.fmt_i64, i64 0, i64 0",
                    fmt
                );
                let as_i64 = self.fresh();
                let _ = writeln!(self.body, "  {} = ptrtoint ptr {} to i64", as_i64, val);
                let _ = writeln!(
                    self.body,
                    "  call i32 (ptr, ...) @printf(ptr {}, i64 {})",
                    fmt, as_i64
                );
            }
        }
    }

    fn emit_expr(&mut self, expr: &Expr) -> (String, VarKind) {
        match expr {
            Expr::Number(n) => ((n.to_i64_trunc()).to_string(), VarKind::Number),
            Expr::String(s) => {
                let g = self.intern_string(s);
                let len = s.len() + 1;
                let ptr = self.fresh();
                let _ = writeln!(
                    self.body,
                    "  {} = getelementptr inbounds [{} x i8], ptr {}, i64 0, i64 0",
                    ptr, len, g
                );
                (ptr, VarKind::String)
            }
            Expr::Bool(b) => ((if *b { 1 } else { 0 }).to_string(), VarKind::Number),
            Expr::Ident(name) => {
                if let Some((ptr, kind)) = self.vars.get(name).cloned() {
                    let loaded = self.fresh();
                    match kind {
                        VarKind::Number => {
                            let _ = writeln!(
                                self.body,
                                "  {} = load i64, ptr {}, align 8",
                                loaded, ptr
                            );
                        }
                        VarKind::String | VarKind::Struct | VarKind::List => {
                            let _ = writeln!(
                                self.body,
                                "  {} = load ptr, ptr {}, align 8",
                                loaded, ptr
                            );
                        }
                    }
                    (loaded, kind)
                } else {
                    self.errors.push(format!("codegen: undefined '{}'", name));
                    ("0".into(), VarKind::Number)
                }
            }
            Expr::Binary { left, op, right } => {
                if *op == BinaryOp::Add {
                    let (lv, lk) = self.emit_expr(left);
                    let (rv, rk) = self.emit_expr(right);
                    if lk == VarKind::String || rk == VarKind::String {
                        return self.emit_str_concat(lv, lk, rv, rk);
                    }
                    let res = self.fresh();
                    let _ = writeln!(self.body, "  {} = add i64 {}, {}", res, lv, rv);
                    return (res, VarKind::Number);
                }
                let (lv, _) = self.emit_expr(left);
                let (rv, _) = self.emit_expr(right);
                let res = self.fresh();
                match op {
                    BinaryOp::Add => {
                        let _ = writeln!(self.body, "  {} = add i64 {}, {}", res, lv, rv);
                    }
                    BinaryOp::Sub => {
                        let _ = writeln!(self.body, "  {} = sub i64 {}, {}", res, lv, rv);
                    }
                    BinaryOp::Mul => {
                        let _ = writeln!(self.body, "  {} = mul i64 {}, {}", res, lv, rv);
                    }
                    BinaryOp::Div => {
                        let _ = writeln!(self.body, "  {} = sdiv i64 {}, {}", res, lv, rv);
                    }
                    BinaryOp::Eq => {
                        let _ = writeln!(self.body, "  {} = icmp eq i64 {}, {}", res, lv, rv);
                    }
                    BinaryOp::NotEq => {
                        let _ = writeln!(self.body, "  {} = icmp ne i64 {}, {}", res, lv, rv);
                    }
                    BinaryOp::Lt => {
                        let _ = writeln!(self.body, "  {} = icmp slt i64 {}, {}", res, lv, rv);
                    }
                    BinaryOp::Gt => {
                        let _ = writeln!(self.body, "  {} = icmp sgt i64 {}, {}", res, lv, rv);
                    }
                    BinaryOp::LtEq => {
                        let _ = writeln!(self.body, "  {} = icmp sle i64 {}, {}", res, lv, rv);
                    }
                    BinaryOp::GtEq => {
                        let _ = writeln!(self.body, "  {} = icmp sge i64 {}, {}", res, lv, rv);
                    }
                }
                (res, VarKind::Number)
            }
            Expr::Unary { op, expr } => {
                let (v, _) = self.emit_expr(expr);
                let res = self.fresh();
                match op {
                    UnaryOp::Neg => {
                        let _ = writeln!(self.body, "  {} = sub i64 0, {}", res, v);
                    }
                    UnaryOp::Not => {
                        let _ = writeln!(self.body, "  {} = icmp eq i64 {}, 0", res, v);
                    }
                }
                (res, VarKind::Number)
            }
            Expr::Range { start, .. } => self.emit_expr(start),
            Expr::List(elements) => {
                let n = elements.len() as i64;
                let bytes = (n + 1) * 8;
                let ptr = self.fresh();
                let _ = writeln!(self.body, "  {} = call ptr @malloc(i64 {})", ptr, bytes);
                // store length
                let _ = writeln!(self.body, "  store i64 {}, ptr {}, align 8", n, ptr);
                for (i, el) in elements.iter().enumerate() {
                    let (v, k) = self.emit_expr(el);
                    if k != VarKind::Number {
                        self.errors
                            .push("codegen: list elements must be numbers for now".into());
                    }
                    let ep = self.fresh();
                    let _ = writeln!(
                        self.body,
                        "  {} = getelementptr inbounds i64, ptr {}, i64 {}",
                        ep,
                        ptr,
                        i + 1
                    );
                    let _ = writeln!(self.body, "  store i64 {}, ptr {}, align 8", v, ep);
                }
                (ptr, VarKind::List)
            }
            Expr::Index { object, index } => {
                let (obj, okind) = self.emit_expr(object);
                let (idx, _) = self.emit_expr(index);
                if okind != VarKind::List {
                    self.errors.push("codegen: indexing non-list".into());
                    return ("0".into(), VarKind::Number);
                }
                // ptr[0] = len, ptr[1+] = elems → offset = index + 1
                let off = self.fresh();
                let _ = writeln!(self.body, "  {} = add i64 {}, 1", off, idx);
                let ep = self.fresh();
                let _ = writeln!(
                    self.body,
                    "  {} = getelementptr inbounds i64, ptr {}, i64 {}",
                    ep, obj, off
                );
                let loaded = self.fresh();
                let _ = writeln!(self.body, "  {} = load i64, ptr {}, align 8", loaded, ep);
                (loaded, VarKind::Number)
            }
            Expr::Call { callee, args } => {
                // Struct construction or function call
                if let Expr::Ident(name) = callee.as_ref() {
                    if let Some(fields) = self.structs.get(name).cloned() {
                        // alloca struct, store fields
                        let ptr = self.fresh();
                        let _ = writeln!(self.body, "  {} = alloca %{}, align 8", ptr, name);
                        for (i, arg) in args.iter().enumerate() {
                            let (v, _) = self.emit_expr(arg);
                            let fp = self.fresh();
                            let _ = writeln!(
                                self.body,
                                "  {} = getelementptr inbounds %{}, ptr {}, i32 0, i32 {}",
                                fp, name, ptr, i
                            );
                            let _ = writeln!(self.body, "  store i64 {}, ptr {}, align 8", v, fp);
                            let _ = fields; // silence
                        }
                        return (ptr, VarKind::Struct);
                    }
                    if self.functions.contains_key(name) {
                        let mut arg_vals = Vec::new();
                        for a in args {
                            let (v, k) = self.emit_expr(a);
                            if k != VarKind::Number {
                                self.errors.push(format!(
                                    "codegen: only Number args supported in calls for now ({})",
                                    name
                                ));
                            }
                            arg_vals.push(v);
                        }
                        let args_ir = arg_vals
                            .iter()
                            .map(|v| format!("i64 {}", v))
                            .collect::<Vec<_>>()
                            .join(", ");
                        let res = self.fresh();
                        let _ = writeln!(self.body, "  {} = call i64 @{}({})", res, name, args_ir);
                        return (res, VarKind::Number);
                    }
                }
                // Method call: obj.method(args) → call @Struct_method(obj, args...)
                if let Expr::Field { object, field } = callee.as_ref() {
                    let (obj_val, obj_kind) = self.emit_expr(object);
                    if obj_kind != VarKind::Struct {
                        self.errors
                            .push("codegen: method call on non-struct".into());
                        return ("0".into(), VarKind::Number);
                    }
                    // Find which struct this method belongs to by scanning registered methods
                    // We store methods as Struct_method in self.functions
                    let mut found_name = None;
                    for (fname, _) in &self.functions {
                        if fname.ends_with(&format!("_{}", field)) {
                            found_name = Some(fname.clone());
                            break;
                        }
                    }
                    if let Some(full_name) = found_name {
                        let mut arg_vals = vec![obj_val];
                        for a in args {
                            let (v, k) = self.emit_expr(a);
                            if k != VarKind::Number {
                                self.errors.push(format!(
                                    "codegen: only Number args supported in method calls for now ({})",
                                    field
                                ));
                            }
                            arg_vals.push(v);
                        }
                        // First arg is ptr (struct), rest i64
                        let mut args_ir_parts = Vec::new();
                        if !arg_vals.is_empty() {
                            args_ir_parts.push(format!("ptr {}", arg_vals[0]));
                            for v in arg_vals.iter().skip(1) {
                                args_ir_parts.push(format!("i64 {}", v));
                            }
                        }
                        let args_ir = args_ir_parts.join(", ");
                        let res = self.fresh();
                        let _ = writeln!(
                            self.body,
                            "  {} = call i64 @{}({})",
                            res, full_name, args_ir
                        );
                        return (res, VarKind::Number);
                    }
                    self.errors
                        .push(format!("codegen: unknown method '{}'", field));
                    return ("0".into(), VarKind::Number);
                }
                self.errors.push("codegen: unsupported call".into());
                ("0".into(), VarKind::Number)
            }
            Expr::Field { object, field } => {
                let (obj, kind) = self.emit_expr(object);
                if field == "length" && kind == VarKind::List {
                    let loaded = self.fresh();
                    let _ = writeln!(self.body, "  {} = load i64, ptr {}, align 8", loaded, obj);
                    return (loaded, VarKind::Number);
                }
                if kind != VarKind::Struct {
                    self.errors
                        .push("codegen: field access on non-struct".into());
                    return ("0".into(), VarKind::Number);
                }
                // find field index from struct type of... we don't track struct name on VarKind
                // Heuristic: look at getelementptr - need struct name. Store as StructName in vars.
                // For MVP: try all structs for field name
                let mut idx = None;
                let mut sname = None;
                for (sn, fields) in &self.structs {
                    if let Some(i) = fields.iter().position(|f| f == field) {
                        idx = Some(i);
                        sname = Some(sn.clone());
                        break;
                    }
                }
                if let (Some(i), Some(sn)) = (idx, sname) {
                    let fp = self.fresh();
                    let _ = writeln!(
                        self.body,
                        "  {} = getelementptr inbounds %{}, ptr {}, i32 0, i32 {}",
                        fp, sn, obj, i
                    );
                    let loaded = self.fresh();
                    let _ = writeln!(self.body, "  {} = load i64, ptr {}, align 8", loaded, fp);
                    (loaded, VarKind::Number)
                } else {
                    self.errors
                        .push(format!("codegen: unknown field '{}'", field));
                    ("0".into(), VarKind::Number)
                }
            }
        }
    }

    fn emit_str_concat(
        &mut self,
        lv: String,
        lk: VarKind,
        rv: String,
        rk: VarKind,
    ) -> (String, VarKind) {
        let left_s = self.ensure_string(lv, lk);
        let right_s = self.ensure_string(rv, rk);
        let len1 = self.fresh();
        let len2 = self.fresh();
        let total = self.fresh();
        let total1 = self.fresh();
        let _ = writeln!(self.body, "  {} = call i64 @strlen(ptr {})", len1, left_s);
        let _ = writeln!(self.body, "  {} = call i64 @strlen(ptr {})", len2, right_s);
        let _ = writeln!(self.body, "  {} = add i64 {}, {}", total, len1, len2);
        let _ = writeln!(self.body, "  {} = add i64 {}, 1", total1, total);
        let dest = self.fresh();
        let _ = writeln!(self.body, "  {} = call ptr @malloc(i64 {})", dest, total1);
        let _ = writeln!(
            self.body,
            "  call ptr @strcpy(ptr {}, ptr {})",
            dest, left_s
        );
        let _ = writeln!(
            self.body,
            "  call ptr @strcat(ptr {}, ptr {})",
            dest, right_s
        );
        (dest, VarKind::String)
    }

    fn ensure_string(&mut self, val: String, kind: VarKind) -> String {
        match kind {
            VarKind::String => val,
            VarKind::Struct | VarKind::List => val,
            VarKind::Number => {
                let buf = self.fresh();
                let _ = writeln!(self.body, "  {} = call ptr @malloc(i64 32)", buf);
                let g = self.intern_string("%lld");
                let fmt = self.fresh();
                let _ = writeln!(
                    self.body,
                    "  {} = getelementptr inbounds [5 x i8], ptr {}, i64 0, i64 0",
                    fmt, g
                );
                let _ = writeln!(
                    self.body,
                    "  call i32 (ptr, i64, ptr, ...) @snprintf(ptr {}, i64 32, ptr {}, i64 {})",
                    buf, fmt, val
                );
                buf
            }
        }
    }
}

trait F64Ext {
    fn to_i64_trunc(self) -> i64;
}
impl F64Ext for f64 {
    fn to_i64_trunc(self) -> i64 {
        self as i64
    }
}
