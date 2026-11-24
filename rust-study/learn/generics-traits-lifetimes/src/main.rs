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
