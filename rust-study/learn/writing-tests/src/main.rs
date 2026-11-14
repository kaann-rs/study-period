// İnce ikili: asıl kod lib.rs'te. Burası sadece "crate çalışıyor" demosu.
// Asıl olay testlerde: cargo test -p writing-tests
use writing_tests::{kelime_say, topla, yas_dogrula};

fn main() {
    println!("topla(2, 3) = {}", topla(2, 3));
    println!("kelime_say(\"  bir iki üç \") = {}", kelime_say("  bir iki üç "));
    println!("yas_dogrula(30) = {}", yas_dogrula(30));
    println!("(testleri koşmak için: cargo test -p writing-tests)");
}
