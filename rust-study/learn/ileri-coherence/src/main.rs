// Sağlamlaştırma — İleri konu: COHERENCE ve ORPHAN RULE (tutarlılık ve yetim kuralı)
// Çalıştır: cargo run -p ileri-coherence
//
// Kaynak: Rust for Rustaceans Böl. 3 ("Coherence and the Orphan Rule") + std.
// Böl. 10 (generics/traits) ve Böl. 20'de (newtype) değindiğimiz orphan rule'un TAM kuralı.
//
// Doğrulanmış (rustc ile): çakışan impl'ler → E0119 "conflicting implementations"; hem trait
// hem tip yabancıysa → E0117. Ayrıntı: DERLEYİCİ PERSPEKTİFİ (altta).

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
    println!("blanket: {} {} {}", 42.etiket(), "metin".etiket(), true.etiket());
    // ✗ Bu blanket varken AYRICA `impl Etiketle for i32` YAZAMAZSIN (i32: Display → çakışır):
    //     E0119 "conflicting implementations of trait `Etiketle` for type `i32`" (DOĞRULANDI).
    //   Coherence tam da bunu yasaklar: i32.etiket() için iki aday olamaz.
}

// ============================================================================
// 2. ORPHAN RULE — impl yazabilmek için trait YA DA tip YEREL olmalı
// ============================================================================
// Yetim kuralı (orphan rule): `impl Trait for Tip` ancak Trait YA DA Tip senin crate'inde
// TANIMLIYSA yazılabilir. İzin verilenler ve neden:
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
impl From<Benim> for String {
    fn from(b: Benim) -> String {
        format!("String::from(Benim({}))", b.0)
    }
}
fn covered_impl() {
    let s: String = Benim(9).into(); // covered impl → .into() çalışır
    println!("covered impl (From<Benim> for String): {s}");
}

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
