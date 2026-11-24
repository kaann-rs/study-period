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

// Learn more
//
// You can learn Cargo from these sources:
//   - Cargo Book, "Cargo Guide": https://doc.rust-lang.org/cargo/guide/index.html
//   - Rust by Example, "Cargo": https://doc.rust-lang.org/rust-by-example/cargo.html
//   - Comprehensive Rust, "Using Cargo": https://google.github.io/comprehensive-rust/cargo.html
//   - Easy: Programiz, "Rust Cargo": https://www.programiz.com/rust/cargo
//   - Easy: Easy Rust, "cargo": https://dhghomon.github.io/easy_rust/Chapter_62.html
//
// You can learn release profiles from these sources:
//   - The Rust Book, "Customizing Builds with Release Profiles": https://doc.rust-lang.org/book/ch14-01-release-profiles.html
//   - Cargo Book, "Profiles": https://doc.rust-lang.org/cargo/reference/profiles.html
//   - Rust Performance Book, "Build Configuration": https://nnethercote.github.io/perf-book/build-configuration.html
//   - Easy: Let's Get Rusty (video), "Publishing a Rust Crate!": https://www.youtube.com/watch?v=4TI153PIEDQ
//
// You can learn workspaces from these sources:
//   - The Rust Book, "Cargo Workspaces": https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html
//   - Cargo Book, "Workspaces": https://doc.rust-lang.org/cargo/reference/workspaces.html
//   - Easy: Let's Get Rusty (video), "Cargo Workspaces": https://www.youtube.com/watch?v=70_9IIsQfjs
