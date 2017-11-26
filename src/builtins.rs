use std::cmp::Ordering;

use num::Zero;
use num::bigint::ToBigInt;

use words::Value;
use calc::Calc;
use errors::*;


impl Calc {
    pub fn builtin_div(&mut self) -> Result<()> {
        let y = self.get_float_cast()?;
        let x = self.get_float_cast()?;
        if y == 0.0 {
            return Err(ErrorKind::DivisionByZero.into());
        }
        self.data.push(Value::Float(x / y));
        Ok(())
    }

    pub fn builtin_print(&mut self) -> Result<()> {
        let val = self.get_operand()?;
        println!("{}", val);
        Ok(())
    }

    pub fn builtin_duplicate(&mut self) -> Result<()> {
        let val = self.get_operand()?;
        self.data.push(val.clone());
        self.data.push(val);
        Ok(())
    }

    pub fn builtin_stdin(&mut self) -> Result<()> {
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


    pub fn builtin_map(&mut self) -> Result<()> {
        let block = self.get_block()?;
        let vec = self.get_vector()?;
        let mut result = Vec::with_capacity(vec.len());
        for val in vec {
            let mut sub_calc = self.sub_calc();
            sub_calc.data.push(val);
            sub_calc.run(&block)?;
            if let Some(res) = sub_calc.data.pop() {
                result.push(res);
            } else {
                return Err(ErrorKind::BlockNoResult.into());
            }
        }
        self.data.push(Value::Vector(result));
        Ok(())
    }

    pub fn builtin_fold(&mut self) -> Result<()> {
        let block = self.get_block()?;
        let init = self.get_operand()?;
        let values = self.get_vector()?;

        let mut sub_calc = self.sub_calc();
        sub_calc.data.push(init);
        for val in values {
            sub_calc.data.push(val);
            sub_calc.run(&block)?;
        }
        if let Some(res) = sub_calc.data.pop() {
            self.data.push(res);
            Ok(())
        } else {
            Err(ErrorKind::BlockNoResult.into())
        }
    }

    pub fn builtin_fold1(&mut self) -> Result<()> {
        let block = self.get_block()?;
        let mut values = self.get_vector()?.into_iter();

        if let Some(init) = values.next() {
            let mut sub_calc = self.sub_calc();
            sub_calc.data.push(init);
            for val in values {
                sub_calc.data.push(val);
                sub_calc.run(&block)?;
            }
            let res = sub_calc.data.pop().ok_or::<Error>(ErrorKind::BlockNoResult.into())?;
            self.data.push(res);
        }
        Ok(())
    }

    pub fn builtin_filter(&mut self) -> Result<()> {
        let block = self.get_block()?;
        let values = self.get_vector()?;
        let mut result = Vec::with_capacity(values.len());
        for val in values {
            let mut sub_calc = self.sub_calc();
            sub_calc.data.push(val.clone());
            sub_calc.run(&block)?;
            if let Some(res) = sub_calc.data.pop() {
                match res {
                    Value::Int(ref x) if !x.is_zero() => { result.push(val); }
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

    pub fn builtin_length(&mut self) -> Result<()> {
        let len = {
            let a = self.get_operand()?;
            if let Value::Vector(ref vec) = a {
                vec.len()
            } else {
                return Err(ErrorKind::WrongTypeOperand(a, "vector").into());
            }
        };
        self.data.push(Value::Int(len.to_bigint().unwrap()));
        Ok(())
    }

    pub fn builtin_swap(&mut self) -> Result<()> {
        let a = self.get_operand()?;
        let b = self.get_operand()?;
        self.data.push(a);
        self.data.push(b);
        Ok(())
    }

    pub fn builtin_over(&mut self) -> Result<()> {
        let a = self.get_operand()?;
        let b = self.get_operand()?;
        self.data.push(b.clone());
        self.data.push(a);
        self.data.push(b);
        Ok(())
    }

    pub fn builtin_repeat(&mut self) -> Result<()> {
        let n = self.get_int_cast()?;
        let block = self.get_block()?;
        for _ in 0..n {
            self.run(&block)?;
        }
        Ok(())
    }

    pub fn builtin_roll3(&mut self) -> Result<()> {
        let a = self.get_operand()?;
        let b = self.get_operand()?;
        let c = self.get_operand()?;
        self.data.push(b);
        self.data.push(c);
        self.data.push(a);
        Ok(())
    }

    pub fn builtin_apply(&mut self) -> Result<()> {
        let op = self.get_operand()?;
        match op {
            Value::QuotedWord(word) => {
                self.run_one(&word)?;
            }
            Value::Block(block) => {
                self.run(block)?;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn builtin_min(&mut self) -> Result<()> {
        let a = self.get_int()?;
        let b = self.get_int()?;
        self.data.push(Value::Int(::std::cmp::min(a, b)));
        Ok(())
    }

    pub fn builtin_max(&mut self) -> Result<()> {
        let a = self.get_int()?;
        let b = self.get_int()?;
        self.data.push(Value::Int(::std::cmp::max(a, b)));
        Ok(())
    }

    pub fn builtin_cmp(&mut self) -> Result<()> {
        let a = self.get_int()?;
        let b = self.get_int()?;
        let cmp = match b.cmp(&a) {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        };
        self.data.push(Value::Int(cmp.to_bigint().unwrap()));
        Ok(())
    }

    pub fn buildin_if(&mut self) -> Result<()> {
        let else_block = self.get_operand()?;
        let then_block = self.get_operand()?;
        let test = self.get_int()?;
        let mut run_block = |block| -> Result<()> {
            match block {
                Value::Block(block) => self.run(block),
                v => {
                    self.data.push(v);
                    Ok(())
                }
            }
        };
        if !test.is_zero() {
            run_block(then_block)
                .chain_err(|| "error while evaluating then-part of if command")
        } else {
            run_block(else_block)
                .chain_err(|| "error while evaluating else-part of if command")
        }
    }
}

