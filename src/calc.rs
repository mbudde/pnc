use std::fmt;
use std::error::Error;
use std::mem;

use words::{BuiltinWord, Operation, Value, Word};
use dict;

#[derive(Debug)]
pub enum CalcError {
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

pub type CalcResult<T> = Result<T, CalcError>;

enum CalcState {
    Executing,
    Reading(Vec<Word>),
}

pub struct Calc {
    dict: dict::Dictionary,
    data: Vec<Value>,
    state: CalcState,
}

impl Calc {
    pub fn new() -> Calc {
        Calc {
            dict: Default::default(),
            data: Vec::new(),
            state: CalcState::Executing,
        }
    }

    pub fn print_stack(&self) -> CalcResult<()> {
        for val in &self.data {
            println!("{}", val);
        }
        Ok(())
    }

    pub fn run<'a, I, T>(&mut self, iter: I) -> CalcResult<()>
        where I: Iterator<Item = T>,
              T: AsRef<str>
    {
        for word in iter {
            let word = word.as_ref();
            try!(self.run_one(word));
        }
        Ok(())
    }

    pub fn run_one(&mut self, word: &str) -> CalcResult<()> {
        trace!("executing {:?}", word);
        let state = mem::replace(&mut self.state, CalcState::Executing);
        match state {
            CalcState::Reading(mut block) => {
                if word == "]" {
                    self.data.push(Value::Block(block));
                } else {
                    block.push(word.to_string());
                    self.state = CalcState::Reading(block);
                }
            }
            CalcState::Executing => {
                if word.starts_with(",") {
                    self.data.push(Value::QuotedWord(word[1..].to_string()));
                } else if word == "[" {
                    self.state = CalcState::Reading(Vec::new());
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
        }
        Ok(())
    }

    fn run_builtin(&mut self, word: BuiltinWord) -> CalcResult<()> {
        use words::BuiltinWord::*;
        match word {
            Add => self.perform_binop(::std::ops::Add::add),
            Sub => self.perform_binop(::std::ops::Sub::sub),
            Mul => self.perform_binop(::std::ops::Mul::mul),
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
                use ::std::io::BufRead;
                let stdin = ::std::io::stdin();
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
            Def => {
                let block = try!(self.get_block());
                let name = try!(self.get_word());
                self.dict.insert(name, Operation::Block(block));
                Ok(())
            }
            Alias => {
                let val = try!(self.get_word());
                let name = try!(self.get_word());
                self.dict.insert_alias(name, val);
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

    fn get_word(&mut self) -> CalcResult<Word> {
        self.get_operand().and_then(|val| val.as_word().ok_or(CalcError::WrongTypeOperand))
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
