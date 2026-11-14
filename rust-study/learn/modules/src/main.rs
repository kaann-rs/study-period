// Rust Book — Bölüm 7: Packages, Crates, Modules (paket, crate, modül), paths, use
// Çalıştır: cargo run -p modules
//
// Sözlük:
//   package = Cargo.toml ile paketlenen bir veya daha çok crate (bu klasör bir package).
//   crate   = tek bir derleme birimi; ya binary (main.rs) ya library (lib.rs). Bu dosya
//             binary crate'in KÖKÜ'dür (crate root). Modül ağacı buradan başlar.
//   module  = isimleri gruplayan, görünürlüğü (privacy) belirleyen birim (`mod`).
//
// Doğrulanmış (rustc ile): private öğe → E0603, private alanla kurma → E0451, private
// alana erişim → E0616. Ayrıntı: dosyanın sonundaki DERLEYİCİ PERSPEKTİFİ.

mod lokanta {
    // pub OLMAYAN bir modül/öğe dışarıdan görünmez — VARSAYILAN private.
    pub mod salon {
        pub fn masa_ayarla() {
            println!("salon: masa ayarlandı");
            // super:: → bir ÜST modül (burada lokanta). salon, lokanta'nın torunu olduğu için
            // lokanta'nın private alt modülü mutfak'a buradan erişebilir.
            super::mutfak::hazirla();
        }
    }

    mod mutfak {
        // Modülün kendisi private (lokanta dışından görünmez) ama içindeki fn pub olduğundan
        // lokanta ve torunları bunu çağırabilir.
        pub fn hazirla() {
            println!("mutfak: hazır");
        }
    }

    pub fn ac() {
        // self:: → BU modül (lokanta). Göreli (relative) path ile çocuklara eriş.
        self::salon::masa_ayarla();
        mutfak::hazirla(); // kardeş private modül, lokanta içinden erişilir
    }

    // pub(crate): yalnız BU crate içinde görünür; crate dışına (başka paket) açılmaz.
    pub(crate) fn ic_rapor() {
        println!("lokanta: iç rapor (yalnız crate içi)");
    }
}

mod siparis {
    #[derive(Debug)]
    pub struct Fis {
        pub urun: String, // pub alan: dışarıdan okunur/yazılır
        tutar: u32,       // private alan: yalnız bu modül içinden erişilir
    }

    impl Fis {
        pub fn yeni(urun: &str, tutar: u32) -> Fis {
            // private alanı YALNIZ modül içinde set edebiliriz; bu yüzden "yeni" bir yapıcı sunulur.
            Fis {
                urun: urun.to_string(),
                tutar,
            }
        }
        // Dışarıya kontrollü erişim (getter): private alanı okutur ama yazdırmaz.
        pub fn tutar(&self) -> u32 {
            self.tutar
        }
    }

    pub fn demo() {
        let f = Fis::yeni("çay", 15);
        println!("fiş: {} = {}", f.urun, f.tutar());
        // Dışarıdan (başka modülden) şunlar OLMAZ:
        //   let f = siparis::Fis { urun: ..., tutar: 15 }; // ✗ E0451: `tutar` private, dışarıdan kuramazsın
        //   let _ = f.tutar;                                // ✗ E0616: `tutar` private, getter kullan (f.tutar())
    }
}

// Re-export (yeniden ihraç): derinlerdeki bir öğeyi sade bir dış API olarak sun.
mod derin {
    pub mod cok {
        pub mod ic {
            pub fn eylem() {
                println!("derin::cok::ic::eylem çalıştı");
            }
        }
    }
}
mod cephe {
    // pub use: içerideki uzun yolu, bu modülün ÜZERİNDEN kısa ve pub olarak dışarı açar.
    pub use crate::derin::cok::ic::eylem;
    pub fn demo() {
        eylem(); // use sayesinde kısa isim
    }
}

// `use`: bir path'i kapsama getirir. Bu bir KOPYA/taşıma değil, sadece yerel bir KISA İSİMdir.
use lokanta::salon;
use lokanta::salon::masa_ayarla as hazirla_masa; // `as` ile yeniden adlandırma

fn main() {
    // Absolute path: crate kökünden tam yol.
    crate::lokanta::ac();

    // use ile getirilen kısa isimler:
    salon::masa_ayarla();
    hazirla_masa();

    lokanta::ic_rapor();
    siparis::demo();
    cephe::demo();
    cephe::eylem(); // re-export sayesinde cephe üzerinden de erişilebilir
}

// ============================================================================
// DERLEYİCİ PERSPEKTİFİ — modüller ve görünürlük
// ============================================================================
//
// • Modüller ÇALIŞMA ANI maliyeti taşımaz; yalnız derleme zamanında isimleri düzenler ve
//   API yüzeyini (neyin dışarı açık olduğunu) tanımlar.
//
// • İsim çözümleme (name resolution): path'ler, `use`'lar ve isimler, TÜR DENETİMİNDEN
//   ÖNCE çözülür — hangi ismin hangi tanıma işaret ettiği erken belirlenir. Kaynak crate:
//   rustc_resolve. (İlginç ayrıntı: Rust'ta iki ayrı "namespace" vardır — türler ve
//   değerler. Bu yüzden unit struct `struct S;` hem bir TÜR `S` hem bir DEĞER `S` tanımlar;
//   çakışmazlar.)
//
// • Görünürlük (visibility/privacy) denetimi ayrı bir geçişte zorlanır (crate: rustc_privacy).
//   Doğrulanmış hatalar: private öğeye erişim E0603; private alanla dışarıdan struct kurma
//   E0451; private alana erişim E0616.
//
// • Neden VARSAYILAN private? Sorun: her şey otomatik public olsaydı, bir modülün tüm iç
//   detayları onun DIŞ API'si sayılırdı; birini değiştirmek uzaktaki kodları kırabilirdi ve
//   neyin "güvenle değiştirilebilir iç detay" olduğunu bilemezdin. Çözüm: encapsulation —
//   dışa açtığını `pub` ile BİLİNÇLİ seçersin; private kalanları özgürce değiştirebilirsin,
//   derleyici sınırı zorlar. pub(crate)/pub(super) ile açıklığı daha da inceli ayarlarsın.
//
// • (Dosyalara bölme) `mod salon;` (gövdesiz) yazarsan derleyici salon'u ayrı bir dosyadan
//   (src/salon.rs ya da src/salon/mod.rs) yükler — modül ağacı aynı kalır, yalnız kaynak
//   birden çok dosyaya dağılır.
