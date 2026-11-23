// Makroyu DIŞARIDAN kullanan entegrasyon testi. tests/ altındaki her dosya ayrı
// bir crate gibi derlenir ve `macros` crate'ine bağımlıdır; makro burada gerçekten
// genişler. Çalıştırmak için: cargo test -p macros

// SADE ANLATIM: src/lib.rs'te hazırlanan damga burada kâğıda basılır. `cevap_uret!();` satırı
// derlenirken yerini `fn cevap() -> u32 { 42 }` fonksiyonuna bırakır; alttaki test de o
// fonksiyonu çağırıp 42 döndüğüne bakar. Bu dosyada `cevap` adında bir fonksiyon YAZILI değildir,
// yine de çağrılabilir, çünkü onu makro üretmiştir.
// Neden ayrı bir dosyada? Prosedürel makro, tanımlandığı crate'in içinde kullanılamaz; tests/
// altındaki her dosya ayrı bir crate gibi derlendiği için burada kullanılabilir.
// (Sade: entegrasyon testi = kütüphaneyi dışarıdan, gerçek bir kullanıcı gibi deneyen test.)
//   Mini örnek:
//     cevap_uret!();               // derleyici buraya `fn cevap() -> u32 { 42 }` yazar
//     assert_eq!(cevap(), 42);     // ✓ üretilen fonksiyon 42 döndürür

use macros::cevap_uret;

// Makro burada `fn cevap() -> u32 { 42 }` fonksiyonunu üretir.
cevap_uret!();

#[test]
fn cevap_42_doner() {
    assert_eq!(cevap(), 42);
}
