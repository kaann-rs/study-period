// Kitap: 25. bölüm, Send ve Sync'in İç Yüzü

use std::cell::UnsafeCell;

fn main() {
    show_unsafe_cell();
    show_auto_traits();
}

struct MiniCell<T> {
    value: UnsafeCell<T>,
}
impl<T: Copy> MiniCell<T> {
    fn new(value: T) -> MiniCell<T> {
        MiniCell {
            value: UnsafeCell::new(value),
        }
    }
    fn get(&self) -> T {
        unsafe { *self.value.get() }
    }
    fn set(&self, new_value: T) {
        unsafe { *self.value.get() = new_value }
    }
}
fn show_unsafe_cell() {
    let cell = MiniCell::new(5);
    let r1 = &cell;
    let r2 = &cell;
    r1.set(10);
    println!("MiniCell: r1.set(10) sonrası r2.get() = {}", r2.get());
}

fn needs_send<T: Send>() {}
fn needs_sync<T: Sync>() {}

fn show_auto_traits() {
    needs_send::<i32>();
    needs_sync::<i32>();
    println!("i32: Send + Sync ✓ (sıradan değer)");

    needs_send::<MiniCell<i32>>();
    println!("MiniCell<i32>: Send ✓  (ama Sync değil — derleme-testi yorumda)");
}

// Learn more
//
// You can learn UnsafeCell from these sources:
//   - Standard library docs, "std::cell::UnsafeCell": https://doc.rust-lang.org/std/cell/struct.UnsafeCell.html
//   - Rust Reference, "Interior mutability": https://doc.rust-lang.org/reference/interior-mutability.html
//   - Crust of Rust (video), "Smart Pointers and Interior Mutability": https://www.youtube.com/watch?v=8O0Nt9qY_vo
//   - nyanpasu64, "An unsafe tour of Rust's Send and Sync": https://nyanpasu64.gitlab.io/blog/an-unsafe-tour-of-rust-s-send-and-sync/
//
// You can learn Send and Sync as auto traits from these sources:
//   - Rustonomicon, "Send and Sync": https://doc.rust-lang.org/nomicon/send-and-sync.html
//   - Rust Reference, "Auto traits": https://doc.rust-lang.org/reference/special-types-and-traits.html#auto-traits
//   - Crust of Rust (video), "Send, Sync, and their implementors": https://www.youtube.com/watch?v=yOezcP-XaIw
//   - Comprehensive Rust, "Marker Traits": https://google.github.io/comprehensive-rust/concurrency/send-sync/marker-traits.html
//   - Easy: 0xpluie, "Making Sense of Send and Sync in Rust": https://pluiee.github.io/making-sense-of-send-and-sync-in-rust/
