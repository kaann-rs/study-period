// Demo: kütüphaneyi KÖK'ten (pub use sayesinde kısa yollarla) kullanıyoruz.
// Asıl olay doc-test'ler + cargo doc: cargo test -p cargo-and-crates
// SADE ANLATIM: Bu dosya boya setini kutusundan çıkarıp kullanan kişidir. Kütüphanedeki
// `karistir` fonksiyonunu dört renk çiftiyle çağırır ve sonucu yazar. Dikkat edilecek tek yer
// aşağıdaki `use` satırıdır: `turler::` ya da `islem::` yazmadan, doğrudan kökten alıyoruz; bunu
// lib.rs'teki `pub use` sağlıyor. Terimler için lib.rs'teki mini sözlüğe bak.
//   Mini örnek (programın yazdıklarından ikisi):
//     Sari + Mavi = Some(Yesil)    // iki ayrı renk: karışım var
//     Mavi + Mavi = None           // aynı renk iki kez: karışım yok
use cargo_and_crates::{BirincilRenk, karistir};

fn main() {
    for (a, b) in [
        (BirincilRenk::Kirmizi, BirincilRenk::Sari),
        (BirincilRenk::Sari, BirincilRenk::Mavi),
        (BirincilRenk::Kirmizi, BirincilRenk::Mavi),
        (BirincilRenk::Mavi, BirincilRenk::Mavi),
    ] {
        println!("{a:?} + {b:?} = {:?}", karistir(a, b));
    }
    println!("(doküman: cargo doc -p cargo-and-crates --open · testler: cargo test -p cargo-and-crates)");
}
