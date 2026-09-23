//! LLVM IR code generator for PureLang (opaque-pointer text emission)

use std::collections::HashMap;
use std::fmt::Write as _;

use crate::ast::*;

pub struct Codegen {
    target_triple: String,
    preamble: String,
    strings_ir: String,
    types_ir: String,
    body: String,
    strings: HashMap<String, String>,
    string_counter: usize,
    temp: usize,
    label: usize,
    vars: HashMap<String, (String, VarKind)>,
    /// function name → param count
    functions: HashMap<String, usize>,
    /// function name → whether each param is a pointer (Struct/String)
    function_param_is_ptr: HashMap<String, Vec<bool>>,
    /// struct name → field names
    structs: HashMap<String, Vec<String>>,
    /// enum name → list of (variant name, field count)
    enums: HashMap<String, Vec<(String, usize)>>,
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
    /// Enum value as i64: (tag << 32) | (payload as i32/i64 lower bits)
    Enum,
}

impl Codegen {
    pub fn new() -> Self {
        Self::with_target(host_triple())
    }

    pub fn with_target(triple: impl Into<String>) -> Self {
        Codegen {
            target_triple: triple.into(),
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
            function_param_is_ptr: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            current_is_main: true,
            errors: Vec::new(),
        }
    }

    fn flatten_items(items: &[Item]) -> Vec<Item> {
        let mut out = Vec::new();
        for item in items {
            match item {
                Item::Module { name, items, .. } => {
                    for inner in Self::flatten_items(items) {
                        match inner {
                            Item::Function {
                                receiver,
                                name: fname,
                                type_params,
                                params,
                                body,
                                is_pub,
                            } => {
                                out.push(Item::Function {
                                    receiver,
                                    name: format!("{}_{}", name, fname),
                                    type_params,
                                    params,
                                    body,
                                    is_pub,
                                });
                            }
                            Item::Struct {
                                name: sname,
                                fields,
                                is_pub,
                            } => {
                                out.push(Item::Struct {
                                    name: format!("{}_{}", name, sname),
                                    fields,
                                    is_pub,
                                });
                            }
                            Item::Enum {
                                name: ename,
                                variants,
                                is_pub,
                            } => {
                                out.push(Item::Enum {
                                    name: format!("{}_{}", name, ename),
                                    variants,
                                    is_pub,
                                });
                            }
                            other => out.push(other),
                        }
                    }
                }
                Item::Impl {
                    type_name, methods, ..
                } => {
                    for m in methods {
                        if let Item::Function {
                            name: fname,
                            type_params,
                            params,
                            body,
                            is_pub,
                            ..
                        } = m
                        {
                            out.push(Item::Function {
                                receiver: Some(type_name.clone()),
                                name: fname.clone(),
                                type_params: type_params.clone(),
                                params: params.clone(),
                                body: body.clone(),
                                is_pub: *is_pub,
                            });
                        }
                    }
                }
                Item::Trait { .. } => {}
                other => out.push(other.clone()),
            }
        }
        out
    }

    pub fn generate(&mut self, program: &Program) -> Result<String, Vec<String>> {
        self.emit_preamble();

        let flat = Self::flatten_items(&program.items);
        // Register structs, enums & functions first
        for item in &flat {
            match item {
                Item::Struct {
                    name,
                    fields,
                    is_pub: _,
                } => {
                    self.structs.insert(name.clone(), fields.clone());
                    // %Point = type { i64, i64, ... }
                    let fields_ir = fields.iter().map(|_| "i64").collect::<Vec<_>>().join(", ");
                    let _ = writeln!(self.types_ir, "%{} = type {{ {} }}", name, fields_ir);
                }
                Item::Enum {
                    name,
                    variants,
                    is_pub: _,
                } => {
                    let vs: Vec<(String, usize)> = variants
                        .iter()
                        .map(|v| (v.name.clone(), v.fields.len()))
                        .collect();
                    self.enums.insert(name.clone(), vs);
                }
                Item::Module { .. } => {}
                Item::Trait { .. } => {}
                Item::Impl { .. } => {}
                Item::Function {
                    receiver,
                    name,
                    params,
                    ..
                } => {
                    let full_name = if let Some(recv) = receiver {
                        format!("{}_{}", recv, name)
                    } else {
                        name.clone()
                    };
                    self.functions.insert(full_name.clone(), params.len());
                    let mut is_ptr = Vec::new();
                    if receiver.is_some() {
                        is_ptr.push(true); // self
                        for p in params.iter().skip(1) {
                            let ptr = match p.ty_annotation.as_deref() {
                                Some("String") | Some("string") => true,
                                Some("Number") | Some("number") | Some("Bool") | Some("bool")
                                | None => false,
                                Some(name)
                                    if name.len() == 1
                                        && name.chars().next().unwrap().is_uppercase() =>
                                {
                                    false // generic type param → i64 for MVP
                                }
                                Some(_) => true, // struct/enum-like
                            };
                            is_ptr.push(ptr);
                        }
                        if params.is_empty() {
                            is_ptr = vec![true];
                        } else if params.len() == 1 {
                            is_ptr = vec![true];
                        }
                    } else {
                        for p in params {
                            let ptr = match p.ty_annotation.as_deref() {
                                Some("String") | Some("string") => true,
                                Some("Number") | Some("number") | Some("Bool") | Some("bool")
                                | None => false,
                                Some(name)
                                    if name.len() == 1
                                        && name.chars().next().unwrap().is_uppercase() =>
                                {
                                    false // generic type param → i64 for MVP
                                }
                                Some(_) => true, // struct/enum-like
                            };
                            is_ptr.push(ptr);
                        }
                    }
                    self.function_param_is_ptr.insert(full_name, is_ptr);
                }
            }
        }

        let mut funcs = String::new();
        for item in &flat {
            if let Item::Function {
                receiver,
                name,
                params,
                body,
                type_params: _,
                is_pub: _,
            } = item
            {
                let full_name = if let Some(recv) = receiver {
                    format!("{}_{}", recv, name)
                } else {
                    name.clone()
                };
                self.emit_function(&full_name, params, body, receiver.is_some());
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
        let _ = writeln!(self.preamble, "target triple = \"{}\"", self.target_triple);
        self.preamble.push('\n');
        self.preamble.push_str("declare i32 @printf(ptr, ...)\n");
        self.preamble.push_str("declare ptr @malloc(i64)\n");
        self.preamble.push_str("declare ptr @strcpy(ptr, ptr)\n");
        self.preamble.push_str("declare ptr @strcat(ptr, ptr)\n");
        self.preamble.push_str("declare i64 @strlen(ptr)\n");
        self.preamble
            .push_str("declare i32 @snprintf(ptr, i64, ptr, ...)\n");
        self.preamble.push_str("declare void @free(ptr)\n");
        self.preamble.push_str("declare ptr @fopen(ptr, ptr)\n");
        self.preamble
            .push_str("declare i64 @fread(ptr, i64, i64, ptr)\n");
        self.preamble
            .push_str("declare i64 @fwrite(ptr, i64, i64, ptr)\n");
        self.preamble.push_str("declare i32 @fclose(ptr)\n");
        self.preamble
            .push_str("declare i32 @fseek(ptr, i64, i32)\n");
        self.preamble.push_str("declare i64 @ftell(ptr)\n");
        self.preamble.push_str("declare void @rewind(ptr)\n");
        // libm (linked via clang -lm)
        self.preamble.push_str("declare double @fabs(double)\n");
        self.preamble
            .push_str("declare double @fmin(double, double)\n");
        self.preamble
            .push_str("declare double @fmax(double, double)\n");
        self.preamble
            .push_str("declare double @pow(double, double)\n");
        self.preamble.push_str("declare double @sqrt(double)\n");
        self.preamble.push_str("declare double @floor(double)\n");
        self.preamble.push_str("declare double @ceil(double)\n");
        self.preamble.push_str("declare double @round(double)\n");
        self.preamble.push_str("declare double @sin(double)\n");
        self.preamble.push_str("declare double @cos(double)\n");
        self.preamble.push_str("declare double @tan(double)\n");
        self.preamble.push_str("declare double @log(double)\n");
        self.preamble.push_str("declare double @exp(double)\n\n");
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

    fn emit_function(&mut self, name: &str, params: &[Param], body: &Block, is_method: bool) {
        self.vars.clear();
        self.temp = 0;
        self.label = 0;
        self.body.clear();

        let is_main = name == "main";
        self.current_is_main = is_main;
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
                let ptr = format!("%{}.addr", first.name);
                let _ = writeln!(self.body, "  {} = alloca ptr, align 8", ptr);
                let _ = writeln!(self.body, "  store ptr %arg0, ptr {}, align 8", ptr);
                self.vars.insert(first.name.clone(), (ptr, VarKind::Struct));
            }
            for (i, p) in params.iter().enumerate().skip(1) {
                let ptr = format!("%{}.addr", p.name);
                let _ = writeln!(self.body, "  {} = alloca i64, align 8", ptr);
                let _ = writeln!(self.body, "  store i64 %arg{}, ptr {}, align 8", i, ptr);
                self.vars.insert(p.name.clone(), (ptr, VarKind::Number));
            }
        } else {
            let is_ptrs = self
                .function_param_is_ptr
                .get(name)
                .cloned()
                .unwrap_or_else(|| params.iter().map(|_| false).collect());
            let param_list: Vec<String> = params
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    if is_ptrs.get(i).copied().unwrap_or(false) {
                        format!("ptr %arg{}", i)
                    } else {
                        format!("i64 %arg{}", i)
                    }
                })
                .collect();
            let _ = writeln!(
                self.body,
                "define i64 @{}({}) {{",
                name,
                param_list.join(", ")
            );
            self.body.push_str("entry:\n");
            for (i, p) in params.iter().enumerate() {
                let is_ptr = is_ptrs.get(i).copied().unwrap_or(false);
                let slot = format!("%{}.addr", p.name);
                if is_ptr {
                    let _ = writeln!(self.body, "  {} = alloca ptr, align 8", slot);
                    let _ = writeln!(self.body, "  store ptr %arg{}, ptr {}, align 8", i, slot);
                    let kind = match p.ty_annotation.as_deref() {
                        Some("String") | Some("string") => VarKind::String,
                        _ => VarKind::Struct,
                    };
                    self.vars.insert(p.name.clone(), (slot, kind));
                } else {
                    let _ = writeln!(self.body, "  {} = alloca i64, align 8", slot);
                    let _ = writeln!(self.body, "  store i64 %arg{}, ptr {}, align 8", i, slot);
                    self.vars.insert(p.name.clone(), (slot, VarKind::Number));
                }
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
        for node in &block.statements {
            self.emit_stmt(&node.stmt);
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
                        VarKind::Number | VarKind::Enum => {
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
                        VarKind::Number | VarKind::Enum => {
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
                        VarKind::Number | VarKind::Enum => {
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
                    if kind == VarKind::Number || kind == VarKind::Enum {
                        let t = self.fresh();
                        let _ = writeln!(self.body, "  {} = trunc i64 {} to i32", t, v);
                        let _ = writeln!(self.body, "  ret i32 {}", t);
                    } else {
                        let _ = writeln!(self.body, "  ret i32 0");
                    }
                } else {
                    match kind {
                        VarKind::Number | VarKind::Enum => {
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
            Stmt::Match { expr, arms } => {
                let (val, kind) = self.emit_expr(expr);
                if kind != VarKind::Enum && kind != VarKind::Number {
                    self.errors.push("codegen: match on non-enum value".into());
                }
                // Extract tag: val >> 32
                let tag = self.fresh();
                let _ = writeln!(self.body, "  {} = lshr i64 {}, 32", tag, val);
                let end_label = self.fresh_label("match.end");
                let mut arm_labels = Vec::new();
                for (i, _) in arms.iter().enumerate() {
                    arm_labels.push(self.fresh_label(&format!("match.arm{}", i)));
                }
                let default_label = self.fresh_label("match.default");
                // Build switch
                let _ = write!(
                    self.body,
                    "  switch i64 {}, label %{} [",
                    tag, default_label
                );
                for (i, arm) in arms.iter().enumerate() {
                    if let Pattern::Variant {
                        enum_name, variant, ..
                    } = &arm.pattern
                    {
                        if let Some(variants) = self.enums.get(enum_name) {
                            if let Some((idx, _)) =
                                variants.iter().enumerate().find(|(_, (v, _))| v == variant)
                            {
                                let _ = write!(self.body, " i64 {}, label %{}", idx, arm_labels[i]);
                            }
                        }
                    }
                }
                let _ = writeln!(self.body, " ]");
                // Arm bodies
                for (i, arm) in arms.iter().enumerate() {
                    let _ = writeln!(self.body, "{}:", arm_labels[i]);
                    // Bind payload if needed: payload = val & 0xFFFFFFFF (sign-extend from lower 32)
                    if let Pattern::Variant {
                        binding: Some(b), ..
                    } = &arm.pattern
                    {
                        let payload = self.fresh();
                        let _ = writeln!(self.body, "  {} = and i64 {}, 4294967295", payload, val);
                        let ptr = self.fresh(); // unique stack slot per arm
                        let _ = writeln!(self.body, "  {} = alloca i64, align 8", ptr);
                        let _ =
                            writeln!(self.body, "  store i64 {}, ptr {}, align 8", payload, ptr);
                        self.vars.insert(b.clone(), (ptr, VarKind::Number));
                    }
                    self.emit_block(&arm.body);
                    let _ = writeln!(self.body, "  br label %{}", end_label);
                }
                let _ = writeln!(self.body, "{}:", default_label);
                let _ = writeln!(self.body, "  br label %{}", end_label);
                let _ = writeln!(self.body, "{}:", end_label);
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
            if rkind == VarKind::Number || rkind == VarKind::Enum {
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
            VarKind::Number | VarKind::Enum => {
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
                        VarKind::Number | VarKind::Enum => {
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
                    // Built-in math (stdlib) — always available
                    let builtin = name.strip_prefix("std_").unwrap_or(name.as_str());
                    let is_math = matches!(
                        builtin,
                        "abs"
                            | "min"
                            | "max"
                            | "pow"
                            | "sqrt"
                            | "floor"
                            | "ceil"
                            | "round"
                            | "sin"
                            | "cos"
                            | "tan"
                            | "log"
                            | "exp"
                    );
                    if is_math {
                        let mut fargs = Vec::new();
                        for a in args {
                            let (v, _) = self.emit_expr(a);
                            let d = self.fresh();
                            let _ = writeln!(self.body, "  {} = sitofp i64 {} to double", d, v);
                            fargs.push(d);
                        }
                        let (c_name, _nargs) = match builtin {
                            "abs" => ("fabs", 1usize),
                            "min" => ("fmin", 2),
                            "max" => ("fmax", 2),
                            "pow" => ("pow", 2),
                            "sqrt" => ("sqrt", 1),
                            "floor" => ("floor", 1),
                            "ceil" => ("ceil", 1),
                            "round" => ("round", 1),
                            "sin" => ("sin", 1),
                            "cos" => ("cos", 1),
                            "tan" => ("tan", 1),
                            "log" => ("log", 1),
                            "exp" => ("exp", 1),
                            _ => ("fabs", 1),
                        };
                        if fargs.len() != _nargs {
                            self.errors.push(format!(
                                "codegen: {} expects {} args, found {}",
                                builtin,
                                _nargs,
                                fargs.len()
                            ));
                            return ("0".into(), VarKind::Number);
                        }
                        let args_ir = fargs
                            .iter()
                            .map(|a| format!("double {}", a))
                            .collect::<Vec<_>>()
                            .join(", ");
                        let fd = self.fresh();
                        let _ =
                            writeln!(self.body, "  {} = call double @{}({})", fd, c_name, args_ir);
                        let res = self.fresh();
                        let _ = writeln!(self.body, "  {} = fptosi double {} to i64", res, fd);
                        return (res, VarKind::Number);
                    }
                    let file_builtin = name.strip_prefix("std_").unwrap_or(name.as_str());
                    if file_builtin == "read_file" {
                        if args.len() != 1 {
                            self.errors.push("codegen: read_file expects 1 arg".into());
                            return ("0".into(), VarKind::String);
                        }
                        let (path, pk) = self.emit_expr(&args[0]);
                        let path_s = self.ensure_string(path, pk);
                        let mode = self.intern_string("rb");
                        let fp = self.fresh();
                        let _ = writeln!(
                            self.body,
                            "  {} = call ptr @fopen(ptr {}, ptr {})",
                            fp, path_s, mode
                        );
                        let empty = self.intern_string("");
                        let isnull = self.fresh();
                        let _ = writeln!(self.body, "  {} = icmp eq ptr {}, null", isnull, fp);
                        let then_l = self.fresh_label("rf.then");
                        let else_l = self.fresh_label("rf.else");
                        let end_l = self.fresh_label("rf.end");
                        let res_slot = self.fresh();
                        let _ = writeln!(self.body, "  {} = alloca ptr, align 8", res_slot);
                        let _ = writeln!(
                            self.body,
                            "  br i1 {}, label %{}, label %{}",
                            isnull, then_l, else_l
                        );
                        let _ = writeln!(self.body, "{}:", then_l);
                        let _ = writeln!(
                            self.body,
                            "  store ptr {}, ptr {}, align 8",
                            empty, res_slot
                        );
                        let _ = writeln!(self.body, "  br label %{}", end_l);
                        let _ = writeln!(self.body, "{}:", else_l);
                        let _ = writeln!(self.body, "  call i32 @fseek(ptr {}, i64 0, i32 2)", fp);
                        let sz = self.fresh();
                        let _ = writeln!(self.body, "  {} = call i64 @ftell(ptr {})", sz, fp);
                        let _ = writeln!(self.body, "  call void @rewind(ptr {})", fp);
                        let sz1 = self.fresh();
                        let _ = writeln!(self.body, "  {} = add i64 {}, 1", sz1, sz);
                        let buf = self.fresh();
                        let _ = writeln!(self.body, "  {} = call ptr @malloc(i64 {})", buf, sz1);
                        let _ = writeln!(
                            self.body,
                            "  call i64 @fread(ptr {}, i64 1, i64 {}, ptr {})",
                            buf, sz, fp
                        );
                        let nul = self.fresh();
                        let _ = writeln!(
                            self.body,
                            "  {} = getelementptr inbounds i8, ptr {}, i64 {}",
                            nul, buf, sz
                        );
                        let _ = writeln!(self.body, "  store i8 0, ptr {}, align 1", nul);
                        let _ = writeln!(self.body, "  call i32 @fclose(ptr {})", fp);
                        let _ =
                            writeln!(self.body, "  store ptr {}, ptr {}, align 8", buf, res_slot);
                        let _ = writeln!(self.body, "  br label %{}", end_l);
                        let _ = writeln!(self.body, "{}:", end_l);
                        let out = self.fresh();
                        let _ =
                            writeln!(self.body, "  {} = load ptr, ptr {}, align 8", out, res_slot);
                        return (out, VarKind::String);
                    }
                    if file_builtin == "write_file" {
                        if args.len() != 2 {
                            self.errors
                                .push("codegen: write_file expects 2 args".into());
                            return ("0".into(), VarKind::Number);
                        }
                        let (path, pk) = self.emit_expr(&args[0]);
                        let path_s = self.ensure_string(path, pk);
                        let (content, ck) = self.emit_expr(&args[1]);
                        let content_s = self.ensure_string(content, ck);
                        let mode = self.intern_string("wb");
                        let fp = self.fresh();
                        let _ = writeln!(
                            self.body,
                            "  {} = call ptr @fopen(ptr {}, ptr {})",
                            fp, path_s, mode
                        );
                        let isnull = self.fresh();
                        let _ = writeln!(self.body, "  {} = icmp eq ptr {}, null", isnull, fp);
                        let fail_l = self.fresh_label("wf.fail");
                        let ok_l = self.fresh_label("wf.ok");
                        let end_l = self.fresh_label("wf.end");
                        let res_slot = self.fresh();
                        let _ = writeln!(self.body, "  {} = alloca i64, align 8", res_slot);
                        let _ = writeln!(
                            self.body,
                            "  br i1 {}, label %{}, label %{}",
                            isnull, fail_l, ok_l
                        );
                        let _ = writeln!(self.body, "{}:", fail_l);
                        let _ = writeln!(self.body, "  store i64 0, ptr {}, align 8", res_slot);
                        let _ = writeln!(self.body, "  br label %{}", end_l);
                        let _ = writeln!(self.body, "{}:", ok_l);
                        let len = self.fresh();
                        let _ =
                            writeln!(self.body, "  {} = call i64 @strlen(ptr {})", len, content_s);
                        let n = self.fresh();
                        let _ = writeln!(
                            self.body,
                            "  {} = call i64 @fwrite(ptr {}, i64 1, i64 {}, ptr {})",
                            n, content_s, len, fp
                        );
                        let _ = writeln!(self.body, "  call i32 @fclose(ptr {})", fp);
                        let _ = writeln!(self.body, "  store i64 {}, ptr {}, align 8", n, res_slot);
                        let _ = writeln!(self.body, "  br label %{}", end_l);
                        let _ = writeln!(self.body, "{}:", end_l);
                        let out = self.fresh();
                        let _ =
                            writeln!(self.body, "  {} = load i64, ptr {}, align 8", out, res_slot);
                        return (out, VarKind::Number);
                    }
                    if file_builtin == "file_exists" {
                        if args.len() != 1 {
                            self.errors
                                .push("codegen: file_exists expects 1 arg".into());
                            return ("0".into(), VarKind::Number);
                        }
                        let (path, pk) = self.emit_expr(&args[0]);
                        let path_s = self.ensure_string(path, pk);
                        let mode = self.intern_string("rb");
                        let fp = self.fresh();
                        let _ = writeln!(
                            self.body,
                            "  {} = call ptr @fopen(ptr {}, ptr {})",
                            fp, path_s, mode
                        );
                        let isnull = self.fresh();
                        let _ = writeln!(self.body, "  {} = icmp eq ptr {}, null", isnull, fp);
                        let yes_l = self.fresh_label("fe.yes");
                        let no_l = self.fresh_label("fe.no");
                        let end_l = self.fresh_label("fe.end");
                        let res_slot = self.fresh();
                        let _ = writeln!(self.body, "  {} = alloca i64, align 8", res_slot);
                        let _ = writeln!(
                            self.body,
                            "  br i1 {}, label %{}, label %{}",
                            isnull, no_l, yes_l
                        );
                        let _ = writeln!(self.body, "{}:", no_l);
                        let _ = writeln!(self.body, "  store i64 0, ptr {}, align 8", res_slot);
                        let _ = writeln!(self.body, "  br label %{}", end_l);
                        let _ = writeln!(self.body, "{}:", yes_l);
                        let _ = writeln!(self.body, "  call i32 @fclose(ptr {})", fp);
                        let _ = writeln!(self.body, "  store i64 1, ptr {}, align 8", res_slot);
                        let _ = writeln!(self.body, "  br label %{}", end_l);
                        let _ = writeln!(self.body, "{}:", end_l);
                        let out = self.fresh();
                        let _ =
                            writeln!(self.body, "  {} = load i64, ptr {}, align 8", out, res_slot);
                        return (out, VarKind::Number);
                    }
                    let list_builtin = name.strip_prefix("std_").unwrap_or(name.as_str());
                    if list_builtin == "list_len" {
                        if args.len() != 1 {
                            self.errors.push("codegen: list_len expects 1 arg".into());
                            return ("0".into(), VarKind::Number);
                        }
                        let (lst, lk) = self.emit_expr(&args[0]);
                        if lk != VarKind::List {
                            self.errors.push("codegen: list_len expects List".into());
                        }
                        let loaded = self.fresh();
                        let _ =
                            writeln!(self.body, "  {} = load i64, ptr {}, align 8", loaded, lst);
                        return (loaded, VarKind::Number);
                    }
                    if list_builtin == "list_get" {
                        if args.len() != 2 {
                            self.errors.push("codegen: list_get expects 2 args".into());
                            return ("0".into(), VarKind::Number);
                        }
                        let (lst, lk) = self.emit_expr(&args[0]);
                        let (idx, _) = self.emit_expr(&args[1]);
                        if lk != VarKind::List {
                            self.errors.push("codegen: list_get expects List".into());
                        }
                        let off = self.fresh();
                        let _ = writeln!(self.body, "  {} = add i64 {}, 1", off, idx);
                        let ep = self.fresh();
                        let _ = writeln!(
                            self.body,
                            "  {} = getelementptr inbounds i64, ptr {}, i64 {}",
                            ep, lst, off
                        );
                        let loaded = self.fresh();
                        let _ = writeln!(self.body, "  {} = load i64, ptr {}, align 8", loaded, ep);
                        return (loaded, VarKind::Number);
                    }
                    if list_builtin == "str_len" {
                        if args.len() != 1 {
                            self.errors.push("codegen: str_len expects 1 arg".into());
                            return ("0".into(), VarKind::Number);
                        }
                        let (s, sk) = self.emit_expr(&args[0]);
                        let sp = self.ensure_string(s, sk);
                        let n = self.fresh();
                        let _ = writeln!(self.body, "  {} = call i64 @strlen(ptr {})", n, sp);
                        return (n, VarKind::Number);
                    }
                    if list_builtin == "list_sum" {
                        if args.len() != 1 {
                            self.errors.push("codegen: list_sum expects 1 arg".into());
                            return ("0".into(), VarKind::Number);
                        }
                        let (lst, lk) = self.emit_expr(&args[0]);
                        if lk != VarKind::List {
                            self.errors.push("codegen: list_sum expects List".into());
                        }
                        let len = self.fresh();
                        let _ = writeln!(self.body, "  {} = load i64, ptr {}, align 8", len, lst);
                        let idx = self.fresh();
                        let _ = writeln!(self.body, "  {} = alloca i64, align 8", idx);
                        let _ = writeln!(self.body, "  store i64 0, ptr {}, align 8", idx);
                        let acc = self.fresh();
                        let _ = writeln!(self.body, "  {} = alloca i64, align 8", acc);
                        let _ = writeln!(self.body, "  store i64 0, ptr {}, align 8", acc);
                        let cond = self.fresh_label("lsum.cond");
                        let body = self.fresh_label("lsum.body");
                        let end_l = self.fresh_label("lsum.end");
                        let _ = writeln!(self.body, "  br label %{}", cond);
                        let _ = writeln!(self.body, "{}:", cond);
                        let cur = self.fresh();
                        let _ = writeln!(self.body, "  {} = load i64, ptr {}, align 8", cur, idx);
                        let cmp = self.fresh();
                        let _ = writeln!(self.body, "  {} = icmp slt i64 {}, {}", cmp, cur, len);
                        let _ = writeln!(
                            self.body,
                            "  br i1 {}, label %{}, label %{}",
                            cmp, body, end_l
                        );
                        let _ = writeln!(self.body, "{}:", body);
                        let off = self.fresh();
                        let _ = writeln!(self.body, "  {} = add i64 {}, 1", off, cur);
                        let ep = self.fresh();
                        let _ = writeln!(
                            self.body,
                            "  {} = getelementptr inbounds i64, ptr {}, i64 {}",
                            ep, lst, off
                        );
                        let elem = self.fresh();
                        let _ = writeln!(self.body, "  {} = load i64, ptr {}, align 8", elem, ep);
                        let a = self.fresh();
                        let _ = writeln!(self.body, "  {} = load i64, ptr {}, align 8", a, acc);
                        let s = self.fresh();
                        let _ = writeln!(self.body, "  {} = add i64 {}, {}", s, a, elem);
                        let _ = writeln!(self.body, "  store i64 {}, ptr {}, align 8", s, acc);
                        let n = self.fresh();
                        let _ = writeln!(self.body, "  {} = add i64 {}, 1", n, cur);
                        let _ = writeln!(self.body, "  store i64 {}, ptr {}, align 8", n, idx);
                        let _ = writeln!(self.body, "  br label %{}", cond);
                        let _ = writeln!(self.body, "{}:", end_l);
                        let out = self.fresh();
                        let _ = writeln!(self.body, "  {} = load i64, ptr {}, align 8", out, acc);
                        return (out, VarKind::Number);
                    }

                    if list_builtin == "list_max" || list_builtin == "list_min" {
                        if args.len() != 1 {
                            self.errors
                                .push("codegen: list_max/min expects 1 arg".into());
                            return ("0".into(), VarKind::Number);
                        }
                        let (lst, lk) = self.emit_expr(&args[0]);
                        let is_max = list_builtin == "list_max";
                        let len = self.fresh();
                        let _ = writeln!(self.body, "  {} = load i64, ptr {}, align 8", len, lst);
                        let idx = self.fresh();
                        let _ = writeln!(self.body, "  {} = alloca i64, align 8", idx);
                        let _ = writeln!(self.body, "  store i64 0, ptr {}, align 8", idx);
                        let best = self.fresh();
                        let _ = writeln!(self.body, "  {} = alloca i64, align 8", best);
                        // init best from first element or 0
                        let ep0 = self.fresh();
                        let _ = writeln!(
                            self.body,
                            "  {} = getelementptr inbounds i64, ptr {}, i64 1",
                            ep0, lst
                        );
                        let e0 = self.fresh();
                        let _ = writeln!(self.body, "  {} = load i64, ptr {}, align 8", e0, ep0);
                        let _ = writeln!(self.body, "  store i64 {}, ptr {}, align 8", e0, best);
                        let _ = writeln!(self.body, "  store i64 1, ptr {}, align 8", idx);
                        let cond = self.fresh_label("lmm.cond");
                        let body = self.fresh_label("lmm.body");
                        let end_l = self.fresh_label("lmm.end");
                        let _ = writeln!(self.body, "  br label %{}", cond);
                        let _ = writeln!(self.body, "{}:", cond);
                        let cur = self.fresh();
                        let _ = writeln!(self.body, "  {} = load i64, ptr {}, align 8", cur, idx);
                        let cmp = self.fresh();
                        let _ = writeln!(self.body, "  {} = icmp slt i64 {}, {}", cmp, cur, len);
                        let _ = writeln!(
                            self.body,
                            "  br i1 {}, label %{}, label %{}",
                            cmp, body, end_l
                        );
                        let _ = writeln!(self.body, "{}:", body);
                        let off = self.fresh();
                        let _ = writeln!(self.body, "  {} = add i64 {}, 1", off, cur);
                        let ep = self.fresh();
                        let _ = writeln!(
                            self.body,
                            "  {} = getelementptr inbounds i64, ptr {}, i64 {}",
                            ep, lst, off
                        );
                        let elem = self.fresh();
                        let _ = writeln!(self.body, "  {} = load i64, ptr {}, align 8", elem, ep);
                        let b = self.fresh();
                        let _ = writeln!(self.body, "  {} = load i64, ptr {}, align 8", b, best);
                        let better = self.fresh();
                        if is_max {
                            let _ =
                                writeln!(self.body, "  {} = icmp sgt i64 {}, {}", better, elem, b);
                        } else {
                            let _ =
                                writeln!(self.body, "  {} = icmp slt i64 {}, {}", better, elem, b);
                        }
                        let upd = self.fresh_label("lmm.upd");
                        let cont = self.fresh_label("lmm.cont");
                        let _ = writeln!(
                            self.body,
                            "  br i1 {}, label %{}, label %{}",
                            better, upd, cont
                        );
                        let _ = writeln!(self.body, "{}:", upd);
                        let _ = writeln!(self.body, "  store i64 {}, ptr {}, align 8", elem, best);
                        let _ = writeln!(self.body, "  br label %{}", cont);
                        let _ = writeln!(self.body, "{}:", cont);
                        let n = self.fresh();
                        let _ = writeln!(self.body, "  {} = add i64 {}, 1", n, cur);
                        let _ = writeln!(self.body, "  store i64 {}, ptr {}, align 8", n, idx);
                        let _ = writeln!(self.body, "  br label %{}", cond);
                        let _ = writeln!(self.body, "{}:", end_l);
                        let out = self.fresh();
                        let _ = writeln!(self.body, "  {} = load i64, ptr {}, align 8", out, best);
                        let _ = lk;
                        return (out, VarKind::Number);
                    }
                    if list_builtin == "str_is_empty" {
                        if args.len() != 1 {
                            self.errors
                                .push("codegen: str_is_empty expects 1 arg".into());
                            return ("0".into(), VarKind::Number);
                        }
                        let (s, sk) = self.emit_expr(&args[0]);
                        let sp = self.ensure_string(s, sk);
                        let n = self.fresh();
                        let _ = writeln!(self.body, "  {} = call i64 @strlen(ptr {})", n, sp);
                        let z = self.fresh();
                        let _ = writeln!(self.body, "  {} = icmp eq i64 {}, 0", z, n);
                        let out = self.fresh();
                        let _ = writeln!(self.body, "  {} = zext i1 {} to i64", out, z);
                        return (out, VarKind::Number);
                    }
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
                        // Built-in math stdlib → libm (values are i64, convert via sitofp/fptosi)
                        let builtin = name.strip_prefix("std_").unwrap_or(name);
                        let is_math = matches!(
                            builtin,
                            "abs"
                                | "min"
                                | "max"
                                | "pow"
                                | "sqrt"
                                | "floor"
                                | "ceil"
                                | "round"
                                | "sin"
                                | "cos"
                                | "tan"
                                | "log"
                                | "exp"
                        );
                        if is_math {
                            let mut fargs = Vec::new();
                            for a in args {
                                let (v, _) = self.emit_expr(a);
                                let d = self.fresh();
                                let _ = writeln!(self.body, "  {} = sitofp i64 {} to double", d, v);
                                fargs.push(d);
                            }
                            let (c_name, _nargs) = match builtin {
                                "abs" => ("fabs", 1),
                                "min" => ("fmin", 2),
                                "max" => ("fmax", 2),
                                "pow" => ("pow", 2),
                                "sqrt" => ("sqrt", 1),
                                "floor" => ("floor", 1),
                                "ceil" => ("ceil", 1),
                                "round" => ("round", 1),
                                "sin" => ("sin", 1),
                                "cos" => ("cos", 1),
                                "tan" => ("tan", 1),
                                "log" => ("log", 1),
                                "exp" => ("exp", 1),
                                _ => ("fabs", 1),
                            };
                            if fargs.len() != _nargs {
                                self.errors
                                    .push(format!("codegen: {} expects {} args", builtin, _nargs));
                                return ("0".into(), VarKind::Number);
                            }
                            let args_ir = fargs
                                .iter()
                                .map(|a| format!("double {}", a))
                                .collect::<Vec<_>>()
                                .join(", ");
                            let fd = self.fresh();
                            let _ = writeln!(
                                self.body,
                                "  {} = call double @{}({})",
                                fd, c_name, args_ir
                            );
                            let res = self.fresh();
                            let _ = writeln!(self.body, "  {} = fptosi double {} to i64", res, fd);
                            return (res, VarKind::Number);
                        }
                        let is_ptrs = self
                            .function_param_is_ptr
                            .get(name)
                            .cloned()
                            .unwrap_or_default();
                        let mut arg_parts = Vec::new();
                        for (i, a) in args.iter().enumerate() {
                            let (v, k) = self.emit_expr(a);
                            let want_ptr = is_ptrs.get(i).copied().unwrap_or(false);
                            if want_ptr {
                                arg_parts.push(format!("ptr {}", v));
                            } else if k == VarKind::Number || k == VarKind::Enum {
                                arg_parts.push(format!("i64 {}", v));
                            } else {
                                self.errors.push(format!(
                                    "codegen: argument type mismatch in call to {}",
                                    name
                                ));
                                arg_parts.push(format!("i64 0"));
                            }
                        }
                        let args_ir = arg_parts.join(", ");
                        let res = self.fresh();
                        let _ = writeln!(self.body, "  {} = call i64 @{}({})", res, name, args_ir);
                        return (res, VarKind::Number);
                    }
                }
                // Module path: math.add(...) → @math_add(...)  OR std.sqrt → libm
                if let Expr::Field { object, field } = callee.as_ref() {
                    if let Expr::Ident(mod_name) = object.as_ref() {
                        let full = format!("{}_{}", mod_name, field);
                        // std.* math builtins
                        if mod_name == "std" {
                            let builtin = field.as_str();
                            let is_math = matches!(
                                builtin,
                                "abs"
                                    | "min"
                                    | "max"
                                    | "pow"
                                    | "sqrt"
                                    | "floor"
                                    | "ceil"
                                    | "round"
                                    | "sin"
                                    | "cos"
                                    | "tan"
                                    | "log"
                                    | "exp"
                            );
                            if is_math {
                                // reuse by synthesizing Ident call path via recursive pattern
                                let mut fargs = Vec::new();
                                for a in args {
                                    let (v, _) = self.emit_expr(a);
                                    let d = self.fresh();
                                    let _ =
                                        writeln!(self.body, "  {} = sitofp i64 {} to double", d, v);
                                    fargs.push(d);
                                }
                                let (c_name, _nargs) = match builtin {
                                    "abs" => ("fabs", 1usize),
                                    "min" => ("fmin", 2),
                                    "max" => ("fmax", 2),
                                    "pow" => ("pow", 2),
                                    "sqrt" => ("sqrt", 1),
                                    "floor" => ("floor", 1),
                                    "ceil" => ("ceil", 1),
                                    "round" => ("round", 1),
                                    "sin" => ("sin", 1),
                                    "cos" => ("cos", 1),
                                    "tan" => ("tan", 1),
                                    "log" => ("log", 1),
                                    "exp" => ("exp", 1),
                                    _ => ("fabs", 1),
                                };
                                let args_ir = fargs
                                    .iter()
                                    .map(|a| format!("double {}", a))
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                let fd = self.fresh();
                                let _ = writeln!(
                                    self.body,
                                    "  {} = call double @{}({})",
                                    fd, c_name, args_ir
                                );
                                let res = self.fresh();
                                let _ =
                                    writeln!(self.body, "  {} = fptosi double {} to i64", res, fd);
                                return (res, VarKind::Number);
                            }
                        }
                        if self.functions.contains_key(&full) {
                            let is_ptrs = self
                                .function_param_is_ptr
                                .get(&full)
                                .cloned()
                                .unwrap_or_default();
                            let mut arg_parts = Vec::new();
                            for (i, a) in args.iter().enumerate() {
                                let (v, k) = self.emit_expr(a);
                                let want_ptr = is_ptrs.get(i).copied().unwrap_or(false);
                                if want_ptr {
                                    arg_parts.push(format!("ptr {}", v));
                                } else {
                                    arg_parts.push(format!("i64 {}", v));
                                    let _ = k;
                                }
                            }
                            let args_ir = arg_parts.join(", ");
                            let res = self.fresh();
                            let _ =
                                writeln!(self.body, "  {} = call i64 @{}({})", res, full, args_ir);
                            return (res, VarKind::Number);
                        }
                    }
                }
                // Enum variant with payload: Option.Some(42) → (tag << 32) | payload
                if let Expr::Field { object, field } = callee.as_ref() {
                    if let Expr::Ident(enum_name) = object.as_ref() {
                        if let Some(variants) = self.enums.get(enum_name).cloned() {
                            if let Some((idx, (_, nfields))) =
                                variants.iter().enumerate().find(|(_, (v, _))| v == field)
                            {
                                if *nfields == args.len() {
                                    let mut payload_val = "0".to_string();
                                    if !args.is_empty() {
                                        let (v, _) = self.emit_expr(&args[0]);
                                        payload_val = v;
                                    }
                                    // pack: (idx << 32) | (payload & 0xFFFFFFFF)
                                    let shifted = self.fresh();
                                    let _ =
                                        writeln!(self.body, "  {} = shl i64 {}, 32", shifted, idx);
                                    let masked = self.fresh();
                                    let _ = writeln!(
                                        self.body,
                                        "  {} = and i64 {}, 4294967295",
                                        masked, payload_val
                                    );
                                    let packed = self.fresh();
                                    let _ = writeln!(
                                        self.body,
                                        "  {} = or i64 {}, {}",
                                        packed, shifted, masked
                                    );
                                    return (packed, VarKind::Enum);
                                }
                            }
                        }
                    }
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
                // Unit enum variant: Color.Red → (tag << 32)
                if let Expr::Ident(enum_name) = object.as_ref() {
                    if let Some(variants) = self.enums.get(enum_name).cloned() {
                        if let Some((idx, (_, nfields))) =
                            variants.iter().enumerate().find(|(_, (v, _))| v == field)
                        {
                            if *nfields == 0 {
                                let packed = self.fresh();
                                let _ = writeln!(self.body, "  {} = shl i64 {}, 32", packed, idx);
                                return (packed, VarKind::Enum);
                            }
                        }
                    }
                }
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
            Expr::Try(inner) => {
                // expr? — if tag is Ok/Some extract payload; else early-return 0
                let (val, kind) = self.emit_expr(inner);
                if kind != VarKind::Enum && kind != VarKind::Number {
                    self.errors.push("codegen: '?' on non-enum".into());
                    return ("0".into(), VarKind::Number);
                }
                let tag = self.fresh();
                let _ = writeln!(self.body, "  {} = lshr i64 {}, 32", tag, val);
                // Find Ok/Some tag index — search all enums for Ok or Some with payload
                let mut ok_tag: Option<usize> = None;
                for (_ename, variants) in &self.enums {
                    for (i, (v, nfields)) in variants.iter().enumerate() {
                        if (v == "Ok" || v == "Some") && *nfields > 0 {
                            ok_tag = Some(i);
                            break;
                        }
                    }
                    if ok_tag.is_some() {
                        break;
                    }
                }
                let ok_idx = ok_tag.unwrap_or(0);
                let is_ok = self.fresh();
                let _ = writeln!(self.body, "  {} = icmp eq i64 {}, {}", is_ok, tag, ok_idx);
                let ok_label = self.fresh_label("try.ok");
                let err_label = self.fresh_label("try.err");
                let cont_label = self.fresh_label("try.cont");
                let _ = writeln!(
                    self.body,
                    "  br i1 {}, label %{}, label %{}",
                    is_ok, ok_label, err_label
                );
                let _ = writeln!(self.body, "{}:", err_label);
                if self.current_is_main {
                    let _ = writeln!(self.body, "  ret i32 1");
                } else {
                    let _ = writeln!(self.body, "  ret i64 0");
                }
                let _ = writeln!(self.body, "{}:", ok_label);
                let payload = self.fresh();
                let _ = writeln!(self.body, "  {} = and i64 {}, 4294967295", payload, val);
                let _ = writeln!(self.body, "  br label %{}", cont_label);
                let _ = writeln!(self.body, "{}:", cont_label);
                // phi not needed if we only reach cont from ok; payload is defined on ok path
                // but LLVM requires dominance — use phi
                let result = self.fresh();
                let _ = writeln!(
                    self.body,
                    "  {} = phi i64 [ {}, %{} ]",
                    result, payload, ok_label
                );
                (result, VarKind::Number)
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
            VarKind::Number | VarKind::Enum => {
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

pub fn host_triple() -> String {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        return "x86_64-unknown-linux-gnu".into();
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        return "aarch64-unknown-linux-gnu".into();
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        return "x86_64-apple-darwin".into();
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        return "arm64-apple-darwin".into();
    }
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        return "x86_64-pc-windows-msvc".into();
    }
    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    {
        return "aarch64-pc-windows-msvc".into();
    }
    #[allow(unreachable_code)]
    {
        "x86_64-unknown-linux-gnu".into()
    }
}
