// Kitap: 23. bölüm, Tutarlılık ve Yetim Kuralı

use std::fmt::Display;

fn main() {
    show_blanket_impl();
    show_local_and_foreign();
    show_covered_impl();
}

trait Label {
    fn label(&self) -> String;
}

impl<T: Display> Label for T {
    fn label(&self) -> String {
        format!("<{self}>")
    }
}

fn show_blanket_impl() {
    println!(
        "blanket: {} {} {}",
        42.label(),
        "metin".label(),
        true.label()
    );
}

struct Mine(i32);

impl Display for Mine {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Mine#{}", self.0)
    }
}

fn show_local_and_foreign() {
    let mine = Mine(7);
    println!("orphan (yabancı trait + yerel tip): {mine}");
    println!("orphan: blanket + yerel tip: {}", mine.label());
}

impl From<Mine> for String {
    fn from(mine: Mine) -> String {
        format!("String::from(Mine({}))", mine.0)
    }
}
fn show_covered_impl() {
    let s: String = Mine(9).into();
    println!("covered impl (From<Mine> for String): {s}");
}

// Learn more
//
// You can learn blanket implementations from these sources:
//   - The Rust Book, "Using Trait Bounds to Conditionally Implement Methods": https://doc.rust-lang.org/book/ch10-02-traits.html#using-trait-bounds-to-conditionally-implement-methods
//   - Comprehensive Rust, "Blanket Implementations": https://google.github.io/comprehensive-rust/idiomatic/polymorphism/generics/blanket-impls.html
//   - pretzelhammer, "Generic Blanket Impls": https://github.com/pretzelhammer/rust-blog/blob/master/posts/tour-of-rusts-standard-library-traits.md#generic-blanket-impls
//   - Easy: Let's Get Rusty (video), "Traits in Rust": https://www.youtube.com/watch?v=T0Xfltu4h3A
//   - Easy: Rust users forum, "What are blanket implementations": https://users.rust-lang.org/t/what-are-blanket-implementations/49904
//
// You can learn the orphan rule (local and foreign types) from these sources:
//   - Rust Reference, "Orphan rules": https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules
//   - Ixrec, "rust-orphan-rules": https://github.com/Ixrec/rust-orphan-rules
//   - Niko Matsakis, "Little Orphan Impls": https://smallcultfollowing.com/babysteps/blog/2015/01/14/little-orphan-impls/
//   - Comprehensive Rust, "Orphan Rule": https://google.github.io/comprehensive-rust/idiomatic/leveraging-the-type-system/extension-traits/orphan-rule.html
//   - SoftwareMill, "Adapter Pattern in Rust: Overcoming the Orphan Rule with Newtype and Extension Traits": https://softwaremill.com/adapter-pattern-in-rust-overcoming-the-orphan-rule-with-newtype-and-extension-traits/
//   - Easy: Rust error codes, "E0117": https://doc.rust-lang.org/error_codes/E0117.html
//
// You can learn covered type parameters from these sources:
//   - Rust Reference, "Orphan rules": https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules
//   - Rust RFC 2451, "re-rebalancing-coherence": https://rust-lang.github.io/rfcs/2451-re-rebalancing-coherence.html
//   - Rust error codes, "E0210": https://doc.rust-lang.org/error_codes/E0210.html
