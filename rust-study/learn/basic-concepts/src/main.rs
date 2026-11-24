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
