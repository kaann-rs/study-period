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
