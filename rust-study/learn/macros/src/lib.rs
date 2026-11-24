// Kitap: 20. bölüm, Makrolar

use proc_macro::TokenStream;

#[proc_macro]
pub fn make_answer(_input: TokenStream) -> TokenStream {
    "fn answer() -> u32 { 42 }".parse().unwrap()
}
