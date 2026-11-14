// Rust Book — Bölüm 15: Smart Pointers (akıllı işaretçiler)
// Çalıştır: cargo run -p smart-pointers
//
// Akıllı işaretçi = bir pointer gibi davranan ama EK yetenek/metadata taşıyan tür (çoğu Deref
// ve Drop uygular). Bunlar sahiplik modelinin "kaçış kapıları"dır: tek-sahip kuralının
// yetmediği durumlarda (heap'te özyineli tip, paylaşımlı sahiplik, iç değişkenlik) devreye
// girer — ama her biri güvenliği BAŞKA bir yolla korur.
//
// Doğrulanmış (rustc ile): Box'sız özyineli tip → E0072 "infinite size"; RefCell'de aynı anda
// iki borrow_mut → ÇALIŞMA ANI panik (BorrowMutError "already borrowed"). Ayrıntı: en altta.

use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

fn main() {
    box_ornegi();
    deref_ornegi();
    drop_ornegi();
    rc_ornegi();
    refcell_ornegi();
}

// ============================================================================
// 1. Box<T> — değeri HEAP'te tutan en basit akıllı işaretçi
// ============================================================================
// En yaygın kullanımı: boyutu derleme anında bilinemeyen ÖZYİNELİ tipler. Bir enum kendini
// doğrudan içerirse boyut sonsuz olur (E0072). Box ARAYA bir pointer koyar: pointer'ın boyutu
// sabittir (bir adres), döngü kırılır.
#[derive(Debug)]
enum Liste {
    Dugum(i32, Box<Liste>), // Box olmadan: "recursive type has infinite size"
    Son,
}
// Değerleri match ile GERÇEKTEN okuyan yürüyüş: hem toplamı verir hem de derive(Debug)
// dead_code tuzağını önler (Böl. 5/8: Debug ile basmak alanları "okundu" saymaz).
fn topla(l: &Liste) -> i32 {
    match l {
        Liste::Dugum(deger, sonraki) => deger + topla(sonraki),
        Liste::Son => 0,
    }
}
fn box_ornegi() {
    use Liste::*;
    let liste = Dugum(1, Box::new(Dugum(2, Box::new(Son))));
    println!("cons list (Box): {liste:?}, toplam = {}", topla(&liste));
    // Box, tek sahiplik + Deref + Drop verir; heap'teki veri Box düşünce otomatik serbest kalır.
}

// ============================================================================
// 2. Deref — `*` operatörünü ve "deref coercion"u özelleştir
// ============================================================================
struct Kutum<T>(T); // kendi mini akıllı işaretçimiz
impl<T> Kutum<T> {
    fn yeni(x: T) -> Kutum<T> {
        Kutum(x)
    }
}
impl<T> Deref for Kutum<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0 // *kutum artık &self.0 demek olur
    }
}
fn selamla(ad: &str) {
    println!("Selam, {ad}!");
}
fn deref_ornegi() {
    let k = Kutum::yeni(5);
    println!("*Kutum(5) = {}", *k); // Deref sayesinde *k çalışır (aslında *(k.deref()))

    // DEREF COERCION: &Kutum<String> → &String → &str zinciri derleyici tarafından otomatik
    // uygulanır; bu yüzden &str isteyen selamla'ya &Kutum<String> verebiliriz.
    let isim = Kutum::yeni(String::from("Kaan"));
    selamla(&isim); // &Kutum<String> --deref--> &String --deref--> &str
}

// ============================================================================
// 3. Drop — değer kapsam dışına çıkınca otomatik temizlik (RAII)
// ============================================================================
struct Kaynak {
    ad: String,
}
impl Drop for Kaynak {
    fn drop(&mut self) {
        // Gerçekte: dosya kapatma, soket bırakma, kilit açma... Burada sadece iz basıyoruz.
        println!("  Drop: '{}' serbest bırakıldı", self.ad);
    }
}
fn drop_ornegi() {
    let _a = Kaynak {
        ad: String::from("A"),
    };
    let b = Kaynak {
        ad: String::from("B"),
    };
    // b'yi ERKEN düşürmek: `b.drop()` çağrılamaz (çift-drop olurdu); std::mem::drop kullanılır.
    drop(b);
    println!("  (drop_ornegi sonu — _a şimdi düşecek)");
    // Düşme SIRASI: tanımın TERSİ. Burada _a, fonksiyon sonunda otomatik düşer.
}

// ============================================================================
// 4. Rc<T> — reference counting: TEK veriye ÇOK sahip (tek thread)
// ============================================================================
#[derive(Debug)]
enum RcListe {
    Dugum(i32, Rc<RcListe>),
    Son,
}
fn rc_topla(l: &RcListe) -> i32 {
    match l {
        RcListe::Dugum(deger, sonraki) => deger + rc_topla(sonraki),
        RcListe::Son => 0,
    }
}
fn rc_ornegi() {
    use RcListe::*;
    // a'yı iki listenin (b ve c) PAYLAŞMASINI istiyoruz. Box olsaydı sahiplik tek olurdu → olmaz.
    let a = Rc::new(Dugum(5, Rc::new(Dugum(10, Rc::new(Son)))));
    println!("a oluştu, sayaç = {}", Rc::strong_count(&a)); // 1
    let b = Dugum(3, Rc::clone(&a)); // Rc::clone VERİYİ KOPYALAMAZ, sadece sayacı artırır
    println!("b paylaştı, sayaç = {}", Rc::strong_count(&a)); // 2
    println!("  b toplamı = {}", rc_topla(&b)); // alanları match ile gerçekten oku
    {
        let c = Dugum(4, Rc::clone(&a));
        println!("c paylaştı, sayaç = {}", Rc::strong_count(&a)); // 3
        println!("  c toplamı = {}", rc_topla(&c));
    }
    println!("c düştü, sayaç = {}", Rc::strong_count(&a)); // 2
    // Sayaç 0'a inince veri serbest kalır. Rc paylaşımlı ama SALT-OKUNUR verir; değiştirmek için →
}

// ============================================================================
// 5. RefCell<T> — iç değişkenlik: borrow kuralını ÇALIŞMA ANINA taşı
// ============================================================================
fn refcell_ornegi() {
    // Rc<RefCell<T>>: hem çok sahip (Rc) hem değiştirilebilir (RefCell) paylaşımlı durum.
    let ortak = Rc::new(RefCell::new(0_i32));
    let a = Rc::clone(&ortak);
    let b = Rc::clone(&ortak);

    *a.borrow_mut() += 10; // borrow_mut() → RefMut (geçici &mut); satır sonunda bırakılır
    *b.borrow_mut() += 5;
    println!("ortak sayaç = {}", ortak.borrow()); // 15

    // ✗ ÇALIŞMA ANI kuralı: aynı anda iki aktif borrow_mut → panik (BorrowMutError):
    //     let _x = ortak.borrow_mut();
    //     let _y = ortak.borrow_mut(); // panik: "already borrowed: BorrowMutError"
    // Kural hâlâ "ya bir &mut ya çok &" — ama RefCell bunu derlemede DEĞİL çalışma anında
    // denetler. Neden? Aşağıda DERLEYİCİ PERSPEKTİFİ.
    println!("  (RefCell borrow kuralı çalışma anında denetlenir)");
}

// ============================================================================
// DERLEYİCİ / KÜTÜPHANE PERSPEKTİFİ — her kaçış kapısı güvenliği NASIL koruyor?
// ============================================================================
//
// • Box<T> (library/alloc/src/boxed.rs): değeri heap'e koyar, kendisi sabit boyutlu bir adres
//   tutar. Özyineli tip (cons list, ağaç) bu yüzden Box ister: aksi halde tip kendini içerir ve
//   boyut sonsuz olur → E0072 "recursive type has infinite size", derleyici "insert indirection
//   (Box/Rc/&)" der. Box tek-sahiptir; Deref + Drop uygular, ekstra runtime yükü yok.
//
// • Deref (library/core/src/ops/deref.rs): `*x`, aslında `*(x.deref())` olarak çözülür.
//   DEREF COERCION derleme anında zincirlenir (&Kutum<String> → &String → &str), böylece akıllı
//   işaretçiler "içindeki değermiş gibi" API'lere geçer. Tamamen statik; runtime maliyeti yok.
//
// • Drop (library/core/src/ops/drop.rs): değer kapsam dışına çıkınca derleyici drop'u OTOMATİK
//   çağırır (RAII) — Bölüm 4'teki "drop elaboration"ın MIR'e eklediği çağrı. Elle drop() yerine
//   std::mem::drop gerekir çünkü değeri taşıyıp bırakır; .drop()'u sen çağırsan sonra bir de
//   otomatik çağrı gelir, çift-drop olurdu — derleyici bu yüzden doğrudan .drop()'u yasaklar.
//
// • Rc<T> (library/alloc/src/rcs/rc.rs): veriyle birlikte bir SAYAÇ (strong_count) tutar.
//   Rc::clone sayacı +1 yapar (veriyi kopyalamaz — ucuz); bir Rc düşünce -1; sayaç 0 olunca veri
//   serbest kalır. Yalnız TEK THREAD için (sayaç atomik değil; çok thread için Arc). Rc salt
//   paylaşımlı & verir — değiştiremezsin. Sorun bu: paylaşım + değişiklik aynı anda isteniyorsa?
//
// • RefCell<T> (library/core/src/cell.rs): İÇ DEĞİŞKENLİK (interior mutability). Aliasing XOR
//   Mutability kuralını KALDIRMAZ — denetimi derleme anından ÇALIŞMA anına taşır. borrow()
//   /borrow_mut() içeride bir borrow sayacı tutar; "bir &mut VEYA çok &" ihlal edilirse derhal
//   PANİK eder (BorrowMutError "already borrowed", cell.rs ~880). Neden böyle bir şey? Sorun:
//   bazı doğru programları borrow checker (statik, temkinli) reddeder; RefCell "ben doğruluğu
//   çalışma anı denetimiyle garanti ederim" demenin güvenli yoludur — hata olursa çökme, bellek
//   bozulması değil. Klasik birleşim Rc<RefCell<T>>: çok sahipli + değiştirilebilir düğüm.
//
// • REFERANS DÖNGÜSÜ ve Weak: Rc<RefCell<...>> ile iki düğüm birbirini Rc ile tutarsa sayaç
//   asla 0 olmaz → BELLEK SIZINTISI (Rust bunu güvensiz saymaz; UB değil, sadece sızıntı).
//   Çözüm: döngüyü kıran kenarda Rc yerine Weak<T> kullanmak. Weak sayacı artırmaz (strong değil,
//   weak sayaç), upgrade() ile Option<Rc> verir — veri hâlâ var mı diye sorar. Çocuk→ebeveyn
//   kenarı Weak yapılırsa döngü oluşmaz, sızıntı önlenir.
