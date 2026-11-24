# 12.1 Closure ve Iterator'lar: Derinleşme

Önceki sayfada yakalama biçimlerini, `Fn` ailesini ve `map`, `filter`, `collect` üçlüsünü tek tek gördük. Bu sayfada hepsini gerçekçi bir işte, bir kargo şubesinin sipariş kayıtlarını işlerken birlikte kullanıyoruz: Kayıtları okuyacak, bozuk olanları eleyecek, süzecek, toplayacak ve gruplayacağız. Küçük notları (closure) bandın istasyonlarına (dönüştürücü) takarak, uzun döngüler yazmadan veriyi adım adım işlemeyi göreceksiniz.

Yol boyunca şu sorulara yanıt arayacağız:

- Hangi gövde hangi trait'i doğurur ve `move` bunu neden değiştirmez?
- Closure bir fonksiyondan nasıl döndürülür, bir yapı alanında nasıl saklanır?
- Sonu olmayan bir diziyi nasıl temsil ederiz ve `collect` hangi koleksiyonu kuracağını nereden bilir?
- En sık karşılaşılan beş ödünç hatası hangi alışkanlıktan doğar?

> [!NOTE]
> Bu sayfanın kodu `learn/closures-iterators/src/deep_dive.rs` dosyasındadır. `cargo run -p closures-iterators` ile çalıştırabilir, testlerini `cargo test -p closures-iterators` ile sınayabilirsiniz.

<details>
<summary>Bu sayfada geçen yeni terimler</summary>

- **gövde:** Closure'ın dik çizgilerden sonraki kısmı; yaptığı iş.
- **veri hattı (pipeline):** Verinin art arda adımlardan geçerek işlendiği zincir.
- **dönüş türünde `impl Fn(..)`:** "Bir closure döndürüyorum, türünün adını yazmıyorum" demek.
- **`Box<dyn Fn(..)>`:** Closure'ı heap'teki bir kutuya koyup somut türünü gizlemek.
- **tür silme (type erasure):** Asıl türü gizleyip geriye yalnızca "çağrılabilir" bilgisini bırakmak.
- **fonksiyon işaretçisi:** Hiçbir şey yakalamayan bir fonksiyonu gösteren `fn(..)` türünde değer.
- **statik gönderim, vtable:** Çağrının derleme zamanında çözülmesi; çağrının çalışma zamanında bakılan bir çizelge üzerinden yapılması ([17. bölüm](17-oop.md)).
- **geri çağrı (callback):** "Şu olunca bunu çalıştır" diye başkasına verilen closure.
- **`filter_map`:** Dönüştürüp `None` çıkanları atmak; `map` ile `filter` tek adımda.
- **`fold` ve birikeç (accumulator):** Elde bir ara sonuç tutup her öğeyle güncellemek; o ara sonuç.
- **`partition`:** Öğeleri tek geçişte uyanlar ve uymayanlar diye ikiye ayırmak.
- **`flat_map`:** Her öğeden birkaç öğe üretip hepsini tek sıraya dizmek.
- **`enumerate`, `zip`:** Sıra numarası eklemek; iki sırayı öğe öğe eşlemek.
- **`take`, `take_while`:** İlk N öğeyi almak; koşul bozulana kadar almak.
- **`windows`, `chunks`:** Bir dilimin örtüşen komşu grupları; örtüşmeyen parçaları.
- **`peekable`, `peek`:** Sıradaki öğeye, onu sıradan almadan bakabilmek.
- **`find`, `count`, `inspect`:** Uyan ilk öğeyi bulmak; saymak; geçerken değiştirmeden göz atmak.
- **`for_each`:** Her öğe için bir iş yapan tüketici.
- **`HashMap`, `BTreeSet`:** Sırası belirsiz anahtar-değer sözlüğü; tekrarsız ve sıralı küme ([7. bölüm](../basics/07-collections.md)).
- **`IntoIterator`, `FromIterator`:** "`for` ile gezilebilir" ve "`collect` ile kurulabilir" sözleri.
- **turbofish (`::<T>`):** Türü çağrı sırasında elle söylemek: `collect::<Vec<_>>()`.
- **taşma (overflow):** Sayının türüne sığmaması. `checked_add` taşarsa `None` verir.
- **`saturating_sub`:** Sıfırın altına inmeyen çıkarma; `30 - 50` yerine 0 verir.
- **NaN:** "Sayı değil" anlamına gelen özel kayan noktalı değer; sıralamayı bozar.
- **yan etki (side effect):** Değer üretmek dışında yapılan iş; ekrana ya da dosyaya yazmak gibi.
- **NLL:** Ödüncün blok sonunda değil, son kullanıldığı yerde bitmesi ([3. bölüm](../basics/03-ownership.md)).

</details>

Örneklerde hep aynı altı siparişi kullanacağız:

| No | Müşteri | Şehir | Tutar | Teslim edildi mi? |
| --- | --- | --- | --- | --- |
| 1 | Ayşe | İzmir | 250 | evet |
| 2 | Can | Ankara | 90 | hayır |
| 3 | Ayşe | İzmir | 40 | evet |
| 4 | Deniz | Bursa | 310 | evet |
| 5 | Can | Ankara | 120 | evet |
| 6 | Ece | İzmir | 75 | hayır |

## Trait'i gövde belirler, `move` yakalama biçimini

Bir görev notu hakkında iki ayrı soru sorulur ve bunlar sık karıştırılır. Birincisi: Not, kullandığı şeyi yanında mı taşıyor, yoksa masada bırakıp oradan mı bakıyor? İkincisi: O şeyle ne yapıyor; okuyor mu, değiştiriyor mu, başkasına mı veriyor? Notun kaç kez uygulanabileceğini yalnızca ikinci soru belirler. Kataloğu çantanıza koymuş olmanız (`move`) onu tek kullanımlık yapmaz; her gün açıp okuyabilirsiniz. Ama kataloğu müşteriye verirseniz, çantada taşımış olun ya da olmayın, ikinci kez veremezsiniz.

Rust'ta da öyledir: `move` yalnızca değişkenin closure'a nasıl girdiğini etkiler. Uygulanan trait'i gövde belirler.[^govde]

| Closure | Yakalama | Gövde | Trait | Kaç kez çağrılır? |
| --- | --- | --- | --- | --- |
| `move \|\| ad.chars().count()` | taşıma | okur | `Fn` | istenildiği kadar |
| `\|\| { sayac += 1; sayac }` | `&mut` | değiştirir | `FnMut` | birden çok kez, özel erişimle |
| `\|\| not` | taşıma | dışarı verir | `FnOnce` | en fazla bir kez |

```rust
fn iki_kez_cagir(f: impl Fn() -> usize) -> usize {
    f() + f()
}

fn uc_kez_cagir(mut f: impl FnMut() -> u32) -> u32 {
    f();
    f();
    f()
}

fn bir_kez_cagir(f: impl FnOnce() -> String) -> String {
    f()
}

fn main() {
    let ad = String::from("Ayşe");
    let uzunluk = move || ad.chars().count(); // move var, gövde yalnızca okuyor
    println!("move + okuma, Fn: {}", iki_kez_cagir(uzunluk));

    let mut sayac = 0;
    let artir = || {
        sayac += 1; // gövde değiştiriyor
        sayac
    };
    println!("değiştirme, FnMut: {}", uc_kez_cagir(artir));

    let not = String::from("teslim edildi");
    let ver = || not; // move yok, ama gövde değeri dışarı veriyor
    println!("dışarı taşıma, FnOnce: {}", bir_kez_cagir(ver));
}
```

Çıktı:

```text
move + okuma, Fn: 8
değiştirme, FnMut: 3
dışarı taşıma, FnOnce: teslim edildi
```

"Ayşe" dört karakterdir; iki çağrının toplamı 8 eder. Üçüncü closure'da `move` yazmadığımız hâlde `not` taşınmıştır: Gövde onu döndürdüğü için derleyicinin seçebileceği başka biçim yoktur. Böyle bir closure ikinci kez çağrılamaz:

```rust,compile_fail,E0382
fn main() {
    let not = String::from("x");
    let ver = || not;
    ver();
    ver();
}
```

Derleyicinin yanıtı:

```text
error[E0382]: use of moved value: `ver`
...
note: closure cannot be invoked more than once because it moves the variable `not` out of its environment
```

## Closure döndürmek

Şube müdürü sabah bir kural kâğıdı yazar: "200 liradan pahalı paketler ayrı rafa." Müdür odasına döner, ama kâğıt görevlide kalır ve gün boyu kullanılır. Kâğıdın işe yaraması için üstündeki 200 sayısı da kâğıtla birlikte gitmelidir. Closure döndüren fonksiyon bu müdürdür: Fonksiyon biter, closure yaşamaya devam eder; bu yüzden yakaladığını `move` ile içine alması gerekir.

```rust
#[derive(Debug, Clone, PartialEq)]
struct Siparis {
    no: u32,
    musteri: &'static str,
    sehir: &'static str,
    tutar: u32,
    teslim: bool,
}

fn en_az(esik: u32) -> impl Fn(&Siparis) -> bool {
    move |s| s.tutar >= esik
}

# const HAM_KAYITLAR: &str = "1;Ayşe;İzmir;250;evet\n2;Can;Ankara;90;hayır\nbozuk satır\n\
#                             3;Ayşe;İzmir;40;evet\n4;Deniz;Bursa;310;evet\n\
#                             5;Can;Ankara;120;evet\n6;Ece;İzmir;75;hayır";
# fn satirdan(satir: &'static str) -> Option<Siparis> {
#     let mut alanlar = satir.split(';');
#     Some(Siparis {
#         no: alanlar.next()?.parse().ok()?,
#         musteri: alanlar.next()?,
#         sehir: alanlar.next()?,
#         tutar: alanlar.next()?.parse().ok()?,
#         teslim: alanlar.next()? == "evet",
#     })
# }
# fn siparisleri_oku(ham: &'static str) -> Vec<Siparis> {
#     ham.lines().filter_map(satirdan).collect()
# }
fn main() {
    let siparisler = siparisleri_oku(HAM_KAYITLAR);

    let pahali_mi = en_az(200);
    println!("{} {}", pahali_mi(&siparisler[0]), pahali_mi(&siparisler[1]));

    let buyukler: Vec<u32> = siparisler
        .iter()
        .filter(|s| pahali_mi(s))
        .map(|s| s.no)
        .collect();
    println!("en_az(200): {buyukler:?}");
}
```

Çıktı:

```text
true false
en_az(200): [1, 4]
```

Dönüş türündeki `impl Fn(&Siparis) -> bool`, "bu trait'i uygulayan tek bir somut tür döndürüyorum" demektir. Derleyici o türü bilir, siz adını yazamazsınız. Heap'te tahsis yapılmaz ve çağrı statik gönderimle çözülür.[^dondurme] `siparisleri_oku` fonksiyonunu birazdan yazacağız.

`move` unutulursa closure, fonksiyon bitince silinecek bir değişkene bakar. Derleyici bunu kabul etmez:

```rust,compile_fail,E0373
fn bozuk(esik: u32) -> impl Fn(u32) -> bool {
    |t| t >= esik
}

fn main() {
    println!("{}", bozuk(200)(250));
}
```

Derleyicinin yanıtı:

```text
error[E0373]: closure may outlive the current function, but it borrows `esik`, which is owned by the current function
  |
2 |     |t| t >= esik
  |     ^^^      ---- `esik` is borrowed here
  |     |
  |     may outlive borrowed value `esik`
  |
note: closure is returned here
help: to force the closure to take ownership of `esik` (and any other referenced variables), use the `move` keyword
```

### Farklı closure'lar döndürmek: `Box<dyn Fn>`

Her closure'ın türü kendine özgüdür; iki closure harfi harfine aynı görünse bile türleri ayrıdır. Bu yüzden ortamını yakalayan iki farklı closure, tek bir `impl Fn` dönüş türünün arkasına sığmaz:

```rust,compile_fail,E0308
fn indirim(yuzde_mi: bool, n: u32) -> impl Fn(u32) -> u32 {
    if yuzde_mi {
        move |tutar| tutar - tutar * n / 100
    } else {
        move |tutar| tutar - n
    }
}

fn main() {
    println!("{}", indirim(true, 10)(250));
}
```

Derleyicinin yanıtı:

```text
error[E0308]: `if` and `else` have incompatible types
...
  = note: no two closures, even if identical, have the same type
  = help: consider boxing your closure and/or using it as a trait object
help: you could change the return type to be a boxed trait object
```

> [!NOTE]
> Kollardaki closure'ların hiçbiri ortamını yakalamasaydı kod derlenirdi: Yakalamayan closure'lar ortak bir `fn` işaretçisi türüne dönüşür.[^lub]

Çözüm, closure'ları dışarıdan aynı görünen zarflara koymaktır. `Box<dyn Fn(u32) -> u32>` böyle bir zarftır: İçindeki closure'ın somut türü silinir, geriye yalnızca "`u32` alıp `u32` veren, çağrılabilir bir şey" olduğu bilgisi kalır.[^kutu] Karşılığında çağrı bir vtable üzerinden yapılır; closure bir şey yakalıyorsa heap'te yer de ayrılır.

```rust
fn indirim_sec(kod: &str) -> Box<dyn Fn(u32) -> u32> {
    match kod.split_once(':') {
        Some(("yuzde", n)) => {
            // Bozuk kod indirim yok demektir; oran %100'de kırpılır.
            let oran: u32 = n.parse().unwrap_or(0).min(100);
            Box::new(move |tutar| {
                let indirim = u64::from(tutar) * u64::from(oran) / 100;
                tutar.saturating_sub(u32::try_from(indirim).unwrap_or(tutar))
            })
        }
        Some(("sabit", n)) => {
            let miktar: u32 = n.parse().unwrap_or(0);
            Box::new(move |tutar| tutar.saturating_sub(miktar))
        }
        _ => Box::new(|tutar| tutar),
    }
}

fn main() {
    let indirimli: Vec<u32> = ["yuzde:10", "sabit:300", "yok"]
        .iter()
        .map(|kod| indirim_sec(kod)(250))
        .collect();
    println!("indirim_sec: {indirimli:?}");

    // Sınır durumları
    assert_eq!(indirim_sec("yuzde:20")(200), 160);
    assert_eq!(indirim_sec("yuzde:150")(250), 0); // %100'e kırpıldı
    assert_eq!(indirim_sec("yuzde:abc")(250), 250); // bozuk kod
    assert_eq!(indirim_sec("sabit:50")(30), 0); // sıfırın altına inmez
    assert_eq!(indirim_sec("bilinmeyen")(30), 30);
    assert_eq!(indirim_sec("yuzde:50")(u32::MAX), u32::MAX - u32::MAX / 2);
}
```

Çıktı:

```text
indirim_sec: [225, 0, 250]
```

Üç ayrıntı fonksiyonu sınır durumlarında ayakta tutar. `%100`'ü aşan oran kırpılır; kırpılmasaydı indirim, tutarın kendisini geçerdi. Çarpma `u64` ile yapılır, çünkü büyük tutarlarda `tutar * oran` çarpımı `u32`'ye sığmayabilir. `saturating_sub` ise sonucu sıfırın altına düşürmez.

## Closure'ı bir alanda saklamak

Bir closure'ı yapı alanında tutmak, kural kâğıdını panoya asmak gibidir. Panoyu iki türlü yapabilirsiniz.

### Jenerik alan

Birinci yolda pano tek bir kâğıt çeşidine göre yapılır: Hızlıdır, ama başka çeşit kâğıt asılmaz. Alanın türü jenerik bir `F` olur:

```rust
struct Fiyatlayici<F: Fn(u32) -> u32> {
    kural: F,
}

impl<F: Fn(u32) -> u32> Fiyatlayici<F> {
    fn uygula(&self, tutar: u32) -> u32 {
        (self.kural)(tutar) // parantez şart
    }
}

fn main() {
    let kdv = Fiyatlayici { kural: |t| t + t / 5 };
    println!("kdv(250) = {}", kdv.uygula(250));
}
```

Çıktı:

```text
kdv(250) = 300
```

Kural derleme zamanında bellidir; ek tahsis ve dolaylama yoktur. Bedeli, her farklı closure'ın ayrı bir `Fiyatlayici<F>` türü doğurmasıdır: Ortamını yakalayan iki ayrı kuralı taşıyan fiyatlayıcıları aynı `Vec`'e koyamazsınız.

Alandaki closure'ı çağırırken parantez şarttır. `self.kural(tutar)` yazarsanız derleyici bunu bir metot çağrısı sayar ve `kural` adlı bir metot arar:[^parantez]

```rust,compile_fail,E0599
struct Fiyatlayici<F: Fn(u32) -> u32> {
    kural: F,
}

impl<F: Fn(u32) -> u32> Fiyatlayici<F> {
    fn uygula(&self, tutar: u32) -> u32 {
        self.kural(tutar)
    }
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0599]: no method named `kural` found for reference `&Fiyatlayici<F>` in the current scope
  |
7 |         self.kural(tutar)
  |              ^^^^^ field, not a method
  |
help: to call the type parameter stored in `kural`, surround the field access with parentheses
```

### Kutulanmış alan

İkinci yolda panoya aynı boy zarflar asılır: İçlerine her çeşit kural girer, karşılığında küçük bir dolaylama vardır. Kurallar çalışma zamanında eklenebilir ve hepsi aynı türde olduğu için tek bir `Vec`'te durur. Geri çağrı (callback) listelerinin ve eklenti noktalarının çoğunlukla `Box<dyn Fn..>` kullanmasının nedeni budur.

```rust
# #[derive(Debug, Clone, PartialEq)]
# struct Siparis {
#     no: u32,
#     musteri: &'static str,
#     sehir: &'static str,
#     tutar: u32,
#     teslim: bool,
# }
# const HAM_KAYITLAR: &str = "1;Ayşe;İzmir;250;evet\n2;Can;Ankara;90;hayır\nbozuk satır\n\
#                             3;Ayşe;İzmir;40;evet\n4;Deniz;Bursa;310;evet\n\
#                             5;Can;Ankara;120;evet\n6;Ece;İzmir;75;hayır";
# fn satirdan(satir: &'static str) -> Option<Siparis> {
#     let mut alanlar = satir.split(';');
#     Some(Siparis {
#         no: alanlar.next()?.parse().ok()?,
#         musteri: alanlar.next()?,
#         sehir: alanlar.next()?,
#         tutar: alanlar.next()?.parse().ok()?,
#         teslim: alanlar.next()? == "evet",
#     })
# }
# fn siparisleri_oku(ham: &'static str) -> Vec<Siparis> {
#     ham.lines().filter_map(satirdan).collect()
# }
type Kural = Box<dyn Fn(&Siparis) -> bool>;

struct KuralSeti {
    kurallar: Vec<(&'static str, Kural)>,
}

impl KuralSeti {
    fn ekle(mut self, ad: &'static str, kural: impl Fn(&Siparis) -> bool + 'static) -> Self {
        self.kurallar.push((ad, Box::new(kural)));
        self
    }

    // Sağlanmayan kuralların adlarını verir.
    fn ihlaller(&self, s: &Siparis) -> Vec<&'static str> {
        let ihlal = |(ad, kural): &(&'static str, Kural)| (!kural(s)).then_some(*ad);
        self.kurallar.iter().filter_map(ihlal).collect()
    }
}

fn main() {
    let siparisler = siparisleri_oku(HAM_KAYITLAR);
    let azami = 300;
    let kurallar = KuralSeti { kurallar: Vec::new() }
        .ekle("teslim edilmemiş", |s| s.teslim)
        .ekle("limit aşımı", move |s| s.tutar <= azami); // ortamı yakalayan kural

    for s in siparisler.iter().filter(|s| !kurallar.ihlaller(s).is_empty()) {
        println!("#{} {:?}", s.no, kurallar.ihlaller(s));
    }
}
```

Çıktı:

```text
#2 ["teslim edilmemiş"]
#4 ["limit aşımı"]
#6 ["teslim edilmemiş"]
```

`ekle`'nin imzasındaki `+ 'static` gereklidir. `Box<dyn Fn(&Siparis) -> bool>` yazıldığında trait nesnesinin varsayılan ömrü `'static`'tir;[^omur] kutuya konacak closure da kısa ömürlü bir şey ödünç almamalıdır. Sınırı kaldırırsanız derleyici tam olarak bunu ister:

```rust,compile_fail,E0310
# struct Siparis { tutar: u32 }
type Kural = Box<dyn Fn(&Siparis) -> bool>;

fn kutula(kural: impl Fn(&Siparis) -> bool) -> Kural {
    Box::new(kural)
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0310]: the parameter type `impl Fn(&Siparis) -> bool` may not live long enough
help: consider adding an explicit lifetime bound
```

## Gerçek hayattan bir örnek: sipariş veri hattı

Şubede siparişler banttan akar ve sırayla istasyonlardan geçer: Okunamayan etiketler ayrılır, yalnızca teslim edilenler bırakılır, her paketin yerine tutarı konur, sonda tutarlar toplanır. Her istasyon tek bir küçük iş yapar; zinciri yukarıdan aşağıya okuyunca ne olduğunu görürsünüz.

### Okumak ve bozukları elemek: `filter_map`

Kayıtlar noktalı virgülle ayrılmış satırlar hâlinde gelir ve aralarında bozuk bir satır vardır. `satirdan` tek bir satırı siparişe çevirir; alan eksikse ya da sayı okunamıyorsa hata vermez, `None` döner. Buradaki `?`, `Option` üzerinde çalışır: `None` görünce fonksiyondan `None` ile çıkar ([8. bölüm](../basics/08-error-handling.md)). `filter_map` de dönüştürür ve `None`'ları banttan atar; `map` ile `filter`'ın tek adımda birleşimidir.[^filter-map]

```rust
# #[derive(Debug, Clone, PartialEq)]
# struct Siparis {
#     no: u32,
#     musteri: &'static str,
#     sehir: &'static str,
#     tutar: u32,
#     teslim: bool,
# }
const HAM_KAYITLAR: &str = "1;Ayşe;İzmir;250;evet\n2;Can;Ankara;90;hayır\nbozuk satır\n\
                            3;Ayşe;İzmir;40;evet\n4;Deniz;Bursa;310;evet\n\
                            5;Can;Ankara;120;evet\n6;Ece;İzmir;75;hayır";

fn satirdan(satir: &'static str) -> Option<Siparis> {
    let mut alanlar = satir.split(';');
    Some(Siparis {
        no: alanlar.next()?.parse().ok()?,
        musteri: alanlar.next()?,
        sehir: alanlar.next()?,
        tutar: alanlar.next()?.parse().ok()?,
        teslim: alanlar.next()? == "evet",
    })
}

fn siparisleri_oku(ham: &'static str) -> Vec<Siparis> {
    ham.lines().filter_map(satirdan).collect()
}

fn main() {
    let siparisler = siparisleri_oku(HAM_KAYITLAR);
    println!("satır: {}, okunan: {}", HAM_KAYITLAR.lines().count(), siparisler.len());

    // Tutarı sayı olmayan satır elenir.
    assert_eq!(satirdan("7;Ali;Van;abc;evet"), None);
}
```

Çıktı:

```text
satır: 7, okunan: 6
```

### Süzmek, dönüştürmek, toplamak

Teslim edilen siparişlerin cirosunu hesaplayalım. Yanına aynı işi yapan döngüyü de koyalım:

```rust
# #[derive(Debug, Clone, PartialEq)]
# struct Siparis {
#     no: u32,
#     musteri: &'static str,
#     sehir: &'static str,
#     tutar: u32,
#     teslim: bool,
# }
# const HAM_KAYITLAR: &str = "1;Ayşe;İzmir;250;evet\n2;Can;Ankara;90;hayır\nbozuk satır\n\
#                             3;Ayşe;İzmir;40;evet\n4;Deniz;Bursa;310;evet\n\
#                             5;Can;Ankara;120;evet\n6;Ece;İzmir;75;hayır";
# fn satirdan(satir: &'static str) -> Option<Siparis> {
#     let mut alanlar = satir.split(';');
#     Some(Siparis {
#         no: alanlar.next()?.parse().ok()?,
#         musteri: alanlar.next()?,
#         sehir: alanlar.next()?,
#         tutar: alanlar.next()?.parse().ok()?,
#         teslim: alanlar.next()? == "evet",
#     })
# }
# fn siparisleri_oku(ham: &'static str) -> Vec<Siparis> {
#     ham.lines().filter_map(satirdan).collect()
# }
fn main() {
    let siparisler = siparisleri_oku(HAM_KAYITLAR);

    let teslim_cirosu: u32 = siparisler
        .iter()
        .filter(|s| s.teslim) // yalnızca teslim edilenler geçer
        .map(|s| s.tutar) // siparişin yerine tutarı konur
        .sum(); // bant sonu: tutarlar toplanır

    let mut dongu_cirosu = 0;
    for s in &siparisler {
        if s.teslim {
            dongu_cirosu += s.tutar;
        }
    }

    println!("zincir = {teslim_cirosu}, döngü = {dongu_cirosu}");
}
```

Çıktı:

```text
zincir = 720, döngü = 720
```

Döngüde süzme, dönüştürme ve toplama iç içe geçmiştir. Bu kadar kısa bir işte fark küçük görünür, ama adımlar arttıkça hangi satırın ne için olduğunu seçmek zorlaşır. Zincirde her adımın adı vardır.

### Tek geçişte iki sonuç: `fold`

`fold`, elinizde bir hesap pusulası tutup her pakette onu güncellemektir. Pusulaya *birikeç* (accumulator) denir. Aşağıda pusulada iki sayı vardır, adet ve toplam; `(0, 0)` ile başlar, her siparişte adet bir, toplam tutar kadar artar:[^fold]

```rust
# #[derive(Debug, Clone, PartialEq)]
# struct Siparis {
#     no: u32,
#     musteri: &'static str,
#     sehir: &'static str,
#     tutar: u32,
#     teslim: bool,
# }
# const HAM_KAYITLAR: &str = "1;Ayşe;İzmir;250;evet\n2;Can;Ankara;90;hayır\nbozuk satır\n\
#                             3;Ayşe;İzmir;40;evet\n4;Deniz;Bursa;310;evet\n\
#                             5;Can;Ankara;120;evet\n6;Ece;İzmir;75;hayır";
# fn satirdan(satir: &'static str) -> Option<Siparis> {
#     let mut alanlar = satir.split(';');
#     Some(Siparis {
#         no: alanlar.next()?.parse().ok()?,
#         musteri: alanlar.next()?,
#         sehir: alanlar.next()?,
#         tutar: alanlar.next()?.parse().ok()?,
#         teslim: alanlar.next()? == "evet",
#     })
# }
# fn siparisleri_oku(ham: &'static str) -> Vec<Siparis> {
#     ham.lines().filter_map(satirdan).collect()
# }
fn main() {
    let siparisler = siparisleri_oku(HAM_KAYITLAR);

    let (adet, toplam) = siparisler
        .iter()
        .fold((0, 0), |(a, t), s| (a + 1, t + s.tutar));

    println!("ortalama = {}", toplam.checked_div(adet).unwrap_or(0));
}
```

Çıktı:

```text
ortalama = 147
```

`sum`, `count` ve `max` aynı fikrin hazır biçimleridir: "Birikeç ile öğeyi al, yeni birikeci ver." Liste boş olsaydı `/` operatörü sıfıra bölme nedeniyle paniklerdi; `checked_div` bunun yerine `None` verir.

### Gruplamak: iterator ve `HashMap::entry`

Paketleri şehir kutularına ayırıp her kutunun üstüne toplam tutarı yazalım. `entry(sehir).or_insert(0)` ifadesi "bu şehrin kutusu yoksa içine 0 yazıp aç, sonra ver" demektir:[^entry]

```rust
use std::cmp::Reverse;
use std::collections::HashMap;

# #[derive(Debug, Clone, PartialEq)]
# struct Siparis {
#     no: u32,
#     musteri: &'static str,
#     sehir: &'static str,
#     tutar: u32,
#     teslim: bool,
# }
# const HAM_KAYITLAR: &str = "1;Ayşe;İzmir;250;evet\n2;Can;Ankara;90;hayır\nbozuk satır\n\
#                             3;Ayşe;İzmir;40;evet\n4;Deniz;Bursa;310;evet\n\
#                             5;Can;Ankara;120;evet\n6;Ece;İzmir;75;hayır";
# fn satirdan(satir: &'static str) -> Option<Siparis> {
#     let mut alanlar = satir.split(';');
#     Some(Siparis {
#         no: alanlar.next()?.parse().ok()?,
#         musteri: alanlar.next()?,
#         sehir: alanlar.next()?,
#         tutar: alanlar.next()?.parse().ok()?,
#         teslim: alanlar.next()? == "evet",
#     })
# }
# fn siparisleri_oku(ham: &'static str) -> Vec<Siparis> {
#     ham.lines().filter_map(satirdan).collect()
# }
fn sehir_cirolari(siparisler: &[Siparis]) -> Vec<(&'static str, u32)> {
    let mut gruplar: HashMap<&'static str, u32> = HashMap::new();
    for s in siparisler {
        *gruplar.entry(s.sehir).or_insert(0) += s.tutar;
    }

    let mut sirali: Vec<_> = gruplar.into_iter().collect();
    // Büyükten küçüğe; eşit cirolarda şehir adına göre.
    sirali.sort_by_key(|&(sehir, ciro)| (Reverse(ciro), sehir));
    sirali
}

fn main() {
    let siparisler = siparisleri_oku(HAM_KAYITLAR);
    println!("şehir ciroları = {:?}", sehir_cirolari(&siparisler));
}
```

Çıktı:

```text
şehir ciroları = [("İzmir", 365), ("Bursa", 310), ("Ankara", 210)]
```

`HashMap`'in gezinme sırası belirsizdir ve varsayılan ayarlarda her çalıştırmada değişebilir. Gösterilecek ya da sınanacak bir sonucu bu yüzden sıralayın ya da baştan `BTreeMap` kullanın. İkinci sıralama anahtarı da aynı nedenle vardır: O olmasaydı eşit cirolu şehirler yine `HashMap`'in rastgele sırasıyla gelirdi.

### En küçüğü ve en büyüğü bulmak

Tam sayılarda `min_by_key` yeterlidir. Kayan noktalı sayılarda ise `max` kullanılamaz:

```rust,compile_fail,E0277
fn main() {
    let en_iyi = [4.5, 3.8, 4.9].into_iter().max();
    println!("{en_iyi:?}");
}
```

Derleyicinin yanıtı:

```text
error[E0277]: the trait bound `{float}: Ord` is not satisfied
```

`max`, öğeler arasında eksiksiz bir sıralama, yani `Ord` ister. `f64` bunu uygulamaz, çünkü NaN hiçbir sayıyla, kendisiyle bile karşılaştırılamaz.[^f64] Sırayı siz verirseniz sorun kalmaz; `total_cmp` NaN'ı da kapsayan bir sıralama tanımlar:

```rust
# #[derive(Debug, Clone, PartialEq)]
# struct Siparis {
#     no: u32,
#     musteri: &'static str,
#     sehir: &'static str,
#     tutar: u32,
#     teslim: bool,
# }
# const HAM_KAYITLAR: &str = "1;Ayşe;İzmir;250;evet\n2;Can;Ankara;90;hayır\nbozuk satır\n\
#                             3;Ayşe;İzmir;40;evet\n4;Deniz;Bursa;310;evet\n\
#                             5;Can;Ankara;120;evet\n6;Ece;İzmir;75;hayır";
# fn satirdan(satir: &'static str) -> Option<Siparis> {
#     let mut alanlar = satir.split(';');
#     Some(Siparis {
#         no: alanlar.next()?.parse().ok()?,
#         musteri: alanlar.next()?,
#         sehir: alanlar.next()?,
#         tutar: alanlar.next()?.parse().ok()?,
#         teslim: alanlar.next()? == "evet",
#     })
# }
# fn siparisleri_oku(ham: &'static str) -> Vec<Siparis> {
#     ham.lines().filter_map(satirdan).collect()
# }
fn main() {
    let siparisler = siparisleri_oku(HAM_KAYITLAR);

    let en_kucuk = siparisler.iter().min_by_key(|s| s.tutar).map(|s| s.no);
    let en_iyi_puan = [4.5, 3.8, 4.9]
        .into_iter()
        .max_by(|a: &f64, b| a.total_cmp(b));

    println!("en küçük sipariş = {en_kucuk:?}, en iyi puan = {en_iyi_puan:?}");
}
```

Çıktı:

```text
en küçük sipariş = Some(3), en iyi puan = Some(4.9)
```

### İkiye ayırmak ve açmak: `partition`, `flat_map`

`partition` bandın sonuna iki sepet koyar: Koşula uyanlar birine, uymayanlar ötekine düşer. İki ayrı `filter` yazmaktan farkı, listenin tek kez gezilmesidir. `flat_map` ise her paketi açıp içindekileri tek tek banda dizer; her öğeden bir iterator üretilir, hepsi tek sıraya düzleştirilir.[^partition]

```rust
use std::collections::BTreeSet;

# #[derive(Debug, Clone, PartialEq)]
# struct Siparis {
#     no: u32,
#     musteri: &'static str,
#     sehir: &'static str,
#     tutar: u32,
#     teslim: bool,
# }
# const HAM_KAYITLAR: &str = "1;Ayşe;İzmir;250;evet\n2;Can;Ankara;90;hayır\nbozuk satır\n\
#                             3;Ayşe;İzmir;40;evet\n4;Deniz;Bursa;310;evet\n\
#                             5;Can;Ankara;120;evet\n6;Ece;İzmir;75;hayır";
# fn satirdan(satir: &'static str) -> Option<Siparis> {
#     let mut alanlar = satir.split(';');
#     Some(Siparis {
#         no: alanlar.next()?.parse().ok()?,
#         musteri: alanlar.next()?,
#         sehir: alanlar.next()?,
#         tutar: alanlar.next()?.parse().ok()?,
#         teslim: alanlar.next()? == "evet",
#     })
# }
# fn siparisleri_oku(ham: &'static str) -> Vec<Siparis> {
#     ham.lines().filter_map(satirdan).collect()
# }
fn main() {
    let siparisler = siparisleri_oku(HAM_KAYITLAR);

    let (teslim, bekleyen): (Vec<&Siparis>, Vec<&Siparis>) =
        siparisler.iter().partition(|s| s.teslim);
    println!("teslim/bekleyen = {}/{}", teslim.len(), bekleyen.len());

    let harfler: BTreeSet<char> = siparisler
        .iter()
        .flat_map(|s| s.musteri.chars().take(1))
        .collect();
    println!("müşteri baş harfleri = {harfler:?}");
}
```

Çıktı:

```text
teslim/bekleyen = 4/2
müşteri baş harfleri = {'A', 'C', 'D', 'E'}
```

`BTreeSet`'e toplamak, sonucu hem tekrarsız hem sıralı yapar: Ayşe ve Can ikişer kez geçtiği hâlde harfleri bir kez görünür.

## Konuma ve komşuya bakan araçlar

Bazı sorular tek pakete bakarak yanıtlanmaz: "Bu paket kaçıncı sırada?", "Bir öncekinden ağır mı?", "Sıradaki de aynı adrese mi gidiyor?" Bunlar için paketin sırasına ya da komşusuna bakan araçlar vardır.

`enumerate` her öğeye sıra numarası ekler; `zip` iki sırayı fermuar gibi eşler ve kısa olan bitince durur; `take_while` koşul ilk bozulduğunda durur, `filter` gibi sonuna kadar taramaz. `windows` örtüşen komşu grupları, `chunks` ise örtüşmeyen parçaları verir. Son ikisi `Iterator`'ın değil, dilimlerin metodudur: Komşuya bakabilmek için verinin bellekte yan yana durması gerekir.[^komsu]

```rust
fn main() {
    let dizi = [10, 20, 30];
    println!("{:?}", dizi.iter().enumerate().collect::<Vec<_>>());
    println!("{:?}", dizi.iter().zip(["a", "b"]).collect::<Vec<_>>());
    println!("{:?}", dizi.windows(2).collect::<Vec<_>>());
    println!("{:?}", dizi.chunks(2).collect::<Vec<_>>());
}
```

Çıktı:

```text
[(0, 10), (1, 20), (2, 30)]
[(10, "a"), (20, "b")]
[[10, 20], [20, 30]]
[[10, 20], [30]]
```

Aynı araçlar günlük ciro listesinde şöyle işe yarar:

```rust
fn main() {
    let gunluk_ciro = [120, 90, 150, 150, 80, 200];
    let hedefler = [100, 100, 160];

    // zip üç öğelik hedef listesi bitince durur.
    let tutan: Vec<usize> = gunluk_ciro
        .iter()
        .zip(hedefler)
        .enumerate()
        .filter(|(_, (ciro, hedef))| *ciro >= hedef)
        .map(|(gun, _)| gun + 1)
        .collect();
    println!("hedefi tutan günler = {tutan:?}");

    let ilk_seri: Vec<_> = gunluk_ciro.iter().take_while(|c| **c >= 90).collect();
    println!("90 altına düşene kadar = {ilk_seri:?}");

    let artis_gunleri = gunluk_ciro.windows(2).filter(|w| w[1] > w[0]).count();
    let paket_toplamlari: Vec<i32> = gunluk_ciro.chunks(4).map(|p| p.iter().sum()).collect();
    println!("artış olan gün = {artis_gunleri}, 4'lü paket toplamları = {paket_toplamlari:?}");
}
```

Çıktı:

```text
hedefi tutan günler = [1]
90 altına düşene kadar = [120, 90, 150, 150]
artış olan gün = 2, 4'lü paket toplamları = [510, 280]
```

`take_while`, 80'i görünce durur; ondan sonra gelen 200 koşula uysa da alınmaz.

### Almadan bakmak: `peekable`

Sıradan bir bantta pakete bakmak için onu almanız gerekir; aldığınız paketi geri koyamazsınız. Art arda gelen aynı kayıtları gruplarken bu bir sorundur: Grubun bittiğini ancak farklı bir kayıt görünce anlarsınız, ama o kaydı çoktan almış olursunuz ve kaybolur. `peekable`, sıradaki öğeye onu tüketmeden göz atmayı sağlar; `next_if_eq` de "eşitse al, değilse yerinde bırak" der:[^peek]

```rust
// Ardışık aynı öğeleri (öğe, adet) çiftlerine sıkıştırır.
fn ardisik_grupla<'a>(ogeler: &[&'a str]) -> Vec<(&'a str, usize)> {
    let mut akis = ogeler.iter().copied().peekable();
    let mut gruplar = Vec::new();
    while let Some(oge) = akis.next() {
        let mut adet = 1;
        while akis.next_if_eq(&oge).is_some() {
            adet += 1;
        }
        gruplar.push((oge, adet));
    }
    gruplar
}

fn main() {
    let gunluk = ["BİLGİ", "BİLGİ", "HATA", "HATA", "HATA", "BİLGİ"];
    println!("ardışık gruplar = {:?}", ardisik_grupla(&gunluk));

    // Sınırdaki öğe kaybolmaz; boş liste boş sonuç verir.
    assert_eq!(ardisik_grupla(&["a", "a", "b", "a"]), vec![("a", 2), ("b", 1), ("a", 1)]);
    assert!(ardisik_grupla(&[]).is_empty());
}
```

Çıktı:

```text
ardışık gruplar = [("BİLGİ", 2), ("HATA", 3), ("BİLGİ", 1)]
```

## Sonsuz bir iterator: Fibonacci

Önceki sayfadaki `Sayac` beşte biter. Fibonacci dizisinin ise doğal bir sonu yoktur. Sıra numarası makinesinde bütün numaraların listesi durmaz; makine yalnızca "şu an kaçtayım" bilgisini tutar ve düğmeye basıldıkça bir sonrakini üretir. Bu yüzden sonu olmayan bir diziyi de verebilir: Ne kadarının isteneceğini tüketici belirler. Sonsuz bir diziyi önceden hesaplayıp bir listeye koymak ise mümkün değildir.

```rust
struct Fibonacci {
    simdiki: u64,
    sonraki: u64,
}

impl Fibonacci {
    fn yeni() -> Self {
        Fibonacci { simdiki: 0, sonraki: 1 }
    }
}

impl Iterator for Fibonacci {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        let yeni_sonraki = self.simdiki.checked_add(self.sonraki)?;
        let cikan = std::mem::replace(&mut self.simdiki, self.sonraki);
        self.sonraki = yeni_sonraki;
        Some(cikan)
    }
}

fn main() {
    let ilk_on: Vec<u64> = Fibonacci::yeni().take(10).collect();
    println!("ilk 10 = {ilk_on:?}");

    let cift_toplam: u64 = Fibonacci::yeni()
        .take_while(|&x| x < 100)
        .filter(|x| x % 2 == 0)
        .sum();
    let bini_asan = Fibonacci::yeni().enumerate().find(|&(_, x)| x > 1000);
    println!("100'den küçük çiftlerin toplamı = {cift_toplam}, 1000'i aşan ilk = {bini_asan:?}");
    println!("50'yi aşan ilk = {:?}", Fibonacci::yeni().find(|&x| x > 50));

    let mut akis = Fibonacci::yeni();
    println!("taşmadan önce üretilen terim sayısı = {}", akis.by_ref().count());
    println!("bittikten sonra = {:?}", akis.next());
    println!("onuncu sıradaki = {:?}", Fibonacci::yeni().nth(10));
}
```

Çıktı:

```text
ilk 10 = [0, 1, 1, 2, 3, 5, 8, 13, 21, 34]
100'den küçük çiftlerin toplamı = 44, 1000'i aşan ilk = Some((17, 1597))
50'yi aşan ilk = Some(55)
taşmadan önce üretilen terim sayısı = 92
bittikten sonra = None
onuncu sıradaki = Some(55)
```

`next` üç iş yapar: Bir sonraki toplamı hesaplar, şimdiki sayıyı dışarı vermek üzere eline alır ve iki alanı bir adım ilerletir. `mem::replace`, bir yere yeni değeri koyup eskisini size verir.[^replace] Dizinin bu uygulamadaki tek bitiş koşulu `u64` taşmasıdır: `checked_add` taşma olacağı anda `None` verir, `?` de bu `None`'ı `next`'in sonucu yapar. Böylece akış paniklemeden ve sessizce yanlış sayı üretmeden kapanır; kapandıktan sonra da `None` vermeyi sürdürür. Bu yalın sürüm bir sonraki toplam taşacağı anda durduğu için, o sırada elde tutulan son iki terimi üretmez.

## Sahiplik, tembellik ve `collect`

### `into_iter` ve fazladan bir `&`

`for` döngüsü verilen değerin `IntoIterator::into_iter` metodunu çağırır; hangi uygulamanın seçileceğini döngüye verdiğiniz ifadenin türü belirler. `for x in &v`, `v.iter()` ile; `for x in &mut v`, `v.iter_mut()` ile; `for x in v`, `v.into_iter()` ile aynı işi yapar.[^into-iter]

Sık şaşırtan bir ayrıntı vardır: `iter()` öğelere referans (`&T`) verir, `filter` da closure'a öğenin referansını geçirir. Sonuçta closure `&&T` alır:

```rust
fn main() {
    let adlar = vec![String::from("ece"), String::from("can")];

    let uzunlar = adlar.iter().filter(|ad: &&String| ad.len() >= 3).count();
    // String'lerin sahipliği alınır, klonlanmadan işlenir.
    let buyuk: Vec<String> = adlar.into_iter().map(|ad| ad.to_uppercase()).collect();

    println!("uzunlar = {uzunlar}, büyük = {buyuk:?}");
}
```

Çıktı:

```text
uzunlar = 2, büyük = ["ECE", "CAN"]
```

`into_iter`'den sonra `adlar` tükenmiştir:

```rust,compile_fail,E0382
fn main() {
    let adlar = vec![String::from("ece"), String::from("can")];
    let buyuk: Vec<String> = adlar.into_iter().map(|ad| ad.to_uppercase()).collect();
    println!("{adlar:?} {buyuk:?}");
}
```

Derleyicinin yanıtı:

```text
error[E0382]: borrow of moved value: `adlar`
```

### Tembelliği görmek

`inspect`, öğelere geçerken göz atmayı sağlar. Aşağıda zincir kurulduğunda hiçbir closure çalışmaz; `take(2)` yüzünden üçüncü ve dördüncü öğe de hiç işlenmez:[^tembellik]

```rust
fn main() {
    let tembel = [1, 2, 3, 4]
        .iter()
        .inspect(|x| println!("  işleniyor: {x}"))
        .map(|x| x * 10);
    println!("zincir kuruldu (henüz işlenen yok)");

    let ilk_iki: Vec<i32> = tembel.take(2).collect();
    println!("ilk_iki = {ilk_iki:?}");
}
```

Çıktı:

```text
zincir kuruldu (henüz işlenen yok)
  işleniyor: 1
  işleniyor: 2
ilk_iki = [10, 20]
```

Tuzak da buradadır: `v.iter().map(|x| println!("{x}"));` zincirinin tüketicisi yoktur, bu yüzden hiçbir şey yazmaz ve derleyici ``unused `Map` that must be used`` uyarısını verir. Yan etki için `map` değil, `for` ya da `for_each` kullanın.

### `collect` hangi kaba toplar?

`collect` bandın sonundaki kaptır. Ama hangi kap: koli mi (`Vec`), tekrarsız ve sıralı bir raf mı (`BTreeSet`), tek parça uzun bir etiket mi (`String`)? `collect`, `FromIterator` trait'ini uygulayan her türe toplayabilir ve hangisini kuracağını yalnızca hedef türden öğrenir.[^collect] Türü söylemezseniz derleyici sizin yerinize tahmin yürütmez:

```rust,compile_fail,E0283
fn main() {
    let d = (1..=3).collect();
    println!("{d:?}");
}
```

Derleyicinin yanıtı:

```text
error[E0283]: type annotations needed
  |
2 |     let d = (1..=3).collect();
  |         ^           ------- type must be known at this point
  |
  = note: the type must implement `FromIterator<i32>`
```

Türü değişkene yazabilir ya da çağrıda turbofish ile söyleyebilirsiniz. Hedef `String` de olabilir:

```rust
fn main() {
    let kareler: Vec<u32> = (1..=4).map(|x| x * x).collect(); // değişkene tür
    let metin = (1..=4).map(|x| x.to_string()).collect::<Vec<_>>().join("-"); // turbofish
    let harfler: String = ["r", "u", "s", "t"].into_iter().collect(); // hedef String

    println!("kareler = {kareler:?}, metin = {metin}, harfler = {harfler}");
}
```

Çıktı:

```text
kareler = [1, 4, 9, 16], metin = 1-2-3-4, harfler = rust
```

## Sık yapılan beş hata

Beş hatanın da özü aynıdır: Bir not bir şeyi yakaladıysa onu elinde tutuyor demektir. Tek kullanımlık notu iki kez uygulayamazsınız; değiştiren notu "yalnızca okur" diye veremezsiniz; not bir listeye yazarken o listeyi okuyamazsınız; iki not aynı listeye aynı anda yazamaz; not, silinecek bir değişkene bakarak fonksiyondan çıkamaz.

### `FnOnce` closure'ı iki kez çağırmak

Gövde yakaladığı değeri dışarı verirse ilk çağrı closure'ın kendisini tüketir:

```rust,compile_fail,E0382
fn main() {
    let etiket = String::from("acil");
    let ver = move || etiket;
    println!("{} {}", ver(), ver());
}
```

Derleyicinin yanıtı:

```text
error[E0382]: use of moved value: `ver`
...
note: closure cannot be invoked more than once because it moves the variable `etiket` out of its environment
note: this value implements `FnOnce`, which causes it to be moved when called
```

Değeri dışarı taşımayın; kopyasını ya da referansını verin. Closure böylece `Fn` olur:[^hata-fnonce]

```rust
fn main() {
    let etiket = String::from("acil");
    let ver = || etiket.clone();
    println!("{} {}", ver(), ver());
}
```

Çıktı:

```text
acil acil
```

### `FnMut` closure'ı `Fn` beklenen yere vermek

Önceki sayfada closure'ı önce bir değişkene bağlamış ve `E0525` almıştık. Closure doğrudan çağrının içine yazıldığında derleyici onu baştan `Fn` olarak denetler; bu kez hata gövdedeki atamada görülür:

```rust,compile_fail,E0594
fn uygula_fn(f: impl Fn()) {
    f();
    f();
}

fn main() {
    let mut n = 0;
    uygula_fn(|| n += 1);
    println!("{n}");
}
```

Derleyicinin yanıtı:

```text
error[E0594]: cannot assign to `n`, as it is a captured variable in a `Fn` closure
  |
1 | fn uygula_fn(f: impl Fn()) {
  |                 --------- change this to accept `FnMut` instead of `Fn`
...
8 |     uygula_fn(|| n += 1);
  |     --------- -- ^^^^^^ cannot assign
  |     |         |
  |     |         in this closure
  |     expects `Fn` instead of `FnMut`
```

İki hata kodunun çözümü aynıdır. Alan taraf gerçekten paylaşımlı çağırmak zorunda değilse imzayı `FnMut`'a gevşetin. Kendi API'nizi yazarken kural şudur: İşinizi gören en gevşek trait'i isteyin; `FnOnce` en gevşek, `Fn` en katı olandır.[^hata-fnmut]

```rust
fn uc_kez_cagir(mut f: impl FnMut() -> u32) -> u32 {
    f();
    f();
    f()
}

fn main() {
    let mut n = 0;
    uc_kez_cagir(|| {
        n += 10;
        n
    });
    println!("n = {n}");
}
```

Çıktı:

```text
n = 30
```

### Closure `&mut` tutarken değişkeni okumak

```rust,compile_fail,E0502
fn main() {
    let mut v = vec![1, 2, 3];
    let mut ekle = |x| v.push(x);
    ekle(4);
    println!("{}", v.len());
    ekle(5);
}
```

Derleyicinin yanıtı:

```text
error[E0502]: cannot borrow `v` as immutable because it is also borrowed as mutable
  |
3 |     let mut ekle = |x| v.push(x);
  |                    --- - first borrow occurs due to use of `v` in closure
  |                    |
  |                    mutable borrow occurs here
4 |     ekle(4);
5 |     println!("{}", v.len());
  |                    ^ immutable borrow occurs here
6 |     ekle(5);
  |     ---- mutable borrow later used here
```

Ödünç, closure'ın son kullanıldığı yerde biter (NLL). Önce closure ile işinizi bitirin, sonra okuyun:[^hata-nll]

```rust
fn main() {
    let mut v = vec![1, 2, 3];
    let mut ekle = |x| v.push(x);
    ekle(4);
    ekle(5);
    println!("uzunluk = {}", v.len());
}
```

Çıktı:

```text
uzunluk = 5
```

### Aynı değişkeni değiştiren iki closure'ı canlı tutmak

```rust,compile_fail,E0499
fn main() {
    let mut v = vec![1, 2, 3];
    let mut a = || v.push(1);
    let mut b = || v.push(2);
    a();
    b();
}
```

Derleyicinin yanıtı:

```text
error[E0499]: cannot borrow `v` as mutable more than once at a time
  |
3 |     let mut a = || v.push(1);
  |                 -- - first borrow occurs due to use of `v` in closure
  |                 |
  |                 first mutable borrow occurs here
4 |     let mut b = || v.push(2);
  |                 ^^ - second borrow occurs due to use of `v` in closure
  |                 |
  |                 second mutable borrow occurs here
5 |     a();
  |     - first borrow later used here
```

Değişkeni yakalamak yerine *parametre* olarak alın. Closure o zaman hiçbir ödünç tutmaz, ödünç yalnızca çağrı süresince yaşar:[^hata-iki]

```rust
fn main() {
    let mut v = vec![1, 2, 3, 4, 5];
    let basa_ekle = |hedef: &mut Vec<i32>| hedef.insert(0, 0);
    let sona_ekle = |hedef: &mut Vec<i32>| hedef.push(6);

    basa_ekle(&mut v);
    sona_ekle(&mut v);
    println!("v = {v:?}");
}
```

Çıktı:

```text
v = [0, 1, 2, 3, 4, 5, 6]
```

### Yereli ödünç alan closure'ı döndürmek

```rust,compile_fail,E0373
fn esik_ustu() -> impl Fn(u32) -> bool {
    let esik = 10;
    |x| x > esik
}

fn main() {
    println!("{}", esik_ustu()(11));
}
```

Derleyicinin yanıtı:

```text
error[E0373]: closure may outlive the current function, but it borrows `esik`, which is owned by the current function
```

`move` ile sahipliği closure'a verin. `en_az` fonksiyonunun baştan beri yaptığı buydu:[^hata-dondurme]

```rust
fn esik_ustu() -> impl Fn(u32) -> bool {
    let esik = 10;
    move |x| x > esik
}

fn main() {
    println!("{} {}", esik_ustu()(11), esik_ustu()(10));
}
```

Çıktı:

```text
true false
```

## Akılda kalması gerekenler

- Trait'i gövde belirler, `move` yalnızca yakalama biçimini. `move` ile yakalayıp yalnızca okuyan closure hâlâ `Fn`'dir.
- Parametre alırken işinizi gören en gevşek trait'i isteyin. Döndürürken tek tür yetiyorsa `impl Fn`, yakalayan farklı closure'lar dönecekse `Box<dyn Fn>` kullanın.
- Dönüştürücü tembeldir, tüketici çalıştırır. Yan etki için `for` ya da `for_each` yazın.
- `HashMap`'in gezinme sırasına güvenmeyin; gösterilecek ya da sınanacak sonucu sıralayın.

## Alıştırmalar

Çözüme bakmadan önce deneyin. Crate'in testleri `cargo test -p closures-iterators` ile çalışır; kendi testlerinizi de yanlarına ekleyebilirsiniz.

**1.** `musteri_ozeti(&[Siparis]) -> Vec<(&'static str, u32, u32)>` fonksiyonunu yazın: Her müşteri için ad, sipariş adedi ve toplam tutarı, ada göre sıralı olarak versin. Beklenen sonuç: `[("Ayşe", 2, 290), ("Can", 2, 210), ("Deniz", 1, 310), ("Ece", 1, 75)]`.

<details>
<summary>İpucu ve çözüm</summary>

`entry(..).or_insert((0, 0))` ile tek geçiş yeterlidir. Demetler alan sırasıyla karşılaştırıldığı için `sort` sonucu ada göre dizer.

```rust
use std::collections::HashMap;

# #[derive(Debug, Clone, PartialEq)]
# struct Siparis {
#     no: u32,
#     musteri: &'static str,
#     sehir: &'static str,
#     tutar: u32,
#     teslim: bool,
# }
# const HAM_KAYITLAR: &str = "1;Ayşe;İzmir;250;evet\n2;Can;Ankara;90;hayır\nbozuk satır\n\
#                             3;Ayşe;İzmir;40;evet\n4;Deniz;Bursa;310;evet\n\
#                             5;Can;Ankara;120;evet\n6;Ece;İzmir;75;hayır";
# fn satirdan(satir: &'static str) -> Option<Siparis> {
#     let mut alanlar = satir.split(';');
#     Some(Siparis {
#         no: alanlar.next()?.parse().ok()?,
#         musteri: alanlar.next()?,
#         sehir: alanlar.next()?,
#         tutar: alanlar.next()?.parse().ok()?,
#         teslim: alanlar.next()? == "evet",
#     })
# }
# fn siparisleri_oku(ham: &'static str) -> Vec<Siparis> {
#     ham.lines().filter_map(satirdan).collect()
# }
fn musteri_ozeti(siparisler: &[Siparis]) -> Vec<(&'static str, u32, u32)> {
    let mut gruplar: HashMap<&'static str, (u32, u32)> = HashMap::new();
    for s in siparisler {
        let (adet, toplam) = gruplar.entry(s.musteri).or_insert((0, 0));
        *adet += 1;
        *toplam += s.tutar;
    }

    let mut ozet: Vec<_> = gruplar
        .into_iter()
        .map(|(ad, (adet, toplam))| (ad, adet, toplam))
        .collect();
    ozet.sort();
    ozet
}

fn main() {
    println!("{:?}", musteri_ozeti(&siparisleri_oku(HAM_KAYITLAR)));
}
```

Çıktı:

```text
[("Ayşe", 2, 290), ("Can", 2, 210), ("Deniz", 1, 310), ("Ece", 1, 75)]
```

</details>

**2.** Bir dilimi `boyut` uzunluğunda sayfalara bölen `Sayfalayici<'a, T>` adlı kendi iterator'ınızı yazın (`type Item = &'a [T]`); `chunks` kullanmayın. Altı siparişte boyut 4 için uzunlukları `[4, 2]` olan iki sayfa, boş dilimde ise hiç sayfa çıkmamalı.

<details>
<summary>İpucu ve çözüm</summary>

Alanlar `kalan: &'a [T]` ve `boyut` olsun. `split_at(boyut.min(kalan.len()))` ön parçayı ayırır (`split_at` uzunluğu aşan konumda panikler); sonra `kalan`'ı güncelleyin. `boyut == 0` durumunu da düşünün: Denetlenmezse iterator hiç ilerlemeden sonsuza dek boş sayfa verirdi.

```rust
struct Sayfalayici<'a, T> {
    kalan: &'a [T],
    boyut: usize,
}

impl<'a, T> Iterator for Sayfalayici<'a, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<&'a [T]> {
        if self.kalan.is_empty() || self.boyut == 0 {
            return None;
        }
        let (sayfa, kalan) = self.kalan.split_at(self.boyut.min(self.kalan.len()));
        self.kalan = kalan;
        Some(sayfa)
    }
}

fn main() {
    let siparis_nolari = [1, 2, 3, 4, 5, 6];
    let sayfalar = Sayfalayici { kalan: &siparis_nolari, boyut: 4 };
    println!("{:?}", sayfalar.map(|s| s.len()).collect::<Vec<_>>());

    let bos: [i32; 0] = [];
    assert_eq!(Sayfalayici { kalan: &bos, boyut: 4 }.count(), 0);
    assert_eq!(Sayfalayici { kalan: &siparis_nolari, boyut: 0 }.count(), 0);
}
```

Çıktı:

```text
[4, 2]
```

</details>

**3.** `KuralSeti`'ni `Box<dyn FnMut(&Siparis) -> bool>` tutacak biçimde değiştirin ve "aynı müşteriden ikinci sipariş" kuralını ekleyin; closure, gördüğü müşterileri bir `HashSet`'te biriktirsin. Örnek kayıtlarda bu kural yalnızca #3 ve #5 için ihlal vermelidir.

<details>
<summary>İpucu ve çözüm</summary>

Yeni kural, yakaladığı kümeyi değiştirdiği için `FnMut`'tır; onu çağırmak `&mut` erişim ister. Bu yüzden `ihlaller` artık `&mut self` almak zorundadır; `&self` bırakılırsa `E0596` alırsınız. Çağıran tarafta da `let mut kurallar` gerekir.

Ana metindeki döngü ise hem `filter`'ın closure'ında hem gövdede `ihlaller`'i çağırdığı için derlenmez: `filter` içindeki closure `kurallar`'ı `&mut` olarak yakalar ve döngü boyunca elinde tutar, gövdedeki çağrı ikinci bir `&mut` ister.

```rust,compile_fail,E0499
# #[derive(Debug, Clone, PartialEq)]
# struct Siparis {
#     no: u32,
#     musteri: &'static str,
#     sehir: &'static str,
#     tutar: u32,
#     teslim: bool,
# }
# const HAM_KAYITLAR: &str = "1;Ayşe;İzmir;250;evet\n2;Can;Ankara;90;hayır\nbozuk satır\n\
#                             3;Ayşe;İzmir;40;evet\n4;Deniz;Bursa;310;evet\n\
#                             5;Can;Ankara;120;evet\n6;Ece;İzmir;75;hayır";
# fn satirdan(satir: &'static str) -> Option<Siparis> {
#     let mut alanlar = satir.split(';');
#     Some(Siparis {
#         no: alanlar.next()?.parse().ok()?,
#         musteri: alanlar.next()?,
#         sehir: alanlar.next()?,
#         tutar: alanlar.next()?.parse().ok()?,
#         teslim: alanlar.next()? == "evet",
#     })
# }
# fn siparisleri_oku(ham: &'static str) -> Vec<Siparis> {
#     ham.lines().filter_map(satirdan).collect()
# }
# struct KuralSeti { kurallar: Vec<(&'static str, Box<dyn FnMut(&Siparis) -> bool>)> }
# impl KuralSeti {
#     fn ihlaller(&mut self, s: &Siparis) -> Vec<&'static str> {
#         self.kurallar.iter_mut().filter_map(|(ad, k)| (!k(s)).then_some(*ad)).collect()
#     }
# }
fn main() {
    let siparisler = siparisleri_oku(HAM_KAYITLAR);
    let mut kurallar = KuralSeti { kurallar: Vec::new() };
    for s in siparisler.iter().filter(|s| !kurallar.ihlaller(s).is_empty()) {
        println!("#{} {:?}", s.no, kurallar.ihlaller(s));
    }
}
```

Derleyicinin yanıtı:

```text
error[E0499]: cannot borrow `kurallar` as mutable more than once at a time
```

Derlenseydi de durum tutan kuralı aynı sipariş için iki kez tetikler ve yanlış sonuç verirdi. Döngüyü sipariş başına tek çağrıya indirmek iki sorunu birden çözer:

```rust
use std::collections::HashSet;

# #[derive(Debug, Clone, PartialEq)]
# struct Siparis {
#     no: u32,
#     musteri: &'static str,
#     sehir: &'static str,
#     tutar: u32,
#     teslim: bool,
# }
# const HAM_KAYITLAR: &str = "1;Ayşe;İzmir;250;evet\n2;Can;Ankara;90;hayır\nbozuk satır\n\
#                             3;Ayşe;İzmir;40;evet\n4;Deniz;Bursa;310;evet\n\
#                             5;Can;Ankara;120;evet\n6;Ece;İzmir;75;hayır";
# fn satirdan(satir: &'static str) -> Option<Siparis> {
#     let mut alanlar = satir.split(';');
#     Some(Siparis {
#         no: alanlar.next()?.parse().ok()?,
#         musteri: alanlar.next()?,
#         sehir: alanlar.next()?,
#         tutar: alanlar.next()?.parse().ok()?,
#         teslim: alanlar.next()? == "evet",
#     })
# }
# fn siparisleri_oku(ham: &'static str) -> Vec<Siparis> {
#     ham.lines().filter_map(satirdan).collect()
# }
type Kural = Box<dyn FnMut(&Siparis) -> bool>;

struct KuralSeti {
    kurallar: Vec<(&'static str, Kural)>,
}

impl KuralSeti {
    fn ekle(mut self, ad: &'static str, kural: impl FnMut(&Siparis) -> bool + 'static) -> Self {
        self.kurallar.push((ad, Box::new(kural)));
        self
    }

    fn ihlaller(&mut self, s: &Siparis) -> Vec<&'static str> {
        self.kurallar
            .iter_mut()
            .filter_map(|(ad, kural)| (!kural(s)).then_some(*ad))
            .collect()
    }
}

fn main() {
    let mut gorulen = HashSet::new();
    // insert, değer kümede yoksa true döner: ilk sipariş kuralı sağlar.
    let mut kurallar = KuralSeti { kurallar: Vec::new() }
        .ekle("aynı müşteriden ikinci sipariş", move |s| gorulen.insert(s.musteri));

    for s in &siparisleri_oku(HAM_KAYITLAR) {
        let ihlal = kurallar.ihlaller(s);
        if !ihlal.is_empty() {
            println!("#{} {ihlal:?}", s.no);
        }
    }
}
```

Çıktı:

```text
#3 ["aynı müşteriden ikinci sipariş"]
#5 ["aynı müşteriden ikinci sipariş"]
```

</details>

## Özet

- Closure'lar `impl Fn` ile döndürülür, jenerik alanda ya da `Box<dyn Fn>` içinde saklanır. Jenerik yol hızlı ve tek türlüdür; kutu, farklı closure'ları bir araya getirir ve küçük bir dolaylama ister.
- Veri hattı, her biri tek iş yapan adımlardan kurulur: `filter_map` okur ve eler, `fold` biriktirir, `partition` ayırır, `flat_map` açar, `peekable` almadan bakar.
- Tembellik sonsuz dizileri mümkün kılar; ne kadarının üretileceğine tüketici karar verir. `collect` ise hedef türü sizden ister.
- Sık görülen beş hatanın hepsi "closure yakaladığını elinde tutar" kuralının sonucudur.

Bu konular ilerideki bölümlere bağlanır. `Box<dyn Fn>` bir `Box`'tır; bir closure'ı birden çok sahibe vermek için `Rc<dyn Fn>`, yakalanan durumu paylaşarak değiştirmek için `Rc<RefCell<..>>` gerekir ([14. bölüm](14-smart-pointers.md)). `thread::spawn`'ın imzası ise `F: FnOnce() -> T + Send + 'static` ister; buradaki `move` ve `E0373` [15. bölümde](15-concurrency.md) sık sık karşınıza çıkacaktır.

Sıradaki bölüm [13. Cargo ve Crate'ler](13-cargo-and-crates.md).

## Kaynaklar

[^govde]: The Rust Reference, [Call traits and coercions](https://doc.rust-lang.org/reference/types/closure.html#call-traits-and-coercions); The Rust Programming Language, [Moving Captured Values Out of Closures](https://doc.rust-lang.org/book/ch13-01-closures.html#moving-captured-values-out-of-closures).
[^dondurme]: The Rust Programming Language, [Returning Closures](https://doc.rust-lang.org/book/ch20-04-advanced-functions-and-closures.html#returning-closures); The Rust Reference, [Abstract return types](https://doc.rust-lang.org/reference/types/impl-trait.html#abstract-return-types); hata dizini, [`E0373`](https://doc.rust-lang.org/error_codes/E0373.html).
[^lub]: The Rust Reference, [Least upper bound coercions](https://doc.rust-lang.org/reference/type-coercions.html#least-upper-bound-coercions) ve [Call traits and coercions](https://doc.rust-lang.org/reference/types/closure.html#call-traits-and-coercions).
[^kutu]: The Rust Programming Language, [Returning Closures](https://doc.rust-lang.org/book/ch20-04-advanced-functions-and-closures.html#returning-closures); The Rust Reference, [Trait object types](https://doc.rust-lang.org/reference/types/trait-object.html); std, [`u32::saturating_sub`](https://doc.rust-lang.org/std/primitive.u32.html#method.saturating_sub).
[^parantez]: rustc 1.99.0 kaynak kodu, [`suggest.rs`, `suggest_calling_field_as_fn`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_typeck/src/method/suggest.rs#L2825); The Rust Reference, [Field access expressions](https://doc.rust-lang.org/reference/expressions/field-expr.html#field-access-expressions); hata dizini, [`E0599`](https://doc.rust-lang.org/error_codes/E0599.html).
[^omur]: The Rust Reference, [Default trait object lifetimes](https://doc.rust-lang.org/reference/lifetime-elision.html#default-trait-object-lifetimes); hata dizini, [`E0310`](https://doc.rust-lang.org/error_codes/E0310.html).
[^filter-map]: std, [`Iterator::filter_map`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.filter_map); The Rust Programming Language, [13.2 Processing a Series of Items with Iterators](https://doc.rust-lang.org/book/ch13-02-iterators.html).
[^fold]: std, [`Iterator::fold`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.fold) ve [`u32::checked_div`](https://doc.rust-lang.org/std/primitive.u32.html#method.checked_div).
[^entry]: std, [`HashMap::entry`](https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.entry), [`HashMap::iter`](https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.iter) ve [`RandomState`](https://doc.rust-lang.org/std/hash/struct.RandomState.html).
[^f64]: std, [`Ord`](https://doc.rust-lang.org/std/cmp/trait.Ord.html) ve [`f64::total_cmp`](https://doc.rust-lang.org/std/primitive.f64.html#method.total_cmp); [`Iterator::max_by`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.max_by).
[^partition]: std, [`Iterator::partition`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.partition), [`Iterator::flat_map`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.flat_map) ve [`BTreeSet`](https://doc.rust-lang.org/std/collections/struct.BTreeSet.html).
[^komsu]: std, [`slice::windows`](https://doc.rust-lang.org/std/primitive.slice.html#method.windows), [`slice::chunks`](https://doc.rust-lang.org/std/primitive.slice.html#method.chunks), [`Iterator::zip`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.zip) ve [`Iterator::take_while`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.take_while).
[^peek]: std, [`Iterator::peekable`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.peekable) ve [`Peekable::next_if_eq`](https://doc.rust-lang.org/std/iter/struct.Peekable.html#method.next_if_eq).
[^replace]: std, [`mem::replace`](https://doc.rust-lang.org/std/mem/fn.replace.html), [`u64::checked_add`](https://doc.rust-lang.org/std/primitive.u64.html#method.checked_add) ve [Infinity](https://doc.rust-lang.org/std/iter/index.html#infinity).
[^into-iter]: std, [`for` loops and `IntoIterator`](https://doc.rust-lang.org/std/iter/index.html#for-loops-and-intoiterator) ve [`Iterator::filter`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.filter).
[^tembellik]: std, [Laziness](https://doc.rust-lang.org/std/iter/index.html#laziness) ve [`Iterator::inspect`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.inspect).
[^collect]: std, [`Iterator::collect`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.collect) ve [`FromIterator`](https://doc.rust-lang.org/std/iter/trait.FromIterator.html); hata dizini, [`E0283`](https://doc.rust-lang.org/error_codes/E0283.html).
[^hata-fnonce]: The Rust Programming Language, [Moving Captured Values Out of Closures](https://doc.rust-lang.org/book/ch13-01-closures.html#moving-captured-values-out-of-closures); rustc 1.99.0 kaynak kodu, [`borrowck_errors.rs`, `cannot_act_on_moved_value`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_borrowck/src/borrowck_errors.rs#L321).
[^hata-fnmut]: hata dizini, [`E0525`](https://doc.rust-lang.org/error_codes/E0525.html) ve [`E0594`](https://doc.rust-lang.org/error_codes/E0594.html); std, [`FnMut`](https://doc.rust-lang.org/std/ops/trait.FnMut.html).
[^hata-nll]: The Rust Programming Language, [Mutable References](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html#mutable-references); hata dizini, [`E0502`](https://doc.rust-lang.org/error_codes/E0502.html).
[^hata-iki]: hata dizini, [`E0499`](https://doc.rust-lang.org/error_codes/E0499.html); The Rust Reference, [Capture modes](https://doc.rust-lang.org/reference/types/closure.html#capture-modes).
[^hata-dondurme]: hata dizini, [`E0373`](https://doc.rust-lang.org/error_codes/E0373.html); rustc 1.99.0 kaynak kodu, [`borrowck_errors.rs`, `cannot_capture_in_long_lived_closure`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_borrowck/src/borrowck_errors.rs#L459).
