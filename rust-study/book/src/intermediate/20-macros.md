# 20. Makrolar

`println!` ile `vec!`'i kitabın başından beri kullanıyoruz; `#[derive(Debug)]` yazınca derleyicinin bizim yerimize kod ürettiğini de gördük. Bunların hepsi makrodur: kod yazan kod. Makro, çağrıldığı yere derleme sırasında başka kod yerleştirir. Aynı kalıptaki kodu elle tekrar tekrar yazmak yerine derleyiciye yazdırmaya yarar.

[19. bölümde](19-advanced-features.md) `macro_rules!` ile yazılan bildirimsel makroları görmüştük. Bu bölümün konusu ikinci aile, yani *yordamsal makrolardır* (procedural macro). Yordamsal makro, derleyicinin sizin kodunuzu derlerken çağırdığı sıradan bir Rust fonksiyonudur: Kod parçaları alır, yerine yeni kod parçaları verir.

Küçük bir makroyu baştan sona yazacak, makronun gördüğü kod parçalarına bakacak, yordamsal makronun üç türünü tanıyacak ve makroyu hazırlayan crate ile kullanan crate'in neden ayrı olduğunu inceleyeceğiz.

> [!NOTE]
> Bu bölümün kodu `learn/macros` dizinindedir. Depo kökünden `cargo test -p macros` komutuyla deneyebilirsiniz.

## Günlük hayattan bir benzetme

Bir atölyeye lastik damga yaptırdığınızı düşünün. Atölye damgayı bir kez hazırlar, siz kâğıda istediğiniz kadar basarsınız. Bastığınızda kâğıtta bir yıldız belirirse şaşırmayın: Yıldızı siz çizmediniz, damga çizdi. Makro bu damgadır; çağırdığınız yere, sizin yazmadığınız kodu basar.

Damgayı hazırlayan atölye ile damgayı basan kişi ayrı yerlerdedir. Yordamsal makroda da öyledir: Makroyu tanımlayan crate atölyedir, makroyu kullanan crate kâğıda basan kişidir ve makro, tanımlandığı crate'in içinde kullanılamaz. Damganın izini görmek için atölyeye değil, basıldığı kâğıda bakarsınız.

<details>
<summary>Bu bölümde geçen terimler</summary>

- **makro (macro):** Kod yazan kod; çağrıldığı yere derleme sırasında kod yerleştirir.
- **bildirimsel makro (declarative macro):** `macro_rules!` ile yazılan, desene göre kod şablonu dolduran makro.
- **yordamsal makro (procedural macro, proc-macro):** Sıradan bir Rust fonksiyonu olarak yazılan makro.
- **token:** Kaynak kodun en küçük parçası: bir ad, bir sayı, bir işaret.
- **`TokenStream`:** Art arda dizilmiş token'lar; yordamsal makronun aldığı ve verdiği değer.
- **ayrıştırmak (parse):** Düz yazıyı okuyup token'lara çevirmek.
- **genişletme (expansion):** Derleyicinin makro çağrısının yerine makronun ürettiği kodu koyması.
- **enjekte etmek:** Üretilen kodu, makronun çağrıldığı yere yerleştirmek.
- **derleme zamanı (compile time):** Programın çalıştığı değil, derleyicinin kodu çevirdiği zaman.
- **crate:** Tek başına derlenen Rust birimi; kütüphane ya da program.
- **hijyen (hygiene):** Makronun içindeki adların, çağrıldığı yerdeki adlarla karışmaması.

</details>

## Fonksiyon neden yetmez?

Makro çağrısının sonundaki `!`, "bu bir fonksiyon değil, makro" demektir. İkisi arasında iki temel fark vardır.[^fark]

Birincisi imzadır. Bir fonksiyon kaç parametre alacağını ve türlerini baştan bildirir; makro değişken sayıda argüman alabilir. `println!("merhaba")` tek, `println!("merhaba {}", ad)` iki argümanlıdır.

İkincisi zamandır. Fonksiyon çalışma zamanında çağrılır; makro ise derleyici kodun anlamını yorumlamadan önce genişletilir. Bu yüzden makro bir türe trait uygulayabilir. Fonksiyon bunu yapamaz, çünkü trait'in derleme zamanında uygulanmış olması gerekir. `#[derive(Debug)]` gibi "her tür için ayrı kod yaz" işlerinin makroyla yapılmasının nedeni budur.

Bu gücün bir bedeli vardır: Makro, Rust kodu yazan Rust kodudur; okunması ve bakımı fonksiyondan zordur. Fonksiyonla çözülebilen bir iş için makro yazmayın.

## İki makro ailesi

Bildirimsel makro bir desen ile bir kod şablonundan oluşur: Çağrının içindeki kod desene uyarsa şablon doldurulur. [19. bölümdeki](19-advanced-features.md) `benim_vec!` böyleydi:

```rust
macro_rules! benim_vec {
    // Virgülle ayrılmış sıfır ya da daha çok ifade
    ( $( $x:expr ),* ) => {{
        let mut v = Vec::new();
        $( v.push($x); )* // yakalanan her ifade için bir push
        v
    }};
}

fn main() {
    let v: Vec<i32> = benim_vec![10, 20, 30];
    let bos: Vec<i32> = benim_vec![];
    println!("{v:?} {bos:?}");
}
```

Çıktı:

```text
[10, 20, 30] []
```

Yordamsal makro ise desen eşlemez. Kod parçalarını alan, onlarla istediği hesabı yapan ve yeni kod parçaları döndüren bir fonksiyondur. İkisi sık karıştırılır:[^aileler]

| | Bildirimsel makro | Yordamsal makro |
| --- | --- | --- |
| Nasıl yazılır? | `macro_rules!` ile desen ve şablon | `TokenStream` alıp `TokenStream` veren bir fonksiyon |
| Nerede durur? | Herhangi bir crate'te | Yalnızca `proc-macro` türündeki bir crate'te |
| Tanımlandığı crate'te kullanılabilir mi? | Evet | Hayır |
| Hijyen | Yerel değişkenlerde var | Varsayılan olarak yok |
| Biçimleri | `ad!(...)` | `ad!(...)`, `#[derive(Ad)]`, `#[ad]` |

## İlk yordamsal makro: `cevap_uret!`

Yazacağımız damga tek bir şey basar: `cevap_uret!();` yazan yere `fn cevap() -> u32 { 42 }` fonksiyonunu.

> [!NOTE]
> Yordamsal makro ayrı bir crate gerektirdiği için bu sayfadaki `#[proc_macro...]` örnekleri kitabın içinde tek başına derlenemez; blokları derleme denetiminin dışında tutulmuştur (`ignore`). Hepsi ayrı bir proc-macro crate'inde rustc 1.99.0 ile derlenip çalıştırılmış, çıktılar ve hata mesajları oradan alınmıştır.

Önce crate'in türü belirlenir. `Cargo.toml` içindeki `proc-macro = true`, *kütüphane hedefinin* yordamsal makro kütüphanesi olduğunu söyler:[^cargo]

```toml
[package]
name = "macros"
version = "0.1.0"
edition = "2024"

[lib]
proc-macro = true
```

Makro, `src/lib.rs`'te `#[proc_macro]` özniteliğiyle işaretlenmiş açık bir fonksiyondur:[^proc-macro]

```rust,ignore
use proc_macro::TokenStream;

#[proc_macro]
pub fn cevap_uret(_girdi: TokenStream) -> TokenStream {
    "fn cevap() -> u32 { 42 }"
        .parse()
        .expect("sabit metin geçerli Rust kodudur")
}
```

`proc_macro`, derleyiciyle birlikte gelen bir crate'tir; `Cargo.toml`'a bağımlılık olarak eklenmez. Fonksiyon girdisine hiç bakmaz (adı bu yüzden `_girdi`), düz bir metni ayrıştırıp token'lara çevirir. `parse` başarısız olabilir: Metin, kapanmamış bir parantez gibi token'lara ayrılamayan bir şey içeriyorsa `LexError` döner. Burada metin sabit olduğu için `expect` ile bunun beklenmediğini belirtiyoruz.

Makroyu kullanan kod ayrı bir crate olmalıdır. `tests/` altındaki her dosya ayrı bir crate olarak derlendiği için entegrasyon testi bu işe uygundur ([10. bölüm](10-writing-tests.md)). `tests/kullanim.rs`:

```rust,ignore
use macros::cevap_uret;

cevap_uret!(); // derleyici buraya fn cevap() -> u32 { 42 } yerleştirir

#[test]
fn cevap_42_doner() {
    assert_eq!(cevap(), 42);
}
```

Bu dosyada `cevap` adlı bir fonksiyon yazılı değildir, yine de çağrılabilir; çünkü onu makro üretmiştir. Testi çalıştıralım:

```console
$ cargo test -p macros
     Running tests/kullanim.rs (target/debug/deps/kullanim-9cc158076044dc50)

running 1 test
test cevap_42_doner ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Makro çağrısı unutulursa fonksiyon da ortada olmaz:

```rust,compile_fail,E0425
fn main() {
    assert_eq!(cevap(), 42); // cevap_uret!() çağrılmadı
}
```

Derleyicinin yanıtı:

```text
error[E0425]: cannot find function `cevap` in this scope
```

## Token'lar ve `TokenStream`

Derleyici kaynak metni önce token'lara ayırır. Token, kodun en küçük anlamlı parçasıdır: bir ad, bir sayı, bir işaret. Yordamsal makroya metin değil, art arda dizilmiş bu parçalar, yani bir `TokenStream` verilir. `TokenStream`'i gezince dört çeşit parça görürüz:[^token]

- **`Ident`:** Ad ya da anahtar sözcük: `fn`, `topla`.
- **`Punct`:** Tek bir işaret: `+`, `=`.
- **`Literal`:** Değişmez değer: `42`, `"iki"`.
- **`Group`:** Yuvarlak, köşeli ya da süslü parantez içindeki parçaların tamamı.

Aşağıdaki makro aldığı parçaları ekrana yazar ve yerine hiçbir şey koymaz:

```rust,ignore
use proc_macro::{TokenStream, TokenTree};

#[proc_macro]
pub fn tokenlari_goster(girdi: TokenStream) -> TokenStream {
    for token in girdi {
        match token {
            TokenTree::Ident(ad) => println!("ad:     {ad}"),
            TokenTree::Punct(isaret) => println!("işaret: {isaret}"),
            TokenTree::Literal(sabit) => println!("sabit:  {sabit}"),
            TokenTree::Group(grup) => println!("grup:   {grup}"),
        }
    }
    TokenStream::new() // yerine hiçbir şey koyma
}
```

Başka bir crate'te `tokenlari_goster!(fn topla(a + 1) "iki");` yazıp *derlediğimizde*, program daha çalışmadan şunlar görünür:

```text
ad:     fn
ad:     topla
grup:   (a + 1)
sabit:  "iki"
```

İki nokta dikkat çeker. Yazılar derleme sırasında çıkar, çünkü makro o sırada çalışır. Ayrıca `fn topla(a + 1) "iki"` geçerli bir Rust kodu değildir, yine de sorun olmaz: Makronun girdisi yalnızca token'lara ayrılabilmeli ve parantezleri dengeli olmalıdır. Kendi küçük dilini tanımlayan `sql!(SELECT ...)` gibi makrolar buna dayanır.

### Girdiyi okuyup ona göre kod üretmek

Gerçek makrolar girdiye bakar. `cevap_uret_adli!(dogru_cevap = 43)`, verilen adla verilen sayıyı döndüren bir fonksiyon üretsin. Parçaları bir dilime toplayıp [18. bölümdeki](18-patterns.md) dilim desenleriyle eşleyebiliriz:

```rust,ignore
use proc_macro::{TokenStream, TokenTree};

#[proc_macro]
pub fn cevap_uret_adli(girdi: TokenStream) -> TokenStream {
    let tokenler: Vec<TokenTree> = girdi.into_iter().collect();

    let kod = match tokenler.as_slice() {
        [TokenTree::Ident(ad), TokenTree::Punct(isaret), TokenTree::Literal(deger)]
            if isaret.as_char() == '=' =>
        {
            format!("fn {ad}() -> u32 {{ {deger} }}")
        }
        _ => r#"compile_error!("beklenen biçim: cevap_uret_adli!(ad = sayı)");"#.to_string(),
    };
    kod.parse().expect("üretilen metin geçerli Rust kodudur")
}
```

Doğru çağrı `fn dogru_cevap() -> u32 { 43 }` üretir. Yanlış çağrıda makro, kod yerine bir `compile_error!` çağrısı döndürür; derleyici de bunu sıradan bir hata olarak gösterir. `cevap_uret_adli!(42)` yazıldığında alınan mesaj:

```text
error: beklenen biçim: cevap_uret_adli!(ad = sayı)
```

> [!TIP]
> Bu sayfadaki makrolar elle yazılacak kadar küçüktür. Gerçek projelerde girdiyi bir sözdizimi ağacına çevirmek için `syn`, kodu bir şablondan üretmek için `quote` crate'i kullanılır; The Rust Programming Language'ın `derive` örneği de bu ikiliyle yazılmıştır.[^syn]

## Yordamsal makronun üç türü

Üç türün fonksiyonları birbirine benzer; fark, nereye yazıldıkları ve ürettikleri koda ne olduğudur:[^uc-tur]

| Tür | Tanım özniteliği | Kullanım | Üretilen koda ne olur? |
| --- | --- | --- | --- |
| Fonksiyon benzeri | `#[proc_macro]` | `cevap_uret!()`, `sql!(...)` | Çağrının yerine geçer. |
| `derive` | `#[proc_macro_derive(Ad)]` | `#[derive(Ad)]` | Türün tanımına dokunulmaz, kod yanına eklenir. |
| Öznitelik | `#[proc_macro_attribute]` | `#[route(GET, "/")]` | Özniteliğin uygulandığı öğenin yerine geçer. |

`cevap_uret!` ve `tokenlari_goster!` birinci türdendi. Öteki ikisine birer küçük örnek yazalım.

### `derive` makrosu

`derive` makrosu bir `struct`, `enum` ya da `union` tanımının token'larını alır ve tanımın yanına yeni öğeler ekler. `Tanit` türetildiği türe, kendi adını veren bir fonksiyon eklesin:

```rust,ignore
use proc_macro::TokenStream;

#[proc_macro_derive(Tanit)]
pub fn derive_tanit(girdi: TokenStream) -> TokenStream {
    let ad = tur_adi(girdi).expect("struct ya da enum bekleniyordu");
    format!("impl {ad} {{ pub fn tanit() -> &'static str {{ \"{ad}\" }} }}")
        .parse()
        .expect("üretilen metin geçerli Rust kodudur")
}

// `struct` ya da `enum` sözcüğünden sonra gelen adı bulur.
fn tur_adi(girdi: TokenStream) -> Option<String> {
    let mut tokenler = girdi.into_iter();
    tokenler.find(|t| matches!(t.to_string().as_str(), "struct" | "enum"))?;
    tokenler.next().map(|ad| ad.to_string())
}
```

Makronun türetme sırasında kullanılan adı fonksiyonun adı değil, öznitelikteki `Tanit`'tir. `tur_adi` ise sıradan, özel bir yardımcıdır. Bu hâli jenerik türleri hesaba katmaz: `struct Kutu<T>` için geçersiz kod üretir. `syn` kullanmanın bir nedeni de böyle ayrıntılardır.

> [!NOTE]
> `#[derive(Debug)]` ve öteki standart türetmeler aynı biçimde kullanılır ve aynı işi yapar; ama ayrı bir proc-macro crate'inde değil, derleyicinin içinde tanımlıdırlar.[^yerlesik] Bildirimsel olarak, yani `macro_rules!` ile `derive` ya da öznitelik makrosu yazmak ise 1.99.0'da da 1.100.0'da da kararsızdır (`macro_derive` ve `macro_attr`).[^kararsiz]

### Öznitelik makrosu

Öznitelik makrosu iki akış alır: özniteliğin parantezi içindeki token'lar ve özniteliğin uygulandığı öğe. Döndürdüğü kod öğenin *yerine* geçer; öğeyi korumak istiyorsanız onu da döndürmelisiniz. `iz_birak`, fonksiyonun gövdesinin başına bir `println!` satırı eklesin:

```rust,ignore
use proc_macro::{Delimiter, Group, TokenStream, TokenTree};

#[proc_macro_attribute]
pub fn iz_birak(nitelik: TokenStream, oge: TokenStream) -> TokenStream {
    let ek: TokenStream = format!("println!({nitelik});")
        .parse()
        .expect("özniteliğe bir metin sabiti verilmeli");

    oge.into_iter()
        .map(|token| match token {
            // Fonksiyonun gövdesi: süslü parantezli grup.
            TokenTree::Group(g) if g.delimiter() == Delimiter::Brace => {
                let mut govde = ek.clone();
                govde.extend(g.stream());
                TokenTree::Group(Group::new(Delimiter::Brace, govde))
            }
            baska => baska,
        })
        .collect()
}
```

`#[iz_birak("selamla çağrıldı")]` yazıldığında `nitelik` `"selamla çağrıldı"` sabitini, `oge` ise `fn selamla(...) { ... }` tanımının tamamını taşır.

### Hepsini kullanmak

Üç türü başka bir crate'te bir araya getirelim (`damga`, makroların bulunduğu crate'in adı):

```rust,ignore
use damga::{Tanit, cevap_uret, cevap_uret_adli, iz_birak};

cevap_uret!();
cevap_uret_adli!(dogru_cevap = 43);

#[derive(Tanit)]
#[allow(dead_code)]
struct Kullanici {
    ad: String,
}

#[iz_birak("selamla çağrıldı")]
fn selamla(ad: &str) {
    println!("Merhaba, {ad}!");
}

fn main() {
    println!("{} {}", cevap(), dogru_cevap());
    println!("{}", Kullanici::tanit());
    selamla("Ece");
}
```

Program çalıştığında:

```text
42 43
Kullanici
selamla çağrıldı
Merhaba, Ece!
```

## Neden ayrı bir crate?

Yordamsal makro, kendisini kullanan crate derlenirken *çalışacak* bir koddur. O anda çoktan derlenmiş, derleyicinin yükleyip çağırabileceği bir kütüphane hâline gelmiş olması gerekir. Aynı crate'in içinde bu mümkün değildir: Makro, henüz derlenmekte olan kodun bir parçası olurdu. Kuralı derleyici uygular.[^ayri-crate] `cevap_uret!()` çağrısını makronun tanımlandığı `src/lib.rs`'e yazarsanız:

```text
error: can't use a procedural macro from the same crate that defines it
```

Aynı düzenin ikinci sonucu, `proc-macro` türündeki bir kütüphanenin yordamsal makrodan başka bir öğe dışa aktaramamasıdır. Bu crate'e açık bir yardımcı fonksiyon (`pub fn yardimci() {}`) eklerseniz:

```text
error: `proc-macro` crate types currently cannot export any items other than functions tagged with `#[proc_macro]`, `#[proc_macro_derive]`, or `#[proc_macro_attribute]`
```

Kısıt *dışa aktarmayla* ilgilidir. `tur_adi` gibi özel ya da `pub(crate)` fonksiyonlar yazabilirsiniz. Kısıt ayrıca yalnızca kütüphane hedefini kapsar: Aynı pakette `src/main.rs` bulunabilir ve bu ikili hedef ayrı bir crate olduğu için makroyu kullanabilir. `learn/macros` crate'inde `fn main` bulunmaması bir zorunluluk değil, tercihtir.

Tersi de geçerlidir. Sıradan bir crate'te `#[proc_macro]` kullanılamaz:

```rust,compile_fail
extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro]
pub fn my_macro(_item: TokenStream) -> TokenStream {
    "fn answer() -> u32 { 42 }".parse().unwrap()
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error: the `proc_macro` attribute is only usable with crates of the `proc-macro` crate type
```

Çözüm, makroyu `[lib] proc-macro = true` olan ayrı bir crate'e taşımaktır. Bir kütüphane hem tür hem makro sunmak istiyorsa iki crate'e bölünür; ana crate makroyu yeniden dışa aktarabilir ve kullanıcı tek bir bağımlılık görür.[^syn]

## Hijyen: Üretilen kod çevresine karışır

Damganın mürekkebi kâğıttaki öteki yazılara bulaşır mı? Bildirimsel makroda yerel değişkenler korunur: Makronun içinde tanımlanan `x`, çağrı yerindeki `x` ile karışmaz.[^hijyen-mbe]

```rust
macro_rules! x_tanimla {
    () => {
        let x = 99; // çağrı yerindeki x'ten ayrıdır
    };
}

fn main() {
    let x = 1;
    x_tanimla!();
    println!("{x}");
}
```

Çıktı:

```text
1
```

Aynı kodu (`"let x = 99;".parse()`) bir yordamsal makro ürettiğinde sonuç 99 olur: Üretilen token'lar, çağrının bulunduğu yere elle yazılmış gibi davranır. Yordamsal makrolar *hijyenik değildir*.[^hijyen-pm] Bu yüzden makro yazarken iki alışkanlık edinin: Dışarıdaki öğelere tam yollarıyla (`::std::option::Option` gibi) başvurun ve ürettiğiniz yardımcı öğelere başkasının kullanmayacağı adlar verin.

Öğe adları (fonksiyon, yapı, modül) iki ailede de çağrı yerine yerleşir. Sabit bir ad üreten makroyu aynı kapsamda iki kez çağırmak bu yüzden hata verir:

```rust,compile_fail,E0428
macro_rules! cevap_uret {
    () => {
        fn cevap() -> u32 { 42 }
    };
}

cevap_uret!();
cevap_uret!();

fn main() {
    println!("{}", cevap());
}
```

Derleyicinin yanıtı:

```text
error[E0428]: the name `cevap` is defined multiple times
```

Yordamsal `cevap_uret!`'i iki kez çağırdığınızda da aynı hatayı alırsınız. Çözüm, adı `cevap_uret_adli!` gibi argümandan almak ya da çağrıları ayrı modüllere koymaktır.

## Hata bildirmek

Yordamsal makro bir fonksiyondur; ya kod döndürür ya panikler ya da hiç bitmez.[^hata] Üç durumda kullanıcının gördüğü şey farklıdır.

**`compile_error!` döndürmek** en iyi yoldur. Mesaj sıradan bir derleme hatası gibi, çağrı yerini göstererek çıkar; yukarıdaki `beklenen biçim` örneği böyleydi.

**Paniklemek** de derlemeyi durdurur. Derleyici paniği yakalar ve hataya çevirir:

```text
error: proc macro panicked
  |
  = help: message: bu damga bozuk
```

Ayrıştırılamayan bir metinle `unwrap` çağırmak da buraya düşer. Kapanmayan bir parantez içeren `"fn sayi( {"` için mesaj şöyledir:

```text
  = help: message: called `Result::unwrap()` on an `Err` value: LexError("this file contains an unclosed delimiter")
```

**Geçerli token'lar, yanlış kod.** Makro token'lara ayrılabilen ama türleri tutmayan bir kod üretirse hatayı derleyici verir ve makronun çağrıldığı yeri gösterir. `fn sayi() -> u32 { "metin" }` üreten bir makro için:

```text
error[E0308]: mismatched types
  |
  = note: this error originates in the macro `damga::hatali_kod` (in Nightly builds, run with -Z macro-backtrace for more info)
```

Sonsuz döngü ise yakalanmaz; derleyici takılır kalır.

## Derleme sırasında çalışan kod

Yordamsal makro derleme sırasında, derleyicinin sahip olduğu kaynaklarla çalışır: Dosya okuyabilir, ekrana yazabilir. `tokenlari_goster!`'in derlerken yazı yazması bunun sonucuydu. Güvenlik açısından anlamı şudur: Bir crate'in yordamsal makrosunu kullanmak, o crate'in kodunu derleme sırasında kendi bilgisayarınızda çalıştırmak demektir. The Rust Reference bu konuda makroları Cargo'nun derleme betikleriyle (build script) aynı sınıfa koyar.[^guvenlik] Bağımlılık eklerken bunu hesaba katın.

Çalışan programda ise makrodan geriye bir şey kalmaz. Makro çağrısı derleme sırasında üretilen kodla değişmiştir; çalışma zamanında bir "makro çağırma" işi yoktur.

## Sık yapılan hatalar

| Ne yapılıyor? | Ne olur? | Doğrusu |
| --- | --- | --- |
| `cevap_uret!()`'i makronun tanımlandığı `src/lib.rs` içinde çağırmak | `can't use a procedural macro from the same crate that defines it` | Çağrıyı başka bir crate'e, örneğin `tests/` altındaki bir dosyaya yazın. |
| Proc-macro kütüphanesinden `pub fn` ya da `pub struct` dışa aktarmak | `proc-macro crate types currently cannot export any items other than ...` | Yardımcıyı özel yapın ya da ayrı bir sıradan crate'e taşıyın. |
| Sıradan crate'te `#[proc_macro]` yazmak | `only usable with crates of the proc-macro crate type` | `Cargo.toml`'a `[lib] proc-macro = true` ekleyin ya da ayrı crate açın. |
| `cevap_uret!()` çağırmadan `cevap()` kullanmak | `E0425` | Fonksiyonu makro üretir; önce makroyu çağırın. |
| Sabit ad üreten makroyu aynı kapsamda iki kez çağırmak | `E0428` | Adı argümandan alın. |
| `macro_rules!` ile yordamsal makroyu aynı şey sanmak | Yanlış araç seçilir | İlki desen ve şablon, ikincisi `TokenStream` alan fonksiyondur. |

## Hata kodu başvurusu

| Kod | Ne zaman görülür? | Ne yapmalı? |
| --- | --- | --- |
| `E0425` | Makronun üreteceği fonksiyon, makro çağrılmadan kullanıldığında | Makroyu çağırın ve kapsama alındığını denetleyin. |
| `E0428` | Aynı kapsamda aynı adlı iki öğe üretildiğinde | Adı değiştirin ya da argümandan alın. |
| `E0308` | Makronun ürettiği kodun türleri tutmadığında | Makronun ürettiği kodu düzeltin; hata çağrı yerinde görünür. |

Yordamsal makro crate'inin kurallarıyla ilgili hataların (aynı crate'te kullanım, dışa aktarma kısıtı, `#[proc_macro]`'nun yeri) hata kodu yoktur. Kodu olanların uzun açıklaması:

```console
rustc --explain E0428
```

## Perde arkası: Derleyici ne yapıyor?

> [!NOTE]
> Bu başlık ilk okuyuşta atlanabilir; makro kullanmak için gerekli değildir.

**Genişletme, derlemenin ilk aşamalarındadır.** Kaynak metin token'lara, token'lar da bir sözdizimi ağacına (AST) çevrilir. `MacroExpander`, ağaçtaki makro çağrılarını bulup genişletir ve sonuçta yeni çağrılar çıkarsa işi bitene kadar sürdürür.[^genisletme] Ad çözümleme bu süreçle iç içe yürür; tür denetimi ise genişletilmiş kod üzerinde yapılır. Üretilen koddaki tür hatasının makroda değil çağrı yerinde görünmesi bu yüzdendir.

**Makro crate'i bir kütüphane olarak yüklenir.** `proc-macro` türündeki crate, derleyicinin çalıştığı makine için dinamik kütüphane olarak derlenir (Linux'ta `libmacros-….so`). Onu kullanan crate derlenirken derleyici bu kütüphaneyi yükler ve içindeki makroların listesini okur.[^yukleme] O listeyi crate'e ekleyen, dışa aktarma kısıtını da denetleyen kod `proc_macro_harness.rs` dosyasındadır.[^harness]

**Çağrı, bir köprüden geçer.** Fonksiyon benzeri bir makro için derleyici `BangProcMacro::expand` üzerinden kütüphanedeki fonksiyonu çağırır, girdi token'larını verir ve çıktıyı alır. Fonksiyon paniklerse bu kod hatayı `proc macro panicked` mesajına dönüştürür.[^bang] Token'lar sözdizimi ağacı yerine bilerek tercih edilmiştir: Ağacın biçimi derleyici sürümleri arasında değişebilir, token'lar ise hem derleyici hem makro yazarları için çok daha kararlı bir arayüzdür.[^token]

## Özet

- Makro, derleme sırasında kod üreten damgadır. Fonksiyondan farkı, değişken sayıda argüman alabilmesi ve derleme zamanında çalışmasıdır.
- İki aile vardır: `macro_rules!` ile yazılan bildirimsel makrolar ve `TokenStream` alıp `TokenStream` veren yordamsal makrolar. Yordamsal makro fonksiyon benzeri, `derive` ya da öznitelik biçiminde olabilir.
- Yordamsal makro `proc-macro = true` olan ayrı bir crate'te tanımlanır ve yalnızca başka bir crate'te kullanılabilir. Kütüphane hedefi makrodan başka öğe dışa aktaramaz; özel fonksiyonlar ve ayrı bir ikili hedef serbesttir.
- Yordamsal makro hijyenik değildir, derleyicinin yetkileriyle çalışır ve hataları en iyi `compile_error!` ile bildirir.

Sıradaki bölüm, üçüncü kısmın ilk konusu olan [21. Alt Tür İlişkisi ve Varyans](../advanced/21-variance.md).

## Kaynaklar

[^fark]: The Rust Programming Language, [The Difference Between Macros and Functions](https://doc.rust-lang.org/book/ch20-05-macros.html#the-difference-between-macros-and-functions); The Rust Reference, [Macro invocation](https://doc.rust-lang.org/reference/macros.html#macro-invocation).
[^aileler]: The Rust Programming Language, [Declarative Macros for General Metaprogramming](https://doc.rust-lang.org/book/ch20-05-macros.html#declarative-macros-for-general-metaprogramming) ve [Procedural Macros for Generating Code from Attributes](https://doc.rust-lang.org/book/ch20-05-macros.html#procedural-macros-for-generating-code-from-attributes).
[^cargo]: The Cargo Book, [The `proc-macro` field](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#the-proc-macro-field); The Rust Reference, [Procedural macros](https://doc.rust-lang.org/reference/procedural-macros.html#r-macro.proc.def).
[^proc-macro]: The Rust Reference, [The `proc_macro` attribute](https://doc.rust-lang.org/reference/procedural-macros.html#the-proc_macro-attribute); [`proc_macro::TokenStream`](https://doc.rust-lang.org/proc_macro/struct.TokenStream.html) ve [`proc_macro::LexError`](https://doc.rust-lang.org/proc_macro/struct.LexError.html).
[^token]: The Rust Reference, [The `proc_macro` crate](https://doc.rust-lang.org/reference/procedural-macros.html#the-proc_macro-crate); [`proc_macro::TokenTree`](https://doc.rust-lang.org/proc_macro/enum.TokenTree.html).
[^syn]: The Rust Programming Language, [Custom `derive` Macros](https://doc.rust-lang.org/book/ch20-05-macros.html#custom-derive-macros).
[^uc-tur]: The Rust Reference, [The `proc_macro_derive` attribute](https://doc.rust-lang.org/reference/procedural-macros.html#the-proc_macro_derive-attribute) ve [The `proc_macro_attribute` attribute](https://doc.rust-lang.org/reference/procedural-macros.html#the-proc_macro_attribute-attribute); The Rust Programming Language, [Attribute-Like Macros](https://doc.rust-lang.org/book/ch20-05-macros.html#attribute-like-macros) ve [Function-Like Macros](https://doc.rust-lang.org/book/ch20-05-macros.html#function-like-macros).
[^yerlesik]: std 1.99.0 kaynak kodu, [`fmt/mod.rs`, `rustc_builtin_macro`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/fmt/mod.rs#L1090).
[^kararsiz]: rustc 1.99.0 kaynak kodu, [`unstable.rs`, `macro_attr`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_feature/src/unstable.rs#L623) ve [`macro_derive`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_feature/src/unstable.rs#L625); izleme kayıtları [143547](https://github.com/rust-lang/rust/issues/143547) ve [143549](https://github.com/rust-lang/rust/issues/143549).
[^ayri-crate]: The Rust Reference, [Procedural macros](https://doc.rust-lang.org/reference/procedural-macros.html#r-macro.proc.def); rustc 1.99.0 kaynak kodu, [`diagnostics/mod.rs`, `ProcMacroSameCrate`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_resolve/src/diagnostics/mod.rs#L598).
[^hijyen-mbe]: The Rust Reference, [Macros by example: Hygiene](https://doc.rust-lang.org/reference/macros-by-example.html#hygiene).
[^hijyen-pm]: The Rust Reference, [Procedural macro hygiene](https://doc.rust-lang.org/reference/procedural-macros.html#procedural-macro-hygiene); hata dizini, [`E0428`](https://doc.rust-lang.org/error_codes/E0428.html).
[^hata]: The Rust Reference, [Procedural macros: result](https://doc.rust-lang.org/reference/procedural-macros.html#r-macro.proc.result) ve [error](https://doc.rust-lang.org/reference/procedural-macros.html#r-macro.proc.error); std, [`compile_error!`](https://doc.rust-lang.org/std/macro.compile_error.html).
[^guvenlik]: The Rust Reference, [Procedural macros: result](https://doc.rust-lang.org/reference/procedural-macros.html#r-macro.proc.result); The Cargo Book, [Build Scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html).
[^genisletme]: rustc 1.99.0 kaynak kodu, [`expand.rs`, `fully_expand_fragment`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_expand/src/expand.rs#L471); rustc-dev-guide, [Macro expansion](https://rustc-dev-guide.rust-lang.org/macro-expansion.html).
[^yukleme]: rustc 1.99.0 kaynak kodu, [`creader.rs`, `dlsym_proc_macros`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_metadata/src/creader.rs#L941); The Rust Reference, [Linkage](https://doc.rust-lang.org/reference/linkage.html).
[^harness]: rustc 1.99.0 kaynak kodu, [`proc_macro_harness.rs`, `inject`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_builtin_macros/src/proc_macro_harness.rs#L49).
[^bang]: rustc 1.99.0 kaynak kodu, [`proc_macro.rs`, `BangProcMacro`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_expand/src/proc_macro.rs#L33).
