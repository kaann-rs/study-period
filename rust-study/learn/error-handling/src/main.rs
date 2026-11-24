// Kitap: 8. bölüm, Hata Yönetimi

use std::fmt;
use std::num::ParseIntError;

mod deep_dive;

fn main() {
    show_panic_and_result();

    println!("divide(10, 2) = {:?}", divide(10, 2));
    println!("divide(10, 0) = {:?}", divide(10, 0));

    match read_sample_number() {
        Ok(n) => println!("propagasyon OK: {n}"),
        Err(e) => println!("propagasyon HATA: {e}"),
    }

    show_from_conversion();
    deep_dive::run();
}

fn show_panic_and_result() {
    println!("panic bölümü: yalnızca yorumda (programı bilerek çökertmiyoruz)");
}

fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err(String::from("sıfıra bölme"))
    } else {
        Ok(a / b)
    }
}

fn parse_number(x: &str) -> Result<i32, ParseIntError> {
    let n: i32 = x.parse()?;
    Ok(n * 2)
}
fn read_sample_number() -> Result<i32, ParseIntError> {
    let a = parse_number("21")?;
    Ok(a)
}

#[derive(Debug)]
enum AppError {
    Parse(ParseIntError),
    Logic(String),
}
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::Parse(e) => write!(f, "ayrıştırma hatası: {e}"),
            AppError::Logic(message) => write!(f, "mantık hatası: {message}"),
        }
    }
}

impl From<ParseIntError> for AppError {
    fn from(e: ParseIntError) -> Self {
        AppError::Parse(e)
    }
}
fn show_from_conversion() {
    fn calculate(x: &str) -> Result<i32, AppError> {
        let n: i32 = x.parse()?;
        if n < 0 {
            return Err(AppError::Logic(String::from("negatif olamaz")));
        }
        Ok(n)
    }
    println!("calculate(\"7\")  = {:?}", calculate("7"));
    println!("calculate(\"xx\") = {:?}", calculate("xx"));
    println!("calculate(\"-3\") = {:?}", calculate("-3"));
    if let Err(e) = calculate("xx") {
        println!("Display: {e}");
    }
}

// Learn more
//
// You can learn panic and Result from these sources:
//   - The Rust Book, "Error Handling": https://doc.rust-lang.org/book/ch09-00-error-handling.html
//   - Rust by Example, "Error handling": https://doc.rust-lang.org/rust-by-example/error.html
//   - Comprehensive Rust, "Result": https://google.github.io/comprehensive-rust/error-handling/result.html
//   - Easy: Programiz, "Rust Error Handling": https://www.programiz.com/rust/error-handling
//   - Easy: Let's Get Rusty (video), "Error Handling in Rust": https://www.youtube.com/watch?v=wM6o70NAWUI
//
// You can learn the ? operator from these sources:
//   - The Rust Book, "The ? Operator Shortcut": https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#the--operator-shortcut
//   - Rust by Example, "Introducing ?": https://doc.rust-lang.org/rust-by-example/error/result/enter_question_mark.html
//   - Comprehensive Rust, "Try Operator": https://google.github.io/comprehensive-rust/error-handling/try.html
//   - Easy: Easy Rust, "The ? operator": https://dhghomon.github.io/easy_rust/Chapter_33.html
//
// You can learn error conversion with From from these sources:
//   - Comprehensive Rust, "Try Conversions": https://google.github.io/comprehensive-rust/error-handling/try-conversions.html
//   - Rust by Example, "Other uses of ?": https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/reenter_question_mark.html
//   - Standard library docs, "std::convert::From": https://doc.rust-lang.org/std/convert/trait.From.html
//   - Easy: Easy Rust, "Traits": https://dhghomon.github.io/easy_rust/Chapter_34.html
