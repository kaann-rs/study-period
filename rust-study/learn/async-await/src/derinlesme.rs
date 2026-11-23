// Derinleşme — Bölüm 17: Async/Await ve Future (pekiştirme: gerçekçi senaryo, sık hatalar, alıştırmalar)
// Çalıştır: cargo run -p async-await   |   Test: cargo test -p async-await
//
// main.rs mekanizmayı kurdu: elle yazılmış executor (block_on), elle yazılmış Future'lar ve
// async/await'in neye dönüştüğü. Bu dosya o executor'u KULLANARAK async kodu yazarken verilen
// kararlara bakar:
//   - future'lar tembeldir: çağırmak başlatmaz (ve `.await`i unutmak neden sessiz bir hatadır),
//   - sıralı (`a.await; b.await`) ile eşzamanlı (join) beklemenin farkı — ölçerek,
//   - gerçekçi senaryo: asenkron API istemcisi (ağ gecikmesi benzetimi + `?` ile hata yönetimi),
//   - elle yazılmış `ikisini_bekle` / `hepsini_bekle`: join aslında nedir,
//   - Pin'in gündelik yüzü: pin! mi, Box::pin mi.
// Görev kuyruğu, spawn, gerçek zamanlayıcı, zaman aşımı ve `Send` future'lar →
// learn/advanced-concepts/ileri-eszamanlilik-async. Waker/vtable ve Pin'in iç yüzü → learn/advanced-concepts/ileri-future.
// ✗ örneklerin hata kodları rustc 1.99 ile derlenerek doğrulandı.

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// main.rs mekanizmayı kurdu; bu dosya async kodu yazarken verilen gündelik kararlara bakar. Her
// bölümün üstündeki "SADE ANLATIM" kutusunu önce oku. Benzetmeler yine EVDE, tek kişiyle geçer.
//
// Bu dosyanın tek cümlelik özeti: İş kartı yazmak işi başlatmaz; birbirinden bağımsız işleri art
// arda değil BİRLİKTE beklersen toplam süre en uzun işin süresine iner.
//
// MİNİ SÖZLÜK (yalnız bu dosyada yeni geçenler; öteki terimler main.rs başındaki sözlükte):
//   sıralı bekleme           → bir iş bitmeden ötekine başlamamak (alt alta iki `.await`).
//   eşzamanlı bekleme / join → işleri birlikte başlatıp hepsi bitene dek sırayla yoklamak.
//   birleştirici (combinator) → birkaç Future'ı alıp tek bir Future yapan fonksiyon (join gibi).
//   poll_fn                  → "her bakışta şu closure'ı çalıştır" diye kısa yoldan Future yazmak.
//   Waker::noop              → çalınca hiçbir şey yapmayan zil; durmadan bakan executor'a yeter.
//   tur                      → bu dosyada sürenin yerine sayılan şey: "kaç kez gidip bakıldı".
//   benzetim (simülasyon)    → gerçeğinin yerine konan taklit; burada ağ gecikmesinin taklidi.
//   API istemcisi            → başka bir servise istek gönderip cevabını bekleyen kod.
//   Result ve `?`            → "ya sonuç ya hata" kutusu / hata varsa hemen çağırana ilet.
//   dyn Future / tür silme   → "türü ne olursa olsun, beklenebilen bir şey" diyen ortak kılıf.
//   async move               → bloğun kullandığı verinin sahipliğini Future'ın içine taşımak.
//   iptal (cancel)           → bir Future'ı bitmeden silmek (drop); kalan adımları hiç çalışmaz.
//   yarış (select)           → birkaç işten İLK biteni almak, ötekileri bırakmak.
//   zaman aşımı (timeout)    → "şu kadar sürede bitmezse vazgeç" kuralı.
//   bloklamak (blocking)     → thread'i gerçekten durdurmak; o sırada başka hiçbir iş ilerlemez.
//   Promise                  → JavaScript'teki benzer kavram; farkı, üretilir üretilmez başlaması.

use std::cell::RefCell;
use std::fmt;
use std::future::{Future, poll_fn};
use std::pin::{Pin, pin};
use std::task::{Context, Poll, Waker};

use crate::block_on; // main.rs'teki elle yazılmış executor

pub fn calistir() {
    println!("\n== Derinleşme: async/await pratikte ==");
    tembellik();
    sirali_ve_eszamanli();
    api_istemcisi_senaryosu();
    pin_gundelik_kullanim();
}

// ============================================================================
// 1. NASIL ÇALIŞIR? — Future tembeldir: çağırmak başlatmaz
// ============================================================================
// Diğer dillerden gelenlerin ilk takıldığı yer: JavaScript'te `fetch()` çağrıldığı an istek
// yola çıkar; Promise yalnız SONUCU temsil eder. Rust'ta `getir()` çağrısı hiçbir şey yapmaz;
// dönen Future İŞİN KENDİSİDİR ve biri onu poll edene dek tek satırı bile çalışmaz.
//
// Neden böyle? (1) Sıfır maliyet: arka planda gizli bir görev/tahsis oluşturulmaz; future
// sıradan bir değerdir, istemezsen düşürürsün ve iş hiç yapılmamış olur (iptal = drop).
// (2) Denetim sende: aynı future'ı sıralı mı, eşzamanlı mı, zaman aşımıyla mı koşturacağına
// ÇAĞIRAN karar verir. Bedeli: `.await`i unutmak derleme hatası değil, sadece bir uyarıdır.
// SADE ANLATIM: Buzdolabındaki listeye "çamaşırı yıka" yazmak çamaşırı yıkamaz. Rust'ta async
// fonksiyonu çağırmak da böyledir: elinde yalnız bir iş kartı (Future) olur. İş, biri o kartı
// takip etmeye başlayınca (`.await` ya da `block_on`) çalışır.
// Neden böyle? Kartı ne zaman, hangi sırayla, başka hangi işlerle birlikte yürüteceğine sen karar
// verirsin; vazgeçersen kartı atarsın ve iş hiç yapılmamış olur. Bedeli şudur: `.await` yazmayı
// unutursan kod derlenir, ama o satır hiçbir şey yapmaz.
//   Mini örnek:
//     let kart = kaydet(&gunluk, "merhaba");   // kart hazır; günlük hâlâ boş
//     block_on(kart);                          // ✓ şimdi çalıştı; günlükte 1 kayıt var
//     kaydet(&gunluk, "kayıp");                // derlenir ama HİÇBİR ŞEY yapmaz (yalnız uyarı)
type Gunluk = RefCell<Vec<String>>;

async fn kaydet(gunluk: &Gunluk, mesaj: &str) {
    gunluk.borrow_mut().push(mesaj.to_string());
}

fn tembellik() {
    let gunluk = Gunluk::default();

    let bekleyen = kaydet(&gunluk, "çalıştım"); // future ÜRETİLDİ; gövde çalışmadı
    println!("çağrıdan sonra kayıt sayısı = {}", gunluk.borrow().len()); // 0
    block_on(bekleyen); // şimdi biri poll etti → gövde çalıştı
    println!(
        "block_on'dan sonra kayıt sayısı = {}",
        gunluk.borrow().len()
    ); // 1

    // SIK YAPILAN HATA: `.await`i unutmak.
    //     kaydet(&gunluk, "kayboldum");
    //         warning: unused implementer of `Future` that must be used
    //         note: futures do nothing unless you `.await` or poll them
    // Program derlenir ve ÇALIŞIR; yalnız o satır hiçbir şey yapmaz. Bu uyarıyı hata gibi oku.
    // Değer bekleyen yerde ise derleyici yakalar:
    //     let n: u32 = sayi_getir();      // ✗ E0308: mismatched types (u32 beklenirken Future)
    //     let n: u32 = sayi_getir().await; // ✓
}

// ============================================================================
// 2. SIRALI mı, EŞZAMANLI mı? — `.await` bir "burada bekle" noktasıdır
// ============================================================================
// async fn içinde alt alta yazılan iki `.await` SIRALI çalışır: ikincisi, birincisi bitmeden
// başlamaz bile (tembeldir, henüz kimse poll etmedi). Bu çoğu zaman istenen şeydir — ikinci
// istek birincinin sonucuna bağlıysa başka yolu yoktur. Ama bağımsız işleri sıralı beklemek,
// async'in bütün kazancını çöpe atar: toplam süre, sürelerin TOPLAMI olur.
//
// Eşzamanlı beklemek için future'ları ÖNCE üretir, sonra hepsini BİRLİKTE poll edecek bir
// birleştiriciye veririz (join). Toplam süre, en YAVAŞ olanın süresi olur.
//
// Ölçebilmek için ağ gecikmesini "tur" cinsinden taklit ediyoruz: Gecikme(n), n kez Pending
// dönüp sonra hazır olur. Aşağıdaki mini executor kaç tur poll ettiğini sayar.
// SADE ANLATIM: Çamaşır 3 saat, fırındaki kek 2 saat sürüyor. Önce çamaşırın bitmesini bekleyip
// keki sonra koyarsan 5 saat geçer. Makineyi çalıştırıp hemen ardından keki de fırına koyarsan
// 3 saat geçer: beklemeler üst üste biner. Rust'ta alt alta yazılan iki `.await` birinci yoldur
// (sıralı); ikinci yol için iki kartı önce hazırlar, sonra birlikte bekleyen bir fonksiyona
// (join) verirsin.
// Neden önemli? İkisinde de çalışan tek kişidir (tek thread). Kazanç, BEKLEYEREK geçen işlerde
// olur; baştan sona senin uğraşman gereken işlerde (ağır hesap) olmaz.
//   Mini örnek (async bir fonksiyonun içinde):
//     let a = adim(&g, "A", 3).await;    // sıralı: önce A biter...
//     let b = adim(&g, "B", 2).await;    // ...B ancak şimdi başlar → bekleme 3 + 2
//     let (a, b) = ikisini_bekle(adim(&g, "A", 3), adim(&g, "B", 2)).await;
//                                        // ✓ birlikte: bekleme en uzunu kadar → 3
struct Gecikme {
    kalan: u32,
}

impl Future for Gecikme {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.kalan == 0 {
            Poll::Ready(())
        } else {
            self.kalan -= 1;
            cx.waker().wake_by_ref(); // "bir sonraki turda beni yine yokla"
            Poll::Pending
        }
    }
}

// Tur sayan executor. Gecikme her Pending'de kendini hemen uyandırdığı için park etmeye gerek
// yok; uyandırma bilgisini kullanmayan en basit waker'ı (Waker::noop) veriyoruz. Dönen sayı:
// kök future'ın kaç kez poll edildiği = "geçen süre"nin benzetimi.
// (Sade: bu executor saat tutmaz, "kaç kez gidip baktım" diye sayar; tur sayısı geçen sürenin
// yerine geçer. Hiç uyumadan durmadan baktığı için zile ihtiyacı yoktur: `Waker::noop()`.)
pub fn tur_sayarak_calistir<F: Future>(fut: F) -> (F::Output, u32) {
    let mut fut = pin!(fut);
    let mut cx = Context::from_waker(Waker::noop());
    let mut tur = 0;
    loop {
        tur += 1;
        if let Poll::Ready(deger) = fut.as_mut().poll(&mut cx) {
            return (deger, tur);
        }
    }
}

async fn adim(gunluk: &Gunluk, ad: &str, gecikme: u32) -> u32 {
    gunluk.borrow_mut().push(format!("{ad} başladı"));
    Gecikme { kalan: gecikme }.await;
    gunluk.borrow_mut().push(format!("{ad} bitti"));
    gecikme
}

// ADIM ADIM KOD ANALİZİ — join aslında nedir?
// Sihir yok: iki future'ı aynı poll çağrısında SIRAYLA yoklayan, ikisi de bitince Ready dönen
// bir future. Biri Pending dönünce beklemeden ötekine geçeriz; "eşzamanlılık" tam olarak bu.
// SADE ANLATIM: join, ocaktaki iki tencereye sırayla bakan tek kişidir. Her turda önce birine,
// sonra ötekine bakar. Pişen tencerenin yemeğini kenara koyar (`Option` içine) ve o tencereye
// bir daha bakmaz. İkisi de pişince "hazır" der ve iki yemeği birlikte verir.
// Neden biten işe bir daha bakılmaz? Ready demiş bir Future'ı yeniden poll etmenin sonucu
// belirsizdir; paniklemesi bile mümkündür.
// (Sade: `poll_fn` = "her bakışta şu closure'ı çalıştır" diyerek kısa yoldan Future yazmak.)
//   Mini örnek (olay sırası):
//     tur 1: A'ya bak → Pending    B'ye bak → Pending     → sonuç: Pending
//     tur 2: A'ya bak → Pending    B'ye bak → Ready(y)    → y'yi sakla, sonuç: Pending
//     tur 3: A'ya bak → Ready(x)   B'ye bakma             → ✓ sonuç: Ready((x, y))
pub async fn ikisini_bekle<A: Future, B: Future>(a: A, b: B) -> (A::Output, B::Output) {
    // Future'lar poll edilmek için sabitlenmeli. pin!: değeri bu async fn'in kendi durumuna
    // (state machine alanına) sabitler; heap tahsisi yoktur.
    let mut a = pin!(a);
    let mut b = pin!(b);
    let mut a_sonuc = None;
    let mut b_sonuc = None;
    // poll_fn: bir closure'dan Future üretir; closure her poll'da çağrılır. Elle
    // `impl Future for ...` yazmanın kısa yolu.
    poll_fn(|cx| {
        // Biten future'ı bir daha poll ETMEYİZ (sözleşme: Ready'den sonra poll'un ne
        // yapacağı belirsizdir — panik edebilir ya da sonsuza dek Pending dönebilir; UB değildir). Sonucu Option'da saklıyoruz.
        if a_sonuc.is_none()
            && let Poll::Ready(v) = a.as_mut().poll(cx)
        {
            a_sonuc = Some(v);
        }
        if b_sonuc.is_none()
            && let Poll::Ready(v) = b.as_mut().poll(cx)
        {
            b_sonuc = Some(v);
        }
        match (a_sonuc.take(), b_sonuc.take()) {
            (Some(x), Some(y)) => Poll::Ready((x, y)),
            (x, y) => {
                // Henüz ikisi de hazır değil: aldıklarımızı yerine koy, Pending dön. Waker'ı
                // ayrıca saklamıyoruz çünkü alt future'lar cx'i zaten gördü; hangisi hazır
                // olursa bu ortak görevi uyandıracak.
                a_sonuc = x;
                b_sonuc = y;
                Poll::Pending
            }
        }
    })
    .await
}

fn sirali_ve_eszamanli() {
    // (a) Sıralı: B, A bitmeden başlamaz.
    let gunluk = Gunluk::default();
    let (toplam, tur) = tur_sayarak_calistir(async {
        let a = adim(&gunluk, "A", 3).await;
        let b = adim(&gunluk, "B", 2).await;
        a + b
    });
    println!(
        "sıralı    : {:?} → {tur} tur (gecikme toplamı {toplam})",
        gunluk.borrow()
    );

    // (b) Eşzamanlı: ikisi de başlar, bekleme anlarında sıra ötekine geçer.
    let gunluk = Gunluk::default();
    let ((a, b), tur) =
        tur_sayarak_calistir(ikisini_bekle(adim(&gunluk, "A", 3), adim(&gunluk, "B", 2)));
    println!(
        "eşzamanlı : {:?} → {tur} tur (gecikme toplamı {})",
        gunluk.borrow(),
        a + b
    );
    // Beklenen çıktı:
    //   sıralı    : ["A başladı", "A bitti", "B başladı", "B bitti"] → 6 tur (gecikme toplamı 5)
    //   eşzamanlı : ["A başladı", "B başladı", "B bitti", "A bitti"] → 4 tur (gecikme toplamı 5)
    // Sıralıda süre 3 + 2 (+1 son tur) = 6; eşzamanlıda max(3, 2) (+1) = 4. Dikkat: TEK thread
    // var. Kazanç paralellikten değil, bekleme sürelerinin ÜST ÜSTE binmesinden geliyor —
    // async'in işe yaradığı yer bu yüzden G/Ç'dir, işlemci yoğun hesap değil.
}

// ============================================================================
// 3. GERÇEK HAYATTAN ÖRNEK — asenkron API istemcisi
// ============================================================================
// Senaryo: bir profil sayfası için üç ayrı servise istek atıyoruz (kullanıcı, siparişler,
// öneriler). Ağ yerine gecikmeli sahte bir sunucu kullanıyoruz; kodun ŞEKLİ gerçek bir HTTP
// istemcisiyle (ör. reqwest + tokio) yazılanla aynıdır: async fn + Result + `?`.
// SADE ANLATIM: Akşam yemeği için fırına, manava ve kasaba telefonla sipariş vereceksin. Sıralı
// yol: birini ara, paketi gelene kadar bekle, sonra ötekini ara. Eşzamanlı yol: üçünü de ara,
// sonra paketleri birlikte bekle. Aşağıdaki kod aynı işi iki yolla yapar ve farkı ölçer (10 tura
// karşı 5 tur). `?` işareti de şunu söyler: biri "elimizde yok" derse (Err) beklemeyi bırak ve
// hatayı seni çağırana ilet.
// Neden böyle? Üç istek birbirinden bağımsızdır; art arda beklemek süreleri boşuna toplar.
//   Mini örnek (async bir fonksiyonun içinde):
//     let ad = getir("/kullanici/1").await?;   // ✓ Ok geldi: ad = "Ada", yola devam
//     let yok = getir("/olmayan").await?;      // Err geldi: fonksiyon burada hatayla biter
//     println!("{ad} {yok}");                  // bu satıra hiç gelinmez
#[derive(Debug, PartialEq, Eq)]
pub enum ApiHatasi {
    Bulunamadi(String),
    SunucuHatasi(u16),
}

impl fmt::Display for ApiHatasi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiHatasi::Bulunamadi(yol) => write!(f, "bulunamadı: {yol}"),
            ApiHatasi::SunucuHatasi(kod) => write!(f, "sunucu hatası: {kod}"),
        }
    }
}

// Sahte sunucu: yola göre gecikir ve cevap verir. `&str` parametresi await boyunca ödünçte
// kalır; async fn'in ürettiği future bu referansı İÇİNDE taşır (dönüş türü gizli bir lifetime
// alır). Çağıran, future yaşadıkça `yol`u canlı tutmak zorundadır — derleyici denetler.
pub async fn getir(yol: &str) -> Result<String, ApiHatasi> {
    let (gecikme, cevap) = match yol {
        "/kullanici/1" => (2, Ok("Ada")),
        "/kullanici/1/siparisler" => (4, Ok("3 sipariş")),
        "/kullanici/1/oneriler" => (3, Ok("klavye, fare")),
        "/bakim" => (1, Err(ApiHatasi::SunucuHatasi(503))),
        _ => (1, Err(ApiHatasi::Bulunamadi(yol.to_string()))),
    };
    Gecikme { kalan: gecikme }.await;
    cevap.map(String::from)
}

// N future'ı birlikte bekleyen birleştirici. Sayı çalışma anında belli olduğundan future'ları
// bir Vec'te tutmamız gerekir; her async fn çağrısı FARKLI ve adsız bir tür ürettiği için
// (ve Vec tek tür tuttuğu için) onları `dyn Future`a siliyoruz. dyn boyutsuzdur → Box;
// poll edilecek → Pin. `Pin<Box<dyn Future<Output = T> + 'a>>` kalıbı buradan gelir.
// (Sade: her async fn çağrısı ayrı ve adsız bir tür üretir, bir Vec ise tek tür tutar.
// `Box<dyn Future>` "türü ne olursa olsun, beklenebilen bir şey" diyen ortak kutudur; `Pin` de
// kutunun içindekinin yerinden oynamayacağını söyler. Aşağıdaki uzun tür adı bu ikisinin toplamı.)
pub type KutuluFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

pub async fn hepsini_bekle<T>(mut gorevler: Vec<KutuluFuture<'_, T>>) -> Vec<T> {
    let mut sonuclar: Vec<Option<T>> = gorevler.iter().map(|_| None).collect();
    poll_fn(|cx| {
        for (gorev, sonuc) in gorevler.iter_mut().zip(sonuclar.iter_mut()) {
            if sonuc.is_none()
                && let Poll::Ready(v) = gorev.as_mut().poll(cx)
            {
                *sonuc = Some(v);
            }
        }
        if sonuclar.iter().all(Option::is_some) {
            // Sonuçlar görev SIRASIYLA döner (bitiş sırasıyla değil) — çağıran için öngörülebilir.
            Poll::Ready(sonuclar.iter_mut().map(|s| s.take().unwrap()).collect())
        } else {
            Poll::Pending
        }
    })
    .await
}

// `?` async fn içinde de aynen çalışır: Err gelirse fonksiyon (yani future) o noktada Err ile
// tamamlanır. Burada istekler SIRALI ve hata ilk başarısız istekte keser.
pub async fn profil_sirali(id: u32) -> Result<String, ApiHatasi> {
    let ad = getir(&format!("/kullanici/{id}")).await?;
    let siparis = getir(&format!("/kullanici/{id}/siparisler")).await?;
    let oneri = getir(&format!("/kullanici/{id}/oneriler")).await?;
    Ok(format!("{ad} | {siparis} | {oneri}"))
}

// Aynı iş EŞZAMANLI: üç future önce üretilir, sonra birlikte beklenir. Hatalar toplandıktan
// sonra ele alınır: `collect::<Result<Vec<_>, _>>()` ilk Err'de durur (error-handling crate'i).
pub async fn profil_eszamanli(id: u32) -> Result<String, ApiHatasi> {
    let yollar = [
        format!("/kullanici/{id}"),
        format!("/kullanici/{id}/siparisler"),
        format!("/kullanici/{id}/oneriler"),
    ];
    let gorevler: Vec<KutuluFuture<_>> = yollar
        .iter()
        .map(|yol| Box::pin(getir(yol)) as KutuluFuture<_>)
        .collect();
    let parcalar = hepsini_bekle(gorevler)
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;
    Ok(parcalar.join(" | "))
}

fn api_istemcisi_senaryosu() {
    let (sonuc, tur) = tur_sayarak_calistir(profil_sirali(1));
    println!("sıralı istemci    : {sonuc:?} ({tur} tur)");
    let (sonuc, tur) = tur_sayarak_calistir(profil_eszamanli(1));
    println!("eşzamanlı istemci : {sonuc:?} ({tur} tur)");
    let (sonuc, _) = tur_sayarak_calistir(profil_eszamanli(7));
    match sonuc {
        Ok(p) => println!("profil: {p}"),
        Err(e) => println!("hata: {e}"),
    }
    let (sonuc, _) = tur_sayarak_calistir(getir("/bakim"));
    println!("bakım: {}", sonuc.unwrap_err());
    // Beklenen çıktı:
    //   sıralı istemci    : Ok("Ada | 3 sipariş | klavye, fare") (10 tur)
    //   eşzamanlı istemci : Ok("Ada | 3 sipariş | klavye, fare") (5 tur)
    //   hata: bulunamadı: /kullanici/7
    //   bakım: sunucu hatası: 503
    // 2 + 4 + 3 (+1) = 10 tura karşı max(2, 4, 3) (+1) = 5 tur: aynı sonuç, yarı süre.
}

// ============================================================================
// 4. PIN'İN GÜNDELİK YÜZÜ — pin! mi, Box::pin mi?
// ============================================================================
// `.await` yazdığın sürece Pin'i hiç görmezsin: derleyici alt future'ı senin yerine sabitler.
// Pin ile ancak bir future'ı ELLE poll ettiğinde (kendi birleştiricin, executor'un) ya da
// future'ları bir koleksiyonda sakladığında karşılaşırsın. O zaman iki araç vardır:
//
//   pin!(fut)      → bulunduğun kapsamın/async fn'in içine sabitler. Tahsis YOK; ama değer o
//                    kapsamdan dışarı çıkamaz (döndüremez, Vec'e koyamazsın).
//   Box::pin(fut)  → heap'e koyup sabitler. Bir tahsis maliyeti var; karşılığında kutuyu
//                    istediğin yere taşıyabilirsin (taşınan kutudur, içindeki future değil) ve
//                    `dyn Future`a silebilirsin.
//
// Neden sabitlemek zorunlu? Async fn'in ürettiği state machine, await noktaları arasında kendi
// yerel değişkenlerine referans tutabilir; taşınırsa bu iç referanslar sarkar. poll'un
// `Pin<&mut Self>` istemesi "bu değer artık taşınmayacak" sözünü tür düzeyine çıkarır
// (ayrıntı ve self-referential örnek: ileri-future).
// SADE ANLATIM: Kendi içini gösteren notlarla dolu defteri (main.rs sonundaki resim) yerinden
// oynatmamak için iki yol var. `pin!`: defteri masana bantlarsın; bedavadır, ama defter o masadan
// ayrılamaz. `Box::pin`: defteri bir kasaya koyup kasanın içinde sabitlersin; kasayı istediğin
// yere taşırsın, defter kasanın içinde hiç kımıldamaz. Bedeli bir kasadır (heap'te yer ayırma).
// Neden böyle? Bir Future'a elle bakabilmen (poll) için önce "bu artık taşınmayacak" sözünü
// vermen gerekir. `.await` yazarken bu sözü derleyici senin yerine verir.
//   Mini örnek:
//     let mut a = pin!(getir("/kullanici/1"));    // ✓ bu fonksiyonun içinde sabit; yer ayırma yok
//     let b = Box::pin(getir("/kullanici/1"));    // ✓ heap'te sabit; kutu bir Vec'e konabilir
//     let mut c = getir("/kullanici/1");
//     // c.poll(&mut cx);                         // ✗ derlenmez: sabitlenmemiş Future'a bakılamaz
fn pin_gundelik_kullanim() {
    // ✗ Sabitlemeden poll edilemez:
    //     let mut fut = getir("/kullanici/1");
    //     fut.poll(&mut cx);
    //         // E0599: no method named `poll` found for opaque type `impl Future<...>`
    //         //        help: consider pinning the expression
    let mut cx = Context::from_waker(Waker::noop());

    // ✓ Yığında: pin!
    let mut yigindaki = pin!(getir("/kullanici/1"));
    let ilk_yoklama = yigindaki.as_mut().poll(&mut cx).is_pending();
    println!("pin! ile ilk poll Pending mi? {ilk_yoklama}"); // true (2 tur gecikme var)

    // ✓ Heap'te: Box::pin — kutu taşınabilir, bir Vec'e konabilir.
    let kutudaki: KutuluFuture<_> = Box::pin(getir("/kullanici/1"));
    let mut liste = vec![kutudaki]; // kutu TAŞINDI; içindeki future'ın adresi değişmedi
    let (sonuclar, _) = tur_sayarak_calistir(hepsini_bekle(std::mem::take(&mut liste)));
    println!("Box::pin ile listeden: {sonuclar:?}"); // [Ok("Ada")]
}

// SADE ANLATIM (aşağıdaki bölüm için): Aşağıda dört yaygın hata, bu dosyanın özeti ve üç
// alıştırma var. Hataların en sinsisi sonuncusudur, çünkü derlenir: async kodun içinde
// `thread::sleep` çağırmak, evdeki tek kişinin koltukta gerçekten uyuyakalmasıdır. O uyurken zil
// çalsa da kalkan olmaz, bütün işler durur. Kural: iki `.await` arasında uzun süren bir şey
// yapma. Alıştırmaları ilk okuyuşta atlayabilirsin.
//   Mini örnek:
//     async fn kotu() { std::thread::sleep(sure); }   // derlenir ama herkesi bekletir
//     async fn iyi() { Gecikme { kalan: 3 }.await; }  // ✓ beklerken sırayı başkasına bırakır
// ============================================================================
// SIK YAPILAN HATALAR (derlenmeyen örnekler)
// ============================================================================
// ✗ `.await`i async olmayan fonksiyonda kullanmak:
//       fn main() { let x = sayi_getir().await; }
//           // E0728: `await` is only allowed inside `async` functions and blocks
//   Senkron dünyadan async dünyaya geçiş TEK bir noktada olur: executor'un block_on'u.
//   ✓ `block_on(sayi_getir())` (tokio'da #[tokio::main] bunu senin yerine yazar).
//
// ✗ Yerel değişkeni ödünç alan async bloğu döndürmek:
//       fn f() -> impl Future<Output = usize> { let s = String::from("a"); async { s.len() } }
//           // E0373: async block may outlive the current function, but it borrows `s`
//   Blok tembeldir: gövdesi fonksiyon döndükten SONRA çalışacak, o sırada `s` ölmüş olacak.
//   ✓ `async move { s.len() }` — sahipliği future'ın içine taşı (closure'lardaki move ile aynı).
//
// ✗ Özyineli async fn:
//       async fn say(n: u32) -> u32 { if n == 0 { 0 } else { 1 + say(n - 1).await } }
//           // E0733: recursion in an async fn requires boxing
//   State machine, beklediği alt future'ı KENDİ içinde saklar; kendini içeren tür sonsuz
//   boyutlu olur (smart-pointers'taki özyineli enum ile aynı sorun, aynı çözüm).
//   ✓ Özyineli çağrıyı kutula: `1 + Box::pin(say(n - 1)).await`.
//
// ✗ (Derlenir ama YANLIŞ) async kod içinde thread'i bloklamak:
//       async fn kotu() { std::thread::sleep(Duration::from_secs(1)); }
//   sleep Pending DÖNMEZ; executor'un thread'ini gerçekten uyutur. O bir saniye boyunca aynı
//   thread'deki bütün öteki görevler de durur — eşzamanlılık biter. Kural: `.await`ler
//   arasında uzun süren hiçbir şey olmamalı. Bekleme için runtime'ın async zamanlayıcısını,
//   ağır hesap için ayrı bir thread'i kullan (çalışan örnek: ileri-eszamanlilik-async).
//
// ============================================================================
// ÖNEMLİ NOKTALAR
// ============================================================================
// • Future = henüz yapılmamış iş. Üretmek bedava, poll edilmezse hiç çalışmaz, düşürülürse
//   iptal olur.
// • `.await` sıralıdır. Bağımsız işleri eşzamanlı yürütmek için join türü bir birleştirici
//   gerekir (gerçek projede futures::join!, tokio::join!, JoinSet).
// • Eşzamanlılık ≠ paralellik: yukarıdaki kazançların hepsi tek thread'de elde edildi.
// • Async fn'e verilen referanslar future'ın içinde yaşar; future'ı saklayacak ya da başka
//   göreve verecek isen sahipli değer (ve `async move`) kullan.
// • Pin yalnız elle poll ederken ve future saklarken görünür: kapsam içiyse pin!, taşınacak ya
//   da tür silinecekse Box::pin.
//
// ============================================================================
// ALIŞTIRMALAR
// ============================================================================
// 1) `async fn ilk_biten<A, B, T>(a: A, b: B) -> T where A: Future<Output = T>, B: Future<Output = T>`
//    yaz: hangisi ÖNCE biterse onun sonucunu döndürsün (yarış / "select"). Öteki ne olur?
//    İpucu: `ikisini_bekle`nin iskeletini al; ilk Ready'de hemen Ready dön. Kaybeden future
//    fonksiyon bitince DÜŞER — yani iptal edilir; bunun için ayrı bir API gerekmez.
//    Beklenen: ilk_biten(adim(g, "A", 3), adim(g, "B", 1)) → 1 ve günlükte "A bitti" YOK.
//
// 2) Alıştırma 1'i kullanarak zaman aşımı yaz:
//    `async fn zaman_asimli<F: Future>(f: F, tur: u32) -> Option<F::Output>`.
//    İpucu: f'yi Some'a, Gecikme { kalan: tur }'u None'a eşle (`async { Some(f.await) }`).
//    Beklenen: zaman_asimli(getir("/kullanici/1/siparisler"), 2) → None (4 tur sürer).
//
// 3) `profil_eszamanli`yi, bir istek başarısız olsa bile ötekilerin sonucunu gösterecek
//    biçimde değiştir: `Vec<Result<String, ApiHatasi>>` döndür ve çağıranda başarılı/başarısız
//    diye ikiye ayır.
//    İpucu: Iterator::partition ya da iki ayrı filter_map (closures-iterators crate'i).
//
// Bir sonraki konuyla bağlantısı: burada tek bir kök future'ı elle sürdük. Gerçek bir runtime
// görevleri bir KUYRUKTA tutar, yalnız uyandırılanı poll eder ve zamanlayıcı/G/Ç ile bütünleşir
// → ileri-eszamanlilik-async. Waker'ın ve Pin'in iç yapısı → ileri-future.

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn future_poll_edilmeden_calismaz() {
        let gunluk = Gunluk::default();
        let fut = kaydet(&gunluk, "x");
        assert!(gunluk.borrow().is_empty());
        block_on(fut);
        assert_eq!(*gunluk.borrow(), ["x"]);
    }

    #[test]
    fn dusurulen_future_hic_calismaz() {
        let gunluk = Gunluk::default();
        drop(kaydet(&gunluk, "asla"));
        assert!(gunluk.borrow().is_empty());
    }

    #[test]
    fn sirali_bekleme_sureleri_toplar() {
        let gunluk = Gunluk::default();
        let (_, tur) = tur_sayarak_calistir(async {
            adim(&gunluk, "A", 3).await;
            adim(&gunluk, "B", 2).await;
        });
        assert_eq!(tur, 6);
        assert_eq!(
            *gunluk.borrow(),
            ["A başladı", "A bitti", "B başladı", "B bitti"]
        );
    }

    #[test]
    fn eszamanli_bekleme_en_yavasi_kadar_surer() {
        let gunluk = Gunluk::default();
        let (sonuc, tur) =
            tur_sayarak_calistir(ikisini_bekle(adim(&gunluk, "A", 3), adim(&gunluk, "B", 2)));
        assert_eq!(sonuc, (3, 2));
        assert_eq!(tur, 4);
        assert_eq!(gunluk.borrow()[..2], ["A başladı", "B başladı"]); // iç içe ilerledi
    }

    #[test]
    fn istemci_sirali_ve_eszamanli_ayni_sonucu_verir() {
        let (sirali, sirali_tur) = tur_sayarak_calistir(profil_sirali(1));
        let (eszamanli, eszamanli_tur) = tur_sayarak_calistir(profil_eszamanli(1));
        assert_eq!(sirali, eszamanli);
        assert_eq!((sirali_tur, eszamanli_tur), (10, 5));
    }

    #[test]
    fn soru_isareti_hatayi_yayar() {
        let (sonuc, _) = tur_sayarak_calistir(profil_sirali(9));
        assert_eq!(
            sonuc,
            Err(ApiHatasi::Bulunamadi(String::from("/kullanici/9")))
        );
        assert_eq!(block_on(getir("/bakim")), Err(ApiHatasi::SunucuHatasi(503)));
    }

    #[test]
    fn hepsini_bekle_gorev_sirasini_korur() {
        let gorevler: Vec<KutuluFuture<u32>> = vec![
            Box::pin(async {
                Gecikme { kalan: 5 }.await;
                1
            }),
            Box::pin(async { 2 }),
        ];
        // İkinci görev önce biter ama sonuçlar görev sırasıyla gelir.
        assert_eq!(block_on(hepsini_bekle(gorevler)), [1, 2]);
    }
}
