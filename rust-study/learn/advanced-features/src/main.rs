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
