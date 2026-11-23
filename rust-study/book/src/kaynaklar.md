# Kaynaklar ve doğrulama

## Hangi sürüme göre?

Kitaptaki örnekler **Rust 1.99.0** ve **2024 edition** ile derlenmiştir. Kitap hazırlanırken güncel kararlı sürüm **1.100.0** idi (12 Kasım 2026).[^1100] Bu sürümle değişen konular ilgili bölümde belirtilir; başlıcaları şunlardır:

| 1.100.0'daki yenilik | İlgili bölüm |
| --- | --- |
| Never type (`!`) her yerde yazılabilir oldu | [19. Gelişmiş Özellikler](orta/19-advanced-features.md) |
| `Allocator` API'sinin çekirdeği kararlı oldu | [14. Akıllı İşaretçiler](orta/14-smart-pointers.md) |
| `std::mem::DropGuard` kararlı oldu | [14. Akıllı İşaretçiler](orta/14-smart-pointers.md) |
| `str::trim_prefix` ve `trim_suffix` kararlı oldu | [9.1 Generic, Trait ve Lifetime: Derinleşme](orta/09-generics-traits-lifetimes-derinlesme.md) |
| Closure'lar `if let` içindeki değişkenleri daha isabetli yakalıyor | [12. Closure ve Iterator'lar](orta/12-closures-iterators.md) |
| Cargo kullanılmayan bağımlılıkları bildiriyor; `cargo install` varsayılan olarak `Cargo.lock`'u kullanıyor | [13. Cargo ve Crate'ler](orta/13-cargo-and-crates.md) |

Derleyicinin içini anlatan bilgiler sürümden sürüme en çok değişen bilgilerdir. Bu yüzden kaynak kodu bağlantıları `1.99.0` etiketini gösterir; bağlantıdaki satır numarası o sürüm için geçerlidir.

## Başvurulan kaynaklar

| Kaynak | Ne içerir? |
| --- | --- |
| [The Rust Programming Language](https://doc.rust-lang.org/book/) | Dilin resmî kitabı. Dipnotlarda bölüm numarasıyla anılır. |
| [The Rust Reference](https://doc.rust-lang.org/reference/) | Dil kurallarının başvuru belgesi. |
| [The Rustonomicon](https://doc.rust-lang.org/nomicon/) | `unsafe` Rust ve bellek modeli rehberi. |
| [Standart kütüphane belgesi](https://doc.rust-lang.org/std/) | `std`, `core` ve `alloc` türlerinin belgesi. |
| [The Cargo Book](https://doc.rust-lang.org/cargo/) | Cargo'nun belgesi. |
| [Error Index](https://doc.rust-lang.org/error_codes/error-index.html) | Derleyici hata kodlarının açıklamaları. |
| [Edition Guide](https://doc.rust-lang.org/edition-guide/) | Edition'lar arasındaki farklar. |
| [Rust by Example](https://doc.rust-lang.org/rust-by-example/) | Örneklerle Rust. |
| [rustc dev guide](https://rustc-dev-guide.rust-lang.org/) | Derleyicinin iç yapısı. |
| [RFC kitabı](https://rust-lang.github.io/rfcs/) | Dil özelliklerinin tasarım belgeleri. |
| [rustc 1.99.0 kaynak kodu](https://github.com/rust-lang/rust/tree/1.99.0) | Derleyicinin ve standart kütüphanenin kendisi. |

Notlarda adı geçen basılı kitaplar ad ve bölümle anılır: Jon Gjengset, *Rust for Rustaceans*; Jim Blandy, Jason Orendorff ve Leonora Tindall, *Programming Rust*; Mara Bos, *Rust Atomics and Locks*.

## Denetim nasıl yapıldı?

Kitabın kökündeki `dogrula.py` betiği bütün sayfaları denetler:

- **Kod.** Her Rust bloğu `rustdoc` ile derlenir ve çalıştırılır. Derlenmemesi gereken örneklerin, yazılan hata koduyla başarısız olduğu da denetlenir.
- **Çıktı.** "Çıktı:" ile verilen metin, programın gerçek çıktısıyla karşılaştırılır. "Derleyicinin yanıtı:" ile verilen hata, `rustc`'nin gerçek mesajıyla karşılaştırılır.
- **Bağlantılar.** Resmî belge bağlantıları ve sayfa içi hedefleri 1.99.0 belgesinde aranır. Kaynak kodu bağlantılarının gösterdiği satırın yakınında, bağlantı metnindeki adın gerçekten geçtiği denetlenir. Öteki bağlantılar internetten denenir.
- **Yapı.** Başlıklar, dipnotlar, bölümler arası bağlantılar ve biçim kuralları denetlenir.
- **Kapsam.** İstenirse, notlarda ve kitabın ilk hâlinde geçip yeni sayfada geçmeyen terimler listelenir.

```console
python3 dogrula.py
python3 dogrula.py src/temel/03-ownership.md
python3 dogrula.py --kapsam src/temel/03-ownership.md src/temel/03-ownership-derinlesme.md
```

## Denetimin sınırları

- Araç, düzyazıdaki bir cümlenin doğru olduğunu kanıtlayamaz; yalnızca kodun, çıktının ve bağlantının geçerli olduğunu gösterir. Cümleler yazılırken dipnottaki kaynak okunarak doğrulanmıştır.
- `ignore` işaretli örnekler (yordamsal makro crate'leri, nightly özellikler, birden çok dosyalı projeler) rustdoc ile çalıştırılamaz. Bunlar ayrı projelerde denenmiştir.
- "Örnek bir çalıştırma:" ile verilen çıktılar makineye ve zamana bağlıdır.
- Birincil kaynakta doğrulanamayan bir bilgi kullanıldıysa yanında bu belirtilir.

Bir hata görürseniz önce dipnottaki kaynağa bakın; kitap ile resmî belge çelişiyorsa **resmî belge geçerlidir**.

## Kaynaklar

[^1100]: The Rust Blog, [Announcing Rust 1.100.0](https://blog.rust-lang.org/2026/11/12/Rust-1.100.0/).
