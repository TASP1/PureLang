//! Type system for PureLang

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Number,
    String,
    Bool,
    /// Inclusive/exclusive range of numbers (from `a..b`)
    Range,
    /// Homogeneous list
    List(Box<Type>),
    /// Named struct type
    Struct(String),
    /// Function type (params -> return)
    Function {
        params: Vec<Type>,
        ret: Box<Type>,
    },
    /// Unit / no value
    Void,
    /// Type could not be determined (error recovery)
    Unknown,
}

impl Type {
    pub fn is_copy(&self) -> bool {
        matches!(
            self,
            Type::Number | Type::Bool | Type::Range | Type::Void | Type::Struct(_)
        )
    }

    #[allow(dead_code)]
    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Number)
    }

    #[allow(dead_code)]
    pub fn is_bool_like(&self) -> bool {
        matches!(self, Type::Bool)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Number => write!(f, "Number"),
            Type::String => write!(f, "String"),
            Type::Bool => write!(f, "Bool"),
            Type::Range => write!(f, "Range"),
            Type::List(inner) => write!(f, "List<{inner}>"),
            Type::Struct(name) => write!(f, "{name}"),
            Type::Function { params, ret } => {
                let ps: Vec<String> = params.iter().map(|p| p.to_string()).collect();
                write!(f, "fn({}) -> {}", ps.join(", "), ret)
            }
            Type::Void => write!(f, "Void"),
            Type::Unknown => write!(f, "?"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeError {
    pub message: String,
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Type error: {}", self.message)
    }
}

impl std::error::Error for TypeError {}

impl TypeError {
    pub fn new(msg: impl Into<String>) -> Self {
        TypeError {
            message: msg.into(),
        }
    }
}
