// Açıklama: learn/macros/ACIKLAMA.md
// Makroyu DIŞARIDAN kullanan entegrasyon testi. tests/ altındaki her dosya ayrı
// bir crate gibi derlenir ve `macros` crate'ine bağımlıdır; makro burada gerçekten
// genişler. Çalıştırmak için: cargo test -p macros

// SADE ANLATIM: src/lib.rs'te hazırlanan damga burada kâğıda basılır: `cevap_uret!();` satırı
// derlenirken `fn cevap() -> u32 { 42 }` fonksiyonuna dönüşür. Bu dosyada `cevap` YAZILI değildir,
// yine de çağrılabilir, çünkü onu makro üretmiştir.
// Neden ayrı bir dosyada? Prosedürel makro, tanımlandığı crate'in içinde kullanılamaz; tests/
// altındaki her dosya ayrı bir crate gibi derlendiği için burada kullanılabilir.
//   Mini örnek:
//     cevap_uret!();               // derleyici buraya `fn cevap() -> u32 { 42 }` yazar
//     assert_eq!(cevap(), 42);     // ✓ üretilen fonksiyon 42 döndürür

use macros::cevap_uret;

cevap_uret!();

#[test]
fn cevap_42_doner() {
    assert_eq!(cevap(), 42);
}
