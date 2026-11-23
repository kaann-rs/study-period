// Sağlamlaştırma — İleri konu: DEREF ve METOT ÇÖZÜMÜ (autoref/autoderef) — "." operatörünün sihri
// Çalıştır: cargo run -p ileri-deref-metot
//
// Kaynak: Rust for Rustaceans Böl. 4 ("Deref and Inherent Methods" kutusu) + The Rust Reference
// (method-call expression) + std (core::ops::Deref). Böl. 5 (alıcılar) ve Böl. 15 (Deref) burada
// birleşiyor: `x.metot()` yazınca derleyici GERÇEKTE ne yapar?
// ↳ DÜZELTME NOTU (son gözden geçirme): kitabın 4. bölümü Error Handling'dir; "Deref and
//   Inherent Methods" kutusu Böl. 3'tedir (Designing Interfaces → Wrapper Types). Bölüm listesi
//   yayıncının içindekiler sayfasıyla doğrulandı; kutu orada ayrıca listelenmiyor.
//
// Doğrulanmış (rustc ile): Box<Vec>.len() autoderef'le çalışır; immutable `v.push()` → E0596
// (autoref &mut gerektirir); inherent metot, Deref-hedefindeki aynı adlı metodu GÖLGELER.

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Bu dosyadaki her bölümün üstünde "SADE ANLATIM" diye bir kutu var. Önce onu oku, sonra
// altındaki teknik açıklamaya geç. Teknik kısım zor gelirse sorun değil: sade kutu ana fikri verir.
//
// Bu dosyayı okumadan önce bilmen gerekenler: `learn/ownership` (`&` ve `&mut`),
// `learn/structs` (metot ve `self`), `learn/generics-traits-lifetimes` (trait) ve
// `learn/smart-pointers` (Box ve Deref'in ilk tanıtımı).
//
// Bu bölümün tek cümlelik özeti: `x.metot()` yazdığında derleyici, metodu bulana kadar
// sarmalları senin yerine açar ve gereken `&` ya da `&mut` işaretini senin yerine koyar.
//
// MİNİ SÖZLÜK (bu dosyada geçen kelimeler):
//   sarmal (wrapper)     → başka bir değeri içinde taşıyan tür: Box<T>, Rc<T>, buradaki Kutu<T>.
//   Deref                → "içimdeki değere şöyle ulaşılır" diyen trait. `*x` onu kullanır.
//   Target               → Deref'in gösterdiği iç tür ("açınca ne çıkar?").
//   deref coercion       → derleyicinin `&Sarmal`'ı kendiliğinden `&İçindeki`'ne çevirmesi.
//   coercion             → derleyicinin bir türü sormadan, güvenle başka bir türe çevirmesi.
//   alıcı (receiver)     → noktanın solundaki değer: `x.metot()` içindeki x.
//   autoderef            → metodu ararken derleyicinin sarmalları sırayla açması.
//   autoref              → metodun istediği `&` ya da `&mut` işaretini derleyicinin eklemesi.
//   inherent metot       → türün kendi `impl Tür { ... }` bloğunda yazılı, trait'ten gelmeyen
//                          metot.
//   gölgeleme            → aynı adlı iki şeyden yakındakinin uzaktakini örtmesi.
//   tam-nitelikli çağrı  → metodun hangi türe ait olduğunu açıkça yazmak: `Ic::ad(&s)`.
//   unsize               → boyu belli diziyi (`[T; 3]`) boyu sonradan belli dilime (`[T]`)
//                          çeviren adım.
//   slice (dilim)        → art arda dizilmiş elemanlar; Vec'in içindeki verinin görünümü.
//   static fonksiyon     → `self` almayan, `Tür::ad(...)` diye çağrılan fonksiyon.
//   E0596                → "değiştirilebilir ödünç alınamıyor" hatasının numarası.

use std::ops::Deref;

fn main() {
    deref_ve_coercion();
    autoderef_autoref();
    inherent_golgeleme();
}

// ============================================================================
// 1. Deref ve DEREF COERCION
// ============================================================================
// SADE ANLATIM: Telefonuna kılıf taktın. Aramak, yazmak, fotoğraf çekmek için kılıfı her
// seferinde çıkarmazsın; kılıf takılıyken de telefonu kullanırsın. `Deref` bir sarmal türe bu
// özelliği verir: "içimdeki şey budur, bana gelen isteği ona ilet". Derleyici de `&Kutu<String>`
// gördüğü yerde `&str` gerekiyorsa kılıfı senin yerine yok sayar (deref coercion).
// Bu neden var? Olmasaydı Box, Rc, String gibi her sarmalı her kullanımda elle açman gerekirdi.
//   Mini örnek:
//     let k = Kutu(String::from("Ada"));
//     selamla(&k);              // ✓ &Kutu<String> → &String → &str: derleyici kendi açar
//     println!("{}", k.len());  // ✓ String'in metodu kılıfın içinden çağrılır
//     let x = Kutu(5);
//     let y = *x + 1;           // ✓ * ile içindeki 5'e elle ulaşırsın
//     // selamla(k);            // ✗ derlenmez: otomatik açma yalnız referansta (&k) çalışır
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
// SADE ANLATIM: Yardımsever bir kütüphane görevlisi düşün. "Şu kitabı istiyorum" dersin; önce
// ön rafa bakar, yoksa arkadaki rafa, o da olmazsa depoya iner. Bulduğu ilk yerde durur
// (autoderef). Kitabı uzatırken de ne yapacağına bakar: yalnız okuyacaksan açıp gösterir (`&`),
// üstüne not alacaksan kalemle birlikte verir (`&mut`). Bunu da sen söylemezsin (autoref).
// Bu neden var? Nokta (`.`) yazmak yetsin, `(&mut *kutu).metot()` gibi işaret yığınları
// yazmak zorunda kalma diye. Kurallar yine geçerlidir: kalem istiyorsan değer `mut` olmalıdır.
//   Mini örnek:
//     let kutulu = Box::new(vec![1, 2, 3]);
//     kutulu.len();             // ✓ Box'ta len yok → içine in → Vec'te var, orada dur
//     kutulu.first();           // ✓ Vec'te first yok → bir kat daha in → dilimde bulundu
//     let mut v = vec![1];
//     v.push(2);                // ✓ derleyici bunu (&mut v).push(2) diye okur
//     let w = vec![1];
//     // w.push(2);             // ✗ derlenmez: w `mut` değil, &mut alınamaz
fn autoderef_autoref() {
    // AUTODEREF: Box<Vec<i32>> üzerinde len() → Box'ı deref et (Vec), Vec'i deref et (slice),
    // slice::len bulundu. Üç seviye zinciri derleyici kendi yürür.
    // ↳ DÜZELTME NOTU (son gözden geçirme): `len` için zincir Vec'te DURUR — Vec'in kendi (inherent)
    //   `len` metodu vardır, slice'a inilmez (Box → Vec, iki seviye). Slice'a gerçekten inen çağrılar,
    //   yalnız `[T]`de tanımlı metotlardır: `kutulu.first()`, `kutulu.iter()`, `kutulu.contains(&1)`.
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
// (Sade: "inherent metot" = türün kendi impl bloğunda yazılı, trait'ten gelmeyen metot demek.)
// SADE ANLATIM: Sınıfta iki Ali var; biri hemen yanında, öteki arka sırada oturuyor. "Ali!"
// diye seslenirsen yanındaki döner; arkadakine sesin ulaşmaz bile. Derleyici de böyle davranır:
// `s.ad()` yazınca önce `s`'nin kendi türüne bakar; orada `ad` varsa onu çağırır ve içteki
// türe hiç inmez. İçtekini istiyorsan onu açıkça göstermen gerekir. Bu neden önemli? Bir gün
// sarmala aynı adlı bir metot eklenirse, eskiden içteki türe giden çağrı sessizce yön
// değiştirir; derleyici de uyarmaz.
//   Mini örnek:
//     let s = Sarmal(Ic);
//     s.ad();        // "SARMAL" → önce Sarmal'ın kendi metoduna bakılır, bulunur, durulur
//     (*s).ad();     // "IC"     → elle içine indin, Ic'in metodu çağrılır
//     Ic::ad(&s);    // "IC"     → "Ic'teki ad" diye açıkça söyledin
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

// SADE ANLATIM (aşağıdaki bölüm için): Yukarıda noktanın (`.`) senin yerine üç iş yaptığını
// gördün: sarmalı açar, `&`/`&mut` ekler, yakındaki metodu seçer. Aşağıdaki bölüm bu üç işin
// resmî kurallarını sırasıyla yazar: derleyici adayları hangi sırayla dener, hangisi önce
// gelir ve Deref'in neden "kalıtım" olmadığı. Kural kitabının ilgili sayfası gibidir. İlk
// okuyuşta atlayabilirsin; yukarıdaki üç bölümü anlamış olman yeter.
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
