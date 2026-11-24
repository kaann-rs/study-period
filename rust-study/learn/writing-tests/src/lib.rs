// Kitap: 10. bölüm, Test Yazma

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn check_age(age: i32) -> i32 {
    if age < 0 {
        panic!("yaş negatif olamaz: {age}");
    }
    age
}

pub fn count_words(text: &str) -> usize {
    trim_text(text).split_whitespace().count()
}

fn trim_text(text: &str) -> &str {
    text.trim()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_works() {
        assert_eq!(add(2, 3), 5);
        assert_ne!(add(2, 2), 5);
    }

    #[test]
    fn logic_is_correct() {
        let result = add(10, -4);
        assert!(result > 0, "beklenen pozitif, gelen {result}");
    }

    #[test]
    fn private_function_can_be_tested() {
        assert_eq!(trim_text("  selam  "), "selam");
    }

    #[test]
    fn words_are_counted() {
        assert_eq!(count_words("  bir  iki üç "), 3);
    }

    #[test]
    #[should_panic(expected = "negatif olamaz")]
    fn negative_age_panics() {
        check_age(-1);
    }

    #[test]
    fn test_can_return_result() -> Result<(), String> {
        if add(2, 2) == 4 {
            Ok(())
        } else {
            Err(String::from("2+2 != 4 ?!"))
        }
    }

    #[test]
    #[ignore]
    fn slow_test() {
        assert_eq!(add(1000, 1000), 2000);
    }
}

// Learn more
//
// You can learn unit tests from these sources:
//   - The Rust Book, "How to Write Tests": https://doc.rust-lang.org/book/ch11-01-writing-tests.html
//   - Rust by Example, "Unit testing": https://doc.rust-lang.org/rust-by-example/testing/unit_testing.html
//   - Comprehensive Rust, "Unit Tests": https://google.github.io/comprehensive-rust/testing/unit-tests.html
//   - Easy: Easy Rust, "Testing": https://dhghomon.github.io/easy_rust/Chapter_58.html
//   - Easy: Let's Get Rusty (video), "Testing in Rust": https://www.youtube.com/watch?v=18-7NoNPO30
//
// You can learn should_panic and tests that return Result from these sources:
//   - The Rust Book, "Checking for Panics with should_panic": https://doc.rust-lang.org/book/ch11-01-writing-tests.html#checking-for-panics-with-should_panic
//   - Rust by Example, "Unit testing": https://doc.rust-lang.org/rust-by-example/testing/unit_testing.html
//   - Rust Reference, "The should_panic attribute": https://doc.rust-lang.org/reference/attributes/testing.html#the-should_panic-attribute
//   - Easy: Let's Get Rusty (video), "Testing in Rust": https://www.youtube.com/watch?v=18-7NoNPO30
//
// You can learn how to test private functions from these sources:
//   - The Rust Book, "Private Function Tests": https://doc.rust-lang.org/book/ch11-03-test-organization.html#private-function-tests
//   - Comprehensive Rust, "Unit Tests": https://google.github.io/comprehensive-rust/testing/unit-tests.html
//   - Rust by Example, "Unit testing": https://doc.rust-lang.org/rust-by-example/testing/unit_testing.html
//   - Easy: Let's Get Rusty (video), "Testing in Rust - Part 2": https://www.youtube.com/watch?v=-L4nKAlmH3M
//
// You can learn how to ignore slow tests from these sources:
//   - The Rust Book, "Ignoring Tests Unless Specifically Requested": https://doc.rust-lang.org/book/ch11-02-running-tests.html#ignoring-tests-unless-specifically-requested
//   - Rust Reference, "The ignore attribute": https://doc.rust-lang.org/reference/attributes/testing.html#the-ignore-attribute
//   - Cargo Book, "cargo test": https://doc.rust-lang.org/cargo/commands/cargo-test.html
//   - Easy: Let's Get Rusty (video), "Testing in Rust - Part 2": https://www.youtube.com/watch?v=-L4nKAlmH3M
