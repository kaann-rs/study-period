//! # cargo-and-crates — Bölüm 14 örneği
//!
//! `//!` ile başlayan yorum, İÇİNDE bulunduğu öğeyi (burada: crate'in tamamını) belgeler.
//! `cargo doc --open` bunu HTML'e çevirir. Bu crate iki şeyi gösterir:
//! - **Doküman yorumları + doc-test**: aşağıdaki `///` örnek kodları `cargo test`'te KOŞAR.
//! - **`pub use` ile yeniden dışa aktarma**: iç modül düzenini kullanıcıya dayatmadan düz API.
//!
//! Bir renk karıştırma kütüphanesi:
//! ```
//! use cargo_and_crates::{BirincilRenk, karistir};
//! // Kullanıcı iç modülleri (turler/islem) bilmek zorunda değil — pub use sayesinde kök'ten.
//! let yesil = karistir(BirincilRenk::Sari, BirincilRenk::Mavi);
//! ```

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Not: Bu dosyada `//!` ve `///` ile başlayan satırlar sıradan yorum değildir; kullanım kılavuzuna
// dönüşen "doküman yorumları"dır. Sade anlatım kutuları ise düz `//` ile yazıldı ve kılavuza
// girmez. Önce kutuyu oku, sonra altındaki koda geç.
// Bu dosyadaki benzetmelerin hepsi aynı yerde geçer: kutusuyla satılan bir boya setinde.
//
// Bu bölümün tek cümlelik özeti: Yazdığın kodu başkalarının kullanabileceği bir pakete
// çevirirsin: kullanım kılavuzunu kodun içine yazarsın, kılavuzdaki örnekler de test edilir.
//
// MİNİ SÖZLÜK (bu dosyada geçen kelimeler):
//   cargo                    → Rust'ın derleme, test, doküman ve paket yönetimi aracı.
//   crate                    → derlenen bir paket: bir kütüphane ya da bir program.
//   crates.io, docs.rs       → paketlerin yayımlandığı site / dokümanlarının yayımlandığı site.
//   doküman yorumu           → `///` ya da `//!` ile yazılan, kılavuza dönüşen yorum.
//   `///` ve `//!`           → kendinden SONRAKİ öğeyi / İÇİNDE bulunduğu öğeyi belgeler.
//   Markdown, HTML           → kılavuzu yazdığın yalın biçim / tarayıcıda açılan sayfa biçimi.
//   cargo doc                → doküman yorumlarından web sayfası üreten komut.
//   doc-test                 → kılavuzdaki örnek kodun `cargo test` ile çalıştırılması.
//   modül (mod)              → kodu gruplayan isimli bölme: `turler`, `islem`.
//   pub                      → "dışarıdan görünsün" işareti.
//   pub use (re-export)      → içerideki bir öğeyi daha kısa bir yoldan da dışarı açmak.
//   kök (crate root)         → crate'in en üst seviyesi: `cargo_and_crates::...` diye başlayan yer.
//   API                      → crate'in dışarıya açtığı fonksiyon ve türlerin tamamı.
//   bağımlılık (dependency)  → senin crate'inin kullandığı başka bir crate.
//   Cargo.toml / Cargo.lock  → paketin ayar dosyası / kullanılan sürümlerin kesin listesi.
//   profil (dev, release)    → derleme ayar takımı: hızlı derleme / hızlı çalışan program.
//   opt-level                → derleyicinin kodu ne kadar hızlandırmaya uğraşacağı (0–3).
//   workspace                → birlikte derlenen crate'lerin toplandığı çatı klasör.
//   target/                  → derleme çıktılarının konduğu klasör.

// ---- İç modüller: kodu mantıksal olarak ayır ----
// SADE ANLATIM: Boya setinin içinde iki bölme var: birinde tüpler (renklerin kendisi), ötekinde
// karıştırma kabı (renklerle yapılan iş). Karışık dursalar da boyarsın, ama aradığını bulmak
// zorlaşır. Modüller bu bölmelerdir: `turler` renk türlerini, `islem` onlarla yapılan işi tutar.
// Neden böyle? Kod büyüdükçe tek dosyada her şey birbirine karışır; bölmeler "bu nerede?"
// sorusunun cevabını kolaylaştırır.
//   Mini örnek:
//     cargo_and_crates::turler::BirincilRenk::Sari    // tür, `turler` bölmesinde
//     cargo_and_crates::islem::karistir(..)           // iş, `islem` bölmesinde
pub mod turler {
    /// Birincil renkler (karışımın girdileri).
    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum BirincilRenk {
        Kirmizi,
        Sari,
        Mavi,
    }

    /// İkincil renkler (iki birincilin karışımı).
    #[derive(Debug, PartialEq)]
    pub enum IkincilRenk {
        Turuncu,
        Yesil,
        Mor,
    }
}

pub mod islem {
    use crate::turler::{BirincilRenk, IkincilRenk};

    // SADE ANLATIM: Boya kutusunun arkasında "sarı + mavi = yeşil" yazar. Üretici formülü
    // değiştirip yazıyı düzeltmeyi unutursa kutu yanlış bilgi verir. Rust bunu şöyle önler:
    // kılavuzdaki örnek (aşağıdaki `///` satırlarında, üç ters tırnak arasındaki kod) her
    // `cargo test`te gerçekten çalıştırılır. Buna doc-test denir. Neden böyle? Kod değişip örnek
    // eski kalırsa test düşer; kılavuz ile kod birbirinden ayrı düşemez.
    //   Mini örnek (kılavuzda yazan iki satır ve anlamları):
    //     assert_eq!(karistir(Kirmizi, Sari), Some(Turuncu));   // ✓ iki ayrı renk: karışım var
    //     assert_eq!(karistir(Mavi, Mavi), None);               // ✓ aynı renk: karışım yok
    //     // karistir bir gün Yesil döndürmeye başlarsa ilk satır tutmaz → cargo test düşer ✗
    /// İki birincil rengi karıştırıp ikincil rengi döndürür.
    ///
    /// Aynı renk iki kez verilirse `None` döner (karışım tanımsız).
    ///
    /// Bu örnek bir **doc-test**'tir: `cargo test` onu ayrı bir program olarak derleyip çalıştırır,
    /// yani dokümandaki kod HER ZAMAN güncel kalmak zorundadır (derlenmezse test düşer).
    /// ```
    /// use cargo_and_crates::{BirincilRenk, IkincilRenk, karistir};
    /// assert_eq!(karistir(BirincilRenk::Kirmizi, BirincilRenk::Sari), Some(IkincilRenk::Turuncu));
    /// assert_eq!(karistir(BirincilRenk::Mavi, BirincilRenk::Mavi), None);
    /// ```
    pub fn karistir(a: BirincilRenk, b: BirincilRenk) -> Option<IkincilRenk> {
        use BirincilRenk::*;
        let mut ciftler = [a, b];
        ciftler.sort_by_key(|r| *r as u8); // sırayı normalle (Kırmızı<Sarı<Mavi)
        match ciftler {
            [Kirmizi, Sari] => Some(IkincilRenk::Turuncu),
            [Sari, Mavi] => Some(IkincilRenk::Yesil),
            [Kirmizi, Mavi] => Some(IkincilRenk::Mor),
            _ => None, // aynı renk iki kez
        }
    }
}

// SADE ANLATIM: Seti alan kişi, tüplerin kutunun hangi bölmesinde durduğunu ezberlemek istemez;
// en çok kullandıklarını kapağı açar açmaz en üstte görmek ister. `pub use` bunu yapar: öğe
// yerinde durur, ama en üst seviyeden de erişilir hâle gelir. Neden böyle? Bölmeleri sen kendi
// düzenin için kurdun; kullanıcının uzun yolu yazması gerekmesin, sen de bölmeleri sonradan
// değiştirebilesin.
//   Mini örnek:
//     use cargo_and_crates::turler::BirincilRenk;   // ✓ uzun yol hâlâ çalışır
//     use cargo_and_crates::BirincilRenk;           // ✓ kısa yol: `pub use` sayesinde
//     use cargo_and_crates::Mor;                    // ✗ derlenmez: kökte böyle bir öğe yok
// ---- pub use: YENİDEN DIŞA AKTARMA ----
// İç yapı: turler::BirincilRenk, islem::karistir. Ama kullanıcı derin yolları yazmak zorunda
// kalmasın diye en çok kullanılanları KÖK seviyeye "yeniden dışa aktarıyoruz". Böylece hem
// `cargo_and_crates::turler::BirincilRenk` hem de kısa `cargo_and_crates::BirincilRenk` çalışır.
pub use islem::karistir;
pub use turler::{BirincilRenk, IkincilRenk};

// (Sade: bu crate'te iki çeşit test birlikte çalışır: aşağıdaki sıradan testler ve yukarıda
// kılavuzun içine yazılmış örnekler. `cargo test -p cargo-and-crates` ikisini de çalıştırır.)
// Normal birim testi de doc-test'in yanında koşar.
#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn yesil_uretilir() {
        assert_eq!(
            karistir(BirincilRenk::Sari, BirincilRenk::Mavi),
            Some(IkincilRenk::Yesil)
        );
    }

    #[test]
    fn ayni_renk_none() {
        assert_eq!(karistir(BirincilRenk::Kirmizi, BirincilRenk::Kirmizi), None);
    }
}

// SADE ANLATIM (aşağıdaki bölüm için): Aşağısı kod yazmayı değil, yazdığın kodu paketleyip
// başkalarına ulaştırmayı anlatır: kılavuzdaki örnekler neden test edilir, iki çeşit doküman
// yorumunun farkı nedir, "deneme için hızlı derleme" ile "kullanıcıya gidecek hızlı program"
// ayarları nasıl ayrılır, birçok paket tek çatı altında nasıl toplanır. İlk okuyuşta
// atlayabilirsin; yukarıdaki üç kutuyu anlamış olman yeter.
// ============================================================================
// CARGO / EKOSİSTEM PERSPEKTİFİ — bu bölüm "neden" çok, "nasıl" az
// ============================================================================
//
// • DOC-TEST neden var? Sorun: dokümandaki örnek kod zamanla koddan sapar ("doc rot") ve
//   kullanıcıyı yanıltır. Rust bunu, `///` içindeki ```kod bloklarını``` birer test olarak
//   derleyip çalıştırarak çözer. Örnek derlenmez veya assert düşerse `cargo test` başarısız olur
//   → dokümanla kod senkron kalmaya ZORLANIR. (Örnekte main yazmadık; doc-test onu sarar.)
//
// • `//!` vs `///`: `//!` İÇİNDE bulunduğu öğeyi (crate/modül) belgeler, dosyanın en başına
//   yazılır; `///` KENDİNDEN SONRAKİ öğeyi (fonksiyon, tür) belgeler. İkisi de Markdown'dır ve
//   `cargo doc` ile HTML üretir (crates.io'ya yayımlanınca docs.rs otomatik yayımlar).
//
// • `pub use` (re-export) neden? Sorun: iç modül hiyerarşin (turler/islem) senin DÜZENİN;
//   kullanıcı onu ezberlemek zorunda kalmamalı. pub use ile "iç organizasyon" ile "public API"
//   ayrışır: kodu istediğin gibi modüllere böl, sonra önemli öğeleri kök'e yeniden dışa aktar.
//
// • RELEASE PROFİLLERİ: `cargo build` = dev profili (opt-level 0, hızlı derleme, yavaş kod);
//   `cargo build --release` = release profili (opt-level 3, yavaş derleme, hızlı kod).
//   Ayarlar workspace KÖK Cargo.toml'undaki [profile.dev]/[profile.release] ile değiştirilir
//   (üye crate'te yazılırsa yok sayılır/uyarır — o yüzden buraya koymadık). Bu yüzden
//   ölçümler hep --release ile yapılır; dev ikilisi "gerçek" hızı yansıtmaz.
//
// • WORKSPACE: bu repo zaten bir workspace — kök Cargo.toml `members`/`exclude` ile crate'leri
//   toplar, hepsi TEK bir target/ ve TEK bir Cargo.lock paylaşır (bağımlılık sürümleri tutarlı,
//   ortak bağımlılık bir kez derlenir). `cargo install` ise bir ikiliyi ~/.cargo/bin'e kurar.
