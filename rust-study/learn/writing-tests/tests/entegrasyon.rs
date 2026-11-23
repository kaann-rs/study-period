// ENTEGRASYON TESTİ — tests/ dizininde, AYRI bir crate olarak derlenir.
// Buradan crate'e, dışarıdaki bir kullanıcı gibi SADECE public API'dan erişebiliriz:
// `kirp` (private) buradan görünmez; sadece pub olanlar.
// SADE ANLATIM: lib.rs'teki testler atölyenin İÇİNDEN yapılan denemelerdi. Bu dosya ise oyuncağı
// kutusundan çıkarıp deneyen MÜŞTERİDİR: atölyeye giremez, içerideki aletleri (private `kirp`)
// göremez; yalnız kutudan çıkanı (`pub` fonksiyonları) kullanabilir. Neden ayrı bir deneme?
// İçeride her parça tek tek sağlam olabilir, ama müşterinin eline geçen bütün çalışmayabilir.
// Terimler için src/lib.rs'in başındaki mini sözlüğe bak.
//   Mini örnek:
//     use writing_tests::topla;    // ✓ pub: dışarıdan görünür
//     use writing_tests::kirp;     // ✗ derlenmez: kirp private, bu dosyadan görünmez
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
