// İnce ikili: asıl kod lib.rs'te. Burası sadece "crate çalışıyor" demosu.
// Asıl olay testlerde: cargo test -p writing-tests
// SADE ANLATIM: Bu dosya atölyenin vitrinidir: lib.rs'teki üç fonksiyonu birer kez çağırıp
// sonucu ekrana yazar, başka bir iş yapmaz. Testler burada değil, lib.rs'in altında ve tests/
// dizinindedir. Terimler için lib.rs'in başındaki mini sözlüğe bak.
//   Mini örnek:
//     cargo run -p writing-tests     // bu dosyayı çalıştırır: üç satır sonuç yazar
//     cargo test -p writing-tests    // testleri çalıştırır: asıl konu budur
use writing_tests::{kelime_say, topla, yas_dogrula};

fn main() {
    println!("topla(2, 3) = {}", topla(2, 3));
    println!("kelime_say(\"  bir iki üç \") = {}", kelime_say("  bir iki üç "));
    println!("yas_dogrula(30) = {}", yas_dogrula(30));
    println!("(testleri koşmak için: cargo test -p writing-tests)");
}
