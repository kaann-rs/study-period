// Makroyu DIŞARIDAN kullanan entegrasyon testi. tests/ altındaki her dosya ayrı
// bir crate gibi derlenir ve `macros` crate'ine bağımlıdır; makro burada gerçekten
// genişler. Çalıştırmak için: cargo test -p macros

use macros::cevap_uret;

// Makro burada `fn cevap() -> u32 { 42 }` fonksiyonunu üretir.
cevap_uret!();

#[test]
fn cevap_42_doner() {
    assert_eq!(cevap(), 42);
}
