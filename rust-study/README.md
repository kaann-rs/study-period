# rust-study

Rust öğrenme çalışmam. Bir cargo workspace: `learn/*` kitabın bölümlerini/kavramları,
`project/*` kitaplardan ve kendi denemelerimden gelen küçük projeleri tutar. `guide/` =
referans için `rust-lang/rust` kaynak ağacı (workspace dışı).

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

## Sağlamlaştırma (ileri konular — `learn/ileri-*`)

Kitap bir temeldir ama mekanizmaların KÖKÜNÜ her zaman açmaz. Sağlamlaştırma fazı, genuinely
zor konuları kaynaktan (std + rustc) ve ileri kitaplardan derinlemesine, çalışan örneklerle
ele alır. Kaynaklar: *Rust for Rustaceans* (Jon Gjengset) ve *Programming Rust* (Blandy &
Orendorff).

| Crate | Konu | Ana kaynak |
| --- | --- | --- |
| `learn/ileri-future` | Future gerçekte nedir: poll/wake döngüsü, Waker + vtable, Pin ve self-referential state machine | std (`core::future`, `core::task`) + RfR Böl. 9 |
| _(sırada)_ `learn/ileri-variance` | Subtyping ve variance: covariant/invariant/contravariant, `&'a mut &'b T` neden iki lifetime ister | RfR Böl. 2 (Listing 2-11) |

## Çalıştırma

Workspace kökünde (`rust-study/`):

```bash
cargo run -p common-concepts      # bir bölümü çalıştır
cargo test -p writing-tests       # testleri olan crate'ler
cargo run -p minigrep -- hiç learn/minigrep/siir.txt
```

`guide/` referans kaynak ağacıdır, derlenmez (workspace'ten hariç).
