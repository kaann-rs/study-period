# Eşzamanlılık, Async/Await, OOP, Desenler ve Gelişmiş Özellikler — Doğruluk Denetimi

## Kapsam, yöntem ve kaynaklar

### Takeaway
Beş klasördeki 15 dosya (`concurrency`, `async-await`, `oop`, `patterns`, `advanced-features`: 5 × ACIKLAMA.md + 10 × .rs) satır satır denetlendi. Teknik çekirdek iddiaların büyük kısmı doğru ve yerel `guide/rust` kaynağı, resmî dokümanlar veya derleyici reprolarıyla birebir eşleşiyor. Bulunan sorunlar dört kümede toplanıyor: (a) async Book-17 "2024 edition'da eklendi" iddiası birincil kaynakla doğrulanamadı, (b) `check_binding_is_irrefutable` kaynak satır numarası yanlış, (c) OOP'de "Böl. 15" atfı düzeltme notuna rağmen gövdede kalmış, (d) encapsulation/E0616 iddiası modül-bazlı gizlilik yüzünden bağlam dışında yanıltıcı (düzeltme notu doğru).

### Cited Findings
- Denetlenen dosyalar: `learn/concurrency/{ACIKLAMA.md,src/main.rs,src/derinlesme.rs}`, `learn/async-await/{ACIKLAMA.md,src/main.rs,src/derinlesme.rs}`, `learn/oop/{ACIKLAMA.md,src/main.rs,src/derinlesme.rs}`, `learn/patterns/{ACIKLAMA.md,src/main.rs,src/derinlesme.rs}`, `learn/advanced-features/{ACIKLAMA.md,src/main.rs,src/derinlesme.rs}`. Bu denetim sırasında `learn/` altında **hiçbir dosya değiştirilmedi**; "Çalıştır:" komutları çalıştırılmadı.
- Yerel araç zinciri: `rustc 1.99.0 (b940084d7 2026-09-28)`. Mini reprolar `/tmp/opencode/rust-verify/` altında derlendi; hata kodları `rustc --explain` ile doğrulandı. Kaynak ağacı: `guide/rust/library/...`, `guide/rust/compiler/...`.
- Resmî kaynaklar: [The Rust Programming Language — Ch. 16 Fearless Concurrency](https://doc.rust-lang.org/book/ch16-01-threads.html), [Ch. 16 Shared State](https://doc.rust-lang.org/book/ch16-03-shared-state.html), [Ch. 16 Send & Sync](https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html), [Ch. 17 Async](https://doc.rust-lang.org/book/ch17-00-async-await.html), [Rust Reference — Undefined Behavior](https://doc.rust-lang.org/reference/behavior-considered-undefined.html), [Rust Reference — Trait objects](https://doc.rust-lang.org/reference/types/trait-object.html), [std::future::Future](https://doc.rust-lang.org/stable/std/future/trait.Future.html), [std::sync::Mutex](https://doc.rust-lang.org/stable/std/sync/struct.Mutex.html), [std::thread::scope](https://doc.rust-lang.org/stable/std/thread/fn.scope.html), [Rust 1.86 blog (trait upcasting)](https://blog.rust-lang.org/2025/04/03/Rust-1.86.0/), [Rust 1.85 blog (2024 edition)](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/).

### Inferences
- "DÜZELTME NOTU (son gözden geçirme)" etiketli paragraf­lar sonradan eklenmiş; gövde metinleri eski sürümden kalmış. OOP dosyasında olduğu gibi bazı eski atıflar düzeltme notuna rağmen gövdede yaşamaya devam ediyor — raporda "iç tutarsızlık" olarak sunmak isabetli.
- Derleyici hata kodları (`E0277`, `E0382`, `E0596`, `E0599`, `E0005`, `E0038`, `E0133`, `E0728`, `E0733`, `E0373`, `E0308`, `E0004`) bu denetimde teker teker reprolanmış; bunlara dayanan iddialar güvenilir.

### Gaps
- Book 17 bölümünün "2024 edition'da kitaba eklendiği" iddiası için resmî bir açıklama bulunamadı (bkz. bölüm 2).
- `check_binding_is_irrefutable` fonksiyonunun hangi rustc sürümünde `~60`. satırda olduğu doğrulanamadı; güncel ağaçta `:657`.

---

## 1. concurrency — Send/Sync, veri yarışı, Mutex/Arc, atomikler

### Takeaway
Bu klasördeki teknik iddiaların tamamı doğru. Özellikle marker trait satır numaraları, `Rc`'nin `!Send` olma nedeni, `Arc` re-export düzeltme notu, `MutexGuard`'ın `!Send` oluşu, poison mekanizması ve `thread::scope` davranışı birincil kaynaklarla birebir eşleşiyor. Hata kodlarının tamamı reprolandı.

### Cited Findings

- **dosya:satır:** `concurrency/src/main.rs:2` — **Alıntı:** `// Rust Book — Bölüm 16: Fearless Concurrency (korkusuz eşzamanlılık)` — **Hüküm:** DOĞRU — **Kaynak:** [Rust Book Ch. 16](https://doc.rust-lang.org/book/ch16-01-threads.html) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `concurrency/src/main.rs:5-7` — **Alıntı:** `"Korkusuz" çünkü sahiplik + tür sistemi, VERİ YARIŞINI (data race: en az iki thread aynı veriye erişir, biri yazar, eşzamanlama yoktur) DERLEME ANINDA engeller. C/C++'ta bu bir çalışma-anı bilinmeyeniyken Rust'ta derlenmeyen bir programdır.` — **Hüküm:** DOĞRU — **Kaynak:** [Rust Reference — Data races UB listesi](https://doc.rust-lang.org/reference/behavior-considered-undefined.html), [Book Ch. 16 Send & Sync](https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html) ("The type system and the borrow checker ensure that the code using these solutions won't end up with data races") — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `concurrency/src/main.rs:9-10` — **Alıntı:** `Doğrulanmış (rustc ile): Rc bir thread'e taşınamaz → E0277 "`Rc<i32>` cannot be sent between threads safely" (Rc, Send değil).` — **Hüküm:** DOĞRU — **Kaynak:** `/tmp/opencode/rust-verify/kodlar/e0277_rc.rs` → `error[E0277]: Rc<i32> cannot be sent between threads safely`; `guide/rust/library/core/src/marker.rs:92` (`pub unsafe auto trait Send`); [Book Ch. 16 Send & Sync](https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `concurrency/src/main.rs:42` — **Alıntı:** `deadlock (kilitlenme) → iki thread'in birbirini sonsuza dek beklemesi. Rust bunu ÖNLEMEZ.` — **Hüküm:** DOĞRU — **Kaynak:** [Book Ch. 16 Shared State](https://doc.rust-lang.org/book/ch16-03-shared-state.html) ("Mutex<T> comes with the risk of creating deadlocks"); `concurrency/src/derinlesme.rs:419-450` bunu çalışma-anı sorunu olarak açıkça ayırıyor — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `concurrency/src/main.rs:179-185` — **Alıntı:** `Send (satır 92): "bu tür başka bir thread'e GÜVENLE TAŞINABİLİR"... Sync (satır 657): "bu türe &T ile birden çok thread'den GÜVENLE ERİŞİLEBİLİLİR" (yani &T Send ise T Sync'tir). İkisi de unsafe auto trait` — **Hüküm:** DOĞRU — **Kaynak:** `guide/rust/library/core/src/marker.rs:92` (`pub unsafe auto trait Send`), `:657` (`pub unsafe auto trait Sync`); [Book Ch. 16 Send & Sync](https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `concurrency/src/main.rs:187-193` — **Alıntı:** `Rc NEDEN Send DEĞİL? Rc'nin sayacı SIRADAN (atomik olmayan) bir tam sayıdır... Bu yüzden Rc bilinçli olarak !Send'dir... E0277 "cannot be sent between threads safely" (DOĞRULANDI). Çok-thread için atomik sayaçlı Arc (library/alloc/src/sync.rs) kullanılır — aynı API, atomik sayaç ↳ DÜZELTME NOTU: güncel kaynak ağacında Arc'ın tanımı library/alloc/src/rcs/arc.rs dosyasındadır; sync.rs yalnız yeniden dışa aktarır (re-export).` — **Hüküm:** DOĞRU (düzeltme notu da doğru) — **Kaynak:** `guide/rust/library/alloc/src/rcs/rc.rs:287-288` (`strong: Cell<usize>`, `weak: Cell<usize>`), `guide/rust/library/alloc/src/sync.rs:14` (`pub use crate::rcs::arc::Arc`), `guide/rust/library/alloc/src/rcs/arc.rs:278,280` (`unsafe impl ... Send for Arc`, `... Sync for Arc`), `:231` (`Sharing a mutable AtomicUsize`); [Book Ch. 16 Send & Sync](https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `concurrency/src/main.rs:195-199` — **Alıntı:** `Mutex<T> (library/std/src/sync/poison/mutex.rs:227) kilit + veriyi BİRLİKTE tutar... guard düşünce kilit bırakılır (Drop)... "poison": kilidi tutan thread panik ederse Mutex zehirlenir; sonraki lock() Err döner` — **Hüküm:** DOĞRU — **Kaynak:** `guide/rust/library/std/src/sync/poison/mutex.rs:227` (`pub struct Mutex`), `:277` (`pub struct MutexGuard`), [std::sync::Mutex — Poisoning](https://doc.rust-lang.org/stable/std/sync/struct.Mutex.html) ("a mutex becomes poisoned if it recognizes that the thread holding it has panicked... lock and try_lock methods return a Result") — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `concurrency/src/main.rs:201-205` — **Alıntı:** `SONUÇ: veri yarışının tanımı "paylaşımlı + değiştirilebilir + eşzamanlamasız erişim"... İhlal, çalışma anı kumarı değil, bir DERLEME hatasıdır — "fearless" tam da bu... (Deadlock/mantık hatası ayrı mesele; onları önlemez.)` — **Hüküm:** DOĞRU — **Kaynak:** [Rust Reference UB listesi](https://doc.rust-lang.org/reference/behavior-considered-undefined.html), [Book Ch. 16 Send & Sync](https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `concurrency/src/derinlesme.rs:92` — **Alıntı:** `// E0373: closure may outlive the current function, but it borrows \`veri\`` — **Hüküm:** DOĞRU — **Kaynak:** `rustc 1.99 --explain E0373` ("closure may outlive the current function, but it borrows ..."); [std::thread::spawn](https://doc.rust-lang.org/stable/std/thread/fn.spawn.html) ('static gereksinimi) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `concurrency/src/derinlesme.rs:97-105` — **Alıntı:** `thread::scope tam bunu ifade eder: kapsamın içinde doğan her thread, kapsam kapanmadan OTOMATİK join edilir. Derleyici bu garantiye dayanarak 'static şartını kaldırır; closure'lar yerel değişkenleri düz & ile (hatta ayrık parçaları &mut ile) ödünç alabilir.` — **Hüküm:** DOĞRU — **Kaynak:** [std::thread::scope](https://doc.rust-lang.org/stable/std/thread/fn.scope.html) ("scoped threads can borrow non-'static data, as the scope guarantees all threads will be joined at the end of the scope... automatically joined before this function returns") — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `concurrency/src/derinlesme.rs:122-127` — **Alıntı:** `collect() burada ŞART: iterator'lar tembeldir. Tutamaçları toplamadan doğrudan .map(spawn).map(join) zincirleseydik her thread doğar doğmaz join edilir, bir sonraki ancak o bitince başlardı — kod "paralel" görünür ama SERİ çalışırdı.` — **Hüküm:** DOĞRU — **Kaynak:** [Book Ch. 13 Iterators — laziness](https://doc.rust-lang.org/book/ch13-02-iterators.html) ("iterators are lazy, meaning they have no effect until you call methods that consume the iterator") — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `concurrency/src/derinlesme.rs:398` — **Alıntı:** `// println!("{m:?}"); // E0382: the type \`Arc\` does not implement \`Copy\`` — **Hüküm:** DOĞRU — **Kaynak:** `/tmp/opencode/rust-verify/kodlar/e0382_arc.rs` → `error[E0382]: the type Arc does not implement Copy` — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `concurrency/src/derinlesme.rs:404` — **Alıntı:** `thread::spawn(move || v.push(2)); // E0596: cannot borrow data in an \`Arc\` as mutable` — **Hüküm:** DOĞRU — **Kaynak:** `/tmp/opencode/rust-verify/kodlar/e0596.rs` → `error[E0596]: cannot borrow data in an Arc as mutable`; `guide/rust/library/alloc/src/rcs/arc.rs:278` — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `concurrency/src/derinlesme.rs:409` — **Alıntı:** `println!("{mesaj}"); // E0382: borrow of moved value: \`mesaj\`` — **Hüküm:** DOĞRU — **Kaynak:** `rustc 1.99 --explain E0382` ("use of moved value"); `std::sync::mpsc::Sender::send` sahipliği alır — [std::sync::mpsc::Sender](https://doc.rust-lang.org/stable/std/sync/mpsc/struct.Sender.html) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `concurrency/src/derinlesme.rs:412-417` — **Alıntı:** `// E0277: \`MutexGuard<'_, i32>\` cannot be sent between threads safely ... std bu yüzden MutexGuard'ı !Send yapar.` — **Hüküm:** DOĞRU — **Kaynak:** `/tmp/opencode/rust-verify/kodlar/e0277_guard.rs` → `error[E0277]: std::sync::MutexGuard<'_, i32> cannot be sent between threads safely`; [std::sync::MutexGuard](https://doc.rust-lang.org/stable/std/sync/struct.MutexGuard.html) (!Send); `guide/rust/library/std/src/sync/poison/mutex.rs:277-285` — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `concurrency/src/derinlesme.rs:422-424` — **Alıntı:** `Aynı thread, elinde tuttuğu Mutex'i ikinci kez kilitlemeye çalışırsa lock() ya sonsuza dek bekler ya da panik eder (std hangisinin olacağını garanti etmez).` — **Hüküm:** DOĞRU — **Kaynak:** [std::sync::Mutex::lock](https://doc.rust-lang.org/stable/std/sync/struct.Mutex.html#method.lock) ("The exact behavior on locking a mutex in the thread which already holds the lock is left unspecified. However, this function will not return on the second call (it might panic or deadlock, for example)... This function might panic when called if the lock is already held by the current thread") — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `concurrency/src/derinlesme.rs:445-448` — **Alıntı:** `Kilidi tutan thread panik ederse veri yarıda kalmış olabilir; std Mutex'i "zehirli" işaretler ve sonraki lock() çağrıları Err döndürür... \`lock().unwrap_or_else(|zehir| zehir.into_inner())\` ile guard'ı kurtarabilirsin.` — **Hüküm:** DOĞRU — **Kaynak:** [std::sync::Mutex — Poisoning](https://doc.rust-lang.org/stable/std/sync/struct.Mutex.html) ("Poisoning is only advisory: the PoisonError type has an into_inner method"); `guide/rust/library/std/src/sync/poison/mutex.rs` test `zehirli_kilit_kurtarilabilir` (`:547-561`) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `concurrency/src/derinlesme.rs:459-468` — **Alıntı:** `Derlenen güvenli Rust kodunda VERİ YARIŞI olamaz; deadlock, kilit çekişmesi ve mantıksal yarış ... olabilir... Kanal kapanışı sahiplikle olur: tüm gönderenler düşünce alıcı döngüsü biter.` — **Hüküm:** DOĞRU — **Kaynak:** [Rust Reference UB listesi](https://doc.rust-lang.org/reference/behavior-considered-undefined.html), [std::sync::mpsc](https://doc.rust-lang.org/stable/std/sync/mpsc/index.html) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `concurrency/src/derinlesme.rs:489-492` — **Alıntı:** `Send ve Sync'in neden böyle çalıştığı → ileri-send-sync. RwLock, Condvar, deadlock önleme ve async çalışma zamanı → ileri-eszamanlilik-async.` — **Hüküm:** DOĞRU — **Kaynak:** `Cargo.toml:19,16` (workspace üyeleri `learn/advanced-concepts/ileri-send-sync`, `ileri-eszamanlilik-async`); hedef klasörler mevcut — **Öneri:** Yok. — **Önem:** —

### Inferences
- `Arc` re-export düzeltme notu (`:192-193`) doğru olmasına rağmen gövdede hâlâ `library/alloc/src/sync.rs` atfı duruyor; ancak düzeltme notu hemen altında olduğu için okuyucu yanıltılmıyor.
- self-lock (`try_lock` ile gözleme) örneği, std'nin "unspecified ama return etmez" ifadesiyle uyumlu: `WouldBlock` dönmesi beklenen davranış, garanti edilen tek şey ikinci `lock()`'un dönmemesi.

### Gaps
- (Yok — bu klasörde doğrulanamayan iddia kalmadı.)

---

## 2. async-await — Future/poll/Waker, executors, Pin, hata kodları

### Takeaway
Mekanik anlatım (tembel Future, state machine, `.await` desugaring, poll/Waker, `Pin`, `dyn Future`, join, bloklayan kodun zararı) resmî kaynaklarla doğru. Tek doğrulanamayan iddia Book 17'nin "2024 edition'da kitaba eklendi" zamanlaması; yazının kendi iddiası olmasına karşın resmî bir kaynak bulunamadı.

### Cited Findings

- **dosya:satır:** `async-await/src/main.rs:2` — **Alıntı:** `// Rust Book — Bölüm 17: Async/Await (2024 edition'da kitaba eklendi)` — **Hüküm:** DOĞRULANAMADI — **Kaynak:** [Book Ch. 17](https://doc.rust-lang.org/book/ch17-00-async-await.html) sayfası mevcut ve başlık "Fundamentals of Asynchronous Programming: Async, Await, Futures, and Streams"; GitHub commits API `src/ch17-00-async-await.md` için `Ch. 17§00: second round of edits` tarihi `2024-08-24`; [Rust 1.85 blog](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/) 2024 edition'ın `Feb. 20, 2025`'te stabilize edildiğini söylüyor; ikincil [jacko.io](https://jacko.io/async_intro.html) "In February 2025 The Book added a new chapter on async/await" diyor — **resmî olarak bölümün 2024 edition ile eklendiğini söyleyen bir kaynak bulunamadı** — **Öneri:** Parantezi kaldır ya da `Rust Book'da async bölümü mevcut (bkz. ch17-00)` de; edition bağlantısı resmî kaynakla desteklenmiyor. — **Önem:** Düşük

- **dosya:satır:** `async-await/src/main.rs:5-8` — **Alıntı:** `Async, EŞZAMANLILIK (concurrency) içindir: tek thread'de, BEKLEME anlarında (I/O, zamanlayıcı) başka işe geçerek çok işi iç içe yürütmek. Paralellikle ... karıştırma` — **Hüküm:** DOĞRU — **Kaynak:** [Book Ch. 17](https://doc.rust-lang.org/book/ch17-00-async-await.html) ("define a number of tasks that we'd like our program to complete and allow the runtime to choose the best order and manner in which to run them") — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `async-await/src/main.rs:10-12` — **Alıntı:** `std async/await SÖZ DİZİMİNİ ve Future trait'ini verir ama bir ÇALIŞTIRICI (runtime/executor) VERMEZ. Gerçek projede tokio/async-std kullanılır; burada "kökü görmek" için executor'u ELLE, std ile yazıyoruz` — **Hüküm:** DOĞRU — **Kaynak:** [Book Ch. 17](https://doc.rust-lang.org/book/ch17-00-async-await.html) ("Rust's futures, streams, and the async and await syntax ... and the third-party crates that implement asynchronous runtimes: code that manages and coordinates the execution of asynchronous operations") — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `async-await/src/main.rs:30` — **Alıntı:** `tembel (lazy) → isteyen olmadıkça çalışmayan. Future'lar tembeldir.` — **Hüküm:** DOĞRU — **Kaynak:** [std::future::Future — Runtime characteristics](https://doc.rust-lang.org/stable/std/future/trait.Future.html) ("Futures alone are inert; they must be actively polled for the underlying computation to make progress") — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `async-await/src/main.rs:76-78` — **Alıntı:** `std::task::Wake trait'i, ham RawWaker/vtable yazmadan Arc'tan Waker üretmemizi sağlar. Burada "uyandır" = bekleyen (park etmiş) thread'i çöz (unpark).` — **Hüküm:** DOĞRU — **Kaynak:** [std::task::Wake](https://doc.rust-lang.org/stable/std/task/trait.Wake.html) ("A Wake implementation can be used to create a Waker via Waker::from"); kod `Waker::from(Arc::new(ThreadWaker(...)))` derleniyor — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `async-await/src/main.rs:101-108` — **Alıntı:** `block_on: bir future'ı bitene kadar DÖNGÜde poll eder. Poll::Pending gelince thread'i park eder (boşa dönmez); future kendini uyandırınca (wake) unpark olup yeniden poll edilir... Rust'ın standart kütüphanesi bu "bakan kişiyi" vermez` — **Hüküm:** DOĞRU — **Kaynak:** [std::future::Future — poll](https://doc.rust-lang.org/stable/std/future/trait.Future.html) ("This method does not block if the value is not ready. Instead, the current task is scheduled to be woken up when it's possible to make further progress by polling again"); [Book Ch. 17](https://doc.rust-lang.org/book/ch17-00-async-await.html) (third-party runtimes) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `async-await/src/main.rs:125-147` — **Alıntı:** `Çok görevli mini executor: aynı thread'de round-robin poll... EŞZAMANLILIK (interleaving) tek thread'de.` — **Hüküm:** DOĞRU — **Kaynak:** [std::future::Future — Runtime characteristics](https://doc.rust-lang.org/stable/std/future/trait.Future.html) ("each time the current task is woken up, it should actively re-poll pending futures that it still has an interest in") — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `async-await/src/main.rs:219-244` — **Alıntı:** `async (fn ya da blok önünde): gövdeyi ÇALIŞTIRMAZ... async fn f() -> T'nin gerçek dönüş tipi impl Future<Output = T>'dir... .await ... loop { match fut.poll(cx) { Poll::Ready(v) => break v, Poll::Pending => return Poll::Pending } }` — **Hüküm:** DOĞRU — **Kaynak:** [std::future::Future](https://doc.rust-lang.org/stable/std/future/trait.Future.html), [Rust Reference — async functions](https://doc.rust-lang.org/reference/items/functions.html#async-functions) ("An async function returns an anonymous future type implementing Future"); `rustc 1.99 --explain E0728` (await yalnız async içinde) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `async-await/src/main.rs:292-314` — **Alıntı:** `async fn/async {} bir STATE MACHINE'e derlenir... Future trait'i tek metotludur: fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output>... Pin NEDEN? ... self-referential... Neden RUNTIME gerekir? std, Future'ı ve async/await'i TANIMLAR ama onu KOŞTURAN döngüyü vermez.` — **Hüküm:** DOĞRU — **Kaynak:** [std::future::Future](https://doc.rust-lang.org/stable/std/future/trait.Future.html) (trait imzası birebir: `fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>`), [std::pin::Pin](https://doc.rust-lang.org/stable/std/pin/struct.Pin.html), [Book Ch. 17](https://doc.rust-lang.org/book/ch17-00-async-await.html) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `async-await/src/derinlesme.rs:63-80` — **Alıntı:** `JavaScript'te fetch() çağrıldığı an istek yola çıkar; Promise yalnız SONUCU temsil eder. Rust'ta getir() çağrısı hiçbir şey yapmaz; dönen Future İŞİN KENDİSİDİR... Bedeli: .awaiti unutmak derleme hatası değil, sadece bir uyarıdır.` — **Hüküm:** DOĞRU — **Kaynak:** [std::future::Future](https://doc.rust-lang.org/stable/std/future/trait.Future.html) ("Futures alone are inert"); `/tmp/opencode/rust-verify` reprosunda `let f = async {1};` → `warning: unused variable` / kullanılmayan Future uyarısı — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `async-await/src/derinlesme.rs:98-105` — **Alıntı:** `warning: unused implementer of \`Future\` that must be used / note: futures do nothing unless you \`.await\` or poll them ... let n: u32 = sayi_getir(); // ✗ E0308: mismatched types (u32 beklenirken Future)` — **Hüküm:** DOĞRU — **Kaynak:** `/tmp/opencode/rust-verify/kodlar/e0308.rs` → `error[E0308]: mismatched types` (`async fn n()->u32; let x:u32 = n();`); [unused_must_use lint](https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#unused-must-use) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `async-await/src/derinlesme.rs:150-154` — **Alıntı:** `Gecikme her Pending'de kendini hemen uyandırdığı için park etmeye gerek yok; uyandırma bilgisini kullanmayan en basit waker'ı (Waker::noop) veriyoruz... Hiç uyumadan durmadan baktığı için zile ihtiyacı yoktur: \`Waker::noop()\`.` — **Hüküm:** DOĞRU — **Kaynak:** [std::task::Waker::noop](https://doc.rust-lang.org/stable/std/task/struct.Waker.html#method.noop); `tur_sayarak_calistir` repro olarak derlendi — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `async-await/src/derinlesme.rs:180-198` — **Alıntı:** `Neden biten işe bir daha bakılmaz? Ready demiş bir Future'ı yeniden poll etmenin sonucu belirsizdir; paniklemesi bile mümkündür... (sözleşme: Ready'den sonra poll'un ne yapacağı belirsizdir — panik edebilir ya da sonsuza dek Pending dönebilir; UB değildir).` — **Hüküm:** DOĞRU — **Kaynak:** [std::future::Future — Panics](https://doc.rust-lang.org/stable/std/future/trait.Future.html) ("Once a future has completed (returned Ready from poll), calling its poll method again may panic, block forever, or cause other kinds of problems; the Future trait places no requirements on the effects of such a call. However, as the poll method is not marked unsafe, Rust's usual rules apply: calls must never cause undefined behavior") — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `async-await/src/derinlesme.rs:285-287` — **Alıntı:** `\`&str\` parametresi await boyunca ödünçte kalır; async fn'in ürettiği future bu referansı İÇİNDE taşır (dönüş türü gizli bir lifetime alır). Çağıran, future yaşadıkça \`yol\`u canlı tutmak zorundadır — derleyici denetler.` — **Hüküm:** DOĞRU — **Kaynak:** [Rust Reference — async lifetime capture](https://doc.rust-lang.org/reference/items/functions.html#async-functions), [Book Ch. 17](https://doc.rust-lang.org/book/ch17-00-async-await.html) (async fn borrows); `/tmp/opencode/rust-verify/kodlar/e0597_dyn.rs` benzer ödünç-ömür hatası veriyor — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `async-await/src/derinlesme.rs:300-307` — **Alıntı:** `her async fn çağrısı FARKLI ve adsız bir tür ürettiği için (ve Vec tek tür tuttuğu için) onları \`dyn Future\`a siliyoruz. dyn boyutsuzdur → Box; poll edilecek → Pin. \`Pin<Box<dyn Future<Output = T> + 'a>>\` kalıbı buradan gelir.` — **Hüküm:** DOĞRU — **Kaynak:** [Rust Reference — Trait objects](https://doc.rust-lang.org/reference/types/trait-object.html) ("trait objects are dynamically sized types... used behind some type of pointer; for example &dyn SomeTrait or Box<dyn SomeTrait>"), [std::future::Future](https://doc.rust-lang.org/stable/std/future/trait.Future.html) ("This trait is dyn compatible") — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `async-await/src/derinlesme.rs:339` — **Alıntı:** `\`collect::<Result<Vec<_>, _>>()\` ilk Err'de durur (error-handling crate'i).` — **Hüküm:** DOĞRU — **Kaynak:** `learn/error-handling/Cargo.toml:2` (`name = "error-handling"`), `Cargo.toml:12` (`"learn/*"` workspace üyesi); [std::Iterator::collect — Result](https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html#method.collect) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `async-await/src/derinlesme.rs:384-399` — **Alıntı:** `pin!(fut) → bulunduğun kapsamın/async fn'in içine sabitler. Tahsis YOK; ama değer o kapsamdan dışarı çıkamaz... Box::pin(fut) → heap'e koyup sabitler. Bir tahsis maliyeti var; karşılığında kutuyu istediğin yere taşıyabilirsin... dyn Future'a silebilirsin.` — **Hüküm:** DOĞRU — **Kaynak:** [std::pin::pin!](https://doc.rust-lang.org/stable/std/macro.pin.html), [Box::pin](https://doc.rust-lang.org/stable/std/boxed/struct.Box.html#method.pin) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `async-await/src/derinlesme.rs:409` — **Alıntı:** `E0599: no method named \`poll\` found for opaque type \`impl Future<...>\` / help: consider pinning the expression` — **Hüküm:** DOĞRU — **Kaynak:** `/tmp/opencode/rust-verify/kodlar/e0599.rs` → `error[E0599]: no method named poll found for async block ... help: consider pinning the expression` — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `async-await/src/derinlesme.rs:438` — **Alıntı:** `E0728: \`await\` is only allowed inside \`async\` functions and blocks` — **Hüküm:** DOĞRU — **Kaynak:** `/tmp/opencode/rust-verify/kodlar/e0728.rs` → birebir mesaj; `rustc --explain E0728` — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `async-await/src/derinlesme.rs:444` — **Alıntı:** `E0373: async block may outlive the current function, but it borrows \`s\`` — **Hüküm:** DOĞRU — **Kaynak:** `rustc 1.99 --explain E0373` ("closure may outlive the current function, but it borrows"; async bloklar için aynı hata familyası); [Rust Reference — async blocks](https://doc.rust-lang.org/reference/expressions/block-expr.html#async-blocks) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `async-await/src/derinlesme.rs:450-453` — **Alıntı:** `E0733: recursion in an async fn requires boxing... ✓ Özyineli çağrıyı kutula: \`1 + Box::pin(say(n - 1)).await\`.` — **Hüküm:** DOĞRU — **Kaynak:** `/tmp/opencode/rust-verify/kodlar/e0733.rs` → `error[E0733]: recursion in an async fn requires boxing`; `rustc --explain E0733` — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `async-await/src/derinlesme.rs:455-460` — **Alıntı:** `async kod içinde thread'i bloklamak: std::thread::sleep ... sleep Pending DÖNMEZ; executor'un thread'ini gerçekten uyutur. O bir saniye boyunca aynı thread'deki bütün öteki görevler de durur` — **Hüküm:** DOĞRU — **Kaynak:** [Book Ch. 17 — Async vs threads](https://doc.rust-lang.org/book/ch17-00-async-await.html), [std::thread::sleep](https://doc.rust-lang.org/stable/std/thread/fn.sleep.html) ("blocks the current thread"); `async-await/src/derinlesme.rs:431-432` çalışan karşılaştırma — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `async-await/src/derinlesme.rs:465-473` — **Alıntı:** `Future = henüz yapılmamış iş. Üretmek bedava, poll edilmezse hiç çalışmaz, düşürülürse iptal olur... Eşzamanlılık ≠ paralellik: yukarıdaki kazançların hepsi tek thread'de elde edildi.` — **Hüküm:** DOĞRU — **Kaynak:** [std::future::Future](https://doc.rust-lang.org/stable/std/future/trait.Future.html) ("Futures alone are inert"), [Book Ch. 17](https://doc.rust-lang.org/book/ch17-00-async-await.html) — **Öneri:** Yok. — **Önem:** —

### Inferences
- "2024 edition'da eklendi" ifadesi muhtemelen 2024 edition stabilizasyonu (Şubat 2025) ile Book bölümünün eklenme zamanını (Ağustos 2024 / Şubat 2025) birbirine bağlayan bir kolaylaştırma; iki olay resmî olarak aynı kaynakta ilişkilendirilmemiş.
- Future sözleşmesi ("Ready'den sonra tekrar poll UB değil ama belirsiz") std dokümanındaki "may panic, block forever... must never cause undefined behavior" cümlesiyle birebir uyumlu; derste bu ayrım doğru verilmiş.

### Gaps
- Book 17'nin hangi resmî duyuru/PR ile "2024 edition"a bağlandığı bulunamadı; yalnızca ikincil kaynak (jacko.io) Şubat 2025 diyor.
- `check_binding_is_irrefutable` için eski satır numarası iddiası olmadığından, `~60` ifadesinin hangi sürüme ait olduğu doğrulanamadı (bkz. bölüm 4).

---

## 3. oop — trait object, dyn ömür, upcasting, type-state, gizlilik

### Takeaway
OOP klasörünün teknik içeriği (trait object = fat pointer/vtable, `dyn` varsayılan `'static` sınırı, upcasting 1.86, dyn-uyumsuzluk, blanket impl ile klonlama) doğru. İki sorun var: (a) "Böl. 15" vtable atfı düzeltme notuna rağmen gövdede kalmış, (b) E0616/gizlilik iddiası aynı modülde yanıltıcı — düzeltme notu bunu doğru biçimde açıklıyor.

### Cited Findings

- **dosya:satır:** `oop/src/main.rs:2` — **Alıntı:** `// Rust Book — Bölüm 18: Object-Oriented Programming Features` — **Hüküm:** DOĞRU — **Kaynak:** [Rust Book Ch. 18](https://doc.rust-lang.org/book/ch18-00-oop.html) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `oop/src/main.rs:5-10` — **Alıntı:** `Rust OOP'nin bazı parçalarını (nesne = veri + davranış; encapsulation) destekler ama KALITIM (inheritance) YOKTUR. Yerine: davranış paylaşımı için trait + varsayılan metot, kod paylaşımı için KOMPOZİSYON... type-state: geçersiz durumu DERLEME ANINDA imkânsız kıl.` — **Hüküm:** DOĞRU — **Kaynak:** [Rust Book Ch. 18](https://doc.rust-lang.org/book/ch18-00-oop.html) ("Object-Oriented Programming ... Rust ... encapsulate ... no inheritance") — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `oop/src/main.rs:13-14` — **Alıntı:** `↳ DÜZELTME NOTU (son gözden geçirme): dispatch/vtable konusu Böl. 10'da (generics-traits-lifetimes) ve ileri-sized-dst'de işlendi; Böl. 15 (smart-pointers) bu konuyu içermiyor.` — **Hüküm:** DOĞRU — **Kaynak:** `learn/generics-traits-lifetimes/src/derinlesme.rs:247,260-261` (vtable/dyn anlatımı mevcut); `Cargo.toml:20` (`ileri-sized-dst` workspace üyesi); aşağıda `:178-180` bu notla çelişiyor — **Öneri:** Yok (düzeltme notu doğru); aşağıdaki gövde atfını düzelt. — **Önem:** —

- **dosya:satır:** `oop/src/main.rs:70-71,108-112` — **Alıntı:** `Dikkat: Rust'ta gizlilik TÜRE değil MODÜLE göredir. Aynı modüldeki kod ... private alana erişebilir... // o.liste.push(999); // ✗ E0616: field \`liste\` is private ↳ DÜZELTME NOTU: bu satır BU DOSYADA açılırsa DERLENİR... E0616 yalnız tür ayrı bir \`mod\` içinde tanımlanıp DIŞINDAN erişilince çıkar (rustc ile doğrulandı).` — **Hüküm:** YANILTICI (gövde `:108` ifadesi) / DOĞRU (`:70-71` ve `:109-112` düzeltme notu) — **Kaynak:** [Rust Reference — Visibility and privacy](https://doc.rust-lang.org/reference/visibility-and-privacy.html) ("Privacy is ... module-based"); `/tmp/opencode/rust-verify/kodlar` reprosunda ayrı modülden alan erişimi `error[E0616]: field \`liste\` of struct \`OrtalamaListe\` is private`; `rustc --explain E0616` — **Öneri:** `:108`'deki `✗ E0616` yorumunu ya kaldır ya da `// (bu dosyada derlenir; E0616 için türü ayrı moda koy)` yap; düzeltme notu zaten doğru. — **Önem:** Orta (tek başına okunursa yanıltıcı; düzeltme notu var)

- **dosya:satır:** `oop/src/main.rs:117-125,152-160` — **Alıntı:** `TRAIT OBJECT ile polimorfizm — farklı türleri TEK listede, çalışma anında dispatch... Vec<Box<dyn Ciz>> ... dinamik dispatch (vtable)` — **Hüküm:** DOĞRU — **Kaynak:** [Rust Reference — Trait objects](https://doc.rust-lang.org/reference/types/trait-object.html) ("Calling a method on a trait object results in virtual dispatch at runtime ... a function pointer is loaded from the trait object vtable and invoked indirectly") — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `oop/src/main.rs:178-180` — **Alıntı:** `Buton + OnayKutusu'yu aynı Vec'te tutmak için trait object (dyn) şart. Bedel: vtable dolaylaması (Böl. 10/15'te kök neden).` — **Hüküm:** YANLIŞ (bölüm atfı; `:13-14` düzeltme notuyla çelişiyor) — **Kaynak:** `:13-14` düzeltme notu ("Böl. 15 (smart-pointers) bu konuyu içermiyor"); `learn/smart-pointers` kaynaklarında vtable anlatımı yok — **Öneri:** `(Böl. 10'da ve ileri-sized-dst'de kök neden)` yap; Böl. 15 atfını kaldır. — **Önem:** Düşük

- **dosya:satır:** `oop/src/main.rs:183-297` — **Alıntı:** `TYPE-STATE — "state pattern"i tür sistemiyle ifade et ... Geçiş, eski değeri TÜKETİP (self alır) yeni tür döndürür. Böylece yanlış durumda yanlış metodu çağırmak DERLENMEZ` — **Hüküm:** DOĞRU — **Kaynak:** Rust tür sistemi / sahiplik kuralları; kod `rustc 1.99` ile derleniyor, `.icerik()` yalnız `Yayin`'de tanımlı (`:241-246`), `self` tüketen geçişler (`:222-227`) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `oop/src/main.rs:288-292` — **Alıntı:** `TRAIT OBJECT = fat pointer: Box<dyn Ciz>, (veri-pointer, vtable-pointer) ikilisidir... Trait'in dyn olabilmesi için "dyn compatible" olması şart (Böl. 10, E0038).` — **Hüküm:** DOĞRU — **Kaynak:** [Rust Reference — Trait objects](https://doc.rust-lang.org/reference/types/trait-object.html) ("a pointer to an instance ... a virtual method table ... dyn compatible base trait"), `rustc --explain E0038`, `/tmp/opencode/rust-verify/kodlar/e0038.rs` — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `oop/src/derinlesme.rs:350-359` — **Alıntı:** `Her \`dyn Trait\` aslında \`dyn Trait + 'ömür\` biçimindedir; yazmazsan derleyici doldurur: Box<dyn Sablon> = Box<dyn Sablon + 'static> → içine REFERANS tutan tür konamaz ... aşağıdaki \`+ '_\` budur.` — **Hüküm:** DOĞRU — **Kaynak:** [Rust Reference — Trait object lifetime bounds](https://doc.rust-lang.org/reference/types/trait-object.html#trait-object-lifetime-bounds) ("Since a trait object can contain references, the lifetimes of those references need to be expressed as part of the trait object... There are defaults that allow this lifetime to usually be inferred"); `/tmp/opencode/rust-verify/kodlar/dyn_static.rs` → `returning this value requires that '1 must outlive 'static`, `help: add Box<dyn S + '_>` — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `oop/src/derinlesme.rs:376-379` — **Alıntı:** `error: lifetime may not live long enough (bu hatanın E-kodu yok; "returning this value requires that \`'1\` must outlive \`'static\`" der) ... E0597: \`s\` does not live long enough` — **Hüküm:** DOĞRU — **Kaynak:** `/tmp/opencode/rust-verify/kodlar/dyn_static.rs` (E-kodsuz lifetime hatası, mesaj birebir), `/tmp/opencode/rust-verify/kodlar/e0597_dyn.rs` → `error[E0597]: s does not live long enough` — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `oop/src/derinlesme.rs:397-398` — **Alıntı:** `Upcasting: \`&dyn Kanal\` → \`&dyn Debug\` / \`&dyn Any\` ... Rust 1.86'dan beri kararlı; öncesinde her trait'e elle \`fn as_any(&self) -> &dyn Any\` yazılırdı.` — **Hüküm:** DOĞRU — **Kaynak:** [Rust 1.86.0 blog](https://blog.rust-lang.org/2025/04/03/Rust-1.86.0/) ("trait upcasting coercion is now stable"); `dyn Any` downcast için [std::any::Any](https://doc.rust-lang.org/stable/std/any/trait.Any.html) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `oop/src/derinlesme.rs:496-518` — **Alıntı:** `Clone::clone \`Self\` döndürür → \`trait Kanal: Clone\` yazarsan trait dyn-uyumsuz olur ... Çözüm: Self yerine KUTU döndüren, dyn-uyumlu bir yardımcı metot ... blanket impl ... impl Clone for Box<dyn Kanal>` — **Hüküm:** DOĞRU — **Kaynak:** [Rust Reference — Trait objects / dyn compatibility](https://doc.rust-lang.org/reference/types/trait-object.html), `rustc --explain E0038`, `/tmp/opencode/rust-verify/kodlar/e0038.rs` (`trait K { fn f(&self) -> Self; }` → `error[E0038]: the trait K is not dyn compatible`) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `oop/src/derinlesme.rs:553-559` — **Alıntı:** `Self dönen ya da generic metotlu trait'i dyn yapmak ... E0038: the trait \`Kanal\` is not dyn compatible ... ✓ O metodu \`where Self: Sized\` ile dyn'in dışında bırak ... "the \`etiketli\` method cannot be invoked on a trait object" (E-kodu yok).` — **Hüküm:** DOĞRU — **Kaynak:** `/tmp/opencode/rust-verify/kodlar/e0038.rs`; `rustc --explain E0038`; [Reference — dyn compatibility](https://doc.rust-lang.org/reference/types/trait-object.html) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `oop/src/derinlesme.rs:561-564` — **Alıntı:** `\`Vec<dyn Kanal>\` ... E0277: the size for values of type \`dyn Kanal\` cannot be known at compilation time ... dyn bir DST'dir, yalnız işaretçi ardında durur (ileri-sized-dst).` — **Hüküm:** DOĞRU — **Kaynak:** `/tmp/opencode/rust-verify/kodlar/e0277_unsized.rs` → birebir mesaj "the size for values of type `dyn K` cannot be known at compilation time"; [Reference — Trait objects (DST)](https://doc.rust-lang.org/reference/types/trait-object.html) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `oop/src/derinlesme.rs:566-571` — **Alıntı:** `\`let k2 = kutu.clone()\` ... E0599: the method \`clone\` exists for struct \`Box<(dyn Kanal + 'static)>\`, but its trait bounds were not satisfied ... \`trait Kanal: Clone\` → E0038` — **Hüküm:** DOĞRU — **Kaynak:** `/tmp/opencode/rust-verify/kodlar/e0599_clone2.rs` → `error[E0599]: the method clone exists for struct Box<dyn K>, but its trait bounds were not satisfied` ("`dyn K: Clone` or `dyn K: Sized`"); `rustc --explain E0599` — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `oop/src/derinlesme.rs:575` — **Alıntı:** `dyn; Send/Sync ve ömür bilgisini kendiliğinden taşımaz: \`+ Send\`, \`+ 'a\` ile sen yazarsın.` — **Hüküm:** DOĞRU — **Kaynak:** [Rust Reference — Trait objects](https://doc.rust-lang.org/reference/types/trait-object.html) ("Trait objects implement the base trait, its auto traits... dyn Trait, dyn Trait + Send, dyn Trait + 'static ... There must be ... no more than one lifetime"); [Rust Reference — Auto traits](https://doc.rust-lang.org/reference/special-types-traits.html#auto-traits) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `oop/src/derinlesme.rs:593` — **Alıntı:** `Beklenen: enum'da \`gonder\` içindeki match derlenmez (E0004) ve bir kol istenir; dyn'de sıfır yer.` — **Hüküm:** DOĞRU — **Kaynak:** `rustc --explain E0004` ("non-exhaustive patterns: ... not covered"); `/tmp/opencode/rust-verify` E0004 repro (`match` ile `None` kolu eksik → `error[E0004]: non-exhaustive patterns`); [Book Ch. 19](https://doc.rust-lang.org/book/ch19-00-patterns.html) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `oop/src/derinlesme.rs:596-599` — **Alıntı:** `patterns (Böl. 19): §2'deki enum çözümünün gücü \`match\`ten gelir ... ileri-sized-dst: \`Box<dyn Kanal>\`'ın neden iki word olduğu ... orada.` — **Hüküm:** DOĞRU — **Kaynak:** `Cargo.toml:12,20` (`learn/*`, `ileri-sized-dst` workspace üyeleri); hedef dosyalar mevcut — **Öneri:** Yok. — **Önem:** —

### Inferences
- `:108`'deki E0616 yorumu, düzeltme notu (`:109-112`) ile aynı satır bloğunda durduğu için aslında "yanlış iddia + hemen düzeltmesi" şeklinde okunuyor; tek başına kopyalanırsa yanıltıcı. Rapor yazarken "modül bazlı gizlilik" vurgusu öne çıkarılmalı.
- "Böl. 15" atfı, `:13-14`'teki düzeltme notunun kapsamına girmesine rağmen gövdeye yansıtılmamış — bu, sibling notlarda ("ileri_unsafe_async.md") gözlenen "düzeltme notu gövdeye işlenmemiş" deseninin aynısı.

### Gaps
- `:559`'daki "the `etiketli` method cannot be invoked on a trait object (E-kodu yok)" mesajı için tam repro çalıştırılmadı; ancak `where Self: Sized` yönteminin dyn üzerinde çağrılamayacağı Reference ile doğru.

---

## 4. patterns — reddedilebilirlik, exhaustiveness, kaynak satır atfı

### Takeaway
Desenler klasörünün iddiaları doğru; tek somut hata `check_binding_is_irrefutable` fonksiyonunun kaynak satır numarası (`~60` → güncel ağaçta `:657`). E0005, E0004 ve kısmi taşıma iddiaları reprolandı.

### Cited Findings

- **dosya:satır:** `patterns/src/main.rs:2` — **Alıntı:** `// Rust Book — Bölüm 19: Patterns and Matching (desenler ve eşleştirme)` — **Hüküm:** DOĞRU — **Kaynak:** [Rust Book Ch. 19](https://doc.rust-lang.org/book/ch19-00-patterns.html) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `patterns/src/main.rs:5-6` — **Alıntı:** `Desen (pattern) = bir değerin YAPISINA göre eşleştirme + içindeki parçaları bağlama (destructuring). match'ten ibaret değil; dilin birçok yerinde aynı dilbilgisi çalışır.` — **Hüküm:** DOĞRU — **Kaynak:** [Rust Book Ch. 19](https://doc.rust-lang.org/book/ch19-00-patterns.html), [Rust Reference — Patterns](https://doc.rust-lang.org/reference/patterns.html) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `patterns/src/main.rs:8-9` — **Alıntı:** `Doğrulanmış (rustc ile): \`let\` REDDEDİLEMEZ (irrefutable) desen ister; Some(x) gibi reddedilebilir desen koyunca → E0005, derleyici let-else önerir.` — **Hüküm:** DOĞRU — **Kaynak:** `/tmp/opencode/rust-verify/kodlar/e0005` repro → `error[E0005]: refutable pattern in local binding`, `pattern None not covered`, `let bindings require an "irrefutable pattern"`, `help: let...else`; `rustc --explain E0005`; [Book Ch. 19](https://doc.rust-lang.org/book/ch19-00-patterns.html) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `patterns/src/main.rs:67` — **Alıntı:** `// let (c, d) = (1, 2, 3); // ✗ derlenmez: iki gözlü desen, üç parçalı değer` — **Hüküm:** DOĞRU — **Kaynak:** `/tmp/opencode/rust-verify/kodlar/e0527.rs` → `error[E0527]: pattern requires 2 elements but array has 3` (benzer tuple arity hatası); [Reference — Tuple patterns](https://doc.rust-lang.org/reference/patterns.html#tuple-patterns) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `patterns/src/main.rs:126` — **Alıntı:** `// let Some(n) = veri; // ✗ derlenmez: ya None gelirse?` — **Hüküm:** DOĞRU — **Kaynak:** E0005 repro (yukarıda); [Rust Reference — Refutability](https://doc.rust-lang.org/reference/patterns.html#refutability) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `patterns/src/main.rs:262-264` — **Alıntı:** `E0005 "refutable pattern in local binding", derleyici let-else önerir. Kontrol rustc_mir_build/src/thir/pattern/check_match.rs (check_binding_is_irrefutable, ~60).` — **Hüküm:** YANLIŞ (satır numarası) — **Kaynak:** `guide/rust/compiler/rustc_mir_build/src/thir/pattern/check_match.rs:657` → `fn check_binding_is_irrefutable(`; `:30` (`check_match` girişi); dosya yolu doğru — **Öneri:** `~60` yerine `~657` (ya da `check_binding_is_irrefutable, :657`) yaz. — **Önem:** Düşük

- **dosya:satır:** `patterns/src/main.rs:268-274` — **Alıntı:** `EXHAUSTIVENESS ... USEFULNESS ... aynı analiz ailesindendir (rustc_pattern_analysis). Erişilemez kol → "unreachable pattern" uyarısı... GUARD'lar ... desen analizinin DIŞINDADIR: derleyici bir guard'ın ne zaman doğru olacağını bilemez` — **Hüküm:** DOĞRU — **Kaynak:** `guide/rust/compiler/rustc_pattern_analysis` (klasör mevcut); `guide/rust/compiler/rustc_mir_build/src/thir/pattern/check_match.rs`; [Rust Reference — exhaustiveness / guard](https://doc.rust-lang.org/reference/patterns.html) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `patterns/src/derinlesme.rs:2` — **Alıntı:** `// Derinleşme — Bölüm 19: Patterns (pekiştirme: gerçekçi senaryo, sık hatalar, alıştırmalar)` — **Hüküm:** DOĞRU — **Kaynak:** [Rust Book Ch. 19](https://doc.rust-lang.org/book/ch19-00-patterns.html) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `patterns/src/derinlesme.rs:5-9` — **Alıntı:** `main.rs desen sözdiziminin araçlarını tek tek gösterdi (literal, \`|\`, aralık, \`..\`, guard, \`@\`, let-else). Bu dosya onları birlikte çalıştırır ve main.rs'in değinmediği üç konuyu ekler: • dilim desenleri ... • bağlama modları (move / \`ref\` / \`ref mut\`, match ergonomics) ... • iç içe yapı çözme, \`matches!\` ve bütünlük denetiminin refactoring'i nasıl güvenli kıldığı` — **Hüküm:** DOĞRU — **Kaynak:** [Rust Reference — Patterns](https://doc.rust-lang.org/reference/patterns.html) (slice, binding modes, match ergonomics); [Rust Book Ch. 18/19](https://doc.rust-lang.org/book/ch19-00-patterns.html) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `patterns/src/derinlesme.rs:214` — **Alıntı:** `// println!("{k:?}"); // ✗ derlenmez: k'nin bir parçası artık yerinde değil` — **Hüküm:** DOĞRU — **Kaynak:** `/tmp/opencode/rust-verify/kodlar/e0382_partial.rs` → `error[E0382]: borrow of partially moved value: k` — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `patterns/src/derinlesme.rs:336-340` — **Alıntı:** `Guard yerine aralık: üç kol tüm i32'leri kapsıyor ve derleyici bunu KANITLAYABİLİYOR. unsigned_abs: \`abs()\` i32::MIN'de taşar (karşılığı i32'ye sığmaz) ve panikler.` — **Hüküm:** DOĞRU — **Kaynak:** [Rust Reference — Range patterns](https://doc.rust-lang.org/reference/patterns.html#range-patterns), [i32::unsigned_abs](https://doc.rust-lang.org/stable/std/primitive.i32.html#method.unsigned_abs), [i32::abs overflow](https://doc.rust-lang.org/stable/std/primitive.i32.html#method.abs) ("Attempting to take the absolute value of the most negative integer (i32::MIN) is not defined... will panic in debug") — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `patterns/src/derinlesme.rs:421` — **Alıntı:** `// \`[] =>\` kolunu silersen // ✗ derlenmez: boş dilim açıkta kalır` — **Hüküm:** DOĞRU — **Kaynak:** `rustc --explain E0004`, Book Ch. 19 exhaustiveness; [Rust Reference — Slice patterns](https://doc.rust-lang.org/reference/patterns.html#slice-patterns) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `patterns/src/derinlesme.rs:491,496` — **Alıntı:** `// match komut { Komut::Listele => .., _ => .. } // derlenir; yeni varyant \`_\`a düşer ... E0004 ile derlenmez; derleyicinin hata listesi "güncellemen gereken yerler" listesidir.` — **Hüküm:** DOĞRU — **Kaynak:** `rustc --explain E0004`, `/tmp/opencode/rust-verify` E0004 repro; [Book Ch. 19](https://doc.rust-lang.org/book/ch19-00-patterns.html) — **Öneri:** Yok. — **Önem:** —

### Inferences
- `check_match.rs` satır kayması büyük olasılıkla farklı bir rustc sürümünden kopyalanmış bir atıf; dosya yolu ve fonksiyon adı doğru olduğu için yalnızca sayıyı güncellemek yeterli.
- "Guard'lar analiz dışında" ifadesi, derleyicinin guard koşulunu değerlendirmediği anlamında doğru; bu yüzden guard'lı kollar exhaustiveness için `_` kolunun yerini almıyor.

### Gaps
- `~60` ifadesinin hangi eski rustc sürümüne ait olduğu doğrulanamadı.

---

## 5. advanced-features — unsafe'ın 5 işlemi, E0133, newtype/never type/makrolar

### Takeaway
Advanced-features klasöründe doğrulanamayan veya yanlış bir iddia yok. Unsafe'ın 5 süper gücü, E0133, newtype orphan-rule avantajı, never type `!`, DST/Sized ve makro aileleri resmî kaynaklarla uyumlu.

### Cited Findings

- **dosya:satır:** `advanced-features/src/main.rs:2` — **Alıntı:** `// Rust Book — Bölüm 20: Advanced Features (kitabın son kavramsal bölümü)` — **Hüküm:** DOĞRU — **Kaynak:** [Rust Book Ch. 20](https://doc.rust-lang.org/book/ch20-00-advanced-features.html) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `advanced-features/src/main.rs:5-6` — **Alıntı:** `Beş başlık: unsafe Rust, gelişmiş trait'ler, gelişmiş tipler, fonksiyon pointer'ları + closure döndürme, ve makrolar. Ortak tema: güçlü araçlar + bunları GÜVENLİ soyutlamalara sarmak.` — **Hüküm:** DOĞRU — **Kaynak:** [Rust Book Ch. 20](https://doc.rust-lang.org/book/ch20-00-advanced-features.html) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `advanced-features/src/main.rs:8-9` — **Alıntı:** `Doğrulanmış (rustc ile): unsafe fonksiyon/raw pointer deref'i \`unsafe\` blok dışında kullanmak → E0133. Ayrıntı: DERLEYİCİ PERSPEKTİFİ (altta).` — **Hüküm:** DOĞRU — **Kaynak:** `rustc 1.99 --explain E0133` ("dereference of raw pointer is unsafe and requires unsafe block"), `:115` satırındaki repro yorumu — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `advanced-features/src/main.rs:85-90,109-115` — **Alıntı:** `unsafe yalnız birkaç işlemi açar; öteki kurallar sürer... Raw pointer: oluşturmak güvenli, DEREFERENCE etmek unsafe (null/dangling olabilir)... \`*r; // ✗ unsafe bloksuz → E0133\`` — **Hüküm:** DOĞRU — **Kaynak:** [Rust Book Ch. 20 — Unsafe Rust](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html) ("raw pointers ... dereferencing a raw pointer ... requires unsafe"), `rustc --explain E0133` — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `advanced-features/src/main.rs:254-270` — **Alıntı:** `UNSAFE'İN 5 SÜPER GÜCÜ: (1) raw pointer dereference, (2) unsafe fonksiyon/metot çağırma, (3) değiştirilebilir static ... (4) unsafe trait uygulama, (5) union alanı okuma. BUNLAR DIŞINDAKİ her şey unsafe blokta da normal kurallara tabidir; unsafe borrow checker'ı KAPATMAZ... E0133 ... Denetim: rustc_mir_build/src/check_unsafety.rs. ... ihlal = tanımsız davranış (UB: raw pointer null/dangling/hizasız olabilir, aliasing'i çiğneyip veri yarışı üretebilir — hepsi UB).` — **Hüküm:** DOĞRU — **Kaynak:** [Rust Book Ch. 20 — Unsafe Superpowers](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html#unsafe-superpowers) (5 işlem birebir), [Rust Reference — Undefined Behavior](https://doc.rust-lang.org/reference/behavior-considered-undefined.html), `guide/rust/compiler/rustc_mir_build/src/check_unsafety.rs` (dosya mevcut), `rustc --explain E0133` — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `advanced-features/src/main.rs:272-274` — **Alıntı:** `DEFAULT GENERIC TYPE PARAM: Add<Rhs = Self> gibi... Operatörler aslında trait'tir (+ → Add::add)` — **Hüküm:** DOĞRU — **Kaynak:** [std::ops::Add](https://doc.rust-lang.org/stable/std/ops/trait.Add.html) (`pub trait Add<Rhs = Self>`), [Rust Reference — Operators](https://doc.rust-lang.org/reference/expressions/operator-expr.html) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `advanced-features/src/main.rs:276-278` — **Alıntı:** `NEWTYPE: tek alanlı tuple struct. İki işi var: (a) orphan rule'u aşmak ... (b) tip güvenliği ... Runtime maliyeti yok — derleyici sarmalı optimize eder.` — **Hüküm:** DOĞRU — **Kaynak:** [Rust Reference — Orphan rules](https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules), [Rust Book Ch. 20 — Newtype Pattern](https://doc.rust-lang.org/book/ch20-02-advanced-traits.html#using-the-newtype-pattern-to-implement-external-traits-for-external-types) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `advanced-features/src/main.rs:280-284` — **Alıntı:** `NEVER TYPE \`!\`: hiçbir değer DÖNDÜRMEYEN ifadelerin türü (panic!, loop{}, continue, return). \`!\` her türe zorlanabilir (coerce)... DST: str, [T], dyn Trait gibi boyutu derleme anında bilinmeyen tipler; bunlar hep bir pointer ARKASINDA ... Varsayılan \`T: Sized\` kısıtı bu yüzden vardır; \`T: ?Sized\` ile gevşetilir.` — **Hüküm:** DOĞRU — **Kaynak:** [Rust Reference — Never type](https://doc.rust-lang.org/reference/types/never.html), [Rust Reference — Dynamically sized types](https://doc.rust-lang.org/reference/dynamically-sized-types.html), [Rust Reference — Sized](https://doc.rust-lang.org/reference/special-types-traits.html#sized) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `advanced-features/src/main.rs:286-293` — **Alıntı:** `MAKROLAR, kod üreten koddur ve fonksiyondan önce AÇILIR (expand). İki tür: Declarative (macro_rules!) ... Procedural: TokenStream alıp TokenStream üreten fonksiyonlar; AYRI bir crate'te (proc-macro = true) yazılır. Üç alt tür: #[derive], öznitelik, fonksiyon-benzeri` — **Hüküm:** DOĞRU — **Kaynak:** [Rust Book Ch. 20 — Macros](https://doc.rust-lang.org/book/ch20-05-macros.html), [Rust Reference — Procedural macros](https://doc.rust-lang.org/reference/procedural-macros.html), [std::proc_macro::TokenStream](https://doc.rust-lang.org/stable/proc_macro/struct.TokenStream.html) — **Öneri:** Yok. — **Önem:** —

### Inferences
- "5 süper güç" listesi Book Ch. 20'nin kendi başlıklarıyla birebir aynı; denetim dosyası yolu (`check_unsafety.rs`) de güncel ağaçta mevcut.
- unsafe'in "borrow checker'ı kapatmadığı" vurgusu, sibling audit notlarındaki (`ileri_unsafe_async.md`) UnsafeCell/Send-Sync bulgularıyla tutarlı.

### Gaps
- (Yok — bu klasörde doğrulanamayan iddia kalmadı.)

---

## Sorunsuz alanlar (doğrulanan ve düzeltme gerektirmeyen iddiaların özeti)

### Takeaway
Aşağıdaki başlıklar bu denetimde hiçbir yanlış/yanıltıcı iddia içermedi; kendi uzmanlık alanlarında birebir kaynaklandı. Rapor yazarken bunlar "sağlam çekirdek" olarak sunulabilir.

### Cited Findings
- **concurrency — Send/Sync mekaniği:** `marker.rs:92/657` satır numaraları, `unsafe auto trait` tanımı, `&T: Send ⇔ T: Sync`, Rc'nin `Cell` sayaçlı `!Send` yapısı, Arc'ın atomik sayacı ve `re-export` düzeltme notu — DOĞRU. Kaynak: `guide/rust/library/core/src/marker.rs:92,657`, `guide/rust/library/alloc/src/rcs/rc.rs:287-288`, `guide/rust/library/alloc/src/rcs/arc.rs:278,280`, `guide/rust/library/alloc/src/sync.rs:14`, [Book Ch. 16](https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html).
- **concurrency — kilit/atomik davranışı:** Mutex poison→Err + `into_inner`, guard RAII, `MutexGuard: !Send`, self-lock "unspecified ama return etmez", `thread::scope` auto-join + `'static` gevşetmesi, `fetch_add(Relaxed)` sayacın kayıp artış yapmaması — DOĞRU. Kaynak: [std::sync::Mutex](https://doc.rust-lang.org/stable/std/sync/struct.Mutex.html), [std::thread::scope](https://doc.rust-lang.org/stable/std/thread/fn.scope.html), `guide/rust/library/std/src/sync/poison/mutex.rs:227,277`, reprolar.
- **async-await — poll/Waker/Pin çekirdeği:** Future tembelliği, poll imzası, "Pending → waker sakla → wake → yeniden poll", Ready-sonrası poll'ün UB olmadığı ama belirsiz olduğu, `impl Future` dönüşü, `.await` desugaring, `Waker::noop` kullanımı, `pin!`/`Box::pin` farkı, bloklayan `thread::sleep` zararı — DOĞRU. Kaynak: [std::future::Future](https://doc.rust-lang.org/stable/std/future/trait.Future.html), [std::pin](https://doc.rust-lang.org/stable/std/pin/struct.Pin.html), [Book Ch. 17](https://doc.rust-lang.org/book/ch17-00-async-await.html), reprolar.
- **oop — dyn ve tasarım kalıpları:** `dyn Trait` varsayılan `'static`, `+ '_` ile gevşetme, fat pointer/vtable, dyn-compatible (`E0038`), `where Self: Sized`, blanket impl ile `Box<dyn Kanal>: Clone`, upcasting 1.86, `dyn Any` downcast'ın koku/meşru kullanım ayrımı, type-state — DOĞRU. Kaynak: [Reference — Trait objects](https://doc.rust-lang.org/reference/types/trait-object.html), [Rust 1.86 blog](https://blog.rust-lang.org/2025/04/03/Rust-1.86.0/), reprolar.
- **patterns — dilbilgisi ve analiz:** literal/`|`/aralık/`..`/guard/`@`/let-else sözdizimi, irrefutability (E0005), exhaustiveness (E0004), kısmi taşıma (E0382), `i32::MIN` `abs()` paniği, `rustc_pattern_analysis` atfı — DOĞRU. Kaynak: [Reference — Patterns](https://doc.rust-lang.org/reference/patterns.html), [Book Ch. 19](https://doc.rust-lang.org/book/ch19-00-patterns.html), reprolar.
- **advanced-features — unsafe ve tipler:** 5 unsafe işlemi, E0133, `check_unsafety.rs` denetim yolu, `Add<Rhs = Self>`, newtype/orphan rule, never type `!`, DST/`?Sized`, makro aileleri — DOĞRU. Kaynak: [Book Ch. 20](https://doc.rust-lang.org/book/ch20-00-advanced-features.html), [Reference — UB](https://doc.rust-lang.org/reference/behavior-considered-undefined.html), `guide/rust/compiler/rustc_mir_build/src/check_unsafety.rs`.
- **ACIKLAMA.md dosyalarının tamamı:** Beş klasörün "Aptala anlatır gibi" / "5 yaşındaki çocuğa anlatır gibi" benzetmeleri teknik olarak yanlış iddia içermiyor (tek istisna: OOP encapsulation benzetmesi `dyn Ciz` örneğinde doğru, gizlilik dersinde olduğu gibi modül ayrımı gerektirmez). Kaynak: ilgili ACIKLAMA.md dosyaları ve yukarıdaki birincil kaynaklar.

### Inferences
- "Sorunsuz alanlar" çoğunlukla dilin kendisi (tür sistemi, trait object, pattern analizi) ile ilgili iddialar; hatalar ise daha çok ikincil atıflar (bölüm/satır numarası, edition zamanlaması, modül-bazlı gizlilik bağlamı) etrafında toplanıyor.

### Gaps
- Hiçbir "sorunsuz alan" iddiası için ek doğrulama gerekmiyor; ancak rapor yazarken modül-bazlı gizlilik farkının OOP benzetmesinde ("kumbara") açıkça belirtilmediği not edilebilir.
