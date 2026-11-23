// Açıklama: learn/oop/ACIKLAMA.md
// Rust Book — Bölüm 18: Object-Oriented Programming Features
// Çalıştır: cargo run -p oop
//
// Rust OOP'nin bazı parçalarını (nesne = veri + davranış; encapsulation) destekler ama KALITIM
// (inheritance) YOKTUR. Yerine: davranış paylaşımı için trait + varsayılan metot, kod paylaşımı
// için KOMPOZİSYON. Bu bölüm iki kalıp gösterir:
//   1) Trait object ile çalışma-anı polimorfizmi (GUI örneği).
//   2) Klasik "state pattern" yerine Rust-idiomatik TYPE-STATE: geçersiz durumu DERLEME ANINDA
//      imkânsız kıl.
//
// (dyn/statik-dinamik dispatch temelini Böl. 10'da (generics-traits-lifetimes) ve
//  ileri-sized-dst'de kurmuştuk; burada tasarım kalıbı var.)

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Her bölümün "SADE ANLATIM" kutusunu önce oku; teknik kısım zorsa ana fikir orada.
// Bu crate'teki benzetmelerin çoğu bir OYUNCAK KUTUSU çevresinde geçer; derinlesme.rs de öyle.
//
// Bu bölümün tek cümlelik özeti: Rust'ta "sınıftan sınıf türetmek" (kalıtım) yoktur; onun yerine
// veriyi korursun, ortak davranışı trait ile tarif edersin ve yanlış kullanımı türlerle önlersin.
//
// MİNİ SÖZLÜK (bu dosyada geçen kelimeler):
//   nesne (object)           → veri ile o veriyi kullanan fonksiyonların bir arada durduğu şey.
//   kapsülleme (encapsulation) → içini saklayıp dışarıya yalnız izin verdiğin düğmeleri açmak.
//   private / pub            → dışarıya kapalı / dışarıya açık.
//   modül (mod)              → kodun bir odası; gizlilik sınırı bu odanın kapısıdır.
//   invariant (değişmez)     → her zaman doğru kalması gereken kural ("ortalama hep güncel").
//   API                      → bir türün dışarıya açtığı fonksiyonların tamamı.
//   önbellek (cache)         → yeniden hesaplamamak için saklanan hazır sonuç.
//   kalıtım (inheritance)    → bir türün başka türün her şeyini devralması. Rust'ta YOK.
//   kompozisyon (bileşim)    → bir türü başka türün içine parça (alan) olarak koymak.
//   trait / varsayılan metot → "şunu yapabilirim" sözü / o sözün hazır gelen hâli.
//   trait object (`dyn Trait`) → "türü ne olursa olsun, şu işi yapabilen bir şey".
//   polimorfizm              → aynı çağrının, karşısındaki türe göre farklı iş yapması.
//   dispatch (statik / dinamik) → çağrılacak fonksiyonun derlerken / çalışırken belirlenmesi.
//   vtable                   → "bu türde şu metot şu adreste" yazan küçük tablo.
//   fat pointer              → iki parçalı işaretçi: biri veriyi, öteki vtable'ı gösterir.
//   heterojen / homojen      → içinde farklı türler olan / hepsi aynı türden olan (liste).
//   generic, monomorphization → türü sonradan dolan şablon / her tür için kopya üretilmesi.
//   state pattern            → bir nesnenin durumuna göre farklı davranmasını sağlayan kalıp.
//   type-state               → her durumu ayrı bir TÜR yapıp yanlış adımı derlenmez kılmak.
//   self'i tüketmek          → metodun değeri alıp bitirmesi; çağrıdan sonra eskisi kullanılamaz.
//   open/closed              → "eski koda dokunmadan yenisi eklenebilsin" tasarım ilkesi.
//   dyn compatible           → bir trait'in `dyn` olarak kullanılabilmesi için gereken şartlar.

mod derinlesme;
fn main() {
    encapsulation();
    gui_trait_object();
    type_state_blog();
    derinlesme::calistir();
}

// ============================================================================
// 1. ENCAPSULATION — private alan + invariant koruyan public API
// ============================================================================
// Alanlar PRIVATE (pub değil): dışarıdan `liste`ye/`ortalama`ya dokunulamaz. Yalnız pub `ekle`
// ile değişir ve o da ORTALAMAYI tutarlı tutar. Dış dünya iç temsili bilmez → iç temsili
// değiştirebiliriz (örn. Vec yerine başka yapı) ve public API bozulmaz.
// SADE ANLATIM: Bir kumbara düşün: parayı yalnız yarığından atarsın, üstündeki gösterge toplamı
// yazar. İçini elle karıştıramadığın için gösterge hep doğrudur. Aşağıdaki türde de `liste` ve
// `ortalama` içeride saklıdır; dışarıya açık tek yol `ekle` düğmesidir ve o her seferinde
// ortalamayı da günceller.
// İçerisi açık olsaydı biri ortalamayı güncellemeyi unutur, gösterge yanlış kalırdı.
// Dikkat: gizlilik TÜRE değil MODÜLE göredir; aynı modüldeki kod private alana erişebilir.
//   Mini örnek:
//     let mut o = OrtalamaListe::yeni();
//     o.ekle(10);                     // ✓ yarıktan at: liste ile ortalama birlikte güncellenir
//     println!("{}", o.ortalama());   // ✓ 10
//     // o.liste.push(999);           // ✗ BAŞKA bir modülden yazılırsa derlenmez: `liste` private
//     //                              //   (bu dosyada, yani aynı modülde yazılırsa derlenir)
pub struct OrtalamaListe {
    liste: Vec<i32>, // private
    ortalama: f64,   // private, önbellek (cache)
}

impl OrtalamaListe {
    pub fn yeni() -> OrtalamaListe {
        OrtalamaListe {
            liste: vec![],
            ortalama: 0.0,
        }
    }
    pub fn ekle(&mut self, deger: i32) {
        self.liste.push(deger);
        self.ortalama_guncelle(); // invariant: her değişimde ortalama güncel kalsın
    }
    pub fn ortalama(&self) -> f64 {
        self.ortalama // dışarı sadece okunur değer veririz
    }
    fn ortalama_guncelle(&mut self) {
        // private: dışarıdan çağrılamaz → kimse ortalamayı liste ile tutarsız bırakamaz
        let toplam: i32 = self.liste.iter().sum();
        self.ortalama = toplam as f64 / self.liste.len() as f64;
    }
}
fn encapsulation() {
    let mut o = OrtalamaListe::yeni();
    o.ekle(10);
    o.ekle(20);
    o.ekle(30);
    // Bu dosyada derlenir: gizlilik MODÜLE göredir (crate kökündeyiz). E0616 ancak tür ayrı bir
    // `mod` içinde tanımlanıp dışından erişilince çıkar (örn. learn/structs/src/derinlesme.rs).
    println!("ortalama = {}", o.ortalama()); // 20
}

// ============================================================================
// 2. TRAIT OBJECT ile polimorfizm — farklı türleri TEK listede, çalışma anında dispatch
// ============================================================================
// SADE ANLATIM: Üstünde "ses çıkarabilen her şey" yazan bir oyuncak kutusu düşün. İçinde düdük,
// davul, zil olabilir. Kutudan çıkanın ne olduğuna bakmadan "ses çıkar" dersin; her oyuncak
// KENDİ sesini çıkarır. Rust'ta `Box<dyn Ciz>` de böyledir: "türü ne olursa olsun, çizilebilen
// bir şey". Hangi `ciz` fonksiyonunun çalışacağı program çalışırken, oyuncağın yanındaki küçük
// tablodan (vtable) bulunur.
// Vec tek tür tutar; kutu olmasaydı Buton ile OnayKutusu aynı listeye giremezdi.
//   Mini örnek:
//     let kutu: Vec<Box<dyn Ciz>> = vec![
//         Box::new(Buton { etiket: String::from("Tamam") }),
//         Box::new(OnayKutusu { isaretli: true }),     // ✓ farklı türler aynı listede
//     ];
//     for b in &kutu { println!("{}", b.ciz()); }      // ✓ her biri kendi ciz'ini çalıştırır
//     // let v: Vec<Buton> = vec![OnayKutusu { isaretli: true }];   // ✗ derlenmez: tek tür tutar
trait Ciz {
    fn ciz(&self) -> String;
}
struct Buton {
    etiket: String,
}
struct OnayKutusu {
    isaretli: bool,
}
impl Ciz for Buton {
    fn ciz(&self) -> String {
        format!("[ {} ]", self.etiket)
    }
}
impl Ciz for OnayKutusu {
    fn ciz(&self) -> String {
        format!("({}) seçim", if self.isaretli { "x" } else { " " })
    }
}
// Ekran, SOMUT türleri bilmez; yalnız "Ciz uygulayan bir şey". Yeni bir bileşen türü eklemek
// için Ekran'a dokunmak GEREKMEZ (open/closed) — kalıtım olmadan genişletilebilirlik.
struct Ekran {
    bilesenler: Vec<Box<dyn Ciz>>,
}
impl Ekran {
    fn calistir(&self) {
        for b in &self.bilesenler {
            println!("  {}", b.ciz()); // dinamik dispatch (vtable)
        }
    }
}
fn gui_trait_object() {
    let ekran = Ekran {
        bilesenler: vec![
            Box::new(Buton {
                etiket: String::from("Kaydet"),
            }),
            Box::new(OnayKutusu { isaretli: true }),
            Box::new(Buton {
                etiket: String::from("İptal"),
            }),
        ],
    };
    println!("ekran:");
    ekran.calistir();
    // Neden dyn? Vec<T> tek tür tutar, heterojen liste olmaz. Bedel: vtable dolaylaması
    // (kök neden Böl. 10 ve ileri-sized-dst'de).
}

// ============================================================================
// 3. TYPE-STATE — "state pattern"i tür sistemiyle ifade et
// ============================================================================
// Klasik OOP'de bir blog yazısının durumu (Taslak/İnceleme/Yayın) bir alanda tutulur ve her
// metot "doğru durumda mıyım?" diye ÇALIŞMA ANINDA kontrol eder. Rust-idiomatik yol: her durum
// AYRI BİR TÜR. Geçiş, eski değeri TÜKETİP (self alır) yeni tür döndürür. Böylece yanlış durumda
// yanlış metodu çağırmak DERLENMEZ — geçersiz durum temsil edilemez.
// SADE ANLATIM: Bir mektup üç hâlden geçer: yazılan kâğıt, kapalı zarf, postalanmış mektup.
// Kâğıdı zarfa koyup kapatınca elinde artık kâğıt yoktur, zarf vardır; zarfın üstüne yazı
// ekleyemezsin, çünkü zarfın "yazı ekle" diye bir özelliği yoktur. Aşağıda da her hâl ayrı bir
// türdür (Taslak, Inceleme, Yayin). Bir hâlden ötekine geçen metot eskisini tüketir (`self`
// alır) ve yenisini verir.
// Durum tek alanda olsaydı her metot "doğru durumda mıyım?" diye bakardı; burada yanlış adım derlenmez.
//   Mini örnek:
//     let mut m = Taslak::yeni();
//     m.ekle("Merhaba");                   // ✓ taslağa yazılır
//     let zarf = m.incelemeye_gonder();    // taslak tüketildi; elde artık Inceleme var
//     // m.ekle("bir şey daha");           // ✗ derlenmez: m artık yok (taşındı)
//     // zarf.icerik();                    // ✗ derlenmez: icerik() yalnız Yayin'de var
//     println!("{}", zarf.onayla().icerik());   // ✓ ancak yayımlanınca okunur
struct Taslak {
    icerik: String,
}
struct Inceleme {
    icerik: String,
}
struct Yayin {
    icerik: String,
}
impl Taslak {
    fn yeni() -> Taslak {
        Taslak {
            icerik: String::new(),
        }
    }
    fn ekle(&mut self, s: &str) {
        self.icerik.push_str(s);
    }
    fn incelemeye_gonder(self) -> Inceleme {
        // self'i TÜKETİR: artık elde Taslak kalmaz, yanlışlıkla ekle() çağrılamaz
        Inceleme {
            icerik: self.icerik,
        }
    }
}
impl Inceleme {
    fn onayla(self) -> Yayin {
        Yayin {
            icerik: self.icerik,
        }
    }
    fn reddet(self) -> Taslak {
        Taslak {
            icerik: self.icerik,
        }
    }
}
impl Yayin {
    // icerik() SADECE Yayin'de var. Taslak/İnceleme üstünde .icerik() çağırmak derleme hatası.
    fn icerik(&self) -> &str {
        &self.icerik
    }
}
fn type_state_blog() {
    let mut yazi = Taslak::yeni();
    yazi.ekle("Bugün Rust öğrendim. ");
    yazi.ekle("Çok iyiydi.");
    // yazi.icerik(); // ✗ DERLENMEZ: Taslak'ta icerik() yok → yayınlanmamış yazı okunamaz

    let incelemede = yazi.incelemeye_gonder();
    // yazi.ekle(...); // ✗ DERLENMEZ: yazi taşındı (incelemeye_gonder tüketti)

    // reddet → tekrar Taslak: inceleyen beğenmedi, düzeltip yeniden gönderelim.
    let mut tekrar_taslak = incelemede.reddet();
    tekrar_taslak.ekle(" (düzeltildi)");
    let yayinda = tekrar_taslak.incelemeye_gonder().onayla();
    println!("yayınlanan içerik: {}", yayinda.icerik()); // ancak ŞİMDİ okunabilir
    // Klasik state pattern'de derleyici "yanlış durumda yanlış metot"u yakalayamaz; type-state
    // bunu TÜRE taşır → hata derlenmeden gelir ("Make invalid states unrepresentable.").
}

// SADE ANLATIM (aşağıdaki bölüm için): Aşağıdaki bölüm "Rust neden kalıtımı almadı?" sorusunu
// cevaplar. Kısa cevap: kalıtım iki ayrı işi (kodu paylaşmak ve farklı türlere aynı gözle
// bakmak) birbirine bağlar; Rust bunları ayırır. Kod paylaşmak için bir türü ötekinin içine
// parça olarak koyarsın, aynı gözle bakmak için trait kullanırsın. Bölüm ayrıca oyuncak kutusunun
// (trait object) bellekte iki parçalı bir işaretçi olduğunu anlatır. İlk okuyuşta atlayabilirsin.
// ============================================================================
// DERLEYİCİ / TASARIM PERSPEKTİFİ — Rust neden kalıtımsız?
// ============================================================================
//
// • KALITIM YOK, bilinçli bir seçim. Kalıtımın iki işi vardır: (a) kod paylaşımı, (b) polimorfizm.
//   Rust ikisini AYIRIR: (a) için trait varsayılan metotları + kompozisyon (bir türü başka bir
//   türe ALAN olarak koy), (b) için trait + (statik) generic ya da (dinamik) trait object.
//   Sorun (klasik kalıtım): derin sınıf hiyerarşileri kırılgan taban sınıfı (fragile base class),
//   istenmeyen davranış miras alma, "is-a" zorlaması getirir. trait+kompozisyon yalnız İHTİYAÇ
//   duyulan davranışı seçer.
//
// • ENCAPSULATION: alan/metot varsayılan PRIVATE, pub ile açılır (Böl. 7). MODÜL DIŞINDAN erişim E0616.
//   Bu yüzden invariant'ı ("ortalama hep güncel") korumak için türü kendi `mod`una koyarsın — dış kod
//   iç tutarlılığı bozamaz.
//
// • TRAIT OBJECT = fat pointer: Box<dyn Ciz>, (veri-pointer, vtable-pointer) ikilisidir. vtable
//   her somut tür için ciz()'in adresini tutar; çağrı çalışma anında oradan bulunur (dinamik
//   dispatch). Heterojen koleksiyon ve eklenti mimarisi bunu gerektirir; homojen+sıcak yol için
//   generic (monomorphization) daha ucuzdur. Trait'in dyn olabilmesi için "dyn compatible" olması
//   şart (Böl. 10, E0038).
//
// • TYPE-STATE, tür sisteminin bir TASARIM aracı olarak kullanımı: durumu türe kodlayıp geçişi
//   "self'i tüketen" metotlarla yaparsan, derleyici durum makinesinin geçersiz kenarlarını
//   reddeder. Hiçbir çalışma-anı kontrolü/panik gerekmez; yanlış kullanım derlenmez. Bu, Rust
//   felsefesinin özüdür: hataları olabildiğince derleme anına çek.
