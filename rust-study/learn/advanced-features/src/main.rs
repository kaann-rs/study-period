// Kitap: 19. bölüm, Gelişmiş Özellikler

use std::fmt;
use std::ops::Add;
use std::slice;

macro_rules! my_vec {
    ( $( $x:expr ),* ) => {{
        let mut v = Vec::new();
        $( v.push($x); )*
        v
    }};
}

fn main() {
    show_unsafe();
    show_operator_overload();
    show_newtype();
    show_alias_and_fn();
    show_macro();
}

fn split_in_two(slice: &mut [i32], middle: usize) -> (&mut [i32], &mut [i32]) {
    let length = slice.len();
    let pointer = slice.as_mut_ptr();
    assert!(middle <= length);
    unsafe {
        (
            slice::from_raw_parts_mut(pointer, middle),
            slice::from_raw_parts_mut(pointer.add(middle), length - middle),
        )
    }
}
fn show_unsafe() {
    let mut numbers = [1, 2, 3, 4, 5, 6];
    let (left, right) = split_in_two(&mut numbers, 3);
    println!("split_in_two: left={left:?} right={right:?}");

    let x = 42;
    let r = &x as *const i32;
    unsafe {
        println!("raw pointer deref (unsafe): *r = {}", *r);
    }
}

#[derive(Debug, PartialEq)]
struct Meters(u32);

impl Add for Meters {
    type Output = Meters;
    fn add(self, other: Meters) -> Meters {
        Meters(self.0 + other.0)
    }
}

fn show_operator_overload() {
    let total = Meters(3) + Meters(4);
    println!("operator overload: {total:?}");
    assert_eq!(total, Meters(7));
}

struct Wrapper(Vec<String>);
impl fmt::Display for Wrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]", self.0.join(", "))
    }
}
fn show_newtype() {
    let wrapper = Wrapper(vec![
        String::from("a"),
        String::from("b"),
        String::from("c"),
    ]);
    println!("newtype Display: {wrapper}");
}

type Text = String;

fn apply(f: fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}
fn double(x: i32) -> i32 {
    x * 2
}

fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}
fn make_boxed_adder(n: i32) -> Box<dyn Fn(i32) -> i32> {
    Box::new(move |x| x + n)
}
fn show_alias_and_fn() {
    let s: Text = String::from("alias");
    println!("type alias: {s}");

    println!("fn pointer: apply(double, 10) = {}", apply(double, 10));
    let add_hundred = make_adder(100);
    let add_one = make_boxed_adder(1);
    println!("impl Fn closure döndürme: {}", add_hundred(5));
    println!("Box<dyn Fn> closure döndürme: {}", add_one(5));
}

fn show_macro() {
    let v: Vec<i32> = my_vec![10, 20, 30];
    println!("macro_rules! my_vec![10,20,30] = {v:?}");
}

// Learn more
//
// You can learn unsafe Rust from these sources:
//   - The Rust Book, "Unsafe Rust": https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html
//   - Rustonomicon, "Meet Safe and Unsafe": https://doc.rust-lang.org/nomicon/meet-safe-and-unsafe.html
//   - Comprehensive Rust, "Unsafe": https://google.github.io/comprehensive-rust/unsafe-rust/unsafe.html
//   - Easy: Let's Get Rusty (video), "Writing Unsafe Rust": https://www.youtube.com/watch?v=9E2v8pCUc48
//
// You can learn operator overloading from these sources:
//   - The Rust Book, "Using Default Generic Parameters and Operator Overloading": https://doc.rust-lang.org/book/ch20-02-advanced-traits.html#using-default-generic-parameters-and-operator-overloading
//   - Rust by Example, "Operator Overloading": https://doc.rust-lang.org/rust-by-example/trait/ops.html
//   - Comprehensive Rust, "Operators": https://google.github.io/comprehensive-rust/std-traits/operators.html
//   - Easy: Let's Get Rusty (video), "Advanced Traits in Rust": https://www.youtube.com/watch?v=RDxz94tuxqM
//
// You can learn the newtype pattern from these sources:
//   - The Rust Book, "Implementing External Traits with the Newtype Pattern": https://doc.rust-lang.org/book/ch20-02-advanced-traits.html#implementing-external-traits-with-the-newtype-pattern
//   - Rust by Example, "New Type Idiom": https://doc.rust-lang.org/rust-by-example/generics/new_types.html
//   - Effective Rust, "Embrace the newtype pattern": https://effective-rust.com/newtype.html
//   - Easy: worthe-it, "The Newtype Pattern in Rust": https://www.worthe-it.co.za/blog/2020-10-31-newtype-pattern-in-rust.html
//
// You can learn type aliases from these sources:
//   - The Rust Book, "Type Synonyms and Type Aliases": https://doc.rust-lang.org/book/ch20-03-advanced-types.html#type-synonyms-and-type-aliases
//   - Rust by Example, "Aliasing": https://doc.rust-lang.org/rust-by-example/types/alias.html
//   - Comprehensive Rust, "Type Aliases": https://google.github.io/comprehensive-rust/user-defined-types/aliases.html
//   - Easy: Easy Rust, "Type aliases": https://dhghomon.github.io/easy_rust/Chapter_43.html
//   - Easy: Let's Get Rusty (video), "Advanced Types in Rust": https://www.youtube.com/watch?v=aI1oIFqh44Y
//
// You can learn function pointers and returned closures from these sources:
//   - The Rust Book, "Advanced Functions and Closures": https://doc.rust-lang.org/book/ch20-04-advanced-functions-and-closures.html
//   - Standard library docs, "fn": https://doc.rust-lang.org/std/primitive.fn.html
//   - Rust by Example, "As output parameters": https://doc.rust-lang.org/rust-by-example/fn/closures/output_parameters.html
//   - Easy: Let's Get Rusty (video), "Advanced Function and Closures in Rust": https://www.youtube.com/watch?v=rnxutl7t1hI
//
// You can learn macro_rules! from these sources:
//   - The Rust Book, "Declarative Macros for General Metaprogramming": https://doc.rust-lang.org/book/ch20-05-macros.html#declarative-macros-for-general-metaprogramming
//   - Rust by Example, "macro_rules!": https://doc.rust-lang.org/rust-by-example/macros.html
//   - Little Book of Rust Macros, "A Practical Introduction": https://lukaswirth.dev/tlborm/decl-macros/macros-practical.html
//   - Easy: Easy Rust, "Writing macros": https://dhghomon.github.io/easy_rust/Chapter_61.html
//   - Easy: Let's Get Rusty (video), "Declarative Macros in Rust": https://www.youtube.com/watch?v=KsJHlqULpO4
