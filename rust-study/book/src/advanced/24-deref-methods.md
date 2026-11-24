# 24. `Deref` ve Metot Çözümleme

`Box<Vec<i32>>` türünde bir `kutulu` değişkeniniz olsun. `kutulu.len()` yazar ve sonucu alırsınız. Oysa `len` metodu `Box`'ta değil `Vec`'te tanımlıdır ve `&self` ister; siz ne kutuyu açtınız ne `&` işareti koydunuz. Nokta operatörü (`.`) iki işi sizin yerinize yapmıştır: Metodu bulana kadar sarmalları açmış, metodun istediği ödünç alma biçimini eklemiştir.

Bu kolaylık olmasaydı `Box`, `Rc` ve `String` gibi her sarmalı her kullanımda elle açmanız, `(&mut *kutu).metot()` gibi işaret yığınları yazmanız gerekirdi. Ama kolaylığın bir de bedeli vardır: Derleyicinin hangi metodu seçtiği her zaman ilk bakışta belli olmaz. Sarmalın kendi metodu, içindeki türün aynı adlı metodunu sessizce örter.

Bu bölümde önce `Deref` trait'ini ve *deref coercion* denen örtük dönüşümü, sonra metot aramanın kesin kurallarını, en sonda da bu kuralların doğurduğu tuzakları göreceğiz.

> [!NOTE]
> Bu bölümün kodu `learn/advanced-concepts/advanced-deref-methods` dizinindedir. Depo kökünden `cargo run -p advanced-deref-methods` komutuyla çalıştırabilirsiniz. Referansları ([3. bölüm](../basics/03-ownership.md)), metotları ve `self`'i ([4. bölüm](../basics/04-structs.md)), trait'leri ([9. bölüm](../intermediate/09-generics-traits-lifetimes.md)) ve `Box` ile `Deref`'in ilk tanıtımını ([14. bölüm](../intermediate/14-smart-pointers.md)) bildiğiniz varsayılır.

## Günlük hayattan bir benzetme

Telefonunuza kılıf taktığınızı düşünün. Aramak, yazmak ya da fotoğraf çekmek için kılıfı her seferinde çıkarmazsınız; kılıf takılıyken de telefonu kullanırsınız. `Deref`, bir sarmal türe bu özelliği verir: "İçimde şu var, bana gelen isteği ona ilet."

Metot arayan derleyici de yardımsever bir kütüphane görevlisine benzer. Kitabı istersiniz; önce ön rafa bakar, yoksa arkadaki rafa, o da olmazsa depoya iner ve bulduğu ilk yerde durur. Buna *autoderef* denir. Kitabı uzatırken ne yapacağınıza bakar: Yalnızca okuyacaksanız açıp gösterir (`&`), üstüne not alacaksanız kalemle birlikte verir (`&mut`). Buna da *autoref* denir.

Son olarak sınıfta iki Ali olduğunu düşünün; biri yanınızda, öteki arka sırada oturuyor. "Ali!" diye seslenirseniz yanınızdaki döner, arkadakine sesiniz ulaşmaz bile. Aynı durum kapalı bir oyuncak kutusunda da görülür: "Rengi ne?" diye sorulduğunda kutunun üstünde bir renk etiketi varsa önce o okunur; oyuncağın rengini öğrenmek için kutuyu açmanız gerekir. Derleyici de önce sarmalın kendi metoduna bakar.

<details>
<summary>Bu bölümde geçen terimler</summary>

- **sarmal (wrapper):** Başka bir değeri içinde taşıyan tür: `Box<T>`, `Rc<T>` ya da buradaki `Kutu<T>`.
- **`Deref`:** "İçimdeki değere şöyle ulaşılır" diyen trait. `*x` operatörü onu kullanır.
- **`Target`:** `Deref`'in gösterdiği iç tür; "açınca ne çıkar?" sorusunun yanıtı.
- **örtük dönüşüm (coercion):** Derleyicinin bir türü sormadan ve güvenle başka bir türe çevirmesi.
- **deref coercion:** `&Sarmal` türünün kendiliğinden `&İçindeki` türüne çevrilmesi.
- **alıcı (receiver):** Noktanın solundaki değer: `x.metot()` içindeki `x`.
- **autoderef:** Metodu ararken derleyicinin sarmalları sırayla açması.
- **autoref:** Metodun istediği `&` ya da `&mut` işaretini derleyicinin eklemesi.
- **doğal metot (inherent method):** Türün kendi `impl Tür { ... }` bloğunda yazılı, trait'ten gelmeyen metot.
- **gölgeleme (shadowing):** Aynı adlı iki şeyden yakındakinin uzaktakini örtmesi.
- **tam nitelikli çağrı (fully qualified call):** Metodun hangi türe ait olduğunu açıkça yazmak: `Ic::ad(&s)`.
- **boyut silme (unsizing):** Boyu belli diziyi (`[T; 3]`) boyu çalışma zamanında belli olan dilime (`[T]`) çeviren adım.
- **dilim (slice):** Art arda dizilmiş öğelerin görünümü; `Vec`'in içindeki veri gibi.
- **ilişkili fonksiyon (associated function):** `Tür::ad(...)` biçiminde çağrılan fonksiyon. `self` almayanlara "statik fonksiyon" da denir.

</details>

## `Deref`: sarmalın içine giden yol

`Deref` trait'i `*x` operatörünün anlamını belirler. Referanslar için `*` zaten çalışır; kendi türünüz için çalışsın istiyorsanız `Deref`'i uygularsınız:[^deref]

```rust
use std::ops::Deref;

struct Kutu<T>(T);

impl<T> Deref for Kutu<T> {
    type Target = T; // açınca ne çıkar?

    fn deref(&self) -> &T {
        &self.0
    }
}

fn main() {
    let k = Kutu(5);
    println!("*Kutu(5) = {}", *k);
    println!("{}", *k + 1 == *(k.deref()) + 1);
}
```

Çıktı:

```text
*Kutu(5) = 5
true
```

`Deref` uygulayan bir `k` için `*k`, `*Deref::deref(&k)` demektir: `deref` bir `&T` verir, baştaki yerleşik `*` de o referansı açar. `deref`'in değer değil referans döndürmesi bilinçlidir; değer döndürseydi `*k` her yazıldığında içerik kutudan taşınırdı.

Nitekim `*k` üzerinden taşıma yapamazsınız:

```rust,compile_fail,E0507
use std::ops::Deref;

struct Kutu<T>(T);

impl<T> Deref for Kutu<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

fn main() {
    let k = Kutu(String::from("Kaan"));
    let s: String = *k;
    println!("{s}");
}
```

Derleyicinin yanıtı:

```text
error[E0507]: cannot move out of dereference of `Kutu<String>`
```

`Box<T>` bu konuda ayrıcalıklıdır: `let s = *kutu;` kutunun içindekini taşır. Bu yetenek `Deref`'ten değil, derleyicinin `Box`'ı yerleşik olarak tanımasından gelir; kendi türlerinize kazandıramazsınız.[^ozel]

### Tek bir `Target`

Bir türün "içi" tek anlamlı olmalıdır. `Deref` jenerik değildir, `Target` ise ilişkili türdür; bu yüzden bir tür için `Deref` yalnızca bir kez uygulanabilir:

```rust,compile_fail,E0119
use std::ops::Deref;

struct Kutu {
    ad: String,
    yil: u32,
}

impl Deref for Kutu {
    type Target = String;

    fn deref(&self) -> &String {
        &self.ad
    }
}

impl Deref for Kutu {
    type Target = u32;

    fn deref(&self) -> &u32 {
        &self.yil
    }
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0119]: conflicting implementations of trait `Deref` for type `Kutu`
```

## Deref coercion: referans kendiliğinden açılır

Bir fonksiyon `&str` isterken siz `&Kutu<String>` verirseniz derleyici aradaki yolu kendisi kurar. `T: Deref<Target = U>` ise `&T`, gereken yerde `&U`'ya dönüşür ve dönüşüm zincirlenebilir:[^coercion]

```rust
use std::ops::Deref;

struct Kutu<T>(T);

impl<T> Deref for Kutu<T> {
    type Target = T;

    fn deref(&self) -> &T {
        println!("deref çağrıldı");
        &self.0
    }
}

fn selamla(ad: &str) {
    println!("selam, {ad}");
}

fn main() {
    let isim = Kutu(String::from("Kaan"));
    selamla(&isim); // &Kutu<String>, &String, &str

    let ic_ice = Kutu(Kutu(String::from("Ada")));
    selamla(&ic_ice);
}
```

Çıktı:

```text
deref çağrıldı
selam, Kaan
deref çağrıldı
deref çağrıldı
selam, Ada
```

İlk çağrıda bir, ikincide iki kez `Kutu::deref` çalıştı; sondaki `&String`'den `&str`'ye geçiş ise `String`'in kendi `Deref`'idir. Kaç `deref` çağrısı gerektiği derleme zamanında, türlere bakılarak belirlenir. Bu yüzden zinciri derleyiciye kurdurmakla elle yazmak arasında çalışma zamanı farkı yoktur.[^book-coercion] Yine de araya konan her `deref` sıradan bir fonksiyon çağrısıdır ve gerçekten çalışır. Çoğu tür için bu çağrı yalnızca bir adres hesabıdır ve optimizasyonla ortadan kalkar; ama `deref` gövdesine pahalı bir iş yazarsanız o iş her örtük dönüşümde yapılır.

Dönüşüm yalnızca referanslar arasında olur. Kutunun kendisini verirseniz derleyici açmaz:

```rust,compile_fail,E0308
use std::ops::Deref;

struct Kutu<T>(T);

impl<T> Deref for Kutu<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

fn selamla(ad: &str) {
    println!("selam, {ad}");
}

fn main() {
    let k = Kutu(String::from("Ada"));
    selamla(k);
}
```

Derleyicinin yanıtı:

```text
error[E0308]: mismatched types
   |
   = note: expected reference `&str`
                 found struct `Kutu<String>`
help: consider borrowing here
   |
19 |     selamla(&k);
   |             +
```

Deref coercion, hedef türün baştan belli olduğu yerlerde uygulanır: fonksiyon ve metot argümanları, türü yazılmış `let`, struct alanları, dönüş değeri.[^yerler] Jenerik bir parametrede hedef tür belli olmadığından dönüşüm yapılmaz; bunun sonucunu bölümün sonunda göreceğiz. Metodun *alıcısı* da bu listede yoktur. Orada birazdan anlatacağımız ayrı bir arama çalışır; sonuçlar çoğu zaman aynı göründüğünden ikisi sık sık karıştırılır.

### Değiştirilebilir yol: `DerefMut`

`Deref` yalnızca okuma yolu verir. Sarmalın içini değiştirmek için `DerefMut` gerekir. Yoksa metot bulunur ama çağrılamaz:

```rust,compile_fail,E0596
use std::ops::Deref;

struct Kutu<T>(T);

impl<T> Deref for Kutu<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

fn main() {
    let mut k = Kutu(vec![1]);
    k.push(2);
    println!("{}", k.len());
}
```

Derleyicinin yanıtı:

```text
error[E0596]: cannot borrow data in dereference of `Kutu<Vec<i32>>` as mutable
   |
   = help: trait `DerefMut` is required to modify through a dereference, but it is not implemented for `Kutu<Vec<i32>>`
```

`DerefMut`, `Deref`'i üst trait olarak ister ve aynı `Target`'ı kullanır. Derleyici hangisinin gerektiğini kullanıma bakarak seçer:

```rust
use std::ops::{Deref, DerefMut};

struct Kutu<T>(T);

impl<T> Deref for Kutu<T> {
    type Target = T;

    fn deref(&self) -> &T {
        println!("deref");
        &self.0
    }
}

impl<T> DerefMut for Kutu<T> {
    fn deref_mut(&mut self) -> &mut T {
        println!("deref_mut");
        &mut self.0
    }
}

fn main() {
    let mut k = Kutu(vec![1]);
    k.push(2); // push(&mut self)
    println!("uzunluk {}", k.len()); // len(&self)
    *k = vec![7, 8, 9]; // yazma
    println!("uzunluk {}", k.len());
}
```

Çıktı:

```text
deref_mut
deref
uzunluk 2
deref_mut
deref
uzunluk 3
```

Örtük dönüşüm için de kural aynı mantıktadır: `&mut T`, hem `&mut U`'ya hem `&U`'ya dönüşebilir; `&T` ise hiçbir zaman `&mut U`'ya dönüşmez. Paylaşımlı bir ödünçten değiştirilebilir bir ödünç üretmek, ödünç kurallarını bozardı.[^mut]

## Metot nasıl bulunur: autoderef ve autoref

`alıcı.metot()` yazdığınızda derleyici önce bir aday listesi çıkarır. Kurallar Reference'ta şöyledir:[^cagri]

1. Alıcının türünden başlanır ve tekrar tekrar `*` uygulanır; her adımda çıkan tür listeye eklenir. Zincir bir diziyle biterse, listenin sonuna o dizinin dilim hâli de (`[T; N]` için `[T]`) eklenir.
2. Listedeki her `U` türünün hemen ardına `&U` ve `&mut U` konur.
3. Liste baştan sona taranır. Her aday için önce o türün doğal metotlarına, sonra kapsamda görünen trait'lerin metotlarına bakılır. Uyan ilk metot seçilir ve arama biter.

Burada "uymak", metodun `self` parametresinin türünün adayla aynı olması demektir. `&self` alan bir metodun `self` türü `&U`, `&mut self` alanınki `&mut U`, `self` alanınki `U`'dur. `kutulu: Box<Vec<i32>>` için aday listesi şöyle başlar:

| Sıra | Aday | `kutulu.len()` | `kutulu.first()` |
| --- | --- | --- | --- |
| 1–3 | `Box<Vec<i32>>`, `&Box<…>`, `&mut Box<…>` | yok | yok |
| 4 | `Vec<i32>` | yok | yok |
| 5 | `&Vec<i32>` | `Vec::len`, arama biter | yok |
| 6 | `&mut Vec<i32>` |  | yok |
| 7 | `[i32]` |  | yok |
| 8 | `&[i32]` |  | `<[i32]>::first`, arama biter |

`Vec` de `Deref<Target = [T]>` uyguladığından zincir dilime kadar uzanır. Ama `len` çağrısı oraya inmez: `Vec`'in kendi `len` metodu vardır ve arama orada durur. `first`, `iter` ve `contains` ise yalnızca dilimde tanımlıdır; zincir bir adım daha iner.

```rust
fn main() {
    let kutulu = Box::new(vec![1, 2, 3]);

    println!("len      = {}", kutulu.len()); // Vec::len
    println!("first    = {:?}", kutulu.first()); // <[i32]>::first
    println!("contains = {}", kutulu.contains(&1));
    println!("elle     = {} {:?}", Vec::len(&kutulu), <[i32]>::first(&kutulu));
}
```

Çıktı:

```text
len      = 3
first    = Some(1)
contains = true
elle     = 3 Some(1)
```

### Autoref: `&` ve `&mut` kendiliğinden gelir

Aday listesinde `U`'nun yanında `&U` ve `&mut U` da bulunduğu için metodun istediği ödünç biçimini siz yazmazsınız. `v.push(4)`, `Vec::push(&mut v, 4)` olarak; `s.len()`, `String::len(&s)` olarak okunur:

```rust
fn main() {
    let mut v = vec![1, 2, 3];
    v.push(4);
    (&mut v).push(5); // aynı şey, elle
    println!("{v:?}");

    let s = String::from("merhaba");
    println!("{} {}", s.len(), (&s).len());
}
```

Çıktı:

```text
[1, 2, 3, 4, 5]
7 7
```

Autoref ödünç kurallarını gevşetmez. Metot arama, alıcının değiştirilebilir olup olmadığına bakmaz; metodu bulduktan sonra o ödünç alınamıyorsa hata verir:[^cagri]

```rust,compile_fail,E0596
fn main() {
    let w = vec![1];
    w.push(2);
    println!("{w:?}");
}
```

Derleyicinin yanıtı:

```text
error[E0596]: cannot borrow `w` as mutable, as it is not declared as mutable
  |
3 |     w.push(2);
  |     ^ cannot borrow as mutable
  |
help: consider changing this to be mutable
  |
2 |     let mut w = vec![1];
  |         +++
```

Aynı nedenle C ve C++'taki `->` operatörünün Rust'ta karşılığı yoktur. Alıcının değer mi, referans mı, akıllı işaretçi mi olduğuna bakılmaksızın nokta yazılır.[^ok]

## Doğal metot içtekini gölgeler

Arama, zincirin dış halkasından başladığı için sarmalın kendi metodu içtekinden önce bulunur. Önce sarmalın kendi metodu yokken ne olduğuna bakalım:

```rust
use std::ops::Deref;

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

fn main() {
    let s = Sarmal(Ic);
    println!("s.ad() = {}", s.ad());
}
```

Çıktı:

```text
s.ad() = IC
```

Şimdi `Sarmal`'a aynı adlı bir metot ekleyelim. `main` içindeki `s.ad()` satırı hiç değişmez, ama artık başka bir fonksiyonu çağırır:

```rust
# use std::ops::Deref;
# struct Ic;
# impl Ic {
#     fn ad(&self) -> &str {
#         "IC"
#     }
# }
# struct Sarmal(Ic);
# impl Deref for Sarmal {
#     type Target = Ic;
#     fn deref(&self) -> &Ic {
#         &self.0
#     }
# }
impl Sarmal {
    fn ad(&self) -> &str {
        "SARMAL"
    }
}

fn main() {
    let s = Sarmal(Ic);
    println!("s.ad()     = {}", s.ad());
    println!("(*s).ad()  = {}", (*s).ad());
    println!("Ic::ad(&s) = {}", Ic::ad(&s));
}
```

Çıktı:

```text
s.ad()     = SARMAL
(*s).ad()  = IC
Ic::ad(&s) = IC
```

İçtekini istiyorsanız iki yolunuz vardır. `(*s).ad()` elle bir adım iner. `Ic::ad(&s)` ise metodun türünü açıkça söyler; buradaki `&s` argüman konumunda olduğundan deref coercion ile `&Ic`'e dönüşür.

Tehlikeli olan sessizliktir. Sarmala sonradan aynı adlı bir metot eklenirse, eskiden içteki türe giden çağrılar yön değiştirir ve imzalar uyuştuğu sürece derleyici uyarmaz.[^rfr] Akıllı işaretçi yazanlar bu yüzden doğal metottan kaçınır ve `self` almayan ilişkili fonksiyonları yeğler.[^akilli] Standart kütüphane de öyle yapar: `Rc::strong_count(&r)`, `Box::leak(b)`, `Arc::downgrade(&a)`. Kendi türünüzde şöyle görünür:

```rust
use std::ops::Deref;

struct Kutu<T>(T);

impl<T> Kutu<T> {
    fn ac(kutu: Kutu<T>) -> T {
        kutu.0
    }
}

impl<T> Deref for Kutu<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

fn main() {
    let k = Kutu(String::from("Kaan"));
    println!("{}", k.len()); // içteki String'in metodu
    let s: String = Kutu::ac(k); // Kutu'nun kendi işi
    println!("{s}");
}
```

Çıktı:

```text
4
Kaan
```

`T`'nin de `ac` adlı bir metodu olsaydı karışıklık çıkmazdı: `k.ac()` içtekini, `Kutu::ac(k)` sarmalınkini çağırırdı.

### Trait metotları da gölgeler

Dış halkada bulunan metodun doğal olması gerekmez; trait metodu da aramayı bitirir. En sık karşılaşılan örneği `clone`'dur:

```rust
use std::rc::Rc;

fn main() {
    let rc = Rc::new(String::from("Kaan"));

    let ikinci: Rc<String> = rc.clone(); // sayaç artar, metin kopyalanmaz
    let metin: String = (*rc).clone(); // metnin kendisi kopyalanır

    println!("sayaç {}, aynı yer: {}", Rc::strong_count(&rc), Rc::ptr_eq(&rc, &ikinci));
    println!("{metin}");
}
```

Çıktı:

```text
sayaç 2, aynı yer: true
Kaan
```

### Aynı halkada öncelik: önce doğal, sonra trait

Aynı aday için hem doğal metot hem trait metodu varsa doğal olan seçilir. Ama "aynı aday" sözüne dikkat edin: Sıra `U`, `&U`, `&mut U` olduğundan `&self` alan bir trait metodu, `&mut self` alan doğal metottan önce bulunur.[^cagri]

```rust
struct Foo;

impl Foo {
    fn bar(&mut self) {
        println!("doğal metot");
    }
}

trait Bar {
    fn bar(&self);
}

impl Bar for Foo {
    fn bar(&self) {
        println!("trait metodu");
    }
}

fn main() {
    let mut f = Foo;
    f.bar(); // &Foo adayı, &mut Foo'dan önce gelir
    (&mut f).bar(); // aday listesi &mut Foo ile başlar
    Foo::bar(&mut f); // tam nitelikli
    <Foo as Bar>::bar(&f);
}
```

Çıktı:

```text
trait metodu
doğal metot
doğal metot
trait metodu
```

Aynı aday için iki ayrı trait'in aynı adlı metodu uyuyorsa derleyici seçim yapmaz:

```rust,compile_fail,E0034
trait Ucak {
    fn uc(&self);
}

trait Kus {
    fn uc(&self);
}

struct Robot;

impl Ucak for Robot {
    fn uc(&self) {}
}

impl Kus for Robot {
    fn uc(&self) {}
}

fn main() {
    Robot.uc();
}
```

Derleyicinin yanıtı:

```text
error[E0034]: multiple applicable items in scope
```

Çözüm yine tam nitelikli çağrıdır: `Ucak::uc(&Robot)`.

## `Deref` kalıtım değildir

`Deref` uygulayan sarmal, içteki türün metotlarını ve alanlarını nokta ile kullanabilir. Bu görüntü kalıtıma benzer; ama arada bir alt tür ilişkisi yoktur. `Sarmal` bir `Ic` değildir, yalnızca `Ic`'e giden bir yolu vardır:

```rust
use std::ops::Deref;

trait Boya {
    fn boya(&self) -> &'static str;
}

struct Ic {
    yil: u32,
}

impl Boya for Ic {
    fn boya(&self) -> &'static str {
        "mavi"
    }
}

struct Sarmal(Ic);

impl Deref for Sarmal {
    type Target = Ic;

    fn deref(&self) -> &Ic {
        &self.0
    }
}

fn ic_al(ic: &Ic) -> u32 {
    ic.yil
}

fn main() {
    let s = Sarmal(Ic { yil: 2026 });
    println!("{} {} {}", s.boya(), s.yil, ic_al(&s));
}
```

Çıktı:

```text
mavi 2026 2026
```

Metot, alan ve referans argümanı çalıştı. Trait sınırı ise çalışmaz; `Sarmal`, `Boya`'yı uygulamaz ve jenerik parametrede deref coercion yapılmaz:

```rust,compile_fail,E0277
# use std::ops::Deref;
# trait Boya {
#     fn boya(&self) -> &'static str;
# }
# struct Ic {
#     yil: u32,
# }
# impl Boya for Ic {
#     fn boya(&self) -> &'static str {
#         "mavi"
#     }
# }
# struct Sarmal(Ic);
# impl Deref for Sarmal {
#     type Target = Ic;
#     fn deref(&self) -> &Ic {
#         &self.0
#     }
# }
fn jenerik<T: Boya>(t: &T) {
    println!("{}", t.boya());
}

fn main() {
    let s = Sarmal(Ic { yil: 2026 });
    println!("{}", s.yil);
    jenerik(&s);
}
```

Derleyicinin yanıtı:

```text
error[E0277]: the trait bound `Sarmal: Boya` is not satisfied
```

Aynı nedenle `Vec<Ic>` içine `Sarmal` koyamazsınız (`E0308`). Çözüm ya `jenerik(&*s)` yazarak içtekini vermek ya da `Boya`'yı `Sarmal` için ayrıca uygulamaktır. Rust'ta ortak davranış kalıtımla değil trait'lerle paylaşılır ([17. bölüm](../intermediate/17-oop.md)). `Deref`'i yalnızca işaretçi gibi davranan, içindeki türü şeffaf biçimde sunan türler için uygulayın; "metotlar bedavaya gelsin" diye başka türlere eklemeyin.[^yalniz]

## Hata kodu başvurusu

| Kod | Ne zaman görülür? | Ne yapmalı? |
| --- | --- | --- |
| `E0596` | Autoref `&mut` almak isteyip alamadığında: değişken `mut` değilse ya da sarmalda `DerefMut` yoksa | `let mut` yazın ya da `DerefMut` uygulayın. |
| `E0119` | Aynı türe ikinci bir `Deref` yazıldığında | Tek `Target` seçin; ötekini ayrı bir metotla verin. |
| `E0308` | Referans beklenen yere sarmalın kendisi verildiğinde | `&` ekleyin; örtük dönüşüm referanslar arasındadır. |
| `E0507` | `*k` ile kutudan değer taşınmak istendiğinde | İçi veren bir ilişkili fonksiyon yazın ya da klonlayın. |
| `E0034` | Aynı aday için iki trait'in aynı adlı metodu uyduğunda | `Trait::metot(&x)` yazın. |
| `E0277` | İçteki türün uyguladığı trait, sarmal için beklendiğinde | `&*s` verin ya da trait'i sarmala da uygulayın. |

## Perde arkası: Derleyici ne yapıyor?

> [!NOTE]
> Bu başlık ilk okuyuşta atlanabilir; yukarıdaki kuralları anlamış olmanız yeterlidir.

**Arama.** Metot arama, tür denetiminin içinde, `rustc_hir_typeck` crate'indeki `probe.rs` dosyasında yapılır. `method_autoderef_steps` alıcı türünün zincirini çıkarır ve sonda dizi varsa boyut silme adımını ekler. `pick_all_method` bu adımları sırayla dolaşır; her adımda önce `pick_by_value_method` (`U`), sonra `pick_autorefd_method` (`&U` ve `&mut U`) denenir. `pick_method` ise her aday için önce doğal, sonra trait adaylarına bakar.[^probe] Bölümde anlattığımız sıra bu fonksiyonların sırasıdır.

**Düzeltme.** Aday seçildikten sonra `confirm.rs` içindeki `adjust_self_ty`, kaç kez açılacağını ve hangi referansın ekleneceğini ifadeye *düzeltme* (adjustment) olarak kaydeder.[^confirm] Sonraki aşamalar sizin yazdığınız `v.push(4)` yerine düzeltilmiş hâli görür. Ödünç denetleyicisinin `E0596` hatası da bu eklenen `&mut` içindir.

**Örtük dönüşüm.** Deref coercion başka bir yerden, `coercion.rs` içindeki `coerce_to_ref` fonksiyonundan gelir; hedef referans türüne ulaşana kadar `Autoderef` ile zinciri dener.[^coerce] Alıcının neden ayrı kurallara uyduğu buradan da görülür: İkisi ayrı koddur.

**`Deref`'in yeri.** Derleyici `Deref`'i özel olarak tanır. Referans ve ham işaretçide `*` yerleşiktir; öteki türlerde `overloaded_deref_ty` türün `Deref` uygulayıp uygulamadığına bakar ve `Target`'ı bulur.[^autoderef]

## Özet

- `Deref`, `*x`'in anlamını belirler: `*k`, `*Deref::deref(&k)` demektir. Bir türün tek `Target`'ı olur.
- Deref coercion `&T`'yi `&U`'ya çevirir ve zincirlenir; hangi `deref` çağrılarının gerektiği derleme zamanında belirlenir.
- `x.metot()` için aday listesi `U`, `&U`, `&mut U` sırasıyla, dıştan içe taranır; her adayda önce doğal, sonra trait metotlarına bakılır ve ilk uyan seçilir.
- Autoref ödünç kurallarına tabidir: `mut` olmayan değere `&mut self` metodu çağrılamaz.
- Dıştaki metot içtekini sessizce gölgeler; içtekini `(*s).ad()` ya da `Ic::ad(&s)` ile çağırın.
- `Deref` kalıtım değildir: Metot ve alan erişimi kazandırır, trait sınırını karşılamaz.

Sıradaki bölüm [25. `Send` ve `Sync`'in İç Yüzü](25-send-sync.md), thread güvenliğinin hangi türler için nasıl kendiliğinden belirlendiğini anlatıyor.

## Kaynaklar

[^deref]: The Rust Programming Language, [Implementing the `Deref` Trait](https://doc.rust-lang.org/book/ch15-02-deref.html#implementing-the-deref-trait); std, [`Deref`](https://doc.rust-lang.org/std/ops/trait.Deref.html).
[^ozel]: The Rust Reference, [`Box<T>`](https://doc.rust-lang.org/reference/special-types-and-traits.html#boxt); hata dizini, [`E0507`](https://doc.rust-lang.org/error_codes/E0507.html).
[^coercion]: std, [Deref coercion](https://doc.rust-lang.org/std/ops/trait.Deref.html#deref-coercion); The Rust Reference, [Coercion types](https://doc.rust-lang.org/reference/type-coercions.html#coercion-types).
[^book-coercion]: The Rust Programming Language, [Using Deref Coercion in Functions and Methods](https://doc.rust-lang.org/book/ch15-02-deref.html#using-deref-coercion-in-functions-and-methods).
[^yerler]: The Rust Reference, [Coercion sites](https://doc.rust-lang.org/reference/type-coercions.html#coercion-sites).
[^mut]: The Rust Programming Language, [Handling Deref Coercion with Mutable References](https://doc.rust-lang.org/book/ch15-02-deref.html#handling-deref-coercion-with-mutable-references); std, [`DerefMut`](https://doc.rust-lang.org/std/ops/trait.DerefMut.html).
[^cagri]: The Rust Reference, [Method-call expressions](https://doc.rust-lang.org/reference/expressions/method-call-expr.html#method-call-expressions); hata dizini, [`E0596`](https://doc.rust-lang.org/error_codes/E0596.html) ve [`E0034`](https://doc.rust-lang.org/error_codes/E0034.html).
[^ok]: The Rust Programming Language, [Where's the `->` Operator?](https://doc.rust-lang.org/book/ch05-03-method-syntax.html#wheres-the---operator).
[^rfr]: Jon Gjengset, *Rust for Rustaceans*, 3. bölüm ("Designing Interfaces"), "Wrapper Types".
[^akilli]: Rust API Guidelines, [Smart pointers do not add inherent methods](https://rust-lang.github.io/api-guidelines/predictability.html#smart-pointers-do-not-add-inherent-methods-c-smart-ptr).
[^yalniz]: Rust API Guidelines, [Only smart pointers implement `Deref` and `DerefMut`](https://rust-lang.github.io/api-guidelines/predictability.html#only-smart-pointers-implement-deref-and-derefmut-c-deref); std, [When to implement `Deref` or `DerefMut`](https://doc.rust-lang.org/std/ops/trait.Deref.html#when-to-implement-deref-or-derefmut).
[^probe]: rustc 1.99.0 kaynak kodu, [`probe.rs`, `method_autoderef_steps`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_typeck/src/method/probe.rs#L625), [`pick_all_method`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_typeck/src/method/probe.rs#L1314) ve [`pick_method`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_typeck/src/method/probe.rs#L1709).
[^confirm]: rustc 1.99.0 kaynak kodu, [`confirm.rs`, `adjust_self_ty`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_typeck/src/method/confirm.rs#L175).
[^coerce]: rustc 1.99.0 kaynak kodu, [`coercion.rs`, `coerce_to_ref`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_typeck/src/coercion.rs#L412).
[^autoderef]: rustc 1.99.0 kaynak kodu, [`autoderef.rs`, `overloaded_deref_ty`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_analysis/src/autoderef.rs#L137); [`deref.rs`, `trait Deref`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/ops/deref.rs#L139).
