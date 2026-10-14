mod expr;
mod parser;
mod scanner;
#[cfg(test)]
mod tests;
mod token;
mod util;

use crate::expr::evolute;
use crate::parser::Parser;
use crate::scanner::Scanner;
use std::io::Write;

macro_rules! errs {
    ($($arg:tt)*) =>
    {
        {
            Self::error(0, $($arg)*);
        }
    };
}

const TOKEN_TYPE: usize = size_of::<Scanner>();

pub struct Lox;

impl Lox {
    pub const PI: f32 = 3.14;

    pub fn run_file(path: &str) -> Result<(), std::io::Error> {
        let byte: Vec<u8> = std::fs::read(path)?;

        Self::run(&String::from_utf8(byte).unwrap());

        println!("Size: {TOKEN_TYPE}");

        Ok(())
    }

    pub fn run_prompt() {
        let stdin = std::io::stdin();
        let mut input = String::new();

        loop {
            print!("> ");
            std::io::stdout().flush().unwrap();

            input.clear();

            match stdin.read_line(&mut input) {
                Ok(v) => {
                    if v == 0 {
                        break;
                    }

                    Self::run(input.trim());
                }
                Err(_) => break,
            }
        }
    }

    fn run(source: &str) -> String {
        let scan: Scanner = Scanner::new(source);
        let tokens = scan.scan_tokens();

        for token in &tokens {
            let _ = format!("{token}");
        }

        let mut parser = Parser::new(tokens);

        let expr = parser.expression();

        println!("Sonuç: {:?}", evolute(expr));

        let value = String::new();

        errs!("Error");

        value
    }

    fn error(line: i32, msg: &str) {
        Self::_report(line, msg, "");
    }

    fn _report(line: i32, msg: &str, whe: &str) -> String {
        format!("{line} {msg} {whe}")
    }
}
