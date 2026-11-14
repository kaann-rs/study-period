// Rust Book — Bölüm 12: An I/O Project — `minigrep` (küçük bir grep)
// Çalıştır:  cargo run -p minigrep -- hiç siir.txt
//            IGNORE_CASE=1 cargo run -p minigrep -- NE siir.txt
// Test et:   cargo test -p minigrep
//
// Bu bölüm YENİ kavram öğretmez; önceki bölümleri (Result/?, lifetime, trait, closure/iterator,
// test) GERÇEK bir komut satırı aracında birleştirir. Ders: mantığı lib.rs'e koy (test
// edilebilir), main.rs'i ince tut (argüman + hata gösterimi + çıkış kodu).
//
// Doğrulama ve "neden böyle": DERLEYİCİ/TASARIM PERSPEKTİFİ (en altta).

use std::env;
use std::error::Error;
use std::fs;

// Ayarları tek yerde topla: ayrıştırma mantığı main'den çıkıp test edilebilir hale gelir.
pub struct Config {
    pub sorgu: String,
    pub dosya_yolu: String,
    pub harf_duyarsiz: bool, // büyük/küçük harf ayrımı yok mu
}

impl Config {
    // Argüman ITERATOR'ı alır (Bölüm 13 kalıbı): sahipliği devralır, klonlamaya gerek kalmaz.
    // Hata olursa panic ETMEZ; Result döner → main düzgün bir mesajla çıkabilir.
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

// Asıl iş. Box<dyn Error> = "Error uygulayan HERHANGİ bir hata" (trait object, Bölüm 10).
// Böylece fs okuma hatası da başka hata da tek dönüş türünde ? ile yukarı taşınır.
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

// Lifetime 'a (Bölüm 10): dönen diliml(er) `icerik`ten gelir, o yüzden çıktı icerik kadar yaşar.
// `sorgu`nun süresiyle ilişkisi yok — bu yüzden yalnız icerik'e 'a bağlanır.
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
