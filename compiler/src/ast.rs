//! Abstract Syntax Tree for PureLang

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Item {
    Function {
        /// Optional receiver type for methods: `fn Point.distance(self) { ... }`
        receiver: Option<String>,
        name: String,
        params: Vec<String>,
        body: Block,
    },
    Struct {
        name: String,
        fields: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct Block {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Stmt {
    /// `mut name = expr` or `name = expr` (declaration)
    Let {
        mutable: bool,
        name: String,
        value: Expr,
    },
    /// `name = expr` (assignment to existing)
    Assign { name: String, value: Expr },
    /// `print expr`
    Print(Expr),
    /// `if cond { ... } else { ... }`
    If {
        condition: Expr,
        then_block: Block,
        else_block: Option<Block>,
    },
    /// `for var in iterable { ... }`
    For {
        var: String,
        iterable: Expr,
        body: Block,
    },
    /// `return` or `return expr`
    Return(Option<Expr>),
    /// Expression statement
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Expr {
    Number(f64),
    String(String),
    Bool(bool),
    Ident(String),
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    /// `start..end`
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
    },
    List(Vec<Expr>),
    /// `obj.field`
    Field {
        object: Box<Expr>,
        field: String,
    },
    /// `list[index]`
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum UnaryOp {
    Neg,
    Not,
}

impl std::fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Eq => "==",
            BinaryOp::NotEq => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::Gt => ">",
            BinaryOp::LtEq => "<=",
            BinaryOp::GtEq => ">=",
        };
        write!(f, "{s}")
    }
}
