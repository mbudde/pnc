
use std::fmt;
use std::error::Error;
use std::rc::Rc;

mod dict;

pub type Word = String;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BuiltinWord {
    Add,
    Sub,
    Mul,
    Div,
    Print,
    Pop,
    Duplicate,
    Stdin,
    Sum,
    Length,
    Swap,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Operation {
    Builtin(BuiltinWord),
    Block(Vec<Word>),
}

fn block<'a, 'b>(words: &'a [&'b str]) -> Operation {
    Operation::Block(words.into_iter().map(|s| s.to_string()).collect())
}

#[derive(Debug, Clone)]
enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    Vector(Vec<Value>),
    Block(Vec<Word>),
    QuotedWord(Word),
}

impl Value {
    fn parse(s: &str) -> Option<Value> {
        if let Ok(num) = s.parse::<i64>() {
            Some(Value::Int(num))
        } else if let Ok(num) = s.parse::<f64>() {
            Some(Value::Float(num))
        } else {
            None
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match *self { Value::Bool(b) => Some(b), _ => None }
    }

    fn as_int(&self) -> Option<i64> {
        match *self { Value::Int(i) => Some(i), _ => None }
    }

    fn as_float(&self) -> Option<f64> {
        match *self { Value::Float(f) => Some(f), _ => None }
    }

    fn as_vector(&self) -> Option<&Vec<Value>> {
        match *self { Value::Vector(ref v) => Some(v), _ => None }
    }

    fn as_float_cast(&self) -> Option<f64> {
        match *self {
            Value::Float(f) => Some(f),
            Value::Int(i) => Some(i as f64),
            _ => None
        }
    }

    fn as_block(self) -> Option<Vec<Word>> {
        match self { Value::Block(b) => Some(b), _ => None }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Value::*;
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

#[derive(Debug)]
enum CalcError {
    DivisionByZero,
    MissingOperand,
    WrongTypeOperand,
    BlockNoResult,
    WordParseError,
}

impl Error for CalcError {
    fn description(&self) -> &str {
        "A calculator error occured"
    }
}

impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CalcError::DivisionByZero => write!(f, "Division by zero"),
            CalcError::MissingOperand => write!(f, "Operation needs an operand but stack is empty"),
            CalcError::WrongTypeOperand => write!(f, "Operand has a wrong type"),
            CalcError::BlockNoResult => write!(f, "Block left no result on the stack"),
            CalcError::WordParseError => write!(f, "Could not parse word as number or operation"),
        }
    }
}

type CalcResult<T> = Result<T, CalcError>;

struct Calc {
    dict: dict::Dictionary,
    data: Vec<Value>,
}

impl Calc {
    fn new() -> Calc {
        Calc {
            dict: Default::default(),
            data: Vec::new(),
        }
    }

    fn run<'a, I, T>(&mut self, iter: I) -> CalcResult<()>
        where I: Iterator<Item = T>,
              T: AsRef<str>
    {
        for word in iter {
            let word = word.as_ref();
            if word.starts_with(",") {
                self.data.push(Value::QuotedWord(word[1..].to_string()));
            } else if let Some(op) = self.dict.lookup(word) {
                match *op {
                    Operation::Builtin(builtin) => {
                        try!(self.run_builtin(builtin));
                    }
                    Operation::Block(ref block) => {
                        try!(self.run(block.into_iter()));
                    }
                }
            } else {
                if let Some(val) = Value::parse(word) {
                    self.data.push(val);
                } else {
                    println!("{}", word);
                    return Err(CalcError::WordParseError);
                }
            }
        }
        Ok(())
    }

    fn run_builtin(&mut self, word: BuiltinWord) -> CalcResult<()> {
        use BuiltinWord::*;
        match word {
            Add => self.perform_binop(std::ops::Add::add),
            Sub => self.perform_binop(std::ops::Sub::sub),
            Mul => self.perform_binop(std::ops::Mul::mul),
            Div => {
                let y = try!(self.get_float_cast());
                let x = try!(self.get_float_cast());
                if y == 0.0 {
                    return Err(CalcError::DivisionByZero);
                }
                self.data.push(Value::Float(x / y));
                Ok(())
            }
            Print => {
                let a = try!(self.get_operand());
                println!("{}", a);
                Ok(())
            }
            Pop => {
                self.data.pop();
                Ok(())
            }
            Duplicate => {
                let a = try!(self.get_operand());
                self.data.push(a.clone());
                self.data.push(a);
                Ok(())
            }
            Stdin => {
                use std::io::BufRead;
                let stdin = std::io::stdin();
                let stdin = stdin.lock();
                let vec = stdin.lines().map(|r| Value::parse(&r.unwrap()).unwrap()).collect();
                self.data.push(Value::Vector(vec));
                Ok(())
            }
            Sum => {
                let mut sum = 0f64;
                {
                    let a = try!(self.get_operand());
                    if let Value::Vector(ref vec) = a {
                        for val in vec {
                            if let Some(n) = val.as_float_cast() {
                                sum += n;
                            }
                        }
                    } else {
                        return Err(CalcError::WrongTypeOperand);
                    }

                }
                self.data.push(Value::Float(sum));
                Ok(())
            }
            Length => {
                let len = {
                    let a = try!(self.get_operand());
                    if let Value::Vector(ref vec) = a {
                        vec.len()
                    } else {
                        return Err(CalcError::WrongTypeOperand);
                    }
                };
                self.data.push(Value::Int(len as i64));
                Ok(())
            }
            Swap => {
                let a = try!(self.get_operand());
                let b = try!(self.get_operand());
                self.data.push(a);
                self.data.push(b);
                Ok(())
            }
        }
    }

    fn get_operand(&mut self) -> CalcResult<Value> {
        self.data.pop().ok_or(CalcError::MissingOperand)
    }

    fn get_int(&mut self) -> CalcResult<i64> {
        self.get_operand().and_then(|val| val.as_int().ok_or(CalcError::WrongTypeOperand))
    }
    fn get_float(&mut self) -> CalcResult<f64> {
        self.get_operand().and_then(|val| val.as_float().ok_or(CalcError::WrongTypeOperand))
    }

    fn get_float_cast(&mut self) -> CalcResult<f64> {
        self.get_operand().and_then(|val| val.as_float_cast().ok_or(CalcError::WrongTypeOperand))
    }

    fn get_block(&mut self) -> CalcResult<Vec<Word>> {
        self.get_operand().and_then(|val| val.as_block().ok_or(CalcError::WrongTypeOperand))
    }

    fn perform_unary<F>(&mut self, f: F) -> CalcResult<()>
        where F: Fn(f64) -> f64
    {
        let x = try!(self.get_float_cast());
        self.data.push(Value::Float(f(x)));
        Ok(())
    }

    fn perform_binop<F>(&mut self, f: F) -> CalcResult<()>
        where F: Fn(f64, f64) -> f64
    {
        let y = try!(self.get_float_cast());
        let x = try!(self.get_float_cast());
        self.data.push(Value::Float(f(x, y)));
        Ok(())
    }
}

fn main() {
    let mut calc = Calc::new();
    let mut args = std::env::args();
    args.next();
    if let Err(err) = calc.run(args) {
        println!("Error: {}", err);
        std::process::exit(1);
    }
    if let Err(err) = calc.run(Some("print").into_iter()) {
        println!("Error: {}", err);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
    }
}
