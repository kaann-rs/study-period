// Rust Book — Bölüm 11: Writing Automated Tests
// Test et: cargo test -p writing-tests
// Çalıştır (demo): cargo run -p writing-tests
//
// Test yazmak Rust'ta DİLE ve cargo'ya gömülüdür — ayrı bir framework kurmazsın. Bir fonksiyona
// #[test] koyarsın, `cargo test` onları ayrı bir ikiliye derleyip koşturur.
//
// Bu crate hem kütüphane (lib.rs: fonksiyonlar + birim testleri) hem de ince bir ikili (main.rs).
// Entegrasyon testleri ayrıca tests/ dizininde. Doğrulama: DERLEYİCİ/HARNESS PERSPEKTİFİ (altta).

// ---- Test edilecek gerçek kod ----

pub fn topla(a: i32, b: i32) -> i32 {
    a + b
}

/// Yaşı doğrular; geçersizse PANİK eder (should_panic testi için).
pub fn yas_dogrula(yas: i32) -> i32 {
    if yas < 0 {
        panic!("yaş negatif olamaz: {yas}");
    }
    yas
}

/// Metindeki kelime sayısı. Private yardımcıyı kullanır.
pub fn kelime_say(s: &str) -> usize {
    kirp(s).split_whitespace().count()
}

// PRIVATE fonksiyon: dışarıdan erişilemez ama birim testleri (aynı crate içinde) test EDEBİLİR.
fn kirp(s: &str) -> &str {
    s.trim()
}

// ============================================================================
// BİRİM TESTLERİ — test edilen kodla AYNI dosyada, #[cfg(test)] modül içinde
// ============================================================================
// #[cfg(test)]: bu modül YALNIZCA `cargo test`'te derlenir; `cargo build`/`run`'da hiç yoktur
// (ikiliye test kodu sızmaz). `use super::*;` ile üstteki (private dahil) öğeleri içeri alır.
#[cfg(test)]
mod testler {
    use super::*;

    #[test] // bu özniteliği gören fonksiyon bir testtir
    fn topla_calisir() {
        assert_eq!(topla(2, 3), 5); // eşit değilse sol/sağ değerleri basıp testi düşürür
        assert_ne!(topla(2, 2), 5); // eşit OLMAMALI
    }

    #[test]
    fn mantik_dogru() {
        let sonuc = topla(10, -4);
        // assert!: bool bekler. İkinci argümandan sonrası format! gibi ÖZEL HATA MESAJIdır.
        assert!(sonuc > 0, "beklenen pozitif, gelen {sonuc}");
    }

    #[test]
    fn private_fonksiyon_test_edilebilir() {
        // Rust'ta birim testleri private öğelere erişebilir (çocuk modül → üst öğeler görünür).
        assert_eq!(kirp("  selam  "), "selam");
    }

    #[test]
    fn kelime_sayimi() {
        assert_eq!(kelime_say("  bir  iki üç "), 3);
    }

    #[test]
    #[should_panic(expected = "negatif olamaz")] // panik ETMELİ; mesaj bu alt dizgiyi içermeli
    fn negatif_yas_panikler() {
        yas_dogrula(-1);
    }

    // Test Result<(), E> da döndürebilir: Err dönerse test düşer. ? operatörünü teste taşır.
    #[test]
    fn result_donen_test() -> Result<(), String> {
        if topla(2, 2) == 4 {
            Ok(())
        } else {
            Err(String::from("2+2 != 4 ?!"))
        }
    }

    #[test]
    #[ignore] // normalde atlanır; sadece `cargo test -- --ignored` ile koşar (yavaş/isteğe bağlı)
    fn pahali_test() {
        assert_eq!(topla(1000, 1000), 2000);
    }
}

// ============================================================================
// DERLEYİCİ / HARNESS PERSPEKTİFİ — `cargo test` ne yapıyor?
// ============================================================================
//
// • #[test] ve #[cfg(test)]: cargo test, crate'i `--test` yapılandırmasıyla DERLER. Bu sırada
//   `cfg(test)` açıktır, dolayısıyla #[cfg(test)] modüller derlenir; normal build'de kapalıdır,
//   o yüzden test kodu ve test-only bağımlılıkları (dev-dependencies) üretim ikilisine GİRMEZ.
//   Neden? Sorun: testler normal ikiliye gömülseydi boyut ve bağımlılık şişerdi; cfg bunu
//   derleme anında ayırır.
//
// • libtest (test harness): #[test] fonksiyonları, derleyicinin ürettiği bir test KOŞUCUSUyla
//   (libtest) bir araya gelir. Bu koşucu testleri bulur, VARSAYILAN olarak PARALEL (çok iş
//   parçacığı) çalıştırır — bu yüzden testler birbirinden bağımsız olmalı, paylaşılan duruma
//   (dosya, global) dokunan testler çakışabilir. `cargo test -- --test-threads=1` seri yapar.
//   Bir test başarılı olduğunda stdout'u GİZLENİR; görmek için `cargo test -- --show-output`.
//
// • Nasıl "başarısız" olunur? Bir test, panik ederek düşer. assert!/assert_eq! aslında koşul
//   tutmazsa panic! çağırır; #[should_panic] ise bu mantığı TERSİNE çevirir (panik beklenir).
//   Her test kendi mantığında izole panik yakalanır, biri düşünce ötekiler koşmaya devam eder.
//
// • Filtreleme: `cargo test topla` → adında "topla" geçen testleri koşar (substring).
//   `cargo test -- --ignored` → yalnız #[ignore]'luları. Ad çözümü libtest'te, derleyicide değil.
//
// • BİRİM vs ENTEGRASYON:
//     - Birim testleri: src içinde #[cfg(test)] modül; private'a erişir, iç mantığı sınar.
//     - Entegrasyon testleri: tests/ dizininde AYRI crate'ler; crate'i bir kullanıcı gibi
//       yalnız PUBLIC API'dan `use writing_tests::...` ile çağırır (private görünmez). Her
//       tests/ dosyası ayrı bir ikili olarak derlenir. Bu ayrım, "dışarıdan görünen davranış"ı
//       iç ayrıntıdan bağımsız test etmeni sağlar. (Bu yüzden entegrasyon testi için crate'in
//       bir lib hedefi olmalı; salt-ikili crate'in tests/'i public API göremez.)
