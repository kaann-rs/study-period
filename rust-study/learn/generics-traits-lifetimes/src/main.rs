// Kitap: 9. bölüm, Generic, Trait ve Lifetime

use std::fmt::Display;

mod deep_dive;

fn main() {
    show_generics();
    show_traits();
    show_lifetimes();
    deep_dive::run();
}

fn show_generics() {
    fn largest<T: PartialOrd>(list: &[T]) -> &T {
        let mut largest = &list[0];
        for x in list {
            if x > largest {
                largest = x;
            }
        }
        largest
    }
    println!("en büyük (i32)  = {}", largest(&[3, 7, 2, 9, 4]));
    println!("en büyük (char) = {}", largest(&['a', 'z', 'm']));

    #[derive(Debug)]
    struct Point<T> {
        x: T,
        y: T,
    }
    impl<T> Point<T> {
        fn x(&self) -> &T {
            &self.x
        }
    }
    impl Point<f64> {
        fn distance(&self) -> f64 {
            (self.x * self.x + self.y * self.y).sqrt()
        }
    }
    let int_point = Point { x: 3, y: 4 };
    let float_point = Point { x: 3.0, y: 4.0 };
    println!(
        "int_point.x = {}, float_point.distance = {}",
        int_point.x(),
        float_point.distance()
    );
    println!("int_point = {int_point:?} (y = {})", int_point.y);
}

trait Summary {
    fn author(&self) -> String;
    fn summarize(&self) -> String {
        format!("(devamı...) — {}", self.author())
    }
}

struct Article {
    title: String,
    author: String,
}
struct Tweet {
    username: String,
}
impl Summary for Article {
    fn author(&self) -> String {
        self.author.clone()
    }
    fn summarize(&self) -> String {
        format!("{} — {}", self.title, self.author)
    }
}
impl Summary for Tweet {
    fn author(&self) -> String {
        format!("@{}", self.username)
    }
}

fn announce(item: &impl Summary) {
    println!("Yeni: {}", item.summarize());
}
fn _announce_long<T: Summary>(item: &T) {
    println!("Yeni: {}", item.summarize());
}

fn show_traits() {
    let article = Article {
        title: String::from("Rust 2024"),
        author: String::from("Kaan"),
    };
    let tweet = Tweet {
        username: String::from("kaan"),
    };
    announce(&article);
    announce(&tweet);

    let mixed: Vec<Box<dyn Summary>> = vec![
        Box::new(Article {
            title: String::from("Hava"),
            author: String::from("Ajans"),
        }),
        Box::new(Tweet {
            username: String::from("bot"),
        }),
    ];
    for item in &mixed {
        println!("liste: {}", item.summarize());
    }
}

trait Label {
    fn label(&self) -> String;
}
impl<T: Display> Label for T {
    fn label(&self) -> String {
        format!("[{self}]")
    }
}

fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

struct Quote<'a> {
    part: &'a str,
}
impl<'a> Quote<'a> {
    fn part(&self) -> &str {
        self.part
    }
}

fn show_lifetimes() {
    let s1 = String::from("uzun bir cümle");
    let result;
    {
        let s2 = String::from("kısa");
        result = longest(s1.as_str(), s2.as_str());
        println!("en uzun = {result}");
    }

    let text = String::from("İlk cümle. İkincisi.");
    let first = text.split('.').next().unwrap();
    let quote = Quote { part: first };
    println!("alıntı: {}", quote.part());

    let s: &'static str = "program boyu yaşar";
    println!("{s} / etiket örneği: {}", 42.label());
}

// Learn more
//
// You can learn generics from these sources:
//   - The Rust Book, "Generic Data Types": https://doc.rust-lang.org/book/ch10-01-syntax.html
//   - Rust by Example, "Generics": https://doc.rust-lang.org/rust-by-example/generics.html
//   - Comprehensive Rust, "Generic Functions": https://google.github.io/comprehensive-rust/generics/generic-functions.html
//   - Easy: Programiz, "Rust Generics": https://www.programiz.com/rust/generics
//   - Easy: Let's Get Rusty (video), "Generic Types in Rust": https://www.youtube.com/watch?v=6rcTSxPJ6Bw
//
// You can learn traits from these sources:
//   - The Rust Book, "Defining Shared Behavior with Traits": https://doc.rust-lang.org/book/ch10-02-traits.html
//   - Rust by Example, "Traits": https://doc.rust-lang.org/rust-by-example/trait.html
//   - Comprehensive Rust, "Traits": https://google.github.io/comprehensive-rust/methods-and-traits/traits.html
//   - Easy: Easy Rust, "Traits": https://dhghomon.github.io/easy_rust/Chapter_34.html
//   - Easy: Let's Get Rusty (video), "Traits in Rust": https://www.youtube.com/watch?v=T0Xfltu4h3A
//
// You can learn lifetimes from these sources:
//   - The Rust Book, "Validating References with Lifetimes": https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html
//   - Rust by Example, "Lifetimes": https://doc.rust-lang.org/rust-by-example/scope/lifetime.html
//   - Comprehensive Rust, "Lifetimes": https://google.github.io/comprehensive-rust/lifetimes.html
//   - Easy: Easy Rust, "Lifetimes": https://dhghomon.github.io/easy_rust/Chapter_40.html
//   - Easy: Let's Get Rusty (video), "Rust Lifetimes Finally Explained!": https://www.youtube.com/watch?v=juIINGuZyBc
