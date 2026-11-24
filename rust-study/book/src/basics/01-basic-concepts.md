# 1. Temel Kavramlar

Bir dili öğrenmeye, o dille yazılmış en küçük çalışan programı okuyarak başlamak iyi bir yoldur. Bu bölümde öyle yapacağız: Önce tek satırlık bir program çalıştıracak, ardından bir değere isim vermeyi, o ismi değiştirilebilir kılmayı, türünü belirtmeyi, küçük tarifler (fonksiyonlar) yazmayı ve "şu olursa bunu yap" demeyi göreceğiz.

Buradaki amaç her ayrıntıyı öğrenmek değil, Rust kodunun neye benzediğini tanımaktır. Aynı araçları [2. bölümde](02-common-concepts.md) kurallarıyla birlikte, tek tek ele alacağız.

> [!NOTE]
> Bu bölümün kodu `learn/basic-concepts` dizinindedir. O dosya bir deneme defteri gibidir: Temel konuların yanında ileri konulara da değinir ve şu an derlenmez (dört derleme hatası vardır). Bu sayfadaki örnekler derlenir hâle getirilmiştir; ileri konuların kitabın hangi bölümünde anlatıldığını sayfanın sonundaki [tabloda](#ileri-konular-nerede) bulabilirsiniz.

## Günlük hayattan bir benzetme

Defterinize "yaş: 7" yazdığınızı düşünün. Tükenmez kalemle yazdıysanız üstünü değiştiremezsiniz; kurşun kalemle yazdıysanız silip yenisini yazabilirsiniz. Rust'ta varsayılan kalem tükenmezdir: Bir isme değer verdikten sonra o değer değişmez. Kurşun kaleme geçmek istediğinizi `mut` sözcüğüyle açıkça söylersiniz.

Yazdığınızın sayı mı harf mi olduğu da belli olmalıdır; yoksa öğretmen ne yazdığınızı anlayamaz. Rust'ta bunun karşılığı *tür* (type) kavramıdır. Çoğu zaman öğretmen bunu bağlamdan kendisi çıkarır; çıkaramadığı yerde sizden söylemenizi ister.

<details>
<summary>Bu bölümde geçen terimler</summary>

- **bağlama (binding):** Bir ismi bir değere bağlamak. `let` anahtar sözcüğüyle yapılır.
- **değişken (variable):** `let` ile tanımlanan isim. Adına karşın, `mut` yazılmadıkça değeri değişmez.
- **değiştirilebilir (mutable):** Değeri sonradan değiştirilebilen. `mut` ile belirtilir.
- **tür (type):** Bir değerin ne olduğunu ve onunla neler yapılabileceğini belirleyen bilgi. Örneğin `i32`, 32 bitlik işaretli tam sayıdır.
- **literal:** Kodda doğrudan yazılan değer: `7`, `"merhaba"`, `true`.
- **sabit (constant):** Derleme zamanında bilinen ve hiç değişmeyen değer. `const` ile tanımlanır.
- **fonksiyon (function):** Girdi alıp iş yapan, gerekirse sonuç döndüren adlandırılmış kod parçası.
- **closure:** Çevresindeki değişkenleri yakalayabilen isimsiz fonksiyon.
- **kontrol akışı (control flow):** Programın hangi satırların çalışacağına karar vermesi: `if`, `loop`, `while`, `for`.
- **makro (macro):** Kod üreten kod. Adının sonundaki `!` işaretinden tanınır.

</details>

## İlk program

Rust projeleri *Cargo* adlı araçla oluşturulur, derlenir ve çalıştırılır:[^cargo]

```console
cargo new merhaba
cd merhaba
cargo run
```

`cargo new`, içinde ayarların durduğu `Cargo.toml` ile kodun durduğu `src/main.rs` dosyasını hazırlar. `cargo run` ise programı derler ve hemen çalıştırır. Cargo'nun yazdığı ilk program şuna benzer:

```rust
fn main() {
    println!("Merhaba, dünya!");
}
```

Çıktı:

```text
Merhaba, dünya!
```

Bu üç satırda dört şey vardır:[^anatomi]

- `fn main()`, `main` adında bir fonksiyon tanımlar. Çalıştırılabilir her Rust programında ilk çalışan kod budur.
- Süslü parantezler (`{}`) fonksiyonun gövdesini çevreler.
- `println!` ekrana bir satır yazar. Sondaki `!`, bunun fonksiyon değil makro olduğunu gösterir. Makroları [20. bölümde](../intermediate/20-macros.md) göreceğiz.
- Satır noktalı virgülle biter.

## Bir değere isim vermek: `let` ve `mut`

`let`, bir ismi bir değere bağlar. Varsayılan olarak bu bağ değişmez; değiştirebilmek için `mut` yazılır:

```rust
fn main() {
    let mut first_var: i32 = 10_i32;
    println!("önce: {first_var}");

    first_var = 101;
    println!("sonra: {first_var}");
}
```

Çıktı:

```text
önce: 10
sonra: 101
```

`println!` içindeki süslü parantezler, değişkenin değerinin yazılacağı yeri gösterir.

`mut` sözcüğünü kaldırırsanız program derlenmez. Aşağıdaki örnekte defterin tükenmez kalemle yazılmış hâlini görüyorsunuz:

```rust,compile_fail,E0384
fn main() {
    let yas = 7;
    yas = 8;
    println!("{yas}");
}
```

Derleyicinin yanıtı:

```text
error[E0384]: cannot assign twice to immutable variable `yas`
  |
2 |     let yas = 7;
  |         --- first assignment to `yas`
3 |     yas = 8;
  |     ^^^^^^^ cannot assign twice to immutable variable
  |
help: consider making this binding mutable
  |
2 |     let mut yas = 7;
  |         +++
```

Rust'ta bir değerin değişmeyeceğini söylediyseniz derleyici bunu garanti eder. Kodun bir bölümü "bu değer sabit" varsayımıyla yazılmışken başka bir bölümün onu sessizce değiştirmesi, bulunması güç hatalara yol açar. `mut` ise hem izni verir hem de okuyana "bu değer ileride değişecek" der.[^degisken] Bu seçimin altında yatan asıl gerekçeyi (*aliasing XOR mutability*) [2. bölümde](02-common-concepts.md) anlatacağız.

## Türü belirtmek

Önceki örnekte tür iki yerde birden yazılmıştı:

- `first_var: i32` ifadesi bir *tür açıklamasıdır* (type annotation): "`first_var`, `i32` türündedir."
- `10_i32` ifadesindeki `i32`, sayının *son ekidir* (suffix): "Bu `10`, `i32` türündedir." Aradaki alt çizgi okumayı kolaylaştırır; değere etkisi yoktur.

İkisinden biri yeterdi, hatta hiçbiri gerekmezdi: Derleyici türü çoğu zaman bağlamdan çıkarır; bağlam bir tam sayı türünü belirlemiyorsa `i32` seçer.[^tamsayi] Şu dört satır aynı sonucu verir:

```rust
fn main() {
    let a = 10;
    let b: i32 = 10;
    let c = 10i32;
    let d = 10_i32;

    println!("{}", a == b && b == c && c == d);
}
```

Çıktı:

```text
true
```

Rust *statik türlü* (statically typed) bir dildir: Bütün değişkenlerin türü derleme zamanında bilinmek zorundadır.[^tur] Türler uyuşmuyorsa program çalışmadan önce uyarılırsınız:

```rust,compile_fail,E0308
fn main() {
    let yas: i32 = "yedi";
    println!("{yas}");
}
```

Derleyicinin yanıtı:

```text
error[E0308]: mismatched types
  |
2 |     let yas: i32 = "yedi";
  |              ---   ^^^^^^ expected `i32`, found `&str`
  |              |
  |              expected due to this
```

`i32` bir tam sayı, `"yedi"` ise metindir; derleyici birini ötekinin yerine kendiliğinden koymaz.

## Hiç değişmeyen değerler: `const`

Programın her yerinde geçerli, hiç değişmeyecek bir değer `const` ile tanımlanır:

```rust
const N: usize = 4;

fn main() {
    println!("N = {N}");
}
```

Çıktı:

```text
N = 4
```

Sabitler, `mut` yazılmamış değişkenlere benzer; ama üç farkları vardır:[^sabit]

- `mut` ile kullanılamaz. Varsayılan olarak değil, her durumda değiştirilemezler.
- Türleri açıkça yazılmak zorundadır.
- Değerleri derleme zamanında hesaplanabilmelidir. Çalışma zamanında öğrenilen bir değer sabite verilemez.

Sabit adları büyük harfle ve alt çizgiyle yazılır: `EN_COK_DENEME`.

> [!NOTE]
> `const` sözcüğü bir yerde daha karşınıza çıkar: `struct Buffer<const N: usize = 0>`. Bu bir *const jenerik* parametredir ve sıradan sabitten farklıdır; [9. bölümde](../intermediate/09-generics-traits-lifetimes.md) ele alınır.

## Küçük tarifler: fonksiyonlar

Fonksiyon, adı olan bir tariftir: Malzemeleri (*parametreler*) alır, işi yapar, gerekiyorsa sonucu geri verir.

```rust
fn alan(en: i32, boy: i32) -> i32 {
    en * boy
}

fn main() {
    let sonuc = alan(3, 4);
    println!("alan: {sonuc}");
}
```

Çıktı:

```text
alan: 12
```

Parametrelerin türü yazılmak zorundadır. `-> i32`, fonksiyonun bir `i32` döndüreceğini söyler. Gövdedeki `en * boy` satırının sonunda noktalı virgül **yoktur**: Son satır bir ifadeyse o ifadenin değeri fonksiyonun sonucu olur.[^donus]

Buraya noktalı virgül koymak, Rust'a yeni başlayanların sık düştüğü bir hatadır:

```rust,compile_fail,E0308
fn alan(en: i32, boy: i32) -> i32 {
    en * boy;
}
# fn main() { println!("{}", alan(3, 4)); }
```

Derleyicinin yanıtı:

```text
error[E0308]: mismatched types
  |
1 | fn alan(en: i32, boy: i32) -> i32 {
  |    ----                       ^^^ expected `i32`, found `()`
  |    |
  |    implicitly returns `()` as its body has no tail or `return` expression
2 |     en * boy;
  |             - help: remove this semicolon to return this value
```

Noktalı virgül, ifadeyi *deyime* (statement) çevirir; deyim değer döndürmez. Derleyici bunu görür ve noktalı virgülü kaldırmanızı önerir. Mesajdaki `()`, "boş değer" anlamına gelir.

## Karar vermek ve yinelemek

`if`, bir koşula göre hangi kodun çalışacağını seçer. Rust'ta `if` bir *ifadedir* (expression); yani sonucu bir değişkene verilebilir:[^if]

```rust
fn main() {
    let sicaklik = 18;
    let durum = if sicaklik < 20 { "serin" } else { "ılık" };
    println!("hava {durum}");

    for gun in 1..=3 {
        println!("{gun}. gün");
    }
}
```

Çıktı:

```text
hava serin
1. gün
2. gün
3. gün
```

`for`, bir dizinin ya da aralığın öğelerini sırayla dolaşır. `1..=3`, 1'den 3'e kadar (3 dâhil) bütün sayılar demektir. `loop` ve `while` döngüleriyle birlikte ayrıntılarını [2. bölümde](02-common-concepts.md) göreceğiz.

## İsimsiz fonksiyonlar: closure

Küçük bir işi ayrı bir fonksiyon yazmadan, yerinde tanımlamak isterseniz *closure* kullanırsınız. Parametreler dikey çizgiler arasına yazılır:

```rust
fn main() {
    let aynen = |y: i8| -> i8 { y };
    println!("{}", aynen(1));

    let k = 3;
    let ekle = |x| x + k;
    println!("{}", ekle(4));
}
```

Çıktı:

```text
1
7
```

İlk closure, aldığı değeri olduğu gibi geri verir. İkincisi ise dışarıda tanımlanmış `k`'yi kullanır: Closure, çevresindeki değişkeni *yakalar* (capture). "Kapanış" adı da buradan gelir; closure çevresini içine kapatır. Sıradan bir `fn` fonksiyonu bunu yapamaz.[^closure]

İkinci closure'da türleri yazmadık. Derleyici, `ekle(4)` çağrısından ve `x + k` toplamından gerekli türleri çıkarır.

## Hata mesajını okumak

Derleyicinin mesajları bu kitapta sık sık karşınıza çıkacak. Hepsi aynı düzendedir:

| Parça | Ne söyler? |
| --- | --- |
| `error[E0384]: cannot assign twice…` | Hatanın kodu ve özeti. Kod, açıklamasını aramak için kullanılır. |
| `--> src/main.rs:3:5` | Dosya, satır ve sütun. Kitapta kısalık için yazılmaz. |
| Altı çizili satırlar (`^^^`, `---`) | `^` asıl sorunu, `-` ise sorunla ilgili öteki yerleri gösterir. |
| `help:` | Önerilen düzeltme. `+` ile işaretli metin eklenir, `-` ile işaretli metin çıkarılır. |
| `note:` | Ek bilgi ya da gerekçe. |

Bir hata kodunun uzun açıklamasını terminalde okuyabilirsiniz:[^e0384]

```console
rustc --explain E0384
```

Derleme hatası almak sizin kötü bir programcı olduğunuzu göstermez; deneyimli Rust geliştiricileri de bu hataları alır. Hata, programın istediğiniz işi güvenle yapamadığını, çalışmadan önce haber verir.[^degisken]

## İleri konular nerede?

`learn/basic-concepts/src/main.rs` dosyası, adına karşın Rust'ın en ileri konularından bazılarına da değinir. Bu konular, okurun hazır olduğu bölümlerde anlatılır:

| Konu | Kitaptaki yeri |
| --- | --- |
| Ham işaretçi, `ptr::read`, `unsafe`'in gerekçesi, `unsafe extern "C" fn` | [19. Gelişmiş Özellikler](../intermediate/19-advanced-features.md) |
| Const jenerik (`Buffer<const N: usize = 0>`) | [9. Generic, Trait ve Lifetime](../intermediate/09-generics-traits-lifetimes.md) |
| Trait sınırı, ilişkili tür, inherent `impl` (`Color<F>`, `Animal<T>`) | [9. Generic, Trait ve Lifetime](../intermediate/09-generics-traits-lifetimes.md) |
| `Fn`, `FnMut`, `FnOnce` ve fonksiyon işaretçisi | [12. Closure ve Iterator'lar](../intermediate/12-closures-iterators.md) |
| `#[proc_macro]`'nun sıradan crate'te derlenmemesi | [20. Makrolar](../intermediate/20-macros.md) |
| Yetim kuralı, newtype, RFC 3323 | [23. Tutarlılık ve Yetim Kuralı](../advanced/23-coherence.md) |
| Auto trait ve negatif `impl` | [25. `Send` ve `Sync`'in İç Yüzü](../advanced/25-send-sync.md) |
| HRTB (`hex_v2`, `hex_v3`) ve sabit terfisi | [27. İleri Lifetime, HRTB ve GAT](../advanced/27-lifetime-hrtb-gat.md) |

## Hata kodu başvurusu

| Kod | Ne zaman görülür? | Ne yapmalı? |
| --- | --- | --- |
| **E0384** | `mut` olmayan bir değişkene ikinci kez değer atanırsa. | `let mut` yazın ya da yeni bir değişken tanımlayın. |
| **E0308** | Beklenen tür ile verilen tür uyuşmazsa. | Türü düzeltin; fonksiyon sonundaysa fazladan noktalı virgülü silin. |

## Özet

- Çalıştırılabilir program `main` fonksiyonundan başlar; `cargo run` derler ve çalıştırır.
- `let` bir ismi değere bağlar. Değeri değiştirmek için `let mut` gerekir.
- Her değerin bir türü vardır. Derleyici türü çoğu zaman kendisi çıkarır; isterseniz tür açıklaması ya da literal son ekiyle siz belirtirsiniz.
- `const`, derleme zamanında bilinen ve türü yazılmak zorunda olan değişmez bir değerdir.
- Fonksiyonun son ifadesi, noktalı virgülle bitmiyorsa dönüş değeridir.
- `if` bir ifadedir; `for` bir aralığı ya da diziyi dolaşır.
- Closure, çevresindeki değişkenleri yakalayabilen isimsiz fonksiyondur.

Sıradaki bölüm: [2. Yaygın Kavramlar](02-common-concepts.md).

## Kaynaklar

[^cargo]: The Rust Programming Language, [1.3 Hello, Cargo!](https://doc.rust-lang.org/book/ch01-03-hello-cargo.html#creating-a-project-with-cargo).
[^anatomi]: The Rust Programming Language, [1.2 The Anatomy of a Rust Program](https://doc.rust-lang.org/book/ch01-02-hello-world.html#the-anatomy-of-a-rust-program).
[^degisken]: The Rust Programming Language, [3.1 Variables and Mutability](https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html).
[^tamsayi]: The Rust Reference, [Integer literal expressions](https://doc.rust-lang.org/reference/expressions/literal-expr.html#integer-literal-expressions).
[^tur]: The Rust Programming Language, [3.2 Data Types](https://doc.rust-lang.org/book/ch03-02-data-types.html).
[^sabit]: The Rust Programming Language, [3.1 Declaring Constants](https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html#declaring-constants).
[^donus]: The Rust Programming Language, [3.3 Functions with Return Values](https://doc.rust-lang.org/book/ch03-03-how-functions-work.html#functions-with-return-values).
[^if]: The Rust Programming Language, [3.5 Using `if` in a `let` Statement](https://doc.rust-lang.org/book/ch03-05-control-flow.html#using-if-in-a-let-statement).
[^closure]: The Rust Programming Language, [13.1 Closures](https://doc.rust-lang.org/book/ch13-01-closures.html#capturing-the-environment).
[^e0384]: Rust Compiler Error Index, [E0384](https://doc.rust-lang.org/error_codes/E0384.html).
