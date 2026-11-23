# 23. Tutarlılık ve Yetim Kuralı

Bir projede iki kütüphane kullandığınızı düşünün. İkisi de standart kütüphanenin `bool` türü için kendi `Display` uygulamasını yazmış olsun: Biri `evet/hayır`, öteki `doğru/yanlış` yazdırsın. Siz `println!("{}", true)` yazdığınızda hangisi çalışmalı? İkisi de geçerli birer uygulamadır ve derleyicinin aralarında seçim yapacak bir ölçütü yoktur.

Rust bu belirsizliği baştan engeller. *Tutarlılık* (coherence) denen özelliğe göre programın tamamında her tür ve trait çifti için en çok bir uygulama bulunabilir. Bunu crate'ler ayrı ayrı derlenirken de koruyabilmek için *yetim kuralı* (orphan rule) vardır: Bir trait'i bir tür için uygulayabilmeniz için ikisinden en az biri sizin crate'inizde tanımlı olmalıdır.[^yetim]

Bölümde önce tutarlılığı ve genel impl'leri, sonra yetim kuralını ve nedenini, ardından kuralın istisnalarını göreceğiz. Son olarak tersine bir soruyu ele alacağız: Herkese açık bir trait'i başkalarının uygulaması nasıl engellenir?

> [!NOTE]
> Bu bölümün notları ve çalıştırılabilir kodu `learn/advanced-concepts/ileri-coherence` dizinindedir. Depo kökünden `cargo run -p ileri-coherence` komutuyla çalıştırabilirsiniz. Yetim kuralına ilk değinen notlar `learn/basic-concepts` dizinindedir. Trait'leri ([9. bölüm](../orta/09-generics-traits-lifetimes.md)), crate'leri ([13. bölüm](../orta/13-cargo-and-crates.md)) ve newtype kalıbını ([19. bölüm](../orta/19-advanced-features.md)) bildiğiniz varsayılır.

## Günlük hayattan bir benzetme

Okulda her öğrencinin her ders için tek bir notu olur. İki öğretmen aynı öğrenciye aynı dersten iki ayrı not yazarsa karnede hangisinin duracağını kimse bilemez. Okul numarası için de durum aynıdır: İki kişi aynı numarayı alırsa yoklama karışır. Rust'ta öğrenci türdür, ders trait'tir, not da `impl` bloğudur.

*Genel impl* (blanket impl) ise müdürün tek cümlelik duyurusuna benzer: "Okuma bilen herkes kütüphane kartı almış sayılır." Bu duyurudan sonra okuma bilen bir öğrenciye ayrıca kart yazamazsınız; onun kartı zaten vardır.

Yetim kuralını oyun ve oyuncakla düşünün: Trait oyun, tür oyuncaktır. Kendi oyuncağınızı başkasının oyununa sokabilir, kendi oyununuzda başkasının oyuncağına rol verebilirsiniz. Ama oyun da oyuncak da başkasınınsa "bu oyuncak bu oyunda şöyle davranır" diye kural koyamazsınız. Aynı kuralı bir başkası kendi evinde farklı yazabilirdi ve iki ev bir araya gelince hangi kuralın geçerli olduğu bilinemezdi. Çıkış yolu, başkasının oyuncağını kendi kutunuza koymaktır: Kutu sizindir. Buna *newtype* denir.

İstisnayı da çeviri sözlüğü anlatır. Sözlüğü bir yayınevi basar, Türkçe de herkesindir. Ama "sizin uydurduğunuz dilden Türkçeye" sayfasını yalnızca siz yazabilirsiniz, çünkü o dili sizden başka bilen yoktur; aynı sayfayı iki kişi yazamaz.

<details>
<summary>Bu bölümde geçen terimler</summary>

- **crate:** Tek başına derlenen Rust birimi. Sizin projeniz de bir crate'tir, `std` de.
- **impl:** Bir trait'in bir tür için yerine getirildiği kod: `impl Trait for Tip`.
- **yerel (local):** Sizin crate'inizde tanımlanmış tür ya da trait.
- **yabancı (foreign):** Başka bir crate'te, örneğin `std`'de tanımlanmış tür ya da trait.
- **tutarlılık (coherence):** "Her tür ve trait çifti için en çok bir impl" özelliği.
- **yetim kuralı (orphan rule):** Impl yazmak için trait'in ya da türün sizin olması gerektiğini söyleyen kural.
- **genel impl (blanket impl):** Tek blokla bir tür ailesine yazılan impl: `impl<T: X> Trait for T`.
- **newtype:** Yabancı bir türü tek alanlı kendi struct'ınıza sarmak: `struct Sarmal(Vec<i32>)`.
- **kapsanan (covered) parametre:** Başka bir türün argümanı olarak geçen tür parametresi: `Vec<T>` içindeki `T`.
- **kapsanmayan (uncovered) parametre:** Tek başına ya da `&T`, `Box<T>` gibi bir sarmalın içinde geçen parametre.
- **temel (fundamental) tür:** `&`, `&mut`, `Box` ve `Pin`. Yetim kuralı bunların içine bakar, kendilerini saymaz.
- **yukarı akış / aşağı akış (upstream / downstream):** Sizin kullandığınız crate / sizi kullanan crate.
- **kırıcı değişiklik (breaking change):** Yapıldığında başkalarının çalışan kodunu derlenmez hâle getiren değişiklik.
- **ayrı derleme (separate compilation):** Her crate'in ötekilerden bağımsız derlenmesi.

</details>

## Tutarlılık: her çift için en çok bir impl

Aynı trait'i aynı tür için iki kez uygularsanız derleyici durur:

```rust,compile_fail,E0119
trait Etiketle {
    fn etiket(&self) -> String;
}

impl Etiketle for bool {
    fn etiket(&self) -> String {
        String::from("birinci")
    }
}

impl Etiketle for bool {
    fn etiket(&self) -> String {
        String::from("ikinci")
    }
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0119]: conflicting implementations of trait `Etiketle` for type `bool`
   |
 5 | impl Etiketle for bool {
   | ---------------------- first implementation here
...
11 | impl Etiketle for bool {
   | ^^^^^^^^^^^^^^^^^^^^^^ conflicting implementation for `bool`
```

Bu sayede `x.etiket()` çağrısında derleyici hiçbir zaman "hangi impl?" sorusuyla baş başa kalmaz; tek doğru seçim vardır.[^tutarlilik] Aynı crate içindeki çakışmayı yakalamak kolaydır, çünkü iki blok da derleyicinin gözü önündedir. Zor olan, bu özelliği birbirini görmeyen crate'ler arasında da korumaktır. Yetim kuralı bunun için vardır; ona geçmeden önce tutarlılığı en çok zorlayan impl türüne bakalım.

## Genel impl: bir tür ailesine tek blok

*Genel impl*, bir trait'i tek bir türe değil, bir koşulu sağlayan bütün türlere uygular. Standart kütüphanedeki `impl<T: Display + ?Sized> ToString for T` klasik örnektir: Yazdırılabilen her tür `to_string` metodunu kazanır.[^genel] Aynısını kendi trait'iniz için yazabilirsiniz:

```rust
use std::fmt::Display;

trait Etiketle {
    fn etiket(&self) -> String;
}

impl<T: Display> Etiketle for T {
    fn etiket(&self) -> String {
        format!("<{self}>")
    }
}

fn main() {
    println!("{} {} {}", 42.etiket(), "metin".etiket(), true.etiket());
}
```

Çıktı:

```text
<42> <metin> <true>
```

Genel impl varken yazdırılabilen bir tür için ayrıca özel bir impl yazamazsınız; o türün impl'i zaten vardır:

```rust,compile_fail,E0119
use std::fmt::Display;

trait Etiketle {
    fn etiket(&self) -> String;
}

impl<T: Display> Etiketle for T {
    fn etiket(&self) -> String {
        format!("<{self}>")
    }
}

impl Etiketle for i32 {
    fn etiket(&self) -> String {
        format!("sayı: {self}")
    }
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0119]: conflicting implementations of trait `Etiketle` for type `i32`
```

Var olan bir trait'e sonradan genel impl eklemek bu yüzden kırıcı bir değişikliktir. Trait'inizi kullanan bir crate, kendi `Foo` türü için `impl Etiketle for Foo` yazmış olabilir; genel impl'i eklediğiniz gün o crate `E0119` ile derlenmez olur.[^rfc2451] Bir trait'in genel impl'i olacaksa trait'le birlikte yayımlanmalıdır.

## Yetim kuralı: ya trait ya tür sizin olmalı

Yetim kuralının özü şudur: `impl Trait for Tip` yazabilmek için `Trait` ya da `Tip` sizin crate'inizde tanımlı olmalıdır.[^yetim] İki izinli birleşim vardır. Birincisi yabancı trait ve yerel türdür:

```rust
use std::fmt;

struct Benim(i32); // yerel tür

impl fmt::Display for Benim {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Benim#{}", self.0)
    }
}

fn main() {
    println!("{}", Benim(7));
}
```

Çıktı:

```text
Benim#7
```

İkincisi yerel trait ve yabancı türdür. `Vec` ve `bool` standart kütüphanenindir, ama trait sizindir:

```rust
trait KendiTraitin {
    fn ozet(&self) -> String;
}

impl KendiTraitin for Vec<i32> {
    fn ozet(&self) -> String {
        format!("{} öğe", self.len())
    }
}

impl KendiTraitin for bool {
    fn ozet(&self) -> String {
        String::from(if *self { "evet" } else { "hayır" })
    }
}

fn main() {
    println!("{}, {}", vec![1, 2, 3].ozet(), true.ozet());
}
```

Çıktı:

```text
3 öğe, evet
```

İkisi de yabancıysa kapı kapanır:

```rust,compile_fail,E0117
use std::fmt;

impl fmt::Display for Vec<i32> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} öğe", self.len())
    }
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0117]: only traits defined in the current crate can be implemented for types defined outside of the crate
  |
3 | impl fmt::Display for Vec<i32> {
  | ^^^^^^^^^^^^^^^^^^^^^^--------
  |                       |
  |                       `Vec` is not defined in the current crate
  |
  = note: impl doesn't have any local type before any uncovered type parameters
  = note: for more information see https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules
  = note: define and implement a trait or new type instead
```

Son not çözümü de söyler: kendi trait'inizi ya da kendi türünüzü tanımlayın.

### Kural neden böyle?

Crate'ler ayrı ayrı derlenir. Sizin crate'iniz derlenirken onu bir gün kullanacak öteki crate'ler görünmez; aynı crate içindeki çakışmayı yakalayan `E0119` denetimi onlara uzanamaz. Yetim kuralı sorunu başka yerden çözer: Bir impl'i yazabilecek crate'leri baştan sınırlar. Her trait ve her tür tek bir crate'e aittir. O hâlde `impl Trait for Tip` bloğunu yazabilecek en çok iki aday vardır: trait'in crate'i ve türün crate'i. Trait'in crate'i bu impl'i yazabilmek için türü tanımalı, yani türün crate'ine bağımlı olmalıdır; türün crate'i için de tersi geçerlidir. Crate'ler birbirine döngüsel bağımlı olamadığından ikisi birden yazamaz, biri yazdığında öteki onu görür.[^rfc2451]

Ortada hiç impl yokken bile "yabancı trait, yabancı tür" birleşimi yasaktır. Bugün `Vec<i32>` için `Display` yoktur, ama standart kütüphane yarın ekleyebilir; aynı impl'i bir başka crate de yazmış olabilir. Derleyici bu ihtimalleri göremediği için bütün kategoriyi tedbiren kapatır. Adı da buradan gelir: Ne trait'in ne türün sahibi olan bir crate'in yazdığı impl, sahipsiz, yani yetim kalırdı.

Kural iki aşırı ucun arasında durur.[^rfr] "Impl'i yalnızca trait'i tanımlayan yazsın" denseydi kendi türünüze `Debug` ya da bir kütüphanenin `Serialize` trait'ini ekleyemezdiniz. "Yalnızca türü tanımlayan yazsın" denseydi bir trait crate'i `std` türleri için impl veremezdi. Yetim kuralı ikisine de izin verir. Bunun yanında, yukarı akıştaki bir crate kendi trait'ini yeni bir tür için uyguladığında aşağı akıştaki kodun çoğunlukla bozulmamasını sağlar.

### Çözüm: newtype

Yabancı bir trait'i yabancı bir tür için gerçekten istiyorsanız türü kendi struct'ınıza sarın. Sarmalayan tür yerel olduğundan kural sağlanır:[^newtype]

```rust
use std::fmt;

struct BenimVec(Vec<i32>);

impl fmt::Display for BenimVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parcalar: Vec<String> = self.0.iter().map(i32::to_string).collect();
        write!(f, "[{}]", parcalar.join(" | "))
    }
}

fn main() {
    println!("{}", BenimVec(vec![1, 2, 3]));
}
```

Çıktı:

```text
[1 | 2 | 3]
```

Sarmalamak çalışma zamanında bedelsizdir; bedeli yazım tarafındadır, çünkü `BenimVec` artık `Vec`'in metotlarını kendiliğinden taşımaz. Gerekirse `Deref` uygulayarak ([24. bölüm](24-deref-metot.md)) ya da ihtiyaç duyduğunuz metotları tek tek yazarak bu açığı kapatırsınız.

Doğal metotlar (inherent method) için sınır daha da katıdır: Bir türe `impl Tip { ... }` bloğuyla doğrudan metot eklemek yalnızca türü tanımlayan crate'te mümkündür.

```rust,compile_fail,E0116
impl Vec<i32> {
    fn toplam(&self) -> i32 {
        self.iter().sum()
    }
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0116]: cannot define inherent `impl` for a type outside of the crate where the type is defined
  |
  = help: consider defining a trait and implementing it for the type or using a newtype wrapper like `struct MyType(ExternalType);` and implement it
```

## İstisna: trait parametresinde yerel tür

"Ya trait ya tür yerel" cümlesi iyi bir ilk yaklaşımdır ama kuralın tamamı değildir. Aşağıdaki impl'de `From` da `String` de yabancıdır; yine de izin verilir:

```rust
struct Benim(i32);

impl From<Benim> for String {
    fn from(b: Benim) -> String {
        format!("Benim({})", b.0)
    }
}

fn main() {
    let s: String = Benim(9).into(); // From yazınca into() kendiliğinden gelir
    println!("{s}");
}
```

Çıktı:

```text
Benim(9)
```

İzin verilir, çünkü trait'in köşeli parantezi içinde yerel bir tür geçer. `Benim`'i başka hiçbir crate tanımadığından aynı impl'i bir başkası yazamaz. İçinde yerel hiçbir tür olmayan impl ise reddedilir:

```rust,compile_fail,E0117
impl From<i32> for String {
    fn from(x: i32) -> String {
        x.to_string()
    }
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0117]: only traits defined in the current crate can be implemented for types defined outside of the crate
```

### Kuralın tamamı

Reference kuralı şöyle yazar.[^yetim] `impl<P1..=Pn> Trait<T1..=Tn> for T0` biçimindeki bir impl, ancak şunlardan biri doğruysa geçerlidir:

- `Trait` yereldir; ya da
- `T0..=Tn` türlerinden en az biri yereldir ve ilk yerel türe `Ti` dersek, ondan önce gelen `T0..Ti` türlerinde hiçbir *kapsanmayan* tür parametresi geçmez.

Sıralamada `T0` impl'in yazıldığı türdür ve en başta gelir; ardından trait'in parametreleri soldan sağa dizilir. *Kapsanmak*, başka bir türün argümanı olmak demektir: `T`, `Vec<T>` içinde kapsanır; tek başına `T` ya da `&T` ise kapsanmaz. Örneklerle:

| Impl | Sonuç | Neden |
| --- | --- | --- |
| `impl<T> From<Sarmal<T>> for Vec<T>` | derlenir | İlk yerel tür `Sarmal<T>`; öncesindeki `T`, `Vec` içinde kapsanmış. |
| `impl<T> PartialEq<Benim> for Vec<T>` | derlenir | `T`, `Vec` içinde kapsanmış. |
| `impl<T> PartialEq<T> for Benim` | derlenir | İlk yerel tür en başta (`T0`); ondan önce parametre yok. |
| `impl<T: Display> Display for Sarmal<T>` | derlenir | Tür yerel; parametre yerel türün içinde. |
| `impl<T> PartialEq<Benim> for T` | `E0210` | Kapsanmayan `T`, ilk yerel türden önce geliyor. |
| `impl<T> PartialEq<Benim> for &T` | `E0210` | `&` kapsamaz. |
| `impl<T: Yerel> Display for T` | `E0210` | Hiç yerel tür yok, `T` kapsanmamış. |
| `impl Display for Vec<Benim>` | `E0117` | `Vec` yerel değil; `Benim`'i içermesi onu yerel yapmaz. |

Derlenen üç örneği bir arada görelim:

```rust
struct Sarmal<T>(Vec<T>);

impl<T> From<Sarmal<T>> for Vec<T> {
    fn from(s: Sarmal<T>) -> Vec<T> {
        s.0
    }
}

struct Benim(i32);

impl<T: Into<i32> + Copy> PartialEq<T> for Benim {
    fn eq(&self, oteki: &T) -> bool {
        self.0 == (*oteki).into()
    }
}

impl<T> PartialEq<Benim> for Vec<T> {
    fn eq(&self, oteki: &Benim) -> bool {
        self.len() as i32 == oteki.0
    }
}

fn main() {
    let v: Vec<u8> = Sarmal(vec![1, 2]).into();
    println!("{v:?}");
    println!("{} {}", Benim(5) == 5i32, Benim(5) == 7u8);
    println!("{}", v == Benim(2));
}
```

Çıktı:

```text
[1, 2]
true false
true
```

Parametre kapsanmadan önde durduğunda derleyici `E0210` verir ve sıralamayı açıklar:

```rust,compile_fail,E0210
struct Benim(i32);

impl<T> PartialEq<Benim> for T {
    fn eq(&self, _: &Benim) -> bool {
        true
    }
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0210]: type parameter `T` must be covered by another type when it appears before the first local type (`Benim`)
  |
3 | impl<T> PartialEq<Benim> for T {
  |      ^ uncovered type parameter
  |
  = note: implementing a foreign trait is only possible if at least one of the types for which it is implemented is local,
          and no uncovered type parameters appear before that first local type
```

Sıranın önemli olmasının nedeni yine iki crate'in çakışma ihtimalidir. Siz `impl<T> PartialEq<Benim> for T` yazabilseydiniz, bir başka crate de kendi türü için `impl<T> PartialEq<T> for Onlarin` yazabilirdi; ikincisi kurala her durumda uyar, çünkü yerel tür en baştadır. `Onlarin == Benim` karşılaştırmasında iki impl birden geçerli olurdu.[^rfc2451] Parametre `Vec<T>` gibi bir türün içinde kapsandığında böyle bir çakışma ancak standart kütüphanenin yeni bir genel impl eklemesiyle doğabilir; o da zaten kırıcı değişiklik sayılır.

Buradan genel impl'ler için de bir sonuç çıkar: `impl<T> Trait for T` biçimini yalnızca trait'in sahibi yazabilir. Yerel bir türün en başta durduğu `impl<T> PartialEq<T> for Benim` gibi genel biçimler ise yabancı trait'ler için de serbesttir.

## Temel türler: `&`, `&mut`, `Box` ve `Pin`

Dört tür kurucu, yetim kuralı açısından "yokmuş" gibi davranır: `&`, `&mut`, `Box` ve `Pin`. Bunlara *temel* (fundamental) tür denir. `T` yerelse `&T`, `&mut T`, `Box<T>` ve `Pin<T>` de yerel sayılır; buna karşılık içlerindeki parametreyi kapsamazlar.[^temel] Bu yüzden kendi koleksiyonunuz için `IntoIterator`'ı referans üzerinden uygulayabilirsiniz:

```rust
use std::fmt;

struct Koleksiyon(Vec<i32>);

impl<'a> IntoIterator for &'a Koleksiyon {
    type Item = &'a i32;
    type IntoIter = std::slice::Iter<'a, i32>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

struct Benim(i32);

impl fmt::Display for Box<Benim> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "kutuda {}", self.0)
    }
}

fn main() {
    let k = Koleksiyon(vec![1, 2, 3]);
    for x in &k {
        print!("{x} ");
    }
    println!("| {}", Box::new(Benim(7)));
}
```

Çıktı:

```text
1 2 3 | kutuda 7
```

Aynı ayrıcalık `Rc` ya da `Arc` için yoktur; onlar sıradan yabancı türlerdir:

```rust,compile_fail,E0117
use std::fmt;
use std::rc::Rc;

struct Benim(i32);

impl fmt::Display for Rc<Benim> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0117]: only traits defined in the current crate can be implemented for types defined outside of the crate
```

Bu ayrıcalığın bedeli, temel bir tür üzerine sonradan genel impl eklemenin kırıcı bir değişiklik olmasıdır. Trait'lerin de temel olanları vardır; `Fn` ailesi ile `Sized` böyledir ve bunlar için var olan bir türe impl eklemek kırıcı sayılır.[^temel]

## Olmayan impl'e güvenmek

Tutarlılık denetimi yalnızca yazılmış impl'lere bakmaz; ileride yazılabilecekleri de hesaba katar. Aşağıdaki iki blok bugün çakışmaz, çünkü `Vec<i32>` bir `Iterator` değildir. Derleyici yine de kabul etmez:

```rust,compile_fail,E0119
trait Yerel {}

impl<T: Iterator> Yerel for T {}

impl Yerel for Vec<i32> {}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0119]: conflicting implementations of trait `Yerel` for type `Vec<i32>`
  |
3 | impl<T: Iterator> Yerel for T {}
  | ----------------------------- first implementation here
4 |
5 | impl Yerel for Vec<i32> {}
  | ^^^^^^^^^^^^^^^^^^^^^^^ conflicting implementation for `Vec<i32>`
  |
  = note: upstream crates may add a new impl of trait `std::iter::Iterator` for type `std::vec::Vec<i32>` in future versions
```

Not nedeni açıklar: Standart kütüphane gelecekte `Vec<i32>` için `Iterator` uygulayabilir. "Bu tür şu trait'i uygulamıyor" bilgisine ancak o impl'i yazma hakkı sizdeyse güvenebilirsiniz. Nitekim ikinci impl'de `Vec<i32>` yerine kendi türünüz olsaydı kod derlenirdi, çünkü onun `Iterator` olup olmayacağına siz karar verirsiniz.[^rfc1023]

## Trait'i kimin uygulayacağını sınırlamak

Yetim kuralı, herkese açık bir trait'i başkalarının *kendi türleri* için uygulamasını engellemez. Trait'e ileride metot eklemek istiyorsanız bu sorun olabilir: Varsayılan gövdesi olmayan her yeni metot, trait'i uygulayan bütün crate'leri bozar. Kararlı Rust'ta bunun yolu *mühürlü trait* (sealed trait) kalıbıdır. Trait, erişilemeyen bir modüldeki başka bir trait'i üst trait olarak ister:[^muhur]

```rust
mod muhurlu {
    mod ozel {
        pub trait Muhur {}
    }

    pub trait Komut: ozel::Muhur {
        fn ad(&self) -> &'static str;
    }

    pub struct Baslat;

    impl ozel::Muhur for Baslat {}

    impl Komut for Baslat {
        fn ad(&self) -> &'static str {
            "başlat"
        }
    }
}

use muhurlu::{Baslat, Komut};

fn main() {
    println!("{}", Baslat.ad());
}
```

Çıktı:

```text
başlat
```

Dışarıdaki kod `Komut`'u kullanabilir, ama uygulayamaz: `ozel::Muhur`'a erişemediği için üst trait koşulunu sağlayamaz. Derleyici kalıbı tanır ve notunda adıyla anar:

```rust,compile_fail,E0277
mod muhurlu {
    mod ozel {
        pub trait Muhur {}
    }

    pub trait Komut: ozel::Muhur {
        fn ad(&self) -> &'static str;
    }
}

struct Durdur;

impl muhurlu::Komut for Durdur {
    fn ad(&self) -> &'static str {
        "durdur"
    }
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0277]: the trait bound `Durdur: Muhur` is not satisfied
   |
13 | impl muhurlu::Komut for Durdur {
   |                         ^^^^^^ unsatisfied trait bound
   |
   = note: `Komut` is a "sealed trait", because to implement it you also need to implement `muhurlu::ozel::Muhur`, which is not accessible; this is usually done to force you to use one of the provided types that already implement it
```

Aynı amaca dilin içinden ulaşmak için RFC 3323 *impl kısıtlamalarını* önerir. `pub impl(crate) trait Example {}` yazıldığında trait'i herkes kullanabilir, ama yalnızca tanımlandığı crate uygulayabilir. RFC kabul edilip birleştirilmiştir. Derleyicide `impl_restriction` adlı kararsız bir özellik olarak durur; 1.100.0 sürümünde de kararlı değildir, izleme konusu açıktır.[^rfc3323] Kararlı derleyici sözdizimini tanır ama reddeder:

```rust,compile_fail,E0658
pub impl(crate) trait Example {}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0658]: `impl` restrictions are experimental
  |
1 | pub impl(crate) trait Example {}
  |     ^^^^^^^^^^^
  |
  = note: see issue #105077 <https://github.com/rust-lang/rust/issues/105077> for more information
```

## Hata kodu başvurusu

| Kod | Ne zaman görülür? | Ne yapmalı? |
| --- | --- | --- |
| `E0119` | Bir tür ve trait çifti için iki impl olduğunda ya da ileride olabileceğinde | Birini kaldırın; genel impl'in kapsamını daraltın ya da newtype kullanın. |
| `E0117` | Yabancı trait, içinde yerel tür geçmeden yabancı türe uygulandığında | Kendi trait'inizi ya da newtype'ınızı tanımlayın. |
| `E0210` | Kapsanmayan bir tür parametresi ilk yerel türden önce geldiğinde | Parametreyi yerel bir türün içine alın ya da yerel türü başa koyun. |
| `E0116` | Yabancı türe doğal metot eklenmeye çalışıldığında | Bir trait tanımlayıp uygulayın ya da newtype kullanın. |
| `E0277` | Mühürlü bir trait dışarıdan uygulanmaya çalışıldığında | Trait'i uygulayan hazır türleri kullanın; kapı bilerek kapalıdır. |
| `E0658` | Kararsız bir özellik (`impl(crate) trait`) kullanıldığında | Mühürlü trait kalıbını kullanın. |

## Perde arkası: Derleyici ne yapıyor?

> [!NOTE]
> Bu başlık ilk okuyuşta atlanabilir; yukarıdaki kuralları anlamış olmanız yeterlidir.

Tutarlılık ve yetim kuralı yalnızca derleme zamanında vardır; çalışan programda izleri kalmaz. Denetim üç parçadan oluşur.

**Yetim denetimi.** Her impl için `orphan_check_impl` çalışır; asıl kural `orphan_check_trait_ref` fonksiyonundadır.[^yetim-kod] Fonksiyon `T0`'dan başlayarak türleri sırayla dolaşır. Temel türlerin içine iner (`is_fundamental`), yerel bir tür bulduğunda durur, ondan önce kapsanmayan bir parametre gördüyse `UncoveredTyParams` hatası üretir. `E0117` ile `E0210` bu sonuçtan doğar.

**Çakışma denetimi.** Aynı trait'in impl'leri bir çizgeye (graph) eklenirken ikişer ikişer karşılaştırılır. `overlapping_trait_impls` iki başlığın tek bir türde birleşip birleşmediğine bakar; birleşiyorsa koşullardan birinin kesinlikle sağlanamadığını göstermeye çalışır.[^cakisma] Gösteremezse `E0119` verilir.

**Bilinebilirlik.** "Bu koşul sağlanamaz" sonucuna varmadan önce `trait_ref_is_knowable` sorulur: Bu impl'i ileride başka bir crate, yani yukarı akış ya da aşağı akış, ekleyebilir mi? Ekleyebilirse koşul "sağlanamaz" değil "belirsiz" sayılır ve impl'ler çakışıyor kabul edilir.[^bilinebilir] `upstream crates may add a new impl` notu buradan gelir.

## Özet

- Programın tamamında her tür ve trait çifti için en çok bir impl olabilir; ikincisi `E0119`'dur.
- `impl Trait for Tip` için trait ya da tür yerel olmalıdır; ikisi de yabancıysa `E0117`. Çözüm newtype ya da kendi trait'inizdir.
- Trait'in parametresinde yerel bir tür geçiyorsa yabancı trait yabancı türe uygulanabilir; ilk yerel türden önce kapsanmayan parametre bulunamaz.
- `&`, `&mut`, `Box` ve `Pin` yetim kuralında şeffaftır.
- Var olan bir trait'e genel impl eklemek kırıcı değişikliktir. Trait'i başkalarının uygulamasını mühürlü trait kalıbıyla engelleyebilirsiniz.

Sıradaki bölüm [24. `Deref` ve Metot Çözümleme](24-deref-metot.md), nokta operatörünün perde arkasını anlatıyor: `x.metot()` yazdığınızda derleyici metodu nerede arar?

## Kaynaklar

[^yetim]: The Rust Reference, [Orphan rules](https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules); hata dizini, [`E0117`](https://doc.rust-lang.org/error_codes/E0117.html), [`E0210`](https://doc.rust-lang.org/error_codes/E0210.html) ve [`E0116`](https://doc.rust-lang.org/error_codes/E0116.html).
[^tutarlilik]: The Rust Reference, [Trait implementation coherence](https://doc.rust-lang.org/reference/items/implementations.html#trait-implementation-coherence); hata dizini, [`E0119`](https://doc.rust-lang.org/error_codes/E0119.html).
[^genel]: The Rust Programming Language, [Using Trait Bounds to Conditionally Implement Methods](https://doc.rust-lang.org/book/ch10-02-traits.html#using-trait-bounds-to-conditionally-implement-methods); std, [`ToString`](https://doc.rust-lang.org/std/string/trait.ToString.html).
[^rfc2451]: [RFC 2451: Re-rebalancing coherence](https://rust-lang.github.io/rfcs/2451-re-rebalancing-coherence.html), özellikle "What is coherence and why do we care?" ve "Effects on parent crates" başlıkları.
[^rfr]: Jon Gjengset, *Rust for Rustaceans*, 2. bölüm, "Coherence and the Orphan Rule".
[^newtype]: The Rust Programming Language, [Implementing External Traits with the Newtype Pattern](https://doc.rust-lang.org/book/ch20-02-advanced-traits.html#implementing-external-traits-with-the-newtype-pattern).
[^temel]: The Rust Reference, [Fundamental type constructors](https://doc.rust-lang.org/reference/glossary.html#fundamental-type-constructors) ve [Fundamental traits](https://doc.rust-lang.org/reference/glossary.html#fundamental-traits).
[^rfc1023]: [RFC 1023: Rebalancing coherence](https://rust-lang.github.io/rfcs/1023-rebalancing-coherence.html#type-locality-and-negative-reasoning).
[^muhur]: Rust API Guidelines, [Sealed traits protect against downstream implementations](https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed).
[^rfc3323]: [RFC 3323: Restrictions](https://rust-lang.github.io/rfcs/3323-restrictions.html); [izleme konusu #105077](https://github.com/rust-lang/rust/issues/105077); rustc 1.99.0 kaynak kodu, [`unstable.rs`, `impl_restriction`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_feature/src/unstable.rs#L592).
[^yetim-kod]: rustc 1.99.0 kaynak kodu, [`orphan.rs`, `orphan_check_impl`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_analysis/src/coherence/orphan.rs#L22); [`rustc_next_trait_solver/src/coherence.rs`, `orphan_check_trait_ref`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_next_trait_solver/src/coherence.rs#L222) ve [`is_fundamental`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_next_trait_solver/src/coherence.rs#L417).
[^cakisma]: rustc 1.99.0 kaynak kodu, [`rustc_trait_selection/src/traits/coherence.rs`, `overlapping_trait_impls`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_trait_selection/src/traits/coherence.rs#L130) ve [`specialization_graph.rs`, `fn insert`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_trait_selection/src/traits/specialize/specialization_graph.rs#L76).
[^bilinebilir]: rustc 1.99.0 kaynak kodu, [`rustc_next_trait_solver/src/coherence.rs`, `trait_ref_is_knowable`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_next_trait_solver/src/coherence.rs#L48).
