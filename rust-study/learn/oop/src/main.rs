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
// (dyn/statik-dinamik dispatch temelini Böl. 10 ve 15'te kurmuştuk; burada tasarım kalıbı var.)
// ↳ DÜZELTME NOTU (son gözden geçirme): dispatch/vtable konusu Böl. 10'da (generics-traits-lifetimes) ve
//   ileri-sized-dst'de işlendi; Böl. 15 (smart-pointers) bu konuyu içermiyor.

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Bu dosyadaki her bölümün üstünde "SADE ANLATIM" diye bir kutu var. Önce onu oku, sonra
// altındaki teknik açıklamaya geç. Teknik kısım zor gelirse sorun değil: sade kutu ana fikri verir.
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
// Neden böyle? İçerisi herkese açık olsaydı biri listeye sayı ekleyip ortalamayı güncellemeyi
// unutabilir, gösterge yanlış kalırdı.
// Dikkat: Rust'ta gizlilik TÜRE değil MODÜLE göredir. Aynı modüldeki kod (kumbarayla aynı odada
// duran herkes) private alana erişebilir; kilit, odanın kapısındadır.
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
    // o.liste.push(999); // ✗ E0616: field `liste` is private — invariant'ı dışarıdan bozamazsın
    // ↳ DÜZELTME NOTU (son gözden geçirme): bu satır BU DOSYADA açılırsa DERLENİR. Rust'ta gizlilik tür bazlı
    //   değil MODÜL bazlıdır: OrtalamaListe ile bu fonksiyon aynı modülde (crate kökü) olduğu için
    //   private alana erişebilir. E0616 yalnız tür ayrı bir `mod` içinde tanımlanıp DIŞINDAN erişilince
    //   çıkar (rustc ile doğrulandı). Modülle korunan çalışan örnek: learn/structs/src/derinlesme.rs.
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
// Neden böyle? Bir `Vec` normalde tek tür tutar. Bu kutu olmasaydı Buton ile OnayKutusu aynı
// listeye giremezdi; her yeni bileşen türü için Ekran'ı baştan yazman gerekirdi.
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
    // Neden generic <T: Ciz> yerine dyn? Sorun: Vec<T> tek bir T tutar — HETEROJEN liste olmaz.
    // Buton + OnayKutusu'yu aynı Vec'te tutmak için trait object (dyn) şart. Bedel: vtable
    // dolaylaması (Böl. 10/15'te kök neden).
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
// Neden böyle? Durum tek bir alanda yazsaydı ("şu an: taslak") her metot çalışırken "doğru
// durumda mıyım?" diye bakmak zorunda kalırdı; burada yanlış adım hiç derlenmez.
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
    // Neden? Sorun (klasik state pattern): durum bir alandaysa, derleyici "yanlış durumda yanlış
    // metot" hatasını yakalayamaz; çalışma anı kontrolü + hata/panik gerekir. Type-state bunu
    // TÜRE taşır → hata, kod derlenmeden önce yakalanır. "Make invalid states unrepresentable."
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
// • ENCAPSULATION tam destekli: alan/metot varsayılan PRIVATE, pub ile açılır (Böl. 7). Private
//   alana dışarıdan erişim E0616. Bu, bir türün invariant'ını (OrtalamaListe'de "ortalama hep
// ↳ DÜZELTME NOTU (son gözden geçirme): "dışarıdan" = tanımlandığı MODÜLÜN dışından. Aynı modüldeki kod private
//   alana erişebilir; invariant'ı gerçekten korumak için türü kendi `mod`una koymak gerekir.
//   güncel") garanti altına almanın yoludur — dış kod iç tutarlılığı bozamaz.
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
