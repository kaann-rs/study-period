// Sağlamlaştırma — İleri konu: COHERENCE ve ORPHAN RULE (tutarlılık ve yetim kuralı)
// Çalıştır: cargo run -p ileri-coherence
//
// Kaynak: Rust for Rustaceans Böl. 3 ("Coherence and the Orphan Rule") + std.
// ↳ DÜZELTME NOTU (son gözden geçirme): kitapta bu başlık Böl. 2'dedir (Types → Traits and
//   Trait Bounds). Yayıncının ayrıntılı içindekiler listesiyle doğrulandı.
// Böl. 10 (generics/traits) ve Böl. 20'de (newtype) değindiğimiz orphan rule'un TAM kuralı.
//
// Doğrulanmış (rustc ile): çakışan impl'ler → E0119 "conflicting implementations"; hem trait
// hem tip yabancıysa → E0117. Ayrıntı: DERLEYİCİ PERSPEKTİFİ (altta).

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Bu dosyadaki her bölümün üstünde "SADE ANLATIM" diye bir kutu var. Önce onu oku, sonra
// altındaki teknik açıklamaya geç. Teknik kısım zor gelirse sorun değil: sade kutu ana fikri verir.
//
// Bu dosyayı okumadan önce bilmen gerekenler: `learn/generics-traits-lifetimes` (trait ve
// `impl Trait for Tip` yazımı), `learn/modules` ile `learn/cargo-and-crates` (crate nedir) ve
// `learn/advanced-features` (newtype kalıbı).
//
// Bu bölümün tek cümlelik özeti: bir tür ile bir trait'in eşleşmesi bütün programda yalnız
// BİR kez yazılabilir; bunu sağlamak için de o eşleşmeyi ancak trait'in ya da türün sahibi
// yazabilir.
//
// MİNİ SÖZLÜK (bu dosyada geçen kelimeler):
//   crate                → tek başına derlenen Rust paketi. Senin projen bir crate'tir, std de.
//   std                  → Rust'ın hazır gelen standart kütüphanesi (Vec, String, Display...).
//   trait                → "şunu yapabilirim" sözü: bir türün sahip olabileceği yetenek listesi.
//   impl                 → o sözün bir tür için yerine getirildiği kod: `impl Trait for Tip`.
//   yerel (local)        → senin crate'inde tanımlanmış tür ya da trait.
//   yabancı (foreign)    → başka bir crate'te (ör. std'de) tanımlanmış tür ya da trait.
//   coherence (tutarlılık)→ "bir tür + bir trait için en çok bir impl" kuralı.
//   orphan rule          → yetim kuralı: impl yazmak için trait ya da tür senin olmalı.
//   blanket impl         → tek satırla bir tür AİLESİNE yazılan impl: `impl<T: X> Trait for T`.
//   generic (T)          → sonradan doldurulan tür boşluğu.
//   newtype              → yabancı türü kendi tek alanlı struct'ınla sarmak: `struct S(Vec<i32>)`.
//   covered impl         → yabancı trait + yabancı tür olduğu hâlde, trait'in parametresinde
//                          yerel bir tür geçtiği için izin verilen dar istisna.
//   fundamental tip      → `&`, `&mut`, `Box`: yetim kuralı bunların içine bakar, dışını saymaz.
//   upstream / downstream→ senin kullandığın crate / seni kullanan crate.
//   kırıcı değişiklik    → yapılınca başkalarının çalışan kodunu derlenmez hâle getiren değişiklik.
//   ayrı derleme         → her crate'in ötekilerden bağımsız derlenmesi.
//   E0119 / E0117        → "çakışan impl" / "trait de tür de yabancı" hatalarının numaraları.

use std::fmt::Display;

fn main() {
    blanket_impl();
    yerel_yabanci();
    covered_impl();
}

// ============================================================================
// 1. COHERENCE + BLANKET IMPL
// ============================================================================
// COHERENCE özelliği: herhangi bir (tip, trait) çifti için EN FAZLA BİR impl olabilir. Böylece
// `x.metot()` çağrısında derleyici hiçbir zaman "hangi impl?" diye kalmaz — tek doğru seçim vardır.
//
// BLANKET IMPL: bir trait'i bir TÜR ARALIĞINA uygulamak (impl<T: ...> Trait for T). std'nin
// `impl<T: Display> ToString for T`'si bunun klasiğidir. Yalnız trait'i TANIMLAYAN crate blanket
// yazabilir (yoksa iki crate çakışan blanket yazabilirdi).
// SADE ANLATIM: Okulda her öğrencinin her ders için TEK bir notu olur. İki ayrı öğretmen
// aynı öğrenciye aynı dersten iki ayrı not yazarsa karnede hangisinin duracağını kimse
// bilemez. Rust'ta öğrenci türdür, ders trait'tir, not da `impl`'dir: her (tür, trait) çifti
// için en çok bir impl olabilir. Buna coherence denir. Blanket impl ise müdürün tek cümlelik
// duyurusudur: "okuma bilen herkes kütüphane kartı aldı sayılır". Bu duyurudan sonra Ali'ye
// ayrıca bir kart yazamazsın; onun kartı zaten vardır.
//   Mini örnek:
//     trait Etiketle { fn etiket(&self) -> String; }
//     impl<T: Display> Etiketle for T { ... }   // yazdırılabilen HER türe tek seferde
//     42.etiket();                              // ✓ i32 yazdırılabilir, yeteneği kazandı
//     "metin".etiket();                         // ✓ &str de
//     // impl Etiketle for i32 { ... }          // ✗ derlenmez: i32 için ikinci impl olurdu
trait Etiketle {
    fn etiket(&self) -> String;
}

impl<T: Display> Etiketle for T {
    fn etiket(&self) -> String {
        format!("<{self}>")
    }
}

fn blanket_impl() {
    // Blanket sayesinde Display olan HER tür Etiketle kazandı — ek impl yazmadan:
    println!(
        "blanket: {} {} {}",
        42.etiket(),
        "metin".etiket(),
        true.etiket()
    );
    // ✗ Bu blanket varken AYRICA `impl Etiketle for i32` YAZAMAZSIN (i32: Display → çakışır):
    //     E0119 "conflicting implementations of trait `Etiketle` for type `i32`" (DOĞRULANDI).
    //   Coherence tam da bunu yasaklar: i32.etiket() için iki aday olamaz.
}

// ============================================================================
// 2. ORPHAN RULE — impl yazabilmek için trait YA DA tip YEREL olmalı
// ============================================================================
// Yetim kuralı (orphan rule): `impl Trait for Tip` ancak Trait YA DA Tip senin crate'inde
// TANIMLIYSA yazılabilir. İzin verilenler ve neden:
// SADE ANLATIM: Trait'i bir oyun, türü bir oyuncak say. Kural şudur: ya oyun senin olacak ya
// oyuncak. Kendi oyuncağını başkasının oyununa sokabilirsin; kendi oyununda başkasının
// oyuncağına rol verebilirsin. Ama oyun da oyuncak da başkasınınsa "bu oyuncak bu oyunda
// şöyle davranır" diye kural koyamazsın. Neden? Aynı kuralı bir başkası da kendi evinde
// farklı yazabilirdi; iki ev bir araya gelince hangi kuralın geçerli olduğu bilinemezdi.
// Çıkış yolu: başkasının oyuncağını kendi kutuna koymak (newtype). Kutu senindir.
//   Mini örnek:
//     struct Benim(i32);                        // benim oyuncağım (yerel tür)
//     impl Display for Benim { ... }            // ✓ oyun yabancı, oyuncak benim
//     impl Etiketle for bool { ... }            // ✓ oyun benim, oyuncak yabancı
//     //                                        //   (yukarıdaki blanket impl olmasaydı)
//     // impl Display for Vec<i32> { ... }      // ✗ derlenmez: ikisi de std'nin
//     struct Sarmal(Vec<i32>);                  // çözüm: Vec'i kendi kutuna koy
//     impl Display for Sarmal { ... }           // ✓ Sarmal benim
struct Benim(i32); // YEREL tip

// (a) YABANCI trait (Display) + YEREL tip (Benim): tip yerel → İZİNLİ.
impl Display for Benim {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Benim#{}", self.0)
    }
}
// (b) YEREL trait (Etiketle, yukarıda) + YABANCI tip: zaten blanket'le her Display tipe uygulandı;
//     elle `impl Etiketle for bool` de yazabilirdik (trait yerel) — blanket olmasaydı.

fn yerel_yabanci() {
    let b = Benim(7);
    println!("orphan (yabancı trait + yerel tip): {b}"); // Display, Benim yerel olduğu için yazılabildi
    println!("orphan: blanket + yerel tip: {}", b.etiket()); // <Benim#7>

    // ✗ YASAK: hem trait hem tip YABANCI → E0117 (DOĞRULANDI, Böl. 20):
    //     impl Display for Vec<i32> { ... }   // Display de Vec de std'den
    //   Çözüm: NEWTYPE ile sar (Böl. 20) — Sarmal(Vec<...>) senin tipin olur, impl serbest kalır.
}

// ============================================================================
// 3. COVERED IMPL — yabancı trait'i yabancı tipe YAZABİLDİĞİN dar istisna
// ============================================================================
// `impl From<Benim> for String`: From de String de YABANCI (ikisi de std). Yine de İZİNLİ! Çünkü
// generic konumdaki Benim YEREL ve "kapsanmış" (covered). Tehlike yok: çakışan bir impl ancak
// std'nin blanket'iyle gelebilirdi ki o zaten kırıcı değişiklik sayılır.
// (Sade: "covered / kapsanmış" = yerel türün, trait'in köşeli parantezi içinde açıkça
//  geçmesi demek: `From<Benim>` içindeki Benim gibi.)
// SADE ANLATIM: Çeviri sözlüğünü yayınevi basar, Türkçe de herkesindir. Ama "benim uydurduğum
// dilden Türkçeye" sayfasını yalnız ben yazabilirim, çünkü o dili benden başka bilen yoktur.
// `impl From<Benim> for String` böyle bir sayfadır: `From` da `String` de std'nindir, ama
// tanımın içinde `Benim` geçer. Başka hiçbir crate `Benim`'i bilmediği için aynı tanımı
// yazamaz; çakışma tehlikesi olmaz, Rust da izin verir. Bu neden var? Kendi türünü `String`
// gibi hazır türlere çevirebilmen için.
//   Mini örnek:
//     impl From<Benim> for String { ... }       // ✓ From ve String yabancı, ama Benim yerel
//     let s: String = Benim(9).into();          // ✓ From yazınca into() kendiliğinden gelir
//     // impl From<i32> for String { ... }      // ✗ derlenmez: içinde yerel hiçbir tür yok
impl From<Benim> for String {
    fn from(b: Benim) -> String {
        format!("String::from(Benim({}))", b.0)
    }
}
fn covered_impl() {
    let s: String = Benim(9).into(); // covered impl → .into() çalışır
    println!("covered impl (From<Benim> for String): {s}");
}

// SADE ANLATIM (aşağıdaki bölüm için): Yukarıda üç kural gördün: bir çift için tek impl, "ya
// trait ya tür senin olacak" ve içinde senin türün geçen dar istisna. Aşağıdaki bölüm bu
// kuralların NEDEN tam böyle seçildiğini anlatır: bir yanda herkes kendi türüne başkasının
// trait'ini yazabilsin, öte yanda bir kütüphane yeni impl eklediğinde onu kullananların kodu
// bozulmasın. Arada `&` ve `Box` için küçük bir ayrıcalık da anlatılır. İlk okuyuşta
// atlayabilirsin; yukarıdaki üç bölümü anlamış olman yeter.
// ============================================================================
// DERLEYİCİ / RfR PERSPEKTİFİ — kurallar neyi dengeliyor?
// ============================================================================
//
// • COHERENCE: her (tip, trait) için EN FAZLA bir impl. Sorun olmasaydı: ben std'nin bool'u için
//   kendi `Display`imi yazsam, bool yazdıran her kod benim crate'imi de kullanınca derleyici "hangi
//   Display?" diye kalırdı — ikisi de "doğru", seçemez. Coherence bu belirsizliği tümden eler;
//   ayrı derleme (separate compilation) ve crate bağımlılıkları altında bile tek seçim garanti.
//
// • ORPHAN RULE (coherence'ı korumanın pratik yolu): `impl Trait for Tip` ancak trait YA DA tip
//   yerelse yazılır. İki aşırı uç reddedildi: (1) "yalnız trait'i tanımlayan impl yazsın" →
//   Debug/Serialize'ı kendi tipine uygulayamazdın; (2) "yalnız kendi tipine impl yaz" → bir trait
//   crate'i std tiplerine impl veremezdi. Orphan rule ikisini dengeler. İhlal → E0117.
//
// • BLANKET IMPL: yalnız trait'i TANIMLAYAN crate yazabilir; var olan bir trait'e blanket EKLEMEK
//   KIRICI değişikliktir (downstream'deki `impl MyTrait for Foo` aniden E0119 verebilir). Çakışma
//   → E0119 "conflicting implementations" (DOĞRULANDI: blanket + özel impl bir arada olamaz).
//
// • FUNDAMENTAL TİPLER (#[fundamental]: &, &mut, Box): orphan rule açısından "yokmuş" sayılır
//   (kontrol öncesi silinir). Bu yüzden `impl IntoIterator for &MyType` yazabilirsin — & ve
//   IntoIterator yabancı olsa da fundamental & şeffaftır, MyType yereldir. Fundamental tipe blanket
//   eklemek de kırıcı değişikliktir.
//
// • COVERED IMPL: dar istisna. `impl<P..> ForeignTrait<T1..Tn> for T0` ancak en az bir Ti YEREL ve
//   ilk yerel Ti'den ÖNCE bir generic P gelmiyorsa izinli (bir T, Vec<T> gibi başka tipe parametre
//   olursa "covered"; tek başına T ya da &T ise değil). `From<Benim> for String` bu yüzden geçer:
//   T0=String yabancı ama T1=Benim yerel ve kapsanmış. Tehlikesiz çünkü çakışan impl ancak std'nin
//   blanket'iyle gelebilir (o da kırıcı değişiklik).
//
// • ÖZET (RfR): bu kurallar "downstream crate yabancı tipe kendi trait'ini uygulayabilsin" ile
//   "upstream crate kendi trait'ine yeni impl ekleyince downstream'i bozmasın" arasındaki dengeyi
//   kurar. Hepsi DERLEME ANINDA; runtime'da coherence diye bir şey yoktur.
