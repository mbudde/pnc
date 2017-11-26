#![allow(unknown_lints)]

#[macro_use] extern crate log;
extern crate env_logger;
extern crate shlex;
extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate num;

use std::io::prelude::*;
use std::fs::File;

use clap::{App, AppSettings, Arg};

use errors::*;

mod words;
mod builtins;

mod errors {
    use words::Word;
    error_chain! {

        errors {
            DivisionByZero {
                description("division by zero")
            }
            MissingOperand {
                description("operation needs an operand but stack is empty")
            }
            WrongTypeOperand(value: ::words::Value, expected: &'static str) {
                description("operand has a wrong type")
                display("operand has a wrong type, got operand '{}' (of type {}) but expected type {}", value, value.type_of(), expected)
            }
            BlockNoResult {
                description("block left no result on the stack")
            }
            WordParseError(word: Word) {
                description("could not parse word as number or operation")
                display("could not parse word '{}' as number or operation", word)
            }
            BigIntTooLarge {
                display("bigint is too large to convert to float")
            }
            UnknownWord(word: Word) {
                display("the word '{}' does not exist", word)
            }
        }
    }
}

mod calc;
mod dict;

quick_main!(run);

fn run() -> Result<()> {
    env_logger::init().chain_err(|| "failed to setup logging")?;

    let args = App::new("Postfix Notation Calculator")
        .version("0.1")
        .author("Michael Budde <mbudde@gmail.com>")
        .setting(AppSettings::TrailingVarArg)
        .setting(AppSettings::AllowLeadingHyphen)
        .setting(AppSettings::UnifiedHelpMessage)
        .arg(Arg::from_usage("[WORD]... 'Words to execute'").use_delimiter(false))
        .arg_from_usage("-q --quiet 'Do not print stack before exiting'")
        .arg_from_usage("-l --list 'List all defined words'")
        .get_matches();

    let mut calc = calc::Calc::new();

    let builtin_prelude = include_str!("../prelude.pnc");
    let prelude_words = shlex::Shlex::new(builtin_prelude);
    calc.run(prelude_words).chain_err(|| "could not execute builtin prelude")?;

    if let Some(mut p) = std::env::home_dir() {
        p.push(".config/pnc/prelude.pnc");
        match File::open(&p) {
            Ok(mut prelude_file) => {
                let mut prelude = String::new();
                prelude_file.read_to_string(&mut prelude)
                    .chain_err(|| format!("could not read user prelude {:?}", p))?;

                let prelude_words = shlex::Shlex::new(&prelude);
                calc.run(prelude_words).chain_err(|| "failed to execute user prelude")?;
            }
            Err(ref e) if p.exists() => {
                eprintln!("Warning: failed to open user prelude {:?}: {}", p, e);
            }
            _ => {}
        }
    }

    if args.is_present("list") {
        calc.list_available_words();
    } else {
        if let Some(words) = args.values_of("WORD") {
            calc.run(words).chain_err(|| "failed to execute words in arguments")?;
        } else {
            use std::io::BufRead;
            let stdin = ::std::io::stdin();
            let stdin = stdin.lock();
            for line in stdin.lines() {
                let line = line.unwrap();
                let words = shlex::Shlex::new(&line);
                calc.run(words).chain_err(|| "failed to execute words from standard in")?;
            }
        }

        if !args.is_present("quiet") {
            calc.print_stack().chain_err(|| "failed to print stack")?;
        }
    }
    Ok(())
}
