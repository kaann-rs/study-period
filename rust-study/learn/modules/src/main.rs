// Açıklama: learn/modules/ACIKLAMA.md
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

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Her bölümün başındaki "SADE ANLATIM" kutusunu önce oku, sonra teknik açıklamaya geç.
// Benzetmelerin hepsi koddaki örnekle aynı yerde, bir LOKANTADA geçer.
//
// Bu bölümün tek cümlelik özeti: Kod büyüyünce onu odalara (modül) bölersin; her odanın kapısı
// kapalı gelir ve dışarıya açmak istediklerini `pub` yazarak tek tek seçersin.
//
// MİNİ SÖZLÜK (bu dosyada geçen kelimeler; dosyanın en başındaki "Sözlük" ilk üçünü teknik
// dille verir):
//   package (paket)        → içinde Cargo.toml bulunan proje klasörü.
//   Cargo.toml             → paketin adını ve ayarlarını tutan dosya.
//   crate                  → derleyicinin tek seferde derlediği bütün: program ya da kitaplık.
//   binary / library crate → çalıştırılan program (main.rs) / başka kodların kullandığı kitaplık.
//   crate root (kök)       → derlemenin başladığı dosya. Burada: bu main.rs.
//   modül (mod)            → kodun içindeki oda. İçine fonksiyon, struct, başka odalar konur.
//   modül ağacı            → odaların iç içe dizilişi: kök → lokanta → salon.
//   path (yol)             → bir şeyin adresi; odalar `::` ile ayrılır: lokanta::salon::masa_ayarla
//   absolute path          → kökten başlayan tam adres: `crate::lokanta::ac`.
//   relative path          → bulunduğun odadan başlayan tarif: `salon::masa_ayarla`.
//   crate:: self:: super:: → "kökten başla" / "bu odadan başla" / "bir üst odadan başla".
//   private                → yalnız tanımlandığı odadan (ve iç odalarından) görülür. Varsayılan.
//   pub                    → odanın dışından da görülür.
//   pub(crate)             → bu crate'in her yerinden görülür, crate'in dışından görülmez.
//   görünürlük (privacy)   → bir şeyi kimin görebildiğini belirleyen kuralların adı.
//   use                    → uzun bir adrese kısa ad vermek.
//   as                     → `use` ile getirilen şeye başka bir ad takmak.
//   re-export (pub use)    → başka odadaki bir şeyi kendi odanın kapısından da sunmak.
//   API                    → bir odanın dışarıya sunduğu, başkalarının kullandığı şeylerin tamamı.
//   getter                 → private bir alanı yalnız OKUTAN küçük metot.
//   yapıcı (constructor)   → yeni bir değer üreten fonksiyon; burada `Fis::yeni`.
//   encapsulation          → iç ayrıntıyı saklayıp dışarıya yalnız gerekeni açmak.
//   E0603, E0451, E0616    → görünürlük kuralı çiğnenince derleyicinin verdiği hata numaraları.
//   derleyici (rustc)      → kodu programa çeviren ve hataları önceden bulan araç.
//
// SADE ANLATIM: Bir lokantada salon müşteriye açıktır, mutfağa ise yalnız çalışanlar girer.
// Müşteri mutfağa girip tencereyi karıştıramaz; yemeğini salondan ister, salon mutfağa haber verir.
// Rust'ta `mod` bir odadır. Odadaki her şey kapalı (private) gelir; müşteriye açmak istediğinin
// başına `pub` yazarsın. Aşağıda `salon` açık, `mutfak` kapalıdır.
// Neden böyle? Her şey açık olsaydı başka kodlar mutfağın içine bağlanırdı; mutfakta bir rafın
// yerini değiştirdiğinde o kodların hepsi bozulurdu. Kapalı olanı gönül rahatlığıyla değiştirirsin.
//   Mini örnek:
//     mod lokanta {
//         pub fn kapidan_gir() {}                  // pub: dışarıya açık
//         fn kasayi_say() {}                       // pub yok: yalnız lokantanın içi görür
//         pub fn gun_sonu() { kasayi_say(); }      // ✓ aynı odadan çağrılabilir
//     }
//     lokanta::kapidan_gir();    // ✓
//     lokanta::kasayi_say();     // ✗ derlenmez: kasayi_say dışarıya kapalı
mod lokanta {
    // pub OLMAYAN bir modül/öğe dışarıdan görünmez — VARSAYILAN private.
    pub mod salon {
        pub fn masa_ayarla() {
            println!("salon: masa ayarlandı");
            // (Sade: `super::`, "bir üst odaya çık, oradan devam et" demektir. salon'dan bir üst
            //  oda lokanta'dır; mutfak da lokanta'nın içinde olduğu için oradan ulaşılır.)
            // super:: → bir ÜST modül (burada lokanta). private öğeler tanımlandığı modül ve alt
            // modülleri tarafından görülür; salon da lokanta'nın alt modülü olduğu için erişebilir.
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

    // (Sade: pub(crate), "lokantanın bütün çalışanlarına açık, müşteriye kapalı" gibidir: bu
    //  programın her yerinden çağrılır, bu programı kitaplık olarak kullanan başka kod göremez.)
    pub(crate) fn ic_rapor() {
        println!("lokanta: iç rapor (yalnız crate içi)");
    }
}

// SADE ANLATIM: Kasadan çıkan fişi düşün. Üstündeki ürün adını garson değiştirebilir, ama tutarı
// yalnız kasa yazar; garson tutarı okur, kalemle üstünü çizip değiştiremez. Rust'ta struct'ın
// gözleri de tek tek açılır: `pub urun` herkese açık, `tutar` kapalıdır. Kapalı göz yüzünden fişi
// dışarıdan elle dolduramazsın; kasa bunun için `Fis::yeni` fonksiyonunu ve tutarı okutan
// `tutar()` metodunu sunar.
// Neden böyle? Tutar herkese açık olsaydı, programın herhangi bir yeri onu 0 yapabilirdi.
//   Mini örnek:
//     mod kasa {
//         pub struct Fis { pub urun: String, tutar: u32 }
//         impl Fis {
//             pub fn yeni(urun: &str, tutar: u32) -> Fis { Fis { urun: urun.to_string(), tutar } }
//             pub fn tutar(&self) -> u32 { self.tutar }
//         }
//     }
//     let mut f = kasa::Fis::yeni("çay", 15);
//     f.urun = String::from("kahve");    // ✓ urun herkese açık
//     println!("{}", f.tutar());         // ✓ 15 → okumak serbest
//     f.tutar = 0;                       // ✗ derlenmez: tutar gözü dışarıya kapalı
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
// SADE ANLATIM: Tatlılar deponun en dibindeki rafta durur. Müşteriyi depoya indirmek yerine
// tatlıyı girişteki vitrine koyarsın; müşteri tatlının aslında nerede durduğunu bilmeden vitrinden
// ister. `pub use` budur: derindeki bir şeyi, kendi odanın kapısından kısa bir adla sunar.
// Neden böyle? Yarın tatlıyı başka rafa taşırsan yalnız vitrindeki tek satırı düzeltirsin;
// vitrinden isteyenlerin kodu hiç değişmez.
//   Mini örnek:
//     mod depo { pub mod raf { pub fn tatli() {} } }
//     mod vitrin { pub use crate::depo::raf::tatli; }   // tatlı vitrine kondu
//     vitrin::tatli();        // ✓ kısa yol
//     depo::raf::tatli();     // ✓ uzun yol da çalışır (çünkü depo ve raf da açık)
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

// SADE ANLATIM: Arkadaşının numarasını her aramada tek tek tuşlamazsın; rehbere "Ada" diye
// kaydedersin. `use` de uzun bir adresi kısa bir adla kaydeder: bir kez yazarsın, sonra yalnız
// kısa adı kullanırsın. Numara yerinde durur; `use` hiçbir şeyi kopyalamaz ya da taşımaz.
// Adın çakışacaksa `as` ile başka bir ad takarsın.
//   Mini örnek:
//     use lokanta::salon::masa_ayarla;          // adres rehbere kaydedildi
//     masa_ayarla();                            // ✓ kısa adla çağrılır
//     use lokanta::salon::masa_ayarla as kur;   // aynı fonksiyona ikinci bir ad
//     kur();                                    // ✓ aynı fonksiyon çalışır
//     masa_topla();                             // ✗ derlenmez: bu adda bir şey yok, `use` da yok
// `use`: bir path'i kapsama getirir. Bu bir KOPYA/taşıma değil, sadece yerel bir KISA İSİMdir.
use lokanta::salon;
use lokanta::salon::masa_ayarla as hazirla_masa; // `as` ile yeniden adlandırma

// SADE ANLATIM: Bir yeri iki türlü tarif edersin. Tam adres verirsin: "şehir, sokak, bina".
// Ya da bulunduğun yerden tarif edersin: "bir üst kat, soldaki kapı". `crate::` ile başlayan yol
// tam adrestir ve hep kökten başlar. `self::` "bu odadan", `super::` "bir üst odadan" başlar.
// Hiçbiri yazılmamışsa yol, bulunduğun odadan başlar.
//   Mini örnek (main kökte durduğu için ikisi de aynı yere varır):
//     crate::lokanta::ac();     // ✓ tam adres: kökten başla
//     lokanta::ac();            // ✓ bulunduğun yerden tarif
//     ac();                     // ✗ derlenmez: kökte `ac` diye bir şey yok; lokantanın içinde
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

// SADE ANLATIM (aşağıdaki bölüm için): Modüller çalışma anı maliyeti taşımaz; derleyici önce adı
// çözer, sonra izni denetler; son olarak dosyalara bölmeyi görürüz. İlk okuyuşta atlayabilirsin.
// ============================================================================
// DERLEYİCİ PERSPEKTİFİ — modüller ve görünürlük
// ============================================================================
//
// • Modüller ÇALIŞMA ANI maliyeti taşımaz; yalnız derleme zamanında isimleri düzenler ve
//   API yüzeyini (neyin dışarı açık olduğunu) tanımlar.
//
// (Sade: namespace = adların tutulduğu ayrı defter. Türlerin adları bir defterde, değerlerin
//  adları başka defterde durduğu için aynı ad ikisinde birden bulunabilir.)
// • İsim çözümleme (name resolution): path'ler, `use`'lar ve isimler, TÜR DENETİMİNDEN
//   ÖNCE çözülür — hangi ismin hangi tanıma işaret ettiği erken belirlenir. Kaynak crate:
//   rustc_resolve. (İlginç ayrıntı: Rust'ta 5 namespace vardır — tür, değer, makro, lifetime ve
//   label. Bu örnekte çakışmayı önleyenler TÜR ve DEĞER namespace'leridir: unit struct
//   `struct S;` hem bir TÜR `S` hem bir DEĞER `S` tanımlar; çakışmazlar.)
//
// • Görünürlük (visibility/privacy) denetiminin kendisi ayrı bir geçişte zorlanır (crate:
//   rustc_privacy). Hataların yayın yeri farklıdır: E0603 çoğunlukla rustc_resolve'da (tuple
//   struct constructor için rustc_hir_typeck'te de), E0616 typeck/lower'da, E0451 rustc_privacy'de.
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
