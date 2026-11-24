// Kitap: 11. bölüm, minigrep: Gerçek Bir Program

use std::env;
use std::error::Error;
use std::fs;

pub struct Config {
    pub query: String,
    pub file_path: String,
    pub ignore_case: bool,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next();

        let query = match args.next() {
            Some(query) => query,
            None => return Err("sorgu verilmedi"),
        };
        let file_path = match args.next() {
            Some(file_path) => file_path,
            None => return Err("dosya yolu verilmedi"),
        };

        let ignore_case = env::var("IGNORE_CASE").is_ok();

        Ok(Config {
            query,
            file_path,
            ignore_case,
        })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(&config.file_path)?;

    let lines = if config.ignore_case {
        search_ignore_case(&config.query, &contents)
    } else {
        search(&config.query, &contents)
    };

    for line in lines {
        println!("{line}");
    }
    Ok(())
}

pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    contents
        .lines()
        .filter(|line| line.contains(query))
        .collect()
}

pub fn search_ignore_case<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let query = query.to_lowercase();
    contents
        .lines()
        .filter(|line| line.to_lowercase().contains(&query))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_sensitive_gives_one_result() {
        let query = "hiç";
        let contents = "Ben bir hiçim\nSen kimsin\nHiç misin";
        assert_eq!(vec!["Ben bir hiçim"], search(query, contents));
    }

    #[test]
    fn ignore_case_gives_two_results() {
        let query = "hiç";
        let contents = "Ben bir hiçim\nSen kimsin\nHiç misin";
        assert_eq!(
            vec!["Ben bir hiçim", "Hiç misin"],
            search_ignore_case(query, contents)
        );
    }

    #[test]
    fn missing_argument_returns_error() {
        let args = vec![String::from("prog"), String::from("sorgu")].into_iter();
        assert!(Config::build(args).is_err());
    }
}

// Learn more
//
// You can learn how to build a config and return errors from these sources:
//   - The Rust Book, "Refactoring to Improve Modularity and Error Handling": https://doc.rust-lang.org/book/ch12-03-improving-error-handling-and-modularity.html
//   - Rust by Example, "Boxing errors": https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/boxing_errors.html
//   - Command Line Book, "Nicer error reporting": https://rust-cli.github.io/book/tutorial/errors.html
//   - Easy: Let's Get Rusty (video), "Writing a CLI App in Rust! - Part 1": https://www.youtube.com/watch?v=XYkiwsplDTg
//
// You can learn how to read a file from these sources:
//   - The Rust Book, "Reading a File": https://doc.rust-lang.org/book/ch12-02-reading-a-file.html
//   - Command Line Book, "First implementation": https://rust-cli.github.io/book/tutorial/impl-draft.html
//   - Standard library docs, "std::fs::read_to_string": https://doc.rust-lang.org/std/fs/fn.read_to_string.html
//   - Easy: Easy Rust, "Using files": https://dhghomon.github.io/easy_rust/Chapter_64.html
//
// You can learn how to return borrowed search results from these sources:
//   - The Rust Book, "Adding Functionality with Test Driven Development": https://doc.rust-lang.org/book/ch12-04-testing-the-librarys-functionality.html
//   - Comprehensive Rust, "Returning Borrows": https://google.github.io/comprehensive-rust/lifetimes/returning-borrows.html
//   - Rust by Example, "Functions": https://doc.rust-lang.org/rust-by-example/scope/lifetime/fn.html
//   - Easy: Let's Get Rusty (video), "Writing a CLI App in Rust! - Part 2": https://www.youtube.com/watch?v=AABHxixn6Cw
//
// You can learn environment variables from these sources:
//   - The Rust Book, "Working with Environment Variables": https://doc.rust-lang.org/book/ch12-05-working-with-environment-variables.html
//   - Standard library docs, "std::env::var": https://doc.rust-lang.org/std/env/fn.var.html
//   - Command Line Book, "Using config files": https://rust-cli.github.io/book/in-depth/config-files.html
//   - Easy: Let's Get Rusty (video), "Writing a CLI App in Rust! - Part 2": https://www.youtube.com/watch?v=AABHxixn6Cw
//
// You can learn how to test a command line program from these sources:
//   - The Rust Book, "Adding Functionality with Test Driven Development": https://doc.rust-lang.org/book/ch12-04-testing-the-librarys-functionality.html
//   - Command Line Book, "Testing": https://rust-cli.github.io/book/tutorial/testing.html
//   - Rust by Example, "Unit testing": https://doc.rust-lang.org/rust-by-example/testing/unit_testing.html
//   - Easy: Let's Get Rusty (video), "Writing a CLI App in Rust! - Part 2": https://www.youtube.com/watch?v=AABHxixn6Cw
