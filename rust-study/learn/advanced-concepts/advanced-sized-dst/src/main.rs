// Kitap: 22. bölüm, Sized, DST ve Geniş İşaretçiler

use std::mem::size_of;

trait Draw {
    fn draw(&self) -> String;
}
struct Button {
    name: String,
}
impl Draw for Button {
    fn draw(&self) -> String {
        format!("[{}]", self.name)
    }
}

fn main() {
    show_sizes();
    show_sized_bound();
    show_fat_pointer();
}

fn show_sizes() {
    let usize_size = size_of::<usize>();
    println!("usize (bir word) = {usize_size} bayt\n");

    println!("İNCE (thin) pointer'lar — tek word:");
    println!("  &i32       = {}", size_of::<&i32>());
    println!("  &Button    = {}", size_of::<&Button>());
    println!("  Box<i32>   = {}", size_of::<Box<i32>>());

    println!("\nFAT (geniş) pointer'lar — iki word (pointer + metadata):");
    println!("  &str       = {}", size_of::<&str>());
    println!("  &[i32]     = {}", size_of::<&[i32]>());
    println!("  &dyn Draw  = {}", size_of::<&dyn Draw>());
    println!("  Box<str>   = {}", size_of::<Box<str>>());
    println!("  Box<dyn Draw> = {}", size_of::<Box<dyn Draw>>());
}

fn print_line<T: std::fmt::Debug + ?Sized>(x: &T) {
    println!("  ?Sized ile: {x:?}");
}
fn show_sized_bound() {
    println!("?Sized gevşetmesiyle DST'leri &T olarak alabiliriz:");
    print_line("ben bir str'yim");
    print_line(&[1, 2, 3][..]);
    let n: i32 = 42;
    print_line(&n);
}

fn show_fat_pointer() {
    let array = [10, 20, 30, 40, 50];
    let head: &[i32] = &array[..3];
    let tail: &[i32] = &array[2..];
    println!(
        "slice fat pointer: head.len()={} tail.len()={} (ikinci word = uzunluk)",
        head.len(),
        tail.len()
    );

    let objects: Vec<Box<dyn Draw>> = vec![
        Box::new(Button {
            name: String::from("Tamam"),
        }),
        Box::new(Button {
            name: String::from("İptal"),
        }),
    ];
    for object in &objects {
        println!("trait object (vtable ile dispatch): {}", object.draw());
    }
}

// Learn more
//
// You can learn sized types and dynamically sized types (DST) from these sources:
//   - Rust Reference, "Dynamically sized types": https://doc.rust-lang.org/reference/dynamically-sized-types.html
//   - Rustonomicon, "Exotically Sized Types": https://doc.rust-lang.org/nomicon/exotic-sizes.html
//   - pretzelhammer, "Sizedness in Rust": https://github.com/pretzelhammer/rust-blog/blob/master/posts/sizedness-in-rust.md
//   - Easy: DEV Community, "Sized &? Sized: A beginner's guide": https://dev.to/masteringbackend/sized-sized-a-beginners-guide-448e
//   - Easy: Tour of Rust, "Handling Unsized Data": https://tourofrust.com/83_en.html
//
// You can learn the ?Sized bound from these sources:
//   - The Rust Book, "Dynamically Sized Types and the Sized Trait": https://doc.rust-lang.org/book/ch20-03-advanced-types.html#dynamically-sized-types-and-the-sized-trait
//   - pretzelhammer, "Sizedness in Rust": https://github.com/pretzelhammer/rust-blog/blob/master/posts/sizedness-in-rust.md
//   - Comprehensive Rust, "Statically Sized and Dynamically Sized types": https://google.github.io/comprehensive-rust/idiomatic/polymorphism/generics/sized.html
//   - Easy: DEV Community, "Sized &? Sized: A beginner's guide": https://dev.to/masteringbackend/sized-sized-a-beginners-guide-448e
//   - Easy: Let's Get Rusty (video), "Advanced Types in Rust": https://www.youtube.com/watch?v=aI1oIFqh44Y
//
// You can learn fat pointers from these sources:
//   - Crust of Rust (video), "Dispatch and Fat Pointers": https://www.youtube.com/watch?v=xcygqF5LVmM
//   - Rustonomicon, "Exotically Sized Types": https://doc.rust-lang.org/nomicon/exotic-sizes.html
//   - Rust Reference, "Trait object types": https://doc.rust-lang.org/reference/types/trait-object.html
//   - Easy: 0xpluie, "Static vs Dynamic Dispatch in Rust": https://pluiee.github.io/static-vs-dynamic-dispatch-in-rust/
