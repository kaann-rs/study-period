// Açıklama: learn/minigrep/ACIKLAMA.md
// İnce ikili: mantık lib.rs'te; main argümanları toplar, hatayı stderr'e basar, kod 1'le biter.
// SADE ANLATIM: Bu dosya danışma masasının kapısıdır; asıl işi lib.rs'teki görevli yapar:
// argümanlar fişe yazılır (`Config::build`), fiş görevliye verilir (`run`), sorun olursa hata
// mesajı basılıp 1 koduyla çıkılır. `main` entegrasyon testine doğrudan import edilemez; test
// edilebilir mantığı lib.rs'e taşıyıp main'i ince tutarız. Terimler için lib.rs'in mini sözlüğüne bak.
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
