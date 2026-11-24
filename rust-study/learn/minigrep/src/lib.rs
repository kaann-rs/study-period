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
