//! Source formatter: AST → pretty PureLang text

use crate::ast::*;

pub fn format_program(program: &Program) -> String {
    let mut out = String::new();
    for (i, item) in program.items.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        format_item(item, &mut out, 0);
        out.push('\n');
    }
    out
}

fn indent(level: usize) -> String {
    "    ".repeat(level)
}

fn format_params(params: &[Param], out: &mut String) {
    for (i, p) in params.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&p.name);
        if let Some(ty) = &p.ty_annotation {
            out.push_str(": ");
            out.push_str(ty);
        }
    }
}

fn format_item(item: &Item, out: &mut String, level: usize) {
    match item {
        Item::Function {
            receiver,
            name,
            type_params,
            params,
            body,
            is_pub,
        } => {
            out.push_str(&indent(level));
            if *is_pub {
                out.push_str("pub ");
            }
            out.push_str("fn ");
            if let Some(recv) = receiver {
                out.push_str(recv);
                out.push('.');
            }
            out.push_str(name);
            if !type_params.is_empty() {
                out.push('[');
                out.push_str(&type_params.join(", "));
                out.push(']');
            }
            out.push('(');
            format_params(params, out);
            out.push_str(") {\n");
            format_block(body, out, level + 1);
            out.push_str(&indent(level));
            out.push_str("}\n");
        }
        Item::Struct {
            name,
            fields,
            is_pub,
        } => {
            out.push_str(&indent(level));
            if *is_pub {
                out.push_str("pub ");
            }
            out.push_str("struct ");
            out.push_str(name);
            out.push_str(" {\n");
            for f in fields {
                out.push_str(&indent(level + 1));
                out.push_str(f);
                out.push('\n');
            }
            out.push_str(&indent(level));
            out.push_str("}\n");
        }
        Item::Enum {
            name,
            variants,
            is_pub,
        } => {
            out.push_str(&indent(level));
            if *is_pub {
                out.push_str("pub ");
            }
            out.push_str("enum ");
            out.push_str(name);
            out.push_str(" {\n");
            for v in variants {
                out.push_str(&indent(level + 1));
                out.push_str(&v.name);
                if !v.fields.is_empty() {
                    out.push('(');
                    out.push_str(&v.fields.join(", "));
                    out.push(')');
                }
                out.push('\n');
            }
            out.push_str(&indent(level));
            out.push_str("}\n");
        }
        Item::Module {
            name,
            items,
            is_pub,
        } => {
            out.push_str(&indent(level));
            if *is_pub {
                out.push_str("pub ");
            }
            out.push_str("mod ");
            out.push_str(name);
            out.push_str(" {\n");
            for it in items {
                format_item(it, out, level + 1);
            }
            out.push_str(&indent(level));
            out.push_str("}\n");
        }
        Item::Trait {
            name,
            methods,
            is_pub,
        } => {
            out.push_str(&indent(level));
            if *is_pub {
                out.push_str("pub ");
            }
            out.push_str("trait ");
            out.push_str(name);
            out.push_str(" {\n");
            for m in methods {
                out.push_str(&indent(level + 1));
                out.push_str("fn ");
                out.push_str(&m.name);
                out.push('(');
                format_params(&m.params, out);
                out.push_str(")\n");
            }
            out.push_str(&indent(level));
            out.push_str("}\n");
        }
        Item::Impl {
            trait_name,
            type_name,
            methods,
        } => {
            out.push_str(&indent(level));
            out.push_str("impl ");
            if let Some(t) = trait_name {
                out.push_str(t);
                out.push_str(" for ");
            }
            out.push_str(type_name);
            out.push_str(" {\n");
            for m in methods {
                format_item(m, out, level + 1);
            }
            out.push_str(&indent(level));
            out.push_str("}\n");
        }
    }
}

fn format_block(block: &Block, out: &mut String, level: usize) {
    for node in &block.statements {
        format_stmt(&node.stmt, out, level);
    }
}

fn format_stmt(stmt: &Stmt, out: &mut String, level: usize) {
    match stmt {
        Stmt::Let {
            mutable,
            name,
            value,
        } => {
            out.push_str(&indent(level));
            if *mutable {
                out.push_str("mut ");
            }
            out.push_str(name);
            out.push_str(" = ");
            format_expr(value, out);
            out.push('\n');
        }
        Stmt::Assign { name, value } => {
            out.push_str(&indent(level));
            out.push_str(name);
            out.push_str(" = ");
            format_expr(value, out);
            out.push('\n');
        }
        Stmt::Print(expr) => {
            out.push_str(&indent(level));
            out.push_str("print ");
            format_expr(expr, out);
            out.push('\n');
        }
        Stmt::If {
            condition,
            then_block,
            else_block,
        } => {
            out.push_str(&indent(level));
            out.push_str("if ");
            format_expr(condition, out);
            out.push_str(" {\n");
            format_block(then_block, out, level + 1);
            out.push_str(&indent(level));
            out.push('}');
            if let Some(eb) = else_block {
                out.push_str(" else {\n");
                format_block(eb, out, level + 1);
                out.push_str(&indent(level));
                out.push('}');
            }
            out.push('\n');
        }
                Stmt::While { condition, body } => {
            out.push_str(&indent(level));
            out.push_str("while ");
            format_expr(condition, out);
            out.push_str(" {\n");
            format_block(body, out, level + 1);
            out.push_str(&indent(level));
            out.push_str("}\n");
        }
        Stmt::Break => {
            out.push_str(&indent(level));
            out.push_str("break\n");
        }
        Stmt::Continue => {
            out.push_str(&indent(level));
            out.push_str("continue\n");
        }
        Stmt::For {
            var,
            iterable,
            body,
        } => {
            out.push_str(&indent(level));
            out.push_str("for ");
            out.push_str(var);
            out.push_str(" in ");
            format_expr(iterable, out);
            out.push_str(" {\n");
            format_block(body, out, level + 1);
            out.push_str(&indent(level));
            out.push_str("}\n");
        }
        Stmt::Return(None) => {
            out.push_str(&indent(level));
            out.push_str("return\n");
        }
        Stmt::Return(Some(e)) => {
            out.push_str(&indent(level));
            out.push_str("return ");
            format_expr(e, out);
            out.push('\n');
        }
        Stmt::Match { expr, arms } => {
            out.push_str(&indent(level));
            out.push_str("match ");
            format_expr(expr, out);
            out.push_str(" {\n");
            for arm in arms {
                out.push_str(&indent(level + 1));
                format_pattern(&arm.pattern, out);
                out.push_str(" => {\n");
                format_block(&arm.body, out, level + 2);
                out.push_str(&indent(level + 1));
                out.push_str("}\n");
            }
            out.push_str(&indent(level));
            out.push_str("}\n");
        }
        Stmt::Expr(e) => {
            out.push_str(&indent(level));
            format_expr(e, out);
            out.push('\n');
        }
    }
}

fn format_pattern(pat: &Pattern, out: &mut String) {
    match pat {
        Pattern::Variant {
            enum_name,
            variant,
            binding,
        } => {
            out.push_str(enum_name);
            out.push('.');
            out.push_str(variant);
            if let Some(b) = binding {
                out.push('(');
                out.push_str(b);
                out.push(')');
            }
        }
        Pattern::Wildcard => out.push('_'),
    }
}

fn format_expr(expr: &Expr, out: &mut String) {
    match expr {
        Expr::Number(n) => {
            if *n == (*n as i64) as f64 {
                out.push_str(&format!("{}", *n as i64));
            } else {
                out.push_str(&format!("{}", n));
            }
        }
        Expr::String(s) => {
            out.push('"');
            for c in s.chars() {
                match c {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    _ => out.push(c),
                }
            }
            out.push('"');
        }
        Expr::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Expr::Ident(name) => out.push_str(name),
        Expr::Binary { left, op, right } => {
            format_expr(left, out);
            out.push(' ');
            out.push_str(&op.to_string());
            out.push(' ');
            format_expr(right, out);
        }
        Expr::Unary { op, expr } => {
            match op {
                UnaryOp::Neg => out.push('-'),
                UnaryOp::Not => out.push_str("not "),
            }
            format_expr(expr, out);
        }
        Expr::Call { callee, args } => {
            format_expr(callee, out);
            out.push('(');
            for (i, a) in args.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                format_expr(a, out);
            }
            out.push(')');
        }
        Expr::Range { start, end } => {
            format_expr(start, out);
            out.push_str("..");
            format_expr(end, out);
        }
        Expr::List(elems) => {
            out.push('[');
            for (i, e) in elems.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                format_expr(e, out);
            }
            out.push(']');
        }
        Expr::Field { object, field } => {
            format_expr(object, out);
            out.push('.');
            out.push_str(field);
        }
        Expr::Index { object, index } => {
            format_expr(object, out);
            out.push('[');
            format_expr(index, out);
            out.push(']');
        }
        Expr::Try(inner) => {
            format_expr(inner, out);
            out.push('?');
        }
    }
}
