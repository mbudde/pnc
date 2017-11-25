use std::cmp::Ordering;

use words::Value;
use calc::Calc;
use errors::*;

impl Calc {
    pub fn builtin_div(&mut self) -> Result<()> {
        let y = try!(self.get_float_cast());
        let x = try!(self.get_float_cast());
        if y == 0.0 {
            return Err(ErrorKind::DivisionByZero.into());
        }
        self.data.push(Value::Float(x / y));
        Ok(())
    }

    pub fn builtin_print(&mut self) -> Result<()> {
        let val = try!(self.get_operand());
        println!("{}", val);
        Ok(())
    }

    pub fn builtin_duplicate(&mut self) -> Result<()> {
        let val = try!(self.get_operand());
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

    pub fn builtin_fold(&mut self) -> Result<()> {
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

    pub fn builtin_filter(&mut self) -> Result<()> {
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

    pub fn builtin_sum(&mut self) -> Result<()> {
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

    pub fn builtin_length(&mut self) -> Result<()> {
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

    pub fn builtin_swap(&mut self) -> Result<()> {
        let a = try!(self.get_operand());
        let b = try!(self.get_operand());
        self.data.push(a);
        self.data.push(b);
        Ok(())
    }

    pub fn builtin_over(&mut self) -> Result<()> {
        let a = try!(self.get_operand());
        let b = try!(self.get_operand());
        self.data.push(b.clone());
        self.data.push(a);
        self.data.push(b);
        Ok(())
    }

    pub fn builtin_repeat(&mut self) -> Result<()> {
        let n = try!(self.get_int_cast());
        let block = try!(self.get_block());
        for _ in 0..n {
            for word in &block {
                try!(self.run_one(word));
            }
        }
        Ok(())
    }

    pub fn builtin_roll3(&mut self) -> Result<()> {
        let a = try!(self.get_operand());
        let b = try!(self.get_operand());
        let c = try!(self.get_operand());
        self.data.push(b);
        self.data.push(c);
        self.data.push(a);
        Ok(())
    }

    pub fn builtin_apply(&mut self) -> Result<()> {
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

    pub fn builtin_min(&mut self) -> Result<()> {
        let a = try!(self.get_int());
        let b = try!(self.get_int());
        self.data.push(Value::Int(::std::cmp::min(a, b)));
        Ok(())
    }

    pub fn builtin_max(&mut self) -> Result<()> {
        let a = try!(self.get_int());
        let b = try!(self.get_int());
        self.data.push(Value::Int(::std::cmp::max(a, b)));
        Ok(())
    }

    pub fn builtin_cmp(&mut self) -> Result<()> {
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

