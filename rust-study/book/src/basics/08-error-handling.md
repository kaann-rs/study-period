# 8. Hata Yönetimi

Programlar kusursuz bir dünyada çalışmaz. Açılmak istenen dosya yerinde olmayabilir, kullanıcı sayı yerine harf yazabilir, ağ bağlantısı kopabilir; bunlara ek olarak programın kendisinde de yanlış olabilir. Pek çok dil bütün bu durumları tek bir mekanizmayla, *istisna* (exception) fırlatarak bildirir. Fonksiyonun hangi istisnaları fırlatabileceği çoğu zaman imzasından anlaşılmaz; yakalamayı unutursanız bunu ancak program çökünce öğrenirsiniz.

Rust'ta istisna yoktur. Hatalar iki gruba ayrılır ve her grubun kendi aracı vardır:[^genel]

- **Kurtarılamaz hatalar** programdaki bir yanlışın belirtisidir; devam etmek anlamlı değildir. Aracı `panic!` makrosudur.
- **Kurtarılabilir hatalar** beklenen başarısızlıklardır. Aracı `Result<T, E>` türüdür: Hata, fonksiyonun dönüş değerinin içinde gelir ve çağıran onu ele almadan sonuca ulaşamaz.

Bu bölümde önce paniği, ardından `Result`'ı ve onu kullanışlı kılan `?` operatörünü göreceğiz. Son olarak farklı hata türlerini tek bir türde toplayacak ve panik ile `Result` arasında nasıl seçim yapılacağını konuşacağız.

> [!NOTE]
> Bu bölümün kodu `learn/error-handling` dizinindedir. Depo kökünden `cargo run -p error-handling` komutuyla çalıştırabilirsiniz.

## Günlük hayattan bir benzetme

Evin mutfağında kek yaptığınızı düşünün. Ocak tutuşursa keke devam etmezsiniz: Her şeyi bırakır, ocağı kapatır ve mutfaktan çıkarsınız. Çıkarken elinizdekileri de toplarsınız. Bu *paniktir* (panic); bozuk bir durumda devam etmek, durmaktan daha kötüdür.

Süt bitmişse durum farklıdır. Kardeşinizi markete gönderirsiniz; o da eve ya sütle ya da "süt kalmamış" yazan bir notla döner. İkisi de felaket değildir, yalnızca plan değişecektir. Rust'ta bu dönüşün adı `Result`'tır: `Ok(süt)` ya da `Err(not)`. Poşeti açıp içine bakmadan sütü kullanamazsınız.

Kek yapmanızı anneniz istemişse ve kardeşiniz notla dönmüşse yapabileceğiniz en doğru şey keki bırakıp aynı notu annenize götürmektir: "Bunu ben çözemedim." Süt geldiyse hiçbir şey olmamış gibi devam edersiniz. `?` işareti bu iki cümlenin kısaltmasıdır.

Marketten "süt kalmamış", fırından "ekmek daha çıkmadı" notu gelebilir. Anneniz her dükkânın notunu ayrı biçimde okumak istemez; buzdolabının üzerinde tek bir "olmayanlar listesi" tutar ve her not oraya aynı biçimde geçirilir. Bu liste sizin yazdığınız hata enum'udur. `From` ise "market notu listeye şöyle geçirilir" diyen tariftir. Tarifi bir kez yazarsınız, çeviriyi sonra `?` yapar.

Bir de tarif kartı düşünün. "Un: 2 bardak" satırı eksikse yemek yapılamaz; ya durursunuz ya da birine sorarsınız. Kart tamamen boşsa zorunlu satırların hepsi eksiktir ve bunu birinin haber vermesi gerekir. Satırlar arasında kalmış tek bir boş satır ise sorun değildir. Bir sonraki sayfada böyle bir kartı, yani ayar dosyasını okuyan bir program yazacağız.

<details>
<summary>Bu bölümde geçen terimler</summary>

- **hata (error):** İşin planlandığı gibi gitmemesi: dosyanın bulunamaması, metnin sayıya çevrilememesi gibi.
- **panik (panic):** "Devam edemem" deyip işi yarıda kesmek.
- **`Result`, `Ok`, `Err`:** İçinde ya sonuç (`Ok`) ya hata (`Err`) bulunan enum.
- **`Option`, `Some`, `None`:** İçinde ya bir değer (`Some`) bulunan ya da boş olan (`None`) enum.
- **enum:** "Şunlardan biri" diyen tür.
- **`match`:** Bir enum'un hangi varyantta olduğuna bakıp her varyant için yapılacak işi yazmak.
- **`unwrap` ve `expect`:** `Result` ya da `Option` içindeki değeri doğrudan almak. Değer yoksa panik olur.
- **`?` operatörü:** "Hata geldiyse ben de hemen o hatayla döneyim" demenin kısa yolu.
- **yayma (propagation):** Hatayı çözmeyip sizi çağırana iletmek.
- **istisna (exception):** Başka dillerdeki `try`/`catch` ile yakalanan hata bildirme yöntemi. Rust'ta yoktur.
- **thread:** Programın içinde çalışan bir iş kolu. Küçük programlarda tek bir thread vardır.
- **stack:** Çağrılan fonksiyonların üst üste dizildiği bellek bölgesi: "kim kimi çağırdı".
- **yığın çözülmesi (unwinding):** Panikten sonra stack'i geri sarıp yoldaki değerleri düşürmek.
- **`abort`:** Panikten sonra hiçbir şeyi toplamadan süreci hemen sonlandırmak.
- **geri izleme (backtrace):** Paniğe hangi fonksiyonlardan geçerek gelindiğinin listesi.
- **`Drop`:** Bir değer düşürülürken yapılan temizlik. [3. bölümde](03-ownership.md) anlatılmıştı.
- **değişmez (invariant):** Programın hep doğru olduğuna güvendiği koşul: "Yaş eksi olamaz" gibi.
- **trait:** Bir türün "şunu yapabilirim" diye verdiği söz. [9. bölümün](../intermediate/09-generics-traits-lifetimes.md) konusudur.
- **`From`:** "A türünden B türü şöyle yapılır" diyen dönüşüm trait'i.
- **`Display` ve `Debug`:** Bir değeri metne dökmenin iki biçimi: kullanıcı için `{}`, geliştirici için `{:?}`.
- **`#[must_use]`:** "Bu değeri kullanmadan bırakma" diyen öznitelik. Bırakırsanız derleyici uyarır.
- **hata kodu:** `E0277` gibi, derleyici hatasının numarası. `rustc --explain E0277` ayrıntıyı verir.
- **derleyici (`rustc`):** Kodu çalışan programa çeviren ve hataları önceden bulan araç.

</details>

## Kurtarılamaz hata: `panic!`

### Panik nasıl başlar?

Panik iki yoldan başlar: Ya siz `panic!("durum kötü")` yazarsınız ya da çağırdığınız bir işlem sürdürülemez bir durumla karşılaşır. Olmayan bir elemana indeksle erişmek ikinci türdendir:[^panik]

```rust,should_panic
fn main() {
    let v = vec![1, 2, 3];
    println!("{}", v[1]);
    println!("{}", v[99]);
    println!("buraya gelinmez");
}
```

Programın çıktısı (panik iletisi standart hata akışına yazılır):

```text
2
index out of bounds: the len is 3 but the index is 99
```

Gerçek iletide bu satırın üstünde thread'in adı, kimliği ve paniğin kaynak koddaki yeri, altında da geri izlemeyi açmayı öneren bir not bulunur. Kimlik her çalıştırmada değiştiği için bu bölümdeki örneklerde yalnızca iletinin kendisini gösteriyoruz.

Kod derlenir, çünkü indeksin geçerli olup olmadığı genel olarak ancak çalışma zamanında bilinir. İkinci `println!`'den sonra program durur; son satır hiç çalışmaz. C'de böyle bir erişim tanımsız davranıştır ve komşu belleği okuyabilir. Rust yanlış veriyle devam etmek yerine durmayı seçer.

Panik, içinde başladığı thread'i sonlandırır. O thread ana thread ise bütün program biter ve çıkış kodu 101 olur.[^panik-101]

### Yığın çözülmesi ve `abort`

Panikten sonra varsayılan davranış *yığın çözülmesidir* (unwinding): Rust çağrı zincirini geriye doğru yürür ve yoldaki her fonksiyonun değerlerini düşürür.[^unwind] `Drop` uygulayan küçük bir türle bunu gözlemleyebilirsiniz:

```rust,should_panic
struct Gozcu(&'static str);

impl Drop for Gozcu {
    fn drop(&mut self) {
        println!("{} düşürüldü", self.0);
    }
}

fn ic() {
    let _ic = Gozcu("içteki");
    panic!("durum kötü");
}

fn main() {
    let _dis = Gozcu("dıştaki");
    ic();
}
```

Programın çıktısı:

```text
durum kötü
içteki düşürüldü
dıştaki düşürüldü
```

Dosyaların kapanması, kilitlerin bırakılması gibi `Drop`'a bağlı temizlikler bu sayede panikte de yapılır.

Geri sarma işi kod gerektirir ve programı büyütür. Küçük bir ikili dosya istiyorsanız `Cargo.toml`'da panik stratejisini değiştirebilirsiniz:[^abort]

```toml
[profile.release]
panic = "abort"
```

Bu ayarla panik, süreci hemen sonlandırır. Yığın çözülmez, `Drop` çalışmaz; belleği işletim sistemi geri alır. Yukarıdaki program `abort` ile derlendiğinde yalnızca panik iletisini yazar, "düşürüldü" satırlarının hiçbiri görünmez.

### Geri izleme

Panik iletisi paniğin hangi satırda başladığını söyler; ama o satıra nasıl gelindiğini söylemez. `RUST_BACKTRACE` ortam değişkenine 0'dan farklı bir değer verirseniz çağrı zinciri de yazılır:[^backtrace]

```console
RUST_BACKTRACE=1 cargo run
```

Listeyi yukarıdan aşağı okurken ilk kez sizin yazdığınız bir dosyanın adını gördüğünüz satır, araştırmaya başlanacak yerdir.

## Kurtarılabilir hata: `Result<T, E>`

### Tanım ve `match`

`Result`, standart kütüphanede tanımlı sıradan bir enum'dur:[^result]

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
# fn main() {}
```

Başarısız olabilecek bir fonksiyon bu türü döndürür. Süt örneğini yazalım:

```rust
fn sut_al(rafta: u32) -> Result<u32, String> {
    if rafta == 0 {
        Err(String::from("süt kalmamış"))
    } else {
        Ok(1)
    }
}

fn main() {
    match sut_al(0) {
        Ok(adet) => println!("{adet} kutu süt geldi"),
        Err(not) => println!("olmadı: {not}"),
    }
}
```

Çıktı:

```text
olmadı: süt kalmamış
```

Başarısızlık ihtimali artık imzada yazılıdır. Çağıran bu ihtimali görmezden gelip değeri doğrudan kullanamaz:

```rust,compile_fail,E0308
fn sut_al(rafta: u32) -> Result<u32, String> {
    if rafta == 0 {
        Err(String::from("süt kalmamış"))
    } else {
        Ok(1)
    }
}

fn main() {
    let adet: u32 = sut_al(3);
    println!("{adet}");
}
```

Derleyicinin yanıtı:

```text
error[E0308]: mismatched types
   |
10 |     let adet: u32 = sut_al(3);
   |               ---   ^^^^^^^^^ expected `u32`, found `Result<u32, String>`
   |               |
   |               expected due to this
```

Gelen değer bir `u32` değil, açılması gereken bir `Result`'tır. Yanlış kullanım, program çalışmadan yakalanır.

Aynı kalıp, sıfıra bölme gibi sonucu olmayan bir işlemi de hataya çevirir:

```rust
fn ikiye_bol(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err(String::from("sıfıra bölme"))
    } else {
        Ok(a / b)
    }
}

fn main() {
    println!("ikiye_bol(10, 2) = {:?}", ikiye_bol(10, 2));
    println!("ikiye_bol(10, 0) = {:?}", ikiye_bol(10, 0));
}
```

Çıktı:

```text
ikiye_bol(10, 2) = Ok(5)
ikiye_bol(10, 0) = Err("sıfıra bölme")
```

Denetim olmasaydı `a / b` işlemi, `b` sıfır olduğunda panik yaratırdı.

### `unwrap` ve `expect`

Her `Result` için `match` yazmak uzun sürer. `unwrap`, `Ok` içindeki değeri verir; `Err` gelirse panikler. `expect` de aynısını yapar, ama panik iletisinin başına sizin verdiğiniz metni koyar:[^unwrap]

```rust,should_panic
# fn sut_al(rafta: u32) -> Result<u32, String> {
#     if rafta == 0 { Err(String::from("süt kalmamış")) } else { Ok(1) }
# }
fn main() {
    let adet = sut_al(5).expect("rafta süt olmalı");
    println!("{adet} kutu süt geldi");

    let adet = sut_al(0).expect("rafta süt olmalı");
    println!("{adet} kutu süt geldi");
}
```

Programın çıktısı:

```text
1 kutu süt geldi
rafta süt olmalı: "süt kalmamış"
```

İkisi de "burada `Err` gelmesi programdaki bir yanlıştır" dediğiniz yerler içindir. Bunun dışında `match` ya da birazdan göreceğimiz `?` kullanın. İkisi arasında `expect` yeğlenir: Metni, neden `Err` beklemediğinizi söyler ve hata ayıklarken işinize yarar. Standart kütüphane belgesi bu metni "şöyle olmalı" diyen bir önkoşul cümlesi olarak yazmayı önerir.[^expect-mesaj]

### Yok sayılan `Result`

Bir fonksiyonu çağırıp dönen `Result`'a hiç bakmazsanız kod derlenir, ama derleyici uyarır:

```rust
# fn ikiye_bol(a: i32, b: i32) -> Result<i32, String> {
#     if b == 0 { Err(String::from("sıfıra bölme")) } else { Ok(a / b) }
# }
fn main() {
    ikiye_bol(10, 2);
}
```

Derleyicinin uyarısı:

```text
warning: unused `Result` that must be used
  |
9 |     ikiye_bol(10, 2);
  |     ^^^^^^^^^^^^^^^^
  |
  = note: this `Result` may be an `Err` variant, which should be handled
  = note: `#[warn(unused_must_use)]` (part of `#[warn(unused)]`) on by default
help: use `let _ = ...` to ignore the resulting value
```

Uyarının kaynağı `Result`'ın tanımındaki `#[must_use]` özniteliğidir.[^must-use] Sonucu bilerek yok sayıyorsanız bunu kodda gösterin: `let _ = ikiye_bol(10, 2);`.

`Option` türü bu özniteliği taşımaz. `Some(3);` satırı uyarı vermez; yalnızca `Option` döndüren bazı metotlar tek tek işaretlenmiştir. `Result` ise bu konuda yalnız değildir: `ControlFlow` gibi başka türler ve `map` gibi iterator dönüştürücülerinin döndürdüğü türler de `#[must_use]` ile işaretlidir.

## Hatayı yaymak: `?` operatörü

### `match` yerine `?`

Bir fonksiyon, çağırdığı işlemin hatasını çoğu zaman kendisi çözemez; onu çağırana iletir. Buna *hata yayma* (error propagation) denir. `?` operatörü bu işi tek karaktere indirir: Değer `Ok` ise içindekini verir, `Err` ise fonksiyondan hemen o hatayla döner.[^soru]

```rust
use std::num::ParseIntError;

fn metni_sayiya(x: &str) -> Result<i32, ParseIntError> {
    let n: i32 = x.parse()?;
    Ok(n * 2)
}

fn propagasyon() -> Result<i32, ParseIntError> {
    let a = metni_sayiya("21")?;
    Ok(a)
}

fn main() {
    match propagasyon() {
        Ok(n) => println!("propagasyon OK: {n}"),
        Err(e) => println!("propagasyon HATA: {e}"),
    }
    println!("{:?}", metni_sayiya("xx"));
}
```

Çıktı:

```text
propagasyon OK: 42
Err(ParseIntError { kind: InvalidDigit })
```

`metni_sayiya("xx")` çağrısında `parse` bir `Err` döndürür; `?` yüzünden `Ok(n * 2)` satırına hiç gelinmez. `propagasyon` içinde `metni_sayiya("xx")?` yazsaydık o da aynı hatayla hemen dönerdi. `?` olmasaydı her adımdan sonra aynı `match`'i elle yazmanız gerekirdi ve asıl iş, hata kodunun arasında kaybolurdu.

Mutfak örneğinde `?`, kardeşinizin getirdiği notu annenize götürmektir:

```rust
# fn sut_al(rafta: u32) -> Result<u32, String> {
#     if rafta == 0 { Err(String::from("süt kalmamış")) } else { Ok(1) }
# }
fn kek_yap(rafta: u32) -> Result<String, String> {
    let sut = sut_al(rafta)?; // Err gelirse fonksiyon burada biter
    Ok(format!("{sut} kutu sütle kek"))
}

fn main() {
    println!("{:?}", kek_yap(3));
    println!("{:?}", kek_yap(0));
}
```

Çıktı:

```text
Ok("1 kutu sütle kek")
Err("süt kalmamış")
```

### `?` nerede kullanılabilir?

`?` fonksiyondan erken dönmek demektir; öyleyse fonksiyonun dönüş türü o erken dönüşü taşıyabilmelidir. Değer döndürmeyen bir fonksiyonda kullanılamaz:[^soru-yer]

```rust,compile_fail,E0277
# fn sut_al(rafta: u32) -> Result<u32, String> {
#     if rafta == 0 { Err(String::from("süt kalmamış")) } else { Ok(1) }
# }
fn mutfak() {
    let sut = sut_al(3)?;
    println!("{sut}");
}
# fn main() { mutfak(); }
```

Derleyicinin yanıtı:

```text
error[E0277]: the `?` operator can only be used in a function that returns `Result` or `Option` (or another type that implements `FromResidual`)
  |
5 | fn mutfak() {
  | ----------- this function should return `Result` or `Option` to accept `?`
6 |     let sut = sut_al(3)?;
  |                        ^ cannot use the `?` operator in a function that returns `()`
```

Mesajdaki `FromResidual`, `?`'in erken dönüş yapabildiği türlerin uyguladığı trait'tir. Günlük kullanımda bu türler `Result` ve `Option`'dır; ötekileri şimdilik bilmeniz gerekmez. Çözüm iki türlüdür: Ya hatayı `match` ile orada ele alırsınız ya da fonksiyonun dönüş türünü `Result` yaparsınız.

### `main`'den `Result` döndürmek

`main` de `()` döndürdüğü için aynı hatayı verir. Dönüş türünü değiştirerek `main` içinde `?` kullanabilirsiniz:[^main-result]

```rust,no_run
use std::num::ParseIntError;

fn main() -> Result<(), ParseIntError> {
    let n: i32 = "kırk".parse()?;
    println!("{n}");
    Ok(())
}
```

`main` `Ok(())` döndürürse program 0 çıkış koduyla biter. `Err` döndürürse hata `Debug` biçimiyle standart hata akışına yazılır ve çıkış kodu sıfırdan farklı olur. Yukarıdaki program şunu yazar:

```text
Error: ParseIntError { kind: InvalidDigit }
```

### `Option` ile `?`

`?` `Option` üzerinde de çalışır: `Some` ise içindekini verir, `None` ise fonksiyondan `None` ile döner.

```rust
fn ilk_kelimenin_son_harfi(metin: &str) -> Option<char> {
    let kelime = metin.split_whitespace().next()?;
    kelime.chars().last()
}

fn main() {
    println!("{:?}", ilk_kelimenin_son_harfi("merhaba dünya"));
    println!("{:?}", ilk_kelimenin_son_harfi("   "));
}
```

Çıktı:

```text
Some('a')
None
```

Ancak iki aile karıştırılamaz: `Option` döndüren fonksiyonda `Result` üzerinde, `Result` döndüren fonksiyonda `Option` üzerinde doğrudan `?` kullanmak yine `E0277` verir. İkisi arasındaki köprüleri bir sonraki sayfada göreceğiz.

## Farklı hataları tek türde toplamak

### Kendi hata enum'unuz

Bir fonksiyon imzasında tek bir hata türü yazabilir. Oysa içinde yaptığı işler farklı hatalar üretebilir: biri metni sayıya çevirirken, öteki iş kuralını denetlerken. Bu hataların her birine bir varyant ayıran bir enum tanımlamak yerleşik çözümdür:

```rust
use std::fmt;
use std::num::ParseIntError;

#[derive(Debug)]
enum UygulamaHatasi {
    Ayristirma(ParseIntError),
    Mantik(String),
}

impl fmt::Display for UygulamaHatasi {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UygulamaHatasi::Ayristirma(e) => write!(f, "ayrıştırma hatası: {e}"),
            UygulamaHatasi::Mantik(m) => write!(f, "mantık hatası: {m}"),
        }
    }
}

fn main() {
    let h = UygulamaHatasi::Mantik(String::from("negatif olamaz"));
    println!("{h}");
    println!("{h:?}");
}
```

Çıktı:

```text
mantık hatası: negatif olamaz
Mantik("negatif olamaz")
```

`Display`, kullanıcının göreceği tek satırlık mesajı üretir. `Debug` ise geliştiriciye hangi varyantın hangi alanlarla geldiğini gösterir.[^display] Hata türlerinde ikisini de uygulamak yaygındır; standart kütüphanedeki `Error` trait'i zaten ikisini birden ister.

### `From` ve `?`

Şimdi bu türü döndüren bir fonksiyon yazalım. `parse` bir `ParseIntError` üretir, fonksiyon ise `UygulamaHatasi` döndürecektir. `?` aradaki dönüşümü bilmediği için kod bu hâliyle derlenmez:

```rust,compile_fail,E0277
# use std::num::ParseIntError;
#[derive(Debug)]
enum UygulamaHatasi {
    Ayristirma(ParseIntError),
    Mantik(String),
}

fn hesapla(x: &str) -> Result<i32, UygulamaHatasi> {
    let n = x.parse::<i32>()?;
    Ok(n)
}
# fn main() { let _ = hesapla("7"); }
```

Derleyicinin yanıtı:

```text
error[E0277]: `?` couldn't convert the error to `UygulamaHatasi`
   |
 9 | fn hesapla(x: &str) -> Result<i32, UygulamaHatasi> {
   |                        --------------------------- expected `UygulamaHatasi` because of this
10 |     let n = x.parse::<i32>()?;
   |               --------------^ the trait `From<ParseIntError>` is not implemented for `UygulamaHatasi`
   |               |
   |               this can't be annotated with `?` because it has type `Result<_, ParseIntError>`
   |
note: `UygulamaHatasi` needs to implement `From<ParseIntError>`
```

Mesaj çözümü de söyler. `From<ParseIntError>` uygulandığında `?` dönüşümü kendiliğinden yapar:[^from]

```rust
# use std::fmt;
# use std::num::ParseIntError;
#
# #[derive(Debug)]
# enum UygulamaHatasi {
#     Ayristirma(ParseIntError),
#     Mantik(String),
# }
#
# impl fmt::Display for UygulamaHatasi {
#     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
#         match self {
#             UygulamaHatasi::Ayristirma(e) => write!(f, "ayrıştırma hatası: {e}"),
#             UygulamaHatasi::Mantik(m) => write!(f, "mantık hatası: {m}"),
#         }
#     }
# }
#
impl From<ParseIntError> for UygulamaHatasi {
    fn from(e: ParseIntError) -> Self {
        UygulamaHatasi::Ayristirma(e)
    }
}

fn hesapla(x: &str) -> Result<i32, UygulamaHatasi> {
    let n: i32 = x.parse()?; // ParseIntError, From ile UygulamaHatasi olur
    if n < 0 {
        return Err(UygulamaHatasi::Mantik(String::from("negatif olamaz")));
    }
    Ok(n)
}

fn main() {
    println!("hesapla(\"7\")  = {:?}", hesapla("7"));
    println!("hesapla(\"xx\") = {:?}", hesapla("xx"));
    println!("hesapla(\"-3\") = {:?}", hesapla("-3"));

    if let Err(e) = hesapla("xx") {
        println!("Display: {e}");
    }
}
```

Çıktı:

```text
hesapla("7")  = Ok(7)
hesapla("xx") = Err(Ayristirma(ParseIntError { kind: InvalidDigit }))
hesapla("-3") = Err(Mantik("negatif olamaz"))
Display: ayrıştırma hatası: invalid digit found in string
```

Dönüşüm tarifini bir kez yazdınız; `ParseIntError` üreten her `?` artık onu kullanır. Tarif olmasaydı her adımda gelen hatayı elle çevirmeniz gerekirdi.

## Panik mi, `Result` mı?

Seçimin ölçütü "Çağıran bu durumda bir şey yapabilir mi?" sorusudur.[^panik-mi]

Dış dünyadan gelen başarısızlıklar beklenir: Kullanıcı yanlış yazar, dosya silinmiştir, sunucu yanıt vermez. Bunlarda `Result` döndürün. Çağıran yeniden deneyebilir, varsayılan bir değer kullanabilir ya da hatayı kendi çağıranına iletebilir.

Programın kendi değişmezlerinden biri bozulmuşsa durum başkadır: Boş olmaması gereken liste boştur, imkânsız denen kola girilmiştir. Bu bir programlama hatasıdır ve çağıranın yapabileceği bir şey yoktur. Bozuk durumla devam etmek yanlış sonuç üretir; `panic!` ile durmak daha güvenlidir.

Örnek kod, prototip ve testlerde `unwrap` ile `expect` kullanmak olağandır. Testlerde panik zaten testin düştüğünü bildirmenin yoludur; bunu [10. bölümde](../intermediate/10-writing-tests.md) göreceğiz.

İyi bir hata ise üç soruyu yanıtlar: Ne oldu, nerede oldu, neden oldu?

## Hata kodu başvurusu

| Kod | Ne zaman görülür? | Ne yapmalı? |
| --- | --- | --- |
| `E0277` | `?`, `Result` ya da `Option` döndürmeyen bir fonksiyonda kullanıldığında | Dönüş türünü `Result` yapın ya da hatayı `match` ile ele alın. |
| `E0277` | `?`'in hatayı dönüş türündeki hataya çevirecek `From` uygulaması bulunmadığında | `From` uygulayın ya da `map_err` ile elle çevirin. |
| `E0277` | `Option` ile `Result` `?`'te karıştırıldığında | `.ok()?` ya da `.ok_or(..)?` kullanın. |
| `E0308` | `Result` içindeki değer açılmadan kullanıldığında | `match`, `?`, `expect` ya da `unwrap_or` gibi bir metotla açın. |

Derleyici ayrıca kullanılmayan bir `Result` için `unused_must_use` uyarısını verir; bu bir hata değildir, ama görmezden gelinmemelidir. Çalışma zamanında ise `Err` üzerinde çağrılan `unwrap` ve `expect` panik yaratır.

## Perde arkası: Derleyici ne yapıyor?

> [!NOTE]
> Bu başlık ilk okuyuşta atlanabilir. Burada `?` operatörünün sıradan bir `match`'e çevrildiğini, kullanılmayan sonuç uyarısının nereden geldiğini ve panikte kaynakların nasıl toplandığını görüyoruz.

**`Result` ve `Option` sıradan enum'lardır.** Derleyici bu türleri hata mesajları ve lint'ler için özel olarak tanır, ama çalışma zamanında hata taşıyan ayrı bir mekanizma yoktur: `Err` dönmek, herhangi bir enum değeri döndürmekle aynıdır.[^result-kod] "Hata olabilir" bilgisi türün kendisindedir; `Ok` içindeki değere ulaşmak isteyen, `Err` ihtimaline bir yanıt vermek zorundadır.

**`#[must_use]` bir lint'tir.** Öznitelik, kullanılmadan bırakılan değerler için `unused_must_use` uyarısını tetikler; denetim rustc'nin `rustc_lint` bileşenindeki `must_use.rs` dosyasındadır.[^must-use-kod] Bilinmesi gereken bir istisna vardır: Hata türü hiçbir değer alamayan (uninhabited) bir türse, örneğin `Result<(), Infallible>` ise, uyarı verilmez; çünkü ele alınacak bir hata zaten oluşamaz. Bu istisna 1.92.0 sürümüyle geldi.[^must-use-surum]

**`?` bir `match`'e açılır.** Derleyicinin kısa yazımı aynı işi yapan daha yalın bir koda çevirmesine *indirgeme* (lowering, desugaring) denir. Sözdizimi ağacı (AST) HIR adlı ara biçime çevrilirken `lower_expr_try` fonksiyonu `ifade?` yazımını kabaca şuna dönüştürür:[^lower]

```text
match Try::branch(ifade) {
    ControlFlow::Continue(deger) => deger,
    ControlFlow::Break(artik) => return FromResidual::from_residual(artik),
}
```

`Try::branch` değeri ikiye ayırır: `Continue` "devam et", `Break` "dur ve dön" demektir. `Break` kolundaki *artık* (residual), değerin yalnızca başarısız yarısıdır: `Result` için `Err(e)`, `Option` için `None`. `FromResidual` bu yarıdan fonksiyonun dönüş türünü kurar. `Result` için yazılmış uygulamada `Err(From::from(e))` ifadesi yer alır;[^fromresidual] `From<ParseIntError>` yazmanın yetmesi bu yüzdendir. Dönüş türü `FromResidual` uygulamıyorsa `E0277` çıkar. Hem `Try` hem `FromResidual` henüz kararlı değildir; kendi türleriniz için uygulayamazsınız, ama `?` aracılığıyla her gün kullanırsınız.[^try-reference]

> [!NOTE]
> Rust 1.99.0'da `Result`'ın artığı `Result<Infallible, E>` türündendir; bu ad bazı hata mesajlarında karşınıza çıkar. 1.100.0'da `!` (never) türü kararlı oldu ve `Infallible` onun başka adı hâline geldi.[^never]

**Panik stratejisi derleme ayarıdır.** `panic = "unwind"` ve `panic = "abort"` değerleri `rustc`'ye `-C panic` bayrağı olarak geçer ve profil başına seçilir. Testler, benchmark'lar, derleme betikleri ve yordamsal makrolar bu ayarı yok sayar; test koşucusu şu anda yığın çözülmesine ihtiyaç duyar.[^abort]

## Özet

- `panic!` kurtarılamaz hatalar içindir. Varsayılan davranış yığını çözüp `Drop`'ları çalıştırmaktır; `panic = "abort"` süreci hemen sonlandırır.
- `Result<T, E>` başarısızlık ihtimalini dönüş türüne yazar; `Ok` içindeki değere ulaşmak için `Err`'e yanıt vermek gerekir. Kullanılmayan `Result` uyarı üretir.
- `?`, `Ok` ise değeri verir, `Err` ise erken döner. Yalnızca dönüş türü bunu taşıyabilen fonksiyonlarda kullanılır; `main` de `Result` döndürebilir.
- Farklı hatalar bir enum'da toplanır; `From` uygulaması sayesinde `?` dönüşümü kendiliğinden yapar. `Display` kullanıcıya, `Debug` geliştiriciye yazılır.
- Dış dünyanın başarısızlığı `Result`, programın kendi yanlışı paniktir.

Sıradaki sayfa [8.1 Hata Yönetimi: Derinleşme](08-error-handling-deep-dive.md), bu araçlarla bir ayar dosyası okuyucusu yazıyor ve hataya bağlam eklemeyi, hata zincirini, `Option` ile `Result` arasındaki köprüleri ve `Box<dyn Error>` kararını ele alıyor.

## Kaynaklar

[^genel]: The Rust Programming Language, [9. Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html).
[^panik]: The Rust Programming Language, [9.1 Unrecoverable Errors with `panic!`](https://doc.rust-lang.org/book/ch09-01-unrecoverable-errors-with-panic.html).
[^panik-101]: std, [`panic!`: Current implementation](https://doc.rust-lang.org/std/macro.panic.html#current-implementation).
[^unwind]: The Rust Programming Language, [Unwinding the Stack or Aborting in Response to a Panic](https://doc.rust-lang.org/book/ch09-01-unrecoverable-errors-with-panic.html#unwinding-the-stack-or-aborting-in-response-to-a-panic); The Rust Reference, [Panic](https://doc.rust-lang.org/reference/panic.html).
[^abort]: The Cargo Book, [Profiles: `panic`](https://doc.rust-lang.org/cargo/reference/profiles.html#panic).
[^backtrace]: The Rust Programming Language, [9.1 Unrecoverable Errors with `panic!`](https://doc.rust-lang.org/book/ch09-01-unrecoverable-errors-with-panic.html); std, [`std::backtrace`: Environment variables](https://doc.rust-lang.org/std/backtrace/index.html#environment-variables).
[^result]: The Rust Programming Language, [9.2 Recoverable Errors with `Result`](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html); std, [`Result`](https://doc.rust-lang.org/std/result/enum.Result.html).
[^unwrap]: The Rust Programming Language, [Shortcuts for Panic on Error](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#shortcuts-for-panic-on-error); std, [`Result::unwrap`](https://doc.rust-lang.org/std/result/enum.Result.html#method.unwrap) ve [`Result::expect`](https://doc.rust-lang.org/std/result/enum.Result.html#method.expect).
[^expect-mesaj]: std, [`std::error`: Common message styles](https://doc.rust-lang.org/std/error/index.html#common-message-styles).
[^must-use]: std, [Results must be used](https://doc.rust-lang.org/std/result/index.html#results-must-be-used); The Rust Reference, [The `must_use` attribute](https://doc.rust-lang.org/reference/attributes/diagnostics.html#the-must_use-attribute); rustc 1.99.0 kaynak kodu, [`control_flow.rs`, `pub enum ControlFlow`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/ops/control_flow.rs#L89).
[^soru]: The Rust Programming Language, [Propagating Errors](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#propagating-errors) ve [The `?` Operator Shortcut](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#the--operator-shortcut).
[^soru-yer]: The Rust Programming Language, [Where to Use the `?` Operator](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#where-to-use-the--operator); hata dizini, [`E0277`](https://doc.rust-lang.org/error_codes/E0277.html).
[^main-result]: The Rust Programming Language, [Where to Use the `?` Operator](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#where-to-use-the--operator); rustc 1.99.0 kaynak kodu, [`process.rs`, `Termination for Result<T, E>`](https://github.com/rust-lang/rust/blob/1.99.0/library/std/src/process.rs#L2719).
[^display]: std, [`std::fmt`](https://doc.rust-lang.org/std/fmt/index.html#fmtdisplay-vs-fmtdebug); std, [`Error` trait'i](https://doc.rust-lang.org/std/error/trait.Error.html).
[^from]: The Rust Programming Language, [The `?` Operator Shortcut](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#the--operator-shortcut); std, [`From` trait'i](https://doc.rust-lang.org/std/convert/trait.From.html).
[^panik-mi]: The Rust Programming Language, [Guidelines for Error Handling](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html#guidelines-for-error-handling) ve [Examples, Prototype Code, and Tests](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html#examples-prototype-code-and-tests).
[^result-kod]: rustc 1.99.0 kaynak kodu, [`result.rs`, `pub enum Result`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/result.rs#L558).
[^must-use-kod]: rustc 1.99.0 kaynak kodu, [`must_use.rs`, `UNUSED_MUST_USE`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_lint/src/unused/must_use.rs#L44) ve [`must_use.rs`, `is_uninhabited`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_lint/src/unused/must_use.rs#L148).
[^must-use-surum]: Rust 1.92.0 sürüm notları, [Language](https://github.com/rust-lang/rust/releases/tag/1.92.0); rust-lang/rust, [PR 147382](https://github.com/rust-lang/rust/pull/147382).
[^lower]: rustc 1.99.0 kaynak kodu, [`expr.rs`, `fn lower_expr_try`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_ast_lowering/src/expr.rs#L1883).
[^fromresidual]: rustc 1.99.0 kaynak kodu, [`result.rs`, `Err(e) => Err(From::from(e))`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/result.rs#L2193).
[^try-reference]: The Rust Reference, [The try propagation expression](https://doc.rust-lang.org/reference/expressions/operator-expr.html#the-try-propagation-expression); std, [`FromResidual`](https://doc.rust-lang.org/std/ops/trait.FromResidual.html).
[^never]: The Rust Blog, [Announcing Rust 1.100.0](https://blog.rust-lang.org/2026/11/12/Rust-1.100.0/); Rust 1.100.0 sürüm notları, [Compatibility Notes](https://github.com/rust-lang/rust/releases/tag/1.100.0).
