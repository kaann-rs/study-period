// Kitap: 9.1. bölüm, Generic, Trait ve Lifetime: Derinleşme

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt::Display;
use std::hash::Hash;

pub fn run() {
    println!("\n== Derinleşme: generics, trait bound, lifetime pratikte ==");
    show_lifetime_relation();
    show_zero_copy_parser();
    show_static();
    show_generic_cache();
    show_associated_type_or_generic();
}

fn remove_prefix_strict<'a>(text: &'a str, prefix: &'a str) -> &'a str {
    text.strip_prefix(prefix).unwrap_or(text)
}

fn remove_prefix<'a>(text: &'a str, prefix: &str) -> &'a str {
    text.strip_prefix(prefix).unwrap_or(text)
}

fn show_lifetime_relation() {
    let path = String::from("/api/kullanicilar");
    let clean;
    {
        let prefix = String::from("/api");
        clean = remove_prefix(&path, &prefix);
        println!(
            "blok içinde sıkı sürüm de çalışır: {}",
            remove_prefix_strict(&path, &prefix)
        );
    }
    println!("prefix öldü, sonuç hâlâ geçerli: {clean}");
}

#[derive(Debug, PartialEq, Eq)]
pub struct Setting<'a> {
    pub key: &'a str,
    pub value: &'a str,
}

pub struct Parser<'a> {
    rest: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Parser { rest: input }
    }

    pub fn next_setting(&mut self) -> Option<Setting<'a>> {
        while !self.rest.is_empty() {
            let (part, rest) = self.rest.split_once(';').unwrap_or((self.rest, ""));
            self.rest = rest;
            if let Some((key, value)) = part.split_once('=') {
                return Some(Setting {
                    key: key.trim(),
                    value: value.trim(),
                });
            }
        }
        None
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Setting<'a>;
    fn next(&mut self) -> Option<Setting<'a>> {
        self.next_setting()
    }
}

fn show_zero_copy_parser() {
    let input = String::from("ad = Kaan; bozuk; dil=tr;tema = koyu");
    let settings: Vec<Setting> = {
        let parser = Parser::new(&input);
        parser.collect()
    };
    for setting in &settings {
        println!("  {} → {}", setting.key, setting.value);
    }

    let input_range = input.as_bytes().as_ptr_range();
    let is_inside = input_range.contains(&settings[0].value.as_ptr());
    println!("  değerler girdinin içine bakıyor (kopya yok): {is_inside}");
}

fn keep_long<T: Display + 'static>(value: T) -> Box<dyn Display> {
    Box::new(value)
}

fn show_static() {
    let owned = String::from("sahipli String");
    let boxed = keep_long(owned);
    println!("{boxed}");
    drop(boxed);

    let number_box = keep_long(42);
    let fixed_box = keep_long("literal");
    println!("{number_box}, {fixed_box}");
}

pub struct Cache<K, V> {
    records: HashMap<K, V>,
    hits: u32,
    misses: u32,
}

impl<K, V> Cache<K, V> {
    pub fn new() -> Self {
        Cache {
            records: HashMap::new(),
            hits: 0,
            misses: 0,
        }
    }

    pub fn stats(&self) -> (u32, u32) {
        (self.hits, self.misses)
    }
}

impl<K: Eq + Hash, V> Cache<K, V> {
    pub fn get_or_make(&mut self, key: K, make: impl FnOnce(&K) -> V) -> &V {
        match self.records.entry(key) {
            Entry::Occupied(full) => {
                self.hits += 1;
                full.into_mut()
            }
            Entry::Vacant(empty) => {
                self.misses += 1;
                let value = make(empty.key());
                empty.insert(value)
            }
        }
    }
}

impl<K: Ord + Display, V: Display> Cache<K, V> {
    pub fn to_text(&self) -> String {
        let mut lines: Vec<(&K, &V)> = self.records.iter().collect();
        lines.sort_by(|a, b| a.0.cmp(b.0));
        lines
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn show_generic_cache() {
    let mut cache: Cache<String, usize> = Cache::new();
    let slow_work = |text: &String| {
        println!("  (hesaplanıyor: {text})");
        text.chars().count()
    };
    let a = *cache.get_or_make(String::from("merhaba"), slow_work);
    let b = *cache.get_or_make(String::from("merhaba"), slow_work);
    let c = *cache.get_or_make(String::from("dünya"), slow_work);
    println!(
        "  sonuçlar = {a}, {b}, {c}; (isabet, ıska) = {:?}",
        cache.stats()
    );
    println!("  döküm: {}", cache.to_text());

    let mut squares: Cache<u32, u64> = Cache::new();
    println!(
        "  12² = {}",
        squares.get_or_make(12, |n| u64::from(*n) * u64::from(*n))
    );
}

pub trait Parse {
    type Output;
    fn parse(&self, raw: &str) -> Option<Self::Output>;
}

pub struct WholeNumber;
pub struct YesNo;

impl Parse for WholeNumber {
    type Output = i64;
    fn parse(&self, raw: &str) -> Option<i64> {
        raw.trim().parse().ok()
    }
}

impl Parse for YesNo {
    type Output = bool;
    fn parse(&self, raw: &str) -> Option<bool> {
        match raw.trim().to_lowercase().as_str() {
            "evet" | "e" | "1" => Some(true),
            "hayır" | "h" | "0" => Some(false),
            _ => None,
        }
    }
}

pub fn parse_all<P: Parse>(parser: &P, inputs: &[&str]) -> Vec<P::Output> {
    inputs
        .iter()
        .filter_map(|input| parser.parse(input))
        .collect()
}

fn show_associated_type_or_generic() {
    let numbers = parse_all(&WholeNumber, &["10", " 20 ", "otuz", "-4"]);
    let answers = parse_all(&YesNo, &["Evet", "h", "belki", "1"]);
    println!("  sayılar = {numbers:?}");
    println!("  cevaplar = {answers:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_result_depends_only_on_text() {
        let text = String::from("ön-ek:gövde");
        let result = {
            let temp = String::from("ön-ek:");
            remove_prefix(&text, &temp)
        };
        assert_eq!(result, "gövde");
        assert_eq!(remove_prefix_strict("abc", "x"), "abc");
    }

    #[test]
    fn parser_skips_broken_parts() {
        let settings: Vec<_> = Parser::new("a=1;;bozuk; b = 2 ;").collect();
        assert_eq!(
            settings,
            [
                Setting {
                    key: "a",
                    value: "1"
                },
                Setting {
                    key: "b",
                    value: "2"
                }
            ]
        );
    }

    #[test]
    fn results_live_longer_than_parser() {
        let input = String::from("x=1;y=2");
        let (first, second) = {
            let mut parser = Parser::new(&input);
            (parser.next_setting(), parser.next_setting())
        };
        assert_eq!(first.unwrap().key, "x");
        assert_eq!(second.unwrap().value, "2");
    }

    #[test]
    fn cache_calls_maker_only_on_miss() {
        let mut calls = 0;
        let mut cache = Cache::new();
        for key in ["a", "b", "a", "a"] {
            cache.get_or_make(key, |k| {
                calls += 1;
                k.len()
            });
        }
        assert_eq!(calls, 2);
        assert_eq!(cache.stats(), (2, 2));
        assert_eq!(cache.to_text(), "a=1, b=1");
    }

    #[test]
    fn associated_type_sets_return_type() {
        let numbers: Vec<i64> = parse_all(&WholeNumber, &["1", "x", "3"]);
        let answers: Vec<bool> = parse_all(&YesNo, &["hayır", "evet"]);
        assert_eq!(numbers, [1, 3]);
        assert_eq!(answers, [false, true]);
    }

    #[test]
    fn static_bound_accepts_owned_value() {
        let boxed = keep_long(String::from("ada"));
        assert_eq!(boxed.to_string(), "ada");
    }
}

// Learn more
//
// You can learn how lifetimes relate to each other from these sources:
//   - The Rust Book, "Relationships": https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html#relationships
//   - Rust by Example, "Functions": https://doc.rust-lang.org/rust-by-example/scope/lifetime/fn.html
//   - pretzelhammer, "Common Rust Lifetime Misconceptions": https://github.com/pretzelhammer/rust-blog/blob/master/posts/common-rust-lifetime-misconceptions.md
//   - Easy: Tour of Rust, "Multiple Lifetimes": https://tourofrust.com/54_en.html
//
// You can learn zero-copy parsers from these sources:
//   - Crust of Rust (video), "Lifetime Annotations": https://www.youtube.com/watch?v=rAl-9HwD858
//   - Comprehensive Rust, "Lifetimes in Data Structures": https://google.github.io/comprehensive-rust/lifetimes/struct-lifetimes.html
//   - Serde, "Deserializer lifetimes": https://serde.rs/lifetimes.html
//   - Manish Goregaokar, "Not a Yoking Matter (Zero-Copy #1)": https://manishearth.github.io/blog/2022/08/03/zero-copy-1-not-a-yoking-matter/
//
// You can learn 'static from these sources:
//   - Rust by Example, "Static": https://doc.rust-lang.org/rust-by-example/scope/lifetime/static_lifetime.html
//   - pretzelhammer, "Common Rust Lifetime Misconceptions": https://github.com/pretzelhammer/rust-blog/blob/master/posts/common-rust-lifetime-misconceptions.md
//   - The Rust Book, "The Static Lifetime": https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html#the-static-lifetime
//   - Easy: Tour of Rust, "Static Lifetimes": https://tourofrust.com/55_en.html
//
// You can learn a generic cache with trait bounds from these sources:
//   - Standard library docs, "std::collections::hash_map::Entry": https://doc.rust-lang.org/std/collections/hash_map/enum.Entry.html
//   - The Rust Book, "Using Trait Bounds to Conditionally Implement Methods": https://doc.rust-lang.org/book/ch10-02-traits.html#using-trait-bounds-to-conditionally-implement-methods
//   - Rust by Example, "Where clauses": https://doc.rust-lang.org/rust-by-example/generics/where.html
//   - Easy: Let's Get Rusty (video), "Traits in Rust": https://www.youtube.com/watch?v=T0Xfltu4h3A
//
// You can learn how to choose associated types or generic parameters from these sources:
//   - The Rust Book, "Defining Traits with Associated Types": https://doc.rust-lang.org/book/ch20-02-advanced-traits.html#defining-traits-with-associated-types
//   - Rust by Example, "Associated types": https://doc.rust-lang.org/rust-by-example/generics/assoc_items/types.html
//   - Comprehensive Rust, "Associated Types": https://google.github.io/comprehensive-rust/methods-and-traits/traits/associated-types.html
//   - Easy: Let's Get Rusty (video), "Advanced Traits in Rust": https://www.youtube.com/watch?v=RDxz94tuxqM
