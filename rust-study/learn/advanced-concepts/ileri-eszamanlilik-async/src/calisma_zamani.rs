// Açıklama: learn/advanced-concepts/ileri-eszamanlilik-async/ACIKLAMA.md
// BÖLÜM B — Mini async çalışma zamanı (runtime): görev kuyruğu, spawn, zamanlayıcı, zaman aşımı
//
// Böl. 17'nin executor'u tek bir kök future'ı döngüde poll ediyordu; "çok görev"i de her turda
// HEPSİNİ yoklayarak yürütüyordu. Gerçek bir runtime böyle çalışmaz: binlerce görevden yalnız
// UYANDIRILANI poll eder. Bunun için üç parça gerekir ve hepsi aşağıda, std ile:
//
//   Gorev       → bir future + "beni yeniden kuyruğa koy" bilgisi. Waker'ın kendisi olur.
//   Calistirici → hazır görevler kuyruğundan alır, bir kez poll eder; Pending ise bırakır.
//                 Görev ancak biri waker.wake() derse kuyruğa geri döner.
//   Uyku        → gerçek bir "olay kaynağı": süre dolunca waker'ı çağıran zamanlayıcı.
//
// Akış:  baslat(f) ─► [kuyruk] ─► poll ─► Ready → görev biter
//                        ▲          └──► Pending → görev kuyrukta DEĞİL, bekliyor
//                        └── wake() ◄── zamanlayıcı / başka görev / G/Ç olayı

// ============================================================================
// SADE ANLATIM — önce buradan başla
// ============================================================================
// Ana sözlük main.rs'in başındadır; önce onu oku. Bu dosyanın tek cümlelik özeti: bir async
// çalışma zamanı, hazır olan görevleri bir sıradan alıp birer kez poll eden ve bekleyen
// görevlere ancak zilleri çalınınca yeniden bakan küçük bir düzendir.
//
// MİNİ SÖZLÜK (bu dosyaya özgü yeni kelimeler):
//   görev kuyruğu        → poll edilmeye hazır görevlerin beklediği sıra.
//   mpsc kanal           → çok gönderenli, tek alıcılı boru; burada kuyruk olarak kullanılır.
//   Sender / Receiver    → kanalın gönderen ucu / alan ucu.
//   tembel (lazy)        → future kendiliğinden başlamaz; ancak poll edilince çalışır.
//   tür-silinmiş (dyn)   → gerçek türü gizlenmiş: farklı future'lar aynı kutuya sığsın diye.
//   Box::pin             → değeri heap'e koyup adresini sabitlemek.
//   block_on / koştur    → "bu future bitene kadar burada bekle" diyen giriş kapısı.
//   yaprak (leaf) future → başka future'ı beklemeyen, doğrudan bir dış olayı bekleyen future.
//   olay kaynağı         → "oldu" haberini üreten taraf: zamanlayıcı, ağ, başka bir thread.
//   birleştirici         → combinator: başka future'ları sarıp yeni davranış ekleyen future.
//   pin projeksiyonu     → sabitlenmiş bir yapının içindeki alana yine sabitlenmiş olarak
//                          ulaşmak.
//   get_mut              → Pin'in içinden sıradan `&mut` almak; yalnız Unpin türlerde vardır.
//   tahsis (allocation)  → heap'ten yer istemek.

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};
use std::thread;
use std::time::Duration;

// ============================================================================
// B1) Görev ve Waker
// ============================================================================
// Kutulu, tür-silinmiş, sabitlenmiş future: farklı async blokları aynı kuyruğa koyabilmek için
// `dyn`, poll edebilmek için `Pin`, kuyruğa taşıyabilmek için `Box` (taşınan kutudur; içindeki
// future'ın adresi değişmez — Pin'in istediği de budur).
// SADE ANLATIM: Üstüne dönüş adresi yazılmış bir iade zarfı düşün. Zarfın içinde yarım kalmış
// bir iş (future) var. İş beklemeye geçince zarf bir kenarda durur; kimse ona bakmaz. Biri
// "hazır" deyince (wake) zarf posta kutusuna atılır ve üstündeki adrese, yani hazır görevler
// sırasına geri döner. `Gorev` bu zarftır: future'ı ve döneceği sıranın adresini birlikte
// taşır. Bu neden var? Çalıştırıcı binlerce bekleyen görevi tek tek yoklamasın, yalnız
// hazır olanla ilgilensin diye.
//   Mini örnek:
//     impl Wake for Gorev {
//         fn wake(self: Arc<Self>) {
//             let kuyruk = self.kuyruk.clone();
//             let _ = kuyruk.send(self);   // uyandırmak = görevi hazır sırasına geri koymak
//         }
//     }
//     // ✓ wake() çağrılan görev yeniden poll edilir
//     // ✗ Pending dönüp kimseye waker bırakmayan görev bir daha hiç poll edilmez
type KutuluGorev = Pin<Box<dyn Future<Output = ()> + Send>>;

struct Gorev {
    // Option: bittiğinde None yaparız; böylece gecikmiş bir wake() bitmiş future'ı yeniden
    // poll edemez. Mutex: Waker başka thread'lerden (zamanlayıcı) çağrılabildiği için Gorev
    // Sync olmalı; future'a erişimi teke indiren şey bu kilittir.
    gelecek: Mutex<Option<KutuluGorev>>,
    kuyruk: Sender<Arc<Gorev>>,
}

// std::task::Wake: Arc<Gorev>'den güvenli bir Waker üretmenin yolu (ham vtable yazmadan;
// içerideki mekanizma için ileri-future). "Uyandırmak" = görevi hazır kuyruğuna geri koymak.
impl Wake for Gorev {
    fn wake(self: Arc<Self>) {
        let kuyruk = self.kuyruk.clone();
        // Alıcı (çalıştırıcı) kapanmışsa gönderilemez; o zaman uyandıracak kimse de yoktur.
        let _ = kuyruk.send(self);
    }
}

// ============================================================================
// B2) Çalıştırıcı ve Başlatıcı
// ============================================================================
// SADE ANLATIM: Bankada sıra numarası alırsın ve oturursun; gişe görevlisi yalnız numarası
// yanan kişiyle ilgilenir. Evrağı eksik olan kişi gişenin önünde dikilmez; yerine oturur,
// evrağı gelince yeni numara alır. `Calistirici` gişe görevlisidir: sıradan bir görev alır,
// bir kez poll eder; görev "daha bitmedi" derse onu bırakır ve sıradakine geçer. Sıra boşsa
// görevli de uyur. `Baslatici` numara veren makinedir: yeni görevi sıraya koyar.
//   Mini örnek:
//     let mut calistirici = Calistirici::yeni();
//     let t = calistirici.baslatici().baslat(async { 6 * 7 });  // sıraya girdi, çalışmadı
//     assert_eq!(t.sonucu_al(), None);          // henüz poll edilmedi: sonuç yok
//     calistirici.calistir();                   // gişe açıldı: görev poll edildi ve bitti
//     assert_eq!(t.sonucu_al(), Some(42));      // ✓
pub struct Calistirici {
    hazir: Receiver<Arc<Gorev>>,
    baslatici: Baslatici,
    poll_sayisi: usize,
}

// Görev başlatma yetkisi: klonlanıp görevlerin İÇİNE taşınabilir (bir görev başka görev başlatır).
#[derive(Clone)]
pub struct Baslatici {
    kuyruk: Sender<Arc<Gorev>>,
    canli: Arc<AtomicUsize>, // henüz bitmemiş görev sayısı → çalıştırıcı ne zaman duracağını bilir
}

impl Baslatici {
    // SADE ANLATIM: Kuryeye paket veriyorsun. Kurye paketi başka bir binaya götürebilir;
    // paket yolculuğa dayanmalı (`Send`). Kurye paketi sen evden çıktıktan sonra da teslim
    // edebilir; paketin içinden senin masandaki bir eşyaya bağlı ip çıkmamalı (`'static`).
    // `'static` burada "sonsuza dek yaşar" demek değildir; "geçici hiçbir şeye borçlu değil"
    // demektir. Görev, onu başlatan fonksiyon bittikten sonra da çalışabildiği için bu iki
    // şart konur.
    //   Mini örnek:
    //     let ad = String::from("ada");
    //     baslatici.baslat(async move { println!("{ad}") });  // ✓ ad'ın sahipliği göreve geçti
    //     // baslatici.baslat(async { println!("{ad}") });    // ✗ derlenmez: görev ad'ı yalnız
    //     //                                                  //   ödünç alıyor, 'static değil
    // Bound'ları oku — her biri bir tasarım kararının sonucudur:
    //   F: Send    → görev, Waker aracılığıyla başka thread'lerden erişilebilir bir Arc'ın
    //                içinde durur. Bu yüzden await noktaları arasında Rc ya da MutexGuard
    //                taşıyan future'lar burada DERLENMEZ (istemci.rs'te "sık hatalar").
    //   F: 'static → görev, onu başlatan fonksiyondan uzun yaşayabilir (thread::spawn ile aynı
    //                gerekçe); yerel değişken ödünç alamaz, sahiplenmeli (`async move`).
    pub fn baslat<F>(&self, gelecek: F) -> Tutamac<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let paylasilan = Arc::new(Mutex::new(TutamacDurumu {
            sonuc: None,
            bekleyen: None,
        }));
        let gorev_tarafi = Arc::clone(&paylasilan);
        // Kullanıcının future'ını sarıyoruz: bitince sonucu tutamaca yaz ve tutamacı bekleyen
        // bir görev varsa onu uyandır.
        let sarili = async move {
            let deger = gelecek.await;
            let bekleyen = {
                let mut durum = gorev_tarafi.lock().unwrap();
                durum.sonuc = Some(deger);
                durum.bekleyen.take()
            }; // kilit burada bırakıldı; wake()'i kilit DIŞINDA çağırıyoruz
            if let Some(waker) = bekleyen {
                waker.wake();
            }
        };
        self.canli.fetch_add(1, Ordering::SeqCst);
        let gorev = Arc::new(Gorev {
            gelecek: Mutex::new(Some(Box::pin(sarili))),
            kuyruk: self.kuyruk.clone(),
        });
        // İlk poll için kuyruğa koy. (Future tembeldir: kuyruğa girmeyen görev hiç başlamaz.)
        let _ = self.kuyruk.send(gorev);
        Tutamac { paylasilan }
    }
}

impl Calistirici {
    pub fn yeni() -> Self {
        let (kuyruk, hazir) = mpsc::channel();
        Calistirici {
            hazir,
            baslatici: Baslatici {
                kuyruk,
                canli: Arc::new(AtomicUsize::new(0)),
            },
            poll_sayisi: 0,
        }
    }

    pub fn baslatici(&self) -> Baslatici {
        self.baslatici.clone()
    }

    // Tüm görevler bitene dek çalışır. Dönen değer: toplam poll sayısı (ne kadar "iş" yapıldı).
    // Sınır: sayaç 0'a inip döngü çıktıktan SONRA başka bir thread'den başlatılan görev poll
    // edilmez; görevler ya calistir()'dan önce ya da çalışan bir görevin içinden başlatılmalı.
    pub fn calistir(&mut self) -> usize {
        while self.baslatici.canli.load(Ordering::SeqCst) > 0 {
            // recv(): hazır görev yoksa bu thread UYUR — boşa dönme yok. Bir zamanlayıcı ya da
            // başka thread wake() deyince kanal dolar ve uyanırız. Hiçbir görev kendini
            // uyandırmayı ayarlamadan Pending dönerse burada sonsuza dek bekleriz: "waker'ı
            // kaydetmeyi unutma" kuralının çalışma anındaki bedeli.
            let Ok(gorev) = self.hazir.recv() else { break };
            let mut yuva = gorev.gelecek.lock().unwrap();
            let Some(mut gelecek) = yuva.take() else {
                continue; // zaten bitmiş bir göreve gecikmiş/yinelenen wake — yok say
            };
            let waker = Waker::from(Arc::clone(&gorev));
            let mut cx = Context::from_waker(&waker);
            self.poll_sayisi += 1;
            match gelecek.as_mut().poll(&mut cx) {
                Poll::Pending => *yuva = Some(gelecek), // yerine koy; wake gelince yine poll edilir
                Poll::Ready(()) => {
                    self.baslatici.canli.fetch_sub(1, Ordering::SeqCst);
                }
            }
        }
        self.poll_sayisi
    }
}

// Tek future'ı sonuna dek koşturan kısa yol (tokio'daki Runtime::block_on'un karşılığı).
pub fn kostur<F>(gelecek: F) -> F::Output
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let mut calistirici = Calistirici::yeni();
    let tutamac = calistirici.baslatici().baslat(gelecek);
    calistirici.calistir();
    tutamac.sonucu_al().expect("görev bitti ama sonuç yok")
}

// ============================================================================
// B3) Tutamac — başlatılan görevin sonucunu beklemek (JoinHandle)
// ============================================================================
// SADE ANLATIM: Kuru temizlemeciye gömlek bırakınca sana bir fiş verirler. Gömleği almaya
// gittiğinde hazırsa alırsın; hazır değilse telefon numaranı bırakırsın, hazır olunca seni
// ararlar. `Tutamac` bu fiştir: görevin sonucu bir kutuda birikir, sen `.await` deyince ya
// sonucu alırsın ya da waker'ını (numaranı) bırakıp beklersin. "Hazır mı?" diye bakmak ile
// numarayı bırakmak aynı anda, tek kilit altında yapılır; arada boşluk kalırsa arama kaçar.
//   Mini örnek:
//     let t1 = baslatici.baslat(async { 1 + 1 });
//     let toplam = t1.await + 10;      // ✓ başka bir görevin içinde: sonuç gelince 12 olur
//     let t2 = baslatici.baslat(async { 5 });
//     calistirici.calistir();          // görevler bitti
//     let s = t2.sonucu_al();          // ✓ async dışında: Some(5)
struct TutamacDurumu<T> {
    sonuc: Option<T>,
    bekleyen: Option<Waker>,
}

pub struct Tutamac<T> {
    paylasilan: Arc<Mutex<TutamacDurumu<T>>>,
}

impl<T> Tutamac<T> {
    // Çalıştırıcı durduktan sonra, async bağlam dışında sonucu almak için.
    pub fn sonucu_al(&self) -> Option<T> {
        self.paylasilan.lock().unwrap().sonuc.take()
    }
}

// Tutamac'ın kendisi de bir Future: başka bir görev `tutamac.await` diyerek sonucu bekler.
impl<T> Future for Tutamac<T> {
    type Output = T;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        // DİKKAT — denetim ve waker kaydı AYNI kilit altında. İki ayrı adımda yapsaydık:
        //   1) "sonuç var mı?" → yok        2) (görev tam şimdi bitip `bekleyen`e baktı: boş)
        //   3) waker'ı kaydet → ama artık kimse çağırmayacak → sonsuza dek Pending.
        // Bu "kayıp uyandırma" (lost wakeup) Condvar'daki ile aynı hatadır; çözümü de aynı.
        let mut durum = self.paylasilan.lock().unwrap();
        match durum.sonuc.take() {
            Some(deger) => Poll::Ready(deger),
            None => {
                durum.bekleyen = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

// ============================================================================
// B4) Uyku — gerçek bir olay kaynağı: zamanlayıcı
// ============================================================================
// Böl. 17'deki sahte future'lar Pending dönerken kendilerini HEMEN uyandırıyordu. Gerçek bir
// yaprak (leaf) future şunu yapar: waker'ı, olayı bilecek tarafa teslim eder ve Pending döner.
// Burada o taraf, süre kadar uyuyan bir yardımcı thread. Üretim runtime'ları tek zamanlayıcı + OS
// olay bildirimi kullanır; ilke aynıdır: olay olunca waker.wake().
// SADE ANLATIM: Sabah kalkmak için saatin başında oturup akrebi izlemezsin; çalar saati
// kurar ve uyursun. `Uyku` böyle çalışır: ilk poll'da saati kurar (süre kadar uyuyan yardımcı
// bir thread), zili (waker) saate teslim eder ve "daha olmadı" der. Süre dolunca saat zili
// çalar, görev sıraya geri döner ve kaldığı yerden sürer. Bu neden var? Bir görev beklerken
// çalıştırıcının thread'i boşta kalsın ve öteki görevlere bakabilsin diye.
//   Mini örnek:
//     kostur(async {
//         uyu_ms(20).await;        // ✓ görev kenara çekilir; 20 ms sonra zil çalar, devam eder
//     });
//     let u = uyu_ms(20);          // yalnız oluşturuldu: saat henüz kurulmadı
//     drop(u);                     // hiç poll edilmedi → hiç beklemedi, thread de açılmadı
//     // thread::sleep(..) ise ✗ bütün çalıştırıcıyı uyutur (ayrıntı: istemci.rs, C2)
pub struct Uyku {
    sure: Duration,
    basladi: bool,
    durum: Arc<Mutex<UykuDurumu>>,
}

struct UykuDurumu {
    doldu: bool,
    waker: Option<Waker>,
}

impl Uyku {
    pub fn yeni(sure: Duration) -> Self {
        Uyku {
            sure,
            basladi: false,
            durum: Arc::new(Mutex::new(UykuDurumu {
                doldu: false,
                waker: None,
            })),
        }
    }
}

pub fn uyu_ms(ms: u64) -> Uyku {
    Uyku::yeni(Duration::from_millis(ms))
}

impl Future for Uyku {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        // Uyku'nun bütün alanları Unpin olduğu için Uyku da Unpin'dir: sabitlenmiş olsa bile
        // güvenle &mut alabiliriz (get_mut yalnız Unpin türlerde vardır). Kendine referans
        // tutmayan, elle yazılmış future'ların çoğu böyledir.
        let bu = self.get_mut();
        let mut durum = bu.durum.lock().unwrap();
        if durum.doldu {
            return Poll::Ready(());
        }
        // Waker'ı HER poll'da güncelle: görev araya başka bir waker'la poll edilmiş olabilir
        // (ör. bir birleştiricinin içine taşındıysa); eskisini uyandırmak işe yaramazdı.
        durum.waker = Some(cx.waker().clone());
        drop(durum); // waker kaydedildi; thread başlatmak gibi pahalı bir işi kilit altında yapma
        if !bu.basladi {
            bu.basladi = true; // zamanlayıcıyı İLK poll'da kur (tembellik: poll edilmeyen uyumaz)
            let paylasilan = Arc::clone(&bu.durum);
            let sure = bu.sure;
            thread::spawn(move || {
                thread::sleep(sure);
                let waker = {
                    let mut durum = paylasilan.lock().unwrap();
                    durum.doldu = true;
                    durum.waker.take()
                };
                if let Some(waker) = waker {
                    waker.wake(); // görevi hazır kuyruğuna geri koyar
                }
            });
        }
        Poll::Pending
    }
}

// ============================================================================
// B5) ZamanAsimi — iki future'ı yarıştırmak; Pin ve Unpin pratikte
// ============================================================================
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ZamanAsimiHatasi;

// Bir birleştirici (combinator), içindeki future'ı poll etmek için ona Pin<&mut _> vermelidir.
// Elimizde `Pin<&mut ZamanAsimi>` var; alanlara nasıl ineriz? İki yol:
//
//  (1) Alanı kendi başına sabitle: `Pin<Box<dyn Future>>`. Kutu Unpin'dir (taşınan kutudur),
//      dolayısıyla tüm struct Unpin olur ve `get_mut()` ile sıradan &mut alırız. Bedeli bir
//      heap tahsisi. Aşağıdaki çözüm bu — tamamen güvenli kod.
//  (2) "Pin projeksiyonu": `Pin<&mut Struct>`ten `Pin<&mut Alan>` üretmek. Tahsis yok ama
//      doğruluğu elle kanıtlamak gerekir (unsafe); pratikte pin-project / pin-project-lite
//      crate'leri bunu güvenli bir makroyla yapar. Kütüphane yazarken tercih edilen yol.
//
//   ✗ Kısayol yok: `Pin::new(&mut self.ic)` yalnız `F: Unpin` ise derlenir ve async blokların
//     ürettiği future'lar Unpin DEĞİLDİR:
//         E0277: `F` cannot be unpinned  (… note: consider using the `pin!` macro)
// SADE ANLATIM: Masaya bir kum saati koyup işe başlıyorsun. İş kum bitmeden biterse sonucu
// alırsın. Kum önce biterse işi olduğu yerde bırakırsın. `ZamanAsimi` iki future'ı yarıştırır:
// asıl iş ve bir `Uyku`. Async'te "işi bırakmak" için ayrı bir düğme yoktur: future'ı poll
// etmeyi kesip düşürürsün, iş orada biter. Bu yüzden async kod her `.await` noktasında yarıda
// kalabileceğini hesaba katmalıdır.
//   Mini örnek:
//     let hizli = kostur(zaman_asimi(Duration::from_millis(200), async { 42 }));
//     assert_eq!(hizli, Ok(42));                    // ✓ iş kum saatinden önce bitti
//     let yavas = kostur(zaman_asimi(Duration::from_millis(10), async {
//         uyu_ms(300).await;                        // kum saatinden uzun sürüyor
//         "bitti"                                   // buraya hiç gelinmez: future düşürüldü
//     }));
//     assert_eq!(yavas, Err(ZamanAsimiHatasi));     // ✓ süre doldu, iş iptal edildi
pub struct ZamanAsimi<T> {
    ic: Pin<Box<dyn Future<Output = T> + Send>>,
    sayac: Uyku,
}

pub fn zaman_asimi<F>(sure: Duration, gelecek: F) -> ZamanAsimi<F::Output>
where
    F: Future + Send + 'static,
{
    ZamanAsimi {
        ic: Box::pin(gelecek),
        sayac: Uyku::yeni(sure),
    }
}

impl<T> Future for ZamanAsimi<T> {
    type Output = Result<T, ZamanAsimiHatasi>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let bu = self.get_mut(); // iki alan da Unpin → struct Unpin → güvenli
        // Önce asıl işi yokla: aynı turda ikisi de hazırsa sonucu tercih ederiz.
        if let Poll::Ready(deger) = bu.ic.as_mut().poll(cx) {
            return Poll::Ready(Ok(deger));
        }
        // Uyku Unpin olduğu için Pin::new yeterli.
        if Pin::new(&mut bu.sayac).poll(cx).is_ready() {
            // Süre doldu. `ic` bu struct ile birlikte DÜŞECEK → iş iptal olur. Async'te iptal
            // budur: future'ı poll etmeyi bırakıp düşürmek. Ayrı bir "cancel" çağrısı yoktur;
            // bu yüzden bir future her await noktasında yarıda kesilmeye hazır olmalıdır.
            return Poll::Ready(Err(ZamanAsimiHatasi));
        }
        Poll::Pending
    }
}

#[cfg(test)]
mod testler {
    use super::*;
    use std::time::Instant;

    #[test]
    fn kosur_sonucu_dondurur() {
        assert_eq!(kostur(async { 6 * 7 }), 42);
    }

    #[test]
    fn uyku_en_az_istenen_kadar_surer() {
        let baslangic = Instant::now();
        kostur(uyu_ms(20));
        assert!(baslangic.elapsed() >= Duration::from_millis(20));
    }

    #[test]
    fn gorev_baska_gorev_baslatip_sonucunu_bekler() {
        let mut calistirici = Calistirici::yeni();
        let baslatici = calistirici.baslatici();
        let ic_baslatici = baslatici.clone();
        let tutamac = baslatici.baslat(async move {
            let a = ic_baslatici.baslat(async {
                uyu_ms(10).await;
                1
            });
            let b = ic_baslatici.baslat(async { 2 });
            a.await + b.await
        });
        calistirici.calistir();
        assert_eq!(tutamac.sonucu_al(), Some(3));
    }

    #[test]
    fn calistirici_yalniz_uyandirilan_gorevi_poll_eder() {
        let mut calistirici = Calistirici::yeni();
        let baslatici = calistirici.baslatici();
        for _ in 0..10 {
            baslatici.baslat(uyu_ms(15));
        }
        // Her görev: 1 ilk poll (Pending) + süre dolunca 1 poll (Ready) = 2. Her turda hepsini
        // yoklayan bir executor burada binlerce poll yapardı.
        assert_eq!(calistirici.calistir(), 20);
    }

    #[test]
    fn zaman_asimi_hizli_isi_gecirir_yavasi_keser() {
        let hizli = kostur(zaman_asimi(Duration::from_millis(200), async {
            uyu_ms(5).await;
            "bitti"
        }));
        assert_eq!(hizli, Ok("bitti"));
        let yavas = kostur(zaman_asimi(Duration::from_millis(10), async {
            uyu_ms(300).await;
            "bitti"
        }));
        assert_eq!(yavas, Err(ZamanAsimiHatasi));
    }
}
