# 3.1 Ownership: Derinleşme

Önceki sayfada kuralları tek tek gördük. Bu sayfa aynı kuralları şu soruyla yeniden ele alıyor: Kod yazarken karşımıza nasıl çıkarlar? Sırasıyla üç konuya bakacağız:

- Fonksiyon imzası yazarken `String` mi, `&str` mi, `&mut String` mi istemeli?
- Gerçek bir veri yapısında sahiplik kimde durur, kim ödünç alır?
- Ödünç denetleyicisinin en sık verdiği beş hata hangi alışkanlıktan doğar ve yerleşik çözümleri nelerdir?

> [!NOTE]
> Bu sayfanın kodu `learn/ownership/src/deep_dive.rs` dosyasındadır. Testlerini `cargo test -p ownership` ile çalıştırabilirsiniz.

<details>
<summary>Bu sayfada geçen yeni terimler</summary>

- **imza (signature):** Fonksiyonun adını, ne aldığını ve ne döndürdüğünü söyleyen ilk satırı.
- **kısmi taşıma (partial move):** Bir yapının tamamını değil, yalnızca bir alanını taşımak.
- **tahsis (allocation):** Heap'ten yeni yer ayırmak.
- **iterator:** Öğeleri sırayla, tek tek veren değer.
- **closure:** Yerinde yazılmış isimsiz fonksiyon; örneğin `|k| k.id == id`.
- **ömür çıkarımı (elision):** İmzada yazılmayan ömür bilgisini derleyicinin kurallara göre tamamlaması.
- **değişmez (invariant):** Programın hiçbir anında bozulmaması gereken koşul.
- **yerleşik (idiomatic):** Dili bilenlerin alışık olduğu yazım biçimi.

</details>

## İmza bir sahiplik sözleşmesidir

Rust'ta parametre türü yalnızca "ne alıyorum" demez; "aldığım şeyle ne yapacağım" da der. Çağıran kişi gövdeyi okumadan, yalnızca imzaya bakarak verisinin akıbetini bilir:

| İmza | Anlamı | Çağıran açısından |
| --- | --- | --- |
| `fn f(x: String)` | Bunu senden alıyorum. | Değer taşınır; artık kullanılamaz. |
| `fn f(x: &str)` | Yalnızca bakacağım. | Değer olduğu gibi kalır. |
| `fn f(x: &mut String)` | Yerinde değiştireceğim. | Sonucu kendi değişkeninde görür. |

Bir kütüphane bankosunda üç sepet düşünün. *Bağış* sepetine bıraktığınız kitap artık kütüphanenindir. *Salonda okunur* sepetindeki kitaba görevli yalnızca bakar. *Onarılacak* sepetindeki kitabın yırtığı yapıştırılır ve kitap size onarılmış hâliyle döner. Parametre türü, sepetin üstündeki yazıdır; bu yazı olmasaydı kitabın geri gelip gelmeyeceğini görevlinin yaptıklarından çıkarmanız gerekirdi.

Çoğu durumda şu üç soru doğru imzayı verir:

1. Değeri saklayacak mısınız, örneğin bir alana koyacak ya da başka bir thread'e mi göndereceksiniz? Sahipliği alın.
2. Yalnızca okuyacak mısınız? Ödünç alın. Metin için `&String` yerine `&str`, dizi için `&Vec<T>` yerine `&[T]` isteyin; dilim daha geneldir.[^dilim-parametre]
3. Yerinde değiştirecek misiniz? `&mut` ile ödünç alın.

Üçünü aynı programda görelim:

```rust
// Yalnızca okur: &String de, literal de, bir alt dilim de verilebilir.
fn kelime_say(metin: &str) -> usize {
    metin.split_whitespace().count()
}

// Yerinde değiştirir: yeni tahsis yapmadan var olan tamponu düzenler.
fn bosluklari_kirp(metin: &mut String) {
    let kirpilmis = metin.trim().len();
    let bas = metin.len() - metin.trim_start().len();
    metin.drain(..bas); // baştaki boşluklar
    metin.truncate(kirpilmis); // sondaki boşluklar
}

// Sahiplenir: aldığı tamponu yeniden kullanıp geri verir, kopya çıkarmaz.
fn selamlamaya_cevir(mut ad: String) -> String {
    ad.insert_str(0, "Merhaba, ");
    ad.push('!');
    ad
}

fn main() {
    let mut girdi = String::from("  Ada Lovelace  ");

    println!("kelime sayısı = {}", kelime_say(&girdi));
    println!("literal de olur = {}", kelime_say("bir iki üç"));

    bosluklari_kirp(&mut girdi);
    println!("kırpılmış = '{girdi}'");

    let selam = selamlamaya_cevir(girdi); // girdi buraya taşındı
    println!("{selam}");
}
```

Çıktı:

```text
kelime sayısı = 2
literal de olur = 3
kırpılmış = 'Ada Lovelace'
Merhaba, Ada Lovelace!
```

Son satırdan sonra `girdi` artık kullanılamaz; kullanırsanız `E0382` alırsınız.

> [!WARNING]
> Yalnızca okuyacak bir fonksiyona `String` parametresi yazmak sık yapılan bir hatadır. Kod derlenir, ama çağıranı her seferinde `.clone()` yazmaya, yani gereksiz tahsise zorlar. İmza ne kadar az şey isterse fonksiyon o kadar çok yerde kullanılabilir.

## Kısmi taşıma

Taşıma yalnızca değişkenin bütününe değil, alanlarına da uygulanır. Bir alanı taşıdığınızda yapı *kısmen taşınmış* olur: Kalan alanları tek tek kullanabilirsiniz, yapıyı bütün olarak kullanamazsınız.[^kismi]

Üç ciltlik bir masal seti gibi: İkinci cildi bağışlarsanız birinci ve üçüncü cildi okumaya devam edersiniz, ama elinizdekini artık "tam set" diye kimseye veremezsiniz.

```rust
struct Profil {
    ad: String,
    eposta: String,
    yas: u32,
}

fn main() {
    let p = Profil {
        ad: String::from("Kaan"),
        eposta: String::from("kaan@ornek.dev"),
        yas: 30,
    };

    let ad = p.ad; // yalnızca bu alan taşındı
    println!("ad = {ad}, e-posta = {}, yaş = {}", p.eposta, p.yas);
}
```

Çıktı:

```text
ad = Kaan, e-posta = kaan@ornek.dev, yaş = 30
```

Yapının bütününü kullanmaya kalkarsanız derleyici itiraz eder:

```rust,compile_fail,E0382
# struct Profil { ad: String, eposta: String, yas: u32 }
# fn main() {
# let p = Profil { ad: String::from("Kaan"), eposta: String::from("k@ornek.dev"), yas: 30 };
let ad = p.ad;
let tamami = p; // bir alanı eksik
# }
```

Derleyicinin yanıtı:

```text
error[E0382]: use of partially moved value: `p`
```

Bütüne izin verilseydi, taşınan ad hem yeni sahibinde hem `tamami` içinde sayılır ve aynı bellek iki kez geri verilmeye çalışılırdı. Derleyici bu yüzden taşımaları değişken düzeyinde değil, alan düzeyinde izler.[^move-path]

Yapıyı gerçekten parçalayacaksanız bütün alanları tek bir desenle çözmek daha açıktır; geride yarım kalmış bir yapı bırakmaz:

```rust
# struct Profil { ad: String, eposta: String, yas: u32 }
# fn main() {
let p = Profil {
    ad: String::from("Ada"),
    eposta: String::from("ada@ornek.dev"),
    yas: 36,
};

let Profil { ad, eposta, yas } = p; // üç alan, üç ayrı sahip
println!("çözüldü: {ad} <{eposta}> ({yas})");
# }
```

Çıktı:

```text
çözüldü: Ada <ada@ornek.dev> (36)
```

Yapıyı bozmadan yalnızca bir alanı okumak istiyorsanız alanı ödünç alın: `&p.ad`.

## Gerçek bir örnek: Kullanıcı deposu

Bir veri yapısı tasarlarken ilk soru her zaman aynıdır: **Bu verinin tek sahibi kim olmalı?** Burada cevap depodur. Kullanıcılar deponun `Vec`'inde yaşar, dışarıya yalnızca referans verilir. Kütüphane benzetmesiyle söylersek kitapların sahibi raftır; okumak isteyen raftaki kitaba bakar, kitabı alıp götürmek isteyen raftan çıkarır.

Metot imzaları sahipliğin nasıl aktığını belgeler:

| Metot | Sahiplik |
| --- | --- |
| `ekle(&mut self, k: Kullanici)` | Kullanıcının sahipliği depoya geçer. |
| `bul(&self, id: u32) -> Option<&Kullanici>` | Sahip depo kalır, çağıran yalnızca bakar. |
| `bul_mut(&mut self, id: u32) -> Option<&mut Kullanici>` | Sahip depo kalır, çağıran yerinde günceller. |
| `cikar(&mut self, id: u32) -> Option<Kullanici>` | Sahiplik depodan çağırana taşınır. |

```rust
#[derive(Debug, PartialEq, Eq)]
pub struct Kullanici {
    pub id: u32,
    pub ad: String,
    pub aktif: bool,
}

#[derive(Default)]
pub struct KullaniciDeposu {
    kayitlar: Vec<Kullanici>,
}

impl KullaniciDeposu {
    pub fn ekle(&mut self, k: Kullanici) {
        self.kayitlar.push(k); // k buraya taşındı
    }

    pub fn bul(&self, id: u32) -> Option<&Kullanici> {
        self.kayitlar.iter().find(|k| k.id == id)
    }

    pub fn bul_mut(&mut self, id: u32) -> Option<&mut Kullanici> {
        self.kayitlar.iter_mut().find(|k| k.id == id)
    }

    pub fn cikar(&mut self, id: u32) -> Option<Kullanici> {
        let sira = self.kayitlar.iter().position(|k| k.id == id)?;
        Some(self.kayitlar.remove(sira)) // Vec'ten çıkar, sahipliğiyle döner
    }

    pub fn aktif_adlar(&self) -> Vec<&str> {
        self.kayitlar
            .iter()
            .filter(|k| k.aktif)
            .map(|k| k.ad.as_str())
            .collect()
    }
}
#
# fn main() {}
```

`aktif_adlar` adları kopyalamaz; depodaki adlara bakan dilimlerden oluşan bir liste döndürür. İmzada ömür yazılmamıştır, ama ömür çıkarımı kuralları dönen `&str` değerlerini `&self` ödüncüne bağlar.[^elision] Yani bu liste kullanılırken depo değiştirilemez.

Depoyu kullanalım:

```rust
# #[derive(Debug, PartialEq, Eq)]
# pub struct Kullanici { pub id: u32, pub ad: String, pub aktif: bool }
# #[derive(Default)]
# pub struct KullaniciDeposu { kayitlar: Vec<Kullanici> }
# impl KullaniciDeposu {
#     pub fn ekle(&mut self, k: Kullanici) { self.kayitlar.push(k); }
#     pub fn bul(&self, id: u32) -> Option<&Kullanici> { self.kayitlar.iter().find(|k| k.id == id) }
#     pub fn bul_mut(&mut self, id: u32) -> Option<&mut Kullanici> { self.kayitlar.iter_mut().find(|k| k.id == id) }
#     pub fn cikar(&mut self, id: u32) -> Option<Kullanici> {
#         let sira = self.kayitlar.iter().position(|k| k.id == id)?;
#         Some(self.kayitlar.remove(sira))
#     }
#     pub fn aktif_adlar(&self) -> Vec<&str> { self.kayitlar.iter().filter(|k| k.aktif).map(|k| k.ad.as_str()).collect() }
# }
fn main() {
    let mut depo = KullaniciDeposu::default();
    for (id, ad, aktif) in [(1, "Ada", true), (2, "Linus", false), (3, "Grace", true)] {
        depo.ekle(Kullanici { id, ad: ad.to_string(), aktif });
    }

    // Okumak için paylaşımlı ödünç
    if let Some(k) = depo.bul(1) {
        println!("bulundu: {} (aktif: {})", k.ad, k.aktif);
    }

    // Yerinde güncellemek için değiştirilebilir ödünç
    if let Some(k) = depo.bul_mut(2) {
        k.aktif = true;
    }
    println!("aktifler: {:?}", depo.aktif_adlar());

    // Sahipliği geri almak
    let cikan = depo.cikar(1);
    println!("çıkarılan: {:?}", cikan.map(|k| k.ad));
    println!("kalan aktifler: {:?}", depo.aktif_adlar());
}
```

Çıktı:

```text
bulundu: Ada (aktif: true)
aktifler: ["Ada", "Linus", "Grace"]
çıkarılan: Some("Ada")
kalan aktifler: ["Linus", "Grace"]
```

`bul_mut`'un verdiği ödünç `k`'nin son kullanıldığı satırda biter; hemen ardından depoyu yeniden okuyabilmemizin nedeni budur.

Ödünç verilmiş veriye bakan bir liste yaşarken depoyu değiştirmek ise derlenmez:

```rust,compile_fail,E0502
# #[derive(Debug)]
# pub struct Kullanici { pub id: u32, pub ad: String, pub aktif: bool }
# #[derive(Default)]
# pub struct KullaniciDeposu { kayitlar: Vec<Kullanici> }
# impl KullaniciDeposu {
#     pub fn cikar(&mut self, id: u32) -> Option<Kullanici> {
#         let sira = self.kayitlar.iter().position(|k| k.id == id)?;
#         Some(self.kayitlar.remove(sira))
#     }
#     pub fn aktif_adlar(&self) -> Vec<&str> { self.kayitlar.iter().filter(|k| k.aktif).map(|k| k.ad.as_str()).collect() }
# }
# fn main() {
# let mut depo = KullaniciDeposu::default();
let adlar = depo.aktif_adlar(); // depo ödünç alındı
depo.cikar(1); // depo değiştirilmek isteniyor
println!("{adlar:?}"); // ödünç hâlâ kullanılıyor
# }
```

Derleyicinin yanıtı:

```text
error[E0502]: cannot borrow `depo` as mutable because it is also borrowed as immutable
```

Bu hata bizi gerçek bir tehlikeden korur. `cikar` kaydı siler ve `Vec`'i kaydırır; `adlar[0]` artık geri verilmiş bir belleğe bakardı. C++'ta bu duruma *iterator invalidation* denir ve sonucu tanımsız davranıştır: Program bazen çöker, bazen sessizce yanlış veri okur. Çözüm sıralamaktır: Ödünç alınan sonucu önce kullanıp bitirin, sonra değiştirin.

## En sık beş hata ve çözümleri

Beş hatanın derdi aynıdır: Kimse, başkası kullanırken altından çekilen ya da değişen bir veriyle baş başa kalmasın.

### Döngüde koleksiyonu tüketmek (`E0382`)

`for x in v` yazmak `v.into_iter()` demektir ve `Vec`'i tüketir:[^into-iter]

```rust,compile_fail,E0382
let adlar = vec![String::from("Ada"), String::from("Grace")];
for ad in adlar {
    println!("{ad}");
}
println!("{} ad", adlar.len());
```

Derleyicinin yanıtı:

```text
error[E0382]: borrow of moved value: `adlar`
```

Yalnızca okuyacaksanız referans üzerinden dolaşın:

```rust
let adlar = vec![String::from("Ada"), String::from("Grace")];
for ad in &adlar {
    println!("{ad}");
}
println!("{} ad, Vec hâlâ bizim", adlar.len());
```

Çıktı:

```text
Ada
Grace
2 ad, Vec hâlâ bizim
```

### Üzerinde dolaşırken değiştirmek (`E0502`)

```rust,compile_fail,E0502
let mut sayilar = vec![1, 2, 3];
for x in &sayilar {
    if *x == 2 {
        sayilar.push(20);
    }
}
```

Derleyicinin yanıtı:

```text
error[E0502]: cannot borrow `sayilar` as mutable because it is also borrowed as immutable
```

Tehlike gerçektir: `push` yeniden tahsis yaparsa döngünün elindeki iterator geri verilmiş belleği gösterir. Çözüm, işi iki adıma bölmektir: Önce ne yapılacağına karar verin, sonra uygulayın. "Dolaşırken sil" ihtiyacı için `retain` hazırdır:[^retain]

```rust
let mut sayilar = vec![1, 2, 3];

let eklenecek: Vec<i32> = sayilar
    .iter()
    .filter(|x| **x % 2 == 0)
    .map(|x| x * 10)
    .collect();

sayilar.extend(eklenecek); // okuma bitti, artık yazılabilir
sayilar.retain(|x| *x != 1); // koşulu sağlamayanları siler
println!("{sayilar:?}");
```

Çıktı:

```text
[2, 3, 20]
```

### Aynı anda iki `&mut` (`E0499`)

```rust,compile_fail,E0499
let mut puanlar = [10, 20, 30];
let a = &mut puanlar[0];
let b = &mut puanlar[1];
std::mem::swap(a, b);
```

Derleyicinin yanıtı:

```text
error[E0499]: cannot borrow `puanlar[_]` as mutable more than once at a time
```

Ödünç denetleyicisi indekslerin farklı olduğunu kanıtlamaya çalışmaz; `0` ve `1` yerine `i` ile `j` de yazılabilirdi. "Bu iki parça çakışmıyor" bilgisini türle ifade eden metotlar vardır: `split_at_mut`, `swap`, `iter_mut`, `chunks_mut`.[^split]

```rust
let mut puanlar = [10, 20, 30];
let (sol, sag) = puanlar.split_at_mut(1); // iki ayrık &mut dilim
std::mem::swap(&mut sol[0], &mut sag[0]);
println!("{puanlar:?}");
```

Çıktı:

```text
[20, 10, 30]
```

### Ödünç verilmişken taşımak (`E0505`)

```rust,compile_fail,E0505
let baslik = String::from("rapor");
let r = &baslik;
let yeni_sahip = baslik;
println!("{r}");
```

Derleyicinin yanıtı:

```text
error[E0505]: cannot move out of `baslik` because it is borrowed
```

Referansın son kullanımını taşımadan önceye alın. İki bağımsız sahip gerçekten gerekiyorsa klonlayın:

```rust
let baslik = String::from("rapor");
let r = &baslik;
println!("önce referans: {r}"); // ödünç burada biter

let yeni_sahip = baslik; // artık taşınabilir
println!("sonra taşıma: {yeni_sahip}");
```

Çıktı:

```text
önce referans: rapor
sonra taşıma: rapor
```

### Referansın ardındaki değeri almak (`E0507`)

```rust,compile_fail,E0507
let liste = vec![String::from("bir"), String::from("iki")];
let ilk = liste[0];
```

Derleyicinin yanıtı:

```text
error[E0507]: cannot move out of index of `Vec<String>`
```

Bir alanı paylaşımlı referansın ardından almak da aynı hatayı verir:

```rust,compile_fail,E0507
struct Kullanici {
    ad: String,
}

fn ad_al(k: &Kullanici) -> String {
    k.ad
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0507]: cannot move out of `k.ad` which is behind a shared reference
```

`&` yalnızca bakma hakkıdır. İçindeki değeri alsaydınız asıl sahip boşalmış bir değerle kalır, aynı bellek iki kez geri verilirdi. Niyete göre üç seçenek vardır:

```rust
let mut liste = vec![String::from("bir"), String::from("iki"), String::from("üç")];

let bakis: &String = &liste[0]; // yalnızca bak
let kopya: String = liste[1].clone(); // bağımsız bir kopya üret
println!("bakış = {bakis}, kopya = {kopya}");

let alinan: String = liste.swap_remove(0); // koleksiyondan çıkararak al
println!("alınan = {alinan}, kalan = {liste:?}");
```

Çıktı:

```text
bakış = bir, kopya = iki
alınan = bir, kalan = ["üç", "iki"]
```

`swap_remove` silinen yere son öğeyi koyar. Bu yüzden sabit sürede çalışır, ama sırayı korumaz; sıra önemliyse `remove` kullanın.[^swap-remove]

## Değeri yerinden almak: `take` ve `replace`

Elinizde yalnızca `&mut T` varken içindeki değeri taşıyamazsınız, çünkü yeri boş bırakmak yasaktır. Çözüm, yerine geçerli başka bir değer koyarak almaktır. Standart kütüphane bunun için üç araç verir:[^take]

| Araç | Ne yapar? |
| --- | --- |
| `std::mem::take(&mut x)` | `x`'i alır, yerine `Default::default()` koyar. |
| `std::mem::replace(&mut x, y)` | `x`'i alır, yerine `y` koyar. |
| `Option::take(&mut secenek)` | `Some(v)` değerini alır, yerine `None` koyar. |

Kütüphanede raftan kitap alınca yerine "bu kitap alındı" yazan boş bir karton konması gibidir: Rafa sonradan bakan biri orada her zaman geçerli bir şey bulur.

```rust
#[derive(Default)]
pub struct Sepet {
    urunler: Vec<String>,
    kupon: Option<String>,
}

impl Sepet {
    pub fn ekle(&mut self, urun: &str) {
        self.urunler.push(urun.to_string());
    }

    pub fn kupon_tanimla(&mut self, kod: &str) {
        self.kupon = Some(kod.to_string());
    }

    // Ürünleri alır, yerine boş bir Vec bırakır.
    pub fn siparise_cevir(&mut self) -> Vec<String> {
        std::mem::take(&mut self.urunler)
    }

    // Kuponu alır, yerine None bırakır: ikinci kez kullanılamaz.
    pub fn kuponu_kullan(&mut self) -> Option<String> {
        self.kupon.take()
    }

    pub fn urun_sayisi(&self) -> usize {
        self.urunler.len()
    }
}

fn main() {
    let mut sepet = Sepet::default();
    sepet.ekle("klavye");
    sepet.ekle("fare");
    sepet.kupon_tanimla("HOSGELDIN");

    let siparis = sepet.siparise_cevir();
    println!("sipariş = {siparis:?}, sepette kalan = {}", sepet.urun_sayisi());
    println!("kupon, 1. kullanım = {:?}", sepet.kuponu_kullan());
    println!("kupon, 2. kullanım = {:?}", sepet.kuponu_kullan());
}
```

Çıktı:

```text
sipariş = ["klavye", "fare"], sepette kalan = 0
kupon, 1. kullanım = Some("HOSGELDIN")
kupon, 2. kullanım = None
```

`siparise_cevir` içinde doğrudan `self.urunler` yazılsaydı derleyici `E0507` verirdi; `clone` edip ardından `clear` çağırmak ise bütün ürünleri boş yere kopyalardı. `mem::take` ürünlere dokunmaz: El değiştiren yalnızca `Vec`'in stack'teki üç parçasıdır (işaretçi, uzunluk, kapasite).

Kuponun tek kullanımlık olması da `Option::take` sayesinde kendiliğinden sağlanır. İkinci çağrıda alınacak bir şey kalmadığı için sonuç `None` olur.

## Akılda kalması gerekenler

- Ödünç denetleyicisi bir engel değil, tasarım hakkında geri bildirimdir. `E0502` ve `E0499` çoğu zaman "bu veriyi iki yer aynı anda yönetmeye çalışıyor" der. Çözüm genellikle `clone` serpiştirmek değil, sahipliği netleştirmek ya da işi "oku, karar ver, yaz" diye sıralamaktır.
- `clone` kötü bir araç değildir; iki bağımsız sahip gerçekten gerekiyorsa doğru araçtır. Sorun, hatayı susturmak için refleksle yazılan `clone`'dur. Kendinize şunu sorun: Bu kopyaya gerçekten ihtiyacım var mı, yoksa bir `&` yeter miydi?
- Ödünç, bloğun sonunda değil son kullanımda biter.
- Bir değerin birden çok sahibi gerçekten gerekiyorsa (graf, paylaşılan yapılandırma) cevap referans değil `Rc` ya da `Arc`'tır; bunları [14. bölümde](../intermediate/14-smart-pointers.md) göreceğiz.

## Alıştırmalar

**1.** En uzun ada sahip aktif kullanıcının adını **kopyalamadan** döndüren fonksiyonu yazın: `fn en_uzun_ad(depo: &KullaniciDeposu) -> Option<&str>`. Ada, Linus ve Grace'in üçü de aktifken sonuç ne olur?

<details>
<summary>İpucu ve çözüm</summary>

`aktif_adlar()` zaten `&str` döndürüyor; `max_by_key` en uzununu bulur. Linus ile Grace beşer harflidir ve `max_by_key` eşitlik hâlinde **sonuncuyu** verir,[^maxby] bu yüzden sonuç `Some("Grace")` olur.

```rust
# pub struct Kullanici { pub ad: String, pub aktif: bool }
# pub struct KullaniciDeposu { kayitlar: Vec<Kullanici> }
# impl KullaniciDeposu {
#     pub fn aktif_adlar(&self) -> Vec<&str> { self.kayitlar.iter().filter(|k| k.aktif).map(|k| k.ad.as_str()).collect() }
# }
fn en_uzun_ad(depo: &KullaniciDeposu) -> Option<&str> {
    depo.aktif_adlar()
        .into_iter()
        .max_by_key(|ad| ad.chars().count())
}
# fn main() {
#     let kayitlar = ["Ada", "Linus", "Grace"].map(|ad| Kullanici { ad: ad.to_string(), aktif: true }).into();
#     assert_eq!(en_uzun_ad(&KullaniciDeposu { kayitlar }), Some("Grace"));
# }
```

</details>

**2.** `Sepet`'e `fn urunu_degistir(&mut self, sira: usize, yeni: String) -> Option<String>` metodunu ekleyin. Verilen sıradaki ürünü `yeni` ile değiştirip eskisini döndürsün; sıra geçersizse `None` versin. `clone` kullanmayın.

<details>
<summary>İpucu ve çözüm</summary>

`get_mut` sırayı denetler, `mem::replace` eskisini geri verir:

```rust
# struct Sepet { urunler: Vec<String> }
impl Sepet {
    fn urunu_degistir(&mut self, sira: usize, yeni: String) -> Option<String> {
        let yer = self.urunler.get_mut(sira)?;
        Some(std::mem::replace(yer, yeni))
    }
}
# fn main() {
#     let mut s = Sepet { urunler: vec![String::from("fare")] };
#     assert_eq!(s.urunu_degistir(0, String::from("klavye")).as_deref(), Some("fare"));
#     assert_eq!(s.urunu_degistir(5, String::from("x")), None);
# }
```

</details>

**3.** Aşağıdaki fonksiyon derlenmez. Hata kodunu tahmin edin, sonra iki ayrı biçimde düzeltin: biri imzayı, öteki gövdeyi değiştirsin. Hangisi çağırana daha fazla seçenek bırakır?

```rust,compile_fail,E0507
fn ilk_iki(v: &Vec<String>) -> (String, String) {
    (v[0], v[1])
}
# fn main() {}
```

<details>
<summary>İpucu ve çözüm</summary>

Hata `E0507`'dir: Paylaşımlı referansın ardındaki öğeler taşınamaz. Birinci çözüm ödünç döndürür, ikincisi klonlar:

```rust
fn ilk_iki_odunc(v: &[String]) -> (&str, &str) {
    (&v[0], &v[1])
}

fn ilk_iki_kopya(v: &[String]) -> (String, String) {
    (v[0].clone(), v[1].clone())
}
# fn main() {
#     let v = vec![String::from("a"), String::from("b")];
#     assert_eq!(ilk_iki_odunc(&v), ("a", "b"));
#     assert_eq!(ilk_iki_kopya(&v).1, "b");
# }
```

Ödünç döndüren biçim daha esnektir: Tahsis yapmaz ve kopya isteyen çağıran `.to_string()` ile kopyasını kendisi çıkarabilir.

</details>

## Özet

- Parametre türü bir sahiplik sözleşmesidir: `String` alır, `&str` bakar, `&mut String` yerinde değiştirir.
- Taşıma alan düzeyinde izlenir; kısmen taşınmış bir yapı bütün olarak kullanılamaz.
- Veri yapısı tasarımında önce tek sahibi belirleyin, dışarıya referans verin.
- Beş klasik hatanın her birinin `clone` gerektirmeyen yerleşik bir çözümü vardır.
- `&mut` ardındaki bir değeri almak için yerine başka bir değer koyun: `take`, `replace`.

Sıradaki bölüm [4. Struct'lar (Yapılar)](04-structs.md): Bu sayfadaki üç imza biçimi orada metot alıcıları (`self`, `&self`, `&mut self`) olarak yeniden karşımıza çıkacak.

## Kaynaklar

[^dilim-parametre]: The Rust Programming Language, [String Slices as Parameters](https://doc.rust-lang.org/book/ch04-03-slices.html#string-slices-as-parameters); Clippy, [`ptr_arg`](https://rust-lang.github.io/rust-clippy/stable/index.html#ptr_arg).
[^kismi]: Rust by Example, [Partial moves](https://doc.rust-lang.org/rust-by-example/scope/move/partial_move.html); hata dizini, [`E0382`](https://doc.rust-lang.org/error_codes/E0382.html).
[^move-path]: rustc 1.99.0 kaynak kodu, [`move_paths/mod.rs`, `MoveData`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_mir_dataflow/src/move_paths/mod.rs#L176); rustc geliştirme kılavuzu, [Move paths](https://rustc-dev-guide.rust-lang.org/borrow-check/moves-and-initialization/move-paths.html).
[^elision]: The Rust Reference, [Lifetime elision in functions](https://doc.rust-lang.org/reference/lifetime-elision.html#lifetime-elision-in-functions).
[^into-iter]: std, [`IntoIterator`](https://doc.rust-lang.org/std/iter/trait.IntoIterator.html); The Rust Reference, [Iterator loops](https://doc.rust-lang.org/reference/expressions/loop-expr.html#iterator-loops).
[^retain]: std, [`Vec::retain`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.retain); The Rust Programming Language, [8.1 Reading Elements of Vectors](https://doc.rust-lang.org/book/ch08-01-vectors.html#reading-elements-of-vectors).
[^split]: std, [`slice::split_at_mut`](https://doc.rust-lang.org/std/primitive.slice.html#method.split_at_mut); The Rustonomicon, [Splitting Borrows](https://doc.rust-lang.org/nomicon/borrow-splitting.html).
[^swap-remove]: std, [`Vec::swap_remove`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.swap_remove); hata dizini, [`E0507`](https://doc.rust-lang.org/error_codes/E0507.html).
[^take]: std, [`mem::take`](https://doc.rust-lang.org/std/mem/fn.take.html), [`mem::replace`](https://doc.rust-lang.org/std/mem/fn.replace.html) ve [`Option::take`](https://doc.rust-lang.org/std/option/enum.Option.html#method.take).
[^maxby]: std, [`Iterator::max_by_key`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.max_by_key).
