# 17.1 Nesne Yönelimli Özellikler: Derinleşme

[17. bölümde](17-oop.md) farklı türleri `Vec<Box<dyn Ciz>>` ile tek listede topladık. Bu sayfada aynı aracı gerçeğe daha yakın bir tasarımda kullanacağız: e-posta, SMS ve webhook gibi kanallara mesaj gönderen bir bildirim sistemi. Ana fikir tek cümledir: Çekirdek kod yalnızca "bildirim gönderebilen bir şey" sözünü bilir, yeni bir kanal türü eklemek için çekirdeğe dokunulmaz.

Statik ve dinamik gönderimin ne olduğunu, bir trait'in neden dyn uyumlu olmayabileceğini [9. bölümde](09-generics-traits-lifetimes.md); geniş işaretçiyi ve vtable'ı [22. bölümde](../ileri/22-sized-dst.md) ele alıyoruz. Burada tasarım kararlarına odaklanacağız: Aynı problem trait nesnesiyle mi, jenerikle mi, enum'la mı çözülmeli? Hangi işaretçi seçilmeli? Trait nesnesi ömür ve thread güvenliği bilgisini nasıl taşır? Kalıtım yerine ne kullanılır? Türü bilinmeyen bir kutu nasıl klonlanır?

> [!NOTE]
> Bu sayfanın notları ve çalıştırılabilir kodu `learn/oop/src/derinlesme.rs` dosyasındadır. Depo kökünden `cargo run -p oop` ile çalıştırabilir, `cargo test -p oop` ile testlerini koşturabilirsiniz.

<details>
<summary>Bu sayfada geçen terimler</summary>

- **sözleşme (contract):** Bir trait'in "beni uygulayan şunları yapar" diye verdiği söz.
- **supertrait:** Bir trait'in ön şartı. "`Kanal` olan her şey `Debug` de olmalı" gibi.
- **şablon metot (template method):** Adımları sabit, bazı adımları türlere bırakılmış tarif.
- **kanca (hook):** Hazır cevabı olan, isteyen türün değiştirdiği küçük adım.
- **eklenti (plugin):** Programa sonradan, çekirdeği değiştirmeden eklenen parça.
- **parametrede `impl Trait`:** "Bu trait'i uygulayan herhangi bir tür" demenin kısa yazımı.
- **`Self: Sized`:** "Bu metot yalnızca türü belli değerlerde çağrılır, trait nesnesinde çağrılmaz" şartı.
- **yukarı dönüşüm (upcasting):** Daha özel bir trait nesnesine daha genel bir gözle bakmak.
- **`Any` ve downcast:** Program çalışırken "sen aslında şu tür müsün?" diye sormak.
- **koku (code smell):** Çalışan, ama tasarımda bir şeyin ters gittiğini gösteren işaret.
- **devretme (delegation):** İşi kendi yapmak yerine içindeki parçaya yaptırmak.
- **sarmalayıcı (wrapper):** Başka bir değeri içine alıp ona bir şey ekleyen tür.
- **genel impl (blanket impl):** "Şu şartı sağlayan her tür için" bir kerede yazılan `impl`.
- **`Send` ve `Sync`:** "Başka bir thread'e verilebilir" ve "birkaç thread birlikte bakabilir".
- **`'static`:** Ömür sınırı olarak kullanıldığında "içinde kısa ömürlü bir ödünç yok" demektir.
- **dinamik boyutlu tür (DST):** Boyutu derleme zamanında bilinmeyen tür; `dyn Trait` gibi.
- **unsized coercion:** `&Sms` gibi belli bir türü gösteren işaretçinin kendiliğinden `&dyn Kanal`'a dönüşmesi.
- **`HashMap`:** Anahtardan değere giden tablo; sırası belli değildir.

</details>

## Genişletilebilir bir bildirim sistemi

Kurabiye tarifi herkes için aynıdır: Hamuru yoğurun, şekil verin, pişirin. Yalnızca "şekil verin" adımını herkes kendince yapar; biri yıldız, biri kalp keser. İsteyen bir de süsler, istemeyen süslemez. Şimdi kuracağımız trait böyle bir tariftir. `gonder` tarifin kendisi, `bicimle` her kanalın kendince yaptığı adım, `azami_uzunluk` da isteğe bağlı süslemedir.

### Sözleşme: `Kanal` trait'i

```rust
use std::any::Any;
use std::fmt::Debug;

pub trait Kanal: Debug + Any {
    fn ad(&self) -> &'static str;

    // her kanalın farklı yaptığı adım
    fn bicimle(&self, mesaj: &str) -> String;

    // kanca: varsayılan cevap "sınır yok"
    fn azami_uzunluk(&self) -> Option<usize> {
        None
    }

    // şablon metot: tarifin iskeleti tek yerde durur
    fn gonder(&self, mesaj: &str) -> String {
        let govde: String = match self.azami_uzunluk() {
            Some(sinir) => mesaj.chars().take(sinir).collect(),
            None => mesaj.to_string(),
        };
        format!("[{}] {}", self.ad(), self.bicimle(&govde))
    }
}
# fn main() {}
```

`gonder` bir *şablon metottur* (template method): Algoritmanın iskeleti bir kez yazılmıştır, türler yalnızca adımları sağlar. Kalıtımlı dillerde bu iş üst sınıfın bir metoduyla yapılır; Rust'ta varsayılan trait metodu kullanılır.[^varsayilan] Arada önemli bir fark vardır: Burada paylaşılan şey davranıştır, alan değildir. Trait'in kendi verisi yoktur; ihtiyaç duyduğu her bilgiyi `ad` gibi zorunlu metotlardan ister. İskelet tek yerde olmasaydı her kanal kırpma işini ayrı ayrı yazar, biri unuttuğunda o kanal uzun mesajlarda bozulurdu.

`Kanal: Debug + Any` yazımındaki `Debug` ve `Any` birer *supertrait*'tir: `Kanal` uygulamak isteyen tür bunları da sağlamalıdır.[^supertrait] Notlardaki son hâlde üç ön şart bulunur:

| Supertrait | Neden istenir? |
| --- | --- |
| `Debug` | `Box<dyn Kanal>` doğrudan `{:?}` ile basılabilsin, onu tutan türlerde `derive(Debug)` çalışsın |
| `Any` | gerektiğinde somut türe geri dönülebilsin ([aşağıda](#yukarı-dönüşüm-ve-dyn-any)); yan etkisi, uygulayan türün `'static` olmasıdır |
| `KanalKlonla` | `Box<dyn Kanal>` klonlanabilsin; bunu [aşağıda](#boxdyn-kanal-klonlamak) ekleyeceğiz |

İki kanal tanımlayalım. `Sms` kancayı değiştirir, `Eposta` olduğu gibi bırakır:

```rust
# use std::any::Any;
# use std::fmt::Debug;
#
# pub trait Kanal: Debug + Any {
#     fn ad(&self) -> &'static str;
#     fn bicimle(&self, mesaj: &str) -> String;
#
#     fn azami_uzunluk(&self) -> Option<usize> {
#         None
#     }
#
#     fn gonder(&self, mesaj: &str) -> String {
#         let govde: String = match self.azami_uzunluk() {
#             Some(sinir) => mesaj.chars().take(sinir).collect(),
#             None => mesaj.to_string(),
#         };
#         format!("[{}] {}", self.ad(), self.bicimle(&govde))
#     }
# }
#
#[derive(Debug, Clone)]
struct Eposta {
    adres: String,
}

impl Kanal for Eposta {
    fn ad(&self) -> &'static str {
        "eposta"
    }
    fn bicimle(&self, mesaj: &str) -> String {
        format!("{} adresine: {mesaj}", self.adres)
    }
}

#[derive(Debug, Clone)]
struct Sms {
    numara: String,
}

impl Kanal for Sms {
    fn ad(&self) -> &'static str {
        "sms"
    }
    fn bicimle(&self, mesaj: &str) -> String {
        format!("{} numarasına: {mesaj}", self.numara)
    }
    fn azami_uzunluk(&self) -> Option<usize> {
        Some(12) // kancayı değiştirir
    }
}

fn main() {
    let e = Eposta { adres: String::from("a@b.c") };
    let s = Sms { numara: String::from("111") };
    println!("{}", e.gonder("çok uzun bir bildirim metni"));
    println!("{}", s.gonder("çok uzun bir bildirim metni"));
}
```

Çıktı:

```text
[eposta] a@b.c adresine: çok uzun bir bildirim metni
[sms] 111 numarasına: çok uzun bir
```

Aynı tarif iki farklı sonuç verdi: E-posta mesajın tamamını gönderdi, SMS ilk 12 karakteri aldı. Kırpma `chars()` ile yapıldığı için sınır bayta değil karaktere göredir; `ğ` ya da `ç` gibi iki baytlık bir harfin ortasından kesmek söz konusu olmaz.

Ön şartı sağlamayan tür `Kanal` olamaz:

```rust,compile_fail,E0277
# use std::any::Any;
# use std::fmt::Debug;
#
# pub trait Kanal: Debug + Any {
#     fn ad(&self) -> &'static str;
#     fn bicimle(&self, mesaj: &str) -> String;
#
#     fn azami_uzunluk(&self) -> Option<usize> {
#         None
#     }
#
#     fn gonder(&self, mesaj: &str) -> String {
#         let govde: String = match self.azami_uzunluk() {
#             Some(sinir) => mesaj.chars().take(sinir).collect(),
#             None => mesaj.to_string(),
#         };
#         format!("[{}] {}", self.ad(), self.bicimle(&govde))
#     }
# }
#
struct Faks;

impl Kanal for Faks {
    fn ad(&self) -> &'static str {
        "faks"
    }
    fn bicimle(&self, mesaj: &str) -> String {
        mesaj.to_string()
    }
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0277]: `Faks` doesn't implement `Debug`
   | impl Kanal for Faks {
   |                ^^^^ the trait `Debug` is not implemented for `Faks`
   |
note: required by a bound in `Kanal`
   | pub trait Kanal: Debug + Any {
   |                  ^^^^^ required by this bound in `Kanal`
help: consider annotating `Faks` with `#[derive(Debug)]`
```

`Any` ön şartının sonucu ise farklıdır. `Any` kendisi `'static` ister; `'static` olan türler de onu kendiliğinden sağlar.[^any] Bu yüzden `Any` için bir şey yazmanız gerekmez, ama içinde kısa ömürlü referans tutan bir tür `Kanal` uygulayamaz.

### Çekirdek: `Dagitici`

Çekirdek kod kanalları adlarıyla saklar ve adı verilen kanala mesaj yollar. Dikkat ederseniz hiçbir satırında somut bir tür adı geçmez:

```rust
# use std::any::Any;
# use std::fmt::Debug;
#
# pub trait Kanal: Debug + Any {
#     fn ad(&self) -> &'static str;
#     fn bicimle(&self, mesaj: &str) -> String;
#
#     fn azami_uzunluk(&self) -> Option<usize> {
#         None
#     }
#
#     fn gonder(&self, mesaj: &str) -> String {
#         let govde: String = match self.azami_uzunluk() {
#             Some(sinir) => mesaj.chars().take(sinir).collect(),
#             None => mesaj.to_string(),
#         };
#         format!("[{}] {}", self.ad(), self.bicimle(&govde))
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Eposta {
#     adres: String,
# }
#
# impl Kanal for Eposta {
#     fn ad(&self) -> &'static str {
#         "eposta"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} adresine: {mesaj}", self.adres)
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Sms {
#     numara: String,
# }
#
# impl Kanal for Sms {
#     fn ad(&self) -> &'static str {
#         "sms"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} numarasına: {mesaj}", self.numara)
#     }
#     fn azami_uzunluk(&self) -> Option<usize> {
#         Some(12)
#     }
# }
#
use std::collections::HashMap;

#[derive(Default)]
struct Dagitici {
    kanallar: HashMap<String, Box<dyn Kanal>>,
}

impl Dagitici {
    fn kaydet(&mut self, kanal: impl Kanal) {
        self.kaydet_kutulu(Box::new(kanal));
    }

    fn kaydet_kutulu(&mut self, kanal: Box<dyn Kanal>) {
        self.kanallar.insert(kanal.ad().to_string(), kanal); // aynı ad üzerine yazar
    }

    fn gonder(&self, kanal_adi: &str, mesaj: &str) -> Option<String> {
        self.kanallar.get(kanal_adi).map(|k| k.gonder(mesaj))
    }

    fn adlar(&self) -> Vec<&str> {
        let mut adlar: Vec<&str> = self.kanallar.keys().map(String::as_str).collect();
        adlar.sort_unstable(); // HashMap'in sırası belli değildir
        adlar
    }
}
# fn main() {
#     let mut d = Dagitici::default();
#     d.kaydet(Sms { numara: String::from("1") });
#     d.kaydet(Eposta { adres: String::from("a") });
#     assert_eq!(d.adlar(), ["eposta", "sms"]);
#     assert_eq!(d.gonder("faks", "x"), None);
# }
```

`kaydet`, `impl Kanal` alıp kutulama işini içeride yapar. Böylece çağıran `Box::new` yazmak zorunda kalmaz; jenerikten trait nesnesine geçiş tek bir noktada olur.

### Sonradan eklenen kanal

Sistem yazıldıktan aylar sonra webhook desteği istendiğini düşünün. Yeni bir tür yazmak yeter; `Dagitici`'de, `Kanal`'da ve öteki türlerde tek satır değişmez. Trait `pub` ise bu tür başka bir crate'te de durabilir. *Eklenti* (plugin) mimarisinin özü budur:[^gonderim]

```rust
# use std::any::Any;
# use std::fmt::Debug;
#
# pub trait Kanal: Debug + Any {
#     fn ad(&self) -> &'static str;
#     fn bicimle(&self, mesaj: &str) -> String;
#
#     fn azami_uzunluk(&self) -> Option<usize> {
#         None
#     }
#
#     fn gonder(&self, mesaj: &str) -> String {
#         let govde: String = match self.azami_uzunluk() {
#             Some(sinir) => mesaj.chars().take(sinir).collect(),
#             None => mesaj.to_string(),
#         };
#         format!("[{}] {}", self.ad(), self.bicimle(&govde))
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Eposta {
#     adres: String,
# }
#
# impl Kanal for Eposta {
#     fn ad(&self) -> &'static str {
#         "eposta"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} adresine: {mesaj}", self.adres)
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Sms {
#     numara: String,
# }
#
# impl Kanal for Sms {
#     fn ad(&self) -> &'static str {
#         "sms"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} numarasına: {mesaj}", self.numara)
#     }
#     fn azami_uzunluk(&self) -> Option<usize> {
#         Some(12)
#     }
# }
#
# use std::collections::HashMap;
#
# #[derive(Default)]
# struct Dagitici {
#     kanallar: HashMap<String, Box<dyn Kanal>>,
# }
#
# impl Dagitici {
#     fn kaydet(&mut self, kanal: impl Kanal) {
#         self.kaydet_kutulu(Box::new(kanal));
#     }
#
#     fn kaydet_kutulu(&mut self, kanal: Box<dyn Kanal>) {
#         self.kanallar.insert(kanal.ad().to_string(), kanal);
#     }
#
#     fn gonder(&self, kanal_adi: &str, mesaj: &str) -> Option<String> {
#         self.kanallar.get(kanal_adi).map(|k| k.gonder(mesaj))
#     }
#
#     fn adlar(&self) -> Vec<&str> {
#         let mut adlar: Vec<&str> = self.kanallar.keys().map(String::as_str).collect();
#         adlar.sort_unstable();
#         adlar
#     }
# }
#
#[derive(Debug, Clone)]
struct Webhook {
    url: String,
}

impl Kanal for Webhook {
    fn ad(&self) -> &'static str {
        "webhook"
    }
    fn bicimle(&self, mesaj: &str) -> String {
        format!("POST {} gövde={mesaj:?}", self.url)
    }
}

fn ornek_dagitici() -> Dagitici {
    let mut dagitici = Dagitici::default();
    dagitici.kaydet(Eposta { adres: String::from("ece@ornek.com") });
    dagitici.kaydet(Sms { numara: String::from("555-0101") });
    dagitici.kaydet(Webhook { url: String::from("/kanca") });
    dagitici
}

fn main() {
    let dagitici = ornek_dagitici();
    println!("kayıtlı: {:?}", dagitici.adlar());
    for ad in ["eposta", "sms", "webhook", "faks"] {
        match dagitici.gonder(ad, "Kargonuz yola çıktı") {
            Some(cikti) => println!("  {cikti}"),
            None => println!("  ({ad}: böyle bir kanal yok)"),
        }
    }
}
```

Çıktı:

```text
kayıtlı: ["eposta", "sms", "webhook"]
  [eposta] ece@ornek.com adresine: Kargonuz yola çıktı
  [sms] 555-0101 numarasına: Kargonuz yol
  [webhook] POST /kanca gövde="Kargonuz yola çıktı"
  (faks: böyle bir kanal yok)
```

## Aynı problem, üç çözüm: `dyn`, jenerik, enum

Aynı oyuncakları üç ayrı kutuda saklayabilirsiniz. Birincinin üstünde "ses çıkaran her şey" yazar; yarın alınacak oyuncak da girer (`dyn`). İkincisi yalnızca düdüklerin kutusudur; hepsi aynı türden olduğu için genellikle en hızlısı odur (jenerik). Üçüncünün bölmeleri sabittir, üstünde "düdük ya da davul" yazar; listeyi yalnızca kutuyu yapan değiştirebilir, ama içinde ne olabileceği baştan bellidir (enum).

```rust
# use std::any::Any;
# use std::fmt::Debug;
#
# pub trait Kanal: Debug + Any {
#     fn ad(&self) -> &'static str;
#     fn bicimle(&self, mesaj: &str) -> String;
#
#     fn azami_uzunluk(&self) -> Option<usize> {
#         None
#     }
#
#     fn gonder(&self, mesaj: &str) -> String {
#         let govde: String = match self.azami_uzunluk() {
#             Some(sinir) => mesaj.chars().take(sinir).collect(),
#             None => mesaj.to_string(),
#         };
#         format!("[{}] {}", self.ad(), self.bicimle(&govde))
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Eposta {
#     adres: String,
# }
#
# impl Kanal for Eposta {
#     fn ad(&self) -> &'static str {
#         "eposta"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} adresine: {mesaj}", self.adres)
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Sms {
#     numara: String,
# }
#
# impl Kanal for Sms {
#     fn ad(&self) -> &'static str {
#         "sms"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} numarasına: {mesaj}", self.numara)
#     }
#     fn azami_uzunluk(&self) -> Option<usize> {
#         Some(12)
#     }
# }
#
fn yayinla_dyn(kanallar: &[Box<dyn Kanal>], mesaj: &str) -> Vec<String> {
    kanallar.iter().map(|k| k.gonder(mesaj)).collect()
}

// K, çağrı yerinde tek bir somut türe sabitlenir
fn yayinla_generic<K: Kanal>(kanallar: &[K], mesaj: &str) -> Vec<String> {
    kanallar.iter().map(|k| k.gonder(mesaj)).collect()
}

// türler kümesi kapalıdır
enum SabitKanal {
    Eposta(Eposta),
    Sms(Sms),
}

impl SabitKanal {
    fn gonder(&self, mesaj: &str) -> String {
        match self {
            SabitKanal::Eposta(e) => e.gonder(mesaj),
            SabitKanal::Sms(s) => s.gonder(mesaj),
        }
    }
}

fn main() {
    let eposta = || Eposta { adres: String::from("a@b.c") };
    let sms = || Sms { numara: String::from("111") };

    let kutulu: Vec<Box<dyn Kanal>> = vec![Box::new(eposta()), Box::new(sms())];
    let sabit = [SabitKanal::Eposta(eposta()), SabitKanal::Sms(sms())];
    let enum_ciktisi: Vec<String> = sabit.iter().map(|k| k.gonder("Merhaba")).collect();

    println!("dyn     : {:?}", yayinla_dyn(&kutulu, "Merhaba"));
    println!("enum    : aynı çıktı mı? {}", enum_ciktisi == yayinla_dyn(&kutulu, "Merhaba"));
    println!("generic : {:?}", yayinla_generic(&[sms(), sms()], "Merhaba"));
}
```

Çıktı:

```text
dyn     : ["[eposta] a@b.c adresine: Merhaba", "[sms] 111 numarasına: Merhaba"]
enum    : aynı çıktı mı? true
generic : ["[sms] 111 numarasına: Merhaba", "[sms] 111 numarasına: Merhaba"]
```

Jenerik çözümde dilim *homojen* olmak zorundadır; iki farklı kanalı bir arada veremezsiniz:

```rust,compile_fail,E0308
# use std::any::Any;
# use std::fmt::Debug;
#
# pub trait Kanal: Debug + Any {
#     fn ad(&self) -> &'static str;
#     fn bicimle(&self, mesaj: &str) -> String;
#
#     fn azami_uzunluk(&self) -> Option<usize> {
#         None
#     }
#
#     fn gonder(&self, mesaj: &str) -> String {
#         let govde: String = match self.azami_uzunluk() {
#             Some(sinir) => mesaj.chars().take(sinir).collect(),
#             None => mesaj.to_string(),
#         };
#         format!("[{}] {}", self.ad(), self.bicimle(&govde))
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Eposta {
#     adres: String,
# }
#
# impl Kanal for Eposta {
#     fn ad(&self) -> &'static str {
#         "eposta"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} adresine: {mesaj}", self.adres)
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Sms {
#     numara: String,
# }
#
# impl Kanal for Sms {
#     fn ad(&self) -> &'static str {
#         "sms"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} numarasına: {mesaj}", self.numara)
#     }
#     fn azami_uzunluk(&self) -> Option<usize> {
#         Some(12)
#     }
# }
#
fn yayinla_generic<K: Kanal>(kanallar: &[K], mesaj: &str) -> Vec<String> {
    kanallar.iter().map(|k| k.gonder(mesaj)).collect()
}

fn main() {
    let eposta = Eposta { adres: String::from("a@b.c") };
    let sms = Sms { numara: String::from("111") };
    yayinla_generic(&[eposta, sms], "Merhaba");
}
```

Derleyicinin yanıtı:

```text
error[E0308]: mismatched types
   |     yayinla_generic(&[eposta, sms], "Merhaba");
   |                               ^^^ expected `Eposta`, found `Sms`
```

Üç çözümün karşılaştırması şöyledir:[^enum-mu][^mono]

| | `dyn Kanal` | jenerik `K: Kanal` | enum |
| --- | --- | --- | --- |
| Tür kümesi | açık, dışarıdan eklenebilir | açık, ama çağrıda tek tür | kapalı, yazarı belirler |
| Heterojen liste | evet | hayır | evet, kutusuz |
| Çağrı | vtable üzerinden, çoğunlukla satır içine alınamaz | statik, satır içine alınabilir | `match` dallanması, satır içine alınabilir |
| Kullanan kod | tek kopya | tür başına bir kopya | tek kopya |
| Yeni tür eklemek | kolay: yeni `impl` | kolay: yeni `impl` | her `match`'e dokunulur |
| Yeni işlem eklemek | trait'e ve her `impl`'e dokunulur | trait'e ve her `impl`'e dokunulur | kolay: yeni fonksiyon ve `match` |

Enum'un gücü eksiksizlik denetimidir: Yeni bir varyant eklendiğinde derleyici, güncellenmesi gereken her `match`'i gösterir. Trait tarafında böyle bir liste yoktur; sorumluluk yeni `impl`'i yazandadır. Varsayılan gövdesi olan bir metot eklemek de öteki `impl`'lere dokunmayı gerektirmez.

Seçim için pratik ölçüt "türleri kim ekleyecek?" sorusudur. Başkaları (kullanıcı, eklenti) ekleyecekse `dyn`; siz ekleyecekseniz ve küme küçükse enum; tek tür ve sık çalışan bir döngü varsa jenerik uygundur.

## Hangi işaretçi?

`dyn Kanal`'ın boyutu bilinmediği için hep bir işaretçinin arkasında durur. Hangi işaretçiyi seçeceğiniz "bunun sahibi kim?" sorusunun cevabıdır. Arkadaşınızın oyuncağıyla bir kez oynayıp geri verecekseniz ödünç alırsınız. Oyuncak sizinse kendi kutunuzda durur. Birkaç çocuk aynı oyuncağın ortak sahibiyse sayaçlı bir kutu gerekir; çocuklar ayrı thread'lerdeyse sayacın thread'ler arasında güvenli olanı kullanılır.

| İşaretçi | Ne zaman? |
| --- | --- |
| `&dyn Kanal` | Yalnızca kullanacaksanız (parametre). Tahsis ve sahiplik yoktur; ilk tercih budur. |
| `Box<dyn Kanal>` | Sahip olacaksanız (alan, koleksiyon, dönüş değeri). Tek sahip vardır. |
| `Rc<dyn Kanal>` | Aynı nesne birden çok sahipte duracaksa, tek thread içinde ([14. bölüm](14-smart-pointers.md)). |
| `Arc<dyn Kanal + Send + Sync>` | Aynısı, thread'ler arasında ([15. bölüm](15-concurrency.md)). |

```rust
# use std::any::Any;
# use std::fmt::Debug;
#
# pub trait Kanal: Debug + Any {
#     fn ad(&self) -> &'static str;
#     fn bicimle(&self, mesaj: &str) -> String;
#
#     fn azami_uzunluk(&self) -> Option<usize> {
#         None
#     }
#
#     fn gonder(&self, mesaj: &str) -> String {
#         let govde: String = match self.azami_uzunluk() {
#             Some(sinir) => mesaj.chars().take(sinir).collect(),
#             None => mesaj.to_string(),
#         };
#         format!("[{}] {}", self.ad(), self.bicimle(&govde))
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Eposta {
#     adres: String,
# }
#
# impl Kanal for Eposta {
#     fn ad(&self) -> &'static str {
#         "eposta"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} adresine: {mesaj}", self.adres)
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Sms {
#     numara: String,
# }
#
# impl Kanal for Sms {
#     fn ad(&self) -> &'static str {
#         "sms"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} numarasına: {mesaj}", self.numara)
#     }
#     fn azami_uzunluk(&self) -> Option<usize> {
#         Some(12)
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Webhook {
#     url: String,
# }
#
# impl Kanal for Webhook {
#     fn ad(&self) -> &'static str {
#         "webhook"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("POST {} gövde={mesaj:?}", self.url)
#     }
# }
#
use std::rc::Rc;
use std::sync::Arc;
use std::thread;

fn bir_kez_bildir(kanal: &dyn Kanal, mesaj: &str) -> String {
    kanal.gonder(mesaj)
}

fn main() {
    let sms = Sms { numara: String::from("222") };
    println!("&dyn : {}", bir_kez_bildir(&sms, "kısa")); // &Sms, &dyn Kanal'a dönüşür

    let ortak: Rc<dyn Kanal> = Rc::new(sms);
    let acil_listesi = [Rc::clone(&ortak)];
    let gunluk_listesi = [Rc::clone(&ortak)];
    let adlar = (acil_listesi[0].ad(), gunluk_listesi[0].ad());
    println!("Rc   : adlar = {adlar:?}, sahip sayısı = {}", Rc::strong_count(&ortak));

    let paylasilan: Arc<dyn Kanal + Send + Sync> =
        Arc::new(Webhook { url: String::from("/olay") });
    let isler: Vec<_> = (1..=2)
        .map(|no| {
            let kanal = Arc::clone(&paylasilan);
            thread::spawn(move || kanal.gonder(&format!("iş {no}")))
        })
        .collect();
    for is in isler {
        println!("Arc  : {}", is.join().expect("iş parçacığı panik etmez"));
    }
}
```

Çıktı:

```text
&dyn : [sms] 222 numarasına: kısa
Rc   : adlar = ("sms", "sms"), sahip sayısı = 3
Arc  : [webhook] POST /olay gövde="iş 1"
Arc  : [webhook] POST /olay gövde="iş 2"
```

Thread'ler hangi sırayla çalışırsa çalışsın `join` sırası sabit olduğu için bu çıktı değişmez. `bir_kez_bildir(&sms, ..)` çağrısında `&Sms`'in `&dyn Kanal`'a dönüşmesi bir *örtük dönüşümdür* (unsized coercion).[^ortuk]

### `dyn Kanal` kendiliğinden `Send` değildir

`Arc` satırında `+ Send + Sync` yazmasaydık program derlenmezdi. Aynı sorunu `Box` ile görelim:

```rust,compile_fail,E0277
# use std::any::Any;
# use std::fmt::Debug;
#
# pub trait Kanal: Debug + Any {
#     fn ad(&self) -> &'static str;
#     fn bicimle(&self, mesaj: &str) -> String;
#
#     fn azami_uzunluk(&self) -> Option<usize> {
#         None
#     }
#
#     fn gonder(&self, mesaj: &str) -> String {
#         let govde: String = match self.azami_uzunluk() {
#             Some(sinir) => mesaj.chars().take(sinir).collect(),
#             None => mesaj.to_string(),
#         };
#         format!("[{}] {}", self.ad(), self.bicimle(&govde))
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Eposta {
#     adres: String,
# }
#
# impl Kanal for Eposta {
#     fn ad(&self) -> &'static str {
#         "eposta"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} adresine: {mesaj}", self.adres)
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Sms {
#     numara: String,
# }
#
# impl Kanal for Sms {
#     fn ad(&self) -> &'static str {
#         "sms"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} numarasına: {mesaj}", self.numara)
#     }
#     fn azami_uzunluk(&self) -> Option<usize> {
#         Some(12)
#     }
# }
#
use std::thread;

fn main() {
    let kanal: Box<dyn Kanal> = Box::new(Sms { numara: String::from("222") });
    let is = thread::spawn(move || kanal.gonder("selam"));
    println!("{:?}", is.join());
}
```

Derleyicinin yanıtı:

```text
error[E0277]: `dyn Kanal` cannot be sent between threads safely
   |     let is = thread::spawn(move || kanal.gonder("selam"));
   |              ------------- ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `dyn Kanal` cannot be sent between threads safely
   |              |
   |              required by a bound introduced by this call
   |
   = help: the trait `Send` is not implemented for `dyn Kanal`
```

`Sms` aslında `Send`'dir, ama derleyici bunu artık göremez: Tür silinmiş, geriye "bir `Kanal`" bilgisi kalmıştır. Kapalı kutunun içini derleyici göremez; sözü siz verirsiniz, o da tutulmasını sağlar. `Send` gibi otomatik trait'ler trait nesnesine sızmaz; türde açıkça yazılmaları gerekir.[^auto] `dyn Kanal + Send + Sync` yazdığınızda yalnızca bu iki trait'i de sağlayan türler kutuya girebilir. Her kanalın thread'ler arasında taşınabilmesini istiyorsanız öteki yol `trait Kanal: Send` yazıp şartı supertrait yapmaktır. `Send` ve `Sync` ayrıntılarıyla [25. bölümdedir](../ileri/25-send-sync.md).

## Trait nesnesinin gizli ömür sınırı

Kapalı bir kutunun içindeki oyuncak kütüphaneden ödünç alınmış olabilir; kapağı açmadan bunu göremezsiniz. Bu yüzden kutunun üstüne bir etiket yapıştırılır. Bir şey yazmazsanız etiket "içinde ödünç şey yok" der ve böyle etiketli bir kutuya ödünç oyuncak konamaz. Ödünç bir şey koyacaksanız etikete "şu ödünç geçerli olduğu sürece kullanılabilir" yazarsınız.

Rust'ta her `dyn Trait` türü aslında `dyn Trait + 'ömür` biçimindedir. Ömrü siz yazmazsanız derleyici kurallara göre doldurur:[^varsayilan-omur]

| Yazdığınız | Derleyicinin anladığı |
| --- | --- |
| `Box<dyn Sablon>` (imzada ya da alanda) | `Box<dyn Sablon + 'static>` |
| `&'a dyn Sablon` | `&'a (dyn Sablon + 'a)` |

Bu bilgi somut türden okunamaz, çünkü tür silinmiştir; trait nesnesinin üstünde taşınması gerekir. Taşınmasaydı kutu, ödünç geri verildikten sonra da onu gösterebilir, yani *sarkan referans* oluşabilirdi. Önce hatayı görelim:

```rust,compile_fail
trait Sablon {
    fn uygula(&self, mesaj: &str) -> String;
}

struct Onek<'a>(&'a str); // referans tutan tür

impl Sablon for Onek<'_> {
    fn uygula(&self, mesaj: &str) -> String {
        format!("{}{mesaj}", self.0)
    }
}

fn sablon_kutula(onek: &str) -> Box<dyn Sablon> {
    Box::new(Onek(onek))
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error: lifetime may not live long enough
   | fn sablon_kutula(onek: &str) -> Box<dyn Sablon> {
   |                        - let's call the lifetime of this reference `'1`
   |     Box::new(Onek(onek))
   |     ^^^^^^^^^^^^^^^^^^^^ returning this value requires that `'1` must outlive `'static`
   |
help: to declare that the trait object captures data from argument `onek`, you can add an explicit `'_` lifetime bound
```

Bu hatanın kodu yoktur. Derleyici çözümü de söyler: Sınırı `+ '_` ile açıkça gevşetirsiniz. `'_` burada girdideki referansın ömrüdür; kutu, `onek` yaşadığı sürece geçerli olur:

```rust
# trait Sablon {
#     fn uygula(&self, mesaj: &str) -> String;
# }
#
# struct Onek<'a>(&'a str);
#
# impl Sablon for Onek<'_> {
#     fn uygula(&self, mesaj: &str) -> String {
#         format!("{}{mesaj}", self.0)
#     }
# }
#
fn sablon_kutula(onek: &str) -> Box<dyn Sablon + '_> {
    Box::new(Onek(onek))
}

fn main() {
    let onek = String::from(">> ");
    let sablon = sablon_kutula(&onek);
    println!("{}", sablon.uygula("ödünç tutan trait object"));
}
```

Çıktı:

```text
>> ödünç tutan trait object
```

Fonksiyon gövdesinde kural biraz gevşer: Yerel bir değişkenin türünde `Box<dyn Sablon>` yazdığınızda sınır `'static` sayılmaz, kullanımdan çıkarılır.[^varsayilan-omur] Ödünç bilgisi yine taşınır; bu kez `E0597` görürsünüz:

```rust,compile_fail,E0597
# trait Sablon {
#     fn uygula(&self, mesaj: &str) -> String;
# }
#
# struct Onek<'a>(&'a str);
#
# impl Sablon for Onek<'_> {
#     fn uygula(&self, mesaj: &str) -> String {
#         format!("{}{mesaj}", self.0)
#     }
# }
#
fn main() {
    let mut sablonlar: Vec<Box<dyn Sablon>> = Vec::new();
    {
        let s = String::from(">> ");
        sablonlar.push(Box::new(Onek(&s)));
    }
    println!("{}", sablonlar[0].uygula("merhaba"));
}
```

Derleyicinin yanıtı:

```text
error[E0597]: `s` does not live long enough
   |         let s = String::from(">> ");
   |             - binding `s` declared here
   |         sablonlar.push(Box::new(Onek(&s)));
   |                                      ^^ borrowed value does not live long enough
   |     }
   |     - `s` dropped here while still borrowed
   |     println!("{}", sablonlar[0].uygula("merhaba"));
   |                    --------- borrow later used here
```

### `kaydet` neden `+ 'static` istemedi?

`Dagitici`, kanalları `Box<dyn Kanal>` olarak sakladığı için aslında `Box<dyn Kanal + 'static>` saklar. O hâlde `kaydet(kanal: impl Kanal)` imzası nasıl derlendi? Cevap `Any` ön şartındadır: `Any` her uygulayandan `'static` olmasını istediği için `impl Kanal` da kendiliğinden `'static` olur. Ön şartı kaldırırsanız derleyici sınırı sizden ister:

```rust,compile_fail,E0310
trait Kanal {
    fn ad(&self) -> &'static str;
}

struct Dagitici {
    kanallar: Vec<Box<dyn Kanal>>,
}

impl Dagitici {
    fn kaydet(&mut self, kanal: impl Kanal) {
        self.kanallar.push(Box::new(kanal));
    }
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0310]: the parameter type `impl Kanal` may not live long enough
   |         self.kanallar.push(Box::new(kanal));
   |                            ^^^^^^^^^^^^^^^
   |                            |
   |                            the parameter type `impl Kanal` must be valid for the static lifetime...
   |                            ...so that the type `impl Kanal` will meet its required lifetime bounds
   |
help: consider adding an explicit lifetime bound
   |     fn kaydet(&mut self, kanal: impl Kanal + 'static) {
   |                                            +++++++++
```

## Kalıtım yerine bileşim: sarmalayıcı

Kalıtımlı bir dilde acil mesajlar için "`AcilSms`, `Sms`'ten türesin" derdiniz. Düdüğü daha gür öttürmek için ise yeni bir düdük icat etmek gerekmez; eldeki düdüğü bir huninin içine yerleştirirsiniz. Huni yine ses çıkaran bir şeydir: Sesi içindeki düdüğe çıkarttırır, üstüne yalnızca kendi katkısını ekler.

`Etiketli` böyle bir *sarmalayıcıdır* (wrapper). Başka bir kanalı alan olarak tutar, işin çoğunu ona *devreder* (delegation), yalnızca bir adımı değiştirir. Sarmayı kolaylaştırmak için trait'e bir de `etiketli` metodu ekleyelim:

```rust
# use std::any::Any;
# use std::fmt::Debug;
#
pub trait Kanal: Debug + Any {
    // ad, bicimle, azami_uzunluk ve gonder aynı
#     fn ad(&self) -> &'static str;
#     fn bicimle(&self, mesaj: &str) -> String;
#
#     fn azami_uzunluk(&self) -> Option<usize> {
#         None
#     }
#
#     fn gonder(&self, mesaj: &str) -> String {
#         let govde: String = match self.azami_uzunluk() {
#             Some(sinir) => mesaj.chars().take(sinir).collect(),
#             None => mesaj.to_string(),
#         };
#         format!("[{}] {}", self.ad(), self.bicimle(&govde))
#     }

    fn etiketli(self, etiket: &'static str) -> Etiketli<Self>
    where
        Self: Sized, // bu metot vtable'a girmez
    {
        Etiketli { ic: self, etiket }
    }
}

#[derive(Debug, Clone)]
pub struct Etiketli<K> {
    ic: K,
    etiket: &'static str,
}

impl<K: Kanal> Kanal for Etiketli<K> {
    fn ad(&self) -> &'static str {
        self.ic.ad() // devretme
    }
    fn bicimle(&self, mesaj: &str) -> String {
        format!("{} {}", self.etiket, self.ic.bicimle(mesaj)) // değişen tek adım
    }
    fn azami_uzunluk(&self) -> Option<usize> {
        self.ic.azami_uzunluk() // devretme
    }
}
#
# #[derive(Debug, Clone)]
# struct Eposta {
#     adres: String,
# }
#
# impl Kanal for Eposta {
#     fn ad(&self) -> &'static str {
#         "eposta"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} adresine: {mesaj}", self.adres)
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Sms {
#     numara: String,
# }
#
# impl Kanal for Sms {
#     fn ad(&self) -> &'static str {
#         "sms"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} numarasına: {mesaj}", self.numara)
#     }
#     fn azami_uzunluk(&self) -> Option<usize> {
#         Some(12)
#     }
# }

fn main() {
    let acil = Sms { numara: String::from("444") }.etiketli("[ACİL]").etiketli("[TEST]");
    println!("{}", acil.gonder("Sunucu yanıt vermiyor"));

    let duyuru = Eposta { adres: String::from("a@b.c") }.etiketli("[DUYURU]");
    println!("{}", duyuru.gonder("Bakım var"));
}
```

Çıktı:

```text
[sms] [TEST] [ACİL] 444 numarasına: Sunucu yanıt
[eposta] [DUYURU] a@b.c adresine: Bakım var
```

Kazanç açıktır: `Etiketli` tek bir taban türe bağlı değildir, her `Kanal`'ı sarar; sarmalayıcılar iç içe de geçebilir. Bedeli ise devredilen her metodu elle yazmaktır. Rust bunu kendiliğinden yapmaz; burada üç metot yazmak yetti.

`etiketli` metodundaki `where Self: Sized` şartına dikkat edin. Metot `self`'i değer olarak alır ve dönüş türünde `Self` geçer (`Etiketli<Self>`); bu yüzden trait nesnesi üzerinden çağrılamaz. Şart, metodu vtable'ın dışında bırakır: Trait dyn uyumlu kalır, metot yalnızca somut türlerde çağrılır.[^dyn] Standart kütüphane de aynı yolu izler: `Iterator::map` ve `Iterator::filter` metotlarında `where Self: Sized` bulunur, `dyn Iterator` böyle mümkün olur.[^iter-map]

## Yukarı dönüşüm ve `dyn Any`

*Yukarı dönüşüm* (upcasting), bir trait nesnesine supertrait'inin gözüyle bakmaktır: `&dyn Kanal`, `&dyn Debug` ya da `&dyn Any` yapılabilir. "Ses çıkaran oyuncak"a yalnızca "bir oyuncak" diye bakmak gibidir ve zararsızdır. Rust 1.86'dan beri kararlıdır;[^upcast] öncesinde her trait'e elle `fn as_any(&self) -> &dyn Any` gibi bir metot yazılırdı.

`dyn Any` ise ters yöndeki soruyu sordurur: "Sen aslında şu tür müsün?" Buna *downcast* denir ve cevap çalışma zamanında verilir:[^any]

```rust
# use std::any::Any;
# use std::fmt::Debug;
#
# pub trait Kanal: Debug + Any {
#     fn ad(&self) -> &'static str;
#     fn bicimle(&self, mesaj: &str) -> String;
#
#     fn azami_uzunluk(&self) -> Option<usize> {
#         None
#     }
#
#     fn gonder(&self, mesaj: &str) -> String {
#         let govde: String = match self.azami_uzunluk() {
#             Some(sinir) => mesaj.chars().take(sinir).collect(),
#             None => mesaj.to_string(),
#         };
#         format!("[{}] {}", self.ad(), self.bicimle(&govde))
#     }
#
#     fn etiketli(self, etiket: &'static str) -> Etiketli<Self>
#     where
#         Self: Sized,
#     {
#         Etiketli { ic: self, etiket }
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Eposta {
#     adres: String,
# }
#
# impl Kanal for Eposta {
#     fn ad(&self) -> &'static str {
#         "eposta"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} adresine: {mesaj}", self.adres)
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Sms {
#     numara: String,
# }
#
# impl Kanal for Sms {
#     fn ad(&self) -> &'static str {
#         "sms"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} numarasına: {mesaj}", self.numara)
#     }
#     fn azami_uzunluk(&self) -> Option<usize> {
#         Some(12)
#     }
# }
#
# #[derive(Debug, Clone)]
# pub struct Etiketli<K> {
#     ic: K,
#     etiket: &'static str,
# }
#
# impl<K: Kanal> Kanal for Etiketli<K> {
#     fn ad(&self) -> &'static str {
#         self.ic.ad()
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} {}", self.etiket, self.ic.bicimle(mesaj))
#     }
#     fn azami_uzunluk(&self) -> Option<usize> {
#         self.ic.azami_uzunluk()
#     }
# }
#
fn main() {
    let sms = Sms { numara: String::from("333") };
    let kanal: &dyn Kanal = &sms;

    let hata_ayiklama: &dyn Debug = kanal; // yukarı dönüşüm: yalnızca Debug yüzü görünür
    println!("Debug yüzü: {hata_ayiklama:?}");

    let herhangi: &dyn Any = kanal; // yukarı dönüşüm
    println!("downcast: {:?}", herhangi.downcast_ref::<Sms>().map(|s| s.numara.as_str()));
    println!("Eposta'ya downcast: {}", herhangi.downcast_ref::<Eposta>().is_some());
}
```

Çıktı:

```text
Debug yüzü: Sms { numara: "333" }
downcast: Some("333")
Eposta'ya downcast: false
```

### Downcast ne zaman kokudur?

Kutudan çıkan oyuncağa "sen düdük müsün?" diye sorabilirsiniz. Ama düdük renkli bir kılıfa sarılmışsa "hayır" cevabını alırsınız, oysa hâlâ öter. Doğru soru "ötebilir misin?" sorusudur. Kodda da soruyu türe değil, trait'in bir metoduna sormak gerekir:

```rust
# use std::any::Any;
# use std::fmt::Debug;
#
# pub trait Kanal: Debug + Any {
#     fn ad(&self) -> &'static str;
#     fn bicimle(&self, mesaj: &str) -> String;
#
#     fn azami_uzunluk(&self) -> Option<usize> {
#         None
#     }
#
#     fn gonder(&self, mesaj: &str) -> String {
#         let govde: String = match self.azami_uzunluk() {
#             Some(sinir) => mesaj.chars().take(sinir).collect(),
#             None => mesaj.to_string(),
#         };
#         format!("[{}] {}", self.ad(), self.bicimle(&govde))
#     }
#
#     fn etiketli(self, etiket: &'static str) -> Etiketli<Self>
#     where
#         Self: Sized,
#     {
#         Etiketli { ic: self, etiket }
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Eposta {
#     adres: String,
# }
#
# impl Kanal for Eposta {
#     fn ad(&self) -> &'static str {
#         "eposta"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} adresine: {mesaj}", self.adres)
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Sms {
#     numara: String,
# }
#
# impl Kanal for Sms {
#     fn ad(&self) -> &'static str {
#         "sms"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} numarasına: {mesaj}", self.numara)
#     }
#     fn azami_uzunluk(&self) -> Option<usize> {
#         Some(12)
#     }
# }
#
# #[derive(Debug, Clone)]
# pub struct Etiketli<K> {
#     ic: K,
#     etiket: &'static str,
# }
#
# impl<K: Kanal> Kanal for Etiketli<K> {
#     fn ad(&self) -> &'static str {
#         self.ic.ad()
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} {}", self.etiket, self.ic.bicimle(mesaj))
#     }
#     fn azami_uzunluk(&self) -> Option<usize> {
#         self.ic.azami_uzunluk()
#     }
# }
#
fn kirpar_mi_kokulu(kanal: &dyn Kanal) -> bool {
    let herhangi: &dyn Any = kanal;
    herhangi.is::<Sms>() // "Sms misin?"
}

fn kirpar_mi(kanal: &dyn Kanal) -> bool {
    kanal.azami_uzunluk().is_some() // "sınırın var mı?"
}

fn main() {
    let sms = Sms { numara: String::from("333") };
    let sarili = sms.clone().etiketli("[ACİL]");

    println!("kokulu: sms={} sarılı={}", kirpar_mi_kokulu(&sms), kirpar_mi_kokulu(&sarili));
    println!("doğru : sms={} sarılı={}", kirpar_mi(&sms), kirpar_mi(&sarili));
}
```

Çıktı:

```text
kokulu: sms=true sarılı=false
doğru : sms=true sarılı=true
```

Genel koda "eğer `Sms` ise" diye yazmak, açık kümenin sözünü bozar: `kirpar_mi_kokulu`, `Etiketli<Sms>`'i de yarın eklenecek her kısa mesaj kanalını da kaçırır. Bu yüzden *koku* (code smell) sayılır.

Downcast'in yerinde olduğu durumlar da vardır. Türü gerçekten bilinmeyen kaplar (türden değere giden haritalar, eklentinin kendi durumunu geri alması) ve bir `Box<dyn Error>` içinden belirli bir hatayı ayıklamak böyledir.[^error-downcast] Hataları [8. bölümde](../temel/08-error-handling.md) ele aldık.

## `Box<dyn Kanal>` klonlamak

`clone` "kendi türümden bir tane daha ver" demektir; dönüş türü `Self`'tir. Kapalı kutuda tür bilinmediği için bu söz tutulamaz: `Box<dyn Kanal>` kendiliğinden `Clone` değildir, `trait Kanal: Clone` yazmak da trait'i dyn uyumsuz yapar (ikisini de [aşağıda](#boxdyn-trait-değerini-doğrudan-klonlamak) göreceğiz).

Çare, `Self` yerine **kutu** döndüren bir yardımcı metottur: "Aynımdan yap, kutuya koyup ver." Bunu her türe elle yazmamak için de bir *genel impl* (blanket impl) kullanırız:[^blanket]

```rust
# use std::any::Any;
# use std::fmt::Debug;
#
pub trait Kanal: Debug + Any + KanalKlonla {
    // ad, bicimle, azami_uzunluk, gonder ve etiketli aynı
#     fn ad(&self) -> &'static str;
#     fn bicimle(&self, mesaj: &str) -> String;
#
#     fn azami_uzunluk(&self) -> Option<usize> {
#         None
#     }
#
#     fn gonder(&self, mesaj: &str) -> String {
#         let govde: String = match self.azami_uzunluk() {
#             Some(sinir) => mesaj.chars().take(sinir).collect(),
#             None => mesaj.to_string(),
#         };
#         format!("[{}] {}", self.ad(), self.bicimle(&govde))
#     }
#
#     fn etiketli(self, etiket: &'static str) -> Etiketli<Self>
#     where
#         Self: Sized,
#     {
#         Etiketli { ic: self, etiket }
#     }
}

pub trait KanalKlonla {
    fn kutu_klonu(&self) -> Box<dyn Kanal>;
}

// Kanal ve Clone olan her T için hazır
impl<T: Kanal + Clone> KanalKlonla for T {
    fn kutu_klonu(&self) -> Box<dyn Kanal> {
        Box::new(self.clone()) // T burada somut bir türdür
    }
}

impl Clone for Box<dyn Kanal> {
    fn clone(&self) -> Self {
        (**self).kutu_klonu() // vtable üzerinden içteki türe gider
    }
}

#[derive(Debug, Clone)]
pub struct Etiketli<K> {
    ic: K,
    etiket: &'static str,
}

// KanalKlonla ön şartı K'nin de Clone olmasını gerektirir
impl<K: Kanal + Clone> Kanal for Etiketli<K> {
#     fn ad(&self) -> &'static str {
#         self.ic.ad()
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} {}", self.etiket, self.ic.bicimle(mesaj))
#     }
#     fn azami_uzunluk(&self) -> Option<usize> {
#         self.ic.azami_uzunluk()
#     }
    // ad, bicimle ve azami_uzunluk aynı
}
#
# #[derive(Debug, Clone)]
# struct Eposta {
#     adres: String,
# }
#
# impl Kanal for Eposta {
#     fn ad(&self) -> &'static str {
#         "eposta"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} adresine: {mesaj}", self.adres)
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Sms {
#     numara: String,
# }
#
# impl Kanal for Sms {
#     fn ad(&self) -> &'static str {
#         "sms"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} numarasına: {mesaj}", self.numara)
#     }
#     fn azami_uzunluk(&self) -> Option<usize> {
#         Some(12)
#     }
# }
#
# use std::collections::HashMap;
#
# #[derive(Debug, Clone)]
# struct Webhook {
#     url: String,
# }
#
# impl Kanal for Webhook {
#     fn ad(&self) -> &'static str {
#         "webhook"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("POST {} gövde={mesaj:?}", self.url)
#     }
# }
#
# #[derive(Default, Clone)]
# struct Dagitici {
#     kanallar: HashMap<String, Box<dyn Kanal>>,
# }
#
# impl Dagitici {
#     fn kaydet(&mut self, kanal: impl Kanal) {
#         self.kaydet_kutulu(Box::new(kanal));
#     }
#
#     fn kaydet_kutulu(&mut self, kanal: Box<dyn Kanal>) {
#         self.kanallar.insert(kanal.ad().to_string(), kanal);
#     }
#
#     fn gonder(&self, kanal_adi: &str, mesaj: &str) -> Option<String> {
#         self.kanallar.get(kanal_adi).map(|k| k.gonder(mesaj))
#     }
#
#     fn adlar(&self) -> Vec<&str> {
#         let mut adlar: Vec<&str> = self.kanallar.keys().map(String::as_str).collect();
#         adlar.sort_unstable();
#         adlar
#     }
# }
#
# fn ornek_dagitici() -> Dagitici {
#     let mut dagitici = Dagitici::default();
#     dagitici.kaydet(Eposta { adres: String::from("ece@ornek.com") });
#     dagitici.kaydet(Sms { numara: String::from("555-0101") });
#     dagitici.kaydet(Webhook { url: String::from("/kanca") });
#     dagitici
# }

fn main() {
    let acil = Sms { numara: String::from("444") }.etiketli("[ACİL]").etiketli("[TEST]");
    let asil: Box<dyn Kanal> = Box::new(acil);
    let kopya = asil.clone(); // somut türü bilmeden derin kopya
    println!("kopya: {kopya:?}");

    let dagitici = ornek_dagitici();
    let mut yedek = dagitici.clone();
    yedek.kaydet_kutulu(kopya); // yalnızca yedekteki "sms" kaydının üzerine yazar
    println!("yedek: {:?}", yedek.gonder("sms", "Deneme"));
    println!("asıl : {:?}", dagitici.gonder("sms", "Deneme"));
}
```

Çıktı:

```text
kopya: Etiketli { ic: Etiketli { ic: Sms { numara: "444" }, etiket: "[ACİL]" }, etiket: "[TEST]" }
yedek: Some("[sms] [TEST] [ACİL] 444 numarasına: Deneme")
asıl : Some("[sms] 555-0101 numarasına: Deneme")
```

Artık `Box<dyn Kanal>` `Clone`'dur; onu alan olarak tutan `Dagitici` de `#[derive(Default, Clone)]` diyebilir. Yedekte yapılan değişiklik asıl dağıtıcıya yansımaz.

`(**self)` yazımı önemli bir sigortadır. `self` burada `&Box<dyn Kanal>`'dır; iki yıldız bizi kutunun içindeki `dyn Kanal`'a götürür ve çağrı vtable üzerinden içteki somut türe gider. Yalnızca `self.kutu_klonu()` yazsaydınız bugün sonuç değişmezdi. Ama ileride biri `impl Kanal for Box<dyn Kanal>` eklerse kutunun kendisi de `Kanal + Clone` olur ve genel impl onun için de devreye girer: `self.kutu_klonu()` kutunun kendi uygulamasını bulur, o `Box::clone`'u çağırır, o yine `kutu_klonu`'nu çağırır. Sonuç sonsuz özyinelemedir; denediğimizde program stack taşmasıyla (stack overflow) sonlandı.

## Sık yapılan hatalar

Üç hatanın da kökü aynıdır: Kapalı kutudaki türün ne adı ne boyu bilinir.

### Dyn uyumsuz metot eklemek (`E0038`)

`Self` döndüren ya da jenerik parametresi olan bir metot, trait'i dyn uyumsuz yapar:[^dyn]

```rust,compile_fail,E0038
trait Kanal {
    fn kopya(&self) -> Self;
}

fn bildir(_: &dyn Kanal) {}
```

Derleyicinin yanıtı:

```text
error[E0038]: the trait `Kanal` is not dyn compatible
   | fn bildir(_: &dyn Kanal) {}
   |               ^^^^^^^^^ `Kanal` is not dyn compatible
   |
note: for a trait to be dyn compatible it needs to allow building a vtable
   | trait Kanal {
   |       ----- this trait is not dyn compatible...
   |     fn kopya(&self) -> Self;
   |                        ^^^^ ...because method `kopya` references the `Self` type in its return type
   = help: consider moving `kopya` to another trait
```

Jenerik metotta da kod aynıdır, neden farklıdır. Her `T` için ayrı bir kopya üretileceğinden tabloya hangisinin yazılacağı bilinemez:

```rust,compile_fail,E0038
use std::fmt::Display;

trait Kanal {
    fn gonder<T: Display>(&self, mesaj: T) -> String;
}

fn bildir(_: &dyn Kanal) {}
```

Derleyicinin yanıtı:

```text
error[E0038]: the trait `Kanal` is not dyn compatible
   |     fn gonder<T: Display>(&self, mesaj: T) -> String;
   |        ^^^^^^ ...because method `gonder` has generic type parameters
   = help: consider moving `gonder` to another trait
```

Çözüm, o metodu `where Self: Sized` ile trait nesnesinin dışında bırakmak ya da jenerik parametre yerine `&dyn Display` gibi bir trait nesnesi almaktır:

```rust
use std::fmt::Display;

trait Kanal {
    fn gonder(&self, mesaj: &dyn Display) -> String;

    fn kopya(&self) -> Self
    where
        Self: Sized;
}

fn bildir(kanal: &dyn Kanal) -> String {
    kanal.gonder(&42)
}
# #[derive(Clone)]
# struct Sms;
# impl Kanal for Sms {
#     fn gonder(&self, mesaj: &dyn Display) -> String { format!("[sms] {mesaj}") }
#     fn kopya(&self) -> Self { self.clone() }
# }
# fn main() {
#     assert_eq!(bildir(&Sms), "[sms] 42");
#     let _ = Sms.kopya();
# }
```

Bedeli, dışarıda bırakılan metodun trait nesnesi üzerinde çağrılamamasıdır:

```rust,compile_fail
# trait Kanal {
#     fn ad(&self) -> &'static str;
#     fn etiketli(self, etiket: &'static str) -> Etiketli<Self>
#     where
#         Self: Sized,
#     {
#         Etiketli { ic: self, etiket }
#     }
# }
# struct Etiketli<K> { ic: K, etiket: &'static str }
# struct Sms;
# impl Kanal for Sms { fn ad(&self) -> &'static str { "sms" } }
fn main() {
    let kutu: Box<dyn Kanal> = Box::new(Sms);
    let acil = kutu.etiketli("[ACİL]");
}
```

Derleyicinin yanıtı:

```text
error: the `etiketli` method cannot be invoked on a trait object
   |         Self: Sized,
   |               ----- this has a `Sized` requirement
...
   |     let acil = kutu.etiketli("[ACİL]");
   |                     ^^^^^^^^
```

Bu hatanın da kodu yoktur.[^cagrilamaz]

### Trait nesnesini işaretçisiz kullanmak (`E0277`)

`dyn Kanal` dinamik boyutlu bir türdür; ne listeye ne de parametreye doğrudan konabilir:

```rust,compile_fail,E0277
# use std::any::Any;
# use std::fmt::Debug;
#
# pub trait Kanal: Debug + Any {
#     fn ad(&self) -> &'static str;
#     fn bicimle(&self, mesaj: &str) -> String;
#
#     fn azami_uzunluk(&self) -> Option<usize> {
#         None
#     }
#
#     fn gonder(&self, mesaj: &str) -> String {
#         let govde: String = match self.azami_uzunluk() {
#             Some(sinir) => mesaj.chars().take(sinir).collect(),
#             None => mesaj.to_string(),
#         };
#         format!("[{}] {}", self.ad(), self.bicimle(&govde))
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Eposta {
#     adres: String,
# }
#
# impl Kanal for Eposta {
#     fn ad(&self) -> &'static str {
#         "eposta"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} adresine: {mesaj}", self.adres)
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Sms {
#     numara: String,
# }
#
# impl Kanal for Sms {
#     fn ad(&self) -> &'static str {
#         "sms"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} numarasına: {mesaj}", self.numara)
#     }
#     fn azami_uzunluk(&self) -> Option<usize> {
#         Some(12)
#     }
# }
#
fn bir_kez_bildir(kanal: dyn Kanal, mesaj: &str) -> String {
    kanal.gonder(mesaj)
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0277]: the size for values of type `(dyn Kanal + 'static)` cannot be known at compilation time
   | fn bir_kez_bildir(kanal: dyn Kanal, mesaj: &str) -> String {
   |                          ^^^^^^^^^ doesn't have a size known at compile-time
   |
   = help: the trait `Sized` is not implemented for `(dyn Kanal + 'static)`
help: function arguments must have a statically known size, borrowed types always have a known size
   | fn bir_kez_bildir(kanal: &dyn Kanal, mesaj: &str) -> String {
   |                          +
```

Parametrede `&dyn Kanal` (ya da statik gönderim isteniyorsa `&impl Kanal`), listede `Vec<Box<dyn Kanal>>` yazın:

```rust
# use std::any::Any;
# use std::fmt::Debug;
#
# pub trait Kanal: Debug + Any {
#     fn ad(&self) -> &'static str;
#     fn bicimle(&self, mesaj: &str) -> String;
#
#     fn azami_uzunluk(&self) -> Option<usize> {
#         None
#     }
#
#     fn gonder(&self, mesaj: &str) -> String {
#         let govde: String = match self.azami_uzunluk() {
#             Some(sinir) => mesaj.chars().take(sinir).collect(),
#             None => mesaj.to_string(),
#         };
#         format!("[{}] {}", self.ad(), self.bicimle(&govde))
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Eposta {
#     adres: String,
# }
#
# impl Kanal for Eposta {
#     fn ad(&self) -> &'static str {
#         "eposta"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} adresine: {mesaj}", self.adres)
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Sms {
#     numara: String,
# }
#
# impl Kanal for Sms {
#     fn ad(&self) -> &'static str {
#         "sms"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} numarasına: {mesaj}", self.numara)
#     }
#     fn azami_uzunluk(&self) -> Option<usize> {
#         Some(12)
#     }
# }
#
fn bir_kez_bildir(kanal: &dyn Kanal, mesaj: &str) -> String {
    kanal.gonder(mesaj)
}
# fn main() {
#     let s = Sms { numara: String::from("1") };
#     assert_eq!(bir_kez_bildir(&s, "selam"), "[sms] 1 numarasına: selam");
# }
```

Nedeni [22. bölümde](../ileri/22-sized-dst.md) ayrıntısıyla anlatılır.[^dst]

### `Box<dyn Trait>` değerini doğrudan klonlamak

Trait'te klonlama desteği yokken `clone` çağırmak derlenmez:

```rust,compile_fail,E0599
# trait Kanal {
#     fn ad(&self) -> &'static str;
# }
# struct Sms;
# impl Kanal for Sms { fn ad(&self) -> &'static str { "sms" } }
fn main() {
    let kutu: Box<dyn Kanal> = Box::new(Sms);
    let k2 = kutu.clone();
}
```

Derleyicinin yanıtı:

```text
error[E0599]: the method `clone` exists for struct `Box<dyn Kanal>`, but its trait bounds were not satisfied
   |     let k2 = kutu.clone();
   |                   ^^^^^ method cannot be called on `Box<dyn Kanal>` due to unsatisfied trait bounds
   = note: the following trait bounds were not satisfied:
           `dyn Kanal: Sized`
           which is required by `Box<dyn Kanal>: Clone`
           `dyn Kanal: Clone`
           which is required by `Box<dyn Kanal>: Clone`
```

İlk akla gelen "o zaman `trait Kanal: Clone` yazayım" olur. Bu da trait'i dyn uyumsuz yapar:

```rust,compile_fail,E0038
trait Kanal: Clone {
    fn ad(&self) -> &'static str;
}

fn bildir(_: &dyn Kanal) {}
```

Derleyicinin yanıtı:

```text
error[E0038]: the trait `Kanal` is not dyn compatible
   | fn bildir(_: &dyn Kanal) {}
   |               ^^^^^^^^^ `Kanal` is not dyn compatible
   |
note: for a trait to be dyn compatible it needs to allow building a vtable
   | trait Kanal: Clone {
   |       -----  ^^^^^ ...because it requires `Self: Sized`
   |       |
   |       this trait is not dyn compatible...
```

Derleyicinin gösterdiği neden `Clone: Sized` ön şartıdır: `clone` değerin kendisini döndürür, döndürülen değerin boyutu bilinmelidir. Doğru çözüm, yukarıdaki `kutu_klonu` metodu ile `impl Clone for Box<dyn Kanal>` ikilisidir.

## Akılda kalması gerekenler

- Soru yalnızca "trait nesnesi mi, jenerik mi?" değildir. "Tür kümesini kim genişletecek ve liste heterojen mi?" diye sorun.
- Şablon metot ile kanca, kalıtımın "ortak iskelet" işini görür. Paylaşılan şey davranıştır, alan değildir.
- İşaretçi seçimi sahiplik sorusudur: kullanmak için `&dyn`, sahip olmak için `Box`, paylaşmak için `Rc` ya da `Arc`.
- `dyn Trait`, `Send`, `Sync` ve ömür bilgisini kendiliğinden taşımaz; `+ Send` ve `+ 'a` ile siz yazarsınız.
- Downcast yerine trait'e bir metot ekleyin; kalıtım yerine sarıp devredin.
- Trait nesnesini klonlamak için `Self` değil kutu döndüren bir yardımcı metot gerekir.

## Alıştırmalar

Çözümlerin tamamı derlenip denenmiştir. Önce kendiniz deneyin.

### 1. Konsol kanalı

`Konsol` adında, alanı olmayan yeni bir kanal yazın ve `ornek_dagitici` içinde kaydedin. `Dagitici`'ye ve `Kanal`'a dokunmayın. `adlar()` artık `["eposta", "konsol", "sms", "webhook"]` döndürmelidir.

Notlardaki dosyada çalışıyorsanız `kutulu_kanal_ve_dagitici_bagimsiz_klonlanir` testi ve ilk bölümün beklenen çıktısı da `ornek_dagitici`'ye bağlıdır; ikisini yeni listeye göre güncellemeniz gerekir.

<details>
<summary>İpucu ve çözüm</summary>

`derive(Debug, Clone)` ile `ad` ve `bicimle` yeterlidir.

```rust
# use std::any::Any;
# use std::fmt::Debug;
#
# pub trait Kanal: Debug + Any {
#     fn ad(&self) -> &'static str;
#     fn bicimle(&self, mesaj: &str) -> String;
#
#     fn azami_uzunluk(&self) -> Option<usize> {
#         None
#     }
#
#     fn gonder(&self, mesaj: &str) -> String {
#         let govde: String = match self.azami_uzunluk() {
#             Some(sinir) => mesaj.chars().take(sinir).collect(),
#             None => mesaj.to_string(),
#         };
#         format!("[{}] {}", self.ad(), self.bicimle(&govde))
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Eposta {
#     adres: String,
# }
#
# impl Kanal for Eposta {
#     fn ad(&self) -> &'static str {
#         "eposta"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} adresine: {mesaj}", self.adres)
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Sms {
#     numara: String,
# }
#
# impl Kanal for Sms {
#     fn ad(&self) -> &'static str {
#         "sms"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} numarasına: {mesaj}", self.numara)
#     }
#     fn azami_uzunluk(&self) -> Option<usize> {
#         Some(12)
#     }
# }
#
# use std::collections::HashMap;
#
# #[derive(Default)]
# struct Dagitici {
#     kanallar: HashMap<String, Box<dyn Kanal>>,
# }
#
# impl Dagitici {
#     fn kaydet(&mut self, kanal: impl Kanal) {
#         self.kaydet_kutulu(Box::new(kanal));
#     }
#
#     fn kaydet_kutulu(&mut self, kanal: Box<dyn Kanal>) {
#         self.kanallar.insert(kanal.ad().to_string(), kanal);
#     }
#
#     fn gonder(&self, kanal_adi: &str, mesaj: &str) -> Option<String> {
#         self.kanallar.get(kanal_adi).map(|k| k.gonder(mesaj))
#     }
#
#     fn adlar(&self) -> Vec<&str> {
#         let mut adlar: Vec<&str> = self.kanallar.keys().map(String::as_str).collect();
#         adlar.sort_unstable();
#         adlar
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Webhook {
#     url: String,
# }
#
# impl Kanal for Webhook {
#     fn ad(&self) -> &'static str {
#         "webhook"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("POST {} gövde={mesaj:?}", self.url)
#     }
# }
#
#[derive(Debug, Clone)]
struct Konsol;

impl Kanal for Konsol {
    fn ad(&self) -> &'static str {
        "konsol"
    }
    fn bicimle(&self, mesaj: &str) -> String {
        mesaj.to_string()
    }
}

fn ornek_dagitici() -> Dagitici {
    let mut dagitici = Dagitici::default();
    dagitici.kaydet(Eposta { adres: String::from("ece@ornek.com") });
    dagitici.kaydet(Sms { numara: String::from("555-0101") });
    dagitici.kaydet(Webhook { url: String::from("/kanca") });
    dagitici.kaydet(Konsol);
    dagitici
}

fn main() {
    println!("{:?}", ornek_dagitici().adlar());
}
```

Çıktı:

```text
["eposta", "konsol", "sms", "webhook"]
```

</details>

### 2. Öncelik sırası

`Kanal`'a `fn oncelik(&self) -> u8 { 0 }` kancasını ekleyin, `Sms`'te 2 döndürün. `Dagitici`'ye de bir mesajı bütün kanallara öncelik sırasıyla (büyükten küçüğe, eşitlikte ada göre) gönderen `herkese` metodunu yazın. İlk satır `[sms]` ile başlamalı, sıra her çalıştırmada aynı olmalıdır.

<details>
<summary>İpucu ve çözüm</summary>

`values()` ile `&Box<dyn Kanal>` listesi alın, `sort_by_key` ile `(Reverse(öncelik), ad)` çiftine göre sıralayın. `Reverse`, sıralamayı tersine çevirir.

```rust
use std::cmp::Reverse;
# use std::any::Any;
# use std::collections::HashMap;
# use std::fmt::Debug;

pub trait Kanal: Debug + Any {
    // öteki metotlar aynı
#     fn ad(&self) -> &'static str;
#     fn bicimle(&self, mesaj: &str) -> String;
#
#     fn azami_uzunluk(&self) -> Option<usize> {
#         None
#     }
#
#     fn gonder(&self, mesaj: &str) -> String {
#         let govde: String = match self.azami_uzunluk() {
#             Some(sinir) => mesaj.chars().take(sinir).collect(),
#             None => mesaj.to_string(),
#         };
#         format!("[{}] {}", self.ad(), self.bicimle(&govde))
#     }

    fn oncelik(&self) -> u8 {
        0
    }
}
#
# #[derive(Debug, Clone)]
# struct Eposta { adres: String }
# impl Kanal for Eposta {
#     fn ad(&self) -> &'static str { "eposta" }
#     fn bicimle(&self, mesaj: &str) -> String { format!("{} adresine: {mesaj}", self.adres) }
# }
#
# #[derive(Debug, Clone)]
# struct Sms { numara: String }

impl Kanal for Sms {
#     fn ad(&self) -> &'static str { "sms" }
#     fn bicimle(&self, mesaj: &str) -> String { format!("{} numarasına: {mesaj}", self.numara) }
#     fn azami_uzunluk(&self) -> Option<usize> { Some(12) }
    // öteki metotlar aynı
    fn oncelik(&self) -> u8 {
        2
    }
}
#
# #[derive(Debug, Clone)]
# struct Webhook {
#     url: String,
# }
#
# impl Kanal for Webhook {
#     fn ad(&self) -> &'static str {
#         "webhook"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("POST {} gövde={mesaj:?}", self.url)
#     }
# }
#
# #[derive(Default)]
# struct Dagitici {
#     kanallar: HashMap<String, Box<dyn Kanal>>,
# }

impl Dagitici {
#     fn kaydet(&mut self, kanal: impl Kanal) {
#         let kanal: Box<dyn Kanal> = Box::new(kanal);
#         self.kanallar.insert(kanal.ad().to_string(), kanal);
#     }
    fn herkese(&self, mesaj: &str) -> Vec<String> {
        let mut sirali: Vec<&Box<dyn Kanal>> = self.kanallar.values().collect();
        sirali.sort_by_key(|k| (Reverse(k.oncelik()), k.ad()));
        sirali.iter().map(|k| k.gonder(mesaj)).collect()
    }
}

fn main() {
    let mut dagitici = Dagitici::default();
    dagitici.kaydet(Eposta { adres: String::from("ece@ornek.com") });
    dagitici.kaydet(Sms { numara: String::from("555-0101") });
    dagitici.kaydet(Webhook { url: String::from("/kanca") });

    for satir in dagitici.herkese("Bakım") {
        println!("{satir}");
    }
}
```

Çıktı:

```text
[sms] 555-0101 numarasına: Bakım
[eposta] ece@ornek.com adresine: Bakım
[webhook] POST /kanca gövde="Bakım"
```

</details>

### 3. Enum'a yeni tür eklemek

`SabitKanal`'a bir `Webhook` varyantı ekleyin ve derleyicinin sizi nereye götürdüğünü not edin. Aynı eklemenin trait nesnesi çözümünde kaç yere dokunduğuyla karşılaştırın.

<details>
<summary>İpucu ve çözüm</summary>

Yalnızca varyantı eklediğinizde `gonder` içindeki `match` derlenmez:

```rust,compile_fail,E0004
# use std::any::Any;
# use std::fmt::Debug;
#
# pub trait Kanal: Debug + Any {
#     fn ad(&self) -> &'static str;
#     fn bicimle(&self, mesaj: &str) -> String;
#
#     fn azami_uzunluk(&self) -> Option<usize> {
#         None
#     }
#
#     fn gonder(&self, mesaj: &str) -> String {
#         let govde: String = match self.azami_uzunluk() {
#             Some(sinir) => mesaj.chars().take(sinir).collect(),
#             None => mesaj.to_string(),
#         };
#         format!("[{}] {}", self.ad(), self.bicimle(&govde))
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Eposta {
#     adres: String,
# }
#
# impl Kanal for Eposta {
#     fn ad(&self) -> &'static str {
#         "eposta"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} adresine: {mesaj}", self.adres)
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Sms {
#     numara: String,
# }
#
# impl Kanal for Sms {
#     fn ad(&self) -> &'static str {
#         "sms"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} numarasına: {mesaj}", self.numara)
#     }
#     fn azami_uzunluk(&self) -> Option<usize> {
#         Some(12)
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Webhook {
#     url: String,
# }
#
# impl Kanal for Webhook {
#     fn ad(&self) -> &'static str {
#         "webhook"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("POST {} gövde={mesaj:?}", self.url)
#     }
# }
#
enum SabitKanal {
    Eposta(Eposta),
    Sms(Sms),
    Webhook(Webhook),
}

impl SabitKanal {
    fn gonder(&self, mesaj: &str) -> String {
        match self {
            SabitKanal::Eposta(e) => e.gonder(mesaj),
            SabitKanal::Sms(s) => s.gonder(mesaj),
        }
    }
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0004]: non-exhaustive patterns: `&SabitKanal::Webhook(_)` not covered
   |         match self {
   |               ^^^^ pattern `&SabitKanal::Webhook(_)` not covered
```

Eksik kol eklenince sorun biter:

```rust
# use std::any::Any;
# use std::fmt::Debug;
#
# pub trait Kanal: Debug + Any {
#     fn ad(&self) -> &'static str;
#     fn bicimle(&self, mesaj: &str) -> String;
#
#     fn azami_uzunluk(&self) -> Option<usize> {
#         None
#     }
#
#     fn gonder(&self, mesaj: &str) -> String {
#         let govde: String = match self.azami_uzunluk() {
#             Some(sinir) => mesaj.chars().take(sinir).collect(),
#             None => mesaj.to_string(),
#         };
#         format!("[{}] {}", self.ad(), self.bicimle(&govde))
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Eposta {
#     adres: String,
# }
#
# impl Kanal for Eposta {
#     fn ad(&self) -> &'static str {
#         "eposta"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} adresine: {mesaj}", self.adres)
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Sms {
#     numara: String,
# }
#
# impl Kanal for Sms {
#     fn ad(&self) -> &'static str {
#         "sms"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("{} numarasına: {mesaj}", self.numara)
#     }
#     fn azami_uzunluk(&self) -> Option<usize> {
#         Some(12)
#     }
# }
#
# #[derive(Debug, Clone)]
# struct Webhook {
#     url: String,
# }
#
# impl Kanal for Webhook {
#     fn ad(&self) -> &'static str {
#         "webhook"
#     }
#     fn bicimle(&self, mesaj: &str) -> String {
#         format!("POST {} gövde={mesaj:?}", self.url)
#     }
# }
#
# enum SabitKanal {
#     Eposta(Eposta),
#     Sms(Sms),
#     Webhook(Webhook),
# }
#
impl SabitKanal {
    fn gonder(&self, mesaj: &str) -> String {
        match self {
            SabitKanal::Eposta(e) => e.gonder(mesaj),
            SabitKanal::Sms(s) => s.gonder(mesaj),
            SabitKanal::Webhook(w) => w.gonder(mesaj),
        }
    }
}

fn main() {
    let k = SabitKanal::Webhook(Webhook { url: String::from("/olay") });
    println!("{}", k.gonder("Merhaba"));
}
```

Çıktı:

```text
[webhook] POST /olay gövde="Merhaba"
```

Enum çözümünde enum'un tanımına ve her `match`'e dokundunuz; derleyici hepsini listeledi. Trait nesnesi çözümünde var olan kodda değişen satır sayısı sıfırdır: `Webhook` için bir `impl Kanal` yazmak yetmişti. Karşılaştırma tablosundaki "yeni tür eklemek" satırını böylece kendi gözünüzle doğrulamış oldunuz.

</details>

## Özet

Bildirim sistemi üç fikre dayanır. Çekirdek yalnızca sözleşmeyi (trait'i) tanır, yeni tür yeni bir `impl` demektir. Ortak iskelet şablon metotta durur, türler yalnızca farklı adımlarını yazar. Ek özellik kalıtımla değil, sarmalayıcıyla gelir. Trait nesnesinin sildiği bilgileri ise siz yazarsınız: thread güvenliği için `+ Send + Sync`, ödünç için `+ 'a`. Tür kümesi kapalıysa enum, türler aynı ve çağrı sıksa jenerik daha uygun olabilir.

Enum çözümünün gücü `match`'ten gelir: eksiksizlik denetimi, iç içe desenler ve koruyucu koşullar. Bunları [18. bölümde](18-patterns.md) ele alacağız. `Box<dyn Kanal>`'ın neden iki işaretçi boyunda olduğu, `where Self: Sized` şartının neyi dışarıda bıraktığı ve `Vec<dyn Kanal>`'ın neden imkânsız olduğu ise [22. bölümdedir](../ileri/22-sized-dst.md).

## Kaynaklar

[^varsayilan]: The Rust Programming Language, [10.2: Using Default Implementations](https://doc.rust-lang.org/book/ch10-02-traits.html#using-default-implementations).
[^supertrait]: The Rust Reference, [Supertraits](https://doc.rust-lang.org/reference/items/traits.html#supertraits).
[^any]: Standart kütüphane, [`std::any::Any`](https://doc.rust-lang.org/std/any/trait.Any.html) ve `core` 1.99.0 kaynak kodu, [`any.rs`, `pub trait Any`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/any.rs#L116).
[^gonderim]: The Rust Programming Language, [18.2: Performing Dynamic Dispatch](https://doc.rust-lang.org/book/ch18-02-trait-objects.html#performing-dynamic-dispatch).
[^enum-mu]: The Rust Programming Language, [18.3: Why Not An Enum?](https://doc.rust-lang.org/book/ch18-03-oo-design-patterns.html#why-not-an-enum) ve [8.1: Using an Enum to Store Multiple Types](https://doc.rust-lang.org/book/ch08-01-vectors.html#using-an-enum-to-store-multiple-types).
[^mono]: The Rust Programming Language, [10.1: Performance of Code Using Generics](https://doc.rust-lang.org/book/ch10-01-syntax.html#performance-of-code-using-generics).
[^ortuk]: The Rust Reference, [Unsized coercions](https://doc.rust-lang.org/reference/type-coercions.html#unsized-coercions).
[^auto]: The Rust Reference, [Auto traits](https://doc.rust-lang.org/reference/special-types-and-traits.html#auto-traits) ve [Trait objects](https://doc.rust-lang.org/reference/types/trait-object.html).
[^varsayilan-omur]: The Rust Reference, [Default trait object lifetimes](https://doc.rust-lang.org/reference/lifetime-elision.html#default-trait-object-lifetimes); rustc 1.99.0 kaynak kodu, [`dyn_trait.rs`, `lower_trait_object_lifetime`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_analysis/src/hir_ty_lowering/dyn_trait.rs#L449).
[^dyn]: The Rust Reference, [Dyn compatibility](https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility).
[^iter-map]: Standart kütüphane, [`Iterator::map`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.map).
[^upcast]: Rust Blog, [Announcing Rust 1.86.0: Trait upcasting](https://blog.rust-lang.org/2025/04/03/Rust-1.86.0/); The Rust Reference, [Unsized coercions](https://doc.rust-lang.org/reference/type-coercions.html#unsized-coercions).
[^error-downcast]: Standart kütüphane, [`Error::downcast_ref`](https://doc.rust-lang.org/std/error/trait.Error.html#method.downcast_ref).
[^blanket]: The Rust Reference, [Generic implementations](https://doc.rust-lang.org/reference/items/implementations.html#generic-implementations).
[^cagrilamaz]: rustc 1.99.0 kaynak kodu, [`suggest.rs`, `cannot be invoked on a trait object`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_typeck/src/method/suggest.rs#L389).
[^dst]: The Rust Programming Language, [20.3: Dynamically Sized Types and the Sized Trait](https://doc.rust-lang.org/book/ch20-03-advanced-types.html#dynamically-sized-types-and-the-sized-trait).
