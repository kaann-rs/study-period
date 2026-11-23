// Açıklama: learn/advanced-features/ACIKLAMA.md
// Rust Book — Bölüm 20: Advanced Features (kitabın son kavramsal bölümü)
// Çalıştır: cargo run -p advanced-features
//
// Beş başlık: unsafe Rust, gelişmiş trait'ler, gelişmiş tipler, fonksiyon pointer'ları + closure
// döndürme, ve makrolar. Ortak tema: güçlü araçlar + bunları GÜVENLİ soyutlamalara sarmak.
//
// Doğrulanmış (rustc ile): unsafe fonksiyon/raw pointer deref'i `unsafe` blok dışında kullanmak
// → E0133. Ayrıntı: DERLEYİCİ PERSPEKTİFİ (altta).

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Her bölümün "SADE ANLATIM" kutusunu önce oku; teknik kısım zorsa ana fikir orada.
// Benzetmeler bir ATÖLYE ve içindeki aletler çevresinde geçer.
//
// Bu bölümün tek cümlelik özeti: Rust'ın seyrek kullanılan güçlü aletleri vardır; doğru
// kullanımı, tehlikeli kısmı küçük tutup dışarıya güvenli bir sap bırakmaktır.
//
// MİNİ SÖZLÜK (bu dosyada geçen kelimeler):
//   unsafe                   → "buranın doğruluğunu derleyici değil, ben garanti ediyorum" bloğu.
//   raw pointer (`*const T`, `*mut T`) → hiçbir güvencesi olmayan çıplak adres.
//   dereference (`*r`)       → adrese gidip orada durana bakmak.
//   null / dangling          → hiçbir yeri göstermeyen / silinmiş bir yeri gösteren adres.
//   UB (tanımsız davranış)   → kural çiğnenince programın ne yapacağının artık bilinmemesi.
//   soyutlama (abstraction)  → karmaşık içi saklayıp dışarıya basit bir kullanım sunmak.
//   borrow checker           → ödünç kurallarını derlerken denetleyen derleyici parçası.
//   assert!                  → "şu doğru olmalı; değilse programı durdur" denetimi.
//   operatör aşırı yükleme   → `+` gibi işaretlere kendi türün için ne yapacağını öğretmek.
//   generic tip parametresi  → sonradan doldurulan tür boşluğu; varsayılanı da olabilir.
//   associated type          → bir trait'in içinde "sonucun türü şudur" diye adı konan tür.
//   newtype                  → var olan bir türü tek alanlı kendi türünle sarmak.
//   orphan rule              → "trait de tür de başkasınınsa ikisini sen bağlayamazsın" kuralı.
//   Display                  → bir değerin `{}` ile nasıl yazdırılacağını söyleyen trait.
//   tip takma adı (type alias) → var olan türe verilen ikinci ad; yeni tür oluşturmaz.
//   fonksiyon pointer'ı (`fn`) → bir fonksiyonu değer gibi taşımak, başka fonksiyona vermek.
//   closure, Fn/FnMut/FnOnce → yerinde yazılan adsız fonksiyon / onun üç çalışma biçimi.
//   `impl Fn`, `Box<dyn Fn>` → closure döndürmenin iki yolu: türü derlerken belli / kutulu.
//   makro, `macro_rules!`    → kod yazan kod / onu desenlerle tanımlamanın yolu.
//   metaprogramlama          → program yazan program yazmak.
//   genişleme (expand)       → makro çağrısının derlemeden önce gerçek koda çevrilmesi.
//   declarative / procedural → desenle tanımlanan makro / kod parçalarını işleyen fonksiyon makro.
//   TokenStream              → kaynak kodun sözcük parçalarına ayrılmış hâli.
//   never type (`!`)         → geri dönmeyen ifadelerin türü (panic!, return gibi).
//   DST ve Sized             → boyu derlerken bilinmeyen tür / boyu bilinen türlerin etiketi.
//   static, union, DSL       → program boyunca yaşayan değişken / alanları üst üste duran tür /
//                              belli bir iş için tasarlanmış küçük dil.

use std::fmt;
use std::ops::Add;
use std::slice;

// Makroyu KULLANILMADAN ÖNCE tanımla (ya da #[macro_use]); en üste koyuyoruz.
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
// SADE ANLATIM: Atölyedeki makinelerin üstünde koruyucu kapak vardır; kapak kapalıyken elini
// kesemezsin. Bazı ince işler için usta kapağı açar. Kapak açıkken makine seni korumaz, dikkat
// etmek ustanın işidir. `unsafe` bloğu o açık kapaktır: "buranın doğru olduğunu derleyici
// göremiyor, ben garanti ediyorum". İyi usta kapağı kısa süre açar ve işi bitince kapatır;
// dışarıdan kullanan kişi (`ikiye_bol`u çağıran) kapağın açıldığını bilmek zorunda kalmaz.
// Derleyici "aynı diziden iki &mut" görünce çakışmasalar bile geri çevirir; unsafe birkaç işlemi açar.
//   Mini örnek:
//     let x = 42;
//     let r = &x as *const i32;         // ✓ çıplak adres OLUŞTURMAK serbest
//     // println!("{}", *r);            // ✗ derlenmez: çıplak adresin gösterdiğine bakmak unsafe
//     unsafe { println!("{}", *r); }    // ✓ kapağı açtık: "adres geçerli, sorumluluk bende"
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
// SADE ANLATIM: Terazi elma tartmayı bilir; kendi yaptığın bir şeyi tartabilmesi için ona nasıl
// tartılacağını öğretmen gerekir. `+` işareti de sayıları toplamayı bilir; `Metre` gibi kendi
// türün için ne yapacağını `impl Add for Metre` yazarak sen öğretirsin. Ondan sonra
// `Metre(3) + Metre(4)` yazmak, aslında senin `add` fonksiyonunu çağırmaktır.
// İşaretler trait'tir; `+` yalnız izin verdiğin türlerde çalışır (Metre + düz sayı olmaz).
//   Mini örnek:
//     let t = Metre(3) + Metre(4);      // ✓ Metre(7) — `+`, yazdığımız add'i çağırır
//     // let k = Metre(3) + 4;          // ✗ derlenmez: Metre ile düz sayıyı toplamayı öğretmedik
//     // let f = Metre(3) - Metre(1);   // ✗ derlenmez: çıkarmayı (Sub) hiç öğretmedik
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
// SADE ANLATIM: Atölyedeki ortak aletlere kendi etiketini yapıştıramazsın: alet de etiket
// makinesi de başkasınındır. Ama aleti KENDİ kutuna koyabilir, etiketi kutuna yapıştırabilirsin.
// Rust'ta da `Vec` ve `Display` ikisi birden başkasının (std'nin) olduğu için onları doğrudan
// bağlayamazsın. `Vec`'i tek alanlı kendi türünle (`Sarmal`) sararsın; tür artık senin olduğu
// için `Display` yazabilirsin. Bu sarmaya newtype denir.
// Yoksa iki kütüphanenin impl'ı çakışır, hangisinin geçerli olduğu bilinemezdi.
//   Mini örnek:
//     // impl fmt::Display for Vec<String> { ... }   // ✗ derlenmez: ne Display ne Vec bizim
//     struct Sarmal(Vec<String>);                    // ✓ kendi kutumuz
//     impl fmt::Display for Sarmal { ... }           // ✓ kutu bizim olduğu için serbest
//     let w = Sarmal(vec![String::from("a")]);
//     println!("{w} {}", w.0.len());                 // ✓ içteki Vec'e `.0` ile ulaşılır
struct Sarmal(Vec<String>);
impl fmt::Display for Sarmal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]", self.0.join(", "))
    }
}
fn newtype_display() {
    let w = Sarmal(vec![
        String::from("a"),
        String::from("b"),
        String::from("c"),
    ]);
    println!("newtype Display: {w}");
}

// ============================================================================
// 4. GELİŞMİŞ TİPLER + FONKSİYON POINTER'LARI + CLOSURE DÖNDÜRME
// ============================================================================
// SADE ANLATIM: Bu bölümde üç küçük alet var. Takma ad: uzun adlı bir alete kısa bir lakap
// takmak; alet aynı alettir. Fonksiyon pointer'ı: bir tarif kartını başkasının eline vermek;
// "şu sayıya ŞU işlemi uygula" derken işlemin kendisini de verirsin. Closure döndürmek: ayarı
// önceden yapılmış bir alet üretmek; `ekleyici_impl(100)` içinde 100'ü saklayan yeni bir
// fonksiyon verir.
// Neden iki dönüş yolu? Her closure'ın türü ayrı ve adsızdır: `impl Fn` "tek tür, adını sen bilme"
// der (hızlı); `Box<dyn Fn>` farklı closure'ları aynı kutuda döndürür, bedeli heap'te bir kutu.
//   Mini örnek:
//     let s: Metin = String::from("a");         // ✓ Metin yalnız lakap; String ile aynı tür
//     let yirmi = uygula(carp_iki, 10);         // ✓ 20 — fonksiyonun adı değer gibi verildi
//     let yuz_ekle = ekleyici_impl(100);        // ✓ içinde 100 saklı yeni bir fonksiyon
//     println!("{}", yuz_ekle(5));              // ✓ 105
//     // uygula(|x| x + yirmi, 1);              // ✗ derlenmez: dışarıdan değişken yakalayan
//                                               //   closure, `fn` pointer'ına dönüşmez
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

    println!(
        "fn pointer: uygula(carp_iki, 10) = {}",
        uygula(carp_iki, 10)
    );
    let f1 = ekleyici_impl(100);
    let f2 = ekleyici_kutulu(1);
    println!("impl Fn closure döndürme: {}", f1(5)); // 105
    println!("Box<dyn Fn> closure döndürme: {}", f2(5)); // 6
}

// ============================================================================
// 5. MAKROLAR — kod üreten kod (metaprogramlama)
// ============================================================================
// SADE ANLATIM: Lastik bir damga düşün: bir kez hazırlarsın, sonra her bastığında aynı yazıyı
// senin yerine yazar. Makro, kod yazan bir damgadır. `benim_vec![10, 20, 30]` yazdığında
// derleyici, programı derlemeden ÖNCE bu satırı "boş bir Vec aç, 10'u ekle, 20'yi ekle, 30'u
// ekle" satırlarına çevirir. Adının sonundaki `!` "bu bir fonksiyon değil, makro" demektir.
// Neden fonksiyon yetmez? Bir fonksiyonun kaç argüman alacağı baştan bellidir; makro istediğin
// kadar argüman alabilir, çünkü çalışan bir şey değil, kod üreten bir kalıptır.
//   Mini örnek:
//     let v = benim_vec![10, 20, 30];
//     // derleyici yukarıdaki satırı önce şuna çevirir:
//     // { let mut v = Vec::new(); v.push(10); v.push(20); v.push(30); v }
//     let bos: Vec<i32> = benim_vec![];     // ✓ sıfır argüman da olur
fn makro() {
    // benim_vec! derleme ANINDA genişler: aşağıdaki çağrı tam da elle yazılmış push'lara döner.
    let v: Vec<i32> = benim_vec![10, 20, 30];
    println!("macro_rules! benim_vec![10,20,30] = {v:?}");
}

// SADE ANLATIM (aşağıdaki bölüm için): Aşağıdaki bölüm bu aletlerin sınırlarını derleyicinin
// gözünden anlatır: unsafe'in tam olarak hangi beş işlemi açtığını (ve neleri AÇMADIĞINI),
// newtype'ın neden bedava olduğunu, "geri dönmeyen" ifadelerin türünü ve makroların iki ailesini.
// Aklında kalması gereken tek cümle: unsafe ödünç kurallarını kapatmaz; yalnız birkaç tehlikeli
// işlem için "sorumluluk bende" dedirtir. İlk okuyuşta bu bölümü atlayabilirsin.
// ============================================================================
// DERLEYİCİ PERSPEKTİFİ — güç + güvenlik dengesi
// ============================================================================
//
// • UNSAFE'İN 5 SÜPER GÜCÜ: (1) raw pointer dereference, (2) unsafe fonksiyon/metot çağırma,
//   (3) değiştirilebilir static erişimi/değiştirmesi, (4) unsafe trait uygulama, (5) union alanı okuma.
//   unsafe borrow checker'ı KAPATMAZ, yalnız bu beş işlemi AÇAR; geri kalan her şey normal kurallara tabidir.
//   Derleyici bunları `unsafe` olmadan reddeder (E0133: "requires unsafe block"), çünkü doğruluklarını
//   (null değil mi, çakışma yok mu) ancak insan garanti edebilir; ihlal = UB (null/dangling/hizasız
//   pointer, aliasing ihlali → veri yarışı). Denetim: rustc_mir_build/src/check_unsafety.rs. Doğru
//   kullanım: unsafe'i KÜÇÜK tut ve GÜVENLİ API arkasına sar (ikiye_bol gibi).
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
