# 29.1 Thread'ler ve Zamanlayıcı

[15. bölüm](../orta/15-concurrency.md) `Mutex`, kanal ve `thread::scope` ile veri yarışı içermeyen kod yazmayı gösterdi. Bu sayfa bir adım ileri gidiyor ve "Beklerken ne yapalım?" sorusunun thread tarafını ele alıyor. Sırasıyla şu sorulara bakacağız:

- Bekleyen bir thread'e ne olur?
- Veri çok okunup seyrek yazılıyorsa hangi kilit seçilmeli?
- Bir koşul gerçekleşene dek işlemciyi yormadan nasıl beklenir?
- Derleyicinin yakalayamadığı kilitlenme tasarımla nasıl önlenir?
- Tek bir sayı ya da bayrak için kilitsiz ne yapılabilir?
- Hangi tür başka bir thread'e verilebilir, hangisi thread'ler arasında paylaşılabilir?

> [!NOTE]
> Bu sayfanın kodu `learn/advanced-concepts/ileri-eszamanlilik-async/src/threadler.rs` dosyasındadır. Testlerini `cargo test -p ileri-eszamanlilik-async` ile çalıştırabilirsiniz. Başlıktaki zamanlayıcı, işletim sisteminin thread'leri sırayla çalıştıran parçasıdır (scheduler). Süre dolunca haber veren zamanlayıcıyı (timer) [bir sonraki sayfada](29-calisma-zamani.md) kendimiz yazacağız.

<details>
<summary>Bu sayfada geçen yeni terimler</summary>

- **`RwLock` (okuma-yazma kilidi):** Aynı anda ya çok sayıda okuyucuya ya da tek bir yazara izin veren kilit.
- **`read`, `write`, `try_read`, `try_write`:** İlk ikisi okuma ya da yazma kilidini ister ve gerekirse bekler. `try_` ile başlayanlar beklemez, kilit alınamıyorsa hemen hata döndürür.
- **koruyucu (guard):** Kilidi aldığınızda elinize geçen değer. Veriye onun üzerinden ulaşırsınız; düşürülünce kilit açılır.
- **kilit yükseltme (lock upgrade):** Okuma kilidini bırakmadan yazma kilidine çevirmek. `std`'nin `RwLock` türünde yoktur.
- **`Condvar` (koşul değişkeni):** "Şu durum gerçekleşince beni uyandır" demeye yarayan zil. `wait` uyumak, `notify_one` ile `notify_all` zili çalmaktır.
- **kayıp uyandırma (lost wakeup):** Zilin, bekleyen daha uyumadan çalıp boşa gitmesi.
- **sebepsiz uyanma (spurious wakeup):** Zil çalmadığı hâlde uyanmak.
- **meşgul bekleme (busy-wait):** Uyumak yerine durmadan "Oldu mu?" diye bakmak.
- **sınırlı kuyruk (bounded queue):** En çok belirli sayıda öğe alan kuyruk.
- **üretici, tüketici (producer, consumer):** Kuyruğa öğe koyan ve kuyruktan öğe alan taraf.
- **geri basınç (backpressure):** Kuyruk dolunca üreticinin de beklemesi.
- **kilit sıralaması (lock ordering):** Bütün kodun kilitleri aynı sırayla alması kuralı.
- **`compare_exchange`:** "Değer hâlâ beklediğimse değiştir, değilse dokunma" hamlesi.
- **`Ordering` (bellek sıralaması):** Atomik bir işlemin, çevresindeki öteki bellek erişimleriyle birlikte başka thread'lere hangi sırayla görüneceğini seçen ayar.
- **`thread::scope` (kapsamlı thread):** İçinde açılan bütün thread'lerin, kendisi dönmeden bitmesini güvence altına alan çağrı.

</details>

## Bekleyen thread'e ne olur?

Hangi thread'in ne zaman çalışacağına işletim sisteminin zamanlayıcısı (scheduler) karar verir; programın thread'ler arasında belirli bir sıraya güvenmesi doğru olmaz.[^sira] Bir thread kilit, kanal, `join` ya da uyku beklerken *bloklanır*: Çekirdeği bırakır, yerine başka bir thread çalışır. İşlemcinin bir thread'i bırakıp ötekine geçmesine *bağlam değişimi* (context switch) denir.

Beklemenin kötü yolu, çekirdeği bırakmadan sürekli "Oldu mu?" diye bakmaktır. Aşağıdaki program bir bayrağı 20 ms boyunca böyle bekler:

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    let hazir = AtomicBool::new(false);
    let baslangic = Instant::now();

    thread::scope(|k| {
        k.spawn(|| {
            thread::sleep(Duration::from_millis(20));
            hazir.store(true, Ordering::Relaxed);
        });

        let mut bakis: u64 = 0;
        while !hazir.load(Ordering::Relaxed) {
            bakis += 1; // meşgul bekleme
        }
        println!("bayrağa {bakis} kez bakıldı");
    });

    println!("20 ms doldu mu? {}", baslangic.elapsed() >= Duration::from_millis(20));
}
```

Örnek bir çalıştırma (ilk satırdaki sayı her seferinde değişir):

```text
bayrağa 13482166 kez bakıldı
20 ms doldu mu? true
```

Milyonlarca bakışın hepsi boşa gitti ve bir çekirdek 20 ms boyunca başka hiçbir işe yaramadı. `thread::sleep` ise öyle değildir: Thread'i *en az* istenen süre kadar uyutur ve bu sürede çekirdeği tutmaz. Süre, zamanlamanın ayrıntıları yüzünden biraz uzayabilir; ama kısalmaz.[^sleep]

Döngünün içine `thread::yield_now()` koymak da tam çözüm değildir. Bu çağrı sırayı gönüllü olarak bırakır, ama çalışmaya hazır başka bir thread yoksa döngü yine boşa döner. Standart kütüphane belgesi bu yüzden ilk tercihin kanal, `Condvar`, `Mutex` ya da `join` olması gerektiğini söyler: Hepsi olay gerçekleşene dek çekirdeği bırakır.[^yield]

## `RwLock`: çok okuyucu ya da tek yazar

`Mutex` okuyanla yazanı ayırmaz: On thread yalnızca okumak istese bile sıraya girer. Oysa paylaşılan verinin çoğu (yapılandırma, yönlendirme tablosu, önbellek) çok okunur, seyrek yazılır. `RwLock<T>` iki isteği ayırır: Aynı anda ya çok sayıda okuyucu ya da tek bir yazar içeridedir. Ödünç alma kuralının, yani "çok sayıda `&T` ya da tek bir `&mut T`" kuralının, çalışma zamanında ve thread'ler arasında uygulanan hâlidir.[^rwlock]

Kütüphanenin okuma salonunu düşünün. Salona aynı anda istediği kadar kişi girip okuyabilir; kimse kimseyi rahatsız etmez. Rafları yeniden dizecek görevli ise salon boşalınca tek başına girer ve o içerideyken okuyucu alınmaz. `read()` okuyucu girişi, `write()` görevli girişidir:

```rust
use std::sync::RwLock;

fn main() {
    let kilit = RwLock::new(5);

    let okuyucu1 = kilit.read().expect("kilit zehirlenmemeli");
    let okuyucu2 = kilit.read().expect("kilit zehirlenmemeli");
    println!("iki okuyucu içeride: {} ve {}", *okuyucu1, *okuyucu2);
    println!("yazar girebilir mi? {}", kilit.try_write().is_ok());

    drop(okuyucu1);
    drop(okuyucu2); // okuyucular çıktı
    *kilit.write().expect("kilit zehirlenmemeli") += 1;
    println!("yazar girip çıktı, değer: {:?}", kilit.into_inner());
}
```

Çıktı:

```text
iki okuyucu içeride: 5 ve 5
yazar girebilir mi? false
yazar girip çıktı, değer: Ok(6)
```

| Çağrı | Verdiği koruyucu | Neye benzer? |
| --- | --- | --- |
| `read()` | `RwLockReadGuard` | `&T`: Aynı anda çok sayıda olabilir. |
| `write()` | `RwLockWriteGuard` | `&mut T`: Tek başına olur. |
| `try_read()`, `try_write()` | Aynı koruyucular | Beklemez; kilit alınamıyorsa `Err` döndürür. |

`expect` çağrılarının nedeni *zehirlenme* (poisoning) olasılığıdır: Yazma kilidini tutan thread panikle sonlanırsa veri yarım kalmış olabilir, bu yüzden sonraki `read` ve `write` çağrıları `Err` döndürür. `RwLock` yalnızca yazma kilidi altındaki panikle zehirlenir; okuma kilidi altındaki panik onu zehirlemez.[^zehir]

### Örnek: ayar deposu

Çok okunan, seyrek yazılan verinin tipik örneği bir programın ayarlarıdır:

```rust
use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct AyarDeposu {
    ayarlar: RwLock<HashMap<String, String>>,
    surum: AtomicU64, // kaç kez yazıldı
}

impl AyarDeposu {
    pub fn yeni() -> Self {
        AyarDeposu {
            ayarlar: RwLock::new(HashMap::new()),
            surum: AtomicU64::new(0),
        }
    }

    pub fn oku(&self, anahtar: &str) -> Option<String> {
        self.ayarlar.read().expect("kilit zehirlenmemeli").get(anahtar).cloned()
    }

    pub fn yaz(&self, anahtar: &str, deger: &str) {
        let mut ayarlar = self.ayarlar.write().expect("kilit zehirlenmemeli");
        ayarlar.insert(anahtar.to_string(), deger.to_string());
        self.surum.fetch_add(1, Ordering::Relaxed);
    }

    pub fn surum(&self) -> u64 {
        self.surum.load(Ordering::Relaxed)
    }
}

fn main() {
    let depo = AyarDeposu::yeni();
    depo.yaz("tema", "açık");

    let okuyucu1 = depo.ayarlar.read().expect("kilit zehirlenmemeli");
    let ikinci_okuyucu = depo.ayarlar.try_read().is_ok();
    let yazar = depo.ayarlar.try_write().is_ok();
    println!("bir okuyucu varken ({} ayar):", okuyucu1.len());
    println!("  ikinci okuyucu = {ikinci_okuyucu}, yazar = {yazar}");

    drop(okuyucu1);
    println!("okuyucu çıkınca yazar = {}", depo.ayarlar.try_write().is_ok());
    println!("tema = {:?}, sürüm = {}", depo.oku("tema"), depo.surum());
}
```

Çıktı:

```text
bir okuyucu varken (1 ayar):
  ikinci okuyucu = true, yazar = false
okuyucu çıkınca yazar = true
tema = Some("açık"), sürüm = 1
```

Tasarımda iki karar var. Birincisi, sürüm sayısı kilidin içinde değil, ayrı bir atomiktedir; tek bir sayı için kilide gerek yoktur. İkincisi, `oku` koruyucuyu dışarı vermez: Değeri klonlayıp kilidi hemen bırakır. Koruyucuyu çağırana verseydik kilidin ne kadar tutulacağı çağırana kalırdı ve unutulan bir okuma koruyucusu bütün yazarları bekletirdi.

Klonlamadan, `&str` döndürerek bu sorundan kaçmak da mümkün değildir. Metin kilidin içindedir; koruyucu fonksiyonun sonunda düşünce metne giden referans da geçersiz kalırdı:

```rust,compile_fail,E0515
# use std::collections::HashMap;
# use std::sync::RwLock;
struct AyarDeposu {
    ayarlar: RwLock<HashMap<String, String>>,
}

impl AyarDeposu {
    fn oku(&self, anahtar: &str) -> Option<&str> {
        let kilit = self.ayarlar.read().expect("kilit zehirlenmemeli");
        kilit.get(anahtar).map(String::as_str)
    }
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0515]: cannot return value referencing local variable `kilit`
  |
8 |         kilit.get(anahtar).map(String::as_str)
  |         -----^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |         |
  |         returns a value referencing data owned by the current function
  |         `kilit` is borrowed here
```

Koruyucu fonksiyonun yerel değişkenidir ve fonksiyonla birlikte düşer; derleyici, onun içinden alınmış referansın dışarı çıkmasına izin vermez.[^e0515]

Şimdi depoyu gerçekten paylaşalım: dört okuyucu ve bir yazar. Okuyucular ya eski ya da yeni değeri görür; yarım yazılmış bir değeri göremezler, çünkü yazar içerideyken kimse okuyamaz:

```rust
# use std::collections::HashMap;
# use std::sync::RwLock;
# use std::sync::atomic::{AtomicU64, Ordering};
# pub struct AyarDeposu { ayarlar: RwLock<HashMap<String, String>>, surum: AtomicU64 }
# impl AyarDeposu {
#     pub fn yeni() -> Self {
#         AyarDeposu { ayarlar: RwLock::new(HashMap::new()), surum: AtomicU64::new(0) }
#     }
#     pub fn oku(&self, anahtar: &str) -> Option<String> {
#         self.ayarlar.read().expect("kilit zehirlenmemeli").get(anahtar).cloned()
#     }
#     pub fn yaz(&self, anahtar: &str, deger: &str) {
#         let mut ayarlar = self.ayarlar.write().expect("kilit zehirlenmemeli");
#         ayarlar.insert(anahtar.to_string(), deger.to_string());
#         self.surum.fetch_add(1, Ordering::Relaxed);
#     }
#     pub fn surum(&self) -> u64 { self.surum.load(Ordering::Relaxed) }
# }
# use std::sync::atomic::AtomicU32;
# use std::thread;
fn main() {
    let depo = AyarDeposu::yeni();
    depo.yaz("tema", "açık");

    let gecersiz_okuma = AtomicU32::new(0);
    thread::scope(|k| {
        for _ in 0..4 {
            k.spawn(|| {
                for _ in 0..1_000 {
                    if !matches!(depo.oku("tema").as_deref(), Some("açık" | "koyu")) {
                        gecersiz_okuma.fetch_add(1, Ordering::Relaxed);
                    }
                }
            });
        }
        k.spawn(|| {
            for i in 0..100 {
                depo.yaz("tema", if i % 2 == 0 { "koyu" } else { "açık" });
            }
        });
    });

    println!("son değer = {:?}, sürüm = {}", depo.oku("tema"), depo.surum());
    println!("geçersiz okuma = {}", gecersiz_okuma.into_inner());
}
```

Çıktı:

```text
son değer = Some("açık"), sürüm = 101
geçersiz okuma = 0
```

Thread'lerin hangi sırayla çalıştığı her seferinde değişir, ama sonuç değişmez: Son yazılan değer "açık"tır, yazma sayısı ilk yazmayla birlikte 101'dir.

### Ne zaman `Mutex` daha uygundur?

`RwLock` her durumda daha hızlı değildir. Üç durumda `Mutex`'i yeğleyin:

- **Kritik bölge çok kısaysa:** Örneğin yalnızca bir sayı artırılıyorsa. `RwLock` içerideki okuyucuları da saymak zorundadır;[^okuyucu-sayaci] bu ek iş, birlikte okumanın kazancını silebilir.
- **Yazma sıksa:** Yazarlar zaten tek tek girer; okuyucular da sık sık onları bekler.
- **`T` türü `Sync` değilse:** Böyle bir `RwLock` thread'ler arasında paylaşılamaz; nedenini aşağıdaki `Send` ve `Sync` tablosunda göreceğiz.

Yalnızca "okuma da var" diye refleksle `RwLock` seçmeyin; okuyucular gerçekten birbirini bekletiyorsa seçin ve kazancı ölçün. Okuyucularla yazarlar aynı anda beklerken kime öncelik verileceği de işletim sistemine bırakılmıştır; `std` belirli bir öncelik sözü vermez.[^rwlock]

### Yazma kilidini okumaya indirmek

Okuma kilidi yazmaya yükseltilemez (bunu [sık hatalar](#sık-hatalar-ve-çözümleri) arasında göreceğiz), ama tersi mümkündür. Rust 1.92'de kararlı olan `RwLockWriteGuard::downgrade`, yazma kilidini araya başka bir yazar sokmadan okuma kilidine çevirir:[^downgrade]

```rust
use std::sync::{RwLock, RwLockWriteGuard};

fn main() {
    let kilit = RwLock::new(1);

    let mut yazar = kilit.write().expect("kilit zehirlenmemeli");
    *yazar += 1;
    let okuyucu = RwLockWriteGuard::downgrade(yazar);

    println!("değer = {}", *okuyucu);
    println!("başka okuyucu girebilir mi? {}", kilit.try_read().is_ok());
    println!("başka yazar girebilir mi?   {}", kilit.try_write().is_ok());
}
```

Çıktı:

```text
değer = 2
başka okuyucu girebilir mi? true
başka yazar girebilir mi?   false
```

Yazdığınız değerin siz okurken değişmeyeceğinden emin olmak, ama öteki okuyucuları da bekletmemek istiyorsanız işe yarar.

## `Condvar`: koşul gerçekleşene dek uyumak

Bir tüketici, kuyruk boşken ne yapsın? Döngüde kilitleyip bakmak bir çekirdeği boşuna yakar; araya `sleep` koymak hem gecikme ekler hem de thread'i yine boş yere uyandırır. İstenen şudur: "Kuyruğa bir şey gelince beni uyandır."

Pastanenin tezgâhında dört börek alan bir tepsi düşünün. Tepsi boşsa müşteri kapıda dikilip durmadan "Geldi mi?" diye sormaz; oturur, börek gelince görevli zili çalar. Tepsi doluysa bu kez fırıncı bekler ve yer açılınca onun zili çalar. `Condvar` (koşul değişkeni) bu zildir ve her zaman bir `Mutex` ile birlikte kullanılır: Kilit tepsiyi korur, zil bekleyeni uyandırır.

En küçük hâliyle kullanımı şöyledir:

```rust
use std::sync::{Condvar, Mutex};
use std::thread;

fn main() {
    let hazir = Mutex::new(false);
    let zil = Condvar::new();

    thread::scope(|k| {
        k.spawn(|| {
            *hazir.lock().expect("kilit zehirlenmemeli") = true; // koşul kilit altında değişir
            zil.notify_one();
        });

        let mut kilit = hazir.lock().expect("kilit zehirlenmemeli");
        while !*kilit {
            kilit = zil.wait(kilit).expect("kilit zehirlenmemeli");
        }
        println!("uyandım, koşul doğru mu? {}", *kilit);
    });
}
```

Çıktı:

```text
uyandım, koşul doğru mu? true
```

`wait(kilit)` iki işi atomik olarak, yani araya kimse giremeden yapar: Kilidi bırakır ve thread'i uyutur. Uyanınca kilidi geri alıp koruyucuyu size döndürür.[^wait] "Bırak ve uyu" ikilisinin atomik olması şarttır. Arada boşluk olsaydı şu sıra yaşanabilirdi:

| Adım | Tüketici | Üretici |
| --- | --- | --- |
| 1 | Koşula bakar: Henüz doğru değil. | |
| 2 | Kilidi bırakır. | |
| 3 | | Koşulu değiştirir ve zili çalar. Bekleyen kimse yoktur. |
| 4 | Uyur; zil bir daha çalmaz. | |

Buna *kayıp uyandırma* denir. Bildirimler biriktirilmez: Kimse beklemiyorken çalan zil boşa gider.[^notify] `wait` atomik olduğu için üretici koşulu ancak tüketici uyuduktan sonra değiştirebilir ve zil yerini bulur.

Buradan iki kural çıkar:

1. **Koşulu döngüde denetleyin.** `wait` sebepsiz yere dönebilir (spurious wakeup). Ayrıca siz kilidi geri alana dek başka bir thread öğeyi kapmış olabilir.[^wait]
2. **Koşulu kilit altında değiştirin, sonra bildirin.** Kilitsiz değiştirirseniz tablodaki boşluğu kendiniz açmış olursunuz.

`wait_while` ilk kuralın döngüsünü sizin yerinize yazar: Verdiğiniz closure `true` döndürdükçe uyur.[^wait-while]

```rust
# use std::sync::{Condvar, Mutex};
# use std::thread;
# fn main() {
#     let hazir = Mutex::new(false);
#     let zil = Condvar::new();
#     thread::scope(|k| {
#         k.spawn(|| {
#             *hazir.lock().expect("kilit zehirlenmemeli") = true;
#             zil.notify_one();
#         });
        let kilit = hazir.lock().expect("kilit zehirlenmemeli");
        let kilit = zil.wait_while(kilit, |h| !*h).expect("kilit zehirlenmemeli");
        assert!(*kilit);
#     });
# }
```

### Sınırlı kuyruk ve geri basınç

`Condvar`'ın klasik örneği *sınırlı kuyruk* (bounded queue) yapısıdır. Kuyruk boşsa tüketici, doluysa üretici bekler. İkincisine *geri basınç* (backpressure) denir: Hızlı üretici, yavaş tüketicinin önünde sınırsız öğe yığıp belleği şişiremez.

İki ayrı bekleme nedeni olduğu için iki ayrı `Condvar` kullanırız. Önce veri yapısı ve üretici tarafı:

```rust
use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};

pub struct SinirliKuyruk<T> {
    durum: Mutex<KuyrukDurumu<T>>,
    dolu_degil: Condvar, // üreticiler bunda bekler
    bos_degil: Condvar,  // tüketiciler bunda bekler
    kapasite: usize,
}

struct KuyrukDurumu<T> {
    ogeler: VecDeque<T>,
    kapali: bool,
    en_yuksek_doluluk: usize,
}

impl<T> SinirliKuyruk<T> {
    pub fn yeni(kapasite: usize) -> Self {
        SinirliKuyruk {
            durum: Mutex::new(KuyrukDurumu {
                ogeler: VecDeque::new(),
                kapali: false,
                en_yuksek_doluluk: 0,
            }),
            dolu_degil: Condvar::new(),
            bos_degil: Condvar::new(),
            kapasite: kapasite.max(1),
        }
    }

    // Yer açılana dek bekler. Kuyruk kapalıysa öğeyi geri verir.
    pub fn koy(&self, oge: T) -> Result<(), T> {
        let kilit = self.durum.lock().expect("kilit zehirlenmemeli");
        let mut durum = self
            .dolu_degil
            .wait_while(kilit, |d| d.ogeler.len() >= self.kapasite && !d.kapali)
            .expect("kilit zehirlenmemeli");
        if durum.kapali {
            return Err(oge);
        }
        durum.ogeler.push_back(oge);
        durum.en_yuksek_doluluk = durum.en_yuksek_doluluk.max(durum.ogeler.len());
        drop(durum); // önce kilidi bırak, sonra bildir
        self.bos_degil.notify_one();
        Ok(())
    }
}
# fn main() {}
```

Bekleme koşulunda kapanış da vardır; olmasaydı kapalı ve dolu bir kuyrukta üretici sonsuza dek beklerdi. `koy`, kapalı kuyrukta öğeyi yok etmek yerine `Err` içinde geri verir.

Bildirimin kilit bırakıldıktan sonra gelmesi bir verim inceliğidir. Kilit eldeyken bildirilseydi uyanan tüketici hemen kilidi ister, alamaz ve yeniden uyurdu. Doğruluk açısından iki sıra da geçerlidir, çünkü koşul kilit altında değişmiştir.

Şimdi tüketici tarafı ve kapanış:

```rust
# use std::collections::VecDeque;
# use std::sync::{Condvar, Mutex};
# pub struct SinirliKuyruk<T> {
#     durum: Mutex<KuyrukDurumu<T>>,
#     dolu_degil: Condvar,
#     bos_degil: Condvar,
#     kapasite: usize,
# }
# struct KuyrukDurumu<T> { ogeler: VecDeque<T>, kapali: bool, en_yuksek_doluluk: usize }
# impl<T> SinirliKuyruk<T> {
#     pub fn yeni(kapasite: usize) -> Self {
#         SinirliKuyruk {
#             durum: Mutex::new(KuyrukDurumu {
#                 ogeler: VecDeque::new(), kapali: false, en_yuksek_doluluk: 0,
#             }),
#             dolu_degil: Condvar::new(),
#             bos_degil: Condvar::new(),
#             kapasite: kapasite.max(1),
#         }
#     }
#     pub fn koy(&self, oge: T) -> Result<(), T> {
#         let kilit = self.durum.lock().expect("kilit zehirlenmemeli");
#         let mut durum = self
#             .dolu_degil
#             .wait_while(kilit, |d| d.ogeler.len() >= self.kapasite && !d.kapali)
#             .expect("kilit zehirlenmemeli");
#         if durum.kapali { return Err(oge); }
#         durum.ogeler.push_back(oge);
#         durum.en_yuksek_doluluk = durum.en_yuksek_doluluk.max(durum.ogeler.len());
#         drop(durum);
#         self.bos_degil.notify_one();
#         Ok(())
#     }
# }
impl<T> SinirliKuyruk<T> {
    // Öğe gelene dek bekler. None: Kuyruk hem kapalı hem boş.
    pub fn al(&self) -> Option<T> {
        let kilit = self.durum.lock().expect("kilit zehirlenmemeli");
        let mut durum = self
            .bos_degil
            .wait_while(kilit, |d| d.ogeler.is_empty() && !d.kapali)
            .expect("kilit zehirlenmemeli");
        let oge = durum.ogeler.pop_front();
        drop(durum);
        if oge.is_some() {
            self.dolu_degil.notify_one(); // yer açıldı
        }
        oge
    }

    pub fn kapat(&self) {
        self.durum.lock().expect("kilit zehirlenmemeli").kapali = true;
        self.bos_degil.notify_all();
        self.dolu_degil.notify_all();
    }

    pub fn en_yuksek_doluluk(&self) -> usize {
        self.durum.lock().expect("kilit zehirlenmemeli").en_yuksek_doluluk
    }
}

fn main() {
    let kuyruk = SinirliKuyruk::yeni(3);
    println!("koy(1) = {:?}, koy(2) = {:?}", kuyruk.koy(1), kuyruk.koy(2));

    kuyruk.kapat();
    println!("kapalıyken koy(7) = {:?}", kuyruk.koy(7));
    println!("kalanlar: {:?}, {:?}", kuyruk.al(), kuyruk.al());
    println!("kapalı ve boş: {:?}", kuyruk.al());
}
```

Çıktı:

```text
koy(1) = Ok(()), koy(2) = Ok(())
kapalıyken koy(7) = Err(7)
kalanlar: Some(1), Some(2)
kapalı ve boş: None
```

Kapanış, içerideki öğeleri silmez: Önce kalanlar tüketilir, ancak kuyruk hem kapalı hem boşsa `al` beklemeden `None` döndürür. `kapat` iki zilde de `notify_all` kullanır, çünkü kapanış bekleyen herkesi ilgilendirir.

Dolu kuyruğun üreticiyi gerçekten beklettiğini `is_finished` ile görebiliriz. Kuyrukta tek yer varken ikinci `koy`, biri `al` diyene kadar dönemez:

```rust
# use std::collections::VecDeque;
# use std::sync::{Condvar, Mutex};
# pub struct SinirliKuyruk<T> {
#     durum: Mutex<KuyrukDurumu<T>>,
#     dolu_degil: Condvar,
#     bos_degil: Condvar,
#     kapasite: usize,
# }
# struct KuyrukDurumu<T> { ogeler: VecDeque<T>, kapali: bool, en_yuksek_doluluk: usize }
# impl<T> SinirliKuyruk<T> {
#     pub fn yeni(kapasite: usize) -> Self {
#         SinirliKuyruk {
#             durum: Mutex::new(KuyrukDurumu {
#                 ogeler: VecDeque::new(), kapali: false, en_yuksek_doluluk: 0,
#             }),
#             dolu_degil: Condvar::new(),
#             bos_degil: Condvar::new(),
#             kapasite: kapasite.max(1),
#         }
#     }
#     pub fn koy(&self, oge: T) -> Result<(), T> {
#         let kilit = self.durum.lock().expect("kilit zehirlenmemeli");
#         let mut durum = self
#             .dolu_degil
#             .wait_while(kilit, |d| d.ogeler.len() >= self.kapasite && !d.kapali)
#             .expect("kilit zehirlenmemeli");
#         if durum.kapali { return Err(oge); }
#         durum.ogeler.push_back(oge);
#         durum.en_yuksek_doluluk = durum.en_yuksek_doluluk.max(durum.ogeler.len());
#         drop(durum);
#         self.bos_degil.notify_one();
#         Ok(())
#     }
#     pub fn al(&self) -> Option<T> {
#         let kilit = self.durum.lock().expect("kilit zehirlenmemeli");
#         let mut durum = self
#             .bos_degil
#             .wait_while(kilit, |d| d.ogeler.is_empty() && !d.kapali)
#             .expect("kilit zehirlenmemeli");
#         let oge = durum.ogeler.pop_front();
#         drop(durum);
#         if oge.is_some() { self.dolu_degil.notify_one(); }
#         oge
#     }
#     pub fn kapat(&self) {
#         self.durum.lock().expect("kilit zehirlenmemeli").kapali = true;
#         self.bos_degil.notify_all();
#         self.dolu_degil.notify_all();
#     }
#     pub fn en_yuksek_doluluk(&self) -> usize {
#         self.durum.lock().expect("kilit zehirlenmemeli").en_yuksek_doluluk
#     }
# }
# use std::thread;
# use std::time::Duration;
fn main() {
    let kuyruk = SinirliKuyruk::yeni(1);
    kuyruk.koy(1).expect("kuyruk açık");

    thread::scope(|k| {
        let uretici = k.spawn(|| kuyruk.koy(2)); // yer açılana dek bekler
        thread::sleep(Duration::from_millis(20));
        println!("20 ms sonra üretici bitti mi? {}", uretici.is_finished());
        println!("yer açıyorum: {:?}", kuyruk.al());
        println!("üreticinin sonucu: {:?}", uretici.join().expect("thread paniklememeli"));
    });

    println!("sıradaki öğe: {:?}", kuyruk.al());
    println!("en yüksek doluluk: {}", kuyruk.en_yuksek_doluluk());
}
```

Çıktı:

```text
20 ms sonra üretici bitti mi? false
yer açıyorum: Some(1)
üreticinin sonucu: Ok(())
sıradaki öğe: Some(2)
en yüksek doluluk: 1
```

Son olarak iki üretici ve iki tüketici. Üreticiler bitince kuyruk kapatılır; tüketiciler kalanları alıp `None` görünce döngüden çıkar:

```rust
# use std::collections::VecDeque;
# use std::sync::{Condvar, Mutex};
# pub struct SinirliKuyruk<T> {
#     durum: Mutex<KuyrukDurumu<T>>,
#     dolu_degil: Condvar,
#     bos_degil: Condvar,
#     kapasite: usize,
# }
# struct KuyrukDurumu<T> { ogeler: VecDeque<T>, kapali: bool, en_yuksek_doluluk: usize }
# impl<T> SinirliKuyruk<T> {
#     pub fn yeni(kapasite: usize) -> Self {
#         SinirliKuyruk {
#             durum: Mutex::new(KuyrukDurumu {
#                 ogeler: VecDeque::new(), kapali: false, en_yuksek_doluluk: 0,
#             }),
#             dolu_degil: Condvar::new(),
#             bos_degil: Condvar::new(),
#             kapasite: kapasite.max(1),
#         }
#     }
#     pub fn koy(&self, oge: T) -> Result<(), T> {
#         let kilit = self.durum.lock().expect("kilit zehirlenmemeli");
#         let mut durum = self
#             .dolu_degil
#             .wait_while(kilit, |d| d.ogeler.len() >= self.kapasite && !d.kapali)
#             .expect("kilit zehirlenmemeli");
#         if durum.kapali { return Err(oge); }
#         durum.ogeler.push_back(oge);
#         durum.en_yuksek_doluluk = durum.en_yuksek_doluluk.max(durum.ogeler.len());
#         drop(durum);
#         self.bos_degil.notify_one();
#         Ok(())
#     }
#     pub fn al(&self) -> Option<T> {
#         let kilit = self.durum.lock().expect("kilit zehirlenmemeli");
#         let mut durum = self
#             .bos_degil
#             .wait_while(kilit, |d| d.ogeler.is_empty() && !d.kapali)
#             .expect("kilit zehirlenmemeli");
#         let oge = durum.ogeler.pop_front();
#         drop(durum);
#         if oge.is_some() { self.dolu_degil.notify_one(); }
#         oge
#     }
#     pub fn kapat(&self) {
#         self.durum.lock().expect("kilit zehirlenmemeli").kapali = true;
#         self.bos_degil.notify_all();
#         self.dolu_degil.notify_all();
#     }
#     pub fn en_yuksek_doluluk(&self) -> usize {
#         self.durum.lock().expect("kilit zehirlenmemeli").en_yuksek_doluluk
#     }
# }
# use std::thread;
// 2 üretici (her biri 1..=n), 2 tüketici. Dönen: tüketilenlerin toplamı.
pub fn uret_tuket(kuyruk: &SinirliKuyruk<u64>, uretici_basina: u64) -> u64 {
    thread::scope(|k| {
        let uret = || {
            for i in 1..=uretici_basina {
                kuyruk.koy(i).expect("kuyruk üretim bitmeden kapatılmamalı");
            }
        };
        let tuket = || {
            let mut toplam = 0;
            while let Some(x) = kuyruk.al() {
                toplam += x;
            }
            toplam
        };
        let ureticiler = [k.spawn(uret), k.spawn(uret)];
        let tuketiciler = [k.spawn(tuket), k.spawn(tuket)];

        for uretici in ureticiler {
            uretici.join().expect("üretici paniklememeli");
        }
        kuyruk.kapat(); // üretim bitti
        tuketiciler.into_iter().map(|t| t.join().expect("tüketici paniklememeli")).sum()
    })
}

fn main() {
    let kuyruk = SinirliKuyruk::yeni(4);
    println!("tüketilen toplam = {}", uret_tuket(&kuyruk, 500));
    println!("doluluk 4'ü hiç aşmadı mı? {}", kuyruk.en_yuksek_doluluk() <= 4);
}
```

Çıktı:

```text
tüketilen toplam = 250500
doluluk 4'ü hiç aşmadı mı? true
```

Hangi öğeyi hangi tüketicinin aldığı değişir, ama hiçbir öğe kaybolmaz: İki üreticinin her biri 1'den 500'e kadar ürettiği için toplam 2 × 125 250 = 250 500'dür.

> [!NOTE]
> `std` aynı davranışı hazır verir: `mpsc::sync_channel(n)` kapasitesi `n` olan bir kanaldır; dolunca `send` bekler, `try_send` ise beklemeden `TrySendError::Full` döndürür.[^sync-channel] Kuyruğu elle yazmamızın nedeni kalıbı görmektir: "Bir `Mutex`'in koruduğu durum üzerinde bir koşulu bekle." Bağlantı havuzu, iş kuyruğu, bariyer ve semafor hep bu iskelettir.

### Süreli bekleme

Bazen beklemenin bir sınırı olmalıdır. `wait_timeout_while` koşul düzelene ya da süre dolana dek bekler; dönen ikilinin ikinci öğesi hangisinin olduğunu söyler:[^sureli]

```rust
use std::sync::{Condvar, Mutex};
use std::time::{Duration, Instant};

fn main() {
    let hazir = Mutex::new(false);
    let zil = Condvar::new(); // kimse çalmayacak

    let baslangic = Instant::now();
    let kilit = hazir.lock().expect("kilit zehirlenmemeli");
    let (kilit, sonuc) = zil
        .wait_timeout_while(kilit, Duration::from_millis(20), |h| !*h)
        .expect("kilit zehirlenmemeli");

    println!("koşul = {}, süre doldu mu? {}", *kilit, sonuc.timed_out());
    println!("en az 20 ms geçti mi? {}", baslangic.elapsed() >= Duration::from_millis(20));
}
```

Çıktı:

```text
koşul = false, süre doldu mu? true
en az 20 ms geçti mi? true
```

Sayfanın sonundaki ilk alıştırmada bunu `SinirliKuyruk`'a ekleyeceksiniz.

## Kilitlenme ve kilit sıralaması

Güvenli Rust veri yarışını derleme zamanında önler, ama *kilitlenmeyi* (deadlock) önlemez. Rust başvuru kılavuzu kilitlenmeyi, istenmese de `unsafe` sayılmayan davranışlar arasında açıkça listeler.[^kilitlenme] İki çocuğun iki oyuncakla oynamak istediğini düşünün. Ali kırmızı arabayı kapıp mavi arabayı bekliyor; Ayşe mavi arabayı kapıp kırmızıyı bekliyor. İkisi de elindekini bırakmadığı için sonsuza dek beklerler.

Aynı durumu iki kilitle kuralım. Programın takılmaması için ikinci kilitleri `lock` yerine `try_lock` ile isteyelim; `Barrier` da iki thread'in aynı noktada buluşmasını sağlasın:[^bariyer]

```rust
use std::sync::{Barrier, Mutex};
use std::thread;

fn main() {
    let a = Mutex::new("a");
    let b = Mutex::new("b");
    let bulusma = Barrier::new(2);

    let (birinci, ikinci) = thread::scope(|k| {
        let birinci = k.spawn(|| {
            let _a = a.lock().expect("kilit zehirlenmemeli");
            bulusma.wait(); // iki taraf da ilk kilidini aldı
            let alindi = b.try_lock().is_ok();
            bulusma.wait(); // iki taraf da denedi
            alindi
        });
        let ikinci = k.spawn(|| {
            let _b = b.lock().expect("kilit zehirlenmemeli");
            bulusma.wait();
            let alindi = a.try_lock().is_ok();
            bulusma.wait();
            alindi
        });
        [birinci, ikinci].map(|t| t.join().expect("thread paniklememeli")).into()
    });

    println!("a'yı tutan thread b'yi alabildi mi? {birinci}");
    println!("b'yi tutan thread a'yı alabildi mi? {ikinci}");
}
```

Çıktı:

```text
a'yı tutan thread b'yi alabildi mi? false
b'yi tutan thread a'yı alabildi mi? false
```

`try_lock` yerine `lock` yazsaydık iki thread de sonsuza dek beklerdi. Böyle bir program derlenir; yüz kez doğru çalışıp yüz birinci seferde takılabilir.

### Örnek: hesaplar arası transfer

Para transferi tam bu tuzaktır. `transfer(a, b)` ile `transfer(b, a)` aynı anda çağrılırsa, "Önce kaynağı, sonra hedefi kilitle" diye yazılmış bir kod yukarıdaki duruma düşebilir.

Çözüm bir kuraldır: Bütün kod, kilitleri aynı genel sırayla alır. Oyuncak örneğinde bu, "Herkes önce küçük numaralı oyuncağa uzanır" diyen bir ev kuralıdır. Burada sıra hesap numarasıdır. İki thread de aynı ilk kilidi isteyeceği için biri bekler; döngüsel bekleme oluşamaz:

```rust
use std::sync::Mutex;

pub struct Hesap {
    pub no: u32,
    bakiye: Mutex<i64>,
}
# impl Hesap {
#     pub fn yeni(no: u32, bakiye: i64) -> Self { Hesap { no, bakiye: Mutex::new(bakiye) } }
#     pub fn bakiye(&self) -> i64 { *self.bakiye.lock().expect("kilit zehirlenmemeli") }
# }

pub fn transfer(kimden: &Hesap, kime: &Hesap, tutar: i64) -> bool {
    if kimden.no == kime.no || tutar <= 0 {
        return false;
    }
    // 1) Sırayı yöne göre değil, hesap numarasına göre belirle.
    let kaynak_once = kimden.no < kime.no;
    let (ilk, ikinci) = if kaynak_once { (kimden, kime) } else { (kime, kimden) };

    // 2) İki kilidi de al; ondan önce hiçbir bakiyeye dokunma.
    let mut ilk_kilit = ilk.bakiye.lock().expect("kilit zehirlenmemeli");
    let mut ikinci_kilit = ikinci.bakiye.lock().expect("kilit zehirlenmemeli");

    // 3) Koruyucuları yeniden kaynak ve hedef rollerine eşle.
    let (kaynak, hedef) = if kaynak_once {
        (&mut *ilk_kilit, &mut *ikinci_kilit)
    } else {
        (&mut *ikinci_kilit, &mut *ilk_kilit)
    };
    if *kaynak < tutar {
        return false;
    }
    *kaynak -= tutar;
    *hedef += tutar;
    true
}

fn main() {
    let a = Hesap::yeni(1, 100);
    let b = Hesap::yeni(2, 100);
    println!("a'dan b'ye 30: {}", transfer(&a, &b, 30)); // kilit sırası 1, 2
    println!("b'den a'ya 10: {}", transfer(&b, &a, 10)); // yön ters, sıra yine 1, 2
    println!("a'dan a'ya 5: {}", transfer(&a, &a, 5));
    println!("a'dan b'ye -5: {}", transfer(&a, &b, -5));
    println!("a = {}, b = {}", a.bakiye(), b.bakiye());
}
```

Çıktı:

```text
a'dan b'ye 30: true
b'den a'ya 10: true
a'dan a'ya 5: false
a'dan b'ye -5: false
a = 80, b = 120
```

Baştaki denetimler süs değildir. Kaynakla hedef aynı hesapsa aynı `Mutex` iki kez kilitlenirdi; bu çağrı geri dönmez, ya bekler ya da panikler.[^ayni-kilit] Negatif tutar ise transferi ters çevirir ve bakiye denetimini atlatırdı.

İki kilit de elde olmadan hiçbir bakiyeye dokunulmadığı için "kaynaktan düştü, ama hedefe eklenmedi" ara durumu dışarıdan görülemez. Koruyucular tanımın tersi sırayla düşer; kilitleri *bırakma* sırasının kilitlenme açısından önemi yoktur, önemli olan *alma* sırasıdır.

Kaynaktan başlayarak kilitleyen bir kodu takabilecek yükü deneyelim: aynı anda bir yönde bin, öteki yönde beş yüz transfer.

```rust
# use std::sync::Mutex;
# pub struct Hesap { pub no: u32, bakiye: Mutex<i64> }
# impl Hesap {
#     pub fn yeni(no: u32, bakiye: i64) -> Self { Hesap { no, bakiye: Mutex::new(bakiye) } }
#     pub fn bakiye(&self) -> i64 { *self.bakiye.lock().expect("kilit zehirlenmemeli") }
# }
# pub fn transfer(kimden: &Hesap, kime: &Hesap, tutar: i64) -> bool {
#     if kimden.no == kime.no || tutar <= 0 { return false; }
#     let kaynak_once = kimden.no < kime.no;
#     let (ilk, ikinci) = if kaynak_once { (kimden, kime) } else { (kime, kimden) };
#     let mut ilk_kilit = ilk.bakiye.lock().expect("kilit zehirlenmemeli");
#     let mut ikinci_kilit = ikinci.bakiye.lock().expect("kilit zehirlenmemeli");
#     let (kaynak, hedef) = if kaynak_once {
#         (&mut *ilk_kilit, &mut *ikinci_kilit)
#     } else {
#         (&mut *ikinci_kilit, &mut *ilk_kilit)
#     };
#     if *kaynak < tutar { return false; }
#     *kaynak -= tutar;
#     *hedef += tutar;
#     true
# }
# use std::thread;
fn main() {
    let a = Hesap::yeni(1, 1_000);
    let b = Hesap::yeni(2, 1_000);
    thread::scope(|k| {
        k.spawn(|| (0..1_000).for_each(|_| assert!(transfer(&a, &b, 1))));
        k.spawn(|| (0..500).for_each(|_| assert!(transfer(&b, &a, 1))));
    });

    println!("A = {}, B = {}", a.bakiye(), b.bakiye());
    println!("toplam = {}", a.bakiye() + b.bakiye());
    println!("yetersiz bakiye: {}", transfer(&a, &b, 10_000));
}
```

Çıktı:

```text
A = 500, B = 1500
toplam = 2000
yetersiz bakiye: false
```

### Öteki savunmalar

Kilit sıralaması tek çare değildir:

- **Tek kilit:** İki hesabı aynı `Mutex<Banka>` korusun. En basit yoldur ve çoğu zaman yeterlidir.
- **Dene ve geri çekil:** İkinci kilit alınamazsa birinciyi bırakıp yeniden deneyin (aşağıda).
- **Kilit eldeyken yabancı kod çağırmamak:** Bir closure'ın ya da trait metodunun içeride hangi kilidi alacağını bilemezsiniz.
- **Mesajlaşma:** Hesapların tek sahibi bir thread olsun, transfer istekleri kanaldan gelsin. Bu yaklaşımı [15.1](../orta/15-concurrency-derinlesme.md) anlatıyor.

İkinci yol kodda şöyle görünür:

```rust
# use std::sync::Mutex;
# pub struct Hesap { pub no: u32, bakiye: Mutex<i64> }
# impl Hesap {
#     pub fn yeni(no: u32, bakiye: i64) -> Self { Hesap { no, bakiye: Mutex::new(bakiye) } }
#     pub fn bakiye(&self) -> i64 { *self.bakiye.lock().expect("kilit zehirlenmemeli") }
# }
use std::sync::TryLockError;
# use std::thread;

pub fn transfer_geri_cekilen(kimden: &Hesap, kime: &Hesap, tutar: i64) -> bool {
    if kimden.no == kime.no || tutar <= 0 {
        return false;
    }
    loop {
        let mut kaynak = kimden.bakiye.lock().expect("kilit zehirlenmemeli");
        match kime.bakiye.try_lock() {
            Ok(mut hedef) => {
                if *kaynak < tutar {
                    return false;
                }
                *kaynak -= tutar;
                *hedef += tutar;
                return true;
            }
            Err(TryLockError::WouldBlock) => {} // şu an dolu
            Err(TryLockError::Poisoned(_)) => panic!("kilit zehirlendi"),
        }
        drop(kaynak); // elindekini bırak
        thread::yield_now(); // sırayı ötekilere ver
    }
}

fn main() {
    let a = Hesap::yeni(1, 1_000);
    let b = Hesap::yeni(2, 1_000);
    thread::scope(|k| {
        k.spawn(|| (0..1_000).for_each(|_| assert!(transfer_geri_cekilen(&a, &b, 1))));
        k.spawn(|| (0..500).for_each(|_| assert!(transfer_geri_cekilen(&b, &a, 1))));
    });
    println!("A = {}, B = {}", a.bakiye(), b.bakiye());
}
```

Çıktı:

```text
A = 500, B = 1500
```

Bu yol döngüsel beklemeyi bozar, çünkü hiçbir thread, elinde bir kilit varken ötekini uyuyarak beklemez. Bedeli boşa giden denemelerdir: Çekişme yüksekse thread'ler birbirine yol vermekle vakit kaybeder. Kilit sıralaması kurulabiliyorsa onu yeğleyin.

## Atomikler: tek adımlık işlemler

Bayrak kapmaca oynadığınızı düşünün. Bayrağı kapmak tek hamledir: İki çocuk aynı bayrağı yarı yarıya kapamaz; biri kapar, öteki boş yere uzanmış olur. *Atomik* değerler böyle çalışır: Okuma, yazma ve "oku, değiştir, yaz" işlemleri yarıda kesilemeyen tek hamledir, bu yüzden kilit gerekmez.[^atomik] Tek bir sayı ya da bayrak için kilit kurmak çoğu zaman gereksizdir.

Kilitsiz ortak yazmayı sıradan bir değişkenle deneseydik program derlenmezdi:

```rust,compile_fail,E0499
use std::thread;

fn main() {
    let mut n = 0;
    thread::scope(|k| {
        k.spawn(|| n += 1);
        k.spawn(|| n += 1);
    });
    println!("{n}");
}
```

Derleyicinin yanıtı:

```text
error[E0499]: cannot borrow `n` as mutable more than once at a time
  |
5 |     thread::scope(|k| {
  |                    - has type `&'1 Scope<'1, '_>`
6 |         k.spawn(|| n += 1);
  |         ------------------
  |         |       |  |
  |         |       |  first borrow occurs due to use of `n` in closure
  |         |       first mutable borrow occurs here
  |         argument requires that `n` is borrowed for `'1`
7 |         k.spawn(|| n += 1);
  |                 ^^ - second borrow occurs due to use of `n` in closure
  |                 |
  |                 second mutable borrow occurs here
```

İki closure da `n`'yi `&mut` ile ödünç almak ister; ödünç kuralı buna izin vermez.[^e0499] Atomik türlerin yazma metotları ise `&self` alır, böylece aynı değer birçok thread'den değiştirilebilir.

### `compare_exchange` ile lider seçmek

`compare_exchange(beklenen, yeni, ..)` şunu der: "Değer hâlâ `beklenen` ise `yeni` yap; değilse dokunma ve gördüğün değeri söyle."[^cas] Kilitsiz algoritmaların temel taşıdır, çünkü "oku, karar ver, yaz" üçlüsünü araya kimse girmeden tek adımda yapar:

```rust
use std::sync::atomic::{AtomicU32, Ordering};

fn main() {
    let lider = AtomicU32::new(0); // 0: bayrak yerde
    let ilk = lider.compare_exchange(0, 7, Ordering::SeqCst, Ordering::SeqCst);
    let gec = lider.compare_exchange(0, 9, Ordering::SeqCst, Ordering::SeqCst);

    println!("7 numara: {ilk:?}");
    println!("9 numara: {gec:?}");
    println!("lider: {}", lider.into_inner());
}
```

Çıktı:

```text
7 numara: Ok(0)
9 numara: Err(7)
lider: 7
```

Başarılı çağrı önceki değeri `Ok` içinde, başarısız çağrı o anki değeri `Err` içinde verir. Aynı yarışı gerçek thread'lerle yapalım:

```rust
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;

pub fn lider_sec(aday_sayisi: u32) -> (u32, u32) {
    let lider = AtomicU32::new(0); // 0: henüz lider yok
    let kazanan_sayisi = AtomicU32::new(0);
    thread::scope(|k| {
        for kimlik in 1..=aday_sayisi {
            let (lider, kazanan_sayisi) = (&lider, &kazanan_sayisi);
            k.spawn(move || {
                if lider.compare_exchange(0, kimlik, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                    kazanan_sayisi.fetch_add(1, Ordering::Relaxed);
                }
            });
        }
    });
    (lider.into_inner(), kazanan_sayisi.into_inner())
}

fn main() {
    let (lider, kazanan_sayisi) = lider_sec(8);
    println!("kazanan sayısı = {kazanan_sayisi}");
    println!("lider geçerli bir aday mı? {}", (1..=8).contains(&lider));
}
```

Çıktı:

```text
kazanan sayısı = 1
lider geçerli bir aday mı? true
```

Hangi adayın kazandığı çalıştırmadan çalıştırmaya değişir, ama kazanan hep tektir: `0`'ı yalnızca ilk gelen görür, ötekiler `Err(mevcut lider)` alır.

### Hangi `Ordering`?

Her atomik işlem bir *bellek sıralaması* ister. Bu değer, işlemin çevresindeki öteki bellek erişimlerinin başka thread'lere hangi sırayla görüneceğini belirler:[^ordering]

| Sıralama | Verdiği söz | Ne zaman? |
| --- | --- | --- |
| `Relaxed` | Yalnızca bu işlemin atomikliği. | Bağımsız sayaçlar, başka veriyi yayımlamayan bayraklar. |
| `Release` (yazarken), `Acquire` (okurken) | Bu yazmadan önce yaptıklarım, yazdığımı `Acquire` ile okuyan thread'e görünür. | Bir bayrak ya da işaretçiyle *başka* veriyi yayımlarken. |
| `SeqCst` | Yukarıdakilerin sözü ve ek olarak bütün `SeqCst` işlemleri için tek bir genel sıra. | Emin değilseniz. En güçlüsü ve üzerinde akıl yürütmesi en kolay olanıdır. |

`lider_sec`'te iki sıralama birlikte görülüyor: Lider seçimi `SeqCst`, hiçbir veriyi yayımlamayan bağımsız sayaç `Relaxed`'dir. Yayımlama örneği de şöyledir:

```rust
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::thread;

static VERI: AtomicU32 = AtomicU32::new(0);
static HAZIR: AtomicBool = AtomicBool::new(false);

fn main() {
    let yazar = thread::spawn(|| {
        VERI.store(42, Ordering::Relaxed);
        HAZIR.store(true, Ordering::Release); // veriyi yayımla
    });

    while !HAZIR.load(Ordering::Acquire) {
        thread::yield_now();
    }
    println!("veri = {}", VERI.load(Ordering::Relaxed));
    yazar.join().expect("thread paniklememeli");
}
```

Çıktı:

```text
veri = 42
```

Okuyan thread bayrağı `true` gördüyse `VERI`'nin 42 olduğunu da görür; bu sözü `Release` ile `Acquire` çifti verir. Yanlış seçilmiş bir `Relaxed`, güçlü sıralamalı x86 işlemcilerde çoğu zaman sorunsuz görünür, ARM gibi zayıf sıralamalı işlemcilerde bozulabilir;[^nomicon-atomik] bu yüzden böyle hatalar testle zor bulunur. Ölçmeden zayıflatmayın.

### Durdurma bayrağı

Bir işçiye "Artık dur" demenin en hafif yolu paylaşılan bir `AtomicBool`'dur. Bayrak başka bir veriyi yayımlamadığı, yalnızca kendi değeri önemli olduğu için `Relaxed` yeter:

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

fn main() {
    let dur = Arc::new(AtomicBool::new(false));
    let isci = {
        let dur = Arc::clone(&dur);
        thread::spawn(move || {
            let mut tur: u64 = 0;
            while !dur.load(Ordering::Relaxed) {
                tur += 1; // bir birim iş
                thread::yield_now();
            }
            tur
        })
    };

    dur.store(true, Ordering::Relaxed);
    let tur = isci.join().expect("işçi paniklememeli");
    println!("işçi bayrağı görüp durdu (sayaç geçerli: {})", tur < u64::MAX);
}
```

Çıktı:

```text
işçi bayrağı görüp durdu (sayaç geçerli: true)
```

Tur sayısı zamanlamaya bağlıdır, 0 bile olabilir. Belirli olan, işçinin durmasıdır.

## `Send` ve `Sync`: pratik tablo

[25. bölüm](25-send-sync.md) bu iki trait'in mekanizmasını açtı. Burada gündelik soruyu yanıtlıyoruz: Bu türü başka bir thread'e verebilir miyim, thread'ler arasında paylaşabilir miyim?

Bir kargo şirketi düşünün. `Send`, "Bu paket başka şubeye yollanabilir" damgasıdır. `Sync`, "Bu vitrine birçok şubeden aynı anda bakılabilir" damgasıdır:

| Tür | `Send` | `Sync` | Neden |
| --- | --- | --- | --- |
| `i32`, `String`, `Vec<i32>` | evet | evet | İç değiştirilebilirlik yoktur; `&T` yalnızca okur. |
| `Cell<T>`, `RefCell<T>` | `T: Send` ise | hayır | Taşınabilir, ama `&Cell<T>` üzerinden kilitsiz yazılır. |
| `Rc<T>` | hayır | hayır | Sayacı atomik değildir; bütün klonlar aynı sayaca dokunur. |
| `Arc<T>` | `T: Send + Sync` ise | `T: Send + Sync` ise | Paylaşılan `T`'ye her thread `&T` ile erişir ve sonuncusu onu düşürür. |
| `Mutex<T>` | `T: Send` ise | `T: Send` ise | Kilit erişimi teke indirir; `T`'nin `Sync` olması gerekmez. |
| `RwLock<T>` | `T: Send` ise | `T: Send + Sync` ise | Aynı anda çok okuyucu `&T` görür. |
| `MutexGuard<'_, T>` | hayır | `T: Sync` ise | Kilidi alan thread bırakmalıdır. |
| `mpsc::Sender<T>` | `T: Send` ise | `T: Send` ise | Klonlanıp thread'lere dağıtılabilir. |
| `mpsc::Receiver<T>` | `T: Send` ise | hayır | Kanalın tek tüketicisi vardır. |

Kullanıcı türleri bu damgaları alanlarından türetir: Bütün alanları `Send` olan bir struct kendiliğinden `Send` olur. Bu tür trait'lere *auto trait* denir.[^auto]

Tablonun en öğretici satırları iki kilidin farkıdır. `Mutex` odasına her seferinde tek kişi girer; içerideki eşyanın yalnızca taşınabilir (`Send`) olması yeter. `RwLock` odasına birçok okuyucu birlikte girer; eşyanın birlikte bakılmaya da uygun (`Sync`) olması gerekir. Standart kütüphanenin kaynağında bu fark tek bir sözcüktür:[^mutex-sync]

- `Mutex` için: `unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}`
- `RwLock` için: `unsafe impl<T: ?Sized + Send + Sync> Sync for RwLock<T> {}`

Bir türün damgasını denemenin en kolay yolu, boş gövdeli iki fonksiyondur. Çalışma zamanında hiçbir şey yapmazlar; denetim derleme zamanında olur:

```rust
use std::cell::Cell;
use std::sync::{Arc, Mutex, RwLock};

fn send_ister<T: Send>() {}
fn sync_ister<T: Sync>() {}

fn main() {
    send_ister::<Arc<Mutex<Vec<u8>>>>(); // thread'ler arası paylaşımın klasik kalıbı
    sync_ister::<Arc<RwLock<String>>>();
    send_ister::<Cell<u8>>(); // taşınabilir
    sync_ister::<Mutex<Cell<u8>>>(); // Mutex, Sync olmayan türü Sync yapar
    println!("derlendi: dört tür de damgasını aldı");
}
```

Çıktı:

```text
derlendi: dört tür de damgasını aldı
```

Kilit değişince aynı `Cell` bu denetimden geçemez:

```rust,compile_fail,E0277
# use std::cell::Cell;
# use std::sync::RwLock;
# fn sync_ister<T: Sync>() {}
fn main() {
    sync_ister::<RwLock<Cell<u8>>>(); // iki okuyucu aynı Cell'e yazabilirdi
}
```

Derleyicinin yanıtı:

```text
error[E0277]: `Cell<u8>` cannot be shared between threads safely
  = help: the trait `Sync` is not implemented for `Cell<u8>`
  = note: if you want to do aliasing and mutation between multiple threads, use `std::sync::RwLock` or `std::sync::atomic::AtomicU8` instead
  = note: required for `std::sync::RwLock<Cell<u8>>` to implement `Sync`
```

Doğrudan `sync_ister::<Cell<u8>>()` yazmak da aynı hatayı verir. Kilidin koruyucusu ise başka bir thread'e gönderilemez:

```rust,compile_fail,E0277
# use std::sync::MutexGuard;
# fn send_ister<T: Send>() {}
fn main() {
    send_ister::<MutexGuard<'static, u8>>();
}
```

Derleyicinin yanıtı:

```text
error[E0277]: `std::sync::MutexGuard<'static, u8>` cannot be sent between threads safely
  = help: the trait `Send` is not implemented for `std::sync::MutexGuard<'static, u8>`
```

Bunun nedeni taşınabilirliktir: POSIX thread'lerini kullanan platformlarda kilidi, onu alan thread bırakmalıdır. `std` bu yüzden `MutexGuard` için `Send`'i açıkça kapatır.[^guard-send] Bu satır [29.3](29-istemci.md)'te, kilidi `.await` boyunca tutan görevlerde yeniden karşımıza çıkacak.

Son olarak, `Arc` içindeki türü kendiliğinden güvenli yapmaz:

```rust,compile_fail,E0277
# use std::cell::RefCell;
# use std::sync::Arc;
# fn send_ister<T: Send>() {}
fn main() {
    send_ister::<Arc<RefCell<u8>>>();
}
```

Derleyicinin yanıtı:

```text
error[E0277]: `RefCell<u8>` cannot be shared between threads safely
  = help: the trait `Sync` is not implemented for `RefCell<u8>`
  = note: required for `Arc<RefCell<u8>>` to implement `Send`
```

`Arc<T>`'nin `Send` olması için `T`'nin hem `Send` hem `Sync` olması gerekir.[^arc-send] Çözüm `Arc<Mutex<u8>>`'dir.

## Sık hatalar ve çözümleri

### Okuma kilidi eldeyken yazma kilidi istemek

"Oku, yoksa ekle" derken yazılan bu kod derlenir, ama `write` çağrısında takılabilir ya da panikleyebilir:

```rust,no_run
# use std::collections::HashMap;
# use std::sync::RwLock;
fn yoksa_ekle(ayarlar: &RwLock<HashMap<String, u32>>, anahtar: &str) {
    let okuma = ayarlar.read().expect("kilit zehirlenmemeli");
    if !okuma.contains_key(anahtar) {
        // okuma kilidi hâlâ elimizde
        ayarlar.write().expect("kilit zehirlenmemeli").insert(anahtar.to_string(), 0);
    }
}
# fn main() { yoksa_ekle(&RwLock::new(HashMap::new()), "x"); }
```

Yazma kilidi bütün okuyucuların çıkmasını bekler; bekleyen thread'in kendisi de o okuyuculardan biridir. `RwLock` okuma kilidini yazmaya yükseltmez. Belge bu durumda çağrının paniklemesine de izin verir.[^rwlock-panik] Aynı thread'de ikinci kez `read` çağırmak da güvenli değildir: Arada başka bir thread yazma kilidi bekliyorsa ikinci okuma da bekleyip kilitlenebilir.[^rwlock-panik]

Doğrusu, okuma koruyucusunu bırakıp yazma kilidini almak ve koşulu kilit altında *yeniden* denetlemektir; arada başka bir yazar eklemiş olabilir:

```rust
# use std::collections::HashMap;
# use std::sync::RwLock;
fn yoksa_ekle(ayarlar: &RwLock<HashMap<String, u32>>, anahtar: &str) {
    let var_mi = ayarlar.read().expect("kilit zehirlenmemeli").contains_key(anahtar);
    if !var_mi {
        let mut yazma = ayarlar.write().expect("kilit zehirlenmemeli");
        yazma.entry(anahtar.to_string()).or_insert(0); // hâlâ yoksa ekler
    }
}

fn main() {
    let ayarlar = RwLock::new(HashMap::from([(String::from("a"), 1)]));
    yoksa_ekle(&ayarlar, "a");
    yoksa_ekle(&ayarlar, "x");
    let ayarlar = ayarlar.into_inner().expect("kilit zehirlenmemeli");
    println!("a = {:?}, x = {:?}", ayarlar.get("a"), ayarlar.get("x"));
}
```

Çıktı:

```text
a = Some(1), x = Some(0)
```

`entry(..).or_insert(0)` denetimi ve eklemeyi birlikte yapar; araya giren bir yazarın değerini ezmez.

### Koşulu `if` ile denetlemek

```rust,no_run
# use std::sync::{Condvar, Mutex};
fn al(kuyruk: &Mutex<Vec<u32>>, zil: &Condvar) -> u32 {
    let mut ogeler = kuyruk.lock().expect("kilit zehirlenmemeli");
    if ogeler.is_empty() {
        ogeler = zil.wait(ogeler).expect("kilit zehirlenmemeli");
    }
    ogeler.pop().expect("uyandığımıza göre öğe var") // bu varsayım yanlış
}
# fn main() { let _ = al(&Mutex::new(vec![1]), &Condvar::new()); }
```

Thread sebepsiz uyanabilir ya da kilidi geri alana dek başka bir tüketici öğeyi kapabilir; o zaman `expect` panikler. `if` yerine döngü, yani `wait_while` gerekir:

```rust
# use std::sync::{Condvar, Mutex};
fn al(kuyruk: &Mutex<Vec<u32>>, zil: &Condvar) -> u32 {
    let ogeler = kuyruk.lock().expect("kilit zehirlenmemeli");
    let mut ogeler = zil
        .wait_while(ogeler, |o| o.is_empty())
        .expect("kilit zehirlenmemeli");
    ogeler.pop().expect("wait_while boş kuyrukla dönmez")
}
# fn main() { assert_eq!(al(&Mutex::new(vec![1]), &Condvar::new()), 1); }
```

### Kapanışta `notify_one` çağırmak

`SinirliKuyruk::kapat` içinde `notify_one` yazsaydık ve kuyrukta iki tüketici bekleseydi yalnızca biri uyanır, öteki sonsuza dek uyurdu. Kapanış bekleyen herkesi ilgilendirir; `notify_all` gerekir.[^notify] Aşağıdaki program iki bekleyen tüketicinin de çıktığını gösterir:

```rust
# use std::collections::VecDeque;
# use std::sync::{Condvar, Mutex};
# pub struct SinirliKuyruk<T> {
#     durum: Mutex<KuyrukDurumu<T>>,
#     dolu_degil: Condvar,
#     bos_degil: Condvar,
#     kapasite: usize,
# }
# struct KuyrukDurumu<T> { ogeler: VecDeque<T>, kapali: bool, en_yuksek_doluluk: usize }
# impl<T> SinirliKuyruk<T> {
#     pub fn yeni(kapasite: usize) -> Self {
#         SinirliKuyruk {
#             durum: Mutex::new(KuyrukDurumu {
#                 ogeler: VecDeque::new(), kapali: false, en_yuksek_doluluk: 0,
#             }),
#             dolu_degil: Condvar::new(),
#             bos_degil: Condvar::new(),
#             kapasite: kapasite.max(1),
#         }
#     }
#     pub fn koy(&self, oge: T) -> Result<(), T> {
#         let kilit = self.durum.lock().expect("kilit zehirlenmemeli");
#         let mut durum = self
#             .dolu_degil
#             .wait_while(kilit, |d| d.ogeler.len() >= self.kapasite && !d.kapali)
#             .expect("kilit zehirlenmemeli");
#         if durum.kapali { return Err(oge); }
#         durum.ogeler.push_back(oge);
#         durum.en_yuksek_doluluk = durum.en_yuksek_doluluk.max(durum.ogeler.len());
#         drop(durum);
#         self.bos_degil.notify_one();
#         Ok(())
#     }
#     pub fn al(&self) -> Option<T> {
#         let kilit = self.durum.lock().expect("kilit zehirlenmemeli");
#         let mut durum = self
#             .bos_degil
#             .wait_while(kilit, |d| d.ogeler.is_empty() && !d.kapali)
#             .expect("kilit zehirlenmemeli");
#         let oge = durum.ogeler.pop_front();
#         drop(durum);
#         if oge.is_some() { self.dolu_degil.notify_one(); }
#         oge
#     }
#     pub fn kapat(&self) {
#         self.durum.lock().expect("kilit zehirlenmemeli").kapali = true;
#         self.bos_degil.notify_all();
#         self.dolu_degil.notify_all();
#     }
#     pub fn en_yuksek_doluluk(&self) -> usize {
#         self.durum.lock().expect("kilit zehirlenmemeli").en_yuksek_doluluk
#     }
# }
# use std::thread;
fn main() {
    let kuyruk = SinirliKuyruk::<u32>::yeni(1);
    let cikanlar = thread::scope(|k| {
        let tuketiciler = [k.spawn(|| kuyruk.al()), k.spawn(|| kuyruk.al())];
        kuyruk.kapat(); // içeride notify_all var
        tuketiciler.map(|t| t.join().expect("tüketici paniklememeli"))
    });
    println!("{cikanlar:?}");
}
```

Çıktı:

```text
[None, None]
```

### Kilit eldeyken yabancı kod çağırmak

```rust,no_run
# use std::sync::Mutex;
struct Sayac {
    deger: Mutex<u32>,
}

impl Sayac {
    fn oku(&self) -> u32 {
        *self.deger.lock().expect("kilit zehirlenmemeli")
    }

    fn artir(&self, bildir: impl Fn(u32)) {
        let mut kilit = self.deger.lock().expect("kilit zehirlenmemeli");
        *kilit += 1;
        bildir(*kilit); // closure oku() çağırırsa aynı kilidi ister
    }
}

fn main() {
    let sayac = Sayac { deger: Mutex::new(0) };
    sayac.artir(|_| println!("şu an: {}", sayac.oku()));
}
```

`artir` kilidi tutarken closure'ı çağırır, closure `oku` ile aynı kilidi ister ve thread kendi kendini bekler. Değeri kopyalayıp kilidi bıraktıktan sonra çağırmak yeter:

```rust
# use std::sync::Mutex;
# struct Sayac { deger: Mutex<u32> }
# impl Sayac {
#     fn oku(&self) -> u32 { *self.deger.lock().expect("kilit zehirlenmemeli") }
    fn artir(&self, bildir: impl Fn(u32)) {
        let yeni = {
            let mut kilit = self.deger.lock().expect("kilit zehirlenmemeli");
            *kilit += 1;
            *kilit
        }; // kilit burada bırakıldı
        bildir(yeni);
    }
# }
# fn main() {
#     let sayac = Sayac { deger: Mutex::new(0) };
#     sayac.artir(|_| println!("şu an: {}", sayac.oku()));
# }
```

Çıktı:

```text
şu an: 1
```

### Başka veriyi `Relaxed` ile yayımlamak

```rust
# use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
# static VERI: AtomicU32 = AtomicU32::new(0);
# static HAZIR: AtomicBool = AtomicBool::new(false);
fn yayimla() {
    VERI.store(42, Ordering::Relaxed);
    HAZIR.store(true, Ordering::Relaxed); // sıra sözü yok
}
# fn main() { yayimla(); }
```

Derlenir ve tanımsız davranış da değildir, ama bayrağı `true` gören thread `VERI`'yi hâlâ 0 görebilir: `Relaxed` iki yazmanın başka thread'lere hangi sırayla görüneceği konusunda söz vermez.[^ordering] Bayrağı `Release` ile yazıp `Acquire` ile okuyun:

```rust
# use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
# static VERI: AtomicU32 = AtomicU32::new(0);
# static HAZIR: AtomicBool = AtomicBool::new(false);
fn yayimla() {
    VERI.store(42, Ordering::Relaxed);
    HAZIR.store(true, Ordering::Release);
}

fn bekle_ve_oku() -> u32 {
    while !HAZIR.load(Ordering::Acquire) {
        std::thread::yield_now();
    }
    VERI.load(Ordering::Relaxed)
}
# fn main() { yayimla(); assert_eq!(bekle_ve_oku(), 42); }
```

## Akılda kalması gerekenler

- Beklemek gerekiyorsa bloklayan bir araçla bekleyin: kanal, kilit, `Condvar`, `join`. Döngüde bakmak çekirdeği boşa yakar.
- `RwLock`'u yalnızca okuyucular gerçekten birbirini bekletiyorsa seçin. Kısa kritik bölgede `Mutex` çoğu zaman daha uygundur.
- Koruyucuyu yapının dışına vermeyin; kilidin ne kadar tutulacağı sizin kararınız olarak kalsın.
- `Condvar` ile koşulu döngüde denetleyin, kilit altında değiştirin, kapanışta herkesi uyandırın.
- Derleyici kilitlerden habersizdir; yanlış kilit sırasını o değil, sizin kuralınız önler.
- Atomikte emin değilseniz `SeqCst` kullanın. Zayıflatmayı ölçüm ve gerekçe ile yapın.

## Alıştırmalar

**1.** `SinirliKuyruk`'a `fn al_zaman_asimli(&self, sure: Duration) -> Option<T>` metodunu ekleyin: Süre içinde öğe gelmezse `None` dönsün. Boş kuyrukta 20 ms'lik çağrının en az 20 ms sürdüğünü ve `None` döndüğünü sınayın.

<details>
<summary>İpucu ve çözüm</summary>

`wait_timeout_while` koşulu ve süreyi birlikte denetler; dönen ikilinin ikinci öğesi `timed_out()` ile sürenin dolup dolmadığını söyler. Bu metot koşul düzelmişse süre dolmuş olsa bile `false` verir, bu yüzden son anda gelen öğe kaybolmaz.

```rust
# use std::collections::VecDeque;
# use std::sync::{Condvar, Mutex};
# pub struct SinirliKuyruk<T> {
#     durum: Mutex<KuyrukDurumu<T>>,
#     dolu_degil: Condvar,
#     bos_degil: Condvar,
#     kapasite: usize,
# }
# struct KuyrukDurumu<T> { ogeler: VecDeque<T>, kapali: bool, en_yuksek_doluluk: usize }
# impl<T> SinirliKuyruk<T> {
#     pub fn yeni(kapasite: usize) -> Self {
#         SinirliKuyruk {
#             durum: Mutex::new(KuyrukDurumu {
#                 ogeler: VecDeque::new(), kapali: false, en_yuksek_doluluk: 0,
#             }),
#             dolu_degil: Condvar::new(),
#             bos_degil: Condvar::new(),
#             kapasite: kapasite.max(1),
#         }
#     }
#     pub fn koy(&self, oge: T) -> Result<(), T> {
#         let kilit = self.durum.lock().expect("kilit zehirlenmemeli");
#         let mut durum = self
#             .dolu_degil
#             .wait_while(kilit, |d| d.ogeler.len() >= self.kapasite && !d.kapali)
#             .expect("kilit zehirlenmemeli");
#         if durum.kapali { return Err(oge); }
#         durum.ogeler.push_back(oge);
#         durum.en_yuksek_doluluk = durum.en_yuksek_doluluk.max(durum.ogeler.len());
#         drop(durum);
#         self.bos_degil.notify_one();
#         Ok(())
#     }
#     pub fn al(&self) -> Option<T> {
#         let kilit = self.durum.lock().expect("kilit zehirlenmemeli");
#         let mut durum = self
#             .bos_degil
#             .wait_while(kilit, |d| d.ogeler.is_empty() && !d.kapali)
#             .expect("kilit zehirlenmemeli");
#         let oge = durum.ogeler.pop_front();
#         drop(durum);
#         if oge.is_some() { self.dolu_degil.notify_one(); }
#         oge
#     }
#     pub fn kapat(&self) {
#         self.durum.lock().expect("kilit zehirlenmemeli").kapali = true;
#         self.bos_degil.notify_all();
#         self.dolu_degil.notify_all();
#     }
#     pub fn en_yuksek_doluluk(&self) -> usize {
#         self.durum.lock().expect("kilit zehirlenmemeli").en_yuksek_doluluk
#     }
# }
# use std::time::{Duration, Instant};
impl<T> SinirliKuyruk<T> {
    pub fn al_zaman_asimli(&self, sure: Duration) -> Option<T> {
        let kilit = self.durum.lock().expect("kilit zehirlenmemeli");
        let (mut durum, sonuc) = self
            .bos_degil
            .wait_timeout_while(kilit, sure, |d| d.ogeler.is_empty() && !d.kapali)
            .expect("kilit zehirlenmemeli");
        if sonuc.timed_out() {
            return None;
        }
        let oge = durum.ogeler.pop_front();
        drop(durum);
        if oge.is_some() {
            self.dolu_degil.notify_one();
        }
        oge
    }
}

fn main() {
    let kuyruk = SinirliKuyruk::yeni(1);
    let sure = Duration::from_millis(20);

    let baslangic = Instant::now();
    assert_eq!(kuyruk.al_zaman_asimli(sure), None);
    assert!(baslangic.elapsed() >= sure);

    kuyruk.koy(5).expect("kuyruk açık");
    assert_eq!(kuyruk.al_zaman_asimli(sure), Some(5));
}
```

`None` iki anlama gelebilir: Süre dolmuştur ya da kuyruk kapalı ve boştur. Çağıranın ikisini ayırması gerekiyorsa dönüş türünü bir `enum` ya da `Result` yapmak doğru olur.

</details>

**2.** `transfer`'i üç hesaplı bir işleme genelleyin: `fn dagit(kaynak: &Hesap, hedefler: [&Hesap; 2], tutar: i64) -> bool`. Fonksiyon kaynaktan iki hedefin her birine `tutar` aktarsın; kilit sırası yine hesap numarası olsun. Dört thread'le değişik yönlerde çağırıp toplamın korunduğunu ve programın bittiğini sınayın.

<details>
<summary>İpucu ve çözüm</summary>

Üç referansı `no`'ya göre sıralayın, sırayla kilitleyin, koruyucuları bir `Vec`'te tutun ve hesabın numarasından koruyucunun yerini bulun. Bakiye denetimi yine bütün kilitler alındıktan sonra yapılır:

```rust
# use std::sync::Mutex;
# pub struct Hesap { pub no: u32, bakiye: Mutex<i64> }
# impl Hesap {
#     pub fn yeni(no: u32, bakiye: i64) -> Self { Hesap { no, bakiye: Mutex::new(bakiye) } }
#     pub fn bakiye(&self) -> i64 { *self.bakiye.lock().expect("kilit zehirlenmemeli") }
# }
# use std::thread;
fn dagit(kaynak: &Hesap, hedefler: [&Hesap; 2], tutar: i64) -> bool {
    let mut sirali = [kaynak, hedefler[0], hedefler[1]];
    sirali.sort_by_key(|h| h.no); // kilit sırası: küçük numara önce
    let ayni_hesap_var = sirali.windows(2).any(|cift| cift[0].no == cift[1].no);
    if ayni_hesap_var || tutar <= 0 {
        return false;
    }
    let Some(toplam) = tutar.checked_mul(2) else {
        return false;
    };

    let mut kilitler: Vec<_> = sirali
        .iter()
        .map(|h| h.bakiye.lock().expect("kilit zehirlenmemeli"))
        .collect();
    let yer = |hesap: &Hesap| {
        sirali.iter().position(|h| h.no == hesap.no).expect("hesap listede olmalı")
    };

    if *kilitler[yer(kaynak)] < toplam {
        return false;
    }
    *kilitler[yer(kaynak)] -= toplam;
    for hedef in hedefler {
        *kilitler[yer(hedef)] += tutar;
    }
    true
}

fn main() {
    let h = [Hesap::yeni(1, 1_000), Hesap::yeni(2, 1_000), Hesap::yeni(3, 1_000)];
    thread::scope(|k| {
        for i in 0..4 {
            let h = &h;
            k.spawn(move || {
                for tur in 0..300 {
                    let bas = (i + tur) % 3; // her turda yön değişir
                    dagit(&h[bas], [&h[(bas + 1) % 3], &h[(bas + 2) % 3]], 1);
                }
            });
        }
    });
    let toplam: i64 = h.iter().map(Hesap::bakiye).sum();
    assert_eq!(toplam, 3_000);
}
```

`collect` iterator'ı baştan sona işlediği için kilitler `sirali` dizisinin sırasıyla alınır.

</details>

## Özet

- Bloklanan thread çekirdeği bırakır; meşgul bekleyen thread bırakmaz.
- `RwLock` çok okuyucuya ya da tek yazara izin verir. Okuma kilidi yazmaya yükseltilemez, yazma kilidi okumaya indirilebilir.
- `Condvar` kilidi bırakıp uyumayı atomik yapar; sınırlı kuyruk bu iskeletle üreticiye geri basınç uygular.
- Kilitlenme tasarımla önlenir: Bütün kod kilitleri aynı sırayla alır.
- Atomikler tek adımlık işler içindir; `Ordering` başka verinin görünürlüğünü belirler.
- `Mutex<T>` için `T: Send` yeter; `RwLock<T>`'nin paylaşılabilmesi için `T: Sync` de gerekir.

Sıradaki sayfa [29.2 Çalışma Zamanı: Görev Kuyruğu ve Geri Basınç](29-calisma-zamani.md): Aynı "beklerken uyu, hazır olunca uyandır" düşüncesini bu kez thread'ler yerine görevlerle kuracağız.

## Kaynaklar

[^sira]: The Rust Programming Language, [16.1 Using Threads to Run Code Simultaneously](https://doc.rust-lang.org/book/ch16-01-threads.html).
[^sleep]: std, [`thread::sleep`](https://doc.rust-lang.org/std/thread/fn.sleep.html).
[^yield]: std, [`thread::yield_now`](https://doc.rust-lang.org/std/thread/fn.yield_now.html).
[^rwlock]: std, [`RwLock`](https://doc.rust-lang.org/std/sync/struct.RwLock.html); *Rust for Rustaceans*, 10. bölüm (Concurrency).
[^zehir]: std, [`RwLock`: Poisoning](https://doc.rust-lang.org/std/sync/struct.RwLock.html#poisoning).
[^e0515]: Hata dizini, [`E0515`](https://doc.rust-lang.org/error_codes/E0515.html); The Rust Reference, [Drop scopes](https://doc.rust-lang.org/reference/destructors.html#drop-scopes).
[^okuyucu-sayaci]: std 1.99.0 kaynak kodu (Linux), [`rwlock/futex.rs`, `MAX_READERS`](https://github.com/rust-lang/rust/blob/1.99.0/library/std/src/sys/sync/rwlock/futex.rs#L23).
[^downgrade]: std, [`RwLockWriteGuard::downgrade`](https://doc.rust-lang.org/std/sync/struct.RwLockWriteGuard.html#method.downgrade).
[^wait]: std, [`Condvar::wait`](https://doc.rust-lang.org/std/sync/struct.Condvar.html#method.wait).
[^notify]: std, [`Condvar::notify_one`](https://doc.rust-lang.org/std/sync/struct.Condvar.html#method.notify_one) ve [`Condvar::notify_all`](https://doc.rust-lang.org/std/sync/struct.Condvar.html#method.notify_all).
[^wait-while]: std, [`Condvar::wait_while`](https://doc.rust-lang.org/std/sync/struct.Condvar.html#method.wait_while).
[^sync-channel]: std, [`mpsc::sync_channel`](https://doc.rust-lang.org/std/sync/mpsc/fn.sync_channel.html) ve [`SyncSender::try_send`](https://doc.rust-lang.org/std/sync/mpsc/struct.SyncSender.html#method.try_send).
[^sureli]: std, [`Condvar::wait_timeout_while`](https://doc.rust-lang.org/std/sync/struct.Condvar.html#method.wait_timeout_while).
[^kilitlenme]: The Rust Reference, [Behavior not considered unsafe](https://doc.rust-lang.org/reference/behavior-not-considered-unsafe.html); The Rust Programming Language, [16.3 Shared-State Concurrency](https://doc.rust-lang.org/book/ch16-03-shared-state.html#comparing-refcelltrct-and-mutextarct).
[^bariyer]: std, [`Barrier`](https://doc.rust-lang.org/std/sync/struct.Barrier.html) ve [`Mutex::try_lock`](https://doc.rust-lang.org/std/sync/struct.Mutex.html#method.try_lock).
[^ayni-kilit]: std, [`Mutex::lock`](https://doc.rust-lang.org/std/sync/struct.Mutex.html#method.lock).
[^atomik]: std, [`std::sync::atomic`](https://doc.rust-lang.org/std/sync/atomic/index.html); *Rust Atomics and Locks*, 2. bölüm (Atomics).
[^e0499]: Hata dizini, [`E0499`](https://doc.rust-lang.org/error_codes/E0499.html).
[^cas]: std, [`compare_exchange`](https://doc.rust-lang.org/std/sync/atomic/struct.Atomic.html#method.compare_exchange).
[^ordering]: std, [`Ordering`](https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html); *Rust Atomics and Locks*, 3. bölüm (Memory Ordering).
[^nomicon-atomik]: The Rustonomicon, [Atomics](https://doc.rust-lang.org/nomicon/atomics.html#hardware-reordering).
[^auto]: The Rust Reference, [Auto traits](https://doc.rust-lang.org/reference/special-types-and-traits.html#auto-traits); The Rust Programming Language, [16.4 Extensible Concurrency with Send and Sync](https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html).
[^mutex-sync]: std 1.99.0 kaynak kodu, [`mutex.rs`, `Sync for Mutex`](https://github.com/rust-lang/rust/blob/1.99.0/library/std/src/sync/poison/mutex.rs#L257) ve [`rwlock.rs`, `Sync for RwLock`](https://github.com/rust-lang/rust/blob/1.99.0/library/std/src/sync/poison/rwlock.rs#L107).
[^guard-send]: std 1.99.0 kaynak kodu, [`mutex.rs`, `!Send for MutexGuard`](https://github.com/rust-lang/rust/blob/1.99.0/library/std/src/sync/poison/mutex.rs#L289).
[^arc-send]: std 1.99.0 kaynak kodu, [`sync.rs`, `Send for Arc`](https://github.com/rust-lang/rust/blob/1.99.0/library/alloc/src/sync.rs#L284).
[^rwlock-panik]: std, [`RwLock::read`](https://doc.rust-lang.org/std/sync/struct.RwLock.html#method.read) ve [`RwLock::write`](https://doc.rust-lang.org/std/sync/struct.RwLock.html#method.write).
