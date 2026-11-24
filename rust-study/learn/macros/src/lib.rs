// Kitap: 20. bölüm, Makrolar

use proc_macro::TokenStream;

#[proc_macro]
pub fn make_answer(_input: TokenStream) -> TokenStream {
    "fn answer() -> u32 { 42 }".parse().unwrap()
}

// Learn more
//
// You can learn function-like procedural macros from these sources:
//   - The Rust Book, "Function-Like Macros": https://doc.rust-lang.org/book/ch20-05-macros.html#function-like-macros
//   - Rust Reference, "The proc_macro attribute": https://doc.rust-lang.org/reference/procedural-macros.html#the-proc_macro-attribute
//   - Little Book of Rust Macros, "Function-like": https://lukaswirth.dev/tlborm/proc-macros/methodical/function-like.html
//   - Easy: freeCodeCamp, "Procedural Macros in Rust – A Handbook for Beginners": https://www.freecodecamp.org/news/procedural-macros-in-rust/
//   - Easy: Let's Get Rusty (video), "Procedural Macros in Rust": https://www.youtube.com/watch?v=crWfcA064is
//
// You can learn token streams from these sources:
//   - Comprehensive Rust, "Token Streams": https://google.github.io/comprehensive-rust/macros/streams.html
//   - docs.rs, "proc_macro2": https://docs.rs/proc-macro2/latest/proc_macro2/
//   - dtolnay, "proc-macro-workshop": https://github.com/dtolnay/proc-macro-workshop
//   - Easy: freeCodeCamp, "Procedural Macros in Rust – A Handbook for Beginners": https://www.freecodecamp.org/news/procedural-macros-in-rust/
