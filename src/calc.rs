use std::cmp::Ordering;

use errors::*;
use words::{BuiltinWord, Operation, Value, Word};
use dict;

enum CalcState {
    Reading {
        block: Vec<Word>,
        level: u32,
    },
    Collecting {
        calc: Calc,
    },
}

pub struct Calc {
    dict: dict::Dictionary,
    data: Vec<Value>,
    state: Vec<CalcState>,
}

#[allow(new_without_default)]
impl Calc {
    pub fn new() -> Calc {
        Calc {
            dict: Default::default(),
            data: Vec::new(),
            state: Vec::new(),
        }
    }

    fn sub_calc(&self) -> Calc {
        Calc {
            dict: dict::Dictionary::with_parent(&self.dict),
            data: Vec::new(),
            state: Vec::new(),
        }
    }

    pub fn list_available_words(&self) {
        for (word, aliases) in self.dict.available_words() {
            if aliases.is_empty() {
                println!("{}", word);
            } else {
                println!("{} (aliases: {})", word, aliases.join(", "));
            }
        }
    }

    pub fn print_stack(&self) -> Result<()> {
        for val in &self.data {
            println!("{}", val);
        }
        Ok(())
    }

    pub fn run<I, T>(&mut self, iter: I) -> Result<()>
        where I: Iterator<Item = T>,
              T: AsRef<str>
    {
        for word in iter {
            let word = word.as_ref();
            try!(self.run_one(word));
        }
        Ok(())
    }

    #[allow(cyclomatic_complexity)]
    pub fn run_one(&mut self, word: &str) -> Result<()> {
        match self.state.pop() {
            Some(CalcState::Reading { mut block, mut level }) => {
                trace!("reading {}", word);
                if word == "}" && level == 0 {
                    self.data.push(Value::Block(block));
                } else {
                    block.push(word.to_owned());
                    if word == "}" {
                        level = level - 1;
                    } else if word == "{" {
                        level = level + 1;
                    }
                    self.state.push(CalcState::Reading {
                        block: block,
                        level: level,
                    });
                }
            }
            Some(CalcState::Collecting { mut calc }) => {
                trace!("collecting {}", word);
                if word == "]" {
                    self.data.push(Value::Vector(calc.data));
                } else {
                    if let Some(op) = self.dict.lookup(word) {
                        if let Operation::Builtin(::words::BuiltinWord::Arg) = *op {
                            match self.state.pop() {
                                Some(CalcState::Collecting { calc: mut parent }) => {
                                    let val = parent.data.pop()
                                        .ok_or::<Error>(ErrorKind::MissingOperand.into())?;
                                    calc.data.push(val);
                                    self.state.push(CalcState::Collecting { calc: parent });
                                }
                                Some(CalcState::Reading { block, level }) => {
                                    let val = self.data.pop()
                                        .ok_or::<Error>(ErrorKind::MissingOperand.into())?;
                                    calc.data.push(val);
                                    self.state.push(CalcState::Reading { block: block, level: level });
                                }
                                None => {
                                    let val = self.data.pop()
                                        .ok_or::<Error>(ErrorKind::MissingOperand.into())?;
                                    calc.data.push(val);
                                }
                            }
                        } else {
                            try!(calc.run_one(word));
                        }
                    } else {
                        try!(calc.run_one(word));
                    }
                    self.state.push(CalcState::Collecting { calc: calc });
                }
            }
            None => {
                trace!("executing {}", word);
                if word.starts_with(',') {
                    self.data.push(Value::QuotedWord(word[1..].to_owned()));
                } else if word == "{" {
                    self.state.push(CalcState::Reading {
                        block: Vec::new(),
                        level: 0,
                    });
                } else if word == "[" {
                    let state = CalcState::Collecting { calc: self.sub_calc() };
                    self.state.push(state);
                } else if let Some(op) = self.dict.lookup(word) {
                    match *op {
                        Operation::Builtin(builtin) => {
                            try!(self.run_builtin(builtin));
                        }
                        Operation::Block(ref block) => {
                            trace!("executing block: {:?}", block);
                            try!(self.run(block.into_iter()));
                        }
                    }
                } else {
                    let val = Value::parse(word)
                        .ok_or::<Error>(ErrorKind::WordParseError(word.to_owned()).into())?;
                    self.data.push(val);
                }
            }
        }
        Ok(())
    }

    #[allow(cyclomatic_complexity)]
    fn run_builtin(&mut self, word: BuiltinWord) -> Result<()> {
        use words::BuiltinWord::*;
        match word {
            Add => self.perform_binop(::std::ops::Add::add, ::std::ops::Add::add),
            Sub => self.perform_binop(::std::ops::Sub::sub, ::std::ops::Sub::sub),
            Mul => self.perform_binop(::std::ops::Mul::mul, ::std::ops::Mul::mul),
            Div => {
                let y = try!(self.get_float_cast());
                let x = try!(self.get_float_cast());
                if y == 0.0 {
                    return Err(ErrorKind::DivisionByZero.into());
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
                let stdin = ::std::io::stdin();
                let stdin = stdin.lock();
                let mut vec = vec![];
                for line in stdin.lines() {
                    let line = line.unwrap();
                    if let Some(val) = Value::parse(&line) {
                        vec.push(val);
                    }
                }
                self.data.push(Value::Vector(vec));
                Ok(())
            }
            Map => {
                let block = try!(self.get_block());
                let vec = try!(self.get_vector());
                let mut result = Vec::with_capacity(vec.len());
                for val in vec {
                    let mut sub_calc = self.sub_calc();
                    sub_calc.data.push(val);
                    for word in &block {
                        try!(sub_calc.run_one(word));
                    }
                    if let Some(res) = sub_calc.data.pop() {
                        result.push(res);
                    } else {
                        return Err(ErrorKind::BlockNoResult.into());
                    }
                }
                self.data.push(Value::Vector(result));
                Ok(())
            }
            Fold => {
                let block = try!(self.get_block());
                let init = try!(self.get_operand());
                let values = try!(self.get_vector());

                let mut sub_calc = self.sub_calc();
                sub_calc.data.push(init);
                for val in values {
                    sub_calc.data.push(val);
                    for word in &block {
                        try!(sub_calc.run_one(word));
                    }
                }
                if let Some(res) = sub_calc.data.pop() {
                    self.data.push(res);
                    Ok(())
                } else {
                    Err(ErrorKind::BlockNoResult.into())
                }
            }
            Filter => {
                let block = try!(self.get_block());
                let values = try!(self.get_vector());
                let mut result = Vec::with_capacity(values.len());
                for val in values {
                    let mut sub_calc = self.sub_calc();
                    sub_calc.data.push(val.clone());
                    for word in &block {
                        try!(sub_calc.run_one(word));
                    }
                    if let Some(res) = sub_calc.data.pop() {
                        match res {
                            Value::Int(x) if x != 0 => { result.push(val); }
                            Value::Bool(true) => { result.push(val); }
                            _ => {}
                        }
                    } else {
                        return Err(ErrorKind::BlockNoResult.into());
                    }
                }
                result.shrink_to_fit();
                self.data.push(Value::Vector(result));
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
                        return Err(ErrorKind::WrongTypeOperand(a, "vector").into());
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
                        return Err(ErrorKind::WrongTypeOperand(a, "vector").into());
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
            Over => {
                let a = try!(self.get_operand());
                let b = try!(self.get_operand());
                self.data.push(b.clone());
                self.data.push(a);
                self.data.push(b);
                Ok(())
            }
            Repeat => {
                let n = try!(self.get_int_cast());
                let block = try!(self.get_block());
                for _ in 0..n {
                    for word in &block {
                        try!(self.run_one(word));
                    }
                }
                Ok(())
            }
            Roll3 => {
                let a = try!(self.get_operand());
                let b = try!(self.get_operand());
                let c = try!(self.get_operand());
                self.data.push(b);
                self.data.push(c);
                self.data.push(a);
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
            Apply => {
                let op = try!(self.get_operand());
                match op {
                    Value::QuotedWord(word) => {
                        try!(self.run_one(&word));
                    }
                    Value::Block(block) => {
                        for word in &block {
                            try!(self.run_one(word));
                        }
                    }
                    _ => {}
                }
                Ok(())
            }
            Arg => {
                Ok(())
            }
            Min => {
                let a = try!(self.get_int());
                let b = try!(self.get_int());
                self.data.push(Value::Int(::std::cmp::min(a, b)));
                Ok(())
            }
            Max => {
                let a = try!(self.get_int());
                let b = try!(self.get_int());
                self.data.push(Value::Int(::std::cmp::max(a, b)));
                Ok(())
            }
            Cmp => {
                let a = try!(self.get_int());
                let b = try!(self.get_int());
                let cmp = match b.cmp(&a) {
                    Ordering::Less => -1,
                    Ordering::Equal => 0,
                    Ordering::Greater => 1,
                };
                self.data.push(Value::Int(cmp));
                Ok(())
            }
        }
    }

    fn get_operand(&mut self) -> Result<Value> {
        self.data.pop().ok_or(ErrorKind::MissingOperand.into())
    }

    #[allow(dead_code)]
    fn get_int(&mut self) -> Result<i64> {
        self.get_operand().and_then(|val| {
            val.as_int().ok_or(ErrorKind::WrongTypeOperand(val, "int").into())
        })
    }

    fn get_int_cast(&mut self) -> Result<i64> {
        self.get_operand().and_then(|val| {
            val.as_int_cast().ok_or(ErrorKind::WrongTypeOperand(val, "int or float").into())
        })
    }

    #[allow(dead_code)]
    fn get_float(&mut self) -> Result<f64> {
        self.get_operand().and_then(|val| {
            val.as_float().ok_or(ErrorKind::WrongTypeOperand(val, "float").into())
        })
    }

    fn get_float_cast(&mut self) -> Result<f64> {
        self.get_operand().and_then(|val| {
            val.as_float_cast().ok_or(ErrorKind::WrongTypeOperand(val, "int or float").into())
        })
    }

    fn get_block(&mut self) -> Result<Vec<Word>> {
        self.get_operand().and_then(|val| {
            val.into_block().map_err(|v| ErrorKind::WrongTypeOperand(v, "block").into())
        })
    }

    fn get_vector(&mut self) -> Result<Vec<Value>> {
        self.get_operand().and_then(|val| {
            val.into_vector().map_err(|v| ErrorKind::WrongTypeOperand(v, "vector").into())
        })
    }

    fn get_word(&mut self) -> Result<Word> {
        self.get_operand().and_then(|val| {
            val.into_word().map_err(|v| ErrorKind::WrongTypeOperand(v, "quoted word").into())
        })
    }

    #[allow(dead_code)]
    fn perform_unary<F>(&mut self, f: F) -> Result<()>
        where F: Fn(f64) -> f64
    {
        let x = try!(self.get_float_cast());
        self.data.push(Value::Float(f(x)));
        Ok(())
    }

    fn perform_binop<F, G>(&mut self, f: F, g: G) -> Result<()>
        where F: Fn(f64, f64) -> f64,
              G: Fn(i64, i64) -> i64
    {
        let y = try!(self.get_operand());
        let x = try!(self.get_operand());
        match (x, y) {
            (Value::Int(x),   Value::Int(y))   => self.data.push(Value::Int(g(x, y))),
            (Value::Float(x), Value::Float(y)) => self.data.push(Value::Float(f(x, y))),
            (Value::Int(x),   Value::Float(y)) => self.data.push(Value::Float(f(x as f64, y))),
            (Value::Float(x), Value::Int(y))   => self.data.push(Value::Float(f(x, y as f64))),
            (Value::Int(_), y) | (Value::Float(_), y) => {
                return Err(ErrorKind::WrongTypeOperand(y, "int or float").into())
            }
            (x, _) => {
                return Err(ErrorKind::WrongTypeOperand(x, "int or float").into())
            }
        }
        Ok(())
    }
}
