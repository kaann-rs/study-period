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
