// Kitap: 27. bölüm, İleri Lifetime, HRTB ve GAT

use std::fmt::Debug;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

fn main() {
    println!("== 1) Lifetime ilişkileri: iki ayrı süre, 'a: 'b, T: 'a ==");
    show_lifetime_relations();

    println!("\n== 2) HRTB: for<'a> — süreyi F'yi çağıran seçer, F her süreye razıdır ==");
    show_hrtb_closure();
    show_hrtb_trait();

    println!("\n== 3) GAT: ödünç veren iterator ==");
    show_lending_iterator();

    println!("\n== 4) GAT: tür ailesi (Rc mi, Arc mı — sonradan seç) ==");
    show_type_family();
}

struct WordFinder<'t, 'w> {
    text: &'t str,
    word: &'w str,
}

impl<'t, 'w> WordFinder<'t, 'w> {
    fn find_matches(&self) -> Vec<&'t str> {
        self.text
            .match_indices(self.word)
            .map(|(i, _)| &self.text[i..i + self.word.len()])
            .collect()
    }
}

fn default_if_empty<'short, 'long: 'short>(
    input: &'short str,
    default_text: &'long str,
) -> &'short str {
    if input.trim().is_empty() {
        default_text
    } else {
        input
    }
}

fn to_boxed<'a, T: Debug + 'a>(t: T) -> Box<dyn Debug + 'a> {
    Box::new(t)
}

fn keep<'a>(list: &mut Vec<&'a str>, text: &'a str) {
    list.push(text);
}

fn show_lifetime_relations() {
    let text = String::from("kar yağar, kar erir, kar kalmaz");
    let found;
    {
        let word = String::from("kar");
        let finder = WordFinder {
            text: &text,
            word: &word,
        };
        found = finder.find_matches();
    }
    println!("   eşleşme sayısı = {}", found.len());

    let default_text = String::from("anonim");
    let name = {
        let input = String::from("   ");
        default_if_empty(&input, &default_text).to_string()
    };
    println!("   boş girdi → {name}");

    let local = String::from("yerel veri");
    let boxed = to_boxed(&local);
    println!("   kutuda: {boxed:?}");

    let mut list = Vec::new();
    keep(&mut list, &text[..3]);
    keep(&mut list, "sabit");
    println!("   liste = {list:?}");
}

fn convert_lines<F>(raw: &str, convert: F) -> Vec<String>
where
    F: for<'a> Fn(&'a str) -> &'a str,
{
    raw.lines()
        .map(|line| {
            let big = line.to_uppercase();
            convert(&big).to_string()
        })
        .collect()
}

type Trimmer = for<'a> fn(&'a str) -> &'a str;

fn force_signature<F: for<'a> Fn(&'a str) -> &'a str>(f: F) -> F {
    f
}

fn show_hrtb_closure() {
    let output = convert_lines("  ada  \n grace ", |s| s.trim());
    println!("   {output:?}");

    let first_word = force_signature(|s| s.split_whitespace().next().unwrap_or(""));
    println!("   {:?}", convert_lines("merhaba dünya", first_word));

    let pointer: Trimmer = str::trim_start;
    println!("   {:?}", convert_lines("  girintili", pointer));
}

trait Parse<'i>: Sized {
    fn parse(input: &'i str) -> Option<Self>;
}

impl<'i> Parse<'i> for &'i str {
    fn parse(input: &'i str) -> Option<Self> {
        Some(input.trim())
    }
}

impl<'i> Parse<'i> for u32 {
    fn parse(input: &'i str) -> Option<Self> {
        input.trim().parse().ok()
    }
}

impl<'i> Parse<'i> for String {
    fn parse(input: &'i str) -> Option<Self> {
        Some(input.trim().to_string())
    }
}

fn parse_from_source<T>(source: impl FnOnce() -> String) -> Option<T>
where
    T: for<'i> Parse<'i>,
{
    let buffer = source();
    T::parse(&buffer)
}

fn parse_from_buffer<'i, T: Parse<'i>>(buffer: &'i str) -> Option<T> {
    T::parse(buffer)
}

fn show_hrtb_trait() {
    let number: Option<u32> = parse_from_source(|| String::from(" 42 "));
    let text: Option<String> = parse_from_source(|| String::from("  sahipli  "));
    println!("   kaynaktan: {number:?}, {text:?}");

    let buffer = String::from("  ödünç  ");
    let borrowed: Option<&str> = parse_from_buffer(&buffer);
    println!("   tampondan: {borrowed:?}");
}

trait LendingIterator {
    type Item<'a>
    where
        Self: 'a;

    fn next<'a>(&'a mut self) -> Option<Self::Item<'a>>;
}

struct WindowMut<'d, T> {
    data: &'d mut [T],
    start: usize,
    size: usize,
}

impl<'d, T> WindowMut<'d, T> {
    fn new(data: &'d mut [T], size: usize) -> Self {
        WindowMut {
            data,
            start: 0,
            size,
        }
    }
}

impl<'d, T> LendingIterator for WindowMut<'d, T> {
    type Item<'a>
        = &'a mut [T]
    where
        Self: 'a;

    fn next<'a>(&'a mut self) -> Option<Self::Item<'a>> {
        let end = self.start.checked_add(self.size)?;
        let window = self.data.get_mut(self.start..end)?;
        self.start += 1;
        Some(window)
    }
}

fn count_items<I: LendingIterator>(mut it: I) -> usize {
    let mut n = 0;
    while it.next().is_some() {
        n += 1;
    }
    n
}

fn show_lending_iterator() {
    let mut data = [1, 2, 3, 4, 5];
    let mut windows = WindowMut::new(&mut data, 2);
    while let Some(window) = windows.next() {
        window[1] += window[0];
    }
    println!("   birikimli toplam = {data:?}");

    let mut letters = ['a', 'b', 'c', 'd'];
    println!(
        "   3'lü pencere sayısı = {}",
        count_items(WindowMut::new(&mut letters, 3))
    );
}

trait PointerFamily {
    type Pointer<T>: Deref<Target = T> + Clone;
    fn new<T>(value: T) -> Self::Pointer<T>;
}

struct RcFamily;
struct ArcFamily;

impl PointerFamily for RcFamily {
    type Pointer<T> = Rc<T>;
    fn new<T>(value: T) -> Rc<T> {
        Rc::new(value)
    }
}

impl PointerFamily for ArcFamily {
    type Pointer<T> = Arc<T>;
    fn new<T>(value: T) -> Arc<T> {
        Arc::new(value)
    }
}

struct Document<F: PointerFamily> {
    versions: Vec<F::Pointer<String>>,
}

impl<F: PointerFamily> Document<F> {
    fn new(first: &str) -> Self {
        Document {
            versions: vec![F::new(first.to_string())],
        }
    }

    fn edit(&mut self, new_text: &str) {
        self.versions.push(F::new(new_text.to_string()));
    }

    fn current(&self) -> F::Pointer<String> {
        self.versions
            .last()
            .expect("Document en az bir sürümle kurulur (bkz. new)")
            .clone()
    }

    fn version_count(&self) -> usize {
        self.versions.len()
    }
}

fn needs_send<T: Send>(_: &T) {}

fn show_type_family() {
    let mut local: Document<RcFamily> = Document::new("taslak");
    local.edit("taslak v2");
    println!(
        "   Rc belgesi : {} ({} sürüm)",
        *local.current(),
        local.version_count()
    );

    let mut shared: Document<ArcFamily> = Document::new("rapor");
    shared.edit("rapor (son)");
    needs_send(&shared);
    let current = shared.current();
    let length = std::thread::spawn(move || current.chars().count())
        .join()
        .unwrap();
    println!("   Arc belgesi: başka thread'de okunan uzunluk = {length}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_live_longer_than_word() {
        let text = String::from("ab ab");
        let result = {
            let word = String::from("ab");
            WordFinder {
                text: &text,
                word: &word,
            }
            .find_matches()
        };
        assert_eq!(result, ["ab", "ab"]);
    }

    #[test]
    fn choice_with_outlives_bound() {
        let default_text = String::from("v");
        assert_eq!(default_if_empty("", &default_text), "v");
        assert_eq!(default_if_empty("dolu", &default_text), "dolu");
    }

    #[test]
    fn hrtb_closure_works_with_any_lifetime() {
        let result = convert_lines("a b\nc d", |s| &s[..1]);
        assert_eq!(result, ["A", "C"]);
    }

    #[test]
    fn owned_types_parse_for_any_lifetime() {
        assert_eq!(parse_from_source::<u32>(|| String::from("7")), Some(7));
        assert_eq!(parse_from_source::<u32>(|| String::from("yedi")), None);
        let buffer = String::from(" x ");
        assert_eq!(parse_from_buffer::<&str>(&buffer), Some("x"));
    }

    #[test]
    fn window_mut_gives_overlapping_windows_in_order() {
        let mut data = [1, 1, 1, 1];
        let mut it = WindowMut::new(&mut data, 2);
        while let Some(window) = it.next() {
            window[1] += window[0];
        }
        assert_eq!(data, [1, 2, 3, 4]);
    }

    #[test]
    fn no_item_if_window_is_bigger_than_slice() {
        let mut data = [1, 2];
        assert_eq!(count_items(WindowMut::new(&mut data, 3)), 0);
        assert_eq!(count_items(WindowMut::new(&mut data, 2)), 1);
        assert_eq!(count_items(WindowMut::new(&mut data, usize::MAX)), 0);
    }

    #[test]
    fn document_works_with_both_families() {
        let mut rc: Document<RcFamily> = Document::new("a");
        rc.edit("b");
        let previous = rc.current();
        rc.edit("c");
        assert_eq!((previous.as_str(), rc.current().as_str()), ("b", "c"));
        assert_eq!(Rc::strong_count(&previous), 2);

        let arc: Document<ArcFamily> = Document::new("x");
        needs_send(&arc);
        assert_eq!(arc.version_count(), 1);
    }
}

// Learn more
//
// You can learn lifetime relations ('a: 'b and T: 'a) from these sources:
//   - Rust Reference, "Lifetime bounds": https://doc.rust-lang.org/reference/trait-bounds.html#lifetime-bounds
//   - Rust by Example, "Bounds": https://doc.rust-lang.org/rust-by-example/scope/lifetime/lifetime_bounds.html
//   - pretzelhammer, "Common Rust Lifetime Misconceptions": https://github.com/pretzelhammer/rust-blog/blob/master/posts/common-rust-lifetime-misconceptions.md
//   - okryvyts, "Lifetime subtyping": https://okryvyts.github.io/lifetimes-book/lifetime_subtyping.html
//   - Easy: Tour of Rust, "Multiple Lifetimes": https://tourofrust.com/54_en.html
//
// You can learn closures with for<'a> (HRTB) from these sources:
//   - Rustonomicon, "Higher-Rank Trait Bounds": https://doc.rust-lang.org/nomicon/hrtb.html
//   - Rust Reference, "Higher-ranked trait bounds": https://doc.rust-lang.org/reference/trait-bounds.html#higher-ranked-trait-bounds
//   - Learning Rust by quinedot, "Higher-ranked types": https://quinedot.github.io/rust-learning/dyn-hr.html
//   - Easy: Rust users forum, "What is the meaning of for<'a>": https://users.rust-lang.org/t/what-is-the-meaning-of-for-a/55929
//
// You can learn traits with for<'a> bounds from these sources:
//   - Serde, "Deserializer lifetimes": https://serde.rs/lifetimes.html
//   - Rust Reference, "Higher-ranked trait bounds": https://doc.rust-lang.org/reference/trait-bounds.html#higher-ranked-trait-bounds
//   - Rustonomicon, "Higher-Rank Trait Bounds": https://doc.rust-lang.org/nomicon/hrtb.html
//   - Easy: Rust users forum, "What is the meaning of for<'a>": https://users.rust-lang.org/t/what-is-the-meaning-of-for-a/55929
//
// You can learn lending iterators (GAT) from these sources:
//   - Rust Blog, "Generic associated types to be stable in Rust 1.65": https://blog.rust-lang.org/2022/10/28/gats-stabilization/
//   - GATs Initiative, "Iterable and lending iterators": https://rust-lang.github.io/generic-associated-types-initiative/design_patterns/iterable.html
//   - Jack Huey, "A shiny future with GATs": https://jackhuey.me/rust/2022/05/04/a-shiny-future-with-gats.html
//   - Easy: LogRocket, "Using Rust GATs to improve code and application performance": https://blog.logrocket.com/using-rust-gats-improve-code-app-performance/
//
// You can learn type families (GAT) from these sources:
//   - Niko Matsakis, "Associated type constructors, part 2: family traits": https://smallcultfollowing.com/babysteps/blog/2016/11/03/associated-type-constructors-part-2-family-traits/
//   - GATs Initiative, "Pointer types": https://rust-lang.github.io/generic-associated-types-initiative/design_patterns/pointer_types.html
//   - Rust RFC 1598, "generic_associated_types": https://rust-lang.github.io/rfcs/1598-generic_associated_types.html
