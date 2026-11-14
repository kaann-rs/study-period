//! Prosedürel makro çalışması.
//!
//! Bu bir proc-macro kütüphanesidir: derleyici, kodu derlerken bu fonksiyonları
//! çalıştırır ve dönen TokenStream'i kaynak koda ENJEKTE eder. Yani makro,
//! derleme zamanında kod üretir. Böyle bir crate içinde normal kod çalıştırılamaz
//! (fn main olmaz); makroyu ancak BAŞKA bir crate kullanabilir.

use proc_macro::TokenStream;

/// Fonksiyon benzeri makro: `cevap_uret!()` yazdığın yere
/// `fn cevap() -> u32 { 42 }` fonksiyonunu üretir.
///
/// TokenStream, kaynak kodun "token"lara (sözcük parçaları) ayrılmış hâlidir.
/// Burada girdiyi yok sayıp düz bir metni ayrıştırarak (parse) token'a çeviriyoruz.
#[proc_macro]
pub fn cevap_uret(_girdi: TokenStream) -> TokenStream {
    "fn cevap() -> u32 { 42 }".parse().unwrap()
}
