// Kitap: 17. bölüm, Nesne Yönelimli Özellikler

mod deep_dive;

fn main() {
    show_encapsulation();
    show_gui();
    show_type_state_blog();
    deep_dive::run();
}

#[derive(Default)]
pub struct AverageList {
    list: Vec<i32>,
    average: f64,
}

impl AverageList {
    pub fn new() -> AverageList {
        AverageList::default()
    }
    pub fn add(&mut self, value: i32) {
        self.list.push(value);
        self.update_average();
    }
    pub fn average(&self) -> f64 {
        self.average
    }
    fn update_average(&mut self) {
        let total: i32 = self.list.iter().sum();
        self.average = total as f64 / self.list.len() as f64;
    }
}
fn show_encapsulation() {
    let mut list = AverageList::new();
    list.add(10);
    list.add(20);
    list.add(30);
    println!("ortalama = {}", list.average());
}

trait Draw {
    fn draw(&self) -> String;
}
struct Button {
    label: String,
}
struct CheckBox {
    is_checked: bool,
}
impl Draw for Button {
    fn draw(&self) -> String {
        format!("[ {} ]", self.label)
    }
}
impl Draw for CheckBox {
    fn draw(&self) -> String {
        format!("({}) seçim", if self.is_checked { "x" } else { " " })
    }
}

struct Screen {
    components: Vec<Box<dyn Draw>>,
}
impl Screen {
    fn run(&self) {
        for component in &self.components {
            println!("  {}", component.draw());
        }
    }
}
fn show_gui() {
    let screen = Screen {
        components: vec![
            Box::new(Button {
                label: String::from("Kaydet"),
            }),
            Box::new(CheckBox { is_checked: true }),
            Box::new(Button {
                label: String::from("İptal"),
            }),
        ],
    };
    println!("ekran:");
    screen.run();
}

struct DraftPost {
    content: String,
}
struct ReviewPost {
    content: String,
}
struct PublishedPost {
    content: String,
}
impl DraftPost {
    fn new() -> DraftPost {
        DraftPost {
            content: String::new(),
        }
    }
    fn add(&mut self, text: &str) {
        self.content.push_str(text);
    }
    fn send_to_review(self) -> ReviewPost {
        ReviewPost {
            content: self.content,
        }
    }
}
impl ReviewPost {
    fn approve(self) -> PublishedPost {
        PublishedPost {
            content: self.content,
        }
    }
    fn reject(self) -> DraftPost {
        DraftPost {
            content: self.content,
        }
    }
}
impl PublishedPost {
    fn content(&self) -> &str {
        &self.content
    }
}
fn show_type_state_blog() {
    let mut post = DraftPost::new();
    post.add("Bugün Rust öğrendim. ");
    post.add("Çok iyiydi.");

    let in_review = post.send_to_review();

    let mut draft_again = in_review.reject();
    draft_again.add(" (düzeltildi)");
    let published = draft_again.send_to_review().approve();
    println!("yayınlanan içerik: {}", published.content());
}

// Learn more
//
// You can learn encapsulation from these sources:
//   - The Rust Book, "Characteristics of Object-Oriented Languages": https://doc.rust-lang.org/book/ch18-01-what-is-oo.html
//   - Comprehensive Rust, "Encapsulation": https://google.github.io/comprehensive-rust/modules/encapsulation.html
//   - Easy: Tour of Rust, "Abstraction With Selective Exposure": https://tourofrust.com/77_en.html
//   - Easy: Let's Get Rusty (video), "Object Oriented Programming in Rust": https://www.youtube.com/watch?v=UGDa0P2PXrY
//
// You can learn trait objects from these sources:
//   - The Rust Book, "Using Trait Objects to Abstract over Shared Behavior": https://doc.rust-lang.org/book/ch18-02-trait-objects.html
//   - Rust by Example, "Returning Traits with dyn": https://doc.rust-lang.org/rust-by-example/trait/dyn.html
//   - Comprehensive Rust, "dyn Trait": https://google.github.io/comprehensive-rust/generics/dyn-trait.html
//   - Easy: Easy Rust, "Box around traits": https://dhghomon.github.io/easy_rust/Chapter_54.html
//   - Easy: Let's Get Rusty (video), "Using Trait Objects in Rust": https://www.youtube.com/watch?v=ReBmm0eJg6g
//
// You can learn the type state pattern from these sources:
//   - Cliff Biffle, "The Typestate Pattern in Rust": https://cliffle.com/blog/rust-typestate/
//   - The Rust Book, "Encoding States and Behavior as Types": https://doc.rust-lang.org/book/ch18-03-oo-design-patterns.html#encoding-states-and-behavior-as-types
//   - Comprehensive Rust, "Typestate Pattern": https://google.github.io/comprehensive-rust/idiomatic/leveraging-the-type-system/typestate-pattern.html
//   - Easy: Let's Get Rusty (video), "State Design Pattern in Rust": https://www.youtube.com/watch?v=VFmPwvhubow
