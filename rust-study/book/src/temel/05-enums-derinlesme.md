# 5.1 Enum'lar: Derinleşme

Önceki sayfada enum'un ne olduğunu, `Option`'ın null'ın yerini nasıl tuttuğunu ve `match`'in eksiksizlik denetimini gördük. Bu sayfa aynı araçları tasarım yapmak için kullanıyor. Sırasıyla dört konuya bakacağız:

- Anlamsız bir değerin yazılmasını baştan olanaksız kılmak: bayraklı struct ile enum'un karşılaştırması.
- Bir siparişin aşamalarını ve aşamalar arasındaki geçişleri tek bir `match` ile yönetmek.
- İç içe `match` yazmadan "yoksa" ve "hatalıysa" akışları kurmak: `Option` ve `Result` kombinatörleri.
- Enum'larla çalışırken en sık karşılaşılan altı hata ve çözümleri.

> [!NOTE]
> Bu sayfanın kodu `learn/enums/src/derinlesme.rs` dosyasındadır. Dosyadaki testler programla birlikte çalışmaz; onları `cargo test -p enums` ile çalıştırabilirsiniz.

<details>
<summary>Bu sayfada geçen yeni terimler</summary>

- **bayrak (flag):** Evet ya da hayır tutan alan; örneğin `odendi: bool`.
- **durum makinesi (state machine):** Bir şeyin hangi aşamalarda olabileceğini ve aşamalar arasında hangi olayla geçildiğini belirleyen tablo.
- **olay ve geçiş:** "Ödeme alındı" gibi bir haber ve o haberle bir aşamadan ötekine geçmek.
- **`Result`, `Ok`, `Err`:** "İş başarıldı, sonucu bu" (`Ok`) ya da "olmadı, nedeni bu" (`Err`) diyen enum.
- **`?` operatörü:** Hata varsa onu hemen geri döndüren, yoksa içteki değeri veren kısaltma.
- **closure:** Yerinde yazılmış isimsiz fonksiyon. `|n| n + 1` ifadesinde `n` girdidir.
- **kombinatör (combinator):** `Option` ya da `Result` kutusunu elle açmadan üzerinde işlem yapan metot.
- **`map`, `and_then`, `filter`, `ok_or`, `unwrap_or_else`:** Bu sayfadaki başlıca kombinatörler. Her birini ilgili başlıkta göreceğiz.
- **`as_deref`:** Kutudaki `String`'i almadan ona `&str` olarak bakmayı sağlayan metot.
- **`unwrap`, `expect`:** "Kesin dolu" diyerek kutuyu açan metotlar. Kutu boş çıkarsa program panikler.
- **`matches!`:** "Bu değer şu desene uyuyor mu?" sorusuna `true` ya da `false` cevabı veren makro.
- **demet (tuple), `(a, b)`:** Birden çok değeri yan yana tutan paket; iki değere birlikte bakmayı sağlar.
- **`&'static str`:** Program boyunca yaşayan metin dilimi. Tırnak içine yazdığınız her metin böyledir.
- **`saturating_sub`:** Sonucu sıfırın altına inecekse sıfırda duran çıkarma.
- **test ve `assert_eq!`:** Kodun doğru çalıştığını sınayan küçük fonksiyon ve o sınamada eşitlik isteyen makro.

</details>

## Geçersiz durumları yazılamaz kılmak

Üstünde "ödendi", "yolda" ve "takip no" kutucukları bulunan bir kargo fişi düşünün. Kutucuklar ayrı ayrı doldurulduğu için saçma bir fiş yazılabilir: ödenmemiş ama yola çıkmış bir paket. Fişe kutucuklar yerine tek bir damga basılsaydı (sepette, ödendi ya da yolda) ve takip numarası yalnızca "yolda" damgasının yanına yazılabilseydi böyle bir fiş kesilemezdi.

İlk fiş bayraklı bir struct'tır:

```rust
struct GevsekSiparis {
    odendi: bool,
    kargoda: bool,
    takip_no: Option<String>,
}

impl GevsekSiparis {
    // Kurallar türde değil, burada yaşar.
    fn tutarli_mi(&self) -> bool {
        let kargo_odemesiz = self.kargoda && !self.odendi;
        let kargo_takipsiz = self.kargoda != self.takip_no.is_some();
        !kargo_odemesiz && !kargo_takipsiz
    }
}

fn main() {
    // Ödenmemiş ama kargoda; üstelik takip numarası da yok.
    let tuhaf = GevsekSiparis { odendi: false, kargoda: true, takip_no: None };
    println!("bayraklı model derlendi, tutarlı mı? {}", tuhaf.tutarli_mi());

    let (mut kurulan, mut anlamli) = (0, 0);
    for odendi in [false, true] {
        for kargoda in [false, true] {
            for takip_no in [None, Some(String::from("TR123"))] {
                let siparis = GevsekSiparis { odendi, kargoda, takip_no };
                kurulan += 1;
                if siparis.tutarli_mi() {
                    anlamli += 1;
                }
            }
        }
    }
    println!("{kurulan} değerin {anlamli} tanesi anlamlı");
}
```

Çıktı:

```text
bayraklı model derlendi, tutarlı mı? false
8 değerin 3 tanesi anlamlı
```

İki bayrak ile numaranın varlığı 2 × 2 × 2 = 8 farklı değer kurmaya izin verir; bunların yalnızca üçü (sepette, ödendi, kargoda) anlamlıdır. Kalan beşi de derlenir. Doğru olanı ayıran kural türün içinde değil, `tutarli_mi` metodundadır; onu çağırmayı unutan her kod bozuk veriyle çalışır.

Damgalı fiş ise bir enum'dur. Takip numarası yalnızca ait olduğu varyantın yanında bulunur:

```rust
enum Durum {
    Sepette,
    Yolda { takip: String },
}
# fn main() {
let d = Durum::Yolda { takip: String::from("TR1") }; // yoldaysa takip numarası vardır
let s = Durum::Sepette; // sepetteyse takip alanı hiç yoktur
# }
```

Sepetteki bir siparişe takip numarası yazamazsınız, çünkü öyle bir alan yoktur:

```rust,compile_fail,E0559
enum Durum {
    Sepette,
    Yolda { takip: String },
}
# fn main() {
let e = Durum::Sepette { takip: String::from("TR1") };
# }
```

Derleyicinin yanıtı:

```text
error[E0559]: variant `Durum::Sepette` has no field named `takip`
```

Yoldaki bir siparişin takip numarasını da eksik bırakamazsınız:

```rust,compile_fail,E0063
enum Durum {
    Sepette,
    Yolda { takip: String },
}
# fn main() {
let f = Durum::Yolda {};
# }
```

Derleyicinin yanıtı:

```text
error[E0063]: missing field `takip` in initializer of `Durum`
```

Denetlenecek bir tutarsızlık kalmadığı için `tutarli_mi` gibi bir metoda da gerek kalmaz. Bu ilkeye *geçersiz durumları temsil edilemez kılmak* (make invalid states unrepresentable) denir. Rust API kılavuzu da anlamı `bool` ve `Option` ile değil, türlerle anlatmayı öğütler.[^tur-anlam]

## Sipariş durum makinesi

Kargo şubesinin duvarında bir yol haritası asılıdır: Sepetteki sipariş ödeme gelirse "ödendi" aşamasına geçer, ödenmiş sipariş kargoya verilirse "yolda" aşamasına geçer. Haritada çizilmemiş yol kapalıdır; sepetteki bir paket doğrudan "teslim edildi" olamaz. Harita tek bir yerde yazılı olmasaydı kurallar programın dört bir yanına dağılır, biri bir yerde kapalı yoldan paket geçirirdi.

Haritayı koda dökmenin yolu, aşamaya ve gelen habere **birlikte** bakmaktır. İkisini bir demete koyup `match` edersiniz:

```rust
#[derive(Debug)]
enum Asama {
    Sepette,
    Odendi,
    Yolda,
}

enum Haber {
    OdemeGeldi,
    KargoyaVerildi,
}

fn gec(a: Asama, h: Haber) -> Asama {
    match (a, h) {
        (Asama::Sepette, Haber::OdemeGeldi) => Asama::Odendi,
        (Asama::Odendi, Haber::KargoyaVerildi) => Asama::Yolda,
        (a, _) => a, // yol kapalı: olduğu yerde kalır
    }
}

fn main() {
    let a = gec(Asama::Sepette, Haber::OdemeGeldi);
    println!("ödeme geldi: {a:?}");
    let a = gec(a, Haber::KargoyaVerildi);
    println!("kargoya verildi: {a:?}");

    let b = gec(Asama::Sepette, Haber::KargoyaVerildi);
    println!("sepetteyken kargoya verildi: {b:?}");
}
```

Çıktı:

```text
ödeme geldi: Odendi
kargoya verildi: Yolda
sepetteyken kargoya verildi: Sepette
```

Şimdi aynı fikri gerçek bir siparişe uygulayalım.

### Durumlar ve olaylar

Her aşama, o aşamada anlamı olan bilgiyi taşır. Teslim edilmiş siparişin takip numarası vardır; iptal edilmiş siparişin nedeni ve belki iade edilecek ödemesi vardır. Dışarıdan gelen olaylar da bir enum'dur, çünkü "ne oldu?" sorusunun cevapları sayılıdır:[^varyant-verisi]

```rust
#[derive(Debug, Clone, PartialEq)]
enum SiparisDurumu {
    Sepette,
    Odendi { odeme_no: u32 },
    Kargoda { odeme_no: u32, takip_no: String },
    TeslimEdildi { takip_no: String },
    IptalEdildi { neden: String, iade: Option<u32> },
}

#[derive(Debug, Clone, PartialEq)]
enum Olay {
    OdemeAlindi { odeme_no: u32 },
    KargoyaVerildi { takip_no: String },
    TeslimEdildi,
    IptalIstendi { neden: String },
}

#[derive(Debug, PartialEq)]
struct GecersizGecis {
    durum: SiparisDurumu, // reddedilen sipariş çağırana geri verilir
    olay: Olay,
}
# fn main() {}
```

### Geçiş tablosu

`gecis` metodu haritanın kendisidir:

```rust
# #[derive(Debug, Clone, PartialEq)]
# enum SiparisDurumu {
#     Sepette,
#     Odendi { odeme_no: u32 },
#     Kargoda { odeme_no: u32, takip_no: String },
#     TeslimEdildi { takip_no: String },
#     IptalEdildi { neden: String, iade: Option<u32> },
# }
# #[derive(Debug, Clone, PartialEq)]
# enum Olay {
#     OdemeAlindi { odeme_no: u32 },
#     KargoyaVerildi { takip_no: String },
#     TeslimEdildi,
#     IptalIstendi { neden: String },
# }
# #[derive(Debug, PartialEq)]
# struct GecersizGecis { durum: SiparisDurumu, olay: Olay }
impl SiparisDurumu {
    fn gecis(self, olay: Olay) -> Result<SiparisDurumu, GecersizGecis> {
        use SiparisDurumu as D;
        match (self, olay) {
            (D::Sepette, Olay::OdemeAlindi { odeme_no }) => Ok(D::Odendi { odeme_no }),
            (D::Odendi { odeme_no }, Olay::KargoyaVerildi { takip_no }) => {
                Ok(D::Kargoda { odeme_no, takip_no })
            }
            (D::Kargoda { takip_no, .. }, Olay::TeslimEdildi) => {
                Ok(D::TeslimEdildi { takip_no })
            }
            (D::Sepette, Olay::IptalIstendi { neden }) => {
                Ok(D::IptalEdildi { neden, iade: None })
            }
            (D::Odendi { odeme_no }, Olay::IptalIstendi { neden }) => {
                Ok(D::IptalEdildi { neden, iade: Some(odeme_no) })
            }
            (durum, olay) => Err(GecersizGecis { durum, olay }),
        }
    }
}
# fn main() {}
```

`use SiparisDurumu as D` satırı, uzun tür adına bu fonksiyon içinde kısa bir ad takar; `use`'un tamamını [6. bölümde](06-modules.md) göreceğiz. Metodun dört özelliğine dikkat edin:

1. **`self` tüketilir.** Geçişten sonra eski durum yoktur; aynı siparişin hem "sepette" hem "ödendi" diyen iki kopyası ortalıkta dolaşamaz. [Struct'ların derinleşme sayfasındaki](04-structs-derinlesme.md) `pasiflestir(self)` metodu da aynı fikre dayanır.
2. **Her kol bir (durum, olay) çiftidir.** Kâğıda çizilen geçiş tablosu, olduğu gibi koda geçer.
3. **Veri taşınır, kopyalanmaz.** Desen eski varyantın `odeme_no` ve `takip_no` alanlarını çıkarır, kol bunları yeni varyanta yerleştirir; `clone` gerekmez. `..` ise varyantın geri kalan alanlarıyla ilgilenmediğinizi söyler.[^iki-nokta]
4. **Son kol listelenmeyen her çifti reddeder.** Tüketilen durum `GecersizGecis` içinde çağırana geri verildiği için sipariş kaybolmaz.

### Durumu okumak

Aynı enum üzerinde yalnızca okuyan iki metot daha yazalım. `ozet`, durumu metne çevirir; `bitti_mi`, siparişin son aşamalardan birine gelip gelmediğini söyler:

```rust
# #[derive(Debug, Clone, PartialEq)]
# enum SiparisDurumu {
#     Sepette,
#     Odendi { odeme_no: u32 },
#     Kargoda { odeme_no: u32, takip_no: String },
#     TeslimEdildi { takip_no: String },
#     IptalEdildi { neden: String, iade: Option<u32> },
# }
# #[derive(Debug, Clone, PartialEq)]
# enum Olay {
#     OdemeAlindi { odeme_no: u32 },
#     KargoyaVerildi { takip_no: String },
#     TeslimEdildi,
#     IptalIstendi { neden: String },
# }
# #[derive(Debug, PartialEq)]
# struct GecersizGecis { durum: SiparisDurumu, olay: Olay }
impl SiparisDurumu {
    fn ozet(&self) -> String {
        match self {
            SiparisDurumu::Sepette => String::from("sepette"),
            SiparisDurumu::Odendi { odeme_no } => format!("ödendi (ödeme #{odeme_no})"),
            SiparisDurumu::Kargoda { odeme_no, takip_no } => {
                format!("kargoda (ödeme #{odeme_no}, takip {takip_no})")
            }
            SiparisDurumu::TeslimEdildi { takip_no } => {
                format!("teslim edildi (takip {takip_no})")
            }
            SiparisDurumu::IptalEdildi { neden, iade } => {
                let iade = iade.map_or(String::from("iade yok"), |no| format!("ödeme #{no} iade"));
                format!("iptal edildi ({neden}; {iade})")
            }
        }
    }

    fn bitti_mi(&self) -> bool {
        matches!(
            self,
            SiparisDurumu::TeslimEdildi { .. } | SiparisDurumu::IptalEdildi { .. }
        )
    }
}

fn main() {
    let kargoda = SiparisDurumu::Kargoda {
        odeme_no: 4821,
        takip_no: String::from("TR123"),
    };
    println!("{}", kargoda.ozet());
    println!("bitti mi? {}", kargoda.bitti_mi());
}
```

Çıktı:

```text
kargoda (ödeme #4821, takip TR123)
bitti mi? false
```

`map_or`, kutu doluysa içindekini verilen closure ile dönüştürür, boşsa ilk argümanı döndürür.[^map-or] İlk argüman kutu dolu olsa da hesaplanır; hesaplaması pahalıysa `map_or_else` daha uygundur. Desen `&self` üzerinde eşleştiği için `iade` burada `&Option<u32>` türündedir. `Option<u32>` `Copy` olduğundan metot kutunun kendisini değil, kopyasını tüketir.

`matches!` ise "bu değer şu desenlerden birine uyuyor mu?" sorusunu `bool`'a çevirir; `|` desenleri "ya da" anlamında birleştirir. Makro, içinde `_ => false` kolu bulunan bir `match`'e açılır.[^matches]

### Ne zaman `_` yazmalı?

`gecis`'te bütün geri kalanı yakalayan bir kol var, `ozet`'te yok.[^catch-all] İkisi de bilinçli bir seçimdir.

`gecis` için doğru varsayılan **reddetmektir**. Yeni bir durum ya da olay eklendiğinde, siz ona açıkça bir yol çizene kadar onunla hiçbir geçişe izin verilmez.

`ozet`'te ise tersini isteriz. Yarın `IadeBekliyor` diye bir varyant eklenirse bu `match` `E0004` ile derlenmez ve derleyici sizi tam bu satıra getirir. `_ => "bilinmiyor"` yazılmış olsaydı yeni durum sessizce "bilinmiyor" diye gösterilir, hata derleme zamanından müşterinin ekranına taşınırdı. `matches!` de içinde `_ => false` taşıdığı için bu uyarıyı vermez: Yeni bir bitiş durumu eklerseniz `bitti_mi`'yi kendiniz hatırlamanız gerekir.

### Olayları işlemek

Bir olay dizisini sırayla uygulayan fonksiyon, reddedilen olayda durumu olduğu gibi sürdürür:

```rust
# #[derive(Debug, Clone, PartialEq)]
# enum SiparisDurumu {
#     Sepette,
#     Odendi { odeme_no: u32 },
#     Kargoda { odeme_no: u32, takip_no: String },
#     TeslimEdildi { takip_no: String },
#     IptalEdildi { neden: String, iade: Option<u32> },
# }
# #[derive(Debug, Clone, PartialEq)]
# enum Olay {
#     OdemeAlindi { odeme_no: u32 },
#     KargoyaVerildi { takip_no: String },
#     TeslimEdildi,
#     IptalIstendi { neden: String },
# }
# #[derive(Debug, PartialEq)]
# struct GecersizGecis { durum: SiparisDurumu, olay: Olay }
# impl SiparisDurumu {
#     fn gecis(self, olay: Olay) -> Result<SiparisDurumu, GecersizGecis> {
#         use SiparisDurumu as D;
#         match (self, olay) {
#             (D::Sepette, Olay::OdemeAlindi { odeme_no }) => Ok(D::Odendi { odeme_no }),
#             (D::Odendi { odeme_no }, Olay::KargoyaVerildi { takip_no }) => Ok(D::Kargoda { odeme_no, takip_no }),
#             (D::Kargoda { takip_no, .. }, Olay::TeslimEdildi) => Ok(D::TeslimEdildi { takip_no }),
#             (D::Sepette, Olay::IptalIstendi { neden }) => Ok(D::IptalEdildi { neden, iade: None }),
#             (D::Odendi { odeme_no }, Olay::IptalIstendi { neden }) => Ok(D::IptalEdildi { neden, iade: Some(odeme_no) }),
#             (durum, olay) => Err(GecersizGecis { durum, olay }),
#         }
#     }
#     fn ozet(&self) -> String {
#         match self {
#             SiparisDurumu::Sepette => String::from("sepette"),
#             SiparisDurumu::Odendi { odeme_no } => format!("ödendi (ödeme #{odeme_no})"),
#             SiparisDurumu::Kargoda { odeme_no, takip_no } => format!("kargoda (ödeme #{odeme_no}, takip {takip_no})"),
#             SiparisDurumu::TeslimEdildi { takip_no } => format!("teslim edildi (takip {takip_no})"),
#             SiparisDurumu::IptalEdildi { neden, iade } => {
#                 let iade = iade.map_or(String::from("iade yok"), |no| format!("ödeme #{no} iade"));
#                 format!("iptal edildi ({neden}; {iade})")
#             }
#         }
#     }
#     fn bitti_mi(&self) -> bool {
#         matches!(self, SiparisDurumu::TeslimEdildi { .. } | SiparisDurumu::IptalEdildi { .. })
#     }
# }
fn olaylari_isle(baslangic: SiparisDurumu, olaylar: Vec<Olay>) -> SiparisDurumu {
    let mut durum = baslangic;
    for olay in olaylar {
        durum = match durum.gecis(olay) {
            Ok(yeni) => {
                println!("  → {}", yeni.ozet());
                yeni
            }
            Err(GecersizGecis { durum, olay }) => {
                println!("  ret: '{}' durumunda {olay:?} işlenemez", durum.ozet());
                durum
            }
        };
    }
    durum
}

fn main() {
    let son = olaylari_isle(
        SiparisDurumu::Sepette,
        vec![
            Olay::OdemeAlindi { odeme_no: 4821 },
            Olay::KargoyaVerildi { takip_no: String::from("TR123") },
            Olay::IptalIstendi { neden: String::from("vazgeçtim") },
            Olay::TeslimEdildi,
        ],
    );
    println!("bitti mi? {}", son.bitti_mi());

    let iptal = olaylari_isle(
        SiparisDurumu::Odendi { odeme_no: 77 },
        vec![Olay::IptalIstendi { neden: String::from("yanlış ürün") }],
    );
    println!("bitti mi? {}", iptal.bitti_mi());
}
```

Çıktı:

```text
  → ödendi (ödeme #4821)
  → kargoda (ödeme #4821, takip TR123)
  ret: 'kargoda (ödeme #4821, takip TR123)' durumunda IptalIstendi { neden: "vazgeçtim" } işlenemez
  → teslim edildi (takip TR123)
bitti mi? true
  → iptal edildi (yanlış ürün; ödeme #77 iade)
bitti mi? true
```

Kargoya verilmiş siparişin iptali haritada çizilmediği için reddedilir, sipariş yoluna devam eder. Ödenmiş siparişin iptali ise iade doğurur; sepetteyken iptal edilen siparişte iade edilecek bir ödeme yoktur.

`gecis` bir `Result` döndürdüğü için geçişler zincirlenebilir. `and_then`, önceki adım başarılıysa sonrakini çalıştırır, başarısızsa hatayı olduğu gibi aktarır:[^result-and-then]

```rust
# #[derive(Debug, Clone, PartialEq)]
# enum SiparisDurumu {
#     Sepette,
#     Odendi { odeme_no: u32 },
#     Kargoda { odeme_no: u32, takip_no: String },
#     TeslimEdildi { takip_no: String },
#     IptalEdildi { neden: String, iade: Option<u32> },
# }
# #[derive(Debug, Clone, PartialEq)]
# enum Olay {
#     OdemeAlindi { odeme_no: u32 },
#     KargoyaVerildi { takip_no: String },
#     TeslimEdildi,
#     IptalIstendi { neden: String },
# }
# #[derive(Debug, PartialEq)]
# struct GecersizGecis { durum: SiparisDurumu, olay: Olay }
# impl SiparisDurumu {
#     fn gecis(self, olay: Olay) -> Result<SiparisDurumu, GecersizGecis> {
#         use SiparisDurumu as D;
#         match (self, olay) {
#             (D::Sepette, Olay::OdemeAlindi { odeme_no }) => Ok(D::Odendi { odeme_no }),
#             (D::Odendi { odeme_no }, Olay::KargoyaVerildi { takip_no }) => Ok(D::Kargoda { odeme_no, takip_no }),
#             (D::Kargoda { takip_no, .. }, Olay::TeslimEdildi) => Ok(D::TeslimEdildi { takip_no }),
#             (D::Sepette, Olay::IptalIstendi { neden }) => Ok(D::IptalEdildi { neden, iade: None }),
#             (D::Odendi { odeme_no }, Olay::IptalIstendi { neden }) => Ok(D::IptalEdildi { neden, iade: Some(odeme_no) }),
#             (durum, olay) => Err(GecersizGecis { durum, olay }),
#         }
#     }
#     fn ozet(&self) -> String {
#         match self {
#             SiparisDurumu::Sepette => String::from("sepette"),
#             SiparisDurumu::Odendi { odeme_no } => format!("ödendi (ödeme #{odeme_no})"),
#             SiparisDurumu::Kargoda { odeme_no, takip_no } => format!("kargoda (ödeme #{odeme_no}, takip {takip_no})"),
#             SiparisDurumu::TeslimEdildi { takip_no } => format!("teslim edildi (takip {takip_no})"),
#             SiparisDurumu::IptalEdildi { neden, iade } => {
#                 let iade = iade.map_or(String::from("iade yok"), |no| format!("ödeme #{no} iade"));
#                 format!("iptal edildi ({neden}; {iade})")
#             }
#         }
#     }
#     fn bitti_mi(&self) -> bool {
#         matches!(self, SiparisDurumu::TeslimEdildi { .. } | SiparisDurumu::IptalEdildi { .. })
#     }
# }
fn main() {
    let durum = SiparisDurumu::Sepette
        .gecis(Olay::OdemeAlindi { odeme_no: 7 })
        .and_then(|d| d.gecis(Olay::KargoyaVerildi { takip_no: String::from("TR1") }))
        .and_then(|d| d.gecis(Olay::TeslimEdildi));
    println!("{:?}", durum.map(|d| d.ozet()));

    if let Err(hata) = SiparisDurumu::Sepette.gecis(Olay::TeslimEdildi) {
        println!("reddedildi, sipariş hâlâ elimizde: {}", hata.durum.ozet());
    }

    let iptal = SiparisDurumu::Sepette
        .gecis(Olay::IptalIstendi { neden: String::from("vazgeçtim") });
    println!("{:?}", iptal.map(|d| d.ozet()));
}
```

Çıktı:

```text
Ok("teslim edildi (takip TR1)")
reddedildi, sipariş hâlâ elimizde: sepette
Ok("iptal edildi (vazgeçtim; iade yok)")
```

> [!NOTE]
> Durum makinesi kurmanın tek yolu enum değildir. Her aşamayı ayrı bir türle anlatan yaklaşım, yanlış geçişi çalışma zamanında reddetmek yerine derleme zamanında engeller. Seçenekleri [17. bölümde](../orta/17-oop.md) karşılaştıracağız.[^durum-deseni]

## Kombinatörler: Kutuyu açmadan çalışmak

Şubede paketler bir bandın üstünden geçer ve görevliler kutuyu açmadan iş yapar. Biri "içinde ne varsa bir artır" der; kutu doluysa olur, boş kutu boş geçer. Biri "kutu boşsa yerine şunu koy" der. Bir başkası boş kutuya "neden boş" notu yapıştırır. `Option` ve `Result`'ın bu işleri gören metotlarına *kombinatör* denir. Her adımda kutuyu açıp "dolu mu, boş mu?" diye sorsaydınız kod iç içe girer, asıl iş satırların arasında kaybolurdu.

```rust
let dolu: Option<u32> = Some(20);
let bos: Option<u32> = None;

println!("{:?}", dolu.map(|n| n + 1)); // içindeki artırılır
println!("{:?}", bos.map(|n| n + 1)); // boş kutu boş kalır
println!("{}", bos.unwrap_or(0)); // boştu, yedek değer geldi
println!("{:?}", bos.ok_or("kupon yok")); // boşluğa bir neden eklendi
```

Çıktı:

```text
Some(21)
None
0
Err("kupon yok")
```

Bu sayfada kullanacağımız metotlar şunlardır:[^kombinator]

| Metot | Ne yapar? |
| --- | --- |
| `map` | Dolu kutunun içindekini dönüştürür, boş kutuya dokunmaz. |
| `and_then` | `map` gibidir; dönüştürme adımının kendisi de kutu veriyorsa kullanılır. |
| `filter` | Kutu doluysa ama içindeki koşula uymuyorsa kutuyu boşaltır. |
| `map_or` | Doluysa dönüştürür, boşsa verilen değeri döndürür. |
| `unwrap_or`, `unwrap_or_else` | Yedek değerle kutudan çıkar. İkincisi yedeği gerektiği anda bir closure ile hesaplar. |
| `ok_or`, `ok_or_else` | `Option`'ı `Result`'a çevirir: Boş kutuya bir hata değeri ekler. |
| `map_err` | `Result`'ın hatasını dönüştürür, `Ok` tarafına dokunmaz. |
| `ok` | `Result`'ı `Option`'a çevirir: Hatayı atar. |

### Kupon çözmek

Bir mağazada indirim kuponu `IND-20` biçiminde yazılsın, oran en çok 50 olsun. Müşterinin kuponu olmayabilir. Fonksiyonun üç ayrı hata nedeni vardır ve her biri kendi varyantıyla bildirilir:

```rust
#[derive(Debug, PartialEq)]
enum KuponHatasi {
    Yok,
    BicimBozuk(String),
    OranFazla(u8),
}

fn kupon_coz(kupon: Option<&str>) -> Result<u8, KuponHatasi> {
    let kod = kupon.ok_or(KuponHatasi::Yok)?;
    let bozuk = || KuponHatasi::BicimBozuk(kod.to_string());
    let sayi = kod.strip_prefix("IND-").ok_or_else(bozuk)?;
    let oran: u8 = sayi.parse().map_err(|_| bozuk())?;
    if oran > 50 {
        return Err(KuponHatasi::OranFazla(oran));
    }
    Ok(oran)
}

fn main() {
    for kupon in [Some("IND-20"), None, Some("IND-90"), Some("YAZ"), Some("IND-yirmi")] {
        println!("{kupon:?}: {:?}", kupon_coz(kupon));
    }
}
```

Çıktı:

```text
Some("IND-20"): Ok(20)
None: Err(Yok)
Some("IND-90"): Err(OranFazla(90))
Some("YAZ"): Err(BicimBozuk("YAZ"))
Some("IND-yirmi"): Err(BicimBozuk("IND-yirmi"))
```

Satır satır okuyalım:

- `kupon.ok_or(KuponHatasi::Yok)?` boş kutuyu bir hataya çevirir. `?` operatörü hata varsa fonksiyondan hemen döner, yoksa içteki metni verir.[^soru]
- `bozuk`, iki yerde gereken hatayı üreten bir closure'dır. `kod`'u yalnızca paylaşımlı referansla yakaladığı için `Copy`'dir;[^closure-copy] bir kez `ok_or_else`'e verildikten sonra yeniden kullanılabilmesi bundandır.
- `strip_prefix` öneki bulamazsa `None` verir; `ok_or_else` bunu biçim hatasına çevirir. `ok_or` yerine `ok_or_else` seçilir, çünkü hata değeri bir `String` tahsis eder: `ok_or` argümanını kutu dolu olsa bile hesaplar, `ok_or_else` yalnızca gerektiğinde çalışır.[^tembel]
- `parse` bir `Result` döndürür; `map_err` onun hata türünü bizimkine çevirir.

### Adımın kendisi `Option` döndürüyorsa

Müşterinin kuponunu bulmak iki adımdır ve iki adım da boş çıkabilir: Müşteri listede olmayabilir, müşterinin kuponu olmayabilir. `and_then` ikisini tek bir `Option`'da birleştirir; başka dillerde `a?.b` diye yazılan zincirin karşılığıdır:[^and-then]

```rust
struct Musteri {
    ad: &'static str,
    kupon: Option<&'static str>, // kuponu olmayabilir
}

fn musterinin_kuponu(musteriler: &[Musteri], ad: &str) -> Option<&'static str> {
    musteriler.iter().find(|m| m.ad == ad).and_then(|m| m.kupon)
}

fn main() {
    let musteriler = [
        Musteri { ad: "Ada", kupon: Some("IND-20") },
        Musteri { ad: "Naz", kupon: None },
    ];
    for ad in ["Ada", "Naz", "Yok"] {
        println!("{ad}: {:?}", musterinin_kuponu(&musteriler, ad));
    }
}
```

Çıktı:

```text
Ada: Some("IND-20")
Naz: None
Yok: None
```

`and_then` yerine `map` yazılsaydı sonuç `Option<Option<&'static str>>` olurdu.

### Yedek değerle zincirden çıkmak

Kupon bozuksa satış durmamalı, indirim sıfır sayılmalıdır. `unwrap_or_else`, hatayı görüp yedek değeri o anda üretmeye izin verir; indirimin neden sıfır olduğu da kaybolmaz:

```rust
# #[derive(Debug, PartialEq)]
# enum KuponHatasi { Yok, BicimBozuk(String), OranFazla(u8) }
# struct Musteri { ad: &'static str, kupon: Option<&'static str> }
# fn kupon_coz(kupon: Option<&str>) -> Result<u8, KuponHatasi> {
#     let kod = kupon.ok_or(KuponHatasi::Yok)?;
#     let bozuk = || KuponHatasi::BicimBozuk(kod.to_string());
#     let sayi = kod.strip_prefix("IND-").ok_or_else(bozuk)?;
#     let oran: u8 = sayi.parse().map_err(|_| bozuk())?;
#     if oran > 50 { return Err(KuponHatasi::OranFazla(oran)); }
#     Ok(oran)
# }
# fn musterinin_kuponu(musteriler: &[Musteri], ad: &str) -> Option<&'static str> {
#     musteriler.iter().find(|m| m.ad == ad).and_then(|m| m.kupon)
# }
fn odenecek(tutar: u32, kupon: Option<&str>) -> u32 {
    let oran = kupon_coz(kupon).unwrap_or_else(|hata| {
        if hata != KuponHatasi::Yok {
            println!("  (kupon yok sayıldı: {hata:?})");
        }
        0
    });
    let indirim = u64::from(tutar) * u64::from(oran) / 100;
    tutar.saturating_sub(u32::try_from(indirim).unwrap_or(tutar))
}

fn main() {
    let musteriler = [
        Musteri { ad: "Ada", kupon: Some("IND-20") },
        Musteri { ad: "Can", kupon: Some("IND-90") },
        Musteri { ad: "Ece", kupon: Some("YAZ") },
        Musteri { ad: "Naz", kupon: None },
    ];
    for ad in ["Ada", "Can", "Ece", "Naz", "Yok"] {
        let kupon = musterinin_kuponu(&musteriler, ad);
        println!("{ad}: {kupon:?} → {} TL", odenecek(1000, kupon));
    }

    // Büyük tutarda da taşma olmaz.
    assert_eq!(odenecek(4_000_000_000, Some("IND-50")), 2_000_000_000);
    assert_eq!(odenecek(u32::MAX, None), u32::MAX);
}
```

Çıktı:

```text
Ada: Some("IND-20") → 800 TL
  (kupon yok sayıldı: OranFazla(90))
Can: Some("IND-90") → 1000 TL
  (kupon yok sayıldı: BicimBozuk("YAZ"))
Ece: Some("YAZ") → 1000 TL
Naz: None → 1000 TL
Yok: None → 1000 TL
```

Çarpmanın `u64`'te yapılmasının nedeni taşmadır. Çarpma `u32` ile yapılsaydı `4_000_000_000 * 50` sonucu türe sığmazdı: Program, hata ayıklama derlemesinde panikler, varsayılan sürüm (release) derlemesinde ise değeri sessizce sarıp yanlış tutar üretirdi.[^tasma] İndirim oranı en çok 50 olduğundan indirim tutarı aşamaz; yine de çıkarma, sıfırın altına inemeyen `saturating_sub` ile yapılır.[^saturating]

### Hatalıları elemek

`filter_map`, closure'ın verdiği `Option`'lardan dolu olanları toplar. Başına `ok` koyarsanız hatalı kuponlar elenir:[^filter-map]

```rust
# #[derive(Debug, PartialEq)]
# enum KuponHatasi { Yok, BicimBozuk(String), OranFazla(u8) }
# fn kupon_coz(kupon: Option<&str>) -> Result<u8, KuponHatasi> {
#     let kod = kupon.ok_or(KuponHatasi::Yok)?;
#     let bozuk = || KuponHatasi::BicimBozuk(kod.to_string());
#     let sayi = kod.strip_prefix("IND-").ok_or_else(bozuk)?;
#     let oran: u8 = sayi.parse().map_err(|_| bozuk())?;
#     if oran > 50 { return Err(KuponHatasi::OranFazla(oran)); }
#     Ok(oran)
# }
fn main() {
    let kuponlar = [Some("IND-20"), Some("IND-90"), Some("YAZ"), None];
    let en_iyi_oran = kuponlar
        .iter()
        .filter_map(|kupon| kupon_coz(*kupon).ok()) // hatalılar elenir
        .max();

    let uzunluk = kuponlar[0].map(str::len).filter(|n| *n > 3);
    println!("en iyi oran: {en_iyi_oran:?}, ilk kuponun uzunluğu: {uzunluk:?}");
}
```

Çıktı:

```text
en iyi oran: Some(20), ilk kuponun uzunluğu: Some(6)
```

> [!WARNING]
> `unwrap` ve `expect`, kutu boşsa programı panikletir. "Burada `None` olamaz" iddiasını kanıtlayamıyorsanız yukarıdaki metotlardan biri neredeyse her durumda daha doğru cevaptır. `Result` ve `?` konusunun tamamı [8. bölümdedir](08-error-handling.md).

## En sık altı hata ve çözümleri

Hataların çoğu aynı yanılgıdan doğar: kutuyu, içindeki şeyle bir tutmak. `Option<u32>` bir sayı değil, içinde sayı **olabilen** bir kutudur. Kutu sayının yerine geçebilseydi boş kutu geldiğinde program çökerdi.

### Kutuyu içindeki değer sanmak (`E0308`, `E0369`)

```rust,compile_fail,E0308
let bazi: Option<u32> = "5".parse().ok();
let n: u32 = bazi;
```

Derleyicinin yanıtı:

```text
error[E0308]: mismatched types
  |
3 | let n: u32 = bazi;
  |        ---   ^^^^ expected `u32`, found `Option<u32>`
```

Aynı yanılgı toplama içinde başka bir kodla görünür:

```rust,compile_fail,E0369
let bazi: Option<u32> = "5".parse().ok();
let toplam = bazi + 1;
```

Derleyicinin yanıtı:

```text
error[E0369]: cannot add `{integer}` to `Option<u32>`
```

Ya kutunun içini dönüştürün ya yedek değerle kutudan çıkın ya da `if let` ile açın:[^kutu]

```rust
let bazi: Option<u32> = "5".parse().ok();

println!("{:?}", bazi.map(|n| n + 1)); // kutunun içinde kal
println!("{}", bazi.unwrap_or(0) + 1); // yedek değerle çık
if let Some(m) = bazi {
    println!("{}", m + 1); // kutu açıldı, m artık düz bir sayı
}
```

Çıktı:

```text
Some(6)
6
6
```

### `Option` döndüren adımı `map` ile zincirlemek (`E0308`)

```rust,compile_fail,E0308
let n: Option<u32> = Some("42").map(|s| s.parse::<u32>().ok());
```

Derleyicinin yanıtı:

```text
error[E0308]: mismatched types
  |
  = note: expected enum `Option<u32>`
             found enum `Option<Option<u32>>`
```

`map`, closure'ın döndürdüğü kutuyu bir kutunun daha içine koyar.[^and-then] `and_then` ise tek kutuda bırakır:

```rust
let sayi: Option<u32> = Some("42").and_then(|s| s.parse().ok());
println!("{sayi:?}");
```

Çıktı:

```text
Some(42)
```

### `Result` döndüren fonksiyonda `Option`'a `?` uygulamak (`E0277`)

```rust,compile_fail,E0277
# #[derive(Debug)]
# enum KuponHatasi { Yok }
fn f(k: Option<&str>) -> Result<usize, KuponHatasi> {
    let kod = k?;
    Ok(kod.len())
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0277]: the `?` operator can only be used on `Result`s, not `Option`s, in a function that returns `Result`
  |
4 |     let kod = k?;
  |                ^ use `.ok_or(...)?` to provide an error compatible with `Result<usize, KuponHatasi>`
```

`None`'ın hangi hataya karşılık geldiğini derleyici bilemez.[^soru-nerede] Karşılığı siz söylersiniz:

```rust
#[derive(Debug)]
enum KuponHatasi {
    Yok,
}

fn f(k: Option<&str>) -> Result<usize, KuponHatasi> {
    let kod = k.ok_or(KuponHatasi::Yok)?;
    Ok(kod.len())
}

fn main() {
    println!("{:?} {:?}", f(Some("IND-20")), f(None));
}
```

Çıktı:

```text
Ok(6) Err(Yok)
```

### Ödünç alınmış kutunun içini taşımak (`E0507`)

```rust,compile_fail,E0507
struct Profil {
    takma_ad: Option<String>,
}

fn gorunen_ad(p: &Profil) -> String {
    p.takma_ad.unwrap_or(String::from("anonim"))
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0507]: cannot move out of `p.takma_ad` which is behind a shared reference
  |
6 |     p.takma_ad.unwrap_or(String::from("anonim"))
  |     ^^^^^^^^^^ move occurs because `p.takma_ad` has type `Option<String>`, which does not implement the `Copy` trait
```

`unwrap_or` metodu `self` alır, yani kutuyu tüketir; `&Profil` üzerinden bir şey tüketemezsiniz. `as_deref`, `&Option<String>`'den `Option<&str>` üretir.[^as-deref] Artık tüketilen şey ucuz bir referans kopyasıdır:

```rust
struct Profil {
    takma_ad: Option<String>,
}

fn main() {
    let profiller = [
        Profil { takma_ad: Some(String::from("kbo")) },
        Profil { takma_ad: None },
    ];
    let adlar: Vec<&str> = profiller
        .iter()
        .map(|p| p.takma_ad.as_deref().unwrap_or("anonim"))
        .collect();
    println!("{adlar:?}");
}
```

Çıktı:

```text
["kbo", "anonim"]
```

### Varyantın içini taşıyıp enum'u yeniden kullanmak (`E0382`)

```rust,compile_fail,E0382
#[derive(Debug)]
enum SiparisDurumu {
    Sepette,
    Kargoda { odeme_no: u32, takip_no: String },
}

fn main() {
    let durum = SiparisDurumu::Kargoda { odeme_no: 1, takip_no: String::from("TR9") };
    match durum {
        SiparisDurumu::Kargoda { takip_no, .. } => println!("{takip_no}"),
        _ => {}
    }
    println!("{durum:?}");
}
```

Derleyicinin yanıtı:

```text
error[E0382]: borrow of partially moved value: `durum`
   |
10 |         SiparisDurumu::Kargoda { takip_no, .. } => println!("{takip_no}"),
   |                                  -------- value partially moved here
```

Desen, `takip_no` metnini `durum`'un içinden çıkardı; geriye kısmen taşınmış bir değer kaldı. Referans üzerinde eşleştirirseniz bağlanan ad `&String` olur ve `durum` yerinde kalır:[^binding]

```rust
# #[derive(Debug)]
# enum SiparisDurumu { Sepette, Kargoda { odeme_no: u32, takip_no: String } }
# fn main() {
let durum = SiparisDurumu::Kargoda { odeme_no: 1, takip_no: String::from("TR9") };
if let SiparisDurumu::Kargoda { takip_no, .. } = &durum {
    println!("takip: {takip_no}");
}
println!("durum hâlâ bizde: {durum:?}");
# }
```

Çıktı:

```text
takip: TR9
durum hâlâ bizde: Kargoda { odeme_no: 1, takip_no: "TR9" }
```

### Alan sayısını tutturamamak (`E0023`)

```rust,compile_fail,E0023
enum Olay2 {
    OdemeAlindi(u32, String),
}

fn numara(o: &Olay2) -> u32 {
    match o {
        Olay2::OdemeAlindi(no) => *no,
    }
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0023]: this pattern has 1 field, but the corresponding tuple variant has 2 fields
```

Kullanmadığınız alanı `_` ile, birden çoksa `..` ile atlayın:[^alan-sayisi]

```rust
enum Olay2 {
    OdemeAlindi(u32, String),
}

fn numara(o: &Olay2) -> u32 {
    match o {
        Olay2::OdemeAlindi(no, _) => *no, // (no, ..) da olurdu
    }
}
# fn main() {
#     assert_eq!(numara(&Olay2::OdemeAlindi(7, String::from("kart"))), 7);
# }
```

Alan sayısı ikiyi geçince struct benzeri varyant (`OdemeAlindi { odeme_no, .. }`) hem daha okunaklıdır hem de yeni alan eklenmesine dayanır: Desende `..` varsa yeni alan eski kodu bozmaz. Bu sayfadaki `Olay` bu yüzden öyle yazıldı.

## Akılda kalması gerekenler

- Bir tür tasarlarken kendinize şunu sorun: Bu türle anlamsız bir değer kurulabilir mi? Kurulabiliyorsa kuralı bir metoda değil, türün kendisine yazmayı deneyin.
- Durum makinesinde eski durumu tüketin (`self`), reddedilen geçişte geri verin. Veriyi taşıyın, klonlamayın.
- `_` kolu "varsayılan olarak reddet" demek için uygundur; "her durumu göster" diyen kodda ise eksikleri gizler.
- Kutuyla çalışmanın üç yolu vardır: içini dönüştürmek (`map`, `and_then`, `filter`), yedek değerle çıkmak (`unwrap_or`, `unwrap_or_else`) ve yokluğu hataya çevirmek (`ok_or`, `ok_or_else`).
- Varyantın içindeki veriye yalnızca bakacaksanız değeri değil, referansını eşleştirin.

## Alıştırmalar

**1.** `SiparisDurumu`'na `IadeBekliyor { takip_no: String }` varyantını ekleyin; `TeslimEdildi` durumundayken yeni bir `Olay::IadeIstendi` gelince bu duruma geçilsin. Önce yalnızca varyantı ekleyip `cargo check -p enums` çalıştırın. Derleyici sizi hangi metoda götürüyor? `gecis` neden şikâyet etmiyor?

<details>
<summary>İpucu ve çözüm</summary>

`ozet` `E0004` verir, çünkü varyantları tek tek sayar. `gecis` derlenir, çünkü son kolu her çifti yakalar; siz kol ekleyene kadar yeni geçişi reddeder. `bitti_mi` de şikâyet etmez: `matches!` yeni varyant için `false` döndürür ve iade bekleyen bir sipariş gerçekten bitmemiştir.

```rust
#[derive(Debug, PartialEq)]
enum SiparisDurumu {
    TeslimEdildi { takip_no: String },
    IadeBekliyor { takip_no: String }, // yeni
    // öteki durumlar aynı kalır
}

#[derive(Debug, PartialEq)]
enum Olay {
    TeslimEdildi,
    IadeIstendi, // yeni
}
#
# #[derive(Debug, PartialEq)]
# struct GecersizGecis { durum: SiparisDurumu, olay: Olay }

impl SiparisDurumu {
    fn gecis(self, olay: Olay) -> Result<SiparisDurumu, GecersizGecis> {
        use SiparisDurumu as D;
        match (self, olay) {
            // öteki kollar aynı kalır
            (D::TeslimEdildi { takip_no }, Olay::IadeIstendi) => {
                Ok(D::IadeBekliyor { takip_no })
            }
            (durum, olay) => Err(GecersizGecis { durum, olay }),
        }
    }

    fn ozet(&self) -> String {
        match self {
            SiparisDurumu::TeslimEdildi { takip_no } => {
                format!("teslim edildi (takip {takip_no})")
            }
            SiparisDurumu::IadeBekliyor { takip_no } => {
                format!("iade bekliyor (takip {takip_no})")
            }
        }
    }
}
# fn main() {
#     let teslim = || SiparisDurumu::TeslimEdildi { takip_no: String::from("TR1") };
#     let d = teslim().gecis(Olay::IadeIstendi).expect("geçiş tanımlı");
#     assert_eq!(d.ozet(), "iade bekliyor (takip TR1)");
#     assert!(teslim().gecis(Olay::TeslimEdildi).is_err());
# }
```

</details>

**2.** `kupon_coz`'u, `"IND-20"` yanında `"SABIT-150"` (150 TL sabit indirim) biçimini de tanıyacak şekilde genişletin. Beklenen: `odenecek(1000, Some("SABIT-150"))` 850, `odenecek(100, Some("SABIT-150"))` 0 versin.

<details>
<summary>İpucu ve çözüm</summary>

Dönüş türü artık tek bir `u8` olamaz; iki indirim türünü anlatan bir enum gerekir. Tutardan büyük sabit indirimi `saturating_sub` sıfırda durdurur. `map(Indirim::Sabit)` yazımına dikkat edin: Demet benzeri varyantın adı bir fonksiyondur ve closure yerine doğrudan verilebilir.

```rust
#[derive(Debug, PartialEq)]
enum KuponHatasi {
    Yok,
    BicimBozuk(String),
    OranFazla(u8),
}

#[derive(Debug, PartialEq)]
enum Indirim {
    Yuzde(u8),
    Sabit(u32),
}

fn kupon_coz(kupon: Option<&str>) -> Result<Indirim, KuponHatasi> {
    let kod = kupon.ok_or(KuponHatasi::Yok)?;
    let bozuk = || KuponHatasi::BicimBozuk(kod.to_string());

    if let Some(miktar) = kod.strip_prefix("SABIT-") {
        return miktar.parse().map(Indirim::Sabit).map_err(|_| bozuk());
    }
    let sayi = kod.strip_prefix("IND-").ok_or_else(bozuk)?;
    let oran: u8 = sayi.parse().map_err(|_| bozuk())?;
    if oran > 50 {
        return Err(KuponHatasi::OranFazla(oran));
    }
    Ok(Indirim::Yuzde(oran))
}

fn odenecek(tutar: u32, kupon: Option<&str>) -> u32 {
    match kupon_coz(kupon) {
        Ok(Indirim::Yuzde(oran)) => {
            let indirim = u64::from(tutar) * u64::from(oran) / 100;
            tutar.saturating_sub(u32::try_from(indirim).unwrap_or(tutar))
        }
        Ok(Indirim::Sabit(miktar)) => tutar.saturating_sub(miktar),
        Err(_) => tutar,
    }
}

fn main() {
    println!("{}", odenecek(1000, Some("SABIT-150")));
    println!("{}", odenecek(100, Some("SABIT-150")));
    println!("{}", odenecek(1000, Some("IND-20")));
}
# #[test]
# fn bicim() { assert!(kupon_coz(Some("SABIT-x")).is_err()); }
```

Çıktı:

```text
850
0
800
```

</details>

**3.** `fn gorunen_ad(p: &Profil) -> &str` yazın: Takma ad varsa ve boş değilse onu, yoksa `"anonim"` döndürsün. `match` ve `if` kullanmayın. Beklenen: `Some("")` ve `None` için `"anonim"`, `Some("kbo")` için `"kbo"`.

<details>
<summary>İpucu ve çözüm</summary>

`as_deref` içerideki metne bakış verir, `filter` boş metni eler, `unwrap_or` yedeği koyar:

```rust
struct Profil {
    takma_ad: Option<String>,
}

fn gorunen_ad(p: &Profil) -> &str {
    p.takma_ad
        .as_deref()
        .filter(|ad| !ad.is_empty())
        .unwrap_or("anonim")
}

fn main() {
    for takma_ad in [Some(String::from("kbo")), Some(String::new()), None] {
        println!("{}", gorunen_ad(&Profil { takma_ad }));
    }
}
```

Çıktı:

```text
kbo
anonim
anonim
```

</details>

## Özet

- Anlamsız değerleri denetlemek yerine yazılamaz kılın; bayrak kümeleri yerine veri taşıyan varyantlar kullanın.
- Durum makinesi, (durum, olay) çifti üzerinde tek bir `match`'tir. `self` tüketilir, veri taşınır, tanımsız çiftler reddedilir.
- Kombinatörler `Option` ve `Result` zincirlerini iç içe `match` olmadan kurar; yokluğun hangi hata olduğunu `ok_or` ile siz söylersiniz.
- Altı klasik hatanın çoğu, kutuyu içindeki değerle karıştırmaktan doğar.

Bu sayfadaki türler tek dosyada duruyor ve `GevsekSiparis`'in alanlarına bu dosyadaki her kod yazabiliyor. Sıradaki bölüm [6. Modüller](06-modules.md), kodu bölmeyi ve neyin dışarı açılacağını `pub` ile seçmeyi anlatıyor. `gecis`'teki (durum, olay) deseninin dayandığı desen dili ise [18. bölümde](../orta/18-patterns.md) derinleşiyor.

## Kaynaklar

[^tur-anlam]: Rust API Guidelines, [Arguments convey meaning through types, not `bool` or `Option` (`C-CUSTOM-TYPE`)](https://rust-lang.github.io/api-guidelines/type-safety.html#c-custom-type); hata dizini, [`E0559`](https://doc.rust-lang.org/error_codes/E0559.html) ve [`E0063`](https://doc.rust-lang.org/error_codes/E0063.html).
[^varyant-verisi]: The Rust Programming Language, [Enum Values](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html#enum-values).
[^iki-nokta]: The Rust Programming Language, [Remaining Parts of a Value with ..](https://doc.rust-lang.org/book/ch19-03-pattern-syntax.html#remaining-parts-of-a-value-with-) ve [Structs and Tuples](https://doc.rust-lang.org/book/ch19-03-pattern-syntax.html#structs-and-tuples).
[^map-or]: std, [`Option::map_or`](https://doc.rust-lang.org/std/option/enum.Option.html#method.map_or).
[^matches]: std, [`matches!`](https://doc.rust-lang.org/std/macro.matches.html); std 1.99.0 kaynak kodu, [`core/src/macros/mod.rs`, `macro_rules! matches`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/macros/mod.rs#L431).
[^catch-all]: The Rust Programming Language, [Catch-All Patterns and the _ Placeholder](https://doc.rust-lang.org/book/ch06-02-match.html#catch-all-patterns-and-the-_-placeholder); hata dizini, [`E0004`](https://doc.rust-lang.org/error_codes/E0004.html).
[^result-and-then]: std, [`Result::and_then`](https://doc.rust-lang.org/std/result/enum.Result.html#method.and_then).
[^durum-deseni]: The Rust Programming Language, [Why Not An Enum?](https://doc.rust-lang.org/book/ch18-03-oo-design-patterns.html#why-not-an-enum) ve [Encoding States and Behavior as Types](https://doc.rust-lang.org/book/ch18-03-oo-design-patterns.html#encoding-states-and-behavior-as-types).
[^kombinator]: std, [`Option` metotları](https://doc.rust-lang.org/std/option/enum.Option.html) (`map`, `and_then`, `filter`, `ok_or`, `ok_or_else`, `unwrap_or_else`) ve [`Result::map_err`](https://doc.rust-lang.org/std/result/enum.Result.html#method.map_err), [`Result::ok`](https://doc.rust-lang.org/std/result/enum.Result.html#method.ok).
[^soru]: The Rust Programming Language, [The ? Operator Shortcut](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#the--operator-shortcut).
[^closure-copy]: The Rust Reference, [Call traits and coercions](https://doc.rust-lang.org/reference/types/closure.html#call-traits-and-coercions).
[^tembel]: std, [`Option::ok_or`](https://doc.rust-lang.org/std/option/enum.Option.html#method.ok_or) ve [`Option::ok_or_else`](https://doc.rust-lang.org/std/option/enum.Option.html#method.ok_or_else).
[^and-then]: std, [`Option::and_then`](https://doc.rust-lang.org/std/option/enum.Option.html#method.and_then) ve [`Option::map`](https://doc.rust-lang.org/std/option/enum.Option.html#method.map).
[^tasma]: The Rust Programming Language, [Integer Overflow](https://doc.rust-lang.org/book/ch03-02-data-types.html#integer-overflow); The Rust Reference, [Overflow](https://doc.rust-lang.org/reference/expressions/operator-expr.html#overflow).
[^saturating]: std, [`u32::saturating_sub`](https://doc.rust-lang.org/std/primitive.u32.html#method.saturating_sub).
[^filter-map]: std, [`Iterator::filter_map`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.filter_map) ve [`Option::filter`](https://doc.rust-lang.org/std/option/enum.Option.html#method.filter).
[^kutu]: The Rust Programming Language, [The `Option` Enum](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html#the-option-enum); hata dizini, [`E0308`](https://doc.rust-lang.org/error_codes/E0308.html) ve [`E0369`](https://doc.rust-lang.org/error_codes/E0369.html).
[^soru-nerede]: The Rust Programming Language, [Where to Use the ? Operator](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#where-to-use-the--operator); hata dizini, [`E0277`](https://doc.rust-lang.org/error_codes/E0277.html).
[^as-deref]: std, [`Option::as_deref`](https://doc.rust-lang.org/std/option/enum.Option.html#method.as_deref); hata dizini, [`E0507`](https://doc.rust-lang.org/error_codes/E0507.html).
[^binding]: The Rust Reference, [Binding modes](https://doc.rust-lang.org/reference/patterns.html#binding-modes); hata dizini, [`E0382`](https://doc.rust-lang.org/error_codes/E0382.html).
[^alan-sayisi]: The Rust Reference, [Rest patterns](https://doc.rust-lang.org/reference/patterns.html#rest-pattern); hata dizini, [`E0023`](https://doc.rust-lang.org/error_codes/E0023.html).
