// Kitap: 10. bölüm, Test Yazma

use writing_tests::{add, check_age, count_words};

fn main() {
    println!("add(2, 3) = {}", add(2, 3));
    println!(
        "count_words(\"  bir iki üç \") = {}",
        count_words("  bir iki üç ")
    );
    println!("check_age(30) = {}", check_age(30));
    println!("(testleri koşmak için: cargo test -p writing-tests)");
}

// Learn more
//
// You can learn packages with a library and a binary from these sources:
//   - The Rust Book, "Packages and Crates": https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html
//   - Cargo Book, "Package Layout": https://doc.rust-lang.org/cargo/guide/project-layout.html
//   - Rust by Example, "Using a Library": https://doc.rust-lang.org/rust-by-example/crates/using_lib.html
//   - Easy: Programiz, "Rust Crate and Package": https://www.programiz.com/rust/crate-and-package
//
// You can learn how to run tests from these sources:
//   - The Rust Book, "Controlling How Tests Are Run": https://doc.rust-lang.org/book/ch11-02-running-tests.html
//   - Cargo Book, "Tests": https://doc.rust-lang.org/cargo/guide/tests.html
//   - Rust by Example, "Tests": https://doc.rust-lang.org/rust-by-example/cargo/test.html
//   - Easy: Easy Rust, "Testing": https://dhghomon.github.io/easy_rust/Chapter_58.html
