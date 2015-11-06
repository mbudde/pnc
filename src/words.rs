use std::fmt;

pub type Word = String;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BuiltinWord {
    Add,
    Alias,
    Def,
    Div,
    Duplicate,
    Fold,
    Length,
    Map,
    Mul,
    Over,
    Pop,
    Print,
    Repeat,
    Roll3,
    Stdin,
    Sub,
    Sum,
    Swap,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Operation {
    Builtin(BuiltinWord),
    Block(Vec<Word>),
}

pub fn block<'a, 'b>(words: &'a [&'b str]) -> Operation {
    Operation::Block(words.into_iter().map(|s| s.to_string()).collect())
}

#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    Vector(Vec<Value>),
    Block(Vec<Word>),
    QuotedWord(Word),
}

impl Value {
    pub fn parse(s: &str) -> Option<Value> {
        if let Ok(num) = s.parse::<i64>() {
            Some(Value::Int(num))
        } else if let Ok(num) = s.parse::<f64>() {
            Some(Value::Float(num))
        } else {
            None
        }
    }

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

    pub fn as_float(&self) -> Option<f64> {
        match *self {
            Value::Float(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_vector(self) -> Option<Vec<Value>> {
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

    pub fn as_block(self) -> Option<Vec<Word>> {
        match self {
            Value::Block(b) => Some(b),
            _ => None,
        }
    }

    pub fn as_word(self) -> Option<Word> {
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
            Int(v) => v.fmt(f),
            Float(v) => v.fmt(f),
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
