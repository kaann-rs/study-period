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
