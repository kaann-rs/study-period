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
