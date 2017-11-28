use std::fmt;

use num::bigint::BigInt;
use num::ToPrimitive;

pub type Word = String;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BuiltinWord {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Sqrt,
    Pow,
    Exp,
    Log,
    Ln,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,

    // Functional
    Fold,
    Fold1,
    Map,
    Filter,
    Repeat,

    // Comparisons
    Max,
    Min,
    Cmp,

    // Blocks
    Alias,
    Def,
    Apply,
    Arg,

    // Stack manipulation
    Swap,
    Duplicate,
    Pop,
    Over,
    Roll3,

    // IO
    Print,
    Dump,
    Stdin,

    // Vectors
    Length,

    // Control flow
    If,
}

#[derive(Debug, PartialEq)]
pub enum Operation {
    Builtin(BuiltinWord),
    Value(Value),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Undef,
    #[allow(dead_code)]
    Bool(bool),
    Int(BigInt),
    Float(f64),
    Vector(Vec<Value>),
    Block(Vec<Word>),
    QuotedWord(Word),
}

impl Value {
    pub fn parse(s: &str) -> Option<Value> {
        let s = s.trim();
        if let Some(num) = BigInt::parse_bytes(s.as_bytes(), 10) {
            Some(Value::Int(num))
        } else if let Ok(num) = s.parse::<f64>() {
            Some(Value::Float(num))
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn as_bool(&self) -> Option<bool> {
        match *self {
            Value::Bool(b) => Some(b),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<BigInt> {
        match *self {
            Value::Int(ref i) => Some(i.clone()),
            _ => None,
        }
    }

    pub fn as_int_cast(&self) -> Option<i64> {
        match *self {
            Value::Int(ref i) => i.to_i64(),
            Value::Float(f) => Some(f as i64),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match *self {
            Value::Float(f) => Some(f),
            _ => None,
        }
    }

    pub fn into_vector(self) -> Result<Vec<Value>, Self> {
        match self {
            Value::Vector(v) => Ok(v),
            v => Err(v),
        }
    }

    pub fn as_float_cast(&self) -> Option<f64> {
        match *self {
            Value::Float(f) => Some(f),
            Value::Int(ref i) => i.to_f64(),
            _ => None,
        }
    }

    pub fn into_block(self) -> Result<Vec<Word>, Self> {
        match self {
            Value::Block(b) => Ok(b),
            v => Err(v),
        }
    }

    pub fn into_word(self) -> Result<Word, Self> {
        match self {
            Value::QuotedWord(w) => Ok(w),
            v => Err(v),
        }
    }

    pub fn type_of(&self) -> &'static str {
        match *self {
            Value::Undef          => "undef",
            Value::Bool(..)       => "type",
            Value::Int(..)        => "int",
            Value::Float(..)      => "float",
            Value::Vector(..)     => "vector",
            Value::Block(..)      => "block",
            Value::QuotedWord(..) => "quoted word",
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Value::*;
        match *self {
            Undef => write!(f, "undef"),
            Bool(v) => v.fmt(f),
            Int(ref v) => write!(f, "{}", v),
            Float(v) => write!(f, "{}", v),
            Vector(ref v) => {
                write!(f, "[")?;
                let mut iter = v.into_iter();
                if let Some(e) = iter.next() {
                    e.fmt(f)?;
                    for e in iter {
                        write!(f, ", ")?;
                        e.fmt(f)?;
                    }
                }
                write!(f, "]")?;
                write!(f, " len: {}", v.len())?;
                Ok(())
            }
            Block(_) => write!(f, "<block>"),
            QuotedWord(ref word) => write!(f, "{}", word),
        }
    }
}
