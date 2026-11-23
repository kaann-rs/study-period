# 15. Eşzamanlılık

Bugünün bilgisayarlarında birden çok işlemci çekirdeği bulunur. Bir program işini birkaç kola ayırabilirse bu çekirdeklerden birlikte yararlanır. Program içinde kendi başına ilerleyen her kola *thread* (iş parçacığı) denir. Kollar birbirinden bağımsız çalıştığı sürece sorun yoktur; asıl güçlük, iki kol aynı veriye dokunduğunda başlar. Hangi kolun önce davranacağı belli olmadığından sonuç çalıştırmadan çalıştırmaya değişebilir, veri yarım yazılmış hâliyle okunabilir.

C ve C++ bu hatayı derleme sırasında yakalamaz. Program çoğu zaman doğru çalışır, yük altında ya da başka bir makinede bozulur; böyle bir hatayı yeniden üretmek bile güçtür. Rust'ta ise sahiplik kuralları ve tür denetimi, güvenli kodda veri yarışını derleme hatasına dönüştürür. Resmî kitap bu yaklaşıma *korkusuz eşzamanlılık* (fearless concurrency) adını verir.[^korkusuz]

Bu bölümde önce thread başlatmayı ve beklemeyi, ardından thread'lerin haberleştiği iki yolu göreceğiz: veriyi kanalla göndermek ve veriyi kilitle paylaşmak. Sonra derleyicinin bu kodu denetlerken baktığı iki trait'i, `Send` ile `Sync`'i tanıyacak, son olarak Rust'ın neyi önlediğini ve neyi önlemediğini ayıracağız.

> [!NOTE]
> Bu bölümün notları ve çalıştırılabilir kodu `learn/concurrency` dizinindedir. Depo kökünden `cargo run -p concurrency` komutuyla çalıştırabilirsiniz.

## Günlük hayattan bir benzetme

Bir lokantanın mutfağını düşünün. Tek aşçı varken yemekler sırayla yapılır. İkinci bir aşçı çağırırsanız o çorbayı karıştırırken siz salatayı hazırlarsınız. Program da işini böyle bölebilir: Her aşçı bir thread'dir.

Üç aşçı aynı tencereye aynı anda uzanır, biri tuz atarken öteki karıştırırsa yemek bozulur. Programda bunun adı *veri yarışıdır* ve Rust buna izin vermez. Bu yüzden tencereyi alan önce "ben kullanıyorum" der, işi bitince bırakır; bu bir *kilittir*. Aşçılar birbirine not da geçebilir: Notu kutuya koyan bırakır, öteki alır; bu kutu bir *kanaldır*. Çağırdığınız aşçının işini bitirmesini beklemek `join`, aynı tencerenin yerini birkaç aşçıya birden bildirmek `Arc` ile olur.

<details>
<summary>Bu bölümde geçen terimler</summary>

- **thread (iş parçacığı):** Programın içinde kendi işini yürüten ayrı kol; mutfaktaki bir aşçı.
- **eşzamanlılık (concurrency):** Birden çok işin aynı dönemde ilerlemesi. İşler sırayla da ilerleyebilir.
- **paralellik (parallelism):** Birden çok işin gerçekten aynı anda yapılması.
- **veri yarışı (data race):** En az iki thread'in aynı bellek yerine eşzamanlama olmadan erişmesi ve bunlardan en az birinin yazması.
- **yarış durumu (race condition):** Sonucun, işlerin hangi sırayla gerçekleştiğine bağlı kalması.
- **`thread::spawn`:** Yeni bir thread başlatan fonksiyon.
- **closure (`|| ...`):** Adı olmayan, yerinde yazılan küçük fonksiyon; thread'e verilen iş.
- **`move`:** Closure'ın kullandığı değerleri sahiplenerek yakalamasını sağlayan anahtar sözcük.
- **tutamaç (handle), `join`:** Başlatılan thread'i temsil eden değer ve onun bitmesini bekleyen metot.
- **sarkan referans (dangling reference):** Gösterdiği veri silinmiş olan referans. Güvenli Rust buna izin vermez.
- **kanal (channel):** Thread'ler arasında tek yönlü ileti hattı.
- **`tx`, `rx`:** Kanalın gönderen ucu (transmitter) ve alan ucu (receiver).
- **mpsc:** Çok üreticili, tek tüketicili (multi-producer, single-consumer) kanal türü.
- **`Mutex<T>`:** İçindeki veriye aynı anda yalnızca bir thread'in erişmesini sağlayan kilit.
- **`lock`, koruyucu (`MutexGuard`):** Kilidi alan metot ve onun döndürdüğü değer. Koruyucu düşünce kilit bırakılır.
- **`Arc<T>`:** Bir değeri birkaç thread'in birlikte sahiplenmesini sağlayan, atomik sayaçlı işaretçi.
- **atomik (atomic):** Başka bir thread'in yarım hâlini göremeyeceği, bölünemez işlem.
- **`Send`, `Sync`:** "Başka thread'e verilebilir" ve "birkaç thread'den birlikte kullanılabilir" anlamını taşıyan trait'ler.
- **işaret trait'i (marker trait):** Metodu olmayan, türün bir özelliğini bildiren trait.
- **zehirlenme (poisoning):** Kilidi tutan thread paniklediğinde kilide konan uyarı işareti.
- **kilitlenme (deadlock):** Thread'lerin birbirini sonsuza dek beklemesi.
- **belirlenimci (deterministic):** Her çalıştırmada aynı sonucu veren.
- **kayıp güncelleme (lost update):** İki yazmadan birinin ötekini ezmesi, bir artışın kaybolması.

</details>

## Eşzamanlılık ve paralellik

Bu iki sözcük sık sık birbirinin yerine kullanılır, oysa aynı şey değildir. Eşzamanlılık, birden çok işin aynı dönemde ilerlemesidir: Tek başına çalışan bir aşçı çorbayı karıştırır, salatayı doğrar, yine çorbaya döner. Paralellik, işlerin gerçekten aynı anda yapılmasıdır: İki aşçı iki tezgâhta birlikte çalışır.[^paralel] Tek çekirdekte de eşzamanlılık olabilir; paralellik için birden çok çekirdek gerekir.

Thread'ler ikisine de yarar. İşletim sistemi elindeki çekirdek sayısına göre thread'leri ya aynı anda ya da dönüşümlü yürütür. Programcı bu sırayı seçmez, bu yüzden kodun doğruluğu belirli bir sıraya dayanmamalıdır.

## Thread başlatmak ve beklemek

`thread::spawn` bir closure alır ve onu yeni bir thread'de çalıştırır. Standart kütüphanenin thread'leri işletim sisteminin thread'leridir: Programdaki her thread'e bir işletim sistemi thread'i karşılık gelir.[^spawn-kitap]

```rust
use std::thread;
use std::time::Duration;

fn main() {
    let asci = thread::spawn(|| {
        for i in 1..=3 {
            println!("aşçı: çorbayı karıştırdı ({i})");
            thread::sleep(Duration::from_millis(1));
        }
    });

    for i in 1..=2 {
        println!("ana thread: salatayı doğradı ({i})");
        thread::sleep(Duration::from_millis(1));
    }

    asci.join().expect("aşçı thread'i panikledi");
}
```

Örnek bir çalıştırma:

```text
ana thread: salatayı doğradı (1)
aşçı: çorbayı karıştırdı (1)
ana thread: salatayı doğradı (2)
aşçı: çorbayı karıştırdı (2)
aşçı: çorbayı karıştırdı (3)
```

Satırların sırası sizde farklı olabilir. Değişmeyen tek şey, beş satırın da yazılmasıdır; bunu son satırdaki `join` sağlar. `spawn`, yeni thread'in bitmesini beklemeden bir `JoinHandle` (tutamaç) döndürür; `join` ise o thread bitene dek bekler. `join`'i silerseniz program, aşçı işini tamamlamadan kapanabilir, çünkü `main` sona erdiğinde öteki thread'ler de kesilir. Tutamacın düşürülmesi thread'i durdurmaz, yalnızca ondan ayırır (detach); artık o thread'i bekleme imkânı kalmaz.[^ayrilma]

### Thread'den sonuç almak

Closure'ın son ifadesi thread'in dönüş değeridir; `join` bu değeri geri verir:

```rust
use std::thread;

fn main() {
    let veri = vec![1, 2, 3];

    let tutamac = thread::spawn(move || {
        let toplam: i32 = veri.iter().sum();
        toplam // thread'in dönüş değeri
    });

    let sonuc = tutamac.join().expect("toplama thread'i panikledi");
    println!("thread topladı = {sonuc}");
}
```

Çıktı:

```text
thread topladı = 6
```

`join` değeri doğrudan değil, bir `Result` içinde döndürür. Thread paniklerse yığın çözülmesi o thread ile sınırlı kalır; panik, bekleyen tarafa `Err` olarak ulaşır:[^join]

```rust
use std::thread;

fn main() {
    let tutamac = thread::spawn(|| -> i32 { panic!("malzeme bozuk") });

    match tutamac.join() {
        Ok(sonuc) => println!("sonuç: {sonuc}"),
        Err(_) => println!("thread panikledi, program sürüyor"),
    }
}
```

Çıktı (panik mesajı ayrıca standart hata akışına yazılır):

```text
thread panikledi, program sürüyor
```

`join().expect(...)` yazmak, "öteki thread paniklediyse ben de panikleyeyim" demektir. Bu davranış yığın çözülmesine dayanır; `panic = "abort"` ile derlenen programda herhangi bir thread'in paniği bütün programı sonlandırır.[^strateji]

## `move`: Sahipliği thread'e vermek

Önceki örnekte closure'ın başında `move` vardı. Onu silince ne olduğuna bakalım:

```rust,compile_fail,E0373
use std::thread;

fn main() {
    let veri = vec![1, 2, 3];

    let tutamac = thread::spawn(|| {
        println!("{}", veri.len());
    });

    tutamac.join().expect("thread panikledi");
}
```

Derleyicinin yanıtı:

```text
error[E0373]: closure may outlive the current function, but it borrows `veri`, which is owned by the current function
  |
6 |     let tutamac = thread::spawn(|| {
  |                                 ^^ may outlive borrowed value `veri`
7 |         println!("{}", veri.len());
  |                        ---- `veri` is borrowed here
  |
help: to force the closure to take ownership of `veri` (and any other referenced variables), use the `move` keyword
  |
6 |     let tutamac = thread::spawn(move || {
  |                                 ++++
```

Closure `veri`'yi yalnızca okuduğu için onu ödünç alır. Derleyici ise yeni thread'in ne kadar yaşayacağını bilmez: Thread, onu başlatan fonksiyondan uzun yaşayabilir. O durumda `veri` fonksiyonla birlikte düşer ve thread'in elinde sarkan bir referans kalır.[^move-kitap] Bu örnekte `join` bekliyor olsa da derleyici `spawn`'ın imzasına bakar; imza closure'ın `'static` sınırını sağlamasını ister.[^spawn-std]

> [!NOTE]
> `'static` sınırı "bu değer programın sonuna kadar yaşar" demek değildir. "İçinde, ömrü daha kısa olan bir ödünç yoktur; istenirse programın sonuna kadar tutulabilir" demektir. `String` ya da `Vec<i32>` gibi verisinin sahibi olan değerler bu sınırı sağlar ve gerektiğinde erkenden düşebilir.

`move` sözcüğü closure'a, kullandığı değerleri sahiplenerek yakalamasını söyler. Malzemeyi aşçıya vermeye benzer: Siz mutfaktan çıksanız bile elindeki malzeme ortadan kalkmaz. Verilen malzeme artık aşçınındır, siz kullanamazsınız:

```rust,compile_fail,E0382
use std::thread;

fn main() {
    let malzeme = vec![1, 2, 3];
    let asci = thread::spawn(move || malzeme.len());

    println!("{malzeme:?}");
    println!("{}", asci.join().expect("thread panikledi"));
}
```

Derleyicinin yanıtı:

```text
error[E0382]: borrow of moved value: `malzeme`
  |
4 |     let malzeme = vec![1, 2, 3];
  |         ------- move occurs because `malzeme` has type `Vec<i32>`, which does not implement the `Copy` trait
5 |     let asci = thread::spawn(move || malzeme.len());
  |                              ------- ------- variable moved due to use in closure
  |                              |
  |                              value moved into closure here
6 |
7 |     println!("{malzeme:?}");
  |                ^^^^^^^ value borrowed here after move
```

Bu, [sahiplik bölümündeki](../temel/03-ownership.md) kuralın aynısıdır; eşzamanlılık için ayrı bir kural yazılmamıştır. Veri tek bir thread'in elinde olduğu için iki thread'in aynı veriye birlikte dokunması da söz konusu olmaz.

Yerel veriyi vermeden, yalnızca ödünç göstererek thread başlatmak da mümkündür. Bunun için `thread::scope` kullanılır; [derinleşme sayfasında](15-concurrency-derinlesme.md) göreceğiz.

## Kanallar: Paylaşmak yerine göndermek

Aşçılar birbirinin tezgâhına uzanmak yerine, hazır olan tabağı servis penceresine koyar; garson tabağı oradan alır. *Kanal* (channel) bu penceredir. `mpsc::channel` iki uç döndürür: gönderen uç `tx` ve alan uç `rx`.[^kanal]

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let tabak = String::from("çorba");
        tx.send(tabak).expect("alıcı kapanmış");
    });

    println!("{}", rx.recv().expect("gönderen kapanmış"));
}
```

Çıktı:

```text
çorba
```

`recv`, bir ileti gelene dek bekler. Beklemeden denemek isterseniz `try_recv` vardır; ileti yoksa hemen hata döndürür. Kanal bir FIFO kuyruğudur: Aynı gönderenin iletileri gönderildikleri sırayla alınır.[^kanal]

### `send` sahipliği taşır

Pencereye konan tabağa, onu koyan aşçı bir daha dokunamaz. Kodda da böyledir: `send`, değerin sahipliğini kanala, oradan alıcıya geçirir.[^kanal-sahiplik]

```rust,compile_fail,E0382
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let tabak = String::from("çorba");
        tx.send(tabak).expect("alıcı kapanmış");
        println!("{tabak}");
    });

    println!("{}", rx.recv().expect("gönderen kapanmış"));
}
```

Derleyicinin yanıtı:

```text
error[E0382]: borrow of moved value: `tabak`
   |
 8 |         let tabak = String::from("çorba");
   |             ----- move occurs because `tabak` has type `String`, which does not implement the `Copy` trait
 9 |         tx.send(tabak).expect("alıcı kapanmış");
   |                 ----- value moved here
10 |         println!("{tabak}");
   |                    ^^^^^ value borrowed here after move
```

Bu kural olmasaydı alıcı metni değiştirirken gönderen onu okuyabilirdi. Sahiplik taşındığı için iki yerden erişim baştan imkânsızdır.

### Çok üretici, tek tüketici

Pencereye birçok aşçı tabak koyabilir: `tx.clone()` her üreticiye kendi gönderen ucunu verir. Alan uç ise tektir. `mpsc` adı da buradan gelir.[^cok-uretici]

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();

    for id in 0..3 {
        let tx = tx.clone(); // her üreticiye kendi göndereni
        thread::spawn(move || {
            tx.send(format!("thread {id} selam")).expect("alıcı kapanmış");
        });
    }
    drop(tx); // özgün gönderen de kapanmalı

    let mut mesajlar: Vec<String> = rx.iter().collect();
    mesajlar.sort(); // varış sırası belirsizdir
    println!("kanaldan gelenler (sıralı): {mesajlar:?}");
}
```

Çıktı:

```text
kanaldan gelenler (sıralı): ["thread 0 selam", "thread 1 selam", "thread 2 selam"]
```

Alıcı bir iterator gibi kullanılabilir: `rx.iter()` gelen iletileri tek tek verir, beklenecek ileti kalmayınca biter. İletilerin hangi sırayla varacağı belli olmadığından liste yazdırılmadan önce sıralanmıştır; böylece çıktı her çalıştırmada aynı olur.

### Kanal nasıl kapanır?

Garson, beklemeyi ne zaman bırakacağını nereden bilir? Bütün aşçılar gittiğinde artık tabak gelmeyecektir. Kanal da aynı kuralla kapanır: Bütün gönderenler düştüğünde ve kuyruk boşaldığında `recv` hata döndürür, `rx.iter()` döngüsü sona erer. Alıcı düştüğünde ise `send` hata döndürür.[^kopma]

```rust
use std::sync::mpsc;

fn main() {
    let (tx, rx) = mpsc::channel();
    tx.send("çorba").expect("alıcı hâlâ açık");
    drop(tx); // son gönderen de gitti

    println!("{:?}", rx.recv());
    println!("{:?}", rx.recv());

    let (tx, rx) = mpsc::channel();
    drop(rx); // alıcı gitti
    println!("{:?}", tx.send("salata"));
}
```

Çıktı:

```text
Ok("çorba")
Err(RecvError)
Err(SendError { .. })
```

Gönderen kapanmadan önce yollanan ileti kaybolmaz; alıcı önce onu alır, kapanışı sonraki çağrıda öğrenir. Önceki örnekteki `drop(tx)` satırının nedeni de budur: Üç üretici kendi kopyalarını düşürse bile özgün `tx` `main`'de yaşamaya devam eder, alıcı "bir ileti daha gelebilir" diye sonsuza dek bekler ve program asılı kalır. Derleyici bu unutkanlığı yakalamaz.

`mpsc::channel` sınırsız bir kuyruk kurar; `send` beklemez. Kapasitesi sınırlı bir kuyruk gerekiyorsa `mpsc::sync_channel` kullanılır; kuyruk doluyken `send` yer açılmasını bekler.[^kanal] Bu bekleyişe *geri basınç* (backpressure) denir ve [29.2'de](../ileri/29-calisma-zamani.md) işlenir.

## Paylaşımlı durum: `Mutex` ve `Arc`

Bazı veriler elden ele gezmez, yerinde durur; mutfağın kileri gibi. Kilerin tek anahtarı vardır: Anahtarı alan içeri girer, ötekiler kapıda bekler. Programdaki karşılığı `Mutex<T>`'dir. Ad, *mutual exclusion* (karşılıklı dışlama) sözünün kısaltmasıdır: Veriye aynı anda yalnızca bir thread erişir.[^mutex-api]

### `Mutex`: Veriye kilitle erişmek

`lock` kilidi alır ve bir `MutexGuard` (koruyucu) döndürür. Veriye bu koruyucu üzerinden ulaşılır; koruyucu düştüğünde kilit kendiliğinden bırakılır.[^mutex-api]

```rust
use std::sync::Mutex;

fn main() {
    let kiler = Mutex::new(5);

    {
        let mut anahtar = kiler.lock().expect("kilit zehirlenmiş");
        *anahtar += 1;
    } // anahtar burada düşer, kilit bırakılır

    println!("{kiler:?}");
}
```

Çıktı:

```text
Mutex { data: 6, poisoned: false, .. }
```

C ve Java gibi dillerde kilit ile veri genellikle ayrı durur; kilidi almadan veriye dokunmak mümkündür. Rust'ta veri `Mutex`'in içindedir ve `lock` çağrılmadan ona ulaşılamaz; önce kilitlemeyi unutmak bu yüzden olanaksızdır. Bırakma da unutulmaz, çünkü işi [önceki bölümde](14-smart-pointers.md) tanıdığımız `Drop` trait'i yapar: Aşçı kilerden çıkarken anahtar kendiliğinden yerine asılır.

`kiler` değişkeni `mut` olmadığı hâlde içindeki sayı değişti. `Mutex`, `RefCell` gibi *iç değiştirilebilirlik* (interior mutability) sağlar: Paylaşımlı bir referans üzerinden, kilidin koruması altında yazmaya izin verir.[^mutex-ic]

`lock`'ın `Result` döndürmesinin nedeni *zehirlenmedir* (poisoning). Kilidi tutan thread paniklerse veri yarım güncellenmiş olabilir; `Mutex` bunu işaretler ve sonraki `lock` çağrıları `Err` verir.[^zehir] Aşçı anahtarla içerideyken bayılmış, kapıya "dikkat" levhası asılmış gibidir. `expect` ya da `unwrap` yazmak "öteki thread paniklediyse ben de durayım" demektir. Veriyi kurtarmanın yolunu [derinleşme sayfasında](15-concurrency-derinlesme.md) göreceğiz.

### Birden çok sahip: `Arc`

Kilidi birkaç thread'e birden vermeyi deneyelim:

```rust,compile_fail,E0382
use std::sync::Mutex;
use std::thread;

fn main() {
    let sayac = Mutex::new(0);
    let mut tutamaclar = vec![];

    for _ in 0..10 {
        tutamaclar.push(thread::spawn(move || {
            *sayac.lock().expect("kilit zehirlenmiş") += 1;
        }));
    }
    for t in tutamaclar {
        t.join().expect("thread panikledi");
    }
}
```

Derleyicinin yanıtı:

```text
error[E0382]: use of moved value: `sayac`
   |
 5 |     let sayac = Mutex::new(0);
   |         ----- move occurs because `sayac` has type `std::sync::Mutex<i32>`, which does not implement the `Copy` trait
...
 8 |     for _ in 0..10 {
   |     -------------- inside of this loop
 9 |         tutamaclar.push(thread::spawn(move || {
   |                                       ^^^^^^^ value moved into closure here, in previous iteration of loop
10 |             *sayac.lock().expect("kilit zehirlenmiş") += 1;
   |              ----- use occurs due to use in closure
```

Döngünün ilk turu `sayac`'ı taşır; ikinci tura taşınacak bir şey kalmaz. Aynı kilidin birden çok sahibi olmalıdır. Önceki bölümde bu iş için `Rc`'yi görmüştük, ama `Rc` thread'ler arasına giremez:

```rust,compile_fail,E0277
use std::rc::Rc;
use std::thread;

fn main() {
    let kiler = Rc::new(0);
    let kiler2 = Rc::clone(&kiler);

    thread::spawn(move || println!("{kiler2}"));
}
```

Derleyicinin yanıtı:

```text
error[E0277]: `Rc<i32>` cannot be sent between threads safely
```

`Rc`, sahip sayısını sıradan bir tam sayıda tutar. İki thread aynı anda klonlar ya da düşürürse sayaç bozulabilir; bozulan sayaç verinin erken silinmesine ya da hiç silinmemesine yol açar. Thread'ler arası paylaşım için aynı arayüzü sunan, ama sayacını atomik işlemlerle değiştiren `Arc` (atomically reference counted) vardır.[^arc] Kilerin yerini gösteren karta benzer: Her aşçıya bir kart verilir, son kart da atılınca kiler kapatılır.

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let sayac = Arc::new(Mutex::new(0));
    let mut tutamaclar = vec![];

    for _ in 0..10 {
        let sayac = Arc::clone(&sayac); // her thread'e bir sahip
        tutamaclar.push(thread::spawn(move || {
            let mut n = sayac.lock().expect("kilit zehirlenmiş");
            *n += 1;
        })); // n düşer, kilit bırakılır
    }
    for t in tutamaclar {
        t.join().expect("thread panikledi");
    }

    println!("Arc<Mutex> sayaç = {}", sayac.lock().expect("kilit zehirlenmiş"));
}
```

Çıktı:

```text
Arc<Mutex> sayaç = 10
```

On thread birer kez artırdı, sonuç her çalıştırmada tam olarak 10'dur. `Arc` ile `Mutex` ayrı sorunları çözer: `Arc` "kaç sahip var?" sorusuna, `Mutex` "şu anda kim erişiyor?" sorusuna yanıt verir. İçinde `Mutex` ya da atomik gibi bir araç yoksa `Arc` yalnızca okumaya izin verir; `Arc`'sız bir `Mutex` ise tek sahibin elinde kalır.

Bu ikili, tek thread'li dünyadaki `Rc<RefCell<T>>`'nin karşılığıdır.[^karsilik]

| | Tek thread | Birden çok thread |
| --- | --- | --- |
| Birden çok sahip | `Rc<T>` | `Arc<T>` |
| Paylaşılan veriyi değiştirmek | `RefCell<T>` | `Mutex<T>` |
| Kural çiğnenirse | `borrow_mut` panikler | `lock` bekler |
| Tipik tehlike | referans döngüsü, sızıntı | kilitlenme (deadlock) |

Her yerde `Arc` kullanılmamasının nedeni bedelidir: Atomik işlemler sıradan bellek erişimlerinden pahalıdır.[^arc] Veri tek thread'de kalıyorsa `Rc` yeter.

## `Send` ve `Sync`: Derleyicinin baktığı iki etiket

Az önceki hata iletisi "`Rc<i32>` cannot be sent between threads safely" diyordu. Derleyici bunu nereden biliyor? Her türün üstünde iki özellik arar:[^send-sync]

- **`Send`:** Bu türün değeri başka bir thread'e taşınabilir. Mutfak diliyle: Başka bir aşçıya verilebilir.
- **`Sync`:** Bu türe, paylaşımlı referansla birden çok thread'den güvenle erişilebilir. Mutfak diliyle: Birkaç aşçı aynı anda bakabilir. Tanım gereği `T`'nin `Sync` olması, `&T`'nin `Send` olması demektir.

İkisi de metodu olmayan *işaret trait'leridir* (marker trait). Programcı bunları genellikle elle yazmaz: İkisi de *otomatik trait* (auto trait) olduğundan, bir türün bütün alanları uygunsa derleyici trait'i kendiliğinden uygulanmış sayar.[^otomatik]

| Tür | `Send` | `Sync` | Neden |
| --- | --- | --- | --- |
| `i32`, `String`, `Vec<u8>` | evet | evet | Sıradan sahipli veri |
| `Rc<T>` | hayır | hayır | Sayacı atomik değildir |
| `Cell<i32>`, `RefCell<i32>` | evet | hayır | Paylaşımlı referansla, eşzamanlama olmadan değişir |
| `Arc<i32>`, `Mutex<i32>` | evet | evet | Sayaç atomiktir, erişim kilitlidir |
| `Arc<Cell<i32>>` | hayır | hayır | `Arc`, içindekini eşzamanlamaz |
| `MutexGuard<'_, i32>` | hayır | evet | Kilidi, alan thread bırakmalıdır |
| `mpsc::Sender<i32>` | evet | evet | Üreticiler arasında paylaşılabilir |
| `mpsc::Receiver<i32>` | evet | hayır | Tüketici tektir |
| `Box<dyn Fn()>` | hayır | hayır | Arkasındaki türün ne olduğu bilinmez |

`Cell` satırını deneyelim. `thread::scope` ile başlatılan thread yerel veriyi ödünç alabilir; ama `&Cell<i32>`'nin `Send` olması için `Cell<i32>`'nin `Sync` olması gerekir:

```rust,compile_fail,E0277
use std::cell::Cell;
use std::thread;

fn main() {
    let sayac = Cell::new(0);

    thread::scope(|kapsam| {
        kapsam.spawn(|| sayac.set(1));
    });
}
```

Derleyicinin yanıtı:

```text
error[E0277]: `Cell<i32>` cannot be shared between threads safely
    |
  8 |         kapsam.spawn(|| sayac.set(1));
    |                ----- ^^^^^^^^^^^^^^^ `Cell<i32>` cannot be shared between threads safely
    |                |
    |                required by a bound introduced by this call
    |
    = help: the trait `Sync` is not implemented for `Cell<i32>`
    = note: if you want to do aliasing and mutation between multiple threads, use `std::sync::RwLock` or `std::sync::atomic::AtomicI32` instead
```

Derleyici yalnızca reddetmekle kalmaz, yerine ne kullanılacağını da önerir.

Trait nesnelerinde bu sınırlar açıkça yazılır. `Box<dyn FnOnce()>`, arkasında hangi türün durduğunu gizler; o türün `Send` olup olmadığı da gizlenmiş olur:

```rust,compile_fail,E0277
use std::thread;

fn baslat(is: Box<dyn FnOnce()>) {
    thread::spawn(move || is());
}

fn main() {
    baslat(Box::new(|| println!("merhaba")));
}
```

Derleyicinin yanıtı:

```text
error[E0277]: `dyn FnOnce()` cannot be sent between threads safely
```

Çözüm, sınırı türe eklemektir. Değer thread'ler arasında paylaşılacaksa (örneğin bir `Arc`'ın içinde duracaksa) `Sync` de gerekir:

```rust
use std::sync::Arc;
use std::thread;

fn baslat(is: Box<dyn FnOnce() + Send>) -> thread::JoinHandle<()> {
    thread::spawn(move || is())
}

fn main() {
    let selam: Arc<dyn Fn() + Send + Sync> = Arc::new(|| println!("merhaba"));
    let selam2 = Arc::clone(&selam);

    baslat(Box::new(move || selam2())).join().expect("thread panikledi");
    selam();
}
```

Çıktı:

```text
merhaba
merhaba
```

`Box<dyn ... + Send + 'static>` biçimini eşzamanlı kodda sıkça göreceksiniz; [17. bölümdeki](17-oop.md) trait nesneleri için de aynı kural geçerlidir.

> [!WARNING]
> `Send` ve `Sync`'i elle uygulamak `unsafe` gerektirir, çünkü derleyici bu sözün doğruluğunu denetleyemez.[^elle] Bu konu [25. bölümde](../ileri/25-send-sync.md) işlenir.

## Rust neyi önler, neyi önlemez?

Veri yarışı (data race) dar bir tanımı olan bir hatadır: En az iki thread aynı bellek yerine eşzamanlama olmadan erişir ve en az biri yazar. Veri yarışı tanımsız davranıştır; güvenli Rust'ta gerçekleşemez.[^yaris]

Bunu sağlayan kural yeni değildir. Ödünç kuralı "ya birçok paylaşımlı referans ya da tek bir değiştirilebilir referans" der; İngilizce kaynaklarda *aliasing XOR mutability* diye anılır. Kural thread'ler arasında da geçerlidir:

```rust,compile_fail,E0499
use std::thread;

fn main() {
    let mut sayac = 0;

    thread::scope(|kapsam| {
        kapsam.spawn(|| sayac += 1);
        kapsam.spawn(|| sayac += 1);
    });
}
```

Derleyicinin yanıtı:

```text
error[E0499]: cannot borrow `sayac` as mutable more than once at a time
    |
  7 |         kapsam.spawn(|| sayac += 1);
    |         ---------------------------
    |         |            |  |
    |         |            |  first borrow occurs due to use of `sayac` in closure
    |         |            first mutable borrow occurs here
    |         argument requires that `sayac` is borrowed for `'1`
  8 |         kapsam.spawn(|| sayac += 1);
    |                      ^^ ----- second borrow occurs due to use of `sayac` in closure
    |                      |
    |                      second mutable borrow occurs here
```

C'de bu kod derlenir ve bazı artışlar kaybolabilir. Artırma tek adım değildir: değer okunur, bir eklenir, geri yazılır. İki thread aynı değeri okursa ikinci yazma birincisini ezer. Bu *kayıp güncellemedir* (lost update). Rust'ta doğru çözüm bir `Mutex` ya da atomik sayaçtır.

Derleyicinin yakalamadığı hatalar da vardır; bunlar bellek güvenliğini bozmaz ve derlenen güvenli kodda görülebilir:[^yaris]

- **Kilitlenme (deadlock):** İki aşçıdan biri bıçağı tutup tahtayı, öteki tahtayı tutup bıçağı bekler; ikisi de sonsuza dek bekler.
- **Yarış durumu (race condition):** Her adım tek başına güvenlidir, ama adımlar arasında durum değişir ve sonuç sıraya bağlı kalır.
- **Asılı kalan kanal:** Unutulan bir gönderen yüzünden alıcı hiç gelmeyecek iletiyi bekler.

Üçünün de örnekleri [derinleşme sayfasındadır](15-concurrency-derinlesme.md).

## Hata kodu başvurusu

| Kod | Ne zaman görülür? | Ne yapmalı? |
| --- | --- | --- |
| `E0373` | `thread::spawn`'a verilen closure yerel bir değişkeni ödünç alır. | `move` ile sahipliği verin, `Arc` ile paylaşın ya da `thread::scope` kullanın. |
| `E0382` | Closure'a taşınan ya da `send` ile gönderilen değer yeniden kullanılır. | Taşımadan önce klonlayın (`Arc::clone`, `tx.clone`) ya da kullanımı öne alın. |
| `E0277` | `Send` ya da `Sync` olmayan bir değer thread'e verilir. | `Rc` yerine `Arc`, `Cell`/`RefCell` yerine atomik tür ya da `Mutex` kullanın; trait nesnesine `+ Send` ekleyin. |
| `E0499` | İki thread aynı değişkeni değiştirilebilir ödünç alır. | `Mutex` ya da atomik tür kullanın; veriyi ayrık parçalara bölün. |

Bir hatanın uzun açıklamasını terminalde görebilirsiniz:

```console
$ rustc --explain E0373
```

## Perde arkası: Derleyici ne yapıyor?

> [!NOTE]
> Bu başlık ilk okuyuşta atlanabilir. Akılda kalması gereken tek şey şudur: Derlenen güvenli kodda veri yarışı olmaz, ama kilitlenme ve mantık hataları olabilir.

### `spawn`'ın imzası

Thread'lere ilişkin denetimin çoğu tek bir fonksiyon imzasından doğar:[^spawn-kod]

```rust
# use std::thread::{self, JoinHandle};
pub fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
# {
#     thread::spawn(f)
# }
```

Üç satırlık `where`, üç soruyu derleyiciye sordurur. `FnOnce`, closure'ın bir kez çalışacağını söyler. `Send`, closure'ın yakaladığı her şeyin başka bir thread'e verilebileceğini güvenceye alır. `'static`, closure'ın içinde kısa ömürlü ödünç bulunmadığını söyler. Dönüş türü `T` için de aynı iki sınır vardır, çünkü sonuç geri taşınır. Thread başlatmak için dilde ayrı bir kural yoktur; genel kurallar bu imza üzerinden uygulanır.

Closure, derleyicinin ürettiği adsız bir yapıdır ve yakaladığı değerler bu yapının alanlarıdır; bu nedenle `Send` olup olmadığı alanlarından çıkarılır.[^closure-send] Sorulara iki ayrı aşama yanıt verir: `Send` ve `Sync`'i trait denetimi (E0277), ödünçlerin yeterince uzun yaşayıp yaşamadığını ödünç denetleyicisi (E0373) inceler. E0373 iletisini `rustc_borrowck` üretir.[^e0373-kod]

### `Send` ve `Sync` nerede tanımlı?

İkisi de `core` crate'inin `marker.rs` dosyasında `unsafe auto trait` olarak durur.[^marker] `auto`, derleyicinin trait'i alanlara bakarak uyguladığını; `unsafe`, elle uygulayanın derleyicinin denetleyemeyeceği bir söz verdiğini belirtir. Tersine, bir türün uygulamadığını kesinleştirmek için standart kütüphane olumsuz uygulama yazar: `rc.rs` dosyasında `impl !Send for Rc` ve `impl !Sync for Rc` satırları bulunur.[^rc-not-send] Olumsuz uygulama yazmak kararlı Rust'ta henüz mümkün değildir; standart kütüphane bunun için kararsız bir özellik kullanır.

### `Rc` ile `Arc` arasındaki fark

`Rc`'nin gösterdiği blokta iki sayaç ve veri durur; sayaçlar `Cell<usize>` türündedir.[^rc-sayac] `Rc::clone` sayacı okur, bir artırır, geri yazar. Bu üç adım iki thread'de iç içe geçerse şu olabilir:

| Adım | Thread 1 | Thread 2 | Sayaç |
| --- | --- | --- | --- |
| 1 | 1 okur | | 1 |
| 2 | | 1 okur | 1 |
| 3 | 2 yazar | | 2 |
| 4 | | 2 yazar | 2 |

Üç sahip vardır, sayaç ise 2 gösterir. İki sahip düştüğünde sayaç sıfırlanır ve veri silinir; üçüncü sahip silinmiş belleğe bakar. Düşürmeler iç içe geçerse tersi olur: Sayaç sıfıra inmez ve bellek sızar. `Arc`'ın bloğu ise sayaçları atomik türlerde tutar; artırma bölünemez tek bir işlemdir.[^arc-kod]

> [!NOTE]
> `Arc<T>`'nin `Send` ve `Sync` olması için `T`'nin de `Send` ve `Sync` olması gerekir. `Arc` yalnızca sayacı korur, içindeki veriye erişimi eşzamanlamaz; bu yüzden `Arc<RefCell<T>>` thread'ler arasında kullanılamaz.[^arc-kod]

### `Mutex` ve koruyucu

`Mutex<T>` üç alandan oluşur: işletim sistemi kilidi, zehirlenme işareti ve veriyi saran bir `UnsafeCell<T>`.[^mutex-kod] Veri bu yapının içinde özel (private) durur; dışarıya açılan kapı `lock`'ın döndürdüğü `MutexGuard`'dır. Koruyucunun `Drop` uygulaması kilidi bırakır.[^guard-drop] `T` `Send` ise `Mutex<T>` hem `Send` hem `Sync` sayılır; `T`'nin `Sync` olması gerekmez, çünkü veriye aynı anda tek thread dokunur. Böylece `Mutex<Cell<i32>>` paylaşılabilir, `Mutex<Rc<i32>>` paylaşılamaz.

## Özet

- `thread::spawn` yeni bir thread başlatır; `join` bitmesini bekler ve sonucu `Result` içinde verir. `main` bitince öteki thread'ler kesilir.
- Thread, onu başlatan fonksiyondan uzun yaşayabilir. Bu yüzden yerel veri ödünç verilemez (E0373); `move` ile sahiplik devredilir.
- Kanal, veriyi bir thread'den ötekine taşır. `send` sahipliği geçirir; bütün gönderenler düşünce alıcı döngüsü biter.
- `Mutex` veriye sırayla erişmeyi, `Arc` birden çok sahip olmayı sağlar. Koruyucu düşünce kilit bırakılır.
- `Send` "başka thread'e taşınabilir", `Sync` "birkaç thread'den paylaşımlı kullanılabilir" demektir. Derleyici ikisini alanlara bakarak çıkarır.
- Güvenli Rust veri yarışını derleme zamanında önler; kilitlenmeyi, yarış durumlarını ve unutulan gönderenleri önlemez.

Sıradaki sayfada bu üç aracı gerçek iş kalıplarına dönüştüreceğiz: [15.1 Eşzamanlılık: Derinleşme](15-concurrency-derinlesme.md).

## Kaynaklar

[^korkusuz]: The Rust Programming Language, [16. Fearless Concurrency](https://doc.rust-lang.org/book/ch16-00-concurrency.html).
[^paralel]: The Rust Programming Language, [17. Parallelism and Concurrency](https://doc.rust-lang.org/book/ch17-00-async-await.html#parallelism-and-concurrency).
[^spawn-kitap]: The Rust Programming Language, [16.1 Creating a New Thread with `spawn`](https://doc.rust-lang.org/book/ch16-01-threads.html#creating-a-new-thread-with-spawn) ve [Waiting for All Threads to Finish](https://doc.rust-lang.org/book/ch16-01-threads.html#waiting-for-all-threads-to-finish).
[^ayrilma]: Standart kütüphane, [`std::thread`: The threading model](https://doc.rust-lang.org/std/thread/index.html#the-threading-model), [Spawning a thread](https://doc.rust-lang.org/std/thread/index.html#spawning-a-thread) ve [`JoinHandle`](https://doc.rust-lang.org/std/thread/struct.JoinHandle.html).
[^join]: Standart kütüphane, [`JoinHandle::join`](https://doc.rust-lang.org/std/thread/struct.JoinHandle.html#method.join) ve [`std::thread`: The threading model](https://doc.rust-lang.org/std/thread/index.html#the-threading-model).
[^strateji]: The Rust Reference, [Panic strategy](https://doc.rust-lang.org/reference/panic.html#panic-strategy).
[^move-kitap]: The Rust Programming Language, [16.1 Using `move` Closures with Threads](https://doc.rust-lang.org/book/ch16-01-threads.html#using-move-closures-with-threads).
[^spawn-std]: Standart kütüphane, [`thread::spawn`](https://doc.rust-lang.org/std/thread/fn.spawn.html).
[^kanal]: The Rust Programming Language, [16.2 Transfer Data Between Threads with Message Passing](https://doc.rust-lang.org/book/ch16-02-message-passing.html#transfer-data-between-threads-with-message-passing); standart kütüphane, [`std::sync::mpsc`](https://doc.rust-lang.org/std/sync/mpsc/index.html).
[^kanal-sahiplik]: The Rust Programming Language, [16.2 Transferring Ownership Through Channels](https://doc.rust-lang.org/book/ch16-02-message-passing.html#transferring-ownership-through-channels).
[^cok-uretici]: The Rust Programming Language, [16.2 Creating Multiple Producers](https://doc.rust-lang.org/book/ch16-02-message-passing.html#creating-multiple-producers).
[^kopma]: Standart kütüphane, [`std::sync::mpsc`: Disconnection](https://doc.rust-lang.org/std/sync/mpsc/index.html#disconnection).
[^mutex-api]: The Rust Programming Language, [16.3 Controlling Access with Mutexes](https://doc.rust-lang.org/book/ch16-03-shared-state.html#controlling-access-with-mutexes) ve [The API of `Mutex<T>`](https://doc.rust-lang.org/book/ch16-03-shared-state.html#the-api-of-mutext).
[^mutex-ic]: The Rust Programming Language, [16.3 Comparing `RefCell<T>`/`Rc<T>` and `Mutex<T>`/`Arc<T>`](https://doc.rust-lang.org/book/ch16-03-shared-state.html#comparing-refcelltrct-and-mutextarct).
[^zehir]: Standart kütüphane, [`Mutex`: Poisoning](https://doc.rust-lang.org/std/sync/struct.Mutex.html#poisoning).
[^arc]: The Rust Programming Language, [16.3 Multiple Ownership with Multiple Threads](https://doc.rust-lang.org/book/ch16-03-shared-state.html#multiple-ownership-with-multiple-threads); standart kütüphane, [`Arc`: Thread Safety](https://doc.rust-lang.org/std/sync/struct.Arc.html#thread-safety).
[^karsilik]: The Rust Programming Language, [16.3 Comparing `RefCell<T>`/`Rc<T>` and `Mutex<T>`/`Arc<T>`](https://doc.rust-lang.org/book/ch16-03-shared-state.html#comparing-refcelltrct-and-mutextarct).
[^send-sync]: The Rust Programming Language, [16.4 Transferring Ownership Between Threads](https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html#transferring-ownership-between-threads) ve [Accessing from Multiple Threads](https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html#accessing-from-multiple-threads); standart kütüphane, [`Sync`](https://doc.rust-lang.org/std/marker/trait.Sync.html).
[^otomatik]: The Rust Reference, [Auto traits](https://doc.rust-lang.org/reference/special-types-and-traits.html#auto-traits).
[^elle]: The Rust Programming Language, [16.4 Implementing `Send` and `Sync` Manually Is Unsafe](https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html#implementing-send-and-sync-manually-is-unsafe); The Rustonomicon, [Send and Sync](https://doc.rust-lang.org/nomicon/send-and-sync.html).
[^yaris]: The Rustonomicon, [Data Races and Race Conditions](https://doc.rust-lang.org/nomicon/races.html); The Rust Reference, [Behavior considered undefined](https://doc.rust-lang.org/reference/behavior-considered-undefined.html).
[^spawn-kod]: Standart kütüphane 1.99.0 kaynak kodu, [`functions.rs`, `pub fn spawn`](https://github.com/rust-lang/rust/blob/1.99.0/library/std/src/thread/functions.rs#L125).
[^closure-send]: The Rust Reference, [Closure types: Other traits](https://doc.rust-lang.org/reference/types/closure.html#other-traits).
[^e0373-kod]: rustc 1.99.0 kaynak kodu, [`borrowck_errors.rs`, `cannot_capture_in_long_lived_closure`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_borrowck/src/borrowck_errors.rs#L456).
[^marker]: Standart kütüphane 1.99.0 kaynak kodu, [`marker.rs`, `auto trait Send`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/marker.rs#L92) ve [`auto trait Sync`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/marker.rs#L657).
[^rc-not-send]: Standart kütüphane 1.99.0 kaynak kodu, [`rc.rs`, `!Send for Rc`](https://github.com/rust-lang/rust/blob/1.99.0/library/alloc/src/rc.rs#L334).
[^rc-sayac]: Standart kütüphane 1.99.0 kaynak kodu, [`rc.rs`, `struct RcInner`](https://github.com/rust-lang/rust/blob/1.99.0/library/alloc/src/rc.rs#L285).
[^arc-kod]: Standart kütüphane 1.99.0 kaynak kodu, [`sync.rs`, `struct ArcInner`](https://github.com/rust-lang/rust/blob/1.99.0/library/alloc/src/sync.rs#L393) ve [`Send for Arc`](https://github.com/rust-lang/rust/blob/1.99.0/library/alloc/src/sync.rs#L284).
[^mutex-kod]: Standart kütüphane 1.99.0 kaynak kodu, [`mutex.rs`, `pub struct Mutex`](https://github.com/rust-lang/rust/blob/1.99.0/library/std/src/sync/poison/mutex.rs#L227).
[^guard-drop]: Standart kütüphane 1.99.0 kaynak kodu, [`mutex.rs`, `Drop for MutexGuard`](https://github.com/rust-lang/rust/blob/1.99.0/library/std/src/sync/poison/mutex.rs#L741).
