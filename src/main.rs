#[macro_use]
extern crate log;
extern crate env_logger;
extern crate shlex;

use std::io::prelude::*;
use std::fs::File;

mod calc;
mod dict;
mod words;


fn main() {
    env_logger::init().unwrap();
    let mut calc = calc::Calc::new();

    let builtin_prelude = include_str!("../prelude.pnc");
    let prelude_words = shlex::Shlex::new(&builtin_prelude);
    if let Err(err) = calc.run(prelude_words) {
        println!("Error while executing builtin prelude: {}", err);
        std::process::exit(1);
    }

    if let Some(mut p) = std::env::home_dir() {
        p.push(".prelude.pnc");
        if let Ok(mut prelude_file) = File::open(p) {
            let mut prelude = String::new();
            prelude_file.read_to_string(&mut prelude).unwrap();

            let prelude_words = shlex::Shlex::new(&prelude);
            if let Err(err) = calc.run(prelude_words) {
                println!("Error while executing prelude: {}", err);
                std::process::exit(1);
            }
        }
    }

    let mut args = std::env::args();
    args.next();
    if let Err(err) = calc.run(args) {
        println!("Error: {}", err);
        std::process::exit(1);
    }
    calc.print_stack().unwrap();
}

#[cfg(test)]
mod tests {

    #[test]
    fn test() {
    }

}
