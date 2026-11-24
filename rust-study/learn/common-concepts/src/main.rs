// Kitap: 2. bölüm, Yaygın Kavramlar

fn main() {
    show_variables();
    show_shadowing();
    show_data_types();
    show_control_flow();

    println!("add(2, 3) = {}", add(2, 3));
    println!("five() = {}", five());
}

fn show_variables() {
    let x = 5;
    println!("x = {x}");

    let mut y = 5;
    println!("ilk y = {y}");
    y = 6;
    println!("yeni y = {y}");

    const MAX_SCORE: u32 = 100_000;
    println!("MAX_SCORE = {MAX_SCORE}");
}

fn show_shadowing() {
    let spaces = "   ";
    let spaces = spaces.len();
    println!("boşluk sayısı = {spaces}");
}

fn show_data_types() {
    let whole: i32 = -42;
    let decimal: f64 = 3.15;
    let is_true: bool = true;
    let letter: char = 'ş';
    println!("{whole} {decimal} {is_true} {letter}");

    let tuple: (i32, f64, char) = (1, 2.0, 'x');
    let (a, b, c) = tuple;
    println!("{a} {b} {c}, .1 = {}", tuple.1);

    let array = [10, 20, 30];
    println!("ilk = {}", array[0]);
}

fn add(x: i32, y: i32) -> i32 {
    x + y
}

fn five() -> i32 {
    let answer = {
        let base = 2;
        base + 3
    };
    answer
}

fn show_control_flow() {
    let number = 7;

    let label = if number % 2 == 0 { "çift" } else { "tek" };
    println!("{number} → {label}");

    let mut counter = 0;
    let result = loop {
        counter += 1;
        if counter == 5 {
            break counter * 2;
        }
    };
    println!("loop sonucu = {result}");

    let mut countdown = 3;
    while countdown > 0 {
        print!("{countdown} ");
        countdown -= 1;
    }
    println!("kalk!");

    for i in 1..4 {
        print!("{i} ");
    }
    println!();

    let array = [10, 20, 30];
    for item in array {
        print!("{item} ");
    }
    println!();
}

// Learn more
//
// You can learn variables and mutability from these sources:
//   - The Rust Book, "Variables and Mutability": https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html
//   - Rust by Example, "Mutability": https://doc.rust-lang.org/rust-by-example/variable_bindings/mut.html
//   - Easy: W3Schools, "Rust Variables": https://www.w3schools.com/rust/rust_variables.php
//   - Easy: Easy Rust, "Mutability (changing)": https://dhghomon.github.io/easy_rust/Chapter_11.html
//
// You can learn constants from these sources:
//   - The Rust Book, "Declaring Constants": https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html#declaring-constants
//   - Rust by Example, "constants": https://doc.rust-lang.org/rust-by-example/custom_types/constants.html
//   - Easy: W3Schools, "Rust Constants": https://www.w3schools.com/rust/rust_constants.php
//   - Easy: Easy Rust, "const and static": https://dhghomon.github.io/easy_rust/Chapter_15.html
//
// You can learn shadowing from these sources:
//   - The Rust Book, "Shadowing": https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html#shadowing
//   - Rust by Example, "Scope and Shadowing": https://doc.rust-lang.org/rust-by-example/variable_bindings/scope.html
//   - Easy: Programiz, "Rust Variable Scope": https://www.programiz.com/rust/variable-scope
//
// You can learn data types from these sources:
//   - The Rust Book, "Data Types": https://doc.rust-lang.org/book/ch03-02-data-types.html
//   - Rust by Example, "Primitives": https://doc.rust-lang.org/rust-by-example/primitives.html
//   - Easy: W3Schools, "Rust Data Types": https://www.w3schools.com/rust/rust_data_types.php
//   - Easy: Let's Get Rusty (video), "Common Programming Concepts in Rust": https://www.youtube.com/watch?v=2V0JaMVjzws
//
// You can learn functions from these sources:
//   - The Rust Book, "Functions": https://doc.rust-lang.org/book/ch03-03-how-functions-work.html
//   - Rust by Example, "Functions": https://doc.rust-lang.org/rust-by-example/fn.html
//   - Comprehensive Rust, "Functions": https://google.github.io/comprehensive-rust/control-flow-basics/functions.html
//   - Easy: W3Schools, "Rust Functions": https://www.w3schools.com/rust/rust_functions.php
//
// You can learn control flow from these sources:
//   - The Rust Book, "Control Flow": https://doc.rust-lang.org/book/ch03-05-control-flow.html
//   - Rust by Example, "loop": https://doc.rust-lang.org/rust-by-example/flow_control/loop.html
//   - Comprehensive Rust, "Loops": https://google.github.io/comprehensive-rust/control-flow-basics/loops.html
//   - Easy: W3Schools, "Rust If .. Else Conditions": https://www.w3schools.com/rust/rust_if_else.php
//   - Easy: Easy Rust, "Loops": https://dhghomon.github.io/easy_rust/Chapter_26.html
