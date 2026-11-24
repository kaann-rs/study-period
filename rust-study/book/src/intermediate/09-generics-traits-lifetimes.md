# 9. Generic, Trait ve Lifetime

Aynı işi farklı türler için tekrar tekrar yazmak yalnızca yorucu değildir: Bir kopyadaki hatayı düzeltip ötekini unutmak kolaylaşır. Rust bu tekrarı üç aracın yardımıyla ortadan kaldırır. *Jenerikler* (generics) tek bir kod gövdesinin birçok türle çalışmasını sağlar. *Trait*'ler, o türlerden beklenen ortak davranışı tanımlar. *Ömürler* (lifetimes) ise referansların ne kadar süre geçerli kaldığını derleyiciye bildirir.[^ch10]

Üç konunun aynı bölümde yer almasının nedeni, üçünün de aynı amaca hizmet etmesidir: tekrarı azaltmak ve güvenliği derleme zamanında korumak. Aralarındaki bağ bundan da yakındır. Trait'ler, jenerik kodun hangi türleri kabul edeceğini belirler; ömürler ise bir türün değil, bir sürenin yerini tutan jenerik parametrelerdir.[^omur-jenerik]

Bu bölümde önce bir fonksiyonu ve bir yapıyı türünden bağımsız yazmayı, ardından türlerden davranış istemeyi, son olarak da referansların geçerliliğini imzada anlatmayı göreceğiz. İmzaları yazarken verilen tasarım kararları [9.1 Generic, Trait ve Lifetime: Derinleşme](09-generics-traits-lifetimes-deep-dive.md) sayfasının konusudur.

> [!NOTE]
> Bu bölümün kodu `learn/generics-traits-lifetimes` dizinindedir. Depo kökünden `cargo run -p generics-traits-lifetimes` komutuyla çalıştırabilirsiniz. `Color` örneği `learn/basic-concepts` dizinindeki koddan uyarlanmıştır.

## Günlük hayattan bir benzetme

Okulda herkesin kullandığı tek tip bir kâğıt form olduğunu düşünün. Formun ad yeri boştur; adını yazan herkes onu kullanabilir, kimse için ayrı form basılmaz. *Jenerik* kod bu formdur: Aynı tarif bir kez yazılır, türün adı sonradan doldurulur.

Formun üstünde "imza atabilen doldurur" yazar. Kimin geldiği önemli değildir; kalemi olup imza atabilen herkes formu doldurabilir. Bu koşul bir *trait*'tir: Bir türden beklenen davranışı, yani türün "şunu yapabilirim" diye verdiği sözü anlatır.

Formun altında bir not daha vardır: "Bu form yalnız bugün geçerlidir." Yarın olunca form işe yaramaz. Bu not *ömür* bilgisidir; ödünç verilen verinin ne zamana kadar geçerli olduğunu söyler.

Bölüm boyunca aynı okulda kalacağız: Sınıftaki oyun, sunum belgesi ve kütüphane kitapları, sırası gelen kavramı anlatmak için yeniden karşınıza çıkacak.

<details>
<summary>Bu bölümde geçen terimler</summary>

- **jenerik (generic):** Türü boş bırakılmış kod: "Hangi tür gelirse onunla çalış."
- **tür parametresi (`T`):** Boş bırakılan yerin adı. `T` yerine `i32` de gelebilir, `char` da.
- **somut tür (concrete type):** Boşluğa gerçekten konan tür: `i32`, `String`, `Haber` gibi.
- **const jenerik (const generic):** Tür değil, derleme zamanında bilinen bir değer alan parametre; örneğin bir dizinin uzunluğu.
- **trait:** Bir türün verdiği söz; metot imzalarının listesi. Başka dillerde benzer kavramın adı arayüzdür (interface).
- **imza (signature):** Fonksiyonun ilk satırı: adı, ne aldığı ve ne döndürdüğü.
- **`impl`:** "Bu tür için şu metotları yazıyorum" bloğu.
- **varsayılan gövde (default implementation):** Trait'in içinde hazır gelen metot gövdesi. Tür isterse kendi gövdesini yazar; buna ezme (override) denir.
- **trait sınırı (trait bound):** "`T` her tür olamaz, şu sözü vermiş olmalı" koşulu: `T: PartialOrd`.
- **`impl Trait`:** "Şu sözü veren herhangi bir tür" demenin kısa yazımı.
- **gönderim (dispatch):** "Hangi türün metodu çağrılacak?" sorusunun yanıtlanması. Yanıt derlenirken bulunuyorsa statik, program çalışırken bulunuyorsa dinamik gönderim denir.
- **monomorfizasyon (monomorphization):** Derleyicinin jenerik kodu, kullanılan her somut tür için ayrı ayrı üretmesi.
- **trait nesnesi (trait object, `dyn Trait`):** Asıl türü unutulmuş, yalnızca verdiği söz bilinen değer.
- **vtable:** Trait nesnesinin yanında taşınan, "bu iş şu fonksiyonla yapılır" diyen çizelge (sanal metot tablosu).
- **`Box`:** Değeri heap'e koyan ve onu gösteren tek işaretçiyi tutan kutu.
- **dolaylama (indirection):** Veriye doğrudan değil, önce bir işaretçiyi izleyerek ulaşmak.
- **satır içine alma (inlining):** Fonksiyon gövdesinin çağrıldığı yere yerleştirilmesi; çağrının kendisi ortadan kalkar.
- **homojen, heterojen:** Elemanlarının hepsi aynı türden olan; elemanları farklı türlerden olan.
- **genel impl (blanket implementation):** "Şu sözü veren her türe bu trait'i de ver" diyen tek kural.
- **yetim kuralı (orphan rule):** Trait de tür de başka crate'lere aitse o ikisi için `impl` yazılamaması.
- **ömür (lifetime, `'a`):** Bir referansın geçerli kaldığı kod bölgesi; `'a` o bölgenin adıdır.
- **ömür çıkarımı (elision):** Belli olan ömrü derleyicinin sizin yerinize doldurması.
- **`'static`:** Program boyunca geçerli kalan ömür.
- **literal:** Koda elle yazılmış sabit değer: `"merhaba"`, `42` gibi.
- **sarkan referans (dangling reference):** Gösterdiği veri artık geçerli olmayan referans.
- **sıfır maliyet (zero-cost):** Kullanılan kolaylığın çalışan programı yavaşlatmaması.

</details>

## Jenerikler: Türü sonraya bırakmak

Sınıfta "en uzununu bul" oyunu oynandığını düşünün. Oyun öğrencilerle de oynanır, kalemlerle de, iplerle de. Kural hep aynıdır: İkisini yan yana koyar, uzun olanı tutarsınız. Oyunu her eşya için yeniden anlatmazsınız.

Kodda ise aynı kuralı her tür için yeniden yazmak kolaydır. Aşağıdaki iki fonksiyon, adları ve türleri dışında birbirinin aynısıdır:

```rust
fn en_buyuk_i32(liste: &[i32]) -> &i32 {
    let mut en = &liste[0];
    for x in liste {
        if x > en {
            en = x;
        }
    }
    en
}

fn en_buyuk_char(liste: &[char]) -> &char {
    let mut en = &liste[0];
    for x in liste {
        if x > en {
            en = x;
        }
    }
    en
}

fn main() {
    println!("{} {}", en_buyuk_i32(&[3, 7, 2]), en_buyuk_char(&['a', 'z']));
}
```

Çıktı:

```text
7 z
```

Gövdedeki bir hatayı düzelttiğinizde aynı düzeltmeyi öteki kopyada da yapmanız gerekir. Jenerikler bu iki gövdeyi tek gövdeye indirir.

### Jenerik fonksiyonlar

Fonksiyon adından sonra açılı parantez içinde bir *tür parametresi* bildirilir. Parametreye istediğiniz adı verebilirsiniz; tek harfli `T`, "type" sözcüğünün kısaltması olarak yaygındır:[^gen-fn]

```rust
fn en_buyuk<T: PartialOrd>(liste: &[T]) -> &T {
    let mut en = &liste[0];
    for x in liste {
        if x > en {
            en = x;
        }
    }
    en
}

fn main() {
    println!("en büyük (i32)  = {}", en_buyuk(&[3, 7, 2, 9, 4]));
    println!("en büyük (char) = {}", en_buyuk(&['a', 'z', 'm']));
}
```

Çıktı:

```text
en büyük (i32)  = 9
en büyük (char) = z
```

İmza şöyle okunur: "`en_buyuk`, karşılaştırılabilen herhangi bir `T` türü için çalışır; `T` dilimi alır, bir `T` referansı döndürür." Çağırırken türü yazmazsınız. Derleyici ilk çağrıda `T`'nin `i32`, ikincisinde `char` olduğunu argümanlardan bulur. `i32` ve `char` burada `T`'nin yerine geçen *somut türlerdir*.

Her jenerik fonksiyonun koşula ihtiyacı yoktur. Aşağıdaki fonksiyon elemanlara yalnızca referans verir, onları karşılaştırmaz; bu yüzden `T` için hiçbir şey istemez:

```rust
fn ilki<T>(liste: &[T]) -> &T {
    &liste[0]
}

fn main() {
    println!("{}", ilki(&[3, 7]));
    println!("{}", ilki(&['a', 'z']));
}
```

Çıktı:

```text
3
a
```

> [!WARNING]
> `en_buyuk` ile `ilki`, boş bir dilim verildiğinde `liste[0]` satırında panikler: `index out of bounds: the len is 0 but the index is 0`. Boş dilim olağan bir durumsa imzayı `Option<&T>` döndürecek biçimde yazın; standart kütüphanedeki `first` metodu da böyle yapar.[^slice-first]

### Trait sınırı: `T` her tür olamaz

`en_buyuk` imzasındaki `: PartialOrd` kısmını silerseniz program derlenmez. Daha kısa bir fonksiyonda görelim:

```rust,compile_fail,E0369
fn buyuk<T>(a: T, b: T) -> bool {
    a > b
}
```

Derleyicinin yanıtı:

```text
error[E0369]: binary operation `>` cannot be applied to type `T`
  |
2 |     a > b
  |     - ^ - T
  |     |
  |     T
  |
help: consider restricting type parameter `T` with trait `PartialOrd`
  |
1 | fn buyuk<T: std::cmp::PartialOrd>(a: T, b: T) -> bool {
  |           ++++++++++++++++++++++
```

Her şey yan yana konup karşılaştırılamaz: Kokuları "hangisi büyük" diye sıralayamazsınız. Rust'ta `a > b` yazımı, `PartialOrd` trait'indeki `gt` metodunun çağrısıdır.[^operator] Derleyici `T` hakkında hiçbir şey bilmediği için bu metodun var olduğunu varsayamaz. Koşulu imzaya yazmak yeterlidir:

```rust
fn buyuk<T: PartialOrd>(a: T, b: T) -> bool {
    a > b
}

fn main() {
    println!("{} {}", buyuk(3, 7), buyuk('z', 'a'));
}
```

Çıktı:

```text
false true
```

`T: PartialOrd` yazımına *trait sınırı* (trait bound) denir: "`T` yerine yalnızca `PartialOrd` uygulayan türler gelebilir." Sınırların ayrıntısına trait'leri tanıdıktan sonra döneceğiz.

### Jenerik struct'lar

Etiketi boş bırakılmış, iki gözlü bir kalem kutusu düşünün. İçine ne konacağına siz karar verirsiniz; ama iki göze de aynı türden şey girer. Jenerik bir struct böyle bir kalıptır:[^gen-struct]

```rust
struct Cift<T> {
    sol: T,
    sag: T,
}

fn main() {
    let a = Cift { sol: 1, sag: 2 }; // T = i32
    let b = Cift { sol: 1.5, sag: 2.5 }; // T = f64
    println!("{} {} / {} {}", a.sol, a.sag, b.sol, b.sag);
}
```

Çıktı:

```text
1 2 / 1.5 2.5
```

Jenerik olmasaydı `CiftI32`, `CiftF64` diye aynı struct'ı her tür için yeniden yazmanız gerekirdi. İki alan aynı `T`'yi paylaştığından farklı türler karıştırılamaz:

```rust,compile_fail,E0308
# struct Cift<T> {
#     sol: T,
#     sag: T,
# }
fn main() {
    let c = Cift { sol: 1, sag: 2.5 };
}
```

Derleyicinin yanıtı:

```text
error[E0308]: mismatched types
  |
2 |     let c = Cift { sol: 1, sag: 2.5 };
  |                                 ^^^ expected integer, found floating-point number
```

`sol: 1` alanı `T`'yi bir tam sayı türü olarak sabitler; `sag` alanı da aynı türden olmak zorundadır. Gözlerin farklı türler alabilmesini istiyorsanız iki ayrı parametre bildirirsiniz: `struct Cift<S, G> { sol: S, sag: G }`. Enum'lar da jenerik olabilir; `Option<T>` ve `Result<T, E>` bu yolla tanımlanmıştır.[^gen-enum]

### Jenerik metotlar

Jenerik bir struct'a metot yazarken tür parametresi `impl` sözcüğünden sonra da bildirilir. `impl<T> Nokta<T>` yazımı "her `T` için bu metot" anlamına gelir. Belirli bir somut tür için metot yazmak da mümkündür; o durumda `impl` sözcüğünün yanına parametre konmaz:[^gen-metot]

```rust
#[derive(Debug)]
struct Nokta<T> {
    x: T,
    y: T,
}

impl<T> Nokta<T> {
    fn x(&self) -> &T {
        &self.x
    }
}

// yalnızca f64 tutan noktalar için
impl Nokta<f64> {
    fn uzaklik(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

fn main() {
    let ni = Nokta { x: 3, y: 4 };
    let nf = Nokta { x: 3.0, y: 4.0 };
    println!("ni.x = {}, nf.uzaklık = {}", ni.x(), nf.uzaklik());
    println!("ni = {ni:?} (y = {})", ni.y);
}
```

Çıktı:

```text
ni.x = 3, nf.uzaklık = 5
ni = Nokta { x: 3, y: 4 } (y = 4)
```

`uzaklik` metodu tam sayı tutan noktada yoktur:

```rust,compile_fail,E0599
# struct Nokta<T> {
#     x: T,
#     y: T,
# }
# impl Nokta<f64> {
#     fn uzaklik(&self) -> f64 {
#         (self.x * self.x + self.y * self.y).sqrt()
#     }
# }
fn main() {
    let ni = Nokta { x: 3, y: 4 };
    ni.uzaklik();
}
```

Derleyicinin yanıtı:

```text
error[E0599]: no method named `uzaklik` found for struct `Nokta<{integer}>` in the current scope
  |
3 |     ni.uzaklik();
  |        ^^^^^^^ method not found in `Nokta<{integer}>`
  |
  = note: the method was found for `Nokta<f64>`
```

`uzaklik` her `T` için yazılamaz, çünkü karekök her türde yoktur: `sqrt` metodu `f64`'te ve `f32`'de bulunur, tam sayılarda bulunmaz.[^sqrt] Tam sayıların, sonucu aşağı yuvarlayan ayrı bir metodu vardır: `isqrt`.[^isqrt]

`impl Nokta<T>` yazıp `<T>` bildirimini unutursanız derleyici `T`'yi somut bir türün adı sanar, öyle bir tür bulamayınca E0425 verir: ``cannot find type `T` in this scope``. Parametreyi `impl` sözcüğünden sonra bildirmek, `T`'nin bir boşluk adı olduğunu söyler.

> [!NOTE]
> Son `println!` satırı `ni.y` alanını ayrıca okur. Okumasaydı derleyici, `Debug` çıktısında göründüğü hâlde `y` alanı için ``field `y` is never read`` uyarısı verirdi: Kullanılmayan kod denetimi, türetilmiş (`derive`) `Debug` uygulamasını bilerek hesaba katmaz.[^dead-code]

### Değer alan parametreler: const jenerikler

Jenerik parametreler yalnızca tür olmak zorunda değildir. *Const jenerik* (const generic), derleme zamanında bilinen bir *değer* alır; en sık kullanıldığı yer dizi uzunluğudur:[^const-gen]

```rust
struct Buffer<const N: usize = 0> {
    data: [u8; N],
}

impl<const N: usize> Buffer<N> {
    fn yeni() -> Self {
        Buffer { data: [0; N] }
    }
}

fn toplam<const N: usize>(dizi: [u8; N]) -> u32 {
    dizi.iter().map(|b| u32::from(*b)).sum()
}

fn main() {
    let dort = Buffer::<4>::yeni();
    let bos: Buffer = Buffer::yeni(); // argüman yazılmadı: N = 0
    println!("uzunluklar: {} {}", dort.data.len(), bos.data.len());
    println!("boyut: {} bayt", size_of::<Buffer<4>>());
    println!("toplamlar: {} {}", toplam([1, 2, 3, 4]), toplam([1; 8]));
}
```

Çıktı:

```text
uzunluklar: 4 0
boyut: 4 bayt
toplamlar: 10 8
```

`[u8; 4]` ile `[u8; 8]` iki ayrı türdür. Const jenerikler olmasaydı her uzunluk için ayrı kod ya da makro yazmanız, ya da heap üzerindeki `Vec`'e geçmeniz gerekirdi. `Vec`'in uzunluğu ise ancak çalışma zamanında bilinir. Const jenerik, uzunluğu türün bir parçası yapar: `Buffer<4>` dört baytlık bir dizidir, boyutu da tam dört bayttır. Uzunluk bellekte ayrıca saklanmaz.

Parametrenin türü tam sayı, `bool` ya da `char` olabilir; `f64` gibi türler kabul edilmez.[^const-gen] `= 0` yazımı varsayılan değerdir; tür ve trait tanımlarında kullanılabilir, fonksiyonlarda ve `impl` bloklarında kullanılamaz. Varsayılan, tür yazıldığında ve argüman verilmediğinde devreye girer; yukarıdaki `let bos: Buffer` satırı bu yüzden sıfır uzunluklu bir tampon kurar. Yalnızca `let bos = Buffer::yeni();` yazsaydınız derleyici varsayılanı kullanmaz, E0284 (`type annotations needed`) ile uzunluğu sorardı.

Const jenerikler Rust 1.51'de, varsayılan değerleri 1.59'da kararlı oldu.[^const-151][^const-159] Bir sınırı hâlâ vardır: Tür içinde `N` tek başına yazılmalıdır, `[u8; N + 1]` gibi bir hesap kararlı Rust'ta derlenmez.[^const-gen]

## Trait'ler: Ortak davranış

Okulda "sunum yapabilir" diye bir belge olduğunu düşünün. Belgeyi alan herkes, "konunu söyle" denince konusunu söyleyebilmek zorundadır; nasıl söyleyeceği kendisine kalmıştır. Belgede bir de hazır kapanış cümlesi yazar. Kendi cümlesini yazmayan bu cümleyi kullanır.

Trait bu belgedir. Metotların adını ve biçimini belirler, gövdelerini her tür kendisi yazar. Belge olmasaydı farklı türlerle aynı işi yapan kod yazamazdınız; her tür için ayrı bir fonksiyon gerekirdi.

### Trait tanımlamak ve uygulamak

Bir trait, metot imzalarının kümesidir.[^trait-tanim] Gövdesiz imzalar zorunludur; gövdesi yazılmış olanlar *varsayılan gövde* sayılır ve hazır gelir:

```rust
trait Sunum {
    fn konu(&self) -> String; // zorunlu
    fn kapanis(&self) -> String {
        String::from("Teşekkürler") // varsayılan gövde
    }
}

struct Ali;

impl Sunum for Ali {
    fn konu(&self) -> String {
        String::from("uzay")
    }
}

fn main() {
    println!("{} / {}", Ali.konu(), Ali.kapanis());
}
```

Çıktı:

```text
uzay / Teşekkürler
```

`Ali` yalnızca `konu` metodunu yazdı; `kapanis` çağrıldığında hazır gövde çalıştı. Zorunlu bir metodu yazmayı unutmak ise derleme hatasıdır:

```rust,compile_fail,E0046
# trait Sunum {
#     fn konu(&self) -> String;
#     fn kapanis(&self) -> String {
#         String::from("Teşekkürler")
#     }
# }
struct Veli;

impl Sunum for Veli {}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0046]: not all trait items implemented, missing: `konu`
  |
3 | impl Sunum for Veli {}
  | ^^^^^^^^^^^^^^^^^^^ missing `konu` in implementation
```

### Varsayılan gövdeyi kullanmak ya da ezmek

Varsayılan gövde, aynı trait'in öteki metotlarını çağırabilir; çağırdığı metodun gövdesi yazılmamış olsa bile.[^varsayilan] Aşağıdaki `ozetle` metodu `yazar`'a dayanır:

```rust
trait Ozet {
    fn yazar(&self) -> String;
    fn ozetle(&self) -> String {
        format!("(devamı...) — {}", self.yazar())
    }
}

struct Haber {
    baslik: String,
    yazar: String,
}

struct Tweet {
    kullanici: String,
}

impl Ozet for Haber {
    fn yazar(&self) -> String {
        self.yazar.clone()
    }
    fn ozetle(&self) -> String {
        format!("{} — {}", self.baslik, self.yazar)
    }
}

impl Ozet for Tweet {
    fn yazar(&self) -> String {
        format!("@{}", self.kullanici)
    }
}

fn main() {
    let h = Haber { baslik: String::from("Rust 2024"), yazar: String::from("Kaan") };
    let t = Tweet { kullanici: String::from("kaan") };
    println!("{}", h.ozetle());
    println!("{}", t.ozetle());
}
```

Çıktı:

```text
Rust 2024 — Kaan
(devamı...) — @kaan
```

`Haber` hazır gövdeyi *ezdi* (override) ve kendi `ozetle` metodunu yazdı. `Tweet` yalnızca zorunlu olan `yazar` metodunu verdi; `ozetle` çağrısında hazır gövde çalıştı ve `Tweet`'in `yazar` metodunu kullandı.

Trait'ler başka dillerdeki arayüzlere (interface) benzer, ama aynı şey değildir.[^trait-tanim] Örneğin bir trait, türün tanımından ayrı bir yerde, hatta başkasının yazdığı bir tür için bile uygulanabilir. Bunun sınırını [yetim kuralı](#yetim-kuralı) başlığında göreceğiz.

### Parametre olarak trait istemek

Öğretmen "sunum belgesi olan herkes tahtaya gelebilir" der. Kimin geldiğine bakmaz, belgeye bakar. `&impl Sunum` tam olarak bunu söyler: "Türün ne olduğu önemli değil, `Sunum` sözünü vermiş olsun."

```rust
# trait Sunum {
#     fn konu(&self) -> String;
# }
# struct Ali;
# impl Sunum for Ali {
#     fn konu(&self) -> String {
#         String::from("uzay")
#     }
# }
fn tahtaya_cagir(x: &impl Sunum) {
    println!("{}", x.konu());
}

fn main() {
    tahtaya_cagir(&Ali);
}
```

Çıktı:

```text
uzay
```

Fonksiyon içeride `konu` metodunu çağırır. Derleyici bu metodun gelen her türde bulunduğundan emin olmak ister; belgesi olmayanı kapıdan çevirir:

```rust,compile_fail,E0277
# trait Sunum {
#     fn konu(&self) -> String;
# }
# fn tahtaya_cagir(x: &impl Sunum) {
#     println!("{}", x.konu());
# }
fn main() {
    tahtaya_cagir(&5);
}
```

Derleyicinin yanıtı:

```text
error[E0277]: the trait bound `{integer}: Sunum` is not satisfied
  |
2 |     tahtaya_cagir(&5);
  |     ------------- ^^ the trait `Sunum` is not implemented for `{integer}`
  |     |
  |     required by a bound introduced by this call
```

Aynı isteğin daha uzun ve daha esnek yazımı, tür parametresine sınır koymaktır:[^sinir]

```rust
# trait Ozet {
#     fn yazar(&self) -> String;
#     fn ozetle(&self) -> String {
#         format!("(devamı...) — {}", self.yazar())
#     }
# }
# struct Haber { baslik: String, yazar: String }
# struct Tweet { kullanici: String }
# impl Ozet for Haber {
#     fn yazar(&self) -> String { self.yazar.clone() }
#     fn ozetle(&self) -> String { format!("{} — {}", self.baslik, self.yazar) }
# }
# impl Ozet for Tweet {
#     fn yazar(&self) -> String { format!("@{}", self.kullanici) }
# }
fn duyur(x: &impl Ozet) {
    println!("Yeni: {}", x.ozetle());
}

// duyur ile aynı isteği tür parametresiyle yazar
fn _duyur_uzun<T: Ozet>(x: &T) {
    println!("Yeni: {}", x.ozetle());
}

fn main() {
    let h = Haber { baslik: String::from("Rust 2024"), yazar: String::from("Kaan") };
    let t = Tweet { kullanici: String::from("kaan") };
    duyur(&h);
    duyur(&t);
    _duyur_uzun(&t);
}
```

Çıktı:

```text
Yeni: Rust 2024 — Kaan
Yeni: (devamı...) — @kaan
Yeni: (devamı...) — @kaan
```

İki yazım hemen hemen eşdeğerdir: `impl Ozet`, adı olmayan bir tür parametresidir. Aralarında iki fark vardır. Adlandırılmış parametrede tür çağrı sırasında açıkça verilebilir (`_duyur_uzun::<Tweet>(&t)`), `impl Trait` yazımında verilemez.[^impl-trait] Ayrıca iki parametrenin *aynı* türden olması gerekiyorsa adlandırılmış parametre yazmak zorundasınız: `fn esit<T: Ozet>(a: &T, b: &T)` iki `Haber` ya da iki `Tweet` ister, `fn serbest(a: &impl Ozet, b: &impl Ozet)` ise farklı türleri de kabul eder.[^sinir] Crate'in kodunda bu fonksiyon (orada `_announce_long`) hiç çağrılmaz; adının başındaki alt çizgi, kullanılmayan fonksiyon uyarısını susturur.

Birden çok trait istemek için `+` kullanılır. Sınırlar uzadığında onları imzanın sonundaki `where` kısmına taşımak okumayı kolaylaştırır:[^where]

```rust
use std::fmt::Display;

fn buyugu_yazdir<T>(a: T, b: T)
where
    T: PartialOrd + Display,
{
    let buyuk = if a > b { a } else { b };
    println!("büyük olan: {buyuk}");
}

fn main() {
    buyugu_yazdir(3, 7);
    buyugu_yazdir("elma", "armut");
}
```

Çıktı:

```text
büyük olan: 7
büyük olan: elma
```

`impl Trait` dönüş türünde de yazılabilir: `fn haber_ver() -> impl Ozet`. O durumda türü çağıran değil fonksiyon seçer ve gövdenin her dalı aynı somut türü döndürmek zorundadır.[^donus]

### Sınırı olmayan jenerik ne yapabilir?

Trait sınırının neden gerektiğini görmek için sınırı kaldıralım. Aşağıdaki `Color<F>` yapısı, çağrılabilir bir değer saklamak ister:

```rust,compile_fail,E0618
struct Color<F> {
    x: F,
}

impl<F> Color<F> {
    fn boya(&self, deger: i8) -> i8 {
        (self.x)(deger)
    }
}
```

Derleyicinin yanıtı:

```text
error[E0618]: expected function, found `F`
  |
7 |         (self.x)(deger)
  |         ^^^^^^^^-------
  |         |
  |         call expression requires function
```

Derleyici `F` hakkında hiçbir şey bilmez; belki bir fonksiyondur, belki bir `String`. Böyle bir değer taşınabilir, ödünç alınabilir, düşürülebilir, boyutu öğrenilebilir (tür parametreleri, aksi söylenmedikçe `Sized` kabul edilir);[^sized] ama üzerinde metot çağrılamaz, kendisi fonksiyon gibi kullanılamaz. Sınır, `F`'nin sözleşmesini baştan yazar: "`i8` ile çağrılır, `i8` döndürür."

```rust
struct Color<F>
where
    F: Fn(i8) -> i8,
{
    x: F,
}

impl<F> Color<F>
where
    F: Fn(i8) -> i8,
{
    fn boya(&self, deger: i8) -> i8 {
        (self.x)(deger)
    }
}

fn main() {
    let k = 3;
    let renk = Color { x: |y| y + k };
    println!("{}", renk.boya(4));
}
```

Çıktı:

```text
7
```

`|y| y + k` bir closure'dır: çevresindeki `k` değişkenini yakalayan adsız fonksiyon. `Fn(i8) -> i8` sınırı, değerin paylaşımlı bir referans üzerinden `i8` ile çağrılabilmesini şart koşar;[^fn-trait] bu closure da koşulu karşılar. `Fn`, `FnMut` ve `FnOnce` ayrımı [12. Closure ve Iterator'lar](12-closures-iterators.md) bölümündedir.

Sınır iki tarafa da yarar. Yapıyı *yazan* taraf `x`'i çağırabilir; *kullanan* taraf uygun olmayan bir değer verdiğinde hatayı kendi satırında görür. `Color { x: 5 }` yazarsanız derleyici ``expected an `Fn(i8)` closure, found `{integer}` `` (E0277) der. Sınır yazılmayan şablon sistemlerinde (örneğin C++20 öncesinin şablonlarında) aynı yanlış, şablonun içindeki bir satırda ve okunması zor bir mesajla ortaya çıkardı.[^cpp]

### Türün kendi impl'i ve trait impl'i

Şimdiye kadar iki tür `impl` bloğu gördük. `impl Ozet for Haber` bir trait'i uygular. `impl<F> Color<F>` ise metotları doğrudan türün kendisine ekler; buna türün *kendi impl'i* (inherent impl) denir. İkincisinde trait'i kapsama almak gerekmez, metot türle birlikte gelir. Karşılığında bir kısıt vardır: Kendi impl'i yalnızca türün tanımlandığı crate'te yazılabilir;[^inherent] başkasının türüne doğrudan metot eklemek E0116 hatasıdır.

Kendi impl'ine, `self` almayan fonksiyonlar da konabilir. Bunlara *ilişkili fonksiyon* (associated function) denir ve bir değer üzerinden değil, tür adıyla çağrılırlar; `String::new()` böyle bir çağrıdır.[^iliskili]

```rust
# struct Color<F>
# where
#     F: Fn(i8) -> i8,
# {
#     x: F,
# }
impl<F> Color<F>
where
    F: Fn(i8) -> i8,
{
    // self almaz: Color::hex(..) diye çağrılır
    fn hex(f: F, _b: fn(i8) -> i8) -> i8 {
        f(1)
    }
}

fn main() {
    let x = |y: i8| -> i8 { y };
    let a: i8 = Color::hex(x, x);
    println!("{a}");
}
```

Çıktı:

```text
1
```

`hex`'in ilk parametresi jenerik `F`'dir; ikincisi somut bir türdür: `fn(i8) -> i8`, yani fonksiyon işaretçisi. Aynı `x` iki yerde iki farklı türle kullanılır: Birincide closure kendi türünü korur ve `F`'yi belirler, ikincide hiçbir şey yakalamadığı için fonksiyon işaretçisine dönüşür. İki yazımın ne zaman seçileceği [19. Gelişmiş Özellikler](19-advanced-features.md) bölümünün konusudur.

### Genel impl: bir kuralla birçok tür

Okulun bir kuralı vardır: "Okumayı bilen herkes kütüphane kartı alır." Kart her öğrenci için ayrı ayrı yazılmaz; tek kural hepsini kapsar, okumayı sonradan öğrenen de kendiliğinden kart sahibi olur. *Genel impl* (blanket implementation) böyle bir kuraldır:[^genel]

```rust
use std::fmt::Display;

trait Etiket {
    fn etiket(&self) -> String;
}

impl<T: Display> Etiket for T {
    fn etiket(&self) -> String {
        format!("[{self}]")
    }
}

fn main() {
    println!("{} {}", 42.etiket(), "ada".etiket());
}
```

Çıktı:

```text
[42] [ada]
```

`impl<T: Display> Etiket for T` satırı "`Display` sözünü veren her türe `Etiket`'i de ver" der. Bu kural olmasaydı `i32`, `String` ve `char` için aynı `impl`'i tek tek yazmanız gerekirdi. `Vec`, `Display` uygulamadığı için kuralın dışında kalır:

```rust,compile_fail,E0599
# use std::fmt::Display;
# trait Etiket {
#     fn etiket(&self) -> String;
# }
# impl<T: Display> Etiket for T {
#     fn etiket(&self) -> String {
#         format!("[{self}]")
#     }
# }
fn main() {
    vec![1].etiket();
}
```

Derleyicinin yanıtı:

```text
error[E0599]: the method `etiket` exists for struct `Vec<{integer}>`, but its trait bounds were not satisfied
  |
2 |     vec![1].etiket();
  |             ^^^^^^
```

Standart kütüphane aynı kalıbı kullanır: `to_string` metodu, `Display` uygulayan her türe `ToString` veren genel impl'den gelir.[^tostring] Bu yüzden kendi türünüz için `Display` yazdığınızda `to_string` de kendiliğinden kullanılabilir olur.

### Yetim kuralı

Bir trait'i her tür için uygulayamazsınız. Trait ya da türden en az biri kendi crate'inize ait olmalıdır; ikisi de başkasınınsa `impl` reddedilir:[^yetim]

```rust,compile_fail,E0117
use std::fmt;

impl fmt::Display for Vec<i32> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.len())
    }
}
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
  = note: define and implement a trait or new type instead
```

Bu *yetim kuralı* (orphan rule), tutarlılık (coherence) denen daha genel bir ilkenin parçasıdır. Kural olmasaydı iki ayrı crate aynı tür için aynı trait'i farklı biçimde uygulayabilir, derleyici de hangisini kullanacağını bilemezdi.[^yetim] `Vec<i32>`'yi kendi crate'inizdeki tek alanlı bir yapıya sararsanız (*newtype*) sarmalayan tür size ait olduğu için `impl` yazılabilir.[^newtype] Kuralın jenerik türlerdeki ayrıntıları [23. Tutarlılık ve Yetim Kuralı](../advanced/23-coherence.md) bölümündedir.

### Statik ve dinamik gönderim

Sunum günü için iki düzen kurulabilir. Birincisinde "bu saatte yalnız Ali sunacak" denir: Kimin geleceği önceden bellidir, sorulacak bir şey kalmaz. İkincisinde "listede kim varsa sırayla gelsin" denir: Gelen herkes, yanında "ben şöyle sunarım" yazan küçük bir kart taşır ve öğretmen o karta bakar. Karta bakmak az da olsa zaman alır; karşılığında farklı kişiler aynı listeye konabilir.

Jenerik kod birinci düzendir. Derleyici her somut tür için kodun ayrı bir kopyasını üretir, hangi metodun çağrılacağını derlerken bilir. Buna *statik gönderim* (static dispatch) denir. Çağrı doğrudandır ve satır içine alınabilir (inline).

İkinci düzenin karşılığı *trait nesnesidir* (trait object): `dyn Ozet`. Kartın adı vtable'dır. Hangi metodun çağrılacağı çalışma zamanında bu tablodan okunur; buna *dinamik gönderim* (dynamic dispatch) denir.[^gonderim] Neden ikisine de ihtiyaç duyulduğunu bir liste örneğinde görelim. `Vec<T>` homojendir, yani tek bir türün değerlerini tutar:

```rust,compile_fail,E0308
# trait Ozet {
#     fn yazar(&self) -> String;
# }
# struct Haber { yazar: String }
# struct Tweet { kullanici: String }
# impl Ozet for Haber {
#     fn yazar(&self) -> String { self.yazar.clone() }
# }
# impl Ozet for Tweet {
#     fn yazar(&self) -> String { format!("@{}", self.kullanici) }
# }
fn main() {
    let h = Haber { yazar: String::from("Ajans") };
    let t = Tweet { kullanici: String::from("bot") };
    let olmaz = vec![h, t];
}
```

Derleyicinin yanıtı:

```text
error[E0308]: mismatched types
  |
4 |     let olmaz = vec![h, t];
  |                         ^ expected `Haber`, found `Tweet`
```

Yalnızca `Haber` tutacaksanız `Vec<Haber>` yeterlidir, `dyn` gerekmez. Farklı türleri bir arada tutmak için değerleri `Box` içine koyup trait nesnesine dönüştürürsünüz:[^trait-nesnesi]

```rust
# trait Ozet {
#     fn yazar(&self) -> String;
#     fn ozetle(&self) -> String {
#         format!("(devamı...) — {}", self.yazar())
#     }
# }
# struct Haber { baslik: String, yazar: String }
# struct Tweet { kullanici: String }
# impl Ozet for Haber {
#     fn yazar(&self) -> String { self.yazar.clone() }
#     fn ozetle(&self) -> String { format!("{} — {}", self.baslik, self.yazar) }
# }
# impl Ozet for Tweet {
#     fn yazar(&self) -> String { format!("@{}", self.kullanici) }
# }
fn main() {
    let karisik: Vec<Box<dyn Ozet>> = vec![
        Box::new(Haber { baslik: String::from("Hava"), yazar: String::from("Ajans") }),
        Box::new(Tweet { kullanici: String::from("bot") }),
    ];
    for oge in &karisik {
        println!("liste: {}", oge.ozetle());
    }
}
```

Çıktı:

```text
liste: Hava — Ajans
liste: (devamı...) — @bot
```

Döngünün gövdesi tek parça koddur; `oge.ozetle()` ilk turda `Haber`'in, ikinci turda `Tweet`'in metoduna gider. Bedeli, veriye ve metoda bir işaretçi üzerinden ulaşmak ve derleyicinin çağrıyı satır içine almaktan çoğunlukla vazgeçmesidir.[^gonderim] Seçim size kalmıştır: Tek bir türle çalışıyorsanız ya da kod programın en sık çalışan kısmındaysa jenerik, farklı türleri bir arada tutacaksanız ya da türleri sonradan ekleyecekseniz (eklenti sistemleri gibi) `dyn` uygundur.

### Hangi trait'ten trait nesnesi olur?

Bir trait'ten `dyn Trait` yapılabilmesi için trait'in *dyn uyumlu* (dyn compatible) olması gerekir; eski adı nesne güvenliğidir (object safety).[^dyn-uyum] Kabaca, her metodun asıl tür bilinmeden de vtable'dan bulunup çağrılabilmesi istenir. `Self` döndüren bir metot bu koşulu bozar:

```rust,compile_fail,E0038
trait Kopya {
    fn uret(&self) -> Self;
}

fn listele(_ogeler: Vec<Box<dyn Kopya>>) {}
```

Derleyicinin yanıtı:

```text
error[E0038]: the trait `Kopya` is not dyn compatible
  |
5 | fn listele(_ogeler: Vec<Box<dyn Kopya>>) {}
  |                             ^^^^^^^^^ `Kopya` is not dyn compatible
  |
note: for a trait to be dyn compatible it needs to allow building a vtable
      for more information, visit <https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility>
  |
1 | trait Kopya {
  |       ----- this trait is not dyn compatible...
2 |     fn uret(&self) -> Self;
  |                       ^^^^ ...because method `uret` references the `Self` type in its return type
  = help: consider moving `uret` to another trait
```

`uret` her tür için farklı bir tür döndürür. Asıl tür unutulduğu için çağıranın eline geçecek değerin türü, dolayısıyla boyutu bilinemez. Jenerik metotlar da benzer bir nedenle engeldir: Vtable derlenirken kurulur, jenerik metodun ise sınırsız sayıda kopyası olabilir; tabloya hangisinin konacağı belli değildir. Hiç `self` almayan `fn uret() -> Self` gibi bir fonksiyonda engel daha da baştadır: Çağrının tabloyu bulacağı bir alıcı yoktur. Böyle bir metodu trait nesnesinden dışarıda bırakmak için `where Self: Sized` eklenebilir.[^dyn-uyum] Trait nesneleriyle tasarım [17. Nesne Yönelimli Özellikler](17-oop.md) bölümünde, vtable'ın bellekteki yeri [22. `Sized`, DST ve Geniş İşaretçiler](../advanced/22-sized-dst.md) bölümündedir.

## Ömürler: Referans ne zamana kadar geçerli?

Her referansın bir ömrü vardır: geçerli olduğu kod bölgesi. Türler gibi ömürler de çoğu zaman çıkarılır ve yazılmaz. Bir referansın ömrü başka referanslarla birden çok biçimde ilişkili olabiliyorsa hangisinin geçerli olduğunu sizin söylemeniz gerekir.[^omur-jenerik] Amaç, [3. Ownership (Sahiplik)](../basics/03-ownership.md) bölümünde gördüğümüz sarkan referansları önlemektir.

### Fonksiyon imzasında ömür

Okul kütüphanesinden iki kitap ödünç aldığınızı düşünün; birinin iade günü cuma, ötekinin pazartesi. Arkadaşınıza "kalın olanı sana vereceğim" dediniz, ama hangisinin kalın olduğu henüz belli değil. Arkadaşınız kitabı güvenle ne zamana kadar tutabilir? İkisinden erken olana, yani cumaya kadar.

Aşağıdaki fonksiyon aynı durumdadır. Dönen referans iki girdiden herhangi birini gösterebilir:

```rust,compile_fail,E0106
fn en_uzun(x: &str, y: &str) -> &str {
    if x.len() > y.len() { x } else { y }
}
```

Derleyicinin yanıtı:

```text
error[E0106]: missing lifetime specifier
  |
1 | fn en_uzun(x: &str, y: &str) -> &str {
  |               ----     ----     ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but the signature does not say whether it is borrowed from `x` or `y`
help: consider introducing a named lifetime parameter
  |
1 | fn en_uzun<'a>(x: &'a str, y: &'a str) -> &'a str {
  |           ++++     ++          ++          ++
```

`help` satırı sorunu açıkça söyler: İmzada dönen değerin `x`'ten mi `y`'den mi ödünç alındığı yazmıyor. Bu bilgi olmadan çağıranın elindeki sonucun ne zamana kadar geçerli olduğu bilinemez. Çözüm bir *ömür parametresi* bildirmektir. Ömür adları kesme işaretiyle başlar ve tür parametreleri gibi açılı parantez içinde tanıtılır:[^omur-imza]

```rust
fn en_uzun<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

fn main() {
    let s1 = String::from("uzun bir cümle");
    let sonuc;
    {
        let s2 = String::from("kısa");
        sonuc = en_uzun(s1.as_str(), s2.as_str());
        println!("en uzun = {sonuc}"); // s2 hâlâ geçerli
    }
}
```

Çıktı:

```text
en uzun = uzun bir cümle
```

`'a`, bir türün değil bir bölgenin yerini tutan jenerik parametredir. İmza şunu bildirir: "Dönen referans, `x` ile `y`'nin ikisinin de geçerli olduğu bölge boyunca geçerlidir." İki girdinin ömrü farklıysa `'a` kısa olanına karşılık gelir.[^omur-imza] Karşılaştırmadaki `len` metodu bayt sayısını verir; örneğin amacı için bu yeterlidir.

### Etiket süreyi uzatmaz

Ömür etiketi yeni bir süre yaratmaz, iade tarihini ertelemez; var olan ömürlerin ilişkisini yazar.[^omur-soz] Bu yüzden sonucu kısa ömürlü girdiden sonra kullanmak yine hatadır:

```rust,compile_fail,E0597
# fn en_uzun<'a>(x: &'a str, y: &'a str) -> &'a str {
#     if x.len() > y.len() { x } else { y }
# }
fn main() {
    let sonuc;
    {
        let kisa = String::from("ab");
        sonuc = en_uzun("abc", &kisa);
    } // kisa burada düşürüldü
    println!("{sonuc}");
}
```

Derleyicinin yanıtı:

```text
error[E0597]: `kisa` does not live long enough
  |
4 |         let kisa = String::from("ab");
  |             ---- binding `kisa` declared here
5 |         sonuc = en_uzun("abc", &kisa);
  |                                ^^^^^ borrowed value does not live long enough
6 |     } // kisa burada düşürüldü
  |     - `kisa` dropped here while still borrowed
7 |     println!("{sonuc}");
  |                ----- borrow later used here
```

Bu çağrıda dönecek değer `"abc"`'dir, yani `kisa` ile ilgisi yoktur. Program yine de reddedilir; çünkü derleyici çağrı yerinde yalnızca imzaya bakar ve imza sonucu iki girdiye birden bağlar. Gövdenin bu söze uyup uymadığı ise fonksiyonun kendisi derlenirken ayrıca denetlenir. Bu ayrım sayesinde bir fonksiyonu kullanmak için gövdesini bilmeniz gerekmez, gövdesi değiştiğinde de çağıran kod bozulmaz.

### Referans tutan struct'lar

Ödünç kitabın bir sayfasına ayraç koyduğunuzu düşünün. Ayraç tek başına bir şey anlatmaz; kitap iade edilince gösterdiği sayfa da gider. İçinde referans tutan bir struct bu ayraçtır. Türüne `<'a>` yazarak "bu yapı bir veriye bakıyor, ondan uzun yaşayamaz" notunu düşersiniz:[^omur-struct]

```rust
struct Alinti<'a> {
    parca: &'a str,
}

impl<'a> Alinti<'a> {
    fn parca(&self) -> &str {
        self.parca
    }
}

fn main() {
    let metin = String::from("İlk cümle. İkincisi.");
    let ilk = metin.split('.').next().expect("split en az bir parça verir");
    let a = Alinti { parca: ilk };
    println!("alıntı: {}", a.parca());
}
```

Çıktı:

```text
alıntı: İlk cümle
```

Ayraç, kitaptan sonra kullanılamaz:

```rust,compile_fail,E0597
struct Ayrac<'a> {
    satir: &'a str,
}

fn main() {
    let a;
    {
        let kitap = String::from("Bir varmış");
        a = Ayrac { satir: &kitap };
        println!("{}", a.satir); // kitap hâlâ burada
    }
    println!("{}", a.satir);
}
```

Derleyicinin yanıtı:

```text
error[E0597]: `kitap` does not live long enough
   |
 8 |         let kitap = String::from("Bir varmış");
   |             ----- binding `kitap` declared here
 9 |         a = Ayrac { satir: &kitap };
   |                            ^^^^^^ borrowed value does not live long enough
10 |         println!("{}", a.satir); // kitap hâlâ burada
11 |     }
   |     - `kitap` dropped here while still borrowed
12 |     println!("{}", a.satir);
   |                    ------- borrow later used here
```

Not düşülmeseydi ayraç, silinmiş bir veriyi göstermeye devam edebilirdi. Struct alanlarında ömür çıkarımı yapılmaz: `parca: &str` yazarsanız derleyici E0106 ile ömür ister. Alan ya bir ömür parametresi taşır ya da açıkça `&'static str` yazılır; ikinci durumda struct'ın parametreye ihtiyacı kalmaz.

### Ömür çıkarımı

`ilk_kelime` gibi pek çok fonksiyon referans alıp referans döndürdüğü hâlde `'a` yazmadan derlenir. Nedeni, belli olan ömürleri derleyicinin doldurmasıdır. Kütüphane görevlisi, tek kitap getirip içinden bir sayfa gösteren birine "bu sayfa hangi kitaptan?" diye sormaz, yanıt bellidir. İki kitap getirene ise sorar.

```rust
fn ilk_kelime(s: &str) -> &str {
    s.split(' ').next().unwrap_or("")
}

fn main() {
    println!("{}", ilk_kelime("merhaba dünya"));
}
```

Çıktı:

```text
merhaba
```

Derleyici üç kural uygular:[^elision]

1. Ömrü yazılmamış her referans parametre kendi ayrı ömrünü alır.
2. Parametrelerde tek bir ömür varsa o ömür, dönüş türündeki yazılmamış ömürlerin tümüne verilir.
3. Metotlarda alıcı `&self` ya da `&mut self` ise onun ömrü, dönüş türündeki yazılmamış ömürlerin tümüne verilir.

`ilk_kelime`'de birinci kural `s`'ye bir ömür verir, ikinci kural onu sonuca taşır. `Alinti::parca` metodunda ise üçüncü kural `&self`'in ömrünü sonuca verir; `'a` bu yüzden yazılmadan bırakılabilmiştir.

Kurallar sonucu tek bir ömre bağlayamazsa derleyici tahmin yürütmez, E0106 verir. Gövdenin ne döndürdüğüne de bakmaz:

```rust,compile_fail,E0106
fn sec(x: &str, y: &str) -> &str {
    x
}
```

Derleyicinin yanıtı:

```text
error[E0106]: missing lifetime specifier
  |
1 | fn sec(x: &str, y: &str) -> &str {
  |           ----     ----     ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but the signature does not say whether it is borrowed from `x` or `y`
help: consider introducing a named lifetime parameter
  |
1 | fn sec<'a>(x: &'a str, y: &'a str) -> &'a str {
  |       ++++     ++          ++          ++
```

Sonuç yalnızca `x`'ten geldiğine göre doğru imza `fn sec<'a>(x: &'a str, y: &str) -> &'a str` olur; `y`'nin ömrü bağımsız kalır. Derleyicinin önerdiği gibi ikisine de `'a` yazmak derlenir, ama çağıranı gereksiz yere kısıtlar. Bu farkın sonuçlarını [derinleşme sayfasında](09-generics-traits-lifetimes-deep-dive.md) göreceğiz.

### `'static`: program boyunca geçerli

Okulun duvarına boyanmış yazıyı kimse ödünç alıp iade etmez; okul durdukça orada durur. Koda tırnak içinde yazdığınız metinler de böyledir: Program dosyasının içine gömülür ve program çalıştığı sürece yerinde kalır. Bu ömrün özel bir adı vardır: `'static`.[^static]

```rust
# use std::fmt::Display;
# trait Etiket {
#     fn etiket(&self) -> String;
# }
# impl<T: Display> Etiket for T {
#     fn etiket(&self) -> String {
#         format!("[{self}]")
#     }
# }
fn main() {
    let s: &'static str = "program boyu yaşar";
    println!("{s} / etiket örneği: {}", 42.etiket());
}
```

Çıktı:

```text
program boyu yaşar / etiket örneği: [42]
```

Sonradan oluşturulup düşürülecek bir veriye `'static` yazmak onu duvar yazısına çevirmez:

```rust,compile_fail,E0597
fn main() {
    let yerel = String::from("defter");
    let r: &'static str = &yerel;
    println!("{r}");
}
```

Derleyicinin yanıtı:

```text
error[E0597]: `yerel` does not live long enough
  |
2 |     let yerel = String::from("defter");
  |         ----- binding `yerel` declared here
3 |     let r: &'static str = &yerel;
  |            ------------   ^^^^^^ borrowed value does not live long enough
  |            |
  |            type annotation requires that `yerel` is borrowed for `'static`
4 |     println!("{r}");
5 | }
  | - `yerel` dropped here while still borrowed
```

Hata mesajları bazen `'static` yazmayı önerir. Öneriyi uygulamadan önce verinin gerçekten program boyunca yaşayıp yaşamadığını sorun: Çoğu zaman asıl sorun, sarkan bir referans oluşturmaya çalışmaktır.[^static] Yerel bir değerin referansını döndürmek bunun tipik örneğidir:

```rust,compile_fail,E0515
fn yeni_metin() -> &'static str {
    let s = String::from("defter");
    &s
}
```

Derleyicinin yanıtı:

```text
error[E0515]: cannot return reference to local variable `s`
  |
3 |     &s
  |     ^^ returns a reference to data owned by the current function
```

Çözüm etiketi uzatmak değil, sahipliği vermektir: `fn yeni_metin() -> String` yazıp `s`'yi döndürün. `T: 'static` sınırının anlamı farklıdır; onu da [derinleşme sayfasında](09-generics-traits-lifetimes-deep-dive.md) ele alıyoruz.

## Hata kodu başvurusu

| Kod | Ne zaman görülür? | Ne yapmalı? |
| --- | --- | --- |
| [E0038](https://doc.rust-lang.org/error_codes/E0038.html) | Dyn uyumlu olmayan bir trait'ten `dyn Trait` yapıldığında | `Self` döndüren ya da jenerik metotları ayırın veya `where Self: Sized` ekleyin; gerekirse jenerik kullanın. |
| [E0046](https://doc.rust-lang.org/error_codes/E0046.html) | Trait'in zorunlu bir metodu yazılmadığında | Eksik metodu yazın. |
| [E0106](https://doc.rust-lang.org/error_codes/E0106.html) | Çıkarılamayan ya da struct alanında yazılmamış bir ömür olduğunda | Sonucun hangi girdiye bağlı olduğunu `'a` ile yazın; alanda ömür parametresi ya da açık `'static` kullanın. |
| [E0116](https://doc.rust-lang.org/error_codes/E0116.html) | Başka crate'teki bir türe kendi impl'i yazıldığında | Bir trait tanımlayıp onu uygulayın ya da türü bir newtype ile sarın. |
| [E0117](https://doc.rust-lang.org/error_codes/E0117.html) | Trait de tür de başka crate'e aitken `impl` yazıldığında | Kendi trait'inizi tanımlayın ya da türü bir newtype ile sarın. |
| [E0277](https://doc.rust-lang.org/error_codes/E0277.html) | Bir tür, istenen trait sınırını sağlamadığında | Trait'i o tür için uygulayın ya da sınırı sağlayan bir değer verin. |
| [E0284](https://doc.rust-lang.org/error_codes/E0284.html) | Jenerik bir parametre çıkarılamadığında; örneğin `Buffer::yeni()` çağrısında uzunluk | Türü ya da değeri açıkça yazın: `Buffer::<4>::yeni()`. |
| [E0308](https://doc.rust-lang.org/error_codes/E0308.html) | Aynı `T`'yi paylaşan yerlere farklı türler verildiğinde | Türleri eşitleyin, ikinci bir parametre ekleyin ya da `Box<dyn Trait>` kullanın. |
| [E0369](https://doc.rust-lang.org/error_codes/E0369.html) | Sınırı olmayan `T` üzerinde `>` gibi bir operatör kullanıldığında | Operatörün trait'ini sınır olarak ekleyin: `T: PartialOrd`. |
| [E0425](https://doc.rust-lang.org/error_codes/E0425.html) | `impl Nokta<T>` yazılıp `<T>` bildirilmediğinde | `impl<T> Nokta<T>` yazın. |
| [E0515](https://doc.rust-lang.org/error_codes/E0515.html) | Fonksiyon, yerel bir değerin referansını döndürdüğünde | Sahipli değeri döndürün. |
| [E0597](https://doc.rust-lang.org/error_codes/E0597.html) | Referans, gösterdiği veriden uzun süre kullanıldığında | Kullanımı kapsamın içine alın ya da veriyi daha uzun yaşatın. |
| [E0599](https://doc.rust-lang.org/error_codes/E0599.html) | Çağrılan metot o türde yoksa, metodu getiren trait sınırı yazılmamışsa ya da trait'te böyle bir metot yoksa | Sınırı ekleyin, trait'i uygulayın ya da metot adını denetleyin. |
| [E0618](https://doc.rust-lang.org/error_codes/E0618.html) | Çağrılabilir olduğu bilinmeyen bir değer çağrıldığında | `F: Fn(..) -> ..` sınırını ekleyin. |

## Perde arkası: Derleyici ne yapıyor?

> [!NOTE]
> Bu başlık ilk okuyuşta atlanabilir.

Üç konunun ortak yanı, işin neredeyse tamamının kod derlenirken yapılmasıdır. Okul açılmadan önce hazırlanan ders programını düşünün: Kimin hangi sınıfa gireceği, hangi belgenin kimde olduğu, hangi kitabın ne zaman iade edileceği önceden belirlenir; ders sırasında kimse durup bunları sormaz.

### Monomorfizasyon

Üretilen makine kodu jenerik değildir. Derleyici, jenerik bir fonksiyonun *kullanıldığı* her somut tür için ayrı bir kopyasını üretir: Yukarıdaki örnekte `en_buyuk::<i32>` ve `en_buyuk::<char>` iki ayrı fonksiyondur.[^mono] Siz bir kez yazarsınız, kopyaları derleyici çıkarır. Bu yüzden jenerik kod, aynı işi somut türlerle elle yazdığınız koddan yavaş çalışmaz;[^gen-basarim] sıfır maliyet (zero-cost) denen budur. Bedeli derleme süresi ve ikili dosyanın boyutudur: Her kopya yer kaplar.[^mono] `dyn Trait` tersini yapar: tek kod, bir vtable ve çalışma zamanında dolaylama.

Hangi kopyaların gerektiğini `rustc_monomorphize` crate'indeki toplayıcı (collector) bulur.[^collector] Toplayıcının iki yöntemi vardır. Tembel (lazy) yöntem yalnızca gerçekten kullanılan öğeleri üretir; hevesli (eager) yöntem artımlı derleme için daha kararlı bir küme oluşturur ve örneğin trait metotlarının varsayılan gövdelerini de üretir.[^collector-strateji] Jenerik bir fonksiyonun tanımı X crate'indeyse kopyaları onu kullanan Y crate'inde de derlenebilir;[^collector-crate] ayrı derleme bu sayede jeneriklerle birlikte çalışır.

### Trait sınırlarının çözülmesi

Bazı dillerde "metodu varsa çağır, yoksa çalışırken hata ver" anlayışı geçerlidir; buna ördek tipi (duck typing) denir. Rust metodun varlığını derlerken kanıtlamak ister. Bir sınırın sağlanıp sağlanmadığını `rustc_trait_selection` araştırır; kanıt bulunamazsa E0277 üretilir.[^e0277-kod] Metot aramasını ve operatörleri `rustc_hir_typeck` denetler: Metot bulunamazsa E0599, operatör uygulanamazsa E0369 çıkar.[^e0599-kod][^e0369-kod]

Jenerik bir gövde yalnızca bir kez, sınırların verdiği bilgiyle denetlenir; kopyalar bu denetimden geçmiş gövdeden üretilir. Derleyicinin trait çözümlemesini nasıl yürüttüğü rustc-dev-guide'da anlatılır.[^trait-cozum]

### Vtable ve dyn uyumluluğu

Trait nesnesini gösteren işaretçi iki parçadan oluşur: verinin adresi ve vtable'ın adresi.[^trait-nesnesi] Vtable'ın başında değerin yıkıcısı, boyutu ve hizası, ardından çağrılabilir metotların adresleri durur; kaynak kodunda bunların karşılığı `VtblEntry` enum'ıdır.[^vtable] Trait'in uygun olup olmadığına `dyn_compatibility.rs` karar verir: Her metodun alıcısı, dönüş türü ve tür parametreleri tek tek incelenir.[^dyn-kod] İzin verilseydi, çalışma zamanında hangi türün kurulacağı ya da hangi kopyanın çağrılacağı bilinmeyen metotlar tabloya girmiş olurdu.

### Ömür çıkarımı ve ömürlerin silinmesi

Çıkarım kuralları, isim çözümleme aşamasında `rustc_resolve` crate'inde uygulanır. Parametreler taranırken `self`'ten gelen ömür öteki adaylardan önce gelir; bu, üçüncü kuralın koddaki karşılığıdır.[^elision-kod] Sonuç tek bir ömre bağlanamazsa eksikler `current_elision_failures` listesinde toplanır ve E0106 burada üretilir.[^e0106-kod] Çıkarım bir tahmin değildir: Aynı imza hep aynı sonucu verir, belirsizlikte ise hata çıkar.

Ömürler yalnızca derleme sırasında denetlenir. Derleyicinin ara gösteriminde (MIR) ve kod üretimi sırasında ömürlerin yerinde, silinmiş bölge anlamına gelen `ReErased` durur; makine koduna ömür bilgisi ulaşmaz.[^silinme] Buna *ömür silme* (lifetime erasure) denir. Jeneriklerde ise tür bilgisi silinmez, monomorfizasyonla çoğaltılır. "Tür silme" (type erasure) deyimi Rust'ta genellikle trait nesneleri için kullanılır: Değerin asıl türü unutulur, vtable'ı kalır.

## Özet

- Jenerik kod, türü sonraya bırakır; tür parametresi `T` her kullanımda somut bir türle doldurulur. Const jenerikler aynı şeyi bir değerle, çoğunlukla dizi uzunluğuyla yapar.
- Trait, türlerden beklenen davranıştır. Trait sınırı hem yazana hangi metotları kullanabileceğini hem de kullanana ne vermesi gerektiğini söyler; sınırsız bir `T` ile yapılabilecekler çok azdır.
- Genel impl, bir koşulu sağlayan bütün türlere trait verir. Yetim kuralı, trait de tür de yabancıyken `impl` yazılmasını engeller.
- Statik gönderim her tür için ayrı kod üretir; dinamik gönderim tek kod ve vtable kullanır, farklı türleri bir arada tutmaya izin verir.
- Ömür etiketi süreyi değiştirmez, referanslar arasındaki ilişkiyi bildirir. Çıkarım üç kurala dayanır; yetmediğinde derleyici sizden etiket ister.
- `'static` sihirli bir çözüm değildir; çoğu hatada doğru yanıt sahipliği vermektir.

Sıradaki sayfada, bir imzayı siz yazarken hangi etiketi ve hangi sınırı nereye koyacağınıza bakacağız: [9.1 Generic, Trait ve Lifetime: Derinleşme](09-generics-traits-lifetimes-deep-dive.md).

## Kaynaklar

[^ch10]: The Rust Programming Language, [10. Generic Types, Traits, and Lifetimes](https://doc.rust-lang.org/book/ch10-00-generics.html).
[^omur-jenerik]: The Rust Programming Language, [10.3 Validating References with Lifetimes](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html#validating-references-with-lifetimes).
[^gen-fn]: The Rust Programming Language, [10.1, In Function Definitions](https://doc.rust-lang.org/book/ch10-01-syntax.html#in-function-definitions).
[^slice-first]: Rust standart kütüphanesi, [`slice::first`](https://doc.rust-lang.org/std/primitive.slice.html#method.first).
[^operator]: The Rust Reference, [Comparison operators](https://doc.rust-lang.org/reference/expressions/operator-expr.html#comparison-operators).
[^gen-struct]: The Rust Programming Language, [10.1, In Struct Definitions](https://doc.rust-lang.org/book/ch10-01-syntax.html#in-struct-definitions).
[^gen-enum]: The Rust Programming Language, [10.1, In Enum Definitions](https://doc.rust-lang.org/book/ch10-01-syntax.html#in-enum-definitions).
[^gen-metot]: The Rust Programming Language, [10.1, In Method Definitions](https://doc.rust-lang.org/book/ch10-01-syntax.html#in-method-definitions).
[^sqrt]: Rust standart kütüphanesi, [`f64::sqrt`](https://doc.rust-lang.org/std/primitive.f64.html#method.sqrt) ve [`f32::sqrt`](https://doc.rust-lang.org/std/primitive.f32.html#method.sqrt).
[^isqrt]: Rust standart kütüphanesi, [`i32::isqrt`](https://doc.rust-lang.org/std/primitive.i32.html#method.isqrt).
[^dead-code]: The rustc book, [dead-code](https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#dead-code). `Debug` ayrıntısı derleyici mesajından alınmıştır: "`Nokta` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis".
[^const-gen]: The Rust Reference, [Const generics](https://doc.rust-lang.org/reference/items/generics.html#const-generics).
[^const-151]: Rust Blog, [Announcing Rust 1.51.0, Const Generics MVP](https://blog.rust-lang.org/2021/03/25/Rust-1.51.0/#const-generics-mvp).
[^const-159]: Rust Blog, [Announcing Rust 1.59.0, Const generics defaults and interleaving](https://blog.rust-lang.org/2022/02/24/Rust-1.59.0/#const-generics-defaults-and-interleaving).
[^trait-tanim]: The Rust Programming Language, [10.2, Defining a Trait](https://doc.rust-lang.org/book/ch10-02-traits.html#defining-a-trait).
[^varsayilan]: The Rust Programming Language, [10.2, Using Default Implementations](https://doc.rust-lang.org/book/ch10-02-traits.html#using-default-implementations).
[^sinir]: The Rust Programming Language, [10.2, Trait Bound Syntax](https://doc.rust-lang.org/book/ch10-02-traits.html#trait-bound-syntax).
[^impl-trait]: The Rust Reference, [Anonymous type parameters](https://doc.rust-lang.org/reference/types/impl-trait.html#anonymous-type-parameters).
[^where]: The Rust Programming Language, [10.2, Clearer Trait Bounds with `where` Clauses](https://doc.rust-lang.org/book/ch10-02-traits.html#clearer-trait-bounds-with-where-clauses).
[^donus]: The Rust Programming Language, [10.2, Returning Types That Implement Traits](https://doc.rust-lang.org/book/ch10-02-traits.html#returning-types-that-implement-traits).
[^sized]: The Rust Reference, [`Sized`](https://doc.rust-lang.org/reference/special-types-and-traits.html#sized).
[^fn-trait]: Rust standart kütüphanesi, [`std::ops::Fn`](https://doc.rust-lang.org/std/ops/trait.Fn.html).
[^cpp]: cppreference, [Constraints and concepts](https://en.cppreference.com/cpp/language/constraints). Aynı sayfa, sınırlı ve sınırsız şablonların hata mesajlarını yan yana gösterir.
[^inherent]: The Rust Reference, [Inherent implementations](https://doc.rust-lang.org/reference/items/implementations.html#inherent-implementations).
[^iliskili]: The Rust Reference, [Associated functions and methods](https://doc.rust-lang.org/reference/items/associated-items.html#associated-functions-and-methods).
[^genel]: The Rust Programming Language, [10.2, Using Trait Bounds to Conditionally Implement Methods](https://doc.rust-lang.org/book/ch10-02-traits.html#using-trait-bounds-to-conditionally-implement-methods).
[^tostring]: Rust 1.99.0 standart kütüphane kaynağı, [`string.rs`, `ToString for T`](https://github.com/rust-lang/rust/blob/1.99.0/library/alloc/src/string.rs#L2928).
[^yetim]: The Rust Programming Language, [10.2, Implementing a Trait on a Type](https://doc.rust-lang.org/book/ch10-02-traits.html#implementing-a-trait-on-a-type); The Rust Reference, [Orphan rules](https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules).
[^newtype]: The Rust Programming Language, [20.2, Implementing External Traits with the Newtype Pattern](https://doc.rust-lang.org/book/ch20-02-advanced-traits.html#implementing-external-traits-with-the-newtype-pattern).
[^gonderim]: The Rust Programming Language, [18.2, Performing Dynamic Dispatch](https://doc.rust-lang.org/book/ch18-02-trait-objects.html#performing-dynamic-dispatch).
[^trait-nesnesi]: The Rust Reference, [Trait objects](https://doc.rust-lang.org/reference/types/trait-object.html#trait-objects).
[^dyn-uyum]: The Rust Reference, [Dyn compatibility](https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility); Rust hata kodları, [E0038](https://doc.rust-lang.org/error_codes/E0038.html#method-has-generic-type-parameters).
[^omur-imza]: The Rust Programming Language, [10.3, In Function Signatures](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html#in-function-signatures).
[^omur-soz]: The Rust Programming Language, [10.3, Lifetime Annotation Syntax](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html#lifetime-annotation-syntax).
[^omur-struct]: The Rust Programming Language, [10.3, In Struct Definitions](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html#in-struct-definitions).
[^elision]: The Rust Programming Language, [10.3, Lifetime Elision](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html#lifetime-elision); The Rust Reference, [Lifetime elision in functions](https://doc.rust-lang.org/reference/lifetime-elision.html#lifetime-elision-in-functions).
[^static]: The Rust Programming Language, [10.3, The Static Lifetime](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html#the-static-lifetime).
[^mono]: rustc-dev-guide, [Monomorphization](https://rustc-dev-guide.rust-lang.org/backend/monomorph.html#monomorphization).
[^gen-basarim]: The Rust Programming Language, [10.1, Performance of Code Using Generics](https://doc.rust-lang.org/book/ch10-01-syntax.html#performance-of-code-using-generics).
[^collector]: rustc 1.99.0 kaynak kodu, [`collector.rs`, `Mono Item Collection`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_monomorphize/src/collector.rs#L1).
[^collector-strateji]: rustc 1.99.0 kaynak kodu, [`collector.rs`, `Eager and Lazy Collection Strategy`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_monomorphize/src/collector.rs#L141).
[^collector-crate]: rustc 1.99.0 kaynak kodu, [`collector.rs`, `in crate X`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_monomorphize/src/collector.rs#L12).
[^e0277-kod]: rustc 1.99.0 kaynak kodu, [`fulfillment_errors.rs`, `E0277`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_trait_selection/src/error_reporting/traits/fulfillment_errors.rs#L281).
[^e0599-kod]: rustc 1.99.0 kaynak kodu, [`suggest.rs`, `E0599`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_typeck/src/method/suggest.rs#L500).
[^e0369-kod]: rustc 1.99.0 kaynak kodu, [`op.rs`, `E0369`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_typeck/src/op.rs#L425).
[^trait-cozum]: rustc-dev-guide, [Trait resolution (old-style)](https://rustc-dev-guide.rust-lang.org/traits/resolution.html).
[^vtable]: rustc 1.99.0 kaynak kodu, [`vtable.rs`, `VtblEntry`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_middle/src/ty/vtable.rs#L13).
[^dyn-kod]: rustc 1.99.0 kaynak kodu, [`dyn_compatibility.rs`, `virtual_call_violations_for_method`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_trait_selection/src/traits/dyn_compatibility.rs#L437).
[^elision-kod]: rustc 1.99.0 kaynak kodu, [`late.rs`, `resolve_fn_signature`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_resolve/src/late.rs#L2417) ve [`resolve_fn_params`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_resolve/src/late.rs#L2470).
[^e0106-kod]: rustc 1.99.0 kaynak kodu, [`late/diagnostics.rs`, `report_missing_lifetime_specifiers`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_resolve/src/late/diagnostics.rs#L4052).
[^silinme]: rustc 1.99.0 kaynak kodu, [`region_kind.rs`, `ReErased`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_type_ir/src/region_kind.rs#L182); [`erase_regions.rs`, `erase_and_anonymize_regions`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_middle/src/ty/erase_regions.rs#L22).
