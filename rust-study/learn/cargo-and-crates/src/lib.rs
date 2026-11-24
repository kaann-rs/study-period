// Kitap: 13. bölüm, Cargo ve Crate'ler

//! # cargo-and-crates
//!
//! Bir renk karıştırma kütüphanesi:
//! ```
//! use cargo_and_crates::{PrimaryColor, mix};
//! let green = mix(PrimaryColor::Yellow, PrimaryColor::Blue);
//! ```

pub mod types {
    /// Birincil renkler (karışımın girdileri).
    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum PrimaryColor {
        Red,
        Yellow,
        Blue,
    }

    /// İkincil renkler (iki birincilin karışımı).
    #[derive(Debug, PartialEq)]
    pub enum SecondaryColor {
        Orange,
        Green,
        Purple,
    }
}

pub mod actions {
    use crate::types::{PrimaryColor, SecondaryColor};

    /// İki birincil rengi karıştırıp ikincil rengi döndürür.
    ///
    /// Aynı renk iki kez verilirse `None` döner (karışım tanımsız).
    /// ```
    /// use cargo_and_crates::{PrimaryColor, SecondaryColor, mix};
    /// assert_eq!(mix(PrimaryColor::Red, PrimaryColor::Yellow), Some(SecondaryColor::Orange));
    /// assert_eq!(mix(PrimaryColor::Blue, PrimaryColor::Blue), None);
    /// ```
    pub fn mix(a: PrimaryColor, b: PrimaryColor) -> Option<SecondaryColor> {
        use PrimaryColor::*;
        let mut pair = [a, b];
        pair.sort_by_key(|r| *r as u8);
        match pair {
            [Red, Yellow] => Some(SecondaryColor::Orange),
            [Yellow, Blue] => Some(SecondaryColor::Green),
            [Red, Blue] => Some(SecondaryColor::Purple),
            _ => None,
        }
    }
}

pub use actions::mix;
pub use types::{PrimaryColor, SecondaryColor};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixes_green() {
        assert_eq!(
            mix(PrimaryColor::Yellow, PrimaryColor::Blue),
            Some(SecondaryColor::Green)
        );
    }

    #[test]
    fn same_color_gives_none() {
        assert_eq!(mix(PrimaryColor::Red, PrimaryColor::Red), None);
    }
}

// Learn more
//
// You can learn documentation comments from these sources:
//   - The Rust Book, "Making Useful Documentation Comments": https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#making-useful-documentation-comments
//   - Rustdoc Book, "How to write documentation": https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html
//   - Rust by Example, "Documentation": https://doc.rust-lang.org/rust-by-example/meta/doc.html
//   - Easy: Easy Rust, "cargo doc": https://dhghomon.github.io/easy_rust/Chapter_65.html
//
// You can learn documentation tests from these sources:
//   - Rustdoc Book, "Documentation tests": https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html
//   - Rust by Example, "Documentation testing": https://doc.rust-lang.org/rust-by-example/testing/doc_testing.html
//   - Comprehensive Rust, "Other Types of Tests": https://google.github.io/comprehensive-rust/testing/other.html
//   - Easy: Let's Get Rusty (video), "Publishing a Rust Crate!": https://www.youtube.com/watch?v=4TI153PIEDQ
//
// You can learn re-exports with pub use from these sources:
//   - The Rust Book, "Exporting a Convenient Public API": https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#exporting-a-convenient-public-api
//   - Rustdoc Book, "Re-exports": https://doc.rust-lang.org/rustdoc/write-documentation/re-exports.html
//   - Effective Rust, "Re-export dependencies whose types appear in your API": https://effective-rust.com/re-export.html
//   - Easy: Let's Get Rusty (video), "Publishing a Rust Crate!": https://www.youtube.com/watch?v=4TI153PIEDQ
