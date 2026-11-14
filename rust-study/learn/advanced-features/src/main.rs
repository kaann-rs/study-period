// Rust Book — Bölüm 20: Advanced Features (kitabın son kavramsal bölümü)
// Çalıştır: cargo run -p advanced-features
//
// Beş başlık: unsafe Rust, gelişmiş trait'ler, gelişmiş tipler, fonksiyon pointer'ları + closure
// döndürme, ve makrolar. Ortak tema: güçlü araçlar + bunları GÜVENLİ soyutlamalara sarmak.
//
// Doğrulanmış (rustc ile): unsafe fonksiyon/raw pointer deref'i `unsafe` blok dışında kullanmak
// → E0133. Ayrıntı: DERLEYİCİ PERSPEKTİFİ (altta).

use std::fmt;
use std::ops::Add;
use std::slice;

// Bir makroyu KULLANILMADAN ÖNCE tanımlamak gerekir (ya da #[macro_use]); en üste koyuyoruz.
macro_rules! benim_vec {
    // $( ... ),*  = virgülle ayrılmış SIFIR veya daha fazla tekrar. $x:expr bir ifade yakalar.
    ( $( $x:expr ),* ) => {{
        let mut v = Vec::new();
        $( v.push($x); )* // her yakalanan $x için bir push üret
        v
    }};
}

fn main() {
    unsafe_rust();
    operator_overload();
    newtype_display();
    tip_takma_ve_fn();
    makro();
}

// ============================================================================
// 1. UNSAFE RUST — derleyicinin garanti edemediği yerde SEN garanti et
// ============================================================================
// Güvenli abstraction: dışarıya tamamen güvenli bir API; unsafe yalnız İÇERİDE, küçük ve
// doğruluğu insan tarafından kanıtlanmış bir blokta. std'nin çoğu (Vec, split_at_mut...) böyledir.
fn ikiye_bol(dilim: &mut [i32], orta: usize) -> (&mut [i32], &mut [i32]) {
    let uzunluk = dilim.len();
    let isaretci = dilim.as_mut_ptr(); // raw pointer
    assert!(orta <= uzunluk); // güvenlik koşulunu BİZ garantiliyoruz
    // Borrow checker "aynı dilimden iki &mut" veremez; ama ikisi ÇAKIŞMAYAN bölgeler. Bunu
    // derleyici göremez, biz biliyoruz → unsafe ile iki ayrı &mut dilim kuruyoruz.
    unsafe {
        (
            slice::from_raw_parts_mut(isaretci, orta),
            slice::from_raw_parts_mut(isaretci.add(orta), uzunluk - orta),
        )
    }
}
fn unsafe_rust() {
    let mut sayilar = [1, 2, 3, 4, 5, 6];
    let (sol, sag) = ikiye_bol(&mut sayilar, 3);
    println!("ikiye_bol: sol={sol:?} sag={sag:?}");

    // Raw pointer: oluşturmak güvenli, DEREFERENCE etmek unsafe (null/dangling olabilir).
    let x = 42;
    let r = &x as *const i32;
    unsafe {
        println!("raw pointer deref (unsafe): *r = {}", *r);
    }
    //   *r;  // ✗ unsafe bloksuz → E0133 "dereference of raw pointer is unsafe"
}

// ============================================================================
// 2. GELİŞMİŞ TRAIT'LER — operatör aşırı yükleme (default generic type param)
// ============================================================================
#[derive(Debug, PartialEq)]
struct Metre(u32);
// Add trait'i:  trait Add<Rhs = Self> { type Output; fn add(self, rhs: Rhs) -> Output; }
// Rhs = Self VARSAYILAN generic tip parametresidir → çoğu zaman yazmana gerek kalmaz.
impl Add for Metre {
    type Output = Metre; // associated type: çıktının türü
    fn add(self, baska: Metre) -> Metre {
        Metre(self.0 + baska.0)
    }
}
fn operator_overload() {
    let toplam = Metre(3) + Metre(4); // artık `+` Metre'de çalışır
    println!("operator overload: {toplam:?}");
    assert_eq!(toplam, Metre(7));
}

// ============================================================================
// 3. NEWTYPE ile dış trait'i dış tipe uygula (orphan rule aşımı)
// ============================================================================
// Daha önce konuştuğumuz orphan rule: ne trait (Display) ne tip (Vec) bizimse impl yazamayız.
// Çözüm: Vec'i ince bir newtype ile SAR — artık tip (Sarmal) bizim, impl serbest.
struct Sarmal(Vec<String>);
impl fmt::Display for Sarmal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]", self.0.join(", "))
    }
}
fn newtype_display() {
    let w = Sarmal(vec![String::from("a"), String::from("b"), String::from("c")]);
    println!("newtype Display: {w}");
}

// ============================================================================
// 4. GELİŞMİŞ TİPLER + FONKSİYON POINTER'LARI + CLOSURE DÖNDÜRME
// ============================================================================
type Metin = String; // tip TAKMA ADI (alias): yeni tip değil, uzun imzalar için kısaltma

// fn (küçük f): FONKSİYON POINTER'ı — bir fonksiyonu değer olarak al. Fn/FnMut/FnOnce'un üçünü
// de uygular, o yüzden closure bekleyen API'lere de geçer.
fn uygula(f: fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}
fn carp_iki(x: i32) -> i32 {
    x * 2
}

// CLOSURE DÖNDÜRME: closure'ların boyutu derleme anında bilinmez (her biri farklı tip).
//   impl Fn  → tek, somut (bilinmeyen) tür; en ucuz, ama tek tür döndürebilirsin.
//   Box<dyn Fn> → heap'te trait object; farklı closure'ları aynı imzadan döndürmek gerekince.
fn ekleyici_impl(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}
fn ekleyici_kutulu(n: i32) -> Box<dyn Fn(i32) -> i32> {
    Box::new(move |x| x + n)
}
fn tip_takma_ve_fn() {
    let s: Metin = String::from("alias"); // Metin == String
    println!("type alias: {s}");

    println!("fn pointer: uygula(carp_iki, 10) = {}", uygula(carp_iki, 10));
    let f1 = ekleyici_impl(100);
    let f2 = ekleyici_kutulu(1);
    println!("impl Fn closure döndürme: {}", f1(5)); // 105
    println!("Box<dyn Fn> closure döndürme: {}", f2(5)); // 6
}

// ============================================================================
// 5. MAKROLAR — kod üreten kod (metaprogramlama)
// ============================================================================
fn makro() {
    // benim_vec! derleme ANINDA genişler: aşağıdaki çağrı tam da elle yazılmış push'lara döner.
    let v: Vec<i32> = benim_vec![10, 20, 30];
    println!("macro_rules! benim_vec![10,20,30] = {v:?}");
}

// ============================================================================
// DERLEYİCİ PERSPEKTİFİ — güç + güvenlik dengesi
// ============================================================================
//
// • UNSAFE'İN 5 SÜPER GÜCÜ: (1) raw pointer dereference, (2) unsafe fonksiyon/metot çağırma,
//   (3) değiştirilebilir static (global) erişimi/değiştirmesi, (4) unsafe trait uygulama,
//   (5) union alanı okuma. BUNLAR DIŞINDAKİ her şey unsafe blokta da normal kurallara tabidir;
//   unsafe borrow checker'ı KAPATMAZ, yalnız bu beş işlemi AÇAR. Derleyici bunları `unsafe`
//   olmadan reddeder (E0133: "requires unsafe block"), çünkü doğruluklarını (null değil mi,
//   çakışma yok mu) ancak insan garanti edebilir; ihlal = tanımsız davranış (UB: raw pointer
//   null/dangling/hizasız olabilir, aliasing'i çiğneyip veri yarışı üretebilir — hepsi UB).
//   Denetim: rustc_mir_build/src/check_unsafety.rs. Doğru kullanım: unsafe'i KÜÇÜK tut ve
//   GÜVENLİ bir API'nin arkasına sar (ikiye_bol gibi) — güvensizlik tek yerde denetlenir.
//
// • DEFAULT GENERIC TYPE PARAM: Add<Rhs = Self> gibi. Varsayılan sayesinde `impl Add for Metre`
//   yazınca Rhs otomatik Metre olur; farklı tür toplamak istersen impl Add<Baska> for Metre.
//   Operatörler aslında trait'tir (+ → Add::add), bu yüzden kendi tiplerinde `+` tanımlayabilirsin.
//
// • NEWTYPE: tek alanlı tuple struct. İki işi var: (a) orphan rule'u aşmak (dış trait'i dış tipe,
//   sarmalayarak), (b) tip güvenliği (Metre ile Saniye karışmaz, ikisi de u32 olsa bile). Runtime
//   maliyeti yok — derleyici sarmalı optimize eder.
//
// • NEVER TYPE `!`: hiçbir değer DÖNDÜRMEYEN ifadelerin türü (panic!, loop{}, continue, return).
//   `!` her türe zorlanabilir (coerce), bu yüzden `let x = match b { true => 5, false => panic!() };`
//   derlenir — panic!'in `!` türü i32'ye uyar. DST (dynamically sized type): str, [T], dyn Trait
//   gibi boyutu derleme anında bilinmeyen tipler; bunlar hep bir pointer ARKASINDA (&str, Box<dyn>)
//   kullanılır. Varsayılan `T: Sized` kısıtı bu yüzden vardır; `T: ?Sized` ile gevşetilir.
//
// • MAKROLAR, kod üreten koddur ve fonksiyondan önce AÇILIR (expand). İki tür:
//     - Declarative (macro_rules!): desen → kod şablonu (benim_vec! gibi). Dilbilgisi üstünde
//       çalışır, değişken sayıda argüman alabilir (fonksiyon alamaz).
//     - Procedural: TokenStream alıp TokenStream üreten fonksiyonlar; AYRI bir crate'te
//       (proc-macro = true) yazılır. Üç alt tür: #[derive] (örn. derive(Debug)), öznitelik
//       (#[route(...)]), ve fonksiyon-benzeri (sql!(...)). Sorun: fonksiyon çalışma anında çağrılır
//       ve sabit imza ister; makro DERLEME anında kod üretir → derive gibi "tür başına kod yaz"
//       işleri ve değişken argümanlı DSL'ler ancak makroyla mümkün.
