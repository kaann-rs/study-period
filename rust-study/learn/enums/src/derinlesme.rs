// Açıklama: learn/enums/ACIKLAMA.md
// Derinleşme — Bölüm 6: Enums (pekiştirme: gerçekçi senaryo, sık hatalar, alıştırmalar)
// Çalıştır: cargo run -p enums   |   Test: cargo test -p enums
//
// main.rs enum'un ne olduğunu, Option'ın null'ı neden kaldırdığını ve match'in bütünlük
// (exhaustiveness) denetimini gösterdi. Bu dosya onları bir tasarım aracına çevirir:
//   • "geçersiz durumları temsil edilemez kıl" → bayraklı struct ile enum'u yan yana koyar
//   • sipariş durum makinesi → geçişler `(durum, olay)` üzerinde tek bir match
//   • Option/Result kombinatörleri → iç içe match yazmadan "yoksa / hatalıysa" akışı
//   • enum'la çalışırken en sık görülen derleme hataları ve düzeltmeleri

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Önce main.rs'i oku; benzetmeler bu kez bir KARGO ŞUBESİNDE, bir siparişin yolculuğunda geçer.
//
// Bu dosyanın tek cümlelik özeti: Bir şeyin "hangi aşamada" olduğunu enum ile yazarsan, saçma
// aşamaları (ödenmemiş ama yola çıkmış paket) kimse yanlışlıkla bile oluşturamaz.
//
// MİNİ SÖZLÜK (yalnız bu dosyada yeni geçenler; öteki terimler main.rs başındaki sözlükte):
//   bayrak (flag)          → evet/hayır tutan alan: `odendi: bool`.
//   durum makinesi         → bir şeyin hangi aşamalarda olabileceğini ve aşamalar arasında hangi
//                            olayla geçildiğini yazan tablo.
//   olay, geçiş            → "ödeme alındı" gibi bir haber / o haberle bir aşamadan ötekine geçmek.
//   Result, Ok, Err        → "iş başarıldı, sonuç bu" (Ok) ya da "olmadı, sebep bu" (Err) kutusu.
//   `?`                    → "hata varsa hemen onu geri döndür, yoksa içindekini ver" kısaltması.
//   closure, |n| n + 1     → yerinde yazılmış isimsiz küçük fonksiyon; `|n|` girdisidir.
//   kombinatör             → Option/Result kutusunu açmadan, üstünde iş yapan metot.
//   map                    → kutu doluysa içindekini dönüştürür; boşsa boş bırakır.
//   and_then               → map gibi, ama dönüştürme işinin kendisi de kutu veriyorsa kullanılır.
//   filter                 → kutu dolu ama içindeki şarta uymuyorsa kutuyu boşaltır.
//   ok_or(hata)            → Option'ı Result'a çevirir: boş kutuya "neden boş" notu ekler.
//   unwrap_or_else         → kutu boşsa (ya da hatalıysa) yedek değeri o anda hesaplar.
//   unwrap / expect        → "kesin dolu" diyerek açar; boş çıkarsa program panikler.
//   as_deref               → kutunun içindeki String'i almadan, ona bakış (&str) verir.
//   matches!               → "bu değer şu desene uyuyor mu" sorusuna true/false cevabı verir.
//   tuple, (a, b)          → iki değeri yan yana tutan paket; ikisine birden bakmayı sağlar.
//   &'static str           → program boyunca yaşayan yazı; tırnak içine yazdığın her yazı böyledir.
//   saturating_sub         → çıkarma; sonuç sıfırın altına inecekse sıfırda durur.
//   test, assert           → kodun doğru çalıştığını denetleyen küçük sınav ve o sınavın sorusu.

pub fn calistir() {
    println!("\n== Derinleşme: Enums — sipariş durum makinesi ve Option/Result ==");
    gecersiz_durumlar();
    durum_makinesi();
    kombinatorler();
    sik_hatalar();
}

// 1. Bu konuyu neden öğreniyoruz? — Geçersiz durumları temsil edilemez kıl
// SADE ANLATIM: Kargo fişinde üç kutucuk: "ödendi", "yolda", "takip no". Ayrı işaretlenince saçma
// fiş yazılabilir: "ödenmedi" ama "yolda". Tek DAMGA (SEPETTE/ÖDENDİ/YOLDA) basılınca saçma fiş
// kesilemez; takip no yalnız YOLDA'nın yanında yazılır. Bayraklı struct ilk fiş, enum ikinci fiş.
// Neden? Bayraklı fişte her kullanan tutarlılığı denetler, biri unutur; enum'da yanlış fiş yazılamaz.
//   Mini örnek:
//     enum Durum { Sepette, Yolda { takip: String } }
//     let d = Durum::Yolda { takip: String::from("TR1") };     // ✓ yoldaysa takip numarası var
//     let s = Durum::Sepette;                                  // ✓ sepetteyse takip gözü hiç yok
//     let e = Durum::Sepette { takip: String::from("TR1") };   // ✗ derlenmez: öyle bir göz yok
//     let f = Durum::Yolda {};                                 // ✗ derlenmez: takip numarası eksik
struct GevsekSiparis {
    odendi: bool,
    kargoda: bool,
    takip_no: Option<String>,
}

impl GevsekSiparis {
    // Kurallar türde değil BURADA yaşıyor; çağırmayı unutan her kod bozuk veriyle çalışır.
    fn tutarli_mi(&self) -> bool {
        let kargo_odemesiz = self.kargoda && !self.odendi;
        let kargo_takipsiz = self.kargoda != self.takip_no.is_some();
        !kargo_odemesiz && !kargo_takipsiz
    }
}

fn gecersiz_durumlar() {
    println!("-- 1. Geçersiz durumları temsil edilemez kıl");
    // 2 × 2 × 2 = 8 farklı değer kurulabilir; anlamlı olanı 3 (sepette, ödendi, kargoda).
    // Kalan 5'i de derlenir. Örneğin "ödenmemiş ama kargoda, takip numarası da yok":
    let tuhaf = GevsekSiparis {
        odendi: false,
        kargoda: true,
        takip_no: None,
    };
    println!(
        "bayraklı model derlendi, tutarlı mı? {}",
        tuhaf.tutarli_mi()
    ); // bayraklı model derlendi, tutarlı mı? false

    // Enum modelinde aynı saçmalığı YAZAMAZSIN: takip no yalnız Kargoda/TeslimEdildi'de var.
    // Denetleyecek şey kalmaz, tutarli_mi()'ye gerek kalmaz.
    let kargoda = SiparisDurumu::Kargoda {
        odeme_no: 4821,
        takip_no: String::from("TR123"),
    };
    println!("enum modeli: {}", kargoda.ozet()); // enum modeli: kargoda (ödeme #4821, takip TR123)
}

// 2. Gerçek hayattan örnek: sipariş durum makinesi
// SADE ANLATIM: Şubenin duvarında bir yol haritası asılıdır: "SEPETTE iken ödeme gelirse ÖDENDİ
// olur", "ÖDENDİ iken kargoya verilirse YOLDA olur". Haritada çizilmemiş yol kapalıdır: SEPETTE
// bir paket doğrudan TESLİM EDİLDİ olamaz. Aşağıdaki `gecis` fonksiyonu bu haritadır: elindeki
// aşamaya ve gelen habere birlikte bakar, haritada varsa yeni aşamayı verir, yoksa "bu yol kapalı"
// der ve paketi olduğu aşamada sana geri verir.
// Neden böyle? Harita tek yerde yazılı olmasaydı, kurallar programın dört bir yanına dağılırdı ve
// biri bir yerde kapalı yoldan paket geçirirdi.
//   Mini örnek:
//     enum Asama { Sepette, Odendi, Yolda }
//     enum Haber { OdemeGeldi, KargoyaVerildi }
//     fn gec(a: Asama, h: Haber) -> Asama {
//         match (a, h) {                                        // ikisine birlikte bak
//             (Asama::Sepette, Haber::OdemeGeldi) => Asama::Odendi,       // ✓ haritada var
//             (Asama::Odendi, Haber::KargoyaVerildi) => Asama::Yolda,     // ✓ haritada var
//             (a, _) => a,                                      // yol kapalı: olduğun yerde kal
//         }
//     }
#[derive(Debug, Clone, PartialEq)]
enum SiparisDurumu {
    Sepette,
    Odendi { odeme_no: u32 },
    Kargoda { odeme_no: u32, takip_no: String },
    TeslimEdildi { takip_no: String },
    IptalEdildi { neden: String, iade: Option<u32> },
}

// Dışarıdan gelen olaylar da bir enum: "ne oldu?" sorusunun kapalı bir cevap kümesi var.
#[derive(Debug, Clone, PartialEq)]
enum Olay {
    OdemeAlindi { odeme_no: u32 },
    KargoyaVerildi { takip_no: String },
    TeslimEdildi,
    IptalIstendi { neden: String },
}

#[derive(Debug, PartialEq)]
struct GecersizGecis {
    durum: SiparisDurumu, // tüketilen durum çağırana GERİ verilir, sipariş kaybolmaz
    olay: Olay,
}

impl SiparisDurumu {
    // Adım adım kod analizi:
    // (1) `self` tüketilir: eski durum geçişten sonra yoktur; "hem ödendi hem sepette" diye
    //     iki kopya ortalıkta dolaşamaz (structs crate'indeki pasiflestir(self) ile aynı fikir).
    // (2) `(self, olay)` tuple'ı üzerinde match: her kol "şu durumdayken şu olay" çiftidir;
    //     geçiş tablosu kâğıttaki hâliyle koda geçer.
    // (3) Desen, eski varyantın verisini çıkarıp yenisine TAŞIR (odeme_no, takip_no): clone yok.
    // (4) Son kol her şeyi yakalar: listelenmeyen her çift geçersizdir. Buradaki catch-all
    //     bilinçli bir "varsayılan olarak reddet"tir; yeni eklenen bir durum/olay kendiliğinden
    //     izin almaz. (ozet()'te ise tam tersini istiyoruz, oraya bak.)
    fn gecis(self, olay: Olay) -> Result<SiparisDurumu, GecersizGecis> {
        use SiparisDurumu as D;
        match (self, olay) {
            (D::Sepette, Olay::OdemeAlindi { odeme_no }) => Ok(D::Odendi { odeme_no }),
            (D::Odendi { odeme_no }, Olay::KargoyaVerildi { takip_no }) => {
                Ok(D::Kargoda { odeme_no, takip_no })
            }
            (D::Kargoda { takip_no, .. }, Olay::TeslimEdildi) => Ok(D::TeslimEdildi { takip_no }),
            (D::Sepette, Olay::IptalIstendi { neden }) => Ok(D::IptalEdildi { neden, iade: None }),
            (D::Odendi { odeme_no }, Olay::IptalIstendi { neden }) => Ok(D::IptalEdildi {
                neden,
                iade: Some(odeme_no),
            }),
            (durum, olay) => Err(GecersizGecis { durum, olay }),
        }
    }

    // Burada `_ =>` kolu YOK ve olmamalı: yarın `IadeBekliyor` varyantı eklenirse bu match
    // E0004 ile derlenmez ve derleyici seni tam bu satıra getirir. `_ => "bilinmiyor"` yazılmış
    // olsaydı yeni durum sessizce "bilinmiyor" diye raporlanırdı — hata derleme anından
    // müşterinin ekranına taşınırdı.
    fn ozet(&self) -> String {
        match self {
            SiparisDurumu::Sepette => String::from("sepette"),
            SiparisDurumu::Odendi { odeme_no } => format!("ödendi (ödeme #{odeme_no})"),
            SiparisDurumu::Kargoda { odeme_no, takip_no } => {
                format!("kargoda (ödeme #{odeme_no}, takip {takip_no})")
            }
            SiparisDurumu::TeslimEdildi { takip_no } => format!("teslim edildi (takip {takip_no})"),
            SiparisDurumu::IptalEdildi { neden, iade } => {
                // Option kombinatörü: Some(no) → metne çevir, None → varsayılan metin.
                let iade = iade.map_or(String::from("iade yok"), |no| format!("ödeme #{no} iade"));
                format!("iptal edildi ({neden}; {iade})")
            }
        }
    }

    // matches!: "şu varyantlardan biri mi?" sorusunu bool'a çevirir (içeride `_ => false`).
    fn bitti_mi(&self) -> bool {
        matches!(
            self,
            SiparisDurumu::TeslimEdildi { .. } | SiparisDurumu::IptalEdildi { .. }
        )
    }
}

// Bir olay dizisini sırayla uygular; reddedilen olay durumu değiştirmez.
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

fn durum_makinesi() {
    println!("-- 2. Sipariş durum makinesi");
    let son = olaylari_isle(
        SiparisDurumu::Sepette,
        vec![
            Olay::OdemeAlindi { odeme_no: 4821 },
            Olay::KargoyaVerildi {
                takip_no: String::from("TR123"),
            },
            Olay::IptalIstendi {
                neden: String::from("vazgeçtim"),
            },
            Olay::TeslimEdildi,
        ],
    );
    println!("bitti mi? {}", son.bitti_mi());
    // Beklenen çıktı:
    //     → ödendi (ödeme #4821)
    //     → kargoda (ödeme #4821, takip TR123)
    //     ret: 'kargoda (ödeme #4821, takip TR123)' durumunda IptalIstendi { neden: "vazgeçtim" } işlenemez
    //     → teslim edildi (takip TR123)
    //   bitti mi? true

    let iptal = olaylari_isle(
        SiparisDurumu::Odendi { odeme_no: 77 },
        vec![Olay::IptalIstendi {
            neden: String::from("yanlış ürün"),
        }],
    );
    println!("bitti mi? {}", iptal.bitti_mi());
    // Beklenen çıktı:
    //     → iptal edildi (yanlış ürün; ödeme #77 iade)
    //   bitti mi? true
}

// 3. Option/Result kombinatörleri — null yerine Option, iç içe match yerine zincir
// SADE ANLATIM: Şubede paketler bir bandın üstünden geçer ve görevliler kutuyu AÇMADAN iş yapar.
// Biri "içinde ne varsa bir artır" der: kutu doluysa olur, boşsa kutu boş geçer (`map`). Biri
// "kutu boşsa yerine şunu koy" der (`unwrap_or`). Biri de boş kutuya "neden boş" notu yapıştırır
// (`ok_or`). Bu metotlara kombinatör denir; her adımda `match` yazmaktan kurtarırlar.
// Neden böyle? Her adımda kutuyu açıp "dolu mu, boş mu" diye sorsaydın kod iç içe girer, asıl iş
// satırların arasında kaybolurdu.
//   Mini örnek:
//     let dolu: Option<u32> = Some(20);
//     let bos: Option<u32> = None;
//     println!("{:?}", dolu.map(|n| n + 1));       // ✓ Some(21) → içindeki artırıldı
//     println!("{:?}", bos.map(|n| n + 1));        // None → boş kutu boş kalır
//     println!("{}", bos.unwrap_or(0));            // ✓ 0 → boştu, yedek değer geldi
//     println!("{:?}", bos.ok_or("kupon yok"));    // Err("kupon yok") → boşluğa bir sebep eklendi
#[derive(Debug, PartialEq)]
enum KuponHatasi {
    Yok,
    BicimBozuk(String),
    OranFazla(u8),
}

struct Musteri {
    ad: &'static str,
    kupon: Option<&'static str>, // "kuponu olmayabilir" bilgisi türde yazılı
}

// Nasıl çalışır? Kombinatör = Option/Result'ın içini AÇMADAN üzerinde işlem yapan metot.
//   ok_or / ok_or_else : Option → Result   ("yokluk"a bir hata değeri ver)
//   map_err            : hatanın türünü değiştir, Ok'a dokunma
//   `?`                : Err ise fonksiyondan hemen dön, Ok ise içini ver
// Kupon biçimi: "IND-<oran>", oran en çok 50.
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

// and_then: adımın KENDİSİ Option döndürüyorsa (başka dillerdeki `a?.b` zinciri).
// `find(..)` müşteri yoksa None, `m.kupon` kupon yoksa None; ikisi tek Option'da birleşir.
fn musterinin_kuponu(musteriler: &[Musteri], ad: &str) -> Option<&'static str> {
    musteriler.iter().find(|m| m.ad == ad).and_then(|m| m.kupon)
}

// unwrap_or_else: hata durumunda yedek değeri bir closure ile üret. Kupon bozuksa satış
// durmaz, indirim 0 sayılır; ama neden 0 olduğu kaybolmasın diye hata burada raporlanır.
fn odenecek(tutar: u32, kupon: Option<&str>) -> u32 {
    let oran = kupon_coz(kupon).unwrap_or_else(|hata| {
        if hata != KuponHatasi::Yok {
            println!("  (kupon yok sayıldı: {hata:?})");
        }
        0
    });
    // Çarpma u64'te: büyük tutarlarda `tutar * oran` u32'ye sığmaz (debug'da panik, release'de
    // sessiz sarma). İndirim tutarı aşamaz; yine de çıkarmayı doyurarak yapıyoruz.
    let indirim = u64::from(tutar) * u64::from(oran) / 100;
    tutar.saturating_sub(u32::try_from(indirim).unwrap_or(tutar))
}

fn kombinatorler() {
    println!("-- 3. Option/Result kombinatörleri");
    let musteriler = [
        Musteri {
            ad: "Ada",
            kupon: Some("IND-20"),
        },
        Musteri {
            ad: "Can",
            kupon: Some("IND-90"),
        },
        Musteri {
            ad: "Ece",
            kupon: Some("YAZ"),
        },
        Musteri {
            ad: "Naz",
            kupon: None,
        },
    ];
    for ad in ["Ada", "Can", "Ece", "Naz", "Yok"] {
        let kupon = musterinin_kuponu(&musteriler, ad);
        println!("{ad}: {kupon:?} → {} TL", odenecek(1000, kupon));
    }
    // Beklenen çıktı:
    //   Ada: Some("IND-20") → 800 TL
    //     (kupon yok sayıldı: OranFazla(90))
    //   Can: Some("IND-90") → 1000 TL
    //     (kupon yok sayıldı: BicimBozuk("YAZ"))
    //   Ece: Some("YAZ") → 1000 TL
    //   Naz: None → 1000 TL
    //   Yok: None → 1000 TL

    // Önemli noktalar:
    //   • map      : içteki değeri dönüştür            Option<T> → Option<U>
    //   • and_then : adım da Option/Result döndürüyor  (map kullansaydın Option<Option<U>> olurdu)
    //   • filter   : Some'ı bir koşula bağla, tutmazsa None
    //   • unwrap_or / unwrap_or_else : zincirin sonunda yedek değerle Option'dan çık
    // unwrap()/expect() ise None/Err'de panikler; "burada None olamaz"ı kanıtlayamıyorsan
    // yukarıdakilerden biri neredeyse her zaman daha doğru cevaptır.
    let en_iyi_oran = musteriler
        .iter()
        .filter_map(|m| kupon_coz(m.kupon).ok()) // Result → Option: hatalıları ele
        .max();
    let uzunluk = musterinin_kuponu(&musteriler, "Ada")
        .map(str::len)
        .filter(|n| *n > 3);
    println!("en iyi oran: {en_iyi_oran:?}, Ada'nın kupon uzunluğu: {uzunluk:?}"); // en iyi oran: Some(20), Ada'nın kupon uzunluğu: Some(6)
}

// 4. Sık yapılan hatalar — Yanlış kullanım / Doğru kullanım
// SADE ANLATIM: Altı hatanın çoğu aynı yanılgıdan: kutuyu içindeki şeyle bir tutmak. `Option<u32>`
// sayı değil, sayı OLABİLEN kutudur; önce güvenle açarsın. Kutu sayı yerine geçebilseydi boş kutu
// geldiğinde program çökerdi.
//   Mini örnek:
//     let kutu: Option<u32> = Some(5);
//     let n: u32 = kutu;                 // ✗ derlenmez: kutu, sayı değildir
//     let n: u32 = kutu.unwrap_or(0);    // ✓ 5 → doluysa içindeki, boşsa 0
//     if let Some(m) = kutu {
//         println!("{}", m + 1);         // ✓ 6 → kutu açıldı, m artık düz bir sayı
//     }
struct Profil {
    takma_ad: Option<String>,
}

fn sik_hatalar() {
    println!("-- 4. Sık yapılan hatalar");
    let bazi: Option<u32> = "5".parse().ok(); // çalışma anında belli olan bir Option

    // (a) Option<T>'yi T gibi kullanmak:
    //   let n: u32 = bazi;      // ✗ E0308: mismatched types (expected `u32`, found `Option<u32>`)
    //   let toplam = bazi + 1;  // ✗ E0369: cannot add `{integer}` to `Option<u32>`
    // ✓ Ya içini dönüştür (map) ya yedek değerle çık (unwrap_or):
    println!("(a) {:?} / {}", bazi.map(|n| n + 1), bazi.unwrap_or(0) + 1); // (a) Some(6) / 6

    // (b) Option döndüren adımı map ile zincirlemek:
    //   let n: Option<u32> = Some("42").map(|s| s.parse::<u32>().ok());
    //                           // ✗ E0308: mismatched types — closure Option<u32> döndürüyor,
    //                           //   sonuç Option<Option<u32>> oluyor
    // ✓ and_then düzleştirir:
    let sayi: Option<u32> = Some("42").and_then(|s| s.parse().ok());
    println!("(b) {sayi:?}"); // (b) Some(42)

    // (c) Result döndüren fonksiyonda Option'a `?` uygulamak:
    //   fn f(k: Option<&str>) -> Result<u8, KuponHatasi> { let kod = k?; .. }
    //                           // ✗ E0277: the `?` operator can only be used on `Result`s,
    //                           //   not `Option`s, in a function that returns `Result`
    // Neden? None'ın hangi hataya karşılık geldiğini derleyici bilemez. ✓ Sen söylersin:
    println!("(c) {:?}", kupon_coz(None)); // (c) Err(Yok)   ← içeride `kupon.ok_or(KuponHatasi::Yok)?`

    // (d) Ödünç alınmış Option<String>'in içini taşımaya çalışmak:
    //   fn gorunen_ad(p: &Profil) -> String { p.takma_ad.unwrap_or(String::from("anonim")) }
    //                           // ✗ E0507: cannot move out of `p.takma_ad` which is behind a
    //                           //   shared reference
    // unwrap_or `self` alır, yani Option'ı tüketir; &Profil üzerinden tüketemezsin.
    // ✓ as_deref: &Option<String> → Option<&str>. Tüketilen şey artık ucuz bir referans kopyası.
    let profiller = [
        Profil {
            takma_ad: Some(String::from("kbo")),
        },
        Profil { takma_ad: None },
    ];
    let adlar: Vec<&str> = profiller
        .iter()
        .map(|p| p.takma_ad.as_deref().unwrap_or("anonim"))
        .collect();
    println!("(d) {adlar:?}"); // (d) ["kbo", "anonim"]

    // (e) match ile varyantın içini taşıyıp enum'u yeniden kullanmak:
    //   match durum { SiparisDurumu::Kargoda { takip_no, .. } => println!("{takip_no}"), _ => {} }
    //   println!("{durum:?}");  // ✗ E0382: borrow of partially moved value: `durum`
    // Desen takip_no String'ini durum'un içinden ÇIKARDI. ✓ Referans üzerinde eşleştir;
    // bağlanan takip_no o zaman &String olur ve durum yerinde kalır:
    let durum = SiparisDurumu::Kargoda {
        odeme_no: 1,
        takip_no: String::from("TR9"),
    };
    if let SiparisDurumu::Kargoda { takip_no, .. } = &durum {
        println!("(e) takip: {takip_no}, durum hâlâ bizde: {}", durum.ozet()); // (e) takip: TR9, durum hâlâ bizde: kargoda (ödeme #1, takip TR9)
    }

    // (f) Tuple-like varyantta alan sayısını tutturamamak (enum Olay2 { OdemeAlindi(u32, String) }):
    //   Olay2::OdemeAlindi(no) => ..  // ✗ E0023: this pattern has 1 field, but the corresponding
    //                                 //   tuple variant has 2 fields
    // ✓ `OdemeAlindi(no, _)` ya da `OdemeAlindi(no, ..)`. Alan sayısı ikiyi geçince struct-like
    // varyant (`OdemeAlindi { odeme_no, .. }`) hem okunur hem de alan eklemeye dayanıklıdır;
    // bu dosyadaki Olay bu yüzden öyle yazıldı.
}

// SADE ANLATIM (aşağıdaki bölüm için): Alıştırmalar kendini denemen içindir. İlki haritaya yeni
// bir aşama ekletir ve derleyicinin seni eksik satıra nasıl götürdüğünü gösterir. İkincisi kupon
// okuyan kodu genişletir, üçüncüsü kombinatörleri art arda kullandırır. Takılırsan ipuçlarına bak.
// ============================================================================
// ALIŞTIRMALAR (çözümler verilmedi)
// ============================================================================
//
// 1. SiparisDurumu'na `IadeBekliyor { takip_no: String }` varyantı ekle; TeslimEdildi iken
//    yeni bir `Olay::IadeIstendi` gelince bu duruma geçilsin.
//    İpucu: önce yalnız varyantı ekleyip `cargo check -p enums` çalıştır ve derleyicinin seni
//    hangi fonksiyona götürdüğüne bak; gecis()'in neden şikâyet ETMEDİĞİNİ düşün.
//    Beklenen: ozet() E0004 verir; gecis() derlenir ama sen kol eklemeden yeni geçişi reddeder.
//
// 2. `kupon_coz`u "IND-20" yanında "SABIT-150" (150 TL sabit indirim) biçimini de tanıyacak
//    şekilde genişlet.
//    İpucu: dönüş türü artık tek bir u8 olamaz; `enum Indirim { Yuzde(u8), Sabit(u32) }` tanımla,
//    odenecek() içinde match et. Tutardan büyük sabit indirim için `saturating_sub`a bak.
//    Beklenen: odenecek(1000, Some("SABIT-150")) == 850, odenecek(100, Some("SABIT-150")) == 0.
//
// 3. `fn gorunen_ad(p: &Profil) -> &str` yaz: takma ad varsa ve boş değilse onu, yoksa
//    "anonim" döndürsün; `match` ve `if` kullanma.
//    İpucu: as_deref + filter + unwrap_or.
//    Beklenen: Some("") ve None için "anonim", Some("kbo") için "kbo".
//
// Bir sonraki konuyla bağlantısı:
// Buradaki türler tek dosyada duruyor ve GevsekSiparis'in alanlarına herkes yazabiliyor.
// `learn/modules` (Bölüm 7) kodu modüllere bölmeyi ve `pub` ile neyin dışarı açılacağını
// anlatır. Result ve `?` operatörünün tamamı `learn/error-handling`de; gecis()'teki
// `(durum, olay)` deseninin dayandığı desen dili ise `learn/patterns`te derinleşir.

// SADE ANLATIM: `testler` programla çalışmaz; `cargo test -p enums` ile sınavlar çalışır.
// `assert_eq!(a, b)` eşitlik ister; eşit değilse sınav kalır ve haber verilir.
#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn buyuk_tutarda_indirim_hesabi_tasmaz() {
        // u32'de `tutar * oran` taşardı (4_000_000_000 * 50); u64'te hesaplandığı için güvenli.
        assert_eq!(odenecek(4_000_000_000, Some("IND-50")), 2_000_000_000);
        assert_eq!(odenecek(u32::MAX, None), u32::MAX);
    }

    #[test]
    fn gecerli_gecis_zinciri_teslime_ulasir() {
        let durum = SiparisDurumu::Sepette
            .gecis(Olay::OdemeAlindi { odeme_no: 7 })
            .and_then(|d| {
                d.gecis(Olay::KargoyaVerildi {
                    takip_no: String::from("TR1"),
                })
            })
            .and_then(|d| d.gecis(Olay::TeslimEdildi))
            .unwrap();
        assert_eq!(
            durum,
            SiparisDurumu::TeslimEdildi {
                takip_no: String::from("TR1")
            }
        );
        assert!(durum.bitti_mi());
    }

    #[test]
    fn gecersiz_gecis_durumu_geri_verir() {
        let hata = SiparisDurumu::Sepette
            .gecis(Olay::TeslimEdildi)
            .unwrap_err();
        assert_eq!(hata.durum, SiparisDurumu::Sepette);
        assert_eq!(hata.olay, Olay::TeslimEdildi);
        assert!(!hata.durum.bitti_mi());
    }

    #[test]
    fn odenmis_siparisin_iptali_iade_dogurur() {
        let iptal = |durum: SiparisDurumu| {
            durum.gecis(Olay::IptalIstendi {
                neden: String::from("x"),
            })
        };
        let odenmis = iptal(SiparisDurumu::Odendi { odeme_no: 9 }).unwrap();
        assert_eq!(odenmis.ozet(), "iptal edildi (x; ödeme #9 iade)");
        let sepetteki = iptal(SiparisDurumu::Sepette).unwrap();
        assert_eq!(sepetteki.ozet(), "iptal edildi (x; iade yok)");
    }

    #[test]
    fn kupon_cozumu_her_hata_turunu_ayirir() {
        assert_eq!(kupon_coz(Some("IND-20")), Ok(20));
        assert_eq!(kupon_coz(None), Err(KuponHatasi::Yok));
        assert_eq!(kupon_coz(Some("IND-90")), Err(KuponHatasi::OranFazla(90)));
        assert_eq!(
            kupon_coz(Some("YAZ")),
            Err(KuponHatasi::BicimBozuk(String::from("YAZ")))
        );
        assert_eq!(
            kupon_coz(Some("IND-yirmi")),
            Err(KuponHatasi::BicimBozuk(String::from("IND-yirmi")))
        );
    }

    #[test]
    fn bozuk_kupon_satisi_durdurmaz() {
        assert_eq!(odenecek(1000, Some("IND-20")), 800);
        assert_eq!(odenecek(1000, Some("IND-90")), 1000);
        assert_eq!(odenecek(1000, None), 1000);
    }

    #[test]
    fn bayrakli_model_tutarsiz_deger_kurmaya_izin_verir() {
        let tuhaf = GevsekSiparis {
            odendi: false,
            kargoda: true,
            takip_no: None,
        };
        assert!(!tuhaf.tutarli_mi());
    }
}
