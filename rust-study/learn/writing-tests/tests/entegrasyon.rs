// ENTEGRASYON TESTİ — tests/ dizininde, AYRI bir crate olarak derlenir.
// Buradan crate'e, dışarıdaki bir kullanıcı gibi SADECE public API'dan erişebiliriz:
// `kirp` (private) buradan görünmez; sadece pub olanlar.
use writing_tests::{kelime_say, topla};

// Bu dosyada #[cfg(test)]/mod GEREKMEZ: tests/ dosyaları zaten yalnız test derlemesinde
// derlenir ve her biri kendi test ikilisidir.
#[test]
fn dışarıdan_topla() {
    assert_eq!(topla(40, 2), 42);
}

#[test]
fn dışarıdan_kelime_say() {
    assert_eq!(kelime_say("merhaba dünya"), 2);
}
