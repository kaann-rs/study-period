# 13. Cargo ve Crate'ler

Yazdığınız kod kendi bilgisayarınızda çalışıyor. Onu başkalarının da kullanabileceği bir kütüphaneye dönüştürmek istediğinizde yeni sorular ortaya çıkar: Kullanıcı hangi fonksiyonu nasıl çağıracağını nereden öğrenecek? Örnekleriniz kod değiştikçe eskimeyecek mi? İçerideki düzeniniz kullanıcıya yük olacak mı? Yavaş çalışan geliştirme sürümü ile dağıtacağınız sürüm nasıl ayrılacak?

Bu soruların yanıtı Cargo'dadır. Cargo paketi derler, test eder, belgelerini üretir, bağımlılıklarını ve sürümlerini yönetir.[^cargo-daha] Bu bölümde küçük bir renk karıştırma kütüphanesini adım adım paketleyeceğiz. Kılavuzu kodun içine yazacağız, kılavuzdaki örnekler test olarak çalışacak, iç düzeni kullanıcıya dayatmadan kısa bir kapı açacağız. Ardından profillere, çalışma alanlarına (workspace) ve yayımlamaya bakacağız.

> [!NOTE]
> Bu bölümün kodu `learn/cargo-and-crates` dizinindedir. Depo kökünden `cargo run -p cargo-and-crates` ile programı, `cargo test -p cargo-and-crates` ile testleri çalıştırabilirsiniz.

## Günlük hayattan bir benzetme

Kutusuyla satılan bir boya seti düşünün. Kutunun içinde bölmeler vardır: birinde tüpler, ötekinde karıştırma kabı durur. Kapağı açan kişi en çok kullanacağı şeyleri en üstte görür; hangi tüpün hangi gözde durduğunu ezberlemesi gerekmez. Yine de isterse gözleri tek tek açıp bakabilir.

Kutunun arkasında "sarı + mavi = yeşil" yazar. Üretici formülü değiştirip yazıyı düzeltmeyi unutursa kutu yanlış bilgi verir. Rust'ta bu yazı yalnızca bir yazı değildir; aynı zamanda sınavdır. Kılavuzdaki her örnek gerçekten denenir, tutmayan örnek hemen ortaya çıkar.

Kutunun kendisi *paket* (package), içindeki derlenen parça *crate*'tir. Bölmeler *modüllere*, kapağın altındaki kısayol `pub use`'a, arkadaki yazı *belgeleme yorumlarına*, denenen örnekler *belge testlerine* karşılık gelir.

<details>
<summary>Bu bölümde geçen terimler</summary>

- **Cargo:** Rust'ın derleme, test, belge üretme ve paket yönetimi aracı.
- **crate:** Derleyicinin bir kerede ele aldığı kod birimi; ya bir kütüphane ya da bir programdır.
- **paket (package):** Bir `Cargo.toml` ile tanımlanan, bir ya da daha çok crate içeren kutu.
- **crate kökü (crate root):** Crate'in en üst düzeyi; `cargo_and_crates::...` yollarının başladığı yer.
- **crates.io ve docs.rs:** Paketlerin yayımlandığı site; yayımlanan paketlerin belgelerinin üretilip sunulduğu site.
- **belgeleme yorumu (documentation comment):** `///` ya da `//!` ile yazılan ve kılavuza dönüşen yorum. Düz `//` yorumu kılavuza girmez.
- **Markdown ve HTML:** Kılavuzu yazdığınız yalın biçim; tarayıcıda açılan sayfa biçimi.
- **belge testi (doc test):** Kılavuzdaki örnek kodun `cargo test` ile derlenip çalıştırılması.
- **modül (module):** Kodu gruplayan adlı bölme.
- **`pub`:** "Dışarıdan görünsün" işareti.
- **yeniden dışa aktarma (re-export, `pub use`):** İçerideki bir öğeyi başka bir yoldan da dışarı açmak.
- **API:** Crate'in dışarıya açtığı fonksiyon ve türlerin tamamı.
- **bağımlılık (dependency):** Crate'inizin kullandığı başka bir crate.
- **`Cargo.toml` ve `Cargo.lock`:** Paketin sizin yazdığınız bildirimi; kullanılan sürümlerin Cargo'nun tuttuğu kesin listesi.
- **profil (profile):** Derleyici ayarlarının adlandırılmış takımı: `dev`, `release`.
- **`opt-level`:** Derleyicinin kodu ne ölçüde iyileştireceğini belirleyen profil ayarı.
- **çalışma alanı (workspace):** Birlikte yönetilen paketlerin toplandığı çatı.
- **`target/`:** Derleme çıktılarının konduğu dizin.

</details>

## Paket, crate ve Cargo dosyaları

*Crate*, derleyicinin bir kerede ele aldığı en küçük kod birimidir. İki biçimi vardır: `main` fonksiyonu olan ve çalıştırılabilir dosyaya dönüşen *program crate'i* (binary crate) ile başka crate'lerin kullanması için yazılan *kütüphane crate'i* (library crate). *Paket* ise bir ya da daha çok crate'i bir araya getiren ve nasıl derleneceğini `Cargo.toml` ile anlatan birimdir. Bir pakette istediğiniz kadar program crate'i bulunabilir, ama en fazla bir kütüphane crate'i olabilir.[^paket]

Örneğimiz ikisini birden içerir:

```text
cargo-and-crates
├── Cargo.toml
└── src
    ├── lib.rs     kütüphane crate'inin kökü
    └── main.rs    program crate'inin kökü; kütüphaneyi kullanır
```

`src/main.rs` ve `src/lib.rs` adları bir uzlaşımdır; Cargo bu dosyaları gördüğünde paketle aynı adı taşıyan iki crate olduğunu anlar.[^paket] Paket adındaki tireler kütüphane crate'inde alt çizgiye dönüşür. Paket `cargo-and-crates` adını taşıdığı hâlde kodda `cargo_and_crates` yazmanızın nedeni budur.[^hedef-adi]

Paketin bildirimi kısadır:

```toml
[package]
name = "cargo-and-crates"
version = "0.1.0"
edition = "2024"
description = "Bölüm 14 örneği: doc yorumları, doc-test ve pub use ile yeniden dışa aktarma"

[dependencies]
```

`description` alanındaki "Bölüm 14", *The Rust Programming Language* kitabının bölüm numarasıdır.

Cargo iki dosyayı birbirinden ayırır. `Cargo.toml`'u siz yazarsınız; bağımlılıklarınızı geniş anlamda, "bu crate'in 1.2 ve uyumlu sürümleri" gibi anlatır. `Cargo.lock`'u Cargo tutar; bağımlılıkların gerçekte hangi sürümüyle derlendiğini kaydeder. `Cargo.lock` depoya eklendiğinde aynı projeyi derleyen herkes aynı sürümleri kullanır.[^lock]

> [!NOTE]
> Cargo 1.100 ile iki yenilik geldi. Birincisi, bildirdiğiniz hâlde kullanmadığınız bağımlılıklar için varsayılan olarak uyarı veren `cargo::unused_dependencies` denetimidir; düzeyi `[lints.cargo]` tablosundan değiştirilebilir.[^unused-dep] İkincisi, isteğe bağlı `min-publish-age` ayarıdır: Ayarlarsanız sürüm çözücü, yayımlanmasının üzerinden belirlediğiniz süre geçmemiş sürümleri `Cargo.lock`'ta zaten yoksa seçmez.[^publish-age] Örnekler 1.99 ile derlenmiştir.

## Modüller: kutunun bölmeleri

Tüplerle karıştırma kabı aynı gözde dursa da boyayabilirsiniz, ama aradığınızı bulmak zorlaşır. Kütüphane bu yüzden iki modüle ayrılmıştır: `turler` renk türlerini, `islem` onlarla yapılan işi tutar:

```rust
pub mod turler {
    /// Birincil renkler (karışımın girdileri).
    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum BirincilRenk {
        Kirmizi,
        Sari,
        Mavi,
    }

    /// İkincil renkler (iki birincilin karışımı).
    #[derive(Debug, PartialEq)]
    pub enum IkincilRenk {
        Turuncu,
        Yesil,
        Mor,
    }
}

pub mod islem {
    use crate::turler::{BirincilRenk, IkincilRenk};

    pub fn karistir(a: BirincilRenk, b: BirincilRenk) -> Option<IkincilRenk> {
        use BirincilRenk::*;
        let mut ciftler = [a, b];
        ciftler.sort_by_key(|r| *r as u8); // sıra: Kirmizi, Sari, Mavi
        match ciftler {
            [Kirmizi, Sari] => Some(IkincilRenk::Turuncu),
            [Sari, Mavi] => Some(IkincilRenk::Yesil),
            [Kirmizi, Mavi] => Some(IkincilRenk::Mor),
            _ => None, // aynı renk iki kez
        }
    }
}

fn main() {
    let sonuc = islem::karistir(turler::BirincilRenk::Mavi, turler::BirincilRenk::Sari);
    println!("{sonuc:?}");
}
```

Çıktı:

```text
Some(Yesil)
```

Gövdedeki `use BirincilRenk::*;` varyantları kısa adlarıyla kullanmayı sağlar. `karistir` girdilerin sırasına bakmaz: Çifti önce sıralar, sonra tek bir biçime göre eşleştirir. `*r as u8`, varyantın tanımdaki sıra numarasını verir (`Kirmizi` 0, `Sari` 1, `Mavi` 2).[^ayirt-edici] Böylece "Mavi, Sari" ile "Sari, Mavi" aynı kola düşer.

Yollara dikkat edin: Türün tam yolu `cargo_and_crates::turler::BirincilRenk`, fonksiyonunki `cargo_and_crates::islem::karistir`'dır. Kod büyüdükçe bölmeler sayesinde "bu nerede?" sorusunu yanıtlamak kolaylaşır. Modüllerin kuralları [6. Modüller](../basics/06-modules.md) bölümünde anlatıldı.

## Belgeleme yorumları

Rust'ta iki tür belgeleme yorumu vardır. `///`, kendisinden *sonra* gelen öğeyi belgeler. `//!` ise *içinde bulunduğu* öğeyi belgeler; çoğunlukla `src/lib.rs`'in başında crate'i, modülün başında da modülü anlatmak için kullanılır.[^belge-yorumu][^ic-yorum] İkisi de Markdown destekler. Düz `//` yorumu belgeye girmez.

```rust
pub mod turler {
    //! Renk türleri. Bu satırlar modülün kendisini anlatır.

    /// Birincil renkler (karışımın girdileri).
    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum BirincilRenk {
        /// Sıcak renk.
        Kirmizi,
        Sari,
        Mavi,
    }
}

fn main() {
    println!("{:?}", turler::BirincilRenk::Kirmizi);
}
```

Çıktı:

```text
Kirmizi
```

Örnek crate'in başında, kütüphanenin tamamını anlatan şu satırlar durur:

```text
//! # cargo-and-crates
//!
//! Bir renk karıştırma kütüphanesi:
//! ```
//! use cargo_and_crates::{BirincilRenk, karistir};
//! let yesil = karistir(BirincilRenk::Sari, BirincilRenk::Mavi);
//! ```
```

Bu yorumları tarayıcıda okunan sayfalara `cargo doc` çevirir. Komut, Rust ile birlikte gelen `rustdoc` aracını çalıştırır ve HTML dosyalarını `target/doc` altına yazar; `--open` seçeneği sonucu tarayıcıda açar:[^belge-yorumu]

```console
$ cargo doc -p cargo-and-crates --open
```

Belgelerde yaygın olarak kullanılan bölüm başlıkları vardır: kullanım örneği için `# Examples`, fonksiyonun hangi durumda panikleyeceğini anlatan `# Panics`, `Result` döndürüyorsa hangi hataların çıkabileceğini anlatan `# Errors`, `unsafe` fonksiyonlarda çağıranın koruması gereken koşulları anlatan `# Safety`.[^bolumler] Hepsini yazmak zorunda değilsiniz; kullanıcının bilmek isteyeceği şeyler için bir denetim listesi gibi düşünün.

## Belge testleri

Kılavuzdaki örnek kod zamanla asıl koddan uzaklaşır ve kullanıcıyı yanıltır; bu duruma *belge çürümesi* (doc rot) denir. Rust bunu, belgeleme yorumlarındaki kod bloklarını test olarak derleyip çalıştırarak önler.[^belge-testi] `karistir`'ın üstündeki kılavuz şöyledir:

```rust
# pub mod turler {
#     #[derive(Debug, PartialEq, Clone, Copy)]
#     pub enum BirincilRenk { Kirmizi, Sari, Mavi }
#     #[derive(Debug, PartialEq)]
#     pub enum IkincilRenk { Turuncu, Yesil, Mor }
# }
pub mod islem {
    use crate::turler::{BirincilRenk, IkincilRenk};

    /// İki birincil rengi karıştırıp ikincil rengi döndürür.
    ///
    /// Aynı renk iki kez verilirse `None` döner (karışım tanımsız).
    /// ```
    /// use cargo_and_crates::{BirincilRenk, IkincilRenk, karistir};
    /// assert_eq!(karistir(BirincilRenk::Kirmizi, BirincilRenk::Sari), Some(IkincilRenk::Turuncu));
    /// assert_eq!(karistir(BirincilRenk::Mavi, BirincilRenk::Mavi), None);
    /// ```
    pub fn karistir(a: BirincilRenk, b: BirincilRenk) -> Option<IkincilRenk> {
        // gövde yukarıdaki gibi
#         use BirincilRenk::*;
#         let mut ciftler = [a, b];
#         ciftler.sort_by_key(|r| *r as u8);
#         match ciftler {
#             [Kirmizi, Sari] => Some(IkincilRenk::Turuncu),
#             [Sari, Mavi] => Some(IkincilRenk::Yesil),
#             [Kirmizi, Mavi] => Some(IkincilRenk::Mor),
#             _ => None,
#         }
    }
}
# fn main() {}
```

Üç ters tırnak arasındaki satırlar `cargo test` komutuyla çalışır. `--doc` seçeneği yalnızca belge testlerini koşturur:

```console
$ cargo test -p cargo-and-crates --doc
   Doc-tests cargo_and_crates

running 2 tests
test learn/cargo-and-crates/src/lib.rs - islem::karistir (line 35) ... ok
test learn/cargo-and-crates/src/lib.rs - (line 6) ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

İki test vardır: biri `karistir`'ın üstündeki örnek, öteki dosyanın başındaki `//!` satırlarında duran örnek. Testler paralel çalıştığından sıraları değişebilir. Belge testleri yalnızca kütüphane crate'inden toplanır;[^cargo-test] program crate'indeki yorumlar test edilmez.

Şimdi formülü değiştirip kılavuzu düzeltmeyi unuttuğumuzu varsayalım: `[Kirmizi, Sari]` kolu yanlışlıkla `Yesil` döndürsün. Test düşer ve hangi değerlerin tutmadığını söyler (çıktı kısaltılmıştır):

```console
$ cargo test -p cargo-and-crates --doc
running 2 tests
test learn/cargo-and-crates/src/lib.rs - (line 6) ... ok
test learn/cargo-and-crates/src/lib.rs - islem::karistir (line 35) ... FAILED

---- learn/cargo-and-crates/src/lib.rs - islem::karistir (line 35) stdout ----
Test executable failed (exit status: 101).

stderr:
thread 'main' panicked at /tmp/rustdoctest.../doctest_bundle_2024.rs:16:1:
assertion `left == right` failed
  left: Some(Yesil)
 right: Some(Turuncu)

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Bir belge testi, derlenip panik vermeden biterse geçer.[^gecer] Örnek derlenmezse de test düşer; artık var olmayan bir fonksiyon adını ya da değişmiş bir imzayı bu yolla yakalarsınız.

> [!NOTE]
> Belge testi kodun her zaman doğru olduğunu kanıtlamaz, yalnızca örnekte yazılanı sınar. `karistir`'ın hiçbir örnekte geçmeyen bir kolu yanlışsa belge testi bunu göremez. Bu görev birim testlerinindir.

### Örneği hazırlayan kurallar

Yukarıdaki örnekte `main` fonksiyonu yoktur. Rustdoc, kodu çalıştırmadan önce biraz hazırlar: Yaygın uyarıları susturur, crate'inizi örneğe tanıtır ve kodda `fn main` yoksa hepsini bir `main` içine alır.[^hazirlik] `#` ve boşlukla başlayan satırlar derlenir, ama belgede gösterilmez; örneğin hazırlık kısmını gizlemek için kullanılır.[^gizli]

Kod bloğunun açılış satırına yazılan öznitelikler davranışı değiştirir:[^nitelik]

| Öznitelik | Ne yapar? |
| --- | --- |
| `should_panic` | Kod derlenmeli ve paniklemelidir. |
| `no_run` | Kod derlenir, çalıştırılmaz; ağa çıkan örnekler için uygundur. |
| `compile_fail` | Kod derlenmemelidir; derlenirse test düşer. |
| `ignore` | Kod hiç denenmez. Çoğu zaman en kötü seçenektir; nedenini yazmak alışkanlıktır. |
| `text` | Blok Rust kodu sayılmaz. |
| `standalone_crate` | Kod öteki örneklerle birleştirilmez, tek başına derlenir. |

Son satır, dilin 2024 sürümüyle (2024 edition) ilgilidir. Daha önce her örnek ayrı bir program olarak derlenirdi. 2024 edition'ında uyumlu örnekler tek bir ikili dosyada birleştirilerek derlenir; bu, çok örnekli crate'lerde derleme süresini önemli ölçüde kısaltır. Her örnek yine *kendi sürecinde* çalışır, bu yüzden genel durum paylaşılmaz.[^edition-2024] `compile_fail` özniteliği taşıyan ya da crate düzeyinde öznitelik (`#![...]`) içeren örnekler birleştirilmez.

## `pub use`: kapağın altındaki kısayol

Seti alan kişi tüplerin hangi bölmede durduğunu ezberlemek istemez. `pub use`, bir öğeyi olduğu yerde bırakır ve başka bir yoldan da erişilir kılar; buna *yeniden dışa aktarma* (re-export) denir.[^yeniden] Örnek crate'in kökünde şu iki satır vardır:

```rust
# pub mod turler {
#     #[derive(Debug, PartialEq, Clone, Copy)]
#     pub enum BirincilRenk { Kirmizi, Sari, Mavi }
#     #[derive(Debug, PartialEq)]
#     pub enum IkincilRenk { Turuncu, Yesil, Mor }
# }
# pub mod islem {
#     use crate::turler::{BirincilRenk, IkincilRenk};
#     pub fn karistir(a: BirincilRenk, b: BirincilRenk) -> Option<IkincilRenk> {
#         use BirincilRenk::*;
#         let mut ciftler = [a, b];
#         ciftler.sort_by_key(|r| *r as u8);
#         match ciftler {
#             [Kirmizi, Sari] => Some(IkincilRenk::Turuncu),
#             [Sari, Mavi] => Some(IkincilRenk::Yesil),
#             [Kirmizi, Mavi] => Some(IkincilRenk::Mor),
#             _ => None,
#         }
#     }
# }
pub use islem::karistir;
pub use turler::{BirincilRenk, IkincilRenk};
# fn main() {}
```

`src/main.rs`, seti kutusundan çıkarıp kullanan kişidir. Artık modül adlarını hiç anmadan kütüphaneyi kökten kullanabilir:

```rust
# mod cargo_and_crates {
#     pub mod turler {
#         #[derive(Debug, PartialEq, Clone, Copy)]
#         pub enum BirincilRenk { Kirmizi, Sari, Mavi }
#         #[derive(Debug, PartialEq)]
#         pub enum IkincilRenk { Turuncu, Yesil, Mor }
#     }
#     pub mod islem {
#         use super::turler::{BirincilRenk, IkincilRenk};
#         pub fn karistir(a: BirincilRenk, b: BirincilRenk) -> Option<IkincilRenk> {
#             use BirincilRenk::*;
#             let mut ciftler = [a, b];
#             ciftler.sort_by_key(|r| *r as u8);
#             match ciftler {
#                 [Kirmizi, Sari] => Some(IkincilRenk::Turuncu),
#                 [Sari, Mavi] => Some(IkincilRenk::Yesil),
#                 [Kirmizi, Mavi] => Some(IkincilRenk::Mor),
#                 _ => None,
#             }
#         }
#     }
#     pub use islem::karistir;
#     pub use turler::{BirincilRenk, IkincilRenk};
# }
use cargo_and_crates::{BirincilRenk, karistir};

fn main() {
    for (a, b) in [
        (BirincilRenk::Kirmizi, BirincilRenk::Sari),
        (BirincilRenk::Sari, BirincilRenk::Mavi),
        (BirincilRenk::Kirmizi, BirincilRenk::Mavi),
        (BirincilRenk::Mavi, BirincilRenk::Mavi),
    ] {
        println!("{a:?} + {b:?} = {:?}", karistir(a, b));
    }
}
```

Çıktı:

```text
Kirmizi + Sari = Some(Turuncu)
Sari + Mavi = Some(Yesil)
Kirmizi + Mavi = Some(Mor)
Mavi + Mavi = None
```

`pub use` iç modülleri *gizlemez*. `use cargo_and_crates::turler::BirincilRenk;` diye yazılan uzun yol çalışmaya devam eder; kısa yol ona ek bir kapıdır. Kısa yol yalnızca dışa aktarılan adlar için vardır. `Mor`, `IkincilRenk`'in bir varyantıdır ve kökte böyle bir öğe yoktur:

```rust,compile_fail,E0432
# mod cargo_and_crates {
#     pub mod turler {
#         pub enum IkincilRenk { Turuncu, Yesil, Mor }
#     }
#     pub use turler::IkincilRenk;
# }
use cargo_and_crates::Mor;
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0432]: unresolved import `cargo_and_crates::Mor`
```

İç yolu gerçekten kapatmak istiyorsanız modülü `pub` yapmazsınız. O zaman kısa yol çalışır, uzun yol E0603 (``module `turler` is private``) verir. Böylece iç düzeninizi sonradan, kullanıcının kodunu bozmadan değiştirebilirsiniz; dışarıya söz verdiğiniz yalnızca köktür.

### Bir ad, birden çok ad alanı

Bir `use` bildirimi tek bir ad alanına (namespace) ait değildir; adı, o adın bulunduğu bütün ad alanlarında kapsama alır.[^use-ad-alani] Aşağıda `Boya` hem bir tür hem de bir kurucu fonksiyondur ve tek satırda ikisi de dışa aktarılır:

```rust
mod renkler {
    #[derive(Debug)]
    pub struct Boya(pub u8);
}

pub use renkler::Boya;

fn main() {
    let uret: fn(u8) -> Boya = Boya; // değer olarak: kurucu
    let b: Boya = uret(3); // tür olarak
    println!("{b:?}");
}
```

Çıktı:

```text
Boya(3)
```

Rust'ta beş ad alanı vardır: tür, değer, makro, ömür ve etiket.[^ad-alani] Aynı ad, farklı ad alanlarında çakışmadan durabilir. Yukarıdaki `Boya` tür ad alanında bir tür, değer ad alanında bir fonksiyondur.

## Bütün testleri birlikte çalıştırmak

Kütüphanede belge testlerinin yanında iki de birim testi bulunur:

```rust
# #[derive(Debug, PartialEq, Clone, Copy)]
# pub enum BirincilRenk { Kirmizi, Sari, Mavi }
# #[derive(Debug, PartialEq)]
# pub enum IkincilRenk { Turuncu, Yesil, Mor }
# fn karistir(a: BirincilRenk, b: BirincilRenk) -> Option<IkincilRenk> {
#     use BirincilRenk::*;
#     let mut ciftler = [a, b];
#     ciftler.sort_by_key(|r| *r as u8);
#     match ciftler {
#         [Kirmizi, Sari] => Some(IkincilRenk::Turuncu),
#         [Sari, Mavi] => Some(IkincilRenk::Yesil),
#         [Kirmizi, Mavi] => Some(IkincilRenk::Mor),
#         _ => None,
#     }
# }
# fn main() {}
#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn yesil_uretilir() {
        assert_eq!(karistir(BirincilRenk::Sari, BirincilRenk::Mavi), Some(IkincilRenk::Yesil));
    }

    #[test]
    fn ayni_renk_none() {
        assert_eq!(karistir(BirincilRenk::Kirmizi, BirincilRenk::Kirmizi), None);
    }
}
```

`cargo test`, paketin bütün testlerini sırayla çalıştırır: kütüphanenin birim testleri, programın birim testleri (burada yoktur) ve belge testleri:[^cargo-test]

```console
$ cargo test -p cargo-and-crates
     Running unittests src/lib.rs

running 2 tests
test testler::yesil_uretilir ... ok
test testler::ayni_renk_none ... ok

     Running unittests src/main.rs

running 0 tests

   Doc-tests cargo_and_crates

running 2 tests
test learn/cargo-and-crates/src/lib.rs - islem::karistir (line 35) ... ok
test learn/cargo-and-crates/src/lib.rs - (line 6) ... ok
```

Birim ve entegrasyon testlerinin ayrıntısı [10. Test Yazma](10-writing-tests.md) bölümündedir.

## Derleme profilleri

*Profil*, derleyici ayarlarının adlandırılmış bir takımıdır. Günlük işte iki tanesiyle karşılaşırsınız. `cargo build` `dev` profilini kullanır: hızlı derleme, yavaş çalışan kod. `cargo build --release` ise `release` profilini kullanır: yavaş derleme, hızlı çalışan kod.[^profil-kitap]

Örnek bir çalıştırma (süreler değişir):

```console
$ cargo build -p cargo-and-crates
   Compiling cargo-and-crates v0.1.0 (...)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.49s
$ cargo build -p cargo-and-crates --release
   Compiling cargo-and-crates v0.1.0 (...)
    Finished `release` profile [optimized] target(s) in 0.42s
```

Bu kadar küçük bir crate'te derleme süreleri arasındaki fark görülmez; fark, kod büyüdükçe belirginleşir. İki profilin davranışını belirleyen ayarların başında `opt-level` gelir; varsayılan değeri `dev` için 0, `release` için 3'tür.[^profil-dev][^profil-release] Ayar şu değerleri alır:[^opt-level]

| Değer | Anlamı |
| --- | --- |
| `0` | İyileştirme yok |
| `1` | Temel iyileştirmeler |
| `2` | Bazı iyileştirmeler |
| `3` | Bütün iyileştirmeler |
| `"s"` | İkili dosyanın boyutunu küçült |
| `"z"` | Boyutu küçült, döngü vektörleştirmesini de kapat |

Büyük değer kesin olarak daha hızlı kod demek değildir; belgeler düzey 3'ün 2'den yavaş kalabileceğini, `"s"` ve `"z"`'nin de her zaman daha küçük çıkmayabileceğini söyler.[^opt-level] Yine de bir kural güvenle uygulanabilir: Hız ölçümünü `--release` ile yapın. `dev` ikilisi gerçek hızı göstermez.

Varsayılanları değiştirmek için `Cargo.toml`'a `[profile.*]` bölümü yazılır:

```toml
[profile.dev]
opt-level = 1
```

Cargo profil ayarlarını yalnızca çalışma alanının kökündeki `Cargo.toml`'dan okur.[^profil-kok] Üye bir pakete yazarsanız ayar yok sayılır ve Cargo uyarır:

```text
warning: profiles for the non root package will be ignored, specify profiles at the workspace root:
```

`cargo-and-crates` paketinde profil bölümü bulunmamasının nedeni budur. Yerleşik profiller `dev`, `release`, `test` ve `bench`'tir; 1.99 ile, bugün `dev`'den farkı olmayan `debug` de eklendi.[^profil-debug] Kendi profilinizi de tanımlayabilirsiniz.

## Çalışma alanları (workspace)

*Çalışma alanı* (workspace), birlikte yönetilen paketlerin kümesidir.[^ws] Bu deponun kendisi bir çalışma alanıdır ve kökündeki `Cargo.toml` yalnızca üyeleri sayar:

```toml
[workspace]
resolver = "3"
members = [
    "learn/*",
    "learn/advanced-concepts/advanced-coherence",
    # ... öteki advanced-* crate'leri
    "project/*",
]
exclude = ["guide", "learn/advanced-concepts"]
```

`[package]` bölümü olmayan böyle bir dosyaya *sanal bildirim* (virtual manifest) denir.[^ws] `members` dizinleri ve `*` gibi kalıpları kabul eder, `exclude` çıkarılacak yolları sayar. `learn/*` kalıbı, kendisi bir crate olmayan `learn/advanced-concepts` klasörünü de yakaladığından bu klasör `exclude`'a yazılmış, içindeki crate'ler tek tek üye yapılmıştır. Açıkça yazılan üye, `exclude` kapsamında olsa da üyedir; Cargo 1.99 ile denendi. `resolver = "3"`, 2024 edition'ının varsayılan bağımlılık çözücüsüdür; sanal bildirimde paket ve dolayısıyla `edition` alanı bulunmadığından açıkça yazılır.[^resolver]

Çalışma alanının üç yararı vardır:

- Bütün üyeler tek bir `Cargo.lock` paylaşır; uyumlu sürüm isteyen üyeler aynı bağımlılık sürümünü kullanır.[^ws-kitap]
- Bütün üyeler tek bir `target/` dizini paylaşır; gereksiz yeniden derlemeler önlenir.[^ws-kitap]
- Komutlar tek yerden verilir: `-p` ile bir üye, `--workspace` ile hepsi seçilir.

Üyeler birbirini kendiliğinden görmez. Bir üyenin ötekini kullanması için onu yol bağımlılığı olarak bildirmesi gerekir: `cargo-and-crates = { path = "../cargo-and-crates" }`.[^ws-kitap]

## Yayımlamak

crates.io, paketlerin yayımlandığı kayıt sitesidir. Yayımlamadan önce paket adının alınmamış olması ve `Cargo.toml`'da en az bir açıklama ile lisans bulunması gerekir:[^metadata]

```toml
[package]
name = "cargo-and-crates"
version = "0.1.0"
edition = "2024"
description = "Bölüm 14 örneği: doc yorumları, doc-test ve pub use ile yeniden dışa aktarma"
license = "MIT OR Apache-2.0"
```

`cargo publish` sürümü yükler. Bu kalıcı bir işlemdir: Yayımlanan bir sürümün üzerine yazılamaz ve kodu silinemez.[^yayim] Yeni bir sürüm için `version` alanını anlamsal sürümlemeye (SemVer) göre artırıp yeniden yayımlarsınız. Yüklemek için crates.io hesabınızdan bir belirteç (token) alıp `cargo login` ile tanıtmanız gerekir; belirteç gizlidir, kimseyle paylaşmayın.[^giris] Sorunlu bir sürümü geri çekmek için `cargo yank --version 1.0.1`, geri çekmeyi iptal etmek için aynı komutun `--undo` seçeneği kullanılır. Bu komut kodu silmez (yanlışlıkla yüklenen bir gizli anahtarı da silmez); yalnızca yeni projelerin o sürümü seçmesini engeller, `Cargo.lock`'unda o sürüm kayıtlı olan projeler çalışmaya devam eder.[^yank] Yayımlanan paketlerin belgelerini docs.rs kendiliğinden üretip yayımlar.[^docs-rs]

## Programları kurmak: `cargo install`

`cargo install`, crates.io'daki bir program crate'ini derleyip bilgisayarınıza kurar. Yalnızca çalıştırılabilir hedefi olan paketler kurulabilir; dosyalar, Rust'ı rustup ile kurduysanız ve ayarı değiştirmediyseniz `~/.cargo/bin` dizinine konur.[^kurulum]

```console
$ cargo install ripgrep
```

Cargo 1.100'den itibaren `cargo install`, paketin içindeki `Cargo.lock`'u varsayılan olarak kullanır; kurulan programın bağımlılık sürümleri yayımlandığı andakilerle aynı olur. Önceden bunun için `--locked` yazmak gerekirdi, artık bu seçenek sonucu değiştirmez.[^kurulum-lock]

Kurulan programlar Cargo'yu genişletebilir: `PATH` üzerinde `cargo-bir-sey` adında bir program varsa `cargo bir-sey` komutuyla çalıştırılır ve `cargo --list` çıktısında görünür.[^genisletme]

## Hata kodu başvurusu

| Kod | Ne zaman görülür? | Ne yapmalı? |
| --- | --- | --- |
| [E0432](https://doc.rust-lang.org/error_codes/E0432.html) | `use` ile istenen ad o yolda bulunmadığında | Adın tam yolunu yazın; kütüphane sizinse köke `pub use` ekleyin. |
| [E0603](https://doc.rust-lang.org/error_codes/E0603.html) | Özel (`pub` olmayan) bir modüle ya da öğeye dışarıdan erişildiğinde | Dışa aktarılan kısa yolu kullanın; kütüphane sizinse öğeyi `pub` yapın. |

## Perde arkası: Derleyici ne yapıyor?

> [!NOTE]
> Bu başlık ilk okuyuşta atlanabilir.

### Belgeleme yorumu bir özniteliktir

`/// Metin` yazımı, öğeye `#[doc = " Metin"]` özniteliği koymakla aynıdır; `//! Metin` de içinde bulunduğu öğeye `#![doc = " Metin"]` koyar.[^doc-nitelik] Aralarındaki fark bu yüzden yalnızca biçimsel değildir: Biri sonraki öğeye, öteki kapsayan öğeye bağlanır. Kendisinden sonra belgeleyeceği bir öğe olmayan `///` derleme hatasıdır.

### Belge testi nasıl birleştirilir?

Rustdoc kod bloklarını belgeleme yorumlarından toplar ve her biri için `add_test` adımında karar verir: Birleştirilebilen örnekler, aynı edition'ı ve aynı crate özniteliklerini paylaşanlarla tek ikili dosyada toplanır; ötekiler tek başına birer test olur.[^doctest-kod] Testler `cargo test --doc -- --show-output` ile çalıştırıldığında birleştirme yapılmaz. Yukarıda düşen testin panik mesajındaki `doctest_bundle_2024.rs` adı, örneğin birleştirilmiş dosyada derlendiğini gösterir.

### `pub use` ve isim çözümleme

`use` bildirimlerini `rustc_resolve` çözer: İçe aktarmalar yinelemeli olarak eşlenir, sonunda çözülemeyenler için hata üretilir.[^import-kod] Reference beş ad alanı sayarken derleyicinin `Namespace` enum'ında üç varyant vardır: `TypeNS`, `ValueNS` ve `MacroNS`.[^namespace-kod] Ömürler ile etiketler ayrı kapsam yığınlarında, `lifetime_ribs` ile `label_ribs` alanlarında tutulur.[^rib-kod] "Üç ad alanı vardır" ifadesine rastlarsanız nedeni bu uygulama ayrıntısıdır; dilin tanımında sayı beştir.

### Profil derleyiciye nasıl ulaşır?

Profil ayarları rustc'ye komut satırı seçenekleri olarak geçer: `opt-level`, `-C opt-level` bayrağını belirler.[^opt-level] Cargo, `-v` ile çalıştırıldığında verdiği komutları gösterir.

## Özet

- Paket `Cargo.toml` ile tanımlanan kutu, crate derlenen birimdir. `Cargo.toml` istekleri, `Cargo.lock` kesin sürümleri tutar.
- `///` sonraki öğeyi, `//!` içinde bulunduğu öğeyi belgeler. Kılavuzdaki örnekler `cargo test` ile çalışır; 2024 edition'ında uyumlu örnekler birlikte derlenir, ayrı süreçlerde koşar.
- `pub use` iç modülleri gizlemez, ek ve kısa bir yol açar. Rust'ta beş ad alanı vardır.
- `dev` hızlı derler, `release` hızlı çalışır. `opt-level` 0 ile 3 arasında bir sayı, `"s"` ya da `"z"` olabilir. Profil ayarları çalışma alanının köküne yazılır.
- Çalışma alanı tek `Cargo.lock` ve tek `target/` paylaştırır. Yayımlanan sürüm kalıcıdır; `cargo yank` kodu silmez.

Sıradaki bölümde veriye sahip olan ve işaretçi gibi davranan türlere bakacağız: [14. Akıllı İşaretçiler](14-smart-pointers.md).

## Kaynaklar

[^cargo-daha]: The Rust Programming Language, [14. More About Cargo and Crates.io](https://doc.rust-lang.org/book/ch14-00-more-about-cargo.html).
[^paket]: The Rust Programming Language, [7.1 Packages and Crates](https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html).
[^hedef-adi]: The Cargo Book, [Cargo Targets, The `name` field](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#the-name-field).
[^lock]: The Cargo Book, [Cargo.toml vs Cargo.lock](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html).
[^unused-dep]: The Cargo Book 1.100.0, [`unused_dependencies`](https://doc.rust-lang.org/1.100.0/cargo/reference/lints.html#unused_dependencies).
[^publish-age]: The Cargo Book 1.100.0, [Publish age](https://doc.rust-lang.org/1.100.0/cargo/reference/resolver.html#publish-age).
[^ayirt-edici]: The Rust Reference, [Implicit discriminants](https://doc.rust-lang.org/reference/items/enumerations.html#implicit-discriminants).
[^belge-yorumu]: The Rust Programming Language, [14.2, Making Useful Documentation Comments](https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#making-useful-documentation-comments).
[^ic-yorum]: The Rust Programming Language, [14.2, Contained Item Comments](https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#contained-item-comments).
[^bolumler]: The Rust Programming Language, [14.2, Commonly Used Sections](https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#commonly-used-sections).
[^belge-testi]: The Rust Programming Language, [14.2, Documentation Comments as Tests](https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#documentation-comments-as-tests).
[^cargo-test]: The Cargo Book, [cargo test, Documentation tests](https://doc.rust-lang.org/cargo/commands/cargo-test.html#documentation-tests).
[^gecer]: The rustdoc book, [Passing or failing a doctest](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html#passing-or-failing-a-doctest).
[^hazirlik]: The rustdoc book, [Pre-processing examples](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html#pre-processing-examples).
[^gizli]: The rustdoc book, [Hiding portions of the example](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html#hiding-portions-of-the-example).
[^nitelik]: The rustdoc book, [Attributes](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html#attributes).
[^edition-2024]: The Rust Edition Guide, [Rustdoc combined tests](https://doc.rust-lang.org/edition-guide/rust-2024/rustdoc-doctests.html#rustdoc-combined-tests).
[^yeniden]: The Rust Programming Language, [14.2, Exporting a Convenient Public API](https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#exporting-a-convenient-public-api).
[^use-ad-alani]: The Rust Reference, [Namespaces, Use declarations](https://doc.rust-lang.org/reference/names/namespaces.html#use-declarations).
[^ad-alani]: The Rust Reference, [Namespaces](https://doc.rust-lang.org/reference/names/namespaces.html#namespaces).
[^profil-kitap]: The Rust Programming Language, [14.1 Customizing Builds with Release Profiles](https://doc.rust-lang.org/book/ch14-01-release-profiles.html#customizing-builds-with-release-profiles).
[^profil-dev]: The Cargo Book, [Profiles, dev](https://doc.rust-lang.org/cargo/reference/profiles.html#dev).
[^profil-release]: The Cargo Book, [Profiles, release](https://doc.rust-lang.org/cargo/reference/profiles.html#release).
[^opt-level]: The Cargo Book, [Profiles, opt-level](https://doc.rust-lang.org/cargo/reference/profiles.html#opt-level).
[^profil-kok]: The Cargo Book, [Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html#profiles).
[^profil-debug]: The Cargo Book, [Profiles, debug](https://doc.rust-lang.org/cargo/reference/profiles.html#debug-1) ve [Custom profiles](https://doc.rust-lang.org/cargo/reference/profiles.html#custom-profiles).
[^ws]: The Cargo Book, [Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html#workspaces) ve [The members and exclude fields](https://doc.rust-lang.org/cargo/reference/workspaces.html#the-members-and-exclude-fields).
[^resolver]: The Cargo Book, [Resolver versions](https://doc.rust-lang.org/cargo/reference/resolver.html#resolver-versions).
[^ws-kitap]: The Rust Programming Language, [14.3 Cargo Workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html).
[^metadata]: The Rust Programming Language, [14.2, Adding Metadata to a New Crate](https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#adding-metadata-to-a-new-crate).
[^yayim]: The Cargo Book, [Publishing on crates.io](https://doc.rust-lang.org/cargo/reference/publishing.html#publishing-on-cratesio).
[^giris]: The Cargo Book, [Before your first publish](https://doc.rust-lang.org/cargo/reference/publishing.html#before-your-first-publish).
[^yank]: The Cargo Book, [cargo yank](https://doc.rust-lang.org/cargo/commands/cargo-yank.html#description).
[^docs-rs]: docs.rs, [Builds](https://docs.rs/about/builds).
[^kurulum]: The Rust Programming Language, [14.4 Installing Binaries with `cargo install`](https://doc.rust-lang.org/book/ch14-04-installing-binaries.html#installing-binaries-with-cargo-install).
[^kurulum-lock]: The Cargo Book 1.100.0, [cargo install, Dealing with the Lockfile](https://doc.rust-lang.org/1.100.0/cargo/commands/cargo-install.html#dealing-with-the-lockfile).
[^genisletme]: The Rust Programming Language, [14.5 Extending Cargo with Custom Commands](https://doc.rust-lang.org/book/ch14-05-extending-cargo.html#extending-cargo-with-custom-commands).
[^doc-nitelik]: The Rust Reference, [Doc comments](https://doc.rust-lang.org/reference/comments.html#doc-comments).
[^doctest-kod]: rustdoc 1.99.0 kaynak kodu, [`doctest.rs`, `add_test`](https://github.com/rust-lang/rust/blob/1.99.0/src/librustdoc/doctest.rs#L1034).
[^import-kod]: rustc 1.99.0 kaynak kodu, [`imports.rs`, `resolve_imports`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_resolve/src/imports.rs#L775) ve [`finalize_import`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_resolve/src/imports.rs#L1193).
[^namespace-kod]: rustc 1.99.0 kaynak kodu, [`def.rs`, `Namespace`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir/src/def.rs#L644).
[^rib-kod]: rustc 1.99.0 kaynak kodu, [`late.rs`, `label_ribs`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_resolve/src/late.rs#L808).
