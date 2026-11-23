// Açıklama: learn/async-await/ACIKLAMA.md
// Rust Book — Bölüm 17: Async/Await (bkz. ch17-00)
// Çalıştır: cargo run -p async-await
//
// Async, EŞZAMANLILIK (concurrency) içindir: tek thread'de, BEKLEME anlarında (I/O, zamanlayıcı)
// başka işe geçerek çok işi iç içe yürütmek. Paralellikle (aynı anda çok çekirdek, Böl. 16)
// karıştırma: async bir işi BÖLÜP bekleme boşluklarını doldurur; thread'ler işi gerçekten aynı
// anda koşturur. İkisi birleştirilebilir.
//
// ÖNEMLİ: std `async`/`await` SÖZ DİZİMİNİ ve Future trait'ini verir ama bir ÇALIŞTIRICI
// (runtime/executor) VERMEZ. Gerçek projede tokio/async-std kullanılır; burada "kökü görmek"
// için executor'u ELLE, std ile yazıyoruz — böylece poll/Waker mekanizması çıplak görünür.

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Her bölümün "SADE ANLATIM" kutusunu önce oku; teknik kısım zorsa ana fikir orada.
// Benzetmeler EVDE, ev işi yapan TEK kişiyle geçer; derinlesme.rs de orada sürer. En zor yer
// (mekanizma) başta; takılırsan "async ve await KEYWORD'LERİ" kutusuna bak.
//
// Bu bölümün tek cümlelik özeti: async, tek kişinin beklerken boş durmamasıdır: çamaşır makinesi
// dönerken bulaşığı yıkarsın; işler aynı anda değil, İÇ İÇE ilerler.
//
// MİNİ SÖZLÜK (bu dosyada geçen kelimeler):
//   async                    → "bu işi şimdi yapma, iş kartını hazırla" diyen anahtar kelime.
//   Future                   → o iş kartı: "sonra hazır olacak" sonuç. Biri bakmadıkça ilerlemez.
//   .await                   → "bu kartı şimdi takip et; beklemek gerekirse sırayı başkasına ver".
//   tembel (lazy)            → isteyen olmadıkça çalışmayan. Future'lar tembeldir.
//   poll (yoklamak)          → karta gidip bakmak ve işi ilerleyebildiği kadar ilerletmek.
//   Poll::Ready / Poll::Pending → bakınca alınan iki cevap: "bitti, sonuç bu" / "daha değil".
//   executor / runtime (çalıştırıcı) → kartlara bakan kişi: poll eder, bekler, zil çalınca döner.
//   Waker / wake             → iş hazır olunca çalan zil / zili çalmak.
//   Context (cx)             → poll'a verilen küçük çanta; içinde o işin zili (Waker) durur.
//   park / unpark            → thread'in uyuması / uyuyan thread'in dürtülüp uyandırılması.
//   geri-çağrı (callback)    → "şu olunca bu fonksiyonu çağır" diye önceden verilen iş.
//   Pin, pin!, Box::pin      → bir değeri bellekte yerinden oynamayacak biçimde sabitlemek.
//   state machine (durum makinesi) → "hangi adımda kaldım, elimde ne var" bilgisini saklayan yapı.
//   eşzamanlılık (concurrency) → işlerin aynı dönemde iç içe İLERLEMESİ; tek kişiyle de olur.
//   paralellik (parallelism) → işlerin gerçekten AYNI ANDA, birden çok kişiyle yapılması.
//   I/O (G/Ç)                → dosya, ağ gibi dış dünyadan veri beklenen işler.
//   round-robin              → sırayla herkese bir tur vermek.
//   interleaving             → iki işin adımlarının birbirinin arasına girerek ilerlemesi.
//   busy-loop / busy-wait    → durmadan "oldu mu?" diye sorup işlemciyi boşa yormak.
//   yield                    → "sıramı savıyorum, bir tur da başkası ilerlesin" demek.
//   desugaring               → derleyicinin kısa yazımı açtığı uzun hâl.
//   vtable / RawWaker        → zilin "çalınca hangi fonksiyon çalışsın" listesi / zilin ham hâli.
//   self-referential         → kendi içindeki bir parçayı gösteren (kendine ok çizen) değer.
//   Unpin                    → "beni taşımak sorun çıkarmaz" etiketi taşıyan türler.
//   tokio / async-std        → hazır executor sunan, en çok kullanılan dış kütüphaneler.

use std::future::Future;
use std::pin::{Pin, pin};
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use std::thread::{self, Thread};

mod derinlesme;

fn main() {
    // (1) Tek future'ı sonuna kadar koştur:
    let s = block_on(akis());
    println!("block_on döndü: {s}");

    // (2) Aynı thread'de İKİ görevi İÇ İÇE (eşzamanlı) koştur:
    println!("-- iki görev eşzamanlı --");
    cok_gorev(vec![Box::pin(gorev(1)), Box::pin(gorev(2))]);
    derinlesme::calistir();
}

// ============================================================================
// ELLE MİNİMAL EXECUTOR (std'nin verdikleriyle)
// ============================================================================

// Waker: future "artık ilerleyebilirim" dediğinde executor'u uyandıran geri-çağrı. std::task::Wake
// trait'i, ham RawWaker/vtable yazmadan Arc'tan Waker üretmemizi sağlar. Burada "uyandır" =
// bekleyen (park etmiş) thread'i çöz (unpark).
// SADE ANLATIM: Çamaşır makinesinin başında durup "bitti mi?" diye durmadan bakmazsın. Makine
// bitince zil çalar, sen de o zaman gidersin. Waker o zildir. Bir iş "daha hazır değilim" derken
// zili yanına alır; hazır olunca `wake()` ile çalar. Bu dosyada zilin çalması, koltukta uyuyan
// kişiyi (thread'i) dürtmek demektir: `unpark()`.
// Zil olmasaydı durmadan bakmak (CPU'yu boşa yormak) ya da hiç öğrenememek gerekirdi.
// (Sade: vtable = zil çalınca hangi fonksiyonun çalışacağı listesi; Wake onu bizim yerimize hazırlar.)
//   Mini örnek:
//     let zil = Waker::from(Arc::new(ThreadWaker(thread::current())));
//     zil.wake_by_ref();     // ✓ zil çaldı: "kalk, işin hazır"
//     thread::park();        // ✓ zil az önce çaldığı için uyuyup kalmaz, hemen döner
struct ThreadWaker(Thread);
impl Wake for ThreadWaker {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}

// block_on: bir future'ı bitene kadar DÖNGÜde poll eder. Poll::Pending gelince thread'i park
// eder (boşa dönmez); future kendini uyandırınca (wake) unpark olup yeniden poll edilir.
// SADE ANLATIM: Executor evdeki kişidir. `block_on` tek bir işi sonuna kadar takip eder: gider,
// bakar (poll). Cevap "bitti" (Ready) ise sonucu alır. Cevap "daha değil" (Pending) ise koltuğa
// oturup uyuklar (`park`); zil çalınca kalkar, yeniden bakar.
// Rust'ın standart kütüphanesi bu "bakan kişiyi" vermez; gerçek projelerde tokio gibi bir
// kütüphane verir. Burada nasıl çalıştığını görmek için kendimiz yazıyoruz.
//   Mini örnek:
//     let kart = async { 1 + 1 };       // iş kartı hazır; henüz kimse bakmadı, toplama yapılmadı
//     let sonuc = block_on(kart);       // ✓ 2 — bakan biri oldu, iş yapıldı
//     // let n: i32 = async { 2 };      // ✗ derlenmez: kart sonucun kendisi değildir
fn block_on<F: Future>(fut: F) -> F::Output {
    let mut fut = pin!(fut); // Future poll edilmek için PIN'lenmeli (yeri sabitlensin — aşağıda neden)
    let waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
    let mut cx = Context::from_waker(&waker);
    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(v) => return v,
            Poll::Pending => thread::park(), // uyandırılana dek uyu
        }
    }
}

// Çok görevli mini executor: aynı thread'de round-robin poll. Hazır olan görev listeden çıkar;
// kalan Pending görevler bir sonraki tura kalır → EŞZAMANLILIK (interleaving) tek thread'de.
// SADE ANLATIM: Çamaşır makinesi dönerken başında beklemezsin, bulaşığa geçersin. Bulaşık suda
// beklerken dönüp çamaşıra bakarsın. Tek kişisin, yine de iki iş iç içe ilerler. `cok_gorev` de
// listedeki her işe sırayla bir kez bakar; biten işi listeden çıkarır, kalanlara sonraki turda
// yeniden bakar.
// Tek thread var; kazanç paralellikten değil, bekleme sürelerinin boşa gitmemesinden (eşzamanlılık).
//   Mini örnek:
//     cok_gorev(vec![Box::pin(gorev(1)), Box::pin(gorev(2))]);
//     // ✓ ekrana yazılan sıra: 1 başladı, 2 başladı, 1 bitti, 2 bitti (işler iç içe ilerledi)
//     // işler tek tek yapılsaydı: 1 başladı, 1 bitti, 2 başladı, 2 bitti
fn cok_gorev(mut gorevler: Vec<Pin<Box<dyn Future<Output = ()>>>>) {
    let waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
    let mut cx = Context::from_waker(&waker);
    while !gorevler.is_empty() {
        // her görevi bir kez poll et; Ready olanı at (retain = Pending olanları tut)
        gorevler.retain_mut(|g| g.as_mut().poll(&mut cx).is_pending());
        if !gorevler.is_empty() {
            thread::park(); // bu turda uyanan görevler unpark etmiştir → hemen döner
        }
    }
}

// ============================================================================
// ELLE YAZILMIŞ FUTURE'LAR — poll/Waker'ı çıplak göster
// ============================================================================

// Bir kez "Pending" dönüp kendini uyandıran, sonra "Ready" olan future. Gerçek dünyada Pending
// = "I/O daha hazır değil"; burada tek bir tur geciktiriyoruz.
// SADE ANLATIM: Bir Future, üzerinde tek soru yazan bir iş kartıdır: "ilerleyebildiğin kadar
// ilerle, sonra durumunu söyle" (poll). İki cevap vardır: `Ready(sonuç)` "bitti" demektir,
// `Pending` ise "daha değil, hazır olunca zili çalarım". Aşağıdaki kart ilk bakışta "daha değil"
// der ve zili hemen çalar; ikinci bakışta 42 verir. Gerçek hayatta "daha değil", ağdan cevap
// beklemek gibi bir durumdur.
// Neden zili çalmak şart? Pending deyip zili çalmayan bir işe bir daha kimse bakmaz.
//   Mini örnek (kod değil, olay sırası):
//     1. poll → Poll::Pending       // "daha değil"; zil çalındı (wake_by_ref)
//     2. poll → Poll::Ready(42)     // ✓ "bitti, sonuç 42"
//     Pending dönüp zili çalmasaydı // ✗ derlenir ama block_on sonsuza dek uyur
struct BirKezBekle {
    hazir: bool,
}

impl Future for BirKezBekle {
    type Output = i32;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<i32> {
        if self.hazir {
            Poll::Ready(42)
        } else {
            self.hazir = true;
            cx.waker().wake_by_ref(); // "beni tekrar poll et" → executor'u uyandır
            Poll::Pending // şimdilik kontrolü geri ver
        }
    }
}

// Bir tur kontrolü bırakıp geri gelen "yield" future'ı (Output = ()).
// (Sade: bu kart "bir tur da öteki iş ilerlesin" demek için vardır; ilk bakışta sırayı bırakır,
// ikinci bakışta biter. `cok_gorev` çıktısındaki iç içe sıra bunun sayesinde oluşur.)
struct Teslim(bool);
impl Future for Teslim {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.0 {
            Poll::Ready(())
        } else {
            self.0 = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

// ============================================================================
// async ve await KEYWORD'LERİ — tam olarak NE YAPAR, NASIL ÇALIŞIR?
// ============================================================================
// SADE ANLATIM: `async` "bu tarifi şimdi yapma, kartını hazırla" demektir. Kartı hazırlamak
// yemeği pişirmez; `async fn` çağırınca elinde yalnız bir kart (Future) olur. `.await` ise "bu
// kartı şimdi takip et" demektir: iş beklemeye girerse sıra başka işe geçer, hazır olunca tam
// kaldığın satırdan devam edersin. Kaldığın yeri hatırlamak için derleyici fonksiyonu ayraçlı
// bir deftere çevirir; buna state machine (durum makinesi) denir.
//   Mini örnek:
//     async fn cay() -> u32 { 1 }
//     async fn mutfak() {
//         let kart = cay();           // kart hazır; cay'ın gövdesi ÇALIŞMADI
//         let n = kart.await;         // ✓ şimdi çalıştı, n = 1
//     }
//     fn duz() { cay().await; }       // ✗ derlenmez: `.await` yalnız async içinde kullanılır
//
// İki AYRI iş yapan iki keyword; karıştırma:
//
// • `async` (fn ya da blok önünde): gövdeyi ÇALIŞTIRMAZ. Onu, gövdeyi uygulayan bir FUTURE
//   değerine çevirir (derleyici arka planda bir state machine üretir). `async fn f() -> T`'nin
//   gerçek dönüş tipi `impl Future<Output = T>`'dir. Yani `f()` çağırmak SIFIR iş yapar — sadece
//   "henüz başlamamış hesap"ı (future'ı) üretir; iş, biri onu poll edene dek BAŞLAMAZ (tembel).
//
// • `.await` (bir future üzerinde): o future'ı, içinde bulunduğun async gövdeden SÜRER (poll eder).
//   Anlamı kabaca şu desugaring'tir (gerçekte state machine'in bir DURUMU):
//       let v = fut.await;
//     ≈   // fut, state machine'in kendi (pinli) alanında saklanır
//         loop {
//             match fut.poll(cx) {                     // (fut makinenin içinde pinli tutulur)
//                 Poll::Ready(v) => break v,           // hazır → değeri al, gövdeye DEVAM et
//                 Poll::Pending  => return Poll::Pending, // durumu sakla, kontrolü ÜST'e (executor)
//                     // geri ver; bu görev yeniden poll edilince TAM BU await noktasından sürer
//                     // (loop en baştan değil — makine hangi durumda kaldığını hatırlar).
//             }
//         }
//   `.await` busy-loop DEĞİL: Pending'de döner, waker.wake() ile yeniden poll edilir; değeri Output'tur.
//
// • Neden yalnız `async` içinde? "Pending'de durumu sakla, sonra buradan devam et" ancak state
//   machine'de mümkün; normal fonksiyonda yok → derleme hatası. Düz `main` de async olamaz; runtime
//   gerekir (biz block_on ile köprü kurduk).
//
// `akis` iki await noktasıyla (BirKezBekle, selam) iki DURUMLU bir makineye derlenir. ELLE yazdığımız
// BirKezBekle, derleyicinin o makineyi (poll → Pending/Ready → wake) nasıl kurduğunun küçük hâli.
// ============================================================================
// async fn'ler — derleyici bunları birer state machine'e çevirir
// ============================================================================

// SADE ANLATIM: Aşağıdaki `akis` iki yerde bekler; yani defterinde iki ayraç yeri vardır. Onu
// `block_on(akis())` ile çalıştırınca olanlar sırasıyla şöyledir.
// Her `.await` bir duraktır; iki `.await` arasındaki kod hiç bölünmeden çalışır.
//   Mini örnek (olay sırası):
//     1. poll → BirKezBekle "daha değil" der; akis kaldığı yeri saklar ve Pending döner
//     2. poll → BirKezBekle 42 verir; selam hemen biter; akis 43 ile Ready döner  ✓
async fn selam(ad: &str) -> String {
    format!("Selam {ad}")
}

async fn akis() -> i32 {
    // .await: future buraya yerleşir; Pending'de kontrol executor'a döner, hazır olunca buradan sürer.
    let n = BirKezBekle { hazir: false }.await; // 42 (bir tur Pending'den sonra)
    let s = selam("Kaan").await; // iç içe await
    println!("  akis içinden: {s}");
    n + 1
}

async fn gorev(id: u32) {
    println!("  görev {id}: başladı");

    Teslim(false).await; // bu noktada kontrolü diğer göreve bırak
    //
    println!("  görev {id}: bitti");
}

// SADE ANLATIM (aşağıdaki bölüm için): Aşağıdaki bölüm dört soruyu derleyicinin gözünden
// cevaplar: async fonksiyon neye dönüşür (kaldığı yeri hatırlayan bir deftere), poll ile zil
// nasıl birlikte çalışır, executor'u neden dışarıdan bir kütüphane verir ve Pin ne işe yarar.
// Pin için akılda kalacak resim: defterin bir sayfasında "üç sayfa ilerideki listeye bak" gibi,
// defterin KENDİ içini gösteren notlar var. Bu notlar yeri adresle gösterdiği için defter başka
// rafa taşınırsa yanlış yeri gösterirler. Pin, "bu defter artık yerinden oynatılmayacak"
// sözüdür. İlk okuyuşta bu bölümü atlayabilirsin; gündelik kodda Pin'i çoğu zaman hiç görmezsin.
// ============================================================================
// DERLEYİCİ PERSPEKTİFİ — async aslında nedir?
// ============================================================================
//
// • `async fn`/`async {}` bir STATE MACHINE'e (durum makinesi) derlenir. Derleyici, fonksiyonu
//   her `.await` noktasında duraklayıp sonra devam edebilen bir enum'a çevirir: her durum, o ana
//   kadarki yerel değişkenleri saklar. Yani async fn çağırmak HİÇBİR İŞ YAPMAZ; sadece bu durum
//   makinesini (bir Future değerini) ÜRETİR — "future'lar tembeldir". İş, biri onu poll edene dek
//   başlamaz. (Bölüm 13 iterator tembelliğiyle aynı felsefe.)
//
// • Future trait'i tek metotludur:
//       fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output>
//   poll "ilerleyebildiğin kadar ilerle" der; dönüş ya Poll::Ready(deger) ya Poll::Pending.
//   Pending dönerken future, cx içindeki WAKER'ı saklar; hazır olunca (I/O geldi, timer doldu)
//   waker.wake() çağırır → executor onu yeniden poll eder. Boşa dönme (busy-wait) YOK: wake
//   tabanlı. block_on'da bunu thread park/unpark ile canlandırdık.
//
// • Pin NEDEN? State machine, bir await'ten sonra kendi yerel değişkenine REFERANS tutabilir
//   (self-referential). Böyle bir değer bellekte TAŞINIRSA o iç referanslar bozulurdu. Pin<&mut T>
//   "bu değer artık yerinden oynatılamaz" garantisi verir; poll bu yüzden Pin alır. Future'ı poll
//   etmeden önce pin!/Box::pin ile sabitlememizin sebebi bu. (Unpin olan türler bu kısıttan muaf.)
//
// • Neden RUNTIME? std Future'ı ve async/await'i TANIMLAR ama koşturan döngüyü vermez; o döngü
//   I/O, zamanlayıcı, görev kuyruğu ister — tek "doğru" seçim yok, bu yüzden std dışına
//   (tokio/async-std) bırakılmış. Biz ÇEKİRDEĞİ (block_on + cok_gorev) yazdık; gerçek runtime aynı
//   prensibi binlerce görev + epoll/iocp ile ölçekler.
//
// • EŞZAMANLILIK vs PARALELLİK: cok_gorev tek thread'de İÇ İÇE yürüttü (çıktıda iki "başladı",
//   iki "bitti"den önce gelir) — bu concurrency. Paralellik (Böl. 16) çok çekirdekte koşturur;
//   async runtime'lar ikisini birleştirir (çok thread + her thread'de çok görev).
