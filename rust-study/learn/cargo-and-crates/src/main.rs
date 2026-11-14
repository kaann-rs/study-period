// Demo: kütüphaneyi KÖK'ten (pub use sayesinde kısa yollarla) kullanıyoruz.
// Asıl olay doc-test'ler + cargo doc: cargo test -p cargo-and-crates
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
