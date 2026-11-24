# 17. Nesne Yönelimli Özellikler

Nesne yönelimli programlama (object-oriented programming, OOP) denince akla çoğunlukla üç özellik gelir: veriyi ve onu işleyen kodu bir arada tutan *nesneler*, iç ayrıntıyı dışarıdan saklayan *kapsülleme* ve bir türün başka bir türün özelliklerini devralması, yani *kalıtım*. Bir dilin hangi özelliklerle nesne yönelimli sayılacağı konusunda uzlaşma yoktur. Rust bu üçünden ilk ikisini sunar, kalıtımı sunmaz.[^oop]

Kalıtımın yokluğu bilinçli bir tercihtir. Rust'ta bir "sınıftan" başka bir sınıf türetmezsiniz. Bunun yerine veriyi modül sınırıyla korur, ortak davranışı trait ile tarif eder, yanlış kullanımı da türlerle önlersiniz.

Bu bölümde önce nesne ve kapsülleme kavramlarını, ardından kalıtımın yaptığı iki işin Rust'ta nasıl ayrıldığını göreceğiz. Sonra iki tasarım kalıbı kuracağız: Farklı türlerdeki arayüz bileşenlerini tek listede tutan bir *trait nesnesi* örneği ve bir yazının yayın sürecini türlerle ifade eden *type-state* kalıbı. Statik ve dinamik gönderimin temeli [9. bölümde](09-generics-traits-lifetimes.md), bellekteki karşılığı ise [22. bölümde](../advanced/22-sized-dst.md) anlatılır; burada ağırlık tasarımdadır.

> [!NOTE]
> Bu bölümün kodu `learn/oop` dizinindedir. Depo kökünden `cargo run -p oop` komutuyla çalıştırabilirsiniz.

## Günlük hayattan bir benzetme

Bir kumbara düşünün. Parayı yalnızca üstündeki yarıktan atabilirsiniz; önündeki gösterge de toplamı yazar. İçini elinizle karıştıramadığınız için gösterge hep doğrudur. Kumbaranın içi açık olsaydı biri para atıp göstergeyi değiştirmeyi unutabilir, gösterge yanlış kalırdı. Buna *kapsülleme* (encapsulation) denir.

Şimdi de bir oyuncak kutusu düşünün. İçinde kırmızı bir araba, mavi bir tren ve sarı bir top var. Hepsi farklıdır ama hepsi yuvarlanabilir. Kutuya yeni bir oyuncak koymak için kutuyu değiştirmeniz gerekmez; yeni oyuncağın da yuvarlanabilmesi yeter. Üstünde "ses çıkarabilen her şey" yazan bir kutuda düdük, davul ve zil bulunabilir: Elinize gelenin ne olduğuna bakmadan "ses çıkar" dersiniz, her oyuncak kendi sesini çıkarır. Bu oyuncak bir *trait nesnesidir* (trait object). Hangi sesin çıkacağı, oyuncağın yanında duran küçük bir tablodan bulunur.

Son olarak bir mektubu düşünün. Mektup üç hâlden geçer: yazılan kâğıt, kapalı zarf, postalanmış mektup. Kâğıdı zarfa koyup kapattığınızda elinizde artık kâğıt yoktur, zarf vardır. Zarfa yeni bir cümle ekleyemezsiniz, çünkü zarfın böyle bir özelliği yoktur. Her hâlin ayrı bir tür olduğu bu düzene *type-state* denir.

<details>
<summary>Bu bölümde geçen terimler</summary>

- **nesne (object):** Veri ile o veriyi kullanan fonksiyonların bir arada durduğu yapı.
- **kapsülleme (encapsulation):** İçerideki ayrıntıyı saklayıp dışarıya yalnızca izin verilen işlemleri açmak.
- **private ve `pub`:** Dışarıya kapalı ve dışarıya açık. Rust'ta `private` diye bir anahtar sözcük yoktur; `pub` yazılmayan öğe kapalıdır.
- **modül (`mod`):** Kodun bir odası. Gizlilik sınırı bu odanın kapısıdır.
- **değişmez (invariant):** Her zaman doğru kalması gereken kural. Örneğin "saklanan ortalama listeyle tutarlıdır".
- **API:** Bir türün dışarıya açtığı fonksiyonların tamamı.
- **önbellek (cache):** Yeniden hesaplamamak için saklanan hazır sonuç.
- **kalıtım (inheritance):** Bir türün başka bir türün verisini ve davranışını devralması. Rust'ta yoktur.
- **bileşim (composition):** Bir türü başka bir türün içine alan olarak koymak.
- **trait ve varsayılan metot:** "Şunu yapabilirim" sözü ve o sözün hazır gelen hâli.
- **trait nesnesi (`dyn Trait`):** "Türü ne olursa olsun, şu işi yapabilen bir şey".
- **polimorfizm (polymorphism):** Aynı çağrının, karşısındaki türe göre farklı iş yapması.
- **gönderim (dispatch), statik ve dinamik:** Çağrılacak fonksiyonun derlenirken ya da program çalışırken belirlenmesi.
- **vtable:** "Bu türde şu metot şu adrestedir" bilgisini tutan küçük tablo.
- **geniş işaretçi (wide/fat pointer):** Biri veriyi, öteki vtable'ı gösteren iki parçalı işaretçi.
- **heterojen ve homojen:** İçinde farklı türler bulunan ve bütün öğeleri aynı türden olan (liste).
- **jenerik (generic) ve monomorfizasyon (monomorphization):** Türü sonradan dolan şablon ve her somut tür için ayrı bir kopya üretilmesi.
- **durum kalıbı (state pattern):** Bir nesnenin, içinde bulunduğu duruma göre farklı davranmasını sağlayan tasarım kalıbı.
- **type-state:** Her durumu ayrı bir tür yaparak yanlış adımı derlenmez kılmak.
- **`self`'i tüketmek:** Metodun değeri sahiplenip bitirmesi. Çağrıdan sonra eski değer kullanılamaz.
- **açık/kapalı ilkesi (open/closed):** Var olan koda dokunmadan yenisinin eklenebilmesini isteyen tasarım ilkesi.
- **dyn uyumlu (dyn compatible):** Bir trait'in `dyn` ile kullanılabilmesi için gereken şartları sağlaması.

</details>

## Nesne: veri ve davranış bir arada

Tasarım kalıpları üzerine yazılmış klasik bir kitap nesneyi, veriyi ve o veri üzerinde çalışan yordamları bir arada paketleyen şey olarak tanımlar. Bu tanıma göre Rust nesne yönelimlidir: Struct ve enum'lar veriyi tutar, `impl` blokları da metotları sağlar. Rust'ta bunlara nesne denmese de gördükleri iş aynıdır.[^nesne]

Aşağıdaki tür bir sayı listesini ve o listenin ortalamasını tutar. Ortalama her sorulduğunda yeniden hesaplanmaz; bir kez hesaplanıp *önbellekte* (cache) saklanır:

```rust
mod istatistik {
    pub struct OrtalamaListe {
        pub liste: Vec<i32>,
        pub ortalama: f64,
    }

    impl OrtalamaListe {
        pub fn ortalama_guncelle(&mut self) {
            let toplam: i32 = self.liste.iter().sum();
            self.ortalama = toplam as f64 / self.liste.len() as f64;
        }
    }
}

use istatistik::OrtalamaListe;

fn main() {
    let mut o = OrtalamaListe { liste: vec![10, 20, 30], ortalama: 20.0 };

    o.liste.push(999); // ortalama_guncelle çağrılmadı
    println!("liste = {:?}, ortalama = {}", o.liste, o.ortalama);
}
```

Çıktı:

```text
liste = [10, 20, 30, 999], ortalama = 20
```

Veri de davranış da bir aradadır, ama sonuç yanlıştır. Bu türün bir kuralı, yani *değişmezi* (invariant) vardır: `ortalama`, `liste`'deki sayıların ortalaması olmalıdır. İki alan da dışarıya açık olduğu için birinin listeyi değiştirip ortalamayı güncellemeyi unutmasını hiçbir şey engellemez. Kumbaranın içi açık kalmıştır.

## Kapsülleme: değişmezi korumak

Çözüm, alanları gizleyip değişikliği tek bir yola indirmektir. `pub` yazılmayan alan ve metotlar, tanımlandıkları modülün dışından kullanılamaz:[^kapsulleme]

```rust
mod istatistik {
    pub struct OrtalamaListe {
        liste: Vec<i32>, // dışarıya kapalı
        ortalama: f64,   // dışarıya kapalı, önbellek
    }

    impl OrtalamaListe {
        pub fn yeni() -> OrtalamaListe {
            OrtalamaListe { liste: vec![], ortalama: 0.0 }
        }

        pub fn ekle(&mut self, deger: i32) {
            self.liste.push(deger);
            self.ortalama_guncelle(); // her değişimde ortalama da güncellenir
        }

        pub fn ortalama(&self) -> f64 {
            self.ortalama // dışarıya yalnızca değeri veririz
        }

        fn ortalama_guncelle(&mut self) {
            let toplam: i32 = self.liste.iter().sum();
            self.ortalama = toplam as f64 / self.liste.len() as f64;
        }
    }
}

use istatistik::OrtalamaListe;

fn main() {
    let mut o = OrtalamaListe::yeni();
    o.ekle(10);
    o.ekle(20);
    o.ekle(30);
    println!("ortalama = {}", o.ortalama());
}
```

Çıktı:

```text
ortalama = 20
```

Listeyi değiştirmenin tek yolu artık `ekle`'dir ve `ekle` ortalamayı da günceller. `ortalama` metodu alanın kendisini değil, içindeki değerin bir kopyasını verir. `ortalama_guncelle` ise `pub` olmadığı için yalnızca bu modülün içinden çağrılabilir.

Kapalı alana ve kapalı metoda dışarıdan erişmeye çalışalım:

```rust,compile_fail,E0616
# mod istatistik {
#     pub struct OrtalamaListe {
#         liste: Vec<i32>,
#         ortalama: f64,
#     }
#
#     impl OrtalamaListe {
#         pub fn yeni() -> OrtalamaListe {
#             OrtalamaListe { liste: vec![], ortalama: 0.0 }
#         }
#
#         pub fn ekle(&mut self, deger: i32) {
#             self.liste.push(deger);
#             self.ortalama_guncelle();
#         }
#
#         pub fn ortalama(&self) -> f64 {
#             self.ortalama
#         }
#
#         fn ortalama_guncelle(&mut self) {
#             let toplam: i32 = self.liste.iter().sum();
#             self.ortalama = toplam as f64 / self.liste.len() as f64;
#         }
#     }
# }
#
# use istatistik::OrtalamaListe;
#
fn main() {
    let mut o = OrtalamaListe::yeni();
    o.ekle(10);

    o.liste.push(999);
    o.ortalama_guncelle();
}
```

Derleyicinin yanıtı:

```text
error[E0616]: field `liste` of struct `OrtalamaListe` is private
   |     o.liste.push(999);
   |       ^^^^^ private field

error[E0624]: method `ortalama_guncelle` is private
   |         fn ortalama_guncelle(&mut self) {
   |         ------------------------------- private method defined here
...
   |     o.ortalama_guncelle();
   |       ^^^^^^^^^^^^^^^^^ private method
```

Kapalı alanı olan bir struct dışarıdan `OrtalamaListe { liste: vec![], ortalama: 5.0 }` diye de kurulamaz; derleyici bu denemeyi `E0451` ile reddeder.[^e0451] Böylece tutarsız bir değer baştan var olamaz: Modülün dışındaki kod için tek kapı `yeni` fonksiyonudur.

### İç temsil değişebilir

Kapsüllemenin ikinci yararı, içeriyi sonradan değiştirebilmektir. Dış dünya yalnızca `yeni`, `ekle` ve `ortalama` metotlarını bilir. Ortalama için bütün listeyi saklamanın gerekmediğini fark ederseniz alanları değiştirebilirsiniz; `main` aynı kalır:

```rust
mod istatistik {
    pub struct OrtalamaListe {
        toplam: i64,
        adet: u32,
    }

    impl OrtalamaListe {
        pub fn yeni() -> OrtalamaListe {
            OrtalamaListe { toplam: 0, adet: 0 }
        }

        pub fn ekle(&mut self, deger: i32) {
            self.toplam += i64::from(deger);
            self.adet += 1;
        }

        pub fn ortalama(&self) -> f64 {
            if self.adet == 0 { 0.0 } else { self.toplam as f64 / f64::from(self.adet) }
        }
    }
}

use istatistik::OrtalamaListe;

fn main() {
    let mut o = OrtalamaListe::yeni();
    o.ekle(10);
    o.ekle(20);
    o.ekle(30);
    println!("ortalama = {}", o.ortalama());
}
```

Çıktı:

```text
ortalama = 20
```

Alanlar `pub` olsaydı `o.liste` yazan her satır bu değişiklikten sonra derlenmezdi.[^kapsulleme]

### Gizlilik türe değil, modüle göredir

Birçok dilde gizli alan yalnızca kendi sınıfından görünür. Rust'ta sınır modüldür: `pub` olmayan bir öğeye, tanımlandığı modülün içindeki ve o modülün alt modüllerindeki kod erişebilir.[^gizlilik] Bu yüzden türü ve onu kullanan kodu aynı modüle yazarsanız kapsülleme işe yaramaz:

```rust
struct OrtalamaListe {
    liste: Vec<i32>,
    ortalama: f64,
}

impl OrtalamaListe {
    fn ortalama(&self) -> f64 {
        self.ortalama
    }
}

fn main() {
    let mut o = OrtalamaListe { liste: vec![10, 20, 30], ortalama: 20.0 };

    o.liste.push(999); // aynı modüldeyiz, derlenir
    println!("{} değer, ortalama = {}", o.liste.len(), o.ortalama());
}
```

Çıktı:

```text
4 değer, ortalama = 20
```

Bir değişmezi korumak istiyorsanız türü kendi modülüne koyun; `E0616` ancak o zaman devreye girer. Modüller [6. bölümde](../basics/06-modules.md), gizli alanla korunan bir türün ayrıntılı örneği [4.1. bölümde](../basics/04-structs-deep-dive.md) anlatılır.

Öğeler varsayılan olarak kapalıdır, ancak iki istisna vardır: Dışarıya açık bir trait'in öğeleri ve dışarıya açık bir enum'un varyantları, `pub` yazılmasa da açıktır.[^gizlilik]

## Kalıtım neden yok?

Kalıtımda bir tür, başka bir türün verisini ve davranışını yeniden yazmadan devralır. Nesne yönelimli olmak için bu özellik şart sayılıyorsa Rust nesne yönelimli değildir: Bir struct'ın, başka bir struct'ın alanlarını ve metotlarını devralmasının, makro kullanmak dışında bir yolu yoktur.[^kalitim]

Kalıtım aslında iki ayrı iş için kullanılır ve Rust bu işleri birbirinden ayırır:[^kalitim]

1. **Kod paylaşmak:** Bir kez yazılan davranışı başka türlerde de kullanmaktır. Rust'ta bunun karşılığı trait'lerin varsayılan metotları ve *bileşimdir* (composition): Bir türü başka bir türün içine alan olarak koyarsınız.
2. **Farklı türlere aynı gözle bakmak:** Yani *polimorfizm*. Rust'ta bunun karşılığı, trait sınırı olan jenerikler (statik) ya da trait nesneleridir (dinamik).

Aşağıdaki örnek birinci işi yapar. `cerceveli` metodu bir kez, trait'in içinde yazılmıştır; `SimgeliButon` ise `Buton`'dan türemez, bir `Buton`'u parça olarak taşır:

```rust
trait Ciz {
    fn ciz(&self) -> String;

    // varsayılan metot: uygulayan her tür bunu hazır bulur
    fn cerceveli(&self) -> String {
        format!("| {} |", self.ciz())
    }
}

struct Buton {
    etiket: String,
}

impl Ciz for Buton {
    fn ciz(&self) -> String {
        format!("[ {} ]", self.etiket)
    }
}

struct SimgeliButon {
    simge: char,
    ic: Buton, // bileşim: Buton bir alandır
}

impl Ciz for SimgeliButon {
    fn ciz(&self) -> String {
        format!("{} {}", self.simge, self.ic.ciz())
    }
}

fn main() {
    let b = Buton { etiket: String::from("Kaydet") };
    let s = SimgeliButon { simge: '*', ic: Buton { etiket: String::from("Sil") } };

    println!("{}", b.cerceveli());
    println!("{}", s.cerceveli());
}
```

Çıktı:

```text
| [ Kaydet ] |
| * [ Sil ] |
```

`SimgeliButon` yalnızca gerçekten gereken davranışı seçip kullanır. Kalıtımda ise alt sınıf, gerekmeyen davranışı da devralır. Bunun bilinen sakıncaları vardır:[^kalitim]

- **İstenmeyen davranış:** Üst sınıfın bir metodu alt sınıfta anlamsız olabilir, yine de çağrılabilir.
- **"Bir tür ...dır" zorlaması (is-a):** İlişki gerçekte "bir ... içerir" olsa bile "bir ...dır" diye modellenir. Üstelik kimi diller yalnızca tek bir üst sınıfa izin verir.
- **Kırılgan taban sınıf (fragile base class):** Üst sınıftaki zararsız görünen bir değişiklik, ondan türeyen sınıfları bozabilir.[^kirilgan]

## Trait nesneleriyle polimorfizm

### Sorun: bir liste tek tür tutar

Ekrana çizilecek bileşenleri bir listede toplamak istiyoruz. Bileşenlerin bir bölümü buton, bir bölümü onay kutusudur:

```rust,compile_fail,E0308
struct Buton {
    etiket: String,
}

struct OnayKutusu {
    isaretli: bool,
}

fn main() {
    let bilesenler: Vec<Buton> = vec![
        Buton { etiket: String::from("Kaydet") },
        OnayKutusu { isaretli: true },
    ];
}
```

Derleyicinin yanıtı:

```text
error[E0308]: mismatched types
   |         OnayKutusu { isaretli: true },
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `Buton`, found `OnayKutusu`
```

Bir `Vec<T>` *homojendir*: Bütün öğeleri aynı `T` türündendir.[^vec] Farklı türleri tek listede tutmak için onlara ortak bir gözle bakmamız gerekir.

### Çözüm: `Box<dyn Ciz>`

Ortak davranışı bir trait ile tarif ederiz. `Box<dyn Ciz>` türü de "türü ne olursa olsun, `Ciz` uygulayan bir şey" anlamına gelir:

```rust
trait Ciz {
    fn ciz(&self) -> String;
}

struct Buton {
    etiket: String,
}

struct OnayKutusu {
    isaretli: bool,
}

impl Ciz for Buton {
    fn ciz(&self) -> String {
        format!("[ {} ]", self.etiket)
    }
}

impl Ciz for OnayKutusu {
    fn ciz(&self) -> String {
        format!("({}) seçim", if self.isaretli { "x" } else { " " })
    }
}

struct Ekran {
    bilesenler: Vec<Box<dyn Ciz>>,
}

impl Ekran {
    fn calistir(&self) {
        for b in &self.bilesenler {
            println!("  {}", b.ciz()); // dinamik gönderim
        }
    }
}

fn main() {
    let ekran = Ekran {
        bilesenler: vec![
            Box::new(Buton { etiket: String::from("Kaydet") }),
            Box::new(OnayKutusu { isaretli: true }),
            Box::new(Buton { etiket: String::from("İptal") }),
        ],
    };
    println!("ekran:");
    ekran.calistir();
}
```

Çıktı:

```text
ekran:
  [ Kaydet ]
  (x) seçim
  [ İptal ]
```

`b.ciz()` ifadesi, o turdaki öğenin türüne göre farklı bir fonksiyon çalıştırır. Derleyici hangisinin çalışacağını bilemez; fonksiyon, program çalışırken öğenin yanında taşınan tablodan, yani *vtable*'dan bulunur. Buna *dinamik gönderim* (dynamic dispatch) denir.[^dinamik]

### Var olan koda dokunmadan genişletmek

`Ekran` somut türleri tanımaz; tek bildiği, elindeki her şeyin `ciz` metodu olduğudur. Bu yüzden yeni bir bileşen eklemek için `Ekran`'a dokunmak gerekmez. *Açık/kapalı ilkesinin* (open/closed) istediği bu genişletilebilirlik, kalıtım olmadan elde edilmiş olur:

```rust
# trait Ciz {
#     fn ciz(&self) -> String;
# }
#
# struct Buton {
#     etiket: String,
# }
#
# struct OnayKutusu {
#     isaretli: bool,
# }
#
# impl Ciz for Buton {
#     fn ciz(&self) -> String {
#         format!("[ {} ]", self.etiket)
#     }
# }
#
# impl Ciz for OnayKutusu {
#     fn ciz(&self) -> String {
#         format!("({}) seçim", if self.isaretli { "x" } else { " " })
#     }
# }
#
# struct Ekran {
#     bilesenler: Vec<Box<dyn Ciz>>,
# }
#
# impl Ekran {
#     fn calistir(&self) {
#         for b in &self.bilesenler {
#             println!("  {}", b.ciz());
#         }
#     }
# }
#
struct Kaydirici {
    yuzde: u8,
}

impl Ciz for Kaydirici {
    fn ciz(&self) -> String {
        format!("<--%{}-->", self.yuzde)
    }
}

fn main() {
    let ekran = Ekran {
        bilesenler: vec![
            Box::new(OnayKutusu { isaretli: false }),
            Box::new(Kaydirici { yuzde: 40 }),
        ],
    };
    ekran.calistir();
}
```

Çıktı:

```text
  ( ) seçim
  <--%40-->
```

Listeye giren türün `Ciz` uygulaması şarttır ve bu şart derleme zamanında denetlenir. Trait'i uygulamayan bir değer koymaya çalışırsanız program derlenmez:[^trait-uygula]

```rust,compile_fail,E0277
trait Ciz {
    fn ciz(&self) -> String;
}

fn main() {
    let bilesenler: Vec<Box<dyn Ciz>> = vec![Box::new(String::from("Merhaba"))];
}
```

Derleyicinin yanıtı:

```text
error[E0277]: the trait bound `String: Ciz` is not satisfied
   |     let bilesenler: Vec<Box<dyn Ciz>> = vec![Box::new(String::from("Merhaba"))];
   |                                              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `Ciz` is not implemented for `String`
   = note: required for the cast from `Box<String>` to `Box<dyn Ciz>`
```

Dinamik türlü dillerde yaygın olan "ördek tiplemesinde" (duck typing) bu hata ancak program çalışırken, metot bulunamadığında ortaya çıkar.[^trait-uygula]

### Statik seçenek: jenerik ekran

Aynı işi jenerik bir türle de yapabilirsiniz:

```rust
# trait Ciz {
#     fn ciz(&self) -> String;
# }
#
# struct Buton {
#     etiket: String,
# }
#
# struct OnayKutusu {
#     isaretli: bool,
# }
#
# impl Ciz for Buton {
#     fn ciz(&self) -> String {
#         format!("[ {} ]", self.etiket)
#     }
# }
#
# impl Ciz for OnayKutusu {
#     fn ciz(&self) -> String {
#         format!("({}) seçim", if self.isaretli { "x" } else { " " })
#     }
# }
#
struct JenerikEkran<T: Ciz> {
    bilesenler: Vec<T>,
}

impl<T: Ciz> JenerikEkran<T> {
    fn calistir(&self) {
        for b in &self.bilesenler {
            println!("  {}", b.ciz()); // statik gönderim
        }
    }
}

fn main() {
    let ekran = JenerikEkran {
        bilesenler: vec![
            Buton { etiket: String::from("Evet") },
            Buton { etiket: String::from("Hayır") },
        ],
    };
    ekran.calistir();
}
```

Çıktı:

```text
  [ Evet ]
  [ Hayır ]
```

`T` tek bir türün yerini tutar: Bir `JenerikEkran` ya yalnızca butonlardan ya da yalnızca onay kutularından oluşur. Buna karşılık çağrı derleme zamanında bağlanır; derleyici `T`'nin her somut türü için ayrı bir kopya üretir (monomorfizasyon) ve çağrıyı çoğunlukla satır içine alabilir.[^jenerik-ekran][^mono]

| | `Vec<T>` (jenerik) | `Vec<Box<dyn Ciz>>` (trait nesnesi) |
| --- | --- | --- |
| Liste | homojen, tek tür | heterojen, karışık türler |
| Gönderim | statik, derleme zamanında | dinamik, vtable üzerinden |
| Bedeli | her tür için ayrı kod üretilir | çağrı başına dolaylama, `Box` için heap tahsisi |
| Ne zaman? | türler aynı ve çağrı sık ise | türler karışıksa ya da sonradan eklenecekse |

### Trait nesnesinin bellekteki hâli

`Box<Buton>` ile `Box<dyn Ciz>` aynı boyda değildir. Trait nesnesini gösteren işaretçi *geniş işaretçidir* (wide/fat pointer): Biri veriye, öteki o türün vtable'ına giden iki işaretçi taşır.[^genis]

```rust
# trait Ciz {
#     fn ciz(&self) -> String;
# }
#
# struct Buton {
#     etiket: String,
# }
#
# struct OnayKutusu {
#     isaretli: bool,
# }
#
# impl Ciz for Buton {
#     fn ciz(&self) -> String {
#         format!("[ {} ]", self.etiket)
#     }
# }
#
# impl Ciz for OnayKutusu {
#     fn ciz(&self) -> String {
#         format!("({}) seçim", if self.isaretli { "x" } else { " " })
#     }
# }
#
fn main() {
    let sozcuk = size_of::<usize>(); // tek bir işaretçinin boyu

    println!("&Buton       : {} işaretçi", size_of::<&Buton>() / sozcuk);
    println!("&dyn Ciz     : {} işaretçi", size_of::<&dyn Ciz>() / sozcuk);
    println!("Box<Buton>   : {} işaretçi", size_of::<Box<Buton>>() / sozcuk);
    println!("Box<dyn Ciz> : {} işaretçi", size_of::<Box<dyn Ciz>>() / sozcuk);
}
```

Çıktı:

```text
&Buton       : 1 işaretçi
&dyn Ciz     : 2 işaretçi
Box<Buton>   : 1 işaretçi
Box<dyn Ciz> : 2 işaretçi
```

`dyn Ciz`'in kendisinin ise derleme zamanında bilinen bir boyutu yoktur; arkasında bir `Buton` da olabilir, bir `OnayKutusu` da. Bu yüzden `dyn Ciz` tek başına bir listeye konamaz, bir işaretçinin arkasında durmak zorundadır:

```rust,compile_fail,E0277
# trait Ciz { fn ciz(&self) -> String; }
#
fn main() {
    let bilesenler: Vec<dyn Ciz> = Vec::new();
}
```

Derleyicinin yanıtı:

```text
error[E0277]: the size for values of type `dyn Ciz` cannot be known at compilation time
   |     let bilesenler: Vec<dyn Ciz> = Vec::new();
   |                     ^^^^^^^^^^^^ doesn't have a size known at compile-time
   |
   = help: the trait `Sized` is not implemented for `dyn Ciz`
```

Boyutu bilinmeyen türler [22. bölümde](../advanced/22-sized-dst.md) ele alınır.

### Her trait `dyn` olamaz

Bir vtable kurulabilmesi için trait'in *dyn uyumlu* (dyn compatible) olması gerekir. Örneğin `Self` döndüren bir metot bunu engeller: `dyn Ciz`'in arkasındaki tür bilinmediğine göre dönecek değerin boyutu da bilinemez.[^dyn]

```rust,compile_fail,E0038
trait Ciz {
    fn ciz(&self) -> String;
    fn kopya(&self) -> Self;
}

fn goster(b: &dyn Ciz) {
    println!("{}", b.ciz());
}
```

Derleyicinin yanıtı:

```text
error[E0038]: the trait `Ciz` is not dyn compatible
   | fn goster(b: &dyn Ciz) {
   |               ^^^^^^^ `Ciz` is not dyn compatible
   |
note: for a trait to be dyn compatible it needs to allow building a vtable
      for more information, visit <https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility>
   | trait Ciz {
   |       --- this trait is not dyn compatible...
   |     fn ciz(&self) -> String;
   |     fn kopya(&self) -> Self;
   |                        ^^^^ ...because method `kopya` references the `Self` type in its return type
   = help: consider moving `kopya` to another trait
```

Uyumluluğu bozan öteki durumlar ve çözümleri, trait nesnesinin taşıdığı gizli ömür sınırı ve `Send` gibi ek sınırlar [17.1. bölümde](17-oop-deep-dive.md) anlatılır.

## Durumu türe taşımak: type-state

Bir blog yazısının üç durumu olsun: taslak, inceleme, yayın. Kural şudur: Yalnızca taslağa metin eklenebilir ve yalnızca yayımlanmış yazının içeriği okunabilir. Durumu bir alanda tutan klasik yaklaşımda her metot "doğru durumda mıyım?" diye **çalışma zamanında** bakar. Nesne yönelimli dillerdeki *durum kalıbı* (state pattern) bu denetimi durum nesnelerine dağıtır ama yine çalışma zamanına bırakır.[^klasik] En yalın hâli şöyledir:

```rust
#[derive(PartialEq)]
enum Durum {
    Taslak,
    Inceleme,
    Yayin,
}

struct Yazi {
    durum: Durum,
    icerik: String,
}

impl Yazi {
    fn ekle(&mut self, s: &str) {
        if self.durum == Durum::Taslak {
            self.icerik.push_str(s);
        }
    }

    fn incelemeye_gonder(&mut self) {
        if self.durum == Durum::Taslak {
            self.durum = Durum::Inceleme;
        }
    }

    fn onayla(&mut self) {
        if self.durum == Durum::Inceleme {
            self.durum = Durum::Yayin;
        }
    }

    fn icerik(&self) -> &str {
        if self.durum == Durum::Yayin { &self.icerik } else { "" }
    }
}

fn main() {
    let mut yazi = Yazi { durum: Durum::Taslak, icerik: String::new() };
    yazi.ekle("Bugün Rust öğrendim.");
    yazi.onayla(); // incelemeye gönderilmemişti: sessizce yok sayıldı
    println!("içerik: {:?}", yazi.icerik());

    yazi.incelemeye_gonder();
    yazi.ekle(" Bir cümle daha."); // artık taslak değil: bu da yok sayıldı
    yazi.onayla();
    println!("içerik: {:?}", yazi.icerik());
}
```

Çıktı:

```text
içerik: ""
içerik: "Bugün Rust öğrendim."
```

Program iki kez yanlış kullanıldı, ikisinde de derlendi ve çalıştı. Hatayı sessizce yutmak yerine panikleyebilir ya da `Result` döndürebilirdik; ama hata yine ancak program çalışınca ortaya çıkardı.

### Her durum ayrı bir tür

*Type-state* kalıbında durum bir alan değil, türün kendisidir. Geçiş yapan metot eski değeri *tüketir* (`self`'i sahiplenir) ve yeni türde bir değer döndürür:[^typestate]

```rust
mod blog {
    pub struct Taslak {
        icerik: String,
    }
    pub struct Inceleme {
        icerik: String,
    }
    pub struct Yayin {
        icerik: String,
    }

    impl Taslak {
        pub fn yeni() -> Taslak {
            Taslak { icerik: String::new() }
        }

        pub fn ekle(&mut self, s: &str) {
            self.icerik.push_str(s);
        }

        // self'i tüketir: çağrıdan sonra elde Taslak kalmaz
        pub fn incelemeye_gonder(self) -> Inceleme {
            Inceleme { icerik: self.icerik }
        }
    }

    impl Inceleme {
        pub fn onayla(self) -> Yayin {
            Yayin { icerik: self.icerik }
        }

        pub fn reddet(self) -> Taslak {
            Taslak { icerik: self.icerik }
        }
    }

    impl Yayin {
        pub fn icerik(&self) -> &str {
            &self.icerik
        }
    }
}
# use blog::Taslak;
#
# fn main() {
#     let mut yazi = Taslak::yeni();
#     yazi.ekle("merhaba");
#     assert_eq!(yazi.incelemeye_gonder().onayla().icerik(), "merhaba");
# }
```

Yazının yolculuğu şöyle yürür. İnceleyen yazıyı önce beğenmez, taslağa geri gönderir; düzeltilen yazı sonra yayımlanır:

```rust
# mod blog {
#     pub struct Taslak {
#         icerik: String,
#     }
#     pub struct Inceleme {
#         icerik: String,
#     }
#     pub struct Yayin {
#         icerik: String,
#     }
#
#     impl Taslak {
#         pub fn yeni() -> Taslak {
#             Taslak { icerik: String::new() }
#         }
#
#         pub fn ekle(&mut self, s: &str) {
#             self.icerik.push_str(s);
#         }
#
#         pub fn incelemeye_gonder(self) -> Inceleme {
#             Inceleme { icerik: self.icerik }
#         }
#     }
#
#     impl Inceleme {
#         pub fn onayla(self) -> Yayin {
#             Yayin { icerik: self.icerik }
#         }
#
#         pub fn reddet(self) -> Taslak {
#             Taslak { icerik: self.icerik }
#         }
#     }
#
#     impl Yayin {
#         pub fn icerik(&self) -> &str {
#             &self.icerik
#         }
#     }
# }
#
# use blog::Taslak;
#
fn main() {
    let mut yazi = Taslak::yeni();
    yazi.ekle("Bugün Rust öğrendim. ");
    yazi.ekle("Çok iyiydi.");

    let incelemede = yazi.incelemeye_gonder();

    let mut tekrar_taslak = incelemede.reddet();
    tekrar_taslak.ekle(" (düzeltildi)");

    let yayinda = tekrar_taslak.incelemeye_gonder().onayla();
    println!("yayınlanan içerik: {}", yayinda.icerik());
}
```

Çıktı:

```text
yayınlanan içerik: Bugün Rust öğrendim. Çok iyiydi. (düzeltildi)
```

Metotların hiçbirinde `if` yoktur. Durumu sormaya gerek kalmamıştır, çünkü bir `Taslak` yalnızca taslak olabilir.

### Yanlış adım derlenmez

İlk yanlış, yayımlanmamış yazıyı okumaya çalışmaktır. `icerik` metodu yalnızca `Yayin`'de vardır:

```rust,compile_fail,E0599
# mod blog {
#     pub struct Taslak {
#         icerik: String,
#     }
#     pub struct Inceleme {
#         icerik: String,
#     }
#     pub struct Yayin {
#         icerik: String,
#     }
#
#     impl Taslak {
#         pub fn yeni() -> Taslak {
#             Taslak { icerik: String::new() }
#         }
#
#         pub fn ekle(&mut self, s: &str) {
#             self.icerik.push_str(s);
#         }
#
#         pub fn incelemeye_gonder(self) -> Inceleme {
#             Inceleme { icerik: self.icerik }
#         }
#     }
#
#     impl Inceleme {
#         pub fn onayla(self) -> Yayin {
#             Yayin { icerik: self.icerik }
#         }
#
#         pub fn reddet(self) -> Taslak {
#             Taslak { icerik: self.icerik }
#         }
#     }
#
#     impl Yayin {
#         pub fn icerik(&self) -> &str {
#             &self.icerik
#         }
#     }
# }
#
# use blog::Taslak;
#
fn main() {
    let mut yazi = Taslak::yeni();
    yazi.ekle("Bugün Rust öğrendim.");
    println!("{}", yazi.icerik());
}
```

Derleyicinin yanıtı:

```text
error[E0599]: no method named `icerik` found for struct `Taslak` in the current scope
   |     pub struct Taslak {
   |     ----------------- method `icerik` not found for this struct
...
   |     println!("{}", yazi.icerik());
   |                         ^^^^^^ private field, not a method
```

Türler `blog` modülünde olduğu için `yazi.icerik` alanına da dışarıdan ulaşılamaz; kapsülleme burada da iş başındadır. İkinci yanlış, incelemeye gönderilmiş yazıya metin eklemektir:

```rust,compile_fail,E0382
# mod blog {
#     pub struct Taslak {
#         icerik: String,
#     }
#     pub struct Inceleme {
#         icerik: String,
#     }
#     pub struct Yayin {
#         icerik: String,
#     }
#
#     impl Taslak {
#         pub fn yeni() -> Taslak {
#             Taslak { icerik: String::new() }
#         }
#
#         pub fn ekle(&mut self, s: &str) {
#             self.icerik.push_str(s);
#         }
#
#         pub fn incelemeye_gonder(self) -> Inceleme {
#             Inceleme { icerik: self.icerik }
#         }
#     }
#
#     impl Inceleme {
#         pub fn onayla(self) -> Yayin {
#             Yayin { icerik: self.icerik }
#         }
#
#         pub fn reddet(self) -> Taslak {
#             Taslak { icerik: self.icerik }
#         }
#     }
#
#     impl Yayin {
#         pub fn icerik(&self) -> &str {
#             &self.icerik
#         }
#     }
# }
#
# use blog::Taslak;
#
fn main() {
    let mut yazi = Taslak::yeni();
    yazi.ekle("Bugün Rust öğrendim.");

    let incelemede = yazi.incelemeye_gonder();
    yazi.ekle(" Bir cümle daha.");
}
```

Derleyicinin yanıtı:

```text
error[E0382]: borrow of moved value: `yazi`
   |     let mut yazi = Taslak::yeni();
   |         -------- move occurs because `yazi` has type `Taslak`, which does not implement the `Copy` trait
...
   |     let incelemede = yazi.incelemeye_gonder();
   |                           ------------------- `yazi` moved due to this method call
   |     yazi.ekle(" Bir cümle daha.");
   |     ^^^^ value borrowed here after move
```

İki hatayı farklı denetimler yakalar. Birincisi tür denetimidir: `Taslak`'ın `icerik` diye bir metodu yoktur. İkincisi [3. bölümdeki](../basics/03-ownership.md) sahiplik kuralıdır: `incelemeye_gonder` `self`'i aldığı için `yazi` artık geçersizdir. Geçersiz durumlar ve geçişler temsil edilemez hâle gelmiştir. Bu yaklaşım çoğu kez "make invalid states unrepresentable" (geçersiz durumları temsil edilemez kıl) sözüyle özetlenir.

### Bedeli ve sınırı

Type-state'in de bir bedeli vardır. Geçiş her seferinde yeni bir değer döndürdüğü için sonucu yeniden bağlamanız gerekir (`let incelemede = ...`); geçişler artık tek bir değerin içinde gizli değildir, çağıran kod onları görür.[^typestate] Ayrıca durumun derleme zamanında bilinmesi şarttır. Durum ancak çalışma zamanında belli oluyorsa, örneğin yazılar veritabanından okunup aynı listede karışık durumlarda tutuluyorsa, farklı türleri doğrudan tek listeye koyamazsınız; o zaman [5. bölümdeki](../basics/05-enums.md) gibi bir enum ve eksiksiz bir `match` daha uygundur.

## Hata kodu başvurusu

| Kod | Ne zaman görülür? | Ne yapmalı? |
| --- | --- | --- |
| `E0616` | Kapalı bir alana modülün dışından erişilince | Açık bir metot kullanın ya da gerçekten gerekiyorsa alanı `pub` yapın |
| `E0624` | Kapalı bir metot modülün dışından çağrılınca | Türün açık API'sini kullanın |
| `E0451` | Kapalı alanı olan struct dışarıdan `Tur { .. }` ile kurulunca | `yeni` gibi bir kurucu fonksiyon çağırın |
| `E0308` | `Vec<Buton>`'a başka türden bir değer konunca | `Vec<Box<dyn Trait>>` ya da bir enum kullanın |
| `E0277` | Trait'i uygulamayan tür trait nesnesine çevrilince; `dyn Trait` işaretçisiz kullanılınca | Trait'i uygulayın; `&`, `Box`, `Rc` ya da `Arc` arkasına koyun |
| `E0038` | Dyn uyumlu olmayan trait `dyn` ile kullanılınca | Metodu ayırın ya da `where Self: Sized` ekleyin ([17.1](17-oop-deep-dive.md)) |
| `E0599` | O türde bulunmayan bir metot çağrılınca (type-state'te yanlış durum) | Önce gereken geçişi yapın |
| `E0382` | `self`'i tüketen metottan sonra eski değer kullanılınca | Dönen yeni değeri kullanın |

## Perde arkası: Derleyici ne yapıyor?

> [!NOTE]
> Bu bölüm derleyicinin iç işleyişini anlatır. İlk okuyuşta atlayabilirsiniz.

**Gizlilik modül ağacında denetlenir.** Derleyici her öğenin görünürlüğünü bir modülle birlikte saklar; erişim sorusu, erişen kodun o modülün içinde (ya da altında) olup olmadığıdır.[^erisim] Tür denetimi sırasında `o.liste` gibi bir alan erişiminde alan bulunur ama erişilebilir değilse `E0616` üretilir.[^e0616] Bu denetim yalnızca derleme zamanındadır: `pub` olan ve olmayan alanlar bellekte farklı biçimde durmaz, çalışan programda bir erişim denetimi yapılmaz.

**Trait nesnesini bir çift işaretçi gösterir.** `Box::new(Buton { .. })` ifadesinin `Box<dyn Ciz>`'e dönüşmesi bir *örtük dönüşümdür* (unsized coercion): Veri işaretçisi aynı kalır, yanına `Buton` için üretilmiş vtable'ın adresi eklenir.[^ortuk] vtable'ın başında yıkıcı, boyut ve hizalama bulunur; trait'in metotları bunların ardından gelir; bu yerleşim derleyicinin iç ayrıntısıdır, dilin sözü değildir.[^vtable] `b.ciz()` çağrısı, metodun tablodaki sıra numarasıyla adresi okuyup o adresi çağıran koda çevrilir.[^cagri] Statik gönderimde ise bu adımların hiçbiri yoktur: Derleyici, kullanılan her somut tür için jenerik fonksiyonun bir kopyasını üretir ve çağrıyı doğrudan o kopyaya bağlar.[^mono-kod]

**Dinamik gönderimin bedeli iki parçadır:** tablodan adres okumak ve çağrının çoğu durumda satır içine alınamaması (inline edilememesi).[^dinamik] Derleyici iyileştirme sırasında arkadaki türü kanıtlayabilirse bu bedeli kaldırabilir, ancak bunun bir güvencesi yoktur.

**Type-state bir derleyici özelliği değildir.** Tür sisteminin bir tasarım aracı olarak kullanılmasıdır. Durum makinesinin her durumu bir tür, her kenarı `self`'i alan bir metottur; derleyici olmayan kenarı "böyle bir metot yok" diye, geride bırakılmış durumu da "değer taşındı" diye reddeder. Çalışma zamanında denetim ya da panik gerekmez; hata derleme zamanına çekilmiştir.

## Özet

Rust'ta nesnenin karşılığı, struct ya da enum ile `impl` bloklarının birlikteliğidir. Kapsüllemenin sınırı tür değil modüldür; değişmezi korumak için tür kendi modülüne konur ve alanları kapalı tutulur. Kalıtımın iki işi birbirinden ayrılmıştır: Kod paylaşmak için varsayılan metotlar ve bileşim, farklı türlere aynı gözle bakmak için jenerikler ya da trait nesneleri kullanılır. Trait nesnesi karışık türde listeler ve sonradan genişleyen tasarımlar içindir, bedeli dinamik gönderimdir. Type-state ise durumu türe taşıyarak yanlış adımı derlenmez kılar.

[17.1. bölümde](17-oop-deep-dive.md) genişletilebilir bir bildirim sistemi kuracak; trait nesnesini enum ve jenerikle karşılaştıracak; gizli ömür sınırlarını, yukarı dönüşümü ve `Box<dyn Trait>` klonlamayı ele alacağız.

## Kaynaklar

[^oop]: The Rust Programming Language, [18. Object-Oriented Programming Features](https://doc.rust-lang.org/book/ch18-00-oop.html) ve [18.1 Characteristics of Object-Oriented Languages](https://doc.rust-lang.org/book/ch18-01-what-is-oo.html).
[^nesne]: The Rust Programming Language, [18.1: Objects Contain Data and Behavior](https://doc.rust-lang.org/book/ch18-01-what-is-oo.html#objects-contain-data-and-behavior).
[^kapsulleme]: The Rust Programming Language, [18.1: Encapsulation That Hides Implementation Details](https://doc.rust-lang.org/book/ch18-01-what-is-oo.html#encapsulation-that-hides-implementation-details).
[^e0451]: Rust hata kodları, [E0451](https://doc.rust-lang.org/error_codes/E0451.html).
[^gizlilik]: The Rust Reference, [Visibility and privacy](https://doc.rust-lang.org/reference/visibility-and-privacy.html).
[^kalitim]: The Rust Programming Language, [18.1: Inheritance as a Type System and as Code Sharing](https://doc.rust-lang.org/book/ch18-01-what-is-oo.html#inheritance-as-a-type-system-and-as-code-sharing).
[^kirilgan]: L. Mikhajlov ve E. Sekerinski, [A Study of The Fragile Base Class Problem](https://link.springer.com/chapter/10.1007/BFb0054099), European Conference on Object-Oriented Programming, 1998.
[^vec]: The Rust Programming Language, [8.1: Using an Enum to Store Multiple Types](https://doc.rust-lang.org/book/ch08-01-vectors.html#using-an-enum-to-store-multiple-types).
[^dinamik]: The Rust Programming Language, [18.2: Performing Dynamic Dispatch](https://doc.rust-lang.org/book/ch18-02-trait-objects.html#performing-dynamic-dispatch).
[^trait-uygula]: The Rust Programming Language, [18.2: Implementing the Trait](https://doc.rust-lang.org/book/ch18-02-trait-objects.html#implementing-the-trait).
[^jenerik-ekran]: The Rust Programming Language, [18.2: Defining a Trait for Common Behavior](https://doc.rust-lang.org/book/ch18-02-trait-objects.html#defining-a-trait-for-common-behavior).
[^mono]: The Rust Programming Language, [10.1: Performance of Code Using Generics](https://doc.rust-lang.org/book/ch10-01-syntax.html#performance-of-code-using-generics).
[^genis]: The Rust Reference, [Trait objects](https://doc.rust-lang.org/reference/types/trait-object.html).
[^dyn]: The Rust Reference, [Dyn compatibility](https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility).
[^klasik]: The Rust Programming Language, [18.3: Attempting Traditional Object-Oriented Style](https://doc.rust-lang.org/book/ch18-03-oo-design-patterns.html#attempting-traditional-object-oriented-style).
[^typestate]: The Rust Programming Language, [18.3: Encoding States and Behavior as Types](https://doc.rust-lang.org/book/ch18-03-oo-design-patterns.html#encoding-states-and-behavior-as-types).
[^erisim]: rustc 1.99.0 kaynak kodu, [`ty/mod.rs`, `is_accessible_from`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_middle/src/ty/mod.rs#L564).
[^e0616]: rustc 1.99.0 kaynak kodu, [`expr.rs`, `private_field_err`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_typeck/src/expr.rs#L3325).
[^ortuk]: The Rust Reference, [Unsized coercions](https://doc.rust-lang.org/reference/type-coercions.html#unsized-coercions).
[^vtable]: rustc 1.99.0 kaynak kodu, [`vtable.rs`, `COMMON_VTABLE_ENTRIES`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_middle/src/ty/vtable.rs#L46).
[^cagri]: rustc 1.99.0 kaynak kodu, [`meth.rs`, `get_fn`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_codegen_ssa/src/meth.rs#L46).
[^mono-kod]: rustc 1.99.0 kaynak kodu, [`collector.rs`, modül belgesi (Mono Item Collection)](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_monomorphize/src/collector.rs#L1).
