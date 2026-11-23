// İnce ikili: tüm mantık lib.rs'te. main yalnız argüman toplar, hatayı stderr'e basar,
// doğru çıkış koduyla biter.
// SADE ANLATIM: Bu dosya danışma masasının önündeki kapıdır; asıl işi lib.rs'teki görevli yapar.
// Burada yalnız üç şey olur: komut satırındaki argümanlar fişe yazılır (`Config::build`), fiş
// görevliye verilir (`run`), bir sorun çıkarsa hata mesajı ekrana yazılıp programdan 1 koduyla
// çıkılır. Neden bu kadar kısa? `main` test edilemez; içinde ne kadar az iş olursa, test edilmemiş
// kod o kadar az olur. Terimler için lib.rs'in başındaki mini sözlüğe bak.
//   Mini örnek (rust-study kökünden):
//     cargo run -p minigrep -- hiç learn/minigrep/siir.txt   // ✓ uyan satırları yazar, kod 0
//     cargo run -p minigrep -- hiç                           // ✗ "dosya yolu verilmedi", kod 1
// (Sade: `unwrap_or_else(|hata| ...)`, "Ok ise içindekini ver; Err ise şu küçük fonksiyonu
// çalıştır" demek. `eprintln!` hata kanalına (stderr) yazar; `process::exit(1)` programı bitirir.)
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
