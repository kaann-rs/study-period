# 4.1 Struct'lar: Derinleşme

Önceki sayfa struct'ın sözdizimini anlattı: tanım, `..eski`, üç alıcı biçimi ve `derive(Debug)`. Bu sayfa aynı araçlarla bir **tasarım** yapıyor: küçük bir kullanıcı yönetimi modülü. Dört soruya yanıt arayacağız:

- Geçersiz bir değerin hiç kurulamaması nasıl sağlanır?
- Çok alanlı bir struct okunur ve eksiksiz biçimde nasıl kurulur?
- Bir metodun alıcısı çağırana hangi sahiplik sözünü verir?
- `..taban` yazınca tam olarak hangi alanlar taşınır?

Öğrenci kartı benzetmesi sürüyor; bu kez kartları basan **kayıt bürosuna** bakıyoruz. Bir kartı yalnızca büronun basmasına izin verirseniz, elinize geçen her kartın denetimden geçtiğine güvenebilirsiniz.

> [!NOTE]
> Bu sayfanın kodu `learn/structs/src/deep_dive.rs` dosyasındadır. Çıktısını `cargo run -p structs` ile görebilir, testlerini `cargo test -p structs` ile çalıştırabilirsiniz.

<details>
<summary>Bu sayfada geçen yeni terimler</summary>

- **modül (module, `mod`):** Kodun, kapısı olan bir odası. İçindekiler dışarıdan görünmeyebilir.
- **`pub` ve özel (private):** `pub` yazılan ögeyi modülün dışındaki kod da kullanabilir; yazılmayan özeldir.
- **gizlilik (privacy):** Neyin hangi modülden görülebileceğini belirleyen kural.
- **değişmez (invariant):** Programın hiçbir anında bozulmaması gereken koşul: "Bu kartta geçerli bir e-posta var."
- **doğrulamak (validate):** Bir bilginin kurallara uyup uymadığına bakmak.
- **newtype:** Tek bir değeri sarıp ona yeni bir tür adı veren struct: `Eposta(String)`.
- **kurucu kalıbı (builder pattern):** Çok alanlı bir değeri adım adım kuran yardımcı tür.
- **zincirleme (chaining):** Metotları art arda nokta ile çağırmak: `a.b().c().d()`.
- **`Result`, `Ok`, `Err`:** "Başarıldı, sonuç bu" ya da "olmadı, sebep bu" diyen tür.
- **`Option`, `Some`, `None`:** "Bir değer var" ya da "yok" diyen tür.
- **`?`:** "Hata varsa hemen onu döndür, yoksa içindeki değerle devam et" kısaltması.
- **`match`:** Bir değerin olası biçimlerini tek tek ele alan ifade.
- **tüketmek (consume):** Bir değerin sahipliğini alıp işini bitirmek; değer sonra kullanılamaz.
- **`Clone`, `PartialEq`, `Default`:** Klonlanabilme, `==` ile karşılaştırılabilme ve varsayılan değer üretebilme yetenekleri.
- **test ve `assert`:** Kodun doğru çalıştığını denetleyen küçük sınav ve o sınavın sorusu.

</details>

## Geçersiz değer kurulamasın

Önceki sayfadaki `Kullanici`'nın alanlarına her kod yazabiliyordu; `eposta: "asdf".into()` sorunsuz derlenirdi. Struct'ın alanlarını yalnızca tanımlandığı modüle açarsanız durum değişir. `pub` yazılmamış bir alana, alanın tanımlandığı modül ve onun alt modülleri dışındaki kod erişemez.[^gizlilik] Kural modülün içinde **bir kez** yazılır; dışarıda onu çiğneyen bir değer üretilemez.

Önce küçük bir örnek. Numarayı yalnızca `buro` modülü basabilir:

```rust
mod buro {
    pub struct Numara(u32); // içindeki sayı modüle özel

    impl Numara {
        pub fn yeni(n: u32) -> Option<Numara> {
            if n > 0 { Some(Numara(n)) } else { None } // denetim tek yerde
        }
    }
}

fn main() {
    println!("{}", buro::Numara::yeni(7).is_some());
    println!("{}", buro::Numara::yeni(0).is_some());
}
```

Çıktı:

```text
true
false
```

Dışarıdan numara basmaya kalkan kod derlenmez. Struct'ın adı `pub` olsa bile içindeki alan özel olduğu için yapıcısı da özeldir:

```rust,compile_fail,E0603
fn main() {
    let k = buro::Numara(0);
}
# mod buro {
#     pub struct Numara(u32);
#     impl Numara {
#         pub fn yeni(n: u32) -> Option<Numara> { if n > 0 { Some(Numara(n)) } else { None } }
#     }
# }
```

Derleyicinin yanıtı:

```text
error[E0603]: tuple struct constructor `Numara` is private
```

### Doğrulanmış metin: `Eposta`

Aynı kalıbı gerçekçi bir türe uygulayalım. `Eposta`, tek alanlı bir tuple struct'tır; bu kullanıma *newtype* denir.[^newtype] Koruduğu *değişmez* (invariant) şudur: Adres boşluksuzdur, küçük harflidir, tek bir `@` içerir, `@`'ten önce en az bir karakter vardır ve alan adında nokta bulunur.

```rust
mod hesap {
    #[derive(Debug, Clone, PartialEq)]
    pub struct Eposta(String); // alan özel

    impl Eposta {
        // Dışarıdan yeni bir Eposta üretmenin tek yolu
        pub fn ayristir(ham: &str) -> Result<Eposta, String> {
            let ham = ham.trim();
            let gecerli = ham.split_once('@').is_some_and(|(yerel, alan)| {
                !yerel.is_empty()
                    && alan.contains('.')
                    && !alan.contains('@') // tek '@'
                    && !ham.contains(char::is_whitespace)
            });
            if gecerli {
                Ok(Eposta(ham.to_lowercase()))
            } else {
                Err(format!("geçersiz e-posta: '{ham}'"))
            }
        }

        pub fn as_str(&self) -> &str {
            &self.0
        }

        pub fn alan_adi(&self) -> &str {
            self.0.split_once('@').map_or("", |(_, alan)| alan)
        }
    }
}

use hesap::Eposta;

fn main() {
    for ham in ["  Kaan@Ornek.COM ", "kaan.ornek.com", "@ornek.com"] {
        match Eposta::ayristir(ham) {
            Ok(e) => println!("kabul: {} (alan adı: {})", e.as_str(), e.alan_adi()),
            Err(hata) => println!("ret: {hata}"),
        }
    }
}
```

Çıktı:

```text
kabul: kaan@ornek.com (alan adı: ornek.com)
ret: geçersiz e-posta: 'kaan.ornek.com'
ret: geçersiz e-posta: '@ornek.com'
```

Üç başvuru geldi: İlki düzeltilip basıldı (baştaki ve sondaki boşluklar atıldı, harfler küçültüldü), ikisi reddedildi. `ayristir` sonucu `Result` ile bildirir: Kabul `Ok`, ret `Err` olur ve `match` ikisini ayrı ayrı ele alır. `Result` ve `match`'i [5.](05-enums.md) ve [8. bölümde](08-error-handling.md) ayrıntılı göreceğiz.

İki ayrıntıya dikkat edin:

- `as_str` salt okunur erişim verir: `&str` döndürür. `&mut String` verseydi dışarıdaki kod metni değiştirip değişmezi bozabilirdi.
- `alan_adi`, adreste `@` bulunduğunu varsayabilir. Değişmez bunu güvenceye aldığı için metot `Option` ya da `Result` değil, doğrudan `&str` döndürür; `map_or`'a verilen boş metne hiç düşülmez. Denetim bir kez yapılır, sonraki kod sadeleşir.

> [!NOTE]
> Gerçek e-posta doğrulaması daha katıdır; buradaki amaç kalıbı göstermektir.

### Olmasaydı ne bozulurdu?

Alan açık olsaydı tek bir `k.eposta = "asdf".into()` satırı, e-postayı geçerli sayan bütün kodu (posta gönderimini, alan adına göre gruplamayı) sessizce yanlışa düşürürdü. Denetimi her kullanım yerine koymak da çözüm değildir; er geç bir yerde unutulur. Denetimi türün kapısına koyunca atlanamaz hâle gelir: Kapıyı dolanmaya çalışan kod derlenmez. The Rust Programming Language de aynı yaklaşımı, doğrulama için özel türler tanımlamayı önerir.[^dogrulama]

### Değişmezi sınamak

Değişmezin her maddesi bir soru olarak yazılabilir. `assert!` koşulun doğru, `assert_eq!` iki değerin eşit olmasını ister; tutmazsa program paniklerle durur:

```rust
use hesap::Eposta;

fn main() {
    let e = Eposta::ayristir(" Kaan@Ornek.COM ").expect("geçerli bir adres");
    assert_eq!(e.as_str(), "kaan@ornek.com");
    assert_eq!(e.alan_adi(), "ornek.com");

    // Değişmezin her maddesini çiğneyen birer örnek
    for bozuk in [
        "kaan.ornek.com",    // '@' yok
        "@ornek.com",        // '@' öncesi boş
        "kaan@ornek",        // alan adında nokta yok
        "ka an@ornek.com",   // boşluk var
        "kaan@or@nek.com",   // iki '@'
    ] {
        assert!(Eposta::ayristir(bozuk).is_err(), "kabul edilmemeliydi: {bozuk}");
    }
    println!("bütün denetimler geçti");
}
# mod hesap {
#     #[derive(Debug, Clone, PartialEq)]
#     pub struct Eposta(String);
#     impl Eposta {
#         pub fn ayristir(ham: &str) -> Result<Eposta, String> {
#             let ham = ham.trim();
#             let gecerli = ham.split_once('@').is_some_and(|(yerel, alan)| {
#                 !yerel.is_empty() && alan.contains('.') && !alan.contains('@') && !ham.contains(char::is_whitespace)
#             });
#             if gecerli { Ok(Eposta(ham.to_lowercase())) } else { Err(format!("geçersiz e-posta: '{ham}'")) }
#         }
#         pub fn as_str(&self) -> &str { &self.0 }
#         pub fn alan_adi(&self) -> &str { self.0.split_once('@').map_or("", |(_, alan)| alan) }
#     }
# }
```

Çıktı:

```text
bütün denetimler geçti
```

Kaynak dosyada bu sorular `#[cfg(test)]` modülünde birim testi olarak durur; normal çalıştırmada derlenmez, `cargo test -p structs` ile çalışır. Testleri [10. bölümde](../intermediate/10-writing-tests.md) göreceğiz.

## Çok alanlı bir değeri okunur kurmak: Kurucu kalıbı

Bir kullanıcının adı, e-postası, yetkisi ve etkinlik durumu olsun. Hepsini tek bir fonksiyona sırayla verirsek çağrı okunmaz hâle gelir:

```rust
fn yeni(ad: &str, _eposta: &str, yonetici: bool, aktif: bool) -> String {
    format!("{ad}: yönetici={yonetici}, aktif={aktif}")
}

fn main() {
    println!("{}", yeni("Ada", "a@b.c", true, false));
    println!("{}", yeni("Ada", "a@b.c", false, true)); // sıra karıştı, derleyici sustu
}
```

Çıktı:

```text
Ada: yönetici=true, aktif=false
Ada: yönetici=false, aktif=true
```

Hangi `true`'nun ne demek olduğu çağrıdan anlaşılmaz. İki `bool`'un yerini karıştırsanız derleyici de uyaramaz, çünkü türleri aynıdır.[^bool] Rust'ta isimli argüman da yoktur. *Kurucu kalıbı* (builder pattern) her değere bir ad verir.[^builder]

Kurucuyu bir kart başvuru formu gibi düşünebilirsiniz. Formu göz göz doldurursunuz: "Adım Ada", "E-postam şu". En sonda formu teslim edersiniz; büro eksik ya da hatalı göz var mı diye bakar, yoksa kartı basar. Zorunlu gözler `Option` ile "verilmedi" (`None`) durumunda başlar:

```rust
mod hesap {
#     #[derive(Debug, Clone, PartialEq)]
#     pub struct Eposta(String);
#     impl Eposta {
#         pub fn ayristir(ham: &str) -> Result<Eposta, String> {
#             let ham = ham.trim();
#             let gecerli = ham.split_once('@').is_some_and(|(yerel, alan)| {
#                 !yerel.is_empty() && alan.contains('.') && !alan.contains('@') && !ham.contains(char::is_whitespace)
#             });
#             if gecerli { Ok(Eposta(ham.to_lowercase())) } else { Err(format!("geçersiz e-posta: '{ham}'")) }
#         }
#         pub fn as_str(&self) -> &str { &self.0 }
#         pub fn alan_adi(&self) -> &str { self.0.split_once('@').map_or("", |(_, alan)| alan) }
#     }
    #[derive(Debug)]
    pub struct Kullanici {
        ad: String,
        eposta: Eposta,
        yonetici: bool,
        giris_sayisi: u32,
    }

    #[derive(Debug, Default)]
    pub struct KullaniciKurucu {
        ad: Option<String>,
        eposta: Option<String>,
        yonetici: bool,
    }

    impl KullaniciKurucu {
        pub fn ad(mut self, ad: &str) -> Self {
            self.ad = Some(ad.trim().to_string());
            self
        }

        pub fn eposta(mut self, eposta: &str) -> Self {
            self.eposta = Some(eposta.to_string());
            self
        }

        pub fn yonetici(mut self, yonetici: bool) -> Self {
            self.yonetici = yonetici;
            self
        }

        pub fn olustur(self) -> Result<Kullanici, String> {
            let ad = self
                .ad
                .filter(|ad| !ad.is_empty())
                .ok_or_else(|| String::from("ad zorunlu"))?;
            let ham = self.eposta.ok_or_else(|| String::from("e-posta zorunlu"))?;
            Ok(Kullanici {
                ad,
                eposta: Eposta::ayristir(&ham)?,
                yonetici: self.yonetici,
                giris_sayisi: 0,
            })
        }
    }
    impl Kullanici {
        pub fn kurucu() -> KullaniciKurucu {
            KullaniciKurucu::default()
        }
#         pub fn ozet(&self) -> String {
#             let rol = if self.yonetici { "yönetici" } else { "üye" };
#             format!("{} <{}> [{rol}], {} giriş", self.ad, self.eposta.as_str(), self.giris_sayisi)
#         }
    }
}
# fn main() {}
```

`Kullanici`'nın alanları özeldir; modül dışında `Kullanici { .. }` yazılamaz. Kullanıcı kurmanın tek yolu `Kullanici::kurucu()` ile bir form alıp doldurmaktır. `olustur` içindeki `?`, hata varsa hemen onu döndürür, yoksa içindeki değeri verip devam eder. `olustur(self)` kurucuyu tükettiği için içindeki `String`'ler `Kullanici`'ya **taşınır**; klonlama yapılmaz.

```rust
use hesap::Kullanici;

fn main() {
    let kaan = Kullanici::kurucu()
        .ad("Kaan")
        .eposta("Kaan@Ornek.com")
        .yonetici(true)
        .olustur();
    let eksik = Kullanici::kurucu().ad("Ada").olustur();
    let bozuk = Kullanici::kurucu().ad("Ada").eposta("ada(at)ornek").olustur();
    let adsiz = Kullanici::kurucu().ad("   ").eposta("a@b.c").olustur();

    println!("{} {}", kaan.is_ok(), eksik.is_ok());
    for sonuc in [kaan, eksik, bozuk, adsiz] {
        match sonuc {
            Ok(k) => println!("kuruldu: {}", k.ozet()),
            Err(hata) => println!("kurulamadı: {hata}"),
        }
    }
}
# mod hesap {
#     #[derive(Debug, Clone, PartialEq)]
#     pub struct Eposta(String);
#     impl Eposta {
#         pub fn ayristir(ham: &str) -> Result<Eposta, String> {
#             let ham = ham.trim();
#             let gecerli = ham.split_once('@').is_some_and(|(yerel, alan)| {
#                 !yerel.is_empty() && alan.contains('.') && !alan.contains('@') && !ham.contains(char::is_whitespace)
#             });
#             if gecerli { Ok(Eposta(ham.to_lowercase())) } else { Err(format!("geçersiz e-posta: '{ham}'")) }
#         }
#         pub fn as_str(&self) -> &str { &self.0 }
#         pub fn alan_adi(&self) -> &str { self.0.split_once('@').map_or("", |(_, alan)| alan) }
#     }
#     #[derive(Debug)]
#     pub struct Kullanici { ad: String, eposta: Eposta, yonetici: bool, giris_sayisi: u32 }
#     #[derive(Debug, Default)]
#     pub struct KullaniciKurucu { ad: Option<String>, eposta: Option<String>, yonetici: bool }
#     impl KullaniciKurucu {
#         pub fn ad(mut self, ad: &str) -> Self { self.ad = Some(ad.trim().to_string()); self }
#         pub fn eposta(mut self, eposta: &str) -> Self { self.eposta = Some(eposta.to_string()); self }
#         pub fn yonetici(mut self, yonetici: bool) -> Self { self.yonetici = yonetici; self }
#         pub fn olustur(self) -> Result<Kullanici, String> {
#             let ad = self.ad.filter(|ad| !ad.is_empty()).ok_or_else(|| String::from("ad zorunlu"))?;
#             let ham = self.eposta.ok_or_else(|| String::from("e-posta zorunlu"))?;
#             Ok(Kullanici { ad, eposta: Eposta::ayristir(&ham)?, yonetici: self.yonetici, giris_sayisi: 0 })
#         }
#     }
#     impl Kullanici {
#         pub fn kurucu() -> KullaniciKurucu { KullaniciKurucu::default() }
#         pub fn ad(&self) -> &str { &self.ad }
#         pub fn eposta(&self) -> &Eposta { &self.eposta }
#         pub fn giris_sayisi(&self) -> u32 { self.giris_sayisi }
#         pub fn ozet(&self) -> String {
#             let rol = if self.yonetici { "yönetici" } else { "üye" };
#             format!("{} <{}> [{rol}], {} giriş", self.ad, self.eposta.as_str(), self.giris_sayisi)
#         }
#     }
# }
```

Çıktı:

```text
true false
kuruldu: Kaan <kaan@ornek.com> [yönetici], 0 giriş
kurulamadı: e-posta zorunlu
kurulamadı: geçersiz e-posta: 'ada(at)ornek'
kurulamadı: ad zorunlu
```

İlk form tamdı ve kart basıldı. İkincide e-posta gözü boş kaldı, üçüncüde e-posta geçersizdi. Sonuncuda ad yalnızca boşluktan oluşuyordu: `ad` ayarlayıcısı boşlukları kırpar, `olustur` boş adı eler. Hiç verilmeyen `yonetici` ise varsayılan değerini (`false`) alır.

### Ayarlayıcılar neden `self` alıp `Self` döndürür?

Her adım kurucuyu devralır, günceller ve bir sonrakine teslim eder. Arada klonlama da ödünç de yoktur. Ayarlayıcılar `&mut Self` döndürseydi zincirin ucunda elde yalnızca bir `&mut` kalırdı; `olustur(self)` ise sahiplik ister. Ödünç alınan şey tüketilemez:

```rust,compile_fail,E0507
#[derive(Default)]
struct KullaniciKurucu {
    ad: Option<String>,
}

impl KullaniciKurucu {
    fn ad(&mut self, ad: &str) -> &mut Self {
        self.ad = Some(ad.to_string());
        self
    }

    fn olustur(self) -> Option<String> {
        self.ad
    }
}

fn main() {
    let k = KullaniciKurucu::default().ad("Kaan").olustur();
}
```

Derleyicinin yanıtı:

```text
error[E0507]: cannot move out of a mutable reference
   |
18 |     let k = KullaniciKurucu::default().ad("Kaan").olustur();
   |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ --------- value moved due to this method call
   |             |
   |             move occurs because value has type `KullaniciKurucu`, which does not implement the `Copy` trait
   |
note: `KullaniciKurucu::olustur` takes ownership of the receiver `self`, which moves value
```

> [!NOTE]
> İki tasarım da yaygındır. Son adım kurucunun verisini tüketmek zorunda değilse Rust API Guidelines, `&mut self` alıp `&mut Self` döndüren ayarlayıcıları önerir; `std::process::Command` böyledir. Son adım sahiplik istiyorsa, burada olduğu gibi, ayarlayıcıların hepsinin `self` alıp `Self` döndürmesi önerilir.[^builder]

Bedeli şudur: Tüketilen kurucu ikinci kez kullanılamaz. İki kullanıcı gerekiyorsa iki zincir yazılır; bunu aşağıda, [sık hatalarda](#kurucuyu-iki-kez-kullanmak-e0382) göreceğiz.

## Alıcı seçimi bir sahiplik sözleşmesidir

Okulu bitiren öğrencinin öğrenci kartı geri alınır, yerine mezun kartı verilir. Mezun kartıyla derse giriş kaydı yapılamaz; kartın üzerinde öyle bir bölüm yoktur. `pasiflestir(self)` tam olarak bunu yapar: Eski kartı alır ve **başka türden** bir kart verir.

```rust
mod hesap {
#     #[derive(Debug, Clone, PartialEq)]
#     pub struct Eposta(String);
#     impl Eposta {
#         pub fn ayristir(ham: &str) -> Result<Eposta, String> {
#             let ham = ham.trim();
#             let gecerli = ham.split_once('@').is_some_and(|(yerel, alan)| {
#                 !yerel.is_empty() && alan.contains('.') && !alan.contains('@') && !ham.contains(char::is_whitespace)
#             });
#             if gecerli { Ok(Eposta(ham.to_lowercase())) } else { Err(format!("geçersiz e-posta: '{ham}'")) }
#         }
#         pub fn as_str(&self) -> &str { &self.0 }
#         pub fn alan_adi(&self) -> &str { self.0.split_once('@').map_or("", |(_, alan)| alan) }
#     }
#     #[derive(Debug)]
#     pub struct Kullanici { ad: String, eposta: Eposta, yonetici: bool, giris_sayisi: u32 }
    impl Kullanici {
        pub fn ad(&self) -> &str {
            &self.ad
        }
        pub fn eposta(&self) -> &Eposta {
            &self.eposta
        }
        pub fn giris_sayisi(&self) -> u32 {
            self.giris_sayisi
        }

        pub fn ozet(&self) -> String {
            let rol = if self.yonetici { "yönetici" } else { "üye" };
            let Kullanici { ad, eposta, giris_sayisi, .. } = self;
            format!("{ad} <{}> [{rol}], {giris_sayisi} giriş", eposta.as_str())
        }

        pub fn giris_kaydet(&mut self) {
            self.giris_sayisi += 1;
        }

        pub fn pasiflestir(self) -> PasifKullanici {
            PasifKullanici {
                ad: self.ad,
                eposta: self.eposta,
                son_giris_sayisi: self.giris_sayisi,
            }
        }
    }

    #[derive(Debug)]
    pub struct PasifKullanici {
        ad: String,
        eposta: Eposta,
        son_giris_sayisi: u32,
    }

    impl PasifKullanici {
        pub fn ozet(&self) -> String {
            let PasifKullanici { ad, eposta, son_giris_sayisi } = self;
            format!("[pasif] {ad} <{}>, son giriş sayısı {son_giris_sayisi}", eposta.as_str())
        }

        pub fn yeniden_etkinlestir(self) -> Kullanici {
            Kullanici {
                ad: self.ad,
                eposta: self.eposta,
                yonetici: false, // yetki geri gelmez
                giris_sayisi: 0,
            }
        }
    }
}
# fn main() {}
```

Alanları okuyan metotların adı alanla aynıdır (`ad`, `eposta`); Rust'ta bu tür metotlara genellikle `get_` öneki konmaz.[^getter] `ad` metodunun döndürdüğü `&str`, `self` ödüncüne bağlıdır ve kullanıcıdan uzun yaşayamaz. `ozet`'teki `..`, desende geri kalan alanları göz ardı eder; desenleri [5. bölümde](05-enums.md) göreceğiz.

`pasiflestir` bir `Kullanici` alır, `PasifKullanici` verir. `PasifKullanici`'da `giris_kaydet` metodu yoktur; onu istemek mümkün değildir:

```rust
use hesap::Kullanici;

fn main() {
    let Ok(mut ada) = Kullanici::kurucu().ad("Ada").eposta("ada@ornek.com").olustur() else {
        return;
    };
    ada.giris_kaydet();
    ada.giris_kaydet();
    println!("{} | {} harf", ada.ozet(), ada.ad().chars().count());
    println!("{} girişin alan adı: {}", ada.giris_sayisi(), ada.eposta().alan_adi());

    let pasif = ada.pasiflestir(); // ada burada tüketildi
    println!("{}", pasif.ozet());

    let geri = pasif.yeniden_etkinlestir();
    println!("{}", geri.ozet());
}
# mod hesap {
#     #[derive(Debug, Clone, PartialEq)]
#     pub struct Eposta(String);
#     impl Eposta {
#         pub fn ayristir(ham: &str) -> Result<Eposta, String> {
#             let ham = ham.trim();
#             let gecerli = ham.split_once('@').is_some_and(|(yerel, alan)| {
#                 !yerel.is_empty() && alan.contains('.') && !alan.contains('@') && !ham.contains(char::is_whitespace)
#             });
#             if gecerli { Ok(Eposta(ham.to_lowercase())) } else { Err(format!("geçersiz e-posta: '{ham}'")) }
#         }
#         pub fn as_str(&self) -> &str { &self.0 }
#         pub fn alan_adi(&self) -> &str { self.0.split_once('@').map_or("", |(_, alan)| alan) }
#     }
#     #[derive(Debug)]
#     pub struct Kullanici { ad: String, eposta: Eposta, yonetici: bool, giris_sayisi: u32 }
#     #[derive(Debug, Default)]
#     pub struct KullaniciKurucu { ad: Option<String>, eposta: Option<String>, yonetici: bool }
#     impl KullaniciKurucu {
#         pub fn ad(mut self, ad: &str) -> Self { self.ad = Some(ad.trim().to_string()); self }
#         pub fn eposta(mut self, eposta: &str) -> Self { self.eposta = Some(eposta.to_string()); self }
#         pub fn yonetici(mut self, yonetici: bool) -> Self { self.yonetici = yonetici; self }
#         pub fn olustur(self) -> Result<Kullanici, String> {
#             let ad = self.ad.filter(|ad| !ad.is_empty()).ok_or_else(|| String::from("ad zorunlu"))?;
#             let ham = self.eposta.ok_or_else(|| String::from("e-posta zorunlu"))?;
#             Ok(Kullanici { ad, eposta: Eposta::ayristir(&ham)?, yonetici: self.yonetici, giris_sayisi: 0 })
#         }
#     }
#     impl Kullanici {
#         pub fn kurucu() -> KullaniciKurucu { KullaniciKurucu::default() }
#         pub fn ad(&self) -> &str { &self.ad }
#         pub fn eposta(&self) -> &Eposta { &self.eposta }
#         pub fn giris_sayisi(&self) -> u32 { self.giris_sayisi }
#         pub fn ozet(&self) -> String {
#             let rol = if self.yonetici { "yönetici" } else { "üye" };
#             format!("{} <{}> [{rol}], {} giriş", self.ad, self.eposta.as_str(), self.giris_sayisi)
#         }
#         pub fn giris_kaydet(&mut self) { self.giris_sayisi += 1; }
#         pub fn pasiflestir(self) -> PasifKullanici {
#             PasifKullanici { ad: self.ad, eposta: self.eposta, son_giris_sayisi: self.giris_sayisi }
#         }
#     }
#     #[derive(Debug)]
#     pub struct PasifKullanici { ad: String, eposta: Eposta, son_giris_sayisi: u32 }
#     impl PasifKullanici {
#         pub fn ozet(&self) -> String {
#             let PasifKullanici { ad, eposta, son_giris_sayisi } = self;
#             format!("[pasif] {ad} <{}>, son giriş sayısı {son_giris_sayisi}", eposta.as_str())
#         }
#         pub fn yeniden_etkinlestir(self) -> Kullanici {
#             Kullanici { ad: self.ad, eposta: self.eposta, yonetici: false, giris_sayisi: 0 }
#         }
#     }
# }
```

Çıktı:

```text
Ada <ada@ornek.com> [üye], 2 giriş | 3 harf
2 girişin alan adı: ornek.com
[pasif] Ada <ada@ornek.com>, son giriş sayısı 2
Ada <ada@ornek.com> [üye], 0 giriş
```

`let Ok(mut ada) = ... else { ... };` satırı *let-else* deyimidir: Desen tutarsa `ada` bağlanır, tutmazsa `else` bloğu akıştan çıkar.[^let-else]

Teslim edilen kartla giriş yapmak isteyen kod derlenmez:

```rust,compile_fail,E0382
use hesap::Kullanici;

fn mezun_et(mut ada: Kullanici) {
    let mezun = ada.pasiflestir();
    println!("{}", mezun.ozet());
    ada.giris_kaydet();
}
# mod hesap { // kısaltılmış hâli: e-posta denetimi yok
#     pub struct Kullanici { ad: String, giris_sayisi: u32 }
#     pub struct PasifKullanici { ad: String }
#     #[derive(Default)]
#     pub struct KullaniciKurucu { ad: String, eposta: String }
#     impl KullaniciKurucu {
#         pub fn ad(mut self, ad: &str) -> Self { self.ad = ad.to_string(); self }
#         pub fn eposta(mut self, eposta: &str) -> Self { self.eposta = eposta.to_string(); self }
#         pub fn olustur(self) -> Result<Kullanici, String> { Ok(Kullanici { ad: self.ad, giris_sayisi: 0 }) }
#     }
#     impl Kullanici {
#         pub fn kurucu() -> KullaniciKurucu { KullaniciKurucu::default() }
#         pub fn giris_sayisi(&self) -> u32 { self.giris_sayisi }
#         pub fn giris_kaydet(&mut self) { self.giris_sayisi += 1; }
#         pub fn pasiflestir(self) -> PasifKullanici { PasifKullanici { ad: self.ad } }
#     }
#     impl PasifKullanici {
#         pub fn ozet(&self) -> String { format!("[pasif] {}", self.ad) }
#     }
# }
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0382]: borrow of moved value: `ada`
  |
3 | fn mezun_et(mut ada: Kullanici) {
  |             ------- move occurs because `ada` has type `Kullanici`, which does not implement the `Copy` trait
4 |     let mezun = ada.pasiflestir();
  |                     ------------- `ada` moved due to this method call
5 |     println!("{}", mezun.ozet());
6 |     ada.giris_kaydet();
  |     ^^^ value borrowed here after move
```

Bu hata bir engel değil, tasarımın **özelliğidir**: Mantık hatası program çalışmadan yakalanır. Tek bir kartın üzerine `aktif: bool` yazsaydık, "hayır" yazan kartla giriş yapılmasını ancak çalışma zamanında, her seferinde elle denetleyerek engelleyebilirdik.

Modül iki kuralı daha güvenceye alır. `giris_sayisi` yalnızca `giris_kaydet` ile artar; aktif bir hesabın giriş sayısı azalmaz. Yeniden etkinleştirme ise bilinçli olarak sıfırdan yeni bir hesap dönemi başlatır ve yetkiyi geri vermez. İki kural da tek yerde, bu modülde yazılıdır. Aşağıdaki denetim yönetici bir kullanıcının dönüşte üye olduğunu gösterir:

```rust
use hesap::Kullanici;

fn main() -> Result<(), String> {
    let mut k = Kullanici::kurucu().ad(" Kaan ").eposta("kaan@ornek.com").yonetici(true).olustur()?;
    assert_eq!(k.ad(), "Kaan"); // boşluklar kırpıldı
    k.giris_kaydet();

    let pasif = k.pasiflestir();
    assert_eq!(pasif.ozet(), "[pasif] Kaan <kaan@ornek.com>, son giriş sayısı 1");

    let geri = pasif.yeniden_etkinlestir();
    assert_eq!(geri.ozet(), "Kaan <kaan@ornek.com> [üye], 0 giriş");
    println!("{}", geri.ozet());
    Ok(())
}
# mod hesap {
#     #[derive(Debug, Clone, PartialEq)]
#     pub struct Eposta(String);
#     impl Eposta {
#         pub fn ayristir(ham: &str) -> Result<Eposta, String> {
#             let ham = ham.trim();
#             let gecerli = ham.split_once('@').is_some_and(|(yerel, alan)| {
#                 !yerel.is_empty() && alan.contains('.') && !alan.contains('@') && !ham.contains(char::is_whitespace)
#             });
#             if gecerli { Ok(Eposta(ham.to_lowercase())) } else { Err(format!("geçersiz e-posta: '{ham}'")) }
#         }
#         pub fn as_str(&self) -> &str { &self.0 }
#         pub fn alan_adi(&self) -> &str { self.0.split_once('@').map_or("", |(_, alan)| alan) }
#     }
#     #[derive(Debug)]
#     pub struct Kullanici { ad: String, eposta: Eposta, yonetici: bool, giris_sayisi: u32 }
#     #[derive(Debug, Default)]
#     pub struct KullaniciKurucu { ad: Option<String>, eposta: Option<String>, yonetici: bool }
#     impl KullaniciKurucu {
#         pub fn ad(mut self, ad: &str) -> Self { self.ad = Some(ad.trim().to_string()); self }
#         pub fn eposta(mut self, eposta: &str) -> Self { self.eposta = Some(eposta.to_string()); self }
#         pub fn yonetici(mut self, yonetici: bool) -> Self { self.yonetici = yonetici; self }
#         pub fn olustur(self) -> Result<Kullanici, String> {
#             let ad = self.ad.filter(|ad| !ad.is_empty()).ok_or_else(|| String::from("ad zorunlu"))?;
#             let ham = self.eposta.ok_or_else(|| String::from("e-posta zorunlu"))?;
#             Ok(Kullanici { ad, eposta: Eposta::ayristir(&ham)?, yonetici: self.yonetici, giris_sayisi: 0 })
#         }
#     }
#     impl Kullanici {
#         pub fn kurucu() -> KullaniciKurucu { KullaniciKurucu::default() }
#         pub fn ad(&self) -> &str { &self.ad }
#         pub fn eposta(&self) -> &Eposta { &self.eposta }
#         pub fn giris_sayisi(&self) -> u32 { self.giris_sayisi }
#         pub fn ozet(&self) -> String {
#             let rol = if self.yonetici { "yönetici" } else { "üye" };
#             format!("{} <{}> [{rol}], {} giriş", self.ad, self.eposta.as_str(), self.giris_sayisi)
#         }
#         pub fn giris_kaydet(&mut self) { self.giris_sayisi += 1; }
#         pub fn pasiflestir(self) -> PasifKullanici {
#             PasifKullanici { ad: self.ad, eposta: self.eposta, son_giris_sayisi: self.giris_sayisi }
#         }
#     }
#     #[derive(Debug)]
#     pub struct PasifKullanici { ad: String, eposta: Eposta, son_giris_sayisi: u32 }
#     impl PasifKullanici {
#         pub fn ozet(&self) -> String {
#             let PasifKullanici { ad, eposta, son_giris_sayisi } = self;
#             format!("[pasif] {ad} <{}>, son giriş sayısı {son_giris_sayisi}", eposta.as_str())
#         }
#         pub fn yeniden_etkinlestir(self) -> Kullanici {
#             Kullanici { ad: self.ad, eposta: self.eposta, yonetici: false, giris_sayisi: 0 }
#         }
#     }
# }
```

Çıktı:

```text
Kaan <kaan@ornek.com> [üye], 0 giriş
```

### Alıcıyı seçerken sorulacak soru

| Metot ne yapıyor? | Alıcı | Çağıran için anlamı |
| --- | --- | --- |
| Yalnızca okuyor | `&self` | En az kısıt; varsayılan tercih |
| Yerinde değiştiriyor | `&mut self` | Çağıran `mut` bir bağlama tutmalı |
| Değeri başka bir şeye dönüştürüyor, eskisi anlamsız | `self` | Eskisinin kullanımı derlenmez |

## `..taban`: Hangi alan taşınır, hangisi kalır?

Önceki sayfada `..eski` yazınca eski karttaki fotoğrafın sökülüp yeni karta geçtiğini gördük. İncelik şudur: Yalnızca `..`'nın **gerçekten aldığı** alanlar taşınır. Taşıma alan alan yapılır; dokunulmayan alanın eski karttan çıkması için bir neden yoktur.[^guncelleme]

```rust
struct Ayar {
    tema: String,
    boyut: u32,
}

fn main() {
    let eski = Ayar { tema: String::from("açık"), boyut: 20 };

    let a = Ayar { tema: String::from("koyu"), ..eski }; // yalnızca boyut (Copy) alındı
    println!("{} {} / eski.tema: {}", a.tema, a.boyut, eski.tema);

    let b = Ayar { boyut: 50, ..eski }; // tema taşındı
    println!("{} {} / eski.boyut: {}", b.tema, b.boyut, eski.boyut);
}
```

Çıktı:

```text
koyu 20 / eski.tema: açık
açık 50 / eski.boyut: 20
```

İlk güncellemeden sonra `eski` bütün hâlde durur; ikincisinden sonra yalnızca `eski.boyut` okunabilir. Daha büyük bir struct'ta iki durum iç içe geçer:

```rust
#[derive(Debug, Clone, PartialEq)]
struct Ayarlar {
    tema: String,
    dil: String,
    bildirim: bool,
    sayfa_boyutu: u32,
}

impl Default for Ayarlar {
    fn default() -> Self {
        Ayarlar {
            tema: String::from("açık"),
            dil: String::from("tr"),
            bildirim: true,
            sayfa_boyutu: 20,
        }
    }
}

fn main() {
    let taban = Ayarlar::default();
    let koyu = Ayarlar {
        tema: String::from("koyu"),
        ..taban // dil taşındı; bildirim ve sayfa_boyutu kopyalandı
    };
    println!("{} / {} / taban.tema hâlâ: {}", koyu.tema, koyu.dil, taban.tema);
}
```

Çıktı:

```text
koyu / tr / taban.tema hâlâ: açık
```

`taban.tema`'ya dokunulmadı, yerine yenisi yazıldı; bu yüzden tek başına okunabilir. Ama `taban` **bütün olarak** artık kullanılamaz, çünkü `dil` alanı taşınmıştır:

```rust,compile_fail,E0382
fn main() {
    let taban = Ayarlar::default();
    let koyu = Ayarlar { tema: String::from("koyu"), ..taban };
    println!("{taban:?}");
}
# #[derive(Debug, Clone, PartialEq)]
# struct Ayarlar { tema: String, dil: String, bildirim: bool, sayfa_boyutu: u32 }
# impl Default for Ayarlar {
#     fn default() -> Self {
#         Ayarlar { tema: String::from("açık"), dil: String::from("tr"), bildirim: true, sayfa_boyutu: 20 }
#     }
# }
```

Derleyicinin yanıtı:

```text
error[E0382]: borrow of partially moved value: `taban`
  |
3 |     let koyu = Ayarlar { tema: String::from("koyu"), ..taban };
  |                ----------------------------------------------- value partially moved here
4 |     println!("{taban:?}");
  |                ^^^^^ value borrowed here after partial move
  |
  = note: partial move occurs because `taban.dil` has type `String`, which does not implement the `Copy` trait
```

Kaynağı bütün tutmanın üç yolu vardır:

```rust
fn main() {
    let taban = Ayarlar::default();

    // 1. Copy olmayan alanların hepsini kendiniz verin
    let ingilizce = Ayarlar {
        tema: taban.tema.clone(),
        dil: String::from("en"),
        ..taban
    };
    println!("{} / taban bütün: {}", ingilizce.dil, taban == Ayarlar::default());

    // 2. Kopyayı açıkça isteyin
    let genis = Ayarlar { sayfa_boyutu: 50, ..taban.clone() };

    // 3. Varsayılanlar ve birkaç fark
    let sessiz = Ayarlar { bildirim: false, ..Default::default() };

    println!(
        "{} / {} / taban: {}",
        genis.sayfa_boyutu, sessiz.bildirim, taban.sayfa_boyutu
    );
}
# #[derive(Debug, Clone, PartialEq)]
# struct Ayarlar { tema: String, dil: String, bildirim: bool, sayfa_boyutu: u32 }
# impl Default for Ayarlar {
#     fn default() -> Self {
#         Ayarlar { tema: String::from("açık"), dil: String::from("tr"), bildirim: true, sayfa_boyutu: 20 }
#     }
# }
```

Çıktı:

```text
en / taban bütün: true
50 / false / taban: 20
```

1. `..taban` yalnızca `Copy` alanları alır; kaynak bütün kalır. Bedeli, taşınabilecek her alanı elle yazmaktır.
2. `..taban.clone()` bütün struct'ı klonlar; maliyeti kodda görünür.
3. `..Default::default()` geçici bir değerden alır. Geçici değerin başka sahibi olmadığından taşınması kimseyi etkilemez. "Varsayılanlar ve birkaç fark" demenin yerleşik yolu budur; struct'a yeni bir alan eklendiğinde bu satırlar değişmeden derlenmeye devam eder.[^default]

## Sık hatalar

### Doğrulamayı atlamak (`E0603`, `E0616`, `E0451`)

Modül dışından doğrulanmamış bir değer elde etmenin üç yolu denenebilir ve üçü de derlenmez. Birincisi, yapıcıyı doğrudan çağırmaktır:

```rust,compile_fail,E0603
fn main() {
    let e = hesap::Eposta(String::from("asdf"));
}
# mod hesap {
#     pub struct Eposta(String);
#     pub struct Kullanici { ad: String, eposta: Eposta, yonetici: bool, giris_sayisi: u32 }
# }
```

Derleyicinin yanıtı:

```text
error[E0603]: tuple struct constructor `Eposta` is private
```

İkincisi, kurulmuş bir değerin alanını okumak ya da yazmaktır:

```rust,compile_fail,E0616
use hesap::{Eposta, Kullanici};

fn oku(e: &Eposta) {
    println!("{}", e.0);
}

fn boz(k: &mut Kullanici) {
    k.eposta = "asdf".into();
}
# mod hesap {
#     pub struct Eposta(String);
#     pub struct Kullanici { ad: String, eposta: Eposta, yonetici: bool, giris_sayisi: u32 }
# }
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0616]: field `0` of struct `Eposta` is private
error[E0616]: field `eposta` of struct `Kullanici` is private
```

Üçüncüsü, struct'ı alanlarını tek tek vererek kurmaktır:

```rust,compile_fail,E0451
use hesap::{Eposta, Kullanici};

fn kur(eposta: Eposta) -> Kullanici {
    Kullanici { ad: String::from("Ada"), eposta, yonetici: true, giris_sayisi: 0 }
}
# mod hesap {
#     pub struct Eposta(String);
#     pub struct Kullanici { ad: String, eposta: Eposta, yonetici: bool, giris_sayisi: u32 }
# }
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0451]: fields `ad`, `eposta`, `yonetici` and `giris_sayisi` of struct `Kullanici` are private
```

Doğru yol, modülün verdiği kapıdan girmektir:

```rust
use hesap::{Eposta, Kullanici};

fn main() -> Result<(), String> {
    let e = Eposta::ayristir("Ada@Ornek.com")?;
    let k = Kullanici::kurucu().ad("Ada").eposta(e.as_str()).olustur()?;
    println!("{}", k.ozet());
    Ok(())
}
# mod hesap {
#     #[derive(Debug, Clone, PartialEq)]
#     pub struct Eposta(String);
#     impl Eposta {
#         pub fn ayristir(ham: &str) -> Result<Eposta, String> {
#             let ham = ham.trim();
#             let gecerli = ham.split_once('@').is_some_and(|(yerel, alan)| {
#                 !yerel.is_empty() && alan.contains('.') && !alan.contains('@') && !ham.contains(char::is_whitespace)
#             });
#             if gecerli { Ok(Eposta(ham.to_lowercase())) } else { Err(format!("geçersiz e-posta: '{ham}'")) }
#         }
#         pub fn as_str(&self) -> &str { &self.0 }
#         pub fn alan_adi(&self) -> &str { self.0.split_once('@').map_or("", |(_, alan)| alan) }
#     }
#     #[derive(Debug)]
#     pub struct Kullanici { ad: String, eposta: Eposta, yonetici: bool, giris_sayisi: u32 }
#     #[derive(Debug, Default)]
#     pub struct KullaniciKurucu { ad: Option<String>, eposta: Option<String>, yonetici: bool }
#     impl KullaniciKurucu {
#         pub fn ad(mut self, ad: &str) -> Self { self.ad = Some(ad.trim().to_string()); self }
#         pub fn eposta(mut self, eposta: &str) -> Self { self.eposta = Some(eposta.to_string()); self }
#         pub fn yonetici(mut self, yonetici: bool) -> Self { self.yonetici = yonetici; self }
#         pub fn olustur(self) -> Result<Kullanici, String> {
#             let ad = self.ad.filter(|ad| !ad.is_empty()).ok_or_else(|| String::from("ad zorunlu"))?;
#             let ham = self.eposta.ok_or_else(|| String::from("e-posta zorunlu"))?;
#             Ok(Kullanici { ad, eposta: Eposta::ayristir(&ham)?, yonetici: self.yonetici, giris_sayisi: 0 })
#         }
#     }
#     impl Kullanici {
#         pub fn kurucu() -> KullaniciKurucu { KullaniciKurucu::default() }
#         pub fn ad(&self) -> &str { &self.ad }
#         pub fn eposta(&self) -> &Eposta { &self.eposta }
#         pub fn giris_sayisi(&self) -> u32 { self.giris_sayisi }
#         pub fn ozet(&self) -> String {
#             let rol = if self.yonetici { "yönetici" } else { "üye" };
#             format!("{} <{}> [{rol}], {} giriş", self.ad, self.eposta.as_str(), self.giris_sayisi)
#         }
#     }
# }
```

Çıktı:

```text
Ada <ada@ornek.com> [üye], 0 giriş
```

### Gizliliği `..` ile aşmaya çalışmak (`E0451`)

`..` alanlara adıyla erişir: Struct'ın bütün alanları, yazılmayanlar da dâhil, o noktada görünür olmalıdır.[^guncelleme] Bu yüzden ilk başlıktaki koruma bu sözdizimiyle de delinemez:

```rust,compile_fail,E0451
use hesap::Kullanici;

fn degistir(k: Kullanici) -> Kullanici {
    Kullanici { ad: String::from("Can"), ..k }
}
# mod hesap {
#     pub struct Eposta(String);
#     pub struct Kullanici { ad: String, eposta: Eposta, yonetici: bool, giris_sayisi: u32 }
# }
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0451]: fields `ad`, `eposta`, `yonetici` and `giris_sayisi` of struct `Kullanici` are private
```

Modülün dışındaysanız yeni değeri yine kurucuyla, yani denetimden geçerek üretirsiniz:

```rust
use hesap::Kullanici;

fn degistir(k: &Kullanici) -> Result<Kullanici, String> {
    Kullanici::kurucu().ad("Can").eposta(k.eposta().as_str()).olustur()
}

fn main() -> Result<(), String> {
    let ada = Kullanici::kurucu().ad("Ada").eposta("ada@ornek.com").olustur()?;
    println!("{}", degistir(&ada)?.ozet());
    Ok(())
}
# mod hesap {
#     #[derive(Debug, Clone, PartialEq)]
#     pub struct Eposta(String);
#     impl Eposta {
#         pub fn ayristir(ham: &str) -> Result<Eposta, String> {
#             let ham = ham.trim();
#             let gecerli = ham.split_once('@').is_some_and(|(yerel, alan)| {
#                 !yerel.is_empty() && alan.contains('.') && !alan.contains('@') && !ham.contains(char::is_whitespace)
#             });
#             if gecerli { Ok(Eposta(ham.to_lowercase())) } else { Err(format!("geçersiz e-posta: '{ham}'")) }
#         }
#         pub fn as_str(&self) -> &str { &self.0 }
#         pub fn alan_adi(&self) -> &str { self.0.split_once('@').map_or("", |(_, alan)| alan) }
#     }
#     #[derive(Debug)]
#     pub struct Kullanici { ad: String, eposta: Eposta, yonetici: bool, giris_sayisi: u32 }
#     #[derive(Debug, Default)]
#     pub struct KullaniciKurucu { ad: Option<String>, eposta: Option<String>, yonetici: bool }
#     impl KullaniciKurucu {
#         pub fn ad(mut self, ad: &str) -> Self { self.ad = Some(ad.trim().to_string()); self }
#         pub fn eposta(mut self, eposta: &str) -> Self { self.eposta = Some(eposta.to_string()); self }
#         pub fn yonetici(mut self, yonetici: bool) -> Self { self.yonetici = yonetici; self }
#         pub fn olustur(self) -> Result<Kullanici, String> {
#             let ad = self.ad.filter(|ad| !ad.is_empty()).ok_or_else(|| String::from("ad zorunlu"))?;
#             let ham = self.eposta.ok_or_else(|| String::from("e-posta zorunlu"))?;
#             Ok(Kullanici { ad, eposta: Eposta::ayristir(&ham)?, yonetici: self.yonetici, giris_sayisi: 0 })
#         }
#     }
#     impl Kullanici {
#         pub fn kurucu() -> KullaniciKurucu { KullaniciKurucu::default() }
#         pub fn ad(&self) -> &str { &self.ad }
#         pub fn eposta(&self) -> &Eposta { &self.eposta }
#         pub fn giris_sayisi(&self) -> u32 { self.giris_sayisi }
#         pub fn ozet(&self) -> String {
#             let rol = if self.yonetici { "yönetici" } else { "üye" };
#             format!("{} <{}> [{rol}], {} giriş", self.ad, self.eposta.as_str(), self.giris_sayisi)
#         }
#     }
# }
```

Çıktı:

```text
Can <ada@ornek.com> [üye], 0 giriş
```

Bu yolda yetki ve giriş sayısı sıfırdan başlar. Var olan kullanıcının yalnızca adını yerinde değiştirmek istiyorsanız bunu modülün içinde, kuralı denetleyen bir metot olarak yazmanız gerekir; üçüncü alıştırma tam olarak bunu istiyor.

### Kurucuyu iki kez kullanmak (`E0382`)

`olustur(self)` kurucuyu tüketir. Elde kalan `b` ikinci kez kullanılamaz:

```rust,compile_fail,E0382
use hesap::Kullanici;

fn main() {
    let b = Kullanici::kurucu().ad("Kaan");
    let k1 = b.olustur();
    let k2 = b.olustur();
}
# mod hesap { // kısaltılmış hâli: e-posta denetimi yok
#     pub struct Kullanici { ad: String, giris_sayisi: u32 }
#     pub struct PasifKullanici { ad: String }
#     #[derive(Default)]
#     pub struct KullaniciKurucu { ad: String, eposta: String }
#     impl KullaniciKurucu {
#         pub fn ad(mut self, ad: &str) -> Self { self.ad = ad.to_string(); self }
#         pub fn eposta(mut self, eposta: &str) -> Self { self.eposta = eposta.to_string(); self }
#         pub fn olustur(self) -> Result<Kullanici, String> { Ok(Kullanici { ad: self.ad, giris_sayisi: 0 }) }
#     }
#     impl Kullanici {
#         pub fn kurucu() -> KullaniciKurucu { KullaniciKurucu::default() }
#         pub fn giris_sayisi(&self) -> u32 { self.giris_sayisi }
#         pub fn giris_kaydet(&mut self) { self.giris_sayisi += 1; }
#         pub fn pasiflestir(self) -> PasifKullanici { PasifKullanici { ad: self.ad } }
#     }
#     impl PasifKullanici {
#         pub fn ozet(&self) -> String { format!("[pasif] {}", self.ad) }
#     }
# }
```

Derleyicinin yanıtı:

```text
error[E0382]: use of moved value: `b`
```

İki kullanıcı için iki zincir yazın:

```rust
use hesap::Kullanici;

fn main() {
    let k1 = Kullanici::kurucu().ad("Kaan").eposta("kaan@ornek.com").olustur();
    let k2 = Kullanici::kurucu().ad("Ada").eposta("ada@ornek.com").olustur();
    println!("{} {}", k1.is_ok(), k2.is_ok());
}
# mod hesap { // kısaltılmış hâli: e-posta denetimi yok
#     pub struct Kullanici { ad: String, giris_sayisi: u32 }
#     pub struct PasifKullanici { ad: String }
#     #[derive(Default)]
#     pub struct KullaniciKurucu { ad: String, eposta: String }
#     impl KullaniciKurucu {
#         pub fn ad(mut self, ad: &str) -> Self { self.ad = ad.to_string(); self }
#         pub fn eposta(mut self, eposta: &str) -> Self { self.eposta = eposta.to_string(); self }
#         pub fn olustur(self) -> Result<Kullanici, String> { Ok(Kullanici { ad: self.ad, giris_sayisi: 0 }) }
#     }
#     impl Kullanici {
#         pub fn kurucu() -> KullaniciKurucu { KullaniciKurucu::default() }
#         pub fn giris_sayisi(&self) -> u32 { self.giris_sayisi }
#         pub fn giris_kaydet(&mut self) { self.giris_sayisi += 1; }
#         pub fn pasiflestir(self) -> PasifKullanici { PasifKullanici { ad: self.ad } }
#     }
#     impl PasifKullanici {
#         pub fn ozet(&self) -> String { format!("[pasif] {}", self.ad) }
#     }
# }
```

Çıktı:

```text
true true
```

### `mut` yazmayı unutmak (`E0596`)

`giris_kaydet` `&mut self` ister; değiştirilemez bir bağlama bunu veremez:

```rust,compile_fail,E0596
use hesap::Kullanici;

fn main() {
    let Ok(ada) = Kullanici::kurucu().ad("Ada").eposta("ada@ornek.com").olustur() else {
        return;
    };
    ada.giris_kaydet();
}
# mod hesap { // kısaltılmış hâli: e-posta denetimi yok
#     pub struct Kullanici { ad: String, giris_sayisi: u32 }
#     pub struct PasifKullanici { ad: String }
#     #[derive(Default)]
#     pub struct KullaniciKurucu { ad: String, eposta: String }
#     impl KullaniciKurucu {
#         pub fn ad(mut self, ad: &str) -> Self { self.ad = ad.to_string(); self }
#         pub fn eposta(mut self, eposta: &str) -> Self { self.eposta = eposta.to_string(); self }
#         pub fn olustur(self) -> Result<Kullanici, String> { Ok(Kullanici { ad: self.ad, giris_sayisi: 0 }) }
#     }
#     impl Kullanici {
#         pub fn kurucu() -> KullaniciKurucu { KullaniciKurucu::default() }
#         pub fn giris_sayisi(&self) -> u32 { self.giris_sayisi }
#         pub fn giris_kaydet(&mut self) { self.giris_sayisi += 1; }
#         pub fn pasiflestir(self) -> PasifKullanici { PasifKullanici { ad: self.ad } }
#     }
#     impl PasifKullanici {
#         pub fn ozet(&self) -> String { format!("[pasif] {}", self.ad) }
#     }
# }
```

Derleyicinin yanıtı:

```text
error[E0596]: cannot borrow `ada` as mutable, as it is not declared as mutable
  |
help: consider changing this to be mutable
  |
4 |     let Ok(mut ada) = Kullanici::kurucu().ad("Ada").eposta("ada@ornek.com").olustur() else {
  |            +++
```

Derleyicinin önerdiği gibi `mut ada` yazmak yeter. `mut`, desenin içinde bağlamanın önüne gelir:

```rust
use hesap::Kullanici;

fn main() {
    let Ok(mut ada) = Kullanici::kurucu().ad("Ada").eposta("ada@ornek.com").olustur() else {
        return;
    };
    ada.giris_kaydet();
    println!("{} giriş", ada.giris_sayisi());
}
# mod hesap { // kısaltılmış hâli: e-posta denetimi yok
#     pub struct Kullanici { ad: String, giris_sayisi: u32 }
#     pub struct PasifKullanici { ad: String }
#     #[derive(Default)]
#     pub struct KullaniciKurucu { ad: String, eposta: String }
#     impl KullaniciKurucu {
#         pub fn ad(mut self, ad: &str) -> Self { self.ad = ad.to_string(); self }
#         pub fn eposta(mut self, eposta: &str) -> Self { self.eposta = eposta.to_string(); self }
#         pub fn olustur(self) -> Result<Kullanici, String> { Ok(Kullanici { ad: self.ad, giris_sayisi: 0 }) }
#     }
#     impl Kullanici {
#         pub fn kurucu() -> KullaniciKurucu { KullaniciKurucu::default() }
#         pub fn giris_sayisi(&self) -> u32 { self.giris_sayisi }
#         pub fn giris_kaydet(&mut self) { self.giris_sayisi += 1; }
#         pub fn pasiflestir(self) -> PasifKullanici { PasifKullanici { ad: self.ad } }
#     }
#     impl PasifKullanici {
#         pub fn ozet(&self) -> String { format!("[pasif] {}", self.ad) }
#     }
# }
```

Çıktı:

```text
1 giriş
```

### Elde yalnızca referans varken tüketen metodu çağırmak (`E0507`)

```rust,compile_fail,E0507
use hesap::{Kullanici, PasifKullanici};

fn arsivle(k: &Kullanici) -> PasifKullanici {
    k.pasiflestir()
}
# mod hesap { // kısaltılmış hâli: e-posta denetimi yok
#     pub struct Kullanici { ad: String, giris_sayisi: u32 }
#     pub struct PasifKullanici { ad: String }
#     #[derive(Default)]
#     pub struct KullaniciKurucu { ad: String, eposta: String }
#     impl KullaniciKurucu {
#         pub fn ad(mut self, ad: &str) -> Self { self.ad = ad.to_string(); self }
#         pub fn eposta(mut self, eposta: &str) -> Self { self.eposta = eposta.to_string(); self }
#         pub fn olustur(self) -> Result<Kullanici, String> { Ok(Kullanici { ad: self.ad, giris_sayisi: 0 }) }
#     }
#     impl Kullanici {
#         pub fn kurucu() -> KullaniciKurucu { KullaniciKurucu::default() }
#         pub fn giris_sayisi(&self) -> u32 { self.giris_sayisi }
#         pub fn giris_kaydet(&mut self) { self.giris_sayisi += 1; }
#         pub fn pasiflestir(self) -> PasifKullanici { PasifKullanici { ad: self.ad } }
#     }
#     impl PasifKullanici {
#         pub fn ozet(&self) -> String { format!("[pasif] {}", self.ad) }
#     }
# }
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0507]: cannot move out of `*k` which is behind a shared reference
  |
4 |     k.pasiflestir()
  |     ^ ------------- `*k` moved due to this method call
  |     |
  |     move occurs because `*k` has type `Kullanici`, which does not implement the `Copy` trait
```

`arsivle` değeri yalnızca ödünç almıştır; başkasının kartını teslim edemez. Tüketecek fonksiyon sahipliği imzasında ister:

```rust
use hesap::{Kullanici, PasifKullanici};

fn arsivle(k: Kullanici) -> PasifKullanici {
    k.pasiflestir()
}

fn main() -> Result<(), String> {
    let ada = Kullanici::kurucu().ad("Ada").eposta("ada@ornek.com").olustur()?;
    println!("{}", arsivle(ada).ozet());
    Ok(())
}
# mod hesap {
#     #[derive(Debug, Clone, PartialEq)]
#     pub struct Eposta(String);
#     impl Eposta {
#         pub fn ayristir(ham: &str) -> Result<Eposta, String> {
#             let ham = ham.trim();
#             let gecerli = ham.split_once('@').is_some_and(|(yerel, alan)| {
#                 !yerel.is_empty() && alan.contains('.') && !alan.contains('@') && !ham.contains(char::is_whitespace)
#             });
#             if gecerli { Ok(Eposta(ham.to_lowercase())) } else { Err(format!("geçersiz e-posta: '{ham}'")) }
#         }
#         pub fn as_str(&self) -> &str { &self.0 }
#         pub fn alan_adi(&self) -> &str { self.0.split_once('@').map_or("", |(_, alan)| alan) }
#     }
#     #[derive(Debug)]
#     pub struct Kullanici { ad: String, eposta: Eposta, yonetici: bool, giris_sayisi: u32 }
#     #[derive(Debug, Default)]
#     pub struct KullaniciKurucu { ad: Option<String>, eposta: Option<String>, yonetici: bool }
#     impl KullaniciKurucu {
#         pub fn ad(mut self, ad: &str) -> Self { self.ad = Some(ad.trim().to_string()); self }
#         pub fn eposta(mut self, eposta: &str) -> Self { self.eposta = Some(eposta.to_string()); self }
#         pub fn yonetici(mut self, yonetici: bool) -> Self { self.yonetici = yonetici; self }
#         pub fn olustur(self) -> Result<Kullanici, String> {
#             let ad = self.ad.filter(|ad| !ad.is_empty()).ok_or_else(|| String::from("ad zorunlu"))?;
#             let ham = self.eposta.ok_or_else(|| String::from("e-posta zorunlu"))?;
#             Ok(Kullanici { ad, eposta: Eposta::ayristir(&ham)?, yonetici: self.yonetici, giris_sayisi: 0 })
#         }
#     }
#     impl Kullanici {
#         pub fn kurucu() -> KullaniciKurucu { KullaniciKurucu::default() }
#         pub fn ad(&self) -> &str { &self.ad }
#         pub fn eposta(&self) -> &Eposta { &self.eposta }
#         pub fn giris_sayisi(&self) -> u32 { self.giris_sayisi }
#         pub fn ozet(&self) -> String {
#             let rol = if self.yonetici { "yönetici" } else { "üye" };
#             format!("{} <{}> [{rol}], {} giriş", self.ad, self.eposta.as_str(), self.giris_sayisi)
#         }
#         pub fn giris_kaydet(&mut self) { self.giris_sayisi += 1; }
#         pub fn pasiflestir(self) -> PasifKullanici {
#             PasifKullanici { ad: self.ad, eposta: self.eposta, son_giris_sayisi: self.giris_sayisi }
#         }
#     }
#     #[derive(Debug)]
#     pub struct PasifKullanici { ad: String, eposta: Eposta, son_giris_sayisi: u32 }
#     impl PasifKullanici {
#         pub fn ozet(&self) -> String {
#             let PasifKullanici { ad, eposta, son_giris_sayisi } = self;
#             format!("[pasif] {ad} <{}>, son giriş sayısı {son_giris_sayisi}", eposta.as_str())
#         }
#         pub fn yeniden_etkinlestir(self) -> Kullanici {
#             Kullanici { ad: self.ad, eposta: self.eposta, yonetici: false, giris_sayisi: 0 }
#         }
#     }
# }
```

Çıktı:

```text
[pasif] Ada <ada@ornek.com>, son giriş sayısı 0
```

### Alan unutmak (`E0063`)

```rust,compile_fail,E0063
fn main() {
    let a = Ayarlar { tema: String::from("koyu") };
}
# #[derive(Debug, Clone, PartialEq)]
# struct Ayarlar { tema: String, dil: String, bildirim: bool, sayfa_boyutu: u32 }
# impl Default for Ayarlar {
#     fn default() -> Self {
#         Ayarlar { tema: String::from("açık"), dil: String::from("tr"), bildirim: true, sayfa_boyutu: 20 }
#     }
# }
```

Derleyicinin yanıtı:

```text
error[E0063]: missing fields `bildirim`, `dil` and `sayfa_boyutu` in initializer of `Ayarlar`
  |
2 |     let a = Ayarlar { tema: String::from("koyu") };
  |             ^^^^^^^ missing `bildirim`, `dil` and `sayfa_boyutu`
```

Rust'ta yarım kurulmuş bir struct olmaz: Ya bütün alanları verirsiniz ya da `..` ile kaynağını söylersiniz.

```rust
fn main() {
    let a = Ayarlar { tema: String::from("koyu"), ..Default::default() };
    println!("{a:?}");
}
# #[derive(Debug, Clone, PartialEq)]
# struct Ayarlar { tema: String, dil: String, bildirim: bool, sayfa_boyutu: u32 }
# impl Default for Ayarlar {
#     fn default() -> Self {
#         Ayarlar { tema: String::from("açık"), dil: String::from("tr"), bildirim: true, sayfa_boyutu: 20 }
#     }
# }
```

Çıktı:

```text
Ayarlar { tema: "koyu", dil: "tr", bildirim: true, sayfa_boyutu: 20 }
```

## Perde arkası: Derleyici ne yapıyor?

> [!NOTE]
> Bu başlık ilk okuyuşta atlanabilir; yukarıdaki kuralları anlamış olmanız yeterlidir.

**Gizlilik hataları üç ayrı aşamadan gelir.** `hesap::Eposta(..)` yolundaki yapıcının görünür olmadığı isim çözümlemesinde anlaşılır (`E0603`).[^e0603] Bir alanı nokta ile okumak ya da yazmak tür denetiminin konusudur (`E0616`).[^e0616] Alanları vererek struct kurmak ise ayrı bir gizlilik geçişinde, `rustc_privacy`'de incelenir; `..` ile alınan alanlar da orada denetlenir (`E0451`).[^e0451] Eksik alan (`E0063`) bir gizlilik hatası değildir; tür denetimi sırasında bulunur.[^e0063]

**Kısmi taşıma alan düzeyinde izlenir.** Derleyici, taşınan ya da başlatılan her alan için ayrı bir *taşıma yolu* (move path) tutar.[^movepath] Taşınan alan "başlatılmamış" sayılır; ödünç denetleyicisi bütün struct'a erişen bir kullanım gördüğünde alanlarından birinin başlatılmamış olduğunu bildirir. "Borrow of *partially* moved value" sözü buradan gelir.

## Akılda kalması gerekenler

- Alanları özel tutup kurmayı doğrulayan bir fonksiyona bırakırsanız değişmez tek yerde denetlenir. Değeri alan kod yeniden denetlemez.
- Newtype, sıradan bir `String`'i "doğrulanmış e-posta" hâline getirir; ikisi artık karışmaz.
- Kurucu kalıbı her değere bir ad verir ve eksikleri tek yerde denetler. Son adım sahiplik istiyorsa ayarlayıcılar `self` alıp `Self` döndürür.
- Alıcı bir sözleşmedir: `&self` okur, `&mut self` değiştirir, `self` değeri başka bir şeye dönüştürür. Durumu ayrı türlerle anlatmak, yanlış kullanımı derleme hatasına çevirir.
- `..taban` yalnızca yazmadığınız alanları alır; `Copy` olmayanları taşır. Kaynağı korumak istiyorsanız o alanları kendiniz verin ya da açıkça klonlayın.

## Alıştırmalar

Üçü de bu sayfadaki kalıpları yeniden kurdurur: denetleyen bir kart, forma yeni bir göz, kartı yerinde değiştiren bir metot. Önce kendiniz deneyin; takılırsanız sonraki bölüme geçip dönebilirsiniz. Kendi testlerinizi `learn/structs/src/deep_dive.rs` dosyasına ekleyip `cargo test -p structs` ile çalıştırabilirsiniz.

**1.** Bir `Parola` newtype'ı yazın: En az 8 karakter ve en az bir rakam içermeyen metni reddetsin. `Parola::ayristir("kisa1")` `Err`, `Parola::ayristir("uzunparola1")` `Ok` vermelidir. Ek soru: `Parola`'ya neden `derive(Debug)` eklemek istemeyebilirsiniz?

<details>
<summary>İpucu ve çözüm</summary>

İpucu: `Eposta::ayristir` ile aynı kalıptır. Alanı özel tutun; rakam aramak için `chars().any(..)` yeterlidir.

```rust
mod hesap {
    pub struct Parola(String);

    impl Parola {
        pub fn ayristir(ham: &str) -> Result<Parola, String> {
            if ham.chars().count() < 8 {
                return Err(String::from("parola en az 8 karakter olmalı"));
            }
            if !ham.chars().any(|c| c.is_ascii_digit()) {
                return Err(String::from("parola en az bir rakam içermeli"));
            }
            Ok(Parola(ham.to_string()))
        }

        pub fn eslesir(&self, deneme: &str) -> bool {
            self.0 == deneme
        }
    }

    impl std::fmt::Debug for Parola {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("Parola(***)") // içeriği gösterme
        }
    }
}

use hesap::Parola;

fn main() {
    println!("{:?}", Parola::ayristir("kisa1"));
    println!("{:?}", Parola::ayristir("uzunparola"));
    println!("{:?}", Parola::ayristir("uzunparola1"));
    println!("{}", Parola::ayristir("uzunparola1").is_ok_and(|p| p.eslesir("uzunparola1")));
}
```

Çıktı:

```text
Err("parola en az 8 karakter olmalı")
Err("parola en az bir rakam içermeli")
Ok(Parola(***))
true
```

Ek sorunun yanıtı: `derive(Debug)` parolanın kendisini yazdırır; bir `{:?}` çıktısı ya da panik iletisi onu kayıtlara (log) sızdırabilir. Yukarıdaki çözüm `Debug`'ı elle yazıp içeriği gizler. Gerçek bir sistemde parolanın kendisi değil, tuzlanmış özeti (salted hash) saklanır.

</details>

**2.** `KullaniciKurucu`'ya `dil(self, &str) -> Self` adımı ekleyin. Verilmezse `"tr"` olsun ve `ozet()` bunu göstersin. `Kullanici`'ya alan eklediğinizde derleyici sizi hangi iki ifadeye götürür? Var olan testlerden hangileri değişmeden geçer?

<details>
<summary>İpucu ve çözüm</summary>

İpucu: `Option<String>` tutun ve `olustur()` içinde `unwrap_or_else` kullanın. Alanı ekleyip derlediğinizde iki yerde `E0063` alırsınız: `olustur()` içindeki `Kullanici { .. }` ifadesi ve `yeniden_etkinlestir()` içindeki. İkincisinde bir karar vermeniz gerekir: Pasif kullanıcı dili taşımadığı için bu çözüm varsayılana döner. Testlere gelince, dosyadaki beş testten üçü değişmeden geçer; `ozet()`'in tam metnini karşılaştıran ikisini (`builder_fills_defaults` ve `inactive_user_keeps_logins_but_loses_admin`) yeni çıktıya göre güncellemek gerekir.

```rust
mod hesap {
#     #[derive(Debug, Clone, PartialEq)]
#     pub struct Eposta(String);
#     impl Eposta {
#         pub fn ayristir(ham: &str) -> Result<Eposta, String> {
#             let ham = ham.trim();
#             let gecerli = ham.split_once('@').is_some_and(|(yerel, alan)| {
#                 !yerel.is_empty() && alan.contains('.') && !alan.contains('@') && !ham.contains(char::is_whitespace)
#             });
#             if gecerli { Ok(Eposta(ham.to_lowercase())) } else { Err(format!("geçersiz e-posta: '{ham}'")) }
#         }
#         pub fn as_str(&self) -> &str { &self.0 }
#         pub fn alan_adi(&self) -> &str { self.0.split_once('@').map_or("", |(_, alan)| alan) }
#     }
    const VARSAYILAN_DIL: &str = "tr";

    #[derive(Debug)]
    pub struct Kullanici {
        ad: String,
        eposta: Eposta,
        yonetici: bool,
        dil: String, // yeni alan
        giris_sayisi: u32,
    }

    #[derive(Debug, Default)]
    pub struct KullaniciKurucu {
        ad: Option<String>,
        eposta: Option<String>,
        yonetici: bool,
        dil: Option<String>, // yeni göz
    }

    impl KullaniciKurucu {
        pub fn dil(mut self, dil: &str) -> Self {
            self.dil = Some(dil.to_string());
            self
        }

        pub fn olustur(self) -> Result<Kullanici, String> {
            // ad ve ham önceki gibi denetlenir
#             let ad = self.ad.filter(|ad| !ad.is_empty()).ok_or_else(|| String::from("ad zorunlu"))?;
#             let ham = self.eposta.ok_or_else(|| String::from("e-posta zorunlu"))?;
            Ok(Kullanici {
                ad,
                eposta: Eposta::ayristir(&ham)?,
                yonetici: self.yonetici,
                dil: self.dil.unwrap_or_else(|| VARSAYILAN_DIL.to_string()),
                giris_sayisi: 0,
            })
        }
#         pub fn ad(mut self, ad: &str) -> Self { self.ad = Some(ad.trim().to_string()); self }
#         pub fn eposta(mut self, eposta: &str) -> Self { self.eposta = Some(eposta.to_string()); self }
#         pub fn yonetici(mut self, yonetici: bool) -> Self { self.yonetici = yonetici; self }
    }

    impl Kullanici {
        pub fn ozet(&self) -> String {
            let rol = if self.yonetici { "yönetici" } else { "üye" };
            let Kullanici { ad, eposta, giris_sayisi, dil, .. } = self;
            format!("{ad} <{}> [{rol}], {giris_sayisi} giriş, dil: {dil}", eposta.as_str())
        }
#         pub fn kurucu() -> KullaniciKurucu { KullaniciKurucu::default() }
#         pub fn pasiflestir(self) -> PasifKullanici { PasifKullanici { ad: self.ad, eposta: self.eposta } }
    }
#     pub struct PasifKullanici { ad: String, eposta: Eposta }

    impl PasifKullanici {
        pub fn yeniden_etkinlestir(self) -> Kullanici {
            Kullanici {
                ad: self.ad,
                eposta: self.eposta,
                yonetici: false,
                dil: VARSAYILAN_DIL.to_string(), // ikinci E0063 burada
                giris_sayisi: 0,
            }
        }
    }
}
#
# fn main() -> Result<(), String> {
#     use hesap::Kullanici;
#     let k = Kullanici::kurucu().ad("Ada").eposta("ada@ornek.com").olustur()?;
#     println!("{}", k.ozet());
#     let k = Kullanici::kurucu().ad("Ada").eposta("ada@ornek.com").dil("en").olustur()?;
#     println!("{}", k.ozet());
#     println!("{}", k.pasiflestir().yeniden_etkinlestir().ozet());
#     Ok(())
# }
```

Üç kullanımın özeti sırasıyla şöyledir: dil verilmeden kurulan kullanıcı, `.dil("en")` ile kurulan kullanıcı ve onun pasifleştirilip yeniden etkinleştirilmiş hâli.

Çıktı:

```text
Ada <ada@ornek.com> [üye], 0 giriş, dil: tr
Ada <ada@ornek.com> [üye], 0 giriş, dil: en
Ada <ada@ornek.com> [üye], 0 giriş, dil: tr
```

</details>

**3.** Boş (ya da yalnızca boşluktan oluşan) adı reddeden `Kullanici::ad_degistir` metodunu yazın. Başarısız bir çağrıdan sonra `ad()` eski değeri döndürmelidir.

<details>
<summary>İpucu ve çözüm</summary>

İpucu: Hangi alıcı? Değer yerinde değişiyor ve eski hâli anlamsızlaşmıyor; öyleyse `&mut self`. Önce denetleyin, sonra yazın: Hata durumunda alana hiç dokunulmamış olur.

```rust
mod hesap {
#     #[derive(Debug, Clone, PartialEq)]
#     pub struct Eposta(String);
#     impl Eposta {
#         pub fn ayristir(ham: &str) -> Result<Eposta, String> {
#             let ham = ham.trim();
#             let gecerli = ham.split_once('@').is_some_and(|(yerel, alan)| {
#                 !yerel.is_empty() && alan.contains('.') && !alan.contains('@') && !ham.contains(char::is_whitespace)
#             });
#             if gecerli { Ok(Eposta(ham.to_lowercase())) } else { Err(format!("geçersiz e-posta: '{ham}'")) }
#         }
#         pub fn as_str(&self) -> &str { &self.0 }
#         pub fn alan_adi(&self) -> &str { self.0.split_once('@').map_or("", |(_, alan)| alan) }
#     }
#     #[derive(Debug)]
#     pub struct Kullanici { ad: String, eposta: Eposta, yonetici: bool, giris_sayisi: u32 }
#     #[derive(Debug, Default)]
#     pub struct KullaniciKurucu { ad: Option<String>, eposta: Option<String>, yonetici: bool }
#     impl KullaniciKurucu {
#         pub fn ad(mut self, ad: &str) -> Self { self.ad = Some(ad.trim().to_string()); self }
#         pub fn eposta(mut self, eposta: &str) -> Self { self.eposta = Some(eposta.to_string()); self }
#         pub fn yonetici(mut self, yonetici: bool) -> Self { self.yonetici = yonetici; self }
#         pub fn olustur(self) -> Result<Kullanici, String> {
#             let ad = self.ad.filter(|ad| !ad.is_empty()).ok_or_else(|| String::from("ad zorunlu"))?;
#             let ham = self.eposta.ok_or_else(|| String::from("e-posta zorunlu"))?;
#             Ok(Kullanici { ad, eposta: Eposta::ayristir(&ham)?, yonetici: self.yonetici, giris_sayisi: 0 })
#         }
#     }
    impl Kullanici {
        pub fn ad_degistir(&mut self, yeni: &str) -> Result<(), String> {
            let yeni = yeni.trim();
            if yeni.is_empty() {
                return Err(String::from("ad boş olamaz"));
            }
            self.ad = yeni.to_string(); // yalnızca denetim geçildiyse
            Ok(())
        }
#         pub fn kurucu() -> KullaniciKurucu { KullaniciKurucu::default() }
#         pub fn ad(&self) -> &str { &self.ad }
    }
}

use hesap::Kullanici;

fn main() -> Result<(), String> {
    let mut k = Kullanici::kurucu().ad("Ada").eposta("ada@ornek.com").olustur()?;
    println!("{:?} / {}", k.ad_degistir("   "), k.ad());
    println!("{:?} / {}", k.ad_degistir(" Grace "), k.ad());
    Ok(())
}
```

Çıktı:

```text
Err("ad boş olamaz") / Ada
Ok(()) / Grace
```

</details>

## Özet

Bu sayfada struct'ı yalnızca veri taşıyan bir kalıp olarak değil, kural koyan bir tür olarak kullandık: Özel alan ve doğrulayan yapıcı geçersiz değeri engelledi, kurucu kalıbı kurmayı okunur kıldı, alıcı seçimi geçersiz bir durum geçişini derleme hatasına çevirdi.

Bir eksiğimiz var: `Kullanici` ile `PasifKullanici` ayrı türler olduğu için ikisini aynı listede tutamıyoruz. `enum Hesap { Aktif(Kullanici), Pasif(PasifKullanici) }` tam bunu çözer. Kurucudaki `Option` ve `olustur()`'un döndürdüğü `Result` da birer enum'dur; [5. Enum'lar ve Desen Eşleme](05-enums.md) onların içini açıyor.

## Kaynaklar

[^gizlilik]: The Rust Reference, [Visibility and privacy](https://doc.rust-lang.org/reference/visibility-and-privacy.html#visibility-and-privacy); The Rust Programming Language, [7.3 Making Structs and Enums Public](https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html#making-structs-and-enums-public); hata dizini, [`E0603`](https://doc.rust-lang.org/error_codes/E0603.html), [`E0616`](https://doc.rust-lang.org/error_codes/E0616.html) ve [`E0451`](https://doc.rust-lang.org/error_codes/E0451.html).
[^newtype]: Rust API Guidelines, [`C-NEWTYPE`](https://rust-lang.github.io/api-guidelines/type-safety.html#c-newtype); The Rust Programming Language, [20.3 Type Safety and Abstraction with the Newtype Pattern](https://doc.rust-lang.org/book/ch20-03-advanced-types.html#type-safety-and-abstraction-with-the-newtype-pattern).
[^dogrulama]: The Rust Programming Language, [9.3 Custom Types for Validation](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html#custom-types-for-validation).
[^bool]: Rust API Guidelines, [`C-CUSTOM-TYPE`](https://rust-lang.github.io/api-guidelines/type-safety.html#c-custom-type).
[^builder]: Rust API Guidelines, [`C-BUILDER`](https://rust-lang.github.io/api-guidelines/type-safety.html#c-builder); Rust Design Patterns, [Builder](https://rust-unofficial.github.io/patterns/patterns/creational/builder.html).
[^getter]: Rust API Guidelines, [`C-GETTER`](https://rust-lang.github.io/api-guidelines/naming.html#c-getter); The Rust Programming Language, [5.3 Method Syntax](https://doc.rust-lang.org/book/ch05-03-method-syntax.html#method-syntax).
[^let-else]: The Rust Programming Language, [6.3 Staying on the "Happy Path" with `let...else`](https://doc.rust-lang.org/book/ch06-03-if-let.html#staying-on-the-happy-path-with-letelse).
[^guncelleme]: The Rust Reference, [Functional update syntax](https://doc.rust-lang.org/reference/expressions/struct-expr.html#functional-update-syntax).
[^e0603]: rustc 1.99.0 kaynak kodu, [`rustc_resolve/src/diagnostics/mod.rs`, `E0603`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_resolve/src/diagnostics/mod.rs#L1160).
[^e0616]: rustc 1.99.0 kaynak kodu, [`rustc_hir_typeck/src/expr.rs`, `E0616`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_typeck/src/expr.rs#L3331).
[^e0451]: rustc 1.99.0 kaynak kodu, [`rustc_privacy/src/diagnostics.rs`, `FieldIsPrivate`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_privacy/src/diagnostics.rs#L14); The Rust Reference, [Functional update syntax](https://doc.rust-lang.org/reference/expressions/struct-expr.html#functional-update-syntax).
[^e0063]: rustc 1.99.0 kaynak kodu, [`rustc_hir_typeck/src/expr.rs`, `E0063`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_typeck/src/expr.rs#L2315); hata dizini, [`E0063`](https://doc.rust-lang.org/error_codes/E0063.html).
[^movepath]: rustc 1.99.0 kaynak kodu, [`move_paths/mod.rs`, `MovePath`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_mir_dataflow/src/move_paths/mod.rs#L66).
[^default]: std, [`Default`](https://doc.rust-lang.org/std/default/trait.Default.html).
