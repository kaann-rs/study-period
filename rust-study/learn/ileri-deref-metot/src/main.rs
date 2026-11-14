// Sağlamlaştırma — İleri konu: DEREF ve METOT ÇÖZÜMÜ (autoref/autoderef) — "." operatörünün sihri
// Çalıştır: cargo run -p ileri-deref-metot
//
// Kaynak: Rust for Rustaceans Böl. 4 ("Deref and Inherent Methods" kutusu) + The Rust Reference
// (method-call expression) + std (core::ops::Deref). Böl. 5 (alıcılar) ve Böl. 15 (Deref) burada
// birleşiyor: `x.metot()` yazınca derleyici GERÇEKTE ne yapar?
//
// Doğrulanmış (rustc ile): Box<Vec>.len() autoderef'le çalışır; immutable `v.push()` → E0596
// (autoref &mut gerektirir); inherent metot, Deref-hedefindeki aynı adlı metodu GÖLGELER.

use std::ops::Deref;

fn main() {
    deref_ve_coercion();
    autoderef_autoref();
    inherent_golgeleme();
}

// ============================================================================
// 1. Deref ve DEREF COERCION
// ============================================================================
struct Kutu<T>(T);
impl<T> Deref for Kutu<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}
fn selamla(ad: &str) {
    println!("  selam, {ad}");
}
fn deref_ve_coercion() {
    let k = Kutu(5);
    // `*k` aslında `*(k.deref())` demektir: Deref::deref &T verir, * onu açar.
    println!("*Kutu(5) = {}", *k);

    // DEREF COERCION: fonksiyon çağrısı/metot alıcısı sırasında derleyici, tür uyması için
    // OTOMATİK deref zinciri uygular. &Kutu<String> -> &String -> &str:
    let isim = Kutu(String::from("Kaan"));
    selamla(&isim); // &Kutu<String> --deref--> &String --deref--> &str  (hepsi derleme anında)
    // Neden? Sorun olmasaydı her sarmalı elle açman gerekirdi: selamla(&(*isim)[..]) gibi.
    // Deref coercion sarmal türleri (Box, Rc, String...) "içindeki değermiş gibi" kullandırır.
}

// ============================================================================
// 2. AUTODEREF ve AUTOREF — `x.metot()` nasıl çözülür?
// ============================================================================
// METOT ÇAĞRI ALGORİTMASI (The Rust Reference): `alıcı.metot()` için derleyici bir ADAY ALICI
// TÜRLERİ listesi kurar:
//   1) Alıcının türünden başla, tekrar tekrar Deref uygulayarak zinciri çıkar:
//        T, <T as Deref>::Target, onun Target'ı, ... (ve en sonda dizi→slice unsize adımı)
//   2) Bu listedeki HER aday tür U için şu SIRAYLA metot ara: U, &U, &mut U (AUTOREF).
//   3) Metodu bulan İLK eşleşmeyi seç.
// Yani `x.metot()` perde arkasında gerektiği kadar `*`/`&`/`&mut` ekler — sen yazmazsın.
fn autoderef_autoref() {
    // AUTODEREF: Box<Vec<i32>> üzerinde len() → Box'ı deref et (Vec), Vec'i deref et (slice),
    // slice::len bulundu. Üç seviye zinciri derleyici kendi yürür.
    let kutulu = Box::new(vec![1, 2, 3]);
    println!("autoderef: Box<Vec<i32>>.len() = {}", kutulu.len()); // 3

    // AUTOREF: push(&mut self) ister. `v.push(4)` aslında `(&mut v).push(4)` — & otomatik eklenir.
    let mut v = vec![1, 2, 3];
    v.push(4);
    println!("autoref: v.push(4) == (&mut v).push(4) -> {v:?}");
    // Eğer v `mut` olmasaydı: autoref &mut almaya çalışır ama alamaz → E0596 "cannot borrow `v`
    // as mutable". Yani autoref sihir değil; borrow kurallarına yine tabi (DOĞRULANDI).

    // Aynı değer üzerinde & ve &mut alıcı otomatik seçilir:
    let s = String::from("merhaba");
    println!("autoref &self: s.len() = {}", s.len()); // len(&self) → &s otomatik
}

// ============================================================================
// 3. İNCE TUZAK — inherent metot, Deref-hedefini GÖLGELER (RfR "Deref and Inherent Methods")
// ============================================================================
struct Ic;
impl Ic {
    fn ad(&self) -> &str {
        "IC"
    }
}
struct Sarmal(Ic);
impl Deref for Sarmal {
    type Target = Ic;
    fn deref(&self) -> &Ic {
        &self.0
    }
}
impl Sarmal {
    fn ad(&self) -> &str {
        "SARMAL"
    } // Sarmal'ın KENDİ (inherent) metodu — Ic'te de ad() var
}
fn inherent_golgeleme() {
    let s = Sarmal(Ic);
    // Algoritma önce ADAYIN KENDİ türünü (Sarmal) dener; ad() orada BULUNUR → Deref'e hiç inmez.
    println!("s.ad()      = {} (inherent kazanır, Deref'e inilmez)", s.ad()); // SARMAL
    // İç tipin metoduna ulaşmak için ya elle deref ya tam-nitelikli çağrı:
    println!("(*s).ad()   = {} (elle deref → Ic)", (*s).ad()); // IC
    println!("Ic::ad(&s)  = {} (tam-nitelikli, &Sarmal coerce)", Ic::ad(&s)); // IC
    // Neden önemli? Sorun (RfR): bir gün Ic'e ya da Sarmal'a aynı adlı yeni bir metot eklenirse,
    // `s.ad()`'ın HANGİSİNİ çağırdığı sessizce değişebilir. Bu yüzden "şeffaf sarmal" türlerde
    // (Box/Rc gibi, kullanıcı tipine deref eden) inherent metottan kaçın; fn ad(s: &Sarmal)
    // biçiminde serbest/static fonksiyon tercih et ki belirsizlik olmasın.
}

// ============================================================================
// DERLEYİCİ / Reference / RfR PERSPEKTİFİ
// ============================================================================
//
// • Deref (core::ops::Deref): `*x` operatörünü belirler, DerefMut &mut sürümü. `*x` == `*x.deref()`.
//   Yalnız TEK bir Target tanımlanabilir (bir türün "içi" tek anlamlı olsun diye; RfR: "cannot
//   implement Deref against multiple Target").
//
// • DEREF COERCION: fonksiyon argümanı ve metot alıcısı konumunda, derleyici tür uydurmak için
//   &T -> &U (T: Deref<Target=U>) dönüşümünü ZİNCİRLEME, otomatik, DERLEME ANINDA uygular. Runtime
//   maliyeti yok (sadece pointer'ı "aynı yere" yeniden yorumlar). &String->&str, &Box<T>->&T, ...
//
// • METOT ÇÖZÜMÜ (The Rust Reference, "Method-call expressions"): `recv.m()` için derleyici bir
//   aday alıcı türleri listesi kurar — recv'in türünden başlayıp Deref zincirini izleyerek (gerekirse
//   son adımda [T;N]->[T] unsize) — ve her aday U için (U, &U, &mut U) sırasıyla uygun bir m arar.
//   İLK bulduğunu seçer. Bu "autoderef (zinciri in) + autoref (&/&mut ekle)" mekanizmasıdır; bu
//   yüzden `(&mut v).push()` yazmana gerek kalmaz. Ama autoref yine borrow kurallarına tabidir:
//   &mut alıcı için değer mut olmalı, yoksa E0596.
//
// • ÖNCELİK (RfR kutusu): her aday türde INHERENT metotlar, trait metotlarından ve bir sonraki
//   deref adımından ÖNCE gelir. Bu yüzden Sarmal::ad, Ic::ad'i gölgeler. Sonuç: aynı adlı metot
//   çakışması "sihir gibi" davranış/sürpriz üretebilir; şeffaf sarmal türlerde inherent metottan
//   kaçınmak (static fonksiyon tercih etmek) bu belirsizliği önler.
//
// • Deref "kalıtım DEĞİLDİR" (Böl. 18): yalnız metot çağrısı/coercion ergonomisi verir; alt-tip
//   ilişkisi kurmaz. Sarmal bir Ic DEĞİLDİR; sadece onun metotlarını "." ile erişilebilir kılar.
