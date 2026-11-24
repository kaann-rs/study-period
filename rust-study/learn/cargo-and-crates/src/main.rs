// Kitap: 13. bölüm, Cargo ve Crate'ler

use cargo_and_crates::{PrimaryColor, mix};

fn main() {
    for (a, b) in [
        (PrimaryColor::Red, PrimaryColor::Yellow),
        (PrimaryColor::Yellow, PrimaryColor::Blue),
        (PrimaryColor::Red, PrimaryColor::Blue),
        (PrimaryColor::Blue, PrimaryColor::Blue),
    ] {
        println!("{a:?} + {b:?} = {:?}", mix(a, b));
    }
    println!(
        "(doküman: cargo doc -p cargo-and-crates --open · testler: cargo test -p cargo-and-crates)"
    );
}
