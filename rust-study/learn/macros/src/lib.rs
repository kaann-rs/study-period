// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Bu dosya çok kısadır; önce bu kutuyu, sonra altındaki açıklamaları oku.
//
// Bu dosyanın tek cümlelik özeti: Prosedürel makro, derleyicinin senin kodunu derlerken
// çağırdığı küçük bir yardımcıdır: kod parçaları alır, yerine yeni kod parçaları verir.
//
// Benzetme: Bir lastik damga hazırlıyorsun. Damgayı hazırlayan atölye (bu crate) ile damgayı
// kâğıda basan kişi (makroyu kullanan başka bir crate) ayrı yerlerdedir. Bu yüzden bu dosyada
// `fn main` yoktur; damganın izini görmek için tests/kullanim.rs dosyasına bakarsın.
//
// MİNİ SÖZLÜK (bu dosyada geçen kelimeler):
//   makro                    → kod yazan kod; çağrıldığı yere derlemeden önce kod yerleştirir.
//   prosedürel makro (proc-macro) → sıradan bir Rust fonksiyonu olarak yazılan makro.
//   token                    → kaynak kodun en küçük parçası: bir ad, bir sayı, bir parantez.
//   TokenStream              → art arda dizilmiş token'lar; makronun aldığı ve verdiği şey.
//   parse (ayrıştırmak)      → düz yazıyı okuyup token'lara çevirmek.
//   enjekte etmek            → üretilen kodu, makronun çağrıldığı yere yerleştirmek.
//   derleme zamanı           → programın çalıştığı an değil, derleyicinin kodu çevirdiği an.
//   crate                    → tek başına derlenen Rust paketi (kütüphane ya da program).

//! Prosedürel makro çalışması.
//!
//! Bu bir proc-macro kütüphanesidir: derleyici, kodu derlerken bu fonksiyonları
//! çalıştırır ve dönen TokenStream'i kaynak koda ENJEKTE eder. Yani makro,
//! derleme zamanında kod üretir. Böyle bir crate içinde normal kod çalıştırılamaz
//! (fn main olmaz); makroyu ancak BAŞKA bir crate kullanabilir.

use proc_macro::TokenStream;

// SADE ANLATIM: Aşağıdaki fonksiyon damganın kendisidir. Biri kodunda `cevap_uret!();` yazarsa
// derleyici bu fonksiyonu çağırır ve o satırın yerine fonksiyonun döndürdüğü kodu koyar:
// `fn cevap() -> u32 { 42 }`. Yani kullanıcı tek satır yazar, elinde hazır bir fonksiyon olur.
// Bu makro kendisine verilen girdiye hiç bakmaz (`_girdi`); gerçek makrolar girdiyi okuyup ona
// göre kod üretir.
// Neden böyle bir şey var? Aynı kalıptaki kodu elle tekrar tekrar yazmak yerine derleyiciye
// yazdırmak için; `#[derive(Debug)]` de bu yolla çalışır.
//   Mini örnek (başka bir crate'te):
//     use macros::cevap_uret;
//     cevap_uret!();                   // derleyici buraya `fn cevap() -> u32 { 42 }` yazar
//     assert_eq!(cevap(), 42);         // ✓ makronun ürettiği fonksiyon çağrılabiliyor
//     // bu dosyanın içinde cevap_uret!() // ✗ derlenmez: makro tanımlandığı crate'te kullanılamaz
/// Fonksiyon benzeri makro: `cevap_uret!()` yazdığın yere
/// `fn cevap() -> u32 { 42 }` fonksiyonunu üretir.
///
/// TokenStream, kaynak kodun "token"lara (sözcük parçaları) ayrılmış hâlidir.
/// Burada girdiyi yok sayıp düz bir metni ayrıştırarak (parse) token'a çeviriyoruz.
#[proc_macro]
pub fn cevap_uret(_girdi: TokenStream) -> TokenStream {
    "fn cevap() -> u32 { 42 }".parse().unwrap()
}
