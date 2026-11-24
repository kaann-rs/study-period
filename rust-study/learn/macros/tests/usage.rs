// Kitap: 20. bölüm, Makrolar

use macros::make_answer;

make_answer!();

#[test]
fn answer_is_42() {
    assert_eq!(answer(), 42);
}
