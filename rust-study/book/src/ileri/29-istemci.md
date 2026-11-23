# 29.3 Asenkron API İstemcisi

Bu sayfada [29.2'deki](29-calisma-zamani.md) parçaları (`baslat`, `uyu_ms`, `zaman_asimi`) bir araya getirip gerçeğe yakın bir istemci yazacağız. Her istemcinin üç temel ihtiyacı vardır:

- birbirinden bağımsız istekleri eşzamanlı göndermek,
- çok uzun süren isteği zaman aşımıyla kesmek,
- geçici hatada yeniden denemek.

İkinci yarıda async kodun en yaygın hatalarına bakacağız: thread'i bloklamak, kilidi `.await` boyunca tutmak ve `baslat`'ın sınırlarından doğan ötekiler. Ağ yerine gecikmeli sahte bir sunucu kullanıyoruz; kodun biçimi tokio ve reqwest ile yazılanla aynıdır.

> [!NOTE]
> Bu sayfanın kodu `learn/advanced-concepts/ileri-eszamanlilik-async/src/istemci.rs` dosyasındadır. Testlerini `cargo test -p ileri-eszamanlilik-async` ile çalıştırabilirsiniz. Bloklarda 29.2'nin çalışma zamanı `calisma_zamani` adlı gizli bir modülde durur; derlenmeyen örneklerde bu modül yalnızca imzaları içerir. Süreler makineye göre değiştiği için yalnızca karşılaştırma sonuçlarını yazdırıyoruz.

<details>
<summary>Bu sayfada geçen yeni terimler</summary>

- **istemci, sunucu (client, server):** Soran taraf ve yanıt veren taraf.
- **API, istek, yanıt (request, response):** Sunucunun dışarıya açtığı kapı, oraya sorulan soru ve dönen karşılık.
- **503, 404:** Sunucunun "Şu an meşgulüm" ve "Böyle bir şey yok" demesi.
- **geçici hata, kalıcı hata:** Biraz sonra düzelebilecek hata ve yeniden denemenin işe yaramadığı hata.
- **yeniden deneme (retry):** Geçici hatada aynı isteği yeniden göndermek.
- **artan bekleme (backoff):** Her yeniden denemeden önce biraz daha uzun beklemek.
- **pano (dashboard):** Birkaç kaynaktan gelen bilgiyi tek ekranda gösteren sayfa.
- **tokio, reqwest:** Rust'ın yaygın async çalışma zamanı ve yaygın HTTP istemci kütüphanesi.
- **`spawn_blocking`:** tokio'da "Bu bloklayan işi ayrı bir thread'de yap" çağrısı.
- **async closure:** Çağrılınca future döndüren, `async` ile yazılmış closure.
- **`poll_fn`:** Bir closure'dan elle küçük bir future üretmenin kısa yolu.

</details>

## Sahte sunucu

Önce hata türü. Yeniden denemenin anlamlı olup olmadığını türün kendisi söyler: 503 geçicidir, 404 kalıcıdır.

```rust
use std::sync::Arc;
use std::sync::atomic::AtomicU32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IstekHatasi {
    Gecici(u16), // 503 gibi: yeniden denemek anlamlı
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
        Arc::new(SahteSunucu { kararsiz_cagri: AtomicU32::new(0) })
    }
}

fn main() {
    println!("503 geçici mi? {}", IstekHatasi::Gecici(503).gecici_mi());
    println!("404 geçici mi? {}", IstekHatasi::Bulunamadi.gecici_mi());
}
```

Çıktı:

```text
503 geçici mi? true
404 geçici mi? false
```

Sunucunun dört yolu var: 20 ms'de yanıt veren `/hizli`, 400 ms süren `/yavas`, ilk iki çağrıda 503 veren `/kararsiz` ve geri kalan her şey için 404.

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
use calisma_zamani::{kostur, uyu_ms};
# use std::sync::Arc;
# use std::sync::atomic::{AtomicU32, Ordering};
# #[derive(Debug, Clone, PartialEq, Eq)]
# pub enum IstekHatasi { Gecici(u16), Bulunamadi, ZamanAsimi }
# pub struct SahteSunucu { kararsiz_cagri: AtomicU32 }
# impl SahteSunucu {
#     pub fn yeni() -> Arc<Self> { Arc::new(SahteSunucu { kararsiz_cagri: AtomicU32::new(0) }) }
# }

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

fn main() {
    let sunucu = SahteSunucu::yeni();
    for yol in ["/hizli", "/kararsiz", "/yok"] {
        println!("{yol}: {:?}", kostur(istek(Arc::clone(&sunucu), yol)));
    }
}
```

Çıktı:

```text
/hizli: Ok("hızlı cevap")
/kararsiz: Err(Gecici(503))
/yok: Err(Bulunamadi)
```

Parametreler neden `Arc<SahteSunucu>` ve `&'static str`? Bu future `baslat` ile ayrı bir görev olacaktır ve görevler `'static` olmalıdır: Onları başlatan fonksiyondan uzun yaşayabilirler.[^spawn] `&SahteSunucu` alsaydık future o referansı içinde taşır, görev olarak başlatılamazdı:

```rust,compile_fail,E0597
# mod calisma_zamani {
#     pub struct Baslatici;
#     impl Baslatici {
#         pub fn baslat<F>(&self, _gelecek: F)
#         where
#             F: Future + Send + 'static,
#             F::Output: Send + 'static,
#         {
#         }
#     }
#     pub async fn uyu_ms(_ms: u64) {}
# }
# use calisma_zamani::{Baslatici, uyu_ms};
# use std::sync::atomic::AtomicU32;
# pub struct SahteSunucu { kararsiz_cagri: AtomicU32 }
async fn istek(sunucu: &SahteSunucu) {
    uyu_ms(20).await;
    println!("{:?}", sunucu.kararsiz_cagri);
}

fn gorev_baslat(baslatici: &Baslatici) {
    let sunucu = SahteSunucu { kararsiz_cagri: AtomicU32::new(0) };
    baslatici.baslat(istek(&sunucu));
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0597]: `sunucu` does not live long enough
   |
 7 |     let sunucu = SahteSunucu { kararsiz_cagri: AtomicU32::new(0) };
   |         ------ binding `sunucu` declared here
 8 |     baslatici.baslat(istek(&sunucu));
   |     -----------------------^^^^^^^--
   |     |                      |
   |     |                      borrowed value does not live long enough
   |     argument requires that `sunucu` is borrowed for `'static`
 9 | }
   | - `sunucu` dropped here while still borrowed
```

`gorev_baslat` döndüğünde `sunucu` düşer, görev ise daha sonra yoklanacaktır. Paylaşılan durumu görevlere vermenin yolu thread'lerdekiyle aynıdır: `Arc`.[^arc]

## Yeniden deneme

Birini aradınız, telefon meşgul çaldı. Biraz bekleyip yeniden ararsınız; yine meşgulse bu kez biraz daha uzun beklersiniz. "Böyle bir numara yok" denirse bir daha aramazsınız, çünkü aramak sonucu değiştirmez. `yeniden_dene` bunu yapar:

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
# use calisma_zamani::{kostur, uyu_ms};
# #[derive(Debug, Clone, PartialEq, Eq)]
# pub enum IstekHatasi { Gecici(u16), Bulunamadi, ZamanAsimi }
# impl IstekHatasi {
#     fn gecici_mi(&self) -> bool { matches!(self, IstekHatasi::Gecici(_)) }
# }
# pub struct SahteSunucu { kararsiz_cagri: std::sync::atomic::AtomicU32 }
# impl SahteSunucu {
#     pub fn yeni() -> std::sync::Arc<Self> {
#         std::sync::Arc::new(SahteSunucu { kararsiz_cagri: 0.into() })
#     }
# }
# type Sunucu = std::sync::Arc<SahteSunucu>;
# pub async fn istek(sunucu: Sunucu, yol: &'static str) -> Result<String, IstekHatasi> {
#     match yol {
#         "/hizli" => { uyu_ms(20).await; Ok(String::from("hızlı cevap")) }
#         "/yavas" => { uyu_ms(400).await; Ok(String::from("yavaş cevap")) }
#         "/kararsiz" => {
#             uyu_ms(10).await;
#             let sayac = &sunucu.kararsiz_cagri;
#             let kacinci = 1 + sayac.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
#             if kacinci <= 2 { return Err(IstekHatasi::Gecici(503)); }
#             Ok(format!("{kacinci}. denemede cevap"))
#         }
#         _ => { uyu_ms(5).await; Err(IstekHatasi::Bulunamadi) }
#     }
# }
# use std::sync::Arc;
pub async fn yeniden_dene<F, Fut, T>(en_fazla: u32, mut dene: F) -> Result<T, IstekHatasi>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, IstekHatasi>>,
{
    let mut deneme = 1;
    loop {
        match dene().await {
            // Yalnızca geçici hatada ve hak varsa yeniden dene.
            Err(hata) if hata.gecici_mi() && deneme < en_fazla => {
                uyu_ms(5 * u64::from(deneme)).await;
                deneme += 1;
            }
            // Başarı, kalıcı hata ya da hakkın bitmesi: olduğu gibi döndür.
            sonuc => return sonuc,
        }
    }
}

fn main() {
    let sunucu = SahteSunucu::yeni();
    let sonuc = kostur(yeniden_dene(4, move || {
        println!("istek gönderiliyor");
        istek(Arc::clone(&sunucu), "/kararsiz")
    }));
    println!("{sonuc:?}");
}
```

Çıktı:

```text
istek gönderiliyor
istek gönderiliyor
istek gönderiliyor
Ok("3. denemede cevap")
```

İlk iki deneme `Gecici(503)` döndürdü; aralarında sırasıyla 5 ve 10 ms beklendi. Artan beklemenin (backoff) amacı, hata veren servise hemen yeniden yüklenmemektir. Bekleme `thread::sleep` ile değil `uyu_ms(..).await` ile yapılır; nedenini [biraz sonra](#yaygın-hata-async-içinde-threadi-bloklamak) göreceğiz.

Öteki iki durum da sınanmaya değer: hakkın bitmesi ve kalıcı hata.

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
# use calisma_zamani::{kostur, uyu_ms};
# #[derive(Debug, Clone, PartialEq, Eq)]
# pub enum IstekHatasi { Gecici(u16), Bulunamadi, ZamanAsimi }
# impl IstekHatasi {
#     fn gecici_mi(&self) -> bool { matches!(self, IstekHatasi::Gecici(_)) }
# }
# pub struct SahteSunucu { kararsiz_cagri: std::sync::atomic::AtomicU32 }
# impl SahteSunucu {
#     pub fn yeni() -> std::sync::Arc<Self> {
#         std::sync::Arc::new(SahteSunucu { kararsiz_cagri: 0.into() })
#     }
# }
# type Sunucu = std::sync::Arc<SahteSunucu>;
# pub async fn istek(sunucu: Sunucu, yol: &'static str) -> Result<String, IstekHatasi> {
#     match yol {
#         "/hizli" => { uyu_ms(20).await; Ok(String::from("hızlı cevap")) }
#         "/yavas" => { uyu_ms(400).await; Ok(String::from("yavaş cevap")) }
#         "/kararsiz" => {
#             uyu_ms(10).await;
#             let sayac = &sunucu.kararsiz_cagri;
#             let kacinci = 1 + sayac.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
#             if kacinci <= 2 { return Err(IstekHatasi::Gecici(503)); }
#             Ok(format!("{kacinci}. denemede cevap"))
#         }
#         _ => { uyu_ms(5).await; Err(IstekHatasi::Bulunamadi) }
#     }
# }
# pub async fn yeniden_dene<F, Fut, T>(en_fazla: u32, mut dene: F) -> Result<T, IstekHatasi>
# where
#     F: FnMut() -> Fut,
#     Fut: Future<Output = Result<T, IstekHatasi>>,
# {
#     let mut deneme = 1;
#     loop {
#         match dene().await {
#             Err(hata) if hata.gecici_mi() && deneme < en_fazla => {
#                 uyu_ms(5 * u64::from(deneme)).await;
#                 deneme += 1;
#             }
#             sonuc => return sonuc,
#         }
#     }
# }
# use std::sync::Arc;
# use std::sync::atomic::{AtomicU32, Ordering};
fn main() {
    let sunucu = SahteSunucu::yeni();
    let sonuc = kostur(yeniden_dene(2, move || istek(Arc::clone(&sunucu), "/kararsiz")));
    println!("2 hak: {sonuc:?}");

    let cagri = Arc::new(AtomicU32::new(0));
    let sayac = Arc::clone(&cagri);
    let sunucu = SahteSunucu::yeni();
    let sonuc = kostur(yeniden_dene(5, move || {
        sayac.fetch_add(1, Ordering::SeqCst);
        istek(Arc::clone(&sunucu), "/yok")
    }));
    println!("kalıcı hata: {sonuc:?}, çağrı sayısı: {}", cagri.load(Ordering::SeqCst));
}
```

Çıktı:

```text
2 hak: Err(Gecici(503))
kalıcı hata: Err(Bulunamadi), çağrı sayısı: 1
```

Hak bitince son hata korunur; 404 ise beş hak olduğu hâlde bir kez denenir.

### Neden hazır bir future değil de closure?

`yeniden_dene`'ye tek bir arama yetmez; her basışta yeni bir arama başlatan bir düğme gerekir. Future yalnızca bir kez tamamlanır; bittikten sonra yeniden yoklanmamalıdır.[^panik] Her deneme taze bir future ister. Hazır bir future almayı denersek derleyici de izin vermez, çünkü `.await` future'ı tüketir:

```rust,compile_fail,E0382
# #[derive(Debug)]
# pub enum IstekHatasi { Gecici(u16) }
async fn yeniden_dene<Fut, T>(en_fazla: u32, gelecek: Fut) -> Result<T, IstekHatasi>
where
    Fut: Future<Output = Result<T, IstekHatasi>>,
{
    let mut sonuc = gelecek.await;
    for _ in 1..en_fazla {
        if sonuc.is_ok() {
            break;
        }
        sonuc = gelecek.await;
    }
    sonuc
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0382]: use of moved value: `gelecek`
   |
   |     let mut sonuc = gelecek.await;
   |                             ----- `gelecek` moved due to this await
...
   |         sonuc = gelecek.await;
   |                 ^^^^^^^ value used here after move
```

Rust 1.85'ten beri aynı sınır bir *async closure* ile de yazılabilir: `impl AsyncFnMut() -> Result<T, IstekHatasi>`.[^async-closure] Çağıran da `async move || ..` verir:

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
# use calisma_zamani::{kostur, uyu_ms};
# #[derive(Debug, Clone, PartialEq, Eq)]
# pub enum IstekHatasi { Gecici(u16), Bulunamadi, ZamanAsimi }
# impl IstekHatasi {
#     fn gecici_mi(&self) -> bool { matches!(self, IstekHatasi::Gecici(_)) }
# }
# pub struct SahteSunucu { kararsiz_cagri: std::sync::atomic::AtomicU32 }
# impl SahteSunucu {
#     pub fn yeni() -> std::sync::Arc<Self> {
#         std::sync::Arc::new(SahteSunucu { kararsiz_cagri: 0.into() })
#     }
# }
# type Sunucu = std::sync::Arc<SahteSunucu>;
# pub async fn istek(sunucu: Sunucu, yol: &'static str) -> Result<String, IstekHatasi> {
#     match yol {
#         "/hizli" => { uyu_ms(20).await; Ok(String::from("hızlı cevap")) }
#         "/yavas" => { uyu_ms(400).await; Ok(String::from("yavaş cevap")) }
#         "/kararsiz" => {
#             uyu_ms(10).await;
#             let sayac = &sunucu.kararsiz_cagri;
#             let kacinci = 1 + sayac.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
#             if kacinci <= 2 { return Err(IstekHatasi::Gecici(503)); }
#             Ok(format!("{kacinci}. denemede cevap"))
#         }
#         _ => { uyu_ms(5).await; Err(IstekHatasi::Bulunamadi) }
#     }
# }
# use std::sync::Arc;
pub async fn yeniden_dene<T>(
    en_fazla: u32,
    mut dene: impl AsyncFnMut() -> Result<T, IstekHatasi>,
) -> Result<T, IstekHatasi> {
    let mut deneme = 1;
    loop {
        match dene().await {
            Err(hata) if hata.gecici_mi() && deneme < en_fazla => {
                uyu_ms(5 * u64::from(deneme)).await;
                deneme += 1;
            }
            sonuc => return sonuc,
        }
    }
}

fn main() {
    let sunucu = SahteSunucu::yeni();
    let sonuc = kostur(yeniden_dene(4, async move || {
        istek(Arc::clone(&sunucu), "/kararsiz").await
    }));
    println!("{sonuc:?}");
}
```

Çıktı:

```text
Ok("3. denemede cevap")
```

Bu bölümde klasik biçimi kullanıyoruz; "Closure çağrılır, future döner, future beklenir" adımları açıkça görünsün diye. Closure'lar ile trait'lerinin ilişkisi [12. bölümün](../orta/12-closures-iterators.md) konusudur.

## Pano: üç kaynağı eşzamanlı yüklemek

Üç arkadaşınıza birer soru soracaksınız; biri 20, biri 35, biri 60 dakikada yanıt yazıyor. Üç mesajı aynı anda gönderirseniz 60 dakika beklersiniz. Birinin yanıtını bekleyip ötekine ondan sonra yazarsanız 115 dakika sürer. Ağ istekleri zamanının çoğunu bekleyerek geçirir; beklemeleri üst üste bindirmek toplam süreyi en uzun isteğin süresine yaklaştırır.[^esazmanli]

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
#     #[derive(Debug, PartialEq, Eq, Clone, Copy)]
#     pub struct ZamanAsimiHatasi;
#     pub struct ZamanAsimi<T> { ic: Pin<Box<dyn Future<Output = T> + Send>>, sayac: Uyku }
#     pub fn zaman_asimi<F>(sure: Duration, gelecek: F) -> ZamanAsimi<F::Output>
#     where F: Future + Send + 'static {
#         ZamanAsimi { ic: Box::pin(gelecek), sayac: Uyku::yeni(sure) }
#     }
#     impl<T> Future for ZamanAsimi<T> {
#         type Output = Result<T, ZamanAsimiHatasi>;
#         fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
#             let bu = self.get_mut();
#             if let Poll::Ready(deger) = bu.ic.as_mut().poll(cx) { return Poll::Ready(Ok(deger)); }
#             Pin::new(&mut bu.sayac).poll(cx).map(|()| Err(ZamanAsimiHatasi))
#         }
#     }
# }
use calisma_zamani::{Calistirici, uyu_ms, zaman_asimi};
use std::sync::Arc;
use std::time::{Duration, Instant};
# #[derive(Debug, Clone, PartialEq, Eq)]
# pub enum IstekHatasi { Gecici(u16), Bulunamadi, ZamanAsimi }
# impl IstekHatasi {
#     fn gecici_mi(&self) -> bool { matches!(self, IstekHatasi::Gecici(_)) }
# }
# pub struct SahteSunucu { kararsiz_cagri: std::sync::atomic::AtomicU32 }
# impl SahteSunucu {
#     pub fn yeni() -> std::sync::Arc<Self> {
#         std::sync::Arc::new(SahteSunucu { kararsiz_cagri: 0.into() })
#     }
# }
# type Sunucu = std::sync::Arc<SahteSunucu>;
# pub async fn istek(sunucu: Sunucu, yol: &'static str) -> Result<String, IstekHatasi> {
#     match yol {
#         "/hizli" => { uyu_ms(20).await; Ok(String::from("hızlı cevap")) }
#         "/yavas" => { uyu_ms(400).await; Ok(String::from("yavaş cevap")) }
#         "/kararsiz" => {
#             uyu_ms(10).await;
#             let sayac = &sunucu.kararsiz_cagri;
#             let kacinci = 1 + sayac.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
#             if kacinci <= 2 { return Err(IstekHatasi::Gecici(503)); }
#             Ok(format!("{kacinci}. denemede cevap"))
#         }
#         _ => { uyu_ms(5).await; Err(IstekHatasi::Bulunamadi) }
#     }
# }
# pub async fn yeniden_dene<F, Fut, T>(en_fazla: u32, mut dene: F) -> Result<T, IstekHatasi>
# where
#     F: FnMut() -> Fut,
#     Fut: Future<Output = Result<T, IstekHatasi>>,
# {
#     let mut deneme = 1;
#     loop {
#         match dene().await {
#             Err(hata) if hata.gecici_mi() && deneme < en_fazla => {
#                 uyu_ms(5 * u64::from(deneme)).await;
#                 deneme += 1;
#             }
#             sonuc => return sonuc,
#         }
#     }
# }

pub fn pano_yukle() -> ([Result<String, IstekHatasi>; 3], Duration) {
    let sunucu = SahteSunucu::yeni();
    let mut calistirici = Calistirici::yeni();
    let baslatici = calistirici.baslatici();
    let baslangic = Instant::now();

    // Her istek ayrı görev; üçü de hemen kuyruğa girer.
    let hizli = baslatici.baslat(istek(Arc::clone(&sunucu), "/hizli"));

    let kararsiz = {
        let sunucu = Arc::clone(&sunucu);
        baslatici.baslat(yeniden_dene(4, move || istek(Arc::clone(&sunucu), "/kararsiz")))
    };

    let yavas = baslatici.baslat(async move {
        match zaman_asimi(Duration::from_millis(60), istek(sunucu, "/yavas")).await {
            Ok(cevap) => cevap,                     // süre dolmadan geldi
            Err(_) => Err(IstekHatasi::ZamanAsimi), // süre doldu: istek iptal edildi
        }
    });

    calistirici.calistir(); // bütün görevler bitene dek
    let sonuclar = [hizli, kararsiz, yavas].map(|t| t.sonucu_al().expect("görev bitmeliydi"));
    (sonuclar, baslangic.elapsed())
}

fn main() {
    let (sonuclar, sure) = pano_yukle();
    for (ad, sonuc) in ["hızlı", "kararsız", "yavaş"].iter().zip(&sonuclar) {
        println!("{ad:<8} : {sonuc:?}");
    }
    let sirali_toplam = Duration::from_millis(125);
    println!("sıralı toplamdan (125 ms) kısa mı? {}", sure < sirali_toplam);
}
```

Çıktı:

```text
hızlı    : Ok("hızlı cevap")
kararsız : Ok("3. denemede cevap")
yavaş    : Err(ZamanAsimi)
sıralı toplamdan (125 ms) kısa mı? true
```

Her görevin sonucu ayrı taşınır: Biri başarılı, biri üçüncü denemede başarılı, biri zaman aşımı. Sıralı yapılsaydı uykuların toplamı 20 + (10 + 5 + 10 + 10 + 10) + 60 = 125 ms olurdu. Eşzamanlı yürüyünce süreyi en uzun görev belirler, yani yaklaşık 60 ms.

İki ayrıntı dikkat istiyor. Yeniden deneme closure'ı `move` ile `Arc`'ın sahibi olur ve her çağrıda yeni bir klonla taze bir future üretir. `zaman_asimi` ise iç içe iki `Result` verir: Dıştaki sürenin dolup dolmadığını, içteki isteğin kendi sonucunu söyler; `match` ikisini tek hata türüne indirir.

tokio ile yazsaydınız biçim aynı kalırdı:

| Buradaki | tokio karşılığı | Fark |
| --- | --- | --- |
| `baslatici.baslat(f)` | `tokio::spawn(f)` | İkisi de `Send + 'static` ister.[^spawn] |
| `Tutamac<T>` | `JoinHandle<T>` | tokio'da `.await`, görevin paniğini de bildiren bir `Result` verir. Tutamacı düşürmek görevi durdurmaz.[^join-handle] |
| `zaman_asimi(sure, f)` | `tokio::time::timeout(sure, f)` | İkisi de önce asıl işi yoklar, süre dolunca işi düşürür.[^timeout] |
| `uyu_ms(n)` | `tokio::time::sleep(..)` | tokio uykuları çalışma zamanının zamanlayıcısında izler, thread açmaz.[^tokio-runtime] |
| `calistirici.calistir()` | `Runtime::block_on(..)` | `block_on` verilen future bitince döner; `calistir` bütün görevleri bekler.[^block-on] |
| `ayri_threadde(is)` (aşağıda) | `tokio::task::spawn_blocking(is)` | tokio thread'leri bir havuzdan verir.[^spawn-blocking] |

Sahte sunucunun yerini gerçek HTTP isteği almak isterse reqwest'in `async` istemcisi aynı yere oturur.[^reqwest]

## Yaygın hata: async içinde thread'i bloklamak

Lokantada tek garson var. İyi garson siparişi mutfağa bırakır ve yemek pişerken öteki masalara bakar. Kötü garson mutfağın kapısında yemek pişene kadar dikilir; o sırada bütün masalar bekler. Çalıştırıcı o tek garsondur. `uyu_ms(30).await`, "Ben bekliyorum, siz ötekilere bakın" demektir; `thread::sleep` garsonun kendisini uyutur.

Çalıştırıcımız tek thread'de yoklar; çok thread'li çalışma zamanlarında da thread sayısı sınırlıdır. Bir görev yoklama içinde uzun kalırsa o thread'deki bütün öteki görevler durur. `thread::sleep`, bloklayan bir dosya ya da ağ çağrısı, ağır bir hesap, `std`'nin `Mutex`'inde uzun bekleme hep aynı sonucu verir:

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
use std::thread;
use std::time::{Duration, Instant};

async fn kotu_bekleme() {
    thread::sleep(Duration::from_millis(30)); // çalıştırıcının thread'ini tutar
}

async fn iyi_bekleme() {
    uyu_ms(30).await; // Pending döner, thread geri verilir
}

// Aynı türden 3 görevi başlatıp toplam süreyi ölçer.
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

fn main() {
    let esik = Duration::from_millis(90); // 3 × 30 ms
    println!("thread::sleep ile en az 90 ms mi? {}", uc_gorev_suresi(kotu_bekleme) >= esik);
    println!("async uyku ile 90 ms'den az mı?   {}", uc_gorev_suresi(iyi_bekleme) < esik);
}
```

Çıktı:

```text
thread::sleep ile en az 90 ms mi? true
async uyku ile 90 ms'den az mı?   true
```

Derleyici bu hatayı yakalamaz: Kod derlenir, doğru sonucu da verir; yalnızca eşzamanlılık kaybolur. `Future` belgesi `poll`'un bloklamamasını ister,[^runtime] `thread::sleep` belgesi de çağrının async fonksiyonlarda kullanılmaması gerektiğini söyler.[^sleep] Kural basittir: İki `.await` arasında geçen süre kısa olmalı, beklemek için çalışma zamanının async araçları kullanılmalıdır.

### Kaçınılmaz bloklayan işi ayrı thread'e vermek

Garsonun patates soyması gerekirse salonu bırakıp mutfağa kapanmaz; işi mutfaktaki yardımcıya verir ve "Bitince haber ver" der. `ayri_threadde` uzun süren, bölünemeyen bir işi yeni bir thread'e yollar. O iş bitince uyandırıcı çağrılır, çalıştırıcı bu sırada öteki görevlere bakmayı sürdürür:

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
# use calisma_zamani::{Calistirici, kostur};
# use std::time::{Duration, Instant};
# fn uc_gorev_suresi<F, Fut>(uret: F) -> Duration
# where
#     F: Fn() -> Fut,
#     Fut: Future<Output = ()> + Send + 'static,
# {
#     let mut calistirici = Calistirici::yeni();
#     let baslatici = calistirici.baslatici();
#     let baslangic = Instant::now();
#     for _ in 0..3 { baslatici.baslat(uret()); }
#     calistirici.calistir();
#     baslangic.elapsed()
# }
use std::future::poll_fn;
use std::sync::{Arc, Mutex};
use std::task::{Poll, Waker};
use std::thread;

pub async fn ayri_threadde<T, F>(is: F) -> T
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let yuva: Arc<Mutex<(Option<T>, Option<Waker>)>> = Arc::new(Mutex::new((None, None)));
    let isci_yuvasi = Arc::clone(&yuva);
    thread::spawn(move || {
        let sonuc = is(); // bloklayan iş, çalıştırıcının thread'i dışında
        let waker = {
            let mut kilit = isci_yuvasi.lock().expect("kilit zehirlenmemeli");
            kilit.0 = Some(sonuc);
            kilit.1.take()
        };
        if let Some(waker) = waker {
            waker.wake();
        }
    });
    poll_fn(move |cx| {
        let mut kilit = yuva.lock().expect("kilit zehirlenmemeli");
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

fn main() {
    let sure = uc_gorev_suresi(|| ayri_threadde(|| thread::sleep(Duration::from_millis(30))));
    println!("ayrı thread ile 90 ms'den az mı? {}", sure < Duration::from_millis(90));

    let kimlik = thread::current().id();
    let baska = kostur(ayri_threadde(move || thread::current().id() != kimlik));
    println!("iş başka bir thread'de mi yapıldı? {baska}");
}
```

Çıktı:

```text
ayrı thread ile 90 ms'den az mı? true
iş başka bir thread'de mi yapıldı? true
```

Kalıp `Tutamac`'la aynıdır: paylaşılan bir yuva ve uyandırıcı. `poll_fn` ayrı bir tür yazmadan, bir closure'dan küçük bir future üretir.[^poll-fn] İptalin sınırına dikkat edin: Bu future düşürülse bile başlamış thread durmaz. `spawn_blocking` de aynıdır; başladıktan sonra durdurulamaz.[^spawn-blocking]

İşin çoğu hesapsa, yani beklemek yerine işlemciyi kullanıyorsa, async'in bir kazancı yoktur; thread'ler daha uygun bir araçtır.[^secim]

## Yaygın hata: kilidi `.await` boyunca tutmak

Ortak dolabın tek anahtarı var. Anahtarı alıp eşyanızı çıkarır, anahtarı yerine asar, sonra otobüs beklemeye gidersiniz. Anahtar cebinizdeyken otobüs beklerseniz dolaba ihtiyacı olan herkes dönmenizi bekler. Kilit anahtardır, `.await` otobüs beklemektir.

Hatalı kod derlenmez:

```rust,compile_fail
# mod calisma_zamani {
#     pub struct Baslatici;
#     impl Baslatici {
#         pub fn baslat<F>(&self, _gelecek: F)
#         where
#             F: Future + Send + 'static,
#             F::Output: Send + 'static,
#         {
#         }
#     }
#     pub async fn uyu_ms(_ms: u64) {}
# }
# use calisma_zamani::{Baslatici, uyu_ms};
# use std::sync::{Arc, Mutex};
async fn sayaci_artir(sayac: Arc<Mutex<u32>>) -> u32 {
    let mut kilit = sayac.lock().expect("kilit zehirlenmemeli");
    uyu_ms(5).await; // koruyucu bu bekleme boyunca elde
    *kilit += 1;
    *kilit
}

fn baslat(baslatici: &Baslatici, sayac: Arc<Mutex<u32>>) {
    baslatici.baslat(sayaci_artir(sayac));
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error: future cannot be sent between threads safely
   |
   |     baslatici.baslat(sayaci_artir(sayac));
   |                      ^^^^^^^^^^^^^^^^^^^ future returned by `sayaci_artir` is not `Send`
   |
   = help: within `impl Future<Output = u32>`, the trait `Send` is not implemented for `std::sync::MutexGuard<'_, u32>`
note: future is not `Send` as this value is used across an await
   |
   |     let mut kilit = sayac.lock().expect("kilit zehirlenmemeli");
   |         --------- has type `std::sync::MutexGuard<'_, u32>` which is not `Send`
   |     uyu_ms(5).await;
   |               ^^^^^ await occurs here, with `mut kilit` maybe used later
```

Bu hatanın `E` kodu yoktur; derleyici onu async'e özgü bu açıklamayla değiştirirken kodu kaldırır.[^kodsuz] Nedeni şudur: `async fn`'in durum makinesi, bekleme noktasında canlı olan her yerel değişkeni içinde saklar ve auto trait'leri bu alanlardan türetir.[^tanik] `MutexGuard` `Send` olmadığı için ([29.1'deki tablo](29-threadler.md)) future da `Send` olmaz ve `baslat`'ın `F: Send` sınırı sağlanmaz. Aynı mesajı `Rc`, `RefCell` ödünçleri (`Ref`, `RefMut`) ve ham işaretçiler için de görürsünüz.

Doğrusu, kilitle işi bitirip koruyucuyu bir blokla düşürmek ve `.await`'e kilitsiz girmektir:

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
use calisma_zamani::{Calistirici, kostur, uyu_ms};
use std::sync::{Arc, Mutex};

pub async fn sayaci_artir(sayac: Arc<Mutex<u32>>) -> u32 {
    let yeni_deger = {
        let mut kilit = sayac.lock().expect("kilit zehirlenmemeli");
        *kilit += 1;
        *kilit
    }; // koruyucu burada düştü
    uyu_ms(5).await; // sonucu bir yere yazdığımızı varsayın
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
    *sayac.lock().expect("kilit zehirlenmemeli")
}

fn main() {
    println!("5 görev sayacı artırdı: {}", paylasimli_sayac(5));
    println!("görev içinden sonuç: {}", kostur(sayaci_artir(Arc::new(Mutex::new(41)))));
}
```

Çıktı:

```text
5 görev sayacı artırdı: 5
görev içinden sonuç: 42
```

### `drop` çağırmak neden yetmiyor?

Akla gelen ilk çözüm, blok yerine `drop(kilit)` yazmaktır. rustc 1.99.0'da bu yetmez:

```rust,compile_fail
# mod calisma_zamani {
#     pub struct Baslatici;
#     impl Baslatici {
#         pub fn baslat<F>(&self, _gelecek: F)
#         where
#             F: Future + Send + 'static,
#             F::Output: Send + 'static,
#         {
#         }
#     }
#     pub async fn uyu_ms(_ms: u64) {}
# }
# use calisma_zamani::{Baslatici, uyu_ms};
# use std::sync::{Arc, Mutex};
async fn sayaci_artir(sayac: Arc<Mutex<u32>>) -> u32 {
    let mut kilit = sayac.lock().expect("kilit zehirlenmemeli");
    *kilit += 1;
    let yeni_deger = *kilit;
    drop(kilit); // koruyucu düştü, ama kapsamı sürüyor
    uyu_ms(5).await;
    yeni_deger
}

fn baslat(baslatici: &Baslatici, sayac: Arc<Mutex<u32>>) {
    baslatici.baslat(sayaci_artir(sayac));
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error: future cannot be sent between threads safely
   = help: within `impl Future<Output = u32>`, the trait `Send` is not implemented for `std::sync::MutexGuard<'_, u32>`
note: future is not `Send` as this value is used across an await
```

Çalışma zamanında kilit gerçekten bırakılmıştır; ama derleyici hangi yerelin bekleme noktasında saklanacağına karar verirken, daha önce ödünç alınmış bir değişkeni taşındıktan sonra da saklanmış sayar (`*kilit` yazmak koruyucuyu ödünç alır).[^saklama] Koruyucunun *kapsamını* bitirmek gerekir. tokio öğreticisi de aynı nedenle blok kullanmayı önerir.[^tokio-kilit] Tek deyimlik `*sayac.lock().expect("kilit zehirlenmemeli") += 1;` de güvenlidir, çünkü geçici koruyucu deyimin sonunda düşer.

Geçici koruyucular başka bir yerde de şaşırtır. `match` ve `if let`'in incelediği ifadedeki geçici değer, seçilen kolun gövdesi boyunca yaşar:[^gecici-kapsam]

```rust,compile_fail
# mod calisma_zamani {
#     pub struct Baslatici;
#     impl Baslatici {
#         pub fn baslat<F>(&self, _gelecek: F)
#         where
#             F: Future + Send + 'static,
#             F::Output: Send + 'static,
#         {
#         }
#     }
#     pub async fn uyu_ms(_ms: u64) {}
# }
# use calisma_zamani::{Baslatici, uyu_ms};
# use std::sync::{Arc, Mutex};
async fn bosalt(kuyruk: Arc<Mutex<Vec<u64>>>) {
    if let Some(ms) = kuyruk.lock().expect("kilit zehirlenmemeli").pop() {
        uyu_ms(ms).await; // geçici koruyucu hâlâ yaşıyor
    }
}

fn baslat(baslatici: &Baslatici, kuyruk: Arc<Mutex<Vec<u64>>>) {
    baslatici.baslat(bosalt(kuyruk));
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error: future cannot be sent between threads safely
```

Değeri önce bir `let` ile çıkarmak yeter; geçici koruyucu o deyimin sonunda düşer:

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
# use calisma_zamani::{kostur, uyu_ms};
# use std::sync::{Arc, Mutex};
async fn bosalt(kuyruk: Arc<Mutex<Vec<u64>>>) {
    let siradaki = kuyruk.lock().expect("kilit zehirlenmemeli").pop();
    if let Some(ms) = siradaki {
        uyu_ms(ms).await;
    }
}

fn main() {
    let kuyruk = Arc::new(Mutex::new(vec![1, 2]));
    kostur(bosalt(Arc::clone(&kuyruk)));
    println!("kalan: {:?}", kuyruk.lock().expect("kilit zehirlenmemeli"));
}
```

Çıktı:

```text
kalan: [1]
```

### Derleyici sizi neyden koruyor?

`Send` istemeyen tek thread'li bir çalışma zamanında hatalı kod derlenir ve ikinci bir tehlike ortaya çıkar: Görev A kilidi alıp `.await`'te bekler. Aynı thread'deki görev B `lock` der ve thread'i bloklar. A artık yoklanamaz, kilidi bırakamaz; program kilitlenir. Görevleri elle yoklayarak o anı dondurabiliriz:

```rust
# use std::pin::{Pin, pin};
# use std::sync::{Mutex, TryLockError};
# use std::task::{Context, Poll, Waker};
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
async fn a_gorevi(kilit: &Mutex<u32>) {
    let mut koruyucu = kilit.lock().expect("kilit zehirlenmemeli");
    sira_ver().await; // kilit eldeyken bekliyor
    *koruyucu += 1;
}

fn main() {
    let kilit = Mutex::new(0);
    let mut a = pin!(a_gorevi(&kilit));
    let mut cx = Context::from_waker(Waker::noop());

    assert!(a.as_mut().poll(&mut cx).is_pending()); // A kilidi aldı ve askıda
    match kilit.try_lock() {
        Err(TryLockError::WouldBlock) => println!("B lock() deseydi thread bloklanırdı"),
        sonuc => println!("{sonuc:?}"),
    }

    assert!(a.as_mut().poll(&mut cx).is_ready()); // A bitti, kilit bırakıldı
    println!("şimdi: {:?}", kilit.try_lock().map(|k| *k));
}
```

Çıktı:

```text
B lock() deseydi thread bloklanırdı
şimdi: Ok(1)
```

Bu örnekte A'yı yeniden yoklayabildik, çünkü `try_lock` thread'i bloklamadı. `lock` yazsaydık o satıra hiç gelemezdik.

Kilidi bekleme boyunca tutmak *gerçekten* gerekiyorsa async'ten haberdar bir kilit gerekir. `tokio::sync::Mutex` beklerken thread'i bloklamaz, görevi askıya alır ve koruyucusu `.await` boyunca tutulabilir; karşılığında `std` kilidinden daha pahalıdır ve tokio belgesi, bekleme boyunca tutmayacaksanız `std`'ninkini önerir.[^tokio-mutex] `Send` sınırının yakalamadığı durumlar için Clippy'nin `await_holding_lock` denetimi vardır.[^clippy]

## Aynı aileden hatalar

### Yereli ödünç alan görev

29.2'de tek görevle gördüğümüz `E0373`, iki görev aynı veriyi istediğinde de karşınıza çıkar:

```rust,compile_fail,E0373
# mod calisma_zamani {
#     pub struct Baslatici;
#     impl Baslatici {
#         pub fn baslat<F>(&self, _gelecek: F)
#         where
#             F: Future + Send + 'static,
#             F::Output: Send + 'static,
#         {
#         }
#     }
#     pub async fn uyu_ms(_ms: u64) {}
# }
# use calisma_zamani::Baslatici;
fn iki_selam(baslatici: &Baslatici) {
    let ad = String::from("ada");
    baslatici.baslat(async { println!("bir: {ad}") });
    baslatici.baslat(async { println!("iki: {ad}") });
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0373]: async block may outlive the current function, but it borrows `ad`, which is owned by the current function
  = note: async blocks are not executed immediately and must either take a reference or ownership of outside variables they use
```

Tek görevde `async move` yeterdi; paylaşılacak veri için her göreve bir `Arc` klonu taşıyın:[^e0373]

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
# use calisma_zamani::{Baslatici, Calistirici};
use std::sync::Arc;

fn iki_selam(baslatici: &Baslatici) {
    let ad: Arc<str> = Arc::from("ada");
    let ikinci_ad = Arc::clone(&ad);
    baslatici.baslat(async move { println!("bir: {ad}") });
    baslatici.baslat(async move { println!("iki: {ikinci_ad}") });
}

fn main() {
    let mut calistirici = Calistirici::yeni();
    iki_selam(&calistirici.baslatici());
    calistirici.calistir();
}
```

Çıktı:

```text
bir: ada
iki: ada
```

### `Rc`'yi `.await` boyunca taşımak

```rust,compile_fail
# mod calisma_zamani {
#     pub struct Baslatici;
#     impl Baslatici {
#         pub fn baslat<F>(&self, _gelecek: F)
#         where
#             F: Future + Send + 'static,
#             F::Output: Send + 'static,
#         {
#         }
#     }
#     pub async fn uyu_ms(_ms: u64) {}
# }
# use calisma_zamani::{Baslatici, uyu_ms};
use std::rc::Rc;

fn baslat(baslatici: &Baslatici) {
    baslatici.baslat(async {
        let r = Rc::new(1);
        uyu_ms(1).await;
        println!("{r}");
    });
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error: future cannot be sent between threads safely
note: future is not `Send` as this value is used across an await
   |
   |         let r = Rc::new(1);
   |             - has type `Rc<i32>` which is not `Send`
   |         uyu_ms(1).await;
   |                   ^^^^^ await occurs here, with `r` maybe used later
```

İki çözüm vardır: `Arc` kullanmak ya da `Rc`'nin kapsamını `.await`'ten önce bitirmek.

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
# use calisma_zamani::{kostur, uyu_ms};
use std::rc::Rc;
use std::sync::Arc;

async fn arc_ile() {
    let r = Arc::new(1);
    uyu_ms(1).await;
    println!("Arc: {r}");
}

async fn rc_ile() {
    let deger = {
        let r = Rc::new(2);
        *r
    }; // Rc burada düştü
    uyu_ms(1).await;
    println!("Rc'den alınan değer: {deger}");
}

fn main() {
    kostur(arc_ile());
    kostur(rc_ile());
}
```

Çıktı:

```text
Arc: 1
Rc'den alınan değer: 2
```

### Uyandırıcısız `Pending`

Üçüncü hata derlenir, ama asılı kalır: Elle yazılmış bir future, uyandırıcıyı kimseye vermeden `Pending` döndürürse çalıştırıcı onu bir daha yoklamaz ve `calistir` dönmez. Bu hatayı ve düzeltmesini [29.2'nin sık hatalarında](29-calisma-zamani.md) gördük: `Pending`'den önce `cx.waker()`'ı olayı bilecek tarafa teslim edin; denetimle kaydı aynı kilit altında yapın.

## Akılda kalması gerekenler

- `baslat` isteği hemen kuyruğa koyar; `.await` ve `sonucu_al` yalnızca sonucu bekler. Eşzamanlılık başlatmadan gelir.
- Hata türünü yeniden denemeyi düşünerek tasarlayın; kalıcı hatayı yeniden denemeyin.
- Yeniden deneme future değil, future *üreten* bir şey ister.
- İki `.await` arası kısa olsun. Bloklayan iş ayrı thread'e, hesap işi thread'lere gider.
- Kilidin koruyucusunu blokla düşürün; `drop` çağrısına güvenmeyin.
- "future cannot be sent" mesajında derleyicinin gösterdiği değeri bulun ve kapsamını `.await`'ten önce bitirin.

## Alıştırmalar

**1.** `yeniden_dene`'ye toplam süre sınırı ekleyin: `zaman_asimi` ile sararak "en fazla 4 deneme ve toplam 50 ms" kuralını kurun. Zaman aşımı bir denemenin ortasında dolarsa ne olur? `yeniden_dene`'nin future'ının `Send + 'static` olması gerekir; hangi parametreler bunu belirler?

<details>
<summary>İpucu ve çözüm</summary>

İki `Result` katmanını `unwrap_or` ile tek katmana indirebilirsiniz: Süre dolduysa `Err(IstekHatasi::ZamanAsimi)`, dolmadıysa denemelerin kendi sonucu.

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
#     #[derive(Debug, PartialEq, Eq, Clone, Copy)]
#     pub struct ZamanAsimiHatasi;
#     pub struct ZamanAsimi<T> { ic: Pin<Box<dyn Future<Output = T> + Send>>, sayac: Uyku }
#     pub fn zaman_asimi<F>(sure: Duration, gelecek: F) -> ZamanAsimi<F::Output>
#     where F: Future + Send + 'static {
#         ZamanAsimi { ic: Box::pin(gelecek), sayac: Uyku::yeni(sure) }
#     }
#     impl<T> Future for ZamanAsimi<T> {
#         type Output = Result<T, ZamanAsimiHatasi>;
#         fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
#             let bu = self.get_mut();
#             if let Poll::Ready(deger) = bu.ic.as_mut().poll(cx) { return Poll::Ready(Ok(deger)); }
#             Pin::new(&mut bu.sayac).poll(cx).map(|()| Err(ZamanAsimiHatasi))
#         }
#     }
# }
# use calisma_zamani::{kostur, uyu_ms, zaman_asimi};
# #[derive(Debug, Clone, PartialEq, Eq)]
# pub enum IstekHatasi { Gecici(u16), Bulunamadi, ZamanAsimi }
# impl IstekHatasi {
#     fn gecici_mi(&self) -> bool { matches!(self, IstekHatasi::Gecici(_)) }
# }
# pub struct SahteSunucu { kararsiz_cagri: std::sync::atomic::AtomicU32 }
# impl SahteSunucu {
#     pub fn yeni() -> std::sync::Arc<Self> {
#         std::sync::Arc::new(SahteSunucu { kararsiz_cagri: 0.into() })
#     }
# }
# type Sunucu = std::sync::Arc<SahteSunucu>;
# pub async fn istek(sunucu: Sunucu, yol: &'static str) -> Result<String, IstekHatasi> {
#     match yol {
#         "/hizli" => { uyu_ms(20).await; Ok(String::from("hızlı cevap")) }
#         "/yavas" => { uyu_ms(400).await; Ok(String::from("yavaş cevap")) }
#         "/kararsiz" => {
#             uyu_ms(10).await;
#             let sayac = &sunucu.kararsiz_cagri;
#             let kacinci = 1 + sayac.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
#             if kacinci <= 2 { return Err(IstekHatasi::Gecici(503)); }
#             Ok(format!("{kacinci}. denemede cevap"))
#         }
#         _ => { uyu_ms(5).await; Err(IstekHatasi::Bulunamadi) }
#     }
# }
# pub async fn yeniden_dene<F, Fut, T>(en_fazla: u32, mut dene: F) -> Result<T, IstekHatasi>
# where
#     F: FnMut() -> Fut,
#     Fut: Future<Output = Result<T, IstekHatasi>>,
# {
#     let mut deneme = 1;
#     loop {
#         match dene().await {
#             Err(hata) if hata.gecici_mi() && deneme < en_fazla => {
#                 uyu_ms(5 * u64::from(deneme)).await;
#                 deneme += 1;
#             }
#             sonuc => return sonuc,
#         }
#     }
# }
use std::sync::Arc;
use std::time::Duration;

async fn sinirli_dene(
    sunucu: Arc<SahteSunucu>,
    yol: &'static str,
) -> Result<String, IstekHatasi> {
    let denemeler = yeniden_dene(4, move || istek(Arc::clone(&sunucu), yol));
    zaman_asimi(Duration::from_millis(50), denemeler)
        .await
        .unwrap_or(Err(IstekHatasi::ZamanAsimi))
}

fn main() {
    let sunucu = SahteSunucu::yeni();
    let yavas = kostur(sinirli_dene(Arc::clone(&sunucu), "/yavas"));
    assert_eq!(yavas, Err(IstekHatasi::ZamanAsimi));

    let yok = kostur(sinirli_dene(sunucu, "/yok"));
    assert_eq!(yok, Err(IstekHatasi::Bulunamadi)); // kalıcı hata, süre dolmadan döner
}
```

Süre bir denemenin ortasında dolarsa iç future düşürülür. Onunla birlikte o anda beklenen `istek` de düşer; `.await`'ten sonraki satırları çalışmaz. Gerçek bir ağ isteğinde sunucu isteği çoktan almış olabilir: İptal, istemcinin beklemeyi bırakmasıdır, sunucunun işi geri alması değil. `/kararsiz` yolu toplam 45 ms sürdüğü için 50 ms sınırına çok yakındır; yavaş bir makinede o da zaman aşımına uğrayabilir.

`Send + 'static`'i üç tür parametresi belirler. `F`, yani closure, future'ın içinde saklanır; yakaladığı `Arc<SahteSunucu>`'nun `Send` olması için `SahteSunucu`'nun `Send + Sync` olması gerekir. `Fut` beklenirken saklanır. `T` de sayılır, çünkü `match`'in incelediği `Result<T, IstekHatasi>` geçicisi içerideki `uyu_ms(..).await` boyunca yaşar.

</details>

## Özet

- `istek` görev olarak başlatılacağı için `Arc` ve `&'static str` alır; ödünç alınmış referans `'static` sınırını aşamaz.
- `yeniden_dene` yalnızca geçici hatayı, artan beklemeyle yeniden dener ve her deneme için taze bir future üretir.
- Pano üç isteği birlikte yürütür; süreyi en uzun görev belirler.
- Bloklayan iş çalıştırıcıyı durdurur; `ayri_threadde` onu başka bir thread'e taşır.
- Kilit, `.await`'ten önce kapsamıyla birlikte bırakılır.

Kitabın konu anlatımı burada bitiyor. Bölümün haritasına [29. İleri Eşzamanlılık ve Async](29-ileri-eszamanlilik-async.md) sayfasından dönebilirsiniz; kullanılan kaynakların ve doğrulama yönteminin açıklaması [Kaynaklar ve doğrulama](../kaynaklar.md) ekindedir.

## Kaynaklar

[^spawn]: tokio, [`spawn`](https://docs.rs/tokio/latest/tokio/task/fn.spawn.html); std, [`thread::spawn`](https://doc.rust-lang.org/std/thread/fn.spawn.html).
[^arc]: The Rust Programming Language, [16.3 Shared-State Concurrency](https://doc.rust-lang.org/book/ch16-03-shared-state.html#controlling-access-with-mutexes).
[^panik]: std, [`Future::poll`: Panics](https://doc.rust-lang.org/std/future/trait.Future.html#panics).
[^async-closure]: Rust Blog, [Rust 1.85.0: async closures](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/#async-closures); std, [`AsyncFnMut`](https://doc.rust-lang.org/std/ops/trait.AsyncFnMut.html).
[^esazmanli]: The Rust Programming Language, [17.2 Applying Concurrency with Async](https://doc.rust-lang.org/book/ch17-02-concurrency-with-async.html).
[^join-handle]: tokio, [`task::JoinHandle`](https://docs.rs/tokio/latest/tokio/task/struct.JoinHandle.html).
[^timeout]: tokio, [`time::timeout`](https://docs.rs/tokio/latest/tokio/time/fn.timeout.html).
[^block-on]: tokio, [`Runtime::block_on`](https://docs.rs/tokio/latest/tokio/runtime/struct.Runtime.html#method.block_on).
[^spawn-blocking]: tokio, [`task::spawn_blocking`](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html).
[^tokio-runtime]: tokio, [`runtime` modülü](https://docs.rs/tokio/latest/tokio/runtime/index.html).
[^reqwest]: docs.rs, [`reqwest`](https://docs.rs/reqwest/latest/reqwest/).
[^runtime]: std, [`Future::poll`: Runtime characteristics](https://doc.rust-lang.org/std/future/trait.Future.html#runtime-characteristics); The Rust Programming Language, [17.3 Yielding Control to the Runtime](https://doc.rust-lang.org/book/ch17-03-more-futures.html#yielding-control-to-the-runtime).
[^sleep]: std, [`thread::sleep`](https://doc.rust-lang.org/std/thread/fn.sleep.html).
[^poll-fn]: std, [`poll_fn`](https://doc.rust-lang.org/std/future/fn.poll_fn.html).
[^secim]: The Rust Programming Language, [17.6 Futures, Tasks, and Threads](https://doc.rust-lang.org/book/ch17-06-futures-tasks-threads.html#putting-it-all-together-futures-tasks-and-threads).
[^kodsuz]: rustc 1.99.0 kaynak kodu, [`suggestions.rs`, `err.code = None`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_trait_selection/src/error_reporting/traits/suggestions.rs#L3375).
[^tanik]: rustc 1.99.0 kaynak kodu, [`coroutine/layout.rs`, `mir_coroutine_witnesses`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_mir_transform/src/coroutine/layout.rs#L445); The Rust Programming Language, [16.4 Transferring Ownership Between Threads](https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html#transferring-ownership-between-threads).
[^saklama]: rustc 1.99.0 kaynak kodu, [`storage_liveness.rs`, `visit_local`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_mir_dataflow/src/impls/storage_liveness.rs#L136) ve [`coroutine/layout.rs`, `locals_live_across_suspend_points`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_mir_transform/src/coroutine/layout.rs#L82).
[^tokio-kilit]: tokio öğreticisi, [Shared state](https://tokio.rs/tokio/tutorial/shared-state).
[^gecici-kapsam]: The Rust Reference, [Temporary scopes](https://doc.rust-lang.org/reference/destructors.html#temporary-scopes).
[^tokio-mutex]: tokio, [`sync::Mutex`](https://docs.rs/tokio/latest/tokio/sync/struct.Mutex.html).
[^clippy]: Clippy, [`await_holding_lock`](https://rust-lang.github.io/rust-clippy/stable/index.html#await_holding_lock).
[^e0373]: Hata dizini, [`E0373`](https://doc.rust-lang.org/error_codes/E0373.html); rustc 1.99.0 kaynak kodu, [`borrowck_errors.rs`, `cannot_capture_in_long_lived_closure`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_borrowck/src/borrowck_errors.rs#L456).
