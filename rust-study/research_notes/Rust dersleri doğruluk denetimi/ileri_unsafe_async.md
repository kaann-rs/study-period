# İleri Unsafe / Send-Sync / Dropck / Future / Async — Doğruluk Denetimi

## Kapsam, yöntem ve kaynaklar

### Takeaway
Dört klasördeki 11 dosya (4 × ACIKLAMA.md + 7 × .rs) satır satır denetlendi. Teknik çekirdek iddiaların (UnsafeCell/Send/Sync, Waker vtable, PhantomData/dropck, RwLock/atomik tablosu) büyük kısmı doğru ve yerel `guide/rust` kaynağıyla birebir eşleşiyor; düzeltme notlarının çoğu da doğru. Bulunan sorunlar üç kümede toplanıyor: (a) düzeltme notlarına rağmen gövdelerde kalmış eski RfR bölüm atıfları, (b) dropck düzeltme notundaki tek bir somut repro iddiasının yeniden üretilememesi, (c) ACIKLAMA.md'de `UnsafeCell` istisnası olmayan genelleme.

### Cited Findings
- Denetlenen dosyalar: `learn/advanced-concepts/ileri-send-sync/{ACIKLAMA.md,src/main.rs}`, `ileri-dropck/{ACIKLAMA.md,src/main.rs}`, `ileri-future/{ACIKLAMA.md,src/main.rs}`, `ileri-eszamanlilik-async/{ACIKLAMA.md,src/main.rs,src/threadler.rs,src/calisma_zamani.rs,src/istemci.rs}`. `learn/` altında değişiklik yapılmadı.
- Yerel araç zinciri: `rustc 1.99.0 (b940084d7 2026-09-28)`; mini reprolar `/tmp/opencode/dogrulama/` altında derlendi. Kaynak ağacı: `guide/rust/library/...`.
- RfR bölüm listesi yayıncıda doğrulandı: 3 Designing Interfaces, 4 Error Handling, 8 Asynchronous Programming, 9 Unsafe Code, 10 Concurrency; kitap toplam 13 bölüm — [No Starch Press TOC](https://nostarch.com/rust-rustaceans).

### Inferences
- Düzeltme notları "son gözden geçirme" etiketiyle sonradan eklenmiş; gövde metinleri ise eski sürümden kalmış. Bu yüzden aynı dosyada çelişen iki atık birlikte yaşıyor. Rapor yazarken bunları "iç tutarsızlık" olarak sunmak en isabetli.

### Gaps
- "RfR Listing 4-8" numarası doğrulanmadı (dosyanın kendisi de bunu itiraf ediyor); yayımcının ayrıntılı içindekiler PDF'si üzerinden listing numarası kontrol edilmedi.
- RfR'nin "generator içinde `[u8; 8192]` saklanır" örneği birincil metinden doğrulanmadı.

---

## 1. ileri-send-sync — UnsafeCell ve Send/Sync

### Takeaway
Mekanik anlatım (negatif `!Sync` impl'i, auto-trait yayılımı, Mutex'in `unsafe impl Sync`'i, atomiklerin UnsafeCell üzerine kurulması) doğru. Tek gerçek hata, düzeltme notuyla çelişen "RfR Böl. 4" atfı; bir de `cell.rs` satır numarası ±3 kaymış.

### Cited Findings

- **dosya:satır:** `ileri-send-sync/src/main.rs:5-9` — **Alıntı:** `// Kaynak: Rust for Rustaceans Böl. 4 (Auto-Traits) & Böl. 10 (UnsafeCell) + std (core::cell).` / `// ↳ DÜZELTME NOTU (son gözden geçirme): bölüm numaraları bir fazla yazılmış. // Auto-trait'ler Böl. 3'te (Designing Interfaces), UnsafeCell Böl. 9'da (Unsafe Code) ele // alınır; Böl. 4 Error Handling, Böl. 10 Concurrency'dir.` — **Hüküm:** DOĞRU — **Kaynak:** [No Starch Press TOC](https://nostarch.com/rust-rustaceans) — **Öneri:** Değişiklik yok. — **Önem:** — (doğrulanan düzeltme notu)

- **dosya:satır:** `ileri-send-sync/src/main.rs:11` — **Alıntı:** `// Böl. 15 (RefCell) + Böl. 16 (Send/Sync) + ileri-future burada tek kökte birleşir:` — **Hüküm:** YANILTICI — **Kaynak:** [No Starch Press TOC](https://nostarch.com/rust-rustaceans) (kitap 13 bölüm; 15/16 diye bölüm yok) — **Öneri:** Bunların müfredatın kendi bölüm numaraları olduğunu yaz: `// Müfredat Böl. 15 (RefCell) + Böl. 16 (Send/Sync) ...` ya da doğrudan konu adıyla anlat. — **Önem:** Düşük

- **dosya:satır:** `ileri-send-sync/src/main.rs:116` — **Alıntı:** `// 2. Send / Sync AUTO-TRAIT olarak — derleme-zamanı üyelik testi (RfR Listing 4-8)` — **Hüküm:** DOĞRULANAMADI — **Kaynak:** dosyanın 9-10. satırındaki itiraf (`listenin tam numarası doğrulanmadı`) — **Öneri:** Listing numarasını kaldır ya da "RfR Böl. 3'teki `is_normal` hilesi" de. — **Önem:** Düşük

- **dosya:satır:** `ileri-send-sync/src/main.rs:179-185` — **Alıntı:** `// • UnsafeCell<T> (std core/src/cell.rs:2313): iç değişkenliğin TEK temel yapı taşı. get(&self)` / `//   -> *mut T verir; ... UnsafeCell ayrıca T'de INVARIANT'tır (#[rustc_assert_variance(invariant)]) —` — **Hüküm:** DOĞRU — **Kaynak:** `guide/rust/library/core/src/cell.rs:2313` (`pub struct UnsafeCell<#[rustc_assert_variance(invariant)] T: ?Sized>`), `:2436` (`pub const fn get(&self) -> *mut T`) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `ileri-send-sync/src/main.rs:192-193` — **Alıntı:** `//   (cell.rs ~2315). Auto-trait "her şey Sync olurdu" diyecekken bu negatif impl araya girer:` — **Hüküm:** YANLIŞ (satır numarası) — **Kaynak:** `guide/rust/library/core/src/cell.rs:2318` → `impl<T: ?Sized> !Sync for UnsafeCell<T> {}` — **Öneri:** `~2315` yerine `cell.rs:2318` yaz. — **Önem:** Düşük

- **dosya:satır:** `ileri-send-sync/src/main.rs:197` — **Alıntı:** `// • AUTO-TRAIT ve API (RfR Böl. 4): Send/Sync "gizli bir söz"dür; neredeyse her tipin parçasıdır.` — **Hüküm:** YANLIŞ — **Kaynak:** [No Starch Press TOC](https://nostarch.com/rust-rustaceans): Böl. 4 = Error Handling, auto-trait/API tasarımı Böl. 3 = Designing Interfaces; aynı dosyanın 7-9. satırındaki düzeltme notu — **Öneri:** `RfR Böl. 3` yap; aksi halde düzeltme notuyla çelişir. — **Önem:** Orta

- **dosya:satır:** `ileri-send-sync/src/main.rs:12,67,182-183` — **Alıntı:** `Cell, RefCell, Mutex, atomikler... hepsi bunun üstüne kurulu.` / `Cell, RefCell, Mutex, RwLock, atomikler — hepsi UnsafeCell sarar` — **Hüküm:** DOĞRU — **Kaynak:** `guide/rust/library/core/src/cell.rs:317,854,2313` (Cell/RefCell/UnsafeCell), `guide/rust/library/core/src/sync/atomic.rs:385` (`v: UnsafeCell<T::Storage>`), `guide/rust/library/std/src/sync/poison/mutex.rs:227-230` (`data: UnsafeCell<T>`) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `ileri-send-sync/src/main.rs:119,187-190` — **Alıntı:** `Sync: &T başka thread'lerce GÜVENLE PAYLAŞILABİLİR mi — tanımı gereği:  T: Sync  ⇔  &T: Send.` / `İkisi de AUTO-TRAIT (unsafe auto trait, core::marker)` — **Hüküm:** DOĞRU — **Kaynak:** [Rust Reference — Special traits / Send](https://doc.rust-lang.org/reference/special-types-traits.html#auto-traits) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `ileri-send-sync/src/main.rs:163-164` — **Alıntı:** `Mutex<T>: İÇİNDE UnsafeCell olduğu hâlde Sync'TİR, çünkü std unsafe impl Sync for Mutex ile "erişimi kilit serileştirir, yarış olmaz" sözünü verir.` — **Hüküm:** DOĞRU — **Kaynak:** `guide/rust/library/std/src/sync/poison/mutex.rs:257` → `unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}`; `:230` (`data: UnsafeCell<T>`) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `ileri-send-sync/ACIKLAMA.md:5` — **Alıntı:** `İçinde UnsafeCell olan bir tür birden çok iş parçacığında aynı anda paylaşılamaz; çoğu başka iş parçacığına taşınabilir ama ortak kullanılamaz.` — **Hüküm:** YANILTICI — **Kaynak:** `guide/rust/library/std/src/sync/poison/mutex.rs:230,257` (UnsafeCell içeren Mutex yine Sync'tir); aynı klasörün `src/main.rs:163-164` satırları bunu açıkça söylüyor — **Öneri:** `UnsafeCell tek başına Sync değildir; std, Mutex gibi türlerde `unsafe impl Sync` ile güvenliği ayrıca garanti eder. Bu yüzden Mutex inner UnsafeCell taşımasına rağmen paylaşılabilir.` yaz. — **Önem:** Orta

- **dosya:satır:** `ileri-send-sync/ACIKLAMA.md:9` (benzetme) — **Alıntı:** `İçinde boya olan kutu taşınabilir ama iki kişi aynı anda boyaya dokunmasın diye ikinci etiket verilmez.` — **Hüküm:** DOĞRU — **Kaynak:** `guide/rust/library/core/src/cell.rs:2318` (`!Sync for UnsafeCell`) — **Öneri:** Yok; benzetme `Cell`/`RefCell` için isabetli. — **Önem:** —

### Inferences
- Kritik olmayan ama sistematik bir sorun: düzeltme notu eklenirken eski gövde metinleri silinmemiş. Aynı desen dropck ve future dosyalarında da var (aşağıya bak).

### Gaps
- `Listing 4-8` numarasının RfR Böl. 3'teki gerçek listing numarasıyla eşleşip eşleşmediği doğrulanamadı.

---

## 2. ileri-dropck — Drop check, PhantomData, #[may_dangle]

### Takeaway
NLL/Drop farkı, E0597 mesajı, `#[may_dangle]`'ın unstable ve `unsafe impl` gerektirmesi, std'nin Box/Vec'te bu ikilisi kullanması doğru. Sorunlu nokta, düzeltme notundaki somut `struct K<T>(*mut T) + impl Drop` → E0597 iddiasının bu hâliyle yeniden üretilememesi; ayrıca yine iki eski RfR atfı.

### Cited Findings

- **dosya:satır:** `ileri-dropck/src/main.rs:7-9` — **Alıntı:** `DÜZELTME NOTU (son gözden geçirme): kitapta "The Drop Check" Böl. 9'dadır (Unsafe Code); Böl. 10 Concurrency'dir.` — **Hüküm:** DOĞRU — **Kaynak:** [No Starch Press TOC](https://nostarch.com/rust-rustaceans) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `ileri-dropck/src/main.rs:97-105` — **Alıntı:** `→ E0597: "`gun` does not live long enough / dropped here while still borrowed / borrow might be used here, when `denetci` is dropped and runs the Drop code". ... KRİTİK FARK: Denetci'nin Drop impl'i OLMASAYDI, aynı ters-sıra kod DERLENİRDİ.` — **Hüküm:** DOĞRU — **Kaynak:** `/tmp/opencode/dogrulama/drop_var.rs` (Drop var → E0597, mesaj birebir: "`gun` dropped here while still borrowed", "borrow might be used here, when `d` is dropped and runs the `Drop` code for type `Denetci`", "values in a scope are dropped in the opposite order they are defined"); `/tmp/opencode/dogrulama/drop_yok.rs` (Drop yok → derlenir); [Rustonomicon — Drop Check](https://doc.rust-lang.org/nomicon/dropck.html) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `ileri-dropck/src/main.rs:146-149` — **Alıntı:** `PhantomData<&'a T> → "T'yi 'a boyunca ÖDÜNÇ aldım": 'a covariant, dropck T'yi okumaz. ... PhantomData<fn() -> T> → T'de covariant AMA sahiplenmez (dropck tetiklenmez).` — **Hüküm:** DOĞRU — **Kaynak:** [Rustonomicon — PhantomData tablosu](https://doc.rust-lang.org/nomicon/phantom-data.html) (`PhantomData<&'a T>`: covariant/covariant, dangling allowed; `PhantomData<fn() -> T>`: covariant, Send+Sync, dangling allowed) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `ileri-dropck/src/main.rs:153` — **Alıntı:** `*mut T dropck'e "T'm yok" dedirtir; PhantomData<T> eklemek bu yanlışı düzeltir (RfR Böl. 10).` — **Hüküm:** YANLIŞ (bölüm atfı; paragrafın kendisi 154-158'de düzeltildi) — **Kaynak:** [No Starch Press TOC](https://nostarch.com/rust-rustaceans): doğrusu Böl. 9 — **Öneri:** `(RfR Böl. 9)` yap ya da düzeltme notu zaten Böl. 10'ın Concurrency olduğunu söylediği için parantezi kaldır. — **Önem:** Düşük

- **dosya:satır:** `ileri-dropck/src/main.rs:154-158` — **Alıntı:** `güncel Rust'ta Drop impl'i olan bir tipin TÜM generic parametreleri zaten canlı sayılır: `struct K<T>(*mut T)` + `impl Drop` PhantomData OLMADAN da ters sırada ödünçte E0597 verir (rustc ile doğrulandı). PhantomData<T> dropck açısından yalnız `#[may_dangle]` ile birlikte fark yaratır` — **Hüküm:** YANILTICI (kural kısmı DOĞRU, somut repro DOĞRULANAMADI) — **Kaynak:** Kural için [Rustonomicon — Generic parameters and drop-checking](https://doc.rust-lang.org/nomicon/phantom-data.html) (`impl<T> Drop for Vec<T>` varsa PhantomData<T> "superfluous and accomplishes nothing, dropck-wise"); repro için `/tmp/opencode/dogrulama/k_drop.rs`, `kref_drop.rs`, `generic_raw_drop.rs` — üçü de rustc 1.99.0 ile **derlendi, E0597 üretilmedi**. Buna karşılık `/tmp/opencode/dogrulama/generic_ref_drop.rs` (`struct K<T>(T); impl<T> Drop`) ve `phantom_lt_drop2.rs` (`PhantomData<&'a u8>` + Drop) E0597 verdi. — **Öneri:** "rustc ile doğrulandı" iddiasını kaldır; şunu yaz: `Drop impl'i olan tipin generik parametreleri dropck'te canlı sayılır (RFC 1238); bu yüzden PhantomData<T> eklemek dropck açısından gereksizdir. E0597 örneği için T'nin gerçekten bir ödünç taşıması gerekir: struct K<T>(T) + impl Drop ve let k; let x = 5u8; k = K(&x); derlenmez.` — **Önem:** Orta

- **dosya:satır:** `ileri-dropck/src/main.rs:184-192` — **Alıntı:** `kendi Box<T>/Vec<T>'ini yazarken içeride *mut T tutarsın; dropck *mut T'yi görüp "T'ye sahip değil, T::drop'u çağırmaz" sanır. ... ↳ DÜZELTME NOTU: bu paragraf eski dropck kuralını anlatır; bugün belirleyici olan Drop impl'inin varlığıdır ... PhantomData<T>'nin dropck rolü #[may_dangle] ile sınırlıdır.` — **Hüküm:** DOĞRU (düzeltme notu yerinde; ama son cümle yukarıdaki aynı aşırı genellemeyi taşıyor) — **Kaynak:** [Rustonomicon — PhantomData](https://doc.rust-lang.org/nomicon/phantom-data.html) (RFC 1238 sonrası kural; ayrıca "advanced edge case": Drop impl'i olmayıp drop glue olan tiplerde PhantomData<T> dropck'ı etkiler) — **Öneri:** Son cümleyi `PhantomData<T>'nin bağımsız dropck etkisi yalnızca Drop impl'i olmayan (ama drop glue'ı olan) tiplerde ve #[may_dangle] ile birlikte görünür` yap. — **Önem:** Düşük

- **dosya:satır:** `ileri-dropck/src/main.rs:194-200` — **Alıntı:** `#[may_dangle] (unstable `dropck_eyepatch`) ... unsafe impl<#[may_dangle] T> Drop for KendiBox<T> { ... } ... nightly'dir ... std'de Vec/Box tam da bunu + PhantomData<T>'yi birlikte kullanır.` — **Hüküm:** DOĞRU — **Kaynak:** [Rust Unstable Book — dropck_eyepatch](https://doc.rust-lang.org/unstable-book/language-features/dropck-eyepatch.html) (tracking #34761); [Rustonomicon — Drop Check](https://doc.rust-lang.org/nomicon/dropck.html) (unsafe impl zorunluluğu); `guide/rust/library/alloc/src/boxed.rs:2126`, `guide/rust/library/alloc/src/raw_vec/mod.rs:80,439`, `guide/rust/library/alloc/src/vec/mod.rs:4546` — **Öneri:** Yok. "Kim kullanabilir?" sorusunun cevabı: yalnızca `#![feature(dropck_eyepatch)]` açan nightly crate'ler; std bunu kendi içinde kullanır, kararlı kullanıcı kodu kullanamaz. — **Önem:** —

- **dosya:satır:** `ileri-dropck/ACIKLAMA.md:5,9` (benzetmeler) — **Alıntı:** `Bir değer silinirken (drop) ödünç aldığı veriye son bir kez erişebilir; çünkü silme kodu onu kullanabilir.` / `Çöpe atmadan önce defterine son bir not yazacaksın. ... kitap, defter çöpe atılmadan önce orada durmalı.` — **Hüküm:** DOĞRU — **Kaynak:** [Rustonomicon — Drop Check](https://doc.rust-lang.org/nomicon/dropck.html) — **Öneri:** Yok. — **Önem:** —

### Inferences
- E0597 repro iddiasının yanlış çıkma nedeni büyük olasılıkla ham pointer'a çevirme sırasında ödüncün hemen sona ermesi: `&x as *mut u8` artık borrow checker için canlı bir referans değil. Bu yüzden dropck "canlı generic" kuralı devreye girmesine rağmen çakışan bir borrow kalmaz.

### Gaps
- Dersin "rustc ile doğrulandı" dediği orijinal repro kodu paylaşılmamış; hangi şekilde E0597 aldıkları bilinmiyor (referans alanı mı kullandılar, `PhantomData<&'a T>` mı?).

---

## 3. ileri-future — poll sözleşmesi, Waker, Pin

### Takeaway
Kaynak satır numaraları (Future :37/:113, Poll :13, RawWaker :22, RawWakerVTable :113, Waker :411, wake :443) yerel rust kaynağıyla birebir doğru. Waker'ın 4 fonksiyonlu vtable'ı ve Send+Sync garantisi de doğru. Tek sistematik sorun: üç yerde kalmış "RfR Böl. 9" atfı (doğrusu 8).

### Cited Findings

- **dosya:satır:** `ileri-future/src/main.rs:6-9` — **Alıntı:** `DÜZELTME NOTU: kitapta async (Pin and Unpin, poll sözleşmesi, waker) Böl. 8'dedir (Asynchronous Programming); Böl. 9 Unsafe Code'dur. ... Dosyadaki "RfR Böl. 9" atıfları Böl. 8 olarak okunmalı.` — **Hüküm:** DOĞRU — **Kaynak:** [No Starch Press TOC](https://nostarch.com/rust-rustaceans) — **Öneri:** Yok; aşağıdaki üç satırı da düzelt. — **Önem:** —

- **dosya:satır:** `ileri-future/src/main.rs:87` — **Alıntı:** `println!("\n== 3) Pin ve self-referential (RfR Böl. 9) ==");` — **Hüküm:** YANLIŞ — **Kaynak:** [No Starch Press TOC](https://nostarch.com/rust-rustaceans) — **Öneri:** `RfR Böl. 8` — **Önem:** Düşük

- **dosya:satır:** `ileri-future/src/main.rs:222` — **Alıntı:** `// 3) PIN NEDEN VAR? — self-referential state machine sorunu (RfR Böl. 9)` — **Hüküm:** YANLIŞ — **Kaynak:** aynı — **Öneri:** `RfR Böl. 8` — **Önem:** Düşük

- **dosya:satır:** `ileri-future/src/main.rs:303` — **Alıntı:** `// • async fn/async {} → STATE MACHINE (generator). RfR Böl. 9: makine, her yield/await noktasındaki` — **Hüküm:** YANLIŞ — **Kaynak:** aynı — **Öneri:** `RfR Böl. 8` — **Önem:** Düşük

- **dosya:satır:** `ileri-future/src/main.rs:94-99` — **Alıntı:** `poll'un SÖZLEŞMESİ (contract): ... Bitiremediysen ... ÖNCE cx içindeki Waker'ı sakla/kaydet, SONRA Poll::Pending dön. Waker'ı kaydetmeden Pending dönmek HATADIR: executor seni bir daha asla poll etmez` — **Hüküm:** DOĞRU — **Kaynak:** [std::future::Future — poll docs](https://doc.rust-lang.org/std/future/trait.Future.html#the-poll-method) ("returns Pending and stores a clone of the Waker", "poll should not be called repeatedly in a tight loop — instead only when the future indicates ready by calling wake()") — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `ileri-future/src/main.rs:189-197` — **Alıntı:** `Waker ... bir TRAIT OBJECT gibidir: (veri-pointer, &'static RawWakerVTable). Vtable DÖRT fonksiyon pointer'ı tutar (std, core/src/task/wake.rs:113): clone, wake, wake_by_ref, drop. ... Waker'ın iki garantisi: Clone ... ve Send+Sync` — **Hüküm:** DOĞRU — **Kaynak:** `guide/rust/library/core/src/task/wake.rs:22,113,411,418,420,443`; [std::task::Waker](https://doc.rust-lang.org/std/task/struct.Waker.html) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `ileri-future/src/main.rs:299-311` — **Alıntı:** `Future trait'i: core/src/future/future.rs:37; ... poll ... (:113) ... Poll enum: core/src/task/poll.rs:13 ... Waker (:411) = RawWaker (:22) = (data, vtable) ... Vtable (:113) ... wake() (:443)` — **Hüküm:** DOĞRU (bütün satır numaraları yerel kaynakla eşleşiyor) — **Kaynak:** `guide/rust/library/core/src/future/future.rs:37,113`; `guide/rust/library/core/src/task/poll.rs:13`; `guide/rust/library/core/src/task/wake.rs:22,113,411,443` — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `ileri-future/src/main.rs:303-306` — **Alıntı:** `içindeki [u8; 8192] generator'da saklanır) ve büyük future'ları Box::pin ile heap'e alıp taşımayı ucuzlatmak gerekebilir.` — **Hüküm:** DOĞRULANAMADI — **Kaynak:** mekanizmanın kendisi [Rust Reference — async blocks](https://doc.rust-lang.org/reference/expressions/block-expr.html#async-blocks) ile uyumlu; spesifik 8192 sayısı birincil kaynakta doğrulanamadı — **Öneri:** Sayıyı kaldır ya da "örneğin büyük bir dizi" de. — **Önem:** Düşük

- **dosya:satır:** `ileri-future/src/main.rs:224-235,231-235` — **Alıntı:** `async fn bir state machine'e derlenir ve bu makine, bir .await noktasında KENDİ yerel değişkenine REFERANS tutabilir ... poll'un imzası bu yüzden self: Pin<&mut Self>` — **Hüküm:** DOĞRU — **Kaynak:** [Rust Reference — async blocks / Pin](https://doc.rust-lang.org/reference/expressions/block-expr.html#async-blocks); [std::pin::Pin](https://doc.rust-lang.org/std/pin/struct.Pin.html) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `ileri-future/ACIKLAMA.md:5,9` (benzetmeler) — **Alıntı:** `Pending dönerken future bir uyardırıcı (Waker) saklar ... Boşa dönen bekleme (busy-wait) yoktur.` / `Çamaşır makinesine "bittiğinde beni ara" diye not bırakıyorsun.` — **Hüküm:** DOĞRU — **Kaynak:** [std::future::Future docs](https://doc.rust-lang.org/std/future/trait.Future.html#the-poll-method) — **Öneri:** Yok. — **Önem:** —

### Inferences
- Waker benzetmesi ("titreyen çağrı cihazı", "çamaşır makinesi") `Waker::will_wake` kullanımıyla da tutarlı; yanlış yönlendirme yok.

### Gaps
- RfR Böl. 8'deki gerçek listing/örnek numaraları kontrol edilmedi.

---

## 4. ileri-eszamanlilik-async — thread'ler, runtime, istemci

### Takeaway
Send/Sync tablosu, RwLock/Mutex'in trait sınırları (kaynak satırlarıyla), Condvar lost/spurious wakeup anlatımı, waker kaydının tek kilit altında yapılması, waker'ı her poll'da güncelleme kuralı ve 1.85 `AsyncFnMut` notu doğru. İki küçük sorun: müfredatın kendi 16/17 bölüm numaralarının RfR atfı gibi okunması ve "future bir kez tamamlanır, yeniden poll edilemez" ifadesinin keskinliği.

### Cited Findings

- **dosya:satır:** `ileri-eszamanlilik-async/src/main.rs:6` — **Alıntı:** `Rust for Rustaceans Böl. 8 (Asynchronous Programming) ve Böl. 10 (Concurrency).` — **Hüküm:** DOĞRU — **Kaynak:** [No Starch Press TOC](https://nostarch.com/rust-rustaceans) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `ileri-eszamanlilik-async/src/main.rs:9-10` — **Alıntı:** `concurrency (Böl. 16) → thread, kanal, Arc<Mutex>, scope, işçi havuzu` / `async-await (Böl. 17) → Future, elle executor, join, Pin'in gündelik yüzü` — **Hüküm:** YANILTICI — **Kaynak:** [No Starch Press TOC](https://nostarch.com/rust-rustaceans) (RfR 13 bölüm; buradaki 16/17 müfredatın kendi numaraları) — **Öneri:** `learn/concurrency`, `learn/async-await` yol adlarını yaz ya da "müfredat Böl. 16/17" etiketle. — **Önem:** Düşük

- **dosya:satır:** `ileri-eszamanlilik-async/src/main.rs:36` — **Alıntı:** `✗ işaretli derlenmeyen örneklerin hata kodları/mesajları rustc 1.99 ile doğrulandı.` — **Hüküm:** DOĞRU — **Kaynak:** `rustc 1.99.0 (b940084d7 2026-09-28)`; `/tmp/opencode/dogrulama/async_guard.rs`, `async_borrow.rs`, `rwlock_cell.rs`, `mutex_cell.rs` — E-kodu olmayan "future cannot be sent between threads safely", E0373, E0277 mesajları birebir üretildi. — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `ileri-eszamanlilik-async/src/main.rs:139-145` — **Alıntı:** `toplam poll = 6 (3 görev; yalnız uyandırılan poll edildi) ... Sayım: kök 2 ... yavaş 2, hızlı 2. hizli bittiğinde kökü UYANDIRMAZ: kök o sırada hizli'yi değil yavas'ı bekliyor, tutamaca waker kaydetmedi.` — **Hüküm:** DOĞRU — **Kaynak:** `learn/advanced-concepts/ileri-eszamanlilik-async/src/calisma_zamani.rs:147-157,189-211` (Tutamac waker'ı yalnız bekleyen varsa alıyor; `canli` sayacı 3 → 0) — **Öneri:** "yalnız uyandırılan" ifadesini "ilk poll + wake gelen poll" yapmak daha kesin olur. — **Önem:** Düşük

- **dosya:satır:** `ileri-eszamanlilik-async/src/threadler.rs:19` — **Alıntı:** `kilit yükseltme → okuma anahtarını bırakmadan yazma anahtarına çevirmek. RwLock'ta yok.` — **Hüküm:** DOĞRU — **Kaynak:** [std::sync::RwLock docs](https://doc.rust-lang.org/std/sync/struct.RwLock.html) (upgrade API yok) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `ileri-eszamanlilik-async/src/threadler.rs:511-523` — **Alıntı:** `Mutex<T> T:Send → ✓ kilit erişimi TEKE indirir: T'nin Sync olması gerekmez / RwLock<T> Sync için T: Send + Sync ... / MutexGuard<T> ✗ (T:Sync) / mpsc::Sender<T> T:Send → ✓ ✓` — **Hüküm:** DOĞRU (tablonun tamamı) — **Kaynak:** `guide/rust/library/std/src/sync/poison/mutex.rs:257,297`; `guide/rust/library/std/src/sync/poison/rwlock.rs:107`; `guide/rust/library/std/src/sync/mpsc.rs:347,350` (`unsafe impl<T: Send> Sync for Sender<T>`); `/tmp/opencode/dogrulama/mutex_cell.rs` (-derlenir), `rwlock_cell.rs` (E0277) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `ileri-eszamanlilik-async/src/threadler.rs:173-186,246-250,269-275` — **Alıntı:** `wait(guard) → kilidi BIRAKIR + uyur (ikisi atomik); uyanınca kilidi GERİ ALIR ... Koşulu HER ZAMAN döngüde denetle ... wait_while bu döngüyü senin yerine yazar. ... notify_ALL: kapanış, bekleyen HERKESİ ilgilendirir` — **Hüküm:** DOĞRU — **Kaynak:** [std::sync::Condvar docs](https://doc.rust-lang.org/std/sync/struct.Condvar.html#method.wait) (atomic unlock+wait, spurious wakeups, predicate loop) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `ileri-eszamanlilik-async/src/threadler.rs:164-166` — **Alıntı:** `Yanlış seçilmiş Relaxed çoğu x86 makinesinde "çalışıyor" görünür, ARM'da bozulur; bu sınıf hatalar testle bulunmaz. Ölçmeden zayıflatma.` — **Hüküm:** DOĞRU (genel kural) — **Kaynak:** [Rustonomicon — Atomics](https://doc.rust-lang.org/nomicon/atomics.html); [std::sync::atomic docs](https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `ileri-eszamanlilik-async/src/calisma_zamani.rs:336-339` — **Alıntı:** `Waker'ı HER poll'da güncelle: görev araya başka bir waker'la poll edilmiş olabilir ... eskisini uyandırmak işe yaramazdı.` — **Hüküm:** DOĞRU — **Kaynak:** [std::future::Future — poll docs](https://doc.rust-lang.org/std/future/trait.Future.html#return-value) ("only the Waker from the Context passed to the most recent call should be scheduled to receive a wakeup") — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `ileri-eszamanlilik-async/src/calisma_zamani.rs:366-378` — **Alıntı:** `Pin projeksiyonu ... Tahsis yok ama doğruluğu elle kanıtlamak gerekir (unsafe); pratikte pin-project ... ✗ Kısayol yok: Pin::new(&mut self.ic) yalnız F: Unpin ise derlenir ... E0277: F cannot be unpinned` — **Hüküm:** DOĞRU — **Kaynak:** [std::pin::Pin — projection docs](https://doc.rust-lang.org/std/pin/struct.Pin.html#projecting-pin-to-struct-fields); [pin-project crate](https://docs.rs/pin-project) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `ileri-eszamanlilik-async/src/calisma_zamani.rs:187-188` — **Alıntı:** `Sınır: sayaç 0'a inip döngü çıktıktan SONRA başka bir thread'den başlatılan görev poll edilmez; görevler ya calistir()'dan önce ya da çalışan bir görevin içinden başlatılmalı.` — **Hüküm:** DOĞRU — **Kaynak:** `learn/advanced-concepts/ileri-eszamanlilik-async/src/calisma_zamani.rs:189-211` (`while canli > 0` döngüsü ve `recv()`) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `ileri-eszamanlilik-async/src/istemci.rs:127-130` — **Alıntı:** `Not (Rust 1.85+): aynı bound `impl AsyncFnMut() -> Result<T, IstekHatasi>` diye de yazılabilir; çağıran o zaman `async move || istek(..).await` biçiminde bir async closure verir.` — **Hüküm:** DOĞRU — **Kaynak:** [Rust 1.85.0 announcements — async closures](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/#async-closures) (`AsyncFn`, `AsyncFnMut`, `AsyncFnOnce` prelude'a girdi) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `ileri-eszamanlilik-async/src/istemci.rs:121-122` — **Alıntı:** `Çünkü bir future yalnız BİR KEZ tamamlanır; bittikten sonra yeniden poll edilemez. Her deneme taze bir future ister` — **Hüküm:** YANILTICI — **Kaynak:** [std::future::Future — Panics](https://doc.rust-lang.org/std/future/trait.Future.html#panics) ("Once a future has completed ... calling poll again may panic, block forever, or cause other kinds of problems; the Future trait places no requirements ... calls must never cause undefined behavior") — **Öneri:** `bittikten sonra yeniden poll edilmemelidir (sözleşme ihlali; derlenir ama panik/askıda kalma olabilir)` yaz. Sonuç çıkarımı (taze future gerekir) doğru kalır. — **Önem:** Düşük

- **dosya:satır:** `ileri-eszamanlilik-async/src/istemci.rs:332-339` — **Alıntı:** `(Bu hatanın E-kodu yoktur.) Async fn'in state machine'i, await anında canlı olan her yerel değişkeni İÇİNDE saklar; MutexGuard !Send olduğu için future da !Send olur ve baslat`ın F: Send bound'u sağlanmaz.` — **Hüküm:** DOĞRU — **Kaynak:** `/tmp/opencode/dogrulama/async_guard.rs` — rustc 1.99.0 çıktısı `error: future cannot be sent between threads safely` (E-kodu yok), `help: ... Send is not implemented for std::sync::MutexGuard<'_, u32>`, `note: future is not Send as this value is used across an await` — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `ileri-eszamanlilik-async/src/istemci.rs:397` — **Alıntı:** `E0373: async block may outlive the current function, but it borrows `ad`, which is owned by the current function` — **Hüküm:** DOĞRU — **Kaynak:** `/tmp/opencode/dogrulama/async_borrow.rs` — rustc 1.99.0 mesajı birebir — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `ileri-eszamanlilik-async/src/istemci.rs:405-410` — **Alıntı:** `waker'ı kaydetmeden Pending dönen elle yazılmış future ... görev sonsuza dek bekler ve calistir() dönmez. Kural: Pending dönmeden önce cx.waker()'ı olayı bilecek tarafa TESLİM ET ... ve "hazır mı?" denetimi ile waker kaydını aynı kilit altında yap` — **Hüküm:** DOĞRU — **Kaynak:** `learn/advanced-concepts/ileri-eszamanlilik-async/src/calisma_zamani.rs:190-195` (`recv()` bloğu), `:259-275` (Tutamac: kontrol + waker kaydı tek Mutex altında), [std::future::Future docs](https://doc.rust-lang.org/std/future/trait.Future.html#return-value) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `ileri-eszamanlilik-async/src/istemci.rs:214-221,241-252` — **Alıntı:** `Bir görev poll içinde uzun süre kalırsa — thread::sleep, bloklayan dosya/ağ çağrısı, ağır hesap, std Mutex'te uzun bekleme — o thread'deki bütün öteki görevler durur. Derleyici bunu YAKALAMAZ ... tokio'da spawn_blocking; burada ayri_threadde ile aynısını yapıyoruz.` — **Hüküm:** DOĞRU — **Kaynak:** [tokio::task::spawn_blocking docs](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html); [std::future::Future — Runtime characteristics](https://doc.rust-lang.org/std/future/trait.Future.html#runtime-characteristics) ("poll should strive to return quickly, and should not block") — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `ileri-eszamanlilik-async/src/istemci.rs:124-125,142-159` — **Alıntı:** `Bekleme uyu_ms(..).await ile yapılıyor — thread::sleep DEĞİL ... FnMut() -> Fut ... her deneme taze bir future ister` / backoff `uyu_ms(5 * u64::from(deneme))` — **Hüküm:** DOĞRU — **Kaynak:** `learn/advanced-concepts/ileri-eszamanlilik-async/src/calisma_zamani.rs:297-358` (Uyku leaf future) — **Öneri:** Yok. — **Önem:** —

- **dosya:satır:** `ileri-eszamanlilik-async/ACIKLAMA.md:5,9` (benzetmeler) — **Alıntı:** `Thread'ler ve async görevler aynı soruya iki cevaptır: "beklerken ne yapalım?" ... Mutfafta aşçılar (thread) ve tek bir kasiyer (async) var. Kasiyer sipariş beklerken boş durmaz; zil çalan masaya gider.` — **Hüküm:** DOĞRU — **Kaynak:** `learn/advanced-concepts/ileri-eszamanlilik-async/src/main.rs:24-32` ile aynı model — **Öneri:** Yok. — **Önem:** —

### Inferences
- Lost wakeup/spurious wakeup anlatımı üç yerde (Condvar, Tutamac, ayri_threadde) aynı invariant üzerine kurulmuş: durum denetimi + waker kaydı aynı kilit altında. Bu tutarlılık korunmalı.
- `main.rs:139` poll sayısı 6 hesabı, `hizli` tamamlandığında kök görev onu henüz awaitlemediği için kökü uyandırmamasına dayanıyor; bu, görev yapısının doğru bir sonucu.

### Gaps
- `spawn_blocking` ile `ayri_threadde` arasındaki fark (tokio thread pool / izin sınırı / iptal davranışı) ders kapsamı dışında bırakılmış; yanlış değil, eksik.

---

## Sorunsuz alanlar (doğrulanan, düzeltme gerektirmeyen iddialar)

### Takeaway
Aşağıdaki konular kaynaklarla birebir doğrulandı; raporda "sorunsuz" olarak listelenebilir.

### Cited Findings
- `UnsafeCell` = iç değişkenliğin temeli, `get(&self) -> *mut T`, T'de invariant, std'de açık `impl !Sync` — `guide/rust/library/core/src/cell.rs:2313,2318,2436`.
- `Cell`/`RefCell`/`Mutex`/`RwLock`/atomikler UnsafeCell üzerine kurulu — `core/src/cell.rs:317,854`; `core/src/sync/atomic.rs:385`; `std/src/sync/poison/mutex.rs:227-230`; `std/src/sync/poison/rwlock.rs:94`.
- Send/Sync auto-trait; `T: Sync ⇔ &T: Send`; Mutex `T: Send` ise Sync; RwLock `T: Send + Sync` ise Sync; MutexGuard !Send ama `T: Sync` ise Sync; mpsc::Sender `T: Send` ise Send+Sync — `std/src/sync/poison/mutex.rs:257,297`; `std/src/sync/poison/rwlock.rs:107`; `std/src/sync/mpsc.rs:347,350`; [Rust Reference](https://doc.rust-lang.org/reference/special-types-traits.html#auto-traits).
- Drop varsa borrow drop anına kadar uzar (E0597), Drop yoksa NLL erken bitirir; ters düşme sırası kuralı — rustc 1.99 repro + [Rustonomicon Drop Check](https://doc.rust-lang.org/nomicon/dropck.html) + [Rust Reference — Destructors](https://doc.rust-lang.org/reference/destructors.html).
- `#[may_dangle]` = unstable `dropck_eyepatch`, `unsafe impl` ister, std Box/Vec/RawVec kullanır — [Unstable Book](https://doc.rust-lang.org/unstable-book/language-features/dropck-eyepatch.html), `alloc/src/boxed.rs:2126`, `alloc/src/raw_vec/mod.rs:80,439`, `alloc/src/vec/mod.rs:4546`.
- `PhantomData<T>` varyans/dropck/auto-trait için ZST işareti; `PhantomData<&'a T>` covariant ve borçlanma olarak dropck'a izin verir; `PhantomData<fn() -> T>` covariant ve sahiplenmez; `PhantomData<*const T>` !Send+!Sync — [Rustonomicon PhantomData tablosu](https://doc.rust-lang.org/nomicon/phantom-data.html).
- Future/poll/Waker/Pin sözleşmesi ve bütün kaynak satır numaraları — `core/src/future/future.rs:37,113`, `core/src/task/poll.rs:13`, `core/src/task/wake.rs:22,113,411,418,420,443`; [std Future docs](https://doc.rust-lang.org/std/future/trait.Future.html).
- Condvar atomik unlock+wait, spurious wakeup, predicate loop, notify_all; compare_exchange; SeqCst/Acquire-Release/Relaxed rehberi — [Condvar docs](https://doc.rust-lang.org/std/sync/struct.Condvar.html), [Rustonomicon Atomics](https://doc.rust-lang.org/nomicon/atomics.html).
- async → state machine, self-referential → Pin, `!Unpin` generator'lar, `Pin::new` yalnız Unpin için, pin-projeksiyon — [Rust Reference async blocks](https://doc.rust-lang.org/reference/expressions/block-expr.html#async-blocks), [std::pin::Pin](https://doc.rust-lang.org/std/pin/struct.Pin.html).
- `AsyncFnMut` 1.85'te kararlı; async'te iptal = future'ı düşürmek; bloklayan işi ayırmak — [Rust 1.85.0 blog](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/#async-closures), [tokio spawn_blocking](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html).
- Dört ACIKLAMA.md'nin benzetmeleri (etiket/kutu, defter/kitap, çamaşır makinesi/zil, aşçı/kasiyer) teknik içerikle tutarlı; yalnız yukarıda belirtilen `UnsafeCell` genellemesi istisna.

### Inferences
- Genel değerlendirme: 4 klasörün teknik omurgası sağlam; düzeltmeler çoğunlukla doğru uygulanmış. Öncelikli iş, düzeltme notlarıyla çelişen atıfları temizlemek ve dropck repro iddiasını ya kaldırmak ya da gerçek E0597 veren kodla değiştirmek.

### Gaps
- Tüm zamanlama/çıktı yorumları ("≈60 ms", "toplam poll = 6") mantıksal olarak doğrulandı; makinede `cargo test` çalıştırılarak davranışsal doğrulama yapılmadı (görev kapsamı: yorumları denetlemek).
