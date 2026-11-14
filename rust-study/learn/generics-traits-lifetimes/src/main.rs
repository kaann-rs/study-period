// Rust Book — Bölüm 10: Generic Types, Traits, and Lifetimes
// Çalıştır: cargo run -p generics-traits-lifetimes
//
// Üç konu tek bölümde çünkü üçü de AYNI amaca hizmet eder: tekrarı azaltmak ve DERLEME
// ANINDA güvenliği korumak. Generics "farklı türler için tek kod", traits "farklı türlerin
// ortak davranışı", lifetimes "referansların ne kadar geçerli kaldığı".
//
// Doğrulanmış (rustc ile): trait bound sağlanmazsa E0277; olmayan metot E0599; elision
// başarısız → E0106; `dyn` için uygun olmayan trait → E0038; orphan rule → E0117.
// (Lifetime/HRTB ve orphan rule'u daha önce ayrıca derinlemesine konuşmuştuk; burada
// kitabın çerçevesine oturtuyorum.)

use std::fmt::Display;

fn main() {
    generikler();
    traitler();
    lifetimelar();
}

// ============================================================================
// 1. GENERICS (jenerikler) — tek kod gövdesi, çok tür
// ============================================================================
fn generikler() {
    // Generic FONKSİYON: <T> bir TÜR PARAMETRESİdir. T: PartialOrd "T karşılaştırılabilmeli"
    // kısıtını (trait bound) koyar; yoksa `>` kullanamayız (bu bir trait metodudur).
    fn en_buyuk<T: PartialOrd>(liste: &[T]) -> &T {
        let mut en = &liste[0];
        for x in liste {
            if x > en {
                en = x;
            }
        }
        en
    }
    println!("en büyük (i32)  = {}", en_buyuk(&[3, 7, 2, 9, 4]));
    println!("en büyük (char) = {}", en_buyuk(&['a', 'z', 'm']));

    // Generic STRUCT: aynı tanımla hem int hem float noktalar.
    #[derive(Debug)]
    struct Nokta<T> {
        x: T,
        y: T,
    }
    // impl bloğunun da <T>'yi bildirmesi gerekir: "her T için bu metot".
    impl<T> Nokta<T> {
        fn x(&self) -> &T {
            &self.x
        }
    }
    // SADECE belirli bir tür için metot: impl Nokta<f64> { ... } (koşullu metot).
    impl Nokta<f64> {
        fn uzaklik(&self) -> f64 {
            (self.x * self.x + self.y * self.y).sqrt() // sqrt yalnız f64'te var
        }
    }
    let ni = Nokta { x: 3, y: 4 };
    let nf = Nokta { x: 3.0, y: 4.0 };
    println!("ni.x = {}, nf.uzaklık = {}", ni.x(), nf.uzaklik());
    // self.y'yi de okuyalım ki derive(Debug) dead_code tuzağına düşmeyelim (Bölüm 5 nugget'ı).
    println!("ni = {ni:?} (y = {})", ni.y);

    // Neden generics? Sorun: generics olmasaydı her tür için AYNI mantığı kopyalamak gerekirdi
    // (en_buyuk_i32, en_buyuk_char ...). Hem tekrar, hem bir yerde düzeltip ötekini unutma riski.
    // Neden "zero-cost"? Çünkü runtime'da tür bilgisi taşınmaz — bkz. DERLEYİCİ PERSPEKTİFİ.
}

// ============================================================================
// 2. TRAITS (özellikler) — paylaşılan davranışın arayüzü
// ============================================================================

// Trait = bir türün uygulaması gereken metot imzaları kümesi (başka dillerdeki "interface"e
// yakın). VARSAYILAN gövde verilebilir; tür isterse ezer (override).
trait Ozet {
    fn yazar(&self) -> String; // zorunlu: her tür kendi verir
    fn ozetle(&self) -> String {
        // varsayılan: zorunlu metodu kullanabilir
        format!("(devamı...) — {}", self.yazar())
    }
}

struct Haber {
    baslik: String,
    yazar: String,
}
struct Tweet {
    kullanici: String,
}
impl Ozet for Haber {
    fn yazar(&self) -> String {
        self.yazar.clone()
    }
    fn ozetle(&self) -> String {
        // varsayılanı EZER
        format!("{} — {}", self.baslik, self.yazar)
    }
}
impl Ozet for Tweet {
    fn yazar(&self) -> String {
        format!("@{}", self.kullanici) // ozetle'yi EZMEZ → varsayılan gövde çalışır
    }
}

// Trait BOUND fonksiyon: "Ozet uygulayan HER tür". İki eşdeğer yazım:
fn duyur(x: &impl Ozet) {
    // `impl Trait` = kısa yazım (anonim generic)
    println!("Yeni: {}", x.ozetle());
}
fn _duyur_uzun<T: Ozet>(x: &T) {
    println!("Yeni: {}", x.ozetle());
}

fn traitler() {
    let h = Haber {
        baslik: String::from("Rust 2024"),
        yazar: String::from("Kaan"),
    };
    let t = Tweet {
        kullanici: String::from("kaan"),
    };
    duyur(&h); // ezilmiş ozetle
    duyur(&t); // varsayılan ozetle

    // STATİK vs DİNAMİK dispatch — Rust'ın iki çağırma biçimi:
    //
    // (a) GENERIC (statik dispatch): her somut tür için AYRI kod üretilir (monomorphization),
    //     çağrı derleme anında çözülür → hızlı, satır içine (inline) alınabilir.
    //
    // (b) TRAIT OBJECT `dyn Ozet` (dinamik dispatch): tek kod; hangi metot çağrılacağı
    //     ÇALIŞMA ANINDA vtable (sanal metot tablosu) üzerinden bulunur. Esnek: farklı türleri
    //     TEK bir Vec'te tutabilirsin; bedeli bir pointer dolaylaması.
    let karisik: Vec<Box<dyn Ozet>> = vec![
        Box::new(Haber {
            baslik: String::from("Hava"),
            yazar: String::from("Ajans"),
        }),
        Box::new(Tweet {
            kullanici: String::from("bot"),
        }),
    ];
    for oge in &karisik {
        println!("liste: {}", oge.ozetle()); // dinamik dispatch (vtable)
    }
    // Neden iki yol? Sorun: tek yol olsaydı ya hep tür başına kod şişmesi (generic'in bedeli)
    // ya hep dolaylama (dyn'in bedeli) yaşardık. Rust seçimi SANA bırakır: homojen + sıcak yol →
    // generic; heterojen koleksiyon/eklenti → dyn. (dyn olabilmesi için trait "dyn compatible"
    // olmalı; neden, DERLEYİCİ PERSPEKTİFİ'nde.)
}

// Blanket impl örneği: "Display olan HER türe benim trait'imi uygula" — stdlib'in
// `impl<T: Display> ToString for T` kalıbı. Burada küçük bir örnek:
trait Etiket {
    fn etiket(&self) -> String;
}
impl<T: Display> Etiket for T {
    fn etiket(&self) -> String {
        format!("[{self}]")
    }
}

// ============================================================================
// 3. LIFETIMES (yaşam süreleri) — referanslar ne kadar geçerli?
// ============================================================================

// Lifetime, bir referansın İŞARET ETTİĞİ verinin en az ne kadar yaşayacağını söyleyen bir
// GENERIC parametredir (tür değil, SÜRE/bölge parametresi). Yeni bir süre yaratmaz; var olan
// sürelerin İLİŞKİSİNİ derleyiciye bildirir.
//
// 'a burada "dönen referans, x ve y'nin İKİSİNİN de yaşadığı ortak bölge kadar geçerli" der.
fn en_uzun<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

// Referans TUTAN struct: alan bir referanssa, struct kendi verisini aşamaz; bu yüzden
// lifetime ZORUNLU. "Bu Alinti, içindeki &str yaşadığı sürece geçerli."
struct Alinti<'a> {
    parca: &'a str,
}
impl<'a> Alinti<'a> {
    fn parca(&self) -> &str {
        self.parca
    }
}

fn lifetimelar() {
    let s1 = String::from("uzun bir cümle");
    let sonuc;
    {
        let s2 = String::from("kısa");
        sonuc = en_uzun(s1.as_str(), s2.as_str());
        println!("en uzun = {sonuc}"); // s2 HÂLÂ yaşıyorken kullan → OK
    }
    // println!("{sonuc}"); // ✗ burada s2 düştü; 'a ikisinin ortak bölgesi olduğu için E0597 olurdu

    let metin = String::from("İlk cümle. İkincisi.");
    let ilk = metin.split('.').next().unwrap();
    let a = Alinti { parca: ilk };
    println!("alıntı: {}", a.parca());

    // ELISION (lifetime çıkarımı): çoğu referanslı imzada 'a yazmayız çünkü derleyici 3 kuralla
    // kendi doldurur (bkz. DERLEYİCİ PERSPEKTİFİ). `fn parca(&self) -> &str` yukarıda o yüzden
    // 'a'sız yazılabildi: Kural 3, &self'in süresini çıktıya verir.
    //
    // 'static: tüm program boyu yaşayan özel süre. String LİTERALLERİ 'static'tir (ikiliye
    // gömülü). Her yere 'static yazmak çözüm değildir — çoğu zaman gerçek sorun dangling
    // referanstır, süre etiketini uzatmak değil.
    let s: &'static str = "program boyu yaşar";
    println!("{s} / etiket örneği: {}", 42.etiket()); // blanket impl (Display→Etiket)
}

// ============================================================================
// DERLEYİCİ PERSPEKTİFİ — üçü de derleme anında, runtime bedeli (çoğunlukla) sıfır
// ============================================================================
//
// • MONOMORPHIZATION (jeneriklerin "zero-cost" sırrı): generic kod runtime'da tür taşımaz.
//   Derleyici, kodun KULLANDIĞI her somut tür için ayrı bir kopya üretir — en_buyuk::<i32> ve
//   en_buyuk::<char> ayrı iki fonksiyon olur. Bu toplama rustc_monomorphize/src/collector.rs
//   yapar: monomorphic örnekleri "ihtiyaç oldukça" (on demand) toplar. Dosyanın baş yorumu
//   (satır ~12) şunu da söyler: X crate'indeki bir generic, Y crate'inde derlenen bir
//   monomorphization üretebilir — ayrı derlemeyle (separate compilation) uyumlu. Sonuç:
//   C++ template'leri gibi hızlı (dolaylama yok), ama ikili dosya büyür (kod kopyalanır).
//   dyn Trait ise tersi: tek kod + vtable, çalışma anı dolaylaması.
//
// • TRAIT BOUND çözümü tür denetleyicide (rustc_hir_typeck/rustc_trait_selection) yapılır:
//   `T: Ozet` sağlanmazsa daha derlemede E0277. Metot bir trait'ten gelip de bound yoksa
//   "no method named ..." E0599. Yani "ördek tipi" (duck typing) YOK — bir metodu çağırmak
//   için o türün ilgili trait'i uyguladığı KANITLANMALI.
//
// • dyn COMPATIBILITY (eski adıyla "object safety"): her trait `dyn` olamaz. Kontrol
//   rustc_trait_selection/src/traits/dyn_compatibility.rs'tedir. Kabaca: trait bir VTABLE
//   kurabilmeli. Bu yüzden `fn uret() -> Self` gibi Self DÖNEN ya da generic metotlu trait
//   dyn olamaz (E0038): çünkü vtable'da hangi somut türü kuracağı/hangi monomorphization'ı
//   koyacağı belli değildir. Sorun: izin verilse, çalışma anında var olmayan bir tür/kopya
//   çağrılmaya kalkılırdı.
//
// • LIFETIME ELISION (3 kural, çıktı referansının süresini DOLDURUR; isim çözümlemede,
//   rustc_resolve/src/late.rs, `elided`/`current_elision_failures` ~399/~840):
//     1) Her referans parametre kendi ayrı süresini alır.
//     2) TEK girdi süresi varsa, o süre tüm çıktılara verilir.
//     3) Metotta &self/&mut self varsa, self'in süresi tüm çıktılara verilir.
//   Kurallar çıktıyı tek bir süreye bağlayamazsa derleyici PES eder ve senden etiket ister
//   (E0106). Yani elision bir TAHMİN değil, deterministik 3 kuraldır; belirsizlikte sessiz
//   kalmaz, hata verir. Lifetime'lar yalnız derleme anında denetlenir — üretilen makine
//   kodunda lifetime diye bir şey YOKTUR (tür silme / "erasure").
