// BÖLÜM C — Gerçeğe yakın uygulama: asenkron API istemcisi + async kodda yaygın hatalar
//
// calisma_zamani.rs'teki parçaları (baslat, Uyku, zaman_asimi) bir araya getirip gerçek bir
// istemcinin üç temel ihtiyacını yazıyoruz: istekleri EŞZAMANLI atmak, yavaş isteği ZAMAN
// AŞIMIYLA kesmek, geçici hatada YENİDEN DENEMEK. Ağ yerine gecikmeli sahte bir sunucu var;
// kodun şekli tokio + reqwest ile yazılanla aynıdır (karşılıkları yorumlarda).

// ============================================================================
// SADE ANLATIM — önce buradan başla
// ============================================================================
// Ana sözlük main.rs'in başındadır; önce onu oku. Bu dosyanın tek cümlelik özeti: bekleyen
// işleri birlikte yürütmek programı hızlandırır, ama bir görev thread'i uzun süre tutarsa ya
// da kilidi bekleme boyunca bırakmazsa bütün görevler durur.
//
// MİNİ SÖZLÜK (bu dosyaya özgü yeni kelimeler):
//   istemci / sunucu     → soran taraf / cevap veren taraf.
//   API / istek / cevap  → sunucunun dışarıya açtığı kapı / oraya sorulan soru / gelen yanıt.
//   503 / 404            → sunucunun "şu an meşgulüm" / "böyle bir şey yok" demesi.
//   geçici / kalıcı hata → biraz sonra düzelebilecek hata / tekrar denemenin işe yaramadığı hata.
//   yeniden deneme       → geçici hatada aynı isteği tekrar göndermek (retry).
//   backoff              → her denemeden önce biraz daha uzun beklemek.
//   pano (dashboard)     → birkaç kaynaktan gelen bilgiyi tek ekranda gösteren sayfa.
//   tokio / reqwest      → Rust'ın yaygın async çalışma zamanı / yaygın HTTP istemci kütüphanesi.
//   spawn_blocking       → tokio'da "bu ağır işi ayrı bir thread'de yap" çağrısı.
//   async closure        → çağrılınca future döndüren adsız fonksiyon.
//   poll_fn              → bir closure'dan elle küçük bir future üretmenin kısa yolu.

use std::future::Future;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::calisma_zamani::{Calistirici, kostur, uyu_ms, zaman_asimi};

pub fn calistir() {
    println!("== C1) API istemcisi: eşzamanlı istek + zaman aşımı + yeniden deneme ==");
    pano_senaryosu();
    println!("\n== C2) Yaygın hata: async içinde thread'i bloklamak ==");
    bloklama_senaryosu();
    println!("\n== C3) Yaygın hata: kilidi await boyunca tutmak ==");
    kilit_senaryosu();
}

// ============================================================================
// C1) Sahte sunucu ve istemci
// ============================================================================
// SADE ANLATIM: Üç arkadaşına birer soru soracaksın: biri 20, biri 35, biri 60 dakikada cevap
// yazıyor. Üç mesajı aynı anda gönderirsen 60 dakika beklersin; birinin cevabını bekleyip
// ötekine ondan sonra yazarsan 115 dakika. Bu bölümdeki istemci üç isteği aynı anda başlatır,
// çok uzun süreni kum saatiyle keser, "şu an meşgulüm" cevabı alınca da biraz bekleyip yeniden
// sorar. Bu neden var? Ağ istekleri zamanının çoğunu bekleyerek geçirir; beklemeleri üst üste
// bindirmek toplam süreyi en uzun isteğin süresine indirir.
//   Mini örnek:
//     let hizli = baslatici.baslat(istek(Arc::clone(&sunucu), "/hizli"));   // hemen başladı
//     let yavas = baslatici.baslat(async move {
//         match zaman_asimi(Duration::from_millis(60), istek(sunucu, "/yavas")).await {
//             Ok(cevap) => cevap,                       // süre dolmadan geldi
//             Err(_) => Err(IstekHatasi::ZamanAsimi),   // 60 ms doldu: istek iptal edildi
//         }
//     });
//     calistirici.calistir();      // ✓ ikisi birlikte ilerler: toplam ≈ 60 ms, 20 + 60 değil
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IstekHatasi {
    Gecici(u16), // 503 gibi: biraz sonra yeniden denemek anlamlı
    Bulunamadi,  // 404: yeniden denemek anlamsız
    ZamanAsimi,
}

impl IstekHatasi {
    fn gecici_mi(&self) -> bool {
        matches!(self, IstekHatasi::Gecici(_))
    }
}

pub struct SahteSunucu {
    kararsiz_cagri: AtomicU32, // "/kararsiz" kaç kez çağrıldı
}

impl SahteSunucu {
    pub fn yeni() -> Arc<Self> {
        Arc::new(SahteSunucu {
            kararsiz_cagri: AtomicU32::new(0),
        })
    }
}

// Parametreler neden `Arc<SahteSunucu>` ve `&'static str`? Bu future `baslat` ile ayrı bir
// GÖREV olacak; görevler 'static olmalı (çağıran fonksiyondan uzun yaşayabilir). `&SahteSunucu`
// alsaydık future o referansı içinde taşır, 'static olamazdı. Paylaşılan durumu görevlere
// vermenin yolu thread'lerdekiyle aynıdır: Arc.
pub async fn istek(sunucu: Arc<SahteSunucu>, yol: &'static str) -> Result<String, IstekHatasi> {
    match yol {
        "/hizli" => {
            uyu_ms(20).await;
            Ok(String::from("hızlı cevap"))
        }
        "/yavas" => {
            uyu_ms(400).await;
            Ok(String::from("yavaş cevap"))
        }
        "/kararsiz" => {
            uyu_ms(10).await;
            // İlk iki çağrı geçici hata, sonrakiler başarılı.
            let kacinci = sunucu.kararsiz_cagri.fetch_add(1, Ordering::SeqCst) + 1;
            if kacinci <= 2 {
                Err(IstekHatasi::Gecici(503))
            } else {
                Ok(format!("{kacinci}. denemede cevap"))
            }
        }
        _ => {
            uyu_ms(5).await;
            Err(IstekHatasi::Bulunamadi)
        }
    }
}

// Yeniden deneme. Parametre neden hazır bir future değil de future ÜRETEN bir closure?
// Çünkü bir future yalnız BİR KEZ tamamlanır; bittikten sonra yeniden poll edilemez. Her
// deneme taze bir future ister → `FnMut() -> Fut`. (Closure/trait ilişkisi: closures-iterators.)
//
// Denemeler arasında artan bekleme (backoff): hata veren servise hemen yeniden yüklenmemek
// için. Bekleme `uyu_ms(..).await` ile yapılıyor — thread::sleep DEĞİL (bkz. C2).
//
// Not (Rust 1.85+): aynı bound `impl AsyncFnMut() -> Result<T, IstekHatasi>` diye de
// yazılabilir; çağıran o zaman `async move || istek(..).await` biçiminde bir async closure
// verir. Burada klasik `F: FnMut() -> Fut, Fut: Future` biçimini kullandık çünkü "closure
// çağrılır → bir future döner → o future beklenir" iki adımını açıkça gösteriyor.
// SADE ANLATIM: Birini aradın, telefon meşgul çaldı. Biraz bekleyip yeniden ararsın; yine
// meşgulse bu kez biraz daha uzun beklersin. "Böyle bir numara yok" denirse bir daha aramazsın,
// çünkü tekrar aramak sonucu değiştirmez. `yeniden_dene` bunu yapar. Parametresi hazır bir
// arama değil, "ara" düğmesidir (closure): bitmiş bir future yeniden kullanılamaz, her deneme
// için düğmeye basılıp taze bir future üretilir.
//   Mini örnek:
//     let sonuc = yeniden_dene(4, move || istek(Arc::clone(&sunucu), "/kararsiz")).await;
//     // 1. deneme: Err(Gecici(503)) → kısa bekle, yine dene
//     // 2. deneme: Err(Gecici(503)) → biraz daha bekle, yine dene
//     // 3. deneme: Ok("3. denemede cevap")   ✓
//     // "/yok" isteseydik: Err(Bulunamadi) → kalıcı hata, ikinci kez denenmez
pub async fn yeniden_dene<F, Fut, T>(en_fazla: u32, mut dene: F) -> Result<T, IstekHatasi>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, IstekHatasi>>,
{
    let mut deneme = 1;
    loop {
        match dene().await {
            // Yalnız GEÇİCİ hatalarda ve hakkımız varsa yeniden dene.
            Err(hata) if hata.gecici_mi() && deneme < en_fazla => {
                uyu_ms(5 * u64::from(deneme)).await;
                deneme += 1;
            }
            // Başarı, kalıcı hata ya da hakkın bitmesi: olduğu gibi döndür.
            sonuc => return sonuc,
        }
    }
}

// Pano: üç bağımsız kaynağı eşzamanlı yükle. Dönen: (sonuçlar, geçen süre).
pub fn pano_yukle() -> ([Result<String, IstekHatasi>; 3], Duration) {
    let sunucu = SahteSunucu::yeni();
    let mut calistirici = Calistirici::yeni();
    let baslatici = calistirici.baslatici();
    let baslangic = Instant::now();

    // baslat ≈ tokio::spawn: her istek ayrı görev; üçü de HEMEN kuyruğa girer.
    let hizli = baslatici.baslat(istek(Arc::clone(&sunucu), "/hizli"));

    let kararsiz = {
        let sunucu = Arc::clone(&sunucu);
        // `move`: closure Arc'ın sahibi olur; her çağrıda yeni bir klonla yeni future üretir.
        baslatici.baslat(yeniden_dene(4, move || {
            istek(Arc::clone(&sunucu), "/kararsiz")
        }))
    };

    let yavas = baslatici.baslat(async move {
        // zaman_asimi ≈ tokio::time::timeout. İç içe iki Result: dıştaki "süre doldu mu",
        // içteki isteğin kendi sonucu. İkisini tek hata türüne indiriyoruz.
        match zaman_asimi(Duration::from_millis(60), istek(sunucu, "/yavas")).await {
            Ok(cevap) => cevap,
            Err(_) => Err(IstekHatasi::ZamanAsimi),
        }
    });

    calistirici.calistir(); // ≈ runtime.block_on(...): tüm görevler bitene dek
    let sonuclar = [hizli, kararsiz, yavas].map(|t| t.sonucu_al().expect("görev bitmeliydi"));
    (sonuclar, baslangic.elapsed())
}

fn pano_senaryosu() {
    let (sonuclar, sure) = pano_yukle();
    for (ad, sonuc) in ["hızlı", "kararsız", "yavaş"].iter().zip(&sonuclar) {
        println!("   {ad:<8} → {sonuc:?}");
    }
    // Sıralı yapılsaydı: 20 + (10+5+10+10+10) + 60 = 125 ms. Eşzamanlıda en uzun görev (~60 ms)
    // kadar sürer. Kesin süre makineye göre değişir; bu yüzden yalnız karşılaştırmayı basıyoruz.
    println!(
        "   toplam süre sıralı toplamdan (125 ms) kısa mı? {}",
        sure < Duration::from_millis(125)
    );
    // Beklenen çıktı:
    //   hızlı    → Ok("hızlı cevap")
    //   kararsız → Ok("3. denemede cevap")
    //   yavaş    → Err(ZamanAsimi)
    //   toplam süre sıralı toplamdan (125 ms) kısa mı? true
}

// ============================================================================
// C2) YAYGIN HATA — async içinde thread'i bloklamak
// ============================================================================
// Çalıştırıcımız TEK thread'de poll eder (çok thread'li runtime'larda da thread sayısı
// sınırlıdır). Bir görev poll içinde uzun süre kalırsa — thread::sleep, bloklayan dosya/ağ
// çağrısı, ağır hesap, std Mutex'te uzun bekleme — o thread'deki bütün öteki görevler durur.
// Derleyici bunu YAKALAMAZ: kod derlenir, doğru sonucu da verir; yalnız eşzamanlılık kaybolur.
//
// Kural: iki `.await` arasında geçen süre kısa olmalı. Beklemek için runtime'ın async
// araçlarını kullan; kaçınılmaz bloklayan işi ayrı bir thread'e gönder ve sonucunu bekle
// (tokio'da spawn_blocking; burada `ayri_threadde` ile aynısını yapıyoruz).
// SADE ANLATIM: Lokantada tek garson var. İyi garson siparişi mutfağa bırakır ve yemek
// pişerken öteki masalara bakar. Kötü garson mutfağın kapısında yemek pişene kadar dikilir;
// o sırada bütün masalar bekler. Async çalıştırıcı o tek garsondur. `uyu_ms(30).await` "ben
// bekliyorum, sen ötekilere bak" demektir. `thread::sleep` ise garsonun kendisini uyutur.
// Derleyici bu hatayı yakalamaz: kod derlenir, doğru sonucu da verir; yalnız yavaşlar.
//   Mini örnek:
//     async fn iyi() { uyu_ms(30).await; }          // ✓ beklerken thread'i geri verir
//     async fn kotu() {
//         thread::sleep(Duration::from_millis(30)); // ✗ derlenir, ama herkesi bekletir
//     }
//     // 3 görev: iyi ile < 90 ms (birlikte beklediler), kotu ile ≥ 90 ms (sıraya girdiler)
async fn kotu_bekleme() {
    thread::sleep(Duration::from_millis(30)); // ✗ executor thread'ini 30 ms kilitler
}

async fn iyi_bekleme() {
    uyu_ms(30).await; // ✓ Pending döner; çalıştırıcı bu sırada öteki görevlere bakar
}

// Bloklayan bir işi görev kuyruğunu durdurmadan çalıştırmak: işi yeni bir thread'e ver, o
// bitince bizi uyandırsın. Tutamac ile aynı desen: paylaşılan yuva + waker.
// SADE ANLATIM: Garsonun patates soyması gerekirse salonu bırakıp mutfağa kapanmaz; işi
// mutfaktaki yardımcıya verir ve "bitince haber ver" der. `ayri_threadde` bunu yapar: uzun
// süren, bölünemeyen işi yeni bir thread'e yollar; o iş bitince zili (waker) çalar ve sonucu
// teslim eder. Çalıştırıcı bu sırada öteki görevlere bakmayı sürdürür.
//   Mini örnek:
//     let n = ayri_threadde(|| {
//         thread::sleep(Duration::from_millis(30));   // bloklayan iş, ama başka thread'de
//         42
//     })
//     .await;                                         // ✓ beklerken kuyruk tıkanmaz; n = 42
pub async fn ayri_threadde<T, F>(is: F) -> T
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    use std::task::{Poll, Waker};
    let yuva: Arc<Mutex<(Option<T>, Option<Waker>)>> = Arc::new(Mutex::new((None, None)));
    let isci_yuvasi = Arc::clone(&yuva);
    thread::spawn(move || {
        let sonuc = is(); // bloklayan iş, executor thread'inin DIŞINDA
        let waker = {
            let mut kilit = isci_yuvasi.lock().unwrap();
            kilit.0 = Some(sonuc);
            kilit.1.take()
        };
        if let Some(waker) = waker {
            waker.wake();
        }
    });
    std::future::poll_fn(move |cx| {
        let mut kilit = yuva.lock().unwrap();
        match kilit.0.take() {
            Some(sonuc) => Poll::Ready(sonuc),
            None => {
                kilit.1 = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    })
    .await
}

// Aynı türden 3 görevi başlatıp toplam süreyi ölç.
fn uc_gorev_suresi<F, Fut>(uret: F) -> Duration
where
    F: Fn() -> Fut,
    Fut: Future<Output = ()> + Send + 'static,
{
    let mut calistirici = Calistirici::yeni();
    let baslatici = calistirici.baslatici();
    let baslangic = Instant::now();
    for _ in 0..3 {
        baslatici.baslat(uret());
    }
    calistirici.calistir();
    baslangic.elapsed()
}

fn bloklama_senaryosu() {
    let esik = Duration::from_millis(90); // 3 × 30 ms
    let kotu = uc_gorev_suresi(kotu_bekleme);
    let iyi = uc_gorev_suresi(iyi_bekleme);
    let ayri = uc_gorev_suresi(|| ayri_threadde(|| thread::sleep(Duration::from_millis(30))));
    println!(
        "   thread::sleep ile 3 görev ≥ 90 ms (sıraya girdiler)      : {}",
        kotu >= esik
    );
    println!(
        "   async uyku ile 3 görev  < 90 ms (birlikte beklediler)    : {}",
        iyi < esik
    );
    println!(
        "   bloklayan iş ayrı thread'de < 90 ms (kuyruk tıkanmadı)   : {}",
        ayri < esik
    );
    // Beklenen çıktı: üç satır da `true`.
}

// ============================================================================
// C3) YAYGIN HATA — std::sync::MutexGuard'ı bir .await boyunca tutmak
// ============================================================================
// ✗ YANLIŞ KULLANIM:
//     async fn sayaci_artir(sayac: Arc<Mutex<u32>>) -> u32 {
//         let mut kilit = sayac.lock().unwrap();
//         uyu_ms(5).await;              // guard bu await boyunca CANLI
//         *kilit += 1;
//         *kilit
//     }
//     baslatici.baslat(sayaci_artir(sayac));
//         error: future cannot be sent between threads safely
//           help: within `impl Future<Output = u32>`, the trait `Send` is not implemented for
//                 `std::sync::MutexGuard<'_, u32>`
//           note: future is not `Send` as this value is used across an await
// (Bu hatanın E-kodu yoktur.) Async fn'in state machine'i, await anında canlı olan her yerel
// değişkeni İÇİNDE saklar; MutexGuard !Send olduğu için future da !Send olur ve `baslat`ın
// `F: Send` bound'u sağlanmaz. Aynı mesajı `Rc`, `RefCell` borrow'u (`Ref`/`RefMut`) ve ham
// pointer'lar için de görürsün.
//
// Derleyici burada seni ikinci bir hatadan da koruyor: Send şartı olmayan tek thread'li bir
// runtime'da bu kod DERLENİR ve şu olabilir — görev A kilidi alıp await'te bekler; aynı
// thread'deki görev B lock() der ve thread'i BLOKLAR; A artık asla poll edilemez, kilidi
// bırakamaz → deadlock.
//
// ✓ DOĞRU KULLANIM: kilidi await'ten ÖNCE bırak. Blok kapandığında guard düşer; state
// machine'e girmez. (Kilidi await boyunca tutmak GERÇEKTEN gerekiyorsa async farkındalığı olan
// bir kilit gerekir — tokio::sync::Mutex gibi; beklerken thread'i bloklamaz, görevi askıya alır.)
// SADE ANLATIM: Ortak dolabın tek anahtarı var. Anahtarı alıp dolaptan eşyanı çıkarır, anahtarı
// yerine asar, sonra otobüs beklemeye gidersin. Anahtar cebindeyken otobüs beklemeye gidersen
// dolaba ihtiyacı olan herkes senin dönmeni bekler. Kilit anahtardır, `.await` otobüs
// beklemektir. Kural: önce kilitle işini bitir ve kilidi bırak, sonra `.await` de. Rust burada
// yardımcı olur: kilidin anahtarı (`MutexGuard`) başka thread'e geçemediği için, onu `.await`
// boyunca tutan görev `baslat`a verilince derlenmez.
//   Mini örnek:
//     let yeni = {
//         let mut k = sayac.lock().unwrap();
//         *k += 1;
//         *k
//     };                               // ✓ anahtar burada yerine asıldı
//     uyu_ms(5).await;                 // kilitsiz bekleme
//     // let mut k = sayac.lock().unwrap();
//     // uyu_ms(5).await;              // ✗ baslat ile derlenmez: anahtar bekleme boyunca elde
pub async fn sayaci_artir(sayac: Arc<Mutex<u32>>) -> u32 {
    let yeni_deger = {
        let mut kilit = sayac.lock().unwrap();
        *kilit += 1;
        *kilit
    }; // ← guard burada düştü
    uyu_ms(5).await; // (sonucu bir yere yazdığımızı varsay) — kilit elimizde DEĞİL
    yeni_deger
}

pub fn paylasimli_sayac(gorev_sayisi: u32) -> u32 {
    let sayac = Arc::new(Mutex::new(0));
    let mut calistirici = Calistirici::yeni();
    let baslatici = calistirici.baslatici();
    for _ in 0..gorev_sayisi {
        baslatici.baslat(sayaci_artir(Arc::clone(&sayac)));
    }
    calistirici.calistir();
    *sayac.lock().unwrap()
}

fn kilit_senaryosu() {
    println!("   5 görev sayacı artırdı → {}", paylasimli_sayac(5)); // 5
    println!(
        "   görev içinden sonuç → {}",
        kostur(sayaci_artir(Arc::new(Mutex::new(41))))
    ); // 42

    // AYNI AİLEDEN ÜÇ HATA DAHA (hepsi `baslat`ın bound'larından doğar):
    //
    // ✗ Yereli ödünç alan görev:
    //     let ad = String::from("ada");
    //     baslatici.baslat(async { println!("{ad}") });
    //         E0373: async block may outlive the current function, but it borrows `ad`
    //   ✓ `async move { ... }` — sahipliği göreve ver; paylaşılacaksa Arc klonu taşı.
    //
    // ✗ Rc'yi await boyunca taşımak:
    //     baslatici.baslat(async { let r = Rc::new(1); uyu_ms(1).await; println!("{r}"); });
    //         error: future cannot be sent between threads safely  (… `Rc<i32>` …)
    //   ✓ Arc kullan ya da Rc'yi await'ten önce düşür.
    //
    // ✗ (Derlenir, ASILI KALIR) waker'ı kaydetmeden Pending dönen elle yazılmış future:
    //     fn poll(..) -> Poll<()> { if self.hazir { Poll::Ready(()) } else { Poll::Pending } }
    //   Çalıştırıcı görevi yalnız wake() gelince yeniden poll eder; kimse çağırmayacağı için
    //   görev sonsuza dek bekler ve `calistir()` dönmez. Kural: Pending dönmeden önce
    //   cx.waker()'ı olayı bilecek tarafa TESLİM ET (Uyku'daki gibi) — ve "hazır mı?" denetimi
    //   ile waker kaydını aynı kilit altında yap (Tutamac'taki not).
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn pano_her_gorevin_sonucunu_ayri_tasir() {
        let (sonuclar, _) = pano_yukle();
        assert_eq!(sonuclar[0], Ok(String::from("hızlı cevap")));
        assert_eq!(sonuclar[1], Ok(String::from("3. denemede cevap")));
        assert_eq!(sonuclar[2], Err(IstekHatasi::ZamanAsimi));
    }

    #[test]
    fn yeniden_deneme_hakki_bitince_son_hatayi_verir() {
        let sunucu = SahteSunucu::yeni();
        let sonuc = kostur(yeniden_dene(2, move || {
            istek(Arc::clone(&sunucu), "/kararsiz")
        }));
        assert_eq!(sonuc, Err(IstekHatasi::Gecici(503)));
    }

    #[test]
    fn kalici_hata_yeniden_denenmez() {
        let cagri = Arc::new(AtomicU32::new(0));
        let sayac = Arc::clone(&cagri);
        let sunucu = SahteSunucu::yeni();
        let sonuc = kostur(yeniden_dene(5, move || {
            sayac.fetch_add(1, Ordering::SeqCst);
            istek(Arc::clone(&sunucu), "/yok")
        }));
        assert_eq!(sonuc, Err(IstekHatasi::Bulunamadi));
        assert_eq!(cagri.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn bloklayan_is_ayri_threadde_sonuc_dondurur() {
        let kimlik = thread::current().id();
        let baska_thread = kostur(ayri_threadde(move || thread::current().id() != kimlik));
        assert!(baska_thread);
    }

    #[test]
    fn kilit_await_oncesi_birakilinca_tum_gorevler_ilerler() {
        assert_eq!(paylasimli_sayac(20), 20);
    }
}
