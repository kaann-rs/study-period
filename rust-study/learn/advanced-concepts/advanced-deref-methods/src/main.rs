// Kitap: 24. bölüm, Deref ve Metot Çözümleme

use std::ops::Deref;

fn main() {
    show_deref_coercion();
    show_autoderef();
    show_inherent_shadowing();
}

struct MyBox<T>(T);
impl<T> Deref for MyBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}
fn greet(name: &str) {
    println!("  selam, {name}");
}
fn show_deref_coercion() {
    let boxed = MyBox(5);
    println!("*MyBox(5) = {}", *boxed);

    let name = MyBox(String::from("Kaan"));
    greet(&name);
}

fn show_autoderef() {
    let boxed = Box::new(vec![1, 2, 3]);
    println!("autoderef: Box<Vec<i32>>.len() = {}", boxed.len());

    let mut v = vec![1, 2, 3];
    v.push(4);
    println!("autoref: v.push(4) == (&mut v).push(4) -> {v:?}");

    let s = String::from("merhaba");
    println!("autoref &self: s.len() = {}", s.len());
}

struct Inner;
impl Inner {
    fn name(&self) -> &str {
        "IC"
    }
}
struct Wrapper(Inner);
impl Deref for Wrapper {
    type Target = Inner;
    fn deref(&self) -> &Inner {
        &self.0
    }
}
impl Wrapper {
    fn name(&self) -> &str {
        "SARMAL"
    }
}
fn show_inherent_shadowing() {
    let wrapper = Wrapper(Inner);
    println!(
        "wrapper.name()        = {} (inherent kazanır, Deref'e inilmez)",
        wrapper.name()
    );
    println!(
        "(*wrapper).name()     = {} (elle deref → Inner)",
        (*wrapper).name()
    );
    println!(
        "Inner::name(&wrapper) = {} (tam-nitelikli, &Wrapper coerce)",
        Inner::name(&wrapper)
    );
}

// Learn more
//
// You can learn deref coercion from these sources:
//   - The Rust Book, "Using Deref Coercion in Functions and Methods": https://doc.rust-lang.org/book/ch15-02-deref.html#using-deref-coercion-in-functions-and-methods
//   - Standard library docs, "Deref coercion": https://doc.rust-lang.org/std/ops/trait.Deref.html#deref-coercion
//   - Rust Reference, "Type coercions": https://doc.rust-lang.org/reference/type-coercions.html
//   - Easy: Let's Get Rusty (video), "Smart Pointers in Rust - The Deref Trait": https://www.youtube.com/watch?v=dYEC6NElVOg
//
// You can learn autoderef in method calls from these sources:
//   - Rust Reference, "Method call expressions": https://doc.rust-lang.org/reference/expressions/method-call-expr.html
//   - Rustonomicon, "The Dot Operator": https://doc.rust-lang.org/nomicon/dot-operator.html
//   - Rust compiler guide, "Method lookup": https://rustc-dev-guide.rust-lang.org/hir-typeck/method-lookup.html
//   - Easy: Easy Rust, "References and the dot operator": https://dhghomon.github.io/easy_rust/Chapter_29.html
//
// You can learn which method Rust calls (inherent or inner) from these sources:
//   - Rust Reference, "Method call expressions": https://doc.rust-lang.org/reference/expressions/method-call-expr.html
//   - Rust by Example, "Disambiguating overlapping traits": https://doc.rust-lang.org/rust-by-example/trait/disambiguating.html
//   - Rust compiler guide, "Method lookup": https://rustc-dev-guide.rust-lang.org/hir-typeck/method-lookup.html
//   - Easy: Let's Get Rusty (video), "Advanced Traits in Rust": https://www.youtube.com/watch?v=RDxz94tuxqM
