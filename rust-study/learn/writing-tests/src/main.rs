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
