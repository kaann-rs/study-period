# 10. Test Yazma

Rust'ın tür sistemi pek çok hatayı derleme zamanında yakalar, ama bir fonksiyonun *istediğiniz* işi yapıp yapmadığını bilemez. `topla` adını verdiğiniz bir fonksiyon iki sayıyı çıkarıyorsa derleyici buna itiraz etmez; türler doğrudur, yanlış olan anlamdır. Bu boşluğu testler doldurur.

*Test*, "Şunu verirsem şunu almalıyım" diyen küçük bir denemedir. Bir kez yazarsınız, bilgisayar her seferinde sizin yerinize dener. Elle denemek unutulur; yazılı bir test ise her değişiklikten sonra, eskiden çalışan bir şeyi bozup bozmadığınızı saniyeler içinde söyler.

Rust'ta bunun için ayrı bir araç takımı (framework) kurmanız gerekmez: Test yazmak dile, testleri çalıştırmak Cargo'ya yerleşiktir.[^genel] Bu bölümde testlerin nasıl yazıldığını, nasıl çalıştırıldığını ve bir projede nasıl düzenlendiğini göreceğiz.

> [!NOTE]
> Bu bölümün kodu `learn/writing-tests` dizinindedir. Depo kökünden `cargo test -p writing-tests` komutuyla testleri, `cargo run -p writing-tests` komutuyla küçük tanıtım programını çalıştırabilirsiniz.

## Günlük hayattan bir benzetme

Bir oyuncak atölyesi düşünün. Her oyuncak kutuya girmeden önce denenir. Deneme tezgâhı atölyenin içindedir ve müşteriye giden kutuya konmaz. Rust'ta *birim testleri* (unit test) bu tezgâhtır: Asıl kodla aynı dosyada durur, ama son programın içinde yer almaz. Tezgâhta çalışan kalite kontrolcü, müşterinin göremediği aletleri de eline alıp deneyebilir.

Kalite kontrolcünün elinde bir beklentiler listesi vardır: "Kutuya 2 ve 3 koyarsan 5 çıkmalı." Beklentilerden biri tutmazsa oyuncak geri döner ve listede hangi satırın tutmadığı yazar. Listeyi bir kez hazırlarsınız; sonra her sabah birisi gelip hepsini sizin yerinize denetler.

Bazı denemelerde beklenen şey, oyuncağın çalışmamasıdır. Pili ters takarsınız; oyuncak çalışırsa asıl o zaman bir sorun vardır.

Uzun süren denemeler "bugün yapma" diye kenara ayrılabilir. Silinmezler, istendiğinde yine yapılırlar.

Bir de müşteri vardır. İçeride her parça sağlam olabilir, ama kutudan çıkan bütün çalışmayabilir. Oyuncağı kutusundan çıkarıp yalnızca dışarıdan görünen düğmelerine basarak deneyen müşteri, *entegrasyon testidir* (integration test). Atölyenin vitrini ise `main.rs`'tir: Oyuncakları sergiler, üretim orada yapılmaz.

<details>
<summary>Bu bölümde geçen terimler</summary>

- **test:** Kodun beklendiği gibi çalıştığını sınayan küçük fonksiyon.
- **öznitelik (attribute):** Bir öğenin üstüne yazılan `#[...]` etiketi; derleyiciye not düşer.
- **`#[test]`:** "Bu fonksiyon bir testtir" etiketi.
- **`#[cfg(test)]`:** "Bunu yalnızca testler derlenirken derle" etiketi.
- **`assert!`, `assert_eq!`, `assert_ne!`:** "Bu doğru olmalı", "bu ikisi eşit olmalı", "bu ikisi eşit olmamalı" diyen makrolar.
- **geçmek ve düşmek:** Testin başarılı ve başarısız olması.
- **`#[should_panic]`:** "Bu test geçmek için paniklemeli" etiketi.
- **`#[ignore]`:** "Bu testi şimdilik atla" etiketi.
- **birim testi (unit test):** Küçük bir parçayı, kodun içinden sınayan test.
- **entegrasyon testi (integration test):** Parçaları birlikte, dışarıdaki bir kullanıcı gibi sınayan test.
- **belge testi (doc test):** Belge yorumlarındaki kod örneklerinin test olarak çalıştırılması. [13. bölümde](13-cargo-and-crates.md) anlatılır.
- **`pub` ve özel (private):** Dışarıya açılmış öğe; yalnızca kendi modül ağacından görünen öğe.
- **genel arayüz (public API):** Bir crate'in dışarıya açtığı fonksiyon ve türlerin tamamı.
- **crate:** Derleyicinin tek seferde derlediği birim: bir kütüphane ya da bir program.
- **kütüphane (lib) ve ikili (binary):** Başka kodların çağırdığı crate; doğrudan çalıştırılan program.
- **modül (`mod`):** Kodu gruplayan isimli bölme. `super` bir üst modülü gösterir.
- **araç takımı (framework):** Bir iş için ayrıca kurulan hazır kod bütünü.
- **test koşucusu (test harness, libtest):** Testleri çalıştıran ve sonuçları sayan program.
- **paralel:** Aynı anda, yan yana çalışma.
- **standart çıktı (stdout):** Programın olağan çıktısını yazdığı akış.
- **`dev-dependencies`:** Yalnızca testlerde, örneklerde ve benchmark'larda kullanılan bağımlılıklar.
- **Cargo:** Rust'ın derleme, çalıştırma ve test komutlarını sağlayan aracı.

</details>

## İlk test

Üzerinde `#[test]` bulunan her fonksiyon bir testtir. Test fonksiyonu parametre almaz; beklentisini bir `assert` makrosuyla yazar:[^ilk-test]

```rust,noplayground,test_harness
pub fn ikiye_katla(x: i32) -> i32 {
    x * 2
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn dort_sekiz_olur() {
        assert_eq!(ikiye_katla(4), 8);
    }
}
```

`mod testler` ve `#[cfg(test)]` satırlarına birazdan döneceğiz. Testi `cargo test` ile çalıştırırsınız; raporun bizi ilgilendiren bölümü şudur:

```console
$ cargo test
running 1 test
test testler::dort_sekiz_olur ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Testin adı, modül yoluyla birlikte yazılır. Son satır özeti verir: geçenler, düşenler, atlananlar, ölçülenler ve süzülenler.

Şimdi beklentiyi bilerek yanlış yazalım:

```rust,noplayground,should_panic,test_harness
pub fn ikiye_katla(x: i32) -> i32 {
    x * 2
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn dort_sekiz_olur() {
        assert_eq!(ikiye_katla(4), 9);
    }
}
```

Bu kez test düşer:

```console
$ cargo test
running 1 test
test testler::dort_sekiz_olur ... FAILED

failures:

---- testler::dort_sekiz_olur stdout ----

thread 'testler::dort_sekiz_olur' (902371) panicked at src/lib.rs:11:9:
assertion `left == right` failed
  left: 8
 right: 9
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    testler::dort_sekiz_olur

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `--lib`
```

Rapor, düşen testi, hangi satırda düştüğünü ve karşılaştırılan iki değeri birlikte gösterir. Neyin yanlış çıktığını aramanız gerekmez. Bir test panik ederek düşer: `assert_eq!`, koşul tutmadığında `panic!` çağırır.[^assert-panik] Düşen bir test ötekileri durdurmaz; koşucu kalan testleri çalıştırmaya devam eder, sonunda `cargo test` sıfırdan farklı bir kodla çıkar.

## Beklentiyi yazmak: `assert!` ailesi

Beklentilerin çoğu üç makroyla yazılır:[^assert]

- `assert!(koşul)` koşulun doğru olmasını ister.
- `assert_eq!(a, b)` iki değerin eşit olmasını ister.
- `assert_ne!(a, b)` iki değerin eşit olmamasını ister.

Bölümün kalanında şu fonksiyonları sınayacağız:

```rust
pub fn topla(a: i32, b: i32) -> i32 {
    a + b
}

/// Yaşı doğrular; negatifse panikler.
pub fn yas_dogrula(yas: i32) -> i32 {
    if yas < 0 {
        panic!("yaş negatif olamaz: {yas}");
    }
    yas
}

/// Metindeki kelime sayısını verir.
pub fn kelime_say(s: &str) -> usize {
    kirp(s).split_whitespace().count()
}

// Özel yardımcı: dışarıdan görünmez
fn kirp(s: &str) -> &str {
    s.trim()
}
```

İlk testler üç makroyu birlikte kullanır:

```rust,noplayground,test_harness
# pub fn topla(a: i32, b: i32) -> i32 { a + b }
# pub fn kelime_say(s: &str) -> usize { s.trim().split_whitespace().count() }
#
#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn topla_calisir() {
        assert_eq!(topla(2, 3), 5);
        assert_ne!(topla(2, 2), 5);
    }

    #[test]
    fn kelime_sayimi() {
        assert_eq!(kelime_say("  bir  iki üç "), 3);
    }

    #[test]
    fn mantik_dogru() {
        let sonuc = topla(10, -4);
        assert!(sonuc > 0, "beklenen pozitif, gelen {sonuc}");
    }
}
```

`assert_eq!` ve `assert_ne!` tutmadığında iki değeri de yazdığı için değerlerin `PartialEq` ve `Debug` uygulaması gerekir. Kendi türlerinizde çoğu zaman `#[derive(PartialEq, Debug)]` yeterlidir.

`assert!` yalnızca koşulun tutmadığını bilir, değerleri göstermez. Son testteki gibi zorunlu argümanlardan sonra `format!` biçiminde bir mesaj eklerseniz test düştüğünde o mesaj yazılır. Sonuç `topla(-10, 4)` olsaydı rapor şöyle olurdu:

```text
---- testler::mantik_dogru stdout ----

thread 'testler::mantik_dogru' (924890) panicked at src/lib.rs:12:9:
beklenen pozitif, gelen -6
```

> [!NOTE]
> Bir değerin belirli bir desene uyduğunu sınamak için `assert!(matches!(deger, Desen))` yazabilirsiniz. 1.96.0 sürümüyle gelen `assert_matches!` makrosu aynı işi yapar ve uymayan değeri de gösterir; kullanmadan önce `use std::assert_matches;` gerekir.[^assert-matches]

## Birim testleri ve `#[cfg(test)]`

Birim testleri, sınadıkları kodla aynı dosyada, üzerinde `#[cfg(test)]` bulunan bir modülde durur. Bu öznitelik "modülü yalnızca testler derlenirken derle" demektir.[^cfg-test] `cargo test` modülü derler; `cargo build` ve `cargo run` onu hiç görmez. Test kodu böylece son programa girmez. Yalnızca testlerde kullanılan bağımlılıklar da `Cargo.toml`'da `[dev-dependencies]` altına yazılır ve olağan derlemede yer almaz.[^dev-dep]

`use super::*;` satırı, bir üst modüldeki her şeyi test modülüne alır. "Her şey"e özel (private) öğeler de dâhildir:

```rust,noplayground,test_harness
fn kirp(s: &str) -> &str {
    s.trim()
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn private_fonksiyon_test_edilebilir() {
        assert_eq!(kirp("  selam  "), "selam");
    }
}
```

`kirp` `pub` değildir, yine de test edilebilir. Bunun nedeni testlere tanınmış bir ayrıcalık değil, [6. bölümde](../basics/06-modules.md) gördüğümüz görünürlük kuralıdır: Bir öğe, tanımlandığı modülde ve o modülün alt modüllerinde görünür.[^private] `testler` modülü, `kirp`'in bulunduğu modülün çocuğudur. Görünürlük crate'e değil modül ağacına bağlı olduğundan, aynı crate'teki başka bir dalda duran bir modül `kirp`'e erişemezdi.

## Panik bekleyen testler

Bir fonksiyonun yanlış girdide paniklemesini istiyorsanız bunu da sınamanız gerekir. `#[should_panic]` testin mantığını tersine çevirir: Gövde paniklerse test geçer, paniklemezse düşer.[^should-panic]

```rust,noplayground,test_harness
# pub fn yas_dogrula(yas: i32) -> i32 {
#     if yas < 0 {
#         panic!("yaş negatif olamaz: {yas}");
#     }
#     yas
# }
#
#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    #[should_panic(expected = "negatif olamaz")]
    fn negatif_yas_panikler() {
        yas_dogrula(-1);
    }
}
```

`expected`, panik iletisinin verilen metni içermesini de ister. Onsuz, fonksiyon bambaşka bir nedenle paniklese de test geçerdi. Aşağıdaki iki test düşer: Biri hiç paniklemez, ötekinin iletisi beklenene uymaz.

```rust,noplayground,should_panic,test_harness
# pub fn yas_dogrula(yas: i32) -> i32 {
#     if yas < 0 {
#         panic!("yaş negatif olamaz: {yas}");
#     }
#     yas
# }
#
# #[cfg(test)]
# mod testler {
#     use super::*;
#
#[test]
#[should_panic]
fn arti_yas() {
    yas_dogrula(30);
}

#[test]
#[should_panic(expected = "çok büyük")]
fn yanlis_mesaj() {
    yas_dogrula(-1);
}
# }
```

Koşucu iki nedeni ayrı ayrı bildirir:

```text
---- testler::yanlis_mesaj stdout ----

thread 'testler::yanlis_mesaj' (924774) panicked at src/lib.rs:3:9:
yaş negatif olamaz: -1
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
note: panic did not contain expected string
      panic message: "yaş negatif olamaz: -1"
 expected substring: "çok büyük"
---- testler::arti_yas stdout ----
note: test did not panic as expected at src/lib.rs:14:8
```

## `Result` döndüren testler

Test fonksiyonu `Result<(), E>` de döndürebilir. `Ok(())` testin geçtiği, `Err(..)` düştüğü anlamına gelir.[^result-test] Yararı, testin içinde `?` kullanabilmektir:

```rust,noplayground,test_harness
# pub fn topla(a: i32, b: i32) -> i32 { a + b }
#
#[cfg(test)]
mod testler {
    use super::*;
    use std::num::ParseIntError;

    #[test]
    fn result_donen_test() -> Result<(), String> {
        if topla(2, 2) == 4 {
            Ok(())
        } else {
            Err(String::from("2+2 != 4 ?!"))
        }
    }

    #[test]
    fn metinden_gelen_sayilar_toplanir() -> Result<(), ParseIntError> {
        let a: i32 = "40".parse()?;
        let b: i32 = "2".parse()?;
        assert_eq!(topla(a, b), 42);
        Ok(())
    }
}
```

`Err` dönen testin raporunda hata `Debug` biçimiyle görünür; örneğin `Error: "2+2 != 4 ?!"`. Bu tür testlerde `#[should_panic]` kullanılamaz: O öznitelik, test fonksiyonunun `()` döndürmesini ister. Bir işlemin `Err` verdiğini sınamak istiyorsanız `?` değil, `assert!(deger.is_err())` yazın.

## Testleri atlamak: `#[ignore]`

Uzun süren ya da özel bir ortam isteyen testleri her koşuda çalıştırmak istemeyebilirsiniz. `#[ignore]` böyle bir testi silmeden kenara ayırır:[^ignore]

```rust,noplayground,test_harness
# pub fn topla(a: i32, b: i32) -> i32 { a + b }
#
# #[cfg(test)]
# mod testler {
#     use super::*;
#
#[test]
#[ignore = "yavaş: büyük girdi"]
fn pahali_test() {
    assert_eq!(topla(1000, 1000), 2000);
}
# }
```

Eşittir işaretinden sonraki açıklama isteğe bağlıdır ve raporda görünür. Atlanan test yine de derlenir, yani koda uyumunu yitirirse bunu öğrenirsiniz. Çalıştırmak için koşucuya açıkça söylemeniz gerekir:

```console
$ cargo test -- --ignored          # yalnızca atlananlar
$ cargo test -- --include-ignored  # hepsi
```

## Testleri çalıştırmak

`cargo test` iki ayrı programın ayarlarını taşır. `--` işaretinden öncekiler Cargo'ya, sonrakiler test koşucusuna gider.[^cargo-test]

### Ada göre seçmek

Verdiğiniz metni adında içeren testler çalışır.[^filtre] Raporun birim testleriyle ilgili bölümü şöyledir:

```console
$ cargo test topla
running 1 test
test testler::topla_calisir ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s
```

Eşleşme modül yolunu da kapsar; `cargo test testler::` o modüldeki bütün testleri seçer. Birden çok süzgeç vermek için onları `--`'den sonra yazın: `cargo test -- topla kelime`. Adın tam olarak eşleşmesini isterseniz `--exact`, bazı testleri dışarıda bırakmak isterseniz `--skip` ekleyin.

### Paralel çalışma

Koşucu testleri varsayılan olarak birden çok thread'de yan yana çalıştırır.[^paralel] Testlerin birbirinden bağımsız olması bu yüzden gereklidir. Aynı dosyaya yazan, aynı ortam değişkenini değiştiren ya da aynı global durumu kullanan iki test, çalışma sırasına göre bazen geçer bazen düşer. En iyi çözüm paylaşımı kaldırmak, örneğin her teste kendi geçici dosyasını vermektir. Bunu yapamıyorsanız testleri sırayla çalıştırabilirsiniz:

```console
$ cargo test -- --test-threads=1
```

### Testlerin yazdıkları

Geçen bir testin standart çıktıya yazdıkları gizlenir, düşenin yazdıkları raporda gösterilir.[^gosterme] Geçenlerin yazdıklarını da görmek isterseniz:

```console
$ cargo test -- --show-output
```

Hiçbir şeyin yakalanmamasını, yazılanların anında ekrana gelmesini isterseniz `--no-capture` kullanın. Eski yazımı olan `--nocapture` 1.88.0'dan beri önerilmez.[^no-capture]

## Entegrasyon testleri

### `tests/` dizini

Birim testleri atölyenin içindendir. Crate'inizi dışarıdaki bir kullanıcı gibi sınamak için `src`'nin yanına bir `tests` dizini açar, dosyaları oraya koyarsınız:[^entegrasyon]

```text
writing-tests
├── Cargo.toml
├── src
│   ├── lib.rs
│   └── main.rs
└── tests
    └── entegrasyon.rs
```

`tests/entegrasyon.rs` dosyasının tamamı şudur:

```rust,noplayground,test_harness
# mod writing_tests {
#     pub fn topla(a: i32, b: i32) -> i32 { a + b }
#     pub fn kelime_say(s: &str) -> usize { s.trim().split_whitespace().count() }
# }
use writing_tests::{kelime_say, topla};

#[test]
fn dışarıdan_topla() {
    assert_eq!(topla(40, 2), 42);
}

#[test]
fn dışarıdan_kelime_say() {
    assert_eq!(kelime_say("merhaba dünya"), 2);
}
```

Bu dosyada ne `#[cfg(test)]` ne de `mod testler` vardır. Cargo `tests` dizinini özel olarak tanır; içindeki dosyaları yalnızca test derlemesinde derler. Dizindeki her dosya ayrı bir crate, yani ayrı bir test programıdır. Kütüphaneye, başka bir projenin yapacağı gibi `use writing_tests::...` ile erişir.

`cargo test` bütün bölümleri sırayla çalıştırır:

```console
$ cargo test
     Running unittests src/lib.rs (target/debug/deps/writing_tests-de32faa88f57f4bc)

running 7 tests
test testler::pahali_test ... ignored
test testler::kelime_sayimi ... ok
test testler::mantik_dogru ... ok
test testler::private_fonksiyon_test_edilebilir ... ok
test testler::negatif_yas_panikler - should panic ... ok
test testler::result_donen_test ... ok
test testler::topla_calisir ... ok

test result: ok. 6 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/writing_tests-ea53925743b41cbd)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/entegrasyon.rs (target/debug/deps/entegrasyon-0cb035a3fe12c44b)

running 2 tests
test dışarıdan_topla ... ok
test dışarıdan_kelime_say ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests writing_tests

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Dört bölüm görüyorsunuz: kütüphanenin birim testleri, ikili crate'in birim testleri (hiç yok), entegrasyon testleri ve belge testleri. Testler paralel çalıştığı için satırların sırası değişebilir. Bölümlerden birinde test düşerse sonrakiler çalıştırılmaz; hepsinin çalışmasını isterseniz `--no-fail-fast` ekleyin. Tek bir entegrasyon dosyasını çalıştırmak için `cargo test --test entegrasyon` yazarsınız.

### Özel öğeler dışarıdan görünmez

Entegrasyon testi yalnızca genel arayüzü görür. `kirp`'e erişmeye çalışırsa derlenmez:

```rust,compile_fail,E0603
# mod writing_tests {
#     fn kirp(s: &str) -> &str { s.trim() }
# }
use writing_tests::kirp;

#[test]
fn dışarıdan_kirp() {
    assert_eq!(kirp("  selam  "), "selam");
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0603]: function `kirp` is private
  |
4 | use writing_tests::kirp;
  |                    ^^^^ private function
```

Bu bir eksiklik değil, entegrasyon testinin amacıdır. İç ayrıntıyı bilmeyen bir test, o ayrıntı değiştiğinde bozulmaz; yalnızca dışarıdan görünen davranış bozulursa düşer. İç mantığı sınamak istiyorsanız birim testi yazın.

### Neden bir kütüphane hedefi gerekir?

Yalnızca `src/main.rs`'ten oluşan bir projede `tests` dizinindeki dosyalar `main.rs`'teki fonksiyonlara ulaşamaz. İkili crate, başka crate'lerin kullanacağı bir arayüz sunmaz.[^ikili] Deneseydiniz şunu görürdünüz:

```text
error[E0432]: unresolved import `ikili`
 --> tests/e.rs:1:5
  |
1 | use ikili::topla;
  |     ^^^^^ use of unresolved module or unlinked crate `ikili`
```

Bu yüzden yerleşik düzen, mantığı `lib.rs`'e koyup `main.rs`'i ince tutmaktır. Bu bölümün `main.rs` dosyası yalnızca kütüphanedeki fonksiyonları çağırır:

```rust
# mod writing_tests {
#     pub fn topla(a: i32, b: i32) -> i32 { a + b }
#     pub fn yas_dogrula(yas: i32) -> i32 { yas }
#     pub fn kelime_say(s: &str) -> usize { s.trim().split_whitespace().count() }
# }
use writing_tests::{kelime_say, topla, yas_dogrula};

fn main() {
    println!("topla(2, 3) = {}", topla(2, 3));
    println!("kelime_say(\"  bir iki üç \") = {}", kelime_say("  bir iki üç "));
    println!("yas_dogrula(30) = {}", yas_dogrula(30));
}
```

Çıktı:

```text
topla(2, 3) = 5
kelime_say("  bir iki üç ") = 3
yas_dogrula(30) = 30
```

Bu kadar ince bir `main`'de sınanacak mantık kalmaz; kütüphane doğruysa program da büyük olasılıkla doğrudur. Aynı düzeni bir sonraki bölümde `minigrep`'te kullanacağız.

> [!NOTE]
> `tests` dizinindeki her dosya ayrı bir crate olduğu için ortak yardımcı kodu `tests/common.rs` dosyasına koyarsanız o da boş bir test programı olarak derlenir. Alt dizinlerdeki dosyalar ise ayrı crate sayılmaz; bu yüzden ortak kod `tests/common/mod.rs` dosyasına yazılır ve test dosyalarından `mod common;` ile alınır.[^ortak]

## Hata kodu başvurusu

| Kod | Ne zaman görülür? | Ne yapmalı? |
| --- | --- | --- |
| `E0603` | Entegrasyon testi özel bir öğeye erişmeye çalıştığında | Genel arayüzden sınayın; iç mantık için birim testi yazın. |
| `E0432` | Kütüphane hedefi olmayan bir projede `tests` dosyası crate adını içe aktardığında | Mantığı `src/lib.rs`'e taşıyın, `main.rs`'i ince tutun. |

Derleyicinin yakalamadığı, sık rastlanan üç durum daha vardır: Paylaşılan bir kaynağa dokunan testler ara sıra düşer; `#[ignore]` ile kenara ayrılan test, `--ignored` ya da `--include-ignored` verilmedikçe hiç çalışmaz; `expected` verilmemiş bir `#[should_panic]` testi, beklenmeyen bir panikle de geçer.

## Perde arkası: Derleyici ne yapıyor?

> [!NOTE]
> Bu başlık ilk okuyuşta atlanabilir. Burada `cargo test`'in kodu nasıl derlediğini, testlerin koşucuya nasıl verildiğini ve koşucunun onları nasıl çalıştırdığını görüyoruz.

**Test derlemesi ayrı bir derlemedir.** `cargo test`, `rustc`'yi `--test` bayrağıyla çağırır. Bayrak `cfg(test)`'i açar[^cfg-ref] ve giriş noktası olarak test koşucusunu koyar. Olağan derlemede `cfg(test)` kapalıdır; testler normal programa gömülseydi boyut ve bağımlılıklar gereksiz yere büyürdü. Entegrasyon testleri derlenirken kütüphanenin kendisi olağan biçimde, `cfg(test)` kapalıyken derlenir; birim testi modülleri orada bulunmaz.

**`#[test]` bir makro gibi açılır.** Derleyici test yapılandırmasında değilse `#[test]` işaretli fonksiyonu tümüyle siler; test yapılandırmasındaysa fonksiyonun yanına onu tanıtan bir `TestDescAndFn` sabiti üretir.[^expand] Öyleyse `#[cfg(test)]` ne işe yarar? `#[test]` yalnızca işaretlediği fonksiyonu kaldırır. Modüldeki `use super::*;` satırı, yardımcı fonksiyonlar ve `dev-dependencies`'ten gelen içe aktarmalar ancak modülün tamamı `#[cfg(test)]` ile kapatıldığında olağan derlemenin dışında kalır.

**Testleri koşucu bulmaz, derleyici listeler.** Üretilen sabitler, yine derleyicinin yazdığı bir `main` fonksiyonunda bir dizi hâlinde `test::test_main_static`'e verilir.[^mk-main] Bu fonksiyon, Rust ile birlikte gelen `libtest` kütüphanesindedir.[^test-main] Ad süzgeci de orada, adın verilen metni içerip içermediğine bakarak uygulanır.[^filter-kod]

**Her test ayrı bir thread'de çalışır.** Koşucu aynı anda çalışacak test sayısını sistemin sunduğu paralelliğe, yani çoğunlukla işlemci sayısına göre belirler[^concurrency] ve her testi, testin adını taşıyan bir thread'de başlatır.[^run-test] Panik çıktısında thread adının test adı olması bundandır; parantez içindeki sayı thread'in kimliğidir. Test gövdesi `catch_unwind` ile sarılmıştır.[^catch] Panik yakalanır, testin düştüğü kaydedilir ve öteki testler sürer. Testlerin yığın çözülmesine (unwinding) bağlı olması da buradan gelir: `Cargo.toml`'daki `panic = "abort"` ayarı testler için yok sayılır.[^profil] Testin yazdıkları da bu sırada bir tampona yönlendirilir; geçen testin çıktısının görünmemesi bu yüzdendir.

**Birim ve entegrasyon testi ayrımı.** Koşucu açısından ikisi arasında fark yoktur; fark, kodun hangi crate'te derlendiğidir. Birim testleri sınadıkları crate'in içindedir ve özel öğeleri görür. Entegrasyon testleri ayrı crate'lerdir ve yalnızca genel arayüzü görür.

## Özet

- `#[test]` bir fonksiyonu teste çevirir; test panik ederek ya da `Err` döndürerek düşer. `assert!`, `assert_eq!` ve `assert_ne!` beklentiyi yazar.
- Birim testleri kodun yanında, `#[cfg(test)]` modülünde durur, özel öğeleri görür ve son programa girmez.
- `#[should_panic]` paniği bekler, `expected` iletisini de denetler; `#[ignore]` testi kenara ayırır; `Result` döndüren testlerde `?` kullanılabilir.
- Testler varsayılan olarak paralel çalışır, bu yüzden birbirinden bağımsız olmalıdır. `cargo test ad` süzer; `--` sonrasındaki ayarlar koşucuya gider.
- Entegrasyon testleri `tests` dizininde ayrı crate'lerdir ve yalnızca genel arayüzü görür. Bunun için mantık `lib.rs`'te, `main.rs` ince olmalıdır.

Sıradaki bölüm [11. minigrep: Gerçek Bir Program](11-minigrep.md), buraya kadar gördüğümüz parçaları, testler de dâhil, tek bir komut satırı aracında birleştiriyor.

## Kaynaklar

[^genel]: The Rust Programming Language, [11. Writing Automated Tests](https://doc.rust-lang.org/book/ch11-00-testing.html).
[^ilk-test]: The Rust Programming Language, [Structuring Test Functions](https://doc.rust-lang.org/book/ch11-01-writing-tests.html#structuring-test-functions); The Rust Reference, [The `test` attribute](https://doc.rust-lang.org/reference/attributes/testing.html#the-test-attribute).
[^assert-panik]: std, [`assert_eq!`](https://doc.rust-lang.org/std/macro.assert_eq.html); The Rust Programming Language, [Structuring Test Functions](https://doc.rust-lang.org/book/ch11-01-writing-tests.html#structuring-test-functions).
[^assert]: The Rust Programming Language, [Checking Results with `assert!`](https://doc.rust-lang.org/book/ch11-01-writing-tests.html#checking-results-with-assert), [Testing Equality with `assert_eq!` and `assert_ne!`](https://doc.rust-lang.org/book/ch11-01-writing-tests.html#testing-equality-with-assert_eq-and-assert_ne) ve [Adding Custom Failure Messages](https://doc.rust-lang.org/book/ch11-01-writing-tests.html#adding-custom-failure-messages).
[^assert-matches]: std, [`assert_matches!`](https://doc.rust-lang.org/std/macro.assert_matches.html); Rust 1.96.0 sürüm notları, [Stabilized APIs](https://github.com/rust-lang/rust/releases/tag/1.96.0).
[^cfg-test]: The Rust Programming Language, [The `tests` Module and `#[cfg(test)]`](https://doc.rust-lang.org/book/ch11-03-test-organization.html#the-tests-module-and-cfgtest).
[^dev-dep]: The Cargo Book, [Development dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#development-dependencies).
[^private]: The Rust Programming Language, [Private Function Tests](https://doc.rust-lang.org/book/ch11-03-test-organization.html#private-function-tests); The Rust Reference, [Visibility and privacy](https://doc.rust-lang.org/reference/visibility-and-privacy.html).
[^should-panic]: The Rust Programming Language, [Checking for Panics with `should_panic`](https://doc.rust-lang.org/book/ch11-01-writing-tests.html#checking-for-panics-with-should_panic); The Rust Reference, [The `should_panic` attribute](https://doc.rust-lang.org/reference/attributes/testing.html#the-should_panic-attribute).
[^result-test]: The Rust Programming Language, [Using `Result<T, E>` in Tests](https://doc.rust-lang.org/book/ch11-01-writing-tests.html#using-resultt-e-in-tests).
[^ignore]: The Rust Programming Language, [Ignoring Tests Unless Specifically Requested](https://doc.rust-lang.org/book/ch11-02-running-tests.html#ignoring-tests-unless-specifically-requested); The Rust Reference, [The `ignore` attribute](https://doc.rust-lang.org/reference/attributes/testing.html#the-ignore-attribute).
[^cargo-test]: The Cargo Book, [`cargo test`](https://doc.rust-lang.org/cargo/commands/cargo-test.html).
[^filtre]: The Rust Programming Language, [Running a Subset of Tests by Name](https://doc.rust-lang.org/book/ch11-02-running-tests.html#running-a-subset-of-tests-by-name); The rustc book, [Tests](https://doc.rust-lang.org/rustc/tests/index.html).
[^paralel]: The Rust Programming Language, [Running Tests in Parallel or Consecutively](https://doc.rust-lang.org/book/ch11-02-running-tests.html#running-tests-in-parallel-or-consecutively).
[^gosterme]: The Rust Programming Language, [Showing Function Output](https://doc.rust-lang.org/book/ch11-02-running-tests.html#showing-function-output).
[^no-capture]: Rust 1.88.0 sürüm notları, [Libraries](https://github.com/rust-lang/rust/releases/tag/1.88.0); rust-lang/rust, [PR 139224](https://github.com/rust-lang/rust/pull/139224).
[^entegrasyon]: The Rust Programming Language, [Integration Tests](https://doc.rust-lang.org/book/ch11-03-test-organization.html#integration-tests) ve [The `tests` Directory](https://doc.rust-lang.org/book/ch11-03-test-organization.html#the-tests-directory).
[^ikili]: The Rust Programming Language, [Integration Tests for Binary Crates](https://doc.rust-lang.org/book/ch11-03-test-organization.html#integration-tests-for-binary-crates); hata dizini, [`E0432`](https://doc.rust-lang.org/error_codes/E0432.html) ve [`E0603`](https://doc.rust-lang.org/error_codes/E0603.html).
[^ortak]: The Rust Programming Language, [Submodules in Integration Tests](https://doc.rust-lang.org/book/ch11-03-test-organization.html#submodules-in-integration-tests).
[^cfg-ref]: The Rust Reference, [Conditional compilation: `test`](https://doc.rust-lang.org/reference/conditional-compilation.html#test).
[^expand]: rustc 1.99.0 kaynak kodu, [`test.rs`, `fn expand_test_or_bench`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_builtin_macros/src/test.rs#L104) ve [`test.rs`, `test::TestDescAndFn`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_builtin_macros/src/test.rs#L277).
[^mk-main]: rustc 1.99.0 kaynak kodu, [`test_harness.rs`, `fn mk_main`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_builtin_macros/src/test_harness.rs#L290).
[^test-main]: rustc 1.99.0 kaynak kodu, [`lib.rs`, `test_main_static`](https://github.com/rust-lang/rust/blob/1.99.0/library/test/src/lib.rs#L190).
[^filter-kod]: rustc 1.99.0 kaynak kodu, [`lib.rs`, `test_name.contains`](https://github.com/rust-lang/rust/blob/1.99.0/library/test/src/lib.rs#L562).
[^concurrency]: rustc 1.99.0 kaynak kodu, [`concurrency.rs`, `available_parallelism`](https://github.com/rust-lang/rust/blob/1.99.0/library/test/src/helpers/concurrency.rs#L14).
[^run-test]: rustc 1.99.0 kaynak kodu, [`lib.rs`, `thread::Builder::new().name`](https://github.com/rust-lang/rust/blob/1.99.0/library/test/src/lib.rs#L704).
[^catch]: rustc 1.99.0 kaynak kodu, [`lib.rs`, `catch_unwind`](https://github.com/rust-lang/rust/blob/1.99.0/library/test/src/lib.rs#L756) ve [`lib.rs`, `set_output_capture`](https://github.com/rust-lang/rust/blob/1.99.0/library/test/src/lib.rs#L752).
[^profil]: The Cargo Book, [Profiles: `panic`](https://doc.rust-lang.org/cargo/reference/profiles.html#panic).
