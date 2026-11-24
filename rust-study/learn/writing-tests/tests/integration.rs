// Kitap: 10. bölüm, Test Yazma

use writing_tests::{add, count_words};

#[test]
fn add_from_outside() {
    assert_eq!(add(40, 2), 42);
}

#[test]
fn count_words_from_outside() {
    assert_eq!(count_words("merhaba dünya"), 2);
}

// Learn more
//
// You can learn integration tests from these sources:
//   - The Rust Book, "Integration Tests": https://doc.rust-lang.org/book/ch11-03-test-organization.html#integration-tests
//   - Rust by Example, "Integration testing": https://doc.rust-lang.org/rust-by-example/testing/integration_testing.html
//   - Comprehensive Rust, "Other Types of Tests": https://google.github.io/comprehensive-rust/testing/other.html
//   - Easy: Let's Get Rusty (video), "Testing in Rust - Part 2": https://www.youtube.com/watch?v=-L4nKAlmH3M
