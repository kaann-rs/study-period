# rust-study

Rust öğrenme çalışmam. Bir cargo workspace: `learn/*` kitabın bölümlerini/kavramları
(ileri konular `learn/advanced-concepts/` altında),
`project/*` kitaplardan ve kendi denemelerimden gelen küçük projeleri tutar. `guide/` =
referans için `rust-lang/rust` kaynak ağacı (workspace dışı).

## Nasıl okunur?

Anlatım [`book/`](book/) altındaki kitaptadır (`mdbook serve --open`). `learn/` yalnızca kodu tutar:
her `.rs` dosyasının ilk satırı, konunun kitaptaki bölümünü gösterir. Kodda fonksiyon, değişken ve tür
adları İngilizce, program çıktıları ve kitaptaki örnekler Türkçedir. Aşağıdaki tablolardaki "Böl."
sütunu ise resmî Rust Book'un bölüm numarasıdır.

## Rust Book yolculuğu (ch3–ch20)

[doc.rust-lang.org/book](https://doc.rust-lang.org/book/) (2024 edition) tüm kavramsal
bölümlerini sırayla birer crate olarak işledim.

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
| `learn/advanced-concepts/advanced-future` | Future gerçekte nedir: poll/wake döngüsü, Waker + vtable, Pin ve self-referential state machine | std (`core::future`, `core::task`) + RfR Böl. 8 |
| `learn/advanced-concepts/advanced-variance` | Subtyping ve variance: covariant/invariant/contravariant, `&'a mut &'b T` neden iki lifetime ister | RfR Böl. 1 + Reference + Nomicon |
| `learn/advanced-concepts/advanced-sized-dst` | Sized, DST (`str`/`[T]`/`dyn`), fat/wide pointer (ptr + uzunluk/vtable), `?Sized` | RfR Böl. 2 + std |
| `learn/advanced-concepts/advanced-deref-methods` | Deref + metot çözümü: deref coercion, autoref/autoderef, inherent metot gölgeleme | RfR Böl. 3 + Reference + std |
| `learn/advanced-concepts/advanced-send-sync` | UnsafeCell (iç değişkenliğin tek temeli) + Send/Sync auto-trait'leri, !Sync negatif impl | RfR Böl. 3 & 9 + std |
| `learn/advanced-concepts/advanced-dropck` | Drop check: Drop + lifetime neden borrow'u aşmaya zorlar; PhantomData (ZST işaretçi); #[may_dangle] | RfR Böl. 9 + std |
| `learn/advanced-concepts/advanced-coherence` | Coherence + orphan rule (tam kural), blanket impl, fundamental tipler, covered impl | RfR Böl. 2 + std |
| `learn/advanced-concepts/advanced-lifetime-hrtb-gat` | İleri lifetime'lar: iki ayrı süre, `'a: 'b` / `T: 'a`, HRTB (`for<'a>`), GAT (ödünç veren iterator, tür ailesi), lifetime hata mesajı rehberi | Reference + Rustonomicon + RfR Böl. 1–2 |
| `learn/advanced-concepts/advanced-concurrency-async` | İleri eşzamanlılık + async: RwLock, Condvar, kilit sıralaması, atomikler; std ile mini runtime (görev kuyruğu, spawn, zamanlayıcı, zaman aşımı), Pin/Unpin pratikte, asenkron API istemcisi, yaygın async hataları | std + *Rust Atomics and Locks* + RfR Böl. 8 & 10 |

## Derinleşme modülleri (`src/deep_dive.rs`)

Zor konuların crate'lerinde `main.rs` kavramı tanıtır; yanındaki `deep_dive.rs` aynı konuyu
PRATİĞE döker. `cargo run -p <crate>` önce özgün bölümü, ardından derinleşme bölümünü çalıştırır;
`cargo test -p <crate>` birim testlerini koşturur. Senaryolar, sık yapılan hatalar ve alıştırmalar
kitabın derinleşme sayfalarındadır.

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

Önerilen sıra: kitabın bölümünü oku → crate'i çalıştır → derinleşme sayfasını oku → alıştırmaları
çöz → sonraki bölüme geç. Kitap bölümleri bittikten sonra `advanced-*` crate'leri; lifetime ve
eşzamanlılık için sırasıyla `advanced-variance` → `advanced-lifetime-hrtb-gat` ve `advanced-future` →
`advanced-send-sync` → `advanced-concurrency-async`.

## Çalıştırma

Workspace kökünde (`rust-study/`):

```bash
cargo run -p common-concepts      # bir bölümü çalıştır
cargo test -p writing-tests       # testleri olan crate'ler
cargo run -p minigrep -- hiç learn/minigrep/siir.txt
cargo test -p ownership           # derinleşme modüllerinin testleri
cargo run -p advanced-concurrency-async
```

`guide/` referans kaynak ağacıdır, derlenmez (workspace'ten hariç).
