# 9.1 Generic, Trait ve Lifetime: Derinleşme

[9. Generic, Trait ve Lifetime](09-generics-traits-lifetimes.md) bölümünde üç aracın ne olduğunu gördük. Bu sayfa bir adım sonrasını sorar: Bir fonksiyonun imzasını *siz* yazarken neye dikkat etmelisiniz? İmzaya yazdığınız her etiket ve her sınır, sizi çağırana verilmiş bir sözdür. Gereğinden sıkı yazarsanız kod derlenir, ama çağıranın eli kolu bağlanır.

Sayfa üç konu üzerinde durur:

- **Ömür etiketi:** Gereğinden sıkı bir etiketin çağıranı nasıl kısıtladığı, sonucu veriye mi yoksa metodun alıcısına mı bağlayacağınız ve `'static` sınırının gerçekte ne dediği.
- **Sınırın yeri:** Jenerik bir veri yapısında trait sınırının neden yapıya değil, ona ihtiyaç duyan `impl` bloğuna konduğu.
- **Trait tasarımı:** Bir trait'e ilişkili tür mü, jenerik parametre mi koyacağınız.

Ömürlerin alt tür ilişkisi [21. Alt Tür İlişkisi ve Varyans](../advanced/21-variance.md), `for<'a>` ile GAT ise [27. İleri Lifetime, HRTB ve GAT](../advanced/27-lifetime-hrtb-gat.md) bölümündedir.

> [!NOTE]
> Bu sayfanın kodu `learn/generics-traits-lifetimes/src/deep_dive.rs` dosyasındadır; `Animal` örneği `learn/basic-concepts` dizinindeki koddan uyarlanmıştır. Depo kökünden `cargo run -p generics-traits-lifetimes` ile çalıştırabilir, `cargo test -p generics-traits-lifetimes` ile testlerini koşturabilirsiniz.

<details>
<summary>Bu sayfada geçen yeni terimler</summary>

- **ödünç (borrow):** Sahibini değiştirmeden veriye bakmak ya da onu kullanmak. Bunun sürdüğü kod bölgesine ödünç süresi denir; süre bitince veri yeniden serbesttir.
- **alıcı (receiver):** Metodun üzerinde çağrıldığı değer: `ayr.sonraki()` çağrısındaki `ayr`. İmzada `&self` ya da `&mut self` olarak görünür.
- **`'_`:** "Buradaki ömrü sen doldur" anlamına gelen adsız ömür.
- **sahipli tür (owned type):** Verisini kendisi taşıyan, kimseye borçlu olmayan tür: `String`, `Vec<T>`.
- **sıfır kopya (zero-copy):** Veriyi çoğaltmadan, aslının içine bakan dilimlerle çalışmak.
- **tahsis (allocation):** Heap'ten yer istemek. Boş olmayan bir metinle yapılan her `String::from` çağrısı bir tahsistir.
- **`Iterator` ve `Item`:** Elemanları sırayla veren trait ve verdiği elemanın türü.
- **ilişkili tür (associated type):** Trait'in içinde boş bırakılan, trait'i uygulayan türün doldurduğu tür.
- **`HashMap`:** Anahtarları değerlere eşleyen sözlük. Gezinme sırası belirli değildir.
- **`Eq` ve `Hash`:** `HashMap` anahtarından istenen trait'ler: Eşitliği sınanabilmeli, özeti çıkarılabilmelidir.
- **`Entry`:** `HashMap`'te bir anahtarın yuvası; ya doludur ya boş.
- **önbellek (cache):** Hesaplanan sonucun, yeniden hesaplanmasın diye saklandığı yer.
- **isabet ve ıska (hit, miss):** Aranan sonucun önbellekte bulunması ve bulunmaması.
- **`FnOnce`:** En az bir kez çağrılabilen closure'ların trait'i. Bu sınırı isteyen kod closure'ı en fazla bir kez çağırabilir.
- **turbofish (`::<T>`):** Türü çağrı sırasında elle yazmak: `"5".parse::<u32>()`.
- **deyimsel (idiomatic):** O dilde alışılmış, önerilen yazım biçimi.
- **polimorfizm (polymorphism):** Aynı çağrının farklı türlerde farklı iş yapabilmesi.

</details>

## Ömür bir süre değil, ilişki bildirimidir

Kütüphaneden bir kitap, arkadaşınızdan da bir cetvel ödünç aldığınızı düşünün. Cetveli yalnızca kitaptaki bir satırın altını çizmek için kullandınız. İşaretlediğiniz satır kitaba aittir, cetvelle bir ilgisi kalmamıştır. Ödünç fişine "bu satır hem kitaba hem cetvele bağlı" yazarsanız, cetveli geri verdiğiniz an satırı da kullanamazsınız.

Ömür etiketi bu fiştir. Süreyi değiştirmez; yalnızca "sonuç hangi girdiye bağlı" bilgisini yazar.[^omur-soz] Fişe fazladan yazdığınız her bağ, çağıranı gereksiz yere kısıtlar. Aşağıdaki iki fonksiyonun gövdesi aynıdır, yalnızca imzaları farklıdır:

```rust
// gereğinden sıkı: sonuç iki girdiye de bağlı sayılır
fn on_eki_at_siki<'a>(metin: &'a str, on_ek: &'a str) -> &'a str {
    metin.strip_prefix(on_ek).unwrap_or(metin)
}

// doğru: on_ek yalnızca karşılaştırmada kullanılır
fn on_eki_at<'a>(metin: &'a str, on_ek: &str) -> &'a str {
    metin.strip_prefix(on_ek).unwrap_or(metin)
}

fn main() {
    let yol = String::from("/api/kullanicilar");
    let temiz;
    {
        let on_ek = String::from("/api"); // bu blokta düşürülecek
        temiz = on_eki_at(&yol, &on_ek);
        println!("blok içinde sıkı sürüm: {}", on_eki_at_siki(&yol, &on_ek));
    }
    println!("on_ek düşürüldü, sonuç geçerli: {temiz}");
}
```

Çıktı:

```text
blok içinde sıkı sürüm: /kullanicilar
on_ek düşürüldü, sonuç geçerli: /kullanicilar
```

`on_eki_at` imzasında `on_ek` parametresinin ömrü yazılmamıştır; derleyici ona kendi bağımsız ömrünü verir.[^elision] Sonuç yalnızca `yol`'a bağlı kaldığından `on_ek` düştükten sonra da kullanılabilir. Sıkı sürüm ise yalnızca `on_ek` yaşarken işe yarar:

```rust,compile_fail,E0597
# fn on_eki_at_siki<'a>(metin: &'a str, on_ek: &'a str) -> &'a str {
#     metin.strip_prefix(on_ek).unwrap_or(metin)
# }
fn main() {
    let yol = String::from("/api/kullanicilar");
    let temiz;
    {
        let on_ek = String::from("/api");
        temiz = on_eki_at_siki(&yol, &on_ek);
    }
    println!("{temiz}");
}
```

Derleyicinin yanıtı:

```text
error[E0597]: `on_ek` does not live long enough
   |
 5 |         let on_ek = String::from("/api");
   |             ----- binding `on_ek` declared here
 6 |         temiz = on_eki_at_siki(&yol, &on_ek);
   |                                      ^^^^^^ borrowed value does not live long enough
 7 |     }
   |     - `on_ek` dropped here while still borrowed
 8 |     println!("{temiz}");
   |                ----- borrow later used here
```

Gövdede `on_ek` hiçbir zaman döndürülmez, ama derleyici çağrı yerinde gövdeye değil imzaya bakar.[^imza] Kural şöyle özetlenebilir: Bir parametre sonuca katkı vermiyorsa sonuçla aynı ömrü paylaşmasın.

Tersi de denetlenir. İmza "yalnızca `x`'e bağlı" deyip gövde `y`'yi döndürmeye kalkarsa hata bu kez fonksiyonun içinde çıkar. Bu hatanın kodu yoktur:

```rust,compile_fail
fn f<'a, 'b>(x: &'a str, y: &'b str) -> &'a str {
    if x.len() > 1 { x } else { y }
}
```

Derleyicinin yanıtı:

```text
error: lifetime may not live long enough
  |
1 | fn f<'a, 'b>(x: &'a str, y: &'b str) -> &'a str {
  |      --  -- lifetime `'b` defined here
  |      |
  |      lifetime `'a` defined here
2 |     if x.len() > 1 { x } else { y }
  |                                 ^ function was supposed to return data with lifetime `'a` but it is returning data with lifetime `'b`
  |
  = help: consider adding the following bound: `'b: 'a`
```

Önerilen `'b: 'a` sınırı "`'b`, `'a`'dan uzun ya da ona eşit" diye okunur:[^omur-sinir] `y`'nin ömrü sonuçtan kısa olmasın. Böylece etiket bir tahmin değil, iki yönlü denetlenen bir sözleşmedir. Gövde sözü tutmak, çağıran da sonucu yalnızca söz verilen süre içinde kullanmak zorundadır.

> [!NOTE]
> `metin.strip_prefix(on_ek).unwrap_or(metin)` ifadesinin hazır karşılığı olan `trim_prefix`, Rust 1.100'de kararlı oldu.[^trim-prefix] Örnekler 1.99 ile derlendiği için burada açık yazım kullanıldı.

## Sıfır kopyalı bir ayrıştırıcı

`"anahtar=deger;anahtar=deger"` biçimli bir metni ayrıştıracağız. Her anahtar ve değer için yeni bir `String` tahsis etmek yerine girdinin *içine* bakan dilimler döndürmek istiyoruz; böylece parçalar için hiç tahsis yapılmaz. Bedeli, sonuçların girdiye bağlı olması, yani bir ömür taşımasıdır.

Bir kitaptan alıntı toplamaya benzer. Cümleleri deftere tek tek yeniden yazabilir ya da kitabın sayfalarına yapışkan ayraç koyabilirsiniz. Ayraçlar *kitaba* bağlıdır; kitap iade edilince işe yaramazlar. Ama ayracı yapıştıran *parmağınıza* bağlı değildirler; parmağınızı çekince ayraç yerinde durur. Aşağıda girdi metni kitap, `Ayar` ayraç, `Ayristirici` ise sayfalarda gezen parmaktır:

```rust
#[derive(Debug, PartialEq, Eq)]
pub struct Ayar<'a> {
    pub anahtar: &'a str,
    pub deger: &'a str,
}

pub struct Ayristirici<'a> {
    kalan: &'a str,
}

impl<'a> Ayristirici<'a> {
    pub fn yeni(girdi: &'a str) -> Self {
        Ayristirici { kalan: girdi }
    }

    // dönüşteki 'a: sonuç ayrıştırıcıya değil, girdi metnine bağlı
    pub fn sonraki(&mut self) -> Option<Ayar<'a>> {
        while !self.kalan.is_empty() {
            let (parca, gerisi) = self.kalan.split_once(';').unwrap_or((self.kalan, ""));
            self.kalan = gerisi;
            if let Some((anahtar, deger)) = parca.split_once('=') {
                return Some(Ayar { anahtar: anahtar.trim(), deger: deger.trim() });
            }
            // '=' içermeyen parça atlanır
        }
        None
    }
}

fn main() {
    let girdi = String::from("ad=Kaan;dil=tr");
    let mut ayr = Ayristirici::yeni(&girdi);
    let a = ayr.sonraki();
    let b = ayr.sonraki(); // a hâlâ elimizde
    drop(ayr); // parmağı çektik
    println!("{a:?}");
    println!("{b:?}");
}
```

Çıktı:

```text
Some(Ayar { anahtar: "ad", deger: "Kaan" })
Some(Ayar { anahtar: "dil", deger: "tr" })
```

Burada iki ayrı ömür vardır ve onları karıştırmak sık yapılan bir hatadır. `'a`, girdi metninin ömrüdür ve uzundur. `&mut self`'in ödüncü ise tek bir `sonraki` çağrısı kadar sürer. Dönüş türüne `'a` yazıldığı için sonuçlar girdiye bağlıdır: İki sonuç birlikte tutulabilir, ayrıştırıcı düşürüldükten sonra da kullanılabilir.

Ömrü derleyiciye bıraksaydık ne olurdu? Çıkarımın üçüncü kuralı sonucu alıcıya, yani `&mut self`'e bağlardı:[^elision]

```rust,compile_fail,E0499
# #[derive(Debug)]
# pub struct Ayar<'a> {
#     pub anahtar: &'a str,
#     pub deger: &'a str,
# }
# pub struct Ayristirici<'a> {
#     kalan: &'a str,
# }
impl<'a> Ayristirici<'a> {
    pub fn sonraki(&mut self) -> Option<Ayar<'_>> {
        let (parca, gerisi) = self.kalan.split_once(';').unwrap_or((self.kalan, ""));
        self.kalan = gerisi;
        let (anahtar, deger) = parca.split_once('=')?;
        Some(Ayar { anahtar, deger })
    }
}

fn main() {
    let mut ayr = Ayristirici { kalan: "ad=Kaan;dil=tr" };
    let a = ayr.sonraki();
    let b = ayr.sonraki();
    println!("{a:?} {b:?}");
}
```

Derleyicinin yanıtı:

```text
error[E0499]: cannot borrow `ayr` as mutable more than once at a time
   |
12 |     let a = ayr.sonraki();
   |             --- first mutable borrow occurs here
13 |     let b = ayr.sonraki();
   |             ^^^ second mutable borrow occurs here
14 |     println!("{a:?} {b:?}");
   |                - first borrow later used here
```

`Ayar<'_>` yazımı "bu ayar yaşadıkça ayrıştırıcı değiştirilebilir biçimde ödünçte" anlamına gelir. `a` elinizdeyken ikinci çağrıya izin çıkmaz; parmağınız sayfaya yapışmıştır.

### Iterator'a dönüştürmek

`Iterator` uygulamak, ayrıştırıcıyı `for` ile gezilebilir ve `collect` ile toplanabilir yapar. Trait'in tek zorunlu metodu `next`'tir.[^iterator] İmzası, sonucun `&mut self`'in ödüncüne bağlanmasına izin vermez: `Item` türünde o ödüncün ömrü geçemez. Bizim sonuçlarımız girdiye bağlı olduğu için bu bir engel değildir:

```rust
# #[derive(Debug, PartialEq, Eq)]
# pub struct Ayar<'a> {
#     pub anahtar: &'a str,
#     pub deger: &'a str,
# }
# pub struct Ayristirici<'a> {
#     kalan: &'a str,
# }
# impl<'a> Ayristirici<'a> {
#     pub fn yeni(girdi: &'a str) -> Self {
#         Ayristirici { kalan: girdi }
#     }
#     pub fn sonraki(&mut self) -> Option<Ayar<'a>> {
#         while !self.kalan.is_empty() {
#             let (parca, gerisi) = self.kalan.split_once(';').unwrap_or((self.kalan, ""));
#             self.kalan = gerisi;
#             if let Some((anahtar, deger)) = parca.split_once('=') {
#                 return Some(Ayar { anahtar: anahtar.trim(), deger: deger.trim() });
#             }
#         }
#         None
#     }
# }
impl<'a> Iterator for Ayristirici<'a> {
    type Item = Ayar<'a>;

    fn next(&mut self) -> Option<Ayar<'a>> {
        self.sonraki()
    }
}

fn main() {
    let girdi = String::from("ad = Kaan; bozuk; dil=tr;tema = koyu");
    let ayarlar: Vec<Ayar> = {
        let ayr = Ayristirici::yeni(&girdi);
        ayr.collect()
    }; // ayrıştırıcı burada düşürüldü
    for a in &ayarlar {
        println!("{} → {}", a.anahtar, a.deger);
    }

    let girdi_araligi = girdi.as_bytes().as_ptr_range();
    let icinde = girdi_araligi.contains(&ayarlar[0].deger.as_ptr());
    println!("değerler girdinin içine bakıyor: {icinde}");

    let adet = Ayristirici::yeni("a=1;;bozuk; b = 2 ;").count();
    println!("boş ve bozuk parçalar atlandı: {adet}");
}
```

Çıktı:

```text
ad → Kaan
dil → tr
tema → koyu
değerler girdinin içine bakıyor: true
boş ve bozuk parçalar atlandı: 2
```

Son iki satır iki iddiayı sınar. Birincisi, dilimlerin gerçekten girdinin içine baktığıdır: Değerin adresi, girdinin kapladığı bellek aralığının içindedir.[^as-ptr-range] İkincisi, boş (`;;`) ve `=` içermeyen parçaların atlandığıdır.

Sonucu alıcının ödüncüne bağlaması *gereken* iterator'lar da vardır; bunlara ödünç veren iterator (lending iterator) denir ve `Iterator` ile yazılamazlar. Çözümleri GAT'lardır: [27. İleri Lifetime, HRTB ve GAT](../advanced/27-lifetime-hrtb-gat.md).

### Girdi ölürse

Sonuçlar girdiye bağlı olduğu için girdiden uzun yaşayamazlar:

```rust,compile_fail,E0597
# #[derive(Debug, PartialEq, Eq)]
# pub struct Ayar<'a> {
#     pub anahtar: &'a str,
#     pub deger: &'a str,
# }
# pub struct Ayristirici<'a> {
#     kalan: &'a str,
# }
# impl<'a> Ayristirici<'a> {
#     pub fn yeni(girdi: &'a str) -> Self {
#         Ayristirici { kalan: girdi }
#     }
# }
# impl<'a> Iterator for Ayristirici<'a> {
#     type Item = Ayar<'a>;
#     fn next(&mut self) -> Option<Ayar<'a>> {
#         while !self.kalan.is_empty() {
#             let (parca, gerisi) = self.kalan.split_once(';').unwrap_or((self.kalan, ""));
#             self.kalan = gerisi;
#             if let Some((anahtar, deger)) = parca.split_once('=') {
#                 return Some(Ayar { anahtar: anahtar.trim(), deger: deger.trim() });
#             }
#         }
#         None
#     }
# }
fn main() {
    let ayarlar = {
        let gecici = String::from("a=1");
        Ayristirici::yeni(&gecici).collect::<Vec<_>>()
    };
    println!("{ayarlar:?}");
}
```

Derleyicinin yanıtı:

```text
error[E0597]: `gecici` does not live long enough
   |
 2 |     let ayarlar = {
   |         ------- borrow later stored here
 3 |         let gecici = String::from("a=1");
   |             ------ binding `gecici` declared here
 4 |         Ayristirici::yeni(&gecici).collect::<Vec<_>>()
   |                           ^^^^^^^ borrowed value does not live long enough
 5 |     };
   |     - `gecici` dropped here while still borrowed
```

Sonuçların daha uzun yaşaması gerekiyorsa tam o noktada sahipli türe geçin:

```rust
# #[derive(Debug, PartialEq, Eq)]
# pub struct Ayar<'a> {
#     pub anahtar: &'a str,
#     pub deger: &'a str,
# }
# pub struct Ayristirici<'a> {
#     kalan: &'a str,
# }
# impl<'a> Ayristirici<'a> {
#     pub fn yeni(girdi: &'a str) -> Self {
#         Ayristirici { kalan: girdi }
#     }
# }
# impl<'a> Iterator for Ayristirici<'a> {
#     type Item = Ayar<'a>;
#     fn next(&mut self) -> Option<Ayar<'a>> {
#         while !self.kalan.is_empty() {
#             let (parca, gerisi) = self.kalan.split_once(';').unwrap_or((self.kalan, ""));
#             self.kalan = gerisi;
#             if let Some((anahtar, deger)) = parca.split_once('=') {
#                 return Some(Ayar { anahtar: anahtar.trim(), deger: deger.trim() });
#             }
#         }
#         None
#     }
# }
fn main() {
    let ayarlar: Vec<(String, String)> = {
        let gecici = String::from("a=1");
        Ayristirici::yeni(&gecici)
            .map(|a| (a.anahtar.to_owned(), a.deger.to_owned()))
            .collect()
    };
    println!("{ayarlar:?}");
}
```

Çıktı:

```text
[("a", "1")]
```

Sıfır kopya bir iyileştirmedir (optimizasyon), zorunlu değildir. Ömürler imzaları karmaşıklaştırmaya başladığında kopyalamak meşru bir seçimdir.

## `'static` ne der, ne demez?

`'static` iki ayrı yerde karşınıza çıkar ve ikincisi sık sık yanlış anlaşılır:[^omur-sinir]

- `&'static T`: Bu *referans* program sonuna kadar geçerlidir. Metin literalleri ve `static` değişkenler böyledir.
- `T: 'static`: Bu *tür*, `'static`'ten kısa ömürlü bir referans içermez. Değerin kendisi her an düşürülebilir. `String`, `Vec<u8>` ve `i32` bu koşulu sağlar, çünkü kimseye borçlu değildirler.

Kendi kitabınız ile kütüphaneden ödünç aldığınız kitabı karşılaştırın. Kendi kitabınızı istediğiniz yere götürür, istediğiniz kadar tutarsınız; iade gününüz yoktur. `T: 'static` bunu söyler: "Bu çantada emanet eşya yok." Çantanın sonsuza kadar saklanacağı anlamına gelmez; kitabınızı yarın başkasına verebilir ya da atabilirsiniz.

Bir değeri ne kadar tutacağı belli olmayan bir yere verirken içinde yakında geçersiz kalacak bir referans bulunmamalıdır. `Box<dyn Display>` böyle bir yerdir: Fonksiyon imzasında `Box<dyn Display + 'static>` yazılmış sayılır.[^dyn-omur] Yeni bir thread başlatan `thread::spawn` da aynı sınırı ister.[^spawn]

```rust
use std::fmt::Display;

fn uzun_sure_sakla<T: Display + 'static>(deger: T) -> Box<dyn Display> {
    Box::new(deger)
}

fn main() {
    let kutu = uzun_sure_sakla(String::from("sahipli String"));
    println!("{kutu}");
    drop(kutu); // 'static sınırı "hiç düşmez" demek değildir

    let sayi_kutusu = uzun_sure_sakla(42);
    let sabit_kutusu = uzun_sure_sakla("literal"); // &'static str
    println!("{sayi_kutusu}, {sabit_kutusu}");
}
```

Çıktı:

```text
sahipli String
42, literal
```

Yerel bir değerin referansı bu sınırı sağlamaz:

```rust,compile_fail,E0597
# use std::fmt::Display;
# fn uzun_sure_sakla<T: Display + 'static>(deger: T) -> Box<dyn Display> {
#     Box::new(deger)
# }
fn main() {
    let yerel = String::from("x");
    let kutu = uzun_sure_sakla(&yerel);
    println!("{kutu}");
}
```

Derleyicinin yanıtı:

```text
error[E0597]: `yerel` does not live long enough
  |
2 |     let yerel = String::from("x");
  |         ----- binding `yerel` declared here
3 |     let kutu = uzun_sure_sakla(&yerel);
  |                ----------------^^^^^^-
  |                |               |
  |                |               borrowed value does not live long enough
  |                argument requires that `yerel` is borrowed for `'static`
4 |     println!("{kutu}");
5 | }
  | - `yerel` dropped here while still borrowed
```

Sınırı yazmayı unutursanız hata bu kez fonksiyonun içinde çıkar, çünkü kutuya konan türün ne kadar geçerli kalacağı bilinmez:

```rust,compile_fail,E0310
use std::fmt::Display;

fn kutula<T: Display>(t: T) -> Box<dyn Display> {
    Box::new(t)
}
```

Derleyicinin yanıtı:

```text
error[E0310]: the parameter type `T` may not live long enough
  |
3 | fn kutula<T: Display>(t: T) -> Box<dyn Display> {
  |                                    ----------- this `dyn Trait` has an implicit `'static` lifetime bound
4 |     Box::new(t)
  |     ^^^^^^^^^^^
  |     |
  |     the parameter type `T` must be valid for the static lifetime...
  |     ...so that the type `T` will meet its required lifetime bounds
  |
help: consider adding an explicit lifetime bound
  |
3 | fn kutula<T: Display + 'static>(t: T) -> Box<dyn Display> {
  |                      +++++++++
```

Bu mesajları görünce referansı `'static` yapmaya çalışmak (`Box::leak` ile belleği bilerek sızdırmak, genel değişken kullanmak) çoğunlukla yanlış yoldur. İki doğru seçenek vardır. İlki sahipliği vermektir: `&yerel` yerine `yerel`. `'static` sınırını "referans verme, değerin kendisini ver" diye okuyabilirsiniz. İkincisi, kutunun gerçekten ödünç veri taşıması gerekiyorsa bunu imzaya yazmaktır:

```rust
use std::fmt::Display;

fn kutula<'a, T: Display + 'a>(t: T) -> Box<dyn Display + 'a> {
    Box::new(t)
}

fn main() {
    let yerel = String::from("ödünç verildi");
    let kutu = kutula(&yerel); // kutu, yerel'den uzun yaşayamaz
    println!("{kutu}");
}
```

Çıktı:

```text
ödünç verildi
```

## Sınırı nereye koymalı?

Okula kayıt olurken kimse "yüzme biliyor musun?" diye sormaz; bu soru havuzun kapısında sorulur. Rust'ta da trait sınırı, ona gerçekten ihtiyaç duyan metotların `impl` bloğuna yazılır, yapının tanımına değil. Yapıya yazılan sınırı, o türün adının geçtiği imzalarda yeniden yazmanız gerekir:

```rust,compile_fail,E0277
use std::collections::HashMap;
use std::hash::Hash;

struct Defter<K: Eq + Hash, V> {
    kayitlar: HashMap<K, V>,
}

// anahtarla işi yok, yalnızca sayıyor
fn kac_kayit<K, V>(d: &Defter<K, V>) -> usize {
    d.kayitlar.len()
}
```

Derleyicinin yanıtı:

```text
error[E0277]: the trait bound `K: Eq` is not satisfied
   |
 9 | fn kac_kayit<K, V>(d: &Defter<K, V>) -> usize {
   |                        ^^^^^^^^^^^^ the trait `Eq` is not implemented for `K`
   |
note: required by a bound in `Defter`
   |
 4 | struct Defter<K: Eq + Hash, V> {
   |                  ^^ required by this bound in `Defter`
```

Derleyici `Hash` için de aynı hatayı verir. Standart kütüphane bu yolu seçmez: `HashMap` tanımında `K` ile `V` için sınır yoktur,[^hashmap-tanim] `K: Eq + Hash` sınırı `insert` ve `get` gibi metotların bulunduğu `impl` bloğundadır.[^hashmap-impl] Aynı düzen `Defter`'de şöyle görünür:

```rust
use std::collections::HashMap;
use std::hash::Hash;

struct Defter<K, V> {
    kayitlar: HashMap<K, V>,
}

// sınır yok: her K, V için
impl<K, V> Defter<K, V> {
    fn yeni() -> Self {
        Defter { kayitlar: HashMap::new() }
    }
}

// sınır kapıda: yalnızca bakmak isteyen için
impl<K: Eq + Hash, V> Defter<K, V> {
    fn bak(&self, k: &K) -> Option<&V> {
        self.kayitlar.get(k)
    }
}

fn kac_kayit<K, V>(d: &Defter<K, V>) -> usize {
    d.kayitlar.len()
}

fn main() {
    let d: Defter<f64, u32> = Defter::yeni(); // f64, Eq değildir; yine de kurulur
    println!("{}", kac_kayit(&d));
    let d: Defter<&str, u32> = Defter::yeni();
    println!("{:?}", d.bak(&"soru"));
}
```

Çıktı:

```text
0
None
```

`f64` türü `Eq` uygulamadığı için `Defter<f64, u32>` üzerinde `bak` çağrılamaz, ama defter kurulabilir ve kayıtları sayılabilir.

### Gerçek bir örnek: önbellek

Zor bir çarpma sorusunu çözünce yanıtını defterinizin arkasına yazarsınız; aynı soru yine gelirse hesaplamaz, deftere bakarsınız. Önbellek bu defterdir. Sorunun (`K`) ve yanıtın (`V`) türü önceden belli olmadığından jenerik yazılır. Gerçek hayatta yanıt bir veritabanı sorgusu ya da dosya okuması olabilir:

```rust
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::hash::Hash;

pub struct Onbellek<K, V> {
    kayitlar: HashMap<K, V>,
    isabet: u32,
    iska: u32,
}

impl<K, V> Onbellek<K, V> {
    pub fn yeni() -> Self {
        Onbellek { kayitlar: HashMap::new(), isabet: 0, iska: 0 }
    }

    pub fn istatistik(&self) -> (u32, u32) {
        (self.isabet, self.iska)
    }
}

impl<K: Eq + Hash, V> Onbellek<K, V> {
    pub fn getir_ya_da_uret(&mut self, anahtar: K, uret: impl FnOnce(&K) -> V) -> &V {
        match self.kayitlar.entry(anahtar) {
            Entry::Occupied(dolu) => {
                self.isabet += 1;
                dolu.into_mut()
            }
            Entry::Vacant(bos) => {
                self.iska += 1;
                let deger = uret(bos.key());
                bos.insert(deger)
            }
        }
    }
}

fn main() {
    let mut onbellek: Onbellek<String, usize> = Onbellek::yeni();
    let pahali_hesap = |metin: &String| {
        println!("(hesaplanıyor: {metin})");
        metin.chars().count()
    };
    let a = *onbellek.getir_ya_da_uret(String::from("merhaba"), pahali_hesap);
    let b = *onbellek.getir_ya_da_uret(String::from("merhaba"), pahali_hesap);
    let c = *onbellek.getir_ya_da_uret(String::from("dünya"), pahali_hesap);
    let istatistik = onbellek.istatistik();
    println!("sonuçlar = {a}, {b}, {c}; (isabet, ıska) = {istatistik:?}");

    let mut kareler: Onbellek<u32, u64> = Onbellek::yeni();
    let kare = kareler.getir_ya_da_uret(12, |n| u64::from(*n) * u64::from(*n));
    println!("12² = {kare}");
}
```

Çıktı:

```text
(hesaplanıyor: merhaba)
(hesaplanıyor: dünya)
sonuçlar = 7, 7, 5; (isabet, ıska) = (1, 2)
12² = 144
```

İkinci "merhaba" çağrısında hesap yapılmadı. `uret` yalnızca değer yoksa, o zaman da bir kez çağrılır; bu yüzden en az şeyi isteyen sınır olan `FnOnce` yeterlidir.[^fnonce] Değer önbellekte kalır, çağırana `&V` olarak ödünç verilir. Aynı yapı, tek satır eklemeden `u32` anahtar ve `u64` değerle de çalıştı; derleyici her tür ikilisi için ayrı kopya üretir.

`entry` çağrısı anahtarı bir kez arar ve dolu (`Occupied`) ya da boş (`Vacant`) yuvayı verir.[^entry] Akla ilk gelen "önce bak, yoksa ekle" yazımı ise ödünç denetleyicisinden geçmez:

```rust,compile_fail,E0502
use std::collections::HashMap;

fn getir(kayitlar: &mut HashMap<u32, String>, k: u32) -> &String {
    if let Some(v) = kayitlar.get(&k) {
        return v;
    }
    kayitlar.insert(k, String::from("yeni"));
    &kayitlar[&k]
}
```

Derleyicinin yanıtı:

```text
error[E0502]: cannot borrow `*kayitlar` as mutable because it is also borrowed as immutable
  |
3 | fn getir(kayitlar: &mut HashMap<u32, String>, k: u32) -> &String {
  |                    - let's call the lifetime of this reference `'1`
4 |     if let Some(v) = kayitlar.get(&k) {
  |                      -------- immutable borrow occurs here
5 |         return v;
  |                - returning this value requires that `*kayitlar` is borrowed for `'1`
6 |     }
7 |     kayitlar.insert(k, String::from("yeni"));
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
```

`v` döndürüldüğü için `get`'in ödüncü, fonksiyonun dönüş ömrü boyunca sürmek zorundadır. Bugünkü denetleyici bu süreyi yalnızca `return` edilen dalda değil, gövdenin tamamında geçerli sayar; NLL tasarım belgesi bu durumu bilinen bir sınırlama olarak anlatır.[^nll] `Entry` hem bu engeli aşar hem de iki aramayı bire indirir.

### Koşullu metotlar

Bir metodun daha fazla koşulu varsa ayrı bir `impl` bloğuna konur. `dokum`, yalnızca anahtarı sıralanabilen ve içeriği yazdırılabilen önbelleklerde bulunur:

```rust
# use std::collections::HashMap;
use std::fmt::Display;

# pub struct Onbellek<K, V> {
#     kayitlar: HashMap<K, V>,
# }
impl<K: Ord + Display, V: Display> Onbellek<K, V> {
    pub fn dokum(&self) -> String {
        let mut satirlar: Vec<(&K, &V)> = self.kayitlar.iter().collect();
        satirlar.sort_by(|a, b| a.0.cmp(b.0)); // HashMap sırası belirsizdir
        satirlar.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(", ")
    }
}

fn main() {
    let onbellek = Onbellek { kayitlar: HashMap::from([("merhaba", 7), ("dünya", 5)]) };
    println!("döküm: {}", onbellek.dokum());
}
```

Çıktı:

```text
döküm: dünya=5, merhaba=7
```

Anahtarı bu koşulları sağlamayan önbellekler kullanılmaya devam eder, yalnızca bu metot onlarda yoktur:

```rust,compile_fail,E0599
# use std::collections::HashMap;
# use std::fmt::Display;
# pub struct Onbellek<K, V> {
#     kayitlar: HashMap<K, V>,
# }
# impl<K: Ord + Display, V: Display> Onbellek<K, V> {
#     pub fn dokum(&self) -> String {
#         String::new()
#     }
# }
#[derive(PartialEq, Eq, Hash)]
struct Kimlik(u32);

fn main() {
    let o: Onbellek<Kimlik, u32> = Onbellek { kayitlar: HashMap::new() };
    o.dokum();
}
```

Derleyicinin yanıtı:

```text
error[E0599]: the method `dokum` exists for struct `Onbellek<Kimlik, u32>`, but its trait bounds were not satisfied
  |
2 | struct Kimlik(u32);
  | ------------- doesn't satisfy `Kimlik: Ord` or `Kimlik: std::fmt::Display`
...
6 |     o.dokum();
  |       ^^^^^ method cannot be called on `Onbellek<Kimlik, u32>` due to unsatisfied trait bounds
```

## İlişkili tür mü, jenerik parametre mi?

Bir trait'in içinde türü boş bırakmanın iki yolu vardır. Fark, o boşluğu *kimin* doldurduğudur:

| Yazım | Kim seçer? | Kaç yanıt olabilir? | Örnek |
| --- | --- | --- | --- |
| `trait Ayristir { type Cikti; }` | Trait'i uygulayan tür | Bir tür için tek yanıt | `Iterator::Item`, `Deref::Target`, `Add::Output` |
| `trait Donustur<T> { .. }` | Çağıran | Aynı tür birçok `T` için ayrı ayrı uygulayabilir | `From<T>`: `String`, hem `From<&str>` hem `From<char>` uygular |

Her sınıfın tek bir sınıf öğretmeni vardır. "3-A'nın öğretmeni kim?" sorusunun tek yanıtı olur; yanıtı soran kişi seçmez, sınıf belirler. İlişkili tür budur. Öte yandan bir öğrenci hem satranç hem resim kulübüne üye olabilir; "şu kulübe üye mi?" sorusu her kulüp için ayrı ayrı sorulur. Jenerik parametreli trait de budur. Ölçüt şu sorudur: Bu tür için birden fazla anlamlı yanıt olabilir mi? Olamazsa ilişkili tür seçilir; çağıran her yerde tür yazmaktan kurtulur ve belirsizlik baştan kapanır.[^iliskili-tur]

```rust
pub trait Ayristir {
    type Cikti;
    fn ayristir(&self, ham: &str) -> Option<Self::Cikti>;
}

pub struct TamSayi;
pub struct EvetHayir;

impl Ayristir for TamSayi {
    type Cikti = i64;
    fn ayristir(&self, ham: &str) -> Option<i64> {
        ham.trim().parse().ok()
    }
}

impl Ayristir for EvetHayir {
    type Cikti = bool;
    fn ayristir(&self, ham: &str) -> Option<bool> {
        match ham.trim().to_lowercase().as_str() {
            "evet" | "e" | "1" => Some(true),
            "hayır" | "h" | "0" => Some(false),
            _ => None,
        }
    }
}

pub fn hepsini_ayristir<A: Ayristir>(a: &A, girdiler: &[&str]) -> Vec<A::Cikti> {
    girdiler.iter().filter_map(|g| a.ayristir(g)).collect()
}

fn main() {
    let sayilar = hepsini_ayristir(&TamSayi, &["10", " 20 ", "otuz", "-4"]);
    let cevaplar = hepsini_ayristir(&EvetHayir, &["Evet", "h", "belki", "1"]);
    println!("sayılar = {sayilar:?}");
    println!("cevaplar = {cevaplar:?}");
}
```

Çıktı:

```text
sayılar = [10, 20, -4]
cevaplar = [true, false, true]
```

`A::Cikti`, "`A` hangi ayrıştırıcıysa onun çıktı türü" diye okunur. `TamSayi` verince `Vec<i64>`, `EvetHayir` verince `Vec<bool>` gelir; çağıran `hepsini_ayristir::<TamSayi, i64>` gibi ikinci bir tür yazmaz. Aynı trait'i aynı tür için ikinci kez, farklı bir `Cikti` ile uygulayamazsınız:

```rust,compile_fail,E0119
# pub trait Ayristir {
#     type Cikti;
# }
pub struct TamSayi;

impl Ayristir for TamSayi {
    type Cikti = i64;
}

impl Ayristir for TamSayi {
    type Cikti = u8;
}
```

Derleyicinin yanıtı:

```text
error[E0119]: conflicting implementations of trait `Ayristir` for type `TamSayi`
   |
 3 | impl Ayristir for TamSayi {
   | ------------------------- first implementation here
...
 7 | impl Ayristir for TamSayi {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^ conflicting implementation for `TamSayi`
```

Jenerik parametreli bir trait'te ise her `T` ayrı bir uygulamadır:

```rust
struct Ad(String);

impl From<&str> for Ad {
    fn from(s: &str) -> Self {
        Ad(s.to_owned())
    }
}

impl From<char> for Ad {
    fn from(c: char) -> Self {
        Ad(c.to_string())
    }
}

fn main() {
    println!("{} {}", Ad::from("Ada").0, Ad::from('K').0);
}
```

Çıktı:

```text
Ada K
```

### Yanlış seçim neye yol açar?

`Iterator` bir `Item` türü yerine jenerik parametre alsaydı, bir tür hem `i32` hem `String` veren iterator olabilirdi. O zaman döngüde hangisinin kastedildiği belirsiz kalır, türü her kullanımda elle yazmak gerekirdi. Küçük bir taklidi bu belirsizliği gösterir:

```rust,compile_fail,E0283
trait Gezgin<T> {
    fn sonraki(&mut self) -> Option<T>;
}

struct Sayac;

impl Gezgin<i32> for Sayac {
    fn sonraki(&mut self) -> Option<i32> {
        Some(1)
    }
}

impl Gezgin<String> for Sayac {
    fn sonraki(&mut self) -> Option<String> {
        None
    }
}

fn main() {
    let mut s = Sayac;
    let x = s.sonraki();
}
```

Derleyicinin yanıtı:

```text
error[E0283]: type annotations needed for `Option<_>`
   |
21 |     let x = s.sonraki();
   |         ^     ------- type must be known at this point
   |
note: multiple `impl`s satisfying `Sayac: Gezgin<_>` found
   |
 7 | impl Gezgin<i32> for Sayac {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^
...
13 | impl Gezgin<String> for Sayac {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

`let x: Option<i32> = s.sonraki();` yazarsanız belirsizlik kalkar, ama bu yükü her çağıran taşır.

### İkisi bir arada

Bir trait hem jenerik parametre hem ilişkili tür taşıyabilir. Aşağıdaki `Animal<T>` trait'inde `T`'yi çağıran seçer, `Your`'ı her uygulama kendisi doldurur. `voice` bir ilişkili fonksiyondur: `self` almaz, tür üzerinden çağrılır.

```rust
trait Animal<T> {
    type Your;

    fn voice() -> T;
}

struct Color<F>
where
    F: Fn(i8) -> i8,
{
    x: F,
}

impl<F> Animal<i32> for Color<F>
where
    F: Fn(i8) -> i8,
{
    type Your = String;

    fn voice() -> i32 {
        1
    }
}

impl<F> Animal<String> for Color<F>
where
    F: Fn(i8) -> i8,
{
    type Your = u8;

    fn voice() -> String {
        String::from("mırr")
    }
}

fn main() {
    // T çağrıda belli oluyor
    let sayi: i32 = Color::<fn(i8) -> i8>::voice();
    let ses: String = Color::<fn(i8) -> i8>::voice();
    // ilişkili türe erişmek için hangi Animal olduğu yazılır
    let your: <Color<fn(i8) -> i8> as Animal<i32>>::Your = String::from("Your");
    let renk = Color { x: |y: i8| y };
    println!("{sayi} {ses} {your} {}", (renk.x)(5));
}
```

Çıktı:

```text
1 mırr Your 5
```

`voice` hiçbir argüman almadığından `F` argümanlardan çıkarılamaz; turbofish ile somut bir tür verilir. `fn(i8) -> i8` fonksiyon işaretçisi türü de `Fn(i8) -> i8` sınırını sağlar. `T` ise dönüş türünden anlaşılır.

`learn/basic-concepts` kodunda `Color` uygulamasının yanında bir de genel impl vardır: `impl<T: Future> Animal<i32> for T`. İkisi birlikte derlenir. Genel impl'in kapsadığı türlerle `Color`'ın kesişmediğini derleyici kanıtlayabilir: `Color` bu crate'te tanımlıdır ve `Future` uygulamaz. Kesişselerdi tutarlılık denetimi E0119 verirdi.[^tutarlilik] Ayrıntısı [23. Tutarlılık ve Yetim Kuralı](../advanced/23-coherence.md) bölümündedir.

## Sık yapılan hatalar

### Karşılaştırma sınırını unutmak

```rust,compile_fail,E0369
fn buyuk<T>(a: T, b: T) -> T {
    if a > b { a } else { b }
}
```

Derleyicinin yanıtı:

```text
error[E0369]: binary operation `>` cannot be applied to type `T`
  |
2 |     if a > b { a } else { b }
  |        - ^ - T
  |        |
  |        T
  |
help: consider restricting type parameter `T` with trait `PartialOrd`
  |
1 | fn buyuk<T: std::cmp::PartialOrd>(a: T, b: T) -> T {
  |           ++++++++++++++++++++++
```

Jenerik bir gövde, `T` hakkında yalnızca sınırların söylediğini bilir. Sınırı olmayan C++ şablonlarında aynı hata, şablon uygun olmayan bir türle kullanıldığında ve her kullanım yerinde yeniden çıkar;[^cpp] Rust'ta ise tanımın bulunduğu yerde, bir kez çıkar. Düzeltmesi sınırı eklemektir:

```rust
fn buyuk<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

fn main() {
    println!("{} {}", buyuk(3, 7), buyuk("a", "b"));
}
```

Çıktı:

```text
7 b
```

### Hedef türü söylememek

```rust,compile_fail,E0284
fn main() {
    let n = "5".parse().expect("5 bir sayıdır");
}
```

Derleyicinin yanıtı:

```text
error[E0284]: type annotations needed
  |
2 |     let n = "5".parse().expect("5 bir sayıdır");
  |         ^       ----- type must be known at this point
  |
  = note: cannot satisfy `<_ as FromStr>::Err == _`
```

`parse`, `collect` ve `into` gibi metotlarda sonucun türü jeneriktir ve argümanlardan anlaşılmaz; derleyici hedefi tahmin etmez.[^parse] Türü değişkenin yanına ya da turbofish ile çağrıya yazın:

```rust
fn main() {
    let n: u32 = "5".parse().expect("5 bir sayıdır");
    let m = "5".parse::<u32>().expect("5 bir sayıdır");
    println!("{n} {m}");
}
```

Çıktı:

```text
5 5
```

### Struct alanında ömrü yazmamak

```rust,compile_fail,E0106
struct Baslik {
    metin: &str,
}
```

Derleyicinin yanıtı:

```text
error[E0106]: missing lifetime specifier
  |
2 |     metin: &str,
  |            ^ expected named lifetime parameter
  |
help: consider introducing a named lifetime parameter
  |
1 ~ struct Baslik<'a> {
2 ~     metin: &'a str,
  |
```

Ömür çıkarımı fonksiyon imzalarında yapılır, struct alanlarında yapılmaz. Alan ya bir ömür parametresi taşır ya da açıkça `'static` yazılır; ikinci durumda yapının parametreye ihtiyacı yoktur:

```rust
struct Baslik<'a> {
    metin: &'a str,
}

struct SabitBaslik {
    metin: &'static str,
}

fn main() {
    let a = Baslik { metin: "ödünç" };
    let b = SabitBaslik { metin: "gömülü" };
    println!("{} {}", a.metin, b.metin);
}
```

Çıktı:

```text
ödünç gömülü
```

### Alıcıya `&'a mut self` yazmak

```rust,compile_fail,E0499
struct Sayac<'a> {
    ad: &'a str,
    n: u32,
}

impl<'a> Sayac<'a> {
    fn artir(&'a mut self) {
        self.n += 1;
    }
}

fn main() {
    let mut s = Sayac { ad: "sayaç", n: 0 };
    s.artir();
    s.artir();
}
```

Derleyicinin yanıtı:

```text
error[E0499]: cannot borrow `s` as mutable more than once at a time
   |
14 |     s.artir();
   |     - first mutable borrow occurs here
15 |     s.artir();
   |     ^
   |     |
   |     second mutable borrow occurs here
   |     first borrow later used here
```

Struct'ın kendi `'a` parametresini alıcıya yazmak, "beni, içimdeki verinin *bütün* ömrü boyunca özel olarak ödünç al" demektir. Kitap iade edilene kadar parmağınızı sayfadan çekmemeye söz vermiş olursunuz: İlk çağrıdan sonra değer kullanılamaz, alanını okumak bile E0502 verir. Alıcıyı etiketsiz bırakın; derleyici her çağrıya kısa, taze bir ödünç süresi verir:

```rust
struct Sayac<'a> {
    ad: &'a str,
    n: u32,
}

impl<'a> Sayac<'a> {
    fn artir(&mut self) {
        self.n += 1;
    }
}

fn main() {
    let mut s = Sayac { ad: "sayaç", n: 0 };
    s.artir();
    s.artir();
    println!("{} = {}", s.ad, s.n);
}
```

Çıktı:

```text
sayaç = 2
```

## Akılda kalması gerekenler

- Ömür etiketi yazarken "sonuç hangi girdiden türüyor?" diye sorun. Yalnızca o girdi sonuçla aynı etiketi taşımalıdır; hepsine `'a` yazmak derlenir, ama çağıranı kısıtlar.
- Referans tutan bir struct'ta metot iki şeye bağlı sonuç döndürebilir: struct'ın `'a`'sına (veri kaynağına, uzun) ya da `&self`'in ömrüne (struct'ın kendisine, kısa). Mümkünse `'a` döndürün.
- `T: 'static`, "referans içermeyen ya da yalnızca `'static` referans içeren tür" demektir. İçinde ömür parametresi bulunmayan sahipli türler bunu sağlar.
- Jenerik gövde yalnızca sınırların verdiği yetenekleri kullanabilir. Sınırı yapıya değil, ihtiyacı olan `impl`'e koyun ve en az şeyi isteyeni seçin: Bir kez çağrılacak closure için `FnOnce` yeterlidir.
- Tek anlamlı yanıt varsa ilişkili tür, birden çok yanıt varsa jenerik parametre seçin.
- Ömürler çalışan programda yoktur: Denetim derleme zamanında biter, sıfır kopyalı ayrıştırıcıya hiçbir çalışma zamanı denetimi eklenmez.[^silinme]

## Alıştırmalar

Çözümlere bakmadan önce kendiniz deneyin.

1. `fn en_uzun_deger<'a>(ayarlar: &[Ayar<'a>]) -> Option<&'a str>` yazın; en uzun `deger`'i döndürsün. Dönüşün `ayarlar` dilimine değil girdi metnine bağlı olduğunu, `Vec<Ayar>` düşürüldükten sonra sonucu yazdırarak gösterin. Dönüş türü `Option<&str>` olursa fonksiyon neden derlenmez? `fn f<'s, 'a>(ayarlar: &'s [Ayar<'a>]) -> Option<&'s str>` derlenir, ama aynı kullanım derlenmez; neden?
2. `Onbellek`'e `fn gecersiz_kil(&mut self, anahtar: &K) -> Option<V>` ekleyin. Hangi `impl` bloğuna konmalı, neden? Ardından `HashMap::remove` imzasına bakın: Parametre neden `&K` değil de `&Q where K: Borrow<Q>`?
3. `Ayristir`'ı `Vec<A>` için uygulayın: `Cikti = Vec<A::Cikti>` olsun, girdi virgülden bölünsün ve `i`. parça `i`. ayrıştırıcıya verilsin; biri bile başarısızsa `None` dönsün. Beklenen: `vec![TamSayi, TamSayi]` ile `"3, 4"` için `Some([3, 4])`, `"3, x"` için `None`.

<details>
<summary>1. alıştırma: ipucu ve çözüm</summary>

İpucu: `iter().map(|a| a.deger).max_by_key(|d| d.len())`.

```rust
struct Ayar<'a> {
    deger: &'a str,
}

fn en_uzun_deger<'a>(ayarlar: &[Ayar<'a>]) -> Option<&'a str> {
    ayarlar.iter().map(|a| a.deger).max_by_key(|d| d.len())
}

fn main() {
    let girdi = String::from("ad=Kaan;tema=koyu-mavi");
    let sonuc = {
        let ayarlar: Vec<Ayar> = girdi
            .split(';')
            .filter_map(|p| p.split_once('='))
            .map(|(_, deger)| Ayar { deger })
            .collect();
        en_uzun_deger(&ayarlar)
    }; // ayarlar burada düşürüldü
    println!("{sonuc:?}");
}
```

Çıktı:

```text
Some("koyu-mavi")
```

Dönüş `Option<&str>` olsaydı parametrede iki ömür bulunurdu: dilimin ödüncü ve `Ayar`'ın içindeki ömür. İkinci kural tek ömür ister, metot olmadığı için üçüncü kural da uygulanmaz; derleyici E0106 verir:

```rust,compile_fail,E0106
struct Ayar<'a> {
    deger: &'a str,
}

fn en_uzun_deger(ayarlar: &[Ayar]) -> Option<&str> {
    ayarlar.iter().map(|a| a.deger).max_by_key(|d| d.len())
}
```

Derleyicinin yanıtı:

```text
error[E0106]: missing lifetime specifier
  |
5 | fn en_uzun_deger(ayarlar: &[Ayar]) -> Option<&str> {
  |                           -------            ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but the signature does not say which one of `ayarlar`'s 2 lifetimes it is borrowed from
help: consider introducing a named lifetime parameter
```

Derleyicinin önerisi `&'a [Ayar<'a>]` yazmaktır; bu da derlenir, ama iki ömrü gereksiz yere eşitler. `'s` sürümünde sonuç dilimin ödüncüne bağlanır; `ayarlar` düştüğünde sonuç da geçersiz sayılır ve aynı kullanım E0597 ile reddedilir. Sonuç gerçekte girdiden geldiği için doğru etiket `'a`'dır.

</details>

<details>
<summary>2. alıştırma: ipucu ve çözüm</summary>

İpucu: `Onbellek<String, _>` içinden `&str` ile silebilmek istiyorsunuz.

`remove`, `K: Eq + Hash` istediği için metot `getir_ya_da_uret` ile aynı `impl` bloğuna konur; sınırsız bloğa konursa derlenmez, `dokum`'un bloğuna konursa gereksiz yere `Ord + Display` ister.

```rust
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

pub struct Onbellek<K, V> {
    kayitlar: HashMap<K, V>,
}

impl<K: Eq + Hash, V> Onbellek<K, V> {
    pub fn gecersiz_kil(&mut self, anahtar: &K) -> Option<V> {
        self.kayitlar.remove(anahtar)
    }

    pub fn gecersiz_kil_esnek<Q>(&mut self, anahtar: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        self.kayitlar.remove(anahtar)
    }
}

fn main() {
    let mut o = Onbellek {
        kayitlar: HashMap::from([(String::from("ad"), 1), (String::from("dil"), 2)]),
    };
    println!("{:?}", o.gecersiz_kil(&String::from("ad")));
    println!("{:?}", o.gecersiz_kil_esnek("dil"));
    println!("{:?}", o.gecersiz_kil_esnek("dil"));
}
```

Çıktı:

```text
Some(1)
Some(2)
None
```

`&K` isteyen sürüm, anahtarı `String` olan bir önbellekte `&String` ister; elinizde yalnızca `&str` varsa silmek için bir `String` tahsis etmeniz gerekir. `K: Borrow<Q>` "anahtar `Q` olarak ödünç verilebilir" der; `String: Borrow<str>` olduğundan `&str` yeterli olur. `Borrow`, iki biçimin eşitlik ve özetinin aynı olmasını da şart koşar, arama bu yüzden doğru çalışır.[^borrow] `?Sized`, boyutu derleme zamanında bilinmeyen `str` türüne izin verir.

</details>

<details>
<summary>3. alıştırma: ipucu ve çözüm</summary>

İpucu: `zip` ve `collect::<Option<Vec<_>>>()`.

```rust
pub trait Ayristir {
    type Cikti;
    fn ayristir(&self, ham: &str) -> Option<Self::Cikti>;
}

pub struct TamSayi;

impl Ayristir for TamSayi {
    type Cikti = i64;
    fn ayristir(&self, ham: &str) -> Option<i64> {
        ham.trim().parse().ok()
    }
}

impl<A: Ayristir> Ayristir for Vec<A> {
    type Cikti = Vec<A::Cikti>;

    fn ayristir(&self, ham: &str) -> Option<Self::Cikti> {
        let parcalar: Vec<&str> = ham.split(',').collect();
        if parcalar.len() != self.len() {
            return None;
        }
        self.iter().zip(parcalar).map(|(a, p)| a.ayristir(p)).collect()
    }
}

fn main() {
    let ikili = vec![TamSayi, TamSayi];
    println!("{:?}", ikili.ayristir("3, 4"));
    println!("{:?}", ikili.ayristir("3, x"));
    println!("{:?}", ikili.ayristir("3"));
}
```

Çıktı:

```text
Some([3, 4])
None
None
```

`zip` kısa olan tarafta durur; uzunluk denetimi olmasaydı `"3"` girdisi `Some([3])` verirdi. `Option<Vec<_>>` içine toplamak, ilk `None`'da durup bütün sonucu `None` yapar.[^collect-option] Burada `Ayristir` bizim crate'imizde tanımlı olduğu için yabancı bir tür olan `Vec<A>`'ya uygulanabilir; bu da bir genel impl örneğidir.

</details>

## Özet

- İmza bir sözleşmedir: Ömür etiketleri sonucun hangi girdiye bağlı olduğunu, sınırlar türlerden ne beklendiğini yazar. Gereğinden fazla söz, çağıranı kısıtlar.
- Sıfır kopyalı sonuçları veri kaynağının ömrüne bağlayın; alıcının ödüncüne bağlamak ya da `&'a mut self` yazmak değeri kilitler.
- `'static` sınırı "içinde kısa ömürlü referans yok" demektir; çözüm çoğu zaman sahipliği vermektir.
- Sınırlar ihtiyaç duyulan `impl` bloğunda durur; ilişkili tür ile jenerik parametre arasındaki seçim, yanıtı kimin verdiğine bağlıdır.

Trait'ler closure'ların da temelidir (`Fn`, `FnMut` ve `FnOnce` birer trait sınırıdır): [12. Closure ve Iterator'lar](12-closures-iterators.md). `dyn Trait` ile çalışma zamanında polimorfizm [17. Nesne Yönelimli Özellikler](17-oop.md) bölümündedir. Sıradaki bölümde bu imzaların ve sözleşmelerin testlerle nasıl sınandığını göreceğiz: [10. Test Yazma](10-writing-tests.md).

## Kaynaklar

[^omur-soz]: The Rust Programming Language, [10.3, Lifetime Annotation Syntax](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html#lifetime-annotation-syntax).
[^elision]: The Rust Reference, [Lifetime elision in functions](https://doc.rust-lang.org/reference/lifetime-elision.html#lifetime-elision-in-functions).
[^imza]: The Rust Programming Language, [10.3, In Function Signatures](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html#in-function-signatures).
[^omur-sinir]: The Rust Reference, [Lifetime bounds](https://doc.rust-lang.org/reference/trait-bounds.html#lifetime-bounds).
[^trim-prefix]: Rust standart kütüphanesi 1.100.0, [`str::trim_prefix`](https://doc.rust-lang.org/1.100.0/std/primitive.str.html#method.trim_prefix).
[^iterator]: Rust standart kütüphanesi, [`Iterator::next`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#tymethod.next).
[^as-ptr-range]: Rust standart kütüphanesi, [`slice::as_ptr_range`](https://doc.rust-lang.org/std/primitive.slice.html#method.as_ptr_range).
[^dyn-omur]: The Rust Reference, [Default trait object lifetimes](https://doc.rust-lang.org/reference/lifetime-elision.html#default-trait-object-lifetimes).
[^spawn]: Rust standart kütüphanesi, [`std::thread::spawn`](https://doc.rust-lang.org/std/thread/fn.spawn.html).
[^hashmap-tanim]: Rust 1.99.0 standart kütüphane kaynağı, [`map.rs`, `pub struct HashMap`](https://github.com/rust-lang/rust/blob/1.99.0/library/std/src/collections/hash/map.rs#L245).
[^hashmap-impl]: Rust 1.99.0 standart kütüphane kaynağı, [`map.rs`, `K: Eq + Hash`](https://github.com/rust-lang/rust/blob/1.99.0/library/std/src/collections/hash/map.rs#L891).
[^fnonce]: Rust standart kütüphanesi, [`std::ops::FnOnce`](https://doc.rust-lang.org/std/ops/trait.FnOnce.html).
[^entry]: Rust standart kütüphanesi, [`HashMap::entry`](https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.entry).
[^nll]: RFC 2094, [Problem case #3: conditional control flow across functions](https://rust-lang.github.io/rfcs/2094-nll.html#problem-case-3-conditional-control-flow-across-functions).
[^iliskili-tur]: The Rust Programming Language, [20.2, Defining Traits with Associated Types](https://doc.rust-lang.org/book/ch20-02-advanced-traits.html#defining-traits-with-associated-types).
[^tutarlilik]: The Rust Reference, [Trait implementation coherence](https://doc.rust-lang.org/reference/items/implementations.html#trait-implementation-coherence).
[^cpp]: cppreference, [Constraints and concepts](https://en.cppreference.com/cpp/language/constraints).
[^parse]: Rust standart kütüphanesi, [`str::parse`](https://doc.rust-lang.org/std/primitive.str.html#method.parse).
[^silinme]: rustc 1.99.0 kaynak kodu, [`region_kind.rs`, `ReErased`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_type_ir/src/region_kind.rs#L182).
[^borrow]: Rust standart kütüphanesi, [`std::borrow::Borrow`](https://doc.rust-lang.org/std/borrow/trait.Borrow.html).
[^collect-option]: Rust standart kütüphanesi 1.99.0, [`Option` için `FromIterator`](https://doc.rust-lang.org/1.99.0/std/option/enum.Option.html#impl-FromIterator%3COption%3CA%3E%3E-for-Option%3CV%3E).
