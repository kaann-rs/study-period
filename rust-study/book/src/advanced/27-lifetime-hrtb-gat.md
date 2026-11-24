# 27. İleri Lifetime, HRTB ve GAT

[9. bölüm](../intermediate/09-generics-traits-lifetimes.md) ömrü (lifetime) "bu çıktı şu girdiden türedi" bildirimi olarak tanıttı; [21. bölüm](21-variance.md) ömürler arasındaki alt tür ilişkisini açtı. Günlük kodun büyük kısmına bu kadarı yeter. Yetmediği yerlerde üç soru ortaya çıkar:

1. İki ömür arasındaki ilişkiyi (`'a: 'b`, `T: 'a`) ne zaman elle yazmak gerekir?
2. Ömrü, fonksiyonu çağıranın değil, fonksiyonun gövdesinin seçtiği durumlar nasıl ifade edilir?
3. Bir trait'in ilişkili türü (associated type) kendisi de bir ömre ya da türe bağlıysa ne yapılır?

Yanıtlar sırasıyla *ömür sınırları* (lifetime bounds), *yüksek dereceli trait sınırları* (higher-ranked trait bounds, HRTB) ve *jenerik ilişkili türlerdir* (generic associated types, GAT). Tek cümleyle: Ömür etiketleri hangi referansın neye bağlı olduğu bilgisini taşır; `for<'a>` "hangi ömrü verirseniz verin çalışırım" sözüdür; GAT ise trait'in içindeki türe, her kullanımda yeniden doldurulan boş bir hane ekler.

Bölüm, [ödünç almayı](../basics/03-ownership.md), ömürleri ve ilişkili türleri, [closure'ları ve `Iterator`'ı](../intermediate/12-closures-iterators.md) bildiğinizi varsayar. Her aracı, onsuz derlenmeyen bir örnekle birlikte göreceksiniz; çünkü bu araçların hiçbiri yalnızca tanımından anlaşılmaz.

> [!NOTE]
> Bu bölümün kodu `learn/advanced-concepts/advanced-lifetime-hrtb-gat` dizinindedir. Depo kökünden `cargo run -p advanced-lifetime-hrtb-gat` komutuyla çalıştırabilir, `cargo test -p advanced-lifetime-hrtb-gat` komutuyla sınayabilirsiniz. `hex_v2` ve `hex_v3` örnekleri `learn/basic-concepts` dizininden gelir. Sayfadaki hata mesajları rustc 1.99.0 ile alınmıştır.

## Günlük hayattan bir benzetme

Üzerinde tarih yazan bir bilet düşünün. Sıradan bir bilette tarihi bileti veren kişi yazar; siz o tarihe uyarsınız, tarih de sizin açınızdan artık sabittir. Bu, `fn f<'a>(..)` yazımındaki ömür parametresidir: Ömrü çağıran seçer, fonksiyonun gövdesi içinde o ömür tek ve sabittir.

Kimi bilette ise tarih boştur. Girişte tarihi siz seçersiniz, görevli de "hangi tarih olursa olsun kabulüm" der. Bu, `for<'a>` yazımıdır: Ömrü içeriden biz seçeriz, karşı taraf her ömre razı olmak zorundadır.

Son olarak, biletin üzerindeki koltuk numarası her gidişte değişiyorsa ona ayrı bir boş hane gerekir. GAT, trait'in içindeki türe böyle bir hane ekler: `type Oge<'a>` her kullanımda başka bir ömürle doldurulur.

<details>
<summary>Bu bölümde geçen terimler</summary>

- **ömür (lifetime, `'a`):** Bir referansın geçerli olduğu kod aralığının adı. Hiçbir şeyin ömrünü uzatmaz; yalnızca ilişkiyi etiketler.
- **`'static`:** "Geçici hiçbir şeye borçlu değil" etiketi. Sonsuza dek yaşamak demek değildir: Bir `String` de `'static` sayılır ve hemen düşürülebilir.
- **ömür çıkarımı (elision):** Derleyicinin, yazılmamış ömür etiketlerini kurala göre tamamlaması.
- **sınır (bound):** Bir jenerik parametreye konan şart: `T: Debug`, `'a: 'b`.
- **kapsamak, uzun yaşamak (outlives, `'a: 'b`):** "`'a`, `'b` bitmeden bitmez"; yani en az onun kadar sürer.
- **`T: 'a`:** "`T`'nin içindeki bütün referanslar en az `'a` kadar geçerlidir."
- **çıkarılan sınırlar (implied bounds):** Derleyicinin, tür yazımına bakıp kendiliğinden eklediği şartlar.
- **kovaryant / invaryant:** Ömrün kısaltılabildiği yer / hiç esnetilemediği yer. [21. bölüm](21-variance.md) anlatır.
- **closure:** Yerinde yazılan adsız fonksiyon: `|s| s.trim()`.
- **`Fn`, `FnMut`, `FnOnce`:** Çağrılabilir şeylerin (closure'ların ve fonksiyonların) trait'leri.
- **`fn` işaretçisi:** Bir fonksiyonun adresini tutan değer: `fn(&str) -> &str`.
- **HRTB, `for<'a>`:** Yüksek dereceli trait sınırı: "her `'a` ömrü için geçerli" şartı.
- **evrensel niceleme (universal quantification):** "Her ... için" demenin matematikteki adı.
- **ilişkili tür (associated type):** Trait'in içinde adı konan, `impl` bloğunda doldurulan tür: `type Item`.
- **GAT:** Kendi parametresi olan ilişkili tür: `type Oge<'a>` ya da `type Isaretci<T>`.
- **ödünç veren iterator (lending iterator):** Verdiği öğe kendisinden ödünç olan gezici.
- **dilim ve pencere (slice, window):** Art arda duran öğeler; dilimin sabit boylu, kayan bir parçası.
- **tür kurucusu, tür ailesi (type constructor):** İçi henüz doldurulmamış tür kalıbı: `Rc<_>`, `Arc<_>`.
- **trait nesnesi (trait object, `dyn`):** Gerçek türü gizlenmiş değer. Trait'i `dyn Trait` yapılabiliyorsa *dyn uyumludur* (dyn compatible).
- **tampon (buffer):** Verinin geçici olarak durduğu bellek alanı.
- **sıfır kopya (zero-copy):** Veriyi kopyalamadan, durduğu yere bakarak kullanmak.
- **sahipli (owned):** Kimseden ödünç almayan, verisini kendi tutan değer: `String`, `u32`.
- **serde, `Deserialize`:** Rust'ın yaygın veri çözümleme kütüphanesi ve onun trait'i.
- **geç bağlanan (late-bound) ömür:** Fonksiyon adlandırılırken değil, her çağrıda ayrı ayrı seçilen ömür.
- **projeksiyon (projection):** `<I as Trait>::Oge` yazımı: "`I`'nın bu trait'teki `Oge` türü".
- **silinme (erasure):** Ömürlerin derleme bitince koddan kaybolması.

</details>

## Zihinsel model: Ömrü kim seçer?

Bölümün tamamı tek bir soruya dayanır: Jenerik bir parametreyi **kim** seçer? Üç yazım, bu soruya üç ayrı yanıt verir:[^hrtb][^nomicon]

| Yazım | Ömrü kim seçer? | Kim her seçime razı olur? |
| --- | --- | --- |
| `fn f<'a>(x: &'a str)` | `f`'yi çağıran kod | `f`'nin gövdesi; gövde içinde `'a` sabit, tek bir süredir. |
| `F: for<'a> Fn(&'a T)` | `F` türündeki değeri çağıran, yani fonksiyonun gövdesi | `F` yerine verilen closure ya da fonksiyon |
| `type Oge<'a>` | Türü kullanan her yer, ayrı ayrı | Trait'i uygulayan tür |

Birinci satır alıştığınız durumdur. İkinci satır "kim seçer" sorusunu tersine çevirir; HRTB, bu tersine çevirmenin sözdizimidir. Üçüncü satır aynı esnekliği ilişkili türe taşır. Aşağıdaki dört başlık bu tablonun ayrıntısıdır.

## Ömürler arasındaki ilişkiler

Tek bir `'a` çoğu zaman yeter. İlişkiyi elle yazmanın gerektiği yerler dört biçimde karşınıza çıkar; dördünde de yapılan iş, derleyicinin kendi başına bilemeyeceği bir ilişkiyi imzaya yazmaktır.

### Struct'ta iki ayrı ömür

Bir metinde bir kelimenin geçtiği yerleri bulan `Arayici` yapısını düşünün. İki referans alanına aynı `'a` verilirse derleyici ikisini ortak, yani *kısa* olan süreye indirger; paylaşımlı referanslar kovaryant olduğu için bunu yapabilir.[^varyans] Struct'tan çıkan her şey o kısa süreye bağlanır:

```rust,compile_fail,E0597
struct Arayici<'a> {
    metin: &'a str,
    kelime: &'a str,
}

impl<'a> Arayici<'a> {
    fn eslesmeler(&self) -> Vec<&'a str> {
        self.metin
            .match_indices(self.kelime)
            .map(|(i, _)| &self.metin[i..i + self.kelime.len()])
            .collect()
    }
}

fn main() {
    let metin = String::from("kar yağar, kar erir, kar kalmaz");
    let bulunan;
    {
        let kelime = String::from("kar");
        bulunan = Arayici { metin: &metin, kelime: &kelime }.eslesmeler();
    } // kelime burada düşürülür
    println!("{bulunan:?}");
}
```

Derleyicinin yanıtı:

```text
error[E0597]: `kelime` does not live long enough
   |
19 |         let kelime = String::from("kar");
   |             ------ binding `kelime` declared here
20 |         bulunan = Arayici { metin: &metin, kelime: &kelime }.eslesmeler();
   |                                                    ^^^^^^^ borrowed value does not live long enough
21 |     } // kelime burada düşürülür
   |     - `kelime` dropped here while still borrowed
22 |     println!("{bulunan:?}");
   |                ------- borrow later used here
```

Sonuçtaki dilimlerin hepsi `metin`'in içindedir; `kelime`'ye yalnızca arama sırasında ihtiyaç vardır. Bir kitapta bazı cümlelerin altını kurşun kalemle çizdiğinizi düşünün: Çizgiler kitabın içindedir, kalem kırılsa da yerinde durur. `metin` kitap, `kelime` kalem, sonuç da altı çizili yerlerdir. İkisine aynı etiketi vermek "çizgiler kalem kadar yaşar" demektir. Bağımsız yaşayan veriler için bağımsız parametreler gerekir:

```rust
struct Arayici<'m, 'k> {
    metin: &'m str,
    kelime: &'k str,
}

impl<'m, 'k> Arayici<'m, 'k> {
    // Sonuç metnin içindedir; kelimeye de Arayici'nin kendisine de bağlı değildir.
    fn eslesmeler(&self) -> Vec<&'m str> {
        self.metin
            .match_indices(self.kelime)
            .map(|(i, _)| &self.metin[i..i + self.kelime.len()])
            .collect()
    }
}

fn main() {
    let metin = String::from("kar yağar, kar erir, kar kalmaz");
    let bulunan;
    {
        let kelime = String::from("kar");
        let arayici = Arayici { metin: &metin, kelime: &kelime };
        bulunan = arayici.eslesmeler();
    } // kelime ve arayici düştü; sonuç yalnızca metne bağlı
    println!("eşleşme sayısı = {}", bulunan.len());
}
```

Çıktı:

```text
eşleşme sayısı = 3
```

### Bir ömür ötekini kapsar: `'a: 'b`

İki farklı ömürlü referanstan birini döndürmek istediğinizde derleyici, bir ömrün yerine ötekinin geçip geçemeyeceğini bilmek zorundadır:

```rust,compile_fail
fn sec<'a, 'b>(x: &'a str, y: &'b str) -> &'b str {
    if y.is_empty() { x } else { y }
}
```

Derleyicinin yanıtı:

```text
error: lifetime may not live long enough
  |
1 | fn sec<'a, 'b>(x: &'a str, y: &'b str) -> &'b str {
  |        --  -- lifetime `'b` defined here
  |        |
  |        lifetime `'a` defined here
2 |     if y.is_empty() { x } else { y }
  |                       ^ function was supposed to return data with lifetime `'b` but it is returning data with lifetime `'a`
  |
  = help: consider adding the following bound: `'a: 'b`
```

Dönüş türü `'b` boyunca geçerli bir referans söz veriyor; oysa `x`, `'a` bitince geçersizleşebilir. Çözümü mesajın son satırı söylüyor: `'a: 'b` sınırı. Bu yazım "`'a`, `'b`'den uzun yaşar" (`'a` outlives `'b`) diye okunur ve `'a`'nın *en az* `'b` kadar sürdüğünü bildirir.[^omur-siniri]

Maçta sakatlanan oyuncunun yerine yedek girer. Yedeğin en az maçın sonuna kadar sahada kalabilmesi gerekir; ilk yarıda eve gidecek biri yedek olamaz. Aşağıdaki fonksiyonda yedek, `varsayilan` parametresidir:

```rust
fn bos_ise_varsayilan<'kisa, 'uzun: 'kisa>(
    girdi: &'kisa str,
    varsayilan: &'uzun str,
) -> &'kisa str {
    if girdi.trim().is_empty() { varsayilan } else { girdi }
}

// sec fonksiyonunun derlenen hâli
fn sec2<'a: 'b, 'b>(x: &'a str, y: &'b str) -> &'b str {
    if y.is_empty() { x } else { y }
}

fn main() {
    let varsayilan = String::from("anonim");
    let ad = {
        let girdi = String::from("   ");
        bos_ise_varsayilan(&girdi, &varsayilan).to_string()
    };
    println!("boş girdi: {ad}");
    println!("sec2: {}", sec2("x", "y"));
}
```

Çıktı:

```text
boş girdi: anonim
sec2: y
```

Sınır yalnızca gövdeyi derletmez; derleyici aynı şartı her çağrı yerinde de arar. Yedeğin, sonucun kullanıldığı süre boyunca geçerli kalması gerekir:

```rust,compile_fail,E0597
# fn bos_ise_varsayilan<'kisa, 'uzun: 'kisa>(
#     girdi: &'kisa str,
#     varsayilan: &'uzun str,
# ) -> &'kisa str {
#     if girdi.trim().is_empty() { varsayilan } else { girdi }
# }
fn main() {
    let girdi = String::from("ada");
    let secilen;
    {
        let varsayilan = String::from("anonim");
        secilen = bos_ise_varsayilan(&girdi, &varsayilan);
    }
    println!("{secilen}");
}
```

Derleyicinin yanıtı:

```text
error[E0597]: `varsayilan` does not live long enough
```

Bu iki parametreli imza, tek ömürlü `fn f<'a>(girdi: &'a str, varsayilan: &'a str) -> &'a str` imzasından daha esnek değildir: Paylaşımlı referanslar kovaryant olduğundan çağıran, uzun olanı kendiliğinden kısaltır. Ömürleri ayrı adlandırmak ilişkiyi görünür kılar; iki ömür zaten ayrı adlandırılmışsa, örneğin `Arayici<'m, 'k>` gibi bir struct'ın metodunda iki alandan birini döndürüyorsanız, sınırı yazmak zorunlu hâle gelir:

```rust
# struct Arayici<'m, 'k> {
#     metin: &'m str,
#     kelime: &'k str,
# }
impl<'m, 'k: 'm> Arayici<'m, 'k> {
    // 'k: 'm olmasaydı else dalı derlenmezdi.
    fn metin_ya_da_kelime(&self) -> &'m str {
        if self.metin.contains(self.kelime) { self.metin } else { self.kelime }
    }
}

fn main() {
    let arayici = Arayici { metin: "kar yağar", kelime: "yağmur" };
    println!("{}", arayici.metin_ya_da_kelime());
}
```

Çıktı:

```text
yağmur
```

### Tür üzerinde sınır: `T: 'a`

`T: 'a` sınırı, `T`'nin bütün ömür parametrelerinin `'a`'yı kapsadığını söyler.[^omur-siniri] Başka deyişle `T`'nin içinde bir referans varsa, o referans en az `'a` boyunca geçerlidir; `i32` ya da `String` gibi ömür parametresi olmayan türler bu şartı her `'a` için sağlar. Çantanızda kütüphaneden ödünç alınmış bir kitap varsa o çantayı iade tarihinden sonra taşıyamazsınız; `T: 'a`, iade tarihini imzaya yazmaktır.

Bu sınır, bir `T`'yi `'a` ömürlü bir kutuya ya da trait nesnesine koyarken gerekir. Yoksa kutu, `T`'nin içindeki bir referanstan uzun yaşayabilirdi:

```rust,compile_fail,E0309
use std::fmt::Debug;

fn kutula<'a, T: Debug>(t: T) -> Box<dyn Debug + 'a> {
    Box::new(t)
}
```

Derleyicinin yanıtı:

```text
error[E0309]: the parameter type `T` may not live long enough
  |
3 | fn kutula<'a, T: Debug>(t: T) -> Box<dyn Debug + 'a> {
  |           -- the parameter type `T` must be valid for the lifetime `'a` as defined here...
4 |     Box::new(t)
  |     ^^^^^^^^^^^ ...so that the type `T` will meet its required lifetime bounds
  |
help: consider adding an explicit lifetime bound
  |
3 | fn kutula<'a, T: Debug + 'a>(t: T) -> Box<dyn Debug + 'a> {
  |                        ++++
```

Dönüş türü `Box<dyn Debug>` olsaydı, yazılmayan ömür `'static` sayılır[^dyn-omur] ve aynı eksiklik bu kez E0310 koduyla bildirilirdi: `` the parameter type `T` may not live long enough ``. Sınır eklendiğinde fonksiyon hem sahipli hem ödünç değerlerle çalışır:

```rust
use std::fmt::Debug;

fn kutula<'a, T: Debug + 'a>(t: T) -> Box<dyn Debug + 'a> {
    Box::new(t)
}

fn main() {
    let sayi = kutula(42); // i32'nin içinde ödünç yok: her ömre uyar

    let yerel = String::from("yerel veri");
    let kutu = kutula(&yerel); // T = &String; kutu, yerel yaşadığı sürece kullanılabilir

    println!("{sayi:?}, kutuda: {kutu:?}");
}
```

Çıktı:

```text
42, kutuda: "yerel veri"
```

`T: 'static` aynı ailedendir ve sık yanlış okunur. "Sonsuza dek yaşar" demek değildir; "geçici hiçbir şeye borçlu değildir" demektir. Bir `String` bu şartı sağlar ve hemen ardından düşürülebilir, yerel bir değere bakan referans ise sağlamaz:

```rust,compile_fail,E0597
fn statik_ister<T: 'static>(_: T) {}

fn main() {
    statik_ister(String::from("x")); // derlenir: String kimseye borçlu değil

    let yerel = String::from("x");
    statik_ister(&yerel);
}
```

Derleyicinin yanıtı:

```text
error[E0597]: `yerel` does not live long enough
  |
6 |     let yerel = String::from("x");
  |         ----- binding `yerel` declared here
7 |     statik_ister(&yerel);
  |     -------------^^^^^^-
  |     |            |
  |     |            borrowed value does not live long enough
  |     argument requires that `yerel` is borrowed for `'static`
8 | }
  | - `yerel` dropped here while still borrowed
```

Her yerde `T: 'a` yazmanız gerekmez. `&'a T` alanı tutan bir struct'ta derleyici bu şartı alan türünden çıkarır; çünkü `T`'nin içindeki bir referans `'a`'dan önce bitebilseydi `&'a T` türü zaten anlamlı olmazdı. Böyle eklenen şartlara *çıkarılan sınırlar* (implied bounds) denir:[^cikarilan]

```rust
// Yazılmış olsa da olurdu: struct Gosteren<'a, T: 'a>
struct Gosteren<'a, T> {
    deger: &'a T,
}

fn main() {
    let sayi = 7;
    let gosteren = Gosteren { deger: &sayi };
    println!("{}", gosteren.deger);
}
```

Çıktı:

```text
7
```

### `&mut` ardındaki ömür: invaryans

Bir listeye öğe ekleyen fonksiyon, ilk denemede çoğu zaman şöyle yazılır:

```rust,compile_fail
fn sakla(liste: &mut Vec<&str>, s: &str) {
    liste.push(s);
}
```

Derleyicinin yanıtı:

```text
error: lifetime may not live long enough
  |
1 | fn sakla(liste: &mut Vec<&str>, s: &str) {
  |                          -         - let's call the lifetime of this reference `'1`
  |                          |
  |                          let's call the lifetime of this reference `'2`
2 |     liste.push(s);
  |     ^^^^^^^^^^^^^ argument requires that `'1` must outlive `'2`
  |
  = note: requirement occurs because of a mutable reference to `Vec<&str>`
  = note: mutable references are invariant over their type parameter
  = help: see <https://doc.rust-lang.org/nomicon/subtyping.html> for more information about variance
help: consider introducing a named lifetime parameter
  |
1 | fn sakla<'a>(liste: &mut Vec<&'a str>, s: &'a str) {
  |         ++++                  ++           ++
```

Ömür çıkarımı iki `&str`'e iki *ayrı* ömür verdi. `&T`'nin ardındaki ömürler gerektiğinde kısaltılabilir; `&mut T` ise `T` üzerinde *invaryanttır*, yani `Vec<&str>`'in içindeki ömür ne kısaltılabilir ne uzatılabilir, tam tutması gerekir.[^varyans] Bu yüzden derleyici `s`'nin, listenin beklediği süre kadar yaşadığını kanıtlayamaz.

Sınıf gezisi listesinin başında "bu listedeki herkes akşama kadar bizimle" yazdığını düşünün. Öğlen ayrılacak birini o listeye yazamazsınız; yazarsanız akşam yoklamasında olmayan birini ararsınız. Gereken ilişkiyi imzaya siz yazarsınız: Eklenen şey, en az listedekiler kadar geçerlidir.

```rust
fn sakla<'a>(liste: &mut Vec<&'a str>, s: &'a str) {
    liste.push(s);
}

fn main() {
    let metin = String::from("kar yağar");
    let mut liste = Vec::new();
    sakla(&mut liste, &metin[..3]);
    sakla(&mut liste, "sabit");
    println!("liste = {liste:?}");
}
```

Çıktı:

```text
liste = ["kar", "sabit"]
```

İki yerde aynı `'a` yazması, iki ömrün baştan eşit olması gerektiği anlamına gelmez. Listenin ömrü invaryant olduğu için sabittir; `s` ise sıradan bir `&'a str`'dir ve daha uzun yaşayan bir referans (örneğin `"sabit"`) `'a`'ya kısaltılarak verilebilir. Daha *kısa* yaşayan bir referans verilemez:

```rust,compile_fail,E0597
# fn sakla<'a>(liste: &mut Vec<&'a str>, s: &'a str) {
#     liste.push(s);
# }
fn main() {
    let mut liste = Vec::new();
    sakla(&mut liste, "sabit");
    {
        let gecici = String::from("geçici");
        sakla(&mut liste, &gecici);
    } // gecici burada düşürülür
    println!("{liste:?}"); // liste hâlâ kullanılıyor
}
```

Derleyicinin yanıtı:

```text
error[E0597]: `gecici` does not live long enough
```

## HRTB: ömrü içeriden seçmek

Buraya kadar bütün ömürleri fonksiyonu çağıran seçti. Bu başlıkta seçim içeriye, fonksiyonun gövdesine geçiyor.

### Aynı görünen iki imza: `hex_v2` ve `hex_v3`

Aşağıdaki iki fonksiyon, `i8` referansı alan bir closure'ı çağırıyor. Tek farkları `'a`'nın nerede tanıtıldığıdır:

```rust
// 'a, F'nin sınırının içinde: F her ömür için çalışmak zorunda
fn hex_v2<F>(f: F) -> i8
where
    F: for<'a> Fn(&'a i8) -> i8,
{
    f(&1)
}

// 'a, fonksiyonun parametresi: çağıran tek bir ömür seçer
fn hex_v3<'a, F>(f: F) -> i8
where
    F: Fn(&'a i8) -> i8,
{
    f(&1)
}

fn main() {
    println!("{} {}", hex_v2(|x| *x), hex_v3(|x| *x));
}
```

Çıktı:

```text
1 1
```

İkisi de derlendi ve aynı sonucu verdi; demek ki bu örnek farkı göstermiyor. Nedeni bir tuzaktır: `&1` ifadesindeki sabit değer, derleyici tarafından `'static` ömürlü bir yere *terfi ettirilir* (constant promotion).[^terfi] `&1` artık `&'static i8`'dir ve `'static` her ömrü kapsadığı için çağıranın seçtiği herhangi bir `'a`'ya da uyar.

Fark, terfi edemeyen bir yerel değişkenle ortaya çıkar. `for<'a>` ile yazılan sürüm derlenir:

```rust
fn hex_v2_yerel<F>(f: F) -> i8
where
    F: for<'a> Fn(&'a i8) -> i8,
{
    let n: i8 = 1;
    f(&n)
}

fn main() {
    println!("{}", hex_v2_yerel(|x| *x + 1));
}
```

Çıktı:

```text
2
```

Ömrü çağırana seçtiren sürüm ise derlenmez:

```rust,compile_fail,E0597
fn hex_v3_yerel<'a, F>(f: F) -> i8
where
    F: Fn(&'a i8) -> i8,
{
    let n: i8 = 1;
    f(&n)
}
```

Derleyicinin yanıtı:

```text
error[E0597]: `n` does not live long enough
  |
1 | fn hex_v3_yerel<'a, F>(f: F) -> i8
  |                 -- lifetime `'a` defined here
...
5 |     let n: i8 = 1;
  |         - binding `n` declared here
6 |     f(&n)
  |     --^^-
  |     | |
  |     | borrowed value does not live long enough
  |     argument requires that `n` is borrowed for `'a`
7 | }
  | - `n` dropped here while still borrowed
```

Nedenini görmek için ömürleri iç içe kutular gibi düşünün:

```text
{                              // dış bölge = 'a: çağıran seçer, çağrının tamamını kapsar
    hex_v3_yerel(f)
        {                      // iç bölge = n'nin ömrü: bu çağrının stack çerçevesi
            let n: i8 = 1;
            f(&n)              // &n yalnızca iç bölgede geçerli
        }                      // n burada düşürülür
}                              // 'a en erken burada biter
```

`hex_v3_yerel`'de `f`, dış bölge kadar yaşayan bir referans ister; `&n` iç bölgenin dışına çıkamaz. Çağıran hangi `'a`'yı seçerse seçsin, o ömür çağrıyı baştan sona kapsar; çağrı başladığında henüz var olmayan bir yerel değişken böyle bir ömrü dolduramaz. `hex_v2_yerel`'de ise `F` "her ömre uyarım" sözü vermiştir; derleyici, çağrının yapıldığı yerde `'a` yerine iç bölgeyi koyar. Kısacası `for<'a>`, ömrü dışarıdan beklemeyi bırakıp `F`'nin *her* ömre uymasını şart koşar.[^nomicon]

### Fonksiyonun kendi ürettiği veri

Bu fark, gerçek kodda fonksiyonun closure'a *kendi ürettiği* bir veriyi gösterdiği her yerde karşınıza çıkar. İki terzi düşünün: Biri "yalnızca şu an elinizde tuttuğunuz kumaşı dikerim" der, öteki "hangi kumaşı getirirseniz getirin dikerim" der. Kumaş atölyede dokunacaksa ikinci terzi gerekir; çünkü o kumaş siz gelmeden önce yoktu.

```rust
fn uygula<F: for<'a> Fn(&'a str) -> &'a str>(f: F) -> usize {
    let yerel = String::from(" a "); // bu fonksiyonun içinde doğdu
    f(&yerel).len() // f her ömre razı, bu kısa ömre de
}

fn satirlari_donustur<F>(ham: &str, donustur: F) -> Vec<String>
where
    F: for<'a> Fn(&'a str) -> &'a str,
{
    ham.lines()
        .map(|satir| {
            let buyuk = satir.to_uppercase(); // her turda yeni, kısa ömürlü bir String
            donustur(&buyuk).to_string() // closure her turda başka bir ömürle çağrılır
        })
        .collect()
}

fn main() {
    println!("{}", uygula(|s| s.trim()));
    println!("{:?}", satirlari_donustur("  ada  \n grace ", |s| s.trim()));
}
```

Çıktı:

```text
1
["ADA", "GRACE"]
```

`uygula`'nın imzası `fn uygula<'a, F: Fn(&'a str) -> &'a str>(f: F) -> usize` olsaydı, `hex_v3_yerel`'deki hatanın aynısı çıkardı: `` `yerel` does not live long enough `` (E0597). `satirlari_donustur`'da bu daha da belirgindir: `buyuk` her turda yeniden doğar, dolayısıyla closure her turda *başka* bir ömürle çağrılır. Tek bir `'a` bu işi göremez.

### `Fn(&str)` yazımı ve `fn` işaretçileri

Bu kadar sık gereken bir şeyi her seferinde yazmanız gerekmez. `Fn` ailesinin parantezli yazımında ve `fn` işaretçisi türlerinde yazılmayan bir ömür, derleyici tarafından `for<'a>` ile bağlanır.[^elision] Yani `F: Fn(&str) -> &str` ile `F: for<'a> Fn(&'a str) -> &'a str` birebir aynı sınırdır; birini taşıyan bir `F`, ötekini isteyen fonksiyona olduğu gibi devredilebilir:

```rust
fn kisa_yazim<F: Fn(&str) -> &str>(f: F) -> usize {
    acik_yazim(f)
}

fn acik_yazim<F: for<'a> Fn(&'a str) -> &'a str>(f: F) -> usize {
    let yerel = String::from(" a ");
    f(&yerel).len()
}

fn main() {
    println!("{}", kisa_yazim(|s| s.trim()));
}
```

Çıktı:

```text
1
```

`fn` işaretçilerinde de aynı eşitlik geçerlidir; iki yazım birbirinin yerine atanabilir:

```rust
# fn satirlari_donustur<F>(ham: &str, donustur: F) -> Vec<String>
# where
#     F: for<'a> Fn(&'a str) -> &'a str,
# {
#     ham.lines()
#         .map(|satir| {
#             let buyuk = satir.to_uppercase();
#             donustur(&buyuk).to_string()
#         })
#         .collect()
# }
type Kirpici = for<'a> fn(&'a str) -> &'a str;

fn main() {
    let isaretci: Kirpici = str::trim_start;
    let ayni: fn(&str) -> &str = isaretci;

    println!("{:?}", satirlari_donustur("  girintili", isaretci));
    println!("{:?}", ayni("  kar"));
}
```

Çıktı:

```text
["GIRINTILI"]
"kar"
```

### Sık görülen tuzak: `let` ile tanımlanan closure

Fonksiyon imzalarında ve `Fn` sınırlarında işleyen ömür çıkarımı, closure *ifadelerinin* dönüş türü için işlemez. Bu yüzden zararsız görünen şu satır derlenmez:

```rust,compile_fail
fn main() {
    let kirp = |s: &str| s.trim();
    println!("{}", kirp("  a  "));
}
```

Derleyicinin yanıtı:

```text
error: lifetime may not live long enough
  |
2 |     let kirp = |s: &str| s.trim();
  |                    -   - ^^^^^^^^ returning this value requires that `'1` must outlive `'2`
  |                    |   |
  |                    |   return type of closure is &'2 str
  |                    let's call the lifetime of this reference `'1`
```

Bir formda "çıktı hangi girdiye bağlı?" hanesini boş bıraktığınızı düşünün. Fonksiyonlarda bu haneyi sabit bir kural doldurur. `let` ile yazılan closure'da ise derleyici girdiye `'1`, çıktıya `'2` olmak üzere iki ayrı ömür yazar ve aralarında bağ kurmaz. Dönüş türünü `-> &str` diye yazmak da bir şey değiştirmez.

Closure, HRTB bekleyen bir parametreye doğrudan yazılırsa sorun çıkmaz; `uygula(|s| s.trim())` çağrısında derleyici beklenen imzayı sınırdan okur.[^closure-imza] Önce değişkene bağlamak istiyorsanız imzayı bir "kimlik fonksiyonu" ile dayatabilirsiniz. `imza_dayat` hazır doldurulmuş bir form şablonudur: Aldığını olduğu gibi geri verir, tek işi türü öğretmektir.

```rust
# fn satirlari_donustur<F>(ham: &str, donustur: F) -> Vec<String>
# where
#     F: for<'a> Fn(&'a str) -> &'a str,
# {
#     ham.lines()
#         .map(|satir| {
#             let buyuk = satir.to_uppercase();
#             donustur(&buyuk).to_string()
#         })
#         .collect()
# }
fn imza_dayat<F: for<'a> Fn(&'a str) -> &'a str>(f: F) -> F {
    f
}

fn main() {
    let kirp = imza_dayat(|s| s.trim());
    println!("{:?}", kirp("  a  "));

    let ilk_kelime = imza_dayat(|s| s.split_whitespace().next().unwrap_or(""));
    println!("{:?}", satirlari_donustur("merhaba dünya", ilk_kelime));

    // Ortamından bir şey yakalamayan closure, fn işaretçisine de dönüşebilir.
    let kirp_isaretcisi: fn(&str) -> &str = |s| s.trim();
    println!("{:?}", kirp_isaretcisi("  b  "));
}
```

Çıktı:

```text
"a"
["MERHABA"]
"b"
```

Closure'ın başına `for<'a>` yazmayı sağlayan bir sözdizimi de önerilmiştir, ama henüz kararlı değildir. Aşağıdaki satır yalnızca nightly derleyicide, `closure_lifetime_binder` özelliği açıkken derlenir:[^binder]

```rust,ignore
#![feature(closure_lifetime_binder)]

let kirp = for<'a> |s: &'a str| -> &'a str { s.trim() };
```

Aynı tuzağın bir akrabası, parametresinin türü hiç yazılmamış closure'dır. Closure kendi başına tanımlanınca derleyici parametreye *belirli* bir ömür yazar; closure artık "her ömür" sözünü veremez:

```rust,compile_fail
fn uzunluk_topla<F: Fn(&str) -> usize>(f: F) -> usize {
    let yerel = String::from("kar");
    f(&yerel) + f("yağmur")
}

fn main() {
    let uz = |s| str::len(s);
    println!("{}", uzunluk_topla(uz));
}
```

Derleyicinin yanıtı:

```text
error: implementation of `Fn` is not general enough
  |
8 |     println!("{}", uzunluk_topla(uz));
  |                    ^^^^^^^^^^^^^^^^^ implementation of `Fn` is not general enough
  |
  = note: closure with signature `fn(&'2 str) -> usize` must implement `Fn<(&'1 str,)>`, for any lifetime `'1`...
  = note: ...but it actually implements `Fn<(&'2 str,)>`, for some specific lifetime `'2`
help: consider adding an explicit type annotation to the closure's argument
  |
7 |     let uz = |s: &str| str::len(s);
  |                ++++++
```

Çözüm mesajdadır: Parametreye tür yazmak (`|s: &str| str::len(s)`) ya da closure'ı doğrudan çağrının içinde vermek yeter. Dönüş değeri referans olmadığı için burada `imza_dayat` gerekmez.

### `Fn` dışındaki trait'lerde HRTB: sahipli çözümleme

`for<'a>` yalnızca closure'lar için değildir. En bilinen örnek serde kütüphanesidir. Oradaki `Deserialize<'de>` trait'i bir ömür taşır; çünkü `&'de str` gibi bazı türler girdi tamponunun *içine* bakarak, sıfır kopyayla çözülür. Peki girdiyi fonksiyonun içinde okuyup yalnızca sonucu döndürmek isterseniz? Tampon fonksiyonla birlikte düşecektir, dönen değer ona bakamaz. serde bunun için `DeserializeOwned` adıyla `for<'de> Deserialize<'de>` sınırını kullanır: her girdi ömrü için çözülebilen, dolayısıyla girdiden ödünç almayan türler.[^serde]

Duvardaki bir resmi iki yolla "alabilirsiniz": Pencereden bakarsınız ya da fotoğrafını çekersiniz. Duvar yıkılırsa pencereden görünen de gider; `&str` böyledir. Fotoğraf ise sizde kalır; `u32` ve `String` böyledir. Aynı tasarımın küçük bir modeli:

```rust
trait Coz<'g>: Sized {
    fn coz(girdi: &'g str) -> Option<Self>;
}

// Ödünç alan çözümleme: yalnızca o girdi ömrü için uygulanır.
impl<'g> Coz<'g> for &'g str {
    fn coz(girdi: &'g str) -> Option<Self> {
        Some(girdi.trim())
    }
}

// Sahipli çözümleme: sonuç girdiden bağımsız, her 'g için uygulanır.
impl<'g> Coz<'g> for u32 {
    fn coz(girdi: &'g str) -> Option<Self> {
        girdi.trim().parse().ok()
    }
}

impl<'g> Coz<'g> for String {
    fn coz(girdi: &'g str) -> Option<Self> {
        Some(girdi.trim().to_string())
    }
}

// Girdi burada, fonksiyonun içinde üretiliyor.
fn kaynaktan_coz<T>(kaynak: impl FnOnce() -> String) -> Option<T>
where
    T: for<'g> Coz<'g>,
{
    let tampon = kaynak(); // fonksiyon bitince düşecek
    T::coz(&tampon)
}

// Tampon çağırandaysa HRTB gerekmez; sıradan ömür parametresi yeter.
fn tampondan_coz<'g, T: Coz<'g>>(tampon: &'g str) -> Option<T> {
    T::coz(tampon)
}

fn main() {
    let sayi: Option<u32> = kaynaktan_coz(|| String::from(" 42 "));
    let metin: Option<String> = kaynaktan_coz(|| String::from("  sahipli  "));
    let bozuk: Option<u32> = kaynaktan_coz(|| String::from("yedi"));
    println!("kaynaktan: {sayi:?}, {metin:?}, {bozuk:?}");

    let tampon = String::from("  ödünç  ");
    let odunc: Option<&str> = tampondan_coz(&tampon); // tampon bizde: &str de olur
    println!("tampondan: {odunc:?}");
}
```

Çıktı:

```text
kaynaktan: Some(42), Some("sahipli"), None
tampondan: Some("ödünç")
```

`kaynaktan_coz`'dan bir `&str` istemek, silinecek tampona bakmak demektir:

```rust,compile_fail
# trait Coz<'g>: Sized {
#     fn coz(girdi: &'g str) -> Option<Self>;
# }
# impl<'g> Coz<'g> for &'g str {
#     fn coz(girdi: &'g str) -> Option<Self> {
#         Some(girdi.trim())
#     }
# }
# fn kaynaktan_coz<T>(kaynak: impl FnOnce() -> String) -> Option<T>
# where
#     T: for<'g> Coz<'g>,
# {
#     let tampon = kaynak();
#     T::coz(&tampon)
# }
fn main() {
    let odunc: Option<&str> = kaynaktan_coz(|| String::from("x"));
    println!("{odunc:?}");
}
```

Derleyicinin yanıtı:

```text
error: implementation of `Coz` is not general enough
   |
   = note: `Coz<'0>` would have to be implemented for the type `&str`, for any lifetime `'0`...
   = note: ...but `Coz<'1>` is actually implemented for the type `&'1 str`, for some specific lifetime `'1`
```

Bu mesajı okumayı öğrenmekte yarar var. İlk not *isteneni* söyler: herhangi bir `'0` ömrü için `Coz<'0>`. İkinci not *elde olanı* söyler: `&'1 str` türü için yalnızca `Coz<'1>`, yani belirli bir ömür. "Her ömür için" istenmiş, "belirli bir ömür için" verilmiştir. Hatanın kodu yoktur ve hemen her zaman bir `for<'a>` sınırına işaret eder.

## GAT: ödünç veren iterator

Bir dilim üzerinde birbiriyle *örtüşen*, değiştirilebilir pencereler gezmek istediğinizi düşünün: önce `[0, 1]`, sonra `[1, 2]`, sonra `[2, 3]`. Standart kütüphanede paylaşımlı pencereler için `windows` vardır, ama `windows_mut` yoktur.[^windows] `Iterator` ile yazmayı deneyelim:

```rust,compile_fail
struct PencereMut<'v, T> {
    veri: &'v mut [T],
    bas: usize,
    boy: usize,
}

impl<'v, T> Iterator for PencereMut<'v, T> {
    type Item = &'v mut [T];

    fn next(&mut self) -> Option<&'v mut [T]> {
        let son = self.bas.checked_add(self.boy)?;
        let pencere = self.veri.get_mut(self.bas..son)?;
        self.bas += 1;
        Some(pencere)
    }
}
```

Derleyicinin yanıtı:

```text
error: lifetime may not live long enough
   |
 7 | impl<'v, T> Iterator for PencereMut<'v, T> {
   |      -- lifetime `'v` defined here
...
10 |     fn next(&mut self) -> Option<&'v mut [T]> {
   |             - let's call the lifetime of this reference `'1`
...
14 |         Some(pencere)
   |         ^^^^^^^^^^^^^ method was supposed to return data with lifetime `'v` but it is returning data with lifetime `'1`
```

Bu hata bir eksiklik değil, bir korumadır. `Iterator::next`'in imzası `fn next(&mut self) -> Option<Self::Item>` biçimindedir ve `Item`, `&mut self` ödüncünün ömrüne bağlı *değildir*.[^iterator-next] Bu yüzden çağıran, önceki öğeyi elinde tutarken sonrakini isteyebilir; `collect()` tam olarak bunu yapar. Pencereler örtüştüğüne göre bu, aynı elemana iki canlı `&mut` demek olurdu. "Öğe, iterator'ın kendisinden ödünçtür; sonrakini istemeden öncekini bırakın" sözleşmesini `Iterator` trait'i ifade edemez.

### İlişkili türe ömür parametresi vermek

Kütüphanede tek bir büyüteç olduğunu ve ödünç fişiyle verildiğini düşünün. Sıradan `Iterator`'ın fişinde "ne zaman geri gelecek" hanesi yoktur; eskisini geri vermeden yenisini isteyebilirsiniz. Büyüteç tekse bu kabul edilemez. GAT, fişe her seferinde yeniden doldurulan bir hane ekler: `type Oge<'a>`. Her `sonraki()` çağrısında o haneye, o çağrının ödünç ömrü yazılır.[^gat]

```rust
trait OduncVerenIterator {
    type Oge<'a>
    where
        Self: 'a;

    // Öğe, bu çağrının &mut self ödüncü kadar yaşar.
    fn sonraki<'a>(&'a mut self) -> Option<Self::Oge<'a>>;
}

struct PencereMut<'v, T> {
    veri: &'v mut [T],
    bas: usize,
    boy: usize,
}

impl<'v, T> PencereMut<'v, T> {
    fn yeni(veri: &'v mut [T], boy: usize) -> Self {
        PencereMut { veri, bas: 0, boy }
    }
}

impl<'v, T> OduncVerenIterator for PencereMut<'v, T> {
    type Oge<'a>
        = &'a mut [T]
    where
        Self: 'a;

    fn sonraki<'a>(&'a mut self) -> Option<Self::Oge<'a>> {
        let son = self.bas.checked_add(self.boy)?;
        let pencere = self.veri.get_mut(self.bas..son)?; // sığmıyorsa None: bitti
        self.bas += 1;
        Some(pencere)
    }
}

fn main() {
    // Her pencere [önceki, şimdiki]; şimdiki += önceki.
    let mut veri = [1, 2, 3, 4, 5];
    let mut pencereler = PencereMut::yeni(&mut veri, 2);
    while let Some(p) = pencereler.sonraki() {
        p[1] += p[0];
    } // p her turun sonunda düşer; bir sonraki çağrı serbesttir
    println!("birikimli toplam = {veri:?}");
}
```

Çıktı:

```text
birikimli toplam = [1, 3, 6, 10, 15]
```

`Iterator` sürümünden tek farkı, dönüş türünün `'v`'ye değil `&'a mut self` ödüncüne bağlanmasıdır. Gövde aynı kaldı ve bu kez derlendi. `checked_add`, `bas + boy` toplamının taşmasını önler: `boy` olarak `usize::MAX` verilse bile program paniklemez, gezinti hemen `None` ile biter.

Öğe elinizdeyken iterator kilitlidir. İki öğeyi aynı anda tutmak isteyen kod derlenmez; istediğimiz koruma budur:

```rust,compile_fail,E0499
# trait OduncVerenIterator {
#     type Oge<'a> where Self: 'a;
#     fn sonraki<'a>(&'a mut self) -> Option<Self::Oge<'a>>;
# }
# struct PencereMut<'v, T> { veri: &'v mut [T], bas: usize, boy: usize }
# impl<'v, T> PencereMut<'v, T> {
#     fn yeni(veri: &'v mut [T], boy: usize) -> Self { PencereMut { veri, bas: 0, boy } }
# }
# impl<'v, T> OduncVerenIterator for PencereMut<'v, T> {
#     type Oge<'a> = &'a mut [T] where Self: 'a;
#     fn sonraki<'a>(&'a mut self) -> Option<Self::Oge<'a>> {
#         let son = self.bas.checked_add(self.boy)?;
#         let pencere = self.veri.get_mut(self.bas..son)?;
#         self.bas += 1;
#         Some(pencere)
#     }
# }
fn kullan<T>(_a: Option<&mut [T]>, _b: Option<&mut [T]>) {}

fn main() {
    let mut veri = [1, 2, 3];
    let mut pencereler = PencereMut::yeni(&mut veri, 2);
    let a = pencereler.sonraki();
    let b = pencereler.sonraki(); // a hâlâ elimizde
    kullan(a, b);
}
```

Derleyicinin yanıtı:

```text
error[E0499]: cannot borrow `pencereler` as mutable more than once at a time
```

### `where Self: 'a` neden gerekli?

İlişkili türdeki `where Self: 'a` koşulu "`Oge<'a>`, `Self`'ten ödünç aldığı için `Self` en az `'a` kadar yaşamalıdır" demektir. Yazmazsanız trait derlenmez:

```rust,compile_fail
trait OduncVerenIterator {
    type Oge<'a>;

    fn sonraki<'a>(&'a mut self) -> Option<Self::Oge<'a>>;
}
```

Derleyicinin yanıtı:

```text
error: missing required bound on `Oge`
  |
2 |     type Oge<'a>;
  |     ^^^^^^^^^^^^-
  |                 |
  |                 help: add the required where clause: `where Self: 'a`
  |
  = note: this bound is currently required to ensure that impls have maximum flexibility
  = note: we are soliciting feedback, see issue #87479 <https://github.com/rust-lang/rust/issues/87479> for more information
```

Derleyici koşulu kendisi de çıkarabilirdi: `&'a mut self` parametresi zaten `Self: 'a` olduğunu gösterir. Yine de elle yazdırır; çünkü bu kural, `impl` yazanlara en geniş serbestliği bırakmak için konmuştur ve ileride gevşetilebilir.[^gat-where]

### Jenerik kodda kullanım

Trait'i sınır olarak alan kod, hangi iterator olduğunu bilmeden öğeleri gezebilir. Öğe her turda düştüğü için `while` döngüsü yeter:

```rust
# trait OduncVerenIterator {
#     type Oge<'a> where Self: 'a;
#     fn sonraki<'a>(&'a mut self) -> Option<Self::Oge<'a>>;
# }
# struct PencereMut<'v, T> { veri: &'v mut [T], bas: usize, boy: usize }
# impl<'v, T> PencereMut<'v, T> {
#     fn yeni(veri: &'v mut [T], boy: usize) -> Self { PencereMut { veri, bas: 0, boy } }
# }
# impl<'v, T> OduncVerenIterator for PencereMut<'v, T> {
#     type Oge<'a> = &'a mut [T] where Self: 'a;
#     fn sonraki<'a>(&'a mut self) -> Option<Self::Oge<'a>> {
#         let son = self.bas.checked_add(self.boy)?;
#         let pencere = self.veri.get_mut(self.bas..son)?;
#         self.bas += 1;
#         Some(pencere)
#     }
# }
fn say<I: OduncVerenIterator>(mut it: I) -> usize {
    let mut n = 0;
    while it.sonraki().is_some() {
        n += 1;
    }
    n
}

fn main() {
    let mut harfler = ['a', 'b', 'c', 'd'];
    println!("3'lü pencere: {}", say(PencereMut::yeni(&mut harfler, 3)));
    println!("5'li pencere: {}", say(PencereMut::yeni(&mut harfler, 5)));
    println!("dev pencere: {}", say(PencereMut::yeni(&mut harfler, usize::MAX)));
}
```

Çıktı:

```text
3'lü pencere: 2
5'li pencere: 0
dev pencere: 0
```

### Sınırlar

GAT güçlüdür, ama ödünç veren iterator'ın bilmeniz gereken üç sınırı vardır.

**`Iterator` altyapısını kullanamaz.** `for` döngüsü, `map` zincirleri ve `collect` hep `Iterator` (ya da `IntoIterator`) ister. `for p in pencereler` yazarsanız E0277 hatası alırsınız: `` `PencereMut<'_, {integer}>` is not an iterator ``. Standart kütüphanede ödünç veren bir iterator trait'i henüz yoktur.

**Gerekmediğinde yük olur.** Öğeler örtüşmüyorsa sıradan `Iterator` yeter; örneğin `chunks_mut` ayrık parçalar verir. Örtüşen pencereler için de bazen daha yalın bir yol vardır: Belgeler, `windows_mut` yerine dilimi `Cell` dilimi olarak görüp `windows` kullanmayı önerir.[^windows] `Cell` paylaşımlı referans üzerinden yazmaya izin verdiği için pencerelerin örtüşmesi sorun olmaz:

```rust
use std::cell::Cell;

fn main() {
    let mut veri = [1, 2, 3, 4, 5];
    let hucreler = Cell::from_mut(&mut veri[..]).as_slice_of_cells();
    for p in hucreler.windows(2) {
        p[1].set(p[0].get() + p[1].get());
    }
    println!("{veri:?}");
}
```

Çıktı:

```text
[1, 3, 6, 10, 15]
```

GAT'a, öğe gerçekten iterator'dan ödünç olduğunda başvurun.

**Trait nesnesi yapılamaz.** Jenerik ilişkili türü olan bir trait, yazıldığı bu hâliyle dyn uyumlu değildir:[^dyn-uyumlu]

```rust,compile_fail,E0038
trait OduncVerenIterator {
    type Oge<'a>
    where
        Self: 'a;

    fn sonraki<'a>(&'a mut self) -> Option<Self::Oge<'a>>;
}

fn f(_: &dyn OduncVerenIterator) {}
```

Derleyicinin yanıtı:

```text
error[E0038]: the trait `OduncVerenIterator` is not dyn compatible
  |
9 | fn f(_: &dyn OduncVerenIterator) {}
  |              ^^^^^^^^^^^^^^^^^^ `OduncVerenIterator` is not dyn compatible
  |
note: for a trait to be dyn compatible it needs to allow building a vtable
  |
1 | trait OduncVerenIterator {
  |       ------------------ this trait is not dyn compatible...
2 |     type Oge<'a>
  |          ^^^ ...because it contains generic associated type `Oge`
  = help: consider moving `Oge` to another trait
```

Reference bu kuralı koşulsuz yazar. 1.99.0 derleyicisi ise `where Self: Sized` koşulu taşıyan üyeleri denetimden muaf tutar; koşulu hem ilişkili türe hem onu kullanan metoda eklerseniz trait `dyn` ile kullanılabilir. Ama o zaman `Sized` olmayan trait nesnesi üzerinden `sonraki`'yi çağıramazsınız; yani koşul, GAT'ı trait nesnesinin dışında bırakmaktan ibarettir.

## GAT: tür ailesi

GAT yalnızca ömür almaz, *tür* parametresi de alabilir: `type Isaretci<T>`. Böylece `Rc<_>` ya da `Arc<_>` gibi içi henüz doldurulmamış bir tür kurucusunu, yani bir tür *ailesini* soyutlayabilirsiniz.[^gat]

Sürüm geçmişi tutan bir belge düşünün. Aynı metin birçok anlık görüntüde durduğu için sürümler sayaçlı işaretçiyle paylaşılsın. Tek thread'de `Rc` yeter ve daha ucuzdur; birden çok thread'de `Arc` gerekir. İki ayrı `Belge` yazmak yerine işaretçi ailesini parametre yaparız. Taşınırken koli türünü baştan seçersiniz: karton (hafif, ucuz) ya da çelik sandık (ağır, dayanıklı). İçine ne koyacağınız sonra belli olur. `type Isaretci<T>` böyle bir koli türüdür, `T` ise içine konacak şeydir.

```rust
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

trait IsaretciAilesi {
    type Isaretci<T>: Deref<Target = T> + Clone;
    fn yeni<T>(deger: T) -> Self::Isaretci<T>;
}

struct RcAilesi;
struct ArcAilesi;

impl IsaretciAilesi for RcAilesi {
    type Isaretci<T> = Rc<T>;
    fn yeni<T>(deger: T) -> Rc<T> {
        Rc::new(deger)
    }
}

impl IsaretciAilesi for ArcAilesi {
    type Isaretci<T> = Arc<T>;
    fn yeni<T>(deger: T) -> Arc<T> {
        Arc::new(deger)
    }
}

struct Belge<A: IsaretciAilesi> {
    surumler: Vec<A::Isaretci<String>>,
}

impl<A: IsaretciAilesi> Belge<A> {
    fn yeni(ilk: &str) -> Self {
        Belge { surumler: vec![A::yeni(ilk.to_string())] }
    }

    fn duzenle(&mut self, yeni_metin: &str) {
        self.surumler.push(A::yeni(yeni_metin.to_string()));
    }

    // Metni kopyalamaz: yalnızca işaretçi klonlanır, sayaç bir artar.
    fn guncel(&self) -> A::Isaretci<String> {
        self.surumler.last().expect("belge en az bir sürümle kurulur (bkz. yeni)").clone()
    }

    fn surum_sayisi(&self) -> usize {
        self.surumler.len()
    }
}

fn main() {
    let mut yerel: Belge<RcAilesi> = Belge::yeni("taslak");
    yerel.duzenle("taslak v2");
    println!("Rc belgesi: {} ({} sürüm)", *yerel.guncel(), yerel.surum_sayisi());

    let onceki = yerel.guncel(); // belge ve biz: aynı metnin iki sahibi
    println!("sahip sayısı: {}", Rc::strong_count(&onceki));
}
```

Çıktı:

```text
Rc belgesi: taslak v2 (2 sürüm)
sahip sayısı: 2
```

`Belge<RcAilesi>` içeride `Vec<Rc<String>>`, `Belge<ArcAilesi>` ise `Vec<Arc<String>>` tutar. Kod bir kez yazıldı, koli türü kullanırken seçildi. Bunun güzel bir yan sonucu vardır: Derleyici `Send` gibi otomatik trait'leri alan türünden kendisi çıkardığı için,[^otomatik] aynı kaynak kod seçilen aileye göre thread'ler arasında taşınabilir ya da taşınamaz olur. `ArcAilesi` ile güncel metni başka bir thread'e verebiliriz:

```rust
# use std::ops::Deref;
# use std::sync::Arc;
# trait IsaretciAilesi {
#     type Isaretci<T>: Deref<Target = T> + Clone;
#     fn yeni<T>(deger: T) -> Self::Isaretci<T>;
# }
# struct ArcAilesi;
# impl IsaretciAilesi for ArcAilesi {
#     type Isaretci<T> = Arc<T>;
#     fn yeni<T>(deger: T) -> Arc<T> { Arc::new(deger) }
# }
# struct Belge<A: IsaretciAilesi> { surumler: Vec<A::Isaretci<String>> }
# impl<A: IsaretciAilesi> Belge<A> {
#     fn yeni(ilk: &str) -> Self { Belge { surumler: vec![A::yeni(ilk.to_string())] } }
#     fn duzenle(&mut self, yeni_metin: &str) {
#         self.surumler.push(A::yeni(yeni_metin.to_string()));
#     }
#     fn guncel(&self) -> A::Isaretci<String> {
#         self.surumler.last().expect("belge en az bir sürümle kurulur").clone()
#     }
# }
// Derleme zamanı sınaması: çağrı derleniyorsa T: Send.
fn send_ister<T: Send>(_: &T) {}

fn main() {
    let mut paylasilan: Belge<ArcAilesi> = Belge::yeni("rapor");
    paylasilan.duzenle("rapor (son)");
    send_ister(&paylasilan);

    let guncel = paylasilan.guncel();
    let uzunluk = std::thread::spawn(move || guncel.chars().count())
        .join()
        .expect("okuyan thread paniklemedi");
    println!("başka thread'de okunan uzunluk = {uzunluk}");
}
```

Çıktı:

```text
başka thread'de okunan uzunluk = 11
```

Aynı sınama `RcAilesi` ile derlenmez. `send_ister`, [25. bölümdeki](25-send-sync.md) derleme zamanı sınamasıdır:

```rust,compile_fail,E0277
# use std::ops::Deref;
# use std::rc::Rc;
# trait IsaretciAilesi {
#     type Isaretci<T>: Deref<Target = T> + Clone;
#     fn yeni<T>(deger: T) -> Self::Isaretci<T>;
# }
# struct RcAilesi;
# impl IsaretciAilesi for RcAilesi {
#     type Isaretci<T> = Rc<T>;
#     fn yeni<T>(deger: T) -> Rc<T> { Rc::new(deger) }
# }
# struct Belge<A: IsaretciAilesi> { surumler: Vec<A::Isaretci<String>> }
# impl<A: IsaretciAilesi> Belge<A> {
#     fn yeni(ilk: &str) -> Self { Belge { surumler: vec![A::yeni(ilk.to_string())] } }
# }
fn send_ister<T: Send>(_: &T) {}

fn main() {
    let yerel: Belge<RcAilesi> = Belge::yeni("taslak");
    send_ister(&yerel);
}
```

Derleyicinin yanıtı:

```text
error[E0277]: `Rc<String>` cannot be sent between threads safely
    |
    = help: within `Belge<RcAilesi>`, the trait `Send` is not implemented for `Rc<String>`
```

### GAT olmadan yazılabilir miydi?

Evet, `T`'yi ilişkili türden alıp trait'in parametresi yaparak: `trait Aile<T> { type Isaretci; }`. Bedeli, kullanılan her tür için ayrı bir sınır yazmaktır:

```rust
use std::ops::Deref;
use std::rc::Rc;

trait Aile<T> {
    type Isaretci: Deref<Target = T> + Clone;
    fn yeni(deger: T) -> Self::Isaretci;
}

struct RcAilesi;

impl<T> Aile<T> for RcAilesi {
    type Isaretci = Rc<T>;
    fn yeni(deger: T) -> Rc<T> {
        Rc::new(deger)
    }
}

// İki tür kullanıldığı için iki sınır gerekiyor.
struct Kayit<A: Aile<String> + Aile<u32>> {
    ad: <A as Aile<String>>::Isaretci,
    yas: <A as Aile<u32>>::Isaretci,
}

fn main() {
    let kayit: Kayit<RcAilesi> = Kayit {
        ad: RcAilesi::yeni(String::from("ada")),
        yas: RcAilesi::yeni(36),
    };
    println!("{} {}", *kayit.ad, *kayit.yas);
}
```

Çıktı:

```text
ada 36
```

GAT, "her `T` için" sözünü tek bir sınırla verir: `A: IsaretciAilesi`.

## Hata kodu başvurusu

Arabanın gösterge panelinde yanan her lambanın bir anlamı vardır; kılavuzdaki tablo "bu lamba yanarsa şuraya bakın" der. Aşağıdaki tablo ömür hataları için aynı işi görür. Ezberlemeniz gerekmez; hata aldığınızda dönüp bakarsınız.

| Kod | Ne zaman görülür? | Ne yapmalı? |
| --- | --- | --- |
| `E0597` | Bir referans, baktığı değerden uzun yaşamaya çalıştığında: `` `x` does not live long enough `` | Önce sorun: Sonucun bu değere bağlı olması *gerekiyor mu*? Gerekmiyorsa imzadaki etiketler fazla sıkıdır (iki ayrı ömür, `for<'a>`); gerekiyorsa sahipli değere geçin. |
| `E0499`, `E0502` | Aynı değere çakışan ödünçler alındığında | Dönüş türü `&mut self`'e bağlandıysa değer kilitli kalır. Struct'ın kendi `'a`'sını döndürebilir misiniz ([9. bölümün derinleşmesi](../intermediate/09-generics-traits-lifetimes-deep-dive.md))? `&'a mut self` yazdıysanız silin. |
| `E0521` | Fonksiyona gelen kısa ömürlü bir referans, `'static` isteyen bir yere (`thread::spawn`, `static` bir değişken, `Box<dyn Trait>`) kaçtığında | Sahipliği verin ya da `thread::scope` kullanın. |
| `E0309`, `E0310` | Bir `T`, `'a` (E0309) ya da `'static` (E0310) ömürlü bir yere konduğunda | `T: 'a` ya da `T: 'static` sınırını ekleyin. |
| `E0261` | Bildirilmemiş bir ömür adı kullanıldığında | `fn f<'a>`, `impl<'a>` ya da `struct S<'a>` ile bildirin. |
| `E0621` | İmzada ömrü yazılmamış bir parametreden, adlandırılmış bir ömür istendiğinde | Parametrenin ömrünü de yazın: `metin: &'a str`. |
| `E0038` | Jenerik ilişkili türü olan trait, `dyn` ile kullanıldığında | Jenerik parametre (`impl Trait`) kullanın ya da GAT'ı ayrı bir trait'e taşıyın. |
| `E0277` | Verilen tür istenen trait'i uygulamadığında | Ödünç veren iterator'ı `while let` ile gezin; thread'e taşınacak veride `Arc` kullanın. |
| `E0794` | Geç bağlanan ömre elle değer verildiğinde (`f::<'static>`) | Ömrü yazmayın; derleyici her çağrıda seçer. |
| `E0658` | Kararlı olmayan bir özellik (`for<T>`) kullanıldığında | Jenerik metotlu bir trait tanımlayın. |
| kodsuz: `lifetime may not live long enough` | Gövde, imzanın söz verdiğinden fazlasını istediğinde | `help` satırı çoğunlukla eksik ilişkiyi söyler. Notta *invariant* geçiyorsa sorun `&mut` ya da `Cell` ardındaki bir ömürdür. |
| kodsuz: `` implementation of `X` is not general enough `` | Bir `for<'a>` sınırına, yalnızca *belirli* bir ömür için çalışan tür ya da closure verildiğinde | Closure'a parametre türü yazın, sahipli tür kullanın ya da sınırı gevşetin. |
| kodsuz: `` missing required bound on `Oge` `` | GAT bildiriminde `where Self: 'a` eksik olduğunda | Mesajdaki koşulu ekleyin. |

Tablodaki üç durum, bölümün başka bir yerinde örneklenmez; kısa birer örnek yeter.

### `&'a mut self`: kendini kilitleyen struct

`&mut self` yazılacak yere struct'ın kendi ömrünü taşıyan `&'a mut self` yazmak, sık yapılan bir hatadır:

```rust,compile_fail,E0499
struct Parcalar<'a> {
    liste: Vec<&'a str>,
}

impl<'a> Parcalar<'a> {
    fn ekle(&'a mut self, parca: &'a str) {
        self.liste.push(parca);
    }
}

fn main() {
    let mut parcalar = Parcalar { liste: Vec::new() };
    parcalar.ekle("bir");
    parcalar.ekle("iki");
}
```

Derleyicinin yanıtı:

```text
error[E0499]: cannot borrow `parcalar` as mutable more than once at a time
   |
13 |     parcalar.ekle("bir");
   |     -------- first mutable borrow occurs here
14 |     parcalar.ekle("iki");
   |     ^^^^^^^^
   |     |
   |     second mutable borrow occurs here
   |     first borrow later used here
```

`&'a mut Parcalar<'a>`, değiştirilebilir ödüncün struct'ın içindeki referanslar kadar sürmesini ister. Değiştirilebilir referansın ardındaki tür invaryant olduğu için bu süre kısaltılamaz; ilk çağrı, değeri kullanıldığı son ana kadar kilitler. Çözüm `'a`'yı silmektir: `fn ekle(&mut self, parca: &'a str)`.[^varyans]

### E0521: ödünç verinin kaçması

```rust,compile_fail,E0521
use std::thread;

fn arkada_uzunluk(metin: &str) {
    thread::spawn(move || println!("{}", metin.len()));
}
```

Derleyicinin yanıtı:

```text
error[E0521]: borrowed data escapes outside of function
  |
3 | fn arkada_uzunluk(metin: &str) {
  |                   -----  - let's call the lifetime of this reference `'1`
  |                   |
  |                   `metin` is a reference that is only valid in the function body
4 |     thread::spawn(move || println!("{}", metin.len()));
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |     |
  |     `metin` escapes the function body here
  |     argument requires that `'1` must outlive `'static`
```

`thread::spawn`, başlattığı thread'in ne zaman biteceğini bilmediği için closure'dan `'static` ister;[^spawn] `metin` ise yalnızca çağrı boyunca ödünçtür. Ya sahipli veri verin (`String`) ya da thread'lerin kapsam sonunda mutlaka biteceğini bildiren `thread::scope`'u kullanın:

```rust
use std::thread;

fn arkada_uzunluk(metin: &str) -> usize {
    thread::scope(|kapsam| {
        kapsam
            .spawn(|| metin.len())
            .join()
            .expect("sayan thread paniklemedi")
    })
}

fn main() {
    println!("{}", arkada_uzunluk("kar yağar"));
}
```

Çıktı:

```text
10
```

### E0261: bildirilmemiş ömür

```rust,compile_fail,E0261
struct Satir {
    metin: &'a str,
}
```

Derleyicinin yanıtı:

```text
error[E0261]: use of undeclared lifetime name `'a`
  |
2 |     metin: &'a str,
  |             ^^ undeclared lifetime
  |
help: consider introducing lifetime `'a` here
  |
1 | struct Satir<'a> {
  |             ++++
```

`struct Satir<'a>` yazmak yeter. Fonksiyonlarda `fn f<'a>(..)`, uygulama bloklarında `impl<'a>` aynı işi görür.

## Perde arkası: Derleyici ne yapıyor?

> [!NOTE]
> Bu başlık ilk okuyuşta atlanabilir; yukarıdaki dört aracı (ayrı etiketler, `'a: 'b` ve `T: 'a` sınırları, `for<'a>` ve GAT) anlamış olmanız yeterlidir.

### Erken ve geç bağlanan ömürler

`fn gec<'a>(s: &'a str)` içindeki `'a` *geç bağlanır* (late-bound): Fonksiyon bir değer olarak kullanıldığında bile serbest kalır ve her çağrıda yeniden seçilir. Bu yüzden `gec`, tek bir değerken `for<'a> Fn(&'a str)` sınırını sağlar. Aslında fonksiyonlar "her ömür için" sözünü zaten kendiliğinden veriyordu; HRTB, bu özelliği closure'lar ve trait sınırları için yazılabilir kılar.[^erken-gec]

```rust
fn gec<'a>(s: &'a str) -> &'a str {
    s.trim()
}

fn iki_omurle<F: for<'a> Fn(&'a str) -> &'a str>(f: F) -> usize {
    let ilk = String::from(" kar ");
    let toplam = f(&ilk).len();
    let ikinci = String::from(" sis ");
    toplam + f(&ikinci).len() // aynı f, başka bir ömür
}

fn main() {
    println!("{}", iki_omurle(gec));
}
```

Çıktı:

```text
6
```

Geç bağlanan bir ömre elle değer veremezsiniz, çünkü o ömür fonksiyonu adlandırırken değil çağırırken seçilir:

```rust,compile_fail,E0794
fn gec<'a>(s: &'a str) -> &'a str {
    s.trim()
}

fn main() {
    let f = gec::<'static>;
    println!("{}", f(" a "));
}
```

Derleyicinin yanıtı:

```text
error[E0794]: cannot specify lifetime arguments explicitly if late bound lifetime parameters are present
  |
6 |     let f = gec::<'static>;
  |                   ^^^^^^^
  |
note: the late bound lifetime parameter is introduced here
  |
1 | fn gec<'a>(s: &'a str) -> &'a str {
  |        ^^
```

Ömür bir sınırda ya da `where` koşulunda geçiyorsa yahut hiçbir parametrenin türünde yer almıyorsa *erken bağlanır* (early-bound): Fonksiyon adlandırıldığı anda tek bir ömür seçilir.[^gec-kod] Aşağıdaki `'a: 'a` sınırı kendi başına hiçbir şey istemez, ama ömrü erken bağlanan hâle getirir ve `erken` artık `for<'a>` sınırını karşılayamaz:

```rust,compile_fail
fn erken<'a: 'a>(s: &'a str) -> &'a str {
    s.trim()
}

fn uygula<F: for<'a> Fn(&'a str) -> &'a str>(f: F) -> usize {
    let yerel = String::from(" a ");
    f(&yerel).len()
}

fn main() {
    println!("{}", uygula(erken));
}
```

Derleyicinin yanıtı:

```text
error: implementation of `Fn` is not general enough
   |
   = note: `fn(&'2 str) -> &'2 str {erken::<'2>}` must implement `Fn<(&'1 str,)>`, for any lifetime `'1`...
   = note: ...but it actually implements `Fn<(&'2 str,)>`, for some specific lifetime `'2`
```

### `for<...>` yalnızca ömürler içindir

"Her `T` türü için" diyen `for<T>` yazımı kararlı Rust'ta yoktur; `non_lifetime_binders` özelliği altında denenmektedir:[^turler]

```rust,compile_fail,E0658
fn ikisine_de<F>(f: F)
where
    F: for<T> Fn(T),
{
}
```

Derleyicinin yanıtı:

```text
error[E0658]: only lifetime parameters can be used in this context
  |
3 |     F: for<T> Fn(T),
  |            ^
  |
  = note: see issue #108185 <https://github.com/rust-lang/rust/issues/108185> for more information
```

Bu yüzden her `T` için çalışan bir closure parametresi yazamazsınız. Yerine jenerik metotlu bir trait tanımlanır:

```rust
use std::fmt::Debug;

trait HerTur {
    fn uygula<T: Debug>(&self, deger: T) -> String;
}

struct Bicimle;

impl HerTur for Bicimle {
    fn uygula<T: Debug>(&self, deger: T) -> String {
        format!("<{deger:?}>")
    }
}

fn ikisine_de<H: HerTur>(h: H) -> String {
    format!("{} {}", h.uygula(1), h.uygula("iki"))
}

fn main() {
    println!("{}", ikisine_de(Bicimle));
}
```

Çıktı:

```text
<1> <"iki">
```

### Derleyicinin adımları

**Closure imzası.** Closure bir parametreye doğrudan yazıldığında derleyici, parametrenin sınırından *beklenen* imzayı çıkarır; `for<'a>` de oradan gelir. Beklenen imza yoksa yalnızca yazdığınız türlerle yetinir ve yazılmayan ömürlere yeni bölge değişkenleri koyar.[^closure-imza] `imza_dayat`'ın işe yaraması bundandır: Closure'a bir beklenti kazandırır.

**`for<'a>`'nın gösterimi.** Derleyicide bağlı ömürler bir `Binder` içinde tutulur.[^binder-kod] Bir `for<'a>` sınırı denetlenirken `'a` yerine, hiçbir şeyle eşit olmayan bir *yer tutucu* (placeholder) konur; impl'in belirli ömrü bu yer tutucuyla eşleşemezse `is not general enough` mesajı üretilir.[^yer-tutucu]

**GAT'lar.** `<I as OduncVerenIterator>::Oge<'a>` yazımı, her `'a` için ayrı bir projeksiyondur. Trait çözümleyici `I` belli olunca bunu `impl` bloğundaki türe indirger; bu bölümde o tür `&'a mut [T]`'dir. `where Self: 'a` koşulunun gerekli olup olmadığına `check_gat_where_clauses` karar verir;[^gat-kod] koşulun ileride kendiliğinden varsayılıp varsayılmayacağı #87479 numaralı kayıtta tartışılmaktadır. GAT'lar 1.65'ten beri kararlıdır.[^gat-kararli]

**Sabit terfisi.** `hex_v3(&1)` örneğindeki terfi, MIR üzerinde çalışan `promote_consts` geçişinde yapılır: Derleyici ödünç alınan sabit ifadeyi ayrı bir sabite çıkarır ve referans ona bakar.[^terfi-kod]

**Silinme.** Ömür denetimi derleme zamanında biter; `for<'a>` ve GAT'ın ömür parametreleri çalışan programa denetim ya da maliyet eklemez. Kazancınız hız değil, yanlış kullanımın derlenmemesidir. "Ömürler tümüyle silinir" sözü ise biraz yuvarlaktır: Kod üretiminden önce *serbest* ömürler silinir, `for<'a>` ile *bağlı* ömürler ise yalnızca adsızlaştırılır; çünkü tür eşitliğinin parçasıdırlar.[^silinme] Bunu `TypeId` ile görebilirsiniz:[^typeid]

```rust
use std::any::TypeId;

fn main() {
    // Aynı türün iki yazımı
    println!("{}", TypeId::of::<fn(&str)>() == TypeId::of::<for<'a> fn(&'a str)>());
    // Her ömür için çalışan ile yalnızca 'static için çalışan ayrı türlerdir
    println!("{}", TypeId::of::<fn(&str)>() == TypeId::of::<fn(&'static str)>());
}
```

Çıktı:

```text
true
false
```

> [!NOTE]
> Bu alan hareketlidir. 2026'nın ağustos ayında nightly derleyicide iki yenilik varsayılan olarak açıldı: yeni trait çözümleyici ve Polonius ödünç denetleyicisinin alfa sürümü.[^gelecek] Yeni çözümleyici, yüksek dereceli türlerin içindeki ilişkili türlerde bazı davranışları değiştirir. Bu sayfadaki mesajlar kararlı 1.99.0 sürümüne aittir; 1.100.0 sürüm duyurusunda HRTB ya da GAT için yeni bir özellik yer almaz.

## Alıştırmalar

**1.** Şu imzayı yazın ve doldurun: `fn en_uzun_satir<F>(metin: &str, anahtar: F) -> Option<&str> where F: Fn(&str) -> usize`. `F`'nin sınırını açık `for<'a>` ile yeniden yazın; iki imzanın aynı olduğunu, ikisini de aynı closure ile çağırarak doğrulayın. Sonra imzayı bilerek `fn f<'a, F: Fn(&'a str) -> usize>(metin: &'a str, anahtar: F) -> Option<&'a str>` yapın: Gövde hâlâ derleniyor mu? Neden? `metin: &str` bırakırsanız ne olur?

<details>
<summary>İpucu ve çözüm</summary>

İpucu: Satırlar `metin`'den geliyor, yerel bir değişkenden değil.

```rust
fn en_uzun_satir<F>(metin: &str, anahtar: F) -> Option<&str>
where
    F: Fn(&str) -> usize,
{
    metin.lines().max_by_key(|satir| anahtar(satir))
}

fn en_uzun_satir_acik<F>(metin: &str, anahtar: F) -> Option<&str>
where
    F: for<'a> Fn(&'a str) -> usize,
{
    metin.lines().max_by_key(|satir| anahtar(satir))
}

fn f<'a, F: Fn(&'a str) -> usize>(metin: &'a str, anahtar: F) -> Option<&'a str> {
    metin.lines().max_by_key(|satir| anahtar(satir))
}

fn main() {
    let metin = String::from("kar\nyağmur\nsis");
    let uzunluk = |s: &str| s.chars().count();
    println!("{:?}", en_uzun_satir(&metin, uzunluk));
    println!("{:?}", en_uzun_satir_acik(&metin, uzunluk));
    println!("{:?}", f(&metin, uzunluk));
}
```

Çıktı:

```text
Some("yağmur")
Some("yağmur")
Some("yağmur")
```

Üçüncü sürüm de derlenir: `lines()` satırları `&'a str` olarak verir, closure'ın istediği de tam budur. Fonksiyon içeride yeni veri üretmediği için çağıranın seçtiği tek bir `'a` yeter. `metin: &str` bırakıldığında ise `metin`'in ömrü artık `'a` değildir; closure'a verilen satırlar `'a` ömrünü karşılamaz:

```rust,compile_fail,E0621
fn f<'a, F: Fn(&'a str) -> usize>(metin: &str, anahtar: F) -> Option<&'a str> {
    metin.lines().max_by_key(|satir| anahtar(satir))
}
```

Derleyicinin yanıtı:

```text
error[E0621]: explicit lifetime required in the type of `metin`
  |
2 |     metin.lines().max_by_key(|satir| anahtar(satir))
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ lifetime `'a` required
```

</details>

**2.** `OduncVerenIterator`'a varsayılan gövdeli şu metodu ekleyin: `fn her_biri(mut self, mut f: impl for<'a> FnMut(Self::Oge<'a>)) where Self: Sized`. Birikimli toplam örneğini onunla çağırmayı *deneyin*. Trait derlenir, ama çağrı E0597 verir; mesajdaki notu okuyup nedenini açıklayın. Sonra aynı metodu trait yerine `impl<'v, T> PencereMut<'v, T>` içine `f: impl FnMut(&mut [T])` ile yazın. Beklenen çıktı: `[1, 3, 6, 10, 15]`.

<details>
<summary>İpucu ve çözüm</summary>

İpucu: `for<'a>` her `'a`'yı kapsar, `where Self: 'a` ise her `'a` için `Self: 'a` ister.

```rust,compile_fail,E0597
trait OduncVerenIterator {
    type Oge<'a>
    where
        Self: 'a;

    fn sonraki<'a>(&'a mut self) -> Option<Self::Oge<'a>>;

    fn her_biri(mut self, mut f: impl for<'a> FnMut(Self::Oge<'a>))
    where
        Self: Sized,
    {
        while let Some(oge) = self.sonraki() {
            f(oge);
        }
    }
}

# struct PencereMut<'v, T> { veri: &'v mut [T], bas: usize, boy: usize }
# impl<'v, T> PencereMut<'v, T> {
#     fn yeni(veri: &'v mut [T], boy: usize) -> Self { PencereMut { veri, bas: 0, boy } }
# }
# impl<'v, T> OduncVerenIterator for PencereMut<'v, T> {
#     type Oge<'a> = &'a mut [T] where Self: 'a;
#     fn sonraki<'a>(&'a mut self) -> Option<Self::Oge<'a>> {
#         let son = self.bas.checked_add(self.boy)?;
#         let pencere = self.veri.get_mut(self.bas..son)?;
#         self.bas += 1;
#         Some(pencere)
#     }
# }
fn main() {
    let mut veri = [1, 2, 3, 4, 5];
    PencereMut::yeni(&mut veri, 2).her_biri(|p| p[1] += p[0]);
}
```

Derleyicinin yanıtı:

```text
error[E0597]: `veri` does not live long enough
   |
note: due to a current limitation of the type system, this implies a `'static` lifetime
   |
 8 |     fn her_biri(mut self, mut f: impl for<'a> FnMut(Self::Oge<'a>))
   |                                       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

`for<'a> FnMut(Self::Oge<'a>)` sınırı `Oge<'a>`'nın her `'a` için var olmasını, `where Self: 'a` koşulu da bunun için her `'a`'da `Self: 'a` olmasını gerektirir. Her ömrü, dolayısıyla `'static`'i de kapsamak `Self: 'static` demektir; yerel diziden ödünç alan `PencereMut` bunu sağlamaz. Bu, GAT ile HRTB birleşiminin GAT'ların kararlı kılındığı duyuruda da anılan, bilinen bir sınırıdır.[^gat-kararli]

Metodu türün kendisine yazınca GAT aradan çıkar. `FnMut(&mut [T])` sınırında öğe türü somuttur ve ek bir koşul taşımaz:

```rust
struct PencereMut<'v, T> {
    veri: &'v mut [T],
    bas: usize,
    boy: usize,
}

impl<'v, T> PencereMut<'v, T> {
    fn yeni(veri: &'v mut [T], boy: usize) -> Self {
        PencereMut { veri, bas: 0, boy }
    }

    fn sonraki(&mut self) -> Option<&mut [T]> {
        let son = self.bas.checked_add(self.boy)?;
        let pencere = self.veri.get_mut(self.bas..son)?;
        self.bas += 1;
        Some(pencere)
    }

    fn her_biri(mut self, mut f: impl FnMut(&mut [T])) {
        while let Some(pencere) = self.sonraki() {
            f(pencere);
        }
    }
}

fn main() {
    let mut veri = [1, 2, 3, 4, 5];
    PencereMut::yeni(&mut veri, 2).her_biri(|p| p[1] += p[0]);
    println!("{veri:?}");
}
```

Çıktı:

```text
[1, 3, 6, 10, 15]
```

</details>

**3.** `IsaretciAilesi`'ne `KutuAilesi` (`Box`) eklemeyi deneyin. Hangi sınır engel olur, neden? O sınırı kaldırırsanız `Belge::guncel` hangi imzaya dönmek zorunda kalır?

<details>
<summary>İpucu ve çözüm</summary>

İpucu: `Clone`. `Box<T>` yalnızca `T: Clone` ise klonlanır; `Rc<T>` ve `Arc<T>` her `T` için klonlanır, çünkü veriyi değil sayacı çoğaltırlar.

```rust,compile_fail,E0277
use std::ops::Deref;

trait IsaretciAilesi {
    type Isaretci<T>: Deref<Target = T> + Clone;
    fn yeni<T>(deger: T) -> Self::Isaretci<T>;
}

struct KutuAilesi;

impl IsaretciAilesi for KutuAilesi {
    type Isaretci<T> = Box<T>;
    fn yeni<T>(deger: T) -> Box<T> {
        Box::new(deger)
    }
}
```

Derleyicinin yanıtı:

```text
error[E0277]: the trait bound `T: Clone` is not satisfied
   |
11 |     type Isaretci<T> = Box<T>;
   |                        ^^^^^^ the trait `Clone` is not implemented for `T`
   |
   = note: required for `Box<T>` to implement `Clone`
```

`Clone` sınırı kaldırılınca `Box` aileye katılabilir; ama `guncel` artık işaretçiyi çoğaltamaz ve belgeden ödünç verilen bir referans döndürmek zorunda kalır:

```rust
use std::ops::Deref;

trait IsaretciAilesi {
    type Isaretci<T>: Deref<Target = T>;
    fn yeni<T>(deger: T) -> Self::Isaretci<T>;
}

struct KutuAilesi;

impl IsaretciAilesi for KutuAilesi {
    type Isaretci<T> = Box<T>;
    fn yeni<T>(deger: T) -> Box<T> {
        Box::new(deger)
    }
}

struct Belge<A: IsaretciAilesi> {
    surumler: Vec<A::Isaretci<String>>,
}

impl<A: IsaretciAilesi> Belge<A> {
    fn yeni(ilk: &str) -> Self {
        Belge { surumler: vec![A::yeni(ilk.to_string())] }
    }

    // Sahipli işaretçi yerine ödünç: sonuç belgeden uzun yaşayamaz.
    fn guncel(&self) -> &A::Isaretci<String> {
        self.surumler.last().expect("belge en az bir sürümle kurulur (bkz. yeni)")
    }
}

fn main() {
    let belge: Belge<KutuAilesi> = Belge::yeni("taslak");
    println!("{}", **belge.guncel());
}
```

Çıktı:

```text
taslak
```

Bedeli, sonucu başka bir thread'e taşıyamamanız ve belgeyi düzenlerken eski sürümü elinizde tutamamanızdır.

</details>

## Özet

- Bağımsız yaşayan veriler için bağımsız ömür parametreleri kullanılır; tek `'a` her şeyi kısa olana indirger.
- `'a: 'b`, "`'a` en az `'b` kadar sürer"; `T: 'a`, "`T`'nin içindeki referanslar en az `'a` kadar geçerlidir" demektir. `T: 'static`, sonsuza dek yaşamak değil, geçici hiçbir şeye borçlu olmamaktır.
- `&mut` ardındaki ömür esnetilemez (invaryant); eklenen verinin ilişkisini imzaya siz yazarsınız.
- `fn f<'a>` ömrü çağırana, `for<'a>` fonksiyonun gövdesine seçtirir. `Fn(&str)` yazımı `for<'a>`'yı kendiliğinden içerir; `let` ile tanımlanan closure ifadeleri içermez.
- GAT, ilişkili türe kendi ömür ya da tür parametresini verir: ödünç veren iterator ve tür ailesi bunun iki örneğidir.
- `is not general enough` mesajı gördüğünüzde bir `for<'a>` sınırı arayın.

Ömürlerin `Drop` ile etkileşimi [26. bölümde](26-dropck.md), `for<'a>` sınırlarının asenkron kodda `Send` future'larla çıkardığı dertler [29. bölümde](29-advanced-concurrency-async.md) ele alınır. Sıradaki bölüm [28. `Future`'ın İç Yüzü](28-future.md), `async` ve `.await`'in altındaki trait'i, uyandırıcıyı ve `Pin`'i açıyor.

## Kaynaklar

[^hrtb]: The Rust Reference, [Higher-ranked trait bounds](https://doc.rust-lang.org/reference/trait-bounds.html#higher-ranked-trait-bounds); [RFC 387: Higher-ranked trait bounds](https://rust-lang.github.io/rfcs/0387-higher-ranked-trait-bounds.html). Jon Gjengset, *Rust for Rustaceans*, Böl. 1 ve 2.
[^nomicon]: The Rustonomicon, [Higher-Rank Trait Bounds (HRTBs)](https://doc.rust-lang.org/nomicon/hrtb.html).
[^varyans]: The Rust Reference, [Variance](https://doc.rust-lang.org/reference/subtyping.html#variance); The Rustonomicon, [Subtyping and Variance](https://doc.rust-lang.org/nomicon/subtyping.html#variance).
[^omur-siniri]: The Rust Reference, [Lifetime bounds](https://doc.rust-lang.org/reference/trait-bounds.html#lifetime-bounds).
[^dyn-omur]: The Rust Reference, [Default trait object lifetimes](https://doc.rust-lang.org/reference/lifetime-elision.html#default-trait-object-lifetimes); hata dizini, [`E0309`](https://doc.rust-lang.org/error_codes/E0309.html) ve [`E0310`](https://doc.rust-lang.org/error_codes/E0310.html).
[^cikarilan]: The Rust Reference, [Implied bounds](https://doc.rust-lang.org/reference/trait-bounds.html#implied-bounds); struct alanları için [RFC 2093: Infer `T: 'x` outlives requirements on structs](https://rust-lang.github.io/rfcs/2093-infer-outlives.html) ve rustc 1.99.0 kaynak kodu, [`implicit_infer.rs`, `ty::Ref`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_analysis/src/outlives/implicit_infer.rs#L130).
[^terfi]: The Rust Reference, [Constant promotion](https://doc.rust-lang.org/reference/destructors.html#constant-promotion).
[^elision]: The Rust Reference, [Lifetime elision in functions](https://doc.rust-lang.org/reference/lifetime-elision.html#lifetime-elision-in-functions).
[^closure-imza]: rustc 1.99.0 kaynak kodu, [`closure.rs`, `sig_of_closure_with_expectation`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_typeck/src/closure.rs#L694) ve [`closure.rs`, `sig_of_closure_no_expectation`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_typeck/src/closure.rs#L635).
[^binder]: The Unstable Book, [`closure_lifetime_binder`](https://doc.rust-lang.org/unstable-book/language-features/closure-lifetime-binder.html); [RFC 3216: `for<'a> |...|` closures](https://rust-lang.github.io/rfcs/3216-closure-lifetime-binder.html).
[^serde]: serde, [Deserializer lifetimes](https://serde.rs/lifetimes.html) ve [`DeserializeOwned`](https://docs.rs/serde/latest/serde/de/trait.DeserializeOwned.html).
[^windows]: std, [`slice::windows`](https://doc.rust-lang.org/std/primitive.slice.html#method.windows), [`slice::chunks_mut`](https://doc.rust-lang.org/std/primitive.slice.html#method.chunks_mut) ve [`Cell::as_slice_of_cells`](https://doc.rust-lang.org/std/cell/struct.Cell.html#method.as_slice_of_cells).
[^iterator-next]: std, [`Iterator::next`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#tymethod.next); std 1.99.0 kaynak kodu, [`iterator.rs`, `fn next`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/iter/traits/iterator.rs#L78).
[^gat]: The Rust Reference, [Associated types](https://doc.rust-lang.org/reference/items/associated-items.html#associated-types); [RFC 1598: Generic associated types](https://rust-lang.github.io/rfcs/1598-generic_associated_types.html).
[^gat-where]: The Rust Reference, [Required where clauses on generic associated types](https://doc.rust-lang.org/reference/items/associated-items.html#required-where-clauses-on-generic-associated-types).
[^dyn-uyumlu]: The Rust Reference, [Dyn compatibility](https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility); rustc 1.99.0 kaynak kodu, [`dyn_compatibility.rs`, `generics_require_sized_self`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_trait_selection/src/traits/dyn_compatibility.rs#L357); hata dizini, [`E0038`](https://doc.rust-lang.org/error_codes/E0038.html).
[^otomatik]: The Rust Reference, [Auto traits](https://doc.rust-lang.org/reference/special-types-and-traits.html#auto-traits).
[^spawn]: std, [`thread::spawn`](https://doc.rust-lang.org/std/thread/fn.spawn.html) ve [`thread::scope`](https://doc.rust-lang.org/std/thread/fn.scope.html); hata dizini, [`E0521`](https://doc.rust-lang.org/error_codes/E0521.html) ve [`E0261`](https://doc.rust-lang.org/error_codes/E0261.html).
[^erken-gec]: rustc-dev-guide, [Early vs late bound parameters](https://rustc-dev-guide.rust-lang.org/early-late-parameters.html); hata dizini, [`E0794`](https://doc.rust-lang.org/error_codes/E0794.html).
[^gec-kod]: rustc 1.99.0 kaynak kodu, [`resolve_bound_vars.rs`, `is_late_bound_map`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_analysis/src/collect/resolve_bound_vars.rs#L2493).
[^turler]: The Unstable Book, [`non_lifetime_binders`](https://doc.rust-lang.org/unstable-book/language-features/non-lifetime-binders.html); hata dizini, [`E0658`](https://doc.rust-lang.org/error_codes/E0658.html).
[^binder-kod]: rustc 1.99.0 kaynak kodu, [`binder.rs`, `Binder`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_type_ir/src/binder.rs#L29).
[^yer-tutucu]: rustc 1.99.0 kaynak kodu, [`placeholder_error.rs`, `report_trait_placeholder_mismatch`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_trait_selection/src/error_reporting/infer/nice_region_error/placeholder_error.rs#L248).
[^gat-kod]: rustc 1.99.0 kaynak kodu, [`wfcheck.rs`, `check_gat_where_clauses`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_analysis/src/check/wfcheck.rs#L382).
[^gat-kararli]: The Rust Blog, [Generic associated types to be stable in Rust 1.65](https://blog.rust-lang.org/2022/10/28/gats-stabilization/) ve [Announcing Rust 1.65.0](https://blog.rust-lang.org/2022/11/03/Rust-1.65.0/).
[^terfi-kod]: rustc 1.99.0 kaynak kodu, [`promote_consts.rs`, `promote_candidates`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_mir_transform/src/promote_consts.rs#L1007).
[^silinme]: rustc 1.99.0 kaynak kodu, [`erase_regions.rs`, `erase_and_anonymize_regions`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_middle/src/ty/erase_regions.rs#L22).
[^typeid]: std, [`TypeId`: Danger of Improper Variance](https://doc.rust-lang.org/std/any/struct.TypeId.html#danger-of-improper-variance).
[^gelecek]: The Rust Blog, [Enabling the next-generation trait solver on nightly](https://blog.rust-lang.org/2026/08/21/enabling-next-solver-on-nightly/), [Enabling the next iteration of the borrow checker on nightly](https://blog.rust-lang.org/2026/08/04/enabling-polonius-alpha-on-nightly/) ve [Announcing Rust 1.100.0](https://blog.rust-lang.org/2026/11/12/Rust-1.100.0/).
