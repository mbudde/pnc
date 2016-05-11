#[macro_use] extern crate log;
extern crate env_logger;
extern crate shlex;
extern crate clap;

use std::io::prelude::*;
use std::fs::File;

use clap::{App, AppSettings, Arg};

mod calc;
mod dict;
mod words;


fn main() {
    env_logger::init().unwrap();

    let args = App::new("Postfix Notation Calculator")
        .version("0.1")
        .author("Michael Budde")
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

    if args.is_present("list") {
        calc.list_available_words();
    } else {
        if let Some(words) = args.values_of("WORD") {
            if let Err(err) = calc.run(words) {
                println!("Error: {}", err);
                std::process::exit(1);
            }
        }
        if !args.is_present("quiet") {
            calc.print_stack().unwrap();
        }
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn test() {
    }

}
