// Kitap: 4. bölüm, Struct'lar (Yapılar)

#[derive(Debug)]
struct User {
    name: String,
    email: String,
    is_active: bool,
    login_count: u64,
}

struct Quote<'a> {
    text: &'a str,
}

struct Color(u8, u8, u8);

struct Unit;

#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
    fn area(&self) -> u32 {
        self.width * self.height
    }
    fn scale(&mut self, factor: u32) {
        self.width *= factor;
        self.height *= factor;
    }
    fn into_sides(self) -> (u32, u32) {
        (self.width, self.height)
    }
}

mod deep_dive;

fn main() {
    show_define_and_create();
    show_update_syntax();
    show_tuple_and_unit();
    show_owned_fields();
    show_methods();
    show_debug_output();
    deep_dive::run();
}

fn create_user(name: String, email: String) -> User {
    User {
        name,
        email,
        is_active: true,
        login_count: 1,
    }
}

fn show_define_and_create() {
    let mut user = create_user("Kaan".into(), "k@x.com".into());
    user.login_count += 1;
    println!("{user:?}");
    println!(
        "{} giriş: {} aktif: {}",
        user.name, user.login_count, user.is_active
    );
}

fn show_update_syntax() {
    let user1 = create_user("Ada".into(), "a@x.com".into());
    let user2 = User {
        name: "Ada II".into(),
        ..user1
    };
    println!("{} / {}", user2.name, user2.email);
}

fn show_tuple_and_unit() {
    let black = Color(0, 0, 0);
    println!("R={} G={} B={}", black.0, black.1, black.2);

    let _unit = Unit;
}

fn show_owned_fields() {
    let s = String::from("merhaba dünya");
    let quote = Quote { text: &s[0..7] };
    println!("alıntı: {}", quote.text);
}

fn show_methods() {
    let mut rectangle = Rectangle::new(3, 4);
    println!("alan = {}", rectangle.area());
    rectangle.scale(2);
    println!("ölçekli alan = {} ({:?})", rectangle.area(), rectangle);

    let (width, height) = rectangle.into_sides();
    println!("yok edildi: {width}x{height}");
}

fn show_debug_output() {
    let rectangle = Rectangle::new(6, 7);
    println!("{rectangle:?}");
    println!("{rectangle:#?}");
}

// Learn more
//
// You can learn structs from these sources:
//   - The Rust Book, "Defining and Instantiating Structs": https://doc.rust-lang.org/book/ch05-01-defining-structs.html
//   - Rust by Example, "Structures": https://doc.rust-lang.org/rust-by-example/custom_types/structs.html
//   - Comprehensive Rust, "Named Structs": https://google.github.io/comprehensive-rust/user-defined-types/named-structs.html
//   - Easy: W3Schools, "Rust Structs (Structures)": https://www.w3schools.com/rust/rust_structs.php
//   - Easy: Let's Get Rusty (video), "Structs in Rust": https://www.youtube.com/watch?v=n3bPhdiJm9I
//
// You can learn struct update syntax from these sources:
//   - The Rust Book, "Creating Instances with Struct Update Syntax": https://doc.rust-lang.org/book/ch05-01-defining-structs.html#creating-instances-with-struct-update-syntax
//   - Rust Reference, "Functional update syntax": https://doc.rust-lang.org/reference/expressions/struct-expr.html#functional-update-syntax
//   - Comprehensive Rust, "Named Structs": https://google.github.io/comprehensive-rust/user-defined-types/named-structs.html
//   - Easy: Let's Get Rusty (video), "Structs in Rust": https://www.youtube.com/watch?v=n3bPhdiJm9I
//
// You can learn tuple structs and unit structs from these sources:
//   - The Rust Book, "Creating Different Types with Tuple Structs": https://doc.rust-lang.org/book/ch05-01-defining-structs.html#creating-different-types-with-tuple-structs
//   - Comprehensive Rust, "Tuple Structs": https://google.github.io/comprehensive-rust/user-defined-types/tuple-structs.html
//   - Rust Reference, "Structs": https://doc.rust-lang.org/reference/items/structs.html
//   - Easy: Tour of Rust, "Tuple-like Structs": https://tourofrust.com/27_en.html
//
// You can learn owned and borrowed fields from these sources:
//   - The Rust Book, "Ownership of Struct Data": https://doc.rust-lang.org/book/ch05-01-defining-structs.html#ownership-of-struct-data
//   - Rust by Example, "Structs": https://doc.rust-lang.org/rust-by-example/scope/lifetime/struct.html
//   - Comprehensive Rust, "Lifetimes in Data Structures": https://google.github.io/comprehensive-rust/lifetimes/struct-lifetimes.html
//   - Easy: Easy Rust, "Lifetimes": https://dhghomon.github.io/easy_rust/Chapter_40.html
//
// You can learn methods from these sources:
//   - The Rust Book, "Methods": https://doc.rust-lang.org/book/ch05-03-method-syntax.html
//   - Rust by Example, "Methods": https://doc.rust-lang.org/rust-by-example/fn/methods.html
//   - Comprehensive Rust, "Methods": https://google.github.io/comprehensive-rust/methods-and-traits/methods.html
//   - Easy: Easy Rust, "Implementing structs and enums": https://dhghomon.github.io/easy_rust/Chapter_27.html
//   - Easy: Tour of Rust, "Calling Methods": https://tourofrust.com/24_en.html
//
// You can learn how to print structs with Debug from these sources:
//   - The Rust Book, "Adding Functionality with Derived Traits": https://doc.rust-lang.org/book/ch05-02-example-structs.html#adding-functionality-with-derived-traits
//   - Rust by Example, "Debug": https://doc.rust-lang.org/rust-by-example/hello/print/print_debug.html
//   - Standard library docs, "std::fmt::Debug": https://doc.rust-lang.org/std/fmt/trait.Debug.html
//   - Easy: Easy Rust, "Display and debug": https://dhghomon.github.io/easy_rust/Chapter_10.html
