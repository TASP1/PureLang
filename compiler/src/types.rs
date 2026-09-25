//! Type system for PureLang

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Number,
    /// IEEE-754 floating point (codegen: double)
    Float,
    String,
    Bool,
    /// Inclusive/exclusive range of numbers (from `a..b`)
    Range,
    /// Homogeneous list
    List(Box<Type>),
    /// Named struct type
    Struct(String),
    /// Named enum type
    Enum(String),
    /// Generic type parameter (T, U, ...)
    Generic(String),
    /// Map (string keys → number values) — runtime PLMap*
    Map,
    /// Channel handle (runtime PLChan*)
    Channel,
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
    /// Types that are trivially copyable (no ownership transfer needed).
    pub fn is_copy(&self) -> bool {
        matches!(
            self,
            Type::Number
                | Type::Float
                | Type::String
                | Type::Bool
                | Type::Range
                | Type::Void
                | Type::Enum(_)
                | Type::Generic(_)
                | Type::Unknown
        )
    }

    /// Types that own heap/stack resources and move by default.
    pub fn is_move_type(&self) -> bool {
        // Phase 2 MVP: lists/structs copy for field/index/helper use.
        // Strings still move.
        matches!(self, Type::String | Type::Map)
    }

    #[allow(dead_code)]
    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Number | Type::Float)
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
            Type::Float => write!(f, "Float"),
            Type::String => write!(f, "String"),
            Type::Bool => write!(f, "Bool"),
            Type::Range => write!(f, "Range"),
            Type::List(inner) => write!(f, "List<{inner}>"),
            Type::Struct(name) => write!(f, "{name}"),
            Type::Enum(name) => write!(f, "{name}"),
            Type::Generic(name) => write!(f, "{name}"),
            Type::Map => write!(f, "Map"),
            Type::Channel => write!(f, "Channel"),
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
