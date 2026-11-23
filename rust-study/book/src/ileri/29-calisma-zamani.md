# 29.2 Çalışma Zamanı: Görev Kuyruğu ve Geri Basınç

[16. bölümdeki](../orta/16-async-await.md) yürütücü tek bir kök future'ı döngüde yokluyordu; birden çok işi de her turda hepsine bakarak yürütüyordu. Gerçek bir çalışma zamanı böyle çalışmaz: Binlerce görevden yalnızca *uyandırılanı* yoklar. Bu sayfada böyle bir çalışma zamanını yalnızca `std` ile, parça parça kuracağız. Her adımda o ana kadar yazdığımız kod derlenir ve çalışır.

Üç ana parça vardır:

- **Görev (`Gorev`):** Bir future ile "beni yeniden kuyruğa koy" bilgisi. Uyandırıcının (waker) kendisi olur.
- **Çalıştırıcı (`Calistirici`):** Hazır görevler kuyruğundan bir görev alır ve bir kez yoklar. Görev "daha bitmedi" derse onu bırakır.
- **Uyku (`Uyku`):** Gerçek bir olay kaynağı. Süre dolunca uyandırıcıyı çağıran zamanlayıcı.

Akış şöyledir:

```text
baslat(f) ──► [hazır kuyruğu] ──► yokla ──► Ready   : görev biter
                    ▲                └────► Pending : görev kuyrukta değil, bekliyor
                    │                                        │
                    └── wake() ◄── zamanlayıcı, başka görev, G/Ç olayı
```

Bunların üstüne görevin sonucunu bekleyen bir tutamaç, bir zaman aşımı ve son olarak geri basınç ekleyeceğiz. Yoklama sözleşmesini ve uyandırıcının iç yapısını [28. bölümden](28-future.md) bildiğinizi varsayıyoruz.

> [!NOTE]
> Bu sayfanın kodu `learn/advanced-concepts/ileri-eszamanlilik-async/src/calisma_zamani.rs` dosyasındadır. Testlerini `cargo test -p ileri-eszamanlilik-async` ile çalıştırabilirsiniz. Kod bloklarında yinelenen parçalar gizlidir; bloğun sağ üst köşesindeki göz simgesiyle tamamını açabilirsiniz.

<details>
<summary>Bu sayfada geçen yeni terimler</summary>

- **görev kuyruğu:** Yoklanmaya hazır görevlerin beklediği sıra.
- **`mpsc` kanalı, `Sender`, `Receiver`:** Çok gönderenli, tek alıcılı kanal ve onun gönderen ile alan ucu. Burada görev kuyruğu olarak kullanılır.
- **tembel (lazy):** Kendiliğinden başlamayan. Future ancak yoklanınca çalışır.
- **türü silinmiş (type-erased):** Gerçek türü `dyn` ardında gizlenmiş; farklı future'lar böylece aynı kutuya sığar.
- **`Box::pin`:** Değeri heap'e koyup adresini sabitlemek.
- **`block_on`, `kostur`:** "Bu future bitene kadar burada bekle" diyen giriş kapısı.
- **yaprak future (leaf future):** Başka bir future'ı değil, doğrudan bir dış olayı bekleyen future.
- **olay kaynağı (event source):** "Oldu" haberini üreten taraf: zamanlayıcı, ağ, başka bir thread.
- **birleştirici (combinator):** Başka future'ları sarıp yeni davranış ekleyen future.
- **pin projeksiyonu (pin projection):** Sabitlenmiş bir yapının alanına yine sabitlenmiş olarak ulaşmak.
- **`get_mut`:** `Pin`'in içinden sıradan bir `&mut` almak; yalnızca `Unpin` türlerde vardır.
- **tahsis (allocation):** Heap'ten yer istemek.

</details>

## Görev: yarım kalmış iş ve döneceği kuyruk

Üstüne dönüş adresi yazılmış bir iade zarfı düşünün. Zarfın içinde yarım kalmış bir iş vardır. İş beklemeye geçince zarf bir kenarda durur, kimse ona bakmaz. Biri "Hazır" deyince zarf posta kutusuna atılır ve üstündeki adrese, yani hazır görevler kuyruğuna döner. `Gorev` bu zarftır; future'ı ve döneceği kuyruğun adresini birlikte taşır. Böylece çalıştırıcı binlerce bekleyen görevi tek tek yoklamaz, yalnızca hazır olanla ilgilenir.

```rust
use std::future::Future;
use std::pin::Pin;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::task::{Wake, Waker};

type KutuluGorev = Pin<Box<dyn Future<Output = ()> + Send>>;

struct Gorev {
    gelecek: Mutex<Option<KutuluGorev>>, // biten görevde None
    kuyruk: Sender<Arc<Gorev>>,
}

impl Wake for Gorev {
    fn wake(self: Arc<Self>) {
        let kuyruk = self.kuyruk.clone();
        let _ = kuyruk.send(self); // alıcı kapandıysa uyandıracak kimse de yoktur
    }
}

fn main() {
    let (kuyruk, hazir) = mpsc::channel();
    let gorev = Arc::new(Gorev {
        gelecek: Mutex::new(Some(Box::pin(async {}))),
        kuyruk,
    });

    let waker = Waker::from(gorev);
    println!("uyandırmadan önce kuyrukta mı? {}", hazir.try_recv().is_ok());
    waker.wake();
    println!("uyandırınca kuyrukta mı?       {}", hazir.try_recv().is_ok());
}
```

Çıktı:

```text
uyandırmadan önce kuyrukta mı? false
uyandırınca kuyrukta mı?       true
```

Demek ki uyandırmak, görevi hazır kuyruğuna geri koymaktan ibarettir. `std::task::Wake` trait'i, bir `Arc<Gorev>`'den güvenli bir `Waker` üretmenin yoludur; [28. bölümdeki](28-future.md) ham fonksiyon tablosunu elle yazmaya gerek kalmaz.[^wake]

Türlerin her parçası bir ihtiyaca karşılık gelir:

| Parça | Neden gerekli? |
| --- | --- |
| `dyn Future` | Farklı `async` blokları farklı türlerdir; aynı kuyruğa girebilmeleri için türleri silinir. |
| `Pin` | Bir future'ı yoklayabilmek için sabitlenmiş olması gerekir. |
| `Box` | Kuyrukta taşınan kutudur; içindeki future'ın adresi değişmez. |
| `Send` | Görev başka thread'lerden uyandırılabilir. |
| `Option` | Görev bitince `None` olur; gecikmiş bir `wake`, bitmiş future'ı yeniden yoklatamaz. |
| `Mutex` | `Waker` her thread'den çağrılabildiği için `Gorev` `Sync` olmalıdır; kilit, future'a erişimi teke indirir. |

Son satırın keyfî olmadığını derleyici de söyler. `Waker`'ın kendisi `Send + Sync`'tir; bu yüzden `Waker::from(Arc<W>)`, `W`'nin de `Send + Sync` olmasını ister.[^waker-from] Kilidi kaldırırsak program derlenmez:

```rust,compile_fail,E0277
# use std::future::Future;
# use std::pin::Pin;
# use std::sync::Arc;
# use std::sync::mpsc::Sender;
# use std::task::{Wake, Waker};
# type KutuluGorev = Pin<Box<dyn Future<Output = ()> + Send>>;
struct Gorev {
    gelecek: Option<KutuluGorev>, // kilit yok
    kuyruk: Sender<Arc<Gorev>>,
}
# impl Wake for Gorev {
#     fn wake(self: Arc<Self>) {
#         let kuyruk = self.kuyruk.clone();
#         let _ = kuyruk.send(self);
#     }
# }

fn uyandirici(gorev: Arc<Gorev>) -> Waker {
    Waker::from(gorev)
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0277]: `(dyn Future<Output = ()> + Send + 'static)` cannot be shared between threads safely
```

Kutudaki future `Send`'dir, ama `Sync` olduğu hiçbir yerde söylenmemiştir. `Mutex<T>`'nin `Sync` olması için `T: Send` yettiğinden ([29.1'deki tablo](29-threadler.md)) kilit tam da eksik olan damgayı ekler.

## Çalıştırıcı: hazır olanı bir kez yokla

Bankada sıra numarası alıp oturursunuz; gişe görevlisi yalnızca numarası yanan kişiyle ilgilenir. Evrakı eksik olan gişenin önünde dikilmez; yerine oturur, evrakı gelince yeni numara alır. `Calistirici` gişe görevlisi, `Baslatici` numara veren makinedir. Sıra boşsa görevli de uyur.

Önce türler ve göreve ilk numarasını veren metot:

```rust
# use std::future::Future;
# use std::pin::Pin;
# use std::sync::mpsc::Sender;
# use std::sync::{Arc, Mutex};
# use std::task::{Wake, Waker};
# type KutuluGorev = Pin<Box<dyn Future<Output = ()> + Send>>;
# struct Gorev { gelecek: Mutex<Option<KutuluGorev>>, kuyruk: Sender<Arc<Gorev>> }
# impl Wake for Gorev {
#     fn wake(self: Arc<Self>) { let _ = self.kuyruk.clone().send(self); }
# }
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver};

pub struct Calistirici {
    hazir: Receiver<Arc<Gorev>>,
    baslatici: Baslatici,
    poll_sayisi: usize,
}

#[derive(Clone)]
pub struct Baslatici {
    kuyruk: Sender<Arc<Gorev>>,
    canli: Arc<AtomicUsize>, // henüz bitmemiş görev sayısı
}

impl Baslatici {
    fn kuyruga_koy(&self, gelecek: impl Future<Output = ()> + Send + 'static) {
        self.canli.fetch_add(1, Ordering::SeqCst);
        let gorev = Arc::new(Gorev {
            gelecek: Mutex::new(Some(Box::pin(gelecek))),
            kuyruk: self.kuyruk.clone(),
        });
        let _ = self.kuyruk.send(gorev); // ilk yoklama için
    }
}

impl Calistirici {
    pub fn yeni() -> Self {
        let (kuyruk, hazir) = mpsc::channel();
        let baslatici = Baslatici { kuyruk, canli: Arc::new(AtomicUsize::new(0)) };
        Calistirici { hazir, baslatici, poll_sayisi: 0 }
    }

    pub fn baslatici(&self) -> Baslatici {
        self.baslatici.clone()
    }
}

fn main() {
    let calistirici = Calistirici::yeni();
    let baslatici = calistirici.baslatici();
    baslatici.kuyruga_koy(async { println!("bu satır yazılmaz") });

    println!("canlı görev: {}", calistirici.baslatici.canli.load(Ordering::SeqCst));
    println!("kuyrukta mı? {}", calistirici.hazir.try_recv().is_ok());
}
```

Çıktı:

```text
canlı görev: 1
kuyrukta mı? true
```

Görev kuyruğa girdi, ama çalışmadı; future *tembeldir* ve onu yoklayan kimse olmadı.[^future] `Baslatici` klonlanabilir, çünkü bir görevin içine taşınıp oradan yeni görevler başlatabilmelidir. `canli` sayacı, çalıştırıcının ne zaman duracağını bilmesini sağlar.

Kuyruk olarak `mpsc::channel` kullandık. Kanal boşken `recv` thread'i uyutur, `send` ise hiç beklemez; görevler gönderildikleri sırayla alınır.[^kanal] Yoklama döngüsü şimdi kısa:

```rust
# use std::future::Future;
# use std::pin::Pin;
# use std::sync::mpsc::Sender;
# use std::sync::{Arc, Mutex};
# use std::task::{Wake, Waker};
# type KutuluGorev = Pin<Box<dyn Future<Output = ()> + Send>>;
# struct Gorev { gelecek: Mutex<Option<KutuluGorev>>, kuyruk: Sender<Arc<Gorev>> }
# impl Wake for Gorev {
#     fn wake(self: Arc<Self>) { let _ = self.kuyruk.clone().send(self); }
# }
# use std::sync::atomic::{AtomicUsize, Ordering};
# use std::sync::mpsc::{self, Receiver};
# pub struct Calistirici { hazir: Receiver<Arc<Gorev>>, baslatici: Baslatici, poll_sayisi: usize }
# #[derive(Clone)]
# pub struct Baslatici { kuyruk: Sender<Arc<Gorev>>, canli: Arc<AtomicUsize> }
# impl Baslatici {
#     fn kuyruga_koy(&self, gelecek: impl Future<Output = ()> + Send + 'static) {
#         self.canli.fetch_add(1, Ordering::SeqCst);
#         let gelecek = Mutex::new(Some(Box::pin(gelecek) as KutuluGorev));
#         let _ = self.kuyruk.send(Arc::new(Gorev { gelecek, kuyruk: self.kuyruk.clone() }));
#     }
# }
# impl Calistirici {
#     pub fn yeni() -> Self {
#         let (kuyruk, hazir) = mpsc::channel();
#         let baslatici = Baslatici { kuyruk, canli: Arc::new(AtomicUsize::new(0)) };
#         Calistirici { hazir, baslatici, poll_sayisi: 0 }
#     }
#     pub fn baslatici(&self) -> Baslatici { self.baslatici.clone() }
# }
use std::task::{Context, Poll};

impl Calistirici {
    // Bütün görevler bitene dek çalışır; toplam yoklama sayısını döndürür.
    pub fn calistir(&mut self) -> usize {
        while self.baslatici.canli.load(Ordering::SeqCst) > 0 {
            let Ok(gorev) = self.hazir.recv() else { break };
            let mut yuva = gorev.gelecek.lock().expect("kilit zehirlenmemeli");
            let Some(mut gelecek) = yuva.take() else {
                continue; // bitmiş bir göreve gecikmiş ya da yinelenen uyandırma
            };
            let waker = Waker::from(Arc::clone(&gorev));
            let mut cx = Context::from_waker(&waker);
            self.poll_sayisi += 1;
            match gelecek.as_mut().poll(&mut cx) {
                Poll::Pending => *yuva = Some(gelecek), // uyandırılınca yine yoklanır
                Poll::Ready(()) => {
                    self.baslatici.canli.fetch_sub(1, Ordering::SeqCst);
                }
            }
        }
        self.poll_sayisi
    }
}

// Bir kez "sıramı veriyorum" diyen future.
struct SiraVer {
    verildi: bool,
}

impl Future for SiraVer {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.verildi {
            return Poll::Ready(());
        }
        self.verildi = true;
        cx.waker().wake_by_ref(); // hemen kuyruğun sonuna
        Poll::Pending
    }
}

fn main() {
    let mut calistirici = Calistirici::yeni();
    for ad in ["A", "B"] {
        calistirici.baslatici().kuyruga_koy(async move {
            println!("{ad}1");
            SiraVer { verildi: false }.await;
            println!("{ad}2");
        });
    }
    println!("toplam yoklama: {}", calistirici.calistir());
}
```

Çıktı:

```text
A1
B1
A2
B2
toplam yoklama: 4
```

İki görev tek thread üzerinde iç içe ilerledi. `A` sırasını verince kuyruğun sonuna gitti, sıradaki `B` yoklandı. Bekleyen görev kuyrukta durmadığı için boşuna yoklama da yapılmadı: Her görev iki kez yoklandı, biri başlarken, biri uyandırılınca.

### Çalıştırıcının sınırları

Döngü üç duruma dikkat etmeyi gerektirir:

- **Boş kuyrukta uyku:** `recv` hazır görev yoksa thread'i uyutur, boşa dönmez. Bir zamanlayıcı ya da başka bir thread `wake` çağırınca kanal dolar ve çalıştırıcı uyanır.
- **Unutulan uyandırıcı:** Bir görev kendini uyandıracak kimseyi ayarlamadan `Pending` dönerse kanala hiçbir şey gelmez ve `calistir` sonsuza dek bekler. `else { break }` kolu bu durumda kurtarmaz: `recv`, ancak bütün gönderenler düşünce hata verir; oysa `Calistirici` kendi `Baslatici`'sını, yani bir göndereni hep elinde tutar.
- **Geç kalan görev:** Sayaç sıfıra inip döngü çıktıktan *sonra* başka bir thread'den başlatılan görev yoklanmaz. Görevler ya `calistir`'dan önce ya da çalışan bir görevin içinden başlatılmalıdır.

Kilidin yoklama boyunca tutulması sorun çıkarmaz: `wake` yalnızca kuyruğa yazar, `gelecek` kilidine dokunmaz. Yoklanırken uyandırılan görev kuyruğa yeniden girer ve bir sonraki turda yoklanır.

## Tutamaç: başlatılan görevin sonucu

Kuru temizlemeciye gömlek bırakınca size bir fiş verirler. Almaya gittiğinizde gömlek hazırsa alırsınız; değilse telefon numaranızı bırakırsınız, hazır olunca ararlar. `Tutamac` bu fiştir ve thread'lerdeki `JoinHandle`'ın karşılığıdır: Görevin sonucu paylaşılan bir kutuda birikir, siz `.await` deyince ya sonucu alırsınız ya da uyandırıcınızı bırakıp beklersiniz.

```rust
# use std::future::Future;
# use std::pin::Pin;
# use std::sync::mpsc::Sender;
# use std::sync::{Arc, Mutex};
# use std::task::{Wake, Waker};
# type KutuluGorev = Pin<Box<dyn Future<Output = ()> + Send>>;
# struct Gorev { gelecek: Mutex<Option<KutuluGorev>>, kuyruk: Sender<Arc<Gorev>> }
# impl Wake for Gorev {
#     fn wake(self: Arc<Self>) { let _ = self.kuyruk.clone().send(self); }
# }
# use std::sync::atomic::{AtomicUsize, Ordering};
# use std::sync::mpsc::{self, Receiver};
# pub struct Calistirici { hazir: Receiver<Arc<Gorev>>, baslatici: Baslatici, poll_sayisi: usize }
# #[derive(Clone)]
# pub struct Baslatici { kuyruk: Sender<Arc<Gorev>>, canli: Arc<AtomicUsize> }
# impl Baslatici {
#     fn kuyruga_koy(&self, gelecek: impl Future<Output = ()> + Send + 'static) {
#         self.canli.fetch_add(1, Ordering::SeqCst);
#         let gelecek = Mutex::new(Some(Box::pin(gelecek) as KutuluGorev));
#         let _ = self.kuyruk.send(Arc::new(Gorev { gelecek, kuyruk: self.kuyruk.clone() }));
#     }
# }
# impl Calistirici {
#     pub fn yeni() -> Self {
#         let (kuyruk, hazir) = mpsc::channel();
#         let baslatici = Baslatici { kuyruk, canli: Arc::new(AtomicUsize::new(0)) };
#         Calistirici { hazir, baslatici, poll_sayisi: 0 }
#     }
#     pub fn baslatici(&self) -> Baslatici { self.baslatici.clone() }
# }
# use std::task::{Context, Poll};
# impl Calistirici {
#     pub fn calistir(&mut self) -> usize {
#         while self.baslatici.canli.load(Ordering::SeqCst) > 0 {
#             let Ok(gorev) = self.hazir.recv() else { break };
#             let mut yuva = gorev.gelecek.lock().expect("kilit zehirlenmemeli");
#             let Some(mut gelecek) = yuva.take() else { continue };
#             let waker = Waker::from(Arc::clone(&gorev));
#             self.poll_sayisi += 1;
#             match gelecek.as_mut().poll(&mut Context::from_waker(&waker)) {
#                 Poll::Pending => *yuva = Some(gelecek),
#                 Poll::Ready(()) => { self.baslatici.canli.fetch_sub(1, Ordering::SeqCst); }
#             }
#         }
#         self.poll_sayisi
#     }
# }
struct TutamacDurumu<T> {
    sonuc: Option<T>,
    bekleyen: Option<Waker>,
}

pub struct Tutamac<T> {
    paylasilan: Arc<Mutex<TutamacDurumu<T>>>,
}

impl<T> Tutamac<T> {
    // Çalıştırıcı durduktan sonra, async dışında sonucu almak için.
    pub fn sonucu_al(&self) -> Option<T> {
        self.paylasilan.lock().expect("kilit zehirlenmemeli").sonuc.take()
    }
}

impl<T> Future for Tutamac<T> {
    type Output = T;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        let mut durum = self.paylasilan.lock().expect("kilit zehirlenmemeli");
        match durum.sonuc.take() {
            Some(deger) => Poll::Ready(deger),
            None => {
                durum.bekleyen = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}
# fn main() {}
```

`poll` içinde "Sonuç var mı?" denetimi ile uyandırıcının kaydı *aynı kilit altında* yapılır. İki ayrı adım olsaydı şu sıra yaşanabilirdi:

| Adım | Bekleyen görev | Biten görev |
| --- | --- | --- |
| 1 | Sonuca bakar: yok. | |
| 2 | | Sonucu yazar, `bekleyen`'e bakar: boş. |
| 3 | Uyandırıcısını kaydeder; artık kimse çağırmayacak. | |

Bu, 29.1'de `Condvar` için gördüğümüz kayıp uyandırmanın aynısıdır; çözümü de aynıdır.[^donus]

Tutamacı dolduran taraf `baslat`'tır. Kullanıcının future'ını sarar: Future bitince sonucu kutuya yazar, bekleyen varsa onu uyandırır.

```rust
# use std::future::Future;
# use std::pin::Pin;
# use std::sync::mpsc::Sender;
# use std::sync::{Arc, Mutex};
# use std::task::{Wake, Waker};
# type KutuluGorev = Pin<Box<dyn Future<Output = ()> + Send>>;
# struct Gorev { gelecek: Mutex<Option<KutuluGorev>>, kuyruk: Sender<Arc<Gorev>> }
# impl Wake for Gorev {
#     fn wake(self: Arc<Self>) { let _ = self.kuyruk.clone().send(self); }
# }
# use std::sync::atomic::{AtomicUsize, Ordering};
# use std::sync::mpsc::{self, Receiver};
# pub struct Calistirici { hazir: Receiver<Arc<Gorev>>, baslatici: Baslatici, poll_sayisi: usize }
# #[derive(Clone)]
# pub struct Baslatici { kuyruk: Sender<Arc<Gorev>>, canli: Arc<AtomicUsize> }
# impl Baslatici {
#     fn kuyruga_koy(&self, gelecek: impl Future<Output = ()> + Send + 'static) {
#         self.canli.fetch_add(1, Ordering::SeqCst);
#         let gelecek = Mutex::new(Some(Box::pin(gelecek) as KutuluGorev));
#         let _ = self.kuyruk.send(Arc::new(Gorev { gelecek, kuyruk: self.kuyruk.clone() }));
#     }
# }
# impl Calistirici {
#     pub fn yeni() -> Self {
#         let (kuyruk, hazir) = mpsc::channel();
#         let baslatici = Baslatici { kuyruk, canli: Arc::new(AtomicUsize::new(0)) };
#         Calistirici { hazir, baslatici, poll_sayisi: 0 }
#     }
#     pub fn baslatici(&self) -> Baslatici { self.baslatici.clone() }
# }
# use std::task::{Context, Poll};
# impl Calistirici {
#     pub fn calistir(&mut self) -> usize {
#         while self.baslatici.canli.load(Ordering::SeqCst) > 0 {
#             let Ok(gorev) = self.hazir.recv() else { break };
#             let mut yuva = gorev.gelecek.lock().expect("kilit zehirlenmemeli");
#             let Some(mut gelecek) = yuva.take() else { continue };
#             let waker = Waker::from(Arc::clone(&gorev));
#             self.poll_sayisi += 1;
#             match gelecek.as_mut().poll(&mut Context::from_waker(&waker)) {
#                 Poll::Pending => *yuva = Some(gelecek),
#                 Poll::Ready(()) => { self.baslatici.canli.fetch_sub(1, Ordering::SeqCst); }
#             }
#         }
#         self.poll_sayisi
#     }
# }
# struct TutamacDurumu<T> { sonuc: Option<T>, bekleyen: Option<Waker> }
# pub struct Tutamac<T> { paylasilan: Arc<Mutex<TutamacDurumu<T>>> }
# impl<T> Tutamac<T> {
#     pub fn sonucu_al(&self) -> Option<T> {
#         self.paylasilan.lock().expect("kilit zehirlenmemeli").sonuc.take()
#     }
# }
# impl<T> Future for Tutamac<T> {
#     type Output = T;
#     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
#         let mut durum = self.paylasilan.lock().expect("kilit zehirlenmemeli");
#         if let Some(deger) = durum.sonuc.take() { return Poll::Ready(deger); }
#         durum.bekleyen = Some(cx.waker().clone());
#         Poll::Pending
#     }
# }
impl Baslatici {
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
        self.kuyruga_koy(async move {
            let deger = gelecek.await;
            let bekleyen = {
                let mut durum = gorev_tarafi.lock().expect("kilit zehirlenmemeli");
                durum.sonuc = Some(deger);
                durum.bekleyen.take()
            }; // kilit burada bırakıldı
            if let Some(waker) = bekleyen {
                waker.wake(); // kilit dışında uyandır
            }
        });
        Tutamac { paylasilan }
    }
}

fn main() {
    let mut calistirici = Calistirici::yeni();
    let tutamac = calistirici.baslatici().baslat(async { 6 * 7 });

    println!("çalıştırmadan önce: {:?}", tutamac.sonucu_al());
    calistirici.calistir();
    println!("çalıştırdıktan sonra: {:?}", tutamac.sonucu_al());
}
```

Çıktı:

```text
çalıştırmadan önce: None
çalıştırdıktan sonra: Some(42)
```

Uyandırma kilit bırakıldıktan sonra yapılır. Uyandırıcının arkasında ne olduğunu bilmeyiz; kilit eldeyken yabancı kod çağırmamak 29.1'deki kuralın aynısıdır.

Tek bir future'ı sonuna dek koşturmak için kısa bir yol da ekleyelim. tokio'daki `Runtime::block_on` bu işi yapar; buradaki adı `kostur`:[^block-on]

```rust
# use std::future::Future;
# use std::pin::Pin;
# use std::sync::mpsc::Sender;
# use std::sync::{Arc, Mutex};
# use std::task::{Wake, Waker};
# type KutuluGorev = Pin<Box<dyn Future<Output = ()> + Send>>;
# struct Gorev { gelecek: Mutex<Option<KutuluGorev>>, kuyruk: Sender<Arc<Gorev>> }
# impl Wake for Gorev {
#     fn wake(self: Arc<Self>) { let _ = self.kuyruk.clone().send(self); }
# }
# use std::sync::atomic::{AtomicUsize, Ordering};
# use std::sync::mpsc::{self, Receiver};
# pub struct Calistirici { hazir: Receiver<Arc<Gorev>>, baslatici: Baslatici, poll_sayisi: usize }
# #[derive(Clone)]
# pub struct Baslatici { kuyruk: Sender<Arc<Gorev>>, canli: Arc<AtomicUsize> }
# impl Baslatici {
#     fn kuyruga_koy(&self, gelecek: impl Future<Output = ()> + Send + 'static) {
#         self.canli.fetch_add(1, Ordering::SeqCst);
#         let gelecek = Mutex::new(Some(Box::pin(gelecek) as KutuluGorev));
#         let _ = self.kuyruk.send(Arc::new(Gorev { gelecek, kuyruk: self.kuyruk.clone() }));
#     }
# }
# impl Calistirici {
#     pub fn yeni() -> Self {
#         let (kuyruk, hazir) = mpsc::channel();
#         let baslatici = Baslatici { kuyruk, canli: Arc::new(AtomicUsize::new(0)) };
#         Calistirici { hazir, baslatici, poll_sayisi: 0 }
#     }
#     pub fn baslatici(&self) -> Baslatici { self.baslatici.clone() }
# }
# use std::task::{Context, Poll};
# impl Calistirici {
#     pub fn calistir(&mut self) -> usize {
#         while self.baslatici.canli.load(Ordering::SeqCst) > 0 {
#             let Ok(gorev) = self.hazir.recv() else { break };
#             let mut yuva = gorev.gelecek.lock().expect("kilit zehirlenmemeli");
#             let Some(mut gelecek) = yuva.take() else { continue };
#             let waker = Waker::from(Arc::clone(&gorev));
#             self.poll_sayisi += 1;
#             match gelecek.as_mut().poll(&mut Context::from_waker(&waker)) {
#                 Poll::Pending => *yuva = Some(gelecek),
#                 Poll::Ready(()) => { self.baslatici.canli.fetch_sub(1, Ordering::SeqCst); }
#             }
#         }
#         self.poll_sayisi
#     }
# }
# struct TutamacDurumu<T> { sonuc: Option<T>, bekleyen: Option<Waker> }
# pub struct Tutamac<T> { paylasilan: Arc<Mutex<TutamacDurumu<T>>> }
# impl<T> Tutamac<T> {
#     pub fn sonucu_al(&self) -> Option<T> {
#         self.paylasilan.lock().expect("kilit zehirlenmemeli").sonuc.take()
#     }
# }
# impl<T> Future for Tutamac<T> {
#     type Output = T;
#     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
#         let mut durum = self.paylasilan.lock().expect("kilit zehirlenmemeli");
#         if let Some(deger) = durum.sonuc.take() { return Poll::Ready(deger); }
#         durum.bekleyen = Some(cx.waker().clone());
#         Poll::Pending
#     }
# }
# impl Baslatici {
#     pub fn baslat<F>(&self, gelecek: F) -> Tutamac<F::Output>
#     where F: Future + Send + 'static, F::Output: Send + 'static {
#         let paylasilan = Arc::new(Mutex::new(TutamacDurumu { sonuc: None, bekleyen: None }));
#         let gorev_tarafi = Arc::clone(&paylasilan);
#         self.kuyruga_koy(async move {
#             let deger = gelecek.await;
#             let bekleyen = {
#                 let mut durum = gorev_tarafi.lock().expect("kilit zehirlenmemeli");
#                 durum.sonuc = Some(deger);
#                 durum.bekleyen.take()
#             };
#             if let Some(waker) = bekleyen { waker.wake(); }
#         });
#         Tutamac { paylasilan }
#     }
# }
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

fn main() {
    println!("{}", kostur(async { 6 * 7 }));

    // Bir görev başka görevler başlatıp sonuçlarını bekleyebilir.
    let mut calistirici = Calistirici::yeni();
    let baslatici = calistirici.baslatici();
    let ic_baslatici = baslatici.clone();
    let tutamac = baslatici.baslat(async move {
        let a = ic_baslatici.baslat(async { 1 });
        let b = ic_baslatici.baslat(async { 2 });
        a.await + b.await
    });
    calistirici.calistir();
    println!("{:?}", tutamac.sonucu_al());
}
```

Çıktı:

```text
42
Some(3)
```

## Başlatma koşulları: `Send + 'static`

`baslat`'ın sınırlarını bir kuryeye paket vermek gibi düşünün. Kurye paketi başka bir binaya götürebilir; paket yolculuğa dayanmalıdır (`Send`). Paketi siz evden çıktıktan sonra da teslim edebilir; paketin içinden masanızdaki bir eşyaya bağlı ip çıkmamalıdır (`'static`).

- **`Send`:** Görev, `Waker` aracılığıyla başka thread'lerden erişilebilen bir `Arc`'ın içinde durur. Bu yüzden `.await` noktaları arasında `Rc` ya da `MutexGuard` taşıyan future'lar burada derlenmez; [29.3](29-istemci.md) bunu ayrıntısıyla gösteriyor.
- **`'static`:** Görev, onu başlatan fonksiyon bittikten sonra da çalışabilir. `'static` burada "sonsuza dek yaşar" demek değildir; "geçici hiçbir şeyi ödünç almıyor" demektir. `thread::spawn`'ın gerekçesi de budur.[^spawn]

Yerel bir değişkeni ödünç alan görev bu yüzden reddedilir:

```rust,compile_fail,E0373
# use std::future::Future;
# use std::pin::Pin;
# use std::sync::mpsc::Sender;
# use std::sync::{Arc, Mutex};
# use std::task::{Wake, Waker};
# type KutuluGorev = Pin<Box<dyn Future<Output = ()> + Send>>;
# struct Gorev { gelecek: Mutex<Option<KutuluGorev>>, kuyruk: Sender<Arc<Gorev>> }
# impl Wake for Gorev {
#     fn wake(self: Arc<Self>) { let _ = self.kuyruk.clone().send(self); }
# }
# use std::sync::atomic::{AtomicUsize, Ordering};
# use std::sync::mpsc::{self, Receiver};
# pub struct Calistirici { hazir: Receiver<Arc<Gorev>>, baslatici: Baslatici, poll_sayisi: usize }
# #[derive(Clone)]
# pub struct Baslatici { kuyruk: Sender<Arc<Gorev>>, canli: Arc<AtomicUsize> }
# impl Baslatici {
#     fn kuyruga_koy(&self, gelecek: impl Future<Output = ()> + Send + 'static) {
#         self.canli.fetch_add(1, Ordering::SeqCst);
#         let gelecek = Mutex::new(Some(Box::pin(gelecek) as KutuluGorev));
#         let _ = self.kuyruk.send(Arc::new(Gorev { gelecek, kuyruk: self.kuyruk.clone() }));
#     }
# }
# impl Calistirici {
#     pub fn yeni() -> Self {
#         let (kuyruk, hazir) = mpsc::channel();
#         let baslatici = Baslatici { kuyruk, canli: Arc::new(AtomicUsize::new(0)) };
#         Calistirici { hazir, baslatici, poll_sayisi: 0 }
#     }
#     pub fn baslatici(&self) -> Baslatici { self.baslatici.clone() }
# }
# use std::task::{Context, Poll};
# impl Calistirici {
#     pub fn calistir(&mut self) -> usize {
#         while self.baslatici.canli.load(Ordering::SeqCst) > 0 {
#             let Ok(gorev) = self.hazir.recv() else { break };
#             let mut yuva = gorev.gelecek.lock().expect("kilit zehirlenmemeli");
#             let Some(mut gelecek) = yuva.take() else { continue };
#             let waker = Waker::from(Arc::clone(&gorev));
#             self.poll_sayisi += 1;
#             match gelecek.as_mut().poll(&mut Context::from_waker(&waker)) {
#                 Poll::Pending => *yuva = Some(gelecek),
#                 Poll::Ready(()) => { self.baslatici.canli.fetch_sub(1, Ordering::SeqCst); }
#             }
#         }
#         self.poll_sayisi
#     }
# }
# struct TutamacDurumu<T> { sonuc: Option<T>, bekleyen: Option<Waker> }
# pub struct Tutamac<T> { paylasilan: Arc<Mutex<TutamacDurumu<T>>> }
# impl<T> Tutamac<T> {
#     pub fn sonucu_al(&self) -> Option<T> {
#         self.paylasilan.lock().expect("kilit zehirlenmemeli").sonuc.take()
#     }
# }
# impl<T> Future for Tutamac<T> {
#     type Output = T;
#     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
#         let mut durum = self.paylasilan.lock().expect("kilit zehirlenmemeli");
#         if let Some(deger) = durum.sonuc.take() { return Poll::Ready(deger); }
#         durum.bekleyen = Some(cx.waker().clone());
#         Poll::Pending
#     }
# }
# impl Baslatici {
#     pub fn baslat<F>(&self, gelecek: F) -> Tutamac<F::Output>
#     where F: Future + Send + 'static, F::Output: Send + 'static {
#         let paylasilan = Arc::new(Mutex::new(TutamacDurumu { sonuc: None, bekleyen: None }));
#         let gorev_tarafi = Arc::clone(&paylasilan);
#         self.kuyruga_koy(async move {
#             let deger = gelecek.await;
#             let bekleyen = {
#                 let mut durum = gorev_tarafi.lock().expect("kilit zehirlenmemeli");
#                 durum.sonuc = Some(deger);
#                 durum.bekleyen.take()
#             };
#             if let Some(waker) = bekleyen { waker.wake(); }
#         });
#         Tutamac { paylasilan }
#     }
# }
# pub fn kostur<F>(gelecek: F) -> F::Output
# where F: Future + Send + 'static, F::Output: Send + 'static {
#     let mut calistirici = Calistirici::yeni();
#     let tutamac = calistirici.baslatici().baslat(gelecek);
#     calistirici.calistir();
#     tutamac.sonucu_al().expect("görev bitti ama sonuç yok")
# }
fn selamla(baslatici: &Baslatici) {
    let ad = String::from("ada");
    baslatici.baslat(async { println!("{ad}") });
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0373]: async block may outlive the current function, but it borrows `ad`, which is owned by the current function
```

`selamla` dönünce `ad` düşer, ama görev daha sonra yoklanacaktır. Çözüm, sahipliği göreve vermektir:[^e0373]

```rust
# use std::future::Future;
# use std::pin::Pin;
# use std::sync::mpsc::Sender;
# use std::sync::{Arc, Mutex};
# use std::task::{Wake, Waker};
# type KutuluGorev = Pin<Box<dyn Future<Output = ()> + Send>>;
# struct Gorev { gelecek: Mutex<Option<KutuluGorev>>, kuyruk: Sender<Arc<Gorev>> }
# impl Wake for Gorev {
#     fn wake(self: Arc<Self>) { let _ = self.kuyruk.clone().send(self); }
# }
# use std::sync::atomic::{AtomicUsize, Ordering};
# use std::sync::mpsc::{self, Receiver};
# pub struct Calistirici { hazir: Receiver<Arc<Gorev>>, baslatici: Baslatici, poll_sayisi: usize }
# #[derive(Clone)]
# pub struct Baslatici { kuyruk: Sender<Arc<Gorev>>, canli: Arc<AtomicUsize> }
# impl Baslatici {
#     fn kuyruga_koy(&self, gelecek: impl Future<Output = ()> + Send + 'static) {
#         self.canli.fetch_add(1, Ordering::SeqCst);
#         let gelecek = Mutex::new(Some(Box::pin(gelecek) as KutuluGorev));
#         let _ = self.kuyruk.send(Arc::new(Gorev { gelecek, kuyruk: self.kuyruk.clone() }));
#     }
# }
# impl Calistirici {
#     pub fn yeni() -> Self {
#         let (kuyruk, hazir) = mpsc::channel();
#         let baslatici = Baslatici { kuyruk, canli: Arc::new(AtomicUsize::new(0)) };
#         Calistirici { hazir, baslatici, poll_sayisi: 0 }
#     }
#     pub fn baslatici(&self) -> Baslatici { self.baslatici.clone() }
# }
# use std::task::{Context, Poll};
# impl Calistirici {
#     pub fn calistir(&mut self) -> usize {
#         while self.baslatici.canli.load(Ordering::SeqCst) > 0 {
#             let Ok(gorev) = self.hazir.recv() else { break };
#             let mut yuva = gorev.gelecek.lock().expect("kilit zehirlenmemeli");
#             let Some(mut gelecek) = yuva.take() else { continue };
#             let waker = Waker::from(Arc::clone(&gorev));
#             self.poll_sayisi += 1;
#             match gelecek.as_mut().poll(&mut Context::from_waker(&waker)) {
#                 Poll::Pending => *yuva = Some(gelecek),
#                 Poll::Ready(()) => { self.baslatici.canli.fetch_sub(1, Ordering::SeqCst); }
#             }
#         }
#         self.poll_sayisi
#     }
# }
# struct TutamacDurumu<T> { sonuc: Option<T>, bekleyen: Option<Waker> }
# pub struct Tutamac<T> { paylasilan: Arc<Mutex<TutamacDurumu<T>>> }
# impl<T> Tutamac<T> {
#     pub fn sonucu_al(&self) -> Option<T> {
#         self.paylasilan.lock().expect("kilit zehirlenmemeli").sonuc.take()
#     }
# }
# impl<T> Future for Tutamac<T> {
#     type Output = T;
#     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
#         let mut durum = self.paylasilan.lock().expect("kilit zehirlenmemeli");
#         if let Some(deger) = durum.sonuc.take() { return Poll::Ready(deger); }
#         durum.bekleyen = Some(cx.waker().clone());
#         Poll::Pending
#     }
# }
# impl Baslatici {
#     pub fn baslat<F>(&self, gelecek: F) -> Tutamac<F::Output>
#     where F: Future + Send + 'static, F::Output: Send + 'static {
#         let paylasilan = Arc::new(Mutex::new(TutamacDurumu { sonuc: None, bekleyen: None }));
#         let gorev_tarafi = Arc::clone(&paylasilan);
#         self.kuyruga_koy(async move {
#             let deger = gelecek.await;
#             let bekleyen = {
#                 let mut durum = gorev_tarafi.lock().expect("kilit zehirlenmemeli");
#                 durum.sonuc = Some(deger);
#                 durum.bekleyen.take()
#             };
#             if let Some(waker) = bekleyen { waker.wake(); }
#         });
#         Tutamac { paylasilan }
#     }
# }
# pub fn kostur<F>(gelecek: F) -> F::Output
# where F: Future + Send + 'static, F::Output: Send + 'static {
#     let mut calistirici = Calistirici::yeni();
#     let tutamac = calistirici.baslatici().baslat(gelecek);
#     calistirici.calistir();
#     tutamac.sonucu_al().expect("görev bitti ama sonuç yok")
# }
fn selamla(baslatici: &Baslatici) {
    let ad = String::from("ada");
    baslatici.baslat(async move { println!("merhaba {ad}") }); // ad göreve taşındı
}

fn main() {
    let mut calistirici = Calistirici::yeni();
    selamla(&calistirici.baslatici());
    calistirici.calistir();
}
```

Çıktı:

```text
merhaba ada
```

## Uyku: zamanlayıcı

Şimdiye kadarki future'lar ya hemen bitti ya da `Pending` dönerken kendilerini hemen uyandırdı. Gerçek bir *yaprak future* uyandırıcısını, olayı bilecek tarafa teslim edip `Pending` döner.

Sabah kalkmak için saatin başında oturup akrebi izlemezsiniz; çalar saati kurar ve uyursunuz. `Uyku` böyle çalışır: İlk yoklamada saati kurar, zili (uyandırıcıyı) saate teslim eder ve "Daha olmadı" der. Süre dolunca saat zili çalar, görev kuyruğa döner ve kaldığı yerden sürer. O sırada çalıştırıcının thread'i boştadır ve öteki görevlere bakabilir.

Bizim saatimiz, süre kadar uyuyan bir yardımcı thread olacak:

```rust
# use std::future::Future;
# use std::pin::Pin;
# use std::sync::mpsc::Sender;
# use std::sync::{Arc, Mutex};
# use std::task::{Wake, Waker};
# type KutuluGorev = Pin<Box<dyn Future<Output = ()> + Send>>;
# struct Gorev { gelecek: Mutex<Option<KutuluGorev>>, kuyruk: Sender<Arc<Gorev>> }
# impl Wake for Gorev {
#     fn wake(self: Arc<Self>) { let _ = self.kuyruk.clone().send(self); }
# }
# use std::sync::atomic::{AtomicUsize, Ordering};
# use std::sync::mpsc::{self, Receiver};
# pub struct Calistirici { hazir: Receiver<Arc<Gorev>>, baslatici: Baslatici, poll_sayisi: usize }
# #[derive(Clone)]
# pub struct Baslatici { kuyruk: Sender<Arc<Gorev>>, canli: Arc<AtomicUsize> }
# impl Baslatici {
#     fn kuyruga_koy(&self, gelecek: impl Future<Output = ()> + Send + 'static) {
#         self.canli.fetch_add(1, Ordering::SeqCst);
#         let gelecek = Mutex::new(Some(Box::pin(gelecek) as KutuluGorev));
#         let _ = self.kuyruk.send(Arc::new(Gorev { gelecek, kuyruk: self.kuyruk.clone() }));
#     }
# }
# impl Calistirici {
#     pub fn yeni() -> Self {
#         let (kuyruk, hazir) = mpsc::channel();
#         let baslatici = Baslatici { kuyruk, canli: Arc::new(AtomicUsize::new(0)) };
#         Calistirici { hazir, baslatici, poll_sayisi: 0 }
#     }
#     pub fn baslatici(&self) -> Baslatici { self.baslatici.clone() }
# }
# use std::task::{Context, Poll};
# impl Calistirici {
#     pub fn calistir(&mut self) -> usize {
#         while self.baslatici.canli.load(Ordering::SeqCst) > 0 {
#             let Ok(gorev) = self.hazir.recv() else { break };
#             let mut yuva = gorev.gelecek.lock().expect("kilit zehirlenmemeli");
#             let Some(mut gelecek) = yuva.take() else { continue };
#             let waker = Waker::from(Arc::clone(&gorev));
#             self.poll_sayisi += 1;
#             match gelecek.as_mut().poll(&mut Context::from_waker(&waker)) {
#                 Poll::Pending => *yuva = Some(gelecek),
#                 Poll::Ready(()) => { self.baslatici.canli.fetch_sub(1, Ordering::SeqCst); }
#             }
#         }
#         self.poll_sayisi
#     }
# }
# struct TutamacDurumu<T> { sonuc: Option<T>, bekleyen: Option<Waker> }
# pub struct Tutamac<T> { paylasilan: Arc<Mutex<TutamacDurumu<T>>> }
# impl<T> Tutamac<T> {
#     pub fn sonucu_al(&self) -> Option<T> {
#         self.paylasilan.lock().expect("kilit zehirlenmemeli").sonuc.take()
#     }
# }
# impl<T> Future for Tutamac<T> {
#     type Output = T;
#     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
#         let mut durum = self.paylasilan.lock().expect("kilit zehirlenmemeli");
#         if let Some(deger) = durum.sonuc.take() { return Poll::Ready(deger); }
#         durum.bekleyen = Some(cx.waker().clone());
#         Poll::Pending
#     }
# }
# impl Baslatici {
#     pub fn baslat<F>(&self, gelecek: F) -> Tutamac<F::Output>
#     where F: Future + Send + 'static, F::Output: Send + 'static {
#         let paylasilan = Arc::new(Mutex::new(TutamacDurumu { sonuc: None, bekleyen: None }));
#         let gorev_tarafi = Arc::clone(&paylasilan);
#         self.kuyruga_koy(async move {
#             let deger = gelecek.await;
#             let bekleyen = {
#                 let mut durum = gorev_tarafi.lock().expect("kilit zehirlenmemeli");
#                 durum.sonuc = Some(deger);
#                 durum.bekleyen.take()
#             };
#             if let Some(waker) = bekleyen { waker.wake(); }
#         });
#         Tutamac { paylasilan }
#     }
# }
# pub fn kostur<F>(gelecek: F) -> F::Output
# where F: Future + Send + 'static, F::Output: Send + 'static {
#     let mut calistirici = Calistirici::yeni();
#     let tutamac = calistirici.baslatici().baslat(gelecek);
#     calistirici.calistir();
#     tutamac.sonucu_al().expect("görev bitti ama sonuç yok")
# }
use std::thread;
use std::time::{Duration, Instant};

pub struct Uyku {
    sure: Duration,
    basladi: bool,
    durum: Arc<Mutex<UykuDurumu>>,
}

struct UykuDurumu {
    doldu: bool,
    waker: Option<Waker>,
}

pub fn uyu_ms(ms: u64) -> Uyku {
    let durum = UykuDurumu { doldu: false, waker: None };
    Uyku { sure: Duration::from_millis(ms), basladi: false, durum: Arc::new(Mutex::new(durum)) }
}

impl Future for Uyku {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let bu = self.get_mut();
        let mut durum = bu.durum.lock().expect("kilit zehirlenmemeli");
        if durum.doldu {
            return Poll::Ready(());
        }
        durum.waker = Some(cx.waker().clone()); // her yoklamada güncelle
        drop(durum); // thread başlatmayı kilit altında yapma
        if !bu.basladi {
            bu.basladi = true; // saati ilk yoklamada kur
            let (paylasilan, sure) = (Arc::clone(&bu.durum), bu.sure);
            thread::spawn(move || {
                thread::sleep(sure);
                let waker = {
                    let mut durum = paylasilan.lock().expect("kilit zehirlenmemeli");
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

fn main() {
    let baslangic = Instant::now();
    kostur(uyu_ms(20));
    println!("en az 20 ms sürdü mü? {}", baslangic.elapsed() >= Duration::from_millis(20));
}
```

Çıktı:

```text
en az 20 ms sürdü mü? true
```

Yoklamanın satırlarına yakından bakalım.

**Tembellik.** Saat `uyu_ms` çağrısında değil, ilk yoklamada kurulur. Oluşturulup hiç yoklanmadan düşürülen bir `Uyku` ne bekler ne de thread açar; `drop(uyu_ms(20))` hemen döner.

**Uyandırıcıyı her yoklamada güncellemek.** Görev araya başka bir uyandırıcıyla yoklanmış olabilir, örneğin bir birleştiricinin içine taşınmışsa. `Future` sözleşmesi yalnızca *en son* yoklamada verilen uyandırıcının çağrılmasını ister; eskisini çağırmak işe yaramaz.[^donus] Gereksiz klonu azaltmak isterseniz `Waker::clone_from`, iki uyandırıcı aynı görevi uyandıracaksa kopyalamayı atlar.[^clone-from]

**Kilit dışında thread açmak.** Pahalı bir iş kilit altında yapılmaz.

**`get_mut`.** `Uyku`'nun bütün alanları `Unpin` olduğu için kendisi de `Unpin`'dir; sabitlenmiş olsa bile içinden güvenle `&mut` alınabilir. `Pin::get_mut` yalnızca `Unpin` türlerde vardır.[^get-mut] Kendine referans tutmayan, elle yazılmış future'ların çoğu böyledir.

Bu zamanlayıcının gerçek kazancı yoklama sayısında görülür. On görev birlikte uyusun:

```rust
# use std::future::Future;
# use std::pin::Pin;
# use std::sync::mpsc::Sender;
# use std::sync::{Arc, Mutex};
# use std::task::{Wake, Waker};
# type KutuluGorev = Pin<Box<dyn Future<Output = ()> + Send>>;
# struct Gorev { gelecek: Mutex<Option<KutuluGorev>>, kuyruk: Sender<Arc<Gorev>> }
# impl Wake for Gorev {
#     fn wake(self: Arc<Self>) { let _ = self.kuyruk.clone().send(self); }
# }
# use std::sync::atomic::{AtomicUsize, Ordering};
# use std::sync::mpsc::{self, Receiver};
# pub struct Calistirici { hazir: Receiver<Arc<Gorev>>, baslatici: Baslatici, poll_sayisi: usize }
# #[derive(Clone)]
# pub struct Baslatici { kuyruk: Sender<Arc<Gorev>>, canli: Arc<AtomicUsize> }
# impl Baslatici {
#     fn kuyruga_koy(&self, gelecek: impl Future<Output = ()> + Send + 'static) {
#         self.canli.fetch_add(1, Ordering::SeqCst);
#         let gelecek = Mutex::new(Some(Box::pin(gelecek) as KutuluGorev));
#         let _ = self.kuyruk.send(Arc::new(Gorev { gelecek, kuyruk: self.kuyruk.clone() }));
#     }
# }
# impl Calistirici {
#     pub fn yeni() -> Self {
#         let (kuyruk, hazir) = mpsc::channel();
#         let baslatici = Baslatici { kuyruk, canli: Arc::new(AtomicUsize::new(0)) };
#         Calistirici { hazir, baslatici, poll_sayisi: 0 }
#     }
#     pub fn baslatici(&self) -> Baslatici { self.baslatici.clone() }
# }
# use std::task::{Context, Poll};
# impl Calistirici {
#     pub fn calistir(&mut self) -> usize {
#         while self.baslatici.canli.load(Ordering::SeqCst) > 0 {
#             let Ok(gorev) = self.hazir.recv() else { break };
#             let mut yuva = gorev.gelecek.lock().expect("kilit zehirlenmemeli");
#             let Some(mut gelecek) = yuva.take() else { continue };
#             let waker = Waker::from(Arc::clone(&gorev));
#             self.poll_sayisi += 1;
#             match gelecek.as_mut().poll(&mut Context::from_waker(&waker)) {
#                 Poll::Pending => *yuva = Some(gelecek),
#                 Poll::Ready(()) => { self.baslatici.canli.fetch_sub(1, Ordering::SeqCst); }
#             }
#         }
#         self.poll_sayisi
#     }
# }
# struct TutamacDurumu<T> { sonuc: Option<T>, bekleyen: Option<Waker> }
# pub struct Tutamac<T> { paylasilan: Arc<Mutex<TutamacDurumu<T>>> }
# impl<T> Tutamac<T> {
#     pub fn sonucu_al(&self) -> Option<T> {
#         self.paylasilan.lock().expect("kilit zehirlenmemeli").sonuc.take()
#     }
# }
# impl<T> Future for Tutamac<T> {
#     type Output = T;
#     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
#         let mut durum = self.paylasilan.lock().expect("kilit zehirlenmemeli");
#         if let Some(deger) = durum.sonuc.take() { return Poll::Ready(deger); }
#         durum.bekleyen = Some(cx.waker().clone());
#         Poll::Pending
#     }
# }
# impl Baslatici {
#     pub fn baslat<F>(&self, gelecek: F) -> Tutamac<F::Output>
#     where F: Future + Send + 'static, F::Output: Send + 'static {
#         let paylasilan = Arc::new(Mutex::new(TutamacDurumu { sonuc: None, bekleyen: None }));
#         let gorev_tarafi = Arc::clone(&paylasilan);
#         self.kuyruga_koy(async move {
#             let deger = gelecek.await;
#             let bekleyen = {
#                 let mut durum = gorev_tarafi.lock().expect("kilit zehirlenmemeli");
#                 durum.sonuc = Some(deger);
#                 durum.bekleyen.take()
#             };
#             if let Some(waker) = bekleyen { waker.wake(); }
#         });
#         Tutamac { paylasilan }
#     }
# }
# pub fn kostur<F>(gelecek: F) -> F::Output
# where F: Future + Send + 'static, F::Output: Send + 'static {
#     let mut calistirici = Calistirici::yeni();
#     let tutamac = calistirici.baslatici().baslat(gelecek);
#     calistirici.calistir();
#     tutamac.sonucu_al().expect("görev bitti ama sonuç yok")
# }
# use std::time::Duration;
# pub struct Uyku { sure: Duration, basladi: bool, durum: Arc<Mutex<UykuDurumu>> }
# struct UykuDurumu { doldu: bool, waker: Option<Waker> }
# impl Uyku {
#     pub fn yeni(sure: Duration) -> Self {
#         let durum = Arc::new(Mutex::new(UykuDurumu { doldu: false, waker: None }));
#         Uyku { sure, basladi: false, durum }
#     }
# }
# pub fn uyu_ms(ms: u64) -> Uyku { Uyku::yeni(Duration::from_millis(ms)) }
# impl Future for Uyku {
#     type Output = ();
#     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
#         let bu = self.get_mut();
#         let mut durum = bu.durum.lock().expect("kilit zehirlenmemeli");
#         if durum.doldu { return Poll::Ready(()); }
#         durum.waker = Some(cx.waker().clone());
#         drop(durum);
#         if std::mem::replace(&mut bu.basladi, true) { return Poll::Pending; }
#         let (paylasilan, sure) = (Arc::clone(&bu.durum), bu.sure);
#         std::thread::spawn(move || {
#             std::thread::sleep(sure);
#             let mut durum = paylasilan.lock().expect("kilit zehirlenmemeli");
#             durum.doldu = true;
#             let waker = durum.waker.take();
#             drop(durum);
#             if let Some(waker) = waker { waker.wake(); }
#         });
#         Poll::Pending
#     }
# }
fn main() {
    let mut calistirici = Calistirici::yeni();
    let baslatici = calistirici.baslatici();
    for _ in 0..10 {
        baslatici.baslat(uyu_ms(15));
    }
    println!("toplam yoklama: {}", calistirici.calistir());
}
```

Çıktı:

```text
toplam yoklama: 20
```

Her görev iki kez yoklandı: bir kez başlarken (`Pending`), bir kez süre dolunca (`Ready`). Her turda hepsine bakan bir yürütücü 15 ms boyunca durmadan dönüp binlerce yoklama yapardı.

> [!NOTE]
> Uyku başına bir thread açmak yalnızca öğretici bir kısayoldur: Uyuyan on bin görev on bin thread demek olurdu. Üretimdeki çalışma zamanları bütün süreleri tek bir zamanlayıcı yapısında tutar ve işletim sisteminin olay bildirimiyle birleştirir.[^surucu] İlke aynıdır: Olay olunca `wake` çağrılır.

## ZamanAsimi: iki future'ı yarıştırmak

Masaya bir kum saati koyup işe başladığınızı düşünün. İş kum bitmeden biterse sonucu alırsınız; kum önce biterse işi olduğu yerde bırakırsınız. `ZamanAsimi` iki future'ı, asıl işi ve bir `Uyku`'yu yarıştıran bir *birleştiricidir*:

```rust
# use std::future::Future;
# use std::pin::Pin;
# use std::sync::mpsc::Sender;
# use std::sync::{Arc, Mutex};
# use std::task::{Wake, Waker};
# type KutuluGorev = Pin<Box<dyn Future<Output = ()> + Send>>;
# struct Gorev { gelecek: Mutex<Option<KutuluGorev>>, kuyruk: Sender<Arc<Gorev>> }
# impl Wake for Gorev {
#     fn wake(self: Arc<Self>) { let _ = self.kuyruk.clone().send(self); }
# }
# use std::sync::atomic::{AtomicUsize, Ordering};
# use std::sync::mpsc::{self, Receiver};
# pub struct Calistirici { hazir: Receiver<Arc<Gorev>>, baslatici: Baslatici, poll_sayisi: usize }
# #[derive(Clone)]
# pub struct Baslatici { kuyruk: Sender<Arc<Gorev>>, canli: Arc<AtomicUsize> }
# impl Baslatici {
#     fn kuyruga_koy(&self, gelecek: impl Future<Output = ()> + Send + 'static) {
#         self.canli.fetch_add(1, Ordering::SeqCst);
#         let gelecek = Mutex::new(Some(Box::pin(gelecek) as KutuluGorev));
#         let _ = self.kuyruk.send(Arc::new(Gorev { gelecek, kuyruk: self.kuyruk.clone() }));
#     }
# }
# impl Calistirici {
#     pub fn yeni() -> Self {
#         let (kuyruk, hazir) = mpsc::channel();
#         let baslatici = Baslatici { kuyruk, canli: Arc::new(AtomicUsize::new(0)) };
#         Calistirici { hazir, baslatici, poll_sayisi: 0 }
#     }
#     pub fn baslatici(&self) -> Baslatici { self.baslatici.clone() }
# }
# use std::task::{Context, Poll};
# impl Calistirici {
#     pub fn calistir(&mut self) -> usize {
#         while self.baslatici.canli.load(Ordering::SeqCst) > 0 {
#             let Ok(gorev) = self.hazir.recv() else { break };
#             let mut yuva = gorev.gelecek.lock().expect("kilit zehirlenmemeli");
#             let Some(mut gelecek) = yuva.take() else { continue };
#             let waker = Waker::from(Arc::clone(&gorev));
#             self.poll_sayisi += 1;
#             match gelecek.as_mut().poll(&mut Context::from_waker(&waker)) {
#                 Poll::Pending => *yuva = Some(gelecek),
#                 Poll::Ready(()) => { self.baslatici.canli.fetch_sub(1, Ordering::SeqCst); }
#             }
#         }
#         self.poll_sayisi
#     }
# }
# struct TutamacDurumu<T> { sonuc: Option<T>, bekleyen: Option<Waker> }
# pub struct Tutamac<T> { paylasilan: Arc<Mutex<TutamacDurumu<T>>> }
# impl<T> Tutamac<T> {
#     pub fn sonucu_al(&self) -> Option<T> {
#         self.paylasilan.lock().expect("kilit zehirlenmemeli").sonuc.take()
#     }
# }
# impl<T> Future for Tutamac<T> {
#     type Output = T;
#     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
#         let mut durum = self.paylasilan.lock().expect("kilit zehirlenmemeli");
#         if let Some(deger) = durum.sonuc.take() { return Poll::Ready(deger); }
#         durum.bekleyen = Some(cx.waker().clone());
#         Poll::Pending
#     }
# }
# impl Baslatici {
#     pub fn baslat<F>(&self, gelecek: F) -> Tutamac<F::Output>
#     where F: Future + Send + 'static, F::Output: Send + 'static {
#         let paylasilan = Arc::new(Mutex::new(TutamacDurumu { sonuc: None, bekleyen: None }));
#         let gorev_tarafi = Arc::clone(&paylasilan);
#         self.kuyruga_koy(async move {
#             let deger = gelecek.await;
#             let bekleyen = {
#                 let mut durum = gorev_tarafi.lock().expect("kilit zehirlenmemeli");
#                 durum.sonuc = Some(deger);
#                 durum.bekleyen.take()
#             };
#             if let Some(waker) = bekleyen { waker.wake(); }
#         });
#         Tutamac { paylasilan }
#     }
# }
# pub fn kostur<F>(gelecek: F) -> F::Output
# where F: Future + Send + 'static, F::Output: Send + 'static {
#     let mut calistirici = Calistirici::yeni();
#     let tutamac = calistirici.baslatici().baslat(gelecek);
#     calistirici.calistir();
#     tutamac.sonucu_al().expect("görev bitti ama sonuç yok")
# }
# use std::time::Duration;
# pub struct Uyku { sure: Duration, basladi: bool, durum: Arc<Mutex<UykuDurumu>> }
# struct UykuDurumu { doldu: bool, waker: Option<Waker> }
# impl Uyku {
#     pub fn yeni(sure: Duration) -> Self {
#         let durum = Arc::new(Mutex::new(UykuDurumu { doldu: false, waker: None }));
#         Uyku { sure, basladi: false, durum }
#     }
# }
# pub fn uyu_ms(ms: u64) -> Uyku { Uyku::yeni(Duration::from_millis(ms)) }
# impl Future for Uyku {
#     type Output = ();
#     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
#         let bu = self.get_mut();
#         let mut durum = bu.durum.lock().expect("kilit zehirlenmemeli");
#         if durum.doldu { return Poll::Ready(()); }
#         durum.waker = Some(cx.waker().clone());
#         drop(durum);
#         if std::mem::replace(&mut bu.basladi, true) { return Poll::Pending; }
#         let (paylasilan, sure) = (Arc::clone(&bu.durum), bu.sure);
#         std::thread::spawn(move || {
#             std::thread::sleep(sure);
#             let mut durum = paylasilan.lock().expect("kilit zehirlenmemeli");
#             durum.doldu = true;
#             let waker = durum.waker.take();
#             drop(durum);
#             if let Some(waker) = waker { waker.wake(); }
#         });
#         Poll::Pending
#     }
# }
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ZamanAsimiHatasi;

pub struct ZamanAsimi<T> {
    ic: Pin<Box<dyn Future<Output = T> + Send>>,
    sayac: Uyku,
}

pub fn zaman_asimi<F>(sure: Duration, gelecek: F) -> ZamanAsimi<F::Output>
where
    F: Future + Send + 'static,
{
    ZamanAsimi { ic: Box::pin(gelecek), sayac: Uyku::yeni(sure) }
}

impl<T> Future for ZamanAsimi<T> {
    type Output = Result<T, ZamanAsimiHatasi>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let bu = self.get_mut(); // iki alan da Unpin
        // Önce asıl iş: İkisi de hazırsa sonucu yeğleriz.
        if let Poll::Ready(deger) = bu.ic.as_mut().poll(cx) {
            return Poll::Ready(Ok(deger));
        }
        if Pin::new(&mut bu.sayac).poll(cx).is_ready() {
            return Poll::Ready(Err(ZamanAsimiHatasi)); // iç future bu struct ile düşecek
        }
        Poll::Pending
    }
}

fn main() {
    let hizli = kostur(zaman_asimi(Duration::from_millis(200), async {
        uyu_ms(5).await;
        "bitti"
    }));
    let yavas = kostur(zaman_asimi(Duration::from_millis(10), async {
        uyu_ms(300).await;
        "bitti"
    }));
    println!("{hizli:?}");
    println!("{yavas:?}");
}
```

Çıktı:

```text
Ok("bitti")
Err(ZamanAsimiHatasi)
```

Süre, `ZamanAsimi` ilk kez yoklandığında işlemeye başlar, çünkü içindeki `Uyku` da tembeldir.

### İptal, düşürmektir

Async'te "işi bırak" diyen ayrı bir düğme yoktur: Future'ı yoklamayı kesip düşürürsünüz, iş orada biter.[^iptal] İçerideki yerel değişkenler de düşer; bunu bir iz bırakan türle görelim:

```rust
# use std::future::Future;
# use std::pin::Pin;
# use std::sync::mpsc::Sender;
# use std::sync::{Arc, Mutex};
# use std::task::{Wake, Waker};
# type KutuluGorev = Pin<Box<dyn Future<Output = ()> + Send>>;
# struct Gorev { gelecek: Mutex<Option<KutuluGorev>>, kuyruk: Sender<Arc<Gorev>> }
# impl Wake for Gorev {
#     fn wake(self: Arc<Self>) { let _ = self.kuyruk.clone().send(self); }
# }
# use std::sync::atomic::{AtomicUsize, Ordering};
# use std::sync::mpsc::{self, Receiver};
# pub struct Calistirici { hazir: Receiver<Arc<Gorev>>, baslatici: Baslatici, poll_sayisi: usize }
# #[derive(Clone)]
# pub struct Baslatici { kuyruk: Sender<Arc<Gorev>>, canli: Arc<AtomicUsize> }
# impl Baslatici {
#     fn kuyruga_koy(&self, gelecek: impl Future<Output = ()> + Send + 'static) {
#         self.canli.fetch_add(1, Ordering::SeqCst);
#         let gelecek = Mutex::new(Some(Box::pin(gelecek) as KutuluGorev));
#         let _ = self.kuyruk.send(Arc::new(Gorev { gelecek, kuyruk: self.kuyruk.clone() }));
#     }
# }
# impl Calistirici {
#     pub fn yeni() -> Self {
#         let (kuyruk, hazir) = mpsc::channel();
#         let baslatici = Baslatici { kuyruk, canli: Arc::new(AtomicUsize::new(0)) };
#         Calistirici { hazir, baslatici, poll_sayisi: 0 }
#     }
#     pub fn baslatici(&self) -> Baslatici { self.baslatici.clone() }
# }
# use std::task::{Context, Poll};
# impl Calistirici {
#     pub fn calistir(&mut self) -> usize {
#         while self.baslatici.canli.load(Ordering::SeqCst) > 0 {
#             let Ok(gorev) = self.hazir.recv() else { break };
#             let mut yuva = gorev.gelecek.lock().expect("kilit zehirlenmemeli");
#             let Some(mut gelecek) = yuva.take() else { continue };
#             let waker = Waker::from(Arc::clone(&gorev));
#             self.poll_sayisi += 1;
#             match gelecek.as_mut().poll(&mut Context::from_waker(&waker)) {
#                 Poll::Pending => *yuva = Some(gelecek),
#                 Poll::Ready(()) => { self.baslatici.canli.fetch_sub(1, Ordering::SeqCst); }
#             }
#         }
#         self.poll_sayisi
#     }
# }
# struct TutamacDurumu<T> { sonuc: Option<T>, bekleyen: Option<Waker> }
# pub struct Tutamac<T> { paylasilan: Arc<Mutex<TutamacDurumu<T>>> }
# impl<T> Tutamac<T> {
#     pub fn sonucu_al(&self) -> Option<T> {
#         self.paylasilan.lock().expect("kilit zehirlenmemeli").sonuc.take()
#     }
# }
# impl<T> Future for Tutamac<T> {
#     type Output = T;
#     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
#         let mut durum = self.paylasilan.lock().expect("kilit zehirlenmemeli");
#         if let Some(deger) = durum.sonuc.take() { return Poll::Ready(deger); }
#         durum.bekleyen = Some(cx.waker().clone());
#         Poll::Pending
#     }
# }
# impl Baslatici {
#     pub fn baslat<F>(&self, gelecek: F) -> Tutamac<F::Output>
#     where F: Future + Send + 'static, F::Output: Send + 'static {
#         let paylasilan = Arc::new(Mutex::new(TutamacDurumu { sonuc: None, bekleyen: None }));
#         let gorev_tarafi = Arc::clone(&paylasilan);
#         self.kuyruga_koy(async move {
#             let deger = gelecek.await;
#             let bekleyen = {
#                 let mut durum = gorev_tarafi.lock().expect("kilit zehirlenmemeli");
#                 durum.sonuc = Some(deger);
#                 durum.bekleyen.take()
#             };
#             if let Some(waker) = bekleyen { waker.wake(); }
#         });
#         Tutamac { paylasilan }
#     }
# }
# pub fn kostur<F>(gelecek: F) -> F::Output
# where F: Future + Send + 'static, F::Output: Send + 'static {
#     let mut calistirici = Calistirici::yeni();
#     let tutamac = calistirici.baslatici().baslat(gelecek);
#     calistirici.calistir();
#     tutamac.sonucu_al().expect("görev bitti ama sonuç yok")
# }
# use std::time::Duration;
# pub struct Uyku { sure: Duration, basladi: bool, durum: Arc<Mutex<UykuDurumu>> }
# struct UykuDurumu { doldu: bool, waker: Option<Waker> }
# impl Uyku {
#     pub fn yeni(sure: Duration) -> Self {
#         let durum = Arc::new(Mutex::new(UykuDurumu { doldu: false, waker: None }));
#         Uyku { sure, basladi: false, durum }
#     }
# }
# pub fn uyu_ms(ms: u64) -> Uyku { Uyku::yeni(Duration::from_millis(ms)) }
# impl Future for Uyku {
#     type Output = ();
#     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
#         let bu = self.get_mut();
#         let mut durum = bu.durum.lock().expect("kilit zehirlenmemeli");
#         if durum.doldu { return Poll::Ready(()); }
#         durum.waker = Some(cx.waker().clone());
#         drop(durum);
#         if std::mem::replace(&mut bu.basladi, true) { return Poll::Pending; }
#         let (paylasilan, sure) = (Arc::clone(&bu.durum), bu.sure);
#         std::thread::spawn(move || {
#             std::thread::sleep(sure);
#             let mut durum = paylasilan.lock().expect("kilit zehirlenmemeli");
#             durum.doldu = true;
#             let waker = durum.waker.take();
#             drop(durum);
#             if let Some(waker) = waker { waker.wake(); }
#         });
#         Poll::Pending
#     }
# }
# #[derive(Debug, PartialEq, Eq, Clone, Copy)]
# pub struct ZamanAsimiHatasi;
# pub struct ZamanAsimi<T> { ic: Pin<Box<dyn Future<Output = T> + Send>>, sayac: Uyku }
# pub fn zaman_asimi<F>(sure: Duration, gelecek: F) -> ZamanAsimi<F::Output>
# where F: Future + Send + 'static {
#     ZamanAsimi { ic: Box::pin(gelecek), sayac: Uyku::yeni(sure) }
# }
# impl<T> Future for ZamanAsimi<T> {
#     type Output = Result<T, ZamanAsimiHatasi>;
#     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
#         let bu = self.get_mut();
#         if let Poll::Ready(deger) = bu.ic.as_mut().poll(cx) { return Poll::Ready(Ok(deger)); }
#         Pin::new(&mut bu.sayac).poll(cx).map(|()| Err(ZamanAsimiHatasi))
#     }
# }
struct Iz(&'static str);

impl Drop for Iz {
    fn drop(&mut self) {
        println!("{} düşürüldü", self.0);
    }
}

fn main() {
    let sonuc = kostur(zaman_asimi(Duration::from_millis(10), async {
        let _iz = Iz("yarım kalan iş");
        println!("iş başladı");
        uyu_ms(300).await;
        println!("bu satıra gelinmez");
    }));
    println!("{sonuc:?}");
}
```

Çıktı:

```text
iş başladı
yarım kalan iş düşürüldü
Err(ZamanAsimiHatasi)
```

Derleyici, askıda kalmış bir durum makinesi düşürülürken o noktada canlı olan yerelleri düşürecek kodu üretir.[^drop-glue] Bu yüzden async kod, her `.await` noktasında yarıda kalabileceğini hesaba katmalıdır: İki `.await` arasında "yarısı yapılmış" bir iş bırakıyorsanız iptal onu öyle bırakır.

İptalin bizim çalışma zamanımızda görünmeyen bir yüzü de vardır. Düşen `Uyku`'nun yardımcı thread'i durdurulmaz; süresini doldurur ve geç bir `wake` gönderir. Bu zararsızdır: Çalıştırıcı kapanmışsa gönderim boşa gider, açıksa bitmiş görevin yuvası boş olduğu için `continue` ile geçilir.

### İç future'a nasıl ulaşılır?

Bir birleştirici, içindeki future'ı yoklamak için ona `Pin<&mut _>` vermelidir. Elinizde ise `Pin<&mut ZamanAsimi>` vardır. Alanlara inmenin üç güvenli yolu bulunur.

**1. Alanı kendi başına sabitlemek.** Yukarıdaki çözüm budur: `Pin<Box<dyn Future>>` alanı `Unpin`'dir, çünkü taşınan kutudur. Bütün alanlar `Unpin` olunca struct da `Unpin` olur ve `get_mut` kullanılabilir. Bedeli bir heap tahsisidir.

**2. Pin projeksiyonu.** `Pin<&mut Struct>`'tan `Pin<&mut Alan>` üretmek tahsis gerektirmez, ama doğruluğu elle kanıtlamak, yani `unsafe` yazmak gerekir.[^izdusum] Uygulamada `pin-project` ya da `pin-project-lite` crate'leri bunu güvenli bir makroyla yapar;[^pin-project] kütüphane yazarken yeğlenen yol budur.

**3. `pin!` ve `poll_fn`.** Adlandırılmış bir tür gerekmiyorsa future'ları bir `async fn`'in içinde sabitleyip `poll_fn` ile yoklayabilirsiniz. Ne tahsis ne `unsafe` gerekir:[^poll-fn]

```rust
# use std::future::Future;
# use std::pin::Pin;
# use std::sync::mpsc::Sender;
# use std::sync::{Arc, Mutex};
# use std::task::{Wake, Waker};
# type KutuluGorev = Pin<Box<dyn Future<Output = ()> + Send>>;
# struct Gorev { gelecek: Mutex<Option<KutuluGorev>>, kuyruk: Sender<Arc<Gorev>> }
# impl Wake for Gorev {
#     fn wake(self: Arc<Self>) { let _ = self.kuyruk.clone().send(self); }
# }
# use std::sync::atomic::{AtomicUsize, Ordering};
# use std::sync::mpsc::{self, Receiver};
# pub struct Calistirici { hazir: Receiver<Arc<Gorev>>, baslatici: Baslatici, poll_sayisi: usize }
# #[derive(Clone)]
# pub struct Baslatici { kuyruk: Sender<Arc<Gorev>>, canli: Arc<AtomicUsize> }
# impl Baslatici {
#     fn kuyruga_koy(&self, gelecek: impl Future<Output = ()> + Send + 'static) {
#         self.canli.fetch_add(1, Ordering::SeqCst);
#         let gelecek = Mutex::new(Some(Box::pin(gelecek) as KutuluGorev));
#         let _ = self.kuyruk.send(Arc::new(Gorev { gelecek, kuyruk: self.kuyruk.clone() }));
#     }
# }
# impl Calistirici {
#     pub fn yeni() -> Self {
#         let (kuyruk, hazir) = mpsc::channel();
#         let baslatici = Baslatici { kuyruk, canli: Arc::new(AtomicUsize::new(0)) };
#         Calistirici { hazir, baslatici, poll_sayisi: 0 }
#     }
#     pub fn baslatici(&self) -> Baslatici { self.baslatici.clone() }
# }
# use std::task::{Context, Poll};
# impl Calistirici {
#     pub fn calistir(&mut self) -> usize {
#         while self.baslatici.canli.load(Ordering::SeqCst) > 0 {
#             let Ok(gorev) = self.hazir.recv() else { break };
#             let mut yuva = gorev.gelecek.lock().expect("kilit zehirlenmemeli");
#             let Some(mut gelecek) = yuva.take() else { continue };
#             let waker = Waker::from(Arc::clone(&gorev));
#             self.poll_sayisi += 1;
#             match gelecek.as_mut().poll(&mut Context::from_waker(&waker)) {
#                 Poll::Pending => *yuva = Some(gelecek),
#                 Poll::Ready(()) => { self.baslatici.canli.fetch_sub(1, Ordering::SeqCst); }
#             }
#         }
#         self.poll_sayisi
#     }
# }
# struct TutamacDurumu<T> { sonuc: Option<T>, bekleyen: Option<Waker> }
# pub struct Tutamac<T> { paylasilan: Arc<Mutex<TutamacDurumu<T>>> }
# impl<T> Tutamac<T> {
#     pub fn sonucu_al(&self) -> Option<T> {
#         self.paylasilan.lock().expect("kilit zehirlenmemeli").sonuc.take()
#     }
# }
# impl<T> Future for Tutamac<T> {
#     type Output = T;
#     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
#         let mut durum = self.paylasilan.lock().expect("kilit zehirlenmemeli");
#         if let Some(deger) = durum.sonuc.take() { return Poll::Ready(deger); }
#         durum.bekleyen = Some(cx.waker().clone());
#         Poll::Pending
#     }
# }
# impl Baslatici {
#     pub fn baslat<F>(&self, gelecek: F) -> Tutamac<F::Output>
#     where F: Future + Send + 'static, F::Output: Send + 'static {
#         let paylasilan = Arc::new(Mutex::new(TutamacDurumu { sonuc: None, bekleyen: None }));
#         let gorev_tarafi = Arc::clone(&paylasilan);
#         self.kuyruga_koy(async move {
#             let deger = gelecek.await;
#             let bekleyen = {
#                 let mut durum = gorev_tarafi.lock().expect("kilit zehirlenmemeli");
#                 durum.sonuc = Some(deger);
#                 durum.bekleyen.take()
#             };
#             if let Some(waker) = bekleyen { waker.wake(); }
#         });
#         Tutamac { paylasilan }
#     }
# }
# pub fn kostur<F>(gelecek: F) -> F::Output
# where F: Future + Send + 'static, F::Output: Send + 'static {
#     let mut calistirici = Calistirici::yeni();
#     let tutamac = calistirici.baslatici().baslat(gelecek);
#     calistirici.calistir();
#     tutamac.sonucu_al().expect("görev bitti ama sonuç yok")
# }
# use std::time::Duration;
# pub struct Uyku { sure: Duration, basladi: bool, durum: Arc<Mutex<UykuDurumu>> }
# struct UykuDurumu { doldu: bool, waker: Option<Waker> }
# impl Uyku {
#     pub fn yeni(sure: Duration) -> Self {
#         let durum = Arc::new(Mutex::new(UykuDurumu { doldu: false, waker: None }));
#         Uyku { sure, basladi: false, durum }
#     }
# }
# pub fn uyu_ms(ms: u64) -> Uyku { Uyku::yeni(Duration::from_millis(ms)) }
# impl Future for Uyku {
#     type Output = ();
#     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
#         let bu = self.get_mut();
#         let mut durum = bu.durum.lock().expect("kilit zehirlenmemeli");
#         if durum.doldu { return Poll::Ready(()); }
#         durum.waker = Some(cx.waker().clone());
#         drop(durum);
#         if std::mem::replace(&mut bu.basladi, true) { return Poll::Pending; }
#         let (paylasilan, sure) = (Arc::clone(&bu.durum), bu.sure);
#         std::thread::spawn(move || {
#             std::thread::sleep(sure);
#             let mut durum = paylasilan.lock().expect("kilit zehirlenmemeli");
#             durum.doldu = true;
#             let waker = durum.waker.take();
#             drop(durum);
#             if let Some(waker) = waker { waker.wake(); }
#         });
#         Poll::Pending
#     }
# }
# #[derive(Debug, PartialEq, Eq, Clone, Copy)]
# pub struct ZamanAsimiHatasi;
use std::future::poll_fn;
use std::pin::pin;

pub async fn zaman_asimi_kutusuz<F: Future>(
    sure: Duration,
    gelecek: F,
) -> Result<F::Output, ZamanAsimiHatasi> {
    let mut ic = pin!(gelecek);
    let mut sayac = pin!(Uyku::yeni(sure));
    poll_fn(|cx| {
        if let Poll::Ready(deger) = ic.as_mut().poll(cx) {
            return Poll::Ready(Ok(deger));
        }
        sayac.as_mut().poll(cx).map(|()| Err(ZamanAsimiHatasi))
    })
    .await
}

fn main() {
    let sonuc = kostur(zaman_asimi_kutusuz(Duration::from_millis(200), async { 42 }));
    println!("{sonuc:?}");
}
```

Çıktı:

```text
Ok(42)
```

Bunlardan daha kısa görünen bir yol derlenmez: Alanı jenerik bırakıp `Pin::new` ile sarmalamak.

```rust,compile_fail,E0277
# use std::future::Future;
# use std::pin::Pin;
# use std::sync::mpsc::Sender;
# use std::sync::{Arc, Mutex};
# use std::task::{Wake, Waker};
# type KutuluGorev = Pin<Box<dyn Future<Output = ()> + Send>>;
# struct Gorev { gelecek: Mutex<Option<KutuluGorev>>, kuyruk: Sender<Arc<Gorev>> }
# impl Wake for Gorev {
#     fn wake(self: Arc<Self>) { let _ = self.kuyruk.clone().send(self); }
# }
# use std::sync::atomic::{AtomicUsize, Ordering};
# use std::sync::mpsc::{self, Receiver};
# pub struct Calistirici { hazir: Receiver<Arc<Gorev>>, baslatici: Baslatici, poll_sayisi: usize }
# #[derive(Clone)]
# pub struct Baslatici { kuyruk: Sender<Arc<Gorev>>, canli: Arc<AtomicUsize> }
# impl Baslatici {
#     fn kuyruga_koy(&self, gelecek: impl Future<Output = ()> + Send + 'static) {
#         self.canli.fetch_add(1, Ordering::SeqCst);
#         let gelecek = Mutex::new(Some(Box::pin(gelecek) as KutuluGorev));
#         let _ = self.kuyruk.send(Arc::new(Gorev { gelecek, kuyruk: self.kuyruk.clone() }));
#     }
# }
# impl Calistirici {
#     pub fn yeni() -> Self {
#         let (kuyruk, hazir) = mpsc::channel();
#         let baslatici = Baslatici { kuyruk, canli: Arc::new(AtomicUsize::new(0)) };
#         Calistirici { hazir, baslatici, poll_sayisi: 0 }
#     }
#     pub fn baslatici(&self) -> Baslatici { self.baslatici.clone() }
# }
# use std::task::{Context, Poll};
# impl Calistirici {
#     pub fn calistir(&mut self) -> usize {
#         while self.baslatici.canli.load(Ordering::SeqCst) > 0 {
#             let Ok(gorev) = self.hazir.recv() else { break };
#             let mut yuva = gorev.gelecek.lock().expect("kilit zehirlenmemeli");
#             let Some(mut gelecek) = yuva.take() else { continue };
#             let waker = Waker::from(Arc::clone(&gorev));
#             self.poll_sayisi += 1;
#             match gelecek.as_mut().poll(&mut Context::from_waker(&waker)) {
#                 Poll::Pending => *yuva = Some(gelecek),
#                 Poll::Ready(()) => { self.baslatici.canli.fetch_sub(1, Ordering::SeqCst); }
#             }
#         }
#         self.poll_sayisi
#     }
# }
# struct TutamacDurumu<T> { sonuc: Option<T>, bekleyen: Option<Waker> }
# pub struct Tutamac<T> { paylasilan: Arc<Mutex<TutamacDurumu<T>>> }
# impl<T> Tutamac<T> {
#     pub fn sonucu_al(&self) -> Option<T> {
#         self.paylasilan.lock().expect("kilit zehirlenmemeli").sonuc.take()
#     }
# }
# impl<T> Future for Tutamac<T> {
#     type Output = T;
#     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
#         let mut durum = self.paylasilan.lock().expect("kilit zehirlenmemeli");
#         if let Some(deger) = durum.sonuc.take() { return Poll::Ready(deger); }
#         durum.bekleyen = Some(cx.waker().clone());
#         Poll::Pending
#     }
# }
# impl Baslatici {
#     pub fn baslat<F>(&self, gelecek: F) -> Tutamac<F::Output>
#     where F: Future + Send + 'static, F::Output: Send + 'static {
#         let paylasilan = Arc::new(Mutex::new(TutamacDurumu { sonuc: None, bekleyen: None }));
#         let gorev_tarafi = Arc::clone(&paylasilan);
#         self.kuyruga_koy(async move {
#             let deger = gelecek.await;
#             let bekleyen = {
#                 let mut durum = gorev_tarafi.lock().expect("kilit zehirlenmemeli");
#                 durum.sonuc = Some(deger);
#                 durum.bekleyen.take()
#             };
#             if let Some(waker) = bekleyen { waker.wake(); }
#         });
#         Tutamac { paylasilan }
#     }
# }
# pub fn kostur<F>(gelecek: F) -> F::Output
# where F: Future + Send + 'static, F::Output: Send + 'static {
#     let mut calistirici = Calistirici::yeni();
#     let tutamac = calistirici.baslatici().baslat(gelecek);
#     calistirici.calistir();
#     tutamac.sonucu_al().expect("görev bitti ama sonuç yok")
# }
# use std::time::Duration;
# pub struct Uyku { sure: Duration, basladi: bool, durum: Arc<Mutex<UykuDurumu>> }
# struct UykuDurumu { doldu: bool, waker: Option<Waker> }
# impl Uyku {
#     pub fn yeni(sure: Duration) -> Self {
#         let durum = Arc::new(Mutex::new(UykuDurumu { doldu: false, waker: None }));
#         Uyku { sure, basladi: false, durum }
#     }
# }
# pub fn uyu_ms(ms: u64) -> Uyku { Uyku::yeni(Duration::from_millis(ms)) }
# impl Future for Uyku {
#     type Output = ();
#     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
#         let bu = self.get_mut();
#         let mut durum = bu.durum.lock().expect("kilit zehirlenmemeli");
#         if durum.doldu { return Poll::Ready(()); }
#         durum.waker = Some(cx.waker().clone());
#         drop(durum);
#         if std::mem::replace(&mut bu.basladi, true) { return Poll::Pending; }
#         let (paylasilan, sure) = (Arc::clone(&bu.durum), bu.sure);
#         std::thread::spawn(move || {
#             std::thread::sleep(sure);
#             let mut durum = paylasilan.lock().expect("kilit zehirlenmemeli");
#             durum.doldu = true;
#             let waker = durum.waker.take();
#             drop(durum);
#             if let Some(waker) = waker { waker.wake(); }
#         });
#         Poll::Pending
#     }
# }
# pub struct ZamanAsimiHatasi;
pub struct ZamanAsimi<F> {
    ic: F,
    sayac: Uyku,
}

impl<F: Future> Future for ZamanAsimi<F> {
    type Output = Result<F::Output, ZamanAsimiHatasi>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.ic).poll(cx) {
            Poll::Ready(deger) => Poll::Ready(Ok(deger)),
            Poll::Pending => Poll::Pending,
        }
    }
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0277]: `F` cannot be unpinned
  = note: consider using the `pin!` macro
```

`Pin::new` yalnızca `F: Unpin` ise kullanılabilir;[^pin-new] `async` blokların ürettiği future'lar ise `Unpin` değildir.

## Geri basınç

29.1'deki sınırlı kuyruk, dolunca üreticiyi bekletiyordu. Çalışma zamanımızın hazır kuyruğu ise sınırsız bir kanaldır. Önce bunun neden zorunlu olduğuna bakalım.

### Hazır kuyruğu neden sınırsız?

Kuyruğun tek alıcısı çalıştırıcının kendisidir. Bir görev yoklanırken bir başkası uyandırılırsa `wake`, *aynı thread'den* kuyruğa yazar. Kuyruk sınırlı ve dolu olsaydı bu yazma, yer açacak tek thread'i bekletirdi. Kapasitesi 1 olan bir kanalda durumu görelim; `send` takılacağı için `try_send` kullanıyoruz:

```rust
use std::sync::mpsc::{self, TrySendError};

fn main() {
    let (kuyruk, hazir) = mpsc::sync_channel(1);
    kuyruk.send("A görevi").expect("alıcı açık");

    // Kuyruğu boşaltacak olan da bu thread; şimdi B uyandırılıyor.
    match kuyruk.try_send("B görevi") {
        Err(TrySendError::Full(gorev)) => println!("kuyruk dolu, {gorev} eklenemedi"),
        sonuc => println!("{sonuc:?}"),
    }
    println!("sıradaki: {:?}", hazir.recv());
}
```

Çıktı:

```text
kuyruk dolu, B görevi eklenemedi
sıradaki: Ok("A görevi")
```

`send` yazsaydık thread kendi kendini bekleyecekti.[^sync-channel] Kuyruğun boyu zaten iki şeyle sınırlıdır: canlı görev sayısı ve görev başına uyandırma sayısı. Uyandırmalar birleştirilebilir; sözleşme, arka arkaya gelen uyandırmaların en az bir yoklamaya yol açmasını ister, her biri için ayrı bir yoklama istemez.[^wake-birlesme] Bizim sürümümüz birleştirme yapmaz: İki kez uyandırılan görev kuyruğa iki kez girer. İkinci kopya zararsızdır; görev sürüyorsa fazladan bir yoklama olur, bittiyse `continue` ile geçilir.

Demek ki geri basıncın yeri hazır kuyruğu değildir. İki yerde uygulanır: görevler arasında taşınan *veride* ve aynı anda yürüyen *iş sayısında*.

### Dolunca bekleyen kuyruk

29.1'deki `SinirliKuyruk` bekleyen thread'i uyutuyordu. Async'te bekleyen *görev* kenara çekilmeli, thread serbest kalmalıdır. Yapı aynıdır; `Condvar`'ın yerini saklanmış uyandırıcılar alır:

```rust
# use std::future::Future;
# use std::pin::Pin;
# use std::sync::mpsc::Sender;
# use std::sync::{Arc, Mutex};
# use std::task::{Wake, Waker};
# type KutuluGorev = Pin<Box<dyn Future<Output = ()> + Send>>;
# struct Gorev { gelecek: Mutex<Option<KutuluGorev>>, kuyruk: Sender<Arc<Gorev>> }
# impl Wake for Gorev {
#     fn wake(self: Arc<Self>) { let _ = self.kuyruk.clone().send(self); }
# }
# use std::sync::atomic::{AtomicUsize, Ordering};
# use std::sync::mpsc::{self, Receiver};
# pub struct Calistirici { hazir: Receiver<Arc<Gorev>>, baslatici: Baslatici, poll_sayisi: usize }
# #[derive(Clone)]
# pub struct Baslatici { kuyruk: Sender<Arc<Gorev>>, canli: Arc<AtomicUsize> }
# impl Baslatici {
#     fn kuyruga_koy(&self, gelecek: impl Future<Output = ()> + Send + 'static) {
#         self.canli.fetch_add(1, Ordering::SeqCst);
#         let gelecek = Mutex::new(Some(Box::pin(gelecek) as KutuluGorev));
#         let _ = self.kuyruk.send(Arc::new(Gorev { gelecek, kuyruk: self.kuyruk.clone() }));
#     }
# }
# impl Calistirici {
#     pub fn yeni() -> Self {
#         let (kuyruk, hazir) = mpsc::channel();
#         let baslatici = Baslatici { kuyruk, canli: Arc::new(AtomicUsize::new(0)) };
#         Calistirici { hazir, baslatici, poll_sayisi: 0 }
#     }
#     pub fn baslatici(&self) -> Baslatici { self.baslatici.clone() }
# }
# use std::task::{Context, Poll};
# impl Calistirici {
#     pub fn calistir(&mut self) -> usize {
#         while self.baslatici.canli.load(Ordering::SeqCst) > 0 {
#             let Ok(gorev) = self.hazir.recv() else { break };
#             let mut yuva = gorev.gelecek.lock().expect("kilit zehirlenmemeli");
#             let Some(mut gelecek) = yuva.take() else { continue };
#             let waker = Waker::from(Arc::clone(&gorev));
#             self.poll_sayisi += 1;
#             match gelecek.as_mut().poll(&mut Context::from_waker(&waker)) {
#                 Poll::Pending => *yuva = Some(gelecek),
#                 Poll::Ready(()) => { self.baslatici.canli.fetch_sub(1, Ordering::SeqCst); }
#             }
#         }
#         self.poll_sayisi
#     }
# }
# struct TutamacDurumu<T> { sonuc: Option<T>, bekleyen: Option<Waker> }
# pub struct Tutamac<T> { paylasilan: Arc<Mutex<TutamacDurumu<T>>> }
# impl<T> Tutamac<T> {
#     pub fn sonucu_al(&self) -> Option<T> {
#         self.paylasilan.lock().expect("kilit zehirlenmemeli").sonuc.take()
#     }
# }
# impl<T> Future for Tutamac<T> {
#     type Output = T;
#     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
#         let mut durum = self.paylasilan.lock().expect("kilit zehirlenmemeli");
#         if let Some(deger) = durum.sonuc.take() { return Poll::Ready(deger); }
#         durum.bekleyen = Some(cx.waker().clone());
#         Poll::Pending
#     }
# }
# impl Baslatici {
#     pub fn baslat<F>(&self, gelecek: F) -> Tutamac<F::Output>
#     where F: Future + Send + 'static, F::Output: Send + 'static {
#         let paylasilan = Arc::new(Mutex::new(TutamacDurumu { sonuc: None, bekleyen: None }));
#         let gorev_tarafi = Arc::clone(&paylasilan);
#         self.kuyruga_koy(async move {
#             let deger = gelecek.await;
#             let bekleyen = {
#                 let mut durum = gorev_tarafi.lock().expect("kilit zehirlenmemeli");
#                 durum.sonuc = Some(deger);
#                 durum.bekleyen.take()
#             };
#             if let Some(waker) = bekleyen { waker.wake(); }
#         });
#         Tutamac { paylasilan }
#     }
# }
# pub fn kostur<F>(gelecek: F) -> F::Output
# where F: Future + Send + 'static, F::Output: Send + 'static {
#     let mut calistirici = Calistirici::yeni();
#     let tutamac = calistirici.baslatici().baslat(gelecek);
#     calistirici.calistir();
#     tutamac.sonucu_al().expect("görev bitti ama sonuç yok")
# }
use std::collections::VecDeque;
use std::future::poll_fn;

pub struct AsyncKuyruk<T> {
    durum: Mutex<AsyncDurum<T>>,
    kapasite: usize,
}

struct AsyncDurum<T> {
    ogeler: VecDeque<T>,
    bekleyenler: Vec<Waker>, // hem koyan hem alan taraf
}

impl<T> AsyncKuyruk<T> {
    pub fn yeni(kapasite: usize) -> Self {
        let durum = AsyncDurum { ogeler: VecDeque::new(), bekleyenler: Vec::new() };
        AsyncKuyruk { durum: Mutex::new(durum), kapasite: kapasite.max(1) }
    }

    // `dene` bir değer verene dek bekler; durum değişince herkesi uyandırır.
    async fn bekle<R>(&self, mut dene: impl FnMut(&mut VecDeque<T>) -> Option<R>) -> R {
        poll_fn(|cx| {
            let mut durum = self.durum.lock().expect("kilit zehirlenmemeli");
            let Some(deger) = dene(&mut durum.ogeler) else {
                durum.bekleyenler.push(cx.waker().clone()); // denetimle aynı kilit altında
                return Poll::Pending;
            };
            let uyanacaklar = std::mem::take(&mut durum.bekleyenler);
            drop(durum);
            uyanacaklar.into_iter().for_each(Waker::wake);
            Poll::Ready(deger)
        })
        .await
    }

    pub async fn koy(&self, oge: T) {
        let mut oge = Some(oge);
        self.bekle(|ogeler| {
            if ogeler.len() >= self.kapasite {
                return None;
            }
            oge.take().map(|o| ogeler.push_back(o))
        })
        .await
    }

    pub async fn al(&self) -> T {
        self.bekle(|ogeler| ogeler.pop_front()).await
    }
}
# fn main() {
#     let kuyruk = Arc::new(AsyncKuyruk::yeni(1));
#     let koyan = Arc::clone(&kuyruk);
#     let mut calistirici = Calistirici::yeni();
#     let b = calistirici.baslatici();
#     b.baslat(async move { for i in 1..=3 { koyan.koy(i).await; } });
#     let t = b.baslat(async move { [kuyruk.al().await, kuyruk.al().await, kuyruk.al().await] });
#     calistirici.calistir();
#     assert_eq!(t.sonucu_al(), Some([1, 2, 3]));
# }
```

Kalıp `Tutamac`'ınkiyle aynıdır: Denetim ve uyandırıcı kaydı tek kilit altında, uyandırma kilit dışında. Her değişiklikte bütün bekleyenleri uyandırmak gereğinden fazla yoklamaya yol açabilir, ama bir bekleyen iptal edilmiş olsa bile ötekilerin uyandırılmasını güvence altına alır.

Kapasitesi 1 olan kuyrukta üretici ile tüketicinin sırayla ilerlediğini görelim:

```rust
# use std::future::Future;
# use std::pin::Pin;
# use std::sync::mpsc::Sender;
# use std::sync::{Arc, Mutex};
# use std::task::{Wake, Waker};
# type KutuluGorev = Pin<Box<dyn Future<Output = ()> + Send>>;
# struct Gorev { gelecek: Mutex<Option<KutuluGorev>>, kuyruk: Sender<Arc<Gorev>> }
# impl Wake for Gorev {
#     fn wake(self: Arc<Self>) { let _ = self.kuyruk.clone().send(self); }
# }
# use std::sync::atomic::{AtomicUsize, Ordering};
# use std::sync::mpsc::{self, Receiver};
# pub struct Calistirici { hazir: Receiver<Arc<Gorev>>, baslatici: Baslatici, poll_sayisi: usize }
# #[derive(Clone)]
# pub struct Baslatici { kuyruk: Sender<Arc<Gorev>>, canli: Arc<AtomicUsize> }
# impl Baslatici {
#     fn kuyruga_koy(&self, gelecek: impl Future<Output = ()> + Send + 'static) {
#         self.canli.fetch_add(1, Ordering::SeqCst);
#         let gelecek = Mutex::new(Some(Box::pin(gelecek) as KutuluGorev));
#         let _ = self.kuyruk.send(Arc::new(Gorev { gelecek, kuyruk: self.kuyruk.clone() }));
#     }
# }
# impl Calistirici {
#     pub fn yeni() -> Self {
#         let (kuyruk, hazir) = mpsc::channel();
#         let baslatici = Baslatici { kuyruk, canli: Arc::new(AtomicUsize::new(0)) };
#         Calistirici { hazir, baslatici, poll_sayisi: 0 }
#     }
#     pub fn baslatici(&self) -> Baslatici { self.baslatici.clone() }
# }
# use std::task::{Context, Poll};
# impl Calistirici {
#     pub fn calistir(&mut self) -> usize {
#         while self.baslatici.canli.load(Ordering::SeqCst) > 0 {
#             let Ok(gorev) = self.hazir.recv() else { break };
#             let mut yuva = gorev.gelecek.lock().expect("kilit zehirlenmemeli");
#             let Some(mut gelecek) = yuva.take() else { continue };
#             let waker = Waker::from(Arc::clone(&gorev));
#             self.poll_sayisi += 1;
#             match gelecek.as_mut().poll(&mut Context::from_waker(&waker)) {
#                 Poll::Pending => *yuva = Some(gelecek),
#                 Poll::Ready(()) => { self.baslatici.canli.fetch_sub(1, Ordering::SeqCst); }
#             }
#         }
#         self.poll_sayisi
#     }
# }
# struct TutamacDurumu<T> { sonuc: Option<T>, bekleyen: Option<Waker> }
# pub struct Tutamac<T> { paylasilan: Arc<Mutex<TutamacDurumu<T>>> }
# impl<T> Tutamac<T> {
#     pub fn sonucu_al(&self) -> Option<T> {
#         self.paylasilan.lock().expect("kilit zehirlenmemeli").sonuc.take()
#     }
# }
# impl<T> Future for Tutamac<T> {
#     type Output = T;
#     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
#         let mut durum = self.paylasilan.lock().expect("kilit zehirlenmemeli");
#         if let Some(deger) = durum.sonuc.take() { return Poll::Ready(deger); }
#         durum.bekleyen = Some(cx.waker().clone());
#         Poll::Pending
#     }
# }
# impl Baslatici {
#     pub fn baslat<F>(&self, gelecek: F) -> Tutamac<F::Output>
#     where F: Future + Send + 'static, F::Output: Send + 'static {
#         let paylasilan = Arc::new(Mutex::new(TutamacDurumu { sonuc: None, bekleyen: None }));
#         let gorev_tarafi = Arc::clone(&paylasilan);
#         self.kuyruga_koy(async move {
#             let deger = gelecek.await;
#             let bekleyen = {
#                 let mut durum = gorev_tarafi.lock().expect("kilit zehirlenmemeli");
#                 durum.sonuc = Some(deger);
#                 durum.bekleyen.take()
#             };
#             if let Some(waker) = bekleyen { waker.wake(); }
#         });
#         Tutamac { paylasilan }
#     }
# }
# pub fn kostur<F>(gelecek: F) -> F::Output
# where F: Future + Send + 'static, F::Output: Send + 'static {
#     let mut calistirici = Calistirici::yeni();
#     let tutamac = calistirici.baslatici().baslat(gelecek);
#     calistirici.calistir();
#     tutamac.sonucu_al().expect("görev bitti ama sonuç yok")
# }
# use std::collections::VecDeque;
# use std::future::poll_fn;
# pub struct AsyncKuyruk<T> { durum: Mutex<AsyncDurum<T>>, kapasite: usize }
# struct AsyncDurum<T> { ogeler: VecDeque<T>, bekleyenler: Vec<Waker> }
# impl<T> AsyncKuyruk<T> {
#     pub fn yeni(kapasite: usize) -> Self {
#         let durum = AsyncDurum { ogeler: VecDeque::new(), bekleyenler: Vec::new() };
#         AsyncKuyruk { durum: Mutex::new(durum), kapasite: kapasite.max(1) }
#     }
#     async fn bekle<R>(&self, mut dene: impl FnMut(&mut VecDeque<T>) -> Option<R>) -> R {
#         poll_fn(|cx| {
#             let mut durum = self.durum.lock().expect("kilit zehirlenmemeli");
#             let Some(deger) = dene(&mut durum.ogeler) else {
#                 durum.bekleyenler.push(cx.waker().clone());
#                 return Poll::Pending;
#             };
#             let uyanacaklar = std::mem::take(&mut durum.bekleyenler);
#             drop(durum);
#             uyanacaklar.into_iter().for_each(Waker::wake);
#             Poll::Ready(deger)
#         })
#         .await
#     }
#     pub async fn koy(&self, oge: T) {
#         let mut oge = Some(oge);
#         self.bekle(|ogeler| {
#             if ogeler.len() >= self.kapasite { return None; }
#             oge.take().map(|o| ogeler.push_back(o))
#         })
#         .await
#     }
#     pub async fn al(&self) -> T { self.bekle(|ogeler| ogeler.pop_front()).await }
# }
fn main() {
    let kuyruk = Arc::new(AsyncKuyruk::yeni(1));
    let ureticinin = Arc::clone(&kuyruk);

    let mut calistirici = Calistirici::yeni();
    let baslatici = calistirici.baslatici();
    baslatici.baslat(async move {
        for i in 1..=3 {
            ureticinin.koy(i).await;
            println!("üretici: {i} kondu");
        }
    });
    baslatici.baslat(async move {
        for _ in 0..3 {
            println!("tüketici: {} alındı", kuyruk.al().await);
        }
    });
    calistirici.calistir();
}
```

Çıktı:

```text
üretici: 1 kondu
tüketici: 1 alındı
üretici: 2 kondu
tüketici: 2 alındı
üretici: 3 kondu
tüketici: 3 alındı
```

Üretici üç öğeyi peş peşe koyamadı; her seferinde tüketicinin yer açmasını bekledi. tokio'nun sınırlı kanalı da aynı davranışı verir: Kanal doluysa `send(..).await` yer açılana dek görevi bekletir.[^tokio-kanal]

### Aynı anda yürüyen işi sınırlamak

Her gelen iş için `baslat` çağırmak da sınırsız bir kuyruktur; bu kez biriken veri değil, görevdir. Çaresi, sabit sayıda işçi görevi ortak bir iş listesinden beslemektir:

```rust
# use std::future::Future;
# use std::pin::Pin;
# use std::sync::mpsc::Sender;
# use std::sync::{Arc, Mutex};
# use std::task::{Wake, Waker};
# type KutuluGorev = Pin<Box<dyn Future<Output = ()> + Send>>;
# struct Gorev { gelecek: Mutex<Option<KutuluGorev>>, kuyruk: Sender<Arc<Gorev>> }
# impl Wake for Gorev {
#     fn wake(self: Arc<Self>) { let _ = self.kuyruk.clone().send(self); }
# }
# use std::sync::atomic::{AtomicUsize, Ordering};
# use std::sync::mpsc::{self, Receiver};
# pub struct Calistirici { hazir: Receiver<Arc<Gorev>>, baslatici: Baslatici, poll_sayisi: usize }
# #[derive(Clone)]
# pub struct Baslatici { kuyruk: Sender<Arc<Gorev>>, canli: Arc<AtomicUsize> }
# impl Baslatici {
#     fn kuyruga_koy(&self, gelecek: impl Future<Output = ()> + Send + 'static) {
#         self.canli.fetch_add(1, Ordering::SeqCst);
#         let gelecek = Mutex::new(Some(Box::pin(gelecek) as KutuluGorev));
#         let _ = self.kuyruk.send(Arc::new(Gorev { gelecek, kuyruk: self.kuyruk.clone() }));
#     }
# }
# impl Calistirici {
#     pub fn yeni() -> Self {
#         let (kuyruk, hazir) = mpsc::channel();
#         let baslatici = Baslatici { kuyruk, canli: Arc::new(AtomicUsize::new(0)) };
#         Calistirici { hazir, baslatici, poll_sayisi: 0 }
#     }
#     pub fn baslatici(&self) -> Baslatici { self.baslatici.clone() }
# }
# use std::task::{Context, Poll};
# impl Calistirici {
#     pub fn calistir(&mut self) -> usize {
#         while self.baslatici.canli.load(Ordering::SeqCst) > 0 {
#             let Ok(gorev) = self.hazir.recv() else { break };
#             let mut yuva = gorev.gelecek.lock().expect("kilit zehirlenmemeli");
#             let Some(mut gelecek) = yuva.take() else { continue };
#             let waker = Waker::from(Arc::clone(&gorev));
#             self.poll_sayisi += 1;
#             match gelecek.as_mut().poll(&mut Context::from_waker(&waker)) {
#                 Poll::Pending => *yuva = Some(gelecek),
#                 Poll::Ready(()) => { self.baslatici.canli.fetch_sub(1, Ordering::SeqCst); }
#             }
#         }
#         self.poll_sayisi
#     }
# }
# struct TutamacDurumu<T> { sonuc: Option<T>, bekleyen: Option<Waker> }
# pub struct Tutamac<T> { paylasilan: Arc<Mutex<TutamacDurumu<T>>> }
# impl<T> Tutamac<T> {
#     pub fn sonucu_al(&self) -> Option<T> {
#         self.paylasilan.lock().expect("kilit zehirlenmemeli").sonuc.take()
#     }
# }
# impl<T> Future for Tutamac<T> {
#     type Output = T;
#     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
#         let mut durum = self.paylasilan.lock().expect("kilit zehirlenmemeli");
#         if let Some(deger) = durum.sonuc.take() { return Poll::Ready(deger); }
#         durum.bekleyen = Some(cx.waker().clone());
#         Poll::Pending
#     }
# }
# impl Baslatici {
#     pub fn baslat<F>(&self, gelecek: F) -> Tutamac<F::Output>
#     where F: Future + Send + 'static, F::Output: Send + 'static {
#         let paylasilan = Arc::new(Mutex::new(TutamacDurumu { sonuc: None, bekleyen: None }));
#         let gorev_tarafi = Arc::clone(&paylasilan);
#         self.kuyruga_koy(async move {
#             let deger = gelecek.await;
#             let bekleyen = {
#                 let mut durum = gorev_tarafi.lock().expect("kilit zehirlenmemeli");
#                 durum.sonuc = Some(deger);
#                 durum.bekleyen.take()
#             };
#             if let Some(waker) = bekleyen { waker.wake(); }
#         });
#         Tutamac { paylasilan }
#     }
# }
# pub fn kostur<F>(gelecek: F) -> F::Output
# where F: Future + Send + 'static, F::Output: Send + 'static {
#     let mut calistirici = Calistirici::yeni();
#     let tutamac = calistirici.baslatici().baslat(gelecek);
#     calistirici.calistir();
#     tutamac.sonucu_al().expect("görev bitti ama sonuç yok")
# }
# struct SiraVer { verildi: bool }
# impl Future for SiraVer {
#     type Output = ();
#     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
#         if self.verildi { return Poll::Ready(()); }
#         self.verildi = true;
#         cx.waker().wake_by_ref();
#         Poll::Pending
#     }
# }
# fn sira_ver() -> SiraVer { SiraVer { verildi: false } }
# use std::collections::VecDeque;
async fn isle(aktif: &AtomicUsize, en_yuksek: &AtomicUsize) {
    let simdi = aktif.fetch_add(1, Ordering::SeqCst) + 1;
    en_yuksek.fetch_max(simdi, Ordering::SeqCst);
    sira_ver().await; // bekleyen iş
    aktif.fetch_sub(1, Ordering::SeqCst);
}

fn en_yuksek_eszamanlilik(isci_sayisi: usize) -> usize {
    let isler = Arc::new(Mutex::new(VecDeque::from_iter(0..6)));
    let sayaclar = Arc::new((AtomicUsize::new(0), AtomicUsize::new(0)));

    let mut calistirici = Calistirici::yeni();
    for _ in 0..isci_sayisi {
        let (isler, sayaclar) = (Arc::clone(&isler), Arc::clone(&sayaclar));
        calistirici.baslatici().baslat(async move {
            loop {
                let is = isler.lock().expect("kilit zehirlenmemeli").pop_front();
                if is.is_none() {
                    break;
                }
                isle(&sayaclar.0, &sayaclar.1).await;
            }
        });
    }
    calistirici.calistir();
    sayaclar.1.load(Ordering::SeqCst)
}

fn main() {
    println!("6 iş, 6 işçi: aynı anda en çok {}", en_yuksek_eszamanlilik(6));
    println!("6 iş, 2 işçi: aynı anda en çok {}", en_yuksek_eszamanlilik(2));
}
```

Çıktı:

```text
6 iş, 6 işçi: aynı anda en çok 6
6 iş, 2 işçi: aynı anda en çok 2
```

Thread'lerdeki işçi havuzunun ([15.1](../orta/15-concurrency-derinlesme.md)) görevlerle kurulmuş hâlidir. Hazır araç olarak `tokio::sync::Semaphore` aynı sınırı izin sayısıyla uygular.[^semafor]

## Sık hatalar ve çözümleri

### Uyandırıcıyı kaydetmeden `Pending` döndürmek

```rust,no_run
# use std::future::Future;
# use std::pin::Pin;
# use std::sync::mpsc::Sender;
# use std::sync::{Arc, Mutex};
# use std::task::{Wake, Waker};
# type KutuluGorev = Pin<Box<dyn Future<Output = ()> + Send>>;
# struct Gorev { gelecek: Mutex<Option<KutuluGorev>>, kuyruk: Sender<Arc<Gorev>> }
# impl Wake for Gorev {
#     fn wake(self: Arc<Self>) { let _ = self.kuyruk.clone().send(self); }
# }
# use std::sync::atomic::{AtomicUsize, Ordering};
# use std::sync::mpsc::{self, Receiver};
# pub struct Calistirici { hazir: Receiver<Arc<Gorev>>, baslatici: Baslatici, poll_sayisi: usize }
# #[derive(Clone)]
# pub struct Baslatici { kuyruk: Sender<Arc<Gorev>>, canli: Arc<AtomicUsize> }
# impl Baslatici {
#     fn kuyruga_koy(&self, gelecek: impl Future<Output = ()> + Send + 'static) {
#         self.canli.fetch_add(1, Ordering::SeqCst);
#         let gelecek = Mutex::new(Some(Box::pin(gelecek) as KutuluGorev));
#         let _ = self.kuyruk.send(Arc::new(Gorev { gelecek, kuyruk: self.kuyruk.clone() }));
#     }
# }
# impl Calistirici {
#     pub fn yeni() -> Self {
#         let (kuyruk, hazir) = mpsc::channel();
#         let baslatici = Baslatici { kuyruk, canli: Arc::new(AtomicUsize::new(0)) };
#         Calistirici { hazir, baslatici, poll_sayisi: 0 }
#     }
#     pub fn baslatici(&self) -> Baslatici { self.baslatici.clone() }
# }
# use std::task::{Context, Poll};
# impl Calistirici {
#     pub fn calistir(&mut self) -> usize {
#         while self.baslatici.canli.load(Ordering::SeqCst) > 0 {
#             let Ok(gorev) = self.hazir.recv() else { break };
#             let mut yuva = gorev.gelecek.lock().expect("kilit zehirlenmemeli");
#             let Some(mut gelecek) = yuva.take() else { continue };
#             let waker = Waker::from(Arc::clone(&gorev));
#             self.poll_sayisi += 1;
#             match gelecek.as_mut().poll(&mut Context::from_waker(&waker)) {
#                 Poll::Pending => *yuva = Some(gelecek),
#                 Poll::Ready(()) => { self.baslatici.canli.fetch_sub(1, Ordering::SeqCst); }
#             }
#         }
#         self.poll_sayisi
#     }
# }
# struct TutamacDurumu<T> { sonuc: Option<T>, bekleyen: Option<Waker> }
# pub struct Tutamac<T> { paylasilan: Arc<Mutex<TutamacDurumu<T>>> }
# impl<T> Tutamac<T> {
#     pub fn sonucu_al(&self) -> Option<T> {
#         self.paylasilan.lock().expect("kilit zehirlenmemeli").sonuc.take()
#     }
# }
# impl<T> Future for Tutamac<T> {
#     type Output = T;
#     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
#         let mut durum = self.paylasilan.lock().expect("kilit zehirlenmemeli");
#         if let Some(deger) = durum.sonuc.take() { return Poll::Ready(deger); }
#         durum.bekleyen = Some(cx.waker().clone());
#         Poll::Pending
#     }
# }
# impl Baslatici {
#     pub fn baslat<F>(&self, gelecek: F) -> Tutamac<F::Output>
#     where F: Future + Send + 'static, F::Output: Send + 'static {
#         let paylasilan = Arc::new(Mutex::new(TutamacDurumu { sonuc: None, bekleyen: None }));
#         let gorev_tarafi = Arc::clone(&paylasilan);
#         self.kuyruga_koy(async move {
#             let deger = gelecek.await;
#             let bekleyen = {
#                 let mut durum = gorev_tarafi.lock().expect("kilit zehirlenmemeli");
#                 durum.sonuc = Some(deger);
#                 durum.bekleyen.take()
#             };
#             if let Some(waker) = bekleyen { waker.wake(); }
#         });
#         Tutamac { paylasilan }
#     }
# }
# pub fn kostur<F>(gelecek: F) -> F::Output
# where F: Future + Send + 'static, F::Output: Send + 'static {
#     let mut calistirici = Calistirici::yeni();
#     let tutamac = calistirici.baslatici().baslat(gelecek);
#     calistirici.calistir();
#     tutamac.sonucu_al().expect("görev bitti ama sonuç yok")
# }
# use std::time::Duration;
# pub struct Uyku { sure: Duration, basladi: bool, durum: Arc<Mutex<UykuDurumu>> }
# struct UykuDurumu { doldu: bool, waker: Option<Waker> }
# impl Uyku {
#     pub fn yeni(sure: Duration) -> Self {
#         let durum = Arc::new(Mutex::new(UykuDurumu { doldu: false, waker: None }));
#         Uyku { sure, basladi: false, durum }
#     }
# }
# pub fn uyu_ms(ms: u64) -> Uyku { Uyku::yeni(Duration::from_millis(ms)) }
# impl Future for Uyku {
#     type Output = ();
#     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
#         let bu = self.get_mut();
#         let mut durum = bu.durum.lock().expect("kilit zehirlenmemeli");
#         if durum.doldu { return Poll::Ready(()); }
#         durum.waker = Some(cx.waker().clone());
#         drop(durum);
#         if std::mem::replace(&mut bu.basladi, true) { return Poll::Pending; }
#         let (paylasilan, sure) = (Arc::clone(&bu.durum), bu.sure);
#         std::thread::spawn(move || {
#             std::thread::sleep(sure);
#             let mut durum = paylasilan.lock().expect("kilit zehirlenmemeli");
#             durum.doldu = true;
#             let waker = durum.waker.take();
#             drop(durum);
#             if let Some(waker) = waker { waker.wake(); }
#         });
#         Poll::Pending
#     }
# }
struct Bayrak {
    hazir: bool,
}

impl Future for Bayrak {
    type Output = ();
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
        if self.hazir { Poll::Ready(()) } else { Poll::Pending } // uyandırıcı kaydedilmedi
    }
}

fn main() {
    kostur(Bayrak { hazir: false }); // dönmez
}
```

Derlenir, ama `kostur` dönmez: Görev kuyruktan çıktı ve onu geri koyacak kimse yok. `Pending` döndürmeden önce, görevin ileride uyandırılacağını güvence altına almak sözleşmenin parçasıdır.[^pending] Düzeltmesi `Uyku`'daki gibidir: Uyandırıcıyı, olayı bilecek tarafa teslim edin ve "Hazır mı?" denetimi ile kaydı aynı kilit altında yapın.

### Eski uyandırıcıyı saklamak

Kaydı yalnızca ilk yoklamada yapan bir future, araya başka bir uyandırıcı girince yanlış kişiyi uyandırır. İki sayaçlı uyandırıcıyla görelim:

```rust
# use std::future::Future;
# use std::pin::{Pin, pin};
# use std::sync::atomic::{AtomicUsize, Ordering};
# use std::sync::{Arc, Mutex};
# use std::task::{Context, Poll, Wake, Waker};
struct Sayac(AtomicUsize);

impl Wake for Sayac {
    fn wake(self: Arc<Self>) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

struct Olay {
    waker: Arc<Mutex<Option<Waker>>>,
    her_seferinde: bool, // doğru davranış
}

impl Future for Olay {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let mut yuva = self.waker.lock().expect("kilit zehirlenmemeli");
        if self.her_seferinde || yuva.is_none() {
            *yuva = Some(cx.waker().clone());
        }
        Poll::Pending
    }
}

fn dene(her_seferinde: bool) -> (usize, usize) {
    let yuva = Arc::new(Mutex::new(None));
    let mut olay = pin!(Olay { waker: Arc::clone(&yuva), her_seferinde });
    let [eski, yeni] = [(); 2].map(|()| Arc::new(Sayac(AtomicUsize::new(0))));

    for sayac in [&eski, &yeni] {
        let waker = Waker::from(Arc::clone(sayac));
        let _ = olay.as_mut().poll(&mut Context::from_waker(&waker));
    }
    if let Some(waker) = yuva.lock().expect("kilit zehirlenmemeli").take() {
        waker.wake(); // olay gerçekleşti
    }
    (eski.0.load(Ordering::SeqCst), yeni.0.load(Ordering::SeqCst))
}

fn main() {
    println!("yalnız ilk yoklamada kayıt: (eski, yeni) = {:?}", dene(false));
    println!("her yoklamada kayıt:        (eski, yeni) = {:?}", dene(true));
}
```

Çıktı:

```text
yalnız ilk yoklamada kayıt: (eski, yeni) = (1, 0)
her yoklamada kayıt:        (eski, yeni) = (0, 1)
```

Birinci durumda görevi artık yürütmeyen taraf uyandırıldı, yürüten taraf habersiz kaldı.

### Bitmiş future'ı yeniden yoklamak

```rust,should_panic
# use std::future::Future;
# use std::pin::pin;
# use std::task::{Context, Waker};
fn main() {
    let mut gelecek = pin!(async { 42 });
    let mut cx = Context::from_waker(Waker::noop());

    println!("{:?}", gelecek.as_mut().poll(&mut cx));
    println!("{:?}", gelecek.as_mut().poll(&mut cx)); // bitmişti
}
```

Çıktı:

```text
Ready(42)
`async fn` resumed after completion
```

`Future` belgesi, bitmiş bir future'ın yeniden yoklanmaması gerektiğini söyler. Yoklanırsa ne olacağı türe göre değişir: Panikleyebilir, sonsuza dek bekleyebilir; ama tanımsız davranış olmaz.[^panik] Derleyicinin ürettiği durum makineleri yukarıdaki gibi panikler.[^resume] Çalıştırıcımız bu duruma düşmez: Biten görevin yuvası `None` kalır ve gecikmiş uyandırma `continue` ile geçilir.

### Çalıştırıcı bittikten sonra görev başlatmak

```rust
# use std::future::Future;
# use std::pin::Pin;
# use std::sync::mpsc::Sender;
# use std::sync::{Arc, Mutex};
# use std::task::{Wake, Waker};
# type KutuluGorev = Pin<Box<dyn Future<Output = ()> + Send>>;
# struct Gorev { gelecek: Mutex<Option<KutuluGorev>>, kuyruk: Sender<Arc<Gorev>> }
# impl Wake for Gorev {
#     fn wake(self: Arc<Self>) { let _ = self.kuyruk.clone().send(self); }
# }
# use std::sync::atomic::{AtomicUsize, Ordering};
# use std::sync::mpsc::{self, Receiver};
# pub struct Calistirici { hazir: Receiver<Arc<Gorev>>, baslatici: Baslatici, poll_sayisi: usize }
# #[derive(Clone)]
# pub struct Baslatici { kuyruk: Sender<Arc<Gorev>>, canli: Arc<AtomicUsize> }
# impl Baslatici {
#     fn kuyruga_koy(&self, gelecek: impl Future<Output = ()> + Send + 'static) {
#         self.canli.fetch_add(1, Ordering::SeqCst);
#         let gelecek = Mutex::new(Some(Box::pin(gelecek) as KutuluGorev));
#         let _ = self.kuyruk.send(Arc::new(Gorev { gelecek, kuyruk: self.kuyruk.clone() }));
#     }
# }
# impl Calistirici {
#     pub fn yeni() -> Self {
#         let (kuyruk, hazir) = mpsc::channel();
#         let baslatici = Baslatici { kuyruk, canli: Arc::new(AtomicUsize::new(0)) };
#         Calistirici { hazir, baslatici, poll_sayisi: 0 }
#     }
#     pub fn baslatici(&self) -> Baslatici { self.baslatici.clone() }
# }
# use std::task::{Context, Poll};
# impl Calistirici {
#     pub fn calistir(&mut self) -> usize {
#         while self.baslatici.canli.load(Ordering::SeqCst) > 0 {
#             let Ok(gorev) = self.hazir.recv() else { break };
#             let mut yuva = gorev.gelecek.lock().expect("kilit zehirlenmemeli");
#             let Some(mut gelecek) = yuva.take() else { continue };
#             let waker = Waker::from(Arc::clone(&gorev));
#             self.poll_sayisi += 1;
#             match gelecek.as_mut().poll(&mut Context::from_waker(&waker)) {
#                 Poll::Pending => *yuva = Some(gelecek),
#                 Poll::Ready(()) => { self.baslatici.canli.fetch_sub(1, Ordering::SeqCst); }
#             }
#         }
#         self.poll_sayisi
#     }
# }
# struct TutamacDurumu<T> { sonuc: Option<T>, bekleyen: Option<Waker> }
# pub struct Tutamac<T> { paylasilan: Arc<Mutex<TutamacDurumu<T>>> }
# impl<T> Tutamac<T> {
#     pub fn sonucu_al(&self) -> Option<T> {
#         self.paylasilan.lock().expect("kilit zehirlenmemeli").sonuc.take()
#     }
# }
# impl<T> Future for Tutamac<T> {
#     type Output = T;
#     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
#         let mut durum = self.paylasilan.lock().expect("kilit zehirlenmemeli");
#         if let Some(deger) = durum.sonuc.take() { return Poll::Ready(deger); }
#         durum.bekleyen = Some(cx.waker().clone());
#         Poll::Pending
#     }
# }
# impl Baslatici {
#     pub fn baslat<F>(&self, gelecek: F) -> Tutamac<F::Output>
#     where F: Future + Send + 'static, F::Output: Send + 'static {
#         let paylasilan = Arc::new(Mutex::new(TutamacDurumu { sonuc: None, bekleyen: None }));
#         let gorev_tarafi = Arc::clone(&paylasilan);
#         self.kuyruga_koy(async move {
#             let deger = gelecek.await;
#             let bekleyen = {
#                 let mut durum = gorev_tarafi.lock().expect("kilit zehirlenmemeli");
#                 durum.sonuc = Some(deger);
#                 durum.bekleyen.take()
#             };
#             if let Some(waker) = bekleyen { waker.wake(); }
#         });
#         Tutamac { paylasilan }
#     }
# }
# pub fn kostur<F>(gelecek: F) -> F::Output
# where F: Future + Send + 'static, F::Output: Send + 'static {
#     let mut calistirici = Calistirici::yeni();
#     let tutamac = calistirici.baslatici().baslat(gelecek);
#     calistirici.calistir();
#     tutamac.sonucu_al().expect("görev bitti ama sonuç yok")
# }
fn main() {
    let mut calistirici = Calistirici::yeni();
    let baslatici = calistirici.baslatici();
    baslatici.baslat(async {});
    calistirici.calistir();

    let gec = baslatici.baslat(async { "geç kalan" });
    println!("döngü bittikten sonra: {:?}", gec.sonucu_al());

    calistirici.calistir(); // döngüyü yeniden açmak gerekir
    println!("yeniden çalıştırınca: {:?}", gec.sonucu_al());
}
```

Çıktı:

```text
döngü bittikten sonra: None
yeniden çalıştırınca: Some("geç kalan")
```

`baslat` görevi kuyruğa koyar, ama onu yoklayacak döngü çıkmıştır. Görevi ya `calistir`'dan önce ya da çalışan bir görevin içinden başlatın.

## Akılda kalması gerekenler

- Uyandırmak, görevi hazır kuyruğuna geri koymaktır. Bekleyen görev kuyrukta durmaz; on bin bekleyen görev yoklama maliyeti yaratmaz.
- Denetim ile uyandırıcı kaydı aynı kilit altında, uyandırma kilit dışında yapılır. Bu kalıp `Tutamac`'ta, `Uyku`'da ve `AsyncKuyruk`'ta aynıdır.
- Uyandırıcıyı her yoklamada güncelleyin; geçerli olan sonuncusudur.
- `Send + 'static` sınırları birer tasarım kararının sonucudur: Görev başka thread'lerden uyandırılabilir ve onu başlatandan uzun yaşayabilir.
- İptal düşürmektir. Her `.await`, işin yarıda kalabileceği bir noktadır.
- Geri basıncı hazır kuyruğuna değil, verinin ya da işin biriktiği yere koyun.

## Alıştırmalar

**1.** `pub async fn hepsini_bekle<T>(tutamaclar: Vec<Tutamac<T>>) -> Vec<T>` fonksiyonunu yazın. En basit çözüm sırayla `.await` etmektir; bu neden yine de eşzamanlıdır? [16.1'deki](../orta/16-async-await-derinlesme.md) `hepsini_bekle`'den farkını bir cümleyle yazın.

<details>
<summary>İpucu ve çözüm</summary>

Görevler `baslat` anında kuyruğa girmiştir ve onları çalıştırıcı yürütür; `.await` yalnızca *sonucu* bekler. 16.1'deki `hepsini_bekle` ise henüz başlamamış future'ları kendisi yokluyordu: Orada işleri ilerleten fonksiyonun kendisiydi, burada çalıştırıcıdır.

```rust
# use std::future::Future;
# use std::pin::Pin;
# use std::sync::mpsc::Sender;
# use std::sync::{Arc, Mutex};
# use std::task::{Wake, Waker};
# type KutuluGorev = Pin<Box<dyn Future<Output = ()> + Send>>;
# struct Gorev { gelecek: Mutex<Option<KutuluGorev>>, kuyruk: Sender<Arc<Gorev>> }
# impl Wake for Gorev {
#     fn wake(self: Arc<Self>) { let _ = self.kuyruk.clone().send(self); }
# }
# use std::sync::atomic::{AtomicUsize, Ordering};
# use std::sync::mpsc::{self, Receiver};
# pub struct Calistirici { hazir: Receiver<Arc<Gorev>>, baslatici: Baslatici, poll_sayisi: usize }
# #[derive(Clone)]
# pub struct Baslatici { kuyruk: Sender<Arc<Gorev>>, canli: Arc<AtomicUsize> }
# impl Baslatici {
#     fn kuyruga_koy(&self, gelecek: impl Future<Output = ()> + Send + 'static) {
#         self.canli.fetch_add(1, Ordering::SeqCst);
#         let gelecek = Mutex::new(Some(Box::pin(gelecek) as KutuluGorev));
#         let _ = self.kuyruk.send(Arc::new(Gorev { gelecek, kuyruk: self.kuyruk.clone() }));
#     }
# }
# impl Calistirici {
#     pub fn yeni() -> Self {
#         let (kuyruk, hazir) = mpsc::channel();
#         let baslatici = Baslatici { kuyruk, canli: Arc::new(AtomicUsize::new(0)) };
#         Calistirici { hazir, baslatici, poll_sayisi: 0 }
#     }
#     pub fn baslatici(&self) -> Baslatici { self.baslatici.clone() }
# }
# use std::task::{Context, Poll};
# impl Calistirici {
#     pub fn calistir(&mut self) -> usize {
#         while self.baslatici.canli.load(Ordering::SeqCst) > 0 {
#             let Ok(gorev) = self.hazir.recv() else { break };
#             let mut yuva = gorev.gelecek.lock().expect("kilit zehirlenmemeli");
#             let Some(mut gelecek) = yuva.take() else { continue };
#             let waker = Waker::from(Arc::clone(&gorev));
#             self.poll_sayisi += 1;
#             match gelecek.as_mut().poll(&mut Context::from_waker(&waker)) {
#                 Poll::Pending => *yuva = Some(gelecek),
#                 Poll::Ready(()) => { self.baslatici.canli.fetch_sub(1, Ordering::SeqCst); }
#             }
#         }
#         self.poll_sayisi
#     }
# }
# struct TutamacDurumu<T> { sonuc: Option<T>, bekleyen: Option<Waker> }
# pub struct Tutamac<T> { paylasilan: Arc<Mutex<TutamacDurumu<T>>> }
# impl<T> Tutamac<T> {
#     pub fn sonucu_al(&self) -> Option<T> {
#         self.paylasilan.lock().expect("kilit zehirlenmemeli").sonuc.take()
#     }
# }
# impl<T> Future for Tutamac<T> {
#     type Output = T;
#     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
#         let mut durum = self.paylasilan.lock().expect("kilit zehirlenmemeli");
#         if let Some(deger) = durum.sonuc.take() { return Poll::Ready(deger); }
#         durum.bekleyen = Some(cx.waker().clone());
#         Poll::Pending
#     }
# }
# impl Baslatici {
#     pub fn baslat<F>(&self, gelecek: F) -> Tutamac<F::Output>
#     where F: Future + Send + 'static, F::Output: Send + 'static {
#         let paylasilan = Arc::new(Mutex::new(TutamacDurumu { sonuc: None, bekleyen: None }));
#         let gorev_tarafi = Arc::clone(&paylasilan);
#         self.kuyruga_koy(async move {
#             let deger = gelecek.await;
#             let bekleyen = {
#                 let mut durum = gorev_tarafi.lock().expect("kilit zehirlenmemeli");
#                 durum.sonuc = Some(deger);
#                 durum.bekleyen.take()
#             };
#             if let Some(waker) = bekleyen { waker.wake(); }
#         });
#         Tutamac { paylasilan }
#     }
# }
# pub fn kostur<F>(gelecek: F) -> F::Output
# where F: Future + Send + 'static, F::Output: Send + 'static {
#     let mut calistirici = Calistirici::yeni();
#     let tutamac = calistirici.baslatici().baslat(gelecek);
#     calistirici.calistir();
#     tutamac.sonucu_al().expect("görev bitti ama sonuç yok")
# }
# use std::time::Duration;
# pub struct Uyku { sure: Duration, basladi: bool, durum: Arc<Mutex<UykuDurumu>> }
# struct UykuDurumu { doldu: bool, waker: Option<Waker> }
# impl Uyku {
#     pub fn yeni(sure: Duration) -> Self {
#         let durum = Arc::new(Mutex::new(UykuDurumu { doldu: false, waker: None }));
#         Uyku { sure, basladi: false, durum }
#     }
# }
# pub fn uyu_ms(ms: u64) -> Uyku { Uyku::yeni(Duration::from_millis(ms)) }
# impl Future for Uyku {
#     type Output = ();
#     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
#         let bu = self.get_mut();
#         let mut durum = bu.durum.lock().expect("kilit zehirlenmemeli");
#         if durum.doldu { return Poll::Ready(()); }
#         durum.waker = Some(cx.waker().clone());
#         drop(durum);
#         if std::mem::replace(&mut bu.basladi, true) { return Poll::Pending; }
#         let (paylasilan, sure) = (Arc::clone(&bu.durum), bu.sure);
#         std::thread::spawn(move || {
#             std::thread::sleep(sure);
#             let mut durum = paylasilan.lock().expect("kilit zehirlenmemeli");
#             durum.doldu = true;
#             let waker = durum.waker.take();
#             drop(durum);
#             if let Some(waker) = waker { waker.wake(); }
#         });
#         Poll::Pending
#     }
# }
# use std::time::Instant;
pub async fn hepsini_bekle<T>(tutamaclar: Vec<Tutamac<T>>) -> Vec<T> {
    let mut sonuclar = Vec::with_capacity(tutamaclar.len());
    for tutamac in tutamaclar {
        sonuclar.push(tutamac.await);
    }
    sonuclar
}

fn main() {
    let mut calistirici = Calistirici::yeni();
    let baslatici = calistirici.baslatici();
    let baslangic = Instant::now();

    let tutamaclar = [30, 10, 20]
        .map(|ms| {
            baslatici.baslat(async move {
                uyu_ms(ms).await;
                ms
            })
        })
        .into();
    let hepsi = baslatici.baslat(hepsini_bekle(tutamaclar));
    calistirici.calistir();

    assert_eq!(hepsi.sonucu_al(), Some(vec![30, 10, 20])); // başlatma sırası korunur
    assert!(baslangic.elapsed() < Duration::from_millis(60)); // 30 + 10 + 20 değil
}
```

</details>

## Özet

- `Gorev` sabitlenmiş, türü silinmiş bir future ile döneceği kuyruktan oluşur; `Wake` uygulaması onu kuyruğa geri koyar.
- `Calistirici` yalnızca uyandırılan görevi yoklar; kuyruk boşken uyur.
- `baslat` görevi kuyruğa koyar ve `Tutamac` döndürür; tutamaç hem `sonucu_al` ile hem `.await` ile okunur.
- `Uyku` ilk yoklamada kurulan bir zamanlayıcıdır; `ZamanAsimi` onu asıl işle yarıştırır ve kaybedeni düşürür.
- Hazır kuyruğu sınırsızdır; geri basınç sınırlı veri kuyruklarıyla ve işçi sayısıyla uygulanır.

Sıradaki sayfa [29.3 Asenkron API İstemcisi](29-istemci.md): Bu parçalarla eşzamanlı istek atan, yavaş isteği kesen ve geçici hatada yeniden deneyen bir istemci yazacağız.

## Kaynaklar

[^wake]: std, [`std::task::Wake`](https://doc.rust-lang.org/std/task/trait.Wake.html); *Rust for Rustaceans*, 8. bölüm (Asynchronous Programming).
[^waker-from]: std 1.99.0 kaynak kodu, [`task.rs`, `From<Arc<W>> for Waker`](https://github.com/rust-lang/rust/blob/1.99.0/library/alloc/src/task.rs#L114) ve [`wake.rs`, `Sync for Waker`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/task/wake.rs#L416).
[^future]: std, [`Future`](https://doc.rust-lang.org/std/future/trait.Future.html); The Rust Programming Language, [17.1 Futures and the Async Syntax](https://doc.rust-lang.org/book/ch17-01-futures-and-syntax.html).
[^kanal]: std, [`mpsc::channel`](https://doc.rust-lang.org/std/sync/mpsc/fn.channel.html) ve [`Receiver::recv`](https://doc.rust-lang.org/std/sync/mpsc/struct.Receiver.html#method.recv).
[^donus]: std, [`Future::poll`: Return value](https://doc.rust-lang.org/std/future/trait.Future.html#return-value).
[^block-on]: tokio, [`Runtime::block_on`](https://docs.rs/tokio/latest/tokio/runtime/struct.Runtime.html#method.block_on).
[^spawn]: std, [`thread::spawn`](https://doc.rust-lang.org/std/thread/fn.spawn.html); tokio, [`spawn`](https://docs.rs/tokio/latest/tokio/task/fn.spawn.html).
[^e0373]: Hata dizini, [`E0373`](https://doc.rust-lang.org/error_codes/E0373.html); The Rust Reference, [Async blocks: Capture modes](https://doc.rust-lang.org/reference/expressions/block-expr.html#capture-modes).
[^clone-from]: std, [`Waker::clone_from`](https://doc.rust-lang.org/std/task/struct.Waker.html#method.clone_from).
[^get-mut]: std, [`Pin::get_mut`](https://doc.rust-lang.org/std/pin/struct.Pin.html#method.get_mut).
[^surucu]: tokio, [`runtime` modülü](https://docs.rs/tokio/latest/tokio/runtime/index.html); mio, [`Poll`](https://docs.rs/mio/latest/mio/struct.Poll.html).
[^iptal]: tokio, [`time::timeout`](https://docs.rs/tokio/latest/tokio/time/fn.timeout.html); tokio öğreticisi, [Select](https://tokio.rs/tokio/tutorial/select).
[^drop-glue]: rustc 1.99.0 kaynak kodu, [`coroutine/mod.rs`, `drop glue`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_mir_transform/src/coroutine/mod.rs#L48).
[^izdusum]: std, [`pin`: Projections and structural pinning](https://doc.rust-lang.org/std/pin/index.html#projections-and-structural-pinning).
[^pin-project]: docs.rs, [`pin_project`](https://docs.rs/pin-project/latest/pin_project/) ve [`pin_project_lite`](https://docs.rs/pin-project-lite/latest/pin_project_lite/).
[^poll-fn]: std, [`poll_fn`: Capturing a pinned state](https://doc.rust-lang.org/std/future/fn.poll_fn.html#capturing-a-pinned-state).
[^pin-new]: std, [`Pin::new`](https://doc.rust-lang.org/std/pin/struct.Pin.html#method.new); The Rust Programming Language, [17.5 The Pin Type and the Unpin Trait](https://doc.rust-lang.org/book/ch17-05-traits-for-async.html#the-pin-type-and-the-unpin-trait).
[^sync-channel]: std, [`mpsc::sync_channel`](https://doc.rust-lang.org/std/sync/mpsc/fn.sync_channel.html).
[^wake-birlesme]: std, [`Waker::wake`](https://doc.rust-lang.org/std/task/struct.Waker.html#method.wake).
[^tokio-kanal]: tokio, [`sync::mpsc::channel`](https://docs.rs/tokio/latest/tokio/sync/mpsc/fn.channel.html); tokio öğreticisi, [Channels](https://tokio.rs/tokio/tutorial/channels).
[^semafor]: tokio, [`sync::Semaphore`](https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html).
[^pending]: std, [`Poll::Pending`](https://doc.rust-lang.org/std/task/enum.Poll.html#variant.Pending).
[^panik]: std, [`Future::poll`: Panics](https://doc.rust-lang.org/std/future/trait.Future.html#panics).
[^resume]: rustc 1.99.0 kaynak kodu, [`coroutine/mod.rs`, `ResumedAfterReturn`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_mir_transform/src/coroutine/mod.rs#L881).
