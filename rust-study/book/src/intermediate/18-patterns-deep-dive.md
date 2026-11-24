# 18.1 Desenler: Derinleşme

[18. bölümde](18-patterns.md) desen sözdiziminin araçlarını tek tek gördük. Bu sayfada onları bir arada kullanacağız: Kullanıcının yazdığı komutları anlayan küçük bir depo programı yazacağız. Desenler "kaç parça var, ilki ne, içinde ne var?" sorularını tek ifadede sorar; unuttuğunuz bir durum olursa derleyici onu size gösterir.

Yol boyunca üç yeni konu ekleyeceğiz. Dilim desenleri bir dizinin uzunluğuna ve elemanlarına birlikte bakar. Bağlama modları, desenin bir parçayı taşıyacağını mı yoksa ödünç mü alacağını belirler. `matches!` makrosu ve eksiksizlik denetimi de kodu güvenle değiştirmeyi sağlar.

> [!NOTE]
> Bu sayfanın kodu `learn/patterns/src/deep_dive.rs` dosyasındadır. Depo kökünden `cargo run -p patterns` ile çalıştırabilir, `cargo test -p patterns` ile testlerini koşturabilirsiniz.

<details>
<summary>Bu sayfada geçen terimler</summary>

- **dilim deseni (slice pattern):** Bir dizinin uzunluğuna ve elemanlarına birlikte bakan desen.
- **`kalan @ ..`:** "Geri kalan elemanların hepsi". Hiç eleman kalmamış da olabilir.
- **bağlama modu (binding mode):** Desenin parçayı alması mı, yoksa ödünç bakması mı.
- **`ref` ve `ref mut`:** Desende "alma, ödünç bak" ve "alma, değiştirmek için ödünç al".
- **match ergonomics:** Ödünç bir değeri açınca içindekilerin de ödünç gelmesi.
- **`matches!`:** "Bu değer şu desene uyuyor mu?" sorusunun evet ya da hayır cevabı.
- **ayrıştırıcı (parser):** Düz yazıyı okuyup programın anlayacağı yapıya çeviren kod.
- **argüman:** Komutun yanında verilen ek bilgi. `ekle vida 40` komutunda `vida` ve `40`.
- **`Result`, `Ok`, `Err`:** "Ya sonuç ya hata" kutusu ve iki hâli.
- **özyineleme (recursion):** Bir fonksiyonun işi küçültüp kendini yeniden çağırması.
- **refactoring:** Davranışı değiştirmeden kodu yeniden düzenlemek.
- **`saturating_add`:** Taşacaksa en büyük değerde duran, başa sarmayan toplama.
- **`unsigned_abs`:** Mutlak değeri, taşma olmayacak biçimde işaretsiz sayı olarak almak.
- **never türü (`!`):** "Buradan geri dönülmez" demek; `return`, `break` ve `panic!` böyledir.

</details>

## Gerçek bir örnek: komut ayrıştırıcı

Depo programımız, kullanıcının verdiği satırı okuyup dört komuttan birine çevirecek:

| Komut | Örnek | Ne yapar? |
| --- | --- | --- |
| `ekle <ad> [adet]` | `ekle vida 40` | Depoya ürün ekler; adet yazılmazsa 1 sayılır |
| `sil <ad>` | `sil vida` | Ürünü depodan çıkarır |
| `listele` | `listele` | Depodakileri gösterir |
| `not <metin>` | `not raf 3 dolu` | Serbest bir not yazar; metin birkaç kelime olabilir |

Satırı kelimelere bölünce elimizde bir sıra parça olur ve soru şudur: Kaç parça var, ilki ne, gerisi ne?

### Desensiz yazım neden kırılgandır?

İlk akla gelen yol uzunluğa bakıp elemanları sıra numarasıyla almaktır. Bu iki iş ayrı satırlarda durduğu için birini unutmak kolaydır:

```rust,should_panic
fn sil(satir: &str) -> String {
    let parcalar: Vec<&str> = satir.split_whitespace().collect();
    if parcalar[0] == "sil" {
        format!("{} silinecek", parcalar[1]) // uzunluk denetlenmedi
    } else {
        String::from("bilinmeyen komut")
    }
}

fn main() {
    println!("{}", sil("sil vida"));
    println!("{}", sil("sil"));
}
```

İlk satır `vida silinecek` yazar, ikincisinde program panikler:

```text
thread 'main' panicked at src/main.rs:4:41:
index out of bounds: the len is 1 but the index is 1
```

### Dilim deseni: uzunluk ve içerik bir arada

Dilim deseni iki soruyu birden sorar: kaç parça olduğunu ve belli parçaların ne olduğunu. Uzunluk uymuyorsa o kola hiç girilmez, dolayısıyla olmayan bir elemana erişilemez:[^dilim]

```rust
fn anlat(parcalar: &[&str]) -> String {
    match parcalar {
        [] => String::from("boş"),
        ["listele"] => String::from("listelenecek"),
        ["sil", ad] => format!("{ad} silinecek"),
        [ilk, kalan @ ..] => format!("{ilk} ve {} parça daha", kalan.len()),
    }
}

fn main() {
    println!("{}", anlat(&[]));
    println!("{}", anlat(&["sil", "vida"]));
    println!("{}", anlat(&["sil"]));
    println!("{}", anlat(&["ekle", "vida", "40"]));
}
```

Çıktı:

```text
boş
vida silinecek
sil ve 0 parça daha
ekle ve 2 parça daha
```

### Ayrıştırıcı

Ayrıştırıcı düz yazıyı bir `Komut`'a, olmuyorsa bir `AyristirmaHatasi`'na çevirir:

```rust
#[derive(Debug, PartialEq)]
enum Komut {
    Ekle { ad: String, adet: u32 },
    Sil { ad: String },
    Listele,
    Not(String),
}

#[derive(Debug, PartialEq)]
enum AyristirmaHatasi {
    Bos,
    GecersizAdet(String),
    YanlisArguman { komut: String, verilen: usize },
    Bilinmeyen(String),
}

fn ayristir(satir: &str) -> Result<Komut, AyristirmaHatasi> {
    let parcalar: Vec<&str> = satir.split_whitespace().collect();
    match parcalar.as_slice() {
        [] => Err(AyristirmaHatasi::Bos),
        ["listele"] => Ok(Komut::Listele),
        ["sil", ad] => Ok(Komut::Sil { ad: ad.to_string() }),
        ["ekle", ad] => Ok(Komut::Ekle { ad: ad.to_string(), adet: 1 }),
        ["ekle", ad, adet] => {
            let Ok(adet @ 1..) = adet.parse::<u32>() else {
                return Err(AyristirmaHatasi::GecersizAdet(adet.to_string()));
            };
            Ok(Komut::Ekle { ad: ad.to_string(), adet })
        }
        ["not", ilk, kalan @ ..] => {
            let mut metin = ilk.to_string();
            for kelime in kalan {
                metin.push(' ');
                metin.push_str(kelime);
            }
            Ok(Komut::Not(metin))
        }
        [komut @ ("ekle" | "sil" | "listele" | "not"), argumanlar @ ..] => {
            Err(AyristirmaHatasi::YanlisArguman {
                komut: komut.to_string(),
                verilen: argumanlar.len(),
            })
        }
        [bilinmeyen, ..] => Err(AyristirmaHatasi::Bilinmeyen(bilinmeyen.to_string())),
    }
}
# fn main() {
#     assert_eq!(
#         ayristir("  ekle   vida  40 "),
#         Ok(Komut::Ekle { ad: String::from("vida"), adet: 40 })
#     );
#     assert_eq!(
#         ayristir("ekle vida 0"),
#         Err(AyristirmaHatasi::GecersizAdet(String::from("0")))
#     );
#     assert_eq!(ayristir("not raf 3 dolu"), Ok(Komut::Not(String::from("raf 3 dolu"))));
# }
```

Fonksiyonu adım adım okuyalım:

1. `as_slice()` gerekir, çünkü dilim deseni dizilere ve dilimlere uygulanır; `Vec`'e doğrudan uygulanmaz.
2. Kollar yukarıdan aşağıya denenir. Özel biçimler önce, genel yakalayıcılar sonra gelir.
3. `["ekle", ad]` hem uzunluğun iki olduğunu hem ilk elemanın `"ekle"` olduğunu sınar, ikinci elemana `ad` adını verir.
4. `["ekle", ad, adet]` kolunda `let ... else` kullanılır. `adet @ 1..` deseni sayının en az 1 olmasını da ister: Metin sayıya çevrilemezse ya da sayı 0 ise fonksiyon hatayla döner, uyarsa `adet` girintisiz devam eder.
5. `kalan @ ..` geri kalan elemanları bir alt dilim olarak bağlar; bu dilim boş da olabilir.[^dilim]
6. `komut @ ("ekle" | "sil" | "listele" | "not")`, seçeneklerden biri tuttuysa hangisi olduğunu saklar. Bu kola gelen satırda komut doğrudur ama argüman sayısı yanlıştır.

### Komutu uygulamak

Ayrıştırılan komut depo listesini değiştirir. Burada desenler struct'ları, seçenekleri ve dilimleri birlikte açar:

```rust
# #[derive(Debug, PartialEq)]
# enum Komut {
#     Ekle { ad: String, adet: u32 },
#     Sil { ad: String },
#     Listele,
#     Not(String),
# }
#
# #[derive(Debug, PartialEq)]
# enum AyristirmaHatasi {
#     Bos,
#     GecersizAdet(String),
#     YanlisArguman { komut: String, verilen: usize },
#     Bilinmeyen(String),
# }
#
#[derive(Debug, PartialEq)]
struct Kayit {
    ad: String,
    adet: u32,
}

fn uygula(envanter: &mut Vec<Kayit>, komut: Komut) -> String {
    match komut {
        Komut::Ekle { ad, adet } => {
            if let Some(Kayit { adet: mevcut, .. }) = envanter.iter_mut().find(|k| k.ad == ad) {
                *mevcut = mevcut.saturating_add(adet); // mevcut: &mut u32
                format!("{ad}: {mevcut} adet oldu")
            } else {
                let mesaj = format!("{ad} eklendi ({adet} adet)");
                envanter.push(Kayit { ad, adet });
                mesaj
            }
        }
        Komut::Sil { ad } => match envanter.iter().position(|k| k.ad == ad) {
            Some(sira) => {
                let Kayit { ad, adet } = envanter.remove(sira); // parçalarına taşır
                format!("{ad} silindi ({adet} adet vardı)")
            }
            None => format!("{ad} bulunamadı"),
        },
        Komut::Listele => match envanter.as_slice() {
            [] => String::from("envanter boş"),
            [tek] => format!("1 kalem: {}", tek.ad),
            [ilk, .., son] => format!("{} kalem: {} … {}", envanter.len(), ilk.ad, son.ad),
        },
        Komut::Not(metin) => format!("not alındı: {metin}"),
    }
}
# fn main() {
#     let mut e = Vec::new();
#     for _ in 0..2 {
#         uygula(&mut e, Komut::Ekle { ad: String::from("vida"), adet: 4_000_000_000 });
#     }
#     assert_eq!(e[0].adet, u32::MAX);
# }
```

İki ayrıntı önemlidir. `find` bir `Option<&mut Kayit>` döndürür; desen `&mut`'un içine kendiliğinden indiği için `mevcut` bir `&mut u32` olur (bunu [birazdan](#bağlama-modları-almak-mı-bakmak-mı) açıklayacağız). `envanter.remove(sira)` ise `Kayit`'ın sahipliğini verir; desen onu parçalarına *taşıyarak* ayırır.

Adet kullanıcı girdisinden geldiği için toplama `saturating_add` ile yapılır: Sonuç `u32`'ye sığmazsa en büyük değerde durur.[^saturating] `+=` yazsaydık taşma, taşma denetimi açık derlemelerde (varsayılan olarak `cargo build`) panik, kapalı derlemelerde (`--release`) sessizce başa sarma olurdu.[^tasma]

### Programın tamamı

```rust
# #[derive(Debug, PartialEq)]
# enum Komut {
#     Ekle { ad: String, adet: u32 },
#     Sil { ad: String },
#     Listele,
#     Not(String),
# }
#
# #[derive(Debug, PartialEq)]
# enum AyristirmaHatasi {
#     Bos,
#     GecersizAdet(String),
#     YanlisArguman { komut: String, verilen: usize },
#     Bilinmeyen(String),
# }
#
# fn ayristir(satir: &str) -> Result<Komut, AyristirmaHatasi> {
#     let parcalar: Vec<&str> = satir.split_whitespace().collect();
#     match parcalar.as_slice() {
#         [] => Err(AyristirmaHatasi::Bos),
#         ["listele"] => Ok(Komut::Listele),
#         ["sil", ad] => Ok(Komut::Sil { ad: ad.to_string() }),
#         ["ekle", ad] => Ok(Komut::Ekle { ad: ad.to_string(), adet: 1 }),
#         ["ekle", ad, adet] => {
#             let Ok(adet @ 1..) = adet.parse::<u32>() else {
#                 return Err(AyristirmaHatasi::GecersizAdet(adet.to_string()));
#             };
#             Ok(Komut::Ekle { ad: ad.to_string(), adet })
#         }
#         ["not", ilk, kalan @ ..] => {
#             let mut metin = ilk.to_string();
#             for kelime in kalan {
#                 metin.push(' ');
#                 metin.push_str(kelime);
#             }
#             Ok(Komut::Not(metin))
#         }
#         [komut @ ("ekle" | "sil" | "listele" | "not"), argumanlar @ ..] => {
#             Err(AyristirmaHatasi::YanlisArguman {
#                 komut: komut.to_string(),
#                 verilen: argumanlar.len(),
#             })
#         }
#         [bilinmeyen, ..] => Err(AyristirmaHatasi::Bilinmeyen(bilinmeyen.to_string())),
#     }
# }
#
# #[derive(Debug, PartialEq)]
# struct Kayit {
#     ad: String,
#     adet: u32,
# }
#
# fn uygula(envanter: &mut Vec<Kayit>, komut: Komut) -> String {
#     match komut {
#         Komut::Ekle { ad, adet } => {
#             if let Some(Kayit { adet: mevcut, .. }) = envanter.iter_mut().find(|k| k.ad == ad) {
#                 *mevcut = mevcut.saturating_add(adet);
#                 format!("{ad}: {mevcut} adet oldu")
#             } else {
#                 let mesaj = format!("{ad} eklendi ({adet} adet)");
#                 envanter.push(Kayit { ad, adet });
#                 mesaj
#             }
#         }
#         Komut::Sil { ad } => match envanter.iter().position(|k| k.ad == ad) {
#             Some(sira) => {
#                 let Kayit { ad, adet } = envanter.remove(sira);
#                 format!("{ad} silindi ({adet} adet vardı)")
#             }
#             None => format!("{ad} bulunamadı"),
#         },
#         Komut::Listele => match envanter.as_slice() {
#             [] => String::from("envanter boş"),
#             [tek] => format!("1 kalem: {}", tek.ad),
#             [ilk, .., son] => format!("{} kalem: {} … {}", envanter.len(), ilk.ad, son.ad),
#         },
#         Komut::Not(metin) => format!("not alındı: {metin}"),
#     }
# }
#
fn main() {
    let mut envanter = Vec::new();
    for satir in [
        "listele",
        "ekle vida 40",
        "ekle somun",
        "ekle vida 2",
        "not raf  3 dolu",
        "listele",
        "sil somun",
        "listele",
        "ekle vida çok",
        "sil",
        "boya kırmızı",
        "   ",
    ] {
        match ayristir(satir) {
            Ok(komut) => println!("{satir:?} → {}", uygula(&mut envanter, komut)),
            Err(hata) => println!("{satir:?} → hata: {hata:?}"),
        }
    }
}
```

Çıktı:

```text
"listele" → envanter boş
"ekle vida 40" → vida eklendi (40 adet)
"ekle somun" → somun eklendi (1 adet)
"ekle vida 2" → vida: 42 adet oldu
"not raf  3 dolu" → not alındı: raf 3 dolu
"listele" → 2 kalem: vida … somun
"sil somun" → somun silindi (1 adet vardı)
"listele" → 1 kalem: vida
"ekle vida çok" → hata: GecersizAdet("çok")
"sil" → hata: YanlisArguman { komut: "sil", verilen: 0 }
"boya kırmızı" → hata: Bilinmeyen("boya")
"   " → hata: Bos
```

`sil` satırı artık panik değil, anlamlı bir hata üretir.

## Bağlama modları: almak mı, bakmak mı?

Bir paketi açtığınızda içindekileri ya *alırsınız* (paket boşalır) ya da onlara yalnızca *bakarsınız*. Desenlerde de böyledir. Paketin kendisi üzerinde eşleştirirseniz parçalar dışarı taşınır; pakete `&` ile bakarak eşleştirirseniz parçalar da ödünç gelir. `&mut` ile bakarsanız parçaları yerinde değiştirebilirsiniz.

Bir desendeki her ad üç biçimden biriyle bağlanır; buna *bağlama modu* (binding mode) denir:[^baglama]

| Mod | Açık yazımı | Ne olur? |
| --- | --- | --- |
| taşıma | `ad` | Değer taşınır; tür `Copy` ise kopyalanır |
| `ref` | `ref ad` | Ada değerin `&` referansı bağlanır |
| `ref mut` | `ref mut ad` | Ada değerin `&mut` referansı bağlanır |

Değerin kendisi üzerinde eşleştirirken varsayılan mod taşımadır. Eşleştirilen şey bir referans, desen de referans olmayan bir desense (struct, enum, demet ya da dilim deseni gibi) derleyici referansın içine kendisi iner ve alt bağlamaları `ref` ya da `ref mut` yapar. *Match ergonomics* denen bu kural yüzünden günlük kodda `ref` yazmak nadiren gerekir:[^baglama][^ergonomi]

```rust
#[derive(Debug)]
struct Kayit {
    ad: String,
    adet: u32,
}

fn main() {
    let kayit = Kayit { ad: String::from("vida"), adet: 10 };

    let Kayit { ad, adet } = &kayit; // ad: &String, adet: &u32
    println!("ödünç: {ad} × {adet}");
    println!("kayıt yerinde: {kayit:?}");
}
```

Çıktı:

```text
ödünç: vida × 10
kayıt yerinde: Kayit { ad: "vida", adet: 10 }
```

### `ref` ve `ref mut`

`ref`, desenin içinde "taşıma, ödünç al" der. Alanları **farklı** modlarla bağlamak istediğinizde işe yarar. Aşağıda `ad` ödünç alınır, `adet` kopyalanır. `&mut` üzerinden eşleştirmenin açık yazımı da `ref mut`'tur:

```rust
#[derive(Debug)]
struct Kayit {
    ad: String,
    adet: u32,
}

fn main() {
    let mut kayit = Kayit { ad: String::from("vida"), adet: 10 };

    let Kayit { ref ad, adet } = kayit; // ad: &String, adet: u32 (kopya)
    println!("ref: {ad} × {adet}, kayıt yerinde: {}", kayit.ad);

    let Kayit { adet, .. } = &mut kayit; // adet: &mut u32
    *adet += 5;

    let Kayit { adet: ref mut sayac, .. } = kayit; // sayac: &mut u32
    *sayac *= 2;

    println!("güncel: {} × {}", kayit.ad, kayit.adet);
}
```

Çıktı:

```text
ref: vida × 10, kayıt yerinde: vida
güncel: vida × 30
```

### Karıştırmanın sınırı (2024 edition'ı)

Derleyici referansın içine kendisi indiyse, yani alt bağlamalar zaten ödünçse, araya ayrıca `ref`, `mut` ya da `&` yazmak 2024 edition'ında hatadır:[^edition]

```rust,compile_fail
# struct Kayit { ad: String, adet: u32 }
fn main() {
    let kayit = Kayit { ad: String::from("vida"), adet: 10 };
    let Kayit { ref ad, adet } = &kayit;
    println!("{ad} × {adet}");
}
```

Derleyicinin yanıtı:

```text
error: cannot explicitly borrow within an implicitly-borrowing pattern
  |     let Kayit { ref ad, adet } = &kayit;
  |                 ^^^ explicit `ref` binding modifier not allowed when implicitly borrowing
  |
  = note: for more information, see <https://doc.rust-lang.org/reference/patterns.html#binding-modes>
note: matching on a reference type with a non-reference pattern implicitly borrows the contents
  |     let Kayit { ref ad, adet } = &kayit;
  |         ^^^^^^^^^^^^^^^^^^^^^^ this non-reference pattern matches on a reference type `&_`
help: remove the unnecessary binding modifier
```

Modları alan alan seçmek istiyorsanız referansı açıkça açın. `&Kayit { .. }` deseni referansı çözer, varsayılan mod yeniden taşıma olur:

```rust
# struct Kayit { ad: String, adet: u32 }
fn main() {
    let kayit = Kayit { ad: String::from("vida"), adet: 10 };
    let &Kayit { ref ad, mut adet } = &kayit; // ad: &String, adet: u32 (kopya)
    adet += 1;
    println!("{ad} × {adet}, asıl kayıt: {}", kayit.adet);
}
```

Çıktı:

```text
vida × 11, asıl kayıt: 10
```

## İç içe desenler, `@` ve koruyucu koşul

İç içe geçen matruşka bebekleri düşünün. Desen hepsini tek bakışta açabilir: "Dış bebek `Tikla` olsun, içindeki `Nokta`'nın `x`'i 0 olsun, `y`'sine de ad ver." Aynı desende `@` ile bütüne ad verip parçalara da ayrı ad verebilir, sonuna bir koruyucu koşul ekleyebilirsiniz. İç içe `if`'lerle yazsaydınız her katman için ayrı bir denetim ve girinti gerekir, unutulan durum göze çarpmazdı.[^ic-ice]

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum FareTusu {
    Sol,
    Sag,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Nokta {
    x: i32,
    y: i32,
}

#[derive(Debug, PartialEq)]
enum Olay {
    Tus { karakter: char, ctrl: bool },
    Tikla { konum: Nokta, tus: FareTusu },
    Kaydir(i32),
}

fn yorumla(olay: &Olay) -> String {
    match olay {
        Olay::Tus { karakter: 'q', ctrl: true } => String::from("çıkış"),
        Olay::Tus { karakter: harf @ ('a'..='z' | 'A'..='Z'), ctrl: false } => {
            format!("harf: {harf}")
        }
        Olay::Tus { karakter, ctrl } => format!("tuş: {karakter:?} ctrl={ctrl}"),
        Olay::Tikla { konum: Nokta { x: 0, y }, tus: FareTusu::Sol } => {
            format!("sol kenar çubuğu, satır {y}")
        }
        Olay::Tikla { konum: nokta @ Nokta { x, y }, .. } if x == y => {
            format!("köşegende tık: {nokta:?}")
        }
        Olay::Tikla { konum: Nokta { x, y }, tus } => format!("tık ({x},{y}) {tus:?}"),
        Olay::Kaydir(miktar @ ..=-1) => format!("yukarı {}", miktar.unsigned_abs()),
        Olay::Kaydir(0) => String::from("kaydırma yok"),
        Olay::Kaydir(miktar @ 1..) => format!("aşağı {miktar}"),
    }
}
# fn main() {
#     assert_eq!(yorumla(&Olay::Kaydir(i32::MIN)), "yukarı 2147483648");
# }
```

Kolları sırayla okuyalım:

- `karakter: 'q', ctrl: true` alanları sabitlerle karşılaştırır; üç katman (enum, varyant, alan) tek satırdadır.
- `harf @ ('a'..='z' | 'A'..='Z')` aralığı sınar ve tutan değere `harf` adını verir. Parantez gereklidir; nedenini [sık yapılan hatalarda](#seçenekleri-paranteze-almamak-e0408) göreceğiz.
- `konum: Nokta { x: 0, y }` iç içe struct'ı açar, `x`'i 0 ile karşılaştırır, `y`'yi bağlar.
- `nokta @ Nokta { x, y }` hem bütünü hem parçaları bağlar. Koruyucu koşul iki bağlamayı karşılaştırır; desen bunu tek başına yapamaz, çünkü desen değerleri sabitlerle karşılaştırır, birbirleriyle değil.
- `Kaydir`'ın üç kolu koruyucu koşul yerine aralık kullanır. Üçü birlikte bütün `i32` değerlerini kapsar ve derleyici bunu kanıtlayabilir; `_` kolu gerekmez.

`unsigned_abs` da bilinçli bir seçimdir. `i32::MIN`'in mutlak değeri `i32`'ye sığmaz: `abs` çağrısı, taşma denetimi açık derlemede panikler, kapalıyken `i32::MIN`'in kendisini döndürür.[^abs] `unsigned_abs` sonucu `u32` olarak verdiği için böyle bir durum yoktur.[^unsigned-abs]

Olayları sırayla yorumlayalım:

```rust
# #[derive(Debug, Clone, Copy, PartialEq)]
# enum FareTusu {
#     Sol,
#     Sag,
# }
#
# #[derive(Debug, Clone, Copy, PartialEq)]
# struct Nokta {
#     x: i32,
#     y: i32,
# }
#
# #[derive(Debug, PartialEq)]
# enum Olay {
#     Tus { karakter: char, ctrl: bool },
#     Tikla { konum: Nokta, tus: FareTusu },
#     Kaydir(i32),
# }
#
# fn yorumla(olay: &Olay) -> String {
#     match olay {
#         Olay::Tus { karakter: 'q', ctrl: true } => String::from("çıkış"),
#         Olay::Tus { karakter: harf @ ('a'..='z' | 'A'..='Z'), ctrl: false } => {
#             format!("harf: {harf}")
#         }
#         Olay::Tus { karakter, ctrl } => format!("tuş: {karakter:?} ctrl={ctrl}"),
#         Olay::Tikla { konum: Nokta { x: 0, y }, tus: FareTusu::Sol } => {
#             format!("sol kenar çubuğu, satır {y}")
#         }
#         Olay::Tikla { konum: nokta @ Nokta { x, y }, .. } if x == y => {
#             format!("köşegende tık: {nokta:?}")
#         }
#         Olay::Tikla { konum: Nokta { x, y }, tus } => format!("tık ({x},{y}) {tus:?}"),
#         Olay::Kaydir(miktar @ ..=-1) => format!("yukarı {}", miktar.unsigned_abs()),
#         Olay::Kaydir(0) => String::from("kaydırma yok"),
#         Olay::Kaydir(miktar @ 1..) => format!("aşağı {miktar}"),
#     }
# }
#
fn main() {
    let olaylar = [
        Olay::Tus { karakter: 'q', ctrl: true },
        Olay::Tus { karakter: 'q', ctrl: false },
        Olay::Tus { karakter: '7', ctrl: false },
        Olay::Tikla { konum: Nokta { x: 0, y: 12 }, tus: FareTusu::Sol },
        Olay::Tikla { konum: Nokta { x: 5, y: 5 }, tus: FareTusu::Sag },
        Olay::Tikla { konum: Nokta { x: 3, y: 8 }, tus: FareTusu::Sag },
        Olay::Kaydir(-3),
        Olay::Kaydir(2),
    ];
    for olay in &olaylar {
        println!("{}", yorumla(olay));
    }
}
```

Çıktı:

```text
çıkış
harf: q
tuş: '7' ctrl=false
sol kenar çubuğu, satır 12
köşegende tık: Nokta { x: 5, y: 5 }
tık (3,8) Sag
yukarı 3
aşağı 2
```

### Koruyucu koşul `|` zincirinin tamamına uygulanır

`1 | 2 if izinli` yazımı `(1 | 2) if izinli` demektir; koşul yalnızca son seçeneğe ait değildir:[^koruyucu]

```rust
fn main() {
    let izinli = false;
    let sonuc = match 1 {
        1 | 2 if izinli => "bir ya da iki, izinli",
        _ => "diğer",
    };
    println!("guard önceliği: {sonuc}");
}
```

Çıktı:

```text
guard önceliği: diğer
```

Koşul yalnızca `2`'ye ait olsaydı sonuç "bir ya da iki, izinli" olurdu.

## Dilim desenleri

Bir tren düşünün: En önde lokomotif, en sonda son vagon, arada bilinmeyen sayıda vagon vardır. `[ilk, .., son]` "baştakini ve sondakini ver, aradakilere bakmıyorum" demektir. `[ilk, kalan @ ..]` ise "baştakini ayır, geri kalan vagonları da bana `kalan` adıyla ver" demektir; geriye hiç vagon kalmamış da olabilir.[^dilim]

```rust
fn ozetle(sayilar: &[i32]) -> String {
    match sayilar {
        [] => String::from("boş"),
        [tek] => format!("tek eleman: {tek}"),
        [ilk, son] => format!("iki eleman: {ilk} ve {son}"),
        [ilk, orta @ .., son] => format!("{ilk} … {son} (arada {} eleman)", orta.len()),
    }
}

// "boş" ve "baş + kuyruk" durumları bütün dilimleri kapsar
fn topla(sayilar: &[i32]) -> i32 {
    match sayilar {
        [] => 0,
        [ilk, kalan @ ..] => ilk + topla(kalan),
    }
}

fn main() {
    for dilim in [&[][..], &[7], &[1, 2], &[3, 1, 4, 1, 5]] {
        println!("{dilim:?} → {} (toplam {})", ozetle(dilim), topla(dilim));
    }
}
```

Çıktı:

```text
[] → boş (toplam 0)
[7] → tek eleman: 7 (toplam 7)
[1, 2] → iki eleman: 1 ve 2 (toplam 3)
[3, 1, 4, 1, 5] → 3 … 5 (arada 3 eleman) (toplam 14)
```

`orta` bir alt dilimdir, türü `&[i32]`'dir. `topla` fonksiyonu *özyinelemelidir*: Baştaki elemanı ayırır, kalanı aynı fonksiyona verir.

Aynı dilim deseninde `..` yalnızca bir kez geçebilir. İki tane olsaydı dilimin nereden bölüneceği belirsiz kalırdı:

```rust,compile_fail
fn ozetle(sayilar: &[i32]) -> String {
    match sayilar {
        [ilk, .., orta, .., son] => format!("{ilk} {orta} {son}"),
        _ => String::from("kısa"),
    }
}
```

Derleyicinin yanıtı:

```text
error: `..` can only be used once per slice pattern
  |         [ilk, .., orta, .., son] => format!("{ilk} {orta} {son}"),
  |               --        ^^ can only be used once per slice pattern
  |               |
  |               previously used here
```

### Dizi ile dilim arasındaki fark

Dizinin uzunluğu türünde yazılıdır; `[u8; 3]` her zaman üç elemanlıdır. Bu yüzden `[r, g, b]` deseni bir dizi için *çürütülemezdir*, düz `let` ile yazılır. Dilimin uzunluğu ise çalışma zamanında belli olur; aynı desen *çürütülebilir* olur ve `let ... else` gerekir:[^dilim]

```rust
fn main() {
    let renk: [u8; 3] = [255, 128, 0];
    let [r, g, b] = renk;

    let parcalar = ["tasi", "vida", "raf-2"];
    let [komut, argumanlar @ ..] = parcalar.as_slice() else {
        return;
    };
    println!("rgb = {r},{g},{b} | komut = {komut}, {} argüman", argumanlar.len());
}
```

Çıktı:

```text
rgb = 255,128,0 | komut = tasi, 2 argüman
```

## Eksiksizlik denetimi ve güvenli değişiklik

Oyuncağa yeni bir şekil, bir beşgen eklendiğini düşünün. Kapakta beşgen delik yoksa derleyici "bu parça nereye girecek?" diye sizi durdurur; verdiği hata listesi, güncellemeniz gereken yerlerin listesidir. Kapakta "ne gelirse yutan" bir delik (`_`) varsa beşgen sessizce oraya düşer ve kimse fark etmez.

`uygula` fonksiyonu dört varyantı da adıyla sayar, `_` kolu yoktur. `Komut`'a bir varyant eklendiği gün derleyici bizi oraya götürür:

```rust,compile_fail,E0004
enum Komut {
    Ekle { ad: String, adet: u32 },
    Sil { ad: String },
    Listele,
    Not(String),
    Tasi { ad: String, kaynak: String, hedef: String }, // yeni
}

fn ozet(komut: &Komut) -> String {
    match komut {
        Komut::Ekle { ad, adet } => format!("{adet} adet {ad} eklenecek"),
        Komut::Sil { ad } => format!("{ad} silinecek"),
        Komut::Listele => String::from("listelenecek"),
        Komut::Not(metin) => format!("not: {metin}"),
    }
}
```

Derleyicinin yanıtı:

```text
error[E0004]: non-exhaustive patterns: `&Komut::Tasi { .. }` not covered
  |     match komut {
  |           ^^^^^ pattern `&Komut::Tasi { .. }` not covered
```

Son kol `_ => String::from("desteklenmiyor")` olsaydı program derlenir, yeni komut sessizce oraya düşerdi. Kendi enum'unuzda `_` yazmadan önce şunu sorun: "Yeni bir varyant gelirse burada gerçekten aynı şeyi mi yapmak isterim?" Cevap evetse `_` doğrudur; hayırsa varyantları tek tek yazın. `ayristir`'daki gibi açık uçlu girdilerde yakalayıcı bir kol zaten zorunludur: Bir metin sonsuz sayıda değer alabilir, derleyici hangi komutların geçerli olduğunu bilemez.[^eksiksiz]

### `matches!`

`matches!` yalnızca bir soru sorar: "Bu değer şu desene uyar mı?" Cevabı `bool`'dur; koruyucu koşul da alabilir. `filter`, `any` ve `assert!` gibi `bool` beklenen yerlerde kullanışlıdır:[^matches]

```rust
struct Kayit {
    ad: String,
    adet: u32,
}

fn main() {
    let envanter: Vec<Kayit> = [("vida", 42), ("pul", 0), ("somun", 3)]
        .into_iter()
        .map(|(ad, adet)| Kayit { ad: ad.to_string(), adet }) // parametre de bir desendir
        .collect();

    let tukenen = envanter.iter().filter(|k| matches!(k, Kayit { adet: 0, .. })).count();
    let azalan: Vec<&str> = envanter
        .iter()
        .filter(|k| matches!(k.adet, 1..=5))
        .map(|k| k.ad.as_str())
        .collect();
    let kisa_adli_dolu =
        envanter.iter().any(|k| matches!(k, Kayit { ad, adet: 1.. } if ad.len() <= 4));
    println!("tükenen: {tukenen}, azalan: {azalan:?}, kısa adlı dolu var mı: {kisa_adli_dolu}");

    // "ilk kayıt var ve adedi sıfırdan büyük" tek desende
    let Some(Kayit { ad, adet: adet @ 1.. }) = envanter.first() else {
        println!("gösterilecek dolu kayıt yok");
        return;
    };
    println!("ilk dolu kayıt: {ad} ({adet})");
}
```

Çıktı:

```text
tükenen: 1, azalan: ["somun"], kısa adlı dolu var mı: true
ilk dolu kayıt: vida (42)
```

`matches!` içeride `_ => false` kolu olan bir `match`'e açılır.[^matches] Yani eksiksizlik korumasından bilerek vazgeçersiniz: Enum'a yeni bir varyant eklendiğinde `matches!` derlenmeye devam eder ve onun için sessizce `false` der.

## Sık yapılan hatalar

### Bakmak isterken taşımak (`E0382`)

Değerin kendisi üzerinde eşleştirmek, `Copy` olmayan alanları dışarı taşır:[^baglama]

```rust,compile_fail,E0382
# #[derive(Debug)]
# struct Kayit { ad: String, adet: u32 }
fn main() {
    let kayit = Kayit { ad: String::from("vida"), adet: 10 };
    let Kayit { ad, adet } = kayit;
    println!("{ad} × {adet}");
    println!("{kayit:?}");
}
```

Derleyicinin yanıtı:

```text
error[E0382]: borrow of partially moved value: `kayit`
  |     let Kayit { ad, adet } = kayit;
  |                 -- value partially moved here
  |     println!("{ad} × {adet}");
  |     println!("{kayit:?}");
  |                ^^^^^ value borrowed here after partial move
  |
  = note: partial move occurs because `kayit.ad` has type `String`, which does not implement the `Copy` trait
```

Yalnızca bakacaksanız referans üzerinde eşleştirin:

```rust
# #[derive(Debug)]
# struct Kayit { ad: String, adet: u32 }
fn main() {
    let kayit = Kayit { ad: String::from("vida"), adet: 10 };
    let Kayit { ad, adet } = &kayit;
    println!("{ad} × {adet}");
    println!("{kayit:?}");
}
```

Çıktı:

```text
vida × 10
Kayit { ad: "vida", adet: 10 }
```

### Kopyayı değiştirmeye çalışmak (`E0384`)

Desendeki `adet` yeni bir değişkendir ve `u32` kopyalandığı için `kayit.adet`'ten bağımsızdır:

```rust,compile_fail,E0384
# #[derive(Debug)]
# struct Kayit { ad: String, adet: u32 }
fn main() {
    let kayit = Kayit { ad: String::from("vida"), adet: 10 };
    match kayit {
        Kayit { adet, .. } => adet += 1,
    }
}
```

Derleyicinin yanıtı:

```text
error[E0384]: cannot assign twice to immutable variable `adet`
  |         Kayit { adet, .. } => adet += 1,
  |                 ----          ^^^^^^^^^ cannot assign twice to immutable variable
  |                 |
  |                 first assignment to `adet`
  |
help: consider making this binding mutable
help: to modify the original value, take a borrow instead
```

İlk öneriye uyup `mut adet` yazarsanız program derlenir ama kayıt değişmez; yalnızca kopya artar. İstediğiniz ikincisidir: Asıl alanı değiştirmek için `&mut` üzerinde eşleştirin.

```rust
# #[derive(Debug)]
# struct Kayit { ad: String, adet: u32 }
fn main() {
    let mut kayit = Kayit { ad: String::from("vida"), adet: 10 };
    match &mut kayit {
        Kayit { adet, .. } => *adet += 1, // adet: &mut u32
    }
    println!("{kayit:?}");
}
```

Çıktı:

```text
Kayit { ad: "vida", adet: 11 }
```

### Referansın içindekini taşımak (`E0507`)

`&Some(s)` deseni "referansı aç, sonra `s`'yi taşı" demektir. Ödünç alınmış bir değerin içi taşınamaz:

```rust,compile_fail,E0507
fn main() {
    let secili: &Option<String> = &Some(String::from("somun"));
    match secili {
        &Some(s) => println!("seçili: {s}"),
        &None => println!("seçim yok"),
    }
}
```

Derleyicinin yanıtı:

```text
error[E0507]: cannot move out of `secili` as enum variant `Some` which is behind a shared reference
  |     match secili {
  |           ^^^^^^
  |         &Some(s) => println!("seçili: {s}"),
  |               - data moved here because `s` has type `String`, which does not implement the `Copy` trait
  |
help: consider removing the borrow
```

`&`'i yazmazsanız match ergonomics devreye girer ve `s` bir `&String` olur:

```rust
fn main() {
    let secili: &Option<String> = &Some(String::from("somun"));
    match secili {
        Some(s) => println!("seçili: {s} ({} bayt)", s.len()),
        None => println!("seçim yok"),
    }
}
```

Çıktı:

```text
seçili: somun (5 bayt)
```

### Seçeneklerde farklı adlar bağlamak (`E0408`)

`|` ile birleşen her seçenek aynı adları bağlamalıdır. Aşağıda `Listele` tuttuğunda `ad` neye eşit olurdu?[^veya]

```rust,compile_fail,E0408
# #[derive(Debug, PartialEq)]
# enum Komut {
#     Ekle { ad: String, adet: u32 },
#     Sil { ad: String },
#     Listele,
#     Not(String),
# }
#
# #[derive(Debug, PartialEq)]
# enum AyristirmaHatasi {
#     Bos,
#     GecersizAdet(String),
#     YanlisArguman { komut: String, verilen: usize },
#     Bilinmeyen(String),
# }
#
fn yaz(komut: &Komut) {
    match komut {
        Komut::Ekle { ad, .. } | Komut::Listele => println!("{ad}"),
        _ => println!("başka"),
    }
}
```

Derleyicinin yanıtı:

```text
error[E0408]: variable `ad` is not bound in all patterns
  |         Komut::Ekle { ad, .. } | Komut::Listele => println!("{ad}"),
  |                       --         ^^^^^^^^^^^^^^ pattern doesn't bind `ad`
  |                       |
  |                       variable not in all patterns
```

Derleyici bu hatanın yanında `E0381` de üretir; kökü aynıdır. Ya kolları ayırın ya da yalnızca aynı adı aynı türle bağlayan seçenekleri birleştirin:

```rust
# #[derive(Debug, PartialEq)]
# enum Komut {
#     Ekle { ad: String, adet: u32 },
#     Sil { ad: String },
#     Listele,
#     Not(String),
# }
#
# #[derive(Debug, PartialEq)]
# enum AyristirmaHatasi {
#     Bos,
#     GecersizAdet(String),
#     YanlisArguman { komut: String, verilen: usize },
#     Bilinmeyen(String),
# }
#
fn yaz(komut: &Komut) {
    match komut {
        Komut::Ekle { ad, .. } | Komut::Sil { ad } => println!("kalem: {ad}"),
        Komut::Listele => println!("listele"),
        Komut::Not(metin) => println!("not: {metin}"),
    }
}

fn main() {
    yaz(&Komut::Sil { ad: String::from("vida") });
    yaz(&Komut::Listele);
}
```

Çıktı:

```text
kalem: vida
listele
```

### Seçenekleri paranteze almamak (`E0408`)

`@`, `|`'dan daha sıkı bağlanır; parantez yazmazsanız yalnızca ilk seçeneğe ait olur:[^oncelik]

```rust,compile_fail,E0408
fn yaz(karakter: char) {
    match karakter {
        harf @ 'a'..='z' | 'A'..='Z' => println!("harf: {harf}"),
        _ => println!("başka"),
    }
}
```

Derleyicinin yanıtı:

```text
error[E0408]: variable `harf` is not bound in all patterns
  |         harf @ 'a'..='z' | 'A'..='Z' => println!("harf: {harf}"),
  |         ----               ^^^^^^^^^ pattern doesn't bind `harf`
  |         |
  |         variable not in all patterns
```

Seçenekleri parantez içine alın:

```rust
fn yaz(karakter: char) {
    match karakter {
        harf @ ('a'..='z' | 'A'..='Z') => println!("harf: {harf}"),
        _ => println!("başka"),
    }
}

fn main() {
    yaz('R');
    yaz('7');
}
```

Çıktı:

```text
harf: R
başka
```

### Boş dilimi unutmak (`E0004`)

Boş dilim de bir dilimdir:

```rust,compile_fail,E0004
fn topla(sayilar: &[i32]) -> i32 {
    match sayilar {
        [ilk, kalan @ ..] => ilk + topla(kalan),
    }
}
```

Derleyicinin yanıtı:

```text
error[E0004]: non-exhaustive patterns: `&[]` not covered
  |     match sayilar {
  |           ^^^^^^^ pattern `&[]` not covered
```

Sıra numarasıyla yazsaydınız bu unutkanlık çalışma zamanında panik olurdu. Eksik kolu eklemek yeter:

```rust
fn topla(sayilar: &[i32]) -> i32 {
    match sayilar {
        [] => 0,
        [ilk, kalan @ ..] => ilk + topla(kalan),
    }
}

fn main() {
    println!("{} {}", topla(&[]), topla(&[1, 2, 3]));
}
```

Çıktı:

```text
0 6
```

### `Vec`'e dilim deseni yazmak (`E0529`)

Dilim deseni dizilere ve dilimlere uygulanır; `Vec` ikisi de değildir:

```rust,compile_fail,E0529
fn main() {
    let v = vec![1, 2, 3];
    match v {
        [ilk, ..] => println!("{ilk}"),
        _ => println!("boş"),
    }
}
```

Derleyicinin yanıtı:

```text
error[E0529]: expected an array or slice, found `Vec<{integer}>`
  |         [ilk, ..] => println!("{ilk}"),
  |         ^^^^^^^^^ pattern cannot match with input type `Vec<{integer}>`
```

`as_slice()` ile ya da `&v[..]` yazarak dilime çevirin:

```rust
fn main() {
    let v = vec![1, 2, 3];
    match v.as_slice() {
        [ilk, ..] => println!("{ilk}"),
        [] => println!("boş"),
    }
}
```

Çıktı:

```text
1
```

### Dizinin uzunluğunu tutturamamak (`E0527`)

Dizinin uzunluğu türünde yazılı olduğu için eksik desen derleme zamanında yakalanır:

```rust,compile_fail,E0527
fn main() {
    let renk: [u8; 3] = [255, 128, 0];
    let [r, g] = renk;
    println!("{r} {g}");
}
```

Derleyicinin yanıtı:

```text
error[E0527]: pattern requires 2 elements but array has 3
  |     let [r, g] = renk;
  |         ^^^^^^ expected 3 elements
```

Kullanmadığınız elemanları `_` ya da `..` ile açıkça yok sayın:

```rust
fn main() {
    let renk: [u8; 3] = [255, 128, 0];
    let [r, g, _] = renk;
    let [ilk, ..] = renk; // [r, g, ..] da olurdu
    println!("{r} {g} {ilk}");
}
```

Çıktı:

```text
255 128 255
```

## Akılda kalması gerekenler

- Dilim deseni uzunluk sınamasıyla eleman erişimini tek ifadede birleştirir; olmayan elemana erişmek imkânsızlaşır.
- Değerin kendisi üzerinde eşleştirmek taşır, `&` ve `&mut` üzerinde eşleştirmek ödünç alır. `ref` ve `ref mut` bu seçimi alan alan yapmak içindir.
- Desen değerleri sabitlerle karşılaştırır. İki bağlamayı ya da bir dış değişkeni karşılaştırmak için koruyucu koşul gerekir.
- Koruyucu koşul yerine aralık yazabiliyorsanız yazın; derleyici aralıkların her şeyi kapsadığını kanıtlayabilir.
- Kendi enum'unuzda `_` yazmak, eksiksizlik korumasından vazgeçmektir. `matches!` de böyledir.
- Kullanıcı girdisiyle hesap yaparken taşmayı düşünün: `saturating_add`, `unsigned_abs`.

## Alıştırmalar

Çözümlerin tamamı derlenip denenmiştir. Önce kendiniz deneyin.

### 1. `tasi` komutu

Ayrıştırıcıya `tasi <ad> <kaynak> <hedef>` komutunu ekleyin. Örneğin `tasi vida raf-1 raf-2` satırı `Ok(Tasi { .. })` vermeli; `tasi vida` ise `Err(YanlisArguman { komut: "tasi", verilen: 1 })` olmalıdır.

Önce yalnızca `Komut::Tasi` varyantını ekleyip `cargo check -p patterns` çalıştırın. Derleyicinin sizi nereye götürdüğünü ve nereye **götürmediğini** not edin.

<details>
<summary>İpucu ve çözüm</summary>

Derleyici sizi `uygula` içindeki `match`'e götürür (`E0004`). `ayristir`'a ise götürmez: Orada `&str` değerleri sınanır ve bunlar açık uçludur; yeni bir komut adı yazmayı unutmak derleyici için hata değildir. Hem yeni kolu hem `komut @ (..)` listesini elle güncellemeniz gerekir.

```rust
#[derive(Debug, PartialEq)]
enum Komut {
    Ekle { ad: String, adet: u32 },
    Sil { ad: String },
    Listele,
    Not(String),
    Tasi { ad: String, kaynak: String, hedef: String },
}
#
# #[derive(Debug, PartialEq)]
# enum AyristirmaHatasi {
#     Bos,
#     GecersizAdet(String),
#     YanlisArguman { komut: String, verilen: usize },
#     Bilinmeyen(String),
# }

fn ayristir(satir: &str) -> Result<Komut, AyristirmaHatasi> {
    let parcalar: Vec<&str> = satir.split_whitespace().collect();
    match parcalar.as_slice() {
        // öteki kollar aynı
#         [] => Err(AyristirmaHatasi::Bos),
#         ["listele"] => Ok(Komut::Listele),
#         ["sil", ad] => Ok(Komut::Sil { ad: ad.to_string() }),
#         ["ekle", ad] => Ok(Komut::Ekle { ad: ad.to_string(), adet: 1 }),
#         ["ekle", ad, adet] => {
#             let Ok(adet @ 1..) = adet.parse::<u32>() else {
#                 return Err(AyristirmaHatasi::GecersizAdet(adet.to_string()));
#             };
#             Ok(Komut::Ekle { ad: ad.to_string(), adet })
#         }
#         ["not", ilk, kalan @ ..] => {
#             let mut metin = ilk.to_string();
#             for kelime in kalan {
#                 metin.push(' ');
#                 metin.push_str(kelime);
#             }
#             Ok(Komut::Not(metin))
#         }
        ["tasi", ad, kaynak, hedef] => Ok(Komut::Tasi {
            ad: ad.to_string(),
            kaynak: kaynak.to_string(),
            hedef: hedef.to_string(),
        }),
        [komut @ ("ekle" | "sil" | "listele" | "not" | "tasi"), argumanlar @ ..] => {
            Err(AyristirmaHatasi::YanlisArguman {
                komut: komut.to_string(),
                verilen: argumanlar.len(),
            })
        }
        [bilinmeyen, ..] => Err(AyristirmaHatasi::Bilinmeyen(bilinmeyen.to_string())),
    }
}
#
# struct Kayit { ad: String, adet: u32 }

fn uygula(envanter: &mut Vec<Kayit>, komut: Komut) -> String {
    match komut {
        // öteki kollar aynı
#         Komut::Ekle { ad, adet } => {
#             let mesaj = format!("{ad} eklendi ({adet} adet)");
#             envanter.push(Kayit { ad, adet });
#             mesaj
#         }
#         Komut::Sil { ad } => format!("{ad} silindi"),
#         Komut::Listele => format!("{} kalem", envanter.len()),
#         Komut::Not(metin) => format!("not alındı: {metin}"),
        Komut::Tasi { ad, kaynak, hedef } => format!("{ad}: {kaynak} yerinden {hedef} yerine"),
    }
}

fn main() {
    println!("{:?}", ayristir("tasi vida"));
    match ayristir("tasi vida raf-1 raf-2") {
        Ok(komut) => println!("{}", uygula(&mut Vec::new(), komut)),
        Err(hata) => println!("hata: {hata:?}"),
    }
}
```

Çıktı:

```text
Err(YanlisArguman { komut: "tasi", verilen: 1 })
vida: raf-1 yerinden raf-2 yerine
```

</details>

### 2. Baştaki ve sondaki

`fn uclar(sayilar: &[i32]) -> Option<(i32, i32)>` yazın: En az iki eleman varsa ilkini ve sonuncusunu, tek eleman varsa onu iki kez, dilim boşsa `None` döndürsün. Sıra numarası ve `len()` kullanmayın.

<details>
<summary>İpucu ve çözüm</summary>

`ozetle`'deki kolların sırasına bakın. `&[i32]` üzerinde bağlanan adlar `&i32`'dir, bu yüzden `*` ile değeri alırız.

```rust
fn uclar(sayilar: &[i32]) -> Option<(i32, i32)> {
    match sayilar {
        [] => None,
        [tek] => Some((*tek, *tek)),
        [ilk, .., son] => Some((*ilk, *son)),
    }
}

fn main() {
    println!("{:?}", uclar(&[3, 1, 4]));
    println!("{:?}", uclar(&[7]));
    println!("{:?}", uclar(&[]));
}
```

Çıktı:

```text
Some((3, 4))
Some((7, 7))
None
```

`[ilk, .., son]` en az iki eleman ister; `..` sıfır elemana da uyduğu için iki elemanlı dilim de bu kola girer.

</details>

### 3. Ctrl ve rakam

`yorumla`'ya "ctrl ile basılan rakam" durumunu ekleyin: `Tus { karakter: '0'..='9', ctrl: true }` için `sekme N` döndürsün. Ctrl ile 3 `sekme 3` olmalı, ctrl'siz `'7'` yine `tuş: '7' ctrl=false` vermelidir.

<details>
<summary>İpucu ve çözüm</summary>

Rakamı `@` ile bağlayın. Yeni kolu nereye koyduğunuz sonucu değiştirir. Genel `Olay::Tus { karakter, ctrl }` kolunun altına koyarsanız derleyici "unreachable pattern" uyarısı verir ve kol hiç çalışmaz:

```text
warning: unreachable pattern
  |         Olay::Tus { karakter, ctrl } => format!("tuş: {karakter:?} ctrl={ctrl}"),
  |         ---------------------------- matches all the relevant values
  |         Olay::Tus { karakter: rakam @ '0'..='9', ctrl: true } => format!("sekme {rakam}"),
  |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no value can reach this
```

Kolu genel kolun üstüne koyun:

```rust
# #[derive(Debug, Clone, Copy, PartialEq)]
# enum FareTusu {
#     Sol,
#     Sag,
# }
#
# #[derive(Debug, Clone, Copy, PartialEq)]
# struct Nokta {
#     x: i32,
#     y: i32,
# }
#
# #[derive(Debug, PartialEq)]
# enum Olay {
#     Tus { karakter: char, ctrl: bool },
#     Tikla { konum: Nokta, tus: FareTusu },
#     Kaydir(i32),
# }

fn yorumla(olay: &Olay) -> String {
    match olay {
        Olay::Tus { karakter: 'q', ctrl: true } => String::from("çıkış"),
        Olay::Tus { karakter: rakam @ '0'..='9', ctrl: true } => format!("sekme {rakam}"),
        Olay::Tus { karakter: harf @ ('a'..='z' | 'A'..='Z'), ctrl: false } => {
            format!("harf: {harf}")
        }
        Olay::Tus { karakter, ctrl } => format!("tuş: {karakter:?} ctrl={ctrl}"),
        // Tikla ve Kaydir kolları aynı
#         Olay::Tikla { .. } => String::from("tık"),
#         Olay::Kaydir(miktar) => format!("kaydır {miktar}"),
    }
}

fn main() {
    println!("{}", yorumla(&Olay::Tus { karakter: '3', ctrl: true }));
    println!("{}", yorumla(&Olay::Tus { karakter: '7', ctrl: false }));
    println!("{}", yorumla(&Olay::Tus { karakter: 'q', ctrl: true }));
}
```

Çıktı:

```text
sekme 3
tuş: '7' ctrl=false
çıkış
```

</details>

## Özet

Desenler yalnızca bir kontrol akışı aracı değildir; sahipliği de belirler. Değerin kendisi üzerinde eşleştirmek parçaları taşır, referans üzerinde eşleştirmek ödünç alır. Dilim desenleri uzunlukla içeriği birlikte sınar, iç içe desenler katmanları tek bakışta açar, `@` bütüne de parçaya da ad verir. Eksiksizlik denetimi ise enum büyüdüğünde güncellenmesi gereken yerleri sıralar; `_` ve `matches!` bu korumadan vazgeçmek demektir.

Aynı "türe göre davranış" problemini [17. bölümde](17-oop.md) trait nesnesiyle çözmüştük. Kapalı bir varyant kümesine yeni *davranış* eklemek enum'da kolaydır (yeni bir fonksiyon ve `match`); açık bir kümeye yeni *tür* eklemek trait nesnesinde kolaydır. Aynı bölümdeki type-state kalıbı, [5.1. bölümdeki](../basics/05-enums-deep-dive.md) "geçersiz durumu temsil edilemez kıl" fikrinin tür düzeyindeki hâlidir.

Sırada [19. bölüm](19-advanced-features.md) var. Orada anlatılan never türü (`!`), `let ... else`'in `else` bloğunun neden `return`, `break` ya da `panic!` ile bitmek zorunda olduğunu ve bir `match` kolunda neden `panic!` yazılabildiğini açıklar.[^never]

## Kaynaklar

[^dilim]: The Rust Reference, [Slice patterns](https://doc.rust-lang.org/reference/patterns.html#slice-patterns) ve [Rest pattern](https://doc.rust-lang.org/reference/patterns.html#rest-pattern).
[^saturating]: Standart kütüphane, [`u32::saturating_add`](https://doc.rust-lang.org/std/primitive.u32.html#method.saturating_add).
[^tasma]: The Rust Reference, [Overflow](https://doc.rust-lang.org/reference/expressions/operator-expr.html#overflow).
[^baglama]: The Rust Reference, [Binding modes](https://doc.rust-lang.org/reference/patterns.html#binding-modes).
[^ergonomi]: RFC 2005, [Match ergonomics using default binding modes](https://rust-lang.github.io/rfcs/2005-match-ergonomics.html).
[^edition]: The Rust Edition Guide, [Match ergonomics reservations](https://doc.rust-lang.org/edition-guide/rust-2024/match-ergonomics.html).
[^ic-ice]: The Rust Programming Language, [19.3: Destructuring to Break Apart Values](https://doc.rust-lang.org/book/ch19-03-pattern-syntax.html#destructuring-to-break-apart-values) ve The Rust Reference, [Destructuring](https://doc.rust-lang.org/reference/patterns.html#destructuring).
[^abs]: Standart kütüphane, [`i32::abs`](https://doc.rust-lang.org/std/primitive.i32.html#method.abs).
[^unsigned-abs]: Standart kütüphane, [`i32::unsigned_abs`](https://doc.rust-lang.org/std/primitive.i32.html#method.unsigned_abs).
[^koruyucu]: The Rust Reference, [Match guards](https://doc.rust-lang.org/reference/expressions/match-expr.html#match-guards) ve The Rust Programming Language, [19.3: Adding Conditionals with Match Guards](https://doc.rust-lang.org/book/ch19-03-pattern-syntax.html#adding-conditionals-with-match-guards).
[^eksiksiz]: The Rust Programming Language, [6.2: Catch-All Patterns and the _ Placeholder](https://doc.rust-lang.org/book/ch06-02-match.html#catch-all-patterns-and-the-_-placeholder).
[^matches]: Standart kütüphane, [`matches!`](https://doc.rust-lang.org/std/macro.matches.html) ve `core` 1.99.0 kaynak kodu, [`macros/mod.rs`, `matches`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/macros/mod.rs#L431).
[^veya]: The Rust Reference, [Or-patterns](https://doc.rust-lang.org/reference/patterns.html#or-patterns); rustc 1.99.0 kaynak kodu, [`late.rs`, `check_consistent_bindings`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_resolve/src/late.rs#L4119).
[^oncelik]: The Rust Reference, [Precedence with other undelimited patterns](https://doc.rust-lang.org/reference/patterns.html#precedence-with-other-undelimited-patterns).
[^never]: The Rust Programming Language, [20.3: The Never Type That Never Returns](https://doc.rust-lang.org/book/ch20-03-advanced-types.html#the-never-type-that-never-returns).
