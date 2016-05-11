use std::fmt;

pub type Word = String;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BuiltinWord {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Fold,
    Map,
    Filter,
    Sum,
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
    Stdin,

    // Vectors
    Length,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Operation {
    Builtin(BuiltinWord),
    Block(Vec<Word>),
}

#[derive(Debug, Clone)]
pub enum Value {
    #[allow(dead_code)]
    Bool(bool),
    Int(i64),
    Float(f64),
    Vector(Vec<Value>),
    Block(Vec<Word>),
    QuotedWord(Word),
}

impl Value {
    pub fn parse(s: &str) -> Option<Value> {
        let s = s.trim();
        if let Ok(num) = s.parse::<i64>() {
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

    pub fn as_int(&self) -> Option<i64> {
        match *self {
            Value::Int(i) => Some(i),
            _ => None,
        }
    }

    pub fn as_int_cast(&self) -> Option<i64> {
        match *self {
            Value::Int(i) => Some(i),
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

    pub fn into_vector(self) -> Option<Vec<Value>> {
        match self {
            Value::Vector(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_float_cast(&self) -> Option<f64> {
        match *self {
            Value::Float(f) => Some(f),
            Value::Int(i) => Some(i as f64),
            _ => None,
        }
    }

    pub fn into_block(self) -> Option<Vec<Word>> {
        match self {
            Value::Block(b) => Some(b),
            _ => None,
        }
    }

    pub fn into_word(self) -> Option<Word> {
        match self {
            Value::QuotedWord(w) => Some(w),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Value::*;
        match *self {
            Bool(v) => v.fmt(f),
            Int(v) => write!(f, "{}", v),
            Float(v) => write!(f, "{}", v),
            Vector(ref v) => {
                try!(write!(f, "["));
                let mut iter = v.into_iter();
                if let Some(e) = iter.next() {
                    try!(e.fmt(f));
                    for e in iter {
                        try!(write!(f, ", "));
                        try!(e.fmt(f));
                    }
                }
                try!(write!(f, "]"));
                try!(write!(f, " len: {}", v.len()));
                Ok(())
            }
            Block(_) => write!(f, "<block>"),
            QuotedWord(ref word) => write!(f, "{}", word),
        }
    }
}
