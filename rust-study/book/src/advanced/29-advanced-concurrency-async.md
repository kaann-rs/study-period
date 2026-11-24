# 29. İleri Eşzamanlılık ve Async

Önceki bölümler eşzamanlılığın parçalarını tek tek kurdu: [15. bölüm](../intermediate/15-concurrency.md) thread'leri, kanalları ve kilitleri; [16. bölüm](../intermediate/16-async-await.md) `async` ile `.await`'i; [25. bölüm](25-send-sync.md) `Send` ile `Sync`'in, [28. bölüm](28-future.md) `Future`'ın iç yüzünü anlattı. Bu bölüm o parçaları üretim kodunda karşılaşılan sorulara bağlıyor. Hangi kilidi seçmeli? Kilitlenme nasıl önlenir? Bir koşul, işlemciyi yormadan nasıl beklenir? Bir çalışma zamanı binlerce görevden yalnızca hazır olanı nasıl bulur? Yavaş bir istek nasıl kesilir, başarısız olanı nasıl yinelenir?

Bütün bölümü tek bir soru taşır: *Beklerken ne yapalım?* Thread'ler ve async görevler bu soruya iki ayrı yanıttır. Aynı anda birçok iş yürütmek için ya her işe bir işçi verirsiniz ya da az sayıda işçi, bekleyen işi bırakıp sıradakine geçer. İki durumda da ortak veriye kimin, ne zaman dokunacağını kurala bağlamak gerekir.

Bu sayfa iki yanıtı karşılaştırıyor ve bölümün haritasını çıkarıyor. Ayrıntılar üç alt sayfadadır: [29.1](29-threads.md) thread tarafını, [29.2](29-runtime.md) bir çalışma zamanının içini, [29.3](29-client.md) bunların üstüne kurulmuş bir istemciyi ve async kodun yaygın hatalarını ele alır. Sayfaları bu sırayla okuyun.

> [!NOTE]
> Bu bölümün kodu `learn/advanced-concepts/advanced-concurrency-async` dizinindedir. Depo kökünden `cargo run -p advanced-concurrency-async` komutuyla çalıştırabilir, `cargo test -p advanced-concurrency-async` ile testlerini koşturabilirsiniz. Süreler makineden makineye değiştiği için örnekler milisaniye değil, yalnızca her çalıştırmada aynı kalan karşılaştırmaları (`true`, `false`) yazdırır.

## Günlük hayattan bir benzetme

Kalabalık bir mutfak düşünün. Aşçılar thread'lerdir: Her işin başında bir aşçı durur, iş beklerken aşçı da bekler. Tek bir kasiyer ise async'tir: Sipariş beklerken boş durmaz, zili çalan masaya gider. Kasiyer birçok masayı tek başına idare edebilir, yeter ki bir masanın başında dikilip kalmasın.

Aşçılar aynı tencereye aynı anda el atmasın diye kilit koyarsınız ve kilitleri herkes hep aynı sırayla alır. Sıra kuralı olmasaydı iki aşçı birbirinin kilidini bekler, yemek hiç pişmezdi. "Hamur kabardı mı?" diye dakikada bir bakmak yerine zil kurarsınız: Koşul gerçekleşince zil çalar. Tek bir sayıyı, örneğin kalan tabak sayısını tutmak için kilit kurmaya da gerek yoktur; atomik bir sayaç yeter.

<details>
<summary>Bu bölümde geçen terimler</summary>

- **eşzamanlılık (concurrency):** Birden çok işin aynı zaman diliminde ilerlemesi.
- **thread (iş parçacığı):** İşletim sisteminin yönettiği ayrı bir yürütme akışı. Kendi stack'i vardır, heap'i öteki thread'lerle paylaşır.
- **bağlam değişimi (context switch):** İşlemcinin bir thread'i bırakıp ötekine geçmesi. Bedava değildir.
- **stack:** Her thread'in kendi çalışma alanı; yerel değişkenler orada durur.
- **heap:** Paylaşılan büyük bellek deposu; `Box` ve `Arc` değerlerini oraya koyar.
- **veri yarışı (data race):** İki thread'in aynı veriye, aralarında eşzamanlama (synchronization) olmadan erişmesi ve en az birinin yazması.
- **kilit (lock), `Mutex`:** İçeriğe aynı anda yalnızca bir thread'in girmesini sağlayan kapı.
- **koruyucu (guard):** Kilidi aldığınızda elinize geçen değer; düşürülünce kilit açılır.
- **RAII:** Bir kaynağın, onu tutan değer düşerken kendiliğinden bırakılması düzeni. Koruyucu böyledir.
- **kritik bölge (critical section):** Kilidin tutulduğu kod parçası.
- **kilitlenme (deadlock):** İki tarafın sonsuza dek birbirini beklemesi. Rust bunu önlemez.
- **atomik (atomic):** Yarıda kesilemeyen tek hamleyle okunup yazılan küçük değer.
- **`Arc`:** Thread'ler arasında paylaşılan, sayaçlı sahiplik işaretçisi.
- **kanal (channel):** Bir ucundan gönderilen değerin öteki ucundan alındığı boru.
- **`Send`, `Sync`:** "Başka bir thread'e verilebilir" ve "thread'ler birlikte bakabilir" damgaları.
- **auto trait:** Derleyicinin, türün alanlarına bakıp kendiliğinden uyguladığı trait.
- **`async`, `.await`:** "Bu iş bekleyebilir" diye yazılan kod ve bekleme noktası.
- **future:** Henüz bitmemiş iş. Yoklanınca ilerler, kendiliğinden çalışmaz.
- **yoklama (poll), `Pending`, `Ready`:** "Biraz ilerle" demek; yanıtı "daha bitmedi" ya da "bitti, sonuç bu" olur.
- **görev (task):** Yürütücüye teslim edilmiş, bağımsız yürüyen future.
- **yürütücü (executor):** Görevleri yoklayan program parçası. Yazacağımız yürütücünün adı `Calistirici` olduğu için ona *çalıştırıcı* da diyeceğiz.
- **çalışma zamanı (async runtime):** Yürütücünün görev kuyruğu, zamanlayıcı ve G/Ç gibi parçalarla birlikte bütünü.
- **uyandırıcı (waker):** "Ben hazırım, bana yine bak" zili.
- **başlatmak (spawn):** Yeni bir thread ya da görev başlatmak.
- **tutamaç (`JoinHandle`):** Başlattığınız işin sonucunu sonradan almaya yarayan fiş.
- **zamanlayıcı (timer):** Süre dolunca haber veren parça.
- **zaman aşımı (timeout):** "Şu süreden uzun sürerse bırak" kuralı.
- **bloklamak:** Bir thread'i, iş bitene kadar başka hiçbir şey yapamaz hâlde tutmak.
- **iptal (cancellation):** Bir future'ı bitmeden düşürmek.
- **durum makinesi (state machine):** "Hangi adımdayım?" bilgisini saklayıp kaldığı yerden süren yapı.
- **`Pin`, `Unpin`:** "Gösterdiğim değer yerinden oynamayacak" sözü ve bu söze gerek duymayan türlerin işareti.
- **G/Ç (I/O):** Dış dünyayla alışveriş: dosya, ağ, klavye.

</details>

## Beklerken ne yapalım? İki yanıt

Thread tarafında beklemeyi işletim sistemi yönetir: Bekleyen thread'i uyutur, çekirdeği başkasına verir. Yazması kolaydır; ama bekleyen her iş, kendi stack'i olan bir thread demektir. `std` ile açılan bir thread'in varsayılan stack boyutu şu an 1. kademe (Tier 1) platformlarda 2 MiB'tır.[^yigin] Thread'ler arasında geçmenin, yani bağlam değişiminin de bir bedeli vardır.

Async tarafında görev "Şimdilik bu kadar" (`Pending`) deyip thread'i *geri verir*; çalıştırıcı aynı thread'de başka göreve geçer. Böylece on binlerce bekleyen iş birkaç thread ile yürüyebilir. Karşılığında iş birliği gerekir: Thread'i bırakmayan, yani bloklayan tek bir görev, o thread'deki bütün görevleri durdurur.

Aynı üç bekleyişi iki yolla yürütüp kaç thread harcandığını sayalım. Çalışma zamanı olarak 29.2'de yazacağımız küçük sürümü kullanıyoruz:

```rust
# mod calisma_zamani {
#     use std::future::Future;
#     use std::pin::Pin;
#     use std::sync::mpsc::Sender;
#     use std::sync::{Arc, Mutex};
#     use std::task::{Wake, Waker};
#     type KutuluGorev = Pin<Box<dyn Future<Output = ()> + Send>>;
#     struct Gorev { gelecek: Mutex<Option<KutuluGorev>>, kuyruk: Sender<Arc<Gorev>> }
#     impl Wake for Gorev {
#         fn wake(self: Arc<Self>) { let _ = self.kuyruk.clone().send(self); }
#     }
#     use std::sync::atomic::{AtomicUsize, Ordering};
#     use std::sync::mpsc::{self, Receiver};
#     pub struct Calistirici { hazir: Receiver<Arc<Gorev>>, baslatici: Baslatici, poll_sayisi: usize }
#     #[derive(Clone)]
#     pub struct Baslatici { kuyruk: Sender<Arc<Gorev>>, canli: Arc<AtomicUsize> }
#     impl Baslatici {
#         fn kuyruga_koy(&self, gelecek: impl Future<Output = ()> + Send + 'static) {
#             self.canli.fetch_add(1, Ordering::SeqCst);
#             let gelecek = Mutex::new(Some(Box::pin(gelecek) as KutuluGorev));
#             let _ = self.kuyruk.send(Arc::new(Gorev { gelecek, kuyruk: self.kuyruk.clone() }));
#         }
#     }
#     impl Calistirici {
#         pub fn yeni() -> Self {
#             let (kuyruk, hazir) = mpsc::channel();
#             let baslatici = Baslatici { kuyruk, canli: Arc::new(AtomicUsize::new(0)) };
#             Calistirici { hazir, baslatici, poll_sayisi: 0 }
#         }
#         pub fn baslatici(&self) -> Baslatici { self.baslatici.clone() }
#     }
#     use std::task::{Context, Poll};
#     impl Calistirici {
#         pub fn calistir(&mut self) -> usize {
#             while self.baslatici.canli.load(Ordering::SeqCst) > 0 {
#                 let Ok(gorev) = self.hazir.recv() else { break };
#                 let mut yuva = gorev.gelecek.lock().expect("kilit zehirlenmemeli");
#                 let Some(mut gelecek) = yuva.take() else { continue };
#                 let waker = Waker::from(Arc::clone(&gorev));
#                 self.poll_sayisi += 1;
#                 match gelecek.as_mut().poll(&mut Context::from_waker(&waker)) {
#                     Poll::Pending => *yuva = Some(gelecek),
#                     Poll::Ready(()) => { self.baslatici.canli.fetch_sub(1, Ordering::SeqCst); }
#                 }
#             }
#             self.poll_sayisi
#         }
#     }
#     struct TutamacDurumu<T> { sonuc: Option<T>, bekleyen: Option<Waker> }
#     pub struct Tutamac<T> { paylasilan: Arc<Mutex<TutamacDurumu<T>>> }
#     impl<T> Tutamac<T> {
#         pub fn sonucu_al(&self) -> Option<T> {
#             self.paylasilan.lock().expect("kilit zehirlenmemeli").sonuc.take()
#         }
#     }
#     impl<T> Future for Tutamac<T> {
#         type Output = T;
#         fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
#             let mut durum = self.paylasilan.lock().expect("kilit zehirlenmemeli");
#             if let Some(deger) = durum.sonuc.take() { return Poll::Ready(deger); }
#             durum.bekleyen = Some(cx.waker().clone());
#             Poll::Pending
#         }
#     }
#     impl Baslatici {
#         pub fn baslat<F>(&self, gelecek: F) -> Tutamac<F::Output>
#         where F: Future + Send + 'static, F::Output: Send + 'static {
#             let paylasilan = Arc::new(Mutex::new(TutamacDurumu { sonuc: None, bekleyen: None }));
#             let gorev_tarafi = Arc::clone(&paylasilan);
#             self.kuyruga_koy(async move {
#                 let deger = gelecek.await;
#                 let bekleyen = {
#                     let mut durum = gorev_tarafi.lock().expect("kilit zehirlenmemeli");
#                     durum.sonuc = Some(deger);
#                     durum.bekleyen.take()
#                 };
#                 if let Some(waker) = bekleyen { waker.wake(); }
#             });
#             Tutamac { paylasilan }
#         }
#     }
#     pub fn kostur<F>(gelecek: F) -> F::Output
#     where F: Future + Send + 'static, F::Output: Send + 'static {
#         let mut calistirici = Calistirici::yeni();
#         let tutamac = calistirici.baslatici().baslat(gelecek);
#         calistirici.calistir();
#         tutamac.sonucu_al().expect("görev bitti ama sonuç yok")
#     }
#     use std::time::Duration;
#     pub struct Uyku { sure: Duration, basladi: bool, durum: Arc<Mutex<UykuDurumu>> }
#     struct UykuDurumu { doldu: bool, waker: Option<Waker> }
#     impl Uyku {
#         pub fn yeni(sure: Duration) -> Self {
#             let durum = Arc::new(Mutex::new(UykuDurumu { doldu: false, waker: None }));
#             Uyku { sure, basladi: false, durum }
#         }
#     }
#     pub fn uyu_ms(ms: u64) -> Uyku { Uyku::yeni(Duration::from_millis(ms)) }
#     impl Future for Uyku {
#         type Output = ();
#         fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
#             let bu = self.get_mut();
#             let mut durum = bu.durum.lock().expect("kilit zehirlenmemeli");
#             if durum.doldu { return Poll::Ready(()); }
#             durum.waker = Some(cx.waker().clone());
#             drop(durum);
#             if std::mem::replace(&mut bu.basladi, true) { return Poll::Pending; }
#             let (paylasilan, sure) = (Arc::clone(&bu.durum), bu.sure);
#             std::thread::spawn(move || {
#                 std::thread::sleep(sure);
#                 let mut durum = paylasilan.lock().expect("kilit zehirlenmemeli");
#                 durum.doldu = true;
#                 let waker = durum.waker.take();
#                 drop(durum);
#                 if let Some(waker) = waker { waker.wake(); }
#             });
#             Poll::Pending
#         }
#     }
# }
use calisma_zamani::{Calistirici, Tutamac, uyu_ms};
use std::collections::HashSet;
use std::thread;
use std::time::Duration;

fn threadlerle() -> usize {
    let isciler = [(); 3].map(|()| {
        thread::spawn(|| {
            thread::sleep(Duration::from_millis(20));
            thread::current().id()
        })
    });
    let kimlikler = isciler.map(|i| i.join().expect("işçi paniklememeli"));
    HashSet::from(kimlikler).len()
}

fn gorevlerle() -> usize {
    let mut calistirici = Calistirici::yeni();
    let baslatici = calistirici.baslatici();
    let tutamaclar = [(); 3].map(|()| {
        baslatici.baslat(async {
            uyu_ms(20).await;
            thread::current().id()
        })
    });
    calistirici.calistir();
    tutamaclar.iter().filter_map(Tutamac::sonucu_al).collect::<HashSet<_>>().len()
}

fn main() {
    println!("thread'ler: 3 iş, {} thread", threadlerle());
    println!("görevler:   3 iş, {} thread", gorevlerle());

    let gorev = async {
        uyu_ms(20).await;
        thread::current().id()
    };
    println!("bir görev 1 KiB'tan küçük mü? {}", size_of_val(&gorev) < 1024);
}
```

Çıktı:

```text
thread'ler: 3 iş, 3 thread
görevler:   3 iş, 1 thread
bir görev 1 KiB'tan küçük mü? true
```

Üç görevin kodu da çalıştırıcının tek thread'inde yürüdü. Bekleyen bir görevin kapladığı yer, bir stack değil, kendi future'ının boyutu kadardır.

> [!NOTE]
> Sayım, görevlerin *kodunu* çalıştıran thread'leri gösterir. Küçük çalışma zamanımız her uyku için ayrıca bir yardımcı thread açar; bunun neden yalnızca öğretici bir kısayol olduğunu ve üretimde yerini neyin aldığını 29.2'de göreceğiz.

| | Thread | Async görev |
| --- | --- | --- |
| Bekleyişi kim yönetir? | İşletim sistemi | Çalışma zamanı, yani kütüphane kodu |
| Bekleyen işin bedeli | Bir stack ve bağlam değişimi | Future'ın boyutu kadar bellek |
| İş ne zaman kesilir? | İşletim sistemi istediği anda | Yalnızca `.await` noktalarında |
| Tehlikesi | Çok sayıda thread pahalıdır | Bloklayan tek görev ötekileri durdurur |
| Nerede iyidir? | İşlemciyi yoran, bölünebilen işler | Çok sayıda bekleyiş: ağ, dosya, zamanlayıcı |

Son satır, resmî kitabın da önerdiği ayrımdır. İkisini birlikte kullanmak da olağandır: Birçok çalışma zamanı görevleri bir thread havuzunda yürütür, ağır hesabı da ayrı thread'lere gönderir.[^secim]

## Önceki bölümlerin üstüne ne koyuyoruz?

Bu bölüm sahipliği ([3](../basics/03-ownership.md)), `Arc`'ı ([14](../intermediate/14-smart-pointers.md)) ve aşağıdaki bölümleri bildiğinizi varsayar:

| Bölüm | Orada kurulan | Burada eklenen |
| --- | --- | --- |
| [15](../intermediate/15-concurrency.md), [15.1](../intermediate/15-concurrency-deep-dive.md) | Thread, kanal, `Arc<Mutex<T>>`, `thread::scope`, işçi havuzu | `RwLock`, `Condvar`, kilit sıralaması, `compare_exchange` |
| [16](../intermediate/16-async-await.md), [16.1](../intermediate/16-async-await-deep-dive.md) | `Future`, elle yazılmış yürütücü, `join`, `Pin`'in gündelik yüzü | Görev kuyruğu, `spawn`, zamanlayıcı, zaman aşımı, geri basınç |
| [25](25-send-sync.md) | `UnsafeCell`, auto trait mekanizması | "Bu türü paylaşabilir miyim?" tablosu |
| [28](28-future.md) | Yoklama ve uyandırma sözleşmesi, `Waker`'ın iç yapısı, kendine referans ve `Pin` | Kuyruğa geri koyan gerçek bir `Waker`, elle yazılmış bir birleştirici |

Üç katmanın her biri kendi başına okunabilir:

- **29.1, thread tarafı:** Doğru kilidi seçmek, koşul için zil beklemek, kilitlenmeyi tasarımla önlemek, tek bir sayı için atomik kullanmak.
- **29.2, async tarafı:** Bir çalışma zamanı gerçekte ne yapar? Kuyruk, `baslat`, tutamaç, zamanlayıcı, zaman aşımı, geri basınç.
- **29.3, uygulama:** Eşzamanlı istek, zaman aşımı, yeniden deneme ve async kodun yaygın hataları.

## Çalışma zamanına ilk bakış

İki arkadaşınıza aynı anda iş verdiniz: Biri 30 dakikada, öteki 10 dakikada bitirecek. İkisi de hemen işe koyulur. Siz önce yavaş olanın kapısında beklersiniz; o bittiğinde hızlı olan çoktan bitmiştir ve sorar sormaz yanıt verir. Toplam bekleyiş 40 değil, yaklaşık 30 dakikadır.

Aşağıdaki kod bunu yapar. `baslat` işi hemen kuyruğa koyar, `.await` yalnızca sonucu bekler:

```rust
# mod calisma_zamani {
#     use std::future::Future;
#     use std::pin::Pin;
#     use std::sync::mpsc::Sender;
#     use std::sync::{Arc, Mutex};
#     use std::task::{Wake, Waker};
#     type KutuluGorev = Pin<Box<dyn Future<Output = ()> + Send>>;
#     struct Gorev { gelecek: Mutex<Option<KutuluGorev>>, kuyruk: Sender<Arc<Gorev>> }
#     impl Wake for Gorev {
#         fn wake(self: Arc<Self>) { let _ = self.kuyruk.clone().send(self); }
#     }
#     use std::sync::atomic::{AtomicUsize, Ordering};
#     use std::sync::mpsc::{self, Receiver};
#     pub struct Calistirici { hazir: Receiver<Arc<Gorev>>, baslatici: Baslatici, poll_sayisi: usize }
#     #[derive(Clone)]
#     pub struct Baslatici { kuyruk: Sender<Arc<Gorev>>, canli: Arc<AtomicUsize> }
#     impl Baslatici {
#         fn kuyruga_koy(&self, gelecek: impl Future<Output = ()> + Send + 'static) {
#             self.canli.fetch_add(1, Ordering::SeqCst);
#             let gelecek = Mutex::new(Some(Box::pin(gelecek) as KutuluGorev));
#             let _ = self.kuyruk.send(Arc::new(Gorev { gelecek, kuyruk: self.kuyruk.clone() }));
#         }
#     }
#     impl Calistirici {
#         pub fn yeni() -> Self {
#             let (kuyruk, hazir) = mpsc::channel();
#             let baslatici = Baslatici { kuyruk, canli: Arc::new(AtomicUsize::new(0)) };
#             Calistirici { hazir, baslatici, poll_sayisi: 0 }
#         }
#         pub fn baslatici(&self) -> Baslatici { self.baslatici.clone() }
#     }
#     use std::task::{Context, Poll};
#     impl Calistirici {
#         pub fn calistir(&mut self) -> usize {
#             while self.baslatici.canli.load(Ordering::SeqCst) > 0 {
#                 let Ok(gorev) = self.hazir.recv() else { break };
#                 let mut yuva = gorev.gelecek.lock().expect("kilit zehirlenmemeli");
#                 let Some(mut gelecek) = yuva.take() else { continue };
#                 let waker = Waker::from(Arc::clone(&gorev));
#                 self.poll_sayisi += 1;
#                 match gelecek.as_mut().poll(&mut Context::from_waker(&waker)) {
#                     Poll::Pending => *yuva = Some(gelecek),
#                     Poll::Ready(()) => { self.baslatici.canli.fetch_sub(1, Ordering::SeqCst); }
#                 }
#             }
#             self.poll_sayisi
#         }
#     }
#     struct TutamacDurumu<T> { sonuc: Option<T>, bekleyen: Option<Waker> }
#     pub struct Tutamac<T> { paylasilan: Arc<Mutex<TutamacDurumu<T>>> }
#     impl<T> Tutamac<T> {
#         pub fn sonucu_al(&self) -> Option<T> {
#             self.paylasilan.lock().expect("kilit zehirlenmemeli").sonuc.take()
#         }
#     }
#     impl<T> Future for Tutamac<T> {
#         type Output = T;
#         fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
#             let mut durum = self.paylasilan.lock().expect("kilit zehirlenmemeli");
#             if let Some(deger) = durum.sonuc.take() { return Poll::Ready(deger); }
#             durum.bekleyen = Some(cx.waker().clone());
#             Poll::Pending
#         }
#     }
#     impl Baslatici {
#         pub fn baslat<F>(&self, gelecek: F) -> Tutamac<F::Output>
#         where F: Future + Send + 'static, F::Output: Send + 'static {
#             let paylasilan = Arc::new(Mutex::new(TutamacDurumu { sonuc: None, bekleyen: None }));
#             let gorev_tarafi = Arc::clone(&paylasilan);
#             self.kuyruga_koy(async move {
#                 let deger = gelecek.await;
#                 let bekleyen = {
#                     let mut durum = gorev_tarafi.lock().expect("kilit zehirlenmemeli");
#                     durum.sonuc = Some(deger);
#                     durum.bekleyen.take()
#                 };
#                 if let Some(waker) = bekleyen { waker.wake(); }
#             });
#             Tutamac { paylasilan }
#         }
#     }
#     pub fn kostur<F>(gelecek: F) -> F::Output
#     where F: Future + Send + 'static, F::Output: Send + 'static {
#         let mut calistirici = Calistirici::yeni();
#         let tutamac = calistirici.baslatici().baslat(gelecek);
#         calistirici.calistir();
#         tutamac.sonucu_al().expect("görev bitti ama sonuç yok")
#     }
#     use std::time::Duration;
#     pub struct Uyku { sure: Duration, basladi: bool, durum: Arc<Mutex<UykuDurumu>> }
#     struct UykuDurumu { doldu: bool, waker: Option<Waker> }
#     impl Uyku {
#         pub fn yeni(sure: Duration) -> Self {
#             let durum = Arc::new(Mutex::new(UykuDurumu { doldu: false, waker: None }));
#             Uyku { sure, basladi: false, durum }
#         }
#     }
#     pub fn uyu_ms(ms: u64) -> Uyku { Uyku::yeni(Duration::from_millis(ms)) }
#     impl Future for Uyku {
#         type Output = ();
#         fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
#             let bu = self.get_mut();
#             let mut durum = bu.durum.lock().expect("kilit zehirlenmemeli");
#             if durum.doldu { return Poll::Ready(()); }
#             durum.waker = Some(cx.waker().clone());
#             drop(durum);
#             if std::mem::replace(&mut bu.basladi, true) { return Poll::Pending; }
#             let (paylasilan, sure) = (Arc::clone(&bu.durum), bu.sure);
#             std::thread::spawn(move || {
#                 std::thread::sleep(sure);
#                 let mut durum = paylasilan.lock().expect("kilit zehirlenmemeli");
#                 durum.doldu = true;
#                 let waker = durum.waker.take();
#                 drop(durum);
#                 if let Some(waker) = waker { waker.wake(); }
#             });
#             Poll::Pending
#         }
#     }
# }
use calisma_zamani::{Calistirici, uyu_ms};

fn calisma_zamani_tanitimi() {
    let mut calistirici = Calistirici::yeni();
    let baslatici = calistirici.baslatici();
    let ic_baslatici = baslatici.clone();

    // Kök görev iki alt görev başlatır ve sonuçlarını bekler.
    let kok = baslatici.baslat(async move {
        let yavas = ic_baslatici.baslat(async {
            uyu_ms(30).await;
            "yavaş görev (30 ms)"
        });
        let hizli = ic_baslatici.baslat(async {
            uyu_ms(10).await;
            "hızlı görev (10 ms)"
        });
        [yavas.await, hizli.await]
    });

    let poll_sayisi = calistirici.calistir();
    println!("sonuçlar = {:?}", kok.sonucu_al().expect("kök görev bitti"));
    println!("toplam yoklama = {poll_sayisi}");
}

fn main() {
    calisma_zamani_tanitimi();
}
```

Çıktı:

```text
sonuçlar = ["yavaş görev (30 ms)", "hızlı görev (10 ms)"]
toplam yoklama = 6
```

Bekleme sırası sonuçların sırasını belirler; görevler ise eşzamanlı ilerler.[^spawn] `yavas.await` beklerken `hizli` çoktan bitmiş olur ve ikinci `.await` hemen döner.

Yoklama sayısı çalışma zamanının nasıl davrandığını ele verir:

| Görev | Yoklama | Neden? |
| --- | --- | --- |
| kök | 2 | İlk yoklama ve `yavas` bitince gelen uyandırma |
| `yavas` | 2 | İlk yoklama ve uykusu dolunca gelen uyandırma |
| `hizli` | 2 | İlk yoklama ve uykusu dolunca gelen uyandırma |

`hizli` bittiğinde kökü uyandırmaz, çünkü kök o sırada `hizli`'nin değil `yavas`'ın tutamacına uyandırıcı bırakmıştır. Başlarkenki ilk yoklamaların dışında yalnızca *uyandırılan* görev yoklanır; çalıştırıcının thread'i geri kalan sürenin neredeyse tamamını kanalın `recv` çağrısında uyuyarak geçirir. Görevlere durmadan "Bitti mi?" diye soran bir yürütücü aynı 30 ms'de binlerce yoklama yapardı.

## Paylaşılan duruma erişim kuralları aynıdır

Async, paylaşılan veri için yeni bir güvenlik düzeni getirmez. `Send`, `Sync`, kilit ve atomik kuralları görevler için de geçerlidir.[^paylasim] Aynı `artir` fonksiyonunu thread'lerden de görevlerden de çağırabiliriz:

```rust
# mod calisma_zamani {
#     use std::future::Future;
#     use std::pin::Pin;
#     use std::sync::mpsc::Sender;
#     use std::sync::{Arc, Mutex};
#     use std::task::{Wake, Waker};
#     type KutuluGorev = Pin<Box<dyn Future<Output = ()> + Send>>;
#     struct Gorev { gelecek: Mutex<Option<KutuluGorev>>, kuyruk: Sender<Arc<Gorev>> }
#     impl Wake for Gorev {
#         fn wake(self: Arc<Self>) { let _ = self.kuyruk.clone().send(self); }
#     }
#     use std::sync::atomic::{AtomicUsize, Ordering};
#     use std::sync::mpsc::{self, Receiver};
#     pub struct Calistirici { hazir: Receiver<Arc<Gorev>>, baslatici: Baslatici, poll_sayisi: usize }
#     #[derive(Clone)]
#     pub struct Baslatici { kuyruk: Sender<Arc<Gorev>>, canli: Arc<AtomicUsize> }
#     impl Baslatici {
#         fn kuyruga_koy(&self, gelecek: impl Future<Output = ()> + Send + 'static) {
#             self.canli.fetch_add(1, Ordering::SeqCst);
#             let gelecek = Mutex::new(Some(Box::pin(gelecek) as KutuluGorev));
#             let _ = self.kuyruk.send(Arc::new(Gorev { gelecek, kuyruk: self.kuyruk.clone() }));
#         }
#     }
#     impl Calistirici {
#         pub fn yeni() -> Self {
#             let (kuyruk, hazir) = mpsc::channel();
#             let baslatici = Baslatici { kuyruk, canli: Arc::new(AtomicUsize::new(0)) };
#             Calistirici { hazir, baslatici, poll_sayisi: 0 }
#         }
#         pub fn baslatici(&self) -> Baslatici { self.baslatici.clone() }
#     }
#     use std::task::{Context, Poll};
#     impl Calistirici {
#         pub fn calistir(&mut self) -> usize {
#             while self.baslatici.canli.load(Ordering::SeqCst) > 0 {
#                 let Ok(gorev) = self.hazir.recv() else { break };
#                 let mut yuva = gorev.gelecek.lock().expect("kilit zehirlenmemeli");
#                 let Some(mut gelecek) = yuva.take() else { continue };
#                 let waker = Waker::from(Arc::clone(&gorev));
#                 self.poll_sayisi += 1;
#                 match gelecek.as_mut().poll(&mut Context::from_waker(&waker)) {
#                     Poll::Pending => *yuva = Some(gelecek),
#                     Poll::Ready(()) => { self.baslatici.canli.fetch_sub(1, Ordering::SeqCst); }
#                 }
#             }
#             self.poll_sayisi
#         }
#     }
#     struct TutamacDurumu<T> { sonuc: Option<T>, bekleyen: Option<Waker> }
#     pub struct Tutamac<T> { paylasilan: Arc<Mutex<TutamacDurumu<T>>> }
#     impl<T> Tutamac<T> {
#         pub fn sonucu_al(&self) -> Option<T> {
#             self.paylasilan.lock().expect("kilit zehirlenmemeli").sonuc.take()
#         }
#     }
#     impl<T> Future for Tutamac<T> {
#         type Output = T;
#         fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
#             let mut durum = self.paylasilan.lock().expect("kilit zehirlenmemeli");
#             if let Some(deger) = durum.sonuc.take() { return Poll::Ready(deger); }
#             durum.bekleyen = Some(cx.waker().clone());
#             Poll::Pending
#         }
#     }
#     impl Baslatici {
#         pub fn baslat<F>(&self, gelecek: F) -> Tutamac<F::Output>
#         where F: Future + Send + 'static, F::Output: Send + 'static {
#             let paylasilan = Arc::new(Mutex::new(TutamacDurumu { sonuc: None, bekleyen: None }));
#             let gorev_tarafi = Arc::clone(&paylasilan);
#             self.kuyruga_koy(async move {
#                 let deger = gelecek.await;
#                 let bekleyen = {
#                     let mut durum = gorev_tarafi.lock().expect("kilit zehirlenmemeli");
#                     durum.sonuc = Some(deger);
#                     durum.bekleyen.take()
#                 };
#                 if let Some(waker) = bekleyen { waker.wake(); }
#             });
#             Tutamac { paylasilan }
#         }
#     }
#     pub fn kostur<F>(gelecek: F) -> F::Output
#     where F: Future + Send + 'static, F::Output: Send + 'static {
#         let mut calistirici = Calistirici::yeni();
#         let tutamac = calistirici.baslatici().baslat(gelecek);
#         calistirici.calistir();
#         tutamac.sonucu_al().expect("görev bitti ama sonuç yok")
#     }
#     use std::time::Duration;
#     pub struct Uyku { sure: Duration, basladi: bool, durum: Arc<Mutex<UykuDurumu>> }
#     struct UykuDurumu { doldu: bool, waker: Option<Waker> }
#     impl Uyku {
#         pub fn yeni(sure: Duration) -> Self {
#             let durum = Arc::new(Mutex::new(UykuDurumu { doldu: false, waker: None }));
#             Uyku { sure, basladi: false, durum }
#         }
#     }
#     pub fn uyu_ms(ms: u64) -> Uyku { Uyku::yeni(Duration::from_millis(ms)) }
#     impl Future for Uyku {
#         type Output = ();
#         fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
#             let bu = self.get_mut();
#             let mut durum = bu.durum.lock().expect("kilit zehirlenmemeli");
#             if durum.doldu { return Poll::Ready(()); }
#             durum.waker = Some(cx.waker().clone());
#             drop(durum);
#             if std::mem::replace(&mut bu.basladi, true) { return Poll::Pending; }
#             let (paylasilan, sure) = (Arc::clone(&bu.durum), bu.sure);
#             std::thread::spawn(move || {
#                 std::thread::sleep(sure);
#                 let mut durum = paylasilan.lock().expect("kilit zehirlenmemeli");
#                 durum.doldu = true;
#                 let waker = durum.waker.take();
#                 drop(durum);
#                 if let Some(waker) = waker { waker.wake(); }
#             });
#             Poll::Pending
#         }
#     }
# }
use calisma_zamani::{Calistirici, uyu_ms};
use std::sync::{Arc, Mutex};
use std::thread;

fn artir(sayac: &Mutex<u32>) {
    *sayac.lock().expect("kilit zehirlenmemeli") += 1;
} // koruyucu deyimin sonunda düşer

fn main() {
    let sayac = Arc::new(Mutex::new(0));

    thread::scope(|k| {
        for _ in 0..4 {
            k.spawn(|| artir(&sayac));
        }
    });
    println!("4 thread sonra: {:?}", sayac.lock().expect("kilit zehirlenmemeli"));

    let mut calistirici = Calistirici::yeni();
    for _ in 0..4 {
        let sayac = Arc::clone(&sayac);
        calistirici.baslatici().baslat(async move {
            artir(&sayac); // kilit burada alındı ve bırakıldı
            uyu_ms(1).await; // beklerken kilit elde değil
        });
    }
    calistirici.calistir();
    println!("4 görev sonra:  {:?}", sayac.lock().expect("kilit zehirlenmemeli"));
}
```

Çıktı:

```text
4 thread sonra: 4
4 görev sonra:  8
```

Async bu kurallara yalnızca bir yenisini ekler: *Kilidi `.await` boyunca tutmayın.* Nedenini, derleyicinin bunu nasıl yakaladığını ve yakalayamadığı durumları [29.3](29-client.md) anlatıyor.

## Hata kodu başvurusu

| Kod | Ne zaman görülür? | Ne yapmalı? |
| --- | --- | --- |
| `E0277` | Bir tür istenen `Send`, `Sync` ya da `Unpin` sınırını karşılamıyor (`RwLock<Cell<T>>`, `MutexGuard`, `Arc<RefCell<T>>`, kilitsiz görev, `Pin::new`). | Türü değiştirin: `Mutex`, `Arc`, `Box::pin`. ([29.1](29-threads.md), [29.2](29-runtime.md)) |
| `E0368` | Kilidin içindeki değere koruyucu olmadan dokunmaya çalışıyorsunuz. | `lock` ile koruyucuyu alın. |
| `E0373` | Görev ya da thread yerel bir değişkeni ödünç alıyor. | `async move` ile sahipliği verin ya da `Arc` klonu taşıyın. ([29.2](29-runtime.md), [29.3](29-client.md)) |
| `E0382` | Tüketilmiş bir future yeniden bekleniyor. | Her deneme için yeni future üreten bir closure alın. ([29.3](29-client.md)) |
| `E0499` | İki thread aynı değişkeni kilitsiz değiştiriyor. | `Mutex` ya da atomik kullanın. ([29.1](29-threads.md)) |
| `E0515` | Kilidin içine giden bir referans, koruyucudan uzun yaşatılmak isteniyor. | Değeri kopyalayıp döndürün. ([29.1](29-threads.md)) |
| `E0597` | Görev, kendisinden önce düşecek bir değeri ödünç alıyor. | Paylaşılan veriyi `Arc` ile verin. ([29.3](29-client.md)) |
| Kodsuz: `future cannot be sent between threads safely` | `.await` boyunca `MutexGuard`, `Rc` gibi `Send` olmayan bir değer tutuluyor. | Değerin kapsamını `.await`'ten önce bitirin. ([29.3](29-client.md)) |

## Perde arkası: Derleyici ne yapıyor?

> [!NOTE]
> Bu başlık ilk okuyuşta atlanabilir. Alt sayfalardaki örnekleri anlamak için gerekmez.

### Kilitler birer kütüphane türüdür

`Mutex`, `RwLock` ve `Condvar` standart kütüphanede yazılmış türlerdir; derleyicinin kilitlerden haberi yoktur. Güvenliği iki genel mekanizma sağlar. Birincisi, veriye yalnızca koruyucu üzerinden erişilir ve koruyucunun ömrü kilidin süresidir. İkincisi, `Send` ve `Sync` hangi türün thread sınırını geçebileceğini belirler.[^kilit-kutuphane]

"Kilidi almadan veriye dokundum" hatası bu yüzden yazılamaz:

```rust,compile_fail,E0368
use std::sync::Mutex;

fn main() {
    let mut sayac = Mutex::new(0);
    sayac += 1; // koruyucu yok
}
```

Derleyicinin yanıtı:

```text
error[E0368]: binary assignment operation `+=` cannot be applied to type `std::sync::Mutex<{integer}>`
```

"Kilitleri yanlış sırayla aldım" hatası ise yazılabilir; onu tür sistemi görmez.

### `Ordering` bir izindir

Atomik bir işlemin `Ordering` değeri, işlemciye ve derleyiciye verilmiş bir izindir: Sıralama zayıfladıkça çevredeki bellek erişimlerini yeniden sıralama özgürlüğü artar.[^ordering] x86 gibi güçlü sıralamalı işlemciler çoğu durumda fazladan güvence verdiği için yanlış seçilmiş bir `Relaxed` orada "çalışıyor" görünebilir; ARM gibi zayıf sıralamalı bir işlemcide aynı kod bozulabilir. Testler bu tür hataları çoğu zaman yakalayamaz. Ölçmeden zayıflatmayın.

### `async fn` bir durum makinesidir

Derleyici `async fn`'i bir durum makinesine çevirir. Makine, bir bekleme noktasında *canlı* olan her yerel değişkeni alan olarak saklar; hangi yerellerin saklanacağını ayrı bir canlılık çözümlemesi belirler.[^canlilik] `Send`, `Sync` ve `Unpin` gibi auto trait'ler saklanan bu türlere bakılarak denetlenir.[^tanik] "future is not `Send`" hatasının kaynağı budur. Çözüm, sorunlu değerin kapsamını `.await`'ten önce bitirmektir; rustc 1.99.0'da `drop` çağrısının neden her zaman yetmediğini 29.3'te göreceğiz. Durum makinesinin kendisi [28. bölümün](28-future.md) konusudur.

### Uyandırıcı neden `Send + Sync`?

Bir görev her thread'den uyandırılabilmelidir: zamanlayıcının thread'inden, G/Ç olayını bekleyen thread'den, başka bir görevden. `Waker` bu yüzden `Send` ve `Sync`'tir ve `std::task::Wake` ile kurulan uyandırıcı `Arc<W>` için `W: Send + Sync` ister.[^waker] 29.2'de görevin future'ını bir `Mutex`'in ardında tutmamızın nedeni budur.

Görevlerini tek bir thread'den hiç ayırmayan çalışma zamanları `Send` olmayan future'ları da koşturabilir. tokio'nun `LocalSet`'i böyledir; bedeli, görevlerin o thread'de kalmasıdır.[^localset]

### `Pin` bir işaretçi sarmalayıcısıdır

`Pin` verinin değil, işaretçinin özelliğidir: `Pin<&mut T>`, "Bu `T` bir daha taşınmayacak" sözüdür. `T: Unpin` ise söz boştur ve `get_mut` serbesttir. `async` blokların future'ı `Unpin` değildir; onları sabitlemenin iki güvenli yolu vardır: yerinde sabitleyen `pin!` ve heap'te sabitleyen `Box::pin`.[^pin] İkisini 29.2'deki zaman aşımı birlikte gösteriyor.

### Gerçek çalışma zamanları ne ekler?

Üretimdeki çalışma zamanları bu iskeletin üstüne şunları koyar:[^tokio-runtime]

- **İş çalma (work stealing):** Birden çok işçi thread'in kendi kuyruğu vardır; işi biten işçi, kuyruğu dolu olan işçiden görev alır.
- **Olay bildirimiyle bütünleşik G/Ç:** İşletim sisteminin "Şu bağlantıda veri var" deme yolları (Linux'ta `epoll`, macOS ve BSD'lerde `kqueue`, Windows'ta `IOCP`) uyandırıcılara bağlanır.[^mio]
- **Tek bir zamanlayıcı:** Uyku başına bir thread açılmaz.
- **Adil zamanlama:** Görev sayısı sınırsız büyümüyor ve hiçbir görev thread'i bloklamıyorsa, hazır bir görevin sonlu bir sürede yoklanacağı güvence altına alınır.
- **İptal güvenliği:** Yarıda düşürülebilecek işlemlerin veri kaybetmemesi için belgelenmiş kurallar ve yardımcılar.[^iptal]

Kavramlar (görev, uyandırıcı, kuyruk, yaprak future) aynıdır.

## Özet

- "Beklerken ne yapalım?" sorusunun iki yanıtı vardır. Thread'de işletim sistemi başkasını koşturur; async'te görev thread'i geri verir.
- İşlemciyi yoran işler için thread, çok sayıda bekleyiş için async uygundur; ikisi birlikte kullanılabilir.
- Çalışma zamanı her görevi bir kez yoklar, sonra yalnızca uyandırılanı yoklar.
- `Send`, `Sync`, kilit ve atomik kuralları iki tarafta da aynıdır. Async yalnızca "Kilidi `.await` boyunca tutma" kuralını ekler.
- Derleyici veri yarışını önler, kilitlenmeyi önlemez; ona karşı kural koymak sizin işinizdir.

Async imzalarda ömürlerin ve `for<'a>` sınırlarının neden zorlaştığını [27. bölümde](27-lifetime-hrtb-gat.md), koruyucuların hangi sırayla düştüğünü [26. bölümde](26-dropck.md) bulabilirsiniz.

Sıradaki sayfa [29.1 Thread'ler ve Zamanlayıcı](29-threads.md): Doğru kilidi seçmek, beklerken işlemciyi yormamak ve kilitlenmeyi tasarımla önlemek.

## Kaynaklar

[^yigin]: std, [`std::thread`: Stack size](https://doc.rust-lang.org/std/thread/index.html#stack-size).
[^secim]: The Rust Programming Language, [17.6 Putting It All Together: Futures, Tasks, and Threads](https://doc.rust-lang.org/book/ch17-06-futures-tasks-threads.html#putting-it-all-together-futures-tasks-and-threads); *Rust for Rustaceans*, 8. bölüm (Asynchronous Programming).
[^spawn]: The Rust Programming Language, [17.2 Applying Concurrency with Async](https://doc.rust-lang.org/book/ch17-02-concurrency-with-async.html); tokio, [`spawn`](https://docs.rs/tokio/latest/tokio/task/fn.spawn.html).
[^paylasim]: The Rust Programming Language, [16.3 Shared-State Concurrency](https://doc.rust-lang.org/book/ch16-03-shared-state.html#controlling-access-with-mutexes); tokio öğreticisi, [Shared state](https://tokio.rs/tokio/tutorial/shared-state).
[^kilit-kutuphane]: std 1.99.0 kaynak kodu, [`mutex.rs`, `Sync for Mutex`](https://github.com/rust-lang/rust/blob/1.99.0/library/std/src/sync/poison/mutex.rs#L257); The Rust Programming Language, [16.4 Extensible Concurrency with Send and Sync](https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html); *Rust for Rustaceans*, 10. bölüm (Concurrency).
[^ordering]: std, [`Ordering`](https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html); The Rustonomicon, [Atomics](https://doc.rust-lang.org/nomicon/atomics.html); *Rust Atomics and Locks*, 3. bölüm (Memory Ordering).
[^canlilik]: rustc 1.99.0 kaynak kodu, [`coroutine/layout.rs`, `locals_live_across_suspend_points`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_mir_transform/src/coroutine/layout.rs#L82).
[^tanik]: rustc 1.99.0 kaynak kodu, [`coroutine/layout.rs`, `mir_coroutine_witnesses`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_mir_transform/src/coroutine/layout.rs#L445) ve [`select/mod.rs`, `CoroutineWitness`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_trait_selection/src/traits/select/mod.rs#L2258).
[^waker]: std 1.99.0 kaynak kodu, [`wake.rs`, `Sync for Waker`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/task/wake.rs#L416) ve [`task.rs`, `From<Arc<W>> for Waker`](https://github.com/rust-lang/rust/blob/1.99.0/library/alloc/src/task.rs#L114).
[^localset]: tokio, [`task::LocalSet`](https://docs.rs/tokio/latest/tokio/task/struct.LocalSet.html).
[^pin]: std, [`std::pin`](https://doc.rust-lang.org/std/pin/index.html#unpin); The Rust Programming Language, [17.5 The Pin Type and the Unpin Trait](https://doc.rust-lang.org/book/ch17-05-traits-for-async.html#the-pin-type-and-the-unpin-trait).
[^tokio-runtime]: tokio, [`runtime` modülü](https://docs.rs/tokio/latest/tokio/runtime/index.html).
[^mio]: mio, [`Poll`](https://docs.rs/mio/latest/mio/struct.Poll.html).
[^iptal]: tokio, [`select!`: Cancellation safety](https://docs.rs/tokio/latest/tokio/macro.select.html#cancellation-safety).
