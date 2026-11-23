// Derinleşme — Bölüm 9: Error Handling (pekiştirme: gerçekçi senaryo, sık hatalar, alıştırmalar)
// Çalıştır: cargo run -p error-handling   |   Test: cargo test -p error-handling
//
// main.rs Result'ı, `?`'i ve tek bir From dönüşümünü AYRI AYRI gösterdi. Burada hepsini küçük ama
// gerçekçi bir uygulamada birleştiriyoruz: bir ayar dosyasını oku → satır satır ayrıştır → doğrula.
// main.rs'te OLMAYANLAR: `std::error::Error` + `source()` zinciri, hataya BAĞLAM (satır no, anahtar)
// ekleme, Option ↔ Result köprüleri, ilk hatada durmak mı hepsini toplamak mı, `Box<dyn Error>` ile özel
// enum arasında seçim, `unwrap`/`expect`'in meşru olduğu yerler.

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Önce main.rs'i oku; bu dosya oradaki parçaları tek bir işte birleştirir. İş şudur: bir ayar
// dosyasını oku, satırlarını anla, değerleri doğrula. Benzetmelerde yine mutfaktayız; ayar
// dosyasını bir TARİF KARTI gibi düşün: her satırı "un = 2 bardak" biçiminde olmalı.
//
// Bu dosyanın tek cümlelik özeti: İyi bir hata "olmadı" demekle kalmaz; NE olmadığını, NEREDE
// olmadığını ve NEDEN olmadığını da söyler.
//
// MİNİ SÖZLÜK (yalnız bu dosyada yeni geçenler; öteki terimler main.rs'in başındaki sözlükte):
//   varyant (variant)      → bir enum'un seçeneklerinden her biri. Hata enum'unda: hata çeşidi.
//   Error (trait)          → "ben bir hata türüyüm" diyen standart söz. Display + Debug ister.
//   source()               → "bu hataya yol açan alttaki hata hangisi?" sorusunun cevabı.
//   bağlam (context)       → hatanın yanına eklenen yer bilgisi: hangi satır, hangi anahtar.
//   map_err                → Err'in içindeki hatayı başka bir hataya çevirmek; Ok'a dokunmaz.
//   ok_or / ok_or_else     → Option'ı Result'a çevirmek: None ise verdiğin hata konur.
//   .ok()                  → Result'ı Option'a çevirmek: hata varsa nedeni atılır, None kalır.
//   transpose              → iç içe iki kutunun (Result ile Option) sırasını değiştirmek.
//   closure                → `|x| ...` biçiminde yazılan adsız, küçük fonksiyon (Böl. 13).
//   iterator, collect      → elemanları sırayla veren şey / o elemanları bir kapta toplamak.
//   BTreeMap               → anahtar → değer eşleştiren sözlük; anahtarları sıralı tutar.
//   generic (T), where     → "herhangi bir tür" için yazılmış kod / o türden beklenen şartlar.
//   Box<dyn Error>         → "Error sözünü veren herhangi bir hata" koyulabilen kutu.
//   trait object (dyn)     → asıl türü unutulmuş, yalnız verdiği söz bilinen değer.
//   downcast_ref           → o kutuya "içindeki şu tür mü?" diye sormak.
//   API, sözleşme          → fonksiyonun dışarıya verdiği söz: ne alır, ne döner, hangi hata çıkar.
//   literal                → koda elle yazılmış sabit değer: "127.0.0.1", 42 gibi.
//   turbofish (::<T>)      → türü çağrı sırasında elle söylemek: `parse::<i32>()`.
//   tahsis (allocation)    → heap'ten yer istemek; bedava değildir, gereksizse kaçınılır.
//   bug                    → programcının kodda yaptığı yanlış.
//   prototip, betik        → denemek için yazılmış kısa ömürlü kod / küçük yardımcı program.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::num::ParseIntError;
use std::path::Path;
use std::str::FromStr;

pub fn calistir() {
    println!("\n== Derinleşme: Error Handling ==");
    ozel_hata_ve_zincir();
    option_result_kopruleri();
    ilk_hata_mi_hepsi_mi();
    dosya_senaryosu();
    box_dyn_error_ne_zaman();
    unwrap_ne_zaman_mesru();
}

// ============================================================================
// 1. KENDİ HATA TÜRÜN — enum + Display + Error (source zinciri) + From
// ============================================================================
// Bu konuyu neden öğreniyoruz? main.rs'teki `Result<i32, String>` küçük örnekte yeter ama gerçek
// kodda yetmez: çağıran, String'in İÇİNE bakmadan "dosya mı yok, biçim mi bozuk?" ayrımını
// yapamaz. Enum ise her başarısızlık biçimini AYRI bir varyant yapar; çağıran `match` ile karar
// verir ve yeni varyant eklendiğinde derleyici eksik kolu hatırlatır (bütünlük denetimi — exhaustiveness, Böl. 6).
// SADE ANLATIM: Tarif kartından kek yapacaktın ve olmadı. "Olmadı" demek yetmez; yardım edecek
// kişi nedenini bilmek ister: kart mı kayıp, bir satır mı okunmuyor, "3 yumurta" yerine "üç
// tane işte" mi yazıyor, yoksa şeker satırı hiç mi yok? Dört ayrı sorun, dört ayrı çözüm.
// Rust'ta her sorun için enum'a ayrı bir varyant (çeşit) yazarsın; çağıran `match` ile hangisi
// olduğuna bakar. Neden böyle? Hata düz bir yazı olsaydı, çağıranın o yazıyı okuyup tahmin
// yürütmesi gerekirdi; yazıda bir harf değişince tahmin bozulurdu.
//   Mini örnek:
//     enum TarifHatasi { KartYok, SatirBozuk { satir: usize }, MalzemeEksik(&'static str) }
//     match tarifi_oku() {
//         Err(TarifHatasi::KartYok) => println!("ezberden yaparım"),   // ✓ çeşide göre karar
//         Err(TarifHatasi::SatirBozuk { satir }) => println!("{satir}. satırı düzelt"),
//         Err(TarifHatasi::MalzemeEksik(ad)) => println!("{ad} al"),
//         Ok(_tarif) => println!("başlıyorum"),
//     }
//     // Hata String olsaydı: if hata == "kart yok" { .. }   // ✗ kırılgan: yazı değişince bozulur
#[derive(Debug)]
pub enum AyarHatasi {
    Gc(io::Error), // alt katmanın hatasını SARAR (source olarak geri verilecek)
    Bicim {
        satir: usize, // bağlam: kullanıcı hangi satırı düzelteceğini bilsin
        icerik: String,
    },
    Sayi {
        anahtar: &'static str,
        deger: String,
        kaynak: ParseIntError, // asıl neden; Display'de DEĞİL, source()'ta
    },
    Eksik(&'static str),
}

// SADE ANLATIM: Aynı sorunu iki kişiye iki ayrı dille anlatırsın. Misafire "kek biraz gecikecek"
// dersin (Display, `{}`); mutfaktaki yardımcına ise "fırın 180 dereceye çıkmıyor" dersin
// (Debug, `{:?}`). Rust'ta bir hata türü ikisini de sunar. Neden böyle? Kullanıcı kısa ve
// anlaşılır bir cümle ister, geliştirici ise bütün ayrıntıyı.
//   Mini örnek (hata = bozuk bir satırdan çıkan AyarHatasi::Bicim):
//     println!("{hata}");     // satır 2: `anahtar=değer` bekleniyordu, ...   → kullanıcıya
//     println!("{hata:?}");   // Bicim { satir: 2, icerik: "..." }            → geliştiriciye
// Display = SON KULLANICIYA gösterilecek tek satır. Debug ({:?}) ise geliştirici içindir.
impl fmt::Display for AyarHatasi {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // İşletim sisteminin mesajı platforma göre değişir; çıktı her ortamda aynı olsun diye
            // burada yalnız türünü (ErrorKind) basıyoruz. Tam mesaj source() üzerinden erişilebilir.
            AyarHatasi::Gc(e) => write!(f, "ayar dosyası okunamadı ({:?})", e.kind()),
            AyarHatasi::Bicim { satir, icerik } => {
                write!(
                    f,
                    "satır {satir}: `anahtar=değer` bekleniyordu, bulunan {icerik:?}"
                )
            }
            AyarHatasi::Sayi { anahtar, deger, .. } => {
                write!(f, "`{anahtar}` sayı olmalı, bulunan {deger:?}")
            }
            AyarHatasi::Eksik(anahtar) => write!(f, "zorunlu anahtar eksik: {anahtar}"),
        }
    }
}

// SADE ANLATIM: "Kek olmadı." Neden? "Çünkü fırın ısınmadı." Neden? "Çünkü fişi takılı değildi."
// Her cevabın arkasında bir önceki neden durur; sordukça köke inersin. `source()` bu "neden?"
// sorusudur: bir hata, kendisine yol açan alttaki hatayı gösterir; o da varsa kendininkini.
// Neden böyle? Yalnız en üstteki cümleyi görürsen ("kek olmadı") neyi düzelteceğini bilemezsin;
// yalnız en alttakini görürsen ("fiş") hangi işin bozulduğunu bilemezsin.
//   Mini örnek:
//     let hata = ayarlari_coz("port = seksen\nazami_baglanti = 64").unwrap_err();
//     println!("{hata}");                // `port` sayı olmalı, bulunan "seksen"
//     println!("{:?}", hata.source());   // Some(..): alttaki asıl neden, sayıya çevirme hatası
//     // AyarHatasi::Eksik(..) için source() None döner: altında başka bir hata yoktur.
// Error trait'i `Debug + Display` ister (ikisi de yukarıda var); gövdesi boş bile olabilir. Tek
// ilginç metodu source(): "bu hatanın ALTINDAKİ neden ne?" Böylece hatalar bağlı liste gibi
// zincirlenir. Kural: bir hatayı source() ile veriyorsan Display'inde TEKRAR yazma; yoksa zinciri
// yürüyen raporlayıcı aynı mesajı iki kez basar.
impl Error for AyarHatasi {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AyarHatasi::Gc(e) => Some(e),
            AyarHatasi::Sayi { kaynak, .. } => Some(kaynak),
            AyarHatasi::Bicim { .. } | AyarHatasi::Eksik(_) => None,
        }
    }
}

// SADE ANLATIM: Marketten gelen "yok" notunu listeye olduğu gibi geçirmek kolaydır (From). Ama
// "yok" tek başına işe yaramaz: NE yok? Notu taşıyan bunu bilmez; listeyi tutan sen bilirsin ve
// "yumurta yok" diye tamamlarsın. Rust'ta bu tamamlamayı `map_err` yapar: hatayı alır, yanına
// senin bildiğin bilgiyi (hangi satır, hangi anahtar) koyar. Neden böyle? From yalnız hatanın
// kendisini görür; hatanın nerede çıktığını bilen tek yer, onu yakalayan koddur.
//   Mini örnek:
//     let n: u16 = ham.parse()?;   // yalnız From ile (yazılmış olsaydı): "geçersiz rakam" der,
//                                  // hangi anahtarda olduğunu söyleyemez
//     let n: u16 = ham.parse().map_err(|kaynak| AyarHatasi::Sayi {
//         anahtar: "port",         // ✓ bağlam eklendi: sorun `port` anahtarında
//         deger: ham.clone(),
//         kaynak,                  // asıl hata da saklanır; source() onu geri verir
//     })?;
// From, `?`'in otomatik dönüşümünü açar (main.rs §4). SINIRI: From yalnız hatanın KENDİSİNİ görür;
// "hangi satır, hangi anahtar" bilgisini ekleyemez. Bağlam gereken yerde From değil `map_err`.
impl From<io::Error> for AyarHatasi {
    fn from(e: io::Error) -> Self {
        AyarHatasi::Gc(e)
    }
}

// (Sade: aşağıdaki fonksiyon "neden?" sorusunu cevap bitene kadar sorar ve aldığı cevapları
// aralarına " <- " koyarak yan yana yazar.)
// Zinciri yürüyen küçük raporlayıcı: en üstteki mesajdan kök nedene doğru iner.
// (`successors`: "bir öncekinden bir sonrakini üret" iterator'ı — Böl. 13 ile bağlantı.)
fn zinciri_yaz(hata: &(dyn Error + 'static)) -> String {
    std::iter::successors(Some(hata), |&h| h.source())
        .map(|h| h.to_string())
        .collect::<Vec<_>>()
        .join(" <- ")
}

#[derive(Debug, PartialEq)]
pub struct Ayarlar {
    ad: String,
    port: u16,
    azami_baglanti: u32,
}

// SADE ANLATIM: Tarif kartındaki bir satıra bakınca üç şeyden biri olur: satır boştur (sorun
// yok, geç), "un = 2 bardak" gibi düzgün bir satırdır (al, kullan) ya da "un işte biraz" gibi
// anlaşılmaz bir satırdır (hata). "Boş" ile "bozuk" aynı şey değildir. Rust'ta bu üçlüyü iç içe
// iki kutu anlatır: dıştaki Result "bozuk mu?", içteki Option "bir şey var mı?" sorusunu
// cevaplar. Neden böyle? Boş satır da hata sayılsaydı, arasında boşluk olan her kart reddedilirdi.
//   Mini örnek:
//     satir_ayristir(1, "")          // Ok(None): boş satır, hata değil
//     satir_ayristir(2, "un = 2")    // Ok(Some(("un", "2"))): düzgün çift
//     satir_ayristir(3, "un işte")   // Err(Bicim { satir: 3, .. }): `=` yok
// Adım adım kod analizi — tek bir satır:
//   Ok(None)        → boş satır/yorum: hata DEĞİL, sadece "veri yok"
//   Ok(Some((k,v))) → geçerli çift
//   Err(Bicim{..})  → `=` yok
// Üç durumu tek dönüş türünde ayırmak için Result<Option<_>, _> kullanıyoruz.
fn satir_ayristir(no: usize, satir: &str) -> Result<Option<(String, String)>, AyarHatasi> {
    let satir = satir.trim();
    if satir.is_empty() || satir.starts_with('#') {
        return Ok(None);
    }
    // split_once Option döner. ok_or_else onu Result'a çevirir; closure YALNIZ None olursa çalışır
    // (ok_or olsaydı `to_string` tahsisi başarılı her satırda da boşuna yapılırdı).
    let (anahtar, deger) = satir.split_once('=').ok_or_else(|| AyarHatasi::Bicim {
        satir: no,
        icerik: satir.to_string(),
    })?;
    Ok(Some((anahtar.trim().to_string(), deger.trim().to_string())))
}

// (Sade: aşağıdaki fonksiyon kartı satır satır okur, boş satırları atlar, düzgün satırları bir
// tabloya koyar; bozuk bir satır görürse orada durur ve o hatayı döndürür. `transpose`, iç içe iki
// kutunun sırasını değiştirir: "Result içinde Option" → "Option içinde Result".)
// İLK HATADA DUR: `collect`, Result'lardan oluşan bir akışı Result<Koleksiyon, E>'ye toplayabilir;
// ilk Err'de akışı keser ve o Err'i döndürür. transpose: Result<Option<T>, E> → Option<Result<T, E>>;
// böylece filter_map `Ok(None)`'ları (boş satırlar) atar ama Err'leri KAYBETMEZ.
fn tabloya_ayristir(metin: &str) -> Result<BTreeMap<String, String>, AyarHatasi> {
    metin
        .lines()
        .enumerate()
        .filter_map(|(i, satir)| satir_ayristir(i + 1, satir).transpose())
        .collect()
}

// (Sade: aşağıdaki `T: FromStr<Err = ParseIntError>` şartı, "T, yazıdan üretilebilen ve
// üretilemeyince sayı hatası veren herhangi bir tür" demek. Böylece aynı fonksiyon hem u16 hem
// u32 okur; hangisi olacağını çağrıldığı yerdeki alanın türü belirler.)
// Option → Result (ok_or) ve hata türü değiştirme (map_err) aynı fonksiyonda. map_err'in closure'ı
// ParseIntError'ı alır ve çevredeki `anahtar`/`ham` bilgisini ekleyerek KENDİ hatamıza sarar.
fn sayi_al<T>(tablo: &BTreeMap<String, String>, anahtar: &'static str) -> Result<T, AyarHatasi>
where
    T: FromStr<Err = ParseIntError>,
{
    let ham = tablo.get(anahtar).ok_or(AyarHatasi::Eksik(anahtar))?;
    ham.parse().map_err(|kaynak| AyarHatasi::Sayi {
        anahtar,
        deger: ham.clone(),
        kaynak,
    })
}

pub fn ayarlari_coz(metin: &str) -> Result<Ayarlar, AyarHatasi> {
    let tablo = tabloya_ayristir(metin)?;
    Ok(Ayarlar {
        // İsteğe bağlı alan: yokluğu hata değil → Option'da kal, varsayılanla kapat.
        ad: tablo
            .get("ad")
            .cloned()
            .unwrap_or_else(|| String::from("adsız")),
        port: sayi_al(&tablo, "port")?, // u16'ya sığmayan değer de ParseIntError üretir
        azami_baglanti: sayi_al(&tablo, "azami_baglanti")?,
    })
}

// SADE ANLATIM: İş iki adımdır: önce kartı çekmeceden çıkarırsın (dosyayı oku), sonra
// satırlarını anlarsın (ayrıştır). Çekmece boşsa ikinci adıma hiç geçilmez. Aşağıdaki fonksiyon bu
// iki adımı art arda yapar; hangisi bozulursa bozulsun dışarıya aynı türden hata çıkar. Neden
// böyle? Çağıran tek bir hata türüyle uğraşır ve imzaya bakınca neler olabileceğini görür.
//   Mini örnek:
//     let metin = fs::read_to_string(yol)?;   // dosya yoksa: io::Error → AyarHatasi::Gc, dön
//     ayarlari_coz(&metin)                    // dosya varsa: satırları anlamaya geç
// Üst katman: io::Error burada `?` + From ile AyarHatasi::Gc olur; ayrıştırma hataları zaten bizim
// türümüzde. İmzaya bakan, bu fonksiyonun nasıl başarısız olabileceğini eksiksiz görür.
pub fn dosyadan_yukle(yol: &Path) -> Result<Ayarlar, AyarHatasi> {
    let metin = fs::read_to_string(yol)?;
    ayarlari_coz(&metin)
}

fn ozel_hata_ve_zincir() {
    println!("-- 1. özel hata türü ve source zinciri --");
    let gecerli = "# sunucu\nad = deneme\nport = 8080\nazami_baglanti = 64\n";
    println!("geçerli: {:?}", ayarlari_coz(gecerli));
    for bozuk in [
        "port = 8080\nazami baglanti 64",
        "port = seksen\nazami_baglanti = 64",
        "port = 8080",
        "port = 99999\nazami_baglanti = 1",
    ] {
        if let Err(e) = ayarlari_coz(bozuk) {
            println!("hata: {}", zinciri_yaz(&e));
        }
    }
    // Beklenen çıktı:
    //   geçerli: Ok(Ayarlar { ad: "deneme", port: 8080, azami_baglanti: 64 })
    //   hata: satır 2: `anahtar=değer` bekleniyordu, bulunan "azami baglanti 64"
    //   hata: `port` sayı olmalı, bulunan "seksen" <- invalid digit found in string
    //   hata: zorunlu anahtar eksik: azami_baglanti
    //   hata: `port` sayı olmalı, bulunan "99999" <- number too large to fit in target type
}

// ============================================================================
// 2. Option ↔ Result KÖPRÜLERİ
// ============================================================================
// Nasıl çalışır? Option "yok olabilir" der, Result "NEDEN olmadığını" da söyler. Geçişler:
//   Option → Result : ok_or(hata) / ok_or_else(|| hata)   ("yokluk" burada bir hatadır)
//   Result → Option : .ok()                                (nedeni bilerek ATIYORSUN)
//   Result<Option> ↔ Option<Result> : transpose()
// `?` Option üzerinde de çalışır ama yalnız Option dönen fonksiyonda (karıştırma: bkz. §7).
// SADE ANLATIM: Dolabı açtın, yumurta yok. Boş raf sana yalnız "yok" der (Option: None). Kek
// yapacaksan bu bir sorundur ve adını koyman gerekir: "yumurta olmadığı için kek yapılamadı"
// (Result: Err). `ok_or` boş rafa bir neden yapıştırır. Tersi de olur: nedeni önemsemiyorsan
// `.ok()` ile notu atar, yalnız "var mı, yok mu" bilgisini tutarsın. Neden iki ayrı kutu?
// Yokluk bazen hata değildir (isteğe bağlı vanilya); hata olduğunda ise nedeni kaybolmamalıdır.
//   Mini örnek:
//     let raf: Option<u32> = None;
//     let sonuc = raf.ok_or("yumurta yok");   // Err("yumurta yok"): yokluğa neden eklendi
//     let a = "x1".parse::<u16>().ok();       // None: hata vardı, nedeni bilerek atıldı
//     let b = "12".parse::<u16>().ok();       // Some(12): hata yoktu, değer aynen kaldı
fn ilk_anahtar(metin: &str) -> Option<&str> {
    let satir = metin.lines().find(|s| !s.trim().is_empty())?; // None → fonksiyondan None
    let (anahtar, _) = satir.split_once('=')?;
    Some(anahtar.trim())
}

fn option_result_kopruleri() {
    println!("-- 2. Option <-> Result --");
    println!(
        "ilk_anahtar = {:?} / {:?}",
        ilk_anahtar("\nport = 1"),
        ilk_anahtar("bozuk")
    );
    let yok: Option<u16> = None;
    println!(
        "ok_or      = {:?}",
        yok.ok_or(AyarHatasi::Eksik("port"))
            .map_err(|e| e.to_string())
    );
    println!("ok()       = {:?}", "x1".parse::<u16>().ok()); // hata ayrıntısı bilinçli atıldı
    // Beklenen çıktı:
    //   ilk_anahtar = Some("port") / None
    //   ok_or      = Err("zorunlu anahtar eksik: port")
    //   ok()       = None
}

// ============================================================================
// 3. İLK HATADA DUR mu, HEPSİNİ TOPLA mı?
// ============================================================================
// Gerçek hayattan örnek: derleyici ilk hatada durmaz, bulabildiği kadarını listeler; çünkü
// kullanıcı düzelt-çalıştır döngüsünü her hata için tekrarlamak istemez. Ayar dosyası doğrulayıcı
// da böyle olmalı. Buna karşın bir ödeme işleminde ilk hatada durmak (yarım iş yapmamak) doğrudur.
// SADE ANLATIM: Kardeşinin yazdığı alışveriş listesini kontrol ediyorsun. İki yol var: ilk
// yanlışı görünce listeyi geri vermek ya da tamamını okuyup bütün yanlışları işaretlemek. Liste
// düzeltilecekse ikincisi iyidir: kardeşin her yanlış için ayrı ayrı gidip gelmez. Yemeği
// pişirirken ise ilk bozuk malzemede durmak gerekir; yarım pişmiş yemek kimsenin işine yaramaz.
// Rust'ta `?` ve `collect` ilk hatada durur; hepsini görmek istersen hataları bir `Vec`te
// kendin biriktirirsin.
//   Mini örnek (dört satırlık metin: "a=1", "bozuk", "b=2", "yine bozuk"):
//     tabloya_ayristir(metin)   // Err(satır 2 ..): ilk hatada durdu, 3. ve 4. satıra bakmadı
//     hatalari_topla(metin)     // (2, [satır 2 hatası, satır 4 hatası]): sona kadar gitti
fn hatalari_topla(metin: &str) -> (usize, Vec<AyarHatasi>) {
    let mut gecerli = 0;
    let mut hatalar = Vec::new();
    for (i, satir) in metin.lines().enumerate() {
        match satir_ayristir(i + 1, satir) {
            Ok(Some(_)) => gecerli += 1,
            Ok(None) => {}
            Err(e) => hatalar.push(e), // durma; kaydet ve devam et
        }
    }
    (gecerli, hatalar)
}

fn ilk_hata_mi_hepsi_mi() {
    println!("-- 3. ilk hatada dur / hepsini topla --");
    let metin = "port = 1\nbozuk satır\nad = x\nbir bozuk daha";
    let ilk = tabloya_ayristir(metin).map(|t| t.len());
    println!("collect (ilk hata): {:?}", ilk.map_err(|e| e.to_string()));
    let (gecerli, hatalar) = hatalari_topla(metin);
    println!("toplama: {gecerli} geçerli, {} hata", hatalar.len());
    for e in &hatalar {
        println!("  - {e}");
    }
    // Beklenen çıktı:
    //   collect (ilk hata): Err("satır 2: `anahtar=değer` bekleniyordu, bulunan \"bozuk satır\"")
    //   toplama: 2 geçerli, 2 hata
    //     - satır 2: `anahtar=değer` bekleniyordu, bulunan "bozuk satır"
    //     - satır 4: `anahtar=değer` bekleniyordu, bulunan "bir bozuk daha"
}

// ============================================================================
// 4. GERÇEK DOSYA — yaz, oku, sil (geçici dizinde; her ortamda çalışır)
// ============================================================================
// Dosyayı işi bitince silen küçük yardımcı. `is` closure'ı başarısız olsa bile (Err DÖNSE bile)
// silme satırına ulaşılır; çünkü Err bir DEĞERDİR, akışı kesmez. Panik etseydi ulaşılmazdı —
// o garanti için Drop'lu bir koruyucu (RAII guard) gerekir (Böl. 15).
// SADE ANLATIM: Tezgâhı ödünç aldın: hamuru açarsın, işin bitince tezgâhı silersin. Hamur
// yırtılsa da ("olmadı" desen de) tezgâhı silmen gerekir. Aşağıdaki yardımcı böyle çalışır:
// dosyayı oluşturur, senin verdiğin işi çalıştırır, iş başarılı da olsa `Err` de dönse dosyayı
// siler. Neden çalışıyor? `Err` sıradan bir değerdir; onu elinde tutar, bir sonraki satıra
// geçersin. Panik ise sonraki satırları atlar; bu yüzden panikte buradaki silme yapılmaz.
//   Mini örnek:
//     let sonuc = is(&yol);      // Ok da olabilir, Err de; ikisi de yalnızca bir değer
//     fs::remove_file(&yol)?;    // ✓ iki durumda da bu satıra gelinir: dosya silinir
//     Ok(sonuc)                  // işin sonucu, dosya silindikten sonra geri verilir
// (Sade: `is: impl FnOnce(&Path) -> T`, "dışarıdan verilen, bir dosya yolu alan ve en fazla bir
// kez çağrılacak iş" demek. "RAII guard" ise işi bitince temizliği kendiliğinden yapan değerdir.)
fn gecici_dosya_ile<T>(etiket: &str, icerik: &str, is: impl FnOnce(&Path) -> T) -> io::Result<T> {
    let ad = format!("rust_study_ayar_{}_{etiket}.conf", std::process::id());
    let yol = std::env::temp_dir().join(ad);
    fs::write(&yol, icerik)?;
    let sonuc = is(&yol);
    fs::remove_file(&yol)?;
    Ok(sonuc)
}

fn dosya_senaryosu() {
    println!("-- 4. dosyadan yükleme --");
    let icerik = "ad = üretim\nport = 443\nazami_baglanti = 1000\n";
    // (Sade: aşağıda iç içe iki sonuç var. Dıştaki "geçici dosya işi oldu mu?", içteki "ayarlar
    // okunabildi mi?" sorusunun cevabıdır; bu yüzden `Ok(Ok(..))` ve `Ok(Err(..))` kolları var.)
    // Uygulamanın EN DIŞ katmanı: hata burada son kez ele alınır (yazdır / varsayılana dön / çık).
    match gecici_dosya_ile("calistir", icerik, dosyadan_yukle) {
        Ok(Ok(ayarlar)) => println!(
            "yüklendi: {} :{} (azami {})",
            ayarlar.ad, ayarlar.port, ayarlar.azami_baglanti
        ),
        Ok(Err(e)) => println!("ayar hatası: {}", zinciri_yaz(&e)),
        Err(e) => println!("geçici dosya kullanılamadı: {:?}", e.kind()),
    }
    // Olmayan dosya: çağıran VARYANTA bakıp karar verir — String hata ile bu mümkün olmazdı.
    let olmayan = std::env::temp_dir().join("rust_study_boyle_bir_dosya_yok.conf");
    match dosyadan_yukle(&olmayan) {
        Err(AyarHatasi::Gc(e)) if e.kind() == io::ErrorKind::NotFound => {
            println!("dosya yok → varsayılan ayarlarla devam")
        }
        diger => println!("beklenmeyen: {diger:?}"),
    }
    // Beklenen çıktı:
    //   yüklendi: üretim :443 (azami 1000)
    //   dosya yok → varsayılan ayarlarla devam
}

// ============================================================================
// 5. Box<dyn Error> NE ZAMAN, ÖZEL ENUM NE ZAMAN?
// ============================================================================
// Box<dyn Error>, Error uygulayan HER türü kabul eden trait object'tir (Böl. 18): `?` her hatayı
// kutulayıp geçirir, hatta `&str`/`String` bile `.into()` ile hataya dönüşür. Bedeli: tür SİLİNİR;
// çağıran `match` yapamaz, ancak downcast_ref ile "şu tür müydü?" diye sorabilir.
//   • KÜTÜPHANE (başkası çağıracak, hataya göre davranacak) → özel enum: sözleşme imzada.
//   • UYGULAMA / main / betik / prototip (hata yalnız raporlanacak) → Box<dyn Error> yeter.
// Yaygın kalıp ikisinin birleşimidir: iç katmanlar enum, en dıştaki `fn main() -> Result<(), Box<dyn Error>>`.
// SADE ANLATIM: Mutfakta iki çeşit kap vardır. Etiketli kavanozlar: üstünde "un", "şeker" yazar,
// bakınca içindekini bilirsin. Bir de "karışık" yazan büyük kutu: içine her şeyi atabilirsin,
// ama çıkarırken ne çıkacağını bilemezsin; ancak "bu şeker mi?" diye sorabilirsin. Özel hata
// enum'u etiketli kavanozdur; `Box<dyn Error>` karışık kutudur. Neden ikisi de var? Hataya göre
// farklı davranacaksan etiket gerekir; yalnız ekrana yazıp çıkacaksan karışık kutu yeter ve
// daha az kod ister.
//   Mini örnek:
//     fn hizli() -> Result<usize, Box<dyn Error>> {
//         let n = "12".parse::<usize>()?;         // ✓ sayı hatası çıkarsa kutuya girer
//         let metin = fs::read_to_string("a")?;   // ✓ dosya hatası da aynı kutuya girer
//         Ok(n + metin.len())
//     }
//     // Çağıran `match` ile çeşidine bakamaz; e.downcast_ref::<ParseIntError>() ile sorabilir.
fn hizli_toplam(metin: &str) -> Result<u32, Box<dyn Error>> {
    let mut toplam = 0;
    for parca in metin.split(',') {
        toplam += parca.trim().parse::<u32>()?; // ParseIntError → Box<dyn Error> (std'deki From)
    }
    if toplam == 0 {
        return Err("toplam sıfır olamaz".into()); // &str → Box<dyn Error>
    }
    Ok(toplam)
}

fn box_dyn_error_ne_zaman() {
    println!("-- 5. Box<dyn Error> --");
    println!(
        "hizli_toplam(\"1, 2, 3\") = {:?}",
        hizli_toplam("1, 2, 3").ok()
    );
    for girdi in ["1, iki", "0, 0"] {
        if let Err(e) = hizli_toplam(girdi) {
            let sayi_hatasi_mi = e.downcast_ref::<ParseIntError>().is_some();
            println!("{girdi:?} → {e} (ParseIntError mı? {sayi_hatasi_mi})");
        }
    }
    // Beklenen çıktı:
    //   hizli_toplam("1, 2, 3") = Some(6)
    //   "1, iki" → invalid digit found in string (ParseIntError mı? true)
    //   "0, 0" → toplam sıfır olamaz (ParseIntError mı? false)
}

// ============================================================================
// 6. unwrap / expect NE ZAMAN MEŞRU? — panic mi, Result mı: karar ölçütü
// ============================================================================
// Ölçüt tek soru: "Bu başarısızlığın sorumlusu KİM?"
//   • Dış dünya (kullanıcı girdisi, dosya, ağ) → beklenen durum → Result; çağıran karar versin.
//   • Programcı (bozulmuş invariant, "buraya asla gelinmez") → hata bir BUG → panic; sessizce
//     devam etmek bozuk veriyle çalışmak demektir.
// unwrap/expect'in meşru yerleri: (a) testler ve örnek kod, (b) prototip (sonra `?`'e çevrilir),
// (c) derleyicinin göremediği ama SENİN kanıtlayabildiğin durumlar — expect mesajına o kanıtı yaz.
// SADE ANLATIM: Misafir "ben fıstık yiyemem" derse bu senin hatan değildir; beklenen bir
// durumdur, menüyü değiştirirsin (Result). Ama tarifi sen yazdın ve "tuz" yerine "şeker"
// yazdıysan yanlış sendedir; yemeği öyle servis etmek yerine durup tarifi düzeltmek gerekir
// (panic). `unwrap`/`expect`, "burada hata çıkarsa yanlış benim kodumdadır" dediğin yerler
// içindir. Neden bu ayrım? Dışarıdan gelen sorun ele alınır; koddaki yanlış saklanmaz, düzeltilir.
//   Mini örnek:
//     let yas: u8 = girdi.parse()?;            // ✓ kullanıcının ne yazacağı bilinmez: Result
//     let ip: Ipv4Addr = "127.0.0.1".parse().expect("sabit doğru yazıldı");   // ✓ yazan sensin
//     let yas: u8 = girdi.parse().unwrap();    // ✗ derlenir ama yanlış girdide program panikler
fn unwrap_ne_zaman_mesru() {
    println!("-- 6. meşru expect --");
    // Literal sabit: geçersizse bu bir yazım hatasıdır (bug), kullanıcı hatası değil.
    let yerel: std::net::Ipv4Addr = "127.0.0.1".parse().expect("sabit literal geçerli bir IPv4");
    let sayilar = [3, 1, 2];
    // Dizi boş değil → max() None olamaz; kanıt bir satır yukarıda.
    let en_buyuk = sayilar.iter().max().expect("dizi boş değil");
    println!("yerel = {yerel}, en büyük = {en_buyuk}");
    // Beklenen çıktı:
    //   yerel = 127.0.0.1, en büyük = 3
}

// SADE ANLATIM (aşağıdaki bölüm için): Buradan sonrası "en çok takılınan üç yer" listesidir.
// Üçünün de özü aynıdır: `?` bir hatayı yukarı iletmek ister, ama fonksiyonun dönüş türü o
// hatayı taşıyacak biçimde değildir. Çorbayı tabağa koymak gibi düşün: `?` kepçedir, dönüş türü
// tabaktır; tabak yoksa ya da yanlış kap geldiyse derleyici dökmene izin vermez. Her maddede
// önce bozuk (✗), sonra düzeltilmiş (✓) hâl var. Hata numaraları ilk okuyuşta önemli değildir.
// ============================================================================
// 7. SIK YAPILAN HATALAR — yanlış kullanım / doğru kullanım (kodlar rustc ile doğrulandı)
// ============================================================================
//
// (a) `?`'i () dönen fonksiyonda kullanmak (main.rs bunu `main` için anlatmıştı; aynısı HER fn için):
//     ✗ fn oku() { let n: i32 = "5".parse()?; }
//       E0277: the `?` operator can only be used in a function that returns `Result` or `Option`
//     ✓ Ya imzayı Result yap (hatayı yukarı ilet) ya da sınırda ele al:
//       fn oku() { match "5".parse::<i32>() { Ok(n) => .., Err(e) => eprintln!("{e}") } }
//
// (b) From impl'i olmadan `?`:
//     ✗ fn f(s: &str) -> Result<i32, AyarHatasi> { let n = s.parse::<i32>()?; Ok(n) }
//       E0277: `?` couldn't convert the error to `AyarHatasi`
//       (AyarHatasi için From<io::Error> var ama From<ParseIntError> YOK. Türü turbofish
//        yerine `let n: i32 = s.parse()?` ile versen de hata aynıdır.)
//     ✓ Bağlam gerekmiyorsa `impl From<ParseIntError> for AyarHatasi`; gerekiyorsa map_err
//       (yukarıdaki sayi_al tam olarak bunu yapıyor).
//
// (c) Option ile Result'ı `?`'te karıştırmak — `?` dönüş türünün AİLESİNE bakar:
//     ✗ fn f(s: &str) -> Option<i32> { let n: i32 = s.parse()?; Some(n) }
//       E0277: the `?` operator can only be used on `Option`s, not `Result`s, in a function that
//              returns `Option`
//     ✓ fn f(s: &str) -> Option<i32> { let n: i32 = s.parse().ok()?; Some(n) }
//     ✗ fn g(v: &[i32]) -> Result<i32, String> { let n = v.first()?; Ok(*n) }
//       E0277: the `?` operator can only be used on `Result`s, not `Option`s, in a function that
//              returns `Result`
//     ✓ let n = v.first().ok_or("liste boş")?;   // &str → String dönüşümünü `?` From ile yapar
//
// ÖNEMLİ NOKTALAR
//   • Hata türü bir API sözleşmesidir: enum → çağıran ayırt edebilir; Box<dyn Error> → yalnız raporlar.
//   • From = bağlamsız otomatik dönüşüm; map_err = bağlam ekleyerek dönüşüm.
//   • source() ile sardığın hatayı Display'de tekrar yazma.
//   • Hata, en çok bilgiye sahip katmanda ZENGİNLEŞTİRİLİR, karar verebilen katmanda ELE ALINIR.
//
// ============================================================================
// ALIŞTIRMALAR (çözümler verilmedi; `cargo test -p error-handling` ile kendin sına)
// ============================================================================
// 1) `AyarHatasi`'na `Aralik { anahtar, deger, azami }` varyantı ekle; `ayarlari_coz` içinde
//    `azami_baglanti > 10_000` ise bu hatayı döndür.
//    İpucu: Display'e ve source()'a birer kol eklemeden derlenmez — bütünlük denetimi (E0004)
//    sana iki yeri de gösterecek.
//    Beklenen: "azami_baglanti = 20000" girdisi Err(Aralik { .. }) verir; source() None'dır.
// 2) `sayi_al`'a satır numarası bağlamı ekle: tabloyu `BTreeMap<String, (usize, String)>` yap ve
//    Sayi varyantına `satir` alanı koy.
//    İpucu: `satir_ayristir` zaten `no` alıyor; onu değerle birlikte tabloya taşıman yeterli.
//    Beklenen: "port = seksen" için mesaj "satır 1: `port` sayı olmalı, ..." biçiminde başlar.
// 3) `hatalari_topla`'yı döngü yerine iterator ile yaz: `(Vec<_>, Vec<_>)` üreten `partition`
//    kullan, sonra iki yanı `Result::unwrap`/`unwrap_err` ile aç.
//    İpucu: `partition(Result::is_ok)`; bu, §6'daki "kanıtlanabilir unwrap" için iyi bir örnek.
//    Beklenen: §3'teki çıktı değişmez.
//
// BİR SONRAKİ KONUYLA BAĞLANTISI
//   • generics-traits-lifetimes (Böl. 10): `sayi_al<T: FromStr<Err = ParseIntError>>` bir trait bound +
//     associated type kısıtıdır; `Box<dyn Error>` ise oradaki dinamik dispatch'in hata hâlidir.
//   • minigrep (Böl. 12): `run() -> Result<(), Box<dyn Error>>` ve `main`'de `process::exit(1)` —
//     §5'teki "içeride Result, en dışta raporla" kalıbının gerçek bir komut satırı aracındaki hâli.

// (Sade: aşağıdaki `testler` modülü, yukarıdaki fonksiyonların söz verdiği gibi davrandığını
// `cargo test -p error-handling` ile sınar. Testlerin nasıl yazıldığı Böl. 11'de anlatılır.)
#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn gecerli_metin_ayarlara_cozulur() {
        let ayarlar = ayarlari_coz("port=80\nazami_baglanti=5").unwrap();
        let beklenen = Ayarlar {
            ad: String::from("adsız"),
            port: 80,
            azami_baglanti: 5,
        };
        assert_eq!(ayarlar, beklenen);
    }

    #[test]
    fn bicim_hatasi_satir_numarasini_tasir() {
        let hata = ayarlari_coz("port=80\n\nesittir yok").unwrap_err();
        assert!(matches!(hata, AyarHatasi::Bicim { satir: 3, .. }));
    }

    #[test]
    fn sayi_hatasi_kok_nedeni_source_ile_verir() {
        let hata = ayarlari_coz("port=abc\nazami_baglanti=5").unwrap_err();
        assert!(matches!(
            hata,
            AyarHatasi::Sayi {
                anahtar: "port",
                ..
            }
        ));
        let kok = hata.source().expect("Sayi varyantı kaynağını taşır");
        assert!(kok.downcast_ref::<ParseIntError>().is_some());
    }

    #[test]
    fn eksik_anahtar_ayri_bir_varyanttir() {
        let hata = ayarlari_coz("port=80").unwrap_err();
        assert!(matches!(hata, AyarHatasi::Eksik("azami_baglanti")));
        assert!(hata.source().is_none());
    }

    #[test]
    fn hatalari_toplama_ilk_hatada_durmaz() {
        let (gecerli, hatalar) = hatalari_topla("a=1\nbozuk\n# yorum\nb=2\nbozuk2");
        assert_eq!(gecerli, 2);
        assert_eq!(hatalar.len(), 2);
    }

    #[test]
    fn gecici_dosyadan_yukleme_calisir_ve_dosya_silinir() {
        let mut kullanilan_yol = None;
        let sonuc = gecici_dosya_ile("test", "port=1\nazami_baglanti=2", |yol| {
            kullanilan_yol = Some(yol.to_path_buf());
            dosyadan_yukle(yol)
        });
        let ayarlar = sonuc
            .expect("geçici dizin yazılabilir")
            .expect("içerik geçerli");
        assert_eq!((ayarlar.port, ayarlar.azami_baglanti), (1, 2));
        assert!(!kullanilan_yol.expect("closure çağrıldı").exists());
    }

    #[test]
    fn olmayan_dosya_gc_varyantina_donusur() {
        let yol = std::env::temp_dir().join("rust_study_kesinlikle_yok.conf");
        assert!(matches!(dosyadan_yukle(&yol), Err(AyarHatasi::Gc(_))));
    }
}
