// Derinleşme — Bölüm 16: Concurrency ve Thread Safety (pekiştirme: gerçekçi senaryo, sık hatalar, alıştırmalar)
// Çalıştır: cargo run -p concurrency   |   Test: cargo test -p concurrency
//
// main.rs üç yapı taşını ayrı ayrı gösterdi: spawn+join, kanal, Arc<Mutex>. Bu dosya onları
// gerçek iş kalıplarına dönüştürür:
//   - thread::scope: yerel veriyi Arc'sız, klonsuz ÖDÜNÇ vererek paralel işlem,
//   - işçi havuzu (worker pool): iş kuyruğu + sonuç kanalı,
//   - paylaşımlı durumda kilit süresini kısa tutmak ("yerelde biriktir, bir kez birleştir"),
//   - atomikler: tek bir sayı için Mutex'e gerek yok,
//   - en sık dört derleme hatası + derleyicinin YAKALAYAMADIĞI iki çalışma anı sorunu
//     (deadlock ve zehirlenmiş kilit).
// RwLock, Condvar, kilit sıralaması ve async tarafı → learn/advanced-concepts/ileri-eszamanlilik-async.
// ✗ örneklerin hata kodları rustc 1.99 ile derlenerek doğrulandı.
//
// Not: thread'lerin çalışma sırası belirsizdir. Buradaki her çıktı, sıradan bağımsız olacak
// biçimde (toplam, sıralanmış liste) üretilir; bu yüzden her çalıştırmada aynıdır.

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Bu dosya main.rs'teki üç aracı (thread, kanal, kilit) gerçek işlere uygular. Her bölümün
// üstündeki "SADE ANLATIM" kutusunu önce oku. Benzetmeler yine aynı lokanta MUTFAĞINDA geçer.
//
// Bu dosyanın tek cümlelik özeti: İşi aşçılara bölmenin birkaç hazır kalıbı vardır; derleyici
// veri yarışını yakalar, ama aşçıların birbirini sonsuza dek beklemesini (deadlock) yakalayamaz.
//
// MİNİ SÖZLÜK (yalnız bu dosyada yeni geçenler; öteki terimler main.rs başındaki sözlükte):
//   thread::scope            → "bu blok bitmeden işini bitirecek" thread'ler başlatmak.
//   'static                  → "programın sonuna kadar yaşayabilir" demek; ödünç veri buna uymaz.
//   chunks / chunks_mut      → diziyi üst üste binmeyen parçalara bölmek (okumak / yazmak için).
//   tembel (lazy) iterator   → sonucu isteyen olmadıkça hiçbir iş yapmayan sıralı işlem zinciri.
//   işçi havuzu (worker pool) → sayısı sabit, sıradaki işi kuyruktan alan thread grubu.
//   kuyruk (queue)           → işlerin geliş sırasıyla beklediği sıra.
//   kritik bölge (critical section) → kilit elindeyken çalışan kod parçası. Kısa olmalıdır.
//   kilit çekişmesi (lock contention) → thread'lerin zamanını kilit sırası bekleyerek geçirmesi.
//   AtomicU64 / fetch_add    → kilitsiz paylaşılabilen sayı / onu tek hamlede artırma işlemi.
//   Ordering (Relaxed, SeqCst) → atomik işlemin öteki işlemlere göre ne kadar sıkı sıralanacağı.
//   try_lock                 → kilidi beklemeden istemek: alamazsan hemen "olmadı" cevabı gelir.
//   into_inner               → kilidi ya da hücreyi söküp içindeki veriyi sahiplenerek almak.
//   değişmez (invariant)     → her zaman doğru kalması gereken kural ("sayaç = liste uzunluğu").
//   mantıksal yarış          → adımlar tek tek güvenli olsa da aralarında durumun değişmesi.
//   BTreeMap                 → anahtarları sıralı tutan sözlük (anahtar → değer tablosu).
//   G/Ç (I/O)                → dosya, ağ, ekran gibi dış dünyayla yapılan okuma ve yazma.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, TryLockError};
use std::thread;

pub fn calistir() {
    println!("\n== Derinleşme: eşzamanlılık pratikte ==");
    kapsamli_threadler();
    isci_havuzu_senaryosu();
    paylasimli_durum_senaryosu();
    atomik_sayac();
    derleyicinin_yakalayamadiklari();
}

// ============================================================================
// 1. BU KONUYU NEDEN ÖĞRENİYORUZ? — Mesajlaş mı, paylaş mı?
// ============================================================================
// SADE ANLATIM: İki aşçı bir işi iki yolla paylaşır. Ya tabağı elden ele geçirirler (kanal):
// tabak kimdeyse yalnız o dokunur. Ya da ikisi de yerinde duran aynı tencereye sırayla uzanır
// (paylaşımlı durum): tencere tektir, kaşık sırayla alınır.
// Neden iki yol? Akan işler (hazırla → pişir → servis et) elden ele geçirmeye, herkesin baktığı
// ortak kayıtlar (stok listesi) yerinde paylaşmaya daha uygundur.
//   Mini örnek:
//     tx.send(tabak).unwrap();           // kanal: tabak öteki aşçıya GEÇTİ, artık yalnız onda
//     *tencere.lock().unwrap() += 1;     // paylaşım: tencere yerinde durur, sırayla uzanılır
//
// Thread'ler arasında veri iki biçimde dolaşır ve seçim, programın yapısını belirler:
//
//   KANAL (mesajlaşma)  → veri bir thread'den ötekine TAŞINIR; aynı anda tek sahibi vardır.
//       Uygun: iş hattı (üret → işle → topla), görev dağıtımı, olay akışı. Kilit yoktur,
//       dolayısıyla deadlock riski düşüktür; sahiplik zaten kimin dokunabileceğini söyler.
//   PAYLAŞIMLI DURUM (Arc<Mutex<T>>) → veri YERİNDE durur, thread'ler sırayla erişir.
//       Uygun: herkesin okuyup güncellediği ortak tablo (önbellek, sayaçlar, bağlantı havuzu).
//
// Benzetme: kanal, masadan masaya dosya gezdirmektir (dosya kimdeyse o çalışır); Mutex, ortak
// bir beyaz tahtadır (kalemi alan yazar, ötekiler bekler). İlki akış için, ikincisi ortak
// kayıt için doğaldır. Emin değilsen kanalla başla: hataları derleyici daha fazla yakalar.

// ============================================================================
// 2. thread::scope — yerel veriyi ÖDÜNÇ vererek paralel işlem
// ============================================================================
// main.rs'te gördük: thread::spawn'a verilen closure 'static olmalı, çünkü thread onu doğuran
// fonksiyondan uzun yaşayabilir. Bu yüzden yerel veriyi ödünç veremeyiz:
//   ✗  let veri = vec![1, 2, 3];
//      thread::spawn(|| println!("{veri:?}"));
//         // E0373: closure may outlive the current function, but it borrows `veri`
// İki klasik çözüm var: `move` ile sahipliği vermek (veri artık bizde değil) ya da Arc ile
// paylaşmak (sayaç + heap tahsisi). Oysa çoğu zaman istediğimiz daha basit: "şu dilimi dört
// parçaya böl, paralel işle, SONRA devam et" — thread'ler zaten fonksiyon bitmeden bitecek.
//
// thread::scope tam bunu ifade eder: kapsamın içinde doğan her thread, kapsam kapanmadan
// OTOMATİK join edilir. Derleyici bu garantiye dayanarak 'static şartını kaldırır; closure'lar
// yerel değişkenleri düz & ile (hatta ayrık parçaları &mut ile) ödünç alabilir.
// SADE ANLATIM: Dört yardımcıyı "ben mutfaktan çıkmadan işinizi bitireceksiniz" diyerek
// çağırırsın. Senden önce bitirecekleri kesin olduğu için malzemeyi onlara VERMEN gerekmez;
// malzeme senin tezgâhında dururken kullanırlar (ödünç alırlar). `thread::scope` bu sözü verir:
// blok kapanırken içeride başlayan bütün thread'ler kendiliğinden beklenir.
// Neden böyle? Sıradan `thread::spawn` ile başlayan aşçı senden uzun çalışabilir; bu yüzden ona
// ödünç malzeme vermek yasaktır. Scope'ta "önce onlar biter" güvencesi yasağı gereksiz kılar.
//   Mini örnek:
//     let sayilar = vec![1, 2, 3, 4];
//     thread::scope(|k| {
//         k.spawn(|| println!("{}", sayilar[0]));       // ✓ ödünç aldı; `move` yok, Arc yok
//         k.spawn(|| println!("{}", sayilar.len()));    // ✓ ikinci yardımcı da bakabilir
//     });                                               // burada ikisi de bitmiş olur
//     println!("{sayilar:?}");                          // ✓ sayilar hâlâ bizde
//     // thread::spawn(|| println!("{sayilar:?}"));     // ✗ derlenmez: bizden uzun yaşayabilir
pub fn paralel_topla(veri: &[u64], parca_sayisi: usize) -> u64 {
    let parca_boyu = veri.len().div_ceil(parca_sayisi.max(1)).max(1);
    thread::scope(|kapsam| {
        // chunks(): dilimi AYRIK alt dilimlere böler; her thread kendi &[u64] parçasını alır.
        let tutamaclar: Vec<_> = veri
            .chunks(parca_boyu)
            .map(|parca| kapsam.spawn(move || parca.iter().sum::<u64>()))
            .collect();
        // (Sade: "tembel" = sonucu isteyen olmadıkça çalışmayan demek; `collect()` zincire
        // "hepsini şimdi yap" der, böylece bütün yardımcılar beklemeye geçmeden önce işe başlar.)
        // collect() burada ŞART: iterator'lar tembeldir. Tutamaçları toplamadan doğrudan
        // `.map(spawn).map(join)` zincirleseydik her thread doğar doğmaz join edilir, bir
        // sonraki ancak o bitince başlardı — kod "paralel" görünür ama SERİ çalışırdı.
        // Önce hepsini başlat (collect), sonra hepsini bekle.
        // join(): her parçanın sonucunu al. unwrap: işçi panik ettiyse biz de durduralım.
        tutamaclar.into_iter().map(|t| t.join().unwrap()).sum()
    }) // ← kapsam burada kapanır; içeride doğan tüm thread'lerin bittiği GARANTİ
}

// Ayrık parçalara YAZMAK da aynı kalıpla güvenlidir: chunks_mut her thread'e çakışmayan bir
// &mut dilim verir. "Aynı anda iki &mut yok" kuralı bozulmaz çünkü dilimler örtüşmez — ve
// bunu derleyici chunks_mut'un imzasından bilir, bizim dikkatimizden değil.
// (Sade: tezgâhı çizgilerle bölüp her yardımcıya kendi bölümünü verirsen kimse kimsenin
// bölümüne uzanamaz; `chunks_mut` bu çizgileri çeker, bu yüzden aynı anda YAZMAK da güvenlidir.)
pub fn paralel_karesini_al(veri: &mut [u64], parca_sayisi: usize) {
    let parca_boyu = veri.len().div_ceil(parca_sayisi.max(1)).max(1);
    thread::scope(|kapsam| {
        for parca in veri.chunks_mut(parca_boyu) {
            kapsam.spawn(move || {
                for x in parca {
                    *x *= *x;
                }
            });
        }
    });
}

fn kapsamli_threadler() {
    let mut sayilar: Vec<u64> = (1..=1000).collect();
    println!("paralel toplam (4 parça) = {}", paralel_topla(&sayilar, 4)); // 500500
    paralel_karesini_al(&mut sayilar, 4);
    println!(
        "kareler: ilk üç = {:?}, son = {}",
        &sayilar[..3],
        sayilar[999]
    ); // [1, 4, 9], 1000000
    // `sayilar` hâlâ bizim: ne move ettik ne Arc'a sardık.
}

// ============================================================================
// 3. GERÇEK HAYATTAN ÖRNEK — işçi havuzu (worker pool)
// ============================================================================
// Senaryo: gelen kayıtları (burada metin satırları) sabit sayıda işçi thread'e dağıtıp
// sonuçları toplamak. Her iş için yeni thread açmak pahalıdır ve yük altında sınırsız thread
// doğurur; havuz, eşzamanlılığı N ile sınırlar. Web sunucuları ve veri işleme hatları bu
// kalıbın üzerine kuruludur (kitabın son bölümündeki çok thread'li sunucu da budur).
//
// Tasarım — iki kanal:
//   iş kanalı    : ana thread → işçiler. mpsc'de alıcı TEKTİR (single consumer); N işçinin
//                  paylaşabilmesi için alıcıyı Arc<Mutex<Receiver>> içine koyarız. Mutex burada
//                  veriyi değil, "kuyruktan sıradaki işi kim alacak"ı sıraya sokar.
//   sonuç kanalı : işçiler → ana thread. Gönderen klonlanabildiği için her işçiye bir kopya.
// SADE ANLATIM: Lokantada her sipariş için yeni aşçı işe alınmaz. Üç aşçı vardır, siparişler bir
// fiş rayına dizilir. Boşalan aşçı raydan sıradaki fişi çeker, yemeği yapar, tabağı servis
// penceresine koyar. Aşağıdaki kodda ray "iş kanalı", pencere "sonuç kanalı"dır. Rayın alan ucu
// tek olduğu için bir Mutex'in içinde durur: fişi çekerken raya tek aşçı uzanır ve fişi alır
// almaz rayı bırakır. Ray boşalıp kapanınca (`drop(is_tx)`) aşçılar "mesai bitti" diye anlar.
// Neden böyle? Her işe yeni thread açmak pahalıdır; yoğun saatte mutfak aşçıyla dolar taşardı.
//   Mini örnek:
//     let ray = Arc::new(Mutex::new(is_rx));         // fiş rayı: sırayla uzanılır
//     let fis = ray.lock().unwrap().recv();          // ✓ fişi çek; satır bitince ray serbest
//     // while let Ok(f) = ray.lock().unwrap().recv() { pisir(f); }
//     //     ✗ derlenir ama yanlış çalışır: ray, pişirme boyunca kilitli kalır, tek aşçı çalışır
pub struct Is {
    pub id: u32,
    pub metin: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Sonuc {
    pub id: u32,
    pub kelime_sayisi: usize,
}

pub fn havuzda_isle(isler: Vec<Is>, isci_sayisi: usize) -> Vec<Sonuc> {
    let (is_tx, is_rx) = mpsc::channel::<Is>();
    let (sonuc_tx, sonuc_rx) = mpsc::channel::<Sonuc>();
    let is_rx = Arc::new(Mutex::new(is_rx));

    let mut isciler = Vec::new();
    for _ in 0..isci_sayisi.max(1) {
        let is_rx = Arc::clone(&is_rx); // her işçiye alıcının paylaşımlı bir sahibi
        let sonuc_tx = sonuc_tx.clone(); // her işçiye kendi göndereni
        isciler.push(thread::spawn(move || {
            loop {
                // ÖNEMLİ SATIR: lock() geçici bir MutexGuard üretir ve bu `let` deyimi
                // bitince düşer → kilit, iş ALINIR ALINMAZ bırakılır; işi kilitsiz işleriz.
                // Bunu `while let Ok(is) = is_rx.lock().unwrap().recv() { ... }` diye yazsaydık
                // guard döngü GÖVDESİ boyunca yaşardı: bir işçi çalışırken ötekiler kuyruğa
                // erişemez, havuz sessizce tek thread'e inerdi (derlenir, yanlış çalışır).
                let alinan = is_rx.lock().unwrap().recv();
                match alinan {
                    Ok(is) => {
                        let kelime_sayisi = is.metin.split_whitespace().count();
                        // Alıcı kapanmışsa gönderemeyiz; o zaman yapacak iş kalmamıştır.
                        if sonuc_tx
                            .send(Sonuc {
                                id: is.id,
                                kelime_sayisi,
                            })
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(_) => break, // tüm gönderenler kapandı ve kuyruk boş → işçi emekli
                }
            }
        }));
    }
    // Ana thread'deki orijinal sonuç gönderenini kapat. Kapatmazsak aşağıdaki sonuç döngüsü
    // "hâlâ bir gönderen var" diye sonsuza dek bekler (main.rs'teki drop(tx) ile aynı sebep).
    drop(sonuc_tx);

    for is in isler {
        is_tx.send(is).unwrap(); // işin sahipliği kanala, oradan bir işçiye geçer
    }
    drop(is_tx); // "başka iş gelmeyecek" sinyali: işçilerin recv()'i Err döner, döngüleri biter

    let mut sonuclar: Vec<Sonuc> = sonuc_rx.iter().collect(); // tüm işçiler bitene dek toplar
    for isci in isciler {
        isci.join().unwrap();
    }
    sonuclar.sort_by_key(|s| s.id); // varış sırası belirsiz → kimliğe göre sırala
    sonuclar
}

fn isci_havuzu_senaryosu() {
    let satirlar = [
        "bir iki üç",
        "tek",
        "dört kelimelik bir satır",
        "",
        "son satır",
    ];
    let isler: Vec<Is> = (1..)
        .zip(satirlar)
        .map(|(id, metin)| Is {
            id,
            metin: metin.to_string(),
        })
        .collect();
    let sonuclar = havuzda_isle(isler, 3);
    let ozet: Vec<String> = sonuclar
        .iter()
        .map(|s| format!("#{}:{}", s.id, s.kelime_sayisi))
        .collect();
    println!("işçi havuzu sonuçları = {}", ozet.join(" ")); // #1:3 #2:1 #3:4 #4:0 #5:2
}

// ============================================================================
// 4. PAYLAŞIMLI DURUM — kilidi kısa tut: "yerelde biriktir, bir kez birleştir"
// ============================================================================
// Senaryo: birkaç belgeyi paralel tarayıp ortak bir kelime sıklığı tablosu çıkarmak.
//
// YANLIŞ KULLANIM (derlenir, yavaştır): her kelime için kilidi alıp tabloyu güncellemek.
// Thread'ler zamanlarının çoğunu kilit beklerken geçirir; dört çekirdek tek çekirdek gibi
// çalışır. Mutex doğruluğu sağlar, PARALELLİĞİ değil.
//
// DOĞRU KULLANIM: her thread kendi yerel tablosunu KİLİTSİZ doldurur, işi bitince kilidi bir
// kez alıp sonucu ortak tabloya katar. Kilit altında geçen süre "belge başına bir birleştirme"ye
// iner.
//
// Dikkat: burada Arc YOK. thread::scope sayesinde `&Mutex` ödünç vermek yetiyor. Arc "kaç
// sahip var"ı, Mutex "aynı anda kim erişir"i çözer; sahiplik sorunu olmayınca Arc da gerekmez.
// SADE ANLATIM: Stok defteri kilerdedir ve kilerin tek anahtarı vardır. Saydığın her patates
// için kilere gidip deftere bir çizgi atarsan bütün aşçılar gününü kiler kapısında geçirir. Bunun
// yerine herkes kendi kâğıdına sayar, işi bitince kilere BİR kez girip toplamını deftere yazar.
// Aşağıdaki kod da böyle yapar: önce kilitsiz yerel tablo, sonra tek ve kısa bir `lock()`.
// Neden böyle? Kilit sonucun doğru çıkmasını sağlar, hızlı çıkmasını değil; kilit altında geçen
// süre uzadıkça dört aşçı tek aşçı gibi çalışır.
//   Mini örnek:
//     let mut yerel = 0;
//     for _ in 0..1000 { yerel += 1; }                      // ✓ kilitsiz, kendi kâğıdında say
//     *ortak.lock().unwrap() += yerel;                      // ✓ kilidi bir kez, kısacık al
//     // for _ in 0..1000 { *ortak.lock().unwrap() += 1; }  // derlenir ama 1000 kez sıra bekler
pub fn kelime_sikligi(belgeler: &[&str]) -> BTreeMap<String, u32> {
    let ortak: Mutex<BTreeMap<String, u32>> = Mutex::new(BTreeMap::new());
    thread::scope(|kapsam| {
        for belge in belgeler {
            let ortak = &ortak; // Mutex'i taşımadan, referansını closure'a ver
            kapsam.spawn(move || {
                // 1) Kilitsiz, yerel çalışma
                let mut yerel: BTreeMap<String, u32> = BTreeMap::new();
                for kelime in belge.split_whitespace() {
                    *yerel.entry(kelime.to_lowercase()).or_insert(0) += 1;
                }
                // 2) Kısa kritik bölge: yalnız birleştirme
                let mut tablo = ortak.lock().unwrap();
                for (kelime, adet) in yerel {
                    *tablo.entry(kelime).or_insert(0) += adet;
                }
            }); // `tablo` guard'ı burada düşer → kilit bırakılır
        }
    });
    // Tüm thread'ler bitti; Mutex'in tek sahibi biziz → kilitlemeden içindekini al.
    ortak.into_inner().unwrap()
}

fn paylasimli_durum_senaryosu() {
    let belgeler = ["rust hızlı ve güvenli", "Rust korkusuz", "güvenli ve hızlı"];
    let tablo = kelime_sikligi(&belgeler);
    println!("kelime sıklığı = {tablo:?}");
    // Beklenen çıktı (BTreeMap anahtara göre sıralı tutar → çıktı deterministik):
    //   kelime sıklığı = {"güvenli": 2, "hızlı": 2, "korkusuz": 1, "rust": 2, "ve": 2}
}

// ============================================================================
// 5. ATOMİKLER — tek bir sayı için Mutex'e gerek yok
// ============================================================================
// Paylaşılan şey yalnızca bir sayaç ya da bayraksa, işlemci düzeyinde bölünemez (atomic)
// işlemler yeter: kilit yok, bekleme yok, zehirlenme yok. fetch_add "oku + artır + yaz"ı tek
// adımda yapar; iki thread aynı anda çağırsa bile artışlardan biri kaybolmaz.
//
// Ordering parametresi, bu işlemin ÖTEKİ bellek erişimleriyle sıralama ilişkisini söyler.
// Bağımsız bir sayaç için Relaxed yeterlidir (yalnız işlemin kendisi atomik olsun yeter).
// Atomik bir değere bakıp başka veriye erişim kararı veriyorsan (bayrak → veri) daha güçlü
// sıralamalar gerekir; emin değilsen SeqCst kullan. Ayrıntı: ileri-eszamanlilik-async.
// SADE ANLATIM: Mutfak kapısında, basınca bir artan mekanik bir sayaç var. Basmak tek hamledir:
// iki kişi aynı anda bassa da sayı iki artar, hiçbir basış kaybolmaz. Bunun için kapıyı
// kilitlemeye gerek yoktur. Atomik türler bu sayaçtır; `fetch_add(1, ...)` düğmeye basmaktır.
// Neden böyle? Sıradan bir sayıda "oku, artır, yaz" üç ayrı adımdır; iki thread araya girerse bir
// artış kaybolur. Atomik işlem bu üç adımı bölünmez tek adım yapar.
// (Sade: `Ordering::Relaxed` = "yalnız bu sayının doğru sayılması yeter, başka şart yok" demek.)
//   Mini örnek:
//     let sayac = AtomicU64::new(0);
//     thread::scope(|k| {
//         k.spawn(|| sayac.fetch_add(1, Ordering::Relaxed));   // ✓ kilitsiz +1
//         k.spawn(|| sayac.fetch_add(1, Ordering::Relaxed));   // ✓ aynı anda olsa da kaybolmaz
//     });
//     println!("{}", sayac.into_inner());                      // ✓ 2
pub fn paralel_say(thread_sayisi: u64, thread_basina: u64) -> u64 {
    let sayac = AtomicU64::new(0);
    thread::scope(|kapsam| {
        for _ in 0..thread_sayisi {
            kapsam.spawn(|| {
                for _ in 0..thread_basina {
                    sayac.fetch_add(1, Ordering::Relaxed);
                }
            });
        }
    });
    sayac.into_inner()
}

fn atomik_sayac() {
    println!(
        "atomik sayaç (8 thread × 10_000) = {}",
        paralel_say(8, 10_000)
    ); // 80000
    // Sınır: atomikler TEK değeri korur. "Sayaç ile liste birlikte tutarlı kalsın" gibi birden
    // çok değeri kapsayan bir değişmez (invariant) varsa yine Mutex gerekir.
}

// ============================================================================
// 6. SIK YAPILAN HATALAR — derleyicinin yakaladıkları ve yakalayamadıkları
// ============================================================================
// SADE ANLATIM: Derleyici mutfak kapısındaki denetçi gibidir: iki aşçının aynı tencereye sırasız
// uzanmasına yol açacak tarifi içeri almaz. Aşağıdaki dört ✗ örnek böyle geri çevrilen
// tariflerdir. İçeride olanları ise göremez: bir aşçı bıçağı tutup kesme tahtasını beklerken
// öteki tahtayı tutup bıçağı beklerse ikisi de sonsuza dek bekler. Buna deadlock denir; Rust
// veri yarışını önler, deadlock'u ÖNLEMEZ.
// Neden önleyemez? Kilitlerin hangi sırayla alınacağı program çalışırken belli olur; derleyici
// yalnız "veriye kim, hangi izinle uzanıyor" sorusunu denetler.
//   Mini örnek (kod değil, olay sırası):
//     Aşçı 1: bıçağı al → tahtayı bekle          Aşçı 2: tahtayı al → bıçağı bekle
//     ✗ derlenir ama ikisi de sonsuza dek bekler
//     ✓ çare: herkes önce bıçağı, sonra tahtayı alsın (kilitleri her yerde aynı sırayla al)
//
// Derleme hataları (hepsi "veri yarışı olurdu"nun farklı yüzleri):
//
//   ✗ Arc'ı klonlamadan thread'e taşıyıp sonra yine kullanmak:
//       let m = Arc::new(Mutex::new(0));
//       let t = thread::spawn(move || { *m.lock().unwrap() += 1; });
//       println!("{m:?}");            // E0382: the type `Arc` does not implement `Copy`
//                                     // (note: consider using `Arc::clone`)
//     ✓ Taşımadan ÖNCE klonla; closure klonu alsın: `let m2 = Arc::clone(&m);`
//
//   ✗ Arc'ın içini Mutex'siz değiştirmek:
//       let v = Arc::new(vec![1]);
//       thread::spawn(move || v.push(2));   // E0596: cannot borrow data in an `Arc` as mutable
//     Arc yalnız paylaşımlı erişim (&T) verir. ✓ Arc<Mutex<Vec<_>>>.
//
//   ✗ Gönderilen değeri sonra kullanmak:
//       tx.send(mesaj).unwrap();
//       println!("{mesaj}");          // E0382: borrow of moved value: `mesaj`
//     send sahipliği alıcıya taşır — "iki thread aynı veriye bakıyor" durumu baştan yok.
//
//   ✗ Kilidi alıp guard'ı başka thread'e vermek:
//       let g = m.lock().unwrap();
//       kapsam.spawn(move || println!("{}", *g));
//         // E0277: `MutexGuard<'_, i32>` cannot be sent between threads safely
//     Birçok işletim sisteminde kilidi, onu ALAN thread bırakmak zorundadır; std bu yüzden
//     MutexGuard'ı !Send yapar. ✓ Kilidi, veriye dokunacak thread'in içinde al.
//
// Çalışma anı sorunları (derleyici YAKALAMAZ — "fearless" veri yarışını kapsar, bunları değil):
fn derleyicinin_yakalayamadiklari() {
    // --- (a) Kendi kendini kilitlemek ---------------------------------------------------------
    // Aynı thread, elinde tuttuğu Mutex'i ikinci kez kilitlemeye çalışırsa lock() ya sonsuza
    // dek bekler ya da panik eder (std hangisinin olacağını garanti etmez). Bu genellikle
    // "kilit altındayken, yine kilit alan başka bir metodumu çağırdım" biçiminde ortaya çıkar.
    // try_lock ile durumu güvenle gözleyelim:
    let kasa = Mutex::new(100);
    let ilk = kasa.lock().unwrap();
    let ikinci_deneme = matches!(kasa.try_lock(), Err(TryLockError::WouldBlock));
    println!("kilit elimizdeyken ikinci kilit bloklanırdı: {ikinci_deneme}"); // true
    drop(ilk); // ✓ guard'ı bırak; sonra yeniden kilitlemek serbest
    println!(
        "bıraktıktan sonra alınabiliyor: {}",
        kasa.try_lock().is_ok()
    ); // true

    // --- (b) İki kilit, ters sıra → deadlock --------------------------------------------------
    // Thread 1: A'yı al, B'yi bekle.   Thread 2: B'yi al, A'yı bekle.   → ikisi de sonsuza dek.
    // Burada çalıştırmıyoruz (program asılı kalırdı). Kural: birden fazla kilit gerekiyorsa
    // HER YERDE aynı sırayla al (ör. hesap numarası küçük olan önce). Uygulaması ve try_lock
    // ile geri çekilme → ileri-eszamanlilik-async.

    // --- (c) Zehirlenmiş kilit (poison) -------------------------------------------------------
    // (Sade: aşçı elinde kiler anahtarıyla bayılırsa raflar yarım düzenlenmiş kalabilir; kapıya
    // "dikkat" levhası asılır. Sonraki `lock()` bu levhayı Err olarak gösterir, karar senindir.)
    // Kilidi tutan thread panik ederse veri yarıda kalmış olabilir; std Mutex'i "zehirli"
    // işaretler ve sonraki lock() çağrıları Err döndürür. .unwrap() bu yüzden yaygındır:
    // "öteki thread çöktüyse ben de durayım". Veri yine de kullanılabilir durumdaysa
    // `lock().unwrap_or_else(|zehir| zehir.into_inner())` ile guard'ı kurtarabilirsin.
    // Çalışan örneği testlerde: `zehirli_kilit_kurtarilabilir`.
}

// SADE ANLATIM (aşağıdaki bölüm için): Aşağıda bu dosyanın kısa özeti ve üç alıştırma var.
// Özetin ana fikri: kilidi elinde olabildiğince kısa tut ve asılı kalan bir programda önce
// "kapanmayı unuttuğum bir gönderen (tx) var mı?" diye bak. Alıştırmaları ilk okuyuşta
// atlayabilirsin.
// ============================================================================
// ÖNEMLİ NOKTALAR
// ============================================================================
// • Derlenen güvenli Rust kodunda VERİ YARIŞI olamaz; deadlock, kilit çekişmesi ve mantıksal
//   yarış (iki ayrı kilitli adım arasında durumun değişmesi) olabilir.
// • Thread'ler fonksiyon bitmeden bitecekse thread::scope kullan: Arc, clone ve 'static
//   derdi kalkar.
// • Kritik bölgeyi kısa tut: kilit altında G/Ç ve uzun hesap yapma, başka kilit alma, kullanıcı
//   kodu (closure) çağırma.
// • Guard'ın ömrü = kilidin süresi. Geçici guard deyim sonunda, değişkene bağlı guard kapsam
//   sonunda düşer; gerekirse `drop(guard)` ile erken bırak.
// • Kanal kapanışı sahiplikle olur: tüm gönderenler düşünce alıcı döngüsü biter. Asılı kalan
//   bir programda ilk bakılacak yer, unutulmuş bir gönderen klonudur.
//
// ============================================================================
// ALIŞTIRMALAR
// ============================================================================
// 1) `fn paralel_en_buyuk(veri: &[i64], parca_sayisi: usize) -> Option<i64>` yaz
//    (thread::scope + chunks). Boş dilim için None dönsün.
//    İpucu: her thread `parca.iter().copied().max()` döndürsün; sonuçları `flatten().max()`
//    ile birleştir. Beklenen: [3, -7, 12, 5] → Some(12).
//
// 2) `havuzda_isle`yi, bir işçi işlerken panik etse bile ötekiler çalışmaya devam edecek
//    biçimde değiştir: `Sonuc` yerine `Result<Sonuc, u32>` (Err içinde iş kimliği) gönder.
//    İpucu: iş gövdesini `std::panic::catch_unwind` ile sarmak yerine, hatayı baştan Result
//    olarak modelle (ör. metin "HATA" içeriyorsa Err). Panik, "beklenmeyen" içindir.
//
// 3) `kelime_sikligi`ni kanalla yeniden yaz: her thread yerel tablosunu bir kanala göndersin,
//    ana thread birleştirsin — Mutex kullanma. İki sürümü karşılaştır: hangisinde "kimin
//    veriye dokunduğu" daha kolay görülüyor?
//    İpucu: mpsc::channel::<BTreeMap<String, u32>>(); kapsam içinde tx.clone(), kapsamdan
//    önce/sonra orijinal tx'i düşürmeyi unutma.
//
// Bir sonraki konuyla bağlantısı: thread'ler işi GERÇEKTEN aynı anda koşturur (paralellik);
// çok sayıda BEKLEYEN işi az thread'le yürütmek ise async'in alanıdır → async-await. Send ve
// Sync'in neden böyle çalıştığı → ileri-send-sync. RwLock, Condvar, deadlock önleme ve async
// çalışma zamanı → ileri-eszamanlilik-async.

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn paralel_toplam_seri_toplamla_ayni() {
        let veri: Vec<u64> = (1..=100).collect();
        for parca in [1, 3, 7, 100, 500] {
            assert_eq!(paralel_topla(&veri, parca), 5050);
        }
        assert_eq!(paralel_topla(&[], 4), 0);
    }

    #[test]
    fn ayrik_parcalara_paralel_yazilir() {
        let mut veri = vec![1, 2, 3, 4, 5];
        paralel_karesini_al(&mut veri, 2);
        assert_eq!(veri, [1, 4, 9, 16, 25]);
    }

    #[test]
    fn havuz_her_isi_tam_bir_kez_isler() {
        let isler: Vec<Is> = (0..50)
            .map(|id| Is {
                id,
                metin: "a ".repeat(usize::try_from(id).unwrap()),
            })
            .collect();
        let sonuclar = havuzda_isle(isler, 4);
        assert_eq!(sonuclar.len(), 50);
        for (sira, s) in sonuclar.iter().enumerate() {
            assert_eq!(usize::try_from(s.id).unwrap(), sira);
            assert_eq!(s.kelime_sayisi, sira);
        }
    }

    #[test]
    fn kelime_sikligi_belgeleri_birlestirir() {
        let tablo = kelime_sikligi(&["a b", "B c", "c C c"]);
        let beklenen = BTreeMap::from([
            (String::from("a"), 1),
            (String::from("b"), 2),
            (String::from("c"), 4),
        ]);
        assert_eq!(tablo, beklenen);
    }

    #[test]
    fn atomik_sayac_artis_kaybetmez() {
        assert_eq!(paralel_say(4, 5_000), 20_000);
    }

    #[test]
    fn zehirli_kilit_kurtarilabilir() {
        let kasa = Arc::new(Mutex::new(vec![1]));
        let kasa2 = Arc::clone(&kasa);
        let sonuc = thread::spawn(move || {
            let mut g = kasa2.lock().unwrap();
            g.push(2);
            panic!("kasıtlı panik: kilit elimizdeyken");
        })
        .join();
        assert!(sonuc.is_err()); // join, thread'in paniğini Err olarak bildirir
        assert!(kasa.is_poisoned());
        assert!(kasa.lock().is_err()); // sıradan lock() artık Err
        let kurtarilan = kasa.lock().unwrap_or_else(|zehir| zehir.into_inner());
        assert_eq!(*kurtarilan, [1, 2]); // veri duruyor; tutarlı olup olmadığına BİZ karar veririz
    }
}
