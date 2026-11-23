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

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Bu dosyadaki her bölümün üstünde "SADE ANLATIM" diye bir kutu var. Önce onu oku, sonra
// altındaki teknik açıklamaya geç. Teknik kısım zor gelirse sorun değil: sade kutu ana fikri verir.
// Bu crate'teki benzetmelerin hepsi aynı yerde, bir OKULDA geçer; derinlesme.rs de orada sürer.
//
// Bu bölümün tek cümlelik özeti: "Her verinin tek sahibi olur" kuralı bazen dar gelir; akıllı
// işaretçiler o durumlar için hazır kutulardır ve her biri güvenliği kendi yoluyla korur.
//
// MİNİ SÖZLÜK (bu dosyada geçen kelimeler):
//   işaretçi (pointer)       → verinin kendisi değil, nerede durduğunu yazan adres kâğıdı.
//   akıllı işaretçi (smart pointer) → adres kâğıdı + ek iş: sayaç tutar, çıkarken temizler.
//   metadata                 → verinin yanında tutulan ek bilgi (örneğin "kaç sahibi var" sayısı).
//   heap / stack             → bellekteki iki bölge. Stack: küçük ve hızlı raf. Heap: büyük depo.
//   Box<T>                   → veriyi depoya (heap) koyan, elinde yalnız adresini tutan kutu.
//   özyineli tip (recursive type) → içinde kendi türünden bir parça taşıyan tür (iç içe kutular).
//   cons list                → "bir değer + listenin geri kalanı" diye kurulan zincir liste.
//   Deref ve `*`             → adres kâğıdından verinin kendisine gitmek ("adrese git, bak").
//   deref coercion           → derleyicinin bu "adrese git" adımını senin yerine atması.
//   Drop                     → bir değer silinirken kendiliğinden çalışan temizlik işi.
//   RAII                     → "işi biten, arkasını kendi toplar" düzeninin kısa adı.
//   Rc<T>                    → bir veriyi birden çok sahibin paylaşmasını sağlayan sayaçlı kutu.
//   sayaç (strong_count)     → o veriyi şu anda kaç sahibin tuttuğunu gösteren sayı.
//   RefCell<T>               → ödünç kuralını derlerken değil, program çalışırken denetleyen kutu.
//   iç değişkenlik (interior mutability) → dışarıdan değişmez görünen şeyin içini değiştirebilmek.
//   borrow() / borrow_mut()  → RefCell'den "bakmak için" / "değiştirmek için" izin istemek.
//   Aliasing XOR Mutability  → "aynı anda ya çok bakan olur ya tek değiştiren" kuralının adı.
//   panik (panic)            → programın "bu hatayla devam edemem" deyip kendini durdurması.
//   Weak<T>                  → verinin yerini bilen ama sayaca yazılmayan, sahip olmayan işaretçi.
//   referans döngüsü (reference cycle) → iki verinin birbirini tutup hiç silinememesi.
//   bellek sızıntısı (memory leak) → işi biten verinin geri verilmeyip yer kaplamayı sürdürmesi.
//   thread (iş parçacığı)    → programın içinde aynı anda yürüyen ayrı bir iş kolu.
//   enum / match             → "şunlardan biri" türü / hangisi olduğuna bakıp ona göre davranmak.
//   trait                    → bir türün yapabildiği işlerin listesi (Deref ve Drop birer trait).

use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

mod derinlesme;

fn main() {
    box_ornegi();
    deref_ornegi();
    drop_ornegi();
    rc_ornegi();
    refcell_ornegi();
    derinlesme::calistir();
}

// ============================================================================
// 1. Box<T> — değeri HEAP'te tutan en basit akıllı işaretçi
// ============================================================================
// En yaygın kullanımı: boyutu derleme anında bilinemeyen ÖZYİNELİ tipler. Bir enum kendini
// doğrudan içerirse boyut sonsuz olur (E0072). Box ARAYA bir pointer koyar: pointer'ın boyutu
// sabittir (bir adres), döngü kırılır.
// SADE ANLATIM: Büyük bir maket yaptın ve sırana sığmıyor. Onu okulun deposuna koyarsın, sırana
// da yalnız "3. raf, 2. göz" yazan küçük bir kâğıt bırakırsın. Maket ne kadar büyük olursa olsun
// kâğıt hep aynı boydadır. Rust'ta `Box::new(x)` de böyledir: x depoya (heap) gider, elinde sabit
// boylu bir adres kalır. Kâğıdın sahibi gidince depodaki maket de kendiliğinden kaldırılır.
// Neden böyle? "İçinde yine kendinden olan" türlerin (liste, ağaç) boyu baştan bilinemez; araya
// sabit boylu adres kâğıdı girmeseydi derleyici "bu kaç yer kaplar?" sorusunu cevaplayamazdı.
//   Mini örnek:
//     let b = Box::new(5);                           // 5 depoda, b elimizdeki adres kâğıdı
//     println!("{}", *b + 1);                        // ✓ 6 — `*b`: adrese git, değeri al
//     enum Zincir { Halka(i32, Box<Zincir>), Son }   // ✓ araya Box girdi, boyu belli
//     enum Kotu { Halka(i32, Kotu), Son }            // ✗ derlenmez: boyu sonsuz olurdu
#[derive(Debug)]
enum Liste {
    Dugum(i32, Box<Liste>), // Box olmadan: "recursive type has infinite size"
    Son,
}
// Değerleri match ile GERÇEKTEN okuyan yürüyüş: hem toplamı verir hem de derive(Debug)
// dead_code tuzağını önler (Böl. 5/8: Debug ile basmak alanları "okundu" saymaz).
// (Sade: dead_code = derleyicinin "bunu yazmışsın ama hiç kullanmıyorsun" uyarısı demek.)
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
// SADE ANLATIM: Elindeki adres kâğıdına bakarak maketi boyayamazsın; önce depoya gidip maketin
// yanına varman gerekir. `*` işareti bu yürüyüştür: "kâğıtta yazan yere git, oradakini ver".
// Deref, kendi yaptığın kutuya bu yürüyüşü öğretmenin yoludur. Derleyici çoğu zaman yolu senin
// yerine yürür: `&str` isteyen yere `&Kutum<String>` verirsen kutuları sırayla açıp yazıya ulaşır.
// Neden böyle? Bu kolaylık olmasaydı her kutu için açma adımlarını tek tek elle yazman gerekirdi.
//   Mini örnek:
//     let k = Box::new(String::from("Ada"));
//     let uzunluk = k.len();   // ✓ k bir kutu ama içindeki String'in metodu doğrudan çağrılıyor
//     selamla(&k);             // ✓ &Box<String> → &String → &str; derleyici kutuları kendi açar
//     let x = Kutum::yeni(5);
//     let y = *x + 1;          // ✓ Kutum için Deref yazıldığından; yazılmasaydı derlenmezdi
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
// SADE ANLATIM: Resim dersinden çıkarken fırçanı yıkayıp yerine koyarsın. Rust her değer için
// toplama işini kendiliğinden yapar; Drop ile "çıkarken ŞUNU da yap" diye kendi işini eklersin.
// Toplama sırası çıkarma sırasının tersidir: en son çıkardığın malzeme ilk önce kaldırılır.
// Dersin bitmesini beklemeden toplamak istersen `drop(x)` yazarsın.
// Neden böyle? Temizlik sana bırakılsaydı unutabilirdin (açık kalan dosya gibi) ya da aynı şeyi
// iki kez toplamaya kalkardın; `x.drop()` yazmanın yasak olması bu ikinci hatayı önler.
//   Mini örnek:
//     let a = Kaynak { ad: String::from("A") };
//     let b = Kaynak { ad: String::from("B") };
//     drop(b);          // ✓ B şimdi toplandı, ekrana "Drop: 'B' ..." yazılır
//     // b.drop();      // ✗ derlenmez: temizlik metodunu elle çağırmak yasak
//     // fonksiyonun sonu: A kendiliğinden toplanır
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
// SADE ANLATIM: Sınıfın tek bir topu var ve onu birkaç takım birlikte kullanıyor. Tahtada bir
// sayı yazar: "topu kullanan takım sayısı". Katılan takım sayıyı bir artırır, ayrılan bir
// azaltır; sayı 0 olunca top dolaba kaldırılır. Rust'ta Rc de böyledir: `Rc::clone` yeni bir top
// YAPMAZ, yalnız sayacı artırır. Herkes topa bakabilir, ama Rc tek başına onu değiştirtmez.
// Neden böyle? Sayaç olmasaydı topun ne zaman kaldırılacağını kimse bilemezdi: erken kaldırılırsa
// oynayan takım topsuz kalır, hiç kaldırılmazsa top sonsuza dek ortada durur.
// Dikkat: Rc yalnız TEK thread içindir; birden çok thread için aynı işi Arc yapar.
//   Mini örnek:
//     let top = Rc::new(String::from("top"));
//     let takim = Rc::clone(&top);                // sayaç 2 — aynı top, kopya yok
//     println!("{}", Rc::strong_count(&top));     // ✓ 2
//     drop(takim);                                // sayaç yeniden 1
//     // top.push('!');                           // ✗ derlenmez: Rc içindekini değiştirtmez
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
// SADE ANLATIM: Sınıf defterine herkes aynı anda bakabilir, ama yazarken tek kişi olmalıdır. Bu
// kuralı normalde derleyici, daha ders başlamadan denetler. RefCell'de ise kapıda bir nöbetçi
// durur ve kuralı ders SIRASINDA denetler: `borrow()` "bakacağım", `borrow_mut()` "yazacağım"
// demektir. Biri yazarken ikinci kişi de yazmak isterse nöbetçi dersi durdurur: program panikler.
// Kural kalkmaz; yalnız denetimin ZAMANI değişir.
// Neden böyle? Derleyici temkinlidir, bazı doğru programları da geri çevirir. RefCell o
// programları yazmana izin verir; bedeli, hatanın derlerken değil çalışırken ortaya çıkmasıdır.
//   Mini örnek:
//     let defter = RefCell::new(0);        // `mut` yok, yine de içi değişecek
//     *defter.borrow_mut() += 1;           // ✓ yazdı, satır bitince kalemi bıraktı
//     println!("{}", defter.borrow());     // ✓ 1
//     let _yazan1 = defter.borrow_mut();   // kalem hâlâ _yazan1'de
//     let _yazan2 = defter.borrow_mut();   // ✗ derlenir ama çalışırken panikler: iki yazan olmaz
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
    // ↳ DÜZELTME NOTU (son gözden geçirme): güncel std'de panik mesajı "RefCell already borrowed"dur
    //   (eski sürümlerde "already borrowed: BorrowMutError" idi); kaynak: core/src/cell.rs,
    //   panic_already_borrowed.
    // Kural hâlâ "ya bir &mut ya çok &" — ama RefCell bunu derlemede DEĞİL çalışma anında
    // denetler. Neden? Aşağıda DERLEYİCİ PERSPEKTİFİ.
    println!("  (RefCell borrow kuralı çalışma anında denetlenir)");
}

// SADE ANLATIM (aşağıdaki bölüm için): Yukarıdaki beş aracın her biri güvenliği başka bir yerde
// sağlar: Box ve Deref daha derlerken, Rc bir sayaçla, RefCell kapıdaki nöbetçiyle. Aşağıdaki
// bölüm bunların standart kütüphanede nasıl kurulduğunu anlatır. Sonunda bir de tuzak var: iki
// veri birbirini Rc ile tutarsa sayaç hiç 0 olmaz ve ikisi de silinemez; çaresi, yerini bilen ama
// sayaca yazılmayan Weak'tir. İlk okuyuşta bu bölümü atlayabilirsin; döngü ve Weak konusu
// derinlesme.rs'te örnekle yeniden anlatılıyor.
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
