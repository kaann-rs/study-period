// Kitap: 20. bölüm, Makrolar

use macros::make_answer;

make_answer!();

#[test]
fn answer_is_42() {
    assert_eq!(answer(), 42);
}

// Learn more
//
// You can learn how to use and test procedural macros from these sources:
//   - docs.rs, "trybuild": https://docs.rs/trybuild/latest/trybuild/
//   - dtolnay, "cargo-expand": https://github.com/dtolnay/cargo-expand
//   - dtolnay, "proc-macro-workshop": https://github.com/dtolnay/proc-macro-workshop
//   - The Rust Book, "Function-Like Macros": https://doc.rust-lang.org/book/ch20-05-macros.html#function-like-macros
