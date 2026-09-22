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
    errors: Vec<TypeError>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            scopes: vec![Scope {
                vars: HashMap::new(),
            }],
            functions: HashMap::new(),
            errors: Vec::new(),
        }
    }

    pub fn check_program(&mut self, program: &Program) -> Result<(), Vec<TypeError>> {
        // First pass: register function signatures (params typed as Unknown for now)
        for item in &program.items {
            if let Item::Function { name, params, .. } = item {
                let param_tys = params.iter().map(|_| Type::Unknown).collect();
                self.functions.insert(
                    name.clone(),
                    Type::Function {
                        params: param_tys,
                        ret: Box::new(Type::Void),
                    },
                );
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

    fn check_item(&mut self, item: &Item) {
        match item {
            Item::Function {
                name: _,
                params,
                body,
            } => {
                self.push_scope();
                for p in params {
                    // Parameters are immutable by default, type unknown until annotations exist
                    self.declare(p, Type::Unknown, false);
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
                // Minimal field support; expand when structs are typed
                match (&obj_ty, field.as_str()) {
                    (Type::String, "length") | (Type::List(_), "length") => Type::Number,
                    (Type::Unknown, _) => Type::Unknown,
                    _ => {
                        self.error(format!("Type {} has no field '{}'", obj_ty, field));
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
        let callee_ty = self.check_expr(callee);
        let arg_tys: Vec<Type> = args.iter().map(|a| self.check_expr(a)).collect();

        match callee_ty {
            Type::Function { params, ret } => {
                if params.len() != arg_tys.len() && !params.is_empty() {
                    // Allow unknown arity when params are all Unknown
                    let all_unknown = params.iter().all(|p| *p == Type::Unknown);
                    if !all_unknown {
                        self.error(format!(
                            "Function expects {} arguments, found {}",
                            params.len(),
                            arg_tys.len()
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
