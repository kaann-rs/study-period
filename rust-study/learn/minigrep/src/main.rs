// İnce ikili: tüm mantık lib.rs'te. main yalnız argüman toplar, hatayı stderr'e basar,
// doğru çıkış koduyla biter.
use std::env;
use std::process;

use minigrep::{Config, run};

fn main() {
    // env::args() iterator'ını doğrudan build'e veriyoruz (sahiplik devri, klon yok).
    let config = Config::build(env::args()).unwrap_or_else(|hata| {
        // Argüman ayrıştırma hatası → tanı mesajı STDERR'e, çıkış kodu 1.
        eprintln!("Argüman hatası: {hata}");
        eprintln!("Kullanım: minigrep <sorgu> <dosya_yolu>   (büyük/küçük harf: IGNORE_CASE=1)");
        process::exit(1);
    });

    // run'ın hatası da STDERR'e; sonuçları run kendi içinde stdout'a basar.
    if let Err(e) = run(config) {
        eprintln!("Uygulama hatası: {e}");
        process::exit(1);
    }
}
