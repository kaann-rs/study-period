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
