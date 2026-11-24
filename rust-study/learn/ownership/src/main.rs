// Kitap: 3. bölüm, Ownership (Sahiplik)

mod deep_dive;

fn main() {
    show_ownership_and_move();
    show_copy_and_move();
    show_clone();
    show_scope_and_drop();
    show_shared_borrow();
    show_mutable_borrow();
    show_no_dangling();
    show_slices();
    deep_dive::run();
}

fn show_ownership_and_move() {
    let s1 = String::from("merhaba");
    let s2 = s1;
    println!("s2 = {s2}");
}

fn show_copy_and_move() {
    let x = 5;
    let y = x;
    println!("x = {x}, y = {y}");
}

fn show_clone() {
    let s1 = String::from("ada");
    let s2 = s1.clone();
    println!("s1 = {s1}, s2 = {s2}");
}

fn show_scope_and_drop() {
    {
        let s = String::from("geçici");
        println!("içeride: {s}");
    }
}

fn show_shared_borrow() {
    let s = String::from("kitap");
    let n = calculate_length(&s);
    println!("'{s}' uzunluğu {n}");
}
fn calculate_length(x: &String) -> usize {
    x.len()
}

fn show_mutable_borrow() {
    let mut s = String::from("merhaba");
    add_world(&mut s);
    println!("{s}");

    let r1 = &s;
    let r2 = &s;
    println!("{r1} {r2}");

    let r3 = &mut s;
    r3.push('!');
    println!("{r3}");
}
fn add_world(x: &mut String) {
    x.push_str(" dünya");
}

fn show_no_dangling() {
    let s = make_owned();
    println!("sarkmayan: {s}");
}
fn make_owned() -> String {
    String::from("x")
}

fn show_slices() {
    let s = String::from("merhaba dünya");
    let first = first_word(&s);
    println!("ilk kelime: {first}");

    let array = [1, 2, 3, 4, 5];
    let middle = &array[1..4];
    println!("orta = {middle:?}");
}
fn first_word(s: &str) -> &str {
    match s.find(' ') {
        Some(i) => &s[..i],
        None => s,
    }
}

// Learn more
//
// You can learn ownership and move from these sources:
//   - The Rust Book, "What is Ownership?": https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html
//   - Brown University Rust Book, "What is Ownership?": https://rust-book.cs.brown.edu/ch04-01-what-is-ownership.html
//   - Rust by Example, "Ownership and moves": https://doc.rust-lang.org/rust-by-example/scope/move.html
//   - Easy: W3Schools, "Rust Ownership": https://www.w3schools.com/rust/rust_ownership.php
//   - Easy: Let's Get Rusty (video), "Understanding Ownership in Rust": https://www.youtube.com/watch?v=VFIOSWy93H0
//
// You can learn Copy types from these sources:
//   - The Rust Book, "Stack-Only Data: Copy": https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html#stack-only-data-copy
//   - Comprehensive Rust, "Copy Types": https://google.github.io/comprehensive-rust/memory-management/copy-types.html
//   - Standard library docs, "std::marker::Copy": https://doc.rust-lang.org/std/marker/trait.Copy.html
//   - Easy: Easy Rust, "Copy types": https://dhghomon.github.io/easy_rust/Chapter_19.html
//
// You can learn clone from these sources:
//   - The Rust Book, "Variables and Data Interacting with Clone": https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html#variables-and-data-interacting-with-clone
//   - Rust by Example, "Clone": https://doc.rust-lang.org/rust-by-example/trait/clone.html
//   - Comprehensive Rust, "Clone": https://google.github.io/comprehensive-rust/memory-management/clone.html
//   - Easy: W3Schools, "Rust Ownership": https://www.w3schools.com/rust/rust_ownership.php
//
// You can learn scope and drop from these sources:
//   - Rust by Example, "RAII": https://doc.rust-lang.org/rust-by-example/scope/raii.html
//   - The Rust Book, "Running Code on Cleanup with the Drop Trait": https://doc.rust-lang.org/book/ch15-03-drop.html
//   - Comprehensive Rust, "Drop": https://google.github.io/comprehensive-rust/memory-management/drop.html
//   - Easy: Tour of Rust, "Scope-Based Resource Management": https://tourofrust.com/44_en.html
//
// You can learn shared borrows from these sources:
//   - The Rust Book, "References and Borrowing": https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html
//   - Rust by Example, "Borrowing": https://doc.rust-lang.org/rust-by-example/scope/borrow.html
//   - Comprehensive Rust, "Shared References": https://google.github.io/comprehensive-rust/references/shared.html
//   - Easy: W3Schools, "Rust Borrowing and References": https://www.w3schools.com/rust/rust_borrowing_references.php
//   - Easy: Easy Rust, "More on references": https://dhghomon.github.io/easy_rust/Chapter_16.html
//
// You can learn mutable borrows from these sources:
//   - The Rust Book, "Mutable References": https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html#mutable-references
//   - Rust by Example, "Mutability": https://doc.rust-lang.org/rust-by-example/scope/borrow/mut.html
//   - Comprehensive Rust, "Exclusive References": https://google.github.io/comprehensive-rust/references/exclusive.html
//   - Easy: Easy Rust, "Mutable references": https://dhghomon.github.io/easy_rust/Chapter_17.html
//
// You can learn dangling references from these sources:
//   - The Rust Book, "Dangling References": https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html#dangling-references
//   - Brown University Rust Book, "References and Borrowing": https://rust-book.cs.brown.edu/ch04-02-references-and-borrowing.html
//   - Comprehensive Rust, "Reference Validity": https://google.github.io/comprehensive-rust/references/dangling.html
//   - Easy: Tour of Rust, "Passing Around Borrowed Data": https://tourofrust.com/51_en.html
//
// You can learn slices from these sources:
//   - The Rust Book, "The Slice Type": https://doc.rust-lang.org/book/ch04-03-slices.html
//   - Rust by Example, "Arrays and Slices": https://doc.rust-lang.org/rust-by-example/primitives/array.html
//   - Comprehensive Rust, "Slices": https://google.github.io/comprehensive-rust/references/slices.html
//   - Easy: Programiz, "Rust Slice": https://www.programiz.com/rust/slice
//   - Easy: Let's Get Rusty (video), "Understanding Ownership in Rust": https://www.youtube.com/watch?v=VFIOSWy93H0
