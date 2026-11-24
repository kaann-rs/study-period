// Kitap: 26. bölüm, Düşürme Denetimi (Drop Check)

use std::marker::PhantomData;
use std::mem::size_of;

fn main() {
    show_drop_check();
    show_phantom_data();
}

struct Inspector<'a>(&'a u8);
impl<'a> Drop for Inspector<'a> {
    fn drop(&mut self) {
        println!("  Inspector düşerken okuyor: {}", self.0);
    }
}

fn show_drop_check() {
    let day = 5u8;
    let inspector = Inspector(&day);
    println!("dropck: değer = {}", inspector.0);
}

struct Token<T> {
    id: u32,
    _owner: PhantomData<T>,
}

fn show_phantom_data() {
    let token: Token<String> = Token {
        id: 7,
        _owner: PhantomData,
    };
    println!("PhantomData ZST: Token.id = {}", token.id);
    println!(
        "  size_of::<Token<String>>() = {} (== u32 = {})",
        size_of::<Token<String>>(),
        size_of::<u32>()
    );
}

// Learn more
//
// You can learn the drop check from these sources:
//   - Rustonomicon, "Drop Check": https://doc.rust-lang.org/nomicon/dropck.html
//   - Standard library docs, "Drop check": https://doc.rust-lang.org/std/ops/trait.Drop.html#drop-check
//   - Crust of Rust (video), "The Drop Check": https://www.youtube.com/watch?v=TJOFSMpJdzg
//   - std developers guide, "Drop and #[may_dangle]": https://std-dev-guide.rust-lang.org/tricky/may-dangle.html
//   - Easy: Tour of Rust, "Dropping is Hierarchical": https://tourofrust.com/45_en.html
//
// You can learn PhantomData from these sources:
//   - Rustonomicon, "PhantomData": https://doc.rust-lang.org/nomicon/phantom-data.html
//   - Standard library docs, "std::marker::PhantomData": https://doc.rust-lang.org/std/marker/struct.PhantomData.html
//   - Rust by Example, "Phantom type parameters": https://doc.rust-lang.org/rust-by-example/generics/phantom.html
//   - Comprehensive Rust, "PhantomData and Types": https://google.github.io/comprehensive-rust/idiomatic/leveraging-the-type-system/borrow-checker-invariants/phantomdata-01-types.html
//   - Easy: Rust by Example, "Testcase: unit clarification": https://doc.rust-lang.org/rust-by-example/generics/phantom/testcase_units.html
