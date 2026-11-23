# 16. Async ve Await

Programlar çoğu zaman hesaplamaz, bekler: ağdan yanıt, diskten veri, zamanlayıcının dolması. Bekleyen her iş için ayrı bir thread açmak mümkündür, ama her thread kendi stack'ini taşır ve işletim sistemine yük getirir. On binlerce bağlantının aynı anda beklediği bir sunucuda bu yük ağırlaşır. Üstelik bekleyen thread'ler iş yapmaz, yalnızca yer kaplar.

Async programlama, beklemeyi başka bir işin ilerlediği boşluğa dönüştürür. Bir iş beklemeye girdiğinde sıra ötekine geçer; sonuç hazır olunca iş kaldığı yerden sürer. Böylece tek bir thread çok sayıda işi iç içe yürütür.[^eszamanlilik] Bu *eşzamanlılıktır*, paralellik değildir: İşler aynı anda değil, birbirinin bekleme boşluklarında ilerler. Async işler birden çok thread'e dağıtılarak ikisi birleştirilebilir.

Rust bu işin yalnızca çekirdeğini dile ve standart kütüphaneye koyar: `async` ile `await` söz dizimi ve `Future` trait'i. Bu işleri yürüten parça, yani *yürütücü* (executor), standart kütüphanede yoktur; gerçek projelerde tokio gibi bir çalışma zamanı kütüphanesi kullanılır. Bu bölümde yürütücüyü yalnızca standart kütüphaneyle elle yazacağız. Amaç, gündelik kodda gizli kalan mekanizmayı görünür kılmaktır.

> [!NOTE]
> Bu bölümün notları ve çalıştırılabilir kodu `learn/async-await` dizinindedir. Depo kökünden `cargo run -p async-await` komutuyla çalıştırabilirsiniz. Örneklerde `use std::future::Future;` satırı yoktur, çünkü edition 2024'te `Future` ve `IntoFuture` prelude'dadır.[^prelude]

## Günlük hayattan bir benzetme

Evde tek başınıza ev işi yapıyorsunuz. Çamaşırı makineye koydunuz; başında oturup beklemezsiniz, o sırada bulaşıkları yıkarsınız. Makine bitince zil çalar, gidip çamaşırı alırsınız. Tek kişisiniz, yine de iki iş iç içe ilerler. İki makineniz varsa ikisini birden çalıştırırsınız: Geçen zaman iki sürenin toplamı değil, uzun süren işin süresi kadar olur.

Her işi bir *iş kartı* olarak düşünün. Kartı yazmak işi başlatmaz; biri kartı eline alıp takip etmelidir. Programda kart bir `Future`, takip etmek `.await`, zil *uyandırıcı* (waker), kartları çeviren kişi ise *yürütücüdür*. Kartları çeviren kimse yoksa hiçbir iş ilerlemez.

<details>
<summary>Bu bölümde geçen terimler</summary>

- **`async`:** "Bu işi şimdi yapma, kartını hazırla" diyen anahtar sözcük.
- **`Future`:** Sonucu daha sonra hazır olacak iş; iş kartı. Biri yoklamadıkça ilerlemez.
- **`.await`:** "Bu kartı şimdi takip et; beklemek gerekirse sırayı başkasına ver."
- **tembel (lazy):** İsteyen olmadıkça çalışmayan. Future'lar tembeldir.
- **yoklamak (poll):** Karta bakıp işi ilerleyebildiği kadar ilerletmek.
- **`Poll::Ready`, `Poll::Pending`:** Yoklamanın iki yanıtı: "bitti, sonuç bu" ve "daha değil".
- **yürütücü (executor):** Kartları yoklayan, bekleyen ve zil çalınca dönen parça.
- **çalışma zamanı (runtime):** Yürütücüyü zamanlayıcı, ağ ve görev kuyruğuyla birlikte sunan kütüphane.
- **uyandırıcı (waker), `wake`:** İş ilerleyebilecek duruma gelince çalan zil ve zili çalmak.
- **`Context` (`cx`):** `poll`'a verilen küçük çanta; içinde o işin uyandırıcısı durur.
- **`park`, `unpark`:** Thread'in uyuması ve uyuyan thread'in dürtülmesi.
- **geri çağrı (callback):** "Şu olunca bunu çalıştır" diye önceden verilen iş.
- **`Pin`, `pin!`, `Box::pin`:** Bir değeri bellekte yerinden oynamayacak biçimde sabitleyen araçlar.
- **durum makinesi (state machine):** "Hangi adımda kaldım, elimde ne var?" bilgisini saklayan yapı.
- **görev (task):** Yürütücünün bağımsız olarak sürdüğü üst düzey future.
- **eşzamanlılık / paralellik:** İşlerin aynı dönemde iç içe ilerlemesi / gerçekten aynı anda yapılması.
- **G/Ç (I/O):** Dosya ve ağ gibi dış dünyadan veri beklenen işler.
- **dönüşümlü sıra (round-robin):** Herkese sırayla bir tur vermek.
- **iç içe ilerleme (interleaving):** İki işin adımlarının birbirinin arasına girmesi.
- **meşgul bekleme (busy-wait):** Durmadan "oldu mu?" diye sorup işlemciyi boşa yormak.
- **sırayı bırakmak (yield):** "Bir tur da başkası ilerlesin" demek.
- **açılım (desugaring):** Kısa bir yazımın derleyicinin gördüğü uzun biçimi.
- **vtable, `RawWaker`:** Zil çalınca hangi fonksiyonların çağrılacağının listesi ve zilin ham hâli.
- **kendine referanslı (self-referential):** Kendi içindeki bir parçayı gösteren değer.
- **`Unpin`:** "Beni taşımak sorun çıkarmaz" etiketini taşıyan türlerin trait'i.

</details>

## Thread mi, async mi?

İkisi de eşzamanlılık sağlar, ama farklı işlere uygundur:[^hangisi]

| | Thread | Async |
| --- | --- | --- |
| Kim sıraya sokar? | İşletim sistemi | Yürütücü (kütüphane) |
| Sıra ne zaman değişir? | Herhangi bir anda | Yalnızca `.await` noktalarında |
| Bekleyen işin bedeli | Bir thread ve stack'i | Durum makinesi kadar bellek |
| Uygun olduğu iş | Bölünebilen, hesap yoğun (CPU-bound) iş | Çok sayıda bekleyen, G/Ç yoğun (I/O-bound) iş |

Async hesaplamayı hızlandırmaz; yalnızca bekleme sürelerinin boşa gitmesini önler. Bir görüntüyü işlemek için thread'ler, bin bağlantıya yanıt vermek için async uygundur. İkisi birlikte de kullanılır: Yaygın çalışma zamanları birkaç thread açar ve her birinde çok sayıda görev yürütür.

## `async`: İşin kartını hazırlamak

Bir fonksiyonun başına `async` yazmak gövdesini çalıştırmaz, onu bir `Future` değerine çevirir. `async fn cay() -> u32` yazdığınızda çağrının türü `u32` değil, "`u32` üretecek bir future"dır, yani `impl Future<Output = u32>`.[^async-fn] Çağrı sırasında argümanlar değerlendirilir, ama gövdenin tek satırı bile çalışmaz:

```rust
# use std::pin::pin;
# use std::sync::Arc;
# use std::task::{Context, Poll, Wake, Waker};
# use std::thread::{self, Thread};
#
# struct ThreadWaker(Thread);
# impl Wake for ThreadWaker {
#     fn wake(self: Arc<Self>) {
#         self.0.unpark();
#     }
#     fn wake_by_ref(self: &Arc<Self>) {
#         self.0.unpark();
#     }
# }
# fn block_on<F: Future>(fut: F) -> F::Output {
#     let mut fut = pin!(fut);
#     let waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
#     let mut cx = Context::from_waker(&waker);
#     loop {
#         match fut.as_mut().poll(&mut cx) {
#             Poll::Ready(v) => return v,
#             Poll::Pending => thread::park(),
#         }
#     }
# }
async fn cay() -> u32 {
    println!("çay demleniyor");
    1
}

fn main() {
    let kart = cay(); // kart hazır, gövde çalışmadı
    println!("kart hazır");

    let n = block_on(kart); // biri takip etti, gövde çalıştı
    println!("n = {n}");
}
```

Çıktı:

```text
kart hazır
çay demleniyor
n = 1
```

Buradaki `block_on`, bir future'ı sonuna kadar süren küçük yürütücümüzdür; birazdan yazacağız. Future'lar *tembeldir* (lazy): [Iterator'lar](12-closures-iterators.md) gibi, sonucu isteyen olmadıkça çalışmaz.[^tembel]

`async` bir bloğun önüne de yazılabilir. Blok da sonucun kendisi değil, bir karttır. Kartı sonuç yerine koymaya çalışırsanız program derlenmez:

```rust,compile_fail,E0308
fn main() {
    let n: i32 = async { 2 };
}
```

Derleyicinin yanıtı:

```text
error[E0308]: mismatched types
  |
2 |     let n: i32 = async { 2 };
  |            ---   ^^^^^^^^^^^ expected `i32`, found `async` block
  |            |
  |            expected due to this
```

## `.await`: Kartı takip etmek

`.await`, bir future'ı içinde bulunulan async gövdeden sürer. "Bu kartı şimdi takip et; iş beklemeye girerse sıra başkasına geçsin, hazır olunca tam bu satırdan devam et" demektir. Sonucu, future'ın ürettiği değerdir:

```rust
# use std::pin::pin;
# use std::sync::Arc;
# use std::task::{Context, Poll, Wake, Waker};
# use std::thread::{self, Thread};
#
# struct ThreadWaker(Thread);
# impl Wake for ThreadWaker {
#     fn wake(self: Arc<Self>) {
#         self.0.unpark();
#     }
#     fn wake_by_ref(self: &Arc<Self>) {
#         self.0.unpark();
#     }
# }
# fn block_on<F: Future>(fut: F) -> F::Output {
#     let mut fut = pin!(fut);
#     let waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
#     let mut cx = Context::from_waker(&waker);
#     loop {
#         match fut.as_mut().poll(&mut cx) {
#             Poll::Ready(v) => return v,
#             Poll::Pending => thread::park(),
#         }
#     }
# }
async fn cay() -> u32 {
    1
}

async fn mutfak() -> u32 {
    let kart = cay(); // cay'ın gövdesi çalışmadı
    let n = kart.await; // şimdi çalıştı
    n + 10
}

fn main() {
    println!("{}", block_on(mutfak()));
}
```

Çıktı:

```text
11
```

Her `.await` bir duraktır. İki durak arasındaki kod, o görevin içinde hiç bölünmeden çalışır; sıra yalnızca duraklarda el değiştirebilir.[^duraklar] `.await` yapılan şey bir future olmalıdır; değilse E0277 alınır:

```rust,compile_fail,E0277
async fn mutfak() -> i32 {
    let n = 5;
    n.await
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0277]: `{integer}` is not a future
  |
3 |     n.await
  |       ^^^^^ `{integer}` is not a future
```

### Neden yalnızca `async` içinde?

"Beklemeye girince durumu sakla, sonra buradan devam et" işi, yalnızca durum makinesine dönüşen bir gövdede yapılabilir. Sıradan bir fonksiyon durdurulup sürdürülemez; bu yüzden içinde `.await` yazılamaz:[^await-ref]

```rust,compile_fail,E0728
async fn cay() -> u32 {
    1
}

fn duz() {
    let n = cay().await;
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0728]: `await` is only allowed inside `async` functions and blocks
  |
5 | fn duz() {
  | -------- this is not `async`
6 |     let n = cay().await;
  |                   ^^^^^ only allowed inside `async` functions and blocks
```

`main` de async olamaz: Giriş noktasından dönecek future'ı yoklayacak bir yürütücü yoktur.

```rust,compile_fail,E0752
async fn main() {
    println!("merhaba");
}
```

Derleyicinin yanıtı:

```text
error[E0752]: `main` function is not allowed to be `async`
```

Sıradan koddan async koda geçiş, yürütücünün `block_on` fonksiyonuyla yapılır. tokio'daki `#[tokio::main]` özniteliği de bu köprüyü sizin yerinize yazar.[^tokio-main]

## `Future` trait'i: Yoklamak

Bir future, üzerinde tek soru yazan bir karttır: "İlerleyebildiğin kadar ilerle, sonra durumunu söyle." Trait de bundan ibarettir:[^future-trait]

```rust
# use std::pin::Pin;
# use std::task::{Context, Poll};
pub trait Future {
    type Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}
```

`poll` iki yanıttan birini verir: `Poll::Ready(deger)` "bitti", `Poll::Pending` "daha değil" demektir. `cx` içinde uyandırıcı durur; `Pin` ise değerin bellekte yerinden oynatılmayacağını söyler. İkisini sırayla açacağız.

Gerçek hayatta "daha değil" yanıtı çoğunlukla ağı ya da diski beklemek demektir. Burada ağ yerine, ilk yoklamada `Pending`, ikincisinde 42 veren bir future yazalım:

```rust
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

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
            cx.waker().wake_by_ref(); // "beni yeniden yokla"
            Poll::Pending
        }
    }
}

fn main() {
    let mut cx = Context::from_waker(Waker::noop()); // çalınca bir şey yapmayan zil
    let mut kart = BirKezBekle { hazir: false };

    println!("1. yoklama: {:?}", Pin::new(&mut kart).poll(&mut cx));
    println!("2. yoklama: {:?}", Pin::new(&mut kart).poll(&mut cx));
}
```

Çıktı:

```text
1. yoklama: Pending
2. yoklama: Ready(42)
```

Burada yoklamayı elle yaptık; normalde bu iş yürütücünündür. Sözleşmenin iki yarısı vardır.[^sozlesme] Future `Pending` dönüyorsa uyandırıcıyı saklamalı ve ilerleyebilecek duruma gelince çalmalıdır; bizimki zili hemen çalıyor. Yürütücü de zil çalınca yeniden yoklamalıdır. Zili çalmayan bir future'a bir daha kimse bakmayabilir.

## Uyandırıcı: Zil

Çamaşır makinesinin başında durup "bitti mi?" diye durmadan bakmazsınız; makine bitince zil çalar. Zil olmasaydı ya durmadan bakmak (meşgul bekleme, CPU'yu boşa yormak) ya da işin bittiğini hiç öğrenememek gerekirdi. *Uyandırıcı* (waker) bu zildir: "İlerleyebilirim, beni yeniden yokla" demenin yoludur.

Zil çalınca ne olacağını yürütücü belirler. Bizim yürütücümüz, yapacak iş kalmayınca thread'i uyutacak; o yüzden zil çalmak uyuyan thread'i dürtmek demektir. Standart kütüphanenin `Wake` trait'i, bir `Arc`'tan güvenli biçimde `Waker` üretmemizi sağlar:[^wake]

```rust
use std::sync::Arc;
use std::task::{Wake, Waker};
use std::thread::{self, Thread};

struct ThreadWaker(Thread);

impl Wake for ThreadWaker {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}

fn main() {
    let zil = Waker::from(Arc::new(ThreadWaker(thread::current())));

    zil.wake_by_ref(); // zil çaldı
    thread::park(); // zil az önce çaldığı için uyuyup kalmaz
    println!("kalktık");
}
```

Çıktı:

```text
kalktık
```

`park` thread'i uyutur, `unpark` uyandırır. `unpark`, thread henüz uyumamışken çağrılırsa bir işaret bırakır ve sonraki `park` hemen döner; böylece zil uykudan önce çalsa da kaybolmaz.[^park]

`wake_by_ref` yazmak zorunlu değildir; varsayılan gövdesi `Arc`'ı klonlayıp `wake`'i çağırır. Burada gereksiz klonu önlemek için yazılmıştır.

## Bir işi sonuna kadar sürmek: `block_on`

Evdeki kişi karta gider, bakar. Yanıt `Ready` ise sonucu alır. `Pending` ise koltuğa oturup uyur; zil çalınca kalkar, yeniden bakar. `block_on` bu döngüdür:

```rust
use std::pin::{Pin, pin};
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use std::thread::{self, Thread};

# struct ThreadWaker(Thread);
# impl Wake for ThreadWaker {
#     fn wake(self: Arc<Self>) {
#         self.0.unpark();
#     }
# }
# struct BirKezBekle {
#     hazir: bool,
# }
# impl Future for BirKezBekle {
#     type Output = i32;
#     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<i32> {
#         if self.hazir {
#             return Poll::Ready(42);
#         }
#         self.hazir = true;
#         cx.waker().wake_by_ref();
#         Poll::Pending
#     }
# }
fn block_on<F: Future>(fut: F) -> F::Output {
    let mut fut = pin!(fut); // yoklamak için sabitle
    let waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
    let mut cx = Context::from_waker(&waker);

    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(v) => return v,
            Poll::Pending => thread::park(), // uyandırılana dek uyu
        }
    }
}

async fn selam(ad: &str) -> String {
    format!("Selam {ad}")
}

async fn akis() -> i32 {
    let n = BirKezBekle { hazir: false }.await;
    let s = selam("Kaan").await;
    println!("akis içinden: {s}");
    n + 1
}

fn main() {
    let s = block_on(akis());
    println!("block_on döndü: {s}");
}
```

Çıktı:

```text
akis içinden: Selam Kaan
block_on döndü: 43
```

`akis` iki yerde bekleyebilir. Olanlar sırasıyla şöyledir:

1. İlk yoklamada `BirKezBekle` "daha değil" der ve zili çalar. `akis` kaldığı yeri saklar, o da `Pending` döner.
2. Zil çalmış olduğu için `park` hemen döner; `block_on` yeniden yoklar. `akis` kaldığı satırdan sürer: `BirKezBekle` 42 verir, `selam` hiç beklemeden biter, `akis` 43 ile tamamlanır.

İkinci `.await`'te sıra hiç bırakılmadı. `.await` mutlaka beklemek demek değildir; beklenen future hazırsa kod duraksamadan sürer.[^duraklar]

### Zili çalmayan future

Sözleşmenin ilk yarısı çiğnenirse derleyici bunu göremez. Aşağıdaki program derlenir, ama asılı kalır:

```rust,no_run
# use std::pin::pin;
# use std::sync::Arc;
# use std::task::{Context, Poll, Wake, Waker};
# use std::thread::{self, Thread};
#
# struct ThreadWaker(Thread);
# impl Wake for ThreadWaker {
#     fn wake(self: Arc<Self>) {
#         self.0.unpark();
#     }
#     fn wake_by_ref(self: &Arc<Self>) {
#         self.0.unpark();
#     }
# }
# fn block_on<F: Future>(fut: F) -> F::Output {
#     let mut fut = pin!(fut);
#     let waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
#     let mut cx = Context::from_waker(&waker);
#     loop {
#         match fut.as_mut().poll(&mut cx) {
#             Poll::Ready(v) => return v,
#             Poll::Pending => thread::park(),
#         }
#     }
# }
# use std::pin::Pin;
struct ZiliUnutan {
    hazir: bool,
}

impl Future for ZiliUnutan {
    type Output = i32;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<i32> {
        if self.hazir {
            return Poll::Ready(42);
        }
        self.hazir = true;
        Poll::Pending // kimse zili çalmayacak
    }
}

fn main() {
    println!("{}", block_on(ZiliUnutan { hazir: false }));
}
```

Sonraki yoklama 42 verecekti, ama o yoklama yapılmaz: `block_on` uyur ve onu uyandıracak kimse yoktur.[^sozlesme]

### `park` işareti ve sağlam bir uyandırıcı

> [!WARNING]
> Buradaki `block_on` öğretmek içindir. Standart kütüphanenin belgesi de aynı örnek için, basitlik uğruna doğruluktan ödün verildiğini söyler.[^wake] Sorun şudur: `unpark`'ın bıraktığı işaret thread'e aittir, göreve değil. Yoklama sırasında başka bir kod `park` çağırıp işareti tüketirse zil kaybolur ve `block_on` sonsuza dek uyuyabilir.

Çare, "zil çaldı" bilgisini yürütücüye ait bir bayrakta tutmaktır. `Yutan` adlı future, yoklama sırasında `park` çağırarak sorunu bilerek yaratır; bayraklı sürüm yine de doğru çalışır:

```rust
use std::pin::{Pin, pin};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll, Wake, Waker};
use std::thread::{self, Thread};

struct BayrakliWaker {
    thread: Thread,
    uyandi: AtomicBool,
}

impl Wake for BayrakliWaker {
    fn wake(self: Arc<Self>) {
        self.uyandi.store(true, Ordering::Release);
        self.thread.unpark();
    }
}

fn block_on<F: Future>(fut: F) -> F::Output {
    let mut fut = pin!(fut);
    let zil = Arc::new(BayrakliWaker {
        thread: thread::current(),
        uyandi: AtomicBool::new(false),
    });
    let waker = Waker::from(Arc::clone(&zil));
    let mut cx = Context::from_waker(&waker);

    loop {
        if let Poll::Ready(v) = fut.as_mut().poll(&mut cx) {
            return v;
        }
        while !zil.uyandi.swap(false, Ordering::Acquire) {
            thread::park(); // sahte uyanma da olabilir; bayrağa bak
        }
    }
}

struct Yutan(bool);

impl Future for Yutan {
    type Output = u32;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<u32> {
        if self.0 {
            return Poll::Ready(7);
        }
        self.0 = true;
        cx.waker().wake_by_ref();
        thread::park(); // işareti yutar
        Poll::Pending
    }
}

fn main() {
    println!("{}", block_on(Yutan(false)));
}
```

Çıktı:

```text
7
```

`park`'ın nedensiz yere de dönebileceği belgelidir; döngüyle bayrağa bakmak bunu da karşılar.[^park] Sayfanın geri kalanında kısa olan ilk sürümü kullanacağız.

## Tek thread'de birden çok görev: `cok_gorev`

Çamaşır makinesi dönerken bulaşığa geçersiniz; bulaşık suda beklerken dönüp çamaşıra bakarsınız. `cok_gorev` da listedeki her işe sırayla bir kez bakar (round-robin), biteni listeden çıkarır, kalanlara sonraki turda yeniden bakar. Sırayı bırakan bir future'a da gerek var; `Teslim` ilk yoklamada "bir tur da öteki iş ilerlesin" der, ikincisinde biter:

```rust
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use std::thread::{self, Thread};

# struct ThreadWaker(Thread);
# impl Wake for ThreadWaker {
#     fn wake(self: Arc<Self>) {
#         self.0.unpark();
#     }
# }
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

fn cok_gorev(mut gorevler: Vec<Pin<Box<dyn Future<Output = ()>>>>) {
    let waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
    let mut cx = Context::from_waker(&waker);

    while !gorevler.is_empty() {
        // Pending olanları tut, bitenleri çıkar
        gorevler.retain_mut(|g| g.as_mut().poll(&mut cx).is_pending());
        if !gorevler.is_empty() {
            thread::park();
        }
    }
}

async fn gorev(id: u32) {
    println!("görev {id}: başladı");
    Teslim(false).await; // sırayı öteki göreve bırak
    println!("görev {id}: bitti");
}

fn main() {
    println!("-- iki görev eşzamanlı --");
    cok_gorev(vec![Box::pin(gorev(1)), Box::pin(gorev(2))]);
}
```

Çıktı:

```text
-- iki görev eşzamanlı --
görev 1: başladı
görev 2: başladı
görev 1: bitti
görev 2: bitti
```

Görevler tek tek sürülseydi sıra "1 başladı, 1 bitti, 2 başladı, 2 bitti" olurdu. Burada ikinci görev, birincisi bitmeden başladı. Tek thread vardır; kazanç paralellikten değil, bekleme boşluklarının dolmasından gelir.[^yield]

Listenin türü `Vec<Pin<Box<dyn Future<Output = ()>>>>`'dir. Her async fonksiyonun ürettiği future farklı ve adsız bir türdür; hepsini tek listede tutmak için `dyn Future` olarak kutulanır. Boyutu bilinmeyen değer `Box` ister, yoklanacak değer sabitlenmek ister; `Box::pin` ikisini birden yapar.

> [!NOTE]
> `cok_gorev` her uyanmada bütün görevleri yoklar, çünkü hepsine aynı zili verir. Gerçek bir çalışma zamanı her göreve ayrı bir uyandırıcı verir, yalnızca uyandırılan görevi kuyruğa alır ve yoklar. Böyle bir yürütücüyü [29.2'de](../ileri/29-calisma-zamani.md) kuracağız.

## Çalışma zamanı neden standart kütüphanede yok?

Standart kütüphane `Future`'ı ve `async`/`await`'i tanımlar, ama future'ları süren döngüyü vermez.[^runtime] Böyle bir döngü G/Ç, zamanlayıcı ve görev kuyruğu ister; bunların tek doğru biçimi yoktur. Bir web sunucusu çok thread'li bir kuyruk, bir mikrodenetleyici ise thread'siz ve heap'siz bir döngü isteyebilir. Bu yüzden seçim kütüphanelere bırakılmıştır.

Bizim yazdığımız `block_on` ile `cok_gorev` işin çekirdeğidir. Gerçek çalışma zamanları aynı ilkeyi binlerce görevle ölçekler ve zili, işletim sisteminin "şu soket okunabilir oldu" bildirimlerine (`epoll`, `kqueue`, `IOCP`) bağlar. En yaygını tokio'dur.[^runtime][^tokio] Eski kaynaklarda adı geçen async-std artık geliştirilmemektedir; yerine smol önerilir.[^async-std]

## Hata kodu başvurusu

| Kod | Ne zaman görülür? | Ne yapmalı? |
| --- | --- | --- |
| `E0728` | `.await`, async olmayan bir fonksiyonda ya da closure'da kullanılır. | Fonksiyonu `async` yapın ya da `block_on` ile çalıştırın. |
| `E0752` | `main` async yazılır. | Sıradan `main` içinde bir yürütücü çağırın (tokio'da `#[tokio::main]`). |
| `E0308` | Değer beklenen yere future konur. | `.await` ekleyin ya da `block_on` ile sonucu alın. |
| `E0277` | `.await` future olmayan bir değere uygulanır ya da `Unpin` olmayan bir future `Pin::new` ile sabitlenir. | `.await`'i kaldırın; sabitlemek için `pin!` ya da `Box::pin` kullanın. |

Bir hatanın uzun açıklamasını terminalde görebilirsiniz:

```console
$ rustc --explain E0728
```

## Perde arkası: Derleyici ne yapıyor?

> [!NOTE]
> Bu başlık ilk okuyuşta atlanabilir. Gündelik kodda `Pin`'i çoğu zaman hiç görmezsiniz; `.await` yazarken gereken sabitlemeyi derleyici yapar.

### Async fonksiyon bir durum makinesine dönüşür

Derleyici async gövdeyi, her `.await` noktasında duraklayıp sonra sürebilen bir *durum makinesine* çevirir. Makine iki şeyi hatırlar: hangi durakta kaldığını ve o durakta gerekli olan yerel değişkenleri.[^durum-rfc] Yaklaşık olarak bir enum gibi düşünülebilir. `BirKezBekle`'yi bekleyip sonucuna bir ekleyen `async { BirKezBekle { hazir: false }.await + 1 }` bloğunu elle yazsaydık şöyle olurdu:

```rust
# use std::pin::Pin;
# use std::task::{Context, Poll, Waker};
# struct BirKezBekle {
#     hazir: bool,
# }
# impl Future for BirKezBekle {
#     type Output = i32;
#     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<i32> {
#         if self.hazir {
#             return Poll::Ready(42);
#         }
#         self.hazir = true;
#         cx.waker().wake_by_ref();
#         Poll::Pending
#     }
# }
enum Akis {
    Baslamadi,
    Bekliyor(BirKezBekle),
    Bitti,
}

impl Future for Akis {
    type Output = i32;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<i32> {
        loop {
            match &mut *self {
                Akis::Baslamadi => *self = Akis::Bekliyor(BirKezBekle { hazir: false }),
                Akis::Bekliyor(alt) => match Pin::new(alt).poll(cx) {
                    Poll::Ready(n) => {
                        *self = Akis::Bitti;
                        return Poll::Ready(n + 1);
                    }
                    Poll::Pending => return Poll::Pending, // durum saklandı
                },
                Akis::Bitti => panic!("biten future yeniden yoklandı"),
            }
        }
    }
}

fn main() {
    let mut cx = Context::from_waker(Waker::noop());
    let mut akis = Akis::Baslamadi;

    println!("{:?}", Pin::new(&mut akis).poll(&mut cx));
    println!("{:?}", Pin::new(&mut akis).poll(&mut cx));
}
```

Çıktı:

```text
Pending
Ready(43)
```

Enum yalnızca bir benzetmedir; gerçek yerleşim belirtilmemiştir. rustc 1.99.0'da yapı; yakalanan değerlerden, bir durum numarasından ve duraklar arasında yaşayan yerellerden oluşur.[^durum-kod] Üç durum numarası sabittir: henüz başlamadı, tamamlandı, panikledi. Her `.await` bunlara bir durak ekler; iki `.await`'i olan `akis`'in iki durağı vardır. Biten bir async gövdeyi yeniden yoklamak paniktir ("`async fn` resumed after completion").

Yalnızca durak boyunca yaşayan değişkenler saklandığından, büyük bir değeri `.await`'ten önce bitirmek future'ı küçültür:

```rust
# use std::pin::Pin;
# use std::task::{Context, Poll};
# struct Teslim(bool);
# impl Future for Teslim {
#     type Output = ();
#     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
#         if self.0 {
#             return Poll::Ready(());
#         }
#         self.0 = true;
#         cx.waker().wake_by_ref();
#         Poll::Pending
#     }
# }
async fn tasiyan() -> u8 {
    let tampon = [1u8; 1024];
    Teslim(false).await;
    tampon[0] // tampon durak boyunca yaşadı
}

async fn tasimayan() -> u8 {
    let tampon = [1u8; 1024];
    let ilk = tampon[0]; // tampon duraktan önce bitti
    Teslim(false).await;
    ilk
}

fn main() {
    println!("taşıyan:   {} bayt", size_of_val(&tasiyan()));
    println!("taşımayan: {} bayt", size_of_val(&tasimayan()));
}
```

Örnek bir çalıştırma (rustc 1.99.0; sayılar derleyici sürümüyle değişebilir):

```text
taşıyan:   1026 bayt
taşımayan: 3 bayt
```

### `.await` neye açılır?

`let v = fut.await;` satırının yaklaşık açılımı şöyledir:[^await-acilim]

```text
let mut fut = IntoFuture::into_future(fut);   // durum makinesinin alanında saklanır
loop {
    match Pin(&mut fut).poll(cx) {
        Poll::Ready(v)  => break v,           // değeri al, gövdeye devam et
        Poll::Pending   => duraklat,          // durumu sakla, çağırana Pending dön
    }
}
```

İlk adımda değer `IntoFuture` ile future'a çevrilir; bu yüzden future olmayan ama `IntoFuture` uygulayan türler de beklenebilir. `duraklat` bir meşgul bekleme değildir: Denetim yürütücüye döner ve görev yeniden yoklandığında döngü baştan değil, tam bu noktadan sürer. Derleyicide bu açılım, söz dizimi ağacını HIR adlı daha alt bir gösterime indirgeyen aşamada, `make_lowered_await` fonksiyonunda yapılır; async olmayan gövdedeki `.await`'i E0728 ile reddeden denetim de aynı yerdedir.[^await-kod] E0752 ise giriş noktası denetiminden gelir.[^e0752-kod]

### `Pin` neden var?

Durum makinesi, bir duraktan sonra kendi yerel değişkenine referans tutabilir:

```rust
# use std::pin::{Pin, pin};
# use std::task::{Context, Poll, Waker};
# struct Teslim(bool);
# impl Future for Teslim {
#     type Output = ();
#     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
#         if self.0 {
#             return Poll::Ready(());
#         }
#         self.0 = true;
#         cx.waker().wake_by_ref();
#         Poll::Pending
#     }
# }
async fn defter() -> i32 {
    let sayfalar = [10, 20, 30];
    let not = &sayfalar[2]; // makinenin kendi içini gösterir
    Teslim(false).await;
    *not
}

fn main() {
    let mut cx = Context::from_waker(Waker::noop());
    let mut fut = pin!(defter());

    println!("{:?}", fut.as_mut().poll(&mut cx));
    println!("{:?}", fut.as_mut().poll(&mut cx));
}
```

Çıktı:

```text
Pending
Ready(30)
```

`sayfalar` ile onu gösteren `not` aynı makinenin alanlarıdır; değer *kendine referanslıdır* (self-referential). İki yoklama arasında makine bellekte başka yere taşınsaydı `not` eski adresi gösterirdi. Defterin bir sayfasında "üç sayfa ilerideki listeye bak" gibi, yeri adresle gösteren bir not olduğunu düşünün: Defter başka rafa taşınırsa not yanlış yeri gösterir.

`Pin<&mut T>`, gösterdiği değerin artık taşınmayacağı sözünü tür düzeyinde taşır; `poll` bu yüzden `Pin` ister.[^pin] Taşınması sorun olmayan türler `Unpin`'dir ve bu kısıttan muaftır; `BirKezBekle` böyledir, o yüzden `Pin::new` ile yoklayabildik. Async fonksiyonların ürettiği future'lar `Unpin` değildir:

```rust,compile_fail,E0277
use std::pin::Pin;
use std::task::{Context, Waker};

async fn cay() -> u32 {
    1
}

fn main() {
    let mut cx = Context::from_waker(Waker::noop());
    let mut fut = cay();
    let yanit = Pin::new(&mut fut).poll(&mut cx);
}
```

Derleyicinin yanıtı:

```text
error[E0277]: `{async fn body of cay()}` cannot be unpinned
     |
     = note: consider using the `pin!` macro
             consider using `Box::pin` if you need to access the pinned value outside of the current scope
```

`pin!` ile `Box::pin` arasındaki seçim [derinleşme sayfasında](16-async-await-derinlesme.md), kendine referanslı yapıların iç yüzü [28. bölümde](../ileri/28-future.md) işlenir.

### `Wake` ve `RawWaker`

`Waker` içeride bir `RawWaker` taşır: bir veri işaretçisi ve klonlama, uyandırma, düşürme işlerinin fonksiyon işaretçilerini tutan bir tablo (vtable).[^rawwaker] Bu ham biçim `unsafe` gerektirir. `Wake` trait'i tabloyu `Arc<T>` için bizim yerimize kurar; bu yüzden elle `unsafe` yazmadık.[^wake-kod] `Waker` hem `Send` hem `Sync`'tir: Zili, yürütücünün thread'i dışındaki bir thread de çalabilir.

## Özet

- Async, bekleyen işleri az sayıda thread'de iç içe yürütür. Hesabı hızlandırmaz, bekleme boşluklarını doldurur.
- `async` gövdeyi çalıştırmaz, bir `Future` üretir. Future'lar tembeldir; biri yoklamadıkça ilerlemez.
- `.await` bir future'ı sürer ve yalnızca async gövdede yazılabilir. Sıra yalnızca duraklarda el değiştirir.
- `poll` ya `Ready` ya `Pending` döner. `Pending` dönen future, ilerleyebilecek duruma gelince uyandırıcıyı çalmalıdır; yürütücü de yeniden yoklamalıdır.
- Standart kütüphane yürütücü vermez. `block_on` tek bir future'ı, `cok_gorev` bir listeyi sürer; gerçek projede tokio gibi bir çalışma zamanı kullanılır.
- Derleyici async gövdeyi durum makinesine çevirir. Makine kendi içini gösterebildiği için yoklanmadan önce sabitlenir; `Pin` bu sözdür.

Sıradaki sayfada bu yürütücüyle gerçek kararları vereceğiz: [16.1 Async ve Await: Derinleşme](16-async-await-derinlesme.md).

## Kaynaklar

[^eszamanlilik]: The Rust Programming Language, [17. Parallelism and Concurrency](https://doc.rust-lang.org/book/ch17-00-async-await.html#parallelism-and-concurrency).
[^prelude]: The Edition Guide, [Rust 2024: Changes to the prelude](https://doc.rust-lang.org/edition-guide/rust-2024/prelude.html#summary).
[^hangisi]: The Rust Programming Language, [17.6 Putting It All Together: Futures, Tasks, and Threads](https://doc.rust-lang.org/book/ch17-06-futures-tasks-threads.html#putting-it-all-together-futures-tasks-and-threads).
[^async-fn]: The Rust Reference, [Async functions](https://doc.rust-lang.org/reference/items/functions.html#async-functions) ve [`async` blocks](https://doc.rust-lang.org/reference/expressions/block-expr.html#async-blocks).
[^tembel]: The Rust Programming Language, [17.1 Futures and the Async Syntax](https://doc.rust-lang.org/book/ch17-01-futures-and-syntax.html#futures-and-the-async-syntax).
[^duraklar]: The Rust Programming Language, [17.2 Code Within One Async Block Executes Linearly](https://doc.rust-lang.org/book/ch17-02-concurrency-with-async.html#code-within-one-async-block-executes-linearly) ve [17.3 Yielding Control to the Runtime](https://doc.rust-lang.org/book/ch17-03-more-futures.html#yielding-control-to-the-runtime).
[^await-ref]: The Rust Reference, [Await expressions](https://doc.rust-lang.org/reference/expressions/await-expr.html#await-expressions); The Rust Programming Language, [17.1 Executing an Async Function with a Runtime](https://doc.rust-lang.org/book/ch17-01-futures-and-syntax.html#executing-an-async-function-with-a-runtime).
[^tokio-main]: tokio belgesi, [`tokio::main`](https://docs.rs/tokio/latest/tokio/attr.main.html).
[^future-trait]: Standart kütüphane, [`Future`](https://doc.rust-lang.org/std/future/trait.Future.html#tymethod.poll); kaynak kodu 1.99.0, [`future.rs`, `pub trait Future`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/future/future.rs#L37) ve [`poll.rs`, `pub enum Poll`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/task/poll.rs#L14).
[^sozlesme]: Standart kütüphane, [`Future`: Runtime characteristics](https://doc.rust-lang.org/std/future/trait.Future.html#runtime-characteristics).
[^wake]: Standart kütüphane, [`Wake`](https://doc.rust-lang.org/std/task/trait.Wake.html#examples).
[^park]: Standart kütüphane, [`thread::park`](https://doc.rust-lang.org/std/thread/fn.park.html#park-and-unpark).
[^yield]: The Rust Programming Language, [17.3 Yielding Control to the Runtime](https://doc.rust-lang.org/book/ch17-03-more-futures.html#yielding-control-to-the-runtime).
[^runtime]: The Rust Programming Language, [17.1 Executing an Async Function with a Runtime](https://doc.rust-lang.org/book/ch17-01-futures-and-syntax.html#executing-an-async-function-with-a-runtime).
[^tokio]: tokio belgesi, [`tokio::runtime`](https://docs.rs/tokio/latest/tokio/runtime/index.html).
[^async-std]: RustSec, [`RUSTSEC-2025-0052`: async-std has been discontinued](https://rustsec.org/advisories/RUSTSEC-2025-0052.html).
[^durum-rfc]: [RFC 2394: async/await](https://rust-lang.github.io/rfcs/2394-async_await.html); The Rust Programming Language, [17.5 The `Future` Trait](https://doc.rust-lang.org/book/ch17-05-traits-for-async.html#the-future-trait).
[^durum-kod]: rustc 1.99.0 kaynak kodu, [`coroutine/mod.rs`, `StateTransform`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_mir_transform/src/coroutine/mod.rs#L85) ve [`locals_live_across_suspend_points`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_mir_transform/src/coroutine/mod.rs#L1102).
[^await-acilim]: The Rust Reference, [Await expressions: Approximate desugaring](https://doc.rust-lang.org/reference/expressions/await-expr.html#approximate-desugaring); standart kütüphane, [`IntoFuture`](https://doc.rust-lang.org/std/future/trait.IntoFuture.html).
[^await-kod]: rustc 1.99.0 kaynak kodu, [`expr.rs`, `make_lowered_await`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_ast_lowering/src/expr.rs#L938) ve [`AwaitOnlyInAsyncFnAndBlocks`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_ast_lowering/src/expr.rs#L957).
[^e0752-kod]: rustc 1.99.0 kaynak kodu, [`entry.rs`, `main_asyncness`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_analysis/src/check/entry.rs#L94).
[^pin]: Standart kütüphane, [`std::pin`: What is pinning?](https://doc.rust-lang.org/std/pin/index.html#what-is-pinning) ve [A self-referential struct](https://doc.rust-lang.org/std/pin/index.html#a-self-referential-struct); The Rust Programming Language, [17.5 The `Pin` Type and the `Unpin` Trait](https://doc.rust-lang.org/book/ch17-05-traits-for-async.html#the-pin-type-and-the-unpin-trait).
[^rawwaker]: Standart kütüphane 1.99.0 kaynak kodu, [`wake.rs`, `RawWakerVTable`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/task/wake.rs#L109) ve [`pub struct Waker`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/task/wake.rs#L407).
[^wake-kod]: Standart kütüphane 1.99.0 kaynak kodu, [`task.rs`, `pub trait Wake`](https://github.com/rust-lang/rust/blob/1.99.0/library/alloc/src/task.rs#L95).
