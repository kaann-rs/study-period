# 5. Enum'lar ve Desen Eşleme

Programlarda sık sık "bu değer şunlardan biridir" demek gerekir. Postaneye gelen bir gönderi ya mektuptur ya kolidir ya da kartpostaldır; aynı anda ikisi birden olamaz, bunların dışında bir şey de olamaz. Struct bu durumu anlatmaya uygun değildir: Alanlarının hepsi bir arada bulunur, oysa burada seçeneklerden yalnızca biri geçerlidir.

Rust'ta bu işi *enum* (enumeration, numaralandırma) görür. Enum, olasılıkları tek bir türde toplar ve her olasılığın kendi verisini taşımasına izin verir.[^enum] Olasılıkları birbirinden ayıran `match` ise her biri için ne yapılacağını yazdırır; birini unutursanız program derlenmez. Böylece eksik bırakılan bir durum, çalışma zamanında ortaya çıkan bir hata olmaktan çıkıp derleme zamanında görülen bir hataya dönüşür.

Bu bölümde önce enum tanımlamayı, `match` ile dallanmayı ve derleyicinin eksiksizlik denetimini göreceğiz. Ardından null'ın yerini tutan `Option` türünü, son olarak da `if let` ve `let ... else` kısa yazımlarını ele alacağız.

> [!NOTE]
> Bu bölümün kodu `learn/enums` dizinindedir. Depo kökünden `cargo run -p enums` komutuyla çalıştırabilirsiniz.

## Günlük hayattan bir benzetme

Kapınıza gelen postacının elindeki gönderi ya mektup ya koli ya da kartpostaldır. Her cinsin kendi bilgisi vardır: Kolinin kilosu olur, mektubunki yazılmaz. *Enum* böyle bir "şunlardan biri" listesidir; listedeki seçeneklerin her birine *varyant* (variant) denir. Gönderinin akıbeti de böyle bir listedir: Zarf ya kutuya girer ya postacının cebinde kalır ya da geri götürülür. Aynı zarfın hem kutuda hem cepte olması imkânsızdır.

Postacının cebinde bir talimat kâğıdı vardır: "Mektupsa kutuya at, koliyse zili çal, geri kalan her şeyi şubeye götür." Postacı gönderiye bakar ve uyan ilk satırı uygular. `match` bu kâğıttır. Kâğıtta bir cinsin satırı eksikse postacı o cinsten ilk gönderide ne yapacağını bilemez. Neyse ki kâğıdı yola çıkmadan önce şube müdürü okur ve eksik satırı hemen fark eder; Rust'ta müdürün görevini derleyici üstlenir.

Kapının önündeki posta kutusunda ise ya bir mektup vardır ya da kutu boştur; kapağı açıp bakmadan mektubu okuyamazsınız. `Option` bu kutudur: ya dolu (`Some`) ya boş (`None`).

<details>
<summary>Bu bölümde geçen terimler</summary>

- **enum:** Seçenek listesi olan tür. Değeri, seçeneklerden yalnızca biridir.
- **varyant (variant):** Listedeki seçeneklerden her biri; örneğin `Yon::Kuzey`.
- **unit varyant:** Veri taşımayan varyant: `Cik`.
- **demet benzeri varyant (tuple-like):** Adsız veriler taşıyan varyant: `V4(127, 0, 0, 1)`.
- **struct benzeri varyant (struct-like):** Adlı veriler taşıyan varyant: `Tasi { x: 1, y: 2 }`.
- **`Option<T>`:** "İçinde bir `T` olabilir de olmayabilir de" diyen hazır enum.
- **`Some(x)`, `None`:** `Option`'ın iki varyantı: "var, işte `x`" ve "yok".
- **null:** Pek çok dilde "boş" anlamına gelen değer. Rust'ta yerini `None` tutar.
- **`match`:** Bir değeri desenlerle karşılaştırıp uyan ilk kolu çalıştıran ifade.
- **kol (arm):** `match` içindeki her `desen => yapılacak iş` satırı.
- **desen (pattern):** Değerin uyması beklenen kalıp: `Some(n)`, `3`, `_`.
- **bağlama (binding):** Desenin yakaladığı veriye isim vermek; `Some(n)` içindeki `n`.
- **geri kalan her şey kolu (catch-all):** Önceki kolların yakalamadığı bütün değerleri karşılayan kol. `_`, değerin kullanılmayacağını belirtir.
- **eksiksizlik denetimi (exhaustiveness):** Derleyicinin "her olasılık karşılanmış mı?" denetimi.
- **`if let`, `let ... else`:** Tek bir desenle ilgilenirken `match` yerine kullanılan kısa yazımlar.
- **`unwrap_or(y)`:** `Option` doluysa içindekini, boşsa `y`'yi veren metot.
- **`format!`:** `println!` gibi çalışan, ama ekrana yazmak yerine `String` üreten makro.
- **jenerik (generic), `<T>`:** "Herhangi bir tür" için yer tutucu: `Option<i32>`, `Option<String>`.
- **`Box<T>`:** Heap'te duran bir değerin sahibi olan işaretçi.
- **discriminant ve etiket (tag):** Değerin hangi varyant olduğunu belirleyen sayı ve onun bellekteki karşılığı.
- **kısa yazım (sugar):** Aynı işi daha az kodla yazdıran sözdizimi.
- **yeniden düzenleme (refactor):** Kodun davranışını değiştirmeden yapısını iyileştirmek.

</details>

## Enum tanımlamak

`enum` anahtar sözcüğünden sonra türün adı, süslü parantezlerin içinde de varyantlar yazılır:

```rust
#[derive(Debug)]
enum Yon {
    Kuzey,
    Guney,
    Dogu,
    Bati,
}

fn main() {
    let y = Yon::Dogu;
    println!("yön = {y:?}");
}
```

Çıktı:

```text
yön = Dogu
```

Bir `Yon` değeri bu dört varyanttan tam olarak biridir. Varyantlar enum'un adı altında durur; bu yüzden `Dogu` değil `Yon::Dogu` yazılır.[^enum] `#[derive(Debug)]` satırını [struct'lar bölümünden](04-structs.md) tanıyorsunuz: Değeri `{:?}` ile yazdırmayı sağlar.

### Varyantlar veri taşıyabilir

Gönderileri önce bir struct ile anlatmayı deneyelim:

```rust
struct GonderiKarti {
    cins: u8,       // 0: mektup, 1: koli, 2: kartpostal
    kilo: u32,      // mektupta buraya ne yazılacak?
    kimden: String, // kolide buraya ne yazılacak?
}
# fn main() {}
```

Her kartta bütün alanlar bulunur, hangisinin geçerli olduğunu `cins` bayrağı söyler. Mektubun kilo alanına ne yazılacağı belirsizdir; bayrağa bakmayı unutan bir kod o anlamsız kiloyu okuyup yanlış iş yapabilir. `cins: 7` gibi karşılığı olmayan bir değeri engelleyen de yoktur. Enum'da ise her veri yalnızca ait olduğu varyantın yanında bulunur:

```rust
#[derive(Debug)]
enum Gonderi {
    Mektup,                        // veri taşımaz
    Koli(u32),                     // kilosunu taşır
    Kartpostal { kimden: String }, // adlı bir bilgi taşır
}

fn main() {
    let gonderiler = [
        Gonderi::Mektup,
        Gonderi::Koli(3),
        Gonderi::Kartpostal { kimden: String::from("Ada") },
    ];
    println!("{gonderiler:?}");
}
```

Çıktı:

```text
[Mektup, Koli(3), Kartpostal { kimden: "Ada" }]
```

Varyantlar üç biçimde yazılabilir ve aynı enum'da yan yana bulunabilir:[^bicim]

| Biçim | Örnek tanım | Örnek değer |
| --- | --- | --- |
| Unit varyant: veri taşımaz | `Mektup` | `Gonderi::Mektup` |
| Demet benzeri varyant: adsız veriler taşır | `Koli(u32)` | `Gonderi::Koli(3)` |
| Struct benzeri varyant: adlı veriler taşır | `Kartpostal { kimden: String }` | `Gonderi::Kartpostal { kimden: String::from("Ada") }` |

Üçü de aynı türdendir; bir dizide yan yana durabilmelerinin ve aynı fonksiyona verilebilmelerinin nedeni budur. Üç ayrı struct tanımlasaydınız ortada üç ayrı tür olur, hepsini birden kabul eden bir fonksiyon yazmak zorlaşırdı.

Varyant, tanımında yazmayan bir veriyi kabul etmez. Mektubun kilosu yoktur:

```rust,compile_fail,E0618
enum Gonderi {
    Mektup,
    Koli(u32),
}

fn main() {
    let g = Gonderi::Koli(3);
    let h = Gonderi::Mektup(3);
}
```

Derleyicinin yanıtı:

```text
error[E0618]: expected function, found `Gonderi`
  |
2 |     Mektup,
  |     ------ `Gonderi::Mektup` defined here
...
8 |     let h = Gonderi::Mektup(3);
  |             ^^^^^^^^^^^^^^^---
  |             |
  |             call expression requires function
```

Mesajın "fonksiyon" sözcüğünü kullanması rastlantı değildir: Demet benzeri bir varyantın adı, o varyantı kuran bir fonksiyondur. `Gonderi::Koli`, `u32` alıp `Gonderi` döndürür; veri taşımayan `Gonderi::Mektup` ise doğrudan bir değerdir.[^enum]

## `match` ile dallanmak

Bir enum değerinin hangi varyant olduğunu `match` ile öğrenirsiniz. `match` bir değeri sırayla desenlerle karşılaştırır ve uyan **ilk** kolu çalıştırır.[^match] Desen enum varyantı olmak zorunda değildir; önce sayılarla deneyelim:

```rust
fn main() {
    let zar = 6;
    let sonuc = match zar {
        6 => "kazandın",
        1 => "kaybettin",
        _ => "tekrar at", // geri kalan her sayı
    };
    println!("{sonuc}");
}
```

Çıktı:

```text
kazandın
```

`match` bir ifadedir: Seçilen kolun ürettiği değer `match`'in değeri olur ve doğrudan `sonuc` değişkenine konur. Bu yüzden bütün kollar aynı türde bir değer üretmelidir:

```rust,compile_fail,E0308
let n = 4;
let paritesi = match n % 2 {
    0 => "çift",
    _ => 1,
};
```

Derleyicinin yanıtı:

```text
error[E0308]: `match` arms have incompatible types
```

### Desenle veri çıkarmak

Varyantın taşıdığı veriye de desen ile ulaşılır. Desenin içine yazılan ad, oradaki veriye *bağlanır* ve yalnızca o kolda geçerlidir:[^baglama]

```rust
#[derive(Debug)]
enum IpAdres {
    V4(u8, u8, u8, u8),
    V6(String),
    Bilinmeyen,
}

fn main() {
    let adresler = [
        IpAdres::V4(127, 0, 0, 1),
        IpAdres::V6(String::from("::1")),
        IpAdres::Bilinmeyen,
    ];
    for a in adresler {
        let aciklama = match a {
            IpAdres::V4(a, b, c, d) => format!("IPv4 {a}.{b}.{c}.{d}"),
            IpAdres::V6(s) => format!("IPv6 {s}"),
            IpAdres::Bilinmeyen => String::from("bilinmeyen"),
        };
        println!("{aciklama}");
    }
}
```

Çıktı:

```text
IPv4 127.0.0.1
IPv6 ::1
bilinmeyen
```

`IpAdres` her varyantında farklı sayıda ve türde veri taşır: V4 dört küçük sayı, V6 bir metin, üçüncüsü hiçbir şey. Bir struct bunu yapamazdı. (Standart kütüphanede bu iş için hazır bir `IpAddr` enum'u vardır.) İlk koldaki `a`, döngü değişkeni olan `a`'yı o kol boyunca gölgeler; desendeki adlar her zaman yeni bağlamalardır.

### Geri kalan her şey

Olasılıkların hepsini tek tek yazmak gerekmeyebilir. Son kola bir ad yazarsanız önceki kolların yakalamadığı bütün değerler o ada bağlanır. Değere ihtiyacınız yoksa ad yerine `_` yazarsınız; `_` hiçbir şeye bağlanmaz:[^catch-all]

```rust
fn main() {
    let zar = 9;
    match zar {
        3 => println!("üç!"),
        7 => println!("yedi!"),
        diger => println!("diğer: {diger}"), // değeri bağlar
    }

    let n = 4;
    let paritesi = match n % 2 {
        0 => "çift",
        _ => "tek", // 0 dışındaki her şey
    };
    println!("{n} sayısı {paritesi}");
}
```

Çıktı:

```text
diğer: 9
4 sayısı çift
```

Kollar yazıldıkları sırayla denendiği için bu kol en sonda durmalıdır. Daha yukarıda olsaydı altındaki kollara sıra hiç gelmezdi; derleyici böyle kollar için `unreachable pattern` uyarısı verir.

### Enum'lara metot eklemek

Struct'lar gibi enum'lara da `impl` ile metot eklenir. Metodun gövdesi çoğunlukla `self` üzerinde bir `match`'tir:[^metot]

```rust
enum Mesaj {
    Cik,                     // unit
    Tasi { x: i32, y: i32 }, // struct benzeri
    Yaz(String),             // demet benzeri
    RenkDegis(u8, u8, u8),
}

impl Mesaj {
    fn calistir(&self) {
        match self {
            Mesaj::Cik => println!("çık"),
            Mesaj::Tasi { x, y } => println!("taşı {x},{y}"),
            Mesaj::Yaz(s) => println!("yaz: {s}"),
            Mesaj::RenkDegis(r, g, b) => println!("renk {r},{g},{b}"),
        }
    }
}

fn main() {
    for m in [
        Mesaj::Cik,
        Mesaj::Tasi { x: 1, y: 2 },
        Mesaj::Yaz(String::from("selam")),
        Mesaj::RenkDegis(255, 0, 0),
    ] {
        m.calistir();
    }
}
```

Çıktı:

```text
çık
taşı 1,2
yaz: selam
renk 255,0,0
```

`calistir` metodu `&self` aldığı için `match` bir referans üzerinde çalışır. Desenin bağladığı adlar da bu yüzden referanstır: `s` burada `&String`'dir, metin enum'un içinden çıkarılmaz.[^binding-mode]

## Eksiksizlik denetimi

`match`'in kolları bütün olasılıkları karşılamak zorundadır. Aşağıdaki kod dört varyantı da karşıladığı için derlenir:

```rust
# enum Yon { Kuzey, Guney, Dogu, Bati }
# fn main() {
for y in [Yon::Kuzey, Yon::Guney, Yon::Dogu, Yon::Bati] {
    let ok = match y {
        Yon::Kuzey => "↑",
        Yon::Guney => "↓",
        Yon::Dogu => "→",
        Yon::Bati => "←",
    };
    print!("{ok} ");
}
println!();
# }
```

Çıktı:

```text
↑ ↓ → ←
```

İki varyantı atlarsanız derleyici hangilerini unuttuğunuzu adıyla söyler:

```rust,compile_fail,E0004
enum Yon {
    Kuzey,
    Guney,
    Dogu,
    Bati,
}

fn ok(y: Yon) -> &'static str {
    match y {
        Yon::Kuzey => "↑",
        Yon::Guney => "↓",
    }
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0004]: non-exhaustive patterns: `Yon::Dogu` and `Yon::Bati` not covered
  |
9 |     match y {
  |           ^ patterns `Yon::Dogu` and `Yon::Bati` not covered
  |
note: `Yon` defined here
```

Uzun bir `if`/`else` zincirinde bir olasılığı atlamak kolaydır. C'deki `switch` deyiminde unutulan bir `case` de hata sayılmaz: Değer sessizce geçip gider ya da istenmeyen bir `default` dalına düşer. Rust'ta ise atlanan olasılık, program çalışmadan önce karşınıza çıkar.[^eksiksiz]

Denetimin asıl değeri kod değiştiğinde anlaşılır. Aylar sonra gönderi cinslerine `Dergi` eklediğinizi düşünün. Bu cinsi unutan her `match`, derleyici hatası olarak önünüze gelir:

```rust,compile_fail,E0004
enum Gonderi {
    Mektup,
    Koli,
    Dergi, // yeni eklendi
}

fn ne_yapmali(g: Gonderi) -> &'static str {
    match g {
        Gonderi::Mektup => "kutuya at",
        Gonderi::Koli => "zili çal",
    }
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0004]: non-exhaustive patterns: `Gonderi::Dergi` not covered
```

Bu çalışma biçimine *derleyici güdümlü yeniden düzenleme* (compiler-driven refactoring) denir: Türü değiştirir, ardından derleyicinin gösterdiği yerleri tek tek düzeltirsiniz.

> [!WARNING]
> `_ =>` kolu bu korumayı kaldırır. Yeni varyant sessizce "geri kalan her şey" arasına karışır ve hiçbir hata görmezsiniz. Kendi enum'larınızda varyantları tek tek yazmak çoğu zaman daha güvenlidir; hangi durumda hangisinin seçileceğini [derinleşme sayfasında](05-enums-deep-dive.md) tartışacağız.

## `Option`: Değer olmayabilir

Pek çok dilde bir değişken "boş" anlamına gelen null değerini tutabilir. Sorun, null olabilen değerle olamayanın aynı türde görünmesidir: Denetlemeyi unutursanız program çalışırken çöker. Null referansın mucidi Tony Hoare, 2009'daki bir konuşmasında buluşunu "milyar dolarlık hata" diye andı.[^option]

Rust'ın referanslarında null yoktur. Bir değerin olmayabileceği, standart kütüphanedeki `Option` enum'u ile **türün içine** yazılır:

```rust
enum Option<T> {
    None,
    Some(T),
}
# fn main() {}
```

Tanımdaki `<T>`, "herhangi bir tür" demektir: `Option<i32>` içinde bir tam sayı, `Option<String>` içinde bir metin olabilir. Bu tanımı sizin yazmanız gerekmez. `Option` o kadar sık kullanılır ki varyantlarıyla birlikte her modülde hazır bulunur; `Option::` öneki koymadan `Some` ve `None` yazabilirsiniz.[^option]

```rust
fn main() {
    let bazi: Option<i32> = Some(5);
    let yok: Option<i32> = None;

    let sonuc = match bazi {
        Some(n) => n + 1,
        None => 0,
    };
    println!("sonuç = {sonuc}, yok = {yok:?}");
    println!("yok.unwrap_or(-1) = {}", yok.unwrap_or(-1));
}
```

Çıktı:

```text
sonuç = 6, yok = None
yok.unwrap_or(-1) = -1
```

`match` iki ihtimali de karşılar: Kutu doluysa içindeki sayı `n`'ye bağlanır, boşsa yedek sonuç kullanılır. Aynı işi yapan kısa bir metot da vardır: `unwrap_or`, doluysa içindekini, boşsa verdiğiniz yedek değeri döndürür.[^unwrap-or]

`None` tek başına yazıldığında derleyici kutuda hangi türün olabileceğini bilemez. `let yok = None;` satırı, türü başka bir yerden çıkarılamıyorsa `E0282` hatası verir; yukarıda türü açıkça yazmamızın nedeni budur.

### Kutuyu açmadan kullanamazsınız

`Option<i32>` ile `i32` ayrı türlerdir. Kutuyu sayı gibi toplamaya kalkarsanız program derlenmez:

```rust,compile_fail,E0369
let kutu: Option<i32> = Some(5);
let a = kutu + 1;
```

Derleyicinin yanıtı:

```text
error[E0369]: cannot add `{integer}` to `Option<i32>`
```

Toplamaya geçmeden önce `None` ihtimalini ele almak zorundasınız. `unwrap_or` bunun en kısa yoludur:

```rust
let kutu: Option<i32> = Some(5);
let bos: Option<i32> = None;

let b = kutu.unwrap_or(0) + 1; // doluysa içindeki, boşsa 0
let c = bos.unwrap_or(0) + 1;
println!("{b} {c}");
```

Çıktı:

```text
6 1
```

`unwrap_or` kutuyu tüketir; buna karşın `kutu` sonraki satırlarda da kullanılabilir. Çünkü `Option<T>`, içindeki tür `Copy` ise kendisi de `Copy`'dir.[^option-copy] `Option<String>` için bu geçerli değildir; farkın yol açtığı hatayı derinleşme sayfasında göreceğiz.

Yalnızca dolu ihtimali yazmak da yetmez; eksiksizlik denetimi `Option` için de geçerlidir:

```rust,compile_fail,E0004
let kutu: Option<i32> = Some(5);
let sonuc = match kutu {
    Some(n) => n + 1,
};
```

Derleyicinin yanıtı:

```text
error[E0004]: non-exhaustive patterns: `None` not covered
```

Kazanç iki yönlüdür. Türü `Option` olan bir değerde yokluk ihtimalini unutamazsınız; türü `Option` olmayan bir değerde ise o ihtimali düşünmeniz gerekmez. Null denetimi çalışma zamanından derleme zamanına taşınmış olur.

> [!NOTE]
> "Null yoktur" sözü referanslar ve `Box` gibi güvenli işaretçiler için geçerlidir. Ham işaretçiler (`*const T`, `*mut T`) null olabilir; ama gösterdikleri yere erişmek `unsafe` gerektirir.[^ham]

## `if let` ve `let ... else`

Bazen tek bir ihtimal sizi ilgilendirir: Kutuda mektup varsa okuyacak, yoksa hiçbir şey yapmayacaksınız. `match` ile yazınca boş kol gürültü yaratır:

```rust
let ayar: Option<u8> = Some(3);
match ayar {
    Some(v) => println!("ayar = {v}"),
    _ => (),
}
```

Çıktı:

```text
ayar = 3
```

`if let` aynı işi boş kol yazdırmadan görür. Desen tutarsa blok çalışır; tutmazsa atlanır ya da varsa `else` bloğu çalışır:[^if-let]

```rust
let ayar: Option<u8> = Some(3);

if let Some(v) = ayar {
    println!("ayar = {v}");
} else {
    println!("ayar yok");
}
```

Çıktı:

```text
ayar = 3
```

`let ... else` ise soruyu tersinden sorar: "Kutu doluysa içindekini al ve devam et; boşsa buradan çık." Desen tutarsa bağlanan ad sonraki satırlarda sıradan bir değişken gibi kullanılır. Böylece asıl iş iç içe bloklara gömülmez:[^let-else]

```rust
fn main() {
    ayari_yaz(Some(3));
    ayari_yaz(None);
}

fn ayari_yaz(ayar: Option<u8>) {
    let Some(v) = ayar else {
        println!("değer yok, çık");
        return;
    };
    println!("let-else ile v = {v}");
}
```

Çıktı:

```text
let-else ile v = 3
değer yok, çık
```

`else` bloğunun akışı geri döndürmemesi gerekir: `return`, `break`, `continue` ya da `panic!` gibi bir çıkışla bitmelidir. Yalnızca bir şey yazdırıp devam edebilseydiniz `v`'nin değeri belirsiz kalırdı; derleyici buna izin vermez:

```rust,compile_fail,E0308
let ayar: Option<u8> = Some(3);
let Some(v) = ayar else {
    println!("değer yok");
};
println!("{v}");
```

Derleyicinin yanıtı:

```text
error[E0308]: `else` clause of `let...else` does not diverge
```

`else` bloğunu hiç yazmazsanız bu kez `E0005` görürsünüz. Düz bir `let`, desenin her durumda tutmasını ister; `Some(v)` ise kutu boşken tutmaz:

```rust,compile_fail,E0005
let ayar: Option<u8> = Some(3);
let Some(v) = ayar;
```

Derleyicinin yanıtı:

```text
error[E0005]: refutable pattern in local binding
```

Kısa yazımların bir bedeli vardır. Değere denetlemeden ulaşmanızı yine engellerler, ama **eksiksizlik denetiminden vazgeçersiniz**: Enum'a yeni bir varyant eklendiğinde `if let` sizi uyarmaz, yeni varyant `else` tarafına düşer.[^if-let] Bütün olasılıkların ele alınması önemliyse `match`, gerçekten tek bir olasılıkla ilgileniyorsanız kısa yazım uygundur.

> [!TIP]
> 2024 edition'ından itibaren `if let` koşulları `&&` ile zincirlenebilir: `if let Some(v) = ayar && v > 2 { ... }`. Bu yazım Rust 1.88'de kararlı oldu.[^zincir] Desen dilinin tamamını, `while let` ile birlikte [18. bölümde](../intermediate/18-patterns.md) göreceğiz.

## Hata kodu başvurusu

| Kod | Ne zaman görülür? | Ne yapmalı? |
| --- | --- | --- |
| `E0004` | `match` bir olasılığı karşılamadığında | Eksik varyantları yazın ya da bilerek `_` kolu ekleyin. |
| `E0005` | Düz `let`, her durumda tutmayan bir desenle kullanıldığında | `let ... else`, `if let` ya da `match` kullanın. |
| `E0282` | `None` gibi bir değerin türü çıkarılamadığında | Türü yazın: `let yok: Option<i32> = None;`. |
| `E0308` | Kollar farklı türler ürettiğinde, `let ... else` bloğu akıştan çıkmadığında ya da kutu içindeki değer sanıldığında | Kolları aynı türe getirin, bloğu `return` gibi bir çıkışla bitirin, kutuyu açın. |
| `E0369` | `Option` ile aritmetik yapıldığında | Önce `match`, `map` ya da `unwrap_or` ile kutuyu ele alın. |
| `E0618` | Veri taşımayan varyant fonksiyon gibi çağrıldığında | Parantezi kaldırın ya da varyantın tanımına veri ekleyin. |

## Perde arkası: Derleyici ne yapıyor?

> [!NOTE]
> Bu başlık ilk okuyuşta atlanabilir; program yazmak için yukarıdaki başlıklar yeterlidir.

Her gönderinin üstünde cinsini söyleyen küçük bir etiket bulunduğunu düşünün; postacı önce ona bakar. Enum'ların bellekteki düzeni buna benzer.

**Discriminant ve etiket.** Her enum değerinin, hangi varyant olduğunu belirleyen mantıksal bir tam sayısı vardır; buna *discriminant* denir.[^discr] Bu bilgi bellekte genellikle bir *etiket* (tag) olarak saklanır ve etiketin yanında en büyük varyanta yetecek kadar yer ayrılır. Böyle bir düzene *etiketli birleşim* (tagged union) denir. `match`, discriminant'ı okuyup ilgili kola geçer. Yine de bu düzen bir söz değildir: Derleyici, varyantları ayırt etmenin başka bir yolunu bulursa onu kullanabilir.[^layout]

**Kullanılmayan bit desenleri (niche).** Bazı türler, bellekte kapladıkları yerin alabileceği değerlerin hepsini kullanmaz. Örneğin bir referans hiçbir zaman sıfır olamaz. Derleyici bu boş desenlerden birini `None` için kullanır ve ayrı bir etikete yer harcamaz. Ölçelim:

```rust
fn main() {
    println!("i32 = {}, Option<i32> = {}", size_of::<i32>(), size_of::<Option<i32>>());
    println!("&i32 = {}, Option<&i32> = {}", size_of::<&i32>(), size_of::<Option<&i32>>());
    println!(
        "Box<i32> = {}, Option<Box<i32>> = {}",
        size_of::<Box<i32>>(),
        size_of::<Option<Box<i32>>>()
    );
    println!("bool = {}, Option<bool> = {}", size_of::<bool>(), size_of::<Option<bool>>());
}
```

Çıktı:

```text
i32 = 4, Option<i32> = 8
&i32 = 8, Option<&i32> = 8
Box<i32> = 8, Option<Box<i32>> = 8
bool = 1, Option<bool> = 1
```

Bu sayılar rustc 1.99.0 ile 64 bitlik (x86-64) bir makinede ölçülmüştür. Satır satır nedenleri şöyledir:

- `i32` bütün bit desenlerini kullanır, boş desen kalmaz. Ayrı bir etiket gerekir; dört baytlık hizalama da toplamı sekize çıkarır.
- `&i32` ile `Box<i32>` sıfır olamaz. `None` sıfır olarak saklanır; "null olabilen işaretçi" kadar ucuz, ama güvenli bir tür elde edilir.
- `bool` bir baytın yalnızca 0 ve 1 değerlerini kullanır. Geriye kalan 254 geçersiz desenden biri `None` için ayrılır.

`&T`, `&mut T`, `Box<T>`, fonksiyon işaretçileri, `NonNull<T>` ve `NonZero` türleri için `Option<T>`'nin `T` ile aynı boyutta olması belgelenmiş bir güvencedir ve *null işaretçi eniyilemesi* (null pointer optimization) diye anılır. `Option<bool>` bu listede yoktur; onun bir bayt kalması bugünkü derleyicinin davranışıdır.[^npo] Güvence verilmeyen düzenler sürümden sürüme değişebilir: Rust 1.97'de bazı enum'ların kodlaması değişmiştir.[^surum] Derleyicinin içinde iki durum `TagEncoding` enum'unun `Direct` ve `Niche` varyantlarıyla ayrılır.[^tag]

**Eksiksizlik denetimi.** Denetim, tür denetiminden sonra kodun THIR (*Typed High-Level Intermediate Representation*) denen ara biçimi üzerinde yapılır; eksik bir olasılık bulunursa `E0004` burada üretilir.[^e0004] Hesabın kendisi `rustc_pattern_analysis` crate'indeki "yararlılık" (usefulness) algoritmasıdır: Yazılmış kolların altına bir `_` kolu eklemek yeni bir değer yakalayacak olsaydı `match` eksiktir. Aynı algoritma, hiçbir zaman seçilmeyecek kolları da bulur.[^yararlilik]

**Kısa yazımlar neye dönüşür?** Anlamca `match`'e denk olsalar da `if let` ve `let ... else`, HIR'da (*High-level Intermediate Representation*) `match`'e açılmaz: `if let`, koşulunda `Let` ifadesi bulunan bir `If` düğümü olarak,[^hir-let] `let ... else` ise `else` bloğu taşıyan bir `let` deyimi olarak kalır.[^hir-else] `while let` bir `loop` ile içindeki `if let`'e dönüşür.[^hir-while] `for` döngüsünün `loop` ile `match`'e, `?` operatörünün de `match`'e açılması ise gerçekten yapılır.[^hir-for]

## Özet

- Enum, "şunlardan biri" demenin yoludur; her varyant kendi verisini taşır ve hepsi tek bir türdür.
- `match` uyan ilk kolu çalıştıran bir ifadedir. Kollar bütün olasılıkları karşılamalıdır; unutulan varyant `E0004` ile derleme zamanında yakalanır.
- `Option<T>`, yokluk ihtimalini türe yazar. Kutuyu açmadan içindekine ulaşamazsınız.
- `if let` ve `let ... else` tek bir olasılık için kısa yazımlardır; karşılığında eksiksizlik denetiminden vazgeçilir.

Sıradaki sayfa [5.1 Enum'lar: Derinleşme](05-enums-deep-dive.md), bu araçlarla tasarım yapmayı gösteriyor: geçersiz durumları yazılamaz kılmak, bir sipariş durum makinesi, `Option` ve `Result` kombinatörleri ve en sık karşılaşılan altı hata.

## Kaynaklar

[^enum]: The Rust Programming Language, [6.1 Defining an Enum](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html) ve [Enum Values](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html#enum-values); hata dizini, [`E0618`](https://doc.rust-lang.org/error_codes/E0618.html).
[^bicim]: The Rust Reference, [Enumerations](https://doc.rust-lang.org/reference/items/enumerations.html#enumerations); Rust by Example, [Enums](https://doc.rust-lang.org/rust-by-example/custom_types/enum.html).
[^match]: The Rust Programming Language, [6.2 The match Control Flow Construct](https://doc.rust-lang.org/book/ch06-02-match.html); The Rust Reference, [`match` expressions](https://doc.rust-lang.org/reference/expressions/match-expr.html#match-expressions); hata dizini, [`E0308`](https://doc.rust-lang.org/error_codes/E0308.html).
[^baglama]: The Rust Programming Language, [Patterns That Bind to Values](https://doc.rust-lang.org/book/ch06-02-match.html#patterns-that-bind-to-values); std, [`IpAddr`](https://doc.rust-lang.org/std/net/enum.IpAddr.html).
[^catch-all]: The Rust Programming Language, [Catch-All Patterns and the _ Placeholder](https://doc.rust-lang.org/book/ch06-02-match.html#catch-all-patterns-and-the-_-placeholder); The Rust Reference, [Wildcard pattern](https://doc.rust-lang.org/reference/patterns.html#wildcard-pattern).
[^metot]: The Rust Programming Language, [Enum Values](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html#enum-values).
[^binding-mode]: The Rust Reference, [Binding modes](https://doc.rust-lang.org/reference/patterns.html#binding-modes).
[^eksiksiz]: The Rust Programming Language, [Matches Are Exhaustive](https://doc.rust-lang.org/book/ch06-02-match.html#matches-are-exhaustive); hata dizini, [`E0004`](https://doc.rust-lang.org/error_codes/E0004.html).
[^option]: The Rust Programming Language, [The Option Enum](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html#the-option-enum); std, [`Option`](https://doc.rust-lang.org/std/option/enum.Option.html).
[^unwrap-or]: std, [`Option::unwrap_or`](https://doc.rust-lang.org/std/option/enum.Option.html#method.unwrap_or); The Rust Programming Language, [The Option&lt;T&gt; match Pattern](https://doc.rust-lang.org/book/ch06-02-match.html#the-optiont-match-pattern); hata dizini, [`E0282`](https://doc.rust-lang.org/error_codes/E0282.html) ve [`E0369`](https://doc.rust-lang.org/error_codes/E0369.html).
[^option-copy]: std, [`Option`: `Copy` uygulaması](https://doc.rust-lang.org/std/option/enum.Option.html#impl-Copy-for-Option%3CT%3E).
[^ham]: std, [Options and pointers ("nullable" pointers)](https://doc.rust-lang.org/std/option/index.html#options-and-pointers-nullable-pointers) ve [`ptr::null`](https://doc.rust-lang.org/std/ptr/fn.null.html).
[^if-let]: The Rust Programming Language, [6.3 Concise Control Flow with if let and let...else](https://doc.rust-lang.org/book/ch06-03-if-let.html#concise-control-flow-with-if-let-and-letelse).
[^let-else]: The Rust Programming Language, [Staying on the "Happy Path" with let...else](https://doc.rust-lang.org/book/ch06-03-if-let.html#staying-on-the-happy-path-with-letelse); The Rust Reference, [`let` statements](https://doc.rust-lang.org/reference/statements.html#let-statements); hata dizini, [`E0005`](https://doc.rust-lang.org/error_codes/E0005.html).
[^zincir]: The Rust Reference, [Chains of conditions](https://doc.rust-lang.org/reference/expressions/if-expr.html#chains-of-conditions); [Rust sürüm notları, 1.88.0](https://github.com/rust-lang/rust/blob/1.99.0/RELEASES.md).
[^discr]: The Rust Reference, [Discriminants](https://doc.rust-lang.org/reference/items/enumerations.html#discriminants).
[^layout]: The Rust Reference, [The Rust representation](https://doc.rust-lang.org/reference/type-layout.html#the-rust-representation); rustc 1.99.0 kaynak kodu, [`rustc_abi/src/lib.rs`, `Variants`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_abi/src/lib.rs#L1970).
[^npo]: std, [`Option`: Representation](https://doc.rust-lang.org/std/option/index.html#representation).
[^surum]: [Rust sürüm notları, 1.97.0](https://github.com/rust-lang/rust/blob/1.99.0/RELEASES.md), "Compatibility Notes" başlığı.
[^tag]: rustc 1.99.0 kaynak kodu, [`rustc_abi/src/lib.rs`, `TagEncoding`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_abi/src/lib.rs#L1997).
[^e0004]: rustc 1.99.0 kaynak kodu, [`check_match.rs`, `report_non_exhaustive_match`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_mir_build/src/thir/pattern/check_match.rs#L1252); rustc geliştirme kılavuzu, [The THIR](https://rustc-dev-guide.rust-lang.org/thir.html).
[^yararlilik]: rustc geliştirme kılavuzu, [Pattern and exhaustiveness checking](https://rustc-dev-guide.rust-lang.org/pat-exhaustive-checking.html); rustc 1.99.0 kaynak kodu, [`usefulness.rs`, `compute_match_usefulness`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_pattern_analysis/src/usefulness.rs#L1827).
[^hir-let]: rustc 1.99.0 kaynak kodu, [`rustc_ast_lowering/src/expr.rs`, `hir::ExprKind::Let`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_ast_lowering/src/expr.rs#L268) ve [`lower_expr_if`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_ast_lowering/src/expr.rs#L606).
[^hir-else]: rustc 1.99.0 kaynak kodu, [`rustc_ast_lowering/src/block.rs`, `LocalKind::InitElse`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_ast_lowering/src/block.rs#L100).
[^hir-while]: rustc 1.99.0 kaynak kodu, [`rustc_ast_lowering/src/expr.rs`, `lower_expr_while_in_loop_scope`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_ast_lowering/src/expr.rs#L641).
[^hir-for]: rustc 1.99.0 kaynak kodu, [`rustc_ast_lowering/src/expr.rs`, `lower_expr_for`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_ast_lowering/src/expr.rs#L1707) ve [`lower_expr_try`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_ast_lowering/src/expr.rs#L1883); rustc geliştirme kılavuzu, [AST lowering](https://rustc-dev-guide.rust-lang.org/hir/lowering.html).
