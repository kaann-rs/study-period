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
