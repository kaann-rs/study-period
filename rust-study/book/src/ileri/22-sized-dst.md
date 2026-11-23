# 22. `Sized`, DST ve Geniş İşaretçiler

Bir değişken tanımladığınızda derleyicinin o değer için stack'te yer ayırması gerekir; bunun için de değerin kaç bayt olduğunu bilmelidir. `i32` için yanıt bellidir: dört bayt. Peki `str` kaç bayttır? Kısa bir sözcük de uzun bir roman da `str`'dir; türün adı uzunluk hakkında hiçbir şey söylemez. `dyn Ciz` yazdığınızda ise ortada gerçek türün adı bile yoktur.

Rust bu sorunu iki kavramla çözer. *`Sized`* trait'i "bu türün boyu derleme zamanında bilinir" der ve neredeyse her yerde sizden habersiz istenir. Boyu bilinmeyen türler, yani *dinamik boyutlu türler* (dynamically sized types, kısaca DST) ise yalnızca bir işaretçinin ardında kullanılır; eksik bilgiyi işaretçi taşır.[^dst] Adresin yanında bir bilgi daha taşıyan bu işaretçilere *geniş işaretçi* (wide pointer ya da fat pointer) denir; iki ad aynı şeyi anlatır.

Bu bölüm daha önce gördüğünüz pek çok parçayı tek köke bağlar: `str` ve `[T]`'nin neden hep `&` ya da `Box` ardında durduğunu, `dyn Trait`'in doğru metodu nasıl bulduğunu ve `?Sized` yazımının ne anlama geldiğini.

> [!NOTE]
> Bu bölümün notları ve çalıştırılabilir kodu `learn/advanced-concepts/ileri-sized-dst` dizinindedir. Depo kökünden `cargo run -p ileri-sized-dst` komutuyla çalıştırabilirsiniz. Bölümdeki bayt sayıları 64 bitlik platform içindir. Dilimleri ([3. bölüm](../temel/03-ownership.md)), trait'leri ([9. bölüm](../orta/09-generics-traits-lifetimes.md)), `Box`'ı ([14. bölüm](../orta/14-smart-pointers.md)) ve trait nesnelerini ([17. bölüm](../orta/17-oop.md)) bildiğiniz varsayılır.

## Günlük hayattan bir benzetme

Bir kargo şirketi, rafına koyacağı her kutunun boyunu önceden bilmek ister; yoksa rafta ne kadar yer ayıracağını bilemez. `Sized` bu demektir: "boyum belli". Bir ip yumağının ise ne kadar uzun olduğu dışarıdan anlaşılmaz; onu rafa doğrudan koyamazsınız. Ama yumağın ucuna bağlanan küçük etiket hep aynı boydadır ve rafa sığar. Rust'ta `str`, `[T]` ve `dyn Trait` ip yumağıdır; `&str` ya da `Box<str>` ise etikettir.

Etiketin üstünde ne yazmalı? Birine evinizi tarif ederken tek kâğıt yeter: Üstüne adresi yazarsınız. "Şu sokaktaki evler" derseniz ikinci bir kâğıt gerekir: "Kaç ev?" Bir de "ne tür bir bina olduğunu söylemeyeceğim" derseniz ikinci kâğıda "kullanma kılavuzu şurada" yazarsınız. Sıradan referans tek kâğıttır. Dilim, metin ve `dyn Trait` referansları çift kâğıttır: Biri adresi, öteki uzunluğu ya da kullanma kılavuzunu taşır.

Dilimi, bir kitabın tamamını değil "40. sayfadan başlayın, 3 sayfa okuyun" diye bir aralığını tarif etmek gibi düşünebilirsiniz. `dyn Trait`'teki kılavuz ise bir telefon rehberine benzer: Karşınızdakinin kim olduğunu bilmezsiniz, ama rehberde "çizmek için şu numarayı arayın" yazar.

<details>
<summary>Bu bölümde geçen terimler</summary>

- **bayt (byte):** Belleğin adreslenebilen en küçük birimi. Boyutlar baytla ölçülür.
- **sözcük (word) / `usize`:** Bir adresin sığdığı parça. 64 bitlik makinede 8 bayt.
- **işaretçi (pointer):** Bir verinin bellekteki adresini tutan değer. `&T` ve `Box<T>` böyledir.
- **`Sized`:** Boyu derleme zamanında bilinen türlerin işaret trait'i: `i32`, `String`, `Dugme`.
- **işaret trait'i (marker trait):** Metodu olmayan, yalnızca "bu türün şu özelliği var" diyen trait.
- **dinamik boyutlu tür (DST):** Boyu ancak program çalışırken belli olan tür. "Boyutu bilinmeyen" (unsized) de denir.
- **dilim (`[T]`) / `str`:** Art arda dizilmiş öğeler / metin. Kaç tane oldukları türde yazmaz.
- **trait nesnesi (trait object, `dyn Trait`):** Gerçek türü gizlenmiş, yalnızca trait'in metotları bilinen değer.
- **ince işaretçi (thin pointer):** Yalnızca adres taşıyan işaretçi; tek sözcük.
- **geniş işaretçi (wide / fat pointer):** Adresin yanında bir bilgi daha taşıyan işaretçi; iki sözcük.
- **üstveri (metadata):** O ek bilgi: dilimde uzunluk, `dyn Trait`'te vtable'ın adresi.
- **vtable (sanal metot tablosu):** Bir türün "hangi metot hangi adreste" listesini tutan tablo.
- **dinamik gönderim (dynamic dispatch):** Çağrılacak fonksiyonu program çalışırken vtable'dan bulmak.
- **`?Sized`:** "Bu türün boyu bilinmeyebilir, yine de kabul ediyorum" izni.
- **jenerik (generic):** Türü sonradan doldurulan kalıp: `fn f<T>(x: T)`.
- **monomorfizasyon (monomorphization):** Derleyicinin kullanılan her tür için jenerik kodun ayrı bir kopyasını üretmesi.
- **yerleşim / hizalama (layout / alignment):** Bir türün bellekte nasıl dizildiği / hangi sınıra hizalandığı.
- **konum (offset):** Bir alanın, yapının başından kaç bayt ileride durduğu.

</details>

## `Sized`: boyu derleme zamanında bilinen türler

`Sized`, standart kütüphanede tanımlı, metodu olmayan bir işaret trait'idir. Siz uygulamazsınız; boyu derleme zamanında bilinen her tür için derleyici kendiliğinden uygular.[^sized]

```rust
use std::mem::size_of;

struct Dugme {
    ad: String,
}

fn main() {
    println!("i32       = {}", size_of::<i32>());
    println!("(u8, u32) = {}", size_of::<(u8, u32)>());
    println!("String    = {}", size_of::<String>());
    println!("Dugme     = {}", size_of::<Dugme>());
}
```

Çıktı:

```text
i32       = 4
(u8, u32) = 8
String    = 24
Dugme     = 24
```

`String`'in boyu, tuttuğu metin ne kadar uzun olursa olsun 24 bayttır: heap'teki metni gösteren işaretçi, uzunluk ve kapasite. Metnin kendisi bu hesaba girmez.

Derleyici bu bilgi olmadan kod üretemez. Bir demete kaç bayt ayıracağını, dördüncü alanın hangi konumda başladığını, fonksiyonun stack'te ne kadar yer kaplayacağını ancak parçaların boyunu bilerek hesaplar.

## Dinamik boyutlu türler (DST)

Boyu ancak çalışma zamanında belli olan türlerin en yaygınları üç tanedir: dilim `[T]`, metin `str` ve trait nesnesi `dyn Trait`.[^dst] Elinizdeki `dyn Ciz` değerinin 8 baytlık mı 200 baytlık mı bir tür olduğunu derleyici bilemez. Bu yüzden bir DST'yi değer olarak, örneğin bir fonksiyon parametresi olarak kullanamazsınız:

```rust,compile_fail,E0277
fn al(_x: str) {}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0277]: the size for values of type `str` cannot be known at compilation time
  |
1 | fn al(_x: str) {}
  |           ^^^ doesn't have a size known at compile-time
  |
  = help: the trait `Sized` is not implemented for `str`
help: function arguments must have a statically known size, borrowed types always have a known size
  |
1 | fn al(_x: &str) {}
  |           +
```

Mesaj çözümü de söyler: DST'yi bir işaretçinin ardına koyun. İşaretçinin boyu bellidir, gösterdiği verininki olmasa bile.

```rust
fn al_ok(x: &str) -> usize {
    x.len()
}

fn main() {
    let odunc: &str = "yazı";
    let sahipli: Box<str> = Box::from("kutudaki yazı");
    let dilim: Box<[i32]> = vec![1, 2, 3].into_boxed_slice();

    println!("{} {} {}", al_ok(odunc), al_ok(&sahipli), dilim.len());
}
```

Çıktı:

```text
5 14 3
```

`Sized` isteyen konumlar parametrelerle sınırlı değildir. Yerel değişkenler, dönüş değerleri, `static` ve `const` öğeleri, dizi ve `Vec` öğeleri de boyu bilinen bir tür ister:[^dst]

| Yazım | Sonuç | Yerine |
| --- | --- | --- |
| `let x = *"abc";` | `E0277`: "all local variables must have a statically known size" | `let x = "abc";` |
| `Vec<dyn Ciz>` | `E0277`: `dyn Ciz` için `Sized` yok | `Vec<Box<dyn Ciz>>` |
| `fn don() -> dyn Ciz` | `E0746`: "return type cannot be a trait object without pointer indirection" | `Box<dyn Ciz>` ya da `impl Ciz` |

## Geniş işaretçi: adres ve üstveri

Bir DST'ye referans aldığınızda derleyici iki sözcüklük bir işaretçi kurar: Bir sözcük verinin adresidir, ikinci sözcük türü tamamlayan *üstveridir* (metadata). Boyları gözünüzle görebilirsiniz:

```rust
use std::mem::size_of;

trait Ciz {
    fn ciz(&self) -> String;
}

struct Dugme {
    ad: String,
}

fn main() {
    println!("usize (bir sözcük) = {}", size_of::<usize>());

    println!("ince işaretçiler:");
    println!("  &i32         = {}", size_of::<&i32>());
    println!("  &Dugme       = {}", size_of::<&Dugme>());
    println!("  &[i32; 5]    = {}", size_of::<&[i32; 5]>());
    println!("  Box<i32>     = {}", size_of::<Box<i32>>());

    println!("geniş işaretçiler:");
    println!("  &str         = {}", size_of::<&str>());
    println!("  &[i32]       = {}", size_of::<&[i32]>());
    println!("  &dyn Ciz     = {}", size_of::<&dyn Ciz>());
    println!("  Box<str>     = {}", size_of::<Box<str>>());
    println!("  Box<dyn Ciz> = {}", size_of::<Box<dyn Ciz>>());
}
# impl Ciz for Dugme { fn ciz(&self) -> String { self.ad.clone() } }
```

Çıktı:

```text
usize (bir sözcük) = 8
ince işaretçiler:
  &i32         = 8
  &Dugme       = 8
  &[i32; 5]    = 8
  Box<i32>     = 8
geniş işaretçiler:
  &str         = 16
  &[i32]       = 16
  &dyn Ciz     = 16
  Box<str>     = 16
  Box<dyn Ciz> = 16
```

`&[i32; 5]` ile `&[i32]` arasındaki farka dikkat edin. Birincisinde uzunluk türün içindedir, işaretçinin onu taşımasına gerek yoktur. İkincisinde uzunluk türden silinmiş ve işaretçiye yazılmıştır. Geniş işaretçinin *kendisi* `Sized`'dır; bu yüzden değişkende, alanda ya da `Vec` içinde tutulabilir.

> [!NOTE]
> Reference, boyu bilinen türlere giden işaretçilerin `usize` boyunda olduğunu garanti eder. DST'lere giden işaretçiler için yerleşim sayfası yalnızca "en az bu kadar" der; iki sözcük olmaları bugünkü gerçektir ama buna güvenmemeniz öğütlenir.[^isaretci]

### Üstveri ne tutar?

İkinci sözcüğün anlamı türe göre değişir: `[T]` için öğe sayısı, `str` için bayt sayısı, `dyn Trait` için vtable'ın adresidir.[^dst] Dilimlerde aynı diziye farklı aralıklarla bakabilirsiniz; dizi kopyalanmaz, yalnızca adres ve uzunluk değişir:

```rust
fn main() {
    let dizi = [10, 20, 30, 40, 50];
    let d1: &[i32] = &dizi[..3]; // adres: dizi[0], uzunluk: 3
    let d2: &[i32] = &dizi[2..]; // adres: dizi[2], uzunluk: 3

    println!("d1 = {d1:?}, uzunluk = {}", d1.len());
    println!("d2 = {d2:?}, uzunluk = {}", d2.len());
}
```

Çıktı:

```text
d1 = [10, 20, 30], uzunluk = 3
d2 = [30, 40, 50], uzunluk = 3
```

`len` çağrısı diziyi saymaz; uzunluğu işaretçinin içinden okur.

Trait nesnesinde üstveri bir *vtable*'ı (sanal metot tablosunu) gösterir. Tabloda trait'in her metodu için gerçek türün fonksiyonunun adresi durur.[^nesne] Böylece tek bir listede farklı türleri tutabilirsiniz; liste yalnızca "çizilebilen şeyler" der, her öğenin tablosu kendi türüne göre doludur:

```rust
use std::mem::size_of_val;

trait Ciz {
    fn ciz(&self) -> String;
}

struct Dugme {
    ad: String,
}

struct Nokta;

impl Ciz for Dugme {
    fn ciz(&self) -> String {
        format!("[{}]", self.ad)
    }
}

impl Ciz for Nokta {
    fn ciz(&self) -> String {
        String::from(".")
    }
}

fn main() {
    let nesneler: Vec<Box<dyn Ciz>> = vec![
        Box::new(Dugme { ad: String::from("Tamam") }),
        Box::new(Nokta),
        Box::new(Dugme { ad: String::from("İptal") }),
    ];
    for n in &nesneler {
        println!("{} ({} bayt)", n.ciz(), size_of_val(&**n));
    }
}
```

Çıktı:

```text
[Tamam] (24 bayt)
. (0 bayt)
[İptal] (24 bayt)
```

`n.ciz()` çağrısında program önce vtable'a bakar, `ciz`'in adresini bulur ve oraya atlar. Buna *dinamik gönderim* denir; bedeli küçük bir dolaylamadır. `size_of_val`'in her öğe için doğru boyu verebilmesi de aynı tablodan gelir: Boyut vtable'da yazar.

Tabloda olmayan şey bilinemez. `dyn Ciz` yalnızca `Ciz`'in metotlarını tanır; gerçek türün alanlarına erişemezsiniz:

```rust,compile_fail,E0609
# trait Ciz { fn ciz(&self) -> String; }
# struct Dugme { ad: String }
# impl Ciz for Dugme { fn ciz(&self) -> String { self.ad.clone() } }
fn main() {
    let n: Box<dyn Ciz> = Box::new(Dugme { ad: String::from("Tamam") });
    println!("{}", n.ad);
}
```

Derleyicinin yanıtı:

```text
error[E0609]: no field `ad` on type `Box<dyn Ciz>`
```

Alana ihtiyacınız varsa trait'e onu döndüren bir metot ekleyin.

### Geniş işaretçi nereden doğar?

Geniş işaretçi çoğu zaman *boyut silme dönüşümüyle* (unsized coercion) kurulur. Boyu bilinen bir türe giden işaretçi, DST'ye giden bir işaretçinin beklendiği yerde kendiliğinden dönüşür; bilgi kaybolmaz, türden işaretçiye taşınır:[^unsize]

```rust
use std::fmt::Display;

fn main() {
    let dizi: &[i32; 3] = &[1, 2, 3]; // uzunluk türde
    let dilim: &[i32] = dizi; // uzunluk işaretçide

    let sayi: &i32 = &42; // gerçek tür belli
    let nesne: &dyn Display = sayi; // gerçek tür vtable'da

    println!("{} {}", dilim.len(), nesne);
}
```

Çıktı:

```text
3 42
```

## Örtük `Sized` sınırı ve `?Sized`

Yazdığınız her jenerik parametre, siz istemeseniz de `T: Sized` sınırı taşır. İlişkili türler için de böyledir. Sınırı kaldırmak için `?Sized` yazılır ve "`T`'nin boyu bilinmeyebilir" diye okunur:[^ortuk]

```rust
fn tek_satir<T: std::fmt::Debug + ?Sized>(x: &T) {
    println!("{x:?}");
}

fn main() {
    tek_satir("ben bir str'yim"); // T = str
    tek_satir(&[1, 2, 3][..]); // T = [i32]
    tek_satir(&42); // T = i32
}
```

Çıktı:

```text
"ben bir str'yim"
[1, 2, 3]
42
```

`?Sized` yeni bir koşul getirmez, var olanı gevşetir. `i32` gibi boyu bilinen türler de geçmeye devam eder. Sınırı gevşetmezseniz `str` kabul edilmez:

```rust,compile_fail,E0277
use std::fmt::Debug;

fn yaz2<T: Debug>(x: &T) {
    println!("{x:?}");
}

fn main() {
    yaz2("yazı");
}
```

Derleyicinin yanıtı:

```text
error[E0277]: the size for values of type `str` cannot be known at compilation time
  |
8 |     yaz2("yazı");
  |     ---- ^^^^^^ doesn't have a size known at compile-time
  |     |
  |     required by a bound introduced by this call
  |
  = help: the trait `Sized` is not implemented for `str`
note: required by an implicit `Sized` bound in `yaz2`
help: consider relaxing the implicit `Sized` restriction
  |
3 | fn yaz2<T: Debug + ?Sized>(x: &T) {
  |                  ++++++++
```

`required by an implicit Sized bound` satırı, sizin yazmadığınız sınırı gösterir.

Sınırı gevşettiğinizde karşılığında bir şey kaybedersiniz: `T`'yi artık değer olarak alamaz, yalnızca `&T`, `Box<T>` gibi bir işaretçi ardında kullanabilirsiniz.

```rust,compile_fail,E0277
fn tek<T: ?Sized>(x: T) {}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0277]: the size for values of type `T` cannot be known at compilation time
```

### İstisna: trait'in `Self`'i

Örtük sınırın bir istisnası vardır. Trait tanımındaki `Self` varsayılan olarak `?Sized`'dır; bu sayede bir trait `str` ya da `[i32]` gibi DST'ler için de uygulanabilir:[^dst]

```rust
trait Olc {
    fn boy(&self) -> usize {
        std::mem::size_of_val(self)
    }
}

impl Olc for str {}
impl Olc for [i32] {}
impl Olc for i64 {}

fn main() {
    println!("{} {} {}", "kedi".boy(), [1, 2, 3][..].boy(), 7i64.boy());
}
```

Çıktı:

```text
4 12 8
```

Trait'e `Sized`'ı üst trait olarak eklerseniz bu özgürlük gider ve trait artık `dyn` ile kullanılamaz, çünkü `dyn Trait`'in kendisi `Sized` değildir:

```rust,compile_fail,E0038
trait Olcu: Sized {
    fn boy(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

impl Olcu for i32 {}

fn main() {
    let x: &dyn Olcu = &5;
}
```

Derleyicinin yanıtı:

```text
error[E0038]: the trait `Olcu` is not dyn compatible
```

Bütün trait'i kısıtlamak yerine yalnızca ilgili metoda `where Self: Sized` yazmak, trait'i dyn uyumlu tutar.[^ortuk]

## Son alanı DST olan struct'lar

Bir struct'ın *son* alanı DST olabilir; o zaman struct'ın kendisi de bir DST olur. Böyle bir struct'a giden işaretçi, son alanın üstverisini taşır:[^dst]

```rust
struct Paket<T: ?Sized> {
    no: u32,
    veri: T,
}

fn main() {
    let sabit: &Paket<[u8; 4]> = &Paket { no: 7, veri: [1, 2, 3, 4] };
    let esnek: &Paket<[u8]> = sabit; // uzunluk işaretçiye geçti

    println!("no = {}, veri = {:?}", esnek.no, &esnek.veri);
    println!("değer = {} bayt", std::mem::size_of_val(esnek));
    println!("işaretçi = {} bayt", std::mem::size_of::<&Paket<[u8]>>());
}
```

Çıktı:

```text
no = 7, veri = [1, 2, 3, 4]
değer = 8 bayt
işaretçi = 16 bayt
```

DST'nin sonda olması şarttır. Ortada dursaydı, ardından gelen alanın konumu derleme zamanında hesaplanamazdı:

```rust,compile_fail,E0277
struct Kotu {
    veri: [u8],
    n: u32,
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0277]: the size for values of type `[u8]` cannot be known at compilation time
  |
  = help: the trait `Sized` is not implemented for `[u8]`
  = note: only the last field of a struct may have a dynamically sized type
  = help: change the field's type to have a statically known size
```

## `Box`, `Rc` ve `Arc` da geniş olabilir

Geniş işaretçi yalnız referanslara özgü değildir. `Box`, `Rc` ve `Arc` tanımlarında `T: ?Sized` yazar; bu yüzden `Box<str>`, `Rc<str>` ve `Arc<dyn Trait>` gibi türler kurabilirsiniz:[^kutular]

```rust
use std::mem::size_of;
use std::rc::Rc;
use std::sync::Arc;

fn main() {
    let ad: Rc<str> = Rc::from("paylaşılan ad");
    let is: Arc<dyn Fn(i32) -> i32 + Send + Sync> = Arc::new(|x| x + 1);

    println!("{ad} {}", is(41));
    println!("{} {}", size_of::<Rc<str>>(), size_of::<Arc<dyn Fn()>>());
}
```

Çıktı:

```text
paylaşılan ad 42
16 16
```

`Vec<T>` ise `T: Sized` ister; öğeler art arda dizildiği için hepsinin aynı ve bilinen boyda olması gerekir. Farklı boydaki değerleri bu yüzden `Vec<Box<dyn Ciz>>` ile tutarsınız.

## Jenerik mi, `dyn` mi?

`Sized` ve DST ayrımı, çok biçimliliğin iki yolunun neden böyle çalıştığını açıklar. Jenerik kod *monomorfizasyona* uğrar: Derleyici kullanılan her somut tür için kodun ayrı bir kopyasını üretir. Her kopyada türün boyu ve metotları bellidir, çağrı doğrudan yapılır.[^mono] `dyn Trait` kullanan kod ise tek kopyadır; türün boyu ve metotları vtable'dan okunur.[^dinamik]

```rust
trait Ciz {
    fn ciz(&self) -> String;
}

impl Ciz for i32 {
    fn ciz(&self) -> String {
        format!("sayı {self}")
    }
}

impl Ciz for char {
    fn ciz(&self) -> String {
        format!("harf {self}")
    }
}

fn jenerik<T: Ciz>(x: &T) -> String {
    x.ciz() // her T için ayrı kopya, doğrudan çağrı
}

fn dinamik(x: &dyn Ciz) -> String {
    x.ciz() // tek kopya, vtable üzerinden çağrı
}

fn main() {
    println!("{}, {}", jenerik(&5), jenerik(&'a'));
    println!("{}, {}", dinamik(&5), dinamik(&'a'));
}
```

Çıktı:

```text
sayı 5, harf a
sayı 5, harf a
```

Birinin bedeli kod boyutu, ötekinin bedeli küçük bir dolaylama ile kaçırılan bazı optimizasyon fırsatlarıdır. Karşılığında `dyn`, farklı türleri aynı listede tutma esnekliğini verir. Derleyici kopyaları istek üzerine üretir: Çağrılmayan bir jenerik fonksiyonun ya da metodun kopyası çıkarılmaz.[^mono-kod]

## Üstveriye doğrudan erişmek

Kararlı Rust'ta üstverinin kendisini okumanın genel bir yolu yoktur. Uzunluğa `len`, boyuta `size_of_val`, hizalamaya `align_of_val` ile ulaşırsınız. Geniş işaretçiyi parçalayan `std::ptr::metadata` ve `Pointee` trait'i ise henüz kararlı değildir; 1.100.0 sürümünde de `ptr_metadata` özelliğinin arkasındadır:[^metadata]

```rust,compile_fail,E0658
fn main() {
    let d: &[i32] = &[1, 2, 3];
    println!("{}", std::ptr::metadata(d));
}
```

Derleyicinin yanıtı:

```text
error[E0658]: use of unstable library feature `ptr_metadata`
  |
3 |     println!("{}", std::ptr::metadata(d));
  |                    ^^^^^^^^^^^^^^^^^^
  |
  = note: see issue #81513 <https://github.com/rust-lang/rust/issues/81513> for more information
```

## Hata kodu başvurusu

| Kod | Ne zaman görülür? | Ne yapmalı? |
| --- | --- | --- |
| `E0277` | DST değer olarak kullanıldığında ya da örtük `Sized` sınırı sağlanmadığında | `&T` / `Box<T>` kullanın ya da sınırı `?Sized` ile gevşetin. |
| `E0746` | Fonksiyon `dyn Trait` döndürmeye çalıştığında | `Box<dyn Trait>` ya da `impl Trait` döndürün. |
| `E0609` | Trait nesnesi üzerinden gerçek türün alanına erişildiğinde | Trait'e bir metot ekleyin ya da somut türü kullanın. |
| `E0038` | `Sized` üst trait'i olan bir trait `dyn` ile kullanıldığında | Sınırı trait'ten kaldırıp `where Self: Sized` olarak metoda taşıyın. |
| `E0658` | `std::ptr::metadata` kararlı derleyicide kullanıldığında | `len`, `size_of_val`, `align_of_val` kullanın. |

## Perde arkası: Derleyici ne yapıyor?

> [!NOTE]
> Bu başlık ilk okuyuşta atlanabilir; yukarıdaki kuralları anlamış olmanız yeterlidir.

**Örtük sınır.** Jenerik parametrelere `Sized` sınırını, kaynak kodu iç gösterime çeviren aşama ekler; `?Sized` gördüğünde eklemez.[^sized-kod] 1.99.0'ın standart kütüphanesinde `Sized` artık tek başına değildir: "Boyu çalışma zamanında üstveriden hesaplanabilir" anlamındaki `MetaSized` ile "bir işaretçinin gösterebileceği her şey" anlamındaki `PointeeSized` trait'lerinin üzerine kurulmuştur. Bunlar `sized_hierarchy` özelliğinin arkasındadır, kararlı kodda yazılamaz.

**İşaretçinin yerleşimi.** Derleyici bir işaretçinin yerleşimini hesaplarken gösterdiği türün kuyruğuna bakar. Kuyruk dilim ya da `str` ise ikinci parça işaretçi boyunda bir tam sayıdır; `dyn Trait` ise sıfır olamayan bir işaretçidir. Sonuç iki parçalı bir çift olarak yazılır.[^yerlesim]

**Vtable'ın içi.** Her vtable üç ortak girişle başlar: yıkıcının (drop) adresi, türün boyu ve hizalaması. Ardından trait'in metotları gelir. Düşürülecek bir şeyi olmayan türlerde yıkıcı girişi boştur.[^vtable] `size_of_val` bir trait nesnesi için boyu buradan okur; dilimde ise uzunluğu öğe boyuyla çarpar.[^boyut-kod] Vtable'ın bellekteki tam dizilişi dilin garantisi değildir, derleyici sürümleri arasında değişebilir.

**Benzer bir fikir: `RawWakerVTable`.** [28. bölümde](28-future.md) göreceğiniz `RawWaker` de aynı düşünceyle kurulmuştur: bir veri işaretçisi ve bir fonksiyon işaretçileri tablosu. Standart kütüphane belgesi bu tablodan "virtual function pointer table (vtable)" diye söz eder. Fark, tablonun derleyici tarafından değil elle doldurulması ve boyut ile hizalama taşımamasıdır.[^waker]

## Özet

- `Sized`, boyu derleme zamanında bilinen türlerin işaretidir; derleyici kendiliğinden uygular ve jenerik parametrelerde kendiliğinden ister.
- `str`, `[T]` ve `dyn Trait` DST'dir. Değer olarak kullanılamaz, yalnızca bir işaretçinin ardında durur.
- DST'ye giden işaretçi geniştir: adresin yanında dilimde uzunluk, trait nesnesinde vtable adresi taşır. 64 bitte ince işaretçi 8, geniş işaretçi 16 bayttır.
- `?Sized` örtük sınırı gevşetir; karşılığında `T` yalnızca işaretçi ardında kullanılabilir. Trait'in `Self`'i zaten `?Sized`'dır.
- Jenerikler tür başına kopya, `dyn` ise tek kod ve tablo demektir.

Sıradaki bölüm [23. Tutarlılık ve Yetim Kuralı](23-coherence.md), bir trait'i hangi türler için uygulayabileceğinizi ve bu sınırın neden konduğunu anlatıyor.

## Kaynaklar

[^dst]: The Rust Reference, [Dynamically sized types](https://doc.rust-lang.org/reference/dynamically-sized-types.html); The Rust Programming Language, [Dynamically Sized Types and the `Sized` Trait](https://doc.rust-lang.org/book/ch20-03-advanced-types.html#dynamically-sized-types-and-the-sized-trait); Jon Gjengset, *Rust for Rustaceans*, 2. bölüm, "Dynamically Sized Types and Wide Pointers".
[^sized]: std, [`Sized` trait'i](https://doc.rust-lang.org/std/marker/trait.Sized.html); The Rust Reference, [Special types and traits: `Sized`](https://doc.rust-lang.org/reference/special-types-and-traits.html#sized).
[^isaretci]: The Rust Reference, [Pointers and references layout](https://doc.rust-lang.org/reference/type-layout.html#pointers-and-references-layout).
[^nesne]: The Rust Reference, [Trait objects](https://doc.rust-lang.org/reference/types/trait-object.html#trait-objects); The Rust Programming Language, [Performing Dynamic Dispatch](https://doc.rust-lang.org/book/ch18-02-trait-objects.html#performing-dynamic-dispatch).
[^unsize]: The Rust Reference, [Unsized coercions](https://doc.rust-lang.org/reference/type-coercions.html#unsized-coercions).
[^ortuk]: The Rust Reference, [Trait bounds: `Sized`](https://doc.rust-lang.org/reference/trait-bounds.html#sized) ve [Dyn compatibility](https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility).
[^kutular]: std, [`Box`](https://doc.rust-lang.org/std/boxed/struct.Box.html), [`Rc`](https://doc.rust-lang.org/std/rc/struct.Rc.html), [`Arc`](https://doc.rust-lang.org/std/sync/struct.Arc.html) ve [`Vec`](https://doc.rust-lang.org/std/vec/struct.Vec.html).
[^mono]: The Rust Programming Language, [Performance of Code Using Generics](https://doc.rust-lang.org/book/ch10-01-syntax.html#performance-of-code-using-generics).
[^dinamik]: The Rust Programming Language, [Performing Dynamic Dispatch](https://doc.rust-lang.org/book/ch18-02-trait-objects.html#performing-dynamic-dispatch).
[^mono-kod]: rustc 1.99.0 kaynak kodu, [`collector.rs`, `MonoItemCollectionStrategy`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_monomorphize/src/collector.rs#L244).
[^metadata]: std, [`ptr::metadata`](https://doc.rust-lang.org/std/ptr/fn.metadata.html), [`ptr::Pointee`](https://doc.rust-lang.org/std/ptr/trait.Pointee.html), [`mem::size_of_val`](https://doc.rust-lang.org/std/mem/fn.size_of_val.html) ve [`mem::align_of_val`](https://doc.rust-lang.org/std/mem/fn.align_of_val.html).
[^sized-kod]: rustc 1.99.0 kaynak kodu, [`bounds.rs`, `add_implicit_sizedness_bounds`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_analysis/src/hir_ty_lowering/bounds.rs#L149) ve [`marker.rs`, `pub trait Sized: MetaSized`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/marker.rs#L158).
[^yerlesim]: rustc 1.99.0 kaynak kodu, [`layout.rs`, `scalar_pair`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_ty_utils/src/layout.rs#L517).
[^vtable]: rustc 1.99.0 kaynak kodu, [`vtable.rs`, `VtblEntry`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_middle/src/ty/vtable.rs#L13) ve [`COMMON_VTABLE_ENTRIES`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_middle/src/ty/vtable.rs#L45); std kaynak kodu, [`metadata.rs`, `DynMetadata`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/ptr/metadata.rs#L159).
[^boyut-kod]: rustc 1.99.0 kaynak kodu, [`size_of_val.rs`, `size_and_align_of_dst`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_codegen_ssa/src/size_of_val.rs#L15).
[^waker]: std, [`RawWakerVTable`](https://doc.rust-lang.org/std/task/struct.RawWakerVTable.html) ve [`RawWaker`](https://doc.rust-lang.org/std/task/struct.RawWaker.html).
