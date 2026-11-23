// Açıklama: learn/writing-tests/ACIKLAMA.md
// Rust Book — Bölüm 11: Writing Automated Tests
// Test et: cargo test -p writing-tests
// Çalıştır (demo): cargo run -p writing-tests
//
// Test yazmak Rust'ta DİLE ve cargo'ya gömülüdür — ayrı bir framework kurmazsın. Bir fonksiyona
// #[test] koyarsın, `cargo test` onları ayrı bir ikiliye derleyip koşturur.
//
// Bu crate hem kütüphane (lib.rs: fonksiyonlar + birim testleri) hem de ince bir ikili (main.rs).
// Entegrasyon testleri ayrıca tests/ dizininde. Doğrulama: DERLEYİCİ/HARNESS PERSPEKTİFİ (altta).

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Önemli yerlerin üstünde "SADE ANLATIM" kutusu var: önce onu oku; teknik kısım zor gelse de olur.
// Benzetmeler bir oyuncak atölyesinde geçer.
//
// Bu bölümün tek cümlelik özeti: Test, "bu fonksiyona şunu verirsem şunu almalıyım" diyen küçük
// bir denemedir; bir kez yazarsın, bilgisayar her seferinde senin yerine dener.
//
// MİNİ SÖZLÜK (bu dosyada geçen kelimeler):
//   test                     → kodun beklendiği gibi çalıştığını sınayan küçük fonksiyon.
//   öznitelik (#[...])       → bir öğenin üstüne yapıştırılan etiket: derleyiciye not düşer.
//   #[test]                  → "bu fonksiyon bir testtir" etiketi.
//   #[cfg(test)]             → "bunu yalnız test derlenirken derle" etiketi.
//   assert! / assert_eq!     → "bu doğru olmalı" / "bu ikisi eşit olmalı". Değilse test düşer.
//   assert_ne!               → "bu ikisi eşit OLMAMALI".
//   düşmek / geçmek          → testin başarısız / başarılı olması.
//   #[should_panic]          → "bu test geçmek için paniklemeli" etiketi.
//   #[ignore]                → "bu testi şimdilik atla" etiketi.
//   birim testi (unit)       → tek bir küçük parçayı, kodun içinden sınayan test.
//   entegrasyon testi        → parçaları birlikte, dışarıdan bir kullanıcı gibi sınayan test.
//   pub / private            → dışarıdan görünen / yalnız kendi modülünden görünen öğe.
//   public API               → crate'in dışarıya açtığı fonksiyon ve türlerin tamamı.
//   crate                    → Rust'ta derlenen bir paket: bir kütüphane ya da bir program.
//   kütüphane (lib) / ikili  → başkasının çağırdığı kod / doğrudan çalıştırılan program.
//   modül (mod)              → kodu gruplayan isimli bölme. `super` = bir üst bölme.
//   framework                → bir iş için ayrıca kurulan hazır araç takımı.
//   harness, libtest         → testleri bulup çalıştıran ve sonucu sayan koşucu program.
//   paralel                  → aynı anda, yan yana çalışma.
//   stdout                   → programın ekrana yazdığı olağan çıktı.
//   dev-dependencies         → yalnız testlerde kullanılan yardımcı paketler.
//   cargo                    → Rust'ın derleme, çalıştırma ve test komutlarını veren aracı.

// ---- Test edilecek gerçek kod ----

// (Sade: aşağıdaki dört fonksiyon atölyenin oyuncakları; asıl konu onları nasıl denediğimizdir.)
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

// (Sade: "private", atölyenin içindeki bir alet gibidir: müşteri göremez, ama atölyenin içinde
// çalışan kalite kontrolcü (birim testi) onu eline alıp deneyebilir.)
// PRIVATE fonksiyon: dışarıdan erişilemez ama birim testleri (aynı modül ağacında, alt modülden
// `use super::*` ile) erişebilir; aynı crate'deki her modül erişemez.
fn kirp(s: &str) -> &str {
    s.trim()
}

// ============================================================================
// BİRİM TESTLERİ — test edilen kodla AYNI dosyada, #[cfg(test)] modül içinde
// ============================================================================
// #[cfg(test)]: bu modül YALNIZCA `cargo test`'te derlenir; `cargo build`/`run`'da hiç yoktur
// (ikiliye test kodu sızmaz). `use super::*;` ile üstteki (private dahil) öğeleri içeri alır.
// SADE ANLATIM: Oyuncak atölyesinde her araba kutuya girmeden önce denenir. Deneme tezgâhı
// atölyenin içindedir, müşteriye giden kutuya konmaz; Rust'ta testler bu tezgâhtır, asıl kodla
// aynı dosyada durur. Elle denemek unutulur; yazılı test her değişiklikte eskiden çalışan şeyi
// bozup bozmadığını saniyeler içinde söyler.
//   Mini örnek:
//     fn ikiye_katla(x: i32) -> i32 { x * 2 }
//     #[test]
//     fn dort_sekiz_olur() {
//         assert_eq!(ikiye_katla(4), 8);    // ✓ test geçer
//         assert_eq!(ikiye_katla(4), 9);    // ✗ test düşer: "left: 8, right: 9" yazar
//     }
#[cfg(test)]
mod testler {
    use super::*;

    // SADE ANLATIM: Kalite kontrolcünün bir beklentiler listesi vardır; biri tutmazsa oyuncak geri
    // döner. `assert!` ailesi bu satırlardır: `assert!(koşul)` "doğru olmalı", `assert_eq!(a, b)`
    // "eşit olmalı", `assert_ne!(a, b)` "eşit OLMAMALI" der. `assert_eq!` tutmayınca iki değeri de
    // yazar; neyin yanlış çıktığını aramazsın.
    //   Mini örnek:
    //     assert!(topla(2, 3) > 0);         // ✓ 5 > 0 doğru
    //     assert_eq!(topla(2, 3), 5);       // ✓ eşit
    //     assert_ne!(topla(2, 3), 6);       // ✓ eşit değil, beklenen de bu
    //     assert_eq!(topla(2, 3), 6);       // ✗ test düşer; ekranda 5 ve 6 yan yana görünür
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

    // SADE ANLATIM: Bazı denemelerde beklenen şey "bozulması"dır. Oyuncağın pil yuvasına pili
    // ters takarsın; çalışmaması GEREKİR. Çalışırsa bir sorun vardır. `#[should_panic]` böyle bir
    // testtir: içindeki kod paniklerse test geçer, paniklemezse düşer. `expected = "..."` ise
    // "herhangi bir nedenle değil, şu mesajla paniklemeli" diye sıkılaştırır.
    //   Mini örnek:
    //     #[test]
    //     #[should_panic(expected = "negatif olamaz")]
    //     fn eksi_yas() { yas_dogrula(-1); }    // ✓ panikledi, mesaj da uyuyor: test geçer
    //     #[test]
    //     #[should_panic]
    //     fn arti_yas() { yas_dogrula(30); }    // ✗ paniklemedi: test düşer
    #[test]
    #[should_panic(expected = "negatif olamaz")] // panik ETMELİ; mesaj bu alt dizgiyi içermeli
    fn negatif_yas_panikler() {
        yas_dogrula(-1);
    }

    // (Sade: test `Result<(), E>` de döndürebilir: `Ok(())` geçti, `Err(..)` düştü; faydası, testin
    // içinde `?` kullanabilmek.)
    #[test]
    fn result_donen_test() -> Result<(), String> {
        if topla(2, 2) == 4 {
            Ok(())
        } else {
            Err(String::from("2+2 != 4 ?!"))
        }
    }

    // (Sade: `#[ignore]`, uzun süren bir denemeyi "bugün yapma" diye kenara ayırmaktır. Test
    // silinmez; `cargo test -- --ignored` ya da `--include-ignored` dediğinde yine çalışır.)
    #[test]
    #[ignore] // normalde atlanır; `--ignored` veya `--include-ignored` ile koşar (yavaş/isteğe bağlı)
    fn pahali_test() {
        assert_eq!(topla(1000, 1000), 2000);
    }
}

// SADE ANLATIM (aşağıdaki bölüm için): `cargo test` yazınca bir "deneme sorumlusu" #[test]
// fonksiyonları bulup koşturur, kaçının geçtiğini sayar; paralellik, düşme koşulu ve birim/
// entegrasyon farkını aşağıda okursun. İlk okuyuşta atlayabilirsin; yukarıdaki testler yeter.
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
//   `cargo test -- --ignored` → yalnız #[ignore]'luları (`--include-ignored` tümünü koşar).
//
// • BİRİM vs ENTEGRASYON:
//     - Birim testleri: src içinde #[cfg(test)] modül; private'a erişir, iç mantığı sınar.
//     - Entegrasyon testleri: tests/ dizininde AYRI crate'ler; crate'i bir kullanıcı gibi
//       yalnız PUBLIC API'dan `use writing_tests::...` ile çağırır (private görünmez). Her
//       tests/ dosyası ayrı bir ikili olarak derlenir. Bu ayrım, "dışarıdan görünen davranış"ı
//       iç ayrıntıdan bağımsız test etmeni sağlar. (Bu yüzden entegrasyon testi için crate'in
//       bir lib hedefi olmalı; salt-ikili crate'in tests/'i public API göremez.)
