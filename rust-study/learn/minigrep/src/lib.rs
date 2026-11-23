// Açıklama: learn/minigrep/ACIKLAMA.md
// Rust Book — Bölüm 12: An I/O Project — `minigrep` (küçük bir grep)
// Çalıştır (rust-study kökünden):  cargo run -p minigrep -- hiç learn/minigrep/siir.txt
//                                  IGNORE_CASE=1 cargo run -p minigrep -- NE learn/minigrep/siir.txt
//   (`siir.txt` yalnız learn/minigrep dizininden çalıştırılırsa çıplak yazılabilir.)
// Test et:   cargo test -p minigrep
//
// Bu bölüm YENİ kavram öğretmez; önceki bölümleri (Result/?, lifetime, trait, closure/iterator,
// test) GERÇEK bir komut satırı aracında birleştirir.
//
// Doğrulama ve "neden böyle": DERLEYİCİ/TASARIM PERSPEKTİFİ (en altta).

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Her önemli parçanın üstünde "SADE ANLATIM" kutusu var: önce onu oku; teknik kısım zor gelse de
// olur. Benzetmeler bir kütüphanenin danışma masasında geçer.
//
// Bu bölümün tek cümlelik özeti: Programa bir kelime ve bir dosya adı verirsin; o da dosyada o
// kelimenin geçtiği satırları bulup ekrana yazar.
//
// MİNİ SÖZLÜK (bu dosyada geçen kelimeler):
//   grep                    → metin içinde kelime arayan ünlü komut; minigrep onun küçüğüdür.
//   komut satırı (CLI)      → programı fareyle değil, yazı yazarak çalıştırdığın pencere.
//   argüman                 → komutta program adından sonra yazdıkların: `hiç siir.txt`.
//   ortam değişkeni         → programa dışarıdan verilen adlı ayar: `IGNORE_CASE=1`.
//   I/O (girdi/çıktı)       → programın dış dünyayla alışverişi: dosya okuma, ekrana yazma.
//   Config                  → ayarların toplandığı struct (bu dosyada biz tanımlıyoruz).
//   iterator, Item          → elemanları sırayla veren şey / verdiği her elemanın türü.
//   closure                 → `|x| ...` biçiminde yazılan adsız, küçük fonksiyon.
//   lines / filter / collect → satırlara böl / şarta uyanları seç / seçilenleri bir kapta topla.
//   lifetime ('a)           → bir referansın geçerli kaldığı bölge (Böl. 10).
//   Box<dyn Error>          → "herhangi bir hata" koyulabilen kutu (Böl. 9 ve 10).
//   stdout / stderr         → programın iki ayrı çıkış kanalı: sonuçlar için / hatalar için.
//   yönlendirme (>), boru   → çıktıyı ekran yerine dosyaya / başka bir programa göndermek.
//   çıkış kodu              → programın biterken bıraktığı sayı: 0 = oldu, başka = olmadı.
//   kabuk (shell), betik    → komutları yazdığın program / art arda komut çalıştıran dosya.
//   TDD                     → önce testi yazıp sonra onu geçirecek kodu yazma alışkanlığı.
//   tahsis (allocation)     → heap'ten yer istemek; gereksizse kaçınılır.

use std::env;
use std::error::Error;
use std::fs;

// SADE ANLATIM: Kütüphanenin danışma masasında bir istek fişi doldurursun: "hangi kelimeyi
// arıyorum, hangi kitapta, büyük-küçük harf fark eder mi?" Görevli fişe bakarak çalışır; sana
// her adımda yeniden sormaz. `Config` bu fiştir: üç bilgi tek bir struct'ta toplanır. Neden
// böyle? Bilgiler dağınık dursaydı her fonksiyona üçünü ayrı ayrı taşırdın; biri eksik ya da
// yanlış sırada gidebilirdi.
//   Mini örnek:
//     let fis = Config {
//         sorgu: String::from("hiç"),             // aranan kelime
//         dosya_yolu: String::from("siir.txt"),   // hangi dosyada
//         harf_duyarsiz: false,                   // "Hiç" ile "hiç" ayrı sayılsın
//     };
pub struct Config {
    pub sorgu: String,
    pub dosya_yolu: String,
    pub harf_duyarsiz: bool, // büyük/küçük harf ayrımı yok mu
}

impl Config {
    // SADE ANLATIM: Fişi eksik doldurursan görevli sana bağırıp masayı kapatmaz; fişi geri uzatır
    // ve "kitap adını yazmamışsın" der. `build` de böyledir: argüman eksikse programı çökertmez
    // (paniklemez), nedenini söyleyen bir `Err` döndürür. Ne yapılacağına çağıran karar verir.
    // Neden böyle? Eksik argüman kullanıcının yanlışıdır, programın değil; kullanıcıya anlaşılır
    // bir cümle gerekir, çökme raporu değil.
    //   Mini örnek (çağrı: `minigrep hiç siir.txt` → argümanlar: "minigrep", "hiç", "siir.txt"):
    //     args.next();                    // "minigrep": programın kendi adı, atlanır
    //     args.next()                     // Some("hiç")      → sorgu
    //     args.next()                     // Some("siir.txt") → dosya yolu
    //     // `minigrep hiç` yazılsaydı üçüncü next() None verirdi → Err("dosya yolu verilmedi")
    // Argüman ITERATOR'ı (Bölüm 13): sahiplik devri, klon yok; hata olursa panic etmez, Result döner.
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next(); // ilk argüman programın kendi adı; atla

        let sorgu = match args.next() {
            Some(s) => s,
            None => return Err("sorgu verilmedi"),
        };
        let dosya_yolu = match args.next() {
            Some(d) => d,
            None => return Err("dosya yolu verilmedi"),
        };

        // Ortam değişkeni (environment variable): varsa harf duyarsız ara. is_ok() = tanımlı mı.
        let harf_duyarsiz = env::var("IGNORE_CASE").is_ok();

        Ok(Config {
            sorgu,
            dosya_yolu,
            harf_duyarsiz,
        })
    }
}

// SADE ANLATIM: Görevli fişi aldı. Önce kitabı raftan getirmeye gider; kitap yoksa geri gelip
// "bulamadım" der ve iş orada biter. Kitap varsa satırları tek tek tarar, kelimenin geçtiği
// satırları sana okur. `run` bu iki adımdır. Neden `Result` dönüyor? Kitabın rafta olup
// olmadığını önceden bilemezsin; "olmadı" haberini çağırana iletmenin yolu budur.
//   Mini örnek:
//     let icerik = fs::read_to_string("siir.txt")?;   // dosya yoksa: Err ile hemen dön
//     for satir in ara("hiç", &icerik) {              // dosya varsa: uyan satırları gez
//         println!("{satir}");                        // her birini ekrana yaz
//     }
// Asıl iş. Box<dyn Error> = herhangi bir Error (trait object, Bölüm 10); farklı hatalar ? ile taşınır.
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let icerik = fs::read_to_string(&config.dosya_yolu)?; // dosya yoksa Err → ? ile geri döner

    let satirlar = if config.harf_duyarsiz {
        ara_harf_duyarsiz(&config.sorgu, &icerik)
    } else {
        ara(&config.sorgu, &icerik)
    };

    for satir in satirlar {
        println!("{satir}"); // SONUÇLAR stdout'a
    }
    Ok(())
}

// SADE ANLATIM: Görevli bulduğu satırları deftere yeniden yazmaz; kitabın o sayfalarına ayraç
// koyup sana gösterir. Ayraçlar kitaba aittir: kitap rafa dönünce işe yaramazlar. Aradığın
// kelimeyi yazdığın kâğıt ise iş bitince atılabilir; ayraçların onunla bir bağı yoktur.
// İmzadaki `'a` bunu söyler: "dönen satırlar `icerik`e bağlı, `sorgu`ya değil." Arama da üç
// adımdır: satırlara böl, kelimeyi içerenleri seç, seçilenleri topla. Neden `'a` gerekli? İki
// referans girdi olduğu için derleyici, sonucun hangisine bağlı olduğunu kendisi bilemez.
//   Mini örnek:
//     let icerik = "Ben bir hiçim\nSen kimsin\nHiç misin";
//     ara("hiç", icerik)                  // ["Ben bir hiçim"] — "Hiç" büyük harfli, uymaz
//     ara_harf_duyarsiz("hiç", icerik)    // ["Ben bir hiçim", "Hiç misin"]
//     ara("yok", icerik)                  // [] — hiçbir satır uymadı; bu bir hata değildir
pub fn ara<'a>(sorgu: &str, icerik: &'a str) -> Vec<&'a str> {
    icerik
        .lines()
        .filter(|satir| satir.contains(sorgu))
        .collect()
}

pub fn ara_harf_duyarsiz<'a>(sorgu: &str, icerik: &'a str) -> Vec<&'a str> {
    let sorgu = sorgu.to_lowercase(); // yeni String; aşağıda &sorgu ile kıyasla
    icerik
        .lines()
        .filter(|satir| satir.to_lowercase().contains(&sorgu))
        .collect()
}

// SADE ANLATIM: Görevliye küçük bir deneme yaptırılır: "şu üç satırlık kâğıtta 'hiç' kelimesini bul."
// Doğru cevap bellidir; aynı cevabı verirse deneme geçer. TDD sırayı ters çevirir: önce testi
// yazarsın (henüz düşer), sonra onu geçirecek kodu. Beklentiyi yazınca "kod ne yapmalı" netleşir.
//   Mini örnek:
//     cargo test -p minigrep            // üç testi çalıştırır
//     cargo test -p minigrep duyarsiz   // yalnız adında "duyarsiz" geçen testi çalıştırır
// ---- Birim testleri (önce test, sonra kod: TDD kalıbı) ----
#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn harf_duyarli_tek_sonuc() {
        let sorgu = "hiç";
        let icerik = "Ben bir hiçim\nSen kimsin\nHiç misin";
        // "hiç" küçük harfle yalnız ilk satırda geçer ("Hiç" eşleşmez).
        assert_eq!(vec!["Ben bir hiçim"], ara(sorgu, icerik));
    }

    #[test]
    fn harf_duyarsiz_iki_sonuc() {
        let sorgu = "hiç";
        let icerik = "Ben bir hiçim\nSen kimsin\nHiç misin";
        assert_eq!(
            vec!["Ben bir hiçim", "Hiç misin"],
            ara_harf_duyarsiz(sorgu, icerik)
        );
    }

    #[test]
    fn eksik_arguman_hata_doner() {
        // Program adı + sorgu var, dosya yok → Err
        let args = vec![String::from("prog"), String::from("sorgu")].into_iter();
        assert!(Config::build(args).is_err());
    }
}

// SADE ANLATIM (aşağıdaki bölüm için): "Neden böyle düzenlendi?"nin cevabı: iş lib.rs'te, sonuç
// ve hata iki ayrı kanaldan, çıkışta bir durum kodu. İlk okuyuşta atlayabilirsin.
// ============================================================================
// DERLEYİCİ / TASARIM PERSPEKTİFİ — küçük bir CLI'ın doğru iskeleti
// ============================================================================
//
// • env::args() bir ITERATOR döndürür. Config::build'e doğrudan bu iterator'ı verip sahipliği
//   devralmak, önce Vec'e toplayıp sonra klonlamaktan iyidir: her String'i bir kez taşırsın.
//   (args(), geçersiz Unicode argümanda panik eder; env::args_os() OsString verir — burada
//   gerek yok.) Neden iterator? Sorun: &[String] alıp .clone()'lamak gereksiz tahsis/kopyaydı.
//
// • Box<dyn Error> (trait object): run birden çok farklı hata türü üretebilir (IO hatası vb.).
//   Hepsini tek somut enum'da toplamak yerine "Error uygulayan herhangi bir şey"i heap'te bir
//   kutuda döndürürüz; ? operatörü From ile bunlara otomatik çevirir (Bölüm 9 + Bölüm 10 dyn).
//   Bedeli bir heap tahsisi + dinamik dispatch; bir uygulama (binary) için makul. Bir kütüphane
//   genelde kendi hata enum'ını verir (çağırana tür bilgisini korur).
//
// • stdout vs stderr: ARAMA SONUÇLARI println! ile stdout'a; HATA ve tanı mesajları eprintln!
//   ile stderr'e gider (main'de). Neden? Sorun: ikisi de stdout'a gitseydi, `minigrep ... >
//   cikti.txt` ile sonucu dosyaya yönlendirince hata mesajları da dosyaya karışırdı. Ayırınca
//   sonucu borulayıp/yönlendirip hataları yine ekranda görürsün — Unix sözleşmesi.
//
// • Çıkış kodu: hata olunca process::exit(1) ile SIFIR OLMAYAN kodla çıkılır (main.rs). Kabuk
//   ve betikler "başarılı mı?" sorusunu bu kodla yanıtlar; panik/stack-unwind yerine temiz çıkış.
//
// • Katmanlama: parse (Config::build) ve iş (run) lib.rs'te → #[test] ile doğrudan sınanır;
//   main.rs yalnız "argümanları al, hatayı göster, çıkış kodu ver". Bu ayrım bu bölümün ASIL
//   dersidir: test edilemeyen main mantığını olabildiğince küçült.
