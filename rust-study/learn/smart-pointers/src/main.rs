// Kitap: 14. bölüm, Akıllı İşaretçiler

use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

mod deep_dive;

fn main() {
    show_box();
    show_deref();
    show_drop();
    show_rc();
    show_refcell();
    deep_dive::run();
}

#[derive(Debug)]
enum List {
    Node(i32, Box<List>),
    End,
}

fn sum(list: &List) -> i32 {
    match list {
        List::Node(value, next) => value + sum(next),
        List::End => 0,
    }
}
fn show_box() {
    use List::*;
    let list = Node(1, Box::new(Node(2, Box::new(End))));
    println!("cons list (Box): {list:?}, toplam = {}", sum(&list));
}

struct MyBox<T>(T);
impl<T> MyBox<T> {
    fn new(x: T) -> MyBox<T> {
        MyBox(x)
    }
}
impl<T> Deref for MyBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}
fn greet(name: &str) {
    println!("Selam, {name}!");
}
fn show_deref() {
    let boxed = MyBox::new(5);
    println!("*MyBox(5) = {}", *boxed);

    let name = MyBox::new(String::from("Kaan"));
    greet(&name);
}

struct Resource {
    name: String,
}
impl Drop for Resource {
    fn drop(&mut self) {
        println!("  Drop: '{}' serbest bırakıldı", self.name);
    }
}
fn show_drop() {
    let _a = Resource {
        name: String::from("A"),
    };
    let b = Resource {
        name: String::from("B"),
    };
    drop(b);
    println!("  (show_drop sonu — _a şimdi düşecek)");
}

#[derive(Debug)]
enum RcList {
    Node(i32, Rc<RcList>),
    End,
}
fn rc_sum(list: &RcList) -> i32 {
    match list {
        RcList::Node(value, next) => value + rc_sum(next),
        RcList::End => 0,
    }
}
fn show_rc() {
    use RcList::*;
    let a = Rc::new(Node(5, Rc::new(Node(10, Rc::new(End)))));
    println!("a oluştu, sayaç = {}", Rc::strong_count(&a));
    let b = Node(3, Rc::clone(&a));
    println!("b paylaştı, sayaç = {}", Rc::strong_count(&a));
    println!("  b toplamı = {}", rc_sum(&b));
    {
        let c = Node(4, Rc::clone(&a));
        println!("c paylaştı, sayaç = {}", Rc::strong_count(&a));
        println!("  c toplamı = {}", rc_sum(&c));
    }
    println!("c düştü, sayaç = {}", Rc::strong_count(&a));
}

fn show_refcell() {
    let shared = Rc::new(RefCell::new(0_i32));
    let a = Rc::clone(&shared);
    let b = Rc::clone(&shared);

    *a.borrow_mut() += 10;
    *b.borrow_mut() += 5;
    println!("ortak sayaç = {}", shared.borrow());

    println!("  (RefCell borrow kuralı çalışma anında denetlenir)");
}

// Learn more
//
// You can learn Box from these sources:
//   - The Rust Book, "Using Box<T> to Point to Data on the Heap": https://doc.rust-lang.org/book/ch15-01-box.html
//   - Rust by Example, "Box, stack and heap": https://doc.rust-lang.org/rust-by-example/std/box.html
//   - Comprehensive Rust, "Box<T>": https://google.github.io/comprehensive-rust/smart-pointers/box.html
//   - Easy: Easy Rust, "Box": https://dhghomon.github.io/easy_rust/Chapter_53.html
//   - Easy: Let's Get Rusty (video), "The Box Smart Pointer in Rust": https://www.youtube.com/watch?v=m76sRj2VgGo
//
// You can learn Deref from these sources:
//   - The Rust Book, "Treating Smart Pointers Like Regular References": https://doc.rust-lang.org/book/ch15-02-deref.html
//   - Standard library docs, "std::ops::Deref": https://doc.rust-lang.org/std/ops/trait.Deref.html
//   - Rust Design Patterns, "Collections Are Smart Pointers": https://rust-unofficial.github.io/patterns/idioms/deref.html
//   - Easy: Easy Rust, "Deref and DerefMut": https://dhghomon.github.io/easy_rust/Chapter_56.html
//   - Easy: Let's Get Rusty (video), "Smart Pointers in Rust - The Deref Trait": https://www.youtube.com/watch?v=dYEC6NElVOg
//
// You can learn Drop from these sources:
//   - The Rust Book, "Running Code on Cleanup with the Drop Trait": https://doc.rust-lang.org/book/ch15-03-drop.html
//   - Rust by Example, "Drop": https://doc.rust-lang.org/rust-by-example/trait/drop.html
//   - Comprehensive Rust, "Drop": https://google.github.io/comprehensive-rust/memory-management/drop.html
//   - Easy: Let's Get Rusty (video), "Smart Pointers in Rust - The Drop Trait": https://www.youtube.com/watch?v=RPWZcTYBS4k
//
// You can learn Rc from these sources:
//   - The Rust Book, "Rc<T>, the Reference Counted Smart Pointer": https://doc.rust-lang.org/book/ch15-04-rc.html
//   - Rust by Example, "Rc": https://doc.rust-lang.org/rust-by-example/std/rc.html
//   - Comprehensive Rust, "Rc": https://google.github.io/comprehensive-rust/smart-pointers/rc.html
//   - Easy: Easy Rust, "Rc": https://dhghomon.github.io/easy_rust/Chapter_45.html
//   - Easy: Let's Get Rusty (video), "Smart Pointers in Rust - Reference Counting": https://www.youtube.com/watch?v=M9Owp3iLigg
//
// You can learn RefCell from these sources:
//   - The Rust Book, "RefCell<T> and the Interior Mutability Pattern": https://doc.rust-lang.org/book/ch15-05-interior-mutability.html
//   - Comprehensive Rust, "RefCell": https://google.github.io/comprehensive-rust/borrowing/interior-mutability/refcell.html
//   - Standard library docs, "std::cell::RefCell": https://doc.rust-lang.org/std/cell/struct.RefCell.html
//   - Easy: Easy Rust, "Interior mutability": https://dhghomon.github.io/easy_rust/Chapter_41.html
//   - Easy: Let's Get Rusty (video), "Smart Pointers in Rust - Interior Mutability": https://www.youtube.com/watch?v=77aRH6YBKyY
