# 26. Düşürme Denetimi (Drop Check)

Bir değer kapsamdan çıktığında *düşer*: Yıkıcısı çalışır ve kapladığı yer geri verilir. Yıkıcı, yani sizin yazdığınız `Drop::drop` gövdesi, o sırada değerin alanlarını hâlâ okuyabilir. Alanlardan biri bir referanssa akla şu soru gelir: Referansın gösterdiği veri o anda hâlâ yaşıyor mu?

Bu soruyu derleyici sorar ve yanıtını programınızı çalıştırmadan verir. Denetimin adı *düşürme denetimi*dir (drop check, kısaca dropck). Derleyici bu denetimi yapmasaydı, önceden düşmüş bir değere bakan bir yıkıcı serbest bırakılmış belleği okurdu; güvenli koddan serbest bırakıldıktan sonra kullanım hatası çıkardı.

Bölümde önce düşürmenin ödünç kurallarında neden bir kullanım sayıldığını göreceğiz. Ardından `Drop` yazmayan türlerin de düşürülürken iş yaptığına, yer kaplamayan `PhantomData` alanının bu hesaba etkisine ve standart kütüphanenin kullandığı `#[may_dangle]` ayarına bakacağız.

> [!NOTE]
> Bu bölümün notları ve çalıştırılabilir kodu `learn/advanced-concepts/ileri-dropck` dizinindedir. Depo kökünden `cargo run -p ileri-dropck` komutuyla çalıştırabilirsiniz. Kapsam ile ödünç almayı ([3. bölüm](../temel/03-ownership.md)), ömürleri ([9. bölüm](../orta/09-generics-traits-lifetimes.md)), `Drop`'u ([14. bölüm](../orta/14-smart-pointers.md)), varyansı ([21. bölüm](21-variance.md)) ve auto trait'leri ([25. bölüm](25-send-sync.md)) bildiğiniz varsayılır.

## Günlük hayattan bir benzetme

Arkadaşınızın defterini ödünç aldınız ve "odadan çıkarken son bir kez bakacağım" dediniz. O hâlde defter siz çıkana kadar masada kalmalıdır. Arkadaşınız onu sizden önce toplarsa çıkarken boş masaya bakarsınız. `Drop` yazmak bu sözdür: Giderken son bir kez bakılacaktır. Aynı durum şöyle de düşünülebilir: Kendi defterinizi çöpe atmadan önce içine son bir not yazacaksınız ve not için masadaki kitaba bakmanız gerekiyor. Kitap, defter atılana kadar orada durmalıdır.

`Drop` yoksa çıkarken bakmak da yoktur. Defter sizden önce toplansa bile kimse fark etmez.

`PhantomData` ise boş bir odanın kapısına asılan "Zeynep'in odası" levhasıdır. Odada kimse yoktur, levha da yer kaplamaz; ama binadaki görevliler odaya Zeynep'inmiş gibi davranır.

<details>
<summary>Bu bölümde geçen terimler</summary>

- **düşürme (drop):** Bir değerin yok edilmesi; yıkıcısının çalışması ve yerinin geri verilmesi.
- **yıkıcı (destructor):** Düşürme sırasında çalışan kod. `Drop::drop` gövdesi ve alanların yıkıcılarından oluşur.
- **drop glue:** Bir türü düşürmek için derleyicinin ürettiği kod; alanları sırayla düşürür.
- **kapsam (scope):** Bir değişkenin yaşadığı kod aralığı; çoğunlukla bir süslü parantez çifti.
- **düşme sırası:** Kapsam bitince değişkenlerin, tanımlandıkları sıranın tersiyle düşmesi.
- **ödünç (borrow):** Sahibi değiştirmeden veriye referansla ulaşmak.
- **ömür (lifetime, `'a`):** Bir referansın geçerli olduğu kod aralığının adı.
- **NLL (non-lexical lifetimes):** Ödüncün süslü parantezin sonunda değil, son kullanıldığı yerde bitmesi.
- **düşürme denetimi (dropck):** "Bu değer düşerken baktığı veri hâlâ duruyor mu?" denetimi.
- **sarkan referans (dangling):** Gösterdiği veri yok edilmiş referans.
- **serbest bırakıldıktan sonra kullanım (use-after-free):** Geri verilmiş belleğe erişmek; tanımsız davranıştır.
- **tür parametresi (type parameter):** `Belirtec<T>` içindeki `T`.
- **`PhantomData<T>`:** Yer kaplamayan işaret alanı; "`T` ile ilişkim var" bilgisini taşır.
- **sıfır boyutlu tür (ZST, zero-sized type):** Bellekte sıfır bayt kaplayan tür.
- **ham işaretçi (raw pointer):** Geçerliliğini derleyicinin denetlemediği adres: `*const T`, `*mut T`.
- **varyans (variance):** Bir türün içindeki ömrün ya da türün ne kadar esneyebildiği.
- **auto trait:** Derleyicinin alanlara bakarak kendiliğinden verdiği trait: `Send`, `Sync`.
- **`#[may_dangle]`:** "Düşürürken bu `T`'ye bakmayacağım" diye verilen `unsafe` söz; kararsızdır.
- **nightly / kararsız özellik (unstable):** Kararlı sürüme girmemiş deneme özelliği ve onu açan derleyici kanalı.

</details>

## Düşürme bir kullanımdır

`Denetci`, ödünç aldığı sayıyı düşerken yazdırsın:

```rust
struct Denetci<'a>(&'a u8);

impl<'a> Drop for Denetci<'a> {
    fn drop(&mut self) {
        println!("Denetci düşerken okuyor: {}", self.0);
    }
}

fn main() {
    let gun = 5u8; // önce tanımlandı, en son düşer
    let denetci = Denetci(&gun); // sonra tanımlandı, önce düşer
    println!("değer = {}", denetci.0);
}
```

Çıktı:

```text
değer = 5
Denetci düşerken okuyor: 5
```

Yerel değişkenler tanımlandıkları sıranın tersiyle düşer.[^sira] `denetci` önce düşer; yıkıcısı `gun`'u okurken `gun` hâlâ yaşamaktadır. Şimdi yalnızca tanım sırasını değiştirelim:

```rust,compile_fail,E0597
struct Denetci<'a>(&'a u8);

impl<'a> Drop for Denetci<'a> {
    fn drop(&mut self) {
        println!("Denetci düşerken okuyor: {}", self.0);
    }
}

fn main() {
    let denetci; // önce tanımlandı, en son düşer
    let gun = 5u8; // sonra tanımlandı, önce düşer
    denetci = Denetci(&gun);
    println!("değer = {}", denetci.0);
}
```

Derleyicinin yanıtı:

```text
error[E0597]: `gun` does not live long enough
   |
11 |     let gun = 5u8; // sonra tanımlandı, önce düşer
   |         --- binding `gun` declared here
12 |     denetci = Denetci(&gun);
   |                       ^^^^ borrowed value does not live long enough
13 |     println!("değer = {}", denetci.0);
14 | }
   | -
   | |
   | `gun` dropped here while still borrowed
   | borrow might be used here, when `denetci` is dropped and runs the `Drop` code for type `Denetci`
   |
   = note: values in a scope are dropped in the opposite order they are defined
```

Mesaj düşürmeyi bir kullanım olarak anar: "ödünç burada, `denetci` düşerken kullanılabilir". Kural da budur: Bir türün `Drop` impl'i varsa, o türün bir değeri düşürüldüğünde bütün ömür ve tür parametreleri hâlâ geçerli olmalıdır.[^dropck] Yani ödünç alınan veri, onu ödünç alan değerden daha uzun yaşamalıdır. Aynı kapsamda düşmeleri yetmez; düşme sırasında `gun` sonra gelmelidir.

`Drop` impl'ini silerseniz aynı ters sıralı kod derlenir:

```rust
struct Denetci<'a>(&'a u8);

fn main() {
    let denetci;
    let gun = 5u8;
    denetci = Denetci(&gun);
    println!("değer = {}", denetci.0);
}
```

Çıktı:

```text
değer = 5
```

`Denetci`'nin artık yıkıcısı yoktur; düşürülmesi hiçbir şey okumaz ve kullanım sayılmaz. `denetci`'nin son kullanımı `println!` satırıdır, ödünç de orada biter. Bunu NLL'in "ödüncü erken bitirmesi" diye anlatmak yarım kalır. NLL her iki örnekte de ödüncü son kullanıma kadar yaşatır; fark, `Drop` varken son kullanımın kapsam sonundaki düşürme olmasıdır.

### Derleyici `drop` gövdesine bakmaz

Yıkıcının referansı gerçekten okuması gerekmez. Gövde boş olsa bile kural uygulanır:

```rust,compile_fail,E0597
struct K<T>(T);

impl<T> Drop for K<T> {
    fn drop(&mut self) {}
}

fn main() {
    let k;
    let x = 5u8;
    k = K(&x);
}
```

Derleyicinin yanıtı:

```text
error[E0597]: `x` does not live long enough
   |
   = note: values in a scope are dropped in the opposite order they are defined
```

Derleyici, sınırı olmayan bir `T` için "gövde bu değere bakamaz" diye düşünebilirdi; bir zamanlar öyle de yapıyordu. Ancak bu akıl yürütme kırılgandı ve impl özelleştirme (specialization) gibi tasarlanan özelliklerle bağdaşmıyordu. RFC 1238 onu kaldırdı: `Drop` impl'i varsa bütün parametreler canlı olmak zorundadır.[^rfc1238] Gövdeye bakılmadığı için `drop` gövdesini sonradan değiştirmeniz, kullanıcı kodunun derlenip derlenmemesini etkilemez.

Hatanın doğması için `T`'nin gerçekten bir ödünç taşıması gerekir. `K(5u8)` ya da `K(String::new())` yazsaydınız sıra önemsiz olurdu; `Denetci<'static>` için de öyledir, çünkü `'static` her zaman geçerlidir.

### Değiştirilebilir ödünçte aynı kural

Kural, `&mut` tutan bir türde, düşmeden önceki değişiklikleri de engeller. *Rust for Rustaceans* şu örneği verir:[^rfr] `Foo`'nun `Drop`'u yoksa kod derlenir.

```rust
struct Foo<'a>(&'a mut bool);

fn main() {
    let mut x = true;
    let foo = Foo(&mut x);
    *foo.0 = true; // foo'nun son kullanımı
    x = false;
    println!("{x}");
}
```

Çıktı:

```text
false
```

`Drop` eklenince `foo`'nun son kullanımı kapsam sonuna kayar ve `x` o ana kadar ödünçte kalır:

```rust,compile_fail,E0506
struct Foo<'a>(&'a mut bool);

impl Drop for Foo<'_> {
    fn drop(&mut self) {
        println!("{}", self.0);
    }
}

fn main() {
    let mut x = true;
    let foo = Foo(&mut x);
    *foo.0 = true;
    x = false;
    println!("{x}");
}
```

Derleyicinin yanıtı:

```text
error[E0506]: cannot assign to `x` because it is borrowed
```

İzin verilseydi yıkıcı, yalnızca kendisinin değiştirebildiğini sandığı veriyi başkası değiştirdikten sonra okurdu.

### Ne yapmalı?

Çoğu durumda ödünç verilen değişkeni önce tanımlamak yeter. Sırayı değiştiremiyorsanız ödünç alan değeri elle, erkenden düşürebilirsiniz. `drop(d)` değeri taşır; kapsam sonunda düşürülecek bir şey kalmaz:

```rust
struct Denetci<'a>(&'a u8);

impl Drop for Denetci<'_> {
    fn drop(&mut self) {
        println!("düştü {}", self.0);
    }
}

fn main() {
    let d;
    let gun = 5u8;
    d = Denetci(&gun);
    drop(d);
    println!("bitti");
}
```

Çıktı:

```text
düştü 5
bitti
```

Üçüncü yol ödünç almaktan vazgeçmektir: Alan `&'a u8` yerine `u8` ya da `Arc<...>` olursa değer kendi verisinin sahibi olur ve sıra önemini yitirir.

## Drop glue: `Drop` yazmayan türler de düşer

Bir türün `Drop` impl'i olmasa bile alanlarının yıkıcısı olabilir. Derleyici o türü düşürmek için alanları sırayla düşüren bir kod üretir; buna *drop glue* denir. Denetim bu kodu da izler:

```rust,compile_fail,E0597
struct Denetci<'a>(&'a u8);

impl Drop for Denetci<'_> {
    fn drop(&mut self) {
        println!("{}", self.0);
    }
}

struct Sarma<'a> {
    ic: Denetci<'a>,
}

fn main() {
    let s;
    let gun = 5u8;
    s = Sarma { ic: Denetci(&gun) };
    println!("{}", s.ic.0);
}
```

Derleyicinin yanıtı:

```text
error[E0597]: `gun` does not live long enough
   |
   | borrow might be used here, when `s` is dropped and runs the destructor for type `Sarma<'_>`
   |
   = note: values in a scope are dropped in the opposite order they are defined
```

Önceki mesajda ``runs the `Drop` code`` yazıyordu, burada `runs the destructor` yazar; aradaki fark budur. Bir türün drop glue'ı olup olmadığını `needs_drop` ile sorabilirsiniz:[^needs]

```rust
use std::marker::PhantomData;
use std::mem::{ManuallyDrop, needs_drop};

struct Denetci<'a>(&'a u8);

impl Drop for Denetci<'_> {
    fn drop(&mut self) {
        println!("{}", self.0);
    }
}

# #[allow(dead_code)]
struct Sarma<'a> {
    ic: Denetci<'a>,
}

fn main() {
    println!("&u8                    {}", needs_drop::<&u8>());
    println!("Denetci                {}", needs_drop::<Denetci<'_>>());
    println!("Sarma                  {}", needs_drop::<Sarma<'_>>());
    println!("PhantomData<Denetci>   {}", needs_drop::<PhantomData<Denetci<'_>>>());
    println!("ManuallyDrop<Denetci>  {}", needs_drop::<ManuallyDrop<Denetci<'_>>>());
}
```

Çıktı:

```text
&u8                    false
Denetci                true
Sarma                  true
PhantomData<Denetci>   false
ManuallyDrop<Denetci>  false
```

Standart kütüphane belgesi bugünkü kuralları şöyle sıralar; ayrıntıların kararlı bir güvence olmadığını da ekler:[^dropck]

1. Türün drop glue'ı yoksa düşürülürken hiçbir şeyin canlı olması gerekmez. `PhantomData`, sıfır uzunluklu diziler ve `ManuallyDrop` yıkıcısız sayılır.
2. Drop glue'ı varsa alanların *sahip olduğu* türler tek tek incelenir ve her birinin gerektirdikleri toplanır. `PhantomData`, `Box`, demet ve dizilerin içine girilir. Referansta, ham işaretçide ve fonksiyon işaretçisinde durulur, çünkü bunlar bir şeye sahip değildir.
3. Türün kendi `Drop` impl'i de varsa bütün jenerik parametreler canlı olmalıdır.

`Vec` ve `Box` kendi `Drop` impl'leri olduğu hâlde üçüncü kurala takılmaz. Aşağıdaki kod, ters sıraya rağmen derlenir:

```rust
fn main() {
    let mut v: Vec<&u8> = Vec::new();
    let kutu: Box<&u8>;
    let gun = 5u8;
    v.push(&gun);
    kutu = Box::new(&gun);
    println!("{v:?} {kutu}");
}
```

Çıktı:

```text
[5] 5
```

Öğe türü `&u8` yerine `Denetci` olsaydı kod reddedilirdi; `Vec` öğelerini düşürür, öğelerin yıkıcısı da `gun`'u okur. `Vec`'in bu ayrımı yapabilmesi, bölümün sonunda göreceğimiz `#[may_dangle]` sayesindedir.

## `PhantomData`: yer kaplamayan işaret

Rust kullanılmayan bir tür parametresine izin vermez:

```rust,compile_fail,E0392
struct Bos<T> {
    id: u32,
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0392]: type parameter `T` is never used
  |
1 | struct Bos<T> {
  |            ^ unused type parameter
  |
  = help: consider removing `T`, referring to it in a field, or using a marker such as `PhantomData`
  = help: if you intended `T` to be a const parameter, use `const T: /* Type */` instead
```

Yasak keyfî değildir: `T` hiçbir alanda geçmiyorsa derleyici varyansını, `Send`/`Sync` üyeliğini ve düşürme davranışını çıkaramaz. `PhantomData<T>` bu bilgiyi veren, yer kaplamayan bir alandır:[^phantom]

```rust
use std::marker::PhantomData;
use std::mem::size_of;

struct Belirtec<T> {
    id: u32,
    _sahip: PhantomData<T>, // mantıken bir T'ye sahip; bellekte T yok
}

fn main() {
    let b: Belirtec<String> = Belirtec { id: 7, _sahip: PhantomData };
    println!("id = {}", b.id);
    println!("Belirtec<String> = {} bayt", size_of::<Belirtec<String>>());
    println!("u32              = {} bayt", size_of::<u32>());
    println!("PhantomData      = {} bayt", size_of::<PhantomData<[u64; 100]>>());
}
```

Çıktı:

```text
id = 7
Belirtec<String> = 4 bayt
u32              = 4 bayt
PhantomData      = 0 bayt
```

Derleyici tür üzerinde hesap yaparken `PhantomData<T>`'yi "burada bir `T` varmış" diye görür. İçine ne yazdığınız, hangi ilişkiyi bildirdiğinizi belirler. Nomicon'daki tablo şöyledir:[^tablo]

| İşaret | `'a` | `T` | `Send` / `Sync` | Düşerken sarkabilir mi? |
| --- | --- | --- | --- | --- |
| `PhantomData<T>` | - | kovaryant | `T`'den gelir | hayır, "`T`'ye sahibim" |
| `PhantomData<&'a T>` | kovaryant | kovaryant | `T: Sync` ise ikisi de | evet |
| `PhantomData<&'a mut T>` | kovaryant | invaryant | `T`'den gelir | evet |
| `PhantomData<*const T>` | - | kovaryant | ikisi de değil | evet |
| `PhantomData<*mut T>` | - | invaryant | ikisi de değil | evet |
| `PhantomData<fn(T)>` | - | kontravaryant | ikisi de | evet |
| `PhantomData<fn() -> T>` | - | kovaryant | ikisi de | evet |
| `PhantomData<fn(T) -> T>` | - | invaryant | ikisi de | evet |
| `PhantomData<Cell<&'a ()>>` | invaryant | - | yalnızca `Send` | evet |

İşaretin auto trait'lere etkisi gerçektir. Aşağıdaki struct'ta hiçbir işaretçi yoktur, ama varmış gibi sayılır ([25. bölüm](25-send-sync.md)):

```rust,compile_fail,E0277
use std::marker::PhantomData;

struct Ham {
    p: PhantomData<*const u8>,
}

fn send_ister<T: Send>() {}

fn main() {
    send_ister::<Ham>();
}
```

Derleyicinin yanıtı:

```text
error[E0277]: `*const u8` cannot be sent between threads safely
   |
   = help: within `Ham`, the trait `Send` is not implemented for `*const u8`
```

### `PhantomData` düşürme denetimini ne zaman etkiler?

`*mut T` tutan ve `Drop` uygulayan bir türe `PhantomData<T>` eklemek, düşürme denetimi açısından hiçbir şeyi değiştirmez: `Drop` impl'i zaten `T`'nin canlı olmasını gerektirir. Eski kaynaklar bu alanı "denetleyici `T`'yi düşürdüğümü bilsin" diye zorunlu gösterir; RFC 1238'den beri öyle değildir. Alan yine de varyansı ve auto trait üyeliğini belirler.[^sahip]

Bağımsız etkisi, `Drop` impl'i olmayan ama drop glue'ı olan bir türde görülür. Aşağıda `Etiketli`'nin düşürülecek bir `String` alanı vardır; `PhantomData<T>` de `T`'nin düşürülebilir durumda olmasını şart koşar:

```rust,compile_fail,E0597
use std::marker::PhantomData;

struct Denetci<'a>(&'a u8);

impl Drop for Denetci<'_> {
    fn drop(&mut self) {
        println!("{}", self.0);
    }
}

struct Etiketli<T> {
    ad: String,
    _sahip: PhantomData<T>,
}

fn etiketle<T>(_: &T, ad: &str) -> Etiketli<T> {
    Etiketli { ad: ad.to_string(), _sahip: PhantomData }
}

fn main() {
    let e;
    let gun = 5u8;
    let d = Denetci(&gun);
    e = etiketle(&d, "a");
    println!("{}", e.ad);
}
```

Derleyicinin yanıtı:

```text
error[E0597]: `gun` does not live long enough
   |
   | borrow might be used here, when `e` is dropped and runs the destructor for type `Etiketli<Denetci<'_>>`
```

`ad` alanı `u32` olsaydı tür yıkıcısız kalır, birinci kural gereği kod derlenirdi. `T` yerine `&u8` geçseydi de derlenirdi, çünkü bir referansı düşürmek hiçbir şey yapmaz. Tek başına `PhantomData<T>` bu yüzden `E0597` doğurmaz.

## `#[may_dangle]`: kuralı gevşeten söz

Kendi `Box`'ınızı yazdığınızı düşünün. Yıkıcısı içerideki `T`'yi yalnızca düşürür, ona başka türlü dokunmaz. Yine de `impl<T> Drop for KendiBox<T>` yazdığınız için üçüncü kural bütün `T`'lerin canlı olmasını ister ve `KendiBox<&u8>` ters sırada derlenmez. Oysa std'nin `Box<&u8>`'i derleniyordu. Aradaki farkı `#[may_dangle]` özniteliği yaratır: "Bu `T`'ye düşürmek dışında dokunmayacağıma söz veriyorum."[^maydangle]

Öznitelik kararsızdır. Aşağıdaki örnek, `dropck_eyepatch` özelliği açılmış bir nightly derleyici ister; kararlıda derlenmediği için burada çalıştırılmaz:

```rust,ignore
#![feature(dropck_eyepatch)]
use std::marker::PhantomData;

struct KendiBox<T> {
    p: *mut T,
    _sahip: PhantomData<T>,
}

impl<T> KendiBox<T> {
    fn yeni(deger: T) -> KendiBox<T> {
        KendiBox { p: Box::into_raw(Box::new(deger)), _sahip: PhantomData }
    }

    fn al(&self) -> &T {
        // SAFETY: `p`, Box::into_raw'dan gelir ve drop'a kadar geçerlidir.
        unsafe { &*self.p }
    }
}

// SAFETY: drop, T'ye yalnızca onu düşürerek dokunur.
unsafe impl<#[may_dangle] T> Drop for KendiBox<T> {
    fn drop(&mut self) {
        // SAFETY: `p`, Box::into_raw'dan gelir ve yalnızca burada geri alınır.
        drop(unsafe { Box::from_raw(self.p) });
    }
}

fn main() {
    let k;
    let x = 5u8;
    k = KendiBox::yeni(&x); // T = &u8, ters sıra
    println!("{}", k.al());
}
```

Nightly'de derlenir ve `5` yazar. `impl`'in `unsafe` olması gerekir, çünkü söz yanlışsa yıkıcı düşmüş veriyi okur. Kararlı derleyici özniteliği reddeder:

```rust,compile_fail,E0658
struct KendiBox<T>(*mut T);

unsafe impl<#[may_dangle] T> Drop for KendiBox<T> {
    fn drop(&mut self) {}
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0658]: the `may_dangle` attribute has unstable semantics and may be removed in the future
  |
  = note: see issue #34761 <https://github.com/rust-lang/rust/issues/34761> for more information
```

`PhantomData<T>` alanı bu kez süs değildir. Söz "`T`'ye düşürmek dışında dokunmam" der; düşürdüğünü ise `PhantomData<T>` bildirir. Alanı kaldırırsanız denetleyici yalnızca `*mut T` görür, ham işaretçide durur ve `T`'nin yıkıcısını hesaba katmaz. `KendiBox<Denetci>` o zaman ters sırada derlenir ve `Denetci`'nin yıkıcısı düşmüş `gun`'u okur. Alan varken aynı kod `E0597` ile reddedilir. Standart kütüphane `Vec` ve `Box` türlerinde ikisini bu nedenle birlikte kullanır.[^std] Günlük kodda bu özniteliğe ihtiyacınız olmaz.

## Hata kodu başvurusu

| Kod | Ne zaman görülür? | Ne yapmalı? |
| --- | --- | --- |
| `E0597` | Yıkıcısı olan bir değer, ödünç aldığı veriden sonra düştüğünde | Veriyi önce tanımlayın, ödünç alanı erken düşürün ya da sahipliği verin. |
| `E0506` | `Drop`'lu bir değerin `&mut` ile tuttuğu değişkene atama yapıldığında | Atamadan önce `drop(deger)` çağırın. |
| `E0392` | Bir tür parametresi hiçbir alanda geçmediğinde | `PhantomData<...>` alanı ekleyin. |
| `E0277` | `PhantomData<*const T>` taşıyan türden `Send`/`Sync` istendiğinde | İşareti amaca göre seçin: örneğin `PhantomData<T>`. |
| `E0658` | Kararlıda `#[may_dangle]` kullanıldığında | Öznitelikten vazgeçin; `Vec` ya da `Box` üzerine kurun. |

## Perde arkası: Derleyici ne yapıyor?

> [!NOTE]
> Bu başlık ilk okuyuşta atlanabilir; yukarıdaki kuralları anlamış olmanız yeterlidir.

**Düşürmenin gerektirdikleri.** `trivial_dropck_outlives`, düşürülürken hiçbir şey gerektirmeyen türleri (tamsayı, referans, ham işaretçi gibi) baştan ayırır. Kalanlar için `dtorck_constraint_for_ty_inner` türün içine yukarıdaki kurallarla iner ve canlı kalması gereken türleri ve ömürleri toplar.[^outlives] Trait nesnelerinde (`dyn Trait`) somut tür bilinmediği için türün tamamı canlı sayılır. Kendi `Drop` impl'i olan türlerde hangi parametrelerin listeye gireceğini `destructor_constraints` belirler; `#[may_dangle]` ile işaretlenenler listeden çıkarılır.[^destructor]

**Ödünç denetleyicisi.** Bir yerel değişken için iki ayrı canlılık hesaplanır: sıradan kullanım canlılığı ve *düşürme canlılığı*. `compute_drop_live_points_for`, değişkenin ileride düşürüleceği noktaları bulur; `add_drop_live_facts_for` bu noktalarda yalnızca düşürmenin gerektirdiği ömürleri canlı tutar.[^trace] `Denetci`'nin `Drop`'u olmadığında bu liste boştur; ödünç son sıradan kullanımda biter.

**Kod üretimi.** Bir türün düşürülmesi için kod üretilip üretilmeyeceğine `needs_drop_raw` sorgusu karar verir; `std::mem::needs_drop`'un yanıtı da oradan gelir.[^needsraw]

## Özet

- Yıkıcısı olan bir değerin düşürülmesi, taşıdığı ödünçlerin kullanımıdır. Ödünç verilen veri o değerden uzun yaşamalıdır; yoksa `E0597`.
- `Drop` impl'i varsa gövdeye bakılmaz, bütün jenerik parametreler canlı olmalıdır. `Drop` yoksa drop glue üzerinden alanlara bakılır; yıkıcısız türün düşürülmesi kullanım değildir.
- Yerel değişkenler tanımlandıkları sıranın tersiyle düşer; ödünç verileni önce tanımlayın.
- `PhantomData` yer kaplamaz; varyansı, `Send`/`Sync` üyeliğini ve drop glue'ı olan türlerde düşürme denetimini etkiler.
- `#[may_dangle]` kuralı gevşeten, `unsafe` ve kararsız bir sözdür; std onu `PhantomData<T>` ile birlikte kullanır.

Sıradaki bölüm [27. İleri Lifetime, HRTB ve GAT](27-lifetime-hrtb-gat.md), "her ömür için" diyen sınırları ve ömür parametresi alan ilişkili türleri anlatıyor.

## Kaynaklar

[^sira]: The Rust Reference, [Drop scopes](https://doc.rust-lang.org/reference/destructors.html#drop-scopes); hata dizini, [`E0597`](https://doc.rust-lang.org/error_codes/E0597.html).
[^dropck]: std, [`Drop`, Drop check](https://doc.rust-lang.org/std/ops/trait.Drop.html#drop-check); The Rustonomicon, [Drop Check](https://doc.rust-lang.org/nomicon/dropck.html).
[^rfc1238]: [RFC 1238: Nonparametric dropck](https://rust-lang.github.io/rfcs/1238-nonparametric-dropck.html).
[^rfr]: Jon Gjengset, *Rust for Rustaceans*, 9. bölüm ("Unsafe Code"), "The Drop Check"; hata dizini, [`E0506`](https://doc.rust-lang.org/error_codes/E0506.html).
[^needs]: std, [`needs_drop`](https://doc.rust-lang.org/std/mem/fn.needs_drop.html) ve [`ManuallyDrop`](https://doc.rust-lang.org/std/mem/struct.ManuallyDrop.html).
[^phantom]: std, [`PhantomData`](https://doc.rust-lang.org/std/marker/struct.PhantomData.html); hata dizini, [`E0392`](https://doc.rust-lang.org/error_codes/E0392.html).
[^tablo]: The Rustonomicon, [Table of `PhantomData` patterns](https://doc.rust-lang.org/nomicon/phantom-data.html#table-of-phantomdata-patterns).
[^sahip]: The Rustonomicon, [Generic parameters and drop-checking](https://doc.rust-lang.org/nomicon/phantom-data.html#generic-parameters-and-drop-checking).
[^maydangle]: The Rustonomicon, [An escape hatch](https://doc.rust-lang.org/nomicon/dropck.html#an-escape-hatch); [RFC 1327: `dropck_eyepatch`](https://rust-lang.github.io/rfcs/1327-dropck-param-eyepatch.html); rustc 1.99.0 kaynak kodu, [`unstable.rs`, `dropck_eyepatch`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_feature/src/unstable.rs#L324).
[^std]: rustc 1.99.0 kaynak kodu, [`boxed.rs`, `may_dangle`](https://github.com/rust-lang/rust/blob/1.99.0/library/alloc/src/boxed.rs#L1990); [`vec/mod.rs`, `may_dangle`](https://github.com/rust-lang/rust/blob/1.99.0/library/alloc/src/vec/mod.rs#L4350); [`raw_vec/mod.rs`, `_marker: PhantomData<T>`](https://github.com/rust-lang/rust/blob/1.99.0/library/alloc/src/raw_vec/mod.rs#L75); [`unique.rs`, `_marker`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/ptr/unique.rs#L42).
[^outlives]: rustc 1.99.0 kaynak kodu, [`dropck_outlives.rs`, `trivial_dropck_outlives`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_trait_selection/src/traits/query/dropck_outlives.rs#L28) ve [`dtorck_constraint_for_ty_inner`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_trait_selection/src/traits/query/dropck_outlives.rs#L255).
[^destructor]: rustc 1.99.0 kaynak kodu, [`rustc_middle/src/ty/util.rs`, `destructor_constraints`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_middle/src/ty/util.rs#L463).
[^trace]: rustc 1.99.0 kaynak kodu, [`trace.rs`, `compute_drop_live_points_for`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_borrowck/src/type_check/liveness/trace.rs#L298) ve [`add_drop_live_facts_for`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_borrowck/src/type_check/liveness/trace.rs#L551).
[^needsraw]: rustc 1.99.0 kaynak kodu, [`rustc_ty_utils/src/needs_drop.rs`, `needs_drop_raw`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_ty_utils/src/needs_drop.rs#L17).
