// Kitap: 1. bölüm, Temel Kavramlar

use std::ptr;

fn main() {
    let mut first_var: i32 = 10_i32;

    first_var = 101;

    println!("First Variables {}", first_var);

    unsafe {
        let s: i32 = 10;

        let s1: *const i32 = &s as *const i32;

        ptr::read(s1);

        println!("{:?}", *s1);
    }

    test();
}

const N: usize = 0;

struct Color<F>
where
    F: Fn(i8) -> i8,
{
    x: F,
}

trait Animal<T> {
    type Your;

    fn voice() -> T;
}

impl<F> Animal<i32> for Color<F>
where
    F: Fn(i8) -> i8,
{
    type Your = String;

    fn voice() -> i32 {
        1
    }
}

impl<T: Future> Animal<i32> for T {
    type Your = i32;

    fn voice() -> i32 {
        1
    }
}

impl<F> Color<F>
where
    F: Fn(i8) -> i8,
{
    fn hex(f: F, b: fn(i8) -> i8) -> i8 {
        f(1)
    }
}

#[proc_macro]
pub fn my_macro(_item: TokenStream) -> TokenStream {
    "fn answer() -> u32 { 42 }".parser().unwrap()
}

unsafe extern "C" fn noise() -> i8 {
    1
}

fn hex_v2<F>(f: F) -> i8
where
    F: for<'a> Fn(&'a i8) -> i8,
{
    f(&1)
}

fn hex_v3<'a, F>(f: F) -> i8
where
    F: Fn(&'a i8) -> i8,
{
    f(&1)
}

fn test() {
    let x = |y: i8| -> i8 { y };

    let a: i8 = Color::hex(x, x);

    let b = Color::hex(x, x);

    Color::<fn(i8) -> i8>::voice();

    println!("{:?}", a);
}

// Learn more
//
// You can learn variables and mutability from these sources:
//   - The Rust Book, "Variables and Mutability": https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html
//   - Rust by Example, "Mutability": https://doc.rust-lang.org/rust-by-example/variable_bindings/mut.html
//   - Easy: W3Schools, "Rust Variables": https://www.w3schools.com/rust/rust_variables.php
//   - Easy: Easy Rust, "Mutability (changing)": https://dhghomon.github.io/easy_rust/Chapter_11.html
//
// You can learn raw pointers and unsafe from these sources:
//   - The Rust Book, "Unsafe Rust": https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html
//   - Comprehensive Rust, "Dereferencing Raw Pointers": https://google.github.io/comprehensive-rust/unsafe-rust/dereferencing.html
//   - Standard library docs, "std::ptr::read": https://doc.rust-lang.org/std/ptr/fn.read.html
//   - Easy: Tour of Rust, "Raw Pointers": https://tourofrust.com/91_en.html
//
// You can learn generic structs and trait bounds from these sources:
//   - The Rust Book, "Generic Data Types": https://doc.rust-lang.org/book/ch10-01-syntax.html
//   - Rust by Example, "Bounds": https://doc.rust-lang.org/rust-by-example/generics/bounds.html
//   - Comprehensive Rust, "Trait Bounds": https://google.github.io/comprehensive-rust/generics/trait-bounds.html
//   - Easy: Easy Rust, "Generics": https://dhghomon.github.io/easy_rust/Chapter_30.html
//
// You can learn traits with type parameters and associated types from these sources:
//   - The Rust Book, "Defining Traits with Associated Types": https://doc.rust-lang.org/book/ch20-02-advanced-traits.html#defining-traits-with-associated-types
//   - Rust by Example, "Associated types": https://doc.rust-lang.org/rust-by-example/generics/assoc_items/types.html
//   - Comprehensive Rust, "Associated Types": https://google.github.io/comprehensive-rust/methods-and-traits/traits/associated-types.html
//   - Easy: Let's Get Rusty (video), "Advanced Traits in Rust": https://www.youtube.com/watch?v=RDxz94tuxqM
//
// You can learn conflicting implementations from these sources:
//   - Rust Reference, "Trait implementation coherence": https://doc.rust-lang.org/reference/items/implementations.html#trait-implementation-coherence
//   - Ixrec, "rust-orphan-rules": https://github.com/Ixrec/rust-orphan-rules
//   - Easy: Rust error codes, "E0119": https://doc.rust-lang.org/error_codes/E0119.html
//
// You can learn closures and function pointers from these sources:
//   - The Rust Book, "Function Pointers": https://doc.rust-lang.org/book/ch20-04-advanced-functions-and-closures.html#function-pointers
//   - Rust by Example, "Input functions": https://doc.rust-lang.org/rust-by-example/fn/closures/input_functions.html
//   - Standard library docs, "fn": https://doc.rust-lang.org/std/primitive.fn.html
//   - Easy: Let's Get Rusty (video), "Advanced Function and Closures in Rust": https://www.youtube.com/watch?v=rnxutl7t1hI
//
// You can learn for<'a> bounds from these sources:
//   - Rustonomicon, "Higher-Rank Trait Bounds": https://doc.rust-lang.org/nomicon/hrtb.html
//   - Rust Reference, "Higher-ranked trait bounds": https://doc.rust-lang.org/reference/trait-bounds.html#higher-ranked-trait-bounds
//   - Easy: Rust users forum, "What is the meaning of for<'a>": https://users.rust-lang.org/t/what-is-the-meaning-of-for-a/55929
//
// You can learn procedural macros from these sources:
//   - The Rust Book, "Procedural Macros for Generating Code from Attributes": https://doc.rust-lang.org/book/ch20-05-macros.html#procedural-macros-for-generating-code-from-attributes
//   - Rust Reference, "Procedural macros": https://doc.rust-lang.org/reference/procedural-macros.html
//   - Easy: freeCodeCamp, "Procedural Macros in Rust – A Handbook for Beginners": https://www.freecodecamp.org/news/procedural-macros-in-rust/
//   - Easy: Let's Get Rusty (video), "Procedural Macros in Rust": https://www.youtube.com/watch?v=crWfcA064is
//
// You can learn extern functions from these sources:
//   - Rustonomicon, "FFI": https://doc.rust-lang.org/nomicon/ffi.html
//   - Rust by Example, "Foreign Function Interface": https://doc.rust-lang.org/rust-by-example/std_misc/ffi.html
//   - Comprehensive Rust, "Unsafe External Functions": https://google.github.io/comprehensive-rust/unsafe-rust/unsafe-functions/extern-c.html
