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
