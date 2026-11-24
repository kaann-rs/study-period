# 4. Struct'lar (Yapılar)

Birbiriyle ilgili bilgiler çoğu zaman bir arada dolaşır: bir kullanıcının adı, e-postası ve giriş sayısı; bir dikdörtgenin eni ve boyu. Bunları ayrı ayrı değişkenlerde taşırsanız hangisinin kime ait olduğu kolayca karışır. *Struct* (yapı), ilgili değerleri adlandırılmış tek bir türde toplar; bu türe ait işlemleri de *metot* olarak yanına koymanızı sağlar.

Struct'lar [3. bölümdeki](03-ownership.md) sahiplik kurallarının günlük kodda en çok karşınıza çıktığı yerlerden biridir. Alanlar taşınır ya da kopyalanır; metotlar değeri ödünç alır ya da teslim alır. Yeni bir kural öğrenmeyeceğiz, bildiğimiz kuralları alanlara ve metotlara uygulayacağız.

Bölümde önce struct tanımlamayı ve örnek oluşturmayı, sonra eski bir örnekten yenisini kurmayı, adsız alanlı struct'ları, alanlarda referans tutmayı, metotları ve son olarak hata ayıklama çıktısını göreceğiz.

> [!NOTE]
> Bu bölümün kodu `learn/structs` dizinindedir. Depo kökünden `cargo run -p structs` komutuyla çalıştırabilirsiniz.

## Günlük hayattan bir benzetme

Bir öğrenci kartı düşünün. Üzerinde ad, sınıf ve numara gözleri vardır. Okul önce boş bir **kalıp** hazırlar: "Bu kartta şu gözler bulunur" der. Kalıp bir kez tanımlanır; ondan istediğiniz kadar kart basılır. Rust'ta kalıbın karşılığı *struct* tanımı, gözlerin karşılığı *alan* (field), doldurulmuş kartın karşılığı da *örnektir* (instance).

Kartı basan büro bilgileri kontrol eder, siz yalnızca kullanırsınız. Büro doğrulamadan kart basmıyorsa, elinize geçen her kartın geçerli olduğuna güvenebilirsiniz; bu düşünceyi [bir sonraki sayfada](04-structs-deep-dive.md) koda dökeceğiz.

Kartla yapılabilecek işler *metottur* ve kart sahibinden üç türlü şey istenebilir. "Kartını göster" denince karta yalnızca bakılır, kart sizde kalır (`&self`). "Kartındaki sınıfı güncelleyelim" denince elinize silgi alırsınız; kart yerinde değişir, yine sizde kalır (`&mut self`). "Kartını teslim et" denince kartı karşınızdakine uzatırsınız ve bir daha kullanamazsınız (`self`).

<details>
<summary>Bu bölümde geçen terimler</summary>

- **struct (yapı):** Adlandırılmış alanlardan oluşan tür; üzerinde gözler bulunan boş kart kalıbı.
- **alan (field):** Struct'ın parçalarından her biri; karttaki bir göz.
- **örnek (instance):** Struct tanımından oluşturulmuş somut değer; doldurulmuş tek bir kart.
- **kısa alan yazımı (field init shorthand):** Değişkenin adı alanın adıyla aynıysa `ad: ad` yerine `ad` yazmak.
- **struct güncelleme sözdizimi (struct update syntax):** `..eski`: "Yazmadığım alanları eski örnekten al."
- **kısmi taşıma (partial move):** Bir değerin tamamını değil, yalnızca bazı alanlarını taşımak.
- **tuple struct:** Alanlarının adı olmayan, yalnızca sırası olan struct: `Renk(0, 0, 0)`.
- **unit-like struct:** Hiç alanı olmayan struct. Veri taşımaz; çoğunlukla bir işarettir (marker).
- **`impl`:** Bir türün metotlarının ve ilişkili fonksiyonlarının yazıldığı blok.
- **metot (method):** Bir değer üzerinde çağrılan fonksiyon: `kart.goster()`.
- **`self` ve `Self`:** `self` metodun çağrıldığı değerdir; `Self`, `impl` bloğunun ait olduğu türün kısa adıdır.
- **alıcı (receiver):** Metodun ilk parametresi: `self`, `&self` ya da `&mut self`.
- **ilişkili fonksiyon (associated function):** `impl` içinde tanımlanan, türe bağlı fonksiyon. `self` almayanlar `Tur::ad()` biçiminde çağrılır.
- **yapıcı (constructor):** Yeni örnek üreten ilişkili fonksiyon; adı çoğunlukla `new` (bu kitapta `yeni`) olur.
- **`String` ve `&str`:** Sahip olunan metin ve başka yerdeki bir metne ödünç bakış.
- **`u8`, `u32`, `u64`:** Eksi değer almayan 8, 32 ve 64 bitlik tam sayı türleri.
- **kopyalama ve taşıma (copy, move):** `Copy` türler (sayılar, `&T`, öğesi `Copy` olan `Option`) kopyalanır; `String` ve `Vec` gibi türler taşınır.
- **ömür (lifetime), `'a`:** Bir referansın geçerli olduğu süreyi adlandıran etiket.
- **sarkan referans (dangling reference):** Gösterdiği veri yok olmuş referans.
- **trait:** Türlerin ortak yeteneği. `Debug`, "hata ayıklamak için yazdırılabilir" yeteneğidir.
- **türetme (derive):** Derleyiciden bir trait'in kodunu sizin yerinize üretmesini istemek.
- **makro (macro):** Kod üreten kod. `println!` ve `#[derive(...)]` birer makrodur.
- **`.into()`:** Bir değeri beklenen türe çeviren metot; bu bölümde `&str`'den `String` üretir.
- **tür güvenliği (type safety):** Bir türün yerine yanlış türün verilmesini derleyicinin engellemesi.
- **yerleşim ve dolgu (layout, padding):** Alanların bellekteki düzeni ve hizalama için aralarında bırakılan boşluk.
- **derleyici (rustc):** Kodu çalışan programa çeviren ve pek çok hatayı önceden bulan araç.

</details>

## Struct tanımlamak ve örnek oluşturmak

Bir struct tanımı alanların adını ve türünü listeler. Örnek oluştururken her alana değer verilir; alanlara nokta ile ulaşılır:[^tanim]

```rust
struct Kart {
    ad: String,
    sinif: u8,
}

fn main() {
    let mut k = Kart {
        ad: String::from("Ada"),
        sinif: 3,
    };
    println!("{}, {}. sınıf", k.ad, k.sinif);

    k.sinif = 4; // k mut olduğu için alanı değişebilir
    println!("{}, {}. sınıf", k.ad, k.sinif);
}
```

Çıktı:

```text
Ada, 3. sınıf
Ada, 4. sınıf
```

Kart olmasaydı ad ve sınıfı ayrı değişkenlerde taşır, birden çok öğrenci olduğunda hangisinin kime ait olduğunu adlandırmayla ayırmaya çalışırdınız. Alanların sırası önemli değildir; örnek oluştururken onları istediğiniz sırayla yazabilirsiniz. Ama hepsini vermek zorundasınız: Rust'ta yarım kurulmuş bir struct olmaz.

### Kısa alan yazımı

Örnekler çoğunlukla bir fonksiyonun içinde kurulur. Parametrenin adı alanın adıyla aynıysa `ad: ad` yazmak yerine `ad` demek yeter; buna *kısa alan yazımı* (field init shorthand) denir:[^kisa]

```rust
#[derive(Debug)]
struct Kullanici {
    ad: String,
    eposta: String,
    aktif: bool,
    giris: u64,
}

fn kullanici_olustur(ad: String, eposta: String) -> Kullanici {
    Kullanici {
        ad, // ad: ad yerine
        eposta,
        aktif: true,
        giris: 1,
    }
}

fn main() {
    let mut u = kullanici_olustur("Kaan".into(), "k@x.com".into());
    u.giris += 1;
    println!("{u:?}");
    println!("{} giriş: {} aktif: {}", u.ad, u.giris, u.aktif);
}
```

Çıktı:

```text
Kullanici { ad: "Kaan", eposta: "k@x.com", aktif: true, giris: 2 }
Kaan giriş: 2 aktif: true
```

Buradaki `.into()`, `&str` türündeki metni beklenen türe, yani `String`'e çevirir. Beklenen türün ne olduğunu derleyici fonksiyonun imzasından bilir.[^into] `#[derive(Debug)]` ile `{u:?}`'nin ne yaptığını bölümün sonunda göreceğiz.

### Değiştirilebilirlik alana değil, bağlamaya aittir

`mut` olmadan bağlanan bir örneğin hiçbir alanı değişmez:

```rust,compile_fail,E0594
fn main() {
    let sabit = Kart { ad: String::from("Can"), sinif: 3 };
    sabit.sinif = 4;
    println!("{}, {}. sınıf", sabit.ad, sabit.sinif);
}
# struct Kart { ad: String, sinif: u8 }
```

Derleyicinin yanıtı:

```text
error[E0594]: cannot assign to `sabit.sinif`, as `sabit` is not declared as mutable
  |
3 |     sabit.sinif = 4;
  |     ^^^^^^^^^^^^^^^ cannot assign
  |
help: consider changing this to be mutable
  |
2 |     let mut sabit = Kart { ad: String::from("Can"), sinif: 3 };
  |         +++
```

Rust'ta tek bir alanı `mut` diye işaretleyemezsiniz. Değiştirilebilirlik alanın değil, bağlamanın (ya da ödüncün) özelliğidir: Ya bütün örnek değiştirilebilir ya da hiçbir alanı.[^tanim] Bu, tek yazar kuralıyla tutarlıdır: Bir örneğe `&mut` ile ulaşan kod, görebildiği bütün alanlara yazabilir.

```rust
fn guncelle(k: &mut Kart) {
    k.ad.push_str(" II");
    k.sinif += 1;
}

fn main() {
    let mut k = Kart { ad: String::from("Ada"), sinif: 3 };
    guncelle(&mut k);

    let ad = &k.ad; // yalnızca bu alan ödünç alındı
    k.sinif += 1; // başka bir alan: çakışma yok
    println!("{ad}, {}. sınıf", k.sinif);
}
# struct Kart { ad: String, sinif: u8 }
```

Çıktı:

```text
Ada II, 5. sınıf
```

`main`'in son üç satırına dikkat edin. Aynı fonksiyonun içinde derleyici alanları birbirinden ayırt eder; `ad` ödünçteyken `sinif` değiştirilebilir. Yalnızca bir alanın, `mut` olmayan bir örnekte bile değişmesi gerekiyorsa `Cell` ve `RefCell` gibi iç değiştirilebilirlik araçları kullanılır; bunları [14. bölümde](../intermediate/14-smart-pointers.md) göreceğiz.

## Eski örnekten yenisini kurmak: `..`

Yeni bir kart doldururken "sınıfı ben yazıyorum, gerisini eski karttan al" diyebilirsiniz. Bunun yazılışı *struct güncelleme sözdizimidir* (struct update syntax): `..eski`.[^guncelle]

```rust
#[derive(Debug)]
struct Kullanici {
    ad: String,
    eposta: String,
    aktif: bool,
    giris: u64,
}
#
# fn kullanici_olustur(ad: String, eposta: String) -> Kullanici {
#     Kullanici { ad, eposta, aktif: true, giris: 1 }
# }

fn main() {
    let u1 = kullanici_olustur("Ada".into(), "a@x.com".into());
    let u2 = Kullanici {
        ad: "Ada II".into(),
        ..u1 // kalan üç alan u1'den
    };
    println!("{} / {}", u2.ad, u2.eposta);
    println!("{} {}", u1.aktif, u1.giris); // Copy alanlar u1'de de duruyor
}
```

Çıktı:

```text
Ada II / a@x.com
true 1
```

Adına karşın `..u1` bir güncelleme de bir kopya da değildir: `u1` değişmez, yeni bir değer kurulur ve **yazmadığınız** alanlar `u1`'den alınır. `Copy` olan alanlar kopyalanır, olmayanlar taşınır. Eski karttaki fotoğraf sökülüp yeni karta yapıştırılır, sayılar ise yeniden yazılır. Burada `eposta` taşınmıştır; `u1` *kısmen taşınmış* durumdadır:

```rust,compile_fail,E0382
fn main() {
    let eski = Kart { ad: String::from("Ada"), sinif: 3 };
    let yeni = Kart { sinif: 4, ..eski };
    println!("{} {}", yeni.ad, yeni.sinif);
    println!("{}", eski.sinif); // sayı kopyalandı
    println!("{}", eski.ad); // ad artık yeni kartta
}
# struct Kart { ad: String, sinif: u8 }
```

Derleyicinin yanıtı:

```text
error[E0382]: borrow of moved value: `eski.ad`
  |
3 |     let yeni = Kart { sinif: 4, ..eski };
  |                ------------------------- value moved here
...
6 |     println!("{}", eski.ad); // ad artık yeni kartta
  |                    ^^^^^^^ value borrowed here after move
  |
  = note: move occurs because `eski.ad` has type `String`, which does not implement the `Copy` trait
```

Hata yalnızca `eski.ad` için verilir; bir üstteki `eski.sinif` satırı sorunsuzdur. Gerekçe [3. bölümdekiyle](03-ownership.md) aynıdır: Aynı metin iki kartta birden sayılsaydı, kartlar atılırken ikisi de onu geri vermeye çalışırdı. Taşıma, metnin tek sahibi kalmasını sağlar.

> [!TIP]
> Eski örneği bütün hâliyle saklamanız gerekiyorsa `Copy` olmayan alanların hepsini kendiniz verin ya da `..eski.clone()` yazıp kopyanın bedelini açıkça ödeyin. İki yolu da [4.1'de](04-structs-deep-dive.md) karşılaştıracağız.

## Adsız alanlar

### Tuple struct

Bazı kartların gözlerine ad vermeye değmez: Bir renk "kırmızı, yeşil, mavi" sırasıyla üç sayıdır. *Tuple struct*, alanlarının adı değil sırası olan bir struct'tır:[^tuple]

```rust
struct Renk(u8, u8, u8);

fn boya(r: Renk) {
    println!("R={} G={} B={}", r.0, r.1, r.2); // alanlara sıra numarasıyla ulaşılır
}

fn main() {
    let siyah = Renk(0, 0, 0);
    boya(siyah);
    boya(Renk(255, 0, 0));
}
```

Çıktı:

```text
R=0 G=0 B=0
R=255 G=0 B=0
```

Sıradan bir demet (`(u8, u8, u8)`) de üç sayı taşıyabilirdi. Fark şudur: `Renk` kendi başına bir **türdür**. Aynı üç sayıyı taşıyan başka bir struct da adsız bir demet de onun yerine verilemez:

```rust,compile_fail,E0308
struct Renk(u8, u8, u8);
struct Konum(u8, u8, u8);

fn boya(r: Renk) {
    println!("{}", r.0);
}

fn main() {
    boya(Konum(255, 0, 0)); // Konum bir Renk değildir
    boya((255, 0, 0)); // adsız üçlü de değildir
}
```

Derleyicinin yanıtı:

```text
error[E0308]: mismatched types
   |
9  |     boya(Konum(255, 0, 0)); // Konum bir Renk değildir
   |     ---- ^^^^^^^^^^^^^^^^ expected `Renk`, found `Konum`
   |     |
   |     arguments to this function are incorrect

error[E0308]: mismatched types
   |
10 |     boya((255, 0, 0)); // adsız üçlü de değildir
   |     ---- ^^^^^^^^^^^ expected `Renk`, found `({integer}, {integer}, {integer})`
   |     |
   |     arguments to this function are incorrect
```

Türün adı olmasaydı renk bekleyen bir yere yanlışlıkla konum verebilirdiniz; ikisi de üç sayıdır. Anlamı türle taşıyan bu yaklaşıma *tür güvenliği* (type safety) denir.[^tur-guvenligi]

### Unit-like struct

Hiç alanı olmayan struct'a *unit-like struct* denir. Üzerinde yazı olmayan, yalnızca "ben şu cins kartım" diyen bir karta benzer ve bellekte yer kaplamaz:[^unit]

```rust
struct Birim;

fn main() {
    let _b = Birim;
    println!("{} bayt", size_of::<Birim>());
}
```

Çıktı:

```text
0 bayt
```

Veri taşımadığı hâlde bir türdür; bu yüzden ona trait uygulanabilir ve işaret (marker) olarak kullanılabilir. Trait'leri [9. bölümde](../intermediate/09-generics-traits-lifetimes.md) göreceğiz.

## Alanlar veriye sahip olur ya da onu ödünç alır

Karta bir bilgiyi iki yoldan koyabilirsiniz. Ya bilgiyi kartın üzerine yazarsınız ya da karta "bu bilgi şu defterde yazıyor" diye bir not düşersiniz. Birincisi sahipliktir: `ad: String` alanı metnin sahibidir ve metin, örnek yaşadığı sürece geçerlidir. İkincisi referanstır; defter atılırsa kart boşu gösterir. Rust bu yüzden sizden bir söz ister: "Defter, karttan önce atılmayacak." Sözün yazılışı *ömür* (lifetime) etiketidir:[^omur]

```rust
struct Alinti<'a> {
    metin: &'a str,
}

fn main() {
    let s = String::from("merhaba dünya");
    let a = Alinti { metin: &s[0..7] };
    println!("alıntı: {}", a.metin);
}
```

Çıktı:

```text
alıntı: merhaba
```

`'a`, `metin`'in gösterdiği verinin ödünç alındığı süredir; bir `Alinti<'a>` değeri bu süreden uzun kullanılamaz. Etiketi yazmazsanız derleyici sözün eksik olduğunu bildirir:

```rust,compile_fail,E0106
struct Eksik {
    ad: &str,
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0106]: missing lifetime specifier
  |
2 |     ad: &str,
  |         ^ expected named lifetime parameter
  |
help: consider introducing a named lifetime parameter
  |
1 ~ struct Eksik<'a> {
2 ~     ad: &'a str,
  |
```

Sözü verip tutmazsanız, yani veri struct'tan önce düşürülürse, derleyici bunu da yakalar:

```rust,compile_fail,E0597
fn main() {
    let a;
    {
        let s = String::from("merhaba dünya");
        a = Alinti { metin: &s[0..7] };
    } // s burada düşürülür
    println!("alıntı: {}", a.metin);
}
# struct Alinti<'a> { metin: &'a str }
```

Derleyicinin yanıtı:

```text
error[E0597]: `s` does not live long enough
  |
4 |         let s = String::from("merhaba dünya");
  |             - binding `s` declared here
5 |         a = Alinti { metin: &s[0..7] };
  |                              ^ borrowed value does not live long enough
6 |     } // s burada düşürülür
  |     - `s` dropped here while still borrowed
7 |     println!("alıntı: {}", a.metin);
  |                            ------- borrow later used here
```

Bu denetim olmasaydı `a`, geri verilmiş bir belleği veri sanıp okurdu. İlk struct'larınızda alanlara sahipli türler (`String`, `Vec<T>`) koymak çoğu zaman en sade çözümdür. Ömürleri [9. bölümde](../intermediate/09-generics-traits-lifetimes.md) ayrıntılı göreceğiz.

## Metotlar ve ilişkili fonksiyonlar

Bir türe ait fonksiyonlar `impl` bloğunun içinde tanımlanır ve hepsine *ilişkili fonksiyon* (associated function) denir.[^iliskili] Bunlardan ilk parametresi `self`, `&self` ya da `&mut self` olanlar *metottur*; bir değer üzerinde nokta ile çağrılır. Bu ilk parametreye *alıcı* (receiver) denir.[^metot] `self` almayan ilişkili fonksiyonlar ise ortada henüz bir örnek yokken yapılacak işler içindir ve türün adıyla çağrılır:

```rust
struct Kart {
    ad: String,
    sinif: u8,
}

impl Kart {
    fn yeni(ad: &str) -> Self { // self yok: Kart::yeni(...)
        Self { ad: ad.to_string(), sinif: 1 }
    }

    fn goster(&self) -> &str { // yalnızca bakar
        &self.ad
    }

    fn sinif_atla(&mut self) { // yerinde değiştirir
        self.sinif += 1;
    }

    fn teslim_et(self) -> String { // kartı alır
        self.ad
    }
}

fn main() {
    let mut k = Kart::yeni("Ada");
    k.sinif_atla();
    println!("{}, {}. sınıf", k.goster(), k.sinif);

    let ad = k.teslim_et();
    println!("{ad} kartını teslim etti");
}
```

Çıktı:

```text
Ada, 2. sınıf
Ada kartını teslim etti
```

`Self`, `impl` bloğunun ait olduğu türün kısa adıdır; burada `Kart` demektir. `yeni` gibi yeni örnek üreten ilişkili fonksiyonlara *yapıcı* (constructor) denir. Rust'ta yapıcılar için özel bir sözdizimi yoktur; `new` adı yalnızca bir gelenektir.[^iliskili] Bir türün birden çok `impl` bloğu olabilir.

### Alıcının üç biçimi

Alıcı, [3.1'de](03-ownership-deep-dive.md) gördüğümüz üç imza biçiminin metotlardaki karşılığıdır. Sahiplik kuralları aynen uygulanır:[^metot]

| Alıcı | Ne yapar? | Çağrıdan sonra değer | Ne zaman seçilir? |
| --- | --- | --- | --- |
| `&self` | Paylaşımlı ödünç alır, okur | Sizde kalır | Yalnızca okumak gerektiğinde; en yaygın biçim |
| `&mut self` | Değiştirilebilir ödünç alır | Sizde kalır, değişmiş olabilir | Yerinde değiştirmek gerektiğinde |
| `self` | Sahipliği alır (tüketir) | `Copy` değilse kullanılamaz | Değer başka bir şeye dönüşecek ya da işi bitecekse |

Dikdörtgen örneği üç biçimi bir arada gösterir:

```rust
#[derive(Debug)]
struct Dikdortgen {
    en: u32,
    boy: u32,
}

impl Dikdortgen {
    fn yeni(en: u32, boy: u32) -> Self {
        Self { en, boy }
    }

    fn alan(&self) -> u32 {
        self.en * self.boy
    }

    fn olcekle(&mut self, k: u32) {
        self.en *= k;
        self.boy *= k;
    }

    fn yok_et(self) -> (u32, u32) {
        (self.en, self.boy)
    }
}

fn main() {
    let mut d = Dikdortgen::yeni(3, 4);
    println!("alan = {}", d.alan());

    d.olcekle(2); // d'nin mut olması gerekir
    println!("ölçekli alan = {} ({d:?})", d.alan());

    let (en, boy) = d.yok_et(); // d burada tüketilir
    println!("yok edildi: {en}x{boy}");
}
```

Çıktı:

```text
alan = 12
ölçekli alan = 48 (Dikdortgen { en: 6, boy: 8 })
yok edildi: 6x8
```

Teslim edilen değer bir daha kullanılamaz:

```rust,compile_fail,E0382
fn main() {
    let d = Dikdortgen { en: 6, boy: 8 };
    let (en, boy) = d.yok_et();
    println!("yok edildi: {en}x{boy}");
    println!("{}", d.alan());
}
# struct Dikdortgen { en: u32, boy: u32 }
# impl Dikdortgen {
#     fn alan(&self) -> u32 { self.en * self.boy }
#     fn yok_et(self) -> (u32, u32) { (self.en, self.boy) }
# }
```

Derleyicinin yanıtı:

```text
error[E0382]: borrow of moved value: `d`
  |
2 |     let d = Dikdortgen { en: 6, boy: 8 };
  |         - move occurs because `d` has type `Dikdortgen`, which does not implement the `Copy` trait
3 |     let (en, boy) = d.yok_et();
  |                       -------- `d` moved due to this method call
4 |     println!("yok edildi: {en}x{boy}");
5 |     println!("{}", d.alan());
  |                    ^ value borrowed here after move
  |
note: `Dikdortgen::yok_et` takes ownership of the receiver `self`, which moves `d`
```

Mesaj "use" yerine "borrow of moved value" der, çünkü `alan` metodu `d`'yi ödünç almak ister. Son satırdaki not nedeni açıkça söyler: `yok_et` alıcının sahipliğini alır.

### Değiştiren metot, değiştirilebilir bir değer ister

`&mut self` alan bir metodu `mut` olmayan bir örnekte çağıramazsınız:[^mut-metot]

```rust,compile_fail,E0596
fn main() {
    let k = Kart { ad: String::from("Ada"), sinif: 1 };
    k.sinif_atla();
}
# struct Kart { ad: String, sinif: u8 }
# impl Kart { fn sinif_atla(&mut self) { self.sinif += 1; } }
```

Derleyicinin yanıtı:

```text
error[E0596]: cannot borrow `k` as mutable, as it is not declared as mutable
  |
3 |     k.sinif_atla();
  |     ^ cannot borrow as mutable
  |
help: consider changing this to be mutable
  |
2 |     let mut k = Kart { ad: String::from("Ada"), sinif: 1 };
  |         +++
```

Çözüm `let mut k` yazmaktır. Tersini denerseniz, yani metodun alıcısını `&self` bırakıp gövdede alanı değiştirmeye kalkarsanız bu kez `E0594` alırsınız; düzeltmesi alıcıyı `&mut self` yapmaktır:

```rust,compile_fail,E0594
impl Kart {
    fn sinif_atla(&self) {
        self.sinif += 1;
    }
}
# struct Kart { ad: String, sinif: u8 }
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0594]: cannot assign to `self.sinif`, which is behind a `&` reference
  |
3 |         self.sinif += 1;
  |         ^^^^^^^^^^^^^^^ `self` is a `&` reference, so it cannot be written to
  |
help: consider changing this to be a mutable reference
  |
2 |     fn sinif_atla(&mut self) {
  |                    +++
```

Bir metodun döndürdüğü referans, alıcının ödüncünü sürdürür. `goster` ile alınan `ad` hâlâ kullanılacaksa araya `&mut self` alan bir çağrı giremez:

```rust,compile_fail,E0502
fn main() {
    let mut k = Kart { ad: String::from("Ada"), sinif: 1 };
    let ad = k.goster();
    k.sinif_atla();
    println!("{ad}");
}
# struct Kart { ad: String, sinif: u8 }
# impl Kart {
#     fn goster(&self) -> &str { &self.ad }
#     fn sinif_atla(&mut self) { self.sinif += 1; }
# }
```

Derleyicinin yanıtı:

```text
error[E0502]: cannot borrow `k` as mutable because it is also borrowed as immutable
  |
3 |     let ad = k.goster();
  |              - immutable borrow occurs here
4 |     k.sinif_atla();
  |     ^^^^^^^^^^^^^^ mutable borrow occurs here
5 |     println!("{ad}");
  |                -- immutable borrow later used here
```

Oysa yukarıda `&k.ad` ile alınan referans yaşarken `k.sinif` değiştirilebilmişti. Fark şudur: Derleyici burada metodun gövdesine değil imzasına bakar; `goster(&self)` imzası bütün kartı ödünç alır.

### Otomatik referans alma

`alan` metodu `&self` istediği hâlde `d.alan()` yazarken `&` koymadık. Rust, nokta ile yapılan çağrıda alıcıyı metodun istediği biçime kendisi getirir: Gerekirse `&` ya da `&mut` ekler, gerekirse `*` uygular.[^autoref] Aşağıdaki üç çağrı aynı şeyi yapar:

```rust
fn main() {
    let d = Dikdortgen { en: 3, boy: 4 };
    println!("{} {} {}", d.alan(), (&d).alan(), Dikdortgen::alan(&d));
}
# struct Dikdortgen { en: u32, boy: u32 }
# impl Dikdortgen { fn alan(&self) -> u32 { self.en * self.boy } }
```

Çıktı:

```text
12 12 12
```

C ve C++'taki `.` ve `->` ayrımına bu yüzden gerek kalmaz. Otomatik ekleme yalnızca alıcı için geçerlidir; öteki parametrelerde `&` işaretini siz yazarsınız.

### Neden çoğu metot `&self` alır?

Her metot `self` alsaydı ilk çağrı nesneyi tüketirdi: Kartınızı bir kez gösterdikten sonra kartsız kalırdınız. `&self` değeri sahiplenmeden okur; aynı anda birden çok okuyucuya da izin verir.[^metot] Bu yüzden önce `&self` ile başlayın, yazmak gerekiyorsa `&mut self`'e, değer gerçekten el değiştiriyorsa `self`'e geçin.

## Hata ayıklama çıktısı: `Debug`

Bir değerin içinde o an ne olduğunu görmek istediğinizde birinin kartı gözünden gözüne okuması gerekir. Bu kodu her tür için elle yazmak sıkıcıdır ve hataya açıktır. `#[derive(Debug)]` derleyiciden onu sizin yerinize üretmesini ister:[^derive]

```rust
#[derive(Debug)]
struct Dikdortgen {
    en: u32,
    boy: u32,
}

fn main() {
    let d = Dikdortgen { en: 6, boy: 7 };
    println!("{d:?}"); // tek satır
    println!("{d:#?}"); // çok satırlı
}
```

Çıktı:

```text
Dikdortgen { en: 6, boy: 7 }
Dikdortgen {
    en: 6,
    boy: 7,
}
```

`{:?}` ve `{:#?}` geliştiriciye yöneliktir. Kullanıcıya gösterilecek metin için kullanılan `{}` ise `Display` trait'ini ister. Bir struct'ın kullanıcıya nasıl gösterileceğinin tek bir doğru yolu yoktur; bu yüzden `Display` türetilemez, elle yazılır:[^display]

```rust,compile_fail,E0277
#[derive(Debug)]
struct Kart {
    ad: String,
    sinif: u8,
}

fn main() {
    let k = Kart { ad: String::from("Ada"), sinif: 3 };
    println!("{k:?}"); // Kart { ad: "Ada", sinif: 3 }
    println!("{k}");
}
```

Derleyicinin yanıtı:

```text
error[E0277]: `Kart` doesn't implement `std::fmt::Display`
   |
10 |     println!("{k}");
   |               ^^^ `Kart` cannot be formatted with the default formatter
   |
   = note: in format strings you may be able to use `{:?}` (or `{:#?}` for pretty-print) instead
```

`#[derive(Debug)]` satırını silerseniz bu kez `{k:?}` derlenmez. Derleyici hangi satırı eklemeniz gerektiğini de söyler:

```rust,compile_fail,E0277
struct Kart {
    ad: String,
    sinif: u8,
}

fn main() {
    let k = Kart { ad: String::from("Ada"), sinif: 3 };
    println!("{k:?}");
}
```

Derleyicinin yanıtı:

```text
error[E0277]: `Kart` doesn't implement `Debug`
  |
help: consider annotating `Kart` with `#[derive(Debug)]`
  |
1 + #[derive(Debug)]
2 | struct Kart {
  |
```

> [!NOTE]
> `Debug`, `Clone` ve `PartialEq` gibi standart türetmeler derleyiciye gömülüdür (*built-in*). Kütüphanelerin sunduğu türetmeler, örneğin `serde::Serialize`, derleme sırasında çalışan yordamsal makrolardır (procedural macro). Makroları [20. bölümde](../intermediate/20-macros.md) göreceğiz.

### "Alan hiç okunmuyor" uyarısı

Bir alanı yalnızca `{:?}` ile yazdırmak onu "okunmuş" yapmaz:

```rust
#[derive(Debug)]
struct Kart {
    ad: String,
    aktif: bool,
}

fn main() {
    let k = Kart { ad: String::from("Ada"), aktif: true };
    println!("{k:?}");
}
```

Program derlenir ve çalışır, ama derleyici bir uyarı verir:

```text
warning: fields `ad` and `aktif` are never read
  |
  = note: `Kart` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis
  = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default
```

Derleyici, türetilmiş `Debug` kodunu kullanılmayan kod (*dead code*) incelemesinde bilerek yok sayar.[^dead] Böylece yalnızca hata ayıklama çıktısında görünen, programın hiç kullanmadığı alanları fark edersiniz. `learn/structs` kodunda bu alanın (orada `is_active`) ayrıca `println!` ile okunmasının nedeni budur.

## Hata kodu başvurusu

| Kod | Ne zaman görülür? | Ne yapmalı? |
| --- | --- | --- |
| `E0594` | `mut` olmayan bir örneğin ya da `&self` ardındaki bir alanın değiştirilmesi | `let mut` yazın ya da alıcıyı `&mut self` yapın. |
| `E0596` | `mut` olmayan bir örnekte `&mut self` alan metot çağrılması | `let mut` yazın. |
| `E0382` | `..eski` ile taşınan alanın ya da `self` alan metotla tüketilen değerin kullanılması | Alanı kendiniz verin, klonlayın ya da değeri tüketmeyen bir metot kullanın. |
| `E0502` | Metodun döndürdüğü referans yaşarken `&mut self` alan bir çağrı yapılması | Referansın işini bitirip sonra değiştirin. |
| `E0308` | Tuple struct yerine aynı alanlı başka bir tür ya da demet verilmesi | Doğru türü kurun. |
| `E0106` | Referans alanın ömrü yazılmadığında | `struct S<'a> { r: &'a str }` yazın ya da sahipli tür kullanın. |
| `E0597` | Referans tutan struct, gösterdiği veriden uzun yaşadığında | Veriyi daha geniş kapsamda tanımlayın ya da sahipli tür kullanın. |
| `E0277` | `{:?}` için `Debug`, `{}` için `Display` eksik olduğunda | `#[derive(Debug)]` ekleyin ya da `Display`'i elle yazın. |

Herhangi bir hata kodunun uzun açıklamasını terminalde okuyabilirsiniz:

```console
rustc --explain E0594
```

## Perde arkası: Derleyici ne yapıyor?

> [!NOTE]
> Bu başlık ilk okuyuşta atlanabilir; yukarıdaki altı başlığı anlamış olmanız yeterlidir.

**Alıcı yazımları kısaltmadır.** `self`, `&self` ve `&mut self` sırasıyla `self: Self`, `self: &Self` ve `self: &mut Self` demektir.[^alici] Sahiplik kuralları bu yüzden alıcıya aynen uygulanır: `self` tüketir, `&self` paylaşımlı, `&mut self` dışlayıcı ödünç alır. Uzun yazımı siz de kullanabilirsiniz:

```rust
struct Dikdortgen {
    en: u32,
    boy: u32,
}

impl Dikdortgen {
    fn alan(self: &Self) -> u32 { // &self ile aynı
        self.en * self.boy
    }
}

fn main() {
    println!("{}", Dikdortgen { en: 3, boy: 4 }.alan());
}
```

Çıktı:

```text
12
```

**Metot çözümleme.** `d.alan()` için derleyici bir aday alıcı türleri listesi kurar: Önce ifadenin kendi türü, ardından `*` uygulanarak elde edilen türler, en sonda da (varsa) dizinin dilime dönüşmüş hâli. Listedeki her `T` adayını `T`, `&T` ve `&mut T` sırasıyla dener ve ilk uyan metodu seçer.[^cozumleme] Seçim yapılırken değerin `mut` olup olmadığına bakılmaz; `E0596` bu yüzden "metot bulunamadı" değil, "değiştirilebilir ödünç alınamaz" der. Derleyicide adayları `probe.rs` toplar ve sınar, seçimi `confirm.rs` kesinleştirir.[^probe] Ayrıntılar [24. bölümdedir](../advanced/24-deref-methods.md).

**Bellek yerleşimi.** Varsayılan yerleşimde (`repr(Rust)`) alanların bellekteki sırası garanti değildir. Derleyici, hizalama için bırakılan boşluğu (*padding*) azaltmak üzere alanları yeniden sıralayabilir.[^yerlesim] C ile uyumlu, yazıldığı sıraya bağlı bir yerleşim gerekiyorsa `#[repr(C)]` kullanılır:

```rust
use std::mem::offset_of;

struct Esnek {
    a: u8,
    b: u32,
    c: u8,
}

#[repr(C)]
struct Sabit {
    a: u8,
    b: u32,
    c: u8,
}

fn main() {
    println!("Esnek: {} bayt", size_of::<Esnek>());
    println!(
        "Sabit: {} bayt; a={}, b={}, c={}",
        size_of::<Sabit>(),
        offset_of!(Sabit, a),
        offset_of!(Sabit, b),
        offset_of!(Sabit, c),
    );
}
```

rustc 1.99.0 ile x86-64'te örnek bir çalıştırma:

```text
Esnek: 8 bayt
Sabit: 12 bayt; a=0, b=4, c=8
```

`Sabit`'in yerleşimi belirlidir: `a`'dan sonra 3, `c`'den sonra 3 bayt boşluk kalır. `Esnek`'te derleyici `b`'yi başa alıp boşluğu 2 bayta indirir; ancak bu sonuca güvenmeyin, başka bir sürüm başka bir sıra seçebilir. Hesap `rustc_abi` crate'inin `layout.rs` dosyasında yapılır.[^layout]

**Türetme.** `#[derive(Debug)]`, derleyiciye gömülü bir türetme makrosudur ve makro açılımı aşamasında kaynağınıza bir `impl fmt::Debug for ...` bloğu ekler.[^derive-kod] Kütüphanelerin türetmeleri ise ayrı bir yordamsal makro crate'inden gelir.

**Hataların yeri.** Eksik ömür etiketi (`E0106`) isim çözümleme sırasında bulunur.[^e0106] Ömrün ihlali (`E0597`), taşınmış değer (`E0382`) ve izinsiz yazma (`E0594`, `E0596`) ise MIR üzerinde çalışan ödünç denetleyicisinin işidir.[^e0597]

## Özet

- Struct, ilgili değerleri adlandırılmış bir türde toplar. Örnek kurulurken bütün alanlara değer verilir; değiştirilebilirlik bağlamanın özelliğidir.
- `..eski`, yazmadığınız alanları eski örnekten alır; `Copy` olmayanları taşır.
- Tuple struct alanları sırayla, unit-like struct ise hiç alan olmadan tanımlanır. Her ikisi de kendi başına türdür.
- Referans tutan bir alan ömür etiketi ister; struct, gösterdiği veriden uzun yaşayamaz.
- Metodun alıcısı sahipliği belirler: `&self` bakar, `&mut self` değiştirir, `self` teslim alır. Nokta ile çağrıda `&` kendiliğinden eklenir.
- `#[derive(Debug)]` hata ayıklama çıktısını üretir; kullanıcıya dönük çıktı için `Display` elle yazılır.

Sıradaki sayfa [4.1 Struct'lar: Derinleşme](04-structs-deep-dive.md), bu araçlarla geçersiz değerin hiç kurulamadığı türler tasarlamayı gösteriyor.

## Kaynaklar

[^tanim]: The Rust Programming Language, [5.1 Defining and Instantiating Structs](https://doc.rust-lang.org/book/ch05-01-defining-structs.html#defining-and-instantiating-structs); hata dizini, [`E0594`](https://doc.rust-lang.org/error_codes/E0594.html).
[^kisa]: The Rust Programming Language, [5.1 Using the Field Init Shorthand](https://doc.rust-lang.org/book/ch05-01-defining-structs.html#using-the-field-init-shorthand); The Rust Reference, [Struct field init shorthand](https://doc.rust-lang.org/reference/expressions/struct-expr.html#struct-field-init-shorthand).
[^into]: std, [`Into`](https://doc.rust-lang.org/std/convert/trait.Into.html).
[^guncelle]: The Rust Programming Language, [5.1 Creating Instances with Struct Update Syntax](https://doc.rust-lang.org/book/ch05-01-defining-structs.html#creating-instances-with-struct-update-syntax); The Rust Reference, [Functional update syntax](https://doc.rust-lang.org/reference/expressions/struct-expr.html#functional-update-syntax).
[^tuple]: The Rust Programming Language, [5.1 Creating Different Types with Tuple Structs](https://doc.rust-lang.org/book/ch05-01-defining-structs.html#creating-different-types-with-tuple-structs).
[^tur-guvenligi]: Rust API Guidelines, [Type safety](https://rust-lang.github.io/api-guidelines/type-safety.html).
[^unit]: The Rust Programming Language, [5.1 Defining Unit-Like Structs](https://doc.rust-lang.org/book/ch05-01-defining-structs.html#defining-unit-like-structs); The Rust Reference, [Structs](https://doc.rust-lang.org/reference/items/structs.html).
[^omur]: The Rust Programming Language, [5.1 Ownership of Struct Data](https://doc.rust-lang.org/book/ch05-01-defining-structs.html#ownership-of-struct-data) ve [10.3 In Struct Definitions](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html#in-struct-definitions); hata dizini, [`E0106`](https://doc.rust-lang.org/error_codes/E0106.html) ve [`E0597`](https://doc.rust-lang.org/error_codes/E0597.html).
[^metot]: The Rust Programming Language, [5.3 Methods](https://doc.rust-lang.org/book/ch05-03-method-syntax.html#methods) ve [Method Syntax](https://doc.rust-lang.org/book/ch05-03-method-syntax.html#method-syntax).
[^iliskili]: The Rust Programming Language, [5.3 Associated Functions](https://doc.rust-lang.org/book/ch05-03-method-syntax.html#associated-functions) ve [Multiple `impl` Blocks](https://doc.rust-lang.org/book/ch05-03-method-syntax.html#multiple-impl-blocks).
[^mut-metot]: The Rust Reference, [Method-call expressions](https://doc.rust-lang.org/reference/expressions/method-call-expr.html); hata dizini, [`E0596`](https://doc.rust-lang.org/error_codes/E0596.html), [`E0594`](https://doc.rust-lang.org/error_codes/E0594.html) ve [`E0502`](https://doc.rust-lang.org/error_codes/E0502.html).
[^autoref]: The Rust Programming Language, [5.3 Where's the `->` Operator?](https://doc.rust-lang.org/book/ch05-03-method-syntax.html#wheres-the---operator).
[^derive]: The Rust Programming Language, [5.2 Adding Functionality with Derived Traits](https://doc.rust-lang.org/book/ch05-02-example-structs.html#adding-functionality-with-derived-traits); std, [`Debug`](https://doc.rust-lang.org/std/fmt/trait.Debug.html).
[^display]: std, [`Display`](https://doc.rust-lang.org/std/fmt/trait.Display.html); hata dizini, [`E0277`](https://doc.rust-lang.org/error_codes/E0277.html).
[^dead]: rustc 1.99.0 kaynak kodu, [`dead.rs`, `should_ignore_impl_item`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_passes/src/dead.rs#L438).
[^alici]: The Rust Reference, [Methods](https://doc.rust-lang.org/reference/items/associated-items.html#methods).
[^cozumleme]: The Rust Reference, [Method-call expressions](https://doc.rust-lang.org/reference/expressions/method-call-expr.html); Rust Compiler Development Guide, [Method lookup](https://rustc-dev-guide.rust-lang.org/hir-typeck/method-lookup.html).
[^probe]: rustc 1.99.0 kaynak kodu, [`probe.rs`, `pick_all_method`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_typeck/src/method/probe.rs#L1314) ve [`confirm.rs`, `confirm_method`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_typeck/src/method/confirm.rs#L60).
[^yerlesim]: The Rust Reference, [The Rust representation](https://doc.rust-lang.org/reference/type-layout.html#the-rust-representation) ve [`repr(C)` structs](https://doc.rust-lang.org/reference/type-layout.html#reprc-structs).
[^layout]: rustc 1.99.0 kaynak kodu, [`layout.rs`, `univariant`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_abi/src/layout.rs#L266) ve [`optimize_field_order`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_abi/src/layout.rs#L1098).
[^derive-kod]: The Rust Reference, [Derive: built-in](https://doc.rust-lang.org/reference/attributes/derive.html); rustc 1.99.0 kaynak kodu, [`debug.rs`, `expand_deriving_debug`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_builtin_macros/src/deriving/debug.rs#L10).
[^e0106]: rustc 1.99.0 kaynak kodu, [`late/diagnostics.rs`, `E0106`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_resolve/src/late/diagnostics.rs#L4063).
[^e0597]: rustc 1.99.0 kaynak kodu, [`borrowck_errors.rs`, `path_does_not_live_long_enough`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_borrowck/src/borrowck_errors.rs#L430).
