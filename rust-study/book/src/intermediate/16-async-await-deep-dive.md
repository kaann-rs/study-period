# 16.1 Async ve Await: Derinleşme

[Önceki sayfada](16-async-await.md) mekanizmayı kurduk: elle yazılmış bir yürütücü, elle yazılmış future'lar ve `async`/`await`'in neye dönüştüğü. Bu sayfada o yürütücüyü kullanarak async kod yazarken verilen kararlara bakacağız:

- Future'ların tembel olması ve `.await`'i unutmanın neden sessiz bir hata olduğu,
- işleri sıralı ve eşzamanlı beklemenin farkı (ölçerek),
- `?` ile hata yönetimi yapan bir API istemcisi,
- elle yazılmış birleştiriciler: `ikisini_bekle` ve `hepsini_bekle`,
- `pin!` ile `Box::pin` arasındaki seçim.

Sayfanın ana fikri şudur: İş kartı yazmak işi başlatmaz; birbirinden bağımsız işleri art arda değil birlikte beklerseniz toplam bekleme en uzun işin süresine iner.

> [!NOTE]
> Bu sayfanın kodu `learn/async-await/src/deep_dive.rs` dosyasındadır. Depo kökünden `cargo run -p async-await` ile çalıştırabilir, `cargo test -p async-await` ile testlerini koşturabilirsiniz. Görev kuyruğu, `spawn`, gerçek zamanlayıcı, zaman aşımı ve `Send` future'lar [29. bölümde](../advanced/29-advanced-concurrency-async.md); uyandırıcı tablosu ve `Pin`'in iç yüzü [28. bölümde](../advanced/28-future.md) işlenir.

<details>
<summary>Bu sayfada yeni geçen terimler</summary>

- **sıralı bekleme:** Bir iş bitmeden ötekine başlamamak; alt alta iki `.await`.
- **eşzamanlı bekleme (join):** İşleri birlikte başlatıp hepsi bitene dek sırayla yoklamak.
- **birleştirici (combinator):** Birkaç future'ı alıp tek bir future yapan fonksiyon.
- **`poll_fn`:** "Her yoklamada şu closure'ı çalıştır" diyerek kısa yoldan future yazmak.
- **`Waker::noop`:** Çalınca hiçbir şey yapmayan uyandırıcı.
- **tur:** Bu sayfada sürenin yerine sayılan ölçü: "kaç kez yoklandı?"
- **benzetim (simülasyon):** Gerçeğinin yerine konan taklit; burada ağ gecikmesinin taklidi.
- **API istemcisi:** Başka bir servise istek gönderip yanıtını bekleyen kod.
- **`Result` ve `?`:** "Ya sonuç ya hata" değeri ve "hata varsa hemen çağırana ilet" operatörü.
- **`dyn Future`, tür silme:** "Türü ne olursa olsun, beklenebilen bir şey" diyen ortak kılıf.
- **`async move`:** Bloğun kullandığı değerlerin sahipliğini future'ın içine taşımak.
- **iptal (cancellation):** Bir future'ı bitmeden düşürmek; kalan adımları hiç çalışmaz.
- **yarış (select):** Birkaç işten ilk biteni alıp ötekileri bırakmak.
- **zaman aşımı (timeout):** "Şu kadar sürede bitmezse vazgeç" kuralı.
- **bloklamak (blocking):** Thread'i gerçekten durdurmak; o sırada aynı thread'deki başka iş ilerlemez.
- **Promise:** JavaScript'teki benzer kavram. Farkı, elinize geçtiğinde işin çoktan başlamış olmasıdır.

</details>

## Future tembeldir: Çağırmak başlatmaz

JavaScript'te `fetch()` çağrıldığı an istek yola çıkar; dönen Promise yalnızca sonucu temsil eder.[^js] Rust'ta `getir()` çağrısı hiçbir iş yapmaz: Dönen future işin kendisidir ve biri onu yoklayana dek gövdesinin tek satırı bile çalışmaz.[^tembel] Buzdolabındaki listeye "çamaşırı yıka" yazmak çamaşırı yıkamaz.

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
use std::cell::RefCell;

type Gunluk = RefCell<Vec<String>>;

async fn kaydet(gunluk: &Gunluk, mesaj: &str) {
    gunluk.borrow_mut().push(mesaj.to_string());
}

fn main() {
    let gunluk = Gunluk::default();

    let bekleyen = kaydet(&gunluk, "çalıştım"); // future üretildi
    println!("çağrıdan sonra kayıt sayısı = {}", gunluk.borrow().len());

    block_on(bekleyen); // biri yokladı, gövde çalıştı
    println!("block_on'dan sonra kayıt sayısı = {}", gunluk.borrow().len());

    drop(kaydet(&gunluk, "hiç")); // yoklanmadan düştü
    println!("düşürülen future'dan sonra = {}", gunluk.borrow().len());
}
```

Çıktı:

```text
çağrıdan sonra kayıt sayısı = 0
block_on'dan sonra kayıt sayısı = 1
düşürülen future'dan sonra = 1
```

Tasarımın iki kazancı vardır. Birincisi maliyettir: Çağrı gizli bir görev ya da heap tahsisi doğurmaz, future sıradan bir değerdir. İkincisi denetimdir: Sıralı mı, eşzamanlı mı, zaman aşımıyla mı çalışacağına çağıran karar verir. Vazgeçmek isterse future'ı düşürür; *iptal* (cancellation) düşürmekten ibarettir. Yarıda düşürülen bir future kaldığı duraktan sonrasını çalıştırmaz, ama o ana dek yaptığı iş (örneğin gönderilmiş bir istek) geri alınmış olmaz.

### `.await`'i unutmak

Tembelliğin bedeli, `.await` unutulduğunda programın derlenmesidir:

```rust
use std::cell::RefCell;

type Gunluk = RefCell<Vec<String>>;

async fn kaydet(gunluk: &Gunluk, mesaj: &str) {
    gunluk.borrow_mut().push(mesaj.to_string());
}

fn main() {
    let gunluk = Gunluk::default();
    kaydet(&gunluk, "kayboldum"); // hiçbir şey yapmaz
    println!("kayıt sayısı = {}", gunluk.borrow().len());
}
```

Çıktı:

```text
kayıt sayısı = 0
```

Derleyici hata değil, uyarı verir:

```text
warning: unused implementer of `Future` that must be used
   |
10 |     kaydet(&gunluk, "kayboldum");
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: futures do nothing unless you `.await` or poll them
```

Uyarının kaynağı, `Future` trait'inin üzerindeki `#[must_use]` özniteliği ile `unused_must_use` lint'idir.[^must-use] Bu uyarıyı hata gibi okuyun. Sonucu bir değişkene türüyle birlikte koyuyorsanız unutkanlığı derleyici yakalar:

```rust,compile_fail,E0308
async fn sayi_getir() -> u32 {
    7
}

async fn kullan() {
    let n: u32 = sayi_getir();
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0308]: mismatched types
  |
6 |     let n: u32 = sayi_getir();
  |            ---   ^^^^^^^^^^^^ expected `u32`, found future
  |            |
  |            expected due to this
  |
help: consider `await`ing on the `Future`
  |
6 |     let n: u32 = sayi_getir().await;
  |                              ++++++
```

## Sıralı mı, eşzamanlı mı?

Çamaşır 3 saat, fırındaki kek 2 saat sürüyor. Önce çamaşırın bitmesini bekleyip keki sonra koyarsanız 5 saat geçer. Makineyi çalıştırıp hemen ardından keki fırına koyarsanız 3 saat geçer, çünkü beklemeler üst üste biner. Rust'ta alt alta yazılan iki `.await` birinci yoldur.

### Alt alta iki `.await`

İkinci future, birincisi bitmeden yoklanmaz; tembel olduğu için başlamaz bile.[^sirali] Bu çoğu zaman istenen şeydir: İkinci istek birincinin sonucuna bağlıysa başka yol yoktur. Bağımsız işleri böyle beklemek ise async'in kazancını yok eder; toplam süre, sürelerin toplamı olur.

Farkı ölçmek için ağ gecikmesini *tur* cinsinden taklit edelim. `Gecikme`, `kalan` kez `Pending` döner, sonra hazır olur. `tur_sayarak_calistir`, kök future'ı kaç kez yokladığını sayar:

```rust
use std::cell::RefCell;
use std::pin::{Pin, pin};
use std::task::{Context, Poll, Waker};

type Gunluk = RefCell<Vec<String>>;

struct Gecikme {
    kalan: u32,
}

impl Future for Gecikme {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.kalan == 0 {
            Poll::Ready(())
        } else {
            self.kalan -= 1;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

pub fn tur_sayarak_calistir<F: Future>(fut: F) -> (F::Output, u32) {
    let mut fut = pin!(fut);
    let mut cx = Context::from_waker(Waker::noop());
    let mut tur = 0;
    loop {
        tur += 1;
        if let Poll::Ready(deger) = fut.as_mut().poll(&mut cx) {
            return (deger, tur);
        }
    }
}

async fn adim(gunluk: &Gunluk, ad: &str, gecikme: u32) -> u32 {
    gunluk.borrow_mut().push(format!("{ad} başladı"));
    Gecikme { kalan: gecikme }.await;
    gunluk.borrow_mut().push(format!("{ad} bitti"));
    gecikme
}

fn main() {
    let gunluk = Gunluk::default();
    let (toplam, tur) = tur_sayarak_calistir(async {
        let a = adim(&gunluk, "A", 3).await;
        let b = adim(&gunluk, "B", 2).await; // A bitmeden başlamaz
        a + b
    });
    println!("sıralı: {:?}", gunluk.borrow());
    println!("{tur} tur (gecikme toplamı {toplam})");
}
```

Çıktı:

```text
sıralı: ["A başladı", "A bitti", "B başladı", "B bitti"]
6 tur (gecikme toplamı 5)
```

Süre 3 + 2 tur beklemedir; sonucu alan son yoklamayla 6 tur eder.

> [!WARNING]
> `tur_sayarak_calistir` yalnızca ölçüm içindir. `Waker::noop` ile future'ı durmadan yoklar; buradaki `Gecikme` her yoklamada ilerlediği için bu çalışır. Gerçek bir yürütücü sıkı döngüde yoklamaz, uyandırıcının çalmasını bekler.[^sozlesme]

### Birlikte beklemek: `ikisini_bekle`

Eşzamanlı beklemek için future'ları önce üretir, sonra hepsini birlikte yoklayan bir birleştiriciye veririz. Bu işe genellikle *join* denir. Ocaktaki iki tencereye sırayla bakan tek kişiyi düşünün: Her turda önce birine, sonra ötekine bakar. Pişen yemeği kenara koyar ve o tencereye bir daha bakmaz. İkisi de pişince iki yemeği birlikte verir.

```rust
# use std::cell::RefCell;
use std::future::poll_fn;
use std::pin::{Pin, pin};
use std::task::{Context, Poll, Waker};

# type Gunluk = RefCell<Vec<String>>;
# struct Gecikme {
#     kalan: u32,
# }
# impl Future for Gecikme {
#     type Output = ();
#     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
#         if self.kalan == 0 {
#             return Poll::Ready(());
#         }
#         self.kalan -= 1;
#         cx.waker().wake_by_ref();
#         Poll::Pending
#     }
# }
# pub fn tur_sayarak_calistir<F: Future>(fut: F) -> (F::Output, u32) {
#     let mut fut = pin!(fut);
#     let mut cx = Context::from_waker(Waker::noop());
#     let mut tur = 0;
#     loop {
#         tur += 1;
#         if let Poll::Ready(deger) = fut.as_mut().poll(&mut cx) {
#             return (deger, tur);
#         }
#     }
# }
# async fn adim(gunluk: &Gunluk, ad: &str, gecikme: u32) -> u32 {
#     gunluk.borrow_mut().push(format!("{ad} başladı"));
#     Gecikme { kalan: gecikme }.await;
#     gunluk.borrow_mut().push(format!("{ad} bitti"));
#     gecikme
# }
pub async fn ikisini_bekle<A: Future, B: Future>(a: A, b: B) -> (A::Output, B::Output) {
    let mut a = pin!(a); // bu async fn'in durumuna sabitlenir
    let mut b = pin!(b);
    let mut a_sonuc = None;
    let mut b_sonuc = None;

    poll_fn(|cx| {
        if a_sonuc.is_none()
            && let Poll::Ready(v) = a.as_mut().poll(cx)
        {
            a_sonuc = Some(v);
        }
        if b_sonuc.is_none()
            && let Poll::Ready(v) = b.as_mut().poll(cx)
        {
            b_sonuc = Some(v);
        }
        match (a_sonuc.take(), b_sonuc.take()) {
            (Some(x), Some(y)) => Poll::Ready((x, y)),
            (x, y) => {
                a_sonuc = x; // alınanı yerine koy
                b_sonuc = y;
                Poll::Pending
            }
        }
    })
    .await
}

fn main() {
    let gunluk = Gunluk::default();
    let ((a, b), tur) =
        tur_sayarak_calistir(ikisini_bekle(adim(&gunluk, "A", 3), adim(&gunluk, "B", 2)));
    println!("eşzamanlı: {:?}", gunluk.borrow());
    println!("{tur} tur (gecikme toplamı {})", a + b);
}
```

Çıktı:

```text
eşzamanlı: ["A başladı", "B başladı", "B bitti", "A bitti"]
4 tur (gecikme toplamı 5)
```

Aynı iş 6 yerine 4 turda bitti: en uzun bekleme olan 3 tur ve sonucu alan son yoklama. Tek thread vardır; kazanç paralellikten değil, beklemelerin üst üste binmesinden gelir. Async bu yüzden G/Ç için yararlıdır, işlemci yoğun hesap için değil.

Turlarda olanlar şöyledir:

| Tur | A | B | Birleşik future |
| --- | --- | --- | --- |
| 1 | `Pending` | `Pending` | `Pending` |
| 2 | `Pending` | `Pending` | `Pending` |
| 3 | `Pending` | `Ready(2)`, saklanır | `Pending` |
| 4 | `Ready(3)` | yoklanmaz | `Ready((3, 2))` |

Kodda üç ayrıntı vardır:

- **`poll_fn`** bir closure'dan future üretir; closure her yoklamada çağrılır.[^poll-fn] Elle `impl Future` yazmanın kısa yoludur.
- **Biten future bir daha yoklanmaz.** `Ready` dönen bir future'ı yeniden yoklamanın sonucu belirlenmemiştir: Panikleyebilir ya da sonsuza dek `Pending` dönebilir, ama tanımsız davranışa yol açamaz.[^sonra-yoklama] Bu yüzden sonuçlar `Option` içinde saklanır.
- **Uyandırıcı ayrıca saklanmaz.** Alt future'lar aynı `cx`'i gördüğü için hangisi ilerleyebilecek duruma gelirse bu ortak görevi uyandırır.

Gerçek projede bu birleştirici yazılmaz: `futures::join!`, `tokio::join!` ve sayısı çalışırken belli olan görevler için tokio'nun `JoinSet`'i vardır.[^join] Standart kütüphanedeki `std::future::join!` 1.99.0'da kararlı değildir.[^std-join]

## Gerçekçi senaryo: Asenkron API istemcisi

Bir profil sayfası için üç servise istek atacağız: kullanıcı, siparişler, öneriler. Akşam yemeği için fırına, manava ve kasaba telefonla sipariş vermeye benzer. Sıralı yol: Birini arar, paketi gelene kadar bekler, sonra ötekini ararsınız. Eşzamanlı yol: Üçünü de arar, sonra paketleri birlikte beklersiniz.

Ağ yerine, yola göre gecikip yanıt veren sahte bir sunucu kullanıyoruz. Kodun biçimi gerçek bir HTTP istemcisiyle (örneğin reqwest ve tokio ile) yazılanla aynıdır: `async fn`, `Result` ve `?`.

```rust
use std::fmt;
# use std::pin::{Pin, pin};
# use std::task::{Context, Poll, Waker};

# struct Gecikme {
#     kalan: u32,
# }
# impl Future for Gecikme {
#     type Output = ();
#     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
#         if self.kalan == 0 {
#             return Poll::Ready(());
#         }
#         self.kalan -= 1;
#         cx.waker().wake_by_ref();
#         Poll::Pending
#     }
# }
# pub fn tur_sayarak_calistir<F: Future>(fut: F) -> (F::Output, u32) {
#     let mut fut = pin!(fut);
#     let mut cx = Context::from_waker(Waker::noop());
#     let mut tur = 0;
#     loop {
#         tur += 1;
#         if let Poll::Ready(deger) = fut.as_mut().poll(&mut cx) {
#             return (deger, tur);
#         }
#     }
# }
#[derive(Debug, PartialEq, Eq)]
pub enum ApiHatasi {
    Bulunamadi(String),
    SunucuHatasi(u16),
}

impl fmt::Display for ApiHatasi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiHatasi::Bulunamadi(yol) => write!(f, "bulunamadı: {yol}"),
            ApiHatasi::SunucuHatasi(kod) => write!(f, "sunucu hatası: {kod}"),
        }
    }
}

pub async fn getir(yol: &str) -> Result<String, ApiHatasi> {
    let (gecikme, cevap) = match yol {
        "/kullanici/1" => (2, Ok("Ada")),
        "/kullanici/1/siparisler" => (4, Ok("3 sipariş")),
        "/kullanici/1/oneriler" => (3, Ok("klavye, fare")),
        "/bakim" => (1, Err(ApiHatasi::SunucuHatasi(503))),
        _ => (1, Err(ApiHatasi::Bulunamadi(yol.to_string()))),
    };
    Gecikme { kalan: gecikme }.await;
    cevap.map(String::from)
}

pub async fn profil_sirali(id: u32) -> Result<String, ApiHatasi> {
    let ad = getir(&format!("/kullanici/{id}")).await?;
    let siparis = getir(&format!("/kullanici/{id}/siparisler")).await?;
    let oneri = getir(&format!("/kullanici/{id}/oneriler")).await?;
    Ok(format!("{ad} | {siparis} | {oneri}"))
}

fn main() {
    let (sonuc, tur) = tur_sayarak_calistir(profil_sirali(1));
    println!("sıralı istemci: {sonuc:?} ({tur} tur)");

    let (sonuc, tur) = tur_sayarak_calistir(profil_sirali(9));
    println!("sıralı istemci: {sonuc:?} ({tur} tur)");
}
```

Çıktı:

```text
sıralı istemci: Ok("Ada | 3 sipariş | klavye, fare") (10 tur)
sıralı istemci: Err(Bulunamadi("/kullanici/9")) (2 tur)
```

`?` operatörü async fonksiyonda da aynı işi yapar: `Err` gelirse fonksiyon, yani future, o noktada `Err` ile tamamlanır.[^soru] İkinci satırda ilk istek başarısız olduğu için öteki iki istek hiç başlamadı. İlk satırda ise süreler toplandı: 2 + 4 + 3 tur ve son yoklama.

### Eşzamanlı istemci: `hepsini_bekle`

Üç istek birbirinden bağımsız olduğuna göre birlikte beklenebilir. Sayısı önceden belli olmayan future'ları bir `Vec`'te tutmak gerekir, ama her async fonksiyon kendine özgü, adsız bir future türü üretir;[^async-fn] `Vec` ise tek tür tutar. Çözüm türü silmektir: `dyn Future`. Bu tür boyutsuz olduğundan `Box`, yoklanacağı için `Pin` gerekir. `Pin<Box<dyn Future<Output = T> + 'a>>` kalıbı buradan gelir:

```rust
# use std::fmt;
use std::future::poll_fn;
use std::pin::{Pin, pin};
use std::task::{Context, Poll, Waker};

# struct Gecikme {
#     kalan: u32,
# }
# impl Future for Gecikme {
#     type Output = ();
#     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
#         if self.kalan == 0 {
#             return Poll::Ready(());
#         }
#         self.kalan -= 1;
#         cx.waker().wake_by_ref();
#         Poll::Pending
#     }
# }
# pub fn tur_sayarak_calistir<F: Future>(fut: F) -> (F::Output, u32) {
#     let mut fut = pin!(fut);
#     let mut cx = Context::from_waker(Waker::noop());
#     let mut tur = 0;
#     loop {
#         tur += 1;
#         if let Poll::Ready(deger) = fut.as_mut().poll(&mut cx) {
#             return (deger, tur);
#         }
#     }
# }
# #[derive(Debug, PartialEq, Eq)]
# pub enum ApiHatasi {
#     Bulunamadi(String),
#     SunucuHatasi(u16),
# }
# impl fmt::Display for ApiHatasi {
#     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
#         match self {
#             ApiHatasi::Bulunamadi(yol) => write!(f, "bulunamadı: {yol}"),
#             ApiHatasi::SunucuHatasi(kod) => write!(f, "sunucu hatası: {kod}"),
#         }
#     }
# }
# pub async fn getir(yol: &str) -> Result<String, ApiHatasi> {
#     let (gecikme, cevap) = match yol {
#         "/kullanici/1" => (2, Ok("Ada")),
#         "/kullanici/1/siparisler" => (4, Ok("3 sipariş")),
#         "/kullanici/1/oneriler" => (3, Ok("klavye, fare")),
#         "/bakim" => (1, Err(ApiHatasi::SunucuHatasi(503))),
#         _ => (1, Err(ApiHatasi::Bulunamadi(yol.to_string()))),
#     };
#     Gecikme { kalan: gecikme }.await;
#     cevap.map(String::from)
# }
pub type KutuluFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

pub async fn hepsini_bekle<T>(mut gorevler: Vec<KutuluFuture<'_, T>>) -> Vec<T> {
    let mut sonuclar: Vec<Option<T>> = gorevler.iter().map(|_| None).collect();

    poll_fn(|cx| {
        for (gorev, sonuc) in gorevler.iter_mut().zip(sonuclar.iter_mut()) {
            if sonuc.is_none()
                && let Poll::Ready(v) = gorev.as_mut().poll(cx)
            {
                *sonuc = Some(v);
            }
        }
        if sonuclar.iter().all(Option::is_some) {
            Poll::Ready(sonuclar.iter_mut().filter_map(Option::take).collect())
        } else {
            Poll::Pending
        }
    })
    .await
}

pub async fn profil_eszamanli(id: u32) -> Result<String, ApiHatasi> {
    let yollar = [
        format!("/kullanici/{id}"),
        format!("/kullanici/{id}/siparisler"),
        format!("/kullanici/{id}/oneriler"),
    ];
    let gorevler: Vec<KutuluFuture<_>> = yollar
        .iter()
        .map(|yol| Box::pin(getir(yol)) as KutuluFuture<_>)
        .collect();

    let parcalar = hepsini_bekle(gorevler)
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;
    Ok(parcalar.join(" | "))
}

fn main() {
    let (sonuc, tur) = tur_sayarak_calistir(profil_eszamanli(1));
    println!("eşzamanlı istemci: {sonuc:?} ({tur} tur)");

    if let (Err(e), _) = tur_sayarak_calistir(profil_eszamanli(7)) {
        println!("hata: {e}");
    }
    if let (Err(e), _) = tur_sayarak_calistir(getir("/bakim")) {
        println!("bakım: {e}");
    }
}
```

Çıktı:

```text
eşzamanlı istemci: Ok("Ada | 3 sipariş | klavye, fare") (5 tur)
hata: bulunamadı: /kullanici/7
bakım: sunucu hatası: 503
```

Aynı sonuç 10 yerine 5 turda alındı: en uzun istek olan 4 tur ve son yoklama. Sonuçlar bitiş sırasıyla değil görev sırasıyla döner, bu da çağıran için öngörülebilirdir. Hatalar hepsi toplandıktan sonra ele alınır; `collect::<Result<Vec<_>, _>>()` [ilk `Err`'de durur](../basics/08-error-handling.md). Sıralı sürümden farkı, bir istek başarısız olsa da ötekilerin beklenmiş olmasıdır.

### Future, aldığı referansları içinde taşır

`getir` bir `&str` alır. Bu referans `.await` boyunca gerekli olduğu için future'ın içinde saklanır; dönen gizli tür, referansın ömrünü de taşır.[^async-fn] Çağıran, future yaşadıkça metni canlı tutmak zorundadır; bunu derleyici denetler:

```rust,compile_fail,E0597
async fn getir(yol: &str) -> String {
    yol.to_string()
}

fn main() {
    let kart;
    {
        let yol = String::from("/kullanici/1");
        kart = getir(&yol);
    }
    drop(kart);
}
```

Derleyicinin yanıtı:

```text
error[E0597]: `yol` does not live long enough
   |
 8 |         let yol = String::from("/kullanici/1");
   |             --- binding `yol` declared here
 9 |         kart = getir(&yol);
   |                      ^^^^ borrowed value does not live long enough
10 |     }
   |     - `yol` dropped here while still borrowed
11 |     drop(kart);
   |          ---- borrow later used here
```

Future'ı saklayacak ya da başka bir göreve verecekseniz sahipli değer alın: `getir(yol: String)`.

## `Pin`'in gündelik yüzü: `pin!` mi, `Box::pin` mi?

`.await` yazdığınız sürece `Pin`'i görmezsiniz; derleyici alt future'ı sizin yerinize sabitler. `Pin` ile ancak bir future'ı elle yokladığınızda (kendi birleştiriciniz, kendi yürütücünüz) ya da future'ları bir koleksiyonda sakladığınızda karşılaşırsınız. Sabitlenmemiş future yoklanamaz:

```rust,compile_fail,E0599
use std::task::{Context, Waker};

async fn getir(yol: &str) -> String {
    yol.to_string()
}

fn main() {
    let mut cx = Context::from_waker(Waker::noop());
    let mut fut = getir("/kullanici/1");
    let yanit = fut.poll(&mut cx);
}
```

Derleyicinin yanıtı:

```text
error[E0599]: no method named `poll` found for opaque type `impl Future<Output = String>` in the current scope
   |
10 |     let yanit = fut.poll(&mut cx);
   |                     ^^^^ method not found in `impl Future<Output = String>`
   |
help: consider pinning the expression
   |
10 ~     let mut pinned = std::pin::pin!(fut);
11 ~     let yanit = pinned.as_mut().poll(&mut cx);
```

Kendi içini gösteren notlarla dolu defteri yerinden oynatmamanın iki yolu vardır:

- **`pin!(fut)`** defteri masanıza bantlar. Bedavadır, heap tahsisi yapmaz; ama defter o masadan ayrılamaz. Sabitlenen değer bulunduğu kapsamın dışına çıkamaz: döndürülemez, `Vec`'e konamaz.[^pin-makro]
- **`Box::pin(fut)`** defteri bir kasaya koyup kasanın içinde sabitler. Bedeli bir heap tahsisidir; karşılığında kasayı istediğiniz yere taşırsınız, çünkü taşınan kutudur, içindeki future değil. Kutu `dyn Future`'a da dönüştürülebilir.[^box-pin]

```rust
# use std::fmt;
# use std::future::poll_fn;
use std::pin::{Pin, pin};
use std::task::{Context, Poll, Waker};

# struct Gecikme {
#     kalan: u32,
# }
# impl Future for Gecikme {
#     type Output = ();
#     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
#         if self.kalan == 0 {
#             return Poll::Ready(());
#         }
#         self.kalan -= 1;
#         cx.waker().wake_by_ref();
#         Poll::Pending
#     }
# }
# pub fn tur_sayarak_calistir<F: Future>(fut: F) -> (F::Output, u32) {
#     let mut fut = pin!(fut);
#     let mut cx = Context::from_waker(Waker::noop());
#     let mut tur = 0;
#     loop {
#         tur += 1;
#         if let Poll::Ready(deger) = fut.as_mut().poll(&mut cx) {
#             return (deger, tur);
#         }
#     }
# }
# #[derive(Debug, PartialEq, Eq)]
# pub enum ApiHatasi {
#     Bulunamadi(String),
#     SunucuHatasi(u16),
# }
# impl fmt::Display for ApiHatasi {
#     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
#         match self {
#             ApiHatasi::Bulunamadi(yol) => write!(f, "bulunamadı: {yol}"),
#             ApiHatasi::SunucuHatasi(kod) => write!(f, "sunucu hatası: {kod}"),
#         }
#     }
# }
# pub async fn getir(yol: &str) -> Result<String, ApiHatasi> {
#     let (gecikme, cevap) = match yol {
#         "/kullanici/1" => (2, Ok("Ada")),
#         "/kullanici/1/siparisler" => (4, Ok("3 sipariş")),
#         "/kullanici/1/oneriler" => (3, Ok("klavye, fare")),
#         "/bakim" => (1, Err(ApiHatasi::SunucuHatasi(503))),
#         _ => (1, Err(ApiHatasi::Bulunamadi(yol.to_string()))),
#     };
#     Gecikme { kalan: gecikme }.await;
#     cevap.map(String::from)
# }
# pub type KutuluFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;
# pub async fn hepsini_bekle<T>(mut gorevler: Vec<KutuluFuture<'_, T>>) -> Vec<T> {
#     let mut sonuclar: Vec<Option<T>> = gorevler.iter().map(|_| None).collect();
#     poll_fn(|cx| {
#         for (gorev, sonuc) in gorevler.iter_mut().zip(sonuclar.iter_mut()) {
#             if sonuc.is_none()
#                 && let Poll::Ready(v) = gorev.as_mut().poll(cx)
#             {
#                 *sonuc = Some(v);
#             }
#         }
#         if sonuclar.iter().all(Option::is_some) {
#             Poll::Ready(sonuclar.iter_mut().filter_map(Option::take).collect())
#         } else {
#             Poll::Pending
#         }
#     })
#     .await
# }
fn main() {
    let mut cx = Context::from_waker(Waker::noop());

    let mut yigindaki = pin!(getir("/kullanici/1"));
    let ilk_yoklama = yigindaki.as_mut().poll(&mut cx).is_pending();
    println!("pin! ile ilk poll Pending mi? {ilk_yoklama}");

    let kutudaki: KutuluFuture<_> = Box::pin(getir("/kullanici/1"));
    let liste = vec![kutudaki]; // taşınan kutudur
    let (sonuclar, _) = tur_sayarak_calistir(hepsini_bekle(liste));
    println!("Box::pin ile listeden: {sonuclar:?}");
}
```

Çıktı:

```text
pin! ile ilk poll Pending mi? true
Box::pin ile listeden: [Ok("Ada")]
```

İlk yoklama `Pending`'dir, çünkü `/kullanici/1` iki tur gecikir. Kısaca: Kapsamdan çıkmayacaksa `pin!`, taşınacak ya da türü silinecekse `Box::pin`. Sabitlemenin gerekçesi önceki sayfadadır: Durum makinesi kendi yerel değişkenlerine referans tutabilir, taşınırsa bu referanslar sarkar.

## Sık yapılan hatalar

### `.await`'i async olmayan fonksiyonda kullanmak (E0728)

```rust,compile_fail,E0728
async fn sayi_getir() -> u32 {
    7
}

fn main() {
    let x = sayi_getir().await;
}
```

Derleyicinin yanıtı:

```text
error[E0728]: `await` is only allowed inside `async` functions and blocks
```

Sıradan koddan async koda geçiş tek bir noktada olur: yürütücünün `block_on`'u.[^runtime]

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
async fn sayi_getir() -> u32 {
    7
}

fn main() {
    let x = block_on(sayi_getir());
    println!("{x}");
}
```

Çıktı:

```text
7
```

### Yerel değişkeni ödünç alan async bloğu döndürmek (E0373)

```rust,compile_fail,E0373
fn uzunluk() -> impl Future<Output = usize> {
    let s = String::from("a");
    async { s.len() }
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0373]: async block may outlive the current function, but it borrows `s`, which is owned by the current function
  |
3 |     async { s.len() }
  |     ^^^^^   - `s` is borrowed here
  |     |
  |     may outlive borrowed value `s`
  |
help: to force the async block to take ownership of `s` (and any other referenced variables), use the `move` keyword
  |
3 |     async move { s.len() }
  |           ++++
```

Blok tembeldir: Gövdesi fonksiyon döndükten sonra çalışacaktır, o sırada `s` düşmüş olur. Async blok, closure gibi çevresindeki değişkenleri yakalar; derleyici ikisinin yakalamalarını aynı çözümlemede belirler.[^yakalama] `async move`, sahipliği future'ın içine taşır; closure'lardaki `move` ile aynı iştir:[^async-move]

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
fn uzunluk() -> impl Future<Output = usize> {
    let s = String::from("a");
    async move { s.len() }
}

fn main() {
    println!("{}", block_on(uzunluk()));
}
```

Çıktı:

```text
1
```

### Özyineli async fonksiyon (E0733)

```rust,compile_fail,E0733
async fn say(n: u32) -> u32 {
    if n == 0 { 0 } else { 1 + say(n - 1).await }
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0733]: recursion in an async fn requires boxing
  |
1 | async fn say(n: u32) -> u32 {
  | ^^^^^^^^^^^^^^^^^^^^^^^^^^^
2 |     if n == 0 { 0 } else { 1 + say(n - 1).await }
  |                                ---------------- recursive call here
  |
  = note: a recursive `async fn` call must introduce indirection such as `Box::pin` to avoid an infinitely sized future
```

Durum makinesi, beklediği alt future'ı kendi içinde saklar; kendini içeren tür sonsuz boyutlu olur. [Özyineli enum'larla](14-smart-pointers.md) aynı sorundur, çözümü de aynıdır: Özyineli çağrıyı kutulamak.[^e0733]

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
async fn say(n: u32) -> u32 {
    if n == 0 { 0 } else { 1 + Box::pin(say(n - 1)).await }
}

fn main() {
    println!("{}", block_on(say(5)));
}
```

Çıktı:

```text
5
```

### Farklı future'ları kutulamadan listeye koymak (E0308)

```rust,compile_fail,E0308
async fn a() -> u32 {
    1
}
async fn b() -> u32 {
    2
}

fn main() {
    let gorevler = vec![a(), b()];
}
```

Derleyicinin yanıtı:

```text
error[E0308]: mismatched types
  |
9 |     let gorevler = vec![a(), b()];
  |                              ^^^ expected future, found a different future
  |
  = note: distinct uses of `impl Trait` result in different opaque types
```

İki fonksiyonun dönüş türü aynı görünse de ürettikleri future'lar ayrı türlerdir. Kutulayarak ortak türe çevirin. Aşağıda ikinci görev önce biter, ama sonuçlar görev sırasıyla gelir:[^box-pin]

```rust
# use std::future::poll_fn;
use std::pin::{Pin, pin};
use std::task::{Context, Poll, Waker};

# struct Gecikme {
#     kalan: u32,
# }
# impl Future for Gecikme {
#     type Output = ();
#     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
#         if self.kalan == 0 {
#             return Poll::Ready(());
#         }
#         self.kalan -= 1;
#         cx.waker().wake_by_ref();
#         Poll::Pending
#     }
# }
# pub fn tur_sayarak_calistir<F: Future>(fut: F) -> (F::Output, u32) {
#     let mut fut = pin!(fut);
#     let mut cx = Context::from_waker(Waker::noop());
#     let mut tur = 0;
#     loop {
#         tur += 1;
#         if let Poll::Ready(deger) = fut.as_mut().poll(&mut cx) {
#             return (deger, tur);
#         }
#     }
# }
# pub type KutuluFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;
# pub async fn hepsini_bekle<T>(mut gorevler: Vec<KutuluFuture<'_, T>>) -> Vec<T> {
#     let mut sonuclar: Vec<Option<T>> = gorevler.iter().map(|_| None).collect();
#     poll_fn(|cx| {
#         for (gorev, sonuc) in gorevler.iter_mut().zip(sonuclar.iter_mut()) {
#             if sonuc.is_none()
#                 && let Poll::Ready(v) = gorev.as_mut().poll(cx)
#             {
#                 *sonuc = Some(v);
#             }
#         }
#         if sonuclar.iter().all(Option::is_some) {
#             Poll::Ready(sonuclar.iter_mut().filter_map(Option::take).collect())
#         } else {
#             Poll::Pending
#         }
#     })
#     .await
# }
fn main() {
    let gorevler: Vec<KutuluFuture<u32>> = vec![
        Box::pin(async {
            Gecikme { kalan: 5 }.await;
            1
        }),
        Box::pin(async { 2 }),
    ];
    let (sonuclar, _) = tur_sayarak_calistir(hepsini_bekle(gorevler));
    println!("{sonuclar:?}");
}
```

Çıktı:

```text
[1, 2]
```

### Async kodda thread'i bloklamak

Bu hatayı derleyici yakalamaz. `thread::sleep`, `Pending` dönmez; thread'i gerçekten uyutur. Evdeki tek kişinin koltukta uyuyakalmasına benzer: Zil çalsa da kalkan olmaz, bütün işler durur.

```rust
# use std::cell::RefCell;
# use std::future::poll_fn;
# use std::pin::{Pin, pin};
# use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::Duration;

# type Gunluk = RefCell<Vec<String>>;
# struct Gecikme {
#     kalan: u32,
# }
# impl Future for Gecikme {
#     type Output = ();
#     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
#         if self.kalan == 0 {
#             return Poll::Ready(());
#         }
#         self.kalan -= 1;
#         cx.waker().wake_by_ref();
#         Poll::Pending
#     }
# }
# pub fn tur_sayarak_calistir<F: Future>(fut: F) -> (F::Output, u32) {
#     let mut fut = pin!(fut);
#     let mut cx = Context::from_waker(Waker::noop());
#     let mut tur = 0;
#     loop {
#         tur += 1;
#         if let Poll::Ready(deger) = fut.as_mut().poll(&mut cx) {
#             return (deger, tur);
#         }
#     }
# }
# pub async fn ikisini_bekle<A: Future, B: Future>(a: A, b: B) -> (A::Output, B::Output) {
#     let mut a = pin!(a);
#     let mut b = pin!(b);
#     let (mut a_sonuc, mut b_sonuc) = (None, None);
#     poll_fn(|cx| {
#         if a_sonuc.is_none()
#             && let Poll::Ready(v) = a.as_mut().poll(cx)
#         {
#             a_sonuc = Some(v);
#         }
#         if b_sonuc.is_none()
#             && let Poll::Ready(v) = b.as_mut().poll(cx)
#         {
#             b_sonuc = Some(v);
#         }
#         match (a_sonuc.take(), b_sonuc.take()) {
#             (Some(x), Some(y)) => Poll::Ready((x, y)),
#             (x, y) => {
#                 a_sonuc = x;
#                 b_sonuc = y;
#                 Poll::Pending
#             }
#         }
#     })
#     .await
# }
async fn kotu(gunluk: &Gunluk, ad: &str) {
    gunluk.borrow_mut().push(format!("{ad} başladı"));
    thread::sleep(Duration::from_millis(20)); // sırayı bırakmaz
    gunluk.borrow_mut().push(format!("{ad} bitti"));
}

async fn iyi(gunluk: &Gunluk, ad: &str) {
    gunluk.borrow_mut().push(format!("{ad} başladı"));
    Gecikme { kalan: 3 }.await; // beklerken sırayı bırakır
    gunluk.borrow_mut().push(format!("{ad} bitti"));
}

fn main() {
    let gunluk = Gunluk::default();
    tur_sayarak_calistir(ikisini_bekle(kotu(&gunluk, "A"), kotu(&gunluk, "B")));
    println!("kötü: {:?}", gunluk.borrow());

    let gunluk = Gunluk::default();
    tur_sayarak_calistir(ikisini_bekle(iyi(&gunluk, "A"), iyi(&gunluk, "B")));
    println!("iyi : {:?}", gunluk.borrow());
}
```

Çıktı:

```text
kötü: ["A başladı", "A bitti", "B başladı", "B bitti"]
iyi : ["A başladı", "B başladı", "A bitti", "B bitti"]
```

`ikisini_bekle` kullanıldığı hâlde kötü sürümde işler sırayla yapıldı. Kural şudur: İki `.await` arasında uzun süren hiçbir şey olmamalıdır.[^bloklama] Beklemek için çalışma zamanının async zamanlayıcısını, ağır hesap için ayrı bir thread'i kullanın; tokio'da bunun için `spawn_blocking` vardır.[^spawn-blocking] Aynı nedenle async kodun içinden `block_on` çağırmayın; o da thread'i bloklar. Çalışan örnekler [29. bölümdedir](../advanced/29-advanced-concurrency-async.md).

## Hata kodu başvurusu

| Kod | Ne zaman görülür? | Ne yapmalı? |
| --- | --- | --- |
| `E0308` | Değer beklenen yere future konur; farklı future'lar kutulanmadan aynı listeye girer. | `.await` ekleyin; listede `Pin<Box<dyn Future>>` kullanın. |
| `E0373` | Async blok, fonksiyondan önce düşecek bir yereli ödünç alır. | `async move` yazın. |
| `E0597` | Future, kendisinden önce düşen bir değere referans tutar. | Değeri daha uzun yaşatın ya da sahipli değer verin. |
| `E0599` | Sabitlenmemiş future'da `poll` çağrılır. | `pin!` ya da `Box::pin` kullanın. |
| `E0728` | `.await`, async olmayan yerde kullanılır. | Fonksiyonu `async` yapın ya da `block_on` ile sürün. |
| `E0733` | Async fonksiyon kendini kutulamadan çağırır. | Özyineli çağrıyı `Box::pin` ile sarın. |

## Akılda kalması gerekenler

- Future henüz yapılmamış iştir. Üretmek tahsis gerektirmez, yoklanmazsa hiç çalışmaz, düşürülürse iptal olur.
- `.await` sıralıdır. Bağımsız işleri eşzamanlı yürütmek için join türü bir birleştirici gerekir.
- Eşzamanlılık paralellik değildir: Bu sayfadaki kazançların hepsi tek thread'de elde edildi.
- Async fonksiyona verilen referanslar future'ın içinde yaşar. Future'ı saklayacak ya da başka göreve verecekseniz sahipli değer ve `async move` kullanın.
- `Pin` yalnızca elle yoklarken ve future saklarken görünür: Kapsam içiyse `pin!`, taşınacak ya da türü silinecekse `Box::pin`.
- İki `.await` arasında thread'i bloklayan çağrı yapmayın.

## Alıştırmalar

### 1. İlk biten

Şu imzayla, hangi future önce biterse onun sonucunu döndüren bir fonksiyon yazın (*yarış*, select):

```text
async fn ilk_biten<A, B, T>(a: A, b: B) -> T
where
    A: Future<Output = T>,
    B: Future<Output = T>,
```

`ilk_biten(adim(&gunluk, "A", 3), adim(&gunluk, "B", 1))` için sonuç 1 olmalı, günlükte "A bitti" bulunmamalı. Öteki future'a ne olur?

<details>
<summary>İpucu ve çözüm</summary>

`ikisini_bekle`'nin iskeletini alın; ilk `Ready`'de hemen `Ready` dönün.

```rust
# use std::cell::RefCell;
use std::future::poll_fn;
use std::pin::{Pin, pin};
use std::task::{Context, Poll, Waker};

# type Gunluk = RefCell<Vec<String>>;
# struct Gecikme {
#     kalan: u32,
# }
# impl Future for Gecikme {
#     type Output = ();
#     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
#         if self.kalan == 0 {
#             return Poll::Ready(());
#         }
#         self.kalan -= 1;
#         cx.waker().wake_by_ref();
#         Poll::Pending
#     }
# }
# pub fn tur_sayarak_calistir<F: Future>(fut: F) -> (F::Output, u32) {
#     let mut fut = pin!(fut);
#     let mut cx = Context::from_waker(Waker::noop());
#     let mut tur = 0;
#     loop {
#         tur += 1;
#         if let Poll::Ready(deger) = fut.as_mut().poll(&mut cx) {
#             return (deger, tur);
#         }
#     }
# }
# async fn adim(gunluk: &Gunluk, ad: &str, gecikme: u32) -> u32 {
#     gunluk.borrow_mut().push(format!("{ad} başladı"));
#     Gecikme { kalan: gecikme }.await;
#     gunluk.borrow_mut().push(format!("{ad} bitti"));
#     gecikme
# }
async fn ilk_biten<A, B, T>(a: A, b: B) -> T
where
    A: Future<Output = T>,
    B: Future<Output = T>,
{
    let mut a = pin!(a);
    let mut b = pin!(b);

    poll_fn(|cx| {
        if let Poll::Ready(v) = a.as_mut().poll(cx) {
            return Poll::Ready(v);
        }
        b.as_mut().poll(cx)
    })
    .await
}

fn main() {
    let gunluk = Gunluk::default();
    let (sonuc, _) =
        tur_sayarak_calistir(ilk_biten(adim(&gunluk, "A", 3), adim(&gunluk, "B", 1)));
    println!("sonuç = {sonuc}");
    println!("{:?}", gunluk.borrow());
}
```

Çıktı:

```text
sonuç = 1
["A başladı", "B başladı", "B bitti"]
```

Kaybeden future, `ilk_biten` bitince düşer, yani iptal edilir; bunun için ayrı bir API gerekmez.

</details>

### 2. Zaman aşımı

Alıştırma 1'i kullanarak `async fn zaman_asimli<F: Future>(f: F, tur: u32) -> Option<F::Output>` yazın. `zaman_asimli(getir("/kullanici/1/siparisler"), 2)` için `None` beklenir, çünkü istek 4 tur sürer.

<details>
<summary>İpucu ve çözüm</summary>

`f`'nin sonucunu `Some`'a, `Gecikme { kalan: tur }`'un bitişini `None`'a eşleyin; `async { Some(f.await) }` bunun için yeter.

```rust
# use std::fmt;
use std::future::poll_fn;
use std::pin::{Pin, pin};
use std::task::{Context, Poll, Waker};

# struct Gecikme {
#     kalan: u32,
# }
# impl Future for Gecikme {
#     type Output = ();
#     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
#         if self.kalan == 0 {
#             return Poll::Ready(());
#         }
#         self.kalan -= 1;
#         cx.waker().wake_by_ref();
#         Poll::Pending
#     }
# }
# pub fn tur_sayarak_calistir<F: Future>(fut: F) -> (F::Output, u32) {
#     let mut fut = pin!(fut);
#     let mut cx = Context::from_waker(Waker::noop());
#     let mut tur = 0;
#     loop {
#         tur += 1;
#         if let Poll::Ready(deger) = fut.as_mut().poll(&mut cx) {
#             return (deger, tur);
#         }
#     }
# }
# #[derive(Debug, PartialEq, Eq)]
# pub enum ApiHatasi {
#     Bulunamadi(String),
#     SunucuHatasi(u16),
# }
# impl fmt::Display for ApiHatasi {
#     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
#         match self {
#             ApiHatasi::Bulunamadi(yol) => write!(f, "bulunamadı: {yol}"),
#             ApiHatasi::SunucuHatasi(kod) => write!(f, "sunucu hatası: {kod}"),
#         }
#     }
# }
# pub async fn getir(yol: &str) -> Result<String, ApiHatasi> {
#     let (gecikme, cevap) = match yol {
#         "/kullanici/1" => (2, Ok("Ada")),
#         "/kullanici/1/siparisler" => (4, Ok("3 sipariş")),
#         "/kullanici/1/oneriler" => (3, Ok("klavye, fare")),
#         "/bakim" => (1, Err(ApiHatasi::SunucuHatasi(503))),
#         _ => (1, Err(ApiHatasi::Bulunamadi(yol.to_string()))),
#     };
#     Gecikme { kalan: gecikme }.await;
#     cevap.map(String::from)
# }
# async fn ilk_biten<A, B, T>(a: A, b: B) -> T
# where
#     A: Future<Output = T>,
#     B: Future<Output = T>,
# {
#     let mut a = pin!(a);
#     let mut b = pin!(b);
#     poll_fn(|cx| {
#         if let Poll::Ready(v) = a.as_mut().poll(cx) {
#             return Poll::Ready(v);
#         }
#         b.as_mut().poll(cx)
#     })
#     .await
# }
async fn zaman_asimli<F: Future>(f: F, tur: u32) -> Option<F::Output> {
    let is = async { Some(f.await) };
    let sure = async {
        Gecikme { kalan: tur }.await;
        None
    };
    ilk_biten(is, sure).await
}

fn main() {
    let (sonuc, _) = tur_sayarak_calistir(zaman_asimli(getir("/kullanici/1/siparisler"), 2));
    println!("2 tur: {sonuc:?}");

    let (sonuc, _) = tur_sayarak_calistir(zaman_asimli(getir("/kullanici/1/siparisler"), 10));
    println!("10 tur: {sonuc:?}");
}
```

Çıktı:

```text
2 tur: None
10 tur: Some(Ok("3 sipariş"))
```

Resmî kitap da zaman aşımını aynı düşünceyle, bir işi bir uyku future'ıyla yarıştırarak kurar.[^zaman-asimi]

</details>

### 3. Kısmi başarı

`profil_eszamanli`'yi, bir istek başarısız olsa da ötekilerin sonucunu gösterecek biçimde değiştirin: `Vec<Result<String, ApiHatasi>>` döndürsün, çağıran bunu başarılı ve başarısız diye ikiye ayırsın.

<details>
<summary>İpucu ve çözüm</summary>

`Iterator::partition` ya da iki ayrı `filter_map` kullanılabilir. Karışık bir sonuç görmek için listeye bakımdaki servisi (`/bakim`) de ekledik.

```rust
# use std::fmt;
use std::future::poll_fn;
use std::pin::{Pin, pin};
use std::task::{Context, Poll, Waker};

# struct Gecikme {
#     kalan: u32,
# }
# impl Future for Gecikme {
#     type Output = ();
#     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
#         if self.kalan == 0 {
#             return Poll::Ready(());
#         }
#         self.kalan -= 1;
#         cx.waker().wake_by_ref();
#         Poll::Pending
#     }
# }
# pub fn tur_sayarak_calistir<F: Future>(fut: F) -> (F::Output, u32) {
#     let mut fut = pin!(fut);
#     let mut cx = Context::from_waker(Waker::noop());
#     let mut tur = 0;
#     loop {
#         tur += 1;
#         if let Poll::Ready(deger) = fut.as_mut().poll(&mut cx) {
#             return (deger, tur);
#         }
#     }
# }
# #[derive(Debug, PartialEq, Eq)]
# pub enum ApiHatasi {
#     Bulunamadi(String),
#     SunucuHatasi(u16),
# }
# impl fmt::Display for ApiHatasi {
#     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
#         match self {
#             ApiHatasi::Bulunamadi(yol) => write!(f, "bulunamadı: {yol}"),
#             ApiHatasi::SunucuHatasi(kod) => write!(f, "sunucu hatası: {kod}"),
#         }
#     }
# }
# pub async fn getir(yol: &str) -> Result<String, ApiHatasi> {
#     let (gecikme, cevap) = match yol {
#         "/kullanici/1" => (2, Ok("Ada")),
#         "/kullanici/1/siparisler" => (4, Ok("3 sipariş")),
#         "/kullanici/1/oneriler" => (3, Ok("klavye, fare")),
#         "/bakim" => (1, Err(ApiHatasi::SunucuHatasi(503))),
#         _ => (1, Err(ApiHatasi::Bulunamadi(yol.to_string()))),
#     };
#     Gecikme { kalan: gecikme }.await;
#     cevap.map(String::from)
# }
# pub type KutuluFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;
# pub async fn hepsini_bekle<T>(mut gorevler: Vec<KutuluFuture<'_, T>>) -> Vec<T> {
#     let mut sonuclar: Vec<Option<T>> = gorevler.iter().map(|_| None).collect();
#     poll_fn(|cx| {
#         for (gorev, sonuc) in gorevler.iter_mut().zip(sonuclar.iter_mut()) {
#             if sonuc.is_none()
#                 && let Poll::Ready(v) = gorev.as_mut().poll(cx)
#             {
#                 *sonuc = Some(v);
#             }
#         }
#         if sonuclar.iter().all(Option::is_some) {
#             Poll::Ready(sonuclar.iter_mut().filter_map(Option::take).collect())
#         } else {
#             Poll::Pending
#         }
#     })
#     .await
# }
async fn profil_eszamanli(id: u32) -> Vec<Result<String, ApiHatasi>> {
    let yollar = [
        format!("/kullanici/{id}"),
        format!("/kullanici/{id}/siparisler"),
        format!("/kullanici/{id}/oneriler"),
        String::from("/bakim"),
    ];
    let gorevler: Vec<KutuluFuture<_>> = yollar
        .iter()
        .map(|yol| Box::pin(getir(yol)) as KutuluFuture<_>)
        .collect();
    hepsini_bekle(gorevler).await
}

fn main() {
    let (sonuclar, _) = tur_sayarak_calistir(profil_eszamanli(1));
    let (basarili, basarisiz): (Vec<_>, Vec<_>) =
        sonuclar.into_iter().partition(|s| s.is_ok());

    let basarili: Vec<String> = basarili.into_iter().filter_map(Result::ok).collect();
    let basarisiz: Vec<String> =
        basarisiz.into_iter().filter_map(Result::err).map(|e| e.to_string()).collect();

    println!("başarılı: {basarili:?}");
    println!("başarısız: {basarisiz:?}");
}
```

Çıktı:

```text
başarılı: ["Ada", "3 sipariş", "klavye, fare"]
başarısız: ["sunucu hatası: 503"]
```

</details>

## Özet

- Future tembeldir; `.await` unutulursa program derlenir ama o satır iş yapmaz. `unused_must_use` uyarısını hata sayın.
- Alt alta iki `.await` süreleri toplar; join türü bir birleştirici toplam beklemeyi en uzun işe indirir.
- Bir birleştirici, future'ları aynı `poll` içinde sırayla yoklayan sıradan bir future'dır; yarış ve zaman aşımı da böyle yazılır.
- Async fonksiyon aldığı referansları future'ın içinde taşır; future'lar farklı türler olduğundan listede kutulanır.
- `pin!` yerinde, `Box::pin` heap'te sabitler. Async kodda thread'i bloklamayın.

Burada tek bir kök future'ı elle sürdük. Gerçek bir çalışma zamanı görevleri kuyrukta tutar, yalnızca uyandırılanı yoklar ve zamanlayıcı ile G/Ç'ye bağlanır: [29. İleri Eşzamanlılık ve Async](../advanced/29-advanced-concurrency-async.md). Uyandırıcının ve `Pin`'in iç yapısı için: [28. `Future`'ın İç Yüzü](../advanced/28-future.md).

Sıradaki bölüm: [17. Nesne Yönelimli Özellikler](17-oop.md).

## Kaynaklar

[^js]: MDN Web Docs, [`fetch()`](https://developer.mozilla.org/en-US/docs/Web/API/Window/fetch).
[^tembel]: The Rust Programming Language, [17.1 Futures and the Async Syntax](https://doc.rust-lang.org/book/ch17-01-futures-and-syntax.html#futures-and-the-async-syntax); standart kütüphane, [`Future`: Runtime characteristics](https://doc.rust-lang.org/std/future/trait.Future.html#runtime-characteristics).
[^must-use]: Standart kütüphane 1.99.0 kaynak kodu, [`future.rs`, `must_use`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/future/future.rs#L31); rustc 1.99.0 kaynak kodu, [`must_use.rs`, `UNUSED_MUST_USE`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_lint/src/unused/must_use.rs#L44).
[^sirali]: The Rust Programming Language, [17.2 Code Within One Async Block Executes Linearly](https://doc.rust-lang.org/book/ch17-02-concurrency-with-async.html#code-within-one-async-block-executes-linearly).
[^sozlesme]: Standart kütüphane, [`Future`: Runtime characteristics](https://doc.rust-lang.org/std/future/trait.Future.html#runtime-characteristics) ve [`Waker::noop`](https://doc.rust-lang.org/std/task/struct.Waker.html#method.noop).
[^poll-fn]: Standart kütüphane, [`poll_fn`](https://doc.rust-lang.org/std/future/fn.poll_fn.html).
[^sonra-yoklama]: Standart kütüphane, [`Future`: Panics](https://doc.rust-lang.org/std/future/trait.Future.html#panics).
[^join]: futures belgesi, [`join!`](https://docs.rs/futures/latest/futures/macro.join.html); tokio belgesi, [`join!`](https://docs.rs/tokio/latest/tokio/macro.join.html) ve [`JoinSet`](https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html).
[^std-join]: Standart kütüphane, [`std::future::join!`](https://doc.rust-lang.org/std/future/macro.join.html).
[^soru]: The Rust Reference, [`async` blocks: Control-flow operators](https://doc.rust-lang.org/reference/expressions/block-expr.html#control-flow-operators) ve [Async functions](https://doc.rust-lang.org/reference/items/functions.html#async-functions).
[^async-fn]: The Rust Reference, [Async functions](https://doc.rust-lang.org/reference/items/functions.html#async-functions).
[^pin-makro]: Standart kütüphane, [`pin!`](https://doc.rust-lang.org/std/pin/macro.pin.html).
[^box-pin]: Standart kütüphane, [`Box::pin`](https://doc.rust-lang.org/std/boxed/struct.Box.html#method.pin); The Rust Programming Language, [17.5 The `Pin` Type and the `Unpin` Trait](https://doc.rust-lang.org/book/ch17-05-traits-for-async.html#the-pin-type-and-the-unpin-trait).
[^runtime]: The Rust Programming Language, [17.1 Executing an Async Function with a Runtime](https://doc.rust-lang.org/book/ch17-01-futures-and-syntax.html#executing-an-async-function-with-a-runtime).
[^async-move]: The Rust Programming Language, [17.2 Moving Ownership Into an Async Block](https://doc.rust-lang.org/book/ch17-02-concurrency-with-async.html#moving-ownership-into-an-async-block).
[^yakalama]: rustc 1.99.0 kaynak kodu, [`upvar.rs`, `closure_analyze`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_typeck/src/upvar.rs#L78); The Rust Reference, [`async` blocks: Capture modes](https://doc.rust-lang.org/reference/expressions/block-expr.html#capture-modes).
[^e0733]: Hata dizini, [E0733](https://doc.rust-lang.org/error_codes/E0733.html).
[^bloklama]: The Rust Programming Language, [17.3 Yielding Control to the Runtime](https://doc.rust-lang.org/book/ch17-03-more-futures.html#yielding-control-to-the-runtime); standart kütüphane, [`Future`: Runtime characteristics](https://doc.rust-lang.org/std/future/trait.Future.html#runtime-characteristics).
[^spawn-blocking]: tokio belgesi, [`spawn_blocking`](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html).
[^zaman-asimi]: The Rust Programming Language, [17.3 Building Our Own Async Abstractions](https://doc.rust-lang.org/book/ch17-03-more-futures.html#building-our-own-async-abstractions).
