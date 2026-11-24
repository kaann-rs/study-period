# 8.1 Hata Yönetimi: Derinleşme

Önceki sayfada `Result`'ı, `?` operatörünü ve tek bir `From` dönüşümünü ayrı ayrı gördük. Bu sayfada hepsini küçük ama gerçekçi bir işte birleştireceğiz: Bir ayar dosyasını okuyacak, satır satır ayrıştıracak ve değerlerini doğrulayacağız.

İyi bir hata "olmadı" demekle kalmaz; ne olmadığını, nerede olmadığını ve neden olmadığını da söyler. `Result<i32, String>` bunun için yetmez: Bir metnin içine bakmadan "dosya mı yok, biçim mi bozuk?" ayırt edilemez. Ayrıca her katmanın bildiği bilgi başkadır. Alt katman "sayıya çevrilemedi" der; hangi satırda, hangi anahtarda olduğunu ancak üst katman ekleyebilir. Sayfanın ana konusu bu iki ihtiyaçtır: hataları türleriyle ayırt etmek ve onlara bağlam eklemek.

> [!NOTE]
> Bu sayfanın kodu `learn/error-handling/src/deep_dive.rs` dosyasındadır. Depo kökünden `cargo run -p error-handling` ile çalıştırabilir, testlerini `cargo test -p error-handling` ile sınayabilirsiniz.

<details>
<summary>Bu sayfada geçen yeni terimler</summary>

- **varyant (variant):** Bir enum'un seçeneklerinden her biri. Hata enum'unda bir hata çeşidi.
- **`Error` trait'i:** "Ben bir hata türüyüm" diyen standart söz. `Display` ve `Debug` ister.
- **`source`:** "Bu hataya yol açan alttaki hata hangisi?" sorusunun yanıtını veren metot.
- **bağlam (context):** Hataya eklenen yer bilgisi: hangi satır, hangi anahtar.
- **`map_err`:** `Err` içindeki hatayı başka bir hataya çeviren, `Ok`'a dokunmayan metot.
- **`ok_or` ve `ok_or_else`:** `Option`'ı `Result`'a çeviren metotlar. `None` ise verdiğiniz hata konur.
- **`.ok()`:** `Result`'ı `Option`'a çeviren metot. Hatanın nedeni atılır, `None` kalır.
- **`transpose`:** İç içe duran `Result` ile `Option`'ın sırasını değiştiren metot.
- **closure:** `|x| ...` biçiminde yazılan isimsiz fonksiyon. [12. bölümde](../intermediate/12-closures-iterators.md) anlatılır.
- **iterator ve `collect`:** Elemanları sırayla veren değer ve o elemanları bir koleksiyonda toplayan metot.
- **`BTreeMap`:** Anahtarlarını sıralı tutan eşleme türü.
- **jenerik (generic) ve `where`:** Birçok tür için yazılmış kod ve o türlerden beklenen koşullar.
- **`Box<dyn Error>`:** `Error` uygulayan herhangi bir hatayı taşıyabilen kutu.
- **trait nesnesi (trait object):** Asıl türü silinmiş, yalnızca uyguladığı trait bilinen değer.
- **`downcast_ref`:** Bir trait nesnesine "içindeki şu tür mü?" diye soran metot.
- **API, sözleşme (contract):** Fonksiyonun dışarıya verdiği söz: ne alır, ne döndürür, hangi hata çıkabilir.
- **sabit değer (literal):** Koda elle yazılmış değer: `"127.0.0.1"`, `42`.
- **turbofish:** Tür parametresini çağrı sırasında yazmak: `parse::<i32>()`.
- **tahsis (allocation):** Heap'ten yer istemek. Bedava değildir; gereksizse kaçınılır.
- **programlama hatası (bug):** Programcının kodda yaptığı yanlış.
- **prototip, betik (prototype, script):** Denemek için yazılmış kısa ömürlü kod; küçük yardımcı program.

</details>

## Senaryo: bir ayar dosyası

Okuyacağımız dosya, önceki sayfadaki tarif kartına benzer. Her satırı `anahtar = değer` biçimindedir; boş satırlar ve `#` ile başlayan açıklamalar atlanır:

```text
# sunucu
ad = deneme
port = 8080
azami_baglanti = 64
```

`port` ve `azami_baglanti` zorunludur ve sayı olmalıdır. `ad` isteğe bağlıdır. İşin sonunda elimizde şu yapı olacak:

```rust
#[derive(Debug, PartialEq)]
pub struct Ayarlar {
    ad: String,
    port: u16,
    azami_baglanti: u32,
}
# fn main() {}
```

## Kendi hata türünüz

### Her soruna ayrı bir varyant

Tarif kartından kek yapacaktınız ve olmadı. "Olmadı" demek yetmez; size yardım edecek kişi nedenini bilmek ister. Kart mı kayıp, bir satır mı okunmuyor, yoksa bir malzeme mi eksik? Bunlar ayrı sorunlardır ve çözümleri de ayrıdır. Rust'ta her sorun için enum'a ayrı bir varyant yazılır, çağıran `match` ile hangisi olduğuna bakar:

```rust
enum TarifHatasi {
    KartYok,
    SatirBozuk { satir: usize },
    MalzemeEksik(&'static str),
}

fn tarifi_oku() -> Result<String, TarifHatasi> {
    Err(TarifHatasi::SatirBozuk { satir: 3 })
}

fn main() {
    match tarifi_oku() {
        Err(TarifHatasi::KartYok) => println!("ezberden yaparım"),
        Err(TarifHatasi::SatirBozuk { satir }) => println!("{satir}. satırı düzelt"),
        Err(TarifHatasi::MalzemeEksik(ad)) => println!("{ad} al"),
        Ok(_tarif) => println!("başlıyorum"),
    }
}
```

Çıktı:

```text
3. satırı düzelt
```

Hata bir `String` olsaydı çağıran `if hata == "kart yok"` gibi bir karşılaştırma yapmak zorunda kalırdı. Mesajdaki tek bir harf değişince o kod sessizce yanlış çalışırdı. Enum'da ise yanlış yazılmış bir varyant adı derlenmez; yeni bir varyant eklendiğinde de eksik kalan `match` kollarını derleyici hata olarak bildirir.[^enum]

Ayar dosyamızı okurken dört sorunla karşılaşabiliriz; hata türünün de dört varyantı vardır:

```rust
# use std::io;
# use std::num::ParseIntError;
#[derive(Debug)]
pub enum AyarHatasi {
    // Dosya okunamadı; alt katmanın hatası sarılır
    Gc(io::Error),
    // Satırda `=` yok
    Bicim { satir: usize, icerik: String },
    // Değer sayıya çevrilemedi
    Sayi { anahtar: &'static str, deger: String, kaynak: ParseIntError },
    // Zorunlu anahtar yok
    Eksik(&'static str),
}
# fn main() {}
```

Varyantların alanları bağlamı taşır. `Bicim`, kullanıcının hangi satırı düzelteceğini bilmesi için satır numarasını saklar. `Sayi` hem anahtarı ve bulunan değeri hem de asıl nedeni, yani `ParseIntError`'ı saklar. (`Gc`, girdi/çıktı hatasının kısaltmasıdır.)

### `Display` ve `Debug`

Aynı sorunu iki kişiye iki dille anlatırsınız. Misafire "Kek biraz gecikecek" dersiniz; mutfaktaki yardımcınıza "Fırın 180 dereceye çıkmıyor". `Display` (`{}`) birincisidir: son kullanıcının göreceği tek satır. `Debug` (`{:?}`) ikincisidir: geliştiricinin göreceği bütün ayrıntı.[^display]

`Debug`'ı `derive` ile aldık. `Display`'i ise elle yazarız, çünkü kullanıcıya ne söyleneceğine derleyici karar veremez:

```rust
# use std::fmt;
# use std::io;
# use std::num::ParseIntError;
#
# #[derive(Debug)]
# pub enum AyarHatasi {
#     Gc(io::Error),
#     Bicim { satir: usize, icerik: String },
#     Sayi { anahtar: &'static str, deger: String, kaynak: ParseIntError },
#     Eksik(&'static str),
# }
#
impl fmt::Display for AyarHatasi {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AyarHatasi::Gc(e) => write!(f, "ayar dosyası okunamadı ({:?})", e.kind()),
            AyarHatasi::Bicim { satir, icerik } => {
                write!(f, "satır {satir}: `anahtar=değer` bekleniyordu, bulunan {icerik:?}")
            }
            AyarHatasi::Sayi { anahtar, deger, .. } => {
                write!(f, "`{anahtar}` sayı olmalı, bulunan {deger:?}")
            }
            AyarHatasi::Eksik(anahtar) => write!(f, "zorunlu anahtar eksik: {anahtar}"),
        }
    }
}

fn main() {
    let hata = AyarHatasi::Bicim { satir: 2, icerik: String::from("azami baglanti 64") };
    println!("{hata}");
    println!("{hata:?}");
}
```

Çıktı:

```text
satır 2: `anahtar=değer` bekleniyordu, bulunan "azami baglanti 64"
Bicim { satir: 2, icerik: "azami baglanti 64" }
```

İki ayrıntıya dikkat edin. `Gc` kolu işletim sisteminin mesajını değil, yalnızca hatanın türünü (`ErrorKind`) yazar; sistemin mesajı platforma göre değişir, tür adı değişmez. `Sayi` kolu ise asıl nedeni, yani `kaynak` alanını hiç yazmaz. Bunun gerekçesi bir sonraki başlıktadır.

### `Error` trait'i ve `source` zinciri

"Kek olmadı." Neden? "Çünkü fırın ısınmadı." Neden? "Çünkü fişi takılı değildi." Her yanıtın arkasında bir önceki neden durur; sordukça köke inersiniz. Standart kütüphanenin `Error` trait'indeki `source` metodu bu "neden?" sorusudur: Bir hata, kendisine yol açan alttaki hatayı gösterir, o da varsa kendininkini.[^source] En üstteki cümle hangi işin bozulduğunu, en alttaki cümle neyin düzeltileceğini söyler.

`Error` yalnızca `Debug` ve `Display` ister; gövdesi boş bile bırakılabilir, çünkü `source` varsayılan olarak `None` döndürür. Bizim türümüzde iki varyantın altında başka bir hata vardır:

```rust
# use std::error::Error;
# use std::fmt;
# use std::io;
# use std::num::ParseIntError;
#
# #[derive(Debug)]
# pub enum AyarHatasi {
#     Gc(io::Error),
#     Bicim { satir: usize, icerik: String },
#     Sayi { anahtar: &'static str, deger: String, kaynak: ParseIntError },
#     Eksik(&'static str),
# }
#
# impl fmt::Display for AyarHatasi {
#     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
#         match self {
#             AyarHatasi::Gc(e) => write!(f, "ayar dosyası okunamadı ({:?})", e.kind()),
#             AyarHatasi::Bicim { satir, icerik } => {
#                 write!(f, "satır {satir}: `anahtar=değer` bekleniyordu, bulunan {icerik:?}")
#             }
#             AyarHatasi::Sayi { anahtar, deger, .. } => {
#                 write!(f, "`{anahtar}` sayı olmalı, bulunan {deger:?}")
#             }
#             AyarHatasi::Eksik(anahtar) => write!(f, "zorunlu anahtar eksik: {anahtar}"),
#         }
#     }
# }
#
impl Error for AyarHatasi {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AyarHatasi::Gc(e) => Some(e),
            AyarHatasi::Sayi { kaynak, .. } => Some(kaynak),
            AyarHatasi::Bicim { .. } | AyarHatasi::Eksik(_) => None,
        }
    }
}

// "Neden?" sorusunu kök nedene kadar sorar
fn zinciri_yaz(hata: &(dyn Error + 'static)) -> String {
    std::iter::successors(Some(hata), |&h| h.source())
        .map(|h| h.to_string())
        .collect::<Vec<_>>()
        .join(" <- ")
}

fn main() {
    let kaynak = "seksen".parse::<u16>().expect_err("seksen rakam değil");
    let hata = AyarHatasi::Sayi { anahtar: "port", deger: String::from("seksen"), kaynak };

    println!("{hata}");
    println!("{:?}", hata.source());
    println!("{}", zinciri_yaz(&hata));
    println!("{}", AyarHatasi::Eksik("port").source().is_none());
}
```

Çıktı:

```text
`port` sayı olmalı, bulunan "seksen"
Some(ParseIntError { kind: InvalidDigit })
`port` sayı olmalı, bulunan "seksen" <- invalid digit found in string
true
```

`zinciri_yaz`, bir önceki değerden bir sonrakini üreten `successors` iterator'ını kullanır:[^successors] İlk hatadan başlar, `source` `None` verene kadar iner ve mesajları ` <- ` ile birleştirir.

`Display`'de neden `kaynak` yazmadığımız şimdi anlaşılır. Yazsaydık zinciri yürüten her raporlayıcı aynı mesajı iki kez basardı. Standart kütüphanenin kuralı şudur: Alttaki hata ya `source` ile verilir ya da `Display`'de yazılır, ikisi birden yapılmaz.[^source] Aynı belgeye göre hata mesajları genellikle kısa, küçük harfle başlayan ve sonunda noktalama işareti bulunmayan cümlelerdir.

### `From` ve sınırı: `map_err`

Marketten gelen "yok" notunu listeye geçirmek kolaydır; ama "ne yok?" sorusu yanıtsız kalır. Listeyi tutan siz olduğunuz için notu "yumurta yok" diye tamamlayabilirsiniz. Birinci iş `From`, ikinci iş `map_err`'dir.

`From` uygulaması `?`'in kendiliğinden dönüşümünü açar. Dosya okuma hatasında eklenecek bağlam olmadığı için bu yeterlidir:

```rust
# use std::io;
# #[derive(Debug)]
# pub enum AyarHatasi { Gc(io::Error) }
impl From<io::Error> for AyarHatasi {
    fn from(e: io::Error) -> Self {
        AyarHatasi::Gc(e)
    }
}
# fn main() {}
```

`From`'un sınırı, yalnızca hatanın kendisini görmesidir. Sayı ayrıştırma hatasının hangi anahtarda çıktığını ancak o anahtarı okuyan kod bilir. Bu bilgiyi `map_err` ekler:[^map-err]

```rust
# use std::num::ParseIntError;
# #[derive(Debug)]
# pub enum AyarHatasi {
#     Sayi { anahtar: &'static str, deger: String, kaynak: ParseIntError },
# }
fn port_oku(ham: &str) -> Result<u16, AyarHatasi> {
    let n: u16 = ham.parse().map_err(|kaynak| AyarHatasi::Sayi {
        anahtar: "port",         // bağlam: sorun hangi anahtarda
        deger: ham.to_string(),  // bağlam: ne bulundu
        kaynak,                  // asıl neden saklanır
    })?;
    Ok(n)
}

fn main() {
    println!("{:?}", port_oku("8080"));
    println!("{:?}", port_oku("seksen"));
}
```

Çıktı:

```text
Ok(8080)
Err(Sayi { anahtar: "port", deger: "seksen", kaynak: ParseIntError { kind: InvalidDigit } })
```

`map_err` yalnızca `Err` içindeki hatayı değiştirir; `Ok` olduğu gibi geçer. `From<ParseIntError>` yazsaydık `?` yine çalışırdı, ama ortaya çıkan hata "geçersiz rakam" der, hangi anahtarda olduğunu söyleyemezdi.

## Ayrıştırıcıyı yazmak

### Bir satır, üç olasılık

Tarif kartındaki bir satıra bakınca üç şeyden biri olur. Satır boştur; sorun yok, geçersiniz. "un = 2" gibi düzgün bir satırdır; alıp kullanırsınız. "un işte biraz" gibi anlaşılmaz bir satırdır; bu bir hatadır. "Boş" ile "bozuk" aynı şey değildir: Boş satır da hata sayılsaydı, satırları arasında boşluk olan her kart reddedilirdi.

Üç olasılık iç içe iki türle anlatılır. Dıştaki `Result` "Bozuk mu?", içteki `Option` "Bir şey var mı?" sorusunu yanıtlar:

```rust
# use std::io;
# use std::num::ParseIntError;
#
# #[derive(Debug)]
# pub enum AyarHatasi {
#     Gc(io::Error),
#     Bicim { satir: usize, icerik: String },
#     Sayi { anahtar: &'static str, deger: String, kaynak: ParseIntError },
#     Eksik(&'static str),
# }
#
fn satir_ayristir(no: usize, satir: &str) -> Result<Option<(String, String)>, AyarHatasi> {
    let satir = satir.trim();
    if satir.is_empty() || satir.starts_with('#') {
        return Ok(None); // boş satır ya da açıklama: hata değil
    }
    let (anahtar, deger) = satir.split_once('=').ok_or_else(|| AyarHatasi::Bicim {
        satir: no,
        icerik: satir.to_string(),
    })?;
    Ok(Some((anahtar.trim().to_string(), deger.trim().to_string())))
}

fn main() {
    println!("{:?}", satir_ayristir(1, ""));
    println!("{:?}", satir_ayristir(2, "un = 2"));
    println!("{:?}", satir_ayristir(3, "un işte"));
}
```

Çıktı:

```text
Ok(None)
Ok(Some(("un", "2")))
Err(Bicim { satir: 3, icerik: "un işte" })
```

`split_once('=')` bir `Option` döndürür: `=` yoksa `None`. `ok_or_else` bu `None`'ı bizim hatamıza çevirir, `?` de hatayı geri döndürür. `ok_or` yerine `ok_or_else` seçmemizin nedeni tahsistir. `ok_or`'a verilen değer her durumda hesaplanır, yani `satir.to_string()` başarılı satırlarda da boşuna çalışırdı. `ok_or_else`'in closure'ı yalnızca `None` gelince çağrılır.[^ok-or]

### Bütün satırlar: `collect` ile ilk hatada durmak

Kart satır satır okunur, boş satırlar atlanır, bozuk satırda durulur:

```rust
# use std::collections::BTreeMap;
# use std::io;
# use std::num::ParseIntError;
#
# #[derive(Debug)]
# pub enum AyarHatasi {
#     Gc(io::Error),
#     Bicim { satir: usize, icerik: String },
#     Sayi { anahtar: &'static str, deger: String, kaynak: ParseIntError },
#     Eksik(&'static str),
# }
#
# fn satir_ayristir(no: usize, satir: &str) -> Result<Option<(String, String)>, AyarHatasi> {
#     let satir = satir.trim();
#     if satir.is_empty() || satir.starts_with('#') {
#         return Ok(None);
#     }
#     let (anahtar, deger) = satir.split_once('=').ok_or_else(|| AyarHatasi::Bicim {
#         satir: no,
#         icerik: satir.to_string(),
#     })?;
#     Ok(Some((anahtar.trim().to_string(), deger.trim().to_string())))
# }
#
fn tabloya_ayristir(metin: &str) -> Result<BTreeMap<String, String>, AyarHatasi> {
    metin
        .lines()
        .enumerate()
        .filter_map(|(i, satir)| satir_ayristir(i + 1, satir).transpose())
        .collect()
}

fn main() {
    println!("{:?}", tabloya_ayristir("# sunucu\n\nport = 8080\nad = deneme"));
    println!("{:?}", tabloya_ayristir("a=1\nbozuk\nb=2\nyine bozuk"));
}
```

Çıktı:

```text
Ok({"ad": "deneme", "port": "8080"})
Err(Bicim { satir: 2, icerik: "bozuk" })
```

Dört satırlık bu zincirde iki araç iş görür:

- `transpose`, `Result<Option<T>, E>` değerini `Option<Result<T, E>>` değerine çevirir.[^transpose] `Ok(None)` böylece `None` olur ve `filter_map` tarafından atılır. `Err(e)` ise `Some(Err(e))` olur, yani kaybolmaz.
- `collect`, `Result`'lardan oluşan bir akışı `Result<Koleksiyon, E>` olarak toplayabilir. İlk `Err`'de akışı keser ve o hatayı döndürür.[^collect] Örnekte 3. ve 4. satırlara hiç bakılmadı.

### Değerleri okumak

Tablodaki değerler metindir. Onları sayıya çeviren fonksiyon iki köprüyü birlikte kullanır:

```rust
# use std::collections::BTreeMap;
# use std::io;
# use std::num::ParseIntError;
# use std::str::FromStr;
#
# #[derive(Debug)]
# pub enum AyarHatasi {
#     Gc(io::Error),
#     Bicim { satir: usize, icerik: String },
#     Sayi { anahtar: &'static str, deger: String, kaynak: ParseIntError },
#     Eksik(&'static str),
# }
#
fn sayi_al<T>(tablo: &BTreeMap<String, String>, anahtar: &'static str) -> Result<T, AyarHatasi>
where
    T: FromStr<Err = ParseIntError>,
{
    let ham = tablo.get(anahtar).ok_or(AyarHatasi::Eksik(anahtar))?;
    ham.parse().map_err(|kaynak| AyarHatasi::Sayi {
        anahtar,
        deger: ham.clone(),
        kaynak,
    })
}

fn main() {
    let tablo = BTreeMap::from([(String::from("port"), String::from("8080"))]);

    println!("{:?}", sayi_al::<u16>(&tablo, "port"));
    println!("{:?}", sayi_al::<u32>(&tablo, "azami_baglanti"));
}
```

Çıktı:

```text
Ok(8080)
Err(Eksik("azami_baglanti"))
```

İlk satır `Option`'ı `Result`'a çevirir: Anahtarın yokluğu burada bir hatadır. `AyarHatasi::Eksik(anahtar)` değerini kurmak tahsis gerektirmediği için `ok_or` yeterlidir. İkinci satır `ParseIntError`'ı alır, çevredeki `anahtar` ve `ham` bilgisini ekleyerek kendi hatamıza sarar.

`T: FromStr<Err = ParseIntError>` koşulu "T, metinden üretilebilen ve üretilemeyince `ParseIntError` veren herhangi bir tür" demektir.[^fromstr] Böylece aynı fonksiyon hem `u16` hem `u32` okur; hangisi olacağını çağrıldığı yerdeki tür belirler. Jenerik fonksiyonları ve trait sınırlarını [9. bölümde](../intermediate/09-generics-traits-lifetimes.md) göreceğiz.

### Hepsini birleştirmek

`ayarlari_coz` parçaları bir araya getirir:

```rust
# use std::collections::BTreeMap;
# use std::error::Error;
# use std::fmt;
# use std::fs;
# use std::io;
# use std::num::ParseIntError;
# use std::path::Path;
# use std::str::FromStr;
#
# #[derive(Debug)]
# pub enum AyarHatasi {
#     Gc(io::Error),
#     Bicim { satir: usize, icerik: String },
#     Sayi { anahtar: &'static str, deger: String, kaynak: ParseIntError },
#     Eksik(&'static str),
# }
#
# impl fmt::Display for AyarHatasi {
#     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
#         match self {
#             AyarHatasi::Gc(e) => write!(f, "ayar dosyası okunamadı ({:?})", e.kind()),
#             AyarHatasi::Bicim { satir, icerik } => {
#                 write!(f, "satır {satir}: `anahtar=değer` bekleniyordu, bulunan {icerik:?}")
#             }
#             AyarHatasi::Sayi { anahtar, deger, .. } => {
#                 write!(f, "`{anahtar}` sayı olmalı, bulunan {deger:?}")
#             }
#             AyarHatasi::Eksik(anahtar) => write!(f, "zorunlu anahtar eksik: {anahtar}"),
#         }
#     }
# }
#
# impl Error for AyarHatasi {
#     fn source(&self) -> Option<&(dyn Error + 'static)> {
#         match self {
#             AyarHatasi::Gc(e) => Some(e),
#             AyarHatasi::Sayi { kaynak, .. } => Some(kaynak),
#             AyarHatasi::Bicim { .. } | AyarHatasi::Eksik(_) => None,
#         }
#     }
# }
#
# fn zinciri_yaz(hata: &(dyn Error + 'static)) -> String {
#     std::iter::successors(Some(hata), |&h| h.source())
#         .map(|h| h.to_string())
#         .collect::<Vec<_>>()
#         .join(" <- ")
# }
#
# #[derive(Debug, PartialEq)]
# pub struct Ayarlar {
#     ad: String,
#     port: u16,
#     azami_baglanti: u32,
# }
#
# fn satir_ayristir(no: usize, satir: &str) -> Result<Option<(String, String)>, AyarHatasi> {
#     let satir = satir.trim();
#     if satir.is_empty() || satir.starts_with('#') {
#         return Ok(None);
#     }
#     let (anahtar, deger) = satir.split_once('=').ok_or_else(|| AyarHatasi::Bicim {
#         satir: no,
#         icerik: satir.to_string(),
#     })?;
#     Ok(Some((anahtar.trim().to_string(), deger.trim().to_string())))
# }
#
# fn tabloya_ayristir(metin: &str) -> Result<BTreeMap<String, String>, AyarHatasi> {
#     metin
#         .lines()
#         .enumerate()
#         .filter_map(|(i, satir)| satir_ayristir(i + 1, satir).transpose())
#         .collect()
# }
#
# fn sayi_al<T>(tablo: &BTreeMap<String, String>, anahtar: &'static str) -> Result<T, AyarHatasi>
# where
#     T: FromStr<Err = ParseIntError>,
# {
#     let ham = tablo.get(anahtar).ok_or(AyarHatasi::Eksik(anahtar))?;
#     ham.parse().map_err(|kaynak| AyarHatasi::Sayi {
#         anahtar,
#         deger: ham.clone(),
#         kaynak,
#     })
# }
#
pub fn ayarlari_coz(metin: &str) -> Result<Ayarlar, AyarHatasi> {
    let tablo = tabloya_ayristir(metin)?;
    Ok(Ayarlar {
        // İsteğe bağlı alan: yokluğu hata değil
        ad: tablo
            .get("ad")
            .cloned()
            .unwrap_or_else(|| String::from("adsız")),
        port: sayi_al(&tablo, "port")?,
        azami_baglanti: sayi_al(&tablo, "azami_baglanti")?,
    })
}

fn main() {
    let gecerli = "# sunucu\nad = deneme\nport = 8080\nazami_baglanti = 64\n";
    println!("geçerli: {:?}", ayarlari_coz(gecerli));

    for bozuk in [
        "port = 8080\nazami baglanti 64",
        "port = seksen\nazami_baglanti = 64",
        "port = 8080",
        "port = 99999\nazami_baglanti = 1",
    ] {
        if let Err(e) = ayarlari_coz(bozuk) {
            println!("hata: {}", zinciri_yaz(&e));
        }
    }
}
```

Çıktı:

```text
geçerli: Ok(Ayarlar { ad: "deneme", port: 8080, azami_baglanti: 64 })
hata: satır 2: `anahtar=değer` bekleniyordu, bulunan "azami baglanti 64"
hata: `port` sayı olmalı, bulunan "seksen" <- invalid digit found in string
hata: zorunlu anahtar eksik: azami_baglanti
hata: `port` sayı olmalı, bulunan "99999" <- number too large to fit in target type
```

`ad` isteğe bağlı olduğu için `Option`'da kalır ve varsayılan değerle kapatılır. Son satırdaki `99999`, `u16`'ya sığmadığı için hata verir; bu denetimi ayrıca yazmadık, alanın türü yeterli oldu. Her hata mesajı ne olduğunu ve nerede olduğunu söyler, zincirin sonu da nedenini ekler. Girdi tamamen boş olsaydı satırların hiçbiri hata vermezdi, ama `port` bulunamadığı için `zorunlu anahtar eksik: port` hatası çıkardı.

## `Option` ile `Result` arasında köprüler

`Option` "Yok olabilir" der, `Result` ise neden olmadığını da söyler. Dolabı açtınız, yumurta yok: Boş raf yalnızca "yok" der (`None`). Kek yapacaksanız bu bir sorundur ve adını koymanız gerekir: "Yumurta olmadığı için kek yapılamadı" (`Err`). Tersi de olur: Nedeni önemsemiyorsanız notu atar, yalnızca "var mı, yok mu" bilgisini tutarsınız. İki ayrı tür bu yüzden vardır. Yokluk bazen hata değildir, örneğin isteğe bağlı vanilya; hata olduğunda ise nedeni kaybolmamalıdır.

| Yön | Araç | Anlamı |
| --- | --- | --- |
| `Option<T>`'den `Result<T, E>`'ye | `ok_or(hata)`, `ok_or_else(\|\| hata)` | Yokluk burada bir hatadır. |
| `Result<T, E>`'den `Option<T>`'ye | `.ok()` | Nedeni bilerek atıyorsunuz. |
| `Result<Option<T>, E>` ile `Option<Result<T, E>>` arasında | `transpose` | İç içe iki türün sırası değişir. |

```rust
fn ilk_anahtar(metin: &str) -> Option<&str> {
    let satir = metin.lines().find(|s| !s.trim().is_empty())?;
    let (anahtar, _) = satir.split_once('=')?;
    Some(anahtar.trim())
}

fn main() {
    println!("{:?} / {:?}", ilk_anahtar("\nport = 1"), ilk_anahtar("bozuk"));

    let raf: Option<u32> = None;
    println!("{:?}", raf.ok_or("yumurta yok"));

    println!("{:?}", "x1".parse::<u16>().ok());
    println!("{:?}", "12".parse::<u16>().ok());
}
```

Çıktı:

```text
Some("port") / None
Err("yumurta yok")
None
Some(12)
```

`ilk_anahtar`, `?` operatörünün `Option` üzerinde nasıl çalıştığını gösterir: Dolu satır yoksa ya da `=` bulunamazsa fonksiyon `None` ile döner.[^kopru]

## İlk hatada durmak mı, hepsini toplamak mı?

Derleyici ilk hatada durmaz, bulabildiği kadarını listeler; çünkü düzelt ve yeniden çalıştır döngüsünü her hata için tekrarlamak istemezsiniz. Bir ayar dosyası doğrulayıcısı da böyle olmalıdır. Kardeşinizin yazdığı alışveriş listesini denetlerken bütün yanlışları işaretlemek, her yanlış için onu ayrı ayrı geri göndermekten iyidir. Buna karşılık yemek pişerken ilk bozuk malzemede durmak gerekir; ödeme gibi yarım kalması sakıncalı işlemlerde de ilk hatada durmak doğrudur.

`?` ve `collect` ilk hatada durur. Hepsini görmek istiyorsanız hataları bir `Vec`'te kendiniz biriktirirsiniz:

```rust
# use std::collections::BTreeMap;
# use std::error::Error;
# use std::fmt;
# use std::fs;
# use std::io;
# use std::num::ParseIntError;
# use std::path::Path;
# use std::str::FromStr;
#
# #[derive(Debug)]
# pub enum AyarHatasi {
#     Gc(io::Error),
#     Bicim { satir: usize, icerik: String },
#     Sayi { anahtar: &'static str, deger: String, kaynak: ParseIntError },
#     Eksik(&'static str),
# }
#
# impl fmt::Display for AyarHatasi {
#     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
#         match self {
#             AyarHatasi::Gc(e) => write!(f, "ayar dosyası okunamadı ({:?})", e.kind()),
#             AyarHatasi::Bicim { satir, icerik } => {
#                 write!(f, "satır {satir}: `anahtar=değer` bekleniyordu, bulunan {icerik:?}")
#             }
#             AyarHatasi::Sayi { anahtar, deger, .. } => {
#                 write!(f, "`{anahtar}` sayı olmalı, bulunan {deger:?}")
#             }
#             AyarHatasi::Eksik(anahtar) => write!(f, "zorunlu anahtar eksik: {anahtar}"),
#         }
#     }
# }
#
# fn satir_ayristir(no: usize, satir: &str) -> Result<Option<(String, String)>, AyarHatasi> {
#     let satir = satir.trim();
#     if satir.is_empty() || satir.starts_with('#') {
#         return Ok(None);
#     }
#     let (anahtar, deger) = satir.split_once('=').ok_or_else(|| AyarHatasi::Bicim {
#         satir: no,
#         icerik: satir.to_string(),
#     })?;
#     Ok(Some((anahtar.trim().to_string(), deger.trim().to_string())))
# }
#
# fn tabloya_ayristir(metin: &str) -> Result<BTreeMap<String, String>, AyarHatasi> {
#     metin
#         .lines()
#         .enumerate()
#         .filter_map(|(i, satir)| satir_ayristir(i + 1, satir).transpose())
#         .collect()
# }
#
fn hatalari_topla(metin: &str) -> (usize, Vec<AyarHatasi>) {
    let mut gecerli = 0;
    let mut hatalar = Vec::new();
    for (i, satir) in metin.lines().enumerate() {
        match satir_ayristir(i + 1, satir) {
            Ok(Some(_)) => gecerli += 1,
            Ok(None) => {}
            Err(e) => hatalar.push(e), // durma; kaydet ve devam et
        }
    }
    (gecerli, hatalar)
}

fn main() {
    let metin = "port = 1\nbozuk satır\nad = x\nbir bozuk daha";

    match tabloya_ayristir(metin) {
        Ok(tablo) => println!("collect: {} çift", tablo.len()),
        Err(e) => println!("collect (ilk hata): {e}"),
    }

    let (gecerli, hatalar) = hatalari_topla(metin);
    println!("toplama: {gecerli} geçerli, {} hata", hatalar.len());
    for e in &hatalar {
        println!("  - {e}");
    }
}
```

Çıktı:

```text
collect (ilk hata): satır 2: `anahtar=değer` bekleniyordu, bulunan "bozuk satır"
toplama: 2 geçerli, 2 hata
  - satır 2: `anahtar=değer` bekleniyordu, bulunan "bozuk satır"
  - satır 4: `anahtar=değer` bekleniyordu, bulunan "bir bozuk daha"
```

İki fonksiyon da aynı `satir_ayristir`'ı kullanır. Durup durmama kararı satırı ayrıştıran kodda değil, sonuçları toplayan koddadır.[^collect]

## Gerçek bir dosyadan okumak

### İki adım, tek hata türü

Dosyadan yüklemek iki adımdır: önce kartı çekmeceden çıkarmak, sonra satırlarını anlamak. Çekmece boşsa ikinci adıma hiç geçilmez:

```rust,no_run
# use std::collections::BTreeMap;
# use std::error::Error;
# use std::fmt;
# use std::fs;
# use std::io;
# use std::num::ParseIntError;
# use std::path::Path;
# use std::str::FromStr;
#
# #[derive(Debug)]
# pub enum AyarHatasi {
#     Gc(io::Error),
#     Bicim { satir: usize, icerik: String },
#     Sayi { anahtar: &'static str, deger: String, kaynak: ParseIntError },
#     Eksik(&'static str),
# }
#
# impl From<io::Error> for AyarHatasi {
#     fn from(e: io::Error) -> Self {
#         AyarHatasi::Gc(e)
#     }
# }
#
# #[derive(Debug, PartialEq)]
# pub struct Ayarlar {
#     ad: String,
#     port: u16,
#     azami_baglanti: u32,
# }
#
# fn satir_ayristir(no: usize, satir: &str) -> Result<Option<(String, String)>, AyarHatasi> {
#     let satir = satir.trim();
#     if satir.is_empty() || satir.starts_with('#') {
#         return Ok(None);
#     }
#     let (anahtar, deger) = satir.split_once('=').ok_or_else(|| AyarHatasi::Bicim {
#         satir: no,
#         icerik: satir.to_string(),
#     })?;
#     Ok(Some((anahtar.trim().to_string(), deger.trim().to_string())))
# }
#
# fn tabloya_ayristir(metin: &str) -> Result<BTreeMap<String, String>, AyarHatasi> {
#     metin
#         .lines()
#         .enumerate()
#         .filter_map(|(i, satir)| satir_ayristir(i + 1, satir).transpose())
#         .collect()
# }
#
# fn sayi_al<T>(tablo: &BTreeMap<String, String>, anahtar: &'static str) -> Result<T, AyarHatasi>
# where
#     T: FromStr<Err = ParseIntError>,
# {
#     let ham = tablo.get(anahtar).ok_or(AyarHatasi::Eksik(anahtar))?;
#     ham.parse().map_err(|kaynak| AyarHatasi::Sayi {
#         anahtar,
#         deger: ham.clone(),
#         kaynak,
#     })
# }
#
# pub fn ayarlari_coz(metin: &str) -> Result<Ayarlar, AyarHatasi> {
#     let tablo = tabloya_ayristir(metin)?;
#     Ok(Ayarlar {
#         ad: tablo
#             .get("ad")
#             .cloned()
#             .unwrap_or_else(|| String::from("adsız")),
#         port: sayi_al(&tablo, "port")?,
#         azami_baglanti: sayi_al(&tablo, "azami_baglanti")?,
#     })
# }
#
pub fn dosyadan_yukle(yol: &Path) -> Result<Ayarlar, AyarHatasi> {
    let metin = fs::read_to_string(yol)?; // io::Error, From ile Gc olur
    ayarlari_coz(&metin)
}
# fn main() {}
```

Hangi adım bozulursa bozulsun dışarıya aynı türden hata çıkar: Dosya hatası `From` sayesinde `Gc` olur, ayrıştırma hataları zaten bizim türümüzdedir.[^soru-from] Çağıran tek bir hata türüyle uğraşır ve imzaya bakınca neler olabileceğini görür.

Hatanın son kez ele alındığı yer, programın en dış katmanıdır. Orada hata ya yazdırılır ya varsayılana dönülür ya da programdan çıkılır. Varyantlar bu kararı mümkün kılar: "Dosya yoksa varsayılan ayarlarla devam et, başka bir sorun varsa bildir."

```rust
# use std::collections::BTreeMap;
# use std::error::Error;
# use std::fmt;
# use std::fs;
# use std::io;
# use std::num::ParseIntError;
# use std::path::Path;
# use std::str::FromStr;
#
# #[derive(Debug)]
# pub enum AyarHatasi {
#     Gc(io::Error),
#     Bicim { satir: usize, icerik: String },
#     Sayi { anahtar: &'static str, deger: String, kaynak: ParseIntError },
#     Eksik(&'static str),
# }
#
# impl From<io::Error> for AyarHatasi {
#     fn from(e: io::Error) -> Self {
#         AyarHatasi::Gc(e)
#     }
# }
#
# #[derive(Debug, PartialEq)]
# pub struct Ayarlar {
#     ad: String,
#     port: u16,
#     azami_baglanti: u32,
# }
#
# fn satir_ayristir(no: usize, satir: &str) -> Result<Option<(String, String)>, AyarHatasi> {
#     let satir = satir.trim();
#     if satir.is_empty() || satir.starts_with('#') {
#         return Ok(None);
#     }
#     let (anahtar, deger) = satir.split_once('=').ok_or_else(|| AyarHatasi::Bicim {
#         satir: no,
#         icerik: satir.to_string(),
#     })?;
#     Ok(Some((anahtar.trim().to_string(), deger.trim().to_string())))
# }
#
# fn tabloya_ayristir(metin: &str) -> Result<BTreeMap<String, String>, AyarHatasi> {
#     metin
#         .lines()
#         .enumerate()
#         .filter_map(|(i, satir)| satir_ayristir(i + 1, satir).transpose())
#         .collect()
# }
#
# fn sayi_al<T>(tablo: &BTreeMap<String, String>, anahtar: &'static str) -> Result<T, AyarHatasi>
# where
#     T: FromStr<Err = ParseIntError>,
# {
#     let ham = tablo.get(anahtar).ok_or(AyarHatasi::Eksik(anahtar))?;
#     ham.parse().map_err(|kaynak| AyarHatasi::Sayi {
#         anahtar,
#         deger: ham.clone(),
#         kaynak,
#     })
# }
#
# pub fn ayarlari_coz(metin: &str) -> Result<Ayarlar, AyarHatasi> {
#     let tablo = tabloya_ayristir(metin)?;
#     Ok(Ayarlar {
#         ad: tablo
#             .get("ad")
#             .cloned()
#             .unwrap_or_else(|| String::from("adsız")),
#         port: sayi_al(&tablo, "port")?,
#         azami_baglanti: sayi_al(&tablo, "azami_baglanti")?,
#     })
# }
#
# pub fn dosyadan_yukle(yol: &Path) -> Result<Ayarlar, AyarHatasi> {
#     let metin = fs::read_to_string(yol)?;
#     ayarlari_coz(&metin)
# }
#
fn main() {
    let olmayan = std::env::temp_dir().join("rust_study_boyle_bir_dosya_yok.conf");

    match dosyadan_yukle(&olmayan) {
        Err(AyarHatasi::Gc(e)) if e.kind() == io::ErrorKind::NotFound => {
            println!("dosya yok, varsayılan ayarlarla devam");
        }
        diger => println!("beklenmeyen: {diger:?}"),
    }
}
```

Çıktı:

```text
dosya yok, varsayılan ayarlarla devam
```

Hata düz bir metin olsaydı bu ayrım ancak mesajın içinde sözcük arayarak yapılabilirdi.

### `Err` bir değerdir, panik değildir

Örneği baştan sona denemek için geçici bir dosya yazan, verilen işi çalıştıran ve dosyayı silen bir yardımcı kullanalım:

```rust
# use std::collections::BTreeMap;
# use std::error::Error;
# use std::fmt;
# use std::fs;
# use std::io;
# use std::num::ParseIntError;
# use std::path::Path;
# use std::str::FromStr;
#
# #[derive(Debug)]
# pub enum AyarHatasi {
#     Gc(io::Error),
#     Bicim { satir: usize, icerik: String },
#     Sayi { anahtar: &'static str, deger: String, kaynak: ParseIntError },
#     Eksik(&'static str),
# }
#
# impl fmt::Display for AyarHatasi {
#     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
#         match self {
#             AyarHatasi::Gc(e) => write!(f, "ayar dosyası okunamadı ({:?})", e.kind()),
#             AyarHatasi::Bicim { satir, icerik } => {
#                 write!(f, "satır {satir}: `anahtar=değer` bekleniyordu, bulunan {icerik:?}")
#             }
#             AyarHatasi::Sayi { anahtar, deger, .. } => {
#                 write!(f, "`{anahtar}` sayı olmalı, bulunan {deger:?}")
#             }
#             AyarHatasi::Eksik(anahtar) => write!(f, "zorunlu anahtar eksik: {anahtar}"),
#         }
#     }
# }
#
# impl Error for AyarHatasi {
#     fn source(&self) -> Option<&(dyn Error + 'static)> {
#         match self {
#             AyarHatasi::Gc(e) => Some(e),
#             AyarHatasi::Sayi { kaynak, .. } => Some(kaynak),
#             AyarHatasi::Bicim { .. } | AyarHatasi::Eksik(_) => None,
#         }
#     }
# }
#
# impl From<io::Error> for AyarHatasi {
#     fn from(e: io::Error) -> Self {
#         AyarHatasi::Gc(e)
#     }
# }
#
# fn zinciri_yaz(hata: &(dyn Error + 'static)) -> String {
#     std::iter::successors(Some(hata), |&h| h.source())
#         .map(|h| h.to_string())
#         .collect::<Vec<_>>()
#         .join(" <- ")
# }
#
# #[derive(Debug, PartialEq)]
# pub struct Ayarlar {
#     ad: String,
#     port: u16,
#     azami_baglanti: u32,
# }
#
# fn satir_ayristir(no: usize, satir: &str) -> Result<Option<(String, String)>, AyarHatasi> {
#     let satir = satir.trim();
#     if satir.is_empty() || satir.starts_with('#') {
#         return Ok(None);
#     }
#     let (anahtar, deger) = satir.split_once('=').ok_or_else(|| AyarHatasi::Bicim {
#         satir: no,
#         icerik: satir.to_string(),
#     })?;
#     Ok(Some((anahtar.trim().to_string(), deger.trim().to_string())))
# }
#
# fn tabloya_ayristir(metin: &str) -> Result<BTreeMap<String, String>, AyarHatasi> {
#     metin
#         .lines()
#         .enumerate()
#         .filter_map(|(i, satir)| satir_ayristir(i + 1, satir).transpose())
#         .collect()
# }
#
# fn sayi_al<T>(tablo: &BTreeMap<String, String>, anahtar: &'static str) -> Result<T, AyarHatasi>
# where
#     T: FromStr<Err = ParseIntError>,
# {
#     let ham = tablo.get(anahtar).ok_or(AyarHatasi::Eksik(anahtar))?;
#     ham.parse().map_err(|kaynak| AyarHatasi::Sayi {
#         anahtar,
#         deger: ham.clone(),
#         kaynak,
#     })
# }
#
# pub fn ayarlari_coz(metin: &str) -> Result<Ayarlar, AyarHatasi> {
#     let tablo = tabloya_ayristir(metin)?;
#     Ok(Ayarlar {
#         ad: tablo
#             .get("ad")
#             .cloned()
#             .unwrap_or_else(|| String::from("adsız")),
#         port: sayi_al(&tablo, "port")?,
#         azami_baglanti: sayi_al(&tablo, "azami_baglanti")?,
#     })
# }
#
# pub fn dosyadan_yukle(yol: &Path) -> Result<Ayarlar, AyarHatasi> {
#     let metin = fs::read_to_string(yol)?;
#     ayarlari_coz(&metin)
# }
#
fn gecici_dosya_ile<T>(etiket: &str, icerik: &str, is: impl FnOnce(&Path) -> T) -> io::Result<T> {
    let ad = format!("rust_study_ayar_{}_{etiket}.conf", std::process::id());
    let yol = std::env::temp_dir().join(ad);
    fs::write(&yol, icerik)?;
    let sonuc = is(&yol); // Ok da olabilir, Err de
    fs::remove_file(&yol)?; // iki durumda da bu satıra gelinir
    Ok(sonuc)
}

fn main() {
    let icerik = "ad = üretim\nport = 443\nazami_baglanti = 1000\n";

    match gecici_dosya_ile("calistir", icerik, dosyadan_yukle) {
        Ok(Ok(ayarlar)) => println!(
            "yüklendi: {} :{} (azami {})",
            ayarlar.ad, ayarlar.port, ayarlar.azami_baglanti
        ),
        Ok(Err(e)) => println!("ayar hatası: {}", zinciri_yaz(&e)),
        Err(e) => println!("geçici dosya kullanılamadı: {:?}", e.kind()),
    }
}
```

Çıktı:

```text
yüklendi: üretim :443 (azami 1000)
```

`is: impl FnOnce(&Path) -> T` parametresi "bir dosya yolu alan ve en fazla bir kez çağrılacak iş" demektir. İç içe iki sonuç olduğuna dikkat edin: Dıştaki `Result` geçici dosya işinin, içteki ise ayarların okunup okunamadığını söyler. `Ok(Ok(..))` ve `Ok(Err(..))` kolları bu yüzden vardır.

Tezgâhı ödünç aldığınızı düşünün: Hamur yırtılsa da işiniz bitince tezgâhı silmeniz gerekir. Yardımcı da böyle davranır. `is(&yol)` başarısız olsa bile `remove_file` satırına gelinir, çünkü `Err` sıradan bir değerdir ve akışı kesmez; elinizde tutar, bir sonraki satıra geçersiniz. Panik öyle değildir: `is` paniklerse sonraki satırlar atlanır ve dosya silinmez.

Panikte de temizlik istiyorsanız silme işini bir `Drop` uygulamasına taşımanız gerekir. Yığın çözülürken `Drop` yine çalışır.[^unwind] İşi bitince temizliği kendiliğinden yapan böyle değerlere *RAII koruyucusu* (RAII guard) denir; [14. bölümde](../intermediate/14-smart-pointers.md) karşımıza çıkacaklar. 1.100.0 sürümüyle kararlı olan `DropGuard` türü, tek seferlik temizlikler için hazır bir koruyucu sağlar.[^dropguard]

## `Box<dyn Error>` mi, özel enum mu?

Mutfakta iki çeşit kap vardır. Etiketli kavanozların üzerinde "un", "şeker" yazar; baktığınızda içindekini bilirsiniz. Bir de üzerinde "karışık" yazan büyük kutu vardır: İçine her şeyi atabilirsiniz, ama çıkarırken ne geleceğini bilemezsiniz; ancak "Bu şeker mi?" diye sorabilirsiniz. Özel hata enum'u etiketli kavanoz, `Box<dyn Error>` karışık kutudur.

`Box<dyn Error>`, `Error` uygulayan her türü kabul eden bir trait nesnesidir. Standart kütüphanedeki `From` uygulamaları sayesinde `?` her hatayı kutulayıp geçirir; `&str` ve `String` bile `.into()` ile hataya dönüşür:[^box-dyn]

```rust
use std::error::Error;
use std::num::ParseIntError;

fn hizli_toplam(metin: &str) -> Result<u32, Box<dyn Error>> {
    let mut toplam = 0;
    for parca in metin.split(',') {
        toplam += parca.trim().parse::<u32>()?; // ParseIntError kutuya girer
    }
    if toplam == 0 {
        return Err("toplam sıfır olamaz".into()); // &str de hataya dönüşür
    }
    Ok(toplam)
}

fn main() {
    println!("{:?}", hizli_toplam("1, 2, 3").ok());

    for girdi in ["1, iki", "0, 0"] {
        if let Err(e) = hizli_toplam(girdi) {
            let sayi_hatasi_mi = e.downcast_ref::<ParseIntError>().is_some();
            println!("{girdi:?}: {e} (ParseIntError mı? {sayi_hatasi_mi})");
        }
    }
}
```

Çıktı:

```text
Some(6)
"1, iki": invalid digit found in string (ParseIntError mı? true)
"0, 0": toplam sıfır olamaz (ParseIntError mı? false)
```

Bedeli, asıl türün silinmesidir. Çağıran `match` ile çeşidine bakamaz; ancak `downcast_ref` ile "Şu tür müydü?" diye sorabilir.[^downcast] Ayrıca hata değeri heap'e taşınır ve metotları dinamik gönderimle çağrılır; trait nesnelerinin bu yanını [17. bölümde](../intermediate/17-oop.md) inceleyeceğiz.

Farklı hataların aynı kutuya girmesi, küçük programlarda çok pratiktir. Aşağıdaki fonksiyonda `?` bir yerde `ParseIntError`'ı, ötekinde `io::Error`'ı kutular:

```rust,no_run
use std::error::Error;
use std::fs;

fn hizli() -> Result<usize, Box<dyn Error>> {
    let n = "12".parse::<usize>()?;
    let metin = fs::read_to_string("a")?;
    Ok(n + metin.len())
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("{}", hizli()?);
    Ok(())
}
```

Hangisini seçeceğinizi, hatayla ne yapılacağı belirler:

- **Kütüphane** yazıyorsanız, yani kodu başkaları çağıracak ve hataya göre davranacaksa özel enum kullanın. Olası hatalar imzanın, yani verdiğiniz sözün bir parçası olur.
- **Uygulama**, `main`, betik ya da prototip yazıyorsanız ve hata yalnızca raporlanacaksa `Box<dyn Error>` yeter.

Yaygın kalıp ikisini birleştirir: İç katmanlar enum döndürür, en dıştaki fonksiyon `fn main() -> Result<(), Box<dyn Error>>` olur.

## `unwrap` ve `expect` ne zaman yerinde?

Panik mi, `Result` mı kararını veren tek bir soru vardır: Bu başarısızlığın sorumlusu kim?[^unwrap-olcut]

- **Dış dünya:** kullanıcı girdisi, dosya, ağ. Bunlar beklenen durumlardır. Misafirin "Ben fıstık yiyemem" demesi sizin hatanız değildir; menüyü değiştirirsiniz. `Result` döndürün ve kararı çağırana bırakın.
- **Programcı:** bozulmuş bir değişmez, "buraya gelinmez" denen kol. Tarife "tuz" yerine "şeker" yazan sizseniz yemeği öyle servis etmek yerine durup tarifi düzeltmeniz gerekir. Panik doğrudur; sessizce devam etmek bozuk veriyle çalışmak demektir.

`unwrap` ve `expect`, "Burada hata çıkarsa yanlış benim kodumdadır" dediğiniz yerler içindir. Yerinde oldukları üç durum vardır:

1. Testler ve örnek kodlar.
2. Sonradan `?`'e çevrilecek prototipler.
3. Derleyicinin göremediği, ama sizin kanıtlayabildiğiniz durumlar. Bu durumda kanıtı `expect` mesajına yazın.

```rust
use std::net::Ipv4Addr;

fn main() {
    let yerel: Ipv4Addr = "127.0.0.1".parse().expect("sabit literal geçerli bir IPv4");

    let sayilar = [3, 1, 2];
    let en_buyuk = sayilar.iter().max().expect("dizi boş değil");

    println!("yerel = {yerel}, en büyük = {en_buyuk}");
}
```

Çıktı:

```text
yerel = 127.0.0.1, en büyük = 3
```

İlk satırdaki metin koda elle yazılmıştır; geçersizse bu kullanıcının değil, yazanın hatasıdır. İkincisinde dizinin boş olmadığı bir satır yukarıda görülür, dolayısıyla `max` `None` veremez. Aynı `parse` çağrısını kullanıcıdan gelen bir metinle yapıyorsanız `girdi.parse().unwrap()` yanlıştır: Derlenir, ama ilk yanlış girdide program panikler. Doğrusu `let yas: u8 = girdi.parse()?;` gibi hatayı yaymaktır.

## Sık yapılan hatalar

En çok takılınan yerlerin özü aynıdır: `?` hatayı yukarı iletir, ama dönüş türü onu taşıyamaz. `?`'i bir kepçe, dönüş türünü tabak gibi düşünün. Tabak yoksa derleyici dökmenize izin vermez.

### `?` operatörünü `()` döndüren bir fonksiyonda kullanmak

Önceki sayfada bunu `mutfak` fonksiyonunda görmüştük; kural her fonksiyon için geçerlidir:

```rust,compile_fail,E0277
fn oku() {
    let n: i32 = "5".parse()?;
    println!("{n}");
}
# fn main() { oku(); }
```

Derleyicinin yanıtı:

```text
error[E0277]: the `?` operator can only be used in a function that returns `Result` or `Option` (or another type that implements `FromResidual`)
  |
1 | fn oku() {
  | -------- this function should return `Result` or `Option` to accept `?`
2 |     let n: i32 = "5".parse()?;
  |                             ^ cannot use the `?` operator in a function that returns `()`
```

Ya dönüş türünü `Result` yapıp hatayı yukarı iletin ya da hatayı orada ele alın:

```rust
use std::num::ParseIntError;

fn oku() -> Result<(), ParseIntError> {
    let n: i32 = "5".parse()?;
    println!("{n}");
    Ok(())
}

fn oku_ve_bildir() {
    match "beş".parse::<i32>() {
        Ok(n) => println!("{n}"),
        Err(e) => eprintln!("{e}"),
    }
}
#
# fn main() {
#     oku().expect("5 geçerli bir sayı");
#     oku_ve_bildir();
# }
```

### `From` uygulaması yokken `?` kullanmak

`AyarHatasi` için `From<io::Error>` yazmıştık, ama `From<ParseIntError>` yazmadık:

```rust,compile_fail,E0277
# use std::io;
# #[derive(Debug)]
# pub enum AyarHatasi { Gc(io::Error) }
# impl From<io::Error> for AyarHatasi {
#     fn from(e: io::Error) -> Self { AyarHatasi::Gc(e) }
# }
fn f(s: &str) -> Result<i32, AyarHatasi> {
    let n = s.parse::<i32>()?;
    Ok(n)
}
# fn main() { let _ = f("5"); }
```

Derleyicinin yanıtı:

```text
error[E0277]: `?` couldn't convert the error to `AyarHatasi`
  |
8 |     let n = s.parse::<i32>()?;
  |               --------------^ the trait `From<ParseIntError>` is not implemented for `AyarHatasi`
  |               |
  |               this can't be annotated with `?` because it has type `Result<_, ParseIntError>`
```

Türü turbofish yerine `let n: i32 = s.parse()?;` biçiminde verseniz de hata aynıdır. Bağlam gerekmiyorsa `impl From<ParseIntError> for AyarHatasi` yazın; gerekiyorsa `sayi_al`'da yaptığımız gibi `map_err` kullanın.

> [!NOTE]
> Hata türünüz için hiçbir `From` uygulaması yoksa ve türü turbofish ile yazmadıysanız Rust 1.99.0 aynı sorunu başka bir kodla bildirir: ``error[E0271]: type mismatch resolving `<i32 as FromStr>::Err == AyarHatasi` ``. Kod farklı olsa da sorun ve çözüm aynıdır.[^e0271]

### `Option` ile `Result`'ı karıştırmak

`?` dönüş türünün ailesine bakar. `Option` döndüren bir fonksiyonda `Result` üzerinde kullanılamaz:

```rust,compile_fail,E0277
fn f(s: &str) -> Option<i32> {
    let n: i32 = s.parse()?;
    Some(n)
}
# fn main() { let _ = f("5"); }
```

Derleyicinin yanıtı:

```text
error[E0277]: the `?` operator can only be used on `Option`s, not `Result`s, in a function that returns `Option`
  |
1 | fn f(s: &str) -> Option<i32> {
  | ---------------------------- this function returns an `Option`
2 |     let n: i32 = s.parse()?;
  |                           ^ use `.ok()?` if you want to discard the `Result<Infallible, _>` error information
```

`Result` döndüren bir fonksiyonda da `Option` üzerinde kullanılamaz:

```rust,compile_fail,E0277
fn g(v: &[i32]) -> Result<i32, String> {
    let n = v.first()?;
    Ok(*n)
}
# fn main() { let _ = g(&[]); }
```

Derleyicinin yanıtı:

```text
error[E0277]: the `?` operator can only be used on `Result`s, not `Option`s, in a function that returns `Result`
  |
1 | fn g(v: &[i32]) -> Result<i32, String> {
  | -------------------------------------- this function returns a `Result`
2 |     let n = v.first()?;
  |                      ^ use `.ok_or(...)?` to provide an error compatible with `Result<i32, String>`
```

İki mesaj da doğru köprüyü söyler:

```rust
fn f(s: &str) -> Option<i32> {
    let n: i32 = s.parse().ok()?; // nedeni bilerek atıyoruz
    Some(n)
}

fn g(v: &[i32]) -> Result<i32, String> {
    let n = v.first().ok_or("liste boş")?; // &str, From ile String olur
    Ok(*n)
}

fn main() {
    println!("{:?} {:?}", f("5"), f("beş"));
    println!("{:?} {:?}", g(&[7, 8]), g(&[]));
}
```

Çıktı:

```text
Some(5) None
Ok(7) Err("liste boş")
```

Derleyicinin gözünden bakınca `Option` ve `Result` ayrı artık (residual) ailelerindendir; aralarında `FromResidual` uygulaması yoktur. Yanlış karışım bu yüzden tür denetiminde yakalanır, çalışma zamanına kalmaz.[^fromresidual]

### Yeni varyantın kolunu unutmak

Hata enum'una varyant eklediğinizde onu eşleyen her `match` gözden geçirilmelidir. Derleyici bunu size bırakmaz:

```rust,compile_fail,E0004
use std::fmt;

#[derive(Debug)]
enum AyarHatasi {
    Eksik(&'static str),
    Aralik { anahtar: &'static str, azami: u32 },
}

impl fmt::Display for AyarHatasi {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AyarHatasi::Eksik(anahtar) => write!(f, "zorunlu anahtar eksik: {anahtar}"),
        }
    }
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0004]: non-exhaustive patterns: `&AyarHatasi::Aralik { .. }` not covered
   |
11 |         match self {
   |               ^^^^ pattern `&AyarHatasi::Aralik { .. }` not covered
   |
note: `AyarHatasi` defined here
```

Bu denetim ancak `match`'te her şeyi yakalayan bir `_` kolu yoksa işe yarar. Kendi hata enum'unuzu eşlerken `_` yazmaktan kaçının; birinci alıştırma bunun yararını gösterecek.[^e0004]

## Davranışı testlerle sabitlemek

`deep_dive.rs` dosyasının sonunda yedi test bulunur. Hata türü bir enum olduğu için testler mesaj metnine değil, varyanta ve alanlarına bakabilir. Üçü şöyledir:

```rust,noplayground,test_harness
# use std::collections::BTreeMap;
# use std::error::Error;
# use std::fmt;
# use std::fs;
# use std::io;
# use std::num::ParseIntError;
# use std::path::Path;
# use std::str::FromStr;
#
# #[derive(Debug)]
# pub enum AyarHatasi {
#     Gc(io::Error),
#     Bicim { satir: usize, icerik: String },
#     Sayi { anahtar: &'static str, deger: String, kaynak: ParseIntError },
#     Eksik(&'static str),
# }
#
# impl fmt::Display for AyarHatasi {
#     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
#         match self {
#             AyarHatasi::Gc(e) => write!(f, "ayar dosyası okunamadı ({:?})", e.kind()),
#             AyarHatasi::Bicim { satir, icerik } => {
#                 write!(f, "satır {satir}: `anahtar=değer` bekleniyordu, bulunan {icerik:?}")
#             }
#             AyarHatasi::Sayi { anahtar, deger, .. } => {
#                 write!(f, "`{anahtar}` sayı olmalı, bulunan {deger:?}")
#             }
#             AyarHatasi::Eksik(anahtar) => write!(f, "zorunlu anahtar eksik: {anahtar}"),
#         }
#     }
# }
#
# impl Error for AyarHatasi {
#     fn source(&self) -> Option<&(dyn Error + 'static)> {
#         match self {
#             AyarHatasi::Gc(e) => Some(e),
#             AyarHatasi::Sayi { kaynak, .. } => Some(kaynak),
#             AyarHatasi::Bicim { .. } | AyarHatasi::Eksik(_) => None,
#         }
#     }
# }
#
# impl From<io::Error> for AyarHatasi {
#     fn from(e: io::Error) -> Self {
#         AyarHatasi::Gc(e)
#     }
# }
#
# #[derive(Debug, PartialEq)]
# pub struct Ayarlar {
#     ad: String,
#     port: u16,
#     azami_baglanti: u32,
# }
#
# fn satir_ayristir(no: usize, satir: &str) -> Result<Option<(String, String)>, AyarHatasi> {
#     let satir = satir.trim();
#     if satir.is_empty() || satir.starts_with('#') {
#         return Ok(None);
#     }
#     let (anahtar, deger) = satir.split_once('=').ok_or_else(|| AyarHatasi::Bicim {
#         satir: no,
#         icerik: satir.to_string(),
#     })?;
#     Ok(Some((anahtar.trim().to_string(), deger.trim().to_string())))
# }
#
# fn tabloya_ayristir(metin: &str) -> Result<BTreeMap<String, String>, AyarHatasi> {
#     metin
#         .lines()
#         .enumerate()
#         .filter_map(|(i, satir)| satir_ayristir(i + 1, satir).transpose())
#         .collect()
# }
#
# fn sayi_al<T>(tablo: &BTreeMap<String, String>, anahtar: &'static str) -> Result<T, AyarHatasi>
# where
#     T: FromStr<Err = ParseIntError>,
# {
#     let ham = tablo.get(anahtar).ok_or(AyarHatasi::Eksik(anahtar))?;
#     ham.parse().map_err(|kaynak| AyarHatasi::Sayi {
#         anahtar,
#         deger: ham.clone(),
#         kaynak,
#     })
# }
#
# pub fn ayarlari_coz(metin: &str) -> Result<Ayarlar, AyarHatasi> {
#     let tablo = tabloya_ayristir(metin)?;
#     Ok(Ayarlar {
#         ad: tablo
#             .get("ad")
#             .cloned()
#             .unwrap_or_else(|| String::from("adsız")),
#         port: sayi_al(&tablo, "port")?,
#         azami_baglanti: sayi_al(&tablo, "azami_baglanti")?,
#     })
# }
#
#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn bicim_hatasi_satir_numarasini_tasir() {
        let hata = ayarlari_coz("port=80\n\nesittir yok").unwrap_err();
        assert!(matches!(hata, AyarHatasi::Bicim { satir: 3, .. }));
    }

    #[test]
    fn sayi_hatasi_kok_nedeni_source_ile_verir() {
        let hata = ayarlari_coz("port=abc\nazami_baglanti=5").unwrap_err();
        assert!(matches!(hata, AyarHatasi::Sayi { anahtar: "port", .. }));
        let kok = hata.source().expect("Sayi varyantı kaynağını taşır");
        assert!(kok.downcast_ref::<ParseIntError>().is_some());
    }

    #[test]
    fn eksik_anahtar_ayri_bir_varyanttir() {
        let hata = ayarlari_coz("port=80").unwrap_err();
        assert!(matches!(hata, AyarHatasi::Eksik("azami_baglanti")));
        assert!(hata.source().is_none());
    }
}
```

Öteki dört test şunları sabitler: Geçerli metnin `ad` verilmediğinde `adsız` ile çözülmesi; `hatalari_topla`'nın ilk hatada durmaması; geçici dosyadan yüklemenin çalışması ve dosyanın silinmesi; olmayan dosyanın `Gc` varyantına dönüşmesi. Test yazmayı [10. bölümde](../intermediate/10-writing-tests.md) göreceğiz.

## Akılda kalması gerekenler

- Hata türü bir API sözleşmesidir. Enum, çağırana çeşitleri ayırt etme imkânı verir; `Box<dyn Error>` ile hata çoğunlukla yalnızca raporlanır.
- `From` bağlamsız, kendiliğinden dönüşümdür. `map_err` bağlam ekleyerek dönüştürür.
- Alttaki hata ya `source` ile verilir ya `Display`'de yazılır; ikisi birden yapılmaz.
- Hata, en çok bilgiye sahip katmanda zenginleştirilir, karar verebilen katmanda ele alınır.

## Alıştırmalar

**1.** `AyarHatasi`'na `Aralik { anahtar, deger, azami }` varyantını ekleyin ve `ayarlari_coz` içinde `azami_baglanti` 10 000'den büyükse bu hatayı döndürün. `azami_baglanti = 20000` girdisi `Err(Aralik { .. })` vermeli, hatanın `source` metodu `None` döndürmelidir.

<details>
<summary>İpucu ve çözüm</summary>

Varyantı ekleyip derlediğinizde eksiksizlik denetimi (`E0004`) `Display` ve `source` içindeki iki `match`'i gösterir. `source`'ta `_` kolu kullanmadığımız için orası da gözden kaçmaz.

```rust
# use std::collections::BTreeMap;
# use std::error::Error;
# use std::fmt;
# use std::io;
# use std::num::ParseIntError;
# use std::str::FromStr;
#
#[derive(Debug)]
pub enum AyarHatasi {
    Gc(io::Error),
    Bicim { satir: usize, icerik: String },
    Sayi { anahtar: &'static str, deger: String, kaynak: ParseIntError },
    Eksik(&'static str),
    Aralik { anahtar: &'static str, deger: u32, azami: u32 },
}

impl fmt::Display for AyarHatasi {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
#             AyarHatasi::Gc(e) => write!(f, "ayar dosyası okunamadı ({:?})", e.kind()),
#             AyarHatasi::Bicim { satir, icerik } => {
#                 write!(f, "satır {satir}: `anahtar=değer` bekleniyordu, bulunan {icerik:?}")
#             }
#             AyarHatasi::Sayi { anahtar, deger, .. } => {
#                 write!(f, "`{anahtar}` sayı olmalı, bulunan {deger:?}")
#             }
#             AyarHatasi::Eksik(anahtar) => write!(f, "zorunlu anahtar eksik: {anahtar}"),
            // öteki kollar değişmedi
            AyarHatasi::Aralik { anahtar, deger, azami } => {
                write!(f, "`{anahtar}` en çok {azami} olabilir, bulunan {deger}")
            }
        }
    }
}

impl Error for AyarHatasi {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AyarHatasi::Gc(e) => Some(e),
            AyarHatasi::Sayi { kaynak, .. } => Some(kaynak),
            AyarHatasi::Bicim { .. } | AyarHatasi::Eksik(_) | AyarHatasi::Aralik { .. } => None,
        }
    }
}
#
# #[derive(Debug, PartialEq)]
# pub struct Ayarlar {
#     ad: String,
#     port: u16,
#     azami_baglanti: u32,
# }
#
# fn satir_ayristir(no: usize, satir: &str) -> Result<Option<(String, String)>, AyarHatasi> {
#     let satir = satir.trim();
#     if satir.is_empty() || satir.starts_with('#') {
#         return Ok(None);
#     }
#     let (anahtar, deger) = satir.split_once('=').ok_or_else(|| AyarHatasi::Bicim {
#         satir: no,
#         icerik: satir.to_string(),
#     })?;
#     Ok(Some((anahtar.trim().to_string(), deger.trim().to_string())))
# }
#
# fn tabloya_ayristir(metin: &str) -> Result<BTreeMap<String, String>, AyarHatasi> {
#     metin
#         .lines()
#         .enumerate()
#         .filter_map(|(i, satir)| satir_ayristir(i + 1, satir).transpose())
#         .collect()
# }
#
# fn sayi_al<T>(tablo: &BTreeMap<String, String>, anahtar: &'static str) -> Result<T, AyarHatasi>
# where
#     T: FromStr<Err = ParseIntError>,
# {
#     let ham = tablo.get(anahtar).ok_or(AyarHatasi::Eksik(anahtar))?;
#     ham.parse().map_err(|kaynak| AyarHatasi::Sayi {
#         anahtar,
#         deger: ham.clone(),
#         kaynak,
#     })
# }
#
const AZAMI_SINIR: u32 = 10_000;

pub fn ayarlari_coz(metin: &str) -> Result<Ayarlar, AyarHatasi> {
    let tablo = tabloya_ayristir(metin)?;
    let azami_baglanti: u32 = sayi_al(&tablo, "azami_baglanti")?;
    if azami_baglanti > AZAMI_SINIR {
        return Err(AyarHatasi::Aralik {
            anahtar: "azami_baglanti",
            deger: azami_baglanti,
            azami: AZAMI_SINIR,
        });
    }
    Ok(Ayarlar {
        ad: tablo.get("ad").cloned().unwrap_or_else(|| String::from("adsız")),
        port: sayi_al(&tablo, "port")?,
        azami_baglanti,
    })
}

fn main() {
    let hata = ayarlari_coz("port = 80\nazami_baglanti = 20000").expect_err("sınır aşıldı");
    println!("{hata}");
    println!("{}", hata.source().is_none());
}
```

Çıktı:

```text
`azami_baglanti` en çok 10000 olabilir, bulunan 20000
true
```

</details>

**2.** `sayi_al`'a satır numarası bağlamı ekleyin: Tabloyu `BTreeMap<String, (usize, String)>` yapın ve `Sayi` varyantına `satir` alanı koyun. `port = seksen` girdisinin mesajı ``satır 1: `port` sayı olmalı, ...`` biçiminde başlamalıdır.

<details>
<summary>İpucu ve çözüm</summary>

`satir_ayristir` satır numarasını (`no`) zaten alıyor; onu değerle birlikte tabloya taşımak yeterlidir.

```rust
# use std::collections::BTreeMap;
# use std::fmt;
# use std::num::ParseIntError;
# use std::str::FromStr;
#
# #[derive(Debug)]
# pub enum AyarHatasi {
#     Bicim { satir: usize, icerik: String },
#     Sayi { satir: usize, anahtar: &'static str, deger: String, kaynak: ParseIntError },
#     Eksik(&'static str),
# }
#
# impl fmt::Display for AyarHatasi {
#     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
#         match self {
#             AyarHatasi::Bicim { satir, icerik } => {
#                 write!(f, "satır {satir}: `anahtar=değer` bekleniyordu, bulunan {icerik:?}")
#             }
#             AyarHatasi::Sayi { satir, anahtar, deger, .. } => {
#                 write!(f, "satır {satir}: `{anahtar}` sayı olmalı, bulunan {deger:?}")
#             }
#             AyarHatasi::Eksik(anahtar) => write!(f, "zorunlu anahtar eksik: {anahtar}"),
#         }
#     }
# }
#
type Cift = (String, (usize, String)); // anahtar, (satır numarası, değer)

fn satir_ayristir(no: usize, satir: &str) -> Result<Option<Cift>, AyarHatasi> {
    let satir = satir.trim();
    if satir.is_empty() || satir.starts_with('#') {
        return Ok(None);
    }
    let (anahtar, deger) = satir.split_once('=').ok_or_else(|| AyarHatasi::Bicim {
        satir: no,
        icerik: satir.to_string(),
    })?;
    Ok(Some((anahtar.trim().to_string(), (no, deger.trim().to_string()))))
}

type Tablo = BTreeMap<String, (usize, String)>;

fn sayi_al<T>(tablo: &Tablo, anahtar: &'static str) -> Result<T, AyarHatasi>
where
    T: FromStr<Err = ParseIntError>,
{
    let (satir, ham) = tablo.get(anahtar).ok_or(AyarHatasi::Eksik(anahtar))?;
    ham.parse().map_err(|kaynak| AyarHatasi::Sayi {
        satir: *satir,
        anahtar,
        deger: ham.clone(),
        kaynak,
    })
}

# fn tabloya_ayristir(metin: &str) -> Result<Tablo, AyarHatasi> {
#     metin
#         .lines()
#         .enumerate()
#         .filter_map(|(i, satir)| satir_ayristir(i + 1, satir).transpose())
#         .collect()
# }
#
fn main() -> Result<(), AyarHatasi> {
    let tablo = tabloya_ayristir("port = seksen")?;
    if let Err(hata) = sayi_al::<u16>(&tablo, "port") {
        println!("{hata}");
    }
    Ok(())
}
```

Çıktı:

```text
satır 1: `port` sayı olmalı, bulunan "seksen"
```

Enum'da `Sayi { satir: usize, .. }` alanı, `Display` kolunda da `satır {satir}:` öneki eklendi. `tabloya_ayristir`'ın gövdesi değişmedi; yalnızca dönüş türü yeni tablo türü oldu.

</details>

**3.** `hatalari_topla`'yı döngü yerine iterator ile yazın. `(Vec<_>, Vec<_>)` üreten `partition(Result::is_ok)` kullanın, sonra iki yanı `Result::unwrap` ve `Result::unwrap_err` ile açın. Aynı girdi için sonuç değişmemelidir.

<details>
<summary>İpucu ve çözüm</summary>

`partition`, koşulu sağlayanları birinci, sağlamayanları ikinci koleksiyona koyar.[^partition] Ayırma `is_ok` ile yapıldığından birinci vektörde yalnızca `Ok`, ikincide yalnızca `Err` vardır; `unwrap` ve `unwrap_err` bu yüzden panik yaratamaz. Bu, kanıtlanabilir `unwrap` için iyi bir örnektir.

```rust
# use std::collections::BTreeMap;
# use std::error::Error;
# use std::fmt;
# use std::fs;
# use std::io;
# use std::num::ParseIntError;
# use std::path::Path;
# use std::str::FromStr;
#
# #[derive(Debug)]
# pub enum AyarHatasi {
#     Gc(io::Error),
#     Bicim { satir: usize, icerik: String },
#     Sayi { anahtar: &'static str, deger: String, kaynak: ParseIntError },
#     Eksik(&'static str),
# }
#
# impl fmt::Display for AyarHatasi {
#     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
#         match self {
#             AyarHatasi::Gc(e) => write!(f, "ayar dosyası okunamadı ({:?})", e.kind()),
#             AyarHatasi::Bicim { satir, icerik } => {
#                 write!(f, "satır {satir}: `anahtar=değer` bekleniyordu, bulunan {icerik:?}")
#             }
#             AyarHatasi::Sayi { anahtar, deger, .. } => {
#                 write!(f, "`{anahtar}` sayı olmalı, bulunan {deger:?}")
#             }
#             AyarHatasi::Eksik(anahtar) => write!(f, "zorunlu anahtar eksik: {anahtar}"),
#         }
#     }
# }
#
# fn satir_ayristir(no: usize, satir: &str) -> Result<Option<(String, String)>, AyarHatasi> {
#     let satir = satir.trim();
#     if satir.is_empty() || satir.starts_with('#') {
#         return Ok(None);
#     }
#     let (anahtar, deger) = satir.split_once('=').ok_or_else(|| AyarHatasi::Bicim {
#         satir: no,
#         icerik: satir.to_string(),
#     })?;
#     Ok(Some((anahtar.trim().to_string(), deger.trim().to_string())))
# }
#
fn hatalari_topla(metin: &str) -> (usize, Vec<AyarHatasi>) {
    let (gecerliler, hatalilar): (Vec<_>, Vec<_>) = metin
        .lines()
        .enumerate()
        .filter_map(|(i, satir)| satir_ayristir(i + 1, satir).transpose())
        .partition(Result::is_ok);

    let ciftler: Vec<_> = gecerliler.into_iter().map(Result::unwrap).collect();
    let hatalar = hatalilar.into_iter().map(Result::unwrap_err).collect();
    (ciftler.len(), hatalar)
}

fn main() {
    let (gecerli, hatalar) = hatalari_topla("port = 1\nbozuk satır\nad = x\nbir bozuk daha");
    println!("toplama: {gecerli} geçerli, {} hata", hatalar.len());
    for e in &hatalar {
        println!("  - {e}");
    }
}
```

Çıktı:

```text
toplama: 2 geçerli, 2 hata
  - satır 2: `anahtar=değer` bekleniyordu, bulunan "bozuk satır"
  - satır 4: `anahtar=değer` bekleniyordu, bulunan "bir bozuk daha"
```

</details>

## Özet

- Hata türünü enum olarak yazmak, çağıranın hataya göre karar vermesini sağlar. `Display` kullanıcıya, `Debug` geliştiriciye, `source` kök nedene götürür.
- `ok_or`, `ok_or_else`, `.ok()` ve `transpose`, `Option` ile `Result` arasındaki köprülerdir.
- `collect` ilk hatada durur; bütün hataları görmek için onları biriktirmek gerekir.
- Kütüphanede özel enum, yalnızca raporlanan yerde `Box<dyn Error>` uygundur. `expect`, kanıtını yazabildiğiniz yerlerde kullanılır.

Bu sayfadaki `sayi_al<T: FromStr<Err = ParseIntError>>`, trait sınırı ile ilişkili tür koşulunun bir örneğiydi. Sıradaki bölüm [9. Generic, Trait ve Lifetime](../intermediate/09-generics-traits-lifetimes.md) bunları baştan anlatıyor. Buradaki "içeride `Result`, en dışta rapor" kalıbının gerçek bir komut satırı aracındaki hâlini ise [minigrep](../intermediate/11-minigrep.md) bölümünde, `run() -> Result<(), Box<dyn Error>>` ve `process::exit(1)` ile göreceğiz.

## Kaynaklar

[^enum]: The Rust Programming Language, [Matches Are Exhaustive](https://doc.rust-lang.org/book/ch06-02-match.html#matches-are-exhaustive).
[^display]: std, [`Display`](https://doc.rust-lang.org/std/fmt/trait.Display.html) ve [`Debug`](https://doc.rust-lang.org/std/fmt/trait.Debug.html); std, [`io::ErrorKind`](https://doc.rust-lang.org/std/io/enum.ErrorKind.html).
[^source]: std, [`Error`: Error source](https://doc.rust-lang.org/std/error/trait.Error.html#error-source) ve [`Error::source`](https://doc.rust-lang.org/std/error/trait.Error.html#method.source).
[^successors]: std, [`iter::successors`](https://doc.rust-lang.org/std/iter/fn.successors.html).
[^map-err]: std, [`Result::map_err`](https://doc.rust-lang.org/std/result/enum.Result.html#method.map_err); std, [`From` trait'i](https://doc.rust-lang.org/std/convert/trait.From.html).
[^ok-or]: std, [`Option::ok_or`](https://doc.rust-lang.org/std/option/enum.Option.html#method.ok_or) ve [`Option::ok_or_else`](https://doc.rust-lang.org/std/option/enum.Option.html#method.ok_or_else); std, [`str::split_once`](https://doc.rust-lang.org/std/primitive.str.html#method.split_once).
[^transpose]: std, [`Result::transpose`](https://doc.rust-lang.org/std/result/enum.Result.html#method.transpose); std, [`Iterator::filter_map`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.filter_map).
[^collect]: std, [Collecting into `Result`](https://doc.rust-lang.org/std/result/index.html#collecting-into-result).
[^fromstr]: std, [`FromStr`](https://doc.rust-lang.org/std/str/trait.FromStr.html).
[^kopru]: std, [`Option`: The question mark operator](https://doc.rust-lang.org/std/option/index.html#the-question-mark-operator-) ve [Transforming contained values](https://doc.rust-lang.org/std/option/index.html#transforming-contained-values).
[^soru-from]: The Rust Programming Language, [The `?` Operator Shortcut](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#the--operator-shortcut); std, [`fs::read_to_string`](https://doc.rust-lang.org/std/fs/fn.read_to_string.html).
[^unwind]: The Rust Reference, [Panic](https://doc.rust-lang.org/reference/panic.html); std, [`Drop`: Panics](https://doc.rust-lang.org/std/ops/trait.Drop.html#panics).
[^dropguard]: std, [`mem::DropGuard`](https://doc.rust-lang.org/std/mem/struct.DropGuard.html); Rust 1.100.0 sürüm notları, [Stabilized APIs](https://github.com/rust-lang/rust/releases/tag/1.100.0).
[^box-dyn]: std, [`From<E> for Box<dyn Error>`](https://doc.rust-lang.org/std/error/trait.Error.html); The Rust Programming Language, [Where to Use the `?` Operator](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#where-to-use-the--operator).
[^downcast]: std, [`Error::downcast_ref`](https://doc.rust-lang.org/std/error/trait.Error.html#method.downcast_ref).
[^unwrap-olcut]: The Rust Programming Language, [Guidelines for Error Handling](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html#guidelines-for-error-handling) ve [When You Have More Information Than the Compiler](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html#when-you-have-more-information-than-the-compiler).
[^e0271]: hata dizini, [`E0271`](https://doc.rust-lang.org/error_codes/E0271.html) ve [`E0277`](https://doc.rust-lang.org/error_codes/E0277.html).
[^fromresidual]: std, [`FromResidual`](https://doc.rust-lang.org/std/ops/trait.FromResidual.html); rustc 1.99.0 kaynak kodu, [`option.rs`, `FromResidual<Option<convert::Infallible>>`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/option.rs#L2880).
[^e0004]: hata dizini, [`E0004`](https://doc.rust-lang.org/error_codes/E0004.html).
[^partition]: std, [`Iterator::partition`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.partition).
