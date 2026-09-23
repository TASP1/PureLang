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
        /// Generic type parameters: `fn id[T](x: T)`
        type_params: Vec<String>,
        params: Vec<Param>,
        body: Block,
        /// Visible outside its module
        is_pub: bool,
    },
    Struct {
        name: String,
        fields: Vec<String>,
        is_pub: bool,
    },
    /// `enum Color { Red Green Blue }` or with payload `Some(value)`
    Enum {
        name: String,
        variants: Vec<EnumVariant>,
        is_pub: bool,
    },
    /// `mod name { ... }`
    Module {
        name: String,
        items: Vec<Item>,
        is_pub: bool,
    },
    /// `trait Show { fn show(self) }`
    Trait {
        name: String,
        methods: Vec<TraitMethod>,
        is_pub: bool,
    },
    /// `impl Show for Point { fn show(self) { ... } }`
    Impl {
        trait_name: Option<String>,
        type_name: String,
        methods: Vec<Item>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitMethod {
    pub name: String,
    pub params: Vec<Param>,
}

/// Function parameter: `name` or `name: Type`
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    /// Optional type annotation (Number, String, Bool, or struct/enum name)
    pub ty_annotation: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct EnumVariant {
    pub name: String,
    /// Optional payload field names (MVP: 0 or 1 Number payload)
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct Block {
    pub statements: Vec<StmtNode>,
}

/// Statement with source line (1-based)
#[derive(Debug, Clone, PartialEq)]
pub struct StmtNode {
    pub line: u32,
    pub stmt: Stmt,
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
    /// `while condition { ... }`
    While {
        condition: Expr,
        body: Block,
    },
    /// `break`
    Break,
    /// `continue`
    Continue,
    /// `return` or `return expr`
    Return(Option<Expr>),
    /// `match expr { Pattern => block ... }`
    Match { expr: Expr, arms: Vec<MatchArm> },
    /// Expression statement
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct MatchArm {
    /// Pattern like `Color.Red` or `Option.Some(v)`
    pub pattern: Pattern,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Pattern {
    /// `EnumName.Variant` or `EnumName.Variant(bind)`
    Variant {
        enum_name: String,
        variant: String,
        /// Optional binding for payload
        binding: Option<String>,
    },
    /// Wildcard `_` (future)
    Wildcard,
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
    /// `expr?` — unwrap Ok/Some or early-return on Err/None
    Try(Box<Expr>),
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
