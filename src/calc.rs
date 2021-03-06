use num::{BigInt, ToPrimitive};
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
    pub data: Vec<Value>,
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

    pub fn sub_calc(&self) -> Calc {
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
        where I: IntoIterator<Item = T>,
              T: AsRef<str>
    {
        for word in iter.into_iter() {
            let word = word.as_ref();
            self.run_one(word)
                .chain_err(|| format!("error while evaluating word '{}'", word))?;
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
                        level -= 1;
                    } else if word == "{" {
                        level += 1;
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
                                        .ok_or_else::<Error, _>(|| ErrorKind::MissingOperand.into())?;
                                    calc.data.push(val);
                                    self.state.push(CalcState::Collecting { calc: parent });
                                }
                                Some(CalcState::Reading { block, level }) => {
                                    let val = self.data.pop()
                                        .ok_or_else::<Error, _>(|| ErrorKind::MissingOperand.into())?;
                                    calc.data.push(val);
                                    self.state.push(CalcState::Reading { block: block, level: level });
                                }
                                None => {
                                    let val = self.data.pop()
                                        .ok_or_else::<Error, _>(|| ErrorKind::MissingOperand.into())?;
                                    calc.data.push(val);
                                }
                            }
                        } else {
                            calc.run_one(word)?;
                        }
                    } else {
                        calc.run_one(word)?;
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
                            self.run_builtin(builtin)?;
                        }
                        Operation::Value(Value::Block(ref block)) => {
                            trace!("executing block: {:?}", block);
                            self.run(block)?;
                        }
                        Operation::Value(ref v) => {
                            self.data.push(v.clone());
                        }
                    }
                } else {
                    let val = Value::parse(word)
                        .ok_or_else::<Error, _>(|| ErrorKind::WordParseError(word.to_owned()).into())?;
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
            Div => self.builtin_div(),
            Mod => self.builtin_mod(),
            Sqrt => self.perform_unary(f64::sqrt),
            Pow => self.builtin_pow(),
            Exp => self.perform_unary(f64::exp),
            Log => self.perform_float_binary(f64::log),
            Ln => self.perform_unary(f64::ln),
            Sin => self.perform_unary(f64::sin),
            Cos => self.perform_unary(f64::cos),
            Tan => self.perform_unary(f64::tan),
            Asin => self.perform_unary(f64::asin),
            Acos => self.perform_unary(f64::acos),
            Atan => self.perform_unary(f64::atan),

            Print => self.builtin_print(),
            Dump => self.print_stack(),
            Pop => {
                self.data.pop();
                Ok(())
            }
            Duplicate => self.builtin_duplicate(),
            Stdin => self.builtin_stdin(),
            Map => self.builtin_map(),
            Fold => self.builtin_fold(),
            Fold1 => self.builtin_fold1(),
            Filter => self.builtin_filter(),
            Length => self.builtin_length(),
            Swap => self.builtin_swap(),
            Over => self.builtin_over(),
            Repeat => self.builtin_repeat(),
            Roll3 => self.builtin_roll3(),
            Def => {
                let value = self.get_operand()?;
                let name = self.get_word()?;
                self.dict.insert(name, Operation::Value(value));
                Ok(())
            }
            Alias => {
                let val = self.get_word()?;
                let name = self.get_word()?;
                if self.dict.lookup(&val).is_none() {
                    return Err(ErrorKind::UnknownWord(val).into());
                }
                self.dict.insert_alias(name, val);
                Ok(())
            }
            Apply => self.builtin_apply(),
            Arg => {
                Ok(())
            }
            Min => self.builtin_min(),
            Max => self.builtin_max(),
            Cmp => self.builtin_cmp(),
            If => self.buildin_if(),
        }
    }

    pub fn get_operand(&mut self) -> Result<Value> {
        self.data.pop().ok_or_else(|| ErrorKind::MissingOperand.into())
    }

    pub fn get_int(&mut self) -> Result<BigInt> {
        self.get_operand().and_then(|val| {
            val.as_int().ok_or_else(|| ErrorKind::WrongTypeOperand(val, "int").into())
        })
    }

    pub fn get_int_cast(&mut self) -> Result<i64> {
        self.get_operand().and_then(|val| {
            val.as_int_cast().ok_or_else(|| ErrorKind::WrongTypeOperand(val, "int or float").into())
        })
    }

    #[allow(dead_code)]
    pub fn get_float(&mut self) -> Result<f64> {
        self.get_operand().and_then(|val| {
            val.as_float().ok_or_else(|| ErrorKind::WrongTypeOperand(val, "float").into())
        })
    }

    pub fn get_float_cast(&mut self) -> Result<f64> {
        self.get_operand().and_then(|val| {
            val.as_float_cast().ok_or_else(|| ErrorKind::WrongTypeOperand(val, "int or float").into())
        })
    }

    pub fn get_block(&mut self) -> Result<Vec<Word>> {
        self.get_operand().and_then(|val| {
            val.into_block().map_err(|v| ErrorKind::WrongTypeOperand(v, "block").into())
        })
    }

    pub fn get_vector(&mut self) -> Result<Vec<Value>> {
        self.get_operand().and_then(|val| {
            val.into_vector().map_err(|v| ErrorKind::WrongTypeOperand(v, "vector").into())
        })
    }

    pub fn get_word(&mut self) -> Result<Word> {
        self.get_operand().and_then(|val| {
            val.into_word().map_err(|v| ErrorKind::WrongTypeOperand(v, "quoted word").into())
        })
    }

    fn perform_unary<F>(&mut self, f: F) -> Result<()>
        where F: Fn(f64) -> f64
    {
        let x = self.get_float_cast()?;
        self.data.push(Value::Float(f(x)));
        Ok(())
    }

    fn perform_float_binary<F>(&mut self, f: F) -> Result<()>
        where F: Fn(f64, f64) -> f64
    {
        let y = self.get_float_cast()?;
        let x = self.get_float_cast()?;
        self.data.push(Value::Float(f(x, y)));
        Ok(())
    }

    fn perform_binop<F, G>(&mut self, f: F, g: G) -> Result<()>
        where F: Fn(f64, f64) -> f64,
              G: Fn(BigInt, BigInt) -> BigInt
    {
        let y = self.get_operand()?;
        let x = self.get_operand()?;
        match (x, y) {
            (Value::Int(x),   Value::Int(y))   => self.data.push(Value::Int(g(x, y))),
            (Value::Float(x), Value::Float(y)) => self.data.push(Value::Float(f(x, y))),
            (Value::Int(x),   Value::Float(y)) => {
                let x_f = x.to_f64().ok_or::<Error>(ErrorKind::BigIntTooLarge.into())?;
                self.data.push(Value::Float(f(x_f, y)));
            }
            (Value::Float(x), Value::Int(y)) => {
                let y_f = y.to_f64().ok_or::<Error>(ErrorKind::BigIntTooLarge.into())?;
                self.data.push(Value::Float(f(x, y_f)));
            }
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
