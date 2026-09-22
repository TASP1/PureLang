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
            errors: Vec::new(),
        }
    }

    pub fn check_program(&mut self, program: &Program) -> Result<(), Vec<TypeError>> {
        // First pass: register structs and function signatures
        for item in &program.items {
            match item {
                Item::Struct { name, fields } => {
                    self.structs.insert(name.clone(), fields.clone());
                }
                Item::Function {
                    receiver,
                    name,
                    params,
                    body,
                } => {
                    let ret = Self::infer_return_type(body);
                    if let Some(recv) = receiver {
                        // Method: first param is the receiver type (Struct)
                        let mut param_tys: Vec<Type> = vec![Type::Struct(recv.clone())];
                        for _ in params.iter().skip(1) {
                            param_tys.push(Type::Number);
                        }
                        // If no params at all, still treat as method taking only self
                        if params.is_empty() {
                            param_tys = vec![Type::Struct(recv.clone())];
                        } else if params.len() == 1 {
                            // single param is the receiver (usually named self)
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
                        let param_tys: Vec<Type> =
                            params.iter().map(|_| Type::Number).collect();
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
        for item in &program.items {
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
                },
            );
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
                    // Method: first param is the receiver (struct type)
                    if let Some(first) = params.first() {
                        self.declare(first, Type::Struct(recv.clone()), false);
                        for p in params.iter().skip(1) {
                            self.declare(p, Type::Number, false);
                        }
                    } else {
                        // no params listed — still allow, but unusual
                    }
                } else {
                    for p in params {
                        // Parameters are immutable by default; Phase 2: numeric by default
                        self.declare(p, Type::Number, false);
                    }
                }
                self.check_block(body);
                self.pop_scope();
            }
            Item::Struct { .. } => {
                // Struct declarations are fine for now; no body to check
            }
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
                let _ = self.check_expr(expr);
                // print accepts any type
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
                let iter_ty = self.check_expr(iterable);
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
                let obj_ty = self.check_expr(object);
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
                let obj_ty = self.check_expr(object);
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
        }
    }

    fn check_ident(&mut self, name: &str) -> Type {
        // Built-in functions
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
                let ty = info.ty.clone();
                // Move non-Copy types on use (simple ownership)
                if !ty.is_copy() {
                    // Mark moved — PureLang invisible ownership
                    if let Some(info) = self.lookup_mut(name) {
                        info.moved = true;
                    }
                }
                ty
            }
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
        if let Expr::Field { object, field } = callee {
            let obj_ty = self.check_expr(object);
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
