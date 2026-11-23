# 28. `Future`'ın İç Yüzü

[16. bölümde](../orta/16-async-await.md) `async` ve `.await`'i kullandık. Bu bölüm aynı mekanizmanın kapağını açıyor: Yoklama (poll) nedir, uyandırma nasıl işler, uyandırıcı (waker) içeriden neye benzer, `Pin` neden var?

Bölümün tek cümlelik özeti şudur: *Future*, henüz bitmemiş bir iştir; kendi kendine çalışmaz, biri onu yokladıkça ilerler ve bekleyeceği zaman "hazır olunca bana haber verin" diye bir zil bırakır. Zil çalınca yürütücü (executor) future'ı yeniden yoklar; arada kimse "oldu mu, oldu mu?" diye sormaz. Derleyici her `async fn`'i, `.await` noktalarında durup kaldığı yerden süren bir durum makinesine çevirir; o makine, future'ın ta kendisidir.

Yoklama, uyandırıcı ve `Pin` üçlüsünü anladığınızda asenkron kodun neden thread'i bloklamadığı, iptalin neden yalnızca "future'ı düşürmek" olduğu ve derleyicinin `cannot be unpinned` mesajının ne istediği açıklığa kavuşur. Bölüm [sahipliği](../temel/03-ownership.md), [trait'leri](../orta/09-generics-traits-lifetimes.md), [`Box` ile `Arc`'ı](../orta/14-smart-pointers.md) ve async kullanımını bildiğinizi varsayar. Örneklerin tamamı yalnızca standart kütüphaneyle yazılmıştır.

> [!NOTE]
> Bu bölümün notları ve çalıştırılabilir kodu `learn/advanced-concepts/ileri-future` dizinindedir. Depo kökünden `cargo run -p ileri-future` komutuyla çalıştırabilirsiniz.

## Günlük hayattan bir benzetme

Çamaşır makinesini çalıştırdığınızı ve üzerine "bittiğinde beni ara" diye not bıraktığınızı düşünün. Siz başka işlere bakarsınız; telefon çalınca gidip çamaşırı alırsınız. Beş dakikada bir makinenin başına gidip "bitti mi?" diye bakmazsınız. Makine future, telefon uyandırıcı, siz de yürütücüsünüz.

Elinizde bir de birkaç sayfaya bölünmüş uzun bir tarif olsun. Her sayfanın sonunda beklemeniz gerekir: hamur dinlenecek, fırın ısınacak. Beklerken kitabı kapatırsınız ama ayracı bırakırsınız; haber gelince kaldığınız sayfadan devam edersiniz. `async fn` böyle bir tariftir, her `.await` bir sayfa sonudur, derleyicinin ürettiği durum makinesi de ayraçtır.

<details>
<summary>Bu bölümde geçen terimler</summary>

- **future:** Henüz bitmemiş iş; sonucunu sonra verecek olan değer.
- **yoklama (poll):** Future'a "biraz daha ilerle, bitti mi?" diye sormak.
- **`Poll::Ready(değer)`:** "Bitti, sonuç bu" yanıtı.
- **`Poll::Pending`:** "Daha bitmedi, sonra yine sorun" yanıtı.
- **yürütücü (executor):** Future'ları yoklayan program parçası. İşi yürüten odur.
- **`block_on`:** "Bu future bitene kadar burada bekle" diyen en basit yürütücü.
- **uyandırıcı (waker):** "Ben hazırım, bana yine bakın" demeye yarayan zil.
- **`wake`, `wake_by_ref`:** O zili çalmak. `wake` zili tüketir, `wake_by_ref` elde bırakır.
- **`Context` (`cx`):** Yoklamaya verilen küçük çanta; içinde o anki uyandırıcı durur.
- **`async fn`, `.await`:** Future üreten fonksiyon; "bu iş bitene kadar burada durakla" işareti.
- **durum makinesi (state machine):** "Hangi adımdayım?" bilgisini saklayıp kaldığı yerden süren yapı.
- **coroutine (eski adıyla generator):** Derleyicinin `async fn`'den ürettiği durum makinesinin iç adı.
- **meşgul bekleme (busy-wait):** Durmadan "oldu mu?" diye sormak. İşlemciyi boş yere yorar.
- **`park`, `unpark`:** Bir thread'i uyutmak, uyandırmak.
- **thread (iş parçacığı):** Programın içinde, ötekilerle aynı anda çalışabilen ayrı bir yürütme kolu.
- **G/Ç (I/O):** Dış dünyayla alışveriş: dosya okumak, ağdan veri beklemek.
- **sözleşme (contract):** "Bunu yaparsanız şunu garanti ederim" diye verilen, uyulacak söz.
- **trait nesnesi (`dyn`):** Gerçek türü gizlenmiş, yalnızca ne yapabildiği bilinen değer.
- **vtable:** "Bu değer için hangi fonksiyon çağrılacak?" bilgisinin tutulduğu tablo.
- **türü silinmiş (type-erased):** Gerçek türü saklanan, yalnızca yapabildikleri görünen.
- **işaretçi (pointer), ham işaretçi (`*const T`):** Bir verinin bellekteki adresi; ham olanını derleyici denetlemez, doğruluğu size emanettir.
- **`unsafe`:** "Buradaki kuralı derleyici değil, ben garanti ediyorum" denen blok.
- **heap:** Belleğin, `Box`'ın değer koyduğu bölgesi.
- **`Arc`:** Birçok sahibin, thread'ler arasında da, paylaştığı sayaçlı işaretçi.
- **taşıma (move):** Bir değeri bellekte başka bir yere geçirmek; adresi değişir.
- **sarkan işaretçi (dangling):** Gösterdiği yerde artık doğru veri bulunmayan işaretçi.
- **kendine referanslı (self-referential):** İçindeki bir işaretçi, yine kendi içindeki bir alanı gösteren değer.
- **`Pin`:** Bir işaretçinin üstüne takılan kilit: "Gösterdiğim değer yerinden oynatılmayacak."
- **`Unpin`:** "Taşınsam da bozulmam" diyen türlerin işareti. Türlerin çoğu böyledir.
- **`PhantomPinned`:** Bir türü bilerek `Unpin` olmayan (`!Unpin`) yapan, yer kaplamayan alan.
- **`Send` ve `Sync`:** Başka bir thread'e verilebilir; thread'ler arasında paylaşılabilir.

</details>

## Yoklama: bir hesabı biraz ilerletmek

Fırına kek koydunuz. Kek kendi kendine "ben piştim" diye yanınıza gelmez; birinin fırının camından bakması gerekir. Bu bakışa *yoklama* denir ve `Future` trait'inin tek zorunlu metodudur:[^future]

```rust
# use std::pin::Pin;
# use std::task::{Context, Poll};
pub trait Future {
    type Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}
```

İki yanıt vardır: `Poll::Ready(değer)` "pişti, alın" demektir, `Poll::Pending` ise "daha olmadı".[^poll-enum] `Pin<&mut Self>`'i bölümün sonuna bırakalım; `cx`, yoklamaya verilen küçük bir çantadır ve içinde o anki uyandırıcıyı taşır. Bugün için `Context`'in tek işi budur.[^context]

`poll`'un uyması gereken sözleşme üç maddedir:[^future]

- İlerleyebildiği kadar ilerler, ama beklemek için thread'i *bloklamaz*; çabuk dönmeye çalışır.
- Bitirdiyse `Poll::Ready(değer)` döndürür.
- Bitiremediyse *önce* `cx` içindeki uyandırıcıyı, hazır olunca çalacak tarafa bırakır, *sonra* `Poll::Pending` döndürür.

Sözleşmeyi izleyebileceğimiz küçük bir future yazalım. `GecikmeliDeger`, "veri birkaç tur sonra hazır olacak" durumunu taklit eder: Her yoklamada bir tur azaltır, kendini uyandırıp `Pending` döner; sayaç sıfırlanınca `Ready` olur. Yürütücüyü yazmadan önce onu elle yoklayabiliriz. `Waker::noop()`, çalındığında hiçbir şey yapmayan bir uyandırıcıdır.[^noop]

```rust
use std::pin::{Pin, pin};
use std::task::{Context, Poll, Waker};

struct GecikmeliDeger {
    kalan: u32,
}

impl Future for GecikmeliDeger {
    type Output = &'static str;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<&'static str> {
        println!("[future.poll] çağrıldı, kalan = {}", self.kalan);
        if self.kalan == 0 {
            println!("[future.poll] hazır: Poll::Ready");
            return Poll::Ready("İŞ BİTTİ");
        }
        self.kalan -= 1;
        println!("[future.poll] hazır değil: zili çal, Poll::Pending");
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

fn main() {
    let mut is = pin!(GecikmeliDeger { kalan: 1 });
    println!("future oluşturuldu");

    let mut cx = Context::from_waker(Waker::noop());
    println!("birinci yoklama: {:?}", is.as_mut().poll(&mut cx));
    println!("ikinci yoklama: {:?}", is.as_mut().poll(&mut cx));
}
```

Çıktı:

```text
future oluşturuldu
[future.poll] çağrıldı, kalan = 1
[future.poll] hazır değil: zili çal, Poll::Pending
birinci yoklama: Pending
[future.poll] çağrıldı, kalan = 0
[future.poll] hazır: Poll::Ready
ikinci yoklama: Ready("İŞ BİTTİ")
```

İlk satıra dikkat edin: Future'ı oluşturmak işi *başlatmadı*. Future'lar tembeldir; iş ancak yoklandıkça ilerler.[^tembel] (Başka bir görevde yürüyen bir işin sonucunu bekleyen future'lar bunun istisnasıdır: İş kendi başına ilerler, future yalnızca sonucu taşır.) Bu düzenin amacı, bekleyen binlerce iş için binlerce thread tutmamaktır. Bekleyen iş kenara konur, zili çalınca kaldığı yerden sürer.

`GecikmeliDeger` zili yoklamanın içinde, hemen çalıyor; veri sanki o anda hazır olmuş gibi. Gerçek bir future'da uyandırıcı, olayı bilecek tarafa (bir sokete, zamanlayıcıya ya da thread'e) bırakılır. Onu birazdan göreceğiz.

### Zili kurmadan `Pending` dönmek

Sözleşmenin üçüncü maddesini derleyici denetlemez. Uyandırıcıyı kimseye bırakmadan `Pending` dönen future derlenir, ama kimse "artık hazırım" diyemez. Zilin kaç kez çalındığını sayan bir uyandırıcıyla bunu görelim:

```rust
use std::pin::{Pin, pin};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::task::{Context, Poll, Wake, Waker};

struct SayanZil(AtomicU32);

impl Wake for SayanZil {
    fn wake(self: Arc<Self>) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}

struct Sessiz {
    hazir: bool,
}

impl Future for Sessiz {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
        if self.hazir {
            return Poll::Ready(());
        }
        self.hazir = true; // bir sonraki yoklamada hazır olacak, ama zil kurulmadı
        Poll::Pending
    }
}

fn main() {
    let zil = Arc::new(SayanZil(AtomicU32::new(0)));
    let waker = Waker::from(Arc::clone(&zil));
    let mut cx = Context::from_waker(&waker);

    let mut is = pin!(Sessiz { hazir: false });
    println!("{:?}", is.as_mut().poll(&mut cx));
    println!("zil {} kez çaldı", zil.0.load(Ordering::Relaxed));
}
```

Çıktı:

```text
Pending
zil 0 kez çaldı
```

Yürütücü burada uykuya dalsa bir daha uyanmaz: Kek fırında unutulmuştur. Aynı görev başka bir nedenle (örneğin yanındaki bir future uyandığı için) yeniden yoklanabilir ve iş şans eseri sürebilir; ama buna güvenilemez. `Pending` dönen future, kendini yeniden yoklatmanın sorumluluğunu da üstlenmiştir.[^donus]

## Yürütücü: yoklama döngüsü

Future'ı yoklayacak biri olmazsa hiçbir iş ilerlemez; *yürütücü* o "biri"dir. Göl kıyısındaki balıkçıya benzer: Oltasına bakar (yoklar); balık yoksa durmadan suya bakıp yorulmaz, uzanıp uyur (`thread::park`). Oltanın ucundaki çıngırak çalınca (`wake` çağrılınca, o da `unpark` edince) uyanır ve oltaya yeniden bakar. Aşağıdaki `GecikmeliDeger` yukarıdakinin aynısıdır:

```rust
# use std::pin::Pin;
use std::pin::pin;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use std::thread::{self, Thread};

# struct GecikmeliDeger {
#     kalan: u32,
# }
# impl Future for GecikmeliDeger {
#     type Output = &'static str;
#     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<&'static str> {
#         println!("   [future.poll] çağrıldı, kalan = {}", self.kalan);
#         if self.kalan == 0 {
#             println!("   [future.poll] hazır: Poll::Ready");
#             return Poll::Ready("İŞ BİTTİ");
#         }
#         self.kalan -= 1;
#         println!("   [future.poll] hazır değil: zili çal, Poll::Pending");
#         cx.waker().wake_by_ref();
#         Poll::Pending
#     }
# }
struct ThreadWaker(Thread);

impl Wake for ThreadWaker {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}

fn izle_block_on<F: Future>(fut: F) -> F::Output {
    let mut fut = pin!(fut);
    let waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
    let mut cx = Context::from_waker(&waker);
    let mut tur = 0;
    loop {
        tur += 1;
        println!("[yürütücü] tur {tur}: yokluyorum");
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(deger) => {
                println!("[yürütücü] Ready geldi, döngüden çık");
                return deger;
            }
            Poll::Pending => {
                thread::park(); // zil çalana kadar uyu
                println!("[yürütücü] Pending geldi, uyudum, uyandırıldım");
            }
        }
    }
}

fn main() {
    let f = GecikmeliDeger { kalan: 3 }; // henüz hiçbir şey çalışmadı
    let sonuc = izle_block_on(f);
    println!("block_on döndü: {sonuc:?}");
}
```

Çıktı:

```text
[yürütücü] tur 1: yokluyorum
   [future.poll] çağrıldı, kalan = 3
   [future.poll] hazır değil: zili çal, Poll::Pending
[yürütücü] Pending geldi, uyudum, uyandırıldım
[yürütücü] tur 2: yokluyorum
   [future.poll] çağrıldı, kalan = 2
   [future.poll] hazır değil: zili çal, Poll::Pending
[yürütücü] Pending geldi, uyudum, uyandırıldım
[yürütücü] tur 3: yokluyorum
   [future.poll] çağrıldı, kalan = 1
   [future.poll] hazır değil: zili çal, Poll::Pending
[yürütücü] Pending geldi, uyudum, uyandırıldım
[yürütücü] tur 4: yokluyorum
   [future.poll] çağrıldı, kalan = 0
   [future.poll] hazır: Poll::Ready
[yürütücü] Ready geldi, döngüden çık
block_on döndü: "İŞ BİTTİ"
```

Üç `Pending`, bir `Ready`: dört tur. Zil, yürütücü uykuya dalmadan *önce* çalındığı hâlde uyandırma kaybolmadı; çünkü `unpark` bir jeton bırakır ve sonraki `park` o jetonu bulup hemen döner.[^park] `park` kendiliğinden de uyanabilir; bu zararsızdır, döngü yalnızca bir kez fazladan yoklar.

Döngüde `park` yerine hiçbir şey olmasaydı program yine doğru sonucu verirdi, ama yürütücü sürekli "oldu mu?" diye sorup işlemciyi boşa çalıştırırdı. Buna meşgul bekleme (busy-wait) denir; future yalnızca ilerleyebileceği bildirildiğinde yoklanmalıdır.[^tembel]

> [!NOTE]
> `izle_block_on` anlatım için sadeleştirilmiştir. Zilin çalınmasıyla `park` arasında `println!` çağrılmamasına özen gösterildi: Standart kütüphanenin belgeleri, `park` kullanan başka kodun (`println!` de bunlardan biri olabilir) bırakılan jetonu tüketebileceğini ve programın kilitlenebileceğini söyler.[^park] Gerçek yürütücüler uyandırmayı ayrıca bir bayrakta ya da kuyrukta tutar; kuyruklu bir yürütücüyü [29.2'de](29-calisma-zamani.md) yazacağız.

Future'ı `izle_block_on`'a verdiğinizde sahipliğini de verirsiniz. Aynı değeri ikinci kez çalıştıramazsınız:

```rust,compile_fail,E0382
fn izle_block_on<F: Future>(_fut: F) {} // gövdesi yukarıda

async fn hesapla() -> i32 {
    1
}

fn main() {
    let f = hesapla();
    izle_block_on(f);
    izle_block_on(f);
}
```

Derleyicinin yanıtı:

```text
error[E0382]: use of moved value: `f`
   |
 8 |     let f = hesapla();
   |         - move occurs because `f` has type `impl Future<Output = i32>`, which does not implement the `Copy` trait
 9 |     izle_block_on(f);
   |                   - value moved here
10 |     izle_block_on(f);
   |                   ^ value used here after move
```

Standart kütüphanede bir yürütücü yoktur. Kütüphane yalnızca mekanizmayı, yani `Future`, `Poll`, `Waker` ve `Pin`'i tanımlar; yoklama döngüsünün, G/Ç bütünleştirmesinin ve zamanlayıcının tek bir doğru biçimi olmadığı için bunlar tokio ve smol gibi crate'lere bırakılmıştır.[^yurutucu] (Eski kaynaklarda sık anılan async-std'nin geliştirilmesi 2025'te sona ermiştir; yerine smol önerilir.[^async-std]) `izle_block_on`, böyle bir döngünün en yalın çekirdeğidir.

## Uyandırıcı (Waker) gerçekte nedir?

Kalabalık bir kafede sipariş verince elinize titreyen bir çağrı cihazı verirler. Mutfağın içini bilmezsiniz; bilmeniz gereken tek şey, cihaz titrerse siparişin hazır olduğudur. `Waker` bu cihazdır. Future onu saklar; iş hazır olunca çalar, yürütücü de future'a yeniden bakar. Cihazın nasıl çaldığı yürütücünün işidir: Biri thread'i `unpark` eder, öteki görevi bir kuyruğa koyar. Future bunu bilmez, yalnızca `wake()` der.

İçeriden bakıldığında uyandırıcı bir trait nesnesine benzer, ama elle yazılmış hâlidir: bir veri işaretçisi ile `&'static RawWakerVTable` türünde bir tablo.[^rawwaker] Tablo dört fonksiyon işaretçisi tutar: `clone`, `wake`, `wake_by_ref` ve `drop`. Veri işaretçisinin gerçek türü silinmiştir (type-erased); hangi türe ait olduğunu yalnızca tablodaki fonksiyonlar bilir.

Ham `RawWaker` ve `RawWakerVTable` yazmak `unsafe` sözleşmeler üstlenmek demektir. Çoğu zaman gerekmez: `std::task::Wake` trait'ini uygulayan bir türü `Arc` içine koyup `Waker::from` ile güvenli biçimde uyandırıcıya çevirebilirsiniz.[^wake-trait] Bölümün başından beri yaptığımız budur.

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::task::{Wake, Waker};

struct SayanZil(AtomicU32);

impl Wake for SayanZil {
    fn wake(self: Arc<Self>) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}

fn main() {
    let zil = Arc::new(SayanZil(AtomicU32::new(0)));

    let w = Waker::from(Arc::clone(&zil));
    let w2 = w.clone(); // ikinci cihaz: aynı görevi uyandırır
    println!("aynı görevi mi uyandırırlar? {}", w.will_wake(&w2));

    w.wake_by_ref(); // zili çal, cihaz elde kalsın
    w2.wake(); // zili çal, cihazı geri ver
    println!("zil {} kez çaldı", zil.0.load(Ordering::Relaxed));
}
```

Çıktı:

```text
aynı görevi mi uyandırırlar? true
zil 2 kez çaldı
```

`wake` uyandırıcıyı tüketir, `wake_by_ref` tüketmez. Tüketilmiş bir uyandırıcıyı yeniden çalmak derlenmez:

```rust,compile_fail,E0382
use std::task::Waker;

fn main() {
    let w2 = Waker::noop().clone();
    w2.wake();
    w2.wake_by_ref();
}
```

Derleyicinin yanıtı:

```text
error[E0382]: borrow of moved value: `w2`
    |
  4 |     let w2 = Waker::noop().clone();
    |         -- move occurs because `w2` has type `Waker`, which does not implement the `Copy` trait
  5 |     w2.wake();
    |        ------ `w2` moved due to this method call
  6 |     w2.wake_by_ref();
    |     ^^ value borrowed here after move
    |
note: `std::task::Waker::wake` takes ownership of the receiver `self`, which moves `w2`
```

`will_wake` en iyi çaba (best-effort) esasına göre çalışır: `true` dönerse iki uyandırıcı kesinlikle aynı görevi uyandırır; `false` dönmesi ise farklı görevler olduklarını kanıtlamaz. Gereksiz klonlamadan kaçınmak için kullanılır.[^will-wake]

### Uyandırıcıyı olayı bilen tarafa bırakmak

`Waker`; `Clone`, `Send` ve `Sync`'tir.[^waker] Bu yüzden çoğaltılıp başka bir thread'e verilebilir ve oradan çalınabilir. Gerçek bir future tam da bunu yapar: Uyandırıcının bir kopyasını, olayı bilecek tarafın bulacağı yere bırakır. Aşağıda "olay", başka bir thread'in değeri yuvaya koymasıdır:

```rust
use std::pin::{Pin, pin};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};
use std::thread::{self, Thread};

#[derive(Default)]
struct Yuva {
    deger: Option<u32>,
    bekleyen: Option<Waker>,
}

struct YuvadanAl(Arc<Mutex<Yuva>>);

impl Future for YuvadanAl {
    type Output = u32;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<u32> {
        let mut yuva = self.0.lock().expect("kilidi tutan thread paniklemedi");
        if let Some(deger) = yuva.deger.take() {
            return Poll::Ready(deger);
        }
        // Her yoklamada en son uyandırıcı saklanır; aynıysa klonlanmaz.
        match &mut yuva.bekleyen {
            Some(eski) => eski.clone_from(cx.waker()),
            bos => *bos = Some(cx.waker().clone()),
        }
        Poll::Pending
    }
}

# struct ThreadWaker(Thread);
# impl Wake for ThreadWaker {
#     fn wake(self: Arc<Self>) {
#         self.0.unpark();
#     }
# }
# fn block_on<F: Future>(fut: F) -> F::Output {
#     let mut fut = pin!(fut);
#     let waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
#     let mut cx = Context::from_waker(&waker);
#     loop {
#         match fut.as_mut().poll(&mut cx) {
#             Poll::Ready(deger) => return deger,
#             Poll::Pending => thread::park(),
#         }
#     }
# }
fn main() {
    let yuva = Arc::new(Mutex::new(Yuva::default()));
    let yazan_taraf = Arc::clone(&yuva);

    let yazan = thread::spawn(move || {
        let mut yuva = yazan_taraf.lock().expect("kilidi tutan thread paniklemedi");
        yuva.deger = Some(42);
        let bekleyen = yuva.bekleyen.take();
        drop(yuva); // zili kilidi bıraktıktan sonra çal
        if let Some(waker) = bekleyen {
            waker.wake();
        }
    });

    println!("{}", block_on(YuvadanAl(yuva)));
    yazan.join().expect("yazan thread paniklemedi");
}
```

Çıktı:

```text
42
```

(`block_on`, `izle_block_on`'un `println!`'siz hâlidir.) Burada üç ayrıntı önemlidir:

- **Denetim ile kayıt aynı kilit altındadır.** Ayrı ayrı yapılsaydı, "değer yok" denmesiyle uyandırıcının bırakılması arasında değer gelebilir ve zil boş yuvada çalınmadan geçerdi. [29.2](29-calisma-zamani.md) bu yarışı ayrıntılı ele alır.
- **Saklanan uyandırıcı her yoklamada güncellenir.** Yalnızca *en son* yoklamanın `Context`'inden gelen uyandırıcının çalınması garanti edilir;[^donus] görev arada başka bir yere taşınmış olabilir. Eskisini saklayan future hiç uyanmayabilir.
- **Uyandırmalar birleşebilir.** Yürütücü çalıştığı ve görev bitmediği sürece her `wake` çağrısını en az bir yoklama izler; ama art arda gelen birkaç `wake` tek bir yoklamaya karşılık gelebilir.[^wake-metodu] Uyandırma sayısını iş sayısı gibi yorumlamayın.

## Bitmiş future yeniden yoklanmaz

Bir future yalnızca bir kez tamamlanır. Standart kütüphanenin belgesi, `Ready` döndükten sonra yapılacak bir yoklama için hiçbir davranış vaat etmez: Çağrı panikleyebilir, sonsuza dek bloklayabilir ya da başka bir soruna yol açabilir. Tek garanti, `poll` metodu `unsafe` olmadığı için çağrının tanımsız davranışa yol açmamasıdır.[^panik]

Ne olacağı future'ın türüne bağlıdır. Derleyicinin ürettiği future'lar, 1.99.0'da, panikler:

```rust,should_panic
use std::pin::pin;
use std::task::{Context, Waker};

fn main() {
    let mut is = pin!(async { 1 });
    let mut cx = Context::from_waker(Waker::noop());
    println!("{:?}", is.as_mut().poll(&mut cx));
    println!("{:?}", is.as_mut().poll(&mut cx)); // bitmiş future yeniden yoklanıyor
}
```

Örnek bir çalıştırma:

```text
Ready(1)

thread 'main' (12345) panicked at src/main.rs:5:23:
`async fn` resumed after completion
```

`std::future::ready` de panikler; mesajı `` `Ready` polled after completion `` olur. Elle yazdığınız bir future ise (örneğin yukarıdaki `GecikmeliDeger`) her seferinde yeniden `Ready` dönebilir. Bu davranışların hiçbirine güvenilmemelidir. Aynı incelik `Iterator`'da da vardır: `None` döndükten sonra `next` çağırmanın sonucu türe bağlıdır.

Bu kuralın pratik bir sonucu vardır: Başarısız bir işi yeniden denemek isteyen kod, hazır bir future *değil*, her denemede taze bir future üreten closure ister. [29.3'teki](29-istemci.md) `yeniden_dene` bu yüzden closure alır.

## `async fn` bir durum makinesidir

Derleyici bir `async fn`'i, `Future` trait'ini uygulayan adsız bir türe çevirir. Her `.await`, makinenin durabileceği bir noktadır; yoklama geldiğinde makine kaldığı durumdan sürer. Bu makinenin nasıl kullanıldığını [16. bölüm](../orta/16-async-await.md) anlatır; burada nasıl davrandığına bakacağız. Önce duraklamayı gözle görelim. `Durakla`, bir kez `Pending` dönen küçük bir future'dır:

```rust
use std::pin::{Pin, pin};
use std::task::{Context, Poll, Waker};

#[derive(Default)]
struct Durakla {
    durdu: bool,
}

impl Future for Durakla {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.durdu {
            return Poll::Ready(());
        }
        self.durdu = true;
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

async fn tarif() -> &'static str {
    println!("  sayfa 1: hamur yoğruldu, dinleniyor");
    Durakla::default().await;
    println!("  sayfa 2: fırın ısınıyor");
    Durakla::default().await;
    println!("  sayfa 3: kek fırından çıktı");
    "kek"
}

fn main() {
    let mut is = pin!(tarif());
    let mut cx = Context::from_waker(Waker::noop());
    let mut tur = 0;
    let sonuc = loop {
        tur += 1;
        println!("yoklama {tur}");
        if let Poll::Ready(sonuc) = is.as_mut().poll(&mut cx) {
            break sonuc;
        }
    };
    println!("sonuç: {sonuc}");
}
```

Çıktı:

```text
yoklama 1
  sayfa 1: hamur yoğruldu, dinleniyor
yoklama 2
  sayfa 2: fırın ısınıyor
yoklama 3
  sayfa 3: kek fırından çıktı
sonuç: kek
```

Fonksiyonun gövdesi üç parçaya bölündü ve her yoklama bir parçayı yürüttü. (Bu örnekte hemen yeniden yokladık; gerçek bir yürütücü zilin çalmasını beklerdi.)

### Makine neyi saklar?

Makine, bir `.await` noktasını *geçen* yerel değişkenleri ve o sırada beklediği alt future'ı kendi içinde tutar; kaldığı yerden sürebilmesi için buna mecburdur.[^makine] Bu yüzden `async fn`'ler görünmeden büyüyebilir:

```rust
use std::future::ready;

async fn buyuk() -> usize {
    let dizi = [0u8; 1024];
    ready(()).await;
    dizi.len() // dizi, await'ten sonra da gerekli: makinede saklanır
}

async fn kucuk() -> usize {
    let uzunluk = {
        let dizi = [0u8; 1024];
        dizi.len()
    }; // dizi await'ten önce düştü
    ready(()).await;
    uzunluk
}

async fn saran() -> usize {
    buyuk().await // alt future da makinenin içinde
}

async fn kutulu() -> usize {
    Box::pin(buyuk()).await // makinede yalnızca bir işaretçi
}

fn main() {
    println!("buyuk:  {}", size_of_val(&buyuk()));
    println!("kucuk:  {}", size_of_val(&kucuk()));
    println!("saran:  {}", size_of_val(&saran()));
    println!("kutulu: {}", size_of_val(&kutulu()));
}
```

Örnek bir çalıştırma (64 bitlik hedef; yerleşim dilde belirtilmemiştir, sayılar sürüme göre değişebilir[^async-blok]):

```text
buyuk:  1026
kucuk:  16
saran:  1027
kutulu: 16
```

Future'ı taşımak, örneğin bir görev kuyruğuna koymak, bu baytların kopyalanması demektir. Büyük bir future'ı `Box::pin` ile heap'e almak taşımayı bir işaretçi kopyasına indirger;[^box-pin] daha da iyisi, büyük yerel değişkeni `.await`'ten önce düşürmektir.

### İptal, future'ı düşürmektir

Makine yalnızca yoklandıkça ilerlediğine göre, onu yoklamayı bırakıp düşürmek işi iptal etmektir. Derleyici düşürme kodunu da üretir: Makine hangi noktada durduysa, o noktada canlı olan yerel değişkenler düşürülür.[^makine]

```rust
use std::future::pending;
use std::task::{Context, Waker};

struct Bekci;

impl Drop for Bekci {
    fn drop(&mut self) {
        println!("yerel değişken düşürüldü");
    }
}

async fn uzun_is() {
    let _bekci = Bekci;
    println!("iş başladı");
    pending::<()>().await; // bu future hiç bitmez
    println!("buraya gelinmez");
}

fn main() {
    let mut is = Box::pin(uzun_is());
    let mut cx = Context::from_waker(Waker::noop());
    println!("{:?}", is.as_mut().poll(&mut cx));
    drop(is);
    println!("iptal edildi");
}
```

Çıktı:

```text
iş başladı
Pending
yerel değişken düşürüldü
iptal edildi
```

Demek ki bir future her `.await` noktasında yarıda kesilebilir. Zaman aşımı gibi araçlar tam olarak bunu yapar: Süre dolunca içteki future'ı düşürür. [29.2'de](29-calisma-zamani.md) böyle bir araç yazacağız.

## `Pin`: yerinden oynatılmaması gereken değerler

`poll`'un alıcısı neden `&mut self` değil de `self: Pin<&mut Self>`? Çünkü bazı future'lar, bir kez yoklandıktan sonra bellekte yer değiştirirse bozulur.

### Sorun: kendine referanslı bir değeri taşımak

Durum makinesi, bir `.await` noktasında hem bir yerel değişkeni hem de o değişkene bakan bir referansı saklayabilir; örneğin beklenen alt future, argümanına referans tutuyordur. O zaman makinenin içindeki bir işaretçi, yine makinenin kendi alanını gösterir. Böyle değerlere *kendine referanslı* (self-referential) denir.[^pin-oz]

Bir karavanın içine "anahtar şu adresteki karavanın çekmecesinde" yazan bir not astığınızı düşünün. Karavanı başka bir yere çekerseniz not hâlâ eski adresi gösterir; oraya giden boş araziyi bulur. Aynı şeyi, `unsafe` kullanmadan, yalnızca adresleri karşılaştırarak görebiliriz:

```rust
use std::ptr;

struct Karavan {
    cekmece: u32,
    not: *const u32, // kendi cekmece alanını gösterecek
}

fn main() {
    let mut a = Karavan { cekmece: 7, not: ptr::null() };
    a.not = &a.cekmece;
    println!("taşımadan önce not doğru mu? {}", ptr::eq(a.not, &a.cekmece));

    let b = Box::new(a); // değer heap'e taşındı, not değişmedi
    println!("taşımadan sonra not doğru mu? {}", ptr::eq(b.not, &b.cekmece));
}
```

Çıktı:

```text
taşımadan önce not doğru mu? true
taşımadan sonra not doğru mu? false
```

Taşıma yalnızca baytları kopyalar; içerideki işaretçiyi kimse düzeltmez. `b.not` artık *sarkan* bir işaretçidir: Boşaltılmış eski yeri gösterir ve o yer başka bir değere verilebilir. Rust'ta değerler normalde serbestçe taşınabildiği için, önlem alınmasaydı derleyicinin ürettiği durum makineleri bu tehlikeye açık olurdu.

### Çözüm: adresi sabitlemek

`Pin`, karavanın tekerine takılan kilittir. Değerin kendisi değil, değeri gösteren bir işaretçinin sarmalayıcısıdır: `Pin<Box<T>>`, `Pin<&mut T>`. Bir değer sabitlendikten sonra, düşürülene kadar aynı adreste geçerli kalacağı garanti edilir;[^pin-sozlesme] birazdan göreceğimiz `Unpin` türler bu garantinin dışındadır. Adres değişmeyince içerideki işaretçiler de geçerli kalır.

```rust
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::ptr;

struct KendineRef {
    veri: String,
    bana: *const String, // kendi veri alanını gösterecek
    _pin: PhantomPinned, // türü !Unpin yapar
}

impl KendineRef {
    fn yeni(s: &str) -> Pin<Box<Self>> {
        // Önce heap'te oluştur ve sabitle: adres artık değişmez.
        let mut kutu = Box::pin(KendineRef {
            veri: String::from(s),
            bana: ptr::null(),
            _pin: PhantomPinned,
        });
        // Adres sabitken iç işaretçiyi kur.
        let adres: *const String = &kutu.veri;
        // SAFETY: yalnızca `bana` alanına yazılıyor; sabitlenmiş değer taşınmıyor.
        unsafe {
            let mref: &mut KendineRef = Pin::get_unchecked_mut(kutu.as_mut());
            mref.bana = adres;
        }
        kutu
    }

    fn veri(&self) -> &str {
        &self.veri
    }

    fn bana_uzerinden(&self) -> &String {
        // SAFETY: `bana`, `yeni` içinde `veri`nin adresiyle kuruldu. Değer sabitlendiği
        // için yer değiştirmedi; `veri` de kurulduktan sonra değiştirilmiyor.
        unsafe { &*self.bana }
    }
}

fn main() {
    let p = KendineRef::yeni("merhaba");
    println!("veri()           = {}", p.veri());
    println!("bana_uzerinden() = {}", p.bana_uzerinden());

    let p2 = p; // işaretçi taşındı, gösterdiği değer yerinde
    println!("iç işaretçi hâlâ doğru mu? {}", ptr::eq(p2.bana, &p2.veri));
}
```

Çıktı:

```text
veri()           = merhaba
bana_uzerinden() = merhaba
iç işaretçi hâlâ doğru mu? true
```

`PhantomPinned` alanı türü `Unpin` *olmayan* (`!Unpin`) bir tür yapar; "taşınamaz" sözü ancak böyle türlerde anlam kazanır.[^unpin] `!Unpin` bir değere güvenli kodla `&mut` alamazsınız, çünkü `&mut` verilseydi `mem::swap` gibi bir çağrı değeri yerinden oynatabilirdi:

```rust,compile_fail,E0596
# use std::marker::PhantomPinned;
# use std::pin::Pin;
# struct KendineRef {
#     veri: String,
#     _pin: PhantomPinned,
# }
# impl KendineRef {
#     fn yeni(s: &str) -> Pin<Box<Self>> {
#         Box::pin(KendineRef { veri: String::from(s), _pin: PhantomPinned })
#     }
# }
fn main() {
    let mut k = KendineRef::yeni("selam");
    let r: &mut KendineRef = &mut *k;
    r.veri.push('!');
}
```

Derleyicinin yanıtı:

```text
error[E0596]: cannot borrow data in dereference of `Pin<Box<KendineRef>>` as mutable
   |
   = help: trait `DerefMut` is required to modify through a dereference, but it is not implemented for `Pin<Box<KendineRef>>`
```

`get_unchecked_mut` bu yüzden `unsafe`'tir: "Bu `&mut` ile değeri taşımayacağım" sözünü siz verirsiniz.

### `Unpin`: kilidin boş olduğu türler

Türlerin çoğu taşınınca bozulmaz: sayılar, `String`, `Vec<T>`. Bunlar kendiliğinden `Unpin`'dir ve `Pin` onlarda hiçbir şeyi kısıtlamaz.[^unpin] `Pin<Box<i32>>`'nin içine olağan yolla yazabilirsiniz:

```rust
fn main() {
    let mut a = Box::pin(5);
    *a = 6; // i32 Unpin'dir
    println!("{a}");
}
```

Çıktı:

```text
6
```

`async fn` ve `async` bloklarının ürettiği future'lar ise `!Unpin`'dir. `Pin::new` yalnızca `Unpin` türler için güvenli olduğundan onlarla çalışmaz:

```rust,compile_fail,E0277
use std::pin::Pin;

async fn hesapla() -> i32 {
    1
}

fn main() {
    let mut is = hesapla();
    let sabit = Pin::new(&mut is);
}
```

Derleyicinin yanıtı:

```text
error[E0277]: `{async fn body of hesapla()}` cannot be unpinned
     |
   3 | async fn hesapla() -> i32 {
     |                       --- within this `impl Future<Output = i32>`
...
   9 |     let sabit = Pin::new(&mut is);
     |                 -------- ^^^^^^^ within `impl Future<Output = i32>`, the trait `Unpin` is not implemented for `{async fn body of hesapla()}`
     |                 |
     |                 required by a bound introduced by this call
     |
     = note: consider using the `pin!` macro
             consider using `Box::pin` if you need to access the pinned value outside of the current scope
```

Mesaj iki çıkış yolu gösteriyor. Hangisini seçeceğiniz, sabitlenen future'ın nerede yaşayacağına bağlıdır:

| Araç | Ne zaman? |
| --- | --- |
| `Pin::new(&mut x)` | `x` `Unpin` ise; kilit zaten boştur. |
| `pin!(x)` | Future bulunduğu fonksiyonda yoklanacaksa. Tahsis yapmaz, sabit referans o kapsamın dışına çıkamaz.[^pin-makro] |
| `Box::pin(x)` | Future döndürülecek, saklanacak ya da başka bir göreve verilecekse. Heap tahsisi yapar. |
| pin projeksiyonu (pin projection) | Sabitlenmiş bir struct'ın *alanını* yoklamak gerektiğinde. [29.2](29-calisma-zamani.md) gösterir. |

`.await` yazdığınızda bunların hiçbirini görmezsiniz; derleyici alt future'ı sizin yerinize sabitler. `pin!` ile `Box::pin`'in gündelik kullanımı için [Async / Await: Derinleşme](../orta/16-async-await-derinlesme.md) sayfasına bakabilirsiniz.

### Neden `poll`'un imzasında?

Future ilk yoklamaya kadar serbestçe taşınabilir; çünkü iç referanslar ancak gövde çalışmaya başlayınca kurulabilir. İlk yoklamadan sonra taşınmamalıdır. `Pin<&mut Self>` isteyen bir imza, bu kuralı belgeye yazılmış bir ricadan çıkarıp tür düzeyinde bir şarta dönüştürür: Sabitlemeden yoklayamazsınız, sabitledikten sonra da güvenli kodla taşıyamazsınız.

## Hata kodu başvurusu

| Kod | Ne zaman görülür? | Ne yapmalı? |
| --- | --- | --- |
| `E0277` (`cannot be unpinned`) | `Pin::new`, `!Unpin` bir değere uygulandığında | `pin!`, `Box::pin` ya da alanlarda pin projeksiyonu kullanın. |
| `E0382` | Yürütücüye verilmiş future ya da `wake()` ile tüketilmiş uyandırıcı yeniden kullanıldığında | Taze bir future üretin; uyandırıcı elde kalacaksa `wake_by_ref()` kullanın. |
| `E0596` | Sabitlenmiş `!Unpin` bir değere `&mut` alınmak istendiğinde | Değeri `Pin`'in sunduğu yollarla (`as_mut`, projeksiyon) değiştirin. |
| `E0599` | Sabitlenmemiş bir future'da `poll` çağrıldığında | Önce `pin!` ya da `Box::pin` ile sabitleyin. |

Bu bölümün en pahalı hatalarını ise derleyici yakalamaz; kod derlenir, sorun çalışma zamanında ortaya çıkar:

| Hata | Belirti | Çözüm |
| --- | --- | --- |
| Uyandırıcıyı bırakmadan `Pending` dönmek | Görev askıda kalır | Önce `cx.waker()`'ı olayı bilecek tarafa bırakın, sonra `Pending` dönün. |
| Eski uyandırıcıyı saklamak | Görev hiç uyanmayabilir | Her yoklamada `clone_from` ile güncelleyin. |
| Bitmiş future'ı yeniden yoklamak | Panik, askıda kalma ya da başka bir sorun | Her deneme için taze future üretin. |
| `Pending` dönen future'ı sık bir döngüde yoklamak | İşlemci boşa çalışır | Zil çalana kadar uyuyun. |
| `poll` içinde bloklamak | Aynı thread'deki bütün görevler durur | Uzun işi ayrı bir thread'e verin ([29.3](29-istemci.md)). |
| Ham `RawWaker` yazmak | `unsafe` sözleşmeyi siz üstlenirsiniz | `std::task::Wake` ile `Arc` kullanın. |

## Perde arkası: Derleyici ne yapıyor?

> [!NOTE]
> Bu başlık ilk okuyuşta atlanabilir. Yukarıda bir saatin nasıl kullanıldığını ve üç ana parçasını (yoklama, uyandırıcı, `Pin`) gördünüz; burada saatin arka kapağını açıyoruz.

**Trait ve yanıtlar.** `Future` trait'i `core::future`'dadır ve tek zorunlu metodu `poll`'dur;[^kod-future] `Poll`, `Ready(T)` ve `Pending` varyantlarından oluşan sıradan bir enum'dur.[^kod-poll]

**Uyandırıcı.** `Waker`, tek alanı bir `RawWaker` olan bir struct'tır; `RawWaker` de `data: *const ()` ile `vtable: &'static RawWakerVTable` çiftidir.[^kod-waker] 1.99.0'da, 64 bitlik hedefte iki işaretçilik yer, yani 16 bayt tutar. `RawWakerVTable` dört fonksiyon işaretçisini saklar;[^kod-vtable] `wake()`, tablodaki `wake` fonksiyonunu veri işaretçisiyle çağırır, `Clone` ve `Drop` uygulamaları da aynı tabloya gider. Bir trait nesnesi bu işi derleyicinin ürettiği bir vtable ile yapar; burada tablo elle yazılmıştır. `Wake` trait'i sayesinde `Arc<W>`'den uyandırıcı üretirken ek bir tahsis yapılmaz: Veri işaretçisi `Arc`'ın kendi adresidir, `clone` sayacı artırır, `drop` azaltır.[^kod-wake]

**`.await`.** Derleyici `x.await` ifadesini, HIR'e indirirken bir döngüye açar: `IntoFuture::into_future(x)` ile future'ı alır, sabitleyip yoklar; `Ready` gelirse döngüden çıkar, `Pending` gelirse denetimi dışarıya bırakır (`yield`) ve bir sonraki yoklamada döngünün başına döner.[^kod-await]

**Durum makinesi.** Derleyicinin içinde bu makinelere *coroutine* denir; eski kaynaklarda generator adıyla geçer. MIR üzerinde çalışan `StateTransform` geçişi, gövdeyi bir `poll` fonksiyonuna çevirir.[^kod-state] Üretilen yapı yakalanan değişkenleri, bir durum alanını ve bir bekleme noktasını geçen yerel değişkenleri taşır. Üç durum sabittir: 0 hiç başlamamış, 1 bitmiş, 2 zehirlenmiş (gövde paniklemiş) makinedir; ötekiler tek tek `.await` noktalarıdır. Bitmiş makine yoklanınca gördüğümüz `` `async fn` resumed after completion `` paniği buradan gelir. Düşürme fonksiyonu da aynı durum alanına bakar; iptal sırasında doğru değişkenlerin düşürülmesi bundandır. Hangi yerel değişkenlerin saklanacağını `locals_live_across_suspend_points` hesaplar.[^kod-layout]

**Sabitleme.** `Pin<Ptr>` tek alanlı bir sarmalayıcıdır;[^kod-pin] kısıt, hangi metodun hangi şartla güvenli olduğundan doğar. `Pin::new` ve `Pin::get_mut` yalnızca `Unpin` türler için güvenlidir, `new_unchecked` ve `get_unchecked_mut` ise `unsafe`'tir. `Unpin` bir otomatik trait'tir: Derleyici onu alan türlerinden çıkarır; `PhantomPinned` ise `impl !Unpin` ile bu çıkarımı bilerek durdurur.[^kod-unpin]

## Özet

- Future pasiftir: Oluşturmak başlatmaz, yoklamak ilerletir. `poll`, ilerleyebildiği kadar ilerler ve `Ready` ya da `Pending` döndürür.
- `Pending` dönmeden önce uyandırıcı, olayı bilecek tarafa bırakılır. Geçerli olan, en son yoklamanın uyandırıcısıdır; meşgul bekleme yapılmaz.
- `Waker`, türü silinmiş bir uyandırma yeteneğidir: veri işaretçisi ve dört fonksiyonluk bir tablo. `std::task::Wake` ile güvenli biçimde üretilir.
- Bir future bir kez tamamlanır; bitmiş future yeniden yoklanmaz, düşürülen future iptal edilmiştir.
- `async fn` bir durum makinesidir; `.await`'i geçen her şeyi içinde taşır.
- `Pin`, sabitlenen değerin düşürülene kadar yerinden oynamayacağını garanti eder. `Unpin` türlerde kilit boştur; `async` future'lar `!Unpin`'dir.

Sıradaki bölüm [29. İleri Eşzamanlılık ve Async](29-ileri-eszamanlilik-async.md), bu parçalarla bir thread havuzu, görev kuyruklu bir çalışma zamanı ve asenkron bir istemci kuruyor.

## Kaynaklar

[^future]: std, [`Future::poll`](https://doc.rust-lang.org/std/future/trait.Future.html#tymethod.poll) ve [The poll method](https://doc.rust-lang.org/std/future/trait.Future.html#the-poll-method); The Rust Programming Language, [17.5 The Future Trait](https://doc.rust-lang.org/book/ch17-05-traits-for-async.html#the-future-trait).
[^poll-enum]: std, [`Poll`](https://doc.rust-lang.org/std/task/enum.Poll.html).
[^context]: std, [`Context`](https://doc.rust-lang.org/std/task/struct.Context.html).
[^noop]: std, [`Waker::noop`](https://doc.rust-lang.org/std/task/struct.Waker.html#method.noop).
[^tembel]: std, [`Future::poll`: Runtime characteristics](https://doc.rust-lang.org/std/future/trait.Future.html#runtime-characteristics).
[^donus]: std, [`Future::poll`: Return value](https://doc.rust-lang.org/std/future/trait.Future.html#return-value).
[^park]: std, [`thread::park`: park and unpark](https://doc.rust-lang.org/std/thread/fn.park.html#park-and-unpark) ve [`Wake`: Examples](https://doc.rust-lang.org/std/task/trait.Wake.html#examples).
[^yurutucu]: The Rust Programming Language, [Executing an Async Function with a Runtime](https://doc.rust-lang.org/book/ch17-01-futures-and-syntax.html#executing-an-async-function-with-a-runtime).
[^async-std]: RustSec, [async-std has been discontinued (2025-0052)](https://rustsec.org/advisories/RUSTSEC-2025-0052.html).
[^rawwaker]: std, [`RawWaker`](https://doc.rust-lang.org/std/task/struct.RawWaker.html) ve [`RawWakerVTable`](https://doc.rust-lang.org/std/task/struct.RawWakerVTable.html). Jon Gjengset, *Rust for Rustaceans*, Böl. 8.
[^wake-trait]: std, [`Wake`](https://doc.rust-lang.org/std/task/trait.Wake.html).
[^will-wake]: std, [`Waker::will_wake`](https://doc.rust-lang.org/std/task/struct.Waker.html#method.will_wake).
[^waker]: std, [`Waker`](https://doc.rust-lang.org/std/task/struct.Waker.html).
[^wake-metodu]: std, [`Waker::wake`](https://doc.rust-lang.org/std/task/struct.Waker.html#method.wake) ve [`Waker::wake_by_ref`](https://doc.rust-lang.org/std/task/struct.Waker.html#method.wake_by_ref).
[^panik]: std, [`Future::poll`: Panics](https://doc.rust-lang.org/std/future/trait.Future.html#panics).
[^makine]: rustc 1.99.0 kaynak kodu, [`coroutine/mod.rs` başlık belgesi, `mir_locals`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_mir_transform/src/coroutine/mod.rs#L25). Jon Gjengset, *Rust for Rustaceans*, Böl. 8.
[^async-blok]: The Rust Reference, [Async blocks](https://doc.rust-lang.org/reference/expressions/block-expr.html#r-expr.block.async.layout-unspecified).
[^box-pin]: std, [`Box::pin`](https://doc.rust-lang.org/std/boxed/struct.Box.html#method.pin).
[^pin-oz]: std, [`std::pin`: A self-referential struct](https://doc.rust-lang.org/std/pin/index.html#a-self-referential-struct); The Rust Programming Language, [17.5 The Pin Type and the Unpin Trait](https://doc.rust-lang.org/book/ch17-05-traits-for-async.html#the-pin-type-and-the-unpin-trait). Jon Gjengset, *Rust for Rustaceans*, Böl. 8.
[^pin-sozlesme]: std, [`std::pin`: Subtle details and the Drop guarantee](https://doc.rust-lang.org/std/pin/index.html#subtle-details-and-the-drop-guarantee).
[^unpin]: std, [`std::pin`: Unpin](https://doc.rust-lang.org/std/pin/index.html#unpin) ve [`PhantomPinned`](https://doc.rust-lang.org/std/marker/struct.PhantomPinned.html).
[^pin-makro]: std, [`pin!`: Remarks](https://doc.rust-lang.org/std/pin/macro.pin.html#remarks).
[^kod-future]: std 1.99.0 kaynak kodu, [`future.rs`, `pub trait Future`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/future/future.rs#L37) ve [`future.rs`, `fn poll`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/future/future.rs#L113).
[^kod-poll]: std 1.99.0 kaynak kodu, [`poll.rs`, `pub enum Poll`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/task/poll.rs#L14).
[^kod-waker]: std 1.99.0 kaynak kodu, [`wake.rs`, `pub struct Waker`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/task/wake.rs#L407), [`wake.rs`, `pub struct RawWaker`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/task/wake.rs#L22) ve [`wake.rs`, `pub fn wake`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/task/wake.rs#L439).
[^kod-vtable]: std 1.99.0 kaynak kodu, [`wake.rs`, `pub struct RawWakerVTable`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/task/wake.rs#L109).
[^kod-wake]: std 1.99.0 kaynak kodu, [`alloc/src/task.rs`, `From<Arc<W>> for Waker`](https://github.com/rust-lang/rust/blob/1.99.0/library/alloc/src/task.rs#L114) ve [`alloc/src/task.rs`, `fn raw_waker`](https://github.com/rust-lang/rust/blob/1.99.0/library/alloc/src/task.rs#L180).
[^kod-await]: rustc 1.99.0 kaynak kodu, [`expr.rs`, `lower_expr_await`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_ast_lowering/src/expr.rs#L932); The Rust Reference, [Await expressions: Approximate desugaring](https://doc.rust-lang.org/reference/expressions/await-expr.html#approximate-desugaring).
[^kod-state]: rustc 1.99.0 kaynak kodu, [`coroutine/mod.rs`, `StateTransform`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_mir_transform/src/coroutine/mod.rs#L85) ve [`coroutine/mod.rs`, `ResumedAfterReturn`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_mir_transform/src/coroutine/mod.rs#L906).
[^kod-layout]: rustc 1.99.0 kaynak kodu, [`coroutine/layout.rs`, `locals_live_across_suspend_points`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_mir_transform/src/coroutine/layout.rs#L82).
[^kod-pin]: std 1.99.0 kaynak kodu, [`pin.rs`, `pub struct Pin`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/pin.rs#L1091), [`pin.rs`, `pub const fn new`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/pin.rs#L1181) ve [`pin.rs`, `get_unchecked_mut`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/pin.rs#L1621).
[^kod-unpin]: std 1.99.0 kaynak kodu, [`marker.rs`, `pub auto trait Unpin`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/marker.rs#L1014) ve [`marker.rs`, `impl !Unpin for PhantomPinned`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/marker.rs#L1030).
