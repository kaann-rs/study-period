# rust-study

Rust öğrenme çalışmam. Bir cargo workspace: `learn/*` kitabın bölümlerini/kavramları
(ileri konular `learn/advanced-concepts/` altında),
`project/*` kitaplardan ve kendi denemelerimden gelen küçük projeleri tutar. `guide/` =
referans için `rust-lang/rust` kaynak ağacı (workspace dışı).

## Nasıl okunur? (hiç bilmeyenler için)

Her dosya iki katmanlıdır:

1. **SADE ANLATIM kutuları** — dosyanın başında bir *mini sözlük* (o dosyada geçen her terimin
   gündelik dille karşılığı) ve her konunun üstünde, günlük hayattan bir benzetme ile birkaç
   satırlık *mini örnek* içeren bir kutu. Konuyu ilk kez görüyorsan yalnız bunları okuyarak
   ilerleyebilirsin.
2. **Teknik açıklama** — kutunun altındaki özgün yorumlar: neden böyle tasarlandığı, olmasaydı
   ne bozulurdu, derleyici bunu nerede denetliyor. Zor terimlerin yanında `(Sade: …)` notları var.

Dosya sonlarındaki "DERLEYİCİ PERSPEKTİFİ" bölümleri ilk okuyuşta atlanabilir.

## Rust Book yolculuğu (ch3–ch20)

[doc.rust-lang.org/book](https://doc.rust-lang.org/book/) (2024 edition) tüm kavramsal
bölümlerini sırayla birer crate olarak işledim. Her crate aynı derinlikte: **terim (İngilizce) +
Türkçe açıklama + örnek + Neden? (tasarım kararı) + o tasarım olmasaydı çıkacak Sorun +
gerektiğinde rustc'nin hangi aşamada nasıl uyguladığı** (gerçek `rust-lang/rust` kaynak
referanslarıyla). Tüm iddialar rustc ile derlenip/test edilip doğrulandı.

| Böl. | Crate | Konu |
| --- | --- | --- |
| 3 | `learn/common-concepts` | Değişkenler, mutability, shadowing, tipler, fonksiyonlar, akış kontrolü |
| 4 | `learn/ownership` | Sahiplik, move/Copy/clone, Drop, ödünç alma, dangling, dilimler |
| 5 | `learn/structs` | Struct'lar, metotlar, alıcılar (self/&self/&mut self) |
| 6 | `learn/enums` | Enum'lar, Option, match, if let / let-else |
| 7 | `learn/modules` | Modüller, yollar, use, görünürlük (pub/private) |
| 8 | `learn/collections` | Vec, String (UTF-8), HashMap |
| 9 | `learn/error-handling` | panic!, Result, ? operatörü, From ile hata dönüşümü |
| 10 | `learn/generics-traits-lifetimes` | Jenerikler, trait'ler, lifetime'lar + elision |
| 11 | `learn/writing-tests` | #[test], assert'ler, should_panic, birim vs entegrasyon |
| 12 | `learn/minigrep` | G/Ç projesi: argümanlar, dosya, stdout/stderr, Box<dyn Error> |
| 13 | `learn/closures-iterators` | Closure'lar (Fn/FnMut/FnOnce), iterator'lar (tembellik) |
| 14 | `learn/cargo-and-crates` | Doc yorumları + doc-test, pub use, release profilleri, workspace |
| 15 | `learn/smart-pointers` | Box, Deref, Drop, Rc, RefCell (iç değişkenlik) |
| 16 | `learn/concurrency` | Thread'ler, kanallar (mpsc), Arc<Mutex>, Send + Sync |
| 17 | `learn/async-await` | async/await, Future, elle minimal executor (bağımlılıksız) |
| 18 | `learn/oop` | Trait object polimorfizmi, type-state kalıbı |
| 19 | `learn/patterns` | Desenler: tüm yerler, refutable/irrefutable, söz dizimi |
| 20 | `learn/advanced-features` | unsafe, operatör aşırı yükleme, newtype, fn ptr, makrolar |

Kalan: **ch21** (son proje — çok-thread'li web sunucusu), isteğe bağlı.

## Sağlamlaştırma (ileri konular — `learn/advanced-concepts/`)

Kitap bir temeldir ama mekanizmaların KÖKÜNÜ her zaman açmaz. Sağlamlaştırma fazı, genuinely
zor konuları kaynaktan (std + rustc) ve ileri kitaplardan derinlemesine, çalışan örneklerle
ele alır. Kaynaklar: *Rust for Rustaceans* (Jon Gjengset) ve *Programming Rust* (Blandy &
Orendorff).

| Crate | Konu | Ana kaynak |
| --- | --- | --- |
| `learn/advanced-concepts/ileri-future` | Future gerçekte nedir: poll/wake döngüsü, Waker + vtable, Pin ve self-referential state machine | std (`core::future`, `core::task`) + RfR Böl. 9 |
| `learn/advanced-concepts/ileri-variance` | Subtyping ve variance: covariant/invariant/contravariant, `&'a mut &'b T` neden iki lifetime ister | RfR Böl. 2 (Listing 2-11) |
| `learn/advanced-concepts/ileri-sized-dst` | Sized, DST (`str`/`[T]`/`dyn`), fat pointer (ptr + uzunluk/vtable), `?Sized` | RfR Böl. 3 + std |
| `learn/advanced-concepts/ileri-deref-metot` | Deref + metot çözümü: deref coercion, autoref/autoderef, inherent metot gölgeleme | RfR Böl. 4 + Reference + std |
| `learn/advanced-concepts/ileri-send-sync` | UnsafeCell (iç değişkenliğin tek temeli) + Send/Sync auto-trait'leri, !Sync negatif impl | RfR Böl. 4 & 10 + std |
| `learn/advanced-concepts/ileri-dropck` | Drop check: Drop + lifetime neden borrow'u aşmaya zorlar; PhantomData (ZST işaretçi); #[may_dangle] | RfR Böl. 10 + std |
| `learn/advanced-concepts/ileri-coherence` | Coherence + orphan rule (tam kural), blanket impl, fundamental tipler, covered impl | RfR Böl. 3 + std |
| `learn/advanced-concepts/ileri-lifetime-hrtb-gat` | İleri lifetime'lar: iki ayrı süre, `'a: 'b` / `T: 'a`, HRTB (`for<'a>`), GAT (ödünç veren iterator, tür ailesi), lifetime hata mesajı rehberi | Reference + Rustonomicon + RfR Böl. 1–2 |
| `learn/advanced-concepts/ileri-eszamanlilik-async` | İleri eşzamanlılık + async: RwLock, Condvar, kilit sıralaması, atomikler; std ile mini runtime (görev kuyruğu, spawn, zamanlayıcı, zaman aşımı), Pin/Unpin pratikte, asenkron API istemcisi, yaygın async hataları | std + *Rust Atomics and Locks* + RfR Böl. 8 & 10 |

> **Düzeltme notu (RfR bölüm numaraları):** yukarıdaki tabloda ilk yedi satırın *Rust for
> Rustaceans* bölüm numaraları bir fazla yazılmıştır. Yayıncının ayrıntılı içindekiler listesine
> göre doğrusu: variance → Böl. 1 (Listing 1-11), DST ve coherence → Böl. 2, Deref kutusu ve
> auto-trait'ler → Böl. 3, async/Pin → Böl. 8, drop check ve UnsafeCell → Böl. 9. Son iki satır
> (Böl. 1–2 ve Böl. 8 & 10) doğrudur. İlgili dosyaların başlıklarına da aynı not eklendi.

## Derinleşme modülleri (`src/derinlesme.rs`)

Zor konuların crate'lerinde `main.rs` kavramı tanıtır; yanındaki `derinlesme.rs` aynı konuyu
PRATİĞE döker. `main.rs` dosyalarındaki kod ve yorumlar olduğu gibi duruyor; yalnız
`mod derinlesme;` ve `derinlesme::calistir();` satırları ile — son gözden geçirmede rustc ve
std kaynağıyla doğrulanan teknik yanlışların hemen altına — `↳ DÜZELTME NOTU` satırları eklendi: `cargo run -p <crate>` önce özgün bölümü, ardından
derinleşme bölümünü çalıştırır. Her modülde: gerçekçi bir senaryo, adım adım kod analizi, sık
yapılan hatalar (✗ derlenmeyen / ✓ doğru kullanım; hata kodları rustc ile doğrulandı), beklenen
çıktılar, alıştırmalar, bir sonraki konuyla bağlantı ve `cargo test -p <crate>` ile çalışan
birim testleri.

| Crate | Derinleşmede ne var |
| --- | --- |
| `learn/ownership` | İmza = sahiplik sözleşmesi (`String` / `&str` / `&mut String`), kısmi move, kullanıcı deposu, E0382/E0499/E0502/E0505/E0507 ve çözümleri, `mem::take` / `Option::take` |
| `learn/structs` | Alan gizliliğiyle değişmez koruma, builder kalıbı, alıcı seçimi = sahiplik, struct update'te kısmi move |
| `learn/enums` | Geçersiz durumları temsil edilemez kılmak, sipariş durum makinesi, Option/Result kombinatörleri |
| `learn/error-handling` | Kendi hata türü (`Display` + `Error::source` + `From`), Option ↔ Result köprüleri, ilk hatada dur / hepsini topla, dosya G/Ç, `Box<dyn Error>` mi enum mu |
| `learn/generics-traits-lifetimes` | Lifetime bir ilişkidir (fazla sıkı imza), sıfır kopyalı ayrıştırıcı, `'static` / `T: 'static`, generic önbellek (bound'u nereye koymalı), associated type mi generic mi |
| `learn/closures-iterators` | Yakalama modları ve Fn/FnMut/FnOnce, closure döndürme/saklama, sipariş veri işleme hattı, kendi iterator'ün, tembellik tuzağı |
| `learn/smart-pointers` | Seçim tablosu, `Rc` + `Weak` + `RefCell` ile dizin ağacı, referans döngüsü sızıntısını ölçmek, Cell/RefCell ile mock, RefCell tuzakları, `Rc::make_mut` |
| `learn/concurrency` | `thread::scope`, işçi havuzu, kilidi kısa tutmak, atomik sayaç, deadlock ve zehirli kilit |
| `learn/async-await` | Future tembelliği, sıralı vs eşzamanlı (ölçerek), elle `join`, asenkron API istemcisi, `pin!` mi `Box::pin` mi |
| `learn/oop` | Genişletilebilir bildirim sistemi, dyn / generic / enum karşılaştırması, `&dyn` / `Box` / `Rc` / `Arc` seçimi, gizli `'static` sınırı, upcasting, `dyn Any`, bileşim |
| `learn/patterns` | Komut ayrıştırıcı, bağlama modları, `@` ve guard, dilim desenleri, bütünlük (exhaustiveness) ve `matches!` |

Önerilen sıra: bir bölümün `main.rs`'ini oku → çalıştır → `derinlesme.rs`'i oku → alıştırmaları
çöz → sonraki bölüme geç. Kitap bölümleri bittikten sonra `ileri-*` crate'leri; lifetime ve
eşzamanlılık için sırasıyla `ileri-variance` → `ileri-lifetime-hrtb-gat` ve `ileri-future` →
`ileri-send-sync` → `ileri-eszamanlilik-async`.

## Çalıştırma

Workspace kökünde (`rust-study/`):

```bash
cargo run -p common-concepts      # bir bölümü çalıştır
cargo test -p writing-tests       # testleri olan crate'ler
cargo run -p minigrep -- hiç learn/minigrep/siir.txt
cargo test -p ownership           # derinleşme modüllerinin testleri
cargo run -p ileri-eszamanlilik-async
```

`guide/` referans kaynak ağacıdır, derlenmez (workspace'ten hariç).
