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

fn main() {
    encapsulation();
    gui_trait_object();
    type_state_blog();
}

// ============================================================================
// 1. ENCAPSULATION — private alan + invariant koruyan public API
// ============================================================================
// Alanlar PRIVATE (pub değil): dışarıdan `liste`ye/`ortalama`ya dokunulamaz. Yalnız pub `ekle`
// ile değişir ve o da ORTALAMAYI tutarlı tutar. Dış dünya iç temsili bilmez → iç temsili
// değiştirebiliriz (örn. Vec yerine başka yapı) ve public API bozulmaz.
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
    println!("ortalama = {}", o.ortalama()); // 20
}

// ============================================================================
// 2. TRAIT OBJECT ile polimorfizm — farklı türleri TEK listede, çalışma anında dispatch
// ============================================================================
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
