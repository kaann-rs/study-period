// Kitap: 24. bölüm, Deref ve Metot Çözümleme

use std::ops::Deref;

fn main() {
    show_deref_coercion();
    show_autoderef();
    show_inherent_shadowing();
}

struct MyBox<T>(T);
impl<T> Deref for MyBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}
fn greet(name: &str) {
    println!("  selam, {name}");
}
fn show_deref_coercion() {
    let boxed = MyBox(5);
    println!("*MyBox(5) = {}", *boxed);

    let name = MyBox(String::from("Kaan"));
    greet(&name);
}

fn show_autoderef() {
    let boxed = Box::new(vec![1, 2, 3]);
    println!("autoderef: Box<Vec<i32>>.len() = {}", boxed.len());

    let mut v = vec![1, 2, 3];
    v.push(4);
    println!("autoref: v.push(4) == (&mut v).push(4) -> {v:?}");

    let s = String::from("merhaba");
    println!("autoref &self: s.len() = {}", s.len());
}

struct Inner;
impl Inner {
    fn name(&self) -> &str {
        "IC"
    }
}
struct Wrapper(Inner);
impl Deref for Wrapper {
    type Target = Inner;
    fn deref(&self) -> &Inner {
        &self.0
    }
}
impl Wrapper {
    fn name(&self) -> &str {
        "SARMAL"
    }
}
fn show_inherent_shadowing() {
    let wrapper = Wrapper(Inner);
    println!(
        "wrapper.name()        = {} (inherent kazanır, Deref'e inilmez)",
        wrapper.name()
    );
    println!(
        "(*wrapper).name()     = {} (elle deref → Inner)",
        (*wrapper).name()
    );
    println!(
        "Inner::name(&wrapper) = {} (tam-nitelikli, &Wrapper coerce)",
        Inner::name(&wrapper)
    );
}
