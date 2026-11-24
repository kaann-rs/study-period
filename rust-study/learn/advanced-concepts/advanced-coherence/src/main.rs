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
