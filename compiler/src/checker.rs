//! Type checker + basic ownership analysis for PureLang

use std::collections::HashMap;

use crate::ast::*;
use crate::types::{Type, TypeError};

#[derive(Debug, Clone)]
struct VarInfo {
    ty: Type,
    mutable: bool,
    /// True if the value has been moved and the binding is no longer usable
    moved: bool,
    /// Active immutable borrows (field access, print, method receiver, etc.)
    shared_borrows: usize,
    /// Active exclusive mutable borrow
    exclusive_borrow: bool,
}

#[derive(Debug)]
struct Scope {
    vars: HashMap<String, VarInfo>,
}

pub struct TypeChecker {
    scopes: Vec<Scope>,
    /// Top-level functions
    functions: HashMap<String, Type>,
    /// Methods: (StructName, method_name) → Function type (first param is the receiver)
    methods: HashMap<(String, String), Type>,
    /// Struct name → field names (order matters)
    structs: HashMap<String, Vec<String>>,
    /// Enum name → list of (variant name, payload field count)
    enums: HashMap<String, Vec<(String, usize)>>,
    errors: Vec<TypeError>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            scopes: vec![Scope {
                vars: HashMap::new(),
            }],
            functions: HashMap::new(),
            methods: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            errors: Vec::new(),
        }
    }


    /// Flatten `mod name { fn foo }` into functions named `name_foo`.
    fn flatten_items(items: &[Item]) -> Vec<Item> {
        let mut out = Vec::new();
        for item in items {
            match item {
                Item::Module { name, items } => {
                    for inner in Self::flatten_items(items) {
                        match inner {
                            Item::Function {
                                receiver,
                                name: fname,
                                params,
                                body,
                            } => {
                                out.push(Item::Function {
                                    receiver,
                                    name: format!("{}_{}", name, fname),
                                    params,
                                    body,
                                });
                            }
                            Item::Struct { name: sname, fields } => {
                                out.push(Item::Struct {
                                    name: format!("{}_{}", name, sname),
                                    fields,
                                });
                            }
                            Item::Enum { name: ename, variants } => {
                                out.push(Item::Enum {
                                    name: format!("{}_{}", name, ename),
                                    variants,
                                });
                            }
                            Item::Module { .. } => {}
                        }
                    }
                }
                other => out.push(other.clone()),
            }
        }
        out
    }

    pub fn check_program(&mut self, program: &Program) -> Result<(), Vec<TypeError>> {
        let flat = Self::flatten_items(&program.items);
        // First pass: register structs, enums and function signatures
        for item in &flat {
            match item {
                Item::Struct { name, fields } => {
                    self.structs.insert(name.clone(), fields.clone());
                }
                Item::Enum { name, variants } => {
                    let vs: Vec<(String, usize)> = variants
                        .iter()
                        .map(|v| (v.name.clone(), v.fields.len()))
                        .collect();
                    self.enums.insert(name.clone(), vs);
                }
                Item::Module { .. } => {}
                Item::Function {
                    receiver,
                    name,
                    params,
                    body,
                } => {
                    let ret = Self::infer_return_type(body);
                    if let Some(recv) = receiver {
                        let mut param_tys: Vec<Type> = vec![Type::Struct(recv.clone())];
                        for p in params.iter().skip(1) {
                            param_tys.push(Self::resolve_annotation_static(p.ty_annotation.as_deref()));
                        }
                        if params.is_empty() {
                            param_tys = vec![Type::Struct(recv.clone())];
                        } else if params.len() == 1 {
                            param_tys = vec![Type::Struct(recv.clone())];
                        }
                        self.methods.insert(
                            (recv.clone(), name.clone()),
                            Type::Function {
                                params: param_tys,
                                ret: Box::new(ret),
                            },
                        );
                    } else {
                        let param_tys: Vec<Type> = params
                            .iter()
                            .map(|p| Self::resolve_annotation_static(p.ty_annotation.as_deref()))
                            .collect();
                        self.functions.insert(
                            name.clone(),
                            Type::Function {
                                params: param_tys,
                                ret: Box::new(ret),
                            },
                        );
                    }
                }
            }
        }

        // Second pass: check bodies
        for item in &flat {
            self.check_item(item);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    fn error(&mut self, msg: impl Into<String>) {
        self.errors.push(TypeError::new(msg));
    }

    fn push_scope(&mut self) {
        self.scopes.push(Scope {
            vars: HashMap::new(),
        });
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn lookup(&self, name: &str) -> Option<&VarInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.vars.get(name) {
                return Some(info);
            }
        }
        None
    }

    fn lookup_mut(&mut self, name: &str) -> Option<&mut VarInfo> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.vars.contains_key(name) {
                return scope.vars.get_mut(name);
            }
        }
        None
    }

    fn declare(&mut self, name: &str, ty: Type, mutable: bool) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.vars.insert(
                name.to_string(),
                VarInfo {
                    ty,
                    mutable,
                    moved: false,
                    shared_borrows: 0,
                    exclusive_borrow: false,
                },
            );
        }
    }

    /// Use a binding by value (may move non-Copy types).
    fn use_by_value(&mut self, name: &str) -> Type {
        if let Some(ty) = self.functions.get(name) {
            return ty.clone();
        }
        match self.lookup(name) {
            None => {
                self.error(format!("Undefined variable '{}'", name));
                Type::Unknown
            }
            Some(info) => {
                if info.moved {
                    self.error(format!("Use of moved value '{}'", name));
                    return Type::Unknown;
                }
                if info.shared_borrows > 0 || info.exclusive_borrow {
                    self.error(format!(
                        "Cannot move '{}': value is currently borrowed",
                        name
                    ));
                    return Type::Unknown;
                }
                let ty = info.ty.clone();
                if ty.is_move_type() {
                    if let Some(info) = self.lookup_mut(name) {
                        info.moved = true;
                    }
                }
                ty
            }
        }
    }

    /// Use a binding by shared (immutable) borrow — does not move.
    fn use_by_ref(&mut self, name: &str) -> Type {
        if let Some(ty) = self.functions.get(name) {
            return ty.clone();
        }
        match self.lookup(name) {
            None => {
                self.error(format!("Undefined variable '{}'", name));
                Type::Unknown
            }
            Some(info) => {
                if info.moved {
                    self.error(format!("Use of moved value '{}'", name));
                    return Type::Unknown;
                }
                if info.exclusive_borrow {
                    self.error(format!(
                        "Cannot borrow '{}': already mutably borrowed",
                        name
                    ));
                    return Type::Unknown;
                }
                let ty = info.ty.clone();
                // Track shared borrow (released at end of statement in MVP — we just check conflicts)
                if let Some(info) = self.lookup_mut(name) {
                    info.shared_borrows = info.shared_borrows.saturating_add(1);
                    // Immediately release for statement-level analysis (no long-lived borrows yet)
                    info.shared_borrows = info.shared_borrows.saturating_sub(1);
                }
                ty
            }
        }
    }

    fn resolve_annotation_static(ann: Option<&str>) -> Type {
        match ann {
            None => Type::Number,
            Some("Number") | Some("number") => Type::Number,
            Some("String") | Some("string") => Type::String,
            Some("Bool") | Some("bool") => Type::Bool,
            Some(name) => Type::Struct(name.to_string()),
        }
    }

    fn resolve_annotation(&self, ann: Option<&str>) -> Type {
        match ann {
            None => Type::Number,
            Some("Number") | Some("number") => Type::Number,
            Some("String") | Some("string") => Type::String,
            Some("Bool") | Some("bool") => Type::Bool,
            Some(name) => {
                if self.structs.contains_key(name) {
                    Type::Struct(name.to_string())
                } else if self.enums.contains_key(name) {
                    Type::Enum(name.to_string())
                } else {
                    Type::Struct(name.to_string())
                }
            }
        }
    }

    fn infer_return_type(body: &Block) -> Type {
        fn from_block(block: &Block) -> Type {
            let mut found = Type::Void;
            for stmt in &block.statements {
                match stmt {
                    Stmt::Return(Some(expr)) => {
                        found = expr_ty_hint(expr);
                    }
                    Stmt::If {
                        then_block,
                        else_block,
                        ..
                    } => {
                        let t = from_block(then_block);
                        if t != Type::Void {
                            found = t;
                        }
                        if let Some(eb) = else_block {
                            let t = from_block(eb);
                            if t != Type::Void {
                                found = t;
                            }
                        }
                    }
                    Stmt::For { body, .. } => {
                        let t = from_block(body);
                        if t != Type::Void {
                            found = t;
                        }
                    }
                    _ => {}
                }
            }
            found
        }
        fn expr_ty_hint(expr: &Expr) -> Type {
            match expr {
                Expr::Number(_) => Type::Number,
                Expr::String(_) => Type::String,
                Expr::Bool(_) => Type::Bool,
                Expr::Binary { op, .. } => match op {
                    BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => Type::Number,
                    BinaryOp::Eq
                    | BinaryOp::NotEq
                    | BinaryOp::Lt
                    | BinaryOp::Gt
                    | BinaryOp::LtEq
                    | BinaryOp::GtEq => Type::Bool,
                },
                Expr::Field { object, .. } => {
                    if let Expr::Ident(ename) = object.as_ref() {
                        Type::Enum(ename.clone())
                    } else {
                        Type::Number
                    }
                }
                Expr::Call { callee, .. } => {
                    if let Expr::Field { object, .. } = callee.as_ref() {
                        if let Expr::Ident(ename) = object.as_ref() {
                            return Type::Enum(ename.clone());
                        }
                    }
                    Type::Number
                }
                _ => Type::Number,
            }
        }
        from_block(body)
    }

    fn check_item(&mut self, item: &Item) {
        match item {
            Item::Function {
                receiver,
                name: _,
                params,
                body,
            } => {
                self.push_scope();
                if let Some(recv) = receiver {
                    if let Some(first) = params.first() {
                        self.declare(&first.name, Type::Struct(recv.clone()), false);
                        for p in params.iter().skip(1) {
                            let ty = self.resolve_annotation(p.ty_annotation.as_deref());
                            self.declare(&p.name, ty, false);
                        }
                    }
                } else {
                    for p in params {
                        let ty = self.resolve_annotation(p.ty_annotation.as_deref());
                        self.declare(&p.name, ty, false);
                    }
                }
                self.check_block(body);
                self.pop_scope();
            }
            Item::Struct { .. } => {
                // Struct declarations are fine for now; no body to check
            }
            Item::Enum { .. } => {
                // Enum declarations are fine for now; no body to check
            }
            Item::Module { .. } => {}
        }
    }

    fn check_block(&mut self, block: &Block) {
        self.push_scope();
        for stmt in &block.statements {
            self.check_stmt(stmt);
        }
        self.pop_scope();
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let {
                mutable,
                name,
                value,
            } => {
                let value_ty = self.check_expr(value);

                // If the name already exists in current or outer scope:
                // treat as reassignment when mutable, otherwise error.
                if let Some(info) = self.lookup(name) {
                    if !info.mutable {
                        self.error(format!(
                            "Cannot assign to immutable variable '{}'. Declare it with 'mut' to allow changes.",
                            name
                        ));
                    } else if info.moved {
                        self.error(format!("Cannot assign to '{}': value was moved", name));
                    } else {
                        // Reassignment — check type compatibility (allow Unknown)
                        let existing = info.ty.clone();
                        if existing != Type::Unknown
                            && value_ty != Type::Unknown
                            && existing != value_ty
                        {
                            self.error(format!(
                                "Type mismatch assigning to '{}': expected {}, found {}",
                                name, existing, value_ty
                            ));
                        }
                    }
                    // Mark as not moved after successful assign
                    if let Some(info) = self.lookup_mut(name) {
                        info.moved = false;
                        if info.ty == Type::Unknown {
                            info.ty = value_ty;
                        }
                    }
                } else {
                    // Fresh declaration
                    self.declare(name, value_ty, *mutable);
                }
            }
            Stmt::Assign { name, value } => {
                let value_ty = self.check_expr(value);
                match self.lookup(name) {
                    None => {
                        self.error(format!("Undefined variable '{}'", name));
                    }
                    Some(info) => {
                        if !info.mutable {
                            self.error(format!("Cannot assign to immutable variable '{}'", name));
                        } else if info.moved {
                            self.error(format!("Cannot assign to '{}': value was moved", name));
                        } else if info.ty != Type::Unknown
                            && value_ty != Type::Unknown
                            && info.ty != value_ty
                        {
                            self.error(format!(
                                "Type mismatch assigning to '{}': expected {}, found {}",
                                name, info.ty, value_ty
                            ));
                        }
                    }
                }
                if let Some(info) = self.lookup_mut(name) {
                    info.moved = false;
                }
            }
            Stmt::Print(expr) => {
                // print borrows — does not take ownership
                let _ = self.check_expr_ref(expr);
            }
            Stmt::If {
                condition,
                then_block,
                else_block,
            } => {
                let cond_ty = self.check_expr(condition);
                if cond_ty != Type::Bool && cond_ty != Type::Unknown {
                    self.error(format!("If condition must be Bool, found {}", cond_ty));
                }
                self.check_block(then_block);
                if let Some(else_b) = else_block {
                    self.check_block(else_b);
                }
            }
            Stmt::For {
                var,
                iterable,
                body,
            } => {
                // Iteration borrows the list/range
                let iter_ty = self.check_expr_ref(iterable);
                let element_ty = match &iter_ty {
                    Type::Range => Type::Number,
                    Type::List(inner) => *inner.clone(),
                    Type::Unknown => Type::Unknown,
                    other => {
                        self.error(format!(
                            "Cannot iterate over type {} (expected Range or List)",
                            other
                        ));
                        Type::Unknown
                    }
                };
                self.push_scope();
                // loop variable is immutable by default
                self.declare(var, element_ty, false);
                // body is already a Block — but check_block pushes another scope.
                // That's fine (extra nested scope).
                for stmt in &body.statements {
                    self.check_stmt(stmt);
                }
                self.pop_scope();
            }
            Stmt::Return(opt) => {
                if let Some(expr) = opt {
                    let _ = self.check_expr(expr);
                }
            }
            Stmt::Match { expr, arms } => {
                let expr_ty = self.check_expr(expr);
                for arm in arms {
                    self.push_scope();
                    match &arm.pattern {
                        Pattern::Variant {
                            enum_name,
                            variant,
                            binding,
                        } => {
                            if let Type::Enum(en) = &expr_ty {
                                if en != enum_name {
                                    self.error(format!(
                                        "Match arm pattern type '{}' does not match expression type '{}'",
                                        enum_name, en
                                    ));
                                }
                            } else if expr_ty != Type::Unknown {
                                self.error(format!(
                                    "Cannot match on non-enum type {}",
                                    expr_ty
                                ));
                            }
                            if let Some(variants) = self.enums.get(enum_name) {
                                if let Some((_, nfields)) =
                                    variants.iter().find(|(v, _)| v == variant)
                                {
                                    if let Some(b) = binding {
                                        if *nfields == 0 {
                                            self.error(format!(
                                                "Variant '{}.{}' has no payload to bind",
                                                enum_name, variant
                                            ));
                                        } else {
                                            // MVP: payload is Number
                                            self.declare(b, Type::Number, false);
                                        }
                                    } else if *nfields > 0 {
                                        // allow ignoring payload
                                    }
                                } else {
                                    self.error(format!(
                                        "Unknown variant '{}.{}'",
                                        enum_name, variant
                                    ));
                                }
                            } else {
                                self.error(format!("Unknown enum '{}'", enum_name));
                            }
                        }
                        Pattern::Wildcard => {}
                    }
                    for stmt in &arm.body.statements {
                        self.check_stmt(stmt);
                    }
                    self.pop_scope();
                }
            }
            Stmt::Expr(expr) => {
                let _ = self.check_expr(expr);
            }
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Number(_) => Type::Number,
            Expr::String(_) => Type::String,
            Expr::Bool(_) => Type::Bool,
            Expr::Ident(name) => self.check_ident(name),
            Expr::Binary { left, op, right } => self.check_binary(left, *op, right),
            Expr::Unary { op, expr } => self.check_unary(*op, expr),
            Expr::Call { callee, args } => self.check_call(callee, args),
            Expr::Range { start, end } => {
                let s = self.check_expr(start);
                let e = self.check_expr(end);
                if s != Type::Number && s != Type::Unknown {
                    self.error(format!("Range start must be Number, found {}", s));
                }
                if e != Type::Number && e != Type::Unknown {
                    self.error(format!("Range end must be Number, found {}", e));
                }
                Type::Range
            }
            Expr::List(elements) => {
                if elements.is_empty() {
                    return Type::List(Box::new(Type::Unknown));
                }
                let first = self.check_expr(&elements[0]);
                for el in elements.iter().skip(1) {
                    let t = self.check_expr(el);
                    if t != first && t != Type::Unknown && first != Type::Unknown {
                        self.error(format!(
                            "List elements must have the same type: found {} and {}",
                            first, t
                        ));
                    }
                }
                Type::List(Box::new(first))
            }
            Expr::Field { object, field } => {
                // Enum unit variant construction: Color.Red  (object is Ident of enum name)
                if let Expr::Ident(enum_name) = object.as_ref() {
                    if let Some(variants) = self.enums.get(enum_name) {
                        if let Some((_, nfields)) = variants.iter().find(|(v, _)| v == field) {
                            if *nfields == 0 {
                                return Type::Enum(enum_name.clone());
                            } else {
                                self.error(format!(
                                    "Variant '{}.{}' expects {} payload argument(s); use {}.{}(...)",
                                    enum_name, field, nfields, enum_name, field
                                ));
                                return Type::Unknown;
                            }
                        } else {
                            self.error(format!(
                                "Enum '{}' has no variant '{}'",
                                enum_name, field
                            ));
                            return Type::Unknown;
                        }
                    }
                }
                // Field access borrows the object (does not move)
                let obj_ty = self.check_expr_ref(object);
                match &obj_ty {
                    Type::String if field == "length" => Type::Number,
                    Type::List(_) if field == "length" => Type::Number,
                    Type::Struct(sname) => {
                        if let Some(fields) = self.structs.get(sname) {
                            if fields.iter().any(|f| f == field) {
                                Type::Number
                            } else {
                                self.error(format!("Struct '{}' has no field '{}'", sname, field));
                                Type::Unknown
                            }
                        } else {
                            self.error(format!("Unknown struct type '{}'", sname));
                            Type::Unknown
                        }
                    }
                    Type::Unknown => Type::Unknown,
                    other => {
                        self.error(format!("Type {} has no field '{}'", other, field));
                        Type::Unknown
                    }
                }
            }
            Expr::Index { object, index } => {
                // Indexing borrows the list
                let obj_ty = self.check_expr_ref(object);
                let idx_ty = self.check_expr(index);
                if idx_ty != Type::Number && idx_ty != Type::Unknown {
                    self.error(format!("List index must be Number, found {}", idx_ty));
                }
                match obj_ty {
                    Type::List(inner) => *inner,
                    Type::Unknown => Type::Unknown,
                    other => {
                        self.error(format!("Cannot index type {}", other));
                        Type::Unknown
                    }
                }
            }
            Expr::Try(inner) => {
                // `expr?` — expects Result/Option-style enum; yields payload Number
                let t = self.check_expr(inner);
                match t {
                    Type::Enum(name) => {
                        // Convention: Ok/Some has payload, Err/None does not
                        if let Some(variants) = self.enums.get(&name) {
                            let has_ok = variants.iter().any(|(v, n)| {
                                (v == "Ok" || v == "Some") && *n > 0
                            });
                            if !has_ok {
                                self.error(format!(
                                    "'?' requires enum '{}' to have Ok(value) or Some(value) variant",
                                    name
                                ));
                            }
                        }
                        Type::Number
                    }
                    Type::Unknown => Type::Unknown,
                    other => {
                        self.error(format!(
                            "'?' can only be applied to Result/Option-like enums, found {}",
                            other
                        ));
                        Type::Unknown
                    }
                }
            }
        }
    }

    fn check_ident(&mut self, name: &str) -> Type {
        self.use_by_value(name)
    }

    /// Check expression in borrow (by-ref) context — does not move the root binding.
    fn check_expr_ref(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Ident(name) => self.use_by_ref(name),
            other => self.check_expr(other),
        }
    }

    fn check_binary(&mut self, left: &Expr, op: BinaryOp, right: &Expr) -> Type {
        let l = self.check_expr(left);
        let r = self.check_expr(right);

        match op {
            BinaryOp::Add => {
                // Number + Number → Number
                // Any involvement of String → String (convenience concat)
                match (&l, &r) {
                    (Type::Number, Type::Number) => Type::Number,
                    (Type::String, Type::String)
                    | (Type::String, Type::Number)
                    | (Type::Number, Type::String)
                    | (Type::String, Type::Unknown)
                    | (Type::Unknown, Type::String) => Type::String,
                    (Type::Unknown, _) | (_, Type::Unknown) => Type::Unknown,
                    _ => {
                        self.error(format!("Cannot apply '+' to {} and {}", l, r));
                        Type::Unknown
                    }
                }
            }
            BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                if (l == Type::Number || l == Type::Unknown)
                    && (r == Type::Number || r == Type::Unknown)
                {
                    Type::Number
                } else {
                    self.error(format!("Cannot apply '{}' to {} and {}", op, l, r));
                    Type::Unknown
                }
            }
            BinaryOp::Eq | BinaryOp::NotEq => {
                // Equality allowed between same types
                if l != r && l != Type::Unknown && r != Type::Unknown {
                    self.error(format!("Cannot compare {} and {} for equality", l, r));
                }
                Type::Bool
            }
            BinaryOp::Lt | BinaryOp::Gt | BinaryOp::LtEq | BinaryOp::GtEq => {
                if (l == Type::Number || l == Type::Unknown)
                    && (r == Type::Number || r == Type::Unknown)
                {
                    Type::Bool
                } else {
                    self.error(format!("Cannot compare {} and {} with '{}'", l, r, op));
                    Type::Bool
                }
            }
        }
    }

    fn check_unary(&mut self, op: UnaryOp, expr: &Expr) -> Type {
        let t = self.check_expr(expr);
        match op {
            UnaryOp::Neg => {
                if t == Type::Number || t == Type::Unknown {
                    Type::Number
                } else {
                    self.error(format!("Cannot negate {}", t));
                    Type::Unknown
                }
            }
            UnaryOp::Not => {
                if t == Type::Bool || t == Type::Unknown {
                    Type::Bool
                } else {
                    self.error(format!("Cannot apply 'not' to {}", t));
                    Type::Unknown
                }
            }
        }
    }

    fn check_call(&mut self, callee: &Expr, args: &[Expr]) -> Type {
        // Enum variant with payload: Option.Some(42)
        if let Expr::Field { object, field } = callee {
            if let Expr::Ident(enum_name) = object.as_ref() {
                if let Some(variants) = self.enums.get(enum_name).cloned() {
                    if let Some((_, nfields)) = variants.iter().find(|(v, _)| v == field) {
                        if args.len() != *nfields {
                            self.error(format!(
                                "Variant '{}.{}' expects {} argument(s), found {}",
                                enum_name,
                                field,
                                nfields,
                                args.len()
                            ));
                        }
                        for a in args {
                            let t = self.check_expr(a);
                            if t != Type::Number && t != Type::Unknown {
                                self.error(format!(
                                    "Enum payload must be Number for now, found {}",
                                    t
                                ));
                            }
                        }
                        return Type::Enum(enum_name.clone());
                    }
                }
            }
        }


        // Module path call: math.add(1, 2) → function math_add
        if let Expr::Field { object, field } = callee {
            if let Expr::Ident(mod_name) = object.as_ref() {
                let full = format!("{}_{}", mod_name, field);
                if let Some(fty) = self.functions.get(&full).cloned() {
                    let arg_tys: Vec<Type> = args.iter().map(|a| self.check_expr(a)).collect();
                    if let Type::Function { params, ret } = fty {
                        if params.len() != arg_tys.len() {
                            self.error(format!(
                                "Function '{}' expects {} arguments, found {}",
                                full,
                                params.len(),
                                arg_tys.len()
                            ));
                        }
                        for (i, (p, a)) in params.iter().zip(arg_tys.iter()).enumerate() {
                            if *p != Type::Unknown && *a != Type::Unknown && p != a {
                                self.error(format!(
                                    "Argument {} type mismatch: expected {}, found {}",
                                    i + 1, p, a
                                ));
                            }
                        }
                        return *ret;
                    }
                }
            }
        }

        // Struct construction: Point(10, 20)
        if let Expr::Ident(name) = callee
            && let Some(fields) = self.structs.get(name).cloned()
        {
            if args.len() != fields.len() {
                self.error(format!(
                    "Struct '{}' expects {} fields, found {}",
                    name,
                    fields.len(),
                    args.len()
                ));
            }
            for a in args {
                let t = self.check_expr(a);
                if t != Type::Number && t != Type::Unknown {
                    self.error(format!(
                        "Struct field initializer must be Number, found {}",
                        t
                    ));
                }
            }
            return Type::Struct(name.clone());
        }

        // Method call: obj.method(args)  →  Call { callee: Field { object, field }, args }
        // Receiver is borrowed (does not move).
        if let Expr::Field { object, field } = callee {
            let obj_ty = self.check_expr_ref(object);
            if let Type::Struct(struct_name) = &obj_ty {
                if let Some(method_ty) = self.methods.get(&(struct_name.clone(), field.clone())) {
                    let method_ty = method_ty.clone();
                    let arg_tys: Vec<Type> = args.iter().map(|a| self.check_expr(a)).collect();
                    // method params = [receiver] + extra args
                    if let Type::Function { params, ret } = method_ty {
                        let expected_extra = if params.is_empty() {
                            0
                        } else {
                            params.len() - 1
                        };
                        if arg_tys.len() != expected_extra {
                            self.error(format!(
                                "Method '{}.{}' expects {} arguments, found {}",
                                struct_name,
                                field,
                                expected_extra,
                                arg_tys.len()
                            ));
                        }
                        // type-check extra args against params[1..]
                        for (i, (p, a)) in params.iter().skip(1).zip(arg_tys.iter()).enumerate() {
                            if *p != Type::Unknown && *a != Type::Unknown && p != a {
                                self.error(format!(
                                    "Argument {} type mismatch: expected {}, found {}",
                                    i + 1,
                                    p,
                                    a
                                ));
                            }
                        }
                        return *ret;
                    }
                } else {
                    self.error(format!(
                        "No method '{}' found for type '{}'",
                        field, struct_name
                    ));
                    return Type::Unknown;
                }
            } else if obj_ty != Type::Unknown {
                self.error(format!(
                    "Cannot call method '{}' on non-struct type {}",
                    field, obj_ty
                ));
                return Type::Unknown;
            }
        }

        let callee_ty = self.check_expr(callee);
        let arg_tys: Vec<Type> = args.iter().map(|a| self.check_expr(a)).collect();

        match callee_ty {
            Type::Function { params, ret } => {
                if params.len() != arg_tys.len() {
                    self.error(format!(
                        "Function expects {} arguments, found {}",
                        params.len(),
                        arg_tys.len()
                    ));
                }
                for (i, (p, a)) in params.iter().zip(arg_tys.iter()).enumerate() {
                    if *p != Type::Unknown && *a != Type::Unknown && p != a {
                        self.error(format!(
                            "Argument {} type mismatch: expected {}, found {}",
                            i + 1,
                            p,
                            a
                        ));
                    }
                }
                *ret
            }
            Type::Unknown => Type::Unknown,
            other => {
                self.error(format!("Cannot call value of type {}", other));
                Type::Unknown
            }
        }
    }
}
