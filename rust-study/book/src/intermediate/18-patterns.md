# 18. Desenler (Patterns)

*Desen* (pattern), bir değerin yapısını tarif eden kalıptır. Desenle iki iş birden yaparsınız: Değere "şu biçimde misin?" diye sorar, uyuyorsa içindeki parçalara ad verirsiniz. Bu ikinci işe *parçalama* (destructuring) denir. Desen olmasaydı bir demetin "sıfırıncı elemanını, birinci elemanını" elle çıkarır, değerin biçimini de ayrıca sorardınız; ikisinden birini unutmak kolaydır.

Desenleri [5. bölümde](../basics/05-enums.md) `match` ve `if let` ile kullanmıştık. Oysa aynı dil bilgisi `let` deyiminde, `while let` ve `for` döngülerinde, fonksiyon ve closure parametrelerinde de geçerlidir.[^desenler]

Bu bölümde önce desenlerin kullanıldığı yerleri gezeceğiz. Ardından her değere uyan ve uymayabilen desenleri ayıracak, desen sözdiziminin bütün araçlarını tek tek göreceğiz. Son olarak derleyicinin "açıkta kalan durum var mı?" sorusuna nasıl cevap verdiğine bakacağız.

> [!NOTE]
> Bu bölümün kodu `learn/patterns` dizinindedir. Depo kökünden `cargo run -p patterns` komutuyla çalıştırabilirsiniz.

## Günlük hayattan bir benzetme

Kapağında yıldız, kare ve daire biçiminde delikler olan bir oyuncak kutusu düşünün. Elinizdeki parçayı deliklere tutarsınız; şekli uyan delikten geçer, uymayana zorla sokamazsınız. Desen deliktir, değer de parçadır. Üstelik parça delikten geçerken açılır: İçindeki boncuğu alıp "buna Ali diyelim" diyebilirsiniz. Bu, parçaya ad vermektir.

Delikler çeşit çeşittir: tam şu değer, şu ya da bu, şundan şuna kadar, gerisine bakmıyorum. Kimi delik de "şekli uydu ama bir sorum daha var" der. Parça delikleri yukarıdan aşağıya sırayla dener ve ilk uyduğundan geçer; bu yüzden dar delikler üstte, geniş delikler altta durur.

Bazı delikler her parçayı geçirir: kocaman, yuvarlak bir ağız gibi. Bazıları yalnızca tek bir şekli geçirir. Düz `let`, parçanın kesin geçeceği bir delik ister, çünkü "geçmezse ne olacak?" sorusuna verecek cevabı yoktur. `if let` ve `let ... else` ise bu cevabı yanında taşır: Uymazsa bloğu atla ya da buradan çık.

<details>
<summary>Bu bölümde geçen terimler</summary>

- **desen (pattern):** Değerin biçimini tarif eden kalıp. Oyuncaktaki delik.
- **eşleştirme (matching):** Değeri desene tutup "uyuyor mu?" diye bakmak.
- **parçalama (destructuring):** Bir paketi açıp içindeki parçalara ayrı ayrı ad vermek.
- **bağlama (binding):** Eşleşen parçaya bir değişken adı vermek.
- **`match` ve kol (arm):** Değeri sırayla desenlere tutan yapı ve o desenlerin her biri.
- **`if let` ve `while let`:** Tek desene bakan kısa `match` ve desen uydukça dönen döngü.
- **`let ... else`:** "Uyarsa devam et, uymazsa buradan çık" yazımı.
- **çürütülemez (irrefutable):** Her değere uyan desen. Eşleşmeme ihtimali yoktur.
- **çürütülebilir (refutable):** Bazı değerlere uymayabilen desen. `Some(x)`, `None`'a uymaz.
- **literal:** Koda doğrudan yazılmış değer: `1`, `'a'`, `"selam"`.
- **`|` ve `..=`:** "Şu ya da bu" ve "şundan şuna kadar" (aralık).
- **`_` ve `..`:** "Bu parçaya bakmıyorum" ve "geri kalanların hiçbirine bakmıyorum".
- **koruyucu koşul (match guard):** Desenden sonra gelen ek soru: `if x > 0`.
- **`@`:** "Hem şu desene uyup uymadığına bak hem de değere ad ver".
- **gölgeleme (shadowing):** Aynı adın yeni bir değişkene verilmesi; eskisi arkada kalır.
- **demet (tuple):** Birkaç değerin parantez içinde yan yana durduğu yapı: `(1, 2)`.
- **varyant (variant):** Bir enum'un alabileceği biçimlerden her biri.
- **`Option`, `Some`, `None`:** "Ya bir değer var ya da yok" kutusu ve iki hâli.
- **eksiksizlik denetimi (exhaustiveness):** Derleyicinin "açıkta kalan durum var mı?" denetimi.
- **erişilemez kol (unreachable pattern):** Üstteki kollar her şeyi aldığı için hiç çalışmayan kol.

</details>

## Desenlerin kullanıldığı yerler

### `let` deyimi

Her `let` deyiminde bir desen vardır. `let x = 5;` yazdığınızda `x`, "ne gelirse gelsin buna x adını ver" diyen en basit desendir. Daha zengin bir desenle bir demeti tek satırda parçalayabilirsiniz:[^let]

```rust
fn main() {
    let (a, b, c) = (1, 2, 3);
    println!("let destructuring: {a} {b} {c}");
}
```

Çıktı:

```text
let destructuring: 1 2 3
```

Desenin biçimi değerin türüyle uyuşmalıdır. İki parçalı bir desen üç parçalı bir değeri açamaz:

```rust,compile_fail,E0308
fn main() {
    let (c, d) = (1, 2, 3);
}
```

Derleyicinin yanıtı:

```text
error[E0308]: mismatched types
  |     let (c, d) = (1, 2, 3);
  |         ^^^^^^   --------- this expression has type `({integer}, {integer}, {integer})`
  |         |
  |         expected a tuple with 3 elements, found one with 2 elements
```

Elemanlardan birini kullanmayacaksanız `let (c, d, _) = (1, 2, 3);` yazın; `_` ve `..` ile yok saymayı birazdan göreceğiz.

### `match` kolları

`match` kolları `desen => ifade` biçimindedir. Kollar yukarıdan aşağıya denenir, ilk uyan çalışır. Kolların birlikte değerin bütün olası hâllerini kapsaması şarttır; buna *eksiksiz* (exhaustive) olmak denir.[^match]

```rust
fn main() {
    let sayi = 7;
    let parite = match sayi % 2 {
        0 => "çift",
        _ => "tek",
    };
    println!("match: {sayi} -> {parite}");
}
```

Çıktı:

```text
match: 7 -> tek
```

### `if let`, `else if let` ve `else`

`if let`, tek bir desene bakan kısa bir `match`'tir. `else if` ve `else if let` ile zincirlenebilir; zincirdeki koşulların aynı değerle ilgili olması gerekmez:[^iflet]

```rust
fn main() {
    let renk: Option<&str> = Some("mavi");
    if let Some(r) = renk {
        println!("if let: renk {r}");
    } else {
        println!("if let: renk yok");
    }

    let secilen: Option<&str> = None;
    let yas: Result<u8, _> = "34".parse::<u8>();
    if let Some(r) = secilen {
        println!("seçilen renk: {r}");
    } else if let Ok(y) = yas {
        println!("renk seçilmemiş, yaş: {y}");
    } else {
        println!("hiçbir bilgi yok");
    }
}
```

Çıktı:

```text
if let: renk mavi
renk seçilmemiş, yaş: 34
```

`if let` zincirinin bir eksiği vardır: Derleyici burada eksiksizlik denetimi yapmaz. Son `else` unutulursa bazı durumlar sessizce atlanır. Bütün durumların ele alındığından emin olmak istiyorsanız `match` kullanın.

### `while let`

`while let`, desen uyduğu sürece döner:[^whilelet]

```rust
fn main() {
    let mut yigin = vec![1, 2, 3];
    print!("while let (pop): ");
    while let Some(ust) = yigin.pop() {
        print!("{ust} ");
    }
    println!();
}
```

Çıktı:

```text
while let (pop): 3 2 1 
```

`pop`, liste boşalınca `None` döndürür; desen uymadığı için döngü biter.

### `for` döngüsü

`for` sözcüğünden sonra gelen de bir desendir. `enumerate`, her adımda `(sıra, değer)` demeti verir; demeti hemen orada açabilirsiniz:[^for]

```rust
fn main() {
    print!("for destructuring: ");
    for (i, harf) in ['a', 'b', 'c'].iter().enumerate() {
        print!("{i}:{harf} ");
    }
    println!();
}
```

Çıktı:

```text
for destructuring: 0:a 1:b 2:c 
```

### Fonksiyon ve closure parametreleri

Parametreler de desendir. `konum`, kendisine gelen demet referansını imzada açar; closure da aynısını yapar:[^parametre]

```rust
fn konum(&(x, y): &(i32, i32)) -> String {
    format!("({x}, {y})")
}

fn main() {
    println!("fn param destructuring: {}", konum(&(3, 4)));

    let satirlar: Vec<String> =
        [("vida", 42), ("pul", 0)].iter().map(|(ad, adet)| format!("{ad}={adet}")).collect();
    println!("{satirlar:?}");
}
```

Çıktı:

```text
fn param destructuring: (3, 4)
["vida=42", "pul=0"]
```

`&(x, y)` desenindeki `&`, "gelen referansın içindeki demeti aç" demektir. `i32` kopyalanabildiği için `x` ve `y` birer kopya olur.

## Çürütülebilir ve çürütülemez desenler

Her değere uyan desene *çürütülemez* (irrefutable) denir: `let x = 5;` içindeki `x` her şeye uyar. Bazı değerlere uymayabilen desene ise *çürütülebilir* (refutable) denir: `Some(x)`, değer `None` ise uymaz. Fonksiyon parametreleri, `let` deyimi ve `for` döngüsü yalnızca çürütülemez desen kabul eder, çünkü desen uymadığında yapabilecekleri bir şey yoktur. `if let`, `while let` ve `let ... else` ise çürütülebilir desenler içindir.[^curutme]

Düz `let`'e çürütülebilir bir desen yazalım:

```rust,compile_fail,E0005
fn main() {
    let veri: Option<i32> = None;
    let Some(x) = veri;
    println!("{x}");
}
```

Derleyicinin yanıtı:

```text
error[E0005]: refutable pattern in local binding
  |     let Some(x) = veri;
  |         ^^^^^^^ pattern `None` not covered
  |
  = note: `let` bindings require an "irrefutable pattern", like a `struct` or an `enum` with only one variant
  = note: for more information, visit https://doc.rust-lang.org/book/ch19-02-refutability.html
  = note: the matched value is of type `Option<i32>`
help: you might want to use `let...else` to handle the variant that isn't matched
```

Bu satır derlenseydi, `veri` `None` olduğunda `x`'e konacak bir değer bulunamazdı. Aynı hata `for` döngüsünde "refutable pattern in `for` loop binding", parametrede "refutable pattern in function argument" diye görünür; kod yine `E0005`'tir.

Çözüm, uymama durumunun cevabını da yazmaktır. Bunun üç yolu vardır.

### Yol 1: `let ... else`

Desen uyarsa değişken bağlanır ve *dışarıda* kalır; uymazsa `else` bloğu çalışır. Bu bloğun kapsamdan çıkması zorunludur: `return`, `break`, `continue` ya da `panic!` gibi geri dönmeyen bir ifadeyle bitmelidir:[^letelse]

```rust
fn reddedilebilirlik() {
    let veri: Option<i32> = Some(99);
    let Some(x) = veri else {
        println!("veri yok, çık");
        return;
    };
    println!("let-else ile bağlandı: x = {x}"); // x burada görünür
}

fn main() {
    reddedilebilirlik();
}
```

Çıktı:

```text
let-else ile bağlandı: x = 99
```

`if let`'ten farkı, asıl akışın girintisiz devam etmesidir. `let ... else` Rust 1.65'ten beri kararlıdır.[^letelse-surum] `else` bloğu çıkmıyorsa program derlenmez:

```rust,compile_fail,E0308
fn ilk(veri: Option<i32>) -> i32 {
    let Some(x) = veri else {
        println!("veri yok");
    };
    x
}
```

Derleyicinin yanıtı:

```text
error[E0308]: `else` clause of `let...else` does not diverge
  |       let Some(x) = veri else {
  |  _____________________________^
  | |         println!("veri yok");
  | |     };
  | |_____^ expected `!`, found `()`
  |
  = note:   expected type `!`
          found unit type `()`
  = help: try adding a diverging expression, such as `return` or `panic!(..)`
  = help: ...or use `match` instead of `let...else`
```

Mesajdaki `!`, "buradan geri dönülmez" anlamına gelen *never* türüdür; [19. bölümde](19-advanced-features.md) ele alınır. Bu türün adını her yerde tür olarak yazabilmek ise 1.100.0 sürümünde kararlı oldu.[^never-surum]

### Yol 2: `if let`

Yalnızca desen uyduğunda yapılacak bir iş varsa `if let` yeterlidir; uymazsa blok atlanır:

```rust
fn main() {
    let veri: Option<i32> = Some(5);
    if let Some(m) = veri {
        println!("değer: {m}");
    }
}
```

Çıktı:

```text
değer: 5
```

### Yol 3: `let` zincirleri

2024 edition'ında `if` ve `while` koşullarında `let` desenleri `&&` ile birbirine ve sıradan koşullara bağlanabilir. Soldaki `let`'in bağladığı ad, sağındaki koşulda kullanılabilir:[^letzincir]

```rust
fn main() {
    let yas: Result<u8, _> = "34".parse::<u8>();
    let sehir: Option<&str> = Some("İzmir");

    if let Ok(y) = yas
        && y > 30
        && let Some(s) = sehir
    {
        println!("{s}, {y} yaşında");
    }
}
```

Çıktı:

```text
İzmir, 34 yaşında
```

`let` zincirleri Rust 1.88'den beri kararlıdır ve yalnızca 2024 edition'ında kullanılabilir.[^letzincir]

> [!NOTE]
> The Rust Programming Language'ın 19.1. bölümü, `if let Ok(age) = age && age > 30` yazılamayacağını söyler. Bu ifade `let` zincirlerinden önce yazılmıştır; 2024 edition'ında bu koşul derlenir.

### Gereksiz `if let`

Ters yöndeki durum hata değil, uyarıdır. Çürütülemez bir desen `if let` ile yazıldığında koşul hiçbir zaman yanlış olamaz:

```rust
fn main() {
    let sayi = 5;
    if let x = sayi {
        println!("{x}");
    }
}
```

Derleyici şu uyarıyı verir:

```text
warning: irrefutable `if let` pattern
  |     if let x = sayi {
  |        ^^^^^^^^^^^^
  |
  = note: this pattern will always match, so the `if let` is useless
  = help: consider replacing the `if let` with a `let`
```

`let ... else` için de benzer bir uyarı vardır: "unreachable `else` clause". İkisi de `irrefutable_let_patterns` denetiminden gelir.[^irrefutable-lint]

## Desen sözdizimi

### Literaller, `|` ve aralıklar

Desende sabit bir değer, `|` ile ayrılmış seçenekler ya da bir aralık bulunabilir:[^literal][^veya][^aralik]

```rust
fn main() {
    for n in [1, 5, 7, 42] {
        let etiket = match n {
            1 | 2 => "bir ya da iki",
            3..=9 => "üç ile dokuz arası",
            _ => "başka",
        };
        println!("match {n}: {etiket}");
    }
}
```

Çıktı:

```text
match 1: bir ya da iki
match 5: üç ile dokuz arası
match 7: üç ile dokuz arası
match 42: başka
```

Aralık desenleri sayılarda ve `char` değerlerinde kullanılır; derleyici aralığın boş olup olmadığını yalnızca böyle türlerde önceden bilebilir. Uçları açık da bırakabilirsiniz:[^aralik]

| Desen | Anlamı |
| --- | --- |
| `3..=9` | 3 ile 9 arası, 9 dâhil |
| `3..9` | 3 ile 9 arası, 9 hariç (Rust 1.80'den beri)[^aralik-surum] |
| `10..` | 10 ve üstü |
| `..=-1` | -1 ve altı |

### Adlar ve gölgeleme

Desende geçen ad **yeni** bir değişkendir. Dışarıda aynı adda bir değişken bulunsa bile ona bakılmaz; kolun içinde gölgelenir:[^adlar]

```rust
fn main() {
    let x = Some(10);
    let y = 20;
    match x {
        Some(y) => println!("shadowing: içteki y={y} (dıştaki 20 değil)"),
        None => {}
    }
    println!("shadowing: dıştaki y hâlâ {y}");
}
```

Çıktı:

```text
shadowing: içteki y=10 (dıştaki 20 değil)
shadowing: dıştaki y hâlâ 20
```

`Some(y)` "içindeki 20 mi?" diye sormaz, "içindekine y adını ver" der. Dıştaki değişkenle karşılaştırmak için koruyucu koşul gerekir; birazdan göreceğiz.

Tek istisna sabitlerdir. Kapsamda o adda bir `const` (ya da `None` gibi alansız bir varyant) varsa desendeki ad yeni bir değişken değil, o sabitin kendisidir ve değer onunla karşılaştırılır.[^yol-deseni]

### Struct'ları açmak

Struct deseni alan adlarını kullanır. Alanla aynı adda bir değişken istiyorsanız kısa yazım yeter; bir alanı sabit bir değerle karşılaştırabilir, gerisini `..` ile yok sayabilirsiniz:[^struct]

```rust
struct Nokta {
    x: i32,
    y: i32,
}

fn main() {
    let p = Nokta { x: 0, y: 7 };
    let Nokta { x, y } = p; // Nokta { x: x, y: y } yazımının kısası
    println!("struct destructuring: x={x} y={y}");

    let p2 = Nokta { x: 10, y: 20 };
    match p2 {
        Nokta { x: 0, y } => println!("y ekseninde, y={y}"),
        Nokta { x, .. } => println!("x={x}, geri kalanı umursama"),
    }
}
```

Çıktı:

```text
struct destructuring: x=0 y=7
x=10, geri kalanı umursama
```

Bir alanı unutursanız derleyici bunu sessizce geçmez:

```rust,compile_fail,E0027
# struct Nokta {
#     x: i32,
#     y: i32,
# }
#
fn main() {
    let p = Nokta { x: 0, y: 7 };
    let Nokta { x } = p;
    println!("{x}");
}
```

Derleyicinin yanıtı:

```text
error[E0027]: pattern does not mention field `y`
  |     let Nokta { x } = p;
  |         ^^^^^^^^^^^ missing field `y`
  |
help: include the missing field in the pattern
help: if you don't care about this missing field, you can explicitly ignore it
help: or always ignore missing fields here
```

Bu zorunluluk bir güvencedir: Struct'a yeni bir alan eklendiğinde `..` yazılmamış her desen derlenmez olur ve sizi güncellenecek yere götürür. Var olmayan bir alan yazarsanız da `E0026` alırsınız.[^e0026]

### Enum'ları açmak

Enum desenleri varyantın biçimini izler: alanı olmayan, adlandırılmış alanlı ve demet biçimli varyantlar kendi yazımlarıyla açılır:[^enum]

```rust
enum Mesaj {
    Cik,
    Tasi { x: i32, y: i32 },
    Renk(u8, u8, u8),
    Yaz(String),
}

fn main() {
    for m in [
        Mesaj::Cik,
        Mesaj::Tasi { x: 3, y: 4 },
        Mesaj::Renk(255, 0, 0),
        Mesaj::Yaz(String::from("selam")),
    ] {
        match m {
            Mesaj::Cik => println!("enum: çık"),
            Mesaj::Tasi { x, y } => println!("enum: taşı ({x},{y})"),
            Mesaj::Renk(r, _, _) => println!("enum: renk, kırmızı={r}"),
            Mesaj::Yaz(s) => println!("enum: yaz '{s}'"),
        }
    }

    // üç kanalın hepsini bağlayan desen
    if let Mesaj::Renk(r, g, b) = Mesaj::Renk(10, 20, 30) {
        println!("tam renk destructuring: r={r} g={g} b={b}");
    }
}
```

Çıktı:

```text
enum: çık
enum: taşı (3,4)
enum: renk, kırmızı=255
enum: yaz 'selam'
tam renk destructuring: r=10 g=20 b=30
```

Desenler iç içe de yazılabilir; enum içindeki struct'ı, onun alanını tek desende açabilirsiniz. İç içe örnekler [18.1. bölümdedir](18-patterns-deep-dive.md).

### `_` ve `..` ile yok saymak

`_` tek bir değeri, `..` geri kalanların hepsini yok sayar:[^joker][^kalan]

```rust
fn main() {
    let sayilar = (2, 4, 8, 16, 32);
    let (ilk, .., son) = sayilar;
    let (_, ikinci, ..) = sayilar;
    println!("ilk={ilk} ikinci={ikinci} son={son}");
}
```

Çıktı:

```text
ilk=2 ikinci=4 son=32
```

Aynı demet deseninde `..` yalnızca bir kez geçebilir. İki kez yazarsanız derleyici kaç elemanın atlanacağını bilemez:

```rust,compile_fail
fn main() {
    let sayilar = (2, 4, 8, 16, 32);
    let (.., ikinci, ..) = sayilar;
    println!("{ikinci}");
}
```

Derleyicinin yanıtı:

```text
error: `..` can only be used once per tuple pattern
  |     let (.., ikinci, ..) = sayilar;
  |          --          ^^ can only be used once per tuple pattern
  |          |
  |          previously used here
```

`_` ile `_s` gibi alt çizgiyle *başlayan* ad aynı şey değildir. `_` hiçbir şeyi bağlamaz; `_s` ise sıradan bir değişkendir, yalnızca "kullanılmadı" uyarısını susturur. Fark sahiplikte ortaya çıkar:[^altcizgi]

```rust,compile_fail,E0382
fn main() {
    let s = Some(String::from("merhaba"));
    if let Some(_s) = s {
        println!("metin var");
    }
    println!("{s:?}");
}
```

Derleyicinin yanıtı:

```text
error[E0382]: borrow of partially moved value: `s`
  |     if let Some(_s) = s {
  |                 -- value partially moved here
...
  |     println!("{s:?}");
  |                ^ value borrowed here after partial move
  |
  = note: partial move occurs because value has type `String`, which does not implement the `Copy` trait
help: borrow this binding in the pattern to avoid moving the value
```

`_s` metni sahiplendiği için `s`'in içi boşalmıştır. `Some(_)` yazarsanız hiçbir şey taşınmaz:

```rust
fn main() {
    let s = Some(String::from("merhaba"));
    if let Some(_) = s {
        println!("metin var");
    }
    println!("{s:?}");
}
```

Çıktı:

```text
metin var
Some("merhaba")
```

### Koruyucu koşullar

*Koruyucu koşul* (match guard), desenden sonra gelen bir `if`'tir. Desen uysa bile koşul sağlanmazsa kol seçilmez, sıradaki kol denenir:[^koruyucu]

```rust
fn main() {
    let cift = (2, -2);
    match cift {
        (x, y) if x + y == 0 => println!("guard: toplamı sıfır"),
        (x, _) if x % 2 == 0 => println!("guard: ilk çift"),
        _ => println!("guard: diğer"),
    }

    // dıştaki bir değişkenle karşılaştırma
    let aranan = 20;
    match Some(10) {
        Some(n) if n == aranan => println!("aranan bulundu"),
        Some(n) => println!("{n}, {aranan} değil"),
        None => println!("boş"),
    }
}
```

Çıktı:

```text
guard: toplamı sıfır
10, 20 değil
```

İlk `match`'te desenin tek başına soramayacağı bir soru sorduk: Desen parçaları sabitlerle karşılaştırır, birbirleriyle karşılaştıramaz. İkincisinde de gölgeleme tuzağını aştık: `n` yeni bir addır, `aranan` ise dıştaki değişkendir.

Rust 1.95'ten beri koruyucu koşulda `if let` de yazılabilir:[^iflet-guard]

```rust
fn main() {
    for metin in [Some("42"), Some("kırk iki"), None] {
        match metin {
            Some(s) if let Ok(n) = s.parse::<i32>() => println!("sayı: {n}"),
            Some(s) => println!("metin: {s}"),
            None => println!("yok"),
        }
    }
}
```

Çıktı:

```text
sayı: 42
metin: kırk iki
yok
```

### `@` ile ad vermek

`n @ 1..=5`, "1 ile 5 arasında mısın diye bak, öyleyse bu değere n adını ver" demektir. `@` olmasaydı aralığı sınayabilir ama kolun içinde değerin kendisini kullanamazdınız; ya sınamayı ya adı kaybederdiniz:[^at]

```rust
fn main() {
    let id = 5;
    match id {
        n @ 1..=5 => println!("@ bağlama: {n} küçük aralıkta"),
        n @ 6..=10 => println!("@ bağlama: {n} büyük aralıkta"),
        _ => println!("@ bağlama: aralık dışı"),
    }

    let n = 42;
    let etiket = match n {
        1 | 2 => "bir ya da iki",
        3..=9 => "üç ile dokuz arası",
        k @ 10..=99 if k % 2 == 0 => "iki basamaklı çift",
        _ => "başka",
    };
    println!("{n}: {etiket}");
}
```

Çıktı:

```text
@ bağlama: 5 küçük aralıkta
42: iki basamaklı çift
```

İkinci `match` bölümün araçlarını bir arada kullanır: seçenek, aralık, `@` ve koruyucu koşul.

## Eksiksizlik: açıkta durum kalmasın

`match` bütün olası değerleri kapsamak zorundadır. Yukarıdaki `_` kolunu silelim:[^eksiksiz]

```rust,compile_fail,E0004
fn main() {
    let n = 42;
    let etiket = match n {
        1 | 2 => "bir ya da iki",
        3..=9 => "üç ile dokuz arası",
    };
    println!("{etiket}");
}
```

Derleyicinin yanıtı:

```text
error[E0004]: non-exhaustive patterns: `i32::MIN..=0_i32` and `10_i32..=i32::MAX` not covered
  |     let etiket = match n {
  |                        ^ patterns `i32::MIN..=0_i32` and `10_i32..=i32::MAX` not covered
  |
  = note: the matched value is of type `i32`
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern, a match arm with multiple or-patterns as shown, or multiple match arms
```

Derleyici açıkta kalan değerleri bile hesaplayıp söyledi. Böylece "bu durumu unutmuşum" hatası program çalışmadan yakalanır.

### Koruyucu koşullar eksiksizliğe sayılmaz

Aşağıdaki iki koşul birlikte her sayıyı kapsar, ama program derlenmez:[^koruyucu]

```rust,compile_fail,E0004
fn main() {
    let n: i32 = 42;
    let etiket = match n {
        x if x < 0 => "negatif",
        x if x >= 0 => "sıfır ya da pozitif",
    };
    println!("{etiket}");
}
```

Derleyicinin yanıtı:

```text
error[E0004]: non-exhaustive patterns: `i32::MIN..=i32::MAX` not covered
  |     let etiket = match n {
  |                        ^ pattern `i32::MIN..=i32::MAX` not covered
  |
  = note: the matched value is of type `i32`
  = note: match arms with guards don't count towards exhaustivity
```

Derleyici koşulun içine bakmaz, çünkü koşul herhangi bir ifade olabilir. Ya son kolu koşulsuz bırakın ya da koşul yerine aralık yazın; aralıkların her şeyi kapsadığını derleyici kanıtlayabilir:

```rust
fn main() {
    for n in [-3, 0, 42] {
        let etiket = match n {
            ..=-1 => "negatif",
            0.. => "sıfır ya da pozitif",
        };
        println!("{n}: {etiket}");
    }
}
```

Çıktı:

```text
-3: negatif
0: sıfır ya da pozitif
42: sıfır ya da pozitif
```

### Erişilemez kollar

Kolların sırası önemlidir. Üstteki bir kol her şeyi alıyorsa altındakine sıra gelmez; derleyici bunu uyarıyla bildirir:

```rust
fn main() {
    let n = 5;
    match n {
        0..=9 => println!("rakam"),
        5 => println!("beş"),
        _ => println!("başka"),
    }
}
```

Derleyici şu uyarıyı verir:

```text
warning: unreachable pattern
  |         0..=9 => println!("rakam"),
  |         ----- matches all the relevant values
  |         5 => println!("beş"),
  |         ^ no value can reach this
```

Dar kolu yukarı alırsanız uyarı gider. Uyarının adı `unreachable_patterns`'tır.[^unreachable-lint]

## Hata kodu başvurusu

| Kod | Ne zaman görülür? | Ne yapmalı? |
| --- | --- | --- |
| `E0308` | Desenin biçimi değerin türüyle uyuşmayınca; `let ... else` bloğu çıkmayınca | Parça sayısını eşitleyin; bloğu `return`, `break` ya da `panic!` ile bitirin |
| `E0005` | `let`, `for` ya da parametrede çürütülebilir desen yazılınca | `let ... else`, `if let` ya da `match` kullanın |
| `E0004` | `match` bütün durumları kapsamayınca; kollar yalnızca koruyucu koşulluysa | Eksik kolu ya da `_` kolunu ekleyin; koşul yerine aralık yazın |
| `E0027` | Struct deseninde bir alan atlanınca | Alanı yazın, `alan: _` ya da `..` ile yok sayın |
| `E0026` | Struct deseninde var olmayan bir alan yazılınca | Alan adını düzeltin |
| `E0382` | Desen bir parçayı taşıdıktan sonra bütün kullanılınca | Kullanmayacağınız parçaya `_` yazın ya da referans üzerinde eşleştirin |
| `E0408` | `\|` ile ayrılan seçenekler aynı adları bağlamayınca | Kolları ayırın ([18.1](18-patterns-deep-dive.md)) |

## Perde arkası: Derleyici ne yapıyor?

> [!NOTE]
> Bu bölüm derleyicinin iç işleyişini anlatır. İlk okuyuşta atlayabilirsiniz.

**Çürütülebilirlik derleme zamanında belirlenir.** Derleyici desenin, değerin *bütün* olası biçimlerine uyup uymadığını hesaplar. Uymayan bir biçim varsa desen çürütülebilirdir. `let` bir dalı atlayamayacağı için böyle bir deseni `E0005` ile reddeder ve `let ... else` önerir.[^irrefutable]

**Eksiksizlik ve erişilemezlik aynı hesabın iki yüzüdür.** Algoritmanın temel sorusu şudur: "Bu desen, kendinden öncekilerin tutmadığı en az bir değeri tutuyor mu?" Tutuyorsa *kullanışlıdır* (useful). Kullanışlı olmayan bir kol erişilemezdir ve uyarı üretir. Listenin sonuna hayalî bir `_` kolu eklenir; bu kol kullanışlı çıkarsa açıkta değer kalmış demektir ve `E0004` üretilir.[^usefulness][^e0004-kod] `_` ve `..`, "geri kalan her şey" dedikleri için bu hesabı tamamlar.

**Koruyucu koşullar bu hesabın dışındadır.** Derleyici bir koşulun ne zaman doğru olacağını bilemez. Koşullu bir kol bu yüzden "her zaman eşleşir" sayılmaz: Kendinden sonraki kolları erişilemez kılmaz, eksiksizliğe de katkı vermez.[^koruyucu-kod]

**Eşleştirme sıradan koda çevrilir.** Denetimden geçen `match`, derleyicinin ara gösterimi olan MIR'a bir karar ağacı olarak iner: varyant numarasına bak, alanı oku, karşılaştır, dallan.[^mir] `@`, bu ağaçta sınanan yerin değerini bir değişkene de yazmaktan ibarettir. Çalışan programda "desen" diye bir nesne yoktur; desenler tamamen derleme zamanı aracıdır.

## Özet

Desen, bir değerin biçimini sınar ve parçalarına ad verir. `match`, `if let`, `while let`, `let`, `for` ve parametreler aynı dil bilgisini kullanır. Her değere uyan desen çürütülemezdir; düz `let`, `for` ve parametreler yalnızca böyle desenleri kabul eder. Çürütülebilir desenler için `if let`, `let ... else`, `let` zincirleri ya da `match` kullanılır. Sözdiziminin araçları literaller, `|`, aralıklar, struct, enum ve demet desenleri, `_`, `..`, koruyucu koşul ve `@`'tir. Derleyici `match`'in eksiksiz olduğunu denetler, erişilemez kolları bildirir; koruyucu koşulları ise bu hesaba katmaz.

[18.1. bölümde](18-patterns-deep-dive.md) bu araçları küçük bir komut ayrıştırıcısında birleştirecek, dilim desenlerini, bağlama modlarını ve `matches!` makrosunu ele alacağız.

## Kaynaklar

[^desenler]: The Rust Programming Language, [19. Patterns and Matching](https://doc.rust-lang.org/book/ch19-00-patterns.html) ve The Rust Reference, [Patterns](https://doc.rust-lang.org/reference/patterns.html).
[^let]: The Rust Programming Language, [19.1: let Statements](https://doc.rust-lang.org/book/ch19-01-all-the-places-for-patterns.html#let-statements).
[^match]: The Rust Programming Language, [19.1: match Arms](https://doc.rust-lang.org/book/ch19-01-all-the-places-for-patterns.html#match-arms).
[^iflet]: The Rust Programming Language, [19.1: Conditional if let Expressions](https://doc.rust-lang.org/book/ch19-01-all-the-places-for-patterns.html#conditional-if-let-expressions).
[^whilelet]: The Rust Programming Language, [19.1: while let Conditional Loops](https://doc.rust-lang.org/book/ch19-01-all-the-places-for-patterns.html#while-let-conditional-loops).
[^for]: The Rust Programming Language, [19.1: for Loops](https://doc.rust-lang.org/book/ch19-01-all-the-places-for-patterns.html#for-loops).
[^parametre]: The Rust Programming Language, [19.1: Function Parameters](https://doc.rust-lang.org/book/ch19-01-all-the-places-for-patterns.html#function-parameters).
[^curutme]: The Rust Programming Language, [19.2 Refutability](https://doc.rust-lang.org/book/ch19-02-refutability.html) ve The Rust Reference, [Refutability](https://doc.rust-lang.org/reference/patterns.html#refutability).
[^letelse]: The Rust Reference, [let statements](https://doc.rust-lang.org/reference/statements.html#let-statements) ve The Rust Programming Language, [6.3: Staying on the Happy Path with let...else](https://doc.rust-lang.org/book/ch06-03-if-let.html#staying-on-the-happy-path-with-letelse).
[^letelse-surum]: Rust Blog, [Announcing Rust 1.65.0](https://blog.rust-lang.org/2022/11/03/Rust-1.65.0/).
[^never-surum]: Rust Blog, [Announcing Rust 1.100.0](https://blog.rust-lang.org/2026/11/12/Rust-1.100.0/).
[^letzincir]: The Rust Edition Guide, [Let chains in if and while](https://doc.rust-lang.org/edition-guide/rust-2024/let-chains.html); Rust Blog, [Announcing Rust 1.88.0](https://blog.rust-lang.org/2025/06/26/Rust-1.88.0/).
[^irrefutable-lint]: rustc 1.99.0 kaynak kodu, [`builtin.rs`, `IRREFUTABLE_LET_PATTERNS`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_lint_defs/src/builtin.rs#L2071).
[^literal]: The Rust Reference, [Literal patterns](https://doc.rust-lang.org/reference/patterns.html#literal-patterns).
[^veya]: The Rust Reference, [Or-patterns](https://doc.rust-lang.org/reference/patterns.html#or-patterns).
[^aralik]: The Rust Reference, [Range patterns](https://doc.rust-lang.org/reference/patterns.html#range-patterns).
[^aralik-surum]: Rust Blog, [Announcing Rust 1.80.0](https://blog.rust-lang.org/2024/07/25/Rust-1.80.0/).
[^adlar]: The Rust Programming Language, [19.3: Matching Named Variables](https://doc.rust-lang.org/book/ch19-03-pattern-syntax.html#matching-named-variables).
[^yol-deseni]: The Rust Reference, [Path patterns](https://doc.rust-lang.org/reference/patterns.html#path-patterns).
[^struct]: The Rust Reference, [Struct patterns](https://doc.rust-lang.org/reference/patterns.html#struct-patterns).
[^e0026]: Rust hata kodları, [E0026](https://doc.rust-lang.org/error_codes/E0026.html) ve [E0027](https://doc.rust-lang.org/error_codes/E0027.html).
[^enum]: The Rust Programming Language, [19.3: Enums](https://doc.rust-lang.org/book/ch19-03-pattern-syntax.html#enums).
[^joker]: The Rust Reference, [Wildcard pattern](https://doc.rust-lang.org/reference/patterns.html#wildcard-pattern).
[^kalan]: The Rust Reference, [Rest pattern](https://doc.rust-lang.org/reference/patterns.html#rest-pattern).
[^altcizgi]: The Rust Programming Language, [19.3: An Unused Variable by Starting Its Name with _](https://doc.rust-lang.org/book/ch19-03-pattern-syntax.html#an-unused-variable-by-starting-its-name-with-_).
[^koruyucu]: The Rust Reference, [Match guards](https://doc.rust-lang.org/reference/expressions/match-expr.html#match-guards).
[^iflet-guard]: Rust Blog, [Announcing Rust 1.95.0](https://blog.rust-lang.org/2026/04/16/Rust-1.95.0/) ve The Rust Reference, [Match guard chains](https://doc.rust-lang.org/reference/expressions/match-expr.html#match-guard-chains).
[^at]: The Rust Programming Language, [19.3: Using @ Bindings](https://doc.rust-lang.org/book/ch19-03-pattern-syntax.html#using--bindings).
[^eksiksiz]: The Rust Programming Language, [6.2: Matches Are Exhaustive](https://doc.rust-lang.org/book/ch06-02-match.html#matches-are-exhaustive).
[^unreachable-lint]: rustc 1.99.0 kaynak kodu, [`builtin.rs`, `UNREACHABLE_PATTERNS`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_lint_defs/src/builtin.rs#L898).
[^irrefutable]: rustc 1.99.0 kaynak kodu, [`check_match.rs`, `check_binding_is_irrefutable`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_mir_build/src/thir/pattern/check_match.rs#L665).
[^usefulness]: rustc 1.99.0 kaynak kodu, [`usefulness.rs`, `compute_match_usefulness`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_pattern_analysis/src/usefulness.rs#L1827); rustc geliştirme kılavuzu, [Pattern and exhaustiveness checking](https://rustc-dev-guide.rust-lang.org/pat-exhaustive-checking.html).
[^e0004-kod]: rustc 1.99.0 kaynak kodu, [`check_match.rs`, `report_non_exhaustive_match`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_mir_build/src/thir/pattern/check_match.rs#L1252).
[^koruyucu-kod]: rustc 1.99.0 kaynak kodu, [`usefulness.rs`, `is_under_guard`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_pattern_analysis/src/usefulness.rs#L1720).
[^mir]: rustc 1.99.0 kaynak kodu, [`matches/mod.rs`, `lower_match_tree`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_mir_build/src/builder/matches/mod.rs#L1568).
