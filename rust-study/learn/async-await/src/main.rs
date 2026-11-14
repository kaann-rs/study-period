// Rust Book — Bölüm 17: Async/Await (2024 edition'da kitaba eklendi)
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

use std::future::Future;
use std::pin::{Pin, pin};
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use std::thread::{self, Thread};

fn main() {
    // (1) Tek future'ı sonuna kadar koştur:
    let s = block_on(akis());
    println!("block_on döndü: {s}");

    // (2) Aynı thread'de İKİ görevi İÇ İÇE (eşzamanlı) koştur:
    println!("-- iki görev eşzamanlı --");
    cok_gorev(vec![Box::pin(gorev(1)), Box::pin(gorev(2))]);
}

// ============================================================================
// ELLE MİNİMAL EXECUTOR (std'nin verdikleriyle)
// ============================================================================

// Waker: future "artık ilerleyebilirim" dediğinde executor'u uyandıran geri-çağrı. std::task::Wake
// trait'i, ham RawWaker/vtable yazmadan Arc'tan Waker üretmemizi sağlar. Burada "uyandır" =
// bekleyen (park etmiş) thread'i çöz (unpark).
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
// İki AYRI iş yapan iki keyword; karıştırma:
//
// • `async` (fn ya da blok önünde): gövdeyi ÇALIŞTIRMAZ. Onu, gövdeyi uygulayan bir FUTURE
//   değerine çevirir (derleyici arka planda bir state machine üretir). `async fn f() -> T`'nin
//   gerçek dönüş tipi `impl Future<Output = T>`'dir. Yani `f()` çağırmak SIFIR iş yapar — sadece
//   "henüz başlamamış hesap"ı (future'ı) üretir; iş, biri onu poll edene dek BAŞLAMAZ (tembel).
//   Fark: normal fonksiyon çağrılınca HEMEN koşar; async fonksiyon çağrılınca koşmaz, Future verir.
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
//   Yani `.await` bir BUSY-LOOP DEĞİL: Pending'de geri döner (yield), executor başka iş yapar,
//   future hazır olunca (waker.wake()) yeniden poll edilir ve makine bu noktadan devam eder.
//   `.await`'in değeri = future'ın Output'u (Ready'den çıkan değer).
//
// • Neden `.await` yalnız `async` içinde kullanılabilir? Çünkü "Pending'de durumu saklayıp ÜST'e
//   dön, sonra buradan devam et" davranışı ancak bir state machine'de mümkün. Normal fonksiyonda
//   "geri dönüp sonra kaldığın yerden devam etme" diye bir şey yok → `.await` orada derleme hatası.
//   (Bu yüzden düz `main` async olamaz; bir runtime'a ihtiyaç duyar — biz block_on ile köprü kurduk.)
//
// Aşağıdaki `akis`, tam da böyle bir state machine'e derlenir: iki await noktası (BirKezBekle ve
// selam) = makinenin iki duraklama DURUMU. Yukarıda ELLE yazdığımız BirKezBekle Future'ı ise,
// derleyicinin bu makineyi (poll → Pending/Ready → wake) NASIL kurduğunun küçük, elle yapılmış hâli.
// ============================================================================
// async fn'ler — derleyici bunları birer state machine'e çevirir
// ============================================================================

async fn selam(ad: &str) -> String {
    format!("Selam {ad}")
}

async fn akis() -> i32 {
    // .await: future'ı buraya "yerleştir"; hazır olana dek bu async fonksiyon ASILI kalır,
    // kontrol executor'a döner. Hazır olunca kaldığı yerden devam eder.
    let n = BirKezBekle { hazir: false }.await; // 42 (bir tur Pending'den sonra)
    let s = selam("Kaan").await; // iç içe await
    println!("  akis içinden: {s}");
    n + 1
}

async fn gorev(id: u32) {
    println!("  görev {id}: başladı");
    Teslim(false).await; // bu noktada kontrolü diğer göreve bırak
    println!("  görev {id}: bitti");
}

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
// • Neden RUNTIME gerekir? std, Future'ı ve async/await'i TANIMLAR ama onu KOŞTURAN döngüyü
//   (poll et, Pending'de uyu, wake'te devam et, çok görevi zamanla) vermez. Sorun: bu döngü
//   I/O entegrasyonu, zamanlayıcı, görev kuyruğu ister — tek bir "doğru" seçim yok, bu yüzden
//   std dışına (tokio/async-std) bırakılmış. Biz burada o döngünün ÇEKİRDEĞİNİ (block_on +
//   cok_gorev) yazdık: gerçek bir runtime aynı prensibi binlerce görev + epoll/iocp ile ölçekler.
//
// • EŞZAMANLILIK vs PARALELLİK: cok_gorev tek thread'de iki görevi İÇ İÇE yürüttü (biri await'te
//   beklerken öteki ilerledi) — bu concurrency. Çıktıda iki "başladı" iki "bitti"den ÖNCE gelir.
//   Paralellik (Böl. 16) ise işi gerçekten aynı anda çok çekirdekte koşturmaktır; async runtime'lar
//   ikisini birleştirir (çok thread'li executor + her thread'de çok async görev).
