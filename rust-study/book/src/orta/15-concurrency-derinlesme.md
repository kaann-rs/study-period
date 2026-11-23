# 15.1 Eşzamanlılık: Derinleşme

[Önceki sayfada](15-concurrency.md) üç yapı taşını ayrı ayrı gördük: `spawn` ile `join`, kanal ve `Arc<Mutex<T>>`. Bu sayfada onları gerçek işlerde yinelenen kalıplara dönüştüreceğiz:

- `thread::scope` ile yerel veriyi `Arc`'sız ve klonsuz ödünç vermek,
- iş kuyruğu ve sonuç kanalıyla işçi havuzu kurmak,
- paylaşılan durumda kilidi kısa tutmak,
- tek bir sayı için kilit yerine atomik kullanmak,
- en sık görülen derleme hataları ile derleyicinin yakalayamadığı sorunlar.

Sayfanın ana fikri şudur: İşi thread'lere bölmenin birkaç hazır kalıbı vardır; derleyici veri yarışını yakalar, ama thread'lerin birbirini sonsuza dek beklemesini yakalayamaz.

> [!NOTE]
> Bu sayfanın notları `learn/concurrency/src/derinlesme.rs` dosyasındadır. Depo kökünden `cargo run -p concurrency` ile çalıştırabilir, `cargo test -p concurrency` ile testlerini koşturabilirsiniz. `RwLock`, `Condvar`, kilit sıralaması ve async tarafı [29. bölümde](../ileri/29-ileri-eszamanlilik-async.md) işlenir.

Thread'lerin çalışma sırası belirsizdir. Bu sayfadaki örnekler sonucu sıradan bağımsız olacak biçimde (toplam, sıralanmış liste) üretir; bu yüzden çıktıları her çalıştırmada aynıdır.

<details>
<summary>Bu sayfada yeni geçen terimler</summary>

- **`thread::scope`:** "Bu blok bitmeden işini bitirecek" thread'ler başlatan fonksiyon.
- **`'static` sınırı:** "İçinde daha kısa ömürlü bir ödünç yoktur" koşulu. Yerel veriyi ödünç alan bir closure bunu sağlamaz.
- **`chunks`, `chunks_mut`:** Bir dilimi okumak ya da yazmak için üst üste binmeyen parçalara bölen metotlar.
- **tembel (lazy) iterator:** Sonucu isteyen olmadıkça hiçbir iş yapmayan işlem zinciri.
- **işçi havuzu (worker pool):** Sayısı sabit, sıradaki işi kuyruktan alan thread grubu.
- **kuyruk (queue):** İşlerin geliş sırasıyla beklediği yapı.
- **kritik bölge (critical section):** Kilit eldeyken çalışan kod parçası.
- **kilit çekişmesi (lock contention):** Thread'lerin zamanını kilit sırası bekleyerek geçirmesi.
- **`AtomicU64`, `fetch_add`:** Kilitsiz paylaşılabilen sayı ve onu tek hamlede artıran işlem.
- **`Ordering` (`Relaxed`, `SeqCst`):** Atomik işlemin öteki bellek işlemlerine göre ne kadar sıkı sıralanacağı.
- **`try_lock`:** Kilidi beklemeden istemek; alınamazsa hemen hata döner.
- **`into_inner`:** Kilidi ya da hücreyi söküp içindeki veriyi sahiplenerek almak.
- **değişmez (invariant):** Her zaman doğru kalması gereken kural; örneğin "sayaç, listenin uzunluğuna eşittir".
- **mantıksal yarış:** Adımlar tek tek güvenli olsa da aralarında durumun değişmesi.
- **`BTreeMap`:** Anahtarları sıralı tutan sözlük.
- **G/Ç (I/O):** Dosya, ağ ve ekran gibi dış dünyayla yapılan okuma ve yazma.

</details>

## Göndermek mi, paylaşmak mı?

Thread'ler arasında veri iki biçimde dolaşır ve seçim programın yapısını belirler. İki aşçı ya tabağı elden ele geçirir ya da yerinde duran aynı tencereye sırayla uzanır. Birincisinde tabak kimdeyse yalnızca o dokunur; ikincisinde tencere tektir, kaşık sırayla alınır. Kodda ilki `tx.send(tabak)`, ikincisi `*tencere.lock()` ile başlayan satırdır.

| | Kanal (göndermek) | `Arc<Mutex<T>>` (paylaşmak) |
| --- | --- | --- |
| Veri nerede? | Bir thread'den ötekine taşınır, aynı anda tek sahibi vardır | Yerinde durur, thread'ler sırayla erişir |
| Uygun olduğu iş | Üret, işle, topla türü iş hatları; görev dağıtımı; olay akışı | Herkesin okuyup güncellediği tablo: önbellek, sayaç, bağlantı havuzu |
| Hataları kim yakalar? | Çoğunu derleyici: gönderilen değer bir daha kullanılamaz | Kilidin doğru kullanımı programcıya kalır |
| Tipik tehlike | Kapatılması unutulan gönderen | Kilitlenme, uzun tutulan kilit |

Akan işler göndermeye, ortak kayıtlar paylaşmaya uygundur. Emin değilseniz kanalla başlayın; sahiplik kimin neye dokunabileceğini baştan söylediği için hataların daha çoğunu derleyici yakalar.[^kanal-kitap] Kilit kullanılmadığından kilitlenme olasılığı da düşer. Olasılık sıfırlanmaz: İki thread karşılıklı olarak birbirinden ileti beklerse kanallarla da kilitlenilir.

## `thread::scope`: Yerel veriyi ödünç vermek

`thread::spawn`'a verilen closure yerel veriyi ödünç alamaz, çünkü thread onu başlatan fonksiyondan uzun yaşayabilir. Önceki sayfadaki E0373 hatasının iki klasik çözümü vardı: `move` ile sahipliği vermek ya da `Arc` ile paylaşmak. Oysa çoğu zaman istenen daha basittir: "Bu dilimi birlikte işleyin, bitince ben devam edeyim."

Dört yardımcıyı "ben mutfaktan çıkmadan işinizi bitireceksiniz" diyerek çağırdığınızı düşünün. Sizden önce bitirecekleri kesin olduğundan malzemeyi onlara vermeniz gerekmez; malzeme sizin tezgâhınızda dururken kullanırlar. `thread::scope` bu sözü verir: Kapsamda başlatılan bütün thread'ler, `scope` çağrısı dönmeden önce kendiliğinden beklenir. Derleyici de bu güvenceye dayanarak `'static` koşulunu aramaz.[^scope]

```rust
use std::thread;

fn main() {
    let sayilar = vec![1, 2, 3, 4];

    let (ilk, adet) = thread::scope(|k| {
        let ilk = k.spawn(|| sayilar[0]); // ödünç aldı: move yok, Arc yok
        let adet = k.spawn(|| sayilar.len());
        (
            ilk.join().expect("yardımcı panikledi"),
            adet.join().expect("yardımcı panikledi"),
        )
    }); // burada ikisi de bitmiştir

    println!("ilk = {ilk}, adet = {adet}");
    println!("sayilar hâlâ bizde: {sayilar:?}");
}
```

Çıktı:

```text
ilk = 1, adet = 4
sayilar hâlâ bizde: [1, 2, 3, 4]
```

Aynı closure'ları `thread::spawn`'a verseydik E0373 alırdık. Fark imzalardadır: `thread::spawn` closure'dan `'static`, kapsamın `spawn` metodu ise yalnızca `'scope` ister; `'scope`, kapsamın kendi ömrüdür.[^scope-kod] Güvencenin bir bedeli vardır: `join` edilmeyen bir thread paniklerse `scope` çağrısının kendisi de "a scoped thread panicked" iletisiyle panikler.[^scope]

### Dilimi paralel toplamak

Bir dilimi parçalara bölüp her parçayı ayrı thread'de toplayalım:

```rust
use std::thread;

pub fn paralel_topla(veri: &[u64], parca_sayisi: usize) -> u64 {
    let parca_boyu = veri.len().div_ceil(parca_sayisi.max(1)).max(1);

    thread::scope(|kapsam| {
        let tutamaclar: Vec<_> = veri
            .chunks(parca_boyu) // ayrık alt dilimler
            .map(|parca| kapsam.spawn(move || parca.iter().sum::<u64>()))
            .collect(); // hepsini başlat

        tutamaclar
            .into_iter()
            .map(|t| t.join().expect("işçi panikledi"))
            .sum() // sonra hepsini bekle
    })
}

fn main() {
    let sayilar: Vec<u64> = (1..=1000).collect();
    println!("paralel toplam (4 parça) = {}", paralel_topla(&sayilar, 4));

    // parça sayısı veriden çok da olsa, dilim boş da olsa sonuç doğrudur
    let yuz: Vec<u64> = (1..=100).collect();
    for parca in [1, 3, 7, 100, 500] {
        assert_eq!(paralel_topla(&yuz, parca), 5050);
    }
    assert_eq!(paralel_topla(&[], 4), 0);
}
```

Çıktı:

```text
paralel toplam (4 parça) = 500500
```

Kodda üç ayrıntı vardır. `div_ceil` bölmeyi yukarı yuvarlar, böylece son parça dışarıda kalmaz. İki `max(1)`, sıfıra bölmeyi ve `chunks(0)` çağrısını önler; ikisi de panikletirdi.[^chunks] Closure'daki `move` ise yalnızca `parca` referansını taşır; veri yerinde kalır.

En önemli ayrıntı `collect`'tir. Iterator'lar tembeldir; sonucu isteyen olmadıkça çalışmaz. Araya `collect` konmadan `.map(spawn).map(join)` yazılsaydı her thread başlatılır başlatılmaz beklenirdi: Kod paralel görünür, ama seri çalışırdı. Doğru sıra, önce hepsini başlatmak, sonra hepsini beklemektir.

### Ayrık parçalara yazmak

`chunks_mut` her thread'e, ötekilerle çakışmayan bir `&mut` dilim verir. Tezgâhı bölüp her yardımcıya kendi bölümünü vermeye benzer; kimse kimsenin bölümüne uzanamaz:

```rust
use std::thread;

pub fn paralel_karesini_al(veri: &mut [u64], parca_sayisi: usize) {
    let parca_boyu = veri.len().div_ceil(parca_sayisi.max(1)).max(1);

    thread::scope(|kapsam| {
        for parca in veri.chunks_mut(parca_boyu) {
            kapsam.spawn(move || {
                for x in parca {
                    *x *= *x;
                }
            });
        }
    });
}

fn main() {
    let mut sayilar: Vec<u64> = (1..=1000).collect();
    paralel_karesini_al(&mut sayilar, 4);
    println!("kareler: ilk üç = {:?}, son = {}", &sayilar[..3], sayilar[999]);
}
```

Çıktı:

```text
kareler: ilk üç = [1, 4, 9], son = 1000000
```

"Aynı anda iki `&mut` olamaz" kuralı bozulmaz, çünkü `chunks_mut` ayrık dilimler döndürdüğünü imzası ve uygulamasıyla güvence altına alır.[^chunks] Kilit de gerekmez: Hiçbir öğeye iki thread dokunmaz.

## İşçi havuzu: İki kanallı tasarım

Senaryo şöyle: Gelen kayıtları (burada metin satırlarını) sabit sayıda işçiye dağıtıp sonuçları toplayacağız. Her iş için yeni thread açmak pahalıdır; yük arttıkça thread sayısı da sınırsız büyür. Havuz, aynı anda çalışan iş sayısını sınırlar. Web sunucuları ve veri işleme hatları bu kalıba dayanır; resmî kitabın son projesindeki çok thread'li sunucu da bir işçi havuzudur.[^havuz-kitap]

Lokantada her sipariş için yeni aşçı işe alınmaz. Üç aşçı vardır, siparişler bir fiş rayına dizilir. Boşalan aşçı raydan sıradaki fişi çeker, yemeği yapar, tabağı servis penceresine koyar. Koddaki karşılıkları şöyledir:

- **İş kanalı** fiş rayıdır: ana thread'den işçilere. `mpsc`'de alıcı tektir; işçilerin paylaşabilmesi için `Arc<Mutex<Receiver<Is>>>` içine konur. Buradaki `Mutex` veriyi değil, "sıradaki işi kim alacak?" sorusunu sıraya sokar.
- **Sonuç kanalı** servis penceresidir: işçilerden ana thread'e. Gönderen klonlanabildiği için her işçi kendi kopyasını alır.

```rust
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Is {
    pub id: u32,
    pub metin: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Sonuc {
    pub id: u32,
    pub kelime_sayisi: usize,
}

pub fn havuzda_isle(isler: Vec<Is>, isci_sayisi: usize) -> Vec<Sonuc> {
    let (is_tx, is_rx) = mpsc::channel::<Is>();
    let (sonuc_tx, sonuc_rx) = mpsc::channel::<Sonuc>();
    let is_rx = Arc::new(Mutex::new(is_rx));

    let mut isciler = Vec::new();
    for _ in 0..isci_sayisi.max(1) {
        let is_rx = Arc::clone(&is_rx); // alıcının paylaşılan sahibi
        let sonuc_tx = sonuc_tx.clone(); // işçinin kendi göndereni
        isciler.push(thread::spawn(move || {
            loop {
                // koruyucu bu satırın sonunda düşer
                let alinan = is_rx.lock().expect("kuyruk kilidi zehirli").recv();
                let Ok(is) = alinan else { break }; // kuyruk kapandı
                let kelime_sayisi = is.metin.split_whitespace().count();
                let sonuc = Sonuc { id: is.id, kelime_sayisi };
                if sonuc_tx.send(sonuc).is_err() {
                    break; // sonucu bekleyen kalmadı
                }
            }
        }));
    }
    drop(sonuc_tx); // yoksa aşağıdaki toplama hiç bitmez

    for is in isler {
        is_tx.send(is).expect("bütün işçiler kapanmış");
    }
    drop(is_tx); // başka iş gelmeyecek

    let mut sonuclar: Vec<Sonuc> = sonuc_rx.iter().collect();
    for isci in isciler {
        isci.join().expect("işçi panikledi");
    }
    sonuclar.sort_by_key(|s| s.id); // varış sırası belirsizdir
    sonuclar
}
```

Havuzu beş satırlık bir girdiyle çalıştıralım:

```rust
# use std::sync::mpsc;
# use std::sync::{Arc, Mutex};
# use std::thread;
# pub struct Is {
#     pub id: u32,
#     pub metin: String,
# }
# #[derive(Debug, PartialEq, Eq)]
# pub struct Sonuc {
#     pub id: u32,
#     pub kelime_sayisi: usize,
# }
# pub fn havuzda_isle(isler: Vec<Is>, isci_sayisi: usize) -> Vec<Sonuc> {
#     let (is_tx, is_rx) = mpsc::channel::<Is>();
#     let (sonuc_tx, sonuc_rx) = mpsc::channel::<Sonuc>();
#     let is_rx = Arc::new(Mutex::new(is_rx));
#     let mut isciler = Vec::new();
#     for _ in 0..isci_sayisi.max(1) {
#         let is_rx = Arc::clone(&is_rx);
#         let sonuc_tx = sonuc_tx.clone();
#         isciler.push(thread::spawn(move || {
#             loop {
#                 let alinan = is_rx.lock().expect("kuyruk kilidi zehirli").recv();
#                 let Ok(is) = alinan else { break };
#                 let kelime_sayisi = is.metin.split_whitespace().count();
#                 if sonuc_tx.send(Sonuc { id: is.id, kelime_sayisi }).is_err() {
#                     break;
#                 }
#             }
#         }));
#     }
#     drop(sonuc_tx);
#     for is in isler {
#         is_tx.send(is).expect("bütün işçiler kapanmış");
#     }
#     drop(is_tx);
#     let mut sonuclar: Vec<Sonuc> = sonuc_rx.iter().collect();
#     for isci in isciler {
#         isci.join().expect("işçi panikledi");
#     }
#     sonuclar.sort_by_key(|s| s.id);
#     sonuclar
# }
fn main() {
    let satirlar = [
        "bir iki üç",
        "tek",
        "dört kelimelik bir satır",
        "",
        "son satır",
    ];
    let isler: Vec<Is> = (1..)
        .zip(satirlar)
        .map(|(id, metin)| Is { id, metin: metin.to_string() })
        .collect();

    let sonuclar = havuzda_isle(isler, 3);
    let ozet: Vec<String> = sonuclar
        .iter()
        .map(|s| format!("#{}:{}", s.id, s.kelime_sayisi))
        .collect();
    println!("işçi havuzu sonuçları = {}", ozet.join(" "));
}
```

Çıktı:

```text
işçi havuzu sonuçları = #1:3 #2:1 #3:4 #4:0 #5:2
```

Havuzun düzgün kapanması sahipliğe dayanır. `drop(is_tx)` kuyruğu kapatır: Ray boşaldığında işçilerin `recv` çağrısı `Err` döner ve döngüleri biter. İşçiler bitince ellerindeki `sonuc_tx` kopyaları düşer; özgün `sonuc_tx` de daha önce düşürüldüğü için `sonuc_rx.iter()` sona erer.

> [!NOTE]
> Standart kütüphanede çok tüketicili bir kanal (`std::sync::mpmc`) vardır, ama 1.99.0'da kararlı değildir.[^mpmc] Kararlı Rust'ta alıcıyı `Mutex` ile paylaşmak ya da bir crate kullanmak gerekir. Kuyruk boşken bir işçi kilidi tutarak `recv` içinde, ötekiler kilit kapısında bekler; yapacak iş olmadığından bunun zararı yoktur.

### `while let` tuzağı

İşçi döngüsünde `recv` ile işin işlenmesi bilerek ayrı satırlardadır. Kodu kısaltmak isterken şu biçimi yazmak kolaydır:

```rust,no_run
# use std::sync::{Arc, Mutex, mpsc};
# use std::thread;
# use std::time::Duration;
# fn pisir(_fis: u32) { thread::sleep(Duration::from_millis(100)); }
# fn main() {
# let (_is_tx, is_rx) = mpsc::channel::<u32>();
# let ray = Arc::new(Mutex::new(is_rx));
# thread::spawn(move || {
while let Ok(fis) = ray.lock().expect("kuyruk kilidi zehirli").recv() {
    pisir(fis); // ray hâlâ kilitli
}
# });
# }
```

Kod derlenir, ama havuz tek işçiye iner. `lock()` geçici bir koruyucu üretir. `while let`'in koşulundaki geçici değerler döngü gövdesi bitene dek yaşar; ray, pişirme boyunca kilitli kalır.[^gecici] `let` deyimindeki geçici değer ise deyimin sonunda düşer. Dört işçi ve 100 ms süren dört işle yapılan bir ölçüm farkı gösterir.

Örnek bir çalıştırma:

```text
while let: 401 ms
ayrı let: 100 ms
```

Resmî kitap da sunucu projesinde aynı uyarıyı yapar.[^havuz-kitap] Edition 2024'te `if let` koşulundaki geçici değerler `else` dalına girmeden düşer, ama `if let` gövdesi boyunca yine yaşar; `match` ve `while let` için kural değişmemiştir.[^if-let]

## Kilidi kısa tutmak

Senaryo: Birkaç belgeyi birlikte tarayıp ortak bir kelime sıklığı tablosu çıkaracağız. İlk akla gelen çözüm, her kelimede kilidi alıp tabloyu güncellemektir. Bu kod derlenir, doğru sonuç verir, ama yavaştır: Thread'ler zamanlarının çoğunu kilit beklerken geçirir. `Mutex` doğruluğu sağlar, paralelliği değil.

Stok defterinin kilerde durduğunu ve kilerin tek anahtarı olduğunu düşünün. Saydığınız her patates için kilere gidip deftere bir çizgi atarsanız bütün aşçılar gününü kiler kapısında geçirir. Bunun yerine herkes kendi kâğıdına sayar, işi bitince kilere bir kez girip toplamını deftere yazar:

```rust
use std::sync::Mutex;
use std::thread;

fn main() {
    let ortak = Mutex::new(0);

    thread::scope(|kapsam| {
        for _ in 0..4 {
            kapsam.spawn(|| {
                let mut yerel = 0;
                for _ in 0..1000 {
                    yerel += 1; // kilitsiz
                }
                *ortak.lock().expect("kilit zehirlenmiş") += yerel; // tek ve kısa kilit
            });
        }
    });

    println!("{}", ortak.into_inner().expect("kilit zehirlenmiş"));
}
```

Çıktı:

```text
4000
```

Artırmayı kilidin altında yapsaydık sonuç yine 4000 olurdu, ama kilit 4 yerine 4000 kez alınırdı. Aynı kalıbı kelime sayımına uygulayalım: Her thread kendi tablosunu kilitsiz doldurur, sonunda bir kez birleştirir.

```rust
use std::collections::BTreeMap;
use std::sync::Mutex;
use std::thread;

pub fn kelime_sikligi(belgeler: &[&str]) -> BTreeMap<String, u32> {
    let ortak: Mutex<BTreeMap<String, u32>> = Mutex::new(BTreeMap::new());

    thread::scope(|kapsam| {
        for belge in belgeler {
            let ortak = &ortak; // Mutex taşınmaz, referansı verilir
            kapsam.spawn(move || {
                let mut yerel: BTreeMap<String, u32> = BTreeMap::new();
                for kelime in belge.split_whitespace() {
                    *yerel.entry(kelime.to_lowercase()).or_insert(0) += 1;
                }
                // kritik bölge: yalnızca birleştirme
                let mut tablo = ortak.lock().expect("kilit zehirlenmiş");
                for (kelime, adet) in yerel {
                    *tablo.entry(kelime).or_insert(0) += adet;
                }
            }); // `tablo` düşer, kilit bırakılır
        }
    });

    ortak.into_inner().expect("kilit zehirlenmiş")
}

fn main() {
    let belgeler = ["rust hızlı ve güvenli", "Rust korkusuz", "güvenli ve hızlı"];
    println!("kelime sıklığı = {:?}", kelime_sikligi(&belgeler));

    let tablo = kelime_sikligi(&["a b", "B c", "c C c"]);
    println!("{tablo:?}");
}
```

Çıktı:

```text
kelime sıklığı = {"güvenli": 2, "hızlı": 2, "korkusuz": 1, "rust": 2, "ve": 2}
{"a": 1, "b": 2, "c": 4}
```

`BTreeMap` anahtarları sıralı tuttuğu için çıktı belirlenimcidir. Burada `Arc` olmadığına dikkat edin: `thread::scope` sayesinde `&Mutex` ödünç vermek yeter. `Arc` "kaç sahip var?" sorusunu çözer; sahiplik sorunu olmayınca `Arc` da gerekmez. Bütün thread'ler bittikten sonra `Mutex`'in tek sahibi biziz; `into_inner` kilitlemeden içindekini verir.[^into-inner]

## Atomikler: Tek bir sayı için kilide gerek yok

Mutfak kapısında, basıldıkça bir artan mekanik bir sayaç olduğunu düşünün. Basmak tek hamledir: İki kişi aynı anda bassa da sayı iki artar, hiçbir basış kaybolmaz; kapıyı kilitlemek gerekmez. Paylaşılan şey yalnızca bir sayı ya da bayraksa atomik türler bu işi görür. `fetch_add`; okuma, artırma ve yazmayı bölünemez tek bir işlemde yapar.[^atomik]

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;

pub fn paralel_say(thread_sayisi: u64, thread_basina: u64) -> u64 {
    let sayac = AtomicU64::new(0);

    thread::scope(|kapsam| {
        for _ in 0..thread_sayisi {
            kapsam.spawn(|| {
                for _ in 0..thread_basina {
                    sayac.fetch_add(1, Ordering::Relaxed);
                }
            });
        }
    });

    sayac.into_inner()
}

fn main() {
    println!("atomik sayaç (8 thread × 10_000) = {}", paralel_say(8, 10_000));
}
```

Çıktı:

```text
atomik sayaç (8 thread × 10_000) = 80000
```

`sayac` değişkeni `mut` değildir; `fetch_add` paylaşımlı referans alır. Kilit olmadığından zehirlenme de yoktur. Standart kütüphane, bir platformda bulunan atomik türlerin kilitsiz (lock-free) olduğunu güvence altına alır. Bu, hiç bekleme olmayacağı anlamına gelmez: İşlemlerin beklemesiz (wait-free) olması güvence altında değildir. Ayrıca her platformda her boyut bulunmaz; 32 bit işaretçili bazı platformlarda `AtomicU64` yoktur.[^atomik-tasinabilir]

`Ordering` parametresi, işlemin öteki bellek erişimlerine göre nasıl sıralanacağını söyler. Bağımsız bir sayaçta `Relaxed` yeter: İşlemin kendisi atomik olsun, öteki verilerle sıralama ilişkisi kurulmasın. Bir bayrağa bakıp başka veriye erişme kararı veriyorsanız daha güçlü bir sıralama gerekir; emin değilseniz `SeqCst` seçin.[^sira] Ayrıntılar [29. bölümdedir](../ileri/29-ileri-eszamanlilik-async.md); *Rust Atomics and Locks* kitabının 2. ve 3. bölümleri de bu konuyu baştan sona anlatır.

> [!WARNING]
> Atomikler tek bir değeri korur. "Sayaç, listenin uzunluğuna eşit kalsın" gibi birden çok değeri kapsayan bir değişmez varsa yine `Mutex` gerekir.

## Derleyicinin yakaladığı hatalar

Derleyici, mutfak kapısındaki denetçi gibidir: İki aşçının aynı tencereye sırasız uzanmasına yol açacak tarifi içeri almaz. Aşağıdaki beş hata aynı sorunun, "izin verilseydi veri yarışı olurdu" sorununun farklı yüzleridir.

### Yerel veriyi `spawn`'a ödünç vermek (E0373)

```rust,compile_fail,E0373
use std::thread;

fn main() {
    let veri = vec![1, 2, 3];
    thread::spawn(|| println!("{veri:?}"));
}
```

Derleyicinin yanıtı:

```text
error[E0373]: closure may outlive the current function, but it borrows `veri`, which is owned by the current function
```

Thread fonksiyon bitmeden bitecekse `thread::scope`, bitmeyebilirse `move` ya da `Arc` kullanın:[^scope]

```rust
use std::thread;

fn main() {
    let veri = vec![1, 2, 3];
    thread::scope(|kapsam| {
        kapsam.spawn(|| println!("{veri:?}"));
    });
}
```

Çıktı:

```text
[1, 2, 3]
```

### `Arc`'ı klonlamadan taşımak (E0382)

```rust,compile_fail,E0382
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let m = Arc::new(Mutex::new(0));

    let t = thread::spawn(move || {
        *m.lock().expect("kilit zehirlenmiş") += 1;
    });
    t.join().expect("thread panikledi");

    println!("{m:?}");
}
```

Derleyicinin yanıtı:

```text
error[E0382]: the type `Arc` does not implement `Copy`
   |
 5 |     let m = Arc::new(Mutex::new(0));
   |         - this move could be avoided by cloning the original `Arc`, which is inexpensive
 6 |
 7 |     let t = thread::spawn(move || {
   |                           ------- value moved into closure here
 8 |         *m.lock().expect("kilit zehirlenmiş") += 1;
   |          - variable moved due to use in closure
...
12 |     println!("{m:?}");
   |                ^ value borrowed here after move
   |
   = note: consider using `Arc::clone`
```

`Arc` de bir değerdir ve taşınır. Thread'e verilecek klonu taşımadan önce üretin:[^arc-klon]

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let m = Arc::new(Mutex::new(0));

    let m2 = Arc::clone(&m);
    let t = thread::spawn(move || {
        *m2.lock().expect("kilit zehirlenmiş") += 1;
    });
    t.join().expect("thread panikledi");

    println!("{m:?}");
}
```

Çıktı:

```text
Mutex { data: 1, poisoned: false, .. }
```

### `Arc`'ın içini kilitsiz değiştirmek (E0596)

```rust,compile_fail,E0596
use std::sync::Arc;
use std::thread;

fn main() {
    let v = Arc::new(vec![1]);

    thread::spawn(move || v.push(2));
}
```

Derleyicinin yanıtı:

```text
error[E0596]: cannot borrow data in an `Arc` as mutable
  |
7 |     thread::spawn(move || v.push(2));
  |                           ^ cannot borrow as mutable
  |
  = help: trait `DerefMut` is required to modify through a dereference, but it is not implemented for `Arc<Vec<i32>>`
```

`Arc` yalnızca paylaşımlı erişim (`&T`) verir; bu yüzden `DerefMut` uygulamaz. Değiştirmek için veriyi `Mutex`'e koyun:[^arc-klon]

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let v = Arc::new(Mutex::new(vec![1]));

    let v2 = Arc::clone(&v);
    thread::spawn(move || v2.lock().expect("kilit zehirlenmiş").push(2))
        .join()
        .expect("thread panikledi");

    println!("{:?}", v.lock().expect("kilit zehirlenmiş"));
}
```

Çıktı:

```text
[1, 2]
```

### Gönderilen değeri sonra kullanmak (E0382)

```rust,compile_fail,E0382
use std::sync::mpsc;

fn main() {
    let (tx, rx) = mpsc::channel();
    let mesaj = String::from("hazır");

    tx.send(mesaj).expect("alıcı kapanmış");
    println!("{mesaj}");
    println!("{:?}", rx.recv());
}
```

Derleyicinin yanıtı:

```text
error[E0382]: borrow of moved value: `mesaj`
```

`send` sahipliği alıcıya taşır; iki thread'in aynı veriye bakması baştan imkânsızdır. Değere gönderen tarafta da gerek varsa kullanımı öne alın ya da klonunu gönderin:[^kanal-sahiplik]

```rust
use std::sync::mpsc;

fn main() {
    let (tx, rx) = mpsc::channel();
    let mesaj = String::from("hazır");

    println!("gönderiliyor: {mesaj}");
    tx.send(mesaj).expect("alıcı kapanmış");
    println!("{:?}", rx.recv());
}
```

Çıktı:

```text
gönderiliyor: hazır
Ok("hazır")
```

### Kilidin koruyucusunu başka thread'e vermek (E0277)

```rust,compile_fail,E0277
use std::sync::Mutex;
use std::thread;

fn main() {
    let m = Mutex::new(5);
    let g = m.lock().expect("kilit zehirlenmiş");

    thread::scope(|kapsam| {
        kapsam.spawn(move || println!("{}", *g));
    });
}
```

Derleyicinin yanıtı:

```text
error[E0277]: `std::sync::MutexGuard<'_, i32>` cannot be sent between threads safely
```

Bazı işletim sistemlerinde kilidi, onu alan thread bırakmak zorundadır; standart kütüphane bu yüzden `MutexGuard`'ı `!Send` yapar.[^guard-send] Kilidi, veriye dokunacak thread'in içinde alın:

```rust
use std::sync::Mutex;
use std::thread;

fn main() {
    let m = Mutex::new(5);

    thread::scope(|kapsam| {
        kapsam.spawn(|| println!("{}", *m.lock().expect("kilit zehirlenmiş")));
    });
}
```

Çıktı:

```text
5
```

## Derleyicinin yakalayamadıkları

Derleyici "veriye kim, hangi izinle uzanıyor?" sorusunu denetler. Kilitlerin hangi sırayla alınacağı, iletilerin ne zaman geleceği ise program çalışırken belli olur. "Korkusuz eşzamanlılık" veri yarışını kapsar; aşağıdakileri kapsamaz.

### Kendi kendini kilitlemek

Aynı thread, elinde tuttuğu `Mutex`'i ikinci kez kilitlemeye çalışırsa `lock` geri dönmez: Ya sonsuza dek bekler ya da panikler; hangisinin olacağı belirtilmemiştir.[^lock] Hata genellikle, kilit eldeyken yine kilit alan başka bir metot çağrıldığında ortaya çıkar. Durumu `try_lock` ile güvenle gözleyebiliriz:

```rust
use std::sync::{Mutex, TryLockError};

fn main() {
    let kasa = Mutex::new(100);

    let ilk = kasa.lock().expect("kilit zehirlenmiş");
    let ikinci_deneme = matches!(kasa.try_lock(), Err(TryLockError::WouldBlock));
    println!("kilit elimizdeyken ikinci kilit bloklanırdı: {ikinci_deneme}");

    drop(ilk); // koruyucu düştü, kilit serbest
    println!("bıraktıktan sonra alınabiliyor: {}", kasa.try_lock().is_ok());
}
```

Çıktı:

```text
kilit elimizdeyken ikinci kilit bloklanırdı: true
bıraktıktan sonra alınabiliyor: true
```

Çare, kilidi almış kodun içinden aynı kilidi alan metotları çağırmamaktır: Ya koruyucuyu önce düşürün ya da iç metoda kilit yerine `&mut T` verin.

### İki kilidi ters sırayla almak

Birinci aşçı bıçağı alıp tahtayı bekler, ikincisi tahtayı alıp bıçağı bekler. İkisi de sonsuza dek bekler:

```rust,no_run
use std::sync::Mutex;
use std::thread;

fn main() {
    let bicak = Mutex::new("bıçak");
    let tahta = Mutex::new("tahta");

    thread::scope(|kapsam| {
        kapsam.spawn(|| {
            let _b = bicak.lock().expect("kilit zehirlenmiş");
            let _t = tahta.lock().expect("kilit zehirlenmiş");
        });
        kapsam.spawn(|| {
            let _t = tahta.lock().expect("kilit zehirlenmiş"); // ters sıra
            let _b = bicak.lock().expect("kilit zehirlenmiş");
        });
    });
}
```

Program derlenir ve çoğu çalıştırmada biter; iki thread ilk kilitlerini aynı anda alırsa asılı kalır. Bu yüzden testlerde görünmeyip üretimde ortaya çıkabilir.[^yaris] Kural basittir: Birden fazla kilit gerekiyorsa her yerde aynı sırayla alınır; herkes önce bıçağı, sonra tahtayı alır.

```rust
use std::sync::Mutex;
use std::thread;

fn main() {
    let bicak = Mutex::new("bıçak");
    let tahta = Mutex::new("tahta");

    thread::scope(|kapsam| {
        for _ in 0..2 {
            kapsam.spawn(|| {
                let _b = bicak.lock().expect("kilit zehirlenmiş"); // önce bıçak
                let _t = tahta.lock().expect("kilit zehirlenmiş"); // sonra tahta
            });
        }
    });
    println!("iki aşçı da işini bitirdi");
}
```

Çıktı:

```text
iki aşçı da işini bitirdi
```

Kilitler aynı türdense sırayı bir numara belirleyebilir; örneğin iki banka hesabı arasında para aktarırken hesap numarası küçük olan önce kilitlenir. Bunun uygulaması ve `try_lock` ile geri çekilme [29. bölümdedir](../ileri/29-ileri-eszamanlilik-async.md).

### Zehirlenmiş kilit

Aşçı anahtarla içerideyken bayılırsa raflar yarım kalır; kapıya "dikkat" levhası asılır. Kilidi tutan thread paniklerse standart `Mutex` zehirlenir ve sonraki `lock` çağrıları `Err` döndürür. Veri durmaktadır; kullanılabilir olup olmadığına siz karar verirsiniz:

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let kasa = Arc::new(Mutex::new(vec![1]));
    let kasa2 = Arc::clone(&kasa);

    let sonuc = thread::spawn(move || {
        let mut g = kasa2.lock().expect("kilit zehirlenmiş");
        g.push(2);
        panic!("kasıtlı panik: kilit elimizdeyken");
    })
    .join();

    println!("thread panikledi: {}", sonuc.is_err());
    println!("zehirli mi: {}", kasa.is_poisoned());
    println!("lock hata verdi: {}", kasa.lock().is_err());

    let kurtarilan = kasa.lock().unwrap_or_else(|zehir| zehir.into_inner());
    println!("kurtarılan veri: {kurtarilan:?}");
}
```

Çıktı (panik mesajı standart hata akışına gider):

```text
thread panikledi: true
zehirli mi: true
lock hata verdi: true
kurtarılan veri: [1, 2]
```

`PoisonError::into_inner` koruyucuyu yine de verir. Veri gerçekten tutarlıysa `clear_poison` işareti kaldırır. Zehirlenme yardımcı bir işarettir, kesin bir güvence değildir: `unsafe` kod, kilidin zehirlenmesine güvenerek doğruluk kuramaz; `panic = "abort"` ile derlenen programda ise yığın çözülmediğinden zehirlenme hiç görülmez.[^zehir]

### Mantıksal yarış

İki ayrı kilitli adımın her biri güvenlidir, ama aralarında durum değişebilir. Aşağıda iki thread de sayacı okur, ikisi de okuduğunun bir fazlasını yazar. `Barrier`, iki okumanın da yazmalardan önce olmasını sağlayarak hatayı her çalıştırmada görünür kılar:

```rust
use std::sync::{Barrier, Mutex};
use std::thread;

fn main() {
    let sayac = Mutex::new(0);
    let bulusma = Barrier::new(2);

    thread::scope(|kapsam| {
        for _ in 0..2 {
            kapsam.spawn(|| {
                let okunan = *sayac.lock().expect("kilit zehirlenmiş"); // kilit bırakıldı
                bulusma.wait();
                *sayac.lock().expect("kilit zehirlenmiş") = okunan + 1;
            });
        }
    });

    println!("sayaç = {}", sayac.lock().expect("kilit zehirlenmiş"));
}
```

Çıktı:

```text
sayaç = 1
```

Veri yarışı yoktur, her erişim kilitlidir; yine de bir artış kaybolmuştur. Bu bir yarış durumudur.[^yaris] Okuma ile yazma arasında kilit bırakılmamalıdır:

```rust
use std::sync::Mutex;
use std::thread;

fn main() {
    let sayac = Mutex::new(0);

    thread::scope(|kapsam| {
        for _ in 0..2 {
            kapsam.spawn(|| {
                let mut n = sayac.lock().expect("kilit zehirlenmiş");
                *n += 1; // okuma ve yazma aynı kilidin altında
            });
        }
    });

    println!("sayaç = {}", sayac.lock().expect("kilit zehirlenmiş"));
}
```

Çıktı:

```text
sayaç = 2
```

### Unutulan gönderen

Alıcı döngüsü, yaşayan tek bir gönderen kaldığı sürece bitmez. Aşağıdaki program tek iletiyi yazar ve sonsuza dek bekler, çünkü özgün `tx` `main`'in sonuna kadar yaşar:

```rust,no_run
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();
    let tx2 = tx.clone();
    thread::spawn(move || tx2.send("çorba").expect("alıcı kapanmış"));

    for tabak in rx {
        println!("{tabak}"); // tx hâlâ yaşıyor, döngü bitmez
    }
}
```

Asılı kalan bir programda ilk bakılacak yer, unutulmuş bir gönderen klonudur.[^kopma] Döngüden önce düşürmek yeter:

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();
    let tx2 = tx.clone();
    thread::spawn(move || tx2.send("çorba").expect("alıcı kapanmış"));
    drop(tx);

    for tabak in rx {
        println!("{tabak}");
    }
    println!("pencere kapandı");
}
```

Çıktı:

```text
çorba
pencere kapandı
```

## Hata kodu başvurusu

| Kod | Ne zaman görülür? | Ne yapmalı? |
| --- | --- | --- |
| `E0373` | `thread::spawn`'a verilen closure yerel veriyi ödünç alır. | `thread::scope` kullanın ya da `move` ile sahipliği verin. |
| `E0382` | `Arc` klonlanmadan taşınır; gönderilen değer yeniden kullanılır. | Taşımadan önce `Arc::clone` çağırın; kullanımı `send`'den önceye alın. |
| `E0596` | `Arc` içindeki veri kilitsiz değiştirilir. | `Arc<Mutex<T>>` ya da atomik tür kullanın. |
| `E0277` | `MutexGuard` başka thread'e taşınır. | Kilidi, veriye dokunacak thread'in içinde alın. |

## Akılda kalması gerekenler

- Derlenen güvenli Rust kodunda veri yarışı olmaz; kilitlenme, kilit çekişmesi ve mantıksal yarış olabilir.
- Thread'ler fonksiyon bitmeden bitecekse `thread::scope` kullanın: `Arc`, klon ve `'static` derdi kalkar.
- Kritik bölgeyi kısa tutun: Kilit altında G/Ç ve uzun hesap yapmayın, başka kilit almayın, kullanıcıdan gelen closure'ı çağırmayın.
- Koruyucunun ömrü kilidin süresidir. Geçici koruyucu deyimin sonunda, değişkene bağlı koruyucu kapsamın sonunda düşer; gerekirse `drop` ile erken bırakın.
- Kanal sahiplikle kapanır: Bütün gönderenler düşünce alıcı döngüsü biter.
- Önce bütün thread'leri başlatın, sonra hepsini bekleyin; araya `collect` koymayı unutmayın.

## Alıştırmalar

### 1. Paralel en büyük

`fn paralel_en_buyuk(veri: &[i64], parca_sayisi: usize) -> Option<i64>` fonksiyonunu `thread::scope` ve `chunks` ile yazın. Boş dilim için `None` dönsün; `[3, -7, 12, 5]` için `Some(12)` beklenir.

<details>
<summary>İpucu ve çözüm</summary>

Her thread `parca.iter().copied().max()` döndürsün. Sonuçlar `Option<i64>` olduğundan `flatten` ya da `filter_map` ile süzülüp yeniden `max` alınabilir.

```rust
use std::thread;

fn paralel_en_buyuk(veri: &[i64], parca_sayisi: usize) -> Option<i64> {
    let parca_boyu = veri.len().div_ceil(parca_sayisi.max(1)).max(1);

    thread::scope(|kapsam| {
        let tutamaclar: Vec<_> = veri
            .chunks(parca_boyu)
            .map(|parca| kapsam.spawn(move || parca.iter().copied().max()))
            .collect();

        tutamaclar
            .into_iter()
            .filter_map(|t| t.join().expect("işçi panikledi"))
            .max()
    })
}

fn main() {
    println!("{:?}", paralel_en_buyuk(&[3, -7, 12, 5], 2));
    println!("{:?}", paralel_en_buyuk(&[], 4));
}
```

Çıktı:

```text
Some(12)
None
```

</details>

### 2. Hatayı taşıyan havuz

`havuzda_isle`'yi, bir iş başarısız olsa da öteki işler sürecek biçimde değiştirin: Sonuç kanalından `Sonuc` yerine `Result<Sonuc, u32>` gönderilsin; `Err` içinde işin kimliği bulunsun. Metin `"HATA"` içeriyorsa iş başarısız sayılsın.

<details>
<summary>İpucu ve çözüm</summary>

İş gövdesini `std::panic::catch_unwind` ile sarmak yerine hatayı baştan `Result` olarak modelleyin. Panik beklenmeyen durumlar içindir; bozuk bir satır beklenen bir durumdur.

```rust
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

struct Is {
    id: u32,
    metin: String,
}

#[derive(Debug, PartialEq, Eq)]
struct Sonuc {
    id: u32,
    kelime_sayisi: usize,
}

fn isle(is: Is) -> Result<Sonuc, u32> {
    if is.metin.contains("HATA") {
        return Err(is.id);
    }
    Ok(Sonuc { id: is.id, kelime_sayisi: is.metin.split_whitespace().count() })
}

fn havuzda_isle(isler: Vec<Is>, isci_sayisi: usize) -> Vec<Result<Sonuc, u32>> {
    let (is_tx, is_rx) = mpsc::channel::<Is>();
    let (sonuc_tx, sonuc_rx) = mpsc::channel();
    let is_rx = Arc::new(Mutex::new(is_rx));

    for _ in 0..isci_sayisi.max(1) {
        let is_rx = Arc::clone(&is_rx);
        let sonuc_tx = sonuc_tx.clone();
        thread::spawn(move || {
            loop {
                let alinan = is_rx.lock().expect("kuyruk kilidi zehirli").recv();
                let Ok(is) = alinan else { break };
                if sonuc_tx.send(isle(is)).is_err() {
                    break;
                }
            }
        });
    }
    drop(sonuc_tx);

    for is in isler {
        is_tx.send(is).expect("bütün işçiler kapanmış");
    }
    drop(is_tx);

    let mut sonuclar: Vec<_> = sonuc_rx.iter().collect();
    sonuclar.sort_by_key(|s| match s {
        Ok(sonuc) => sonuc.id,
        Err(id) => *id,
    });
    sonuclar
}
# fn main() {
#     let isler: Vec<Is> = (1..)
#         .zip(["bir iki", "HATA: bozuk satır", "son"])
#         .map(|(id, metin)| Is { id, metin: metin.to_string() })
#         .collect();
#     for sonuc in havuzda_isle(isler, 2) {
#         println!("{sonuc:?}");
#     }
# }
```

Üç satırla (`"bir iki"`, `"HATA: bozuk satır"`, `"son"`) çalıştırıldığında çıktı:

```text
Ok(Sonuc { id: 1, kelime_sayisi: 2 })
Err(2)
Ok(Sonuc { id: 3, kelime_sayisi: 1 })
```

Burada işçilerin tutamaçları saklanmaz; `sonuc_rx.iter()` bütün işçiler gönderenlerini düşürünce biter, yani beklemeyi kanal sağlar.

</details>

### 3. Kanallı kelime sıklığı

`kelime_sikligi`'ni kanalla yeniden yazın: Her thread yerel tablosunu kanala göndersin, ana thread birleştirsin; `Mutex` kullanmayın. Sonra iki sürümü karşılaştırın: Hangisinde "veriye kim dokunuyor?" sorusunun yanıtı daha kolay görülüyor?

<details>
<summary>İpucu ve çözüm</summary>

`mpsc::channel::<BTreeMap<String, u32>>()` ile başlayın. Her thread'e `tx.clone()` verin ve özgün `tx`'i düşürmeyi unutmayın; unutursanız birleştirme döngüsü bitmez.

```rust
use std::collections::BTreeMap;
use std::sync::mpsc;
use std::thread;

fn kelime_sikligi(belgeler: &[&str]) -> BTreeMap<String, u32> {
    let (tx, rx) = mpsc::channel::<BTreeMap<String, u32>>();

    thread::scope(|kapsam| {
        for belge in belgeler {
            let tx = tx.clone();
            kapsam.spawn(move || {
                let mut yerel = BTreeMap::new();
                for kelime in belge.split_whitespace() {
                    *yerel.entry(kelime.to_lowercase()).or_insert(0) += 1;
                }
                tx.send(yerel).expect("alıcı kapanmış");
            });
        }
        drop(tx);

        let mut toplam = BTreeMap::new();
        for yerel in rx {
            for (kelime, adet) in yerel {
                *toplam.entry(kelime).or_insert(0) += adet;
            }
        }
        toplam
    })
}

fn main() {
    println!("{:?}", kelime_sikligi(&["a b", "B c", "c C c"]));
}
```

Çıktı:

```text
{"a": 1, "b": 2, "c": 4}
```

Bu sürümde birleşik tablonun tek sahibi ana thread'dir; ona kimin dokunduğu koda bakınca görülür ve kilit yoktur. `Mutex`'li sürümde tablo ortadadır, hangi thread'in ne zaman yazdığını kilitlerin yerinden çıkarmak gerekir. Buna karşılık kilitli sürümde birleştirme işi de thread'lere dağılır; kanallı sürümde hepsini ana thread yapar.

</details>

## Özet

- Akan işler için kanal, yerinde duran ortak kayıtlar için `Arc<Mutex<T>>` uygundur.
- `thread::scope`, thread'lerin bitmesini güvence altına alarak yerel verinin ödünç verilmesini sağlar.
- İşçi havuzu iki kanalla kurulur; kapanış `drop(is_tx)` ve `drop(sonuc_tx)` ile sahiplik üzerinden olur.
- Yerelde biriktirip bir kez birleştirmek kilit çekişmesini azaltır; tek bir sayı için atomik yeter.
- Derleyici veri yarışını yakalar; kilitlenmeyi, zehirlenmeyi, mantıksal yarışı ve unutulan göndereni yakalamaz.

Thread'ler işi paralel yürütür. Bekleyen çok sayıda işi az sayıda thread'le yürütmek ise async'in alanıdır: [16. Async ve Await](16-async-await.md). `Send` ile `Sync`'in ayrıntıları [25. bölümdedir](../ileri/25-send-sync.md).

## Kaynaklar

[^kanal-kitap]: The Rust Programming Language, [16.2 Transfer Data Between Threads with Message Passing](https://doc.rust-lang.org/book/ch16-02-message-passing.html#transfer-data-between-threads-with-message-passing) ve [16.3 Shared-State Concurrency](https://doc.rust-lang.org/book/ch16-03-shared-state.html#shared-state-concurrency).
[^scope]: Standart kütüphane, [`thread::scope`](https://doc.rust-lang.org/std/thread/fn.scope.html#lifetimes); [RFC 3151: Scoped threads](https://rust-lang.github.io/rfcs/3151-scoped-threads.html).
[^scope-kod]: Standart kütüphane 1.99.0 kaynak kodu, [`scoped.rs`, `pub fn spawn`](https://github.com/rust-lang/rust/blob/1.99.0/library/std/src/thread/scoped.rs#L201).
[^chunks]: Standart kütüphane, [`slice::chunks`](https://doc.rust-lang.org/std/primitive.slice.html#method.chunks) ve [`slice::chunks_mut`](https://doc.rust-lang.org/std/primitive.slice.html#method.chunks_mut).
[^havuz-kitap]: The Rust Programming Language, [21.2 Improving Throughput with a Thread Pool](https://doc.rust-lang.org/book/ch21-02-multithreaded.html#improving-throughput-with-a-thread-pool) ve [Implementing the `execute` Method](https://doc.rust-lang.org/book/ch21-02-multithreaded.html#implementing-the-execute-method).
[^mpmc]: Standart kütüphane 1.99.0 kaynak kodu, [`mod.rs`, `mpmc_channel`](https://github.com/rust-lang/rust/blob/1.99.0/library/std/src/sync/mod.rs#L192).
[^gecici]: The Rust Reference, [Temporary scopes](https://doc.rust-lang.org/reference/destructors.html#temporary-scopes).
[^if-let]: The Edition Guide, [`if let` temporary scope](https://doc.rust-lang.org/edition-guide/rust-2024/temporary-if-let-scope.html).
[^into-inner]: Standart kütüphane, [`Mutex::into_inner`](https://doc.rust-lang.org/std/sync/struct.Mutex.html#method.into_inner).
[^atomik]: Standart kütüphane, [`std::sync::atomic`](https://doc.rust-lang.org/std/sync/atomic/index.html).
[^atomik-tasinabilir]: Standart kütüphane, [`std::sync::atomic`: Portability](https://doc.rust-lang.org/std/sync/atomic/index.html#portability).
[^sira]: Standart kütüphane, [`Ordering`](https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html#variants); The Rustonomicon, [Atomics: Sequentially Consistent](https://doc.rust-lang.org/nomicon/atomics.html#sequentially-consistent).
[^arc-klon]: Standart kütüphane, [`Arc`](https://doc.rust-lang.org/std/sync/struct.Arc.html#thread-safety) ve [Cloning references](https://doc.rust-lang.org/std/sync/struct.Arc.html#cloning-references).
[^kanal-sahiplik]: The Rust Programming Language, [16.2 Transferring Ownership Through Channels](https://doc.rust-lang.org/book/ch16-02-message-passing.html#transferring-ownership-through-channels).
[^guard-send]: Standart kütüphane 1.99.0 kaynak kodu, [`mutex.rs`, `!Send for MutexGuard`](https://github.com/rust-lang/rust/blob/1.99.0/library/std/src/sync/poison/mutex.rs#L289).
[^lock]: Standart kütüphane, [`Mutex::lock`](https://doc.rust-lang.org/std/sync/struct.Mutex.html#method.lock) ve [`Mutex::try_lock`](https://doc.rust-lang.org/std/sync/struct.Mutex.html#method.try_lock).
[^yaris]: The Rustonomicon, [Data Races and Race Conditions](https://doc.rust-lang.org/nomicon/races.html).
[^zehir]: Standart kütüphane, [`Mutex`: Poisoning](https://doc.rust-lang.org/std/sync/struct.Mutex.html#poisoning), [`Mutex::clear_poison`](https://doc.rust-lang.org/std/sync/struct.Mutex.html#method.clear_poison) ve [`PoisonError::into_inner`](https://doc.rust-lang.org/std/sync/struct.PoisonError.html#method.into_inner).
[^kopma]: Standart kütüphane, [`std::sync::mpsc`: Disconnection](https://doc.rust-lang.org/std/sync/mpsc/index.html#disconnection).
