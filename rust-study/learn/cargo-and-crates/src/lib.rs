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

// ---- İç modüller: kodu mantıksal olarak ayır ----
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

// ---- pub use: YENİDEN DIŞA AKTARMA ----
// İç yapı: turler::BirincilRenk, islem::karistir. Ama kullanıcı derin yolları yazmak zorunda
// kalmasın diye en çok kullanılanları KÖK seviyeye "yeniden dışa aktarıyoruz". Böylece hem
// `cargo_and_crates::turler::BirincilRenk` hem de kısa `cargo_and_crates::BirincilRenk` çalışır.
pub use islem::karistir;
pub use turler::{BirincilRenk, IkincilRenk};

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
