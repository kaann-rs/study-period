# 11. minigrep: Gerçek Bir Program

Şimdiye kadar kavramları tek tek, birkaç satırlık örneklerle gördük. Bu bölümde yeni bir kavram yok; öğrendiklerimizi gerçek bir komut satırı aracında bir araya getiriyoruz. Program, Unix dünyasının ünlü `grep` komutunun küçük bir kardeşidir: Ona bir kelime ve bir dosya adı verirsiniz, o da kelimenin geçtiği satırları ekrana yazar.

Program küçüktür, ama gerçek bir programın bütün parçalarını taşır: Kullanıcıdan girdi alır, girdinin yanlış olabileceğini hesaba katar, dosya okur, sonucu ve hatayı ayrı kanallara yazar, biterken başarılı olup olmadığını bildirir ve testleri vardır. Bu yüzden `Result` ve `?` ([8. bölüm](../temel/08-error-handling.md)), ömürler ve trait'ler ([9. bölüm](09-generics-traits-lifetimes.md)) ile testler ([10. bölüm](10-writing-tests.md)) burada yan yana çalışır. Birkaç yerde de closure ve iterator kullanacağız; onlar [bir sonraki bölümün](12-closures-iterators.md) konusudur, burada okumaya yetecek kadar açıklanacaklar.

Bölümün asıl dersi ise tasarımla ilgilidir: Mantığı test edilebilen bir kütüphaneye koyun, test etmesi zor olan `main`'i olabildiğince küçültün.

> [!NOTE]
> Bu bölümün kodu `learn/minigrep` dizinindedir. Depo kökünden `cargo run -p minigrep -- hiç learn/minigrep/siir.txt` komutuyla çalıştırabilir, `cargo test -p minigrep` ile sınayabilirsiniz.

## Günlük hayattan bir benzetme

Kütüphanede bir danışma masası düşünün. Masaya gelip bir istek fişi doldurursunuz: Hangi kelimeyi arıyorsunuz, hangi kitapta, büyük-küçük harf fark eder mi? Kapıdaki kişi fişi alıp görevliye verir; aramayı o yapmaz. Görevli kitabı raftan getirir, sayfaları tarar ve "masal" kelimesinin geçtiği yerleri size gösterir.

Fişi eksik doldurursanız görevli bağırıp masayı kapatmaz; fişi geri uzatır ve "kitap adını yazmamışsınız" der. Kitap rafta yoksa "bulamadım" deyip döner. Bulduğu satırları deftere yeniden yazmaz, kitabın o sayfalarına ayraç koyar.

Programdaki karşılıkları şöyledir: Fiş `Config`'tir, fişin doldurulması `Config::build`, görevli `run` fonksiyonudur, ayraçlar `&str` dilimleridir. Kapıdaki kişi de `main`'dir: Fişi doldurtur, sorun olursa hata kanalına yazar ve çıkış koduyla işi bitirir.

<details>
<summary>Bu bölümde geçen terimler</summary>

- **grep:** Metin içinde kelime arayan ünlü komut; minigrep onun küçüğüdür.
- **komut satırı (command line, CLI):** Programı fareyle değil, yazı yazarak çalıştırdığınız arayüz.
- **argüman:** Komutta program adından sonra yazılanlar: `hiç siir.txt`.
- **ortam değişkeni (environment variable):** Programa dışarıdan verilen adlı ayar: `IGNORE_CASE=1`.
- **girdi/çıktı (I/O):** Programın dış dünyayla alışverişi; dosya okumak, ekrana yazmak.
- **`Config`:** Ayarların toplandığı yapı; bu programda biz tanımlıyoruz.
- **iterator, `Item`:** Öğeleri sırayla veren değer; verdiği öğenin türü.
- **closure:** `|x| ...` biçiminde yazılan adsız, küçük fonksiyon.
- **`lines`, `filter`, `collect`:** Satırlara bölmek; koşula uyanları seçmek; seçilenleri bir koleksiyonda toplamak.
- **ömür (lifetime, `'a`):** Bir referansın geçerli kaldığı bölge ([9. bölüm](09-generics-traits-lifetimes.md)).
- **`Box<dyn Error>`:** İçine herhangi bir hata konabilen kutu ([8. bölüm](../temel/08-error-handling.md)).
- **stdout, stderr:** Programın iki ayrı çıkış kanalı; biri sonuçlar, öteki hatalar için.
- **yönlendirme (`>`), boru (`|`):** Çıktıyı ekran yerine bir dosyaya ya da başka bir programa göndermek.
- **çıkış kodu (exit code):** Programın biterken bıraktığı sayı. 0 başarı, başka bir sayı başarısızlık demektir.
- **kabuk (shell), betik (script):** Komutları yazdığınız program; art arda komut çalıştıran dosya.
- **TDD (test-driven development):** Önce testi, sonra onu geçirecek kodu yazma alışkanlığı.
- **tahsis (allocation):** Heap'ten yer istemek; gereksizse kaçınılır.

</details>

## Programın iskeleti

Paket iki hedef içerir. `src/lib.rs` bir kütüphanedir (library crate) ve bütün mantığı taşır; `src/main.rs` ise bu kütüphaneyi kullanan ince bir ikilidir (binary crate):

```text
minigrep/
├── Cargo.toml
├── siir.txt
└── src/
    ├── lib.rs    # Config, run, ara, ara_harf_duyarsiz ve testler
    └── main.rs   # argümanları al, hatayı göster, çıkış kodunu ver
```

Bu ayrımın nedeni testtir. Kütüphanedeki açık (`pub`) fonksiyonlar hem birim testlerinden hem `tests/` dizinindeki entegrasyon testlerinden çağrılabilir. İkili bir crate'in fonksiyonları ise entegrasyon testine içe aktarılamaz; yalnızca ikiliyi bir program gibi çalıştırıp çıktısına bakabilirsiniz.[^entegrasyon] `main.rs`'in içine `#[test]` yazıp `main`'i çağırmak mümkündür, ama argümanlarını işletim sisteminden alan ve programı sonlandırabilen bir fonksiyonu böyle sınamak zahmetlidir. The Rust Programming Language'ın önerisi de aynıdır: Mantık `lib.rs`'e taşınır, `main`'e doğruluğu okuyarak görülebilecek kadar az kod kalır.[^ayrim]

Arama yapacağımız `siir.txt` sekiz satırlık bir şiirdir:

```text
Ben bir hiçim, sen kimsin?
Sen de bir hiç misin?
O zaman ikimiz de hiçiz, sakın söyleme!
Sürerler bizi, bilirsin.

Ne sıkıcı olurdu bir şey olmak!
Ne bayağı, bir kurbağa gibi
Adını söyleyip durmak gün boyu
Hayran bir bataklığa.
```

Programı depo kökünden çalıştırdığınızda görmeyi beklediğimiz sonuç şudur:

```console
$ cargo run -q -p minigrep -- hiç learn/minigrep/siir.txt
Ben bir hiçim, sen kimsin?
Sen de bir hiç misin?
O zaman ikimiz de hiçiz, sakın söyleme!
```

`--` işaretinden sonra yazılanlar Cargo'ya değil, programın kendisine gider. `-q` (quiet) Cargo'nun derleme mesajlarını gizler. Şimdi parçaları sırayla yazalım.

## İstek fişi: `Config`

Programın bilmesi gereken üç şey vardır: aranan kelime, dosyanın yolu ve büyük-küçük harfin önemli olup olmadığı. Bunları tek bir yapıda topluyoruz:

```rust
pub struct Config {
    pub sorgu: String,
    pub dosya_yolu: String,
    pub harf_duyarsiz: bool, // büyük-küçük harf ayrımı yapılmasın mı?
}

fn main() {
    let fis = Config {
        sorgu: String::from("hiç"),
        dosya_yolu: String::from("siir.txt"),
        harf_duyarsiz: false, // "Hiç" ile "hiç" ayrı sayılsın
    };
    println!("{} / {} / {}", fis.sorgu, fis.dosya_yolu, fis.harf_duyarsiz);
}
```

Çıktı:

```text
hiç / siir.txt / false
```

Bilgiler dağınık dursaydı her fonksiyona üçünü ayrı ayrı taşımanız gerekirdi; biri eksik ya da yanlış sırada gidebilirdi. Yapı, birbirine ait değerleri adlarıyla bir arada tutar.[^config]

### Argümanları okumak

Komut satırı argümanlarını `std::env::args` verir. Dönen değer bir *iterator*'dır: `next` çağrısıyla sıradaki argümanı `Some(String)` olarak verir, argüman kalmayınca `None` döner. İlk öğe geleneksel olarak programın kendi adıdır, bu yüzden onu atlarız:[^args]

```rust
use std::env;
# pub struct Config {
#     pub sorgu: String,
#     pub dosya_yolu: String,
#     pub harf_duyarsiz: bool,
# }

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next(); // ilk argüman programın kendi adı

        let sorgu = match args.next() {
            Some(s) => s,
            None => return Err("sorgu verilmedi"),
        };
        let dosya_yolu = match args.next() {
            Some(d) => d,
            None => return Err("dosya yolu verilmedi"),
        };

        let harf_duyarsiz = env::var("IGNORE_CASE").is_ok();

        Ok(Config { sorgu, dosya_yolu, harf_duyarsiz })
    }
}

fn dene(argumanlar: &[&str]) {
    match Config::build(argumanlar.iter().map(|a| a.to_string())) {
        Ok(c) => println!("sorgu = {}, dosya = {}", c.sorgu, c.dosya_yolu),
        Err(hata) => println!("hata: {hata}"),
    }
}

fn main() {
    dene(&["minigrep", "hiç", "siir.txt"]);
    dene(&["minigrep", "hiç"]);
    dene(&["minigrep"]);
}
```

Çıktı:

```text
sorgu = hiç, dosya = siir.txt
hata: dosya yolu verilmedi
hata: sorgu verilmedi
```

`build`'in üç tasarım kararına yakından bakalım.

**Neden paniklemiyor?** Eksik argüman programın değil, kullanıcının yanlışıdır. Kullanıcıya çökme raporu değil, anlaşılır bir cümle gerekir. `build` bu yüzden nedenini söyleyen bir `Err` döndürür; ne yapılacağına çağıran karar verir.[^result]

**Neden iterator alıyor?** `impl Iterator<Item = String>` "sırayla `String` veren herhangi bir şey" demektir. Gerçek programda `env::args()` verilir, testte ise elle kurulmuş bir liste. Üstelik her `String` iterator'dan `Config`'e *taşınır*. Argümanları önce bir `Vec`'e toplayıp `&[String]` olarak alsaydık her birini `.clone()` ile kopyalamamız, yani gereksiz yere tahsis yapmamız gerekirdi.[^klon]

**Programın adına güvenilir mi?** Güvenlikle ilgili bir karar için hayır: Standart kütüphane belgesi, ilk argümanın herhangi bir metin olabileceğini, hatta hiç bulunmayabileceğini söyler.[^args] Biz onu yalnızca atlıyoruz.

> [!NOTE]
> `env::args`, geçerli Unicode olmayan bir argümana rastlarsa gezinme sırasında panikler. Böyle dosya adlarıyla da çalışması gereken programlar, `OsString` veren `env::args_os` kullanır.[^args-os] minigrep için buna gerek yoktur.

### Ortam değişkeni: `IGNORE_CASE`

Büyük-küçük harf ayrımını kapatmak için argüman yerine bir ortam değişkeni kullanıyoruz. Kullanıcı bunu bir kez ayarlayıp o oturumdaki bütün aramalarda kullanabilir. `env::var` değişken tanımlıysa `Ok(değer)`, değilse `Err` döner; biz değerle ilgilenmediğimiz için yalnızca `is_ok` çağırıyoruz.[^ortam]

Bunun sonucu olarak değerin `1` olması gerekmez. `IGNORE_CASE=0` ya da boş bırakılmış `IGNORE_CASE=` da aramayı harf duyarsız yapar. Değişken tanımlı ama geçerli Unicode değilse `env::var` yine `Err` döner, dolayısıyla ayar kapalı sayılır.

## Aramayı yazmak: `ara`

### Önce test

Danışma görevlisine işe başlarken küçük bir deneme yaptırılır: "Şu üç satırlık kâğıtta 'hiç' kelimesini bul." Doğru cevap bellidir. *Test güdümlü geliştirme* (TDD) sırayı tersine çevirir: Önce beklentiyi yazarsınız, test düşer; sonra onu geçirecek kodu yazarsınız. Beklentiyi yazmak, "bu kod ne yapmalı?" sorusunu netleştirir.[^tdd]

```rust
pub fn ara<'a>(_sorgu: &str, _icerik: &'a str) -> Vec<&'a str> {
    vec![] // henüz yazılmadı
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn harf_duyarli_tek_sonuc() {
        let sorgu = "hiç";
        let icerik = "Ben bir hiçim\nSen kimsin\nHiç misin";
        // "hiç" küçük harfle yalnızca ilk satırda geçer.
        assert_eq!(vec!["Ben bir hiçim"], ara(sorgu, icerik));
    }
}
# fn main() {}
```

Bu hâliyle test düşer ve neyin eksik olduğunu gösterir:

```text
assertion `left == right` failed
  left: ["Ben bir hiçim"]
 right: []
```

### Sonra kod

Arama üç adımdır: metni satırlara bölmek, kelimeyi içerenleri seçmek ve seçilenleri toplamak. Bunu önce bildik bir döngüyle yazalım, sonra aynı işi iterator zinciriyle:

```rust
pub fn ara_dongu<'a>(sorgu: &str, icerik: &'a str) -> Vec<&'a str> {
    let mut sonuclar = Vec::new();
    for satir in icerik.lines() {
        if satir.contains(sorgu) {
            sonuclar.push(satir);
        }
    }
    sonuclar
}

pub fn ara<'a>(sorgu: &str, icerik: &'a str) -> Vec<&'a str> {
    icerik
        .lines() // satırlara böl
        .filter(|satir| satir.contains(sorgu)) // uyanları seç
        .collect() // seçilenleri topla
}

fn main() {
    let icerik = "Ben bir hiçim\nSen kimsin\nHiç misin";

    println!("{:?}", ara_dongu("hiç", icerik));
    println!("{:?}", ara("hiç", icerik));
    println!("{:?}", ara("yok", icerik)); // boş sonuç bir hata değildir
}
```

Çıktı:

```text
["Ben bir hiçim"]
["Ben bir hiçim"]
[]
```

"Hiç misin" satırı listede yoktur, çünkü oradaki "Hiç" büyük harfle başlar. İkinci biçimde durumu tutan değiştirilebilir bir `Vec` kalmamıştır; `|satir| satir.contains(sorgu)` yerinde yazılmış küçük bir fonksiyondur ve `sorgu`'yu çevresinden alır.[^lines]

### İmzadaki `'a` ne söylüyor?

Dönen satırlar kopya değil, `icerik`'in içine bakan dilimlerdir. Görevlinin kitaba koyduğu ayraçlar gibi: Kitap rafa dönünce ayraçlar işe yaramaz. Aradığınız kelimeyi yazdığınız kâğıt ise iş bitince atılabilir; ayraçların onunla bir bağı yoktur. İmzadaki `'a` bunu söyler: "Dönen satırlar `icerik`'e bağlıdır, `sorgu`'ya değil."[^omur]

`'a` yazılmazsa derleyici sonucun hangi girdiye bağlı olduğunu bilemez, çünkü iki referans girdi vardır:

```rust,compile_fail,E0106
pub fn ara(sorgu: &str, icerik: &str) -> Vec<&str> {
    icerik
        .lines()
        .filter(|satir| satir.contains(sorgu))
        .collect()
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0106]: missing lifetime specifier
  |
1 | pub fn ara(sorgu: &str, icerik: &str) -> Vec<&str> {
  |                   ----          ----         ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but the signature does not say whether it is borrowed from `sorgu` or `icerik`
```

Ömür yazıldıktan sonra derleyici, dönen dilimlerin `icerik`'ten uzun yaşamasına izin vermez:

```rust,compile_fail,E0597
# pub fn ara<'a>(sorgu: &str, icerik: &'a str) -> Vec<&'a str> {
#     icerik.lines().filter(|satir| satir.contains(sorgu)).collect()
# }
fn main() {
    let sonuc;
    {
        let icerik = String::from("Ben bir hiçim\nSen kimsin");
        sonuc = ara("hiç", &icerik);
    } // icerik burada düşürülür
    println!("{sonuc:?}");
}
```

Derleyicinin yanıtı:

```text
error[E0597]: `icerik` does not live long enough
```

Aynı düzende `sorgu`'nun erken düşmesi ise sorun değildir:

```rust
# pub fn ara<'a>(sorgu: &str, icerik: &'a str) -> Vec<&'a str> {
#     icerik.lines().filter(|satir| satir.contains(sorgu)).collect()
# }
fn main() {
    let icerik = String::from("Ben bir hiçim\nSen kimsin");
    let sonuc;
    {
        let sorgu = String::from("hiç");
        sonuc = ara(&sorgu, &icerik);
    } // sorgu düşürüldü, sonuç hâlâ geçerli
    println!("{sonuc:?}");
}
```

Çıktı:

```text
["Ben bir hiçim"]
```

Derleyici `E0106`'nın altında iki parametreye de `'a` eklemeyi önerir; imzaya bakarak hangisinin gerektiğini bilemez. Kod öyle de derlenir, ama gereksiz bir kısıt gelir: Sonuç `sorgu`'ya da bağlanır ve yukarıdaki program `` `sorgu` does not live long enough `` hatasıyla reddedilir. Ömrü yalnızca gerçekten bağlı olan girdiye verin.

## Harf duyarsız arama

Büyük-küçük harf ayrımını kaldırmak için hem sorguyu hem her satırı küçük harfe çevirip öyle karşılaştırıyoruz:

```rust
pub fn ara_harf_duyarsiz<'a>(sorgu: &str, icerik: &'a str) -> Vec<&'a str> {
    let sorgu = sorgu.to_lowercase(); // yeni bir String
    icerik
        .lines()
        .filter(|satir| satir.to_lowercase().contains(&sorgu))
        .collect()
}

fn main() {
    let icerik = "Ben bir hiçim\nSen kimsin\nHiç misin";
    println!("{:?}", ara_harf_duyarsiz("hiç", icerik));
}
```

Çıktı:

```text
["Ben bir hiçim", "Hiç misin"]
```

`to_lowercase` veriyi yerinde değiştirmez, yeni bir `String` üretir. Bu yüzden `sorgu` adı artık `&str` değil, sahipli bir `String`'i gösterir ve `contains`'e `&sorgu` olarak verilir. Dönen satırlar yine *özgün* metnin dilimleridir; küçük harfli kopyalar yalnızca karşılaştırma içindir.

Yöntem Türkçe için tam doğru değildir. Unicode'un dilden bağımsız kurallarına göre `I` harfinin küçüğü `ı` değil `i`'dir; `İ` ise `i` ile birleşen bir nokta işaretine dönüşür ve ortaya çıkan metin düz "hiç" ile eşleşmez:[^kucuk]

```rust
fn main() {
    println!("{:?}", "I".to_lowercase());
    println!("{:?}", "HİÇ".to_lowercase());
    println!("{}", "HİÇ".to_lowercase().contains("hiç"));
}
```

Çıktı:

```text
"i"
"hi\u{307}ç"
false
```

Dolayısıyla `IGNORE_CASE=1` ile `HİÇ` aramak bu şiirde sonuç vermez. Dile duyarlı dönüşüm standart kütüphanede yoktur; gerekiyorsa bunun için yazılmış bir kütüphane kullanılır.

## İşi yapan fonksiyon: `run`

Görevli fişi aldı. Önce kitabı getirmeye gider; kitap yoksa iş orada biter. Kitap varsa satırları tarar ve uyanları okur. `run` bu iki adımdır:

```rust
use std::error::Error;
use std::fs;
# pub struct Config {
#     pub sorgu: String,
#     pub dosya_yolu: String,
#     pub harf_duyarsiz: bool,
# }
# pub fn ara<'a>(sorgu: &str, icerik: &'a str) -> Vec<&'a str> {
#     icerik.lines().filter(|satir| satir.contains(sorgu)).collect()
# }
# pub fn ara_harf_duyarsiz<'a>(sorgu: &str, icerik: &'a str) -> Vec<&'a str> {
#     let sorgu = sorgu.to_lowercase();
#     icerik.lines().filter(|satir| satir.to_lowercase().contains(&sorgu)).collect()
# }

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let icerik = fs::read_to_string(&config.dosya_yolu)?; // dosya yoksa Err ile dön

    let satirlar = if config.harf_duyarsiz {
        ara_harf_duyarsiz(&config.sorgu, &icerik)
    } else {
        ara(&config.sorgu, &icerik)
    };

    for satir in satirlar {
        println!("{satir}"); // sonuçlar stdout'a gider
    }
    Ok(())
}

fn main() {
    let config = Config {
        sorgu: String::from("hiç"),
        dosya_yolu: String::from("olmayan.txt"),
        harf_duyarsiz: false,
    };
    match run(config) {
        Ok(()) => println!("tamam"),
        Err(e) => println!("hata: {e}"),
    }
}
```

Çıktı:

```text
hata: No such file or directory (os error 2)
```

(Mesaj metni işletim sisteminden gelir; yukarıdaki Linux'taki karşılığıdır.) Bir dosyanın rafta olup olmadığını önceden bilemezsiniz; "olmadı" haberini çağırana iletmenin yolu `Result`'tır.

`Box<dyn Error>`, "`Error` trait'ini uygulayan herhangi bir değer" demektir. `run` ileride farklı hata türleri üretebilir: dosya okuma hatası, sayı çözümleme hatası ve başkaları. Hepsini tek bir enum'da toplamak yerine kutuya koyarız; `?` operatörü, hatayı `From` trait'i aracılığıyla bu kutuya kendiliğinden çevirir.[^box-error] Bunun bedeli bir heap tahsisi ve dinamik gönderimdir; bir uygulama için makuldür. Kütüphaneler ise genellikle kendi hata enum'unu verir, çünkü çağıranın hatanın türüne göre davranabilmesi gerekir. Bu yaklaşımı [8.1 Hata Yönetimi: Derinleşme](../temel/08-error-handling-derinlesme.md) sayfasında bulabilirsiniz.

## Kapı görevlisi: `main`

Kütüphane tamamlandı. `main`'e üç iş kalıyor: argümanları vermek, hatayı göstermek ve çıkış kodunu belirlemek.

```rust,no_run
use std::env;
use std::process;

use minigrep::{Config, run};
# mod minigrep {
#     use std::error::Error;
#     pub struct Config;
#     impl Config {
#         pub fn build(_args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
#             Ok(Config)
#         }
#     }
#     pub fn run(_config: Config) -> Result<(), Box<dyn Error>> {
#         Ok(())
#     }
# }

fn main() {
    let config = Config::build(env::args()).unwrap_or_else(|hata| {
        eprintln!("Argüman hatası: {hata}");
        eprintln!("Kullanım: minigrep <sorgu> <dosya_yolu>   (büyük/küçük harf: IGNORE_CASE=1)");
        process::exit(1);
    });

    if let Err(e) = run(config) {
        eprintln!("Uygulama hatası: {e}");
        process::exit(1);
    }
}
```

`unwrap_or_else`, sonuç `Ok` ise içindeki değeri verir; `Err` ise kendisine verilen küçük fonksiyonu hatayla çağırır. `run`'ın başarı değeri `()` olduğu için orada alınacak bir şey yoktur; yalnızca `Err` durumuna `if let` ile bakarız.[^main]

## Sonuçlar stdout'a, hatalar stderr'e

Her programın iki çıkış kanalı vardır: standart çıktı (stdout) ve standart hata (stderr). `println!` birincisine, `eprintln!` ikincisine yazar. Ekranda ikisi yan yana görünür, ama kabuk onları ayrı ayrı yönlendirebilir. `>` yalnızca stdout'u dosyaya gönderir:[^stderr]

```console
$ cargo run -q -p minigrep -- hiç learn/minigrep/siir.txt > cikti.txt
$ cat cikti.txt
Ben bir hiçim, sen kimsin?
Sen de bir hiç misin?
O zaman ikimiz de hiçiz, sakın söyleme!
```

Dosya adı yanlış olduğunda hata yine ekranda görünür ve `cikti.txt` boş kalır:

```console
$ cargo run -q -p minigrep -- hiç olmayan.txt > cikti.txt
Uygulama hatası: No such file or directory (os error 2)
$ wc -c cikti.txt
0 cikti.txt
```

İkisi de stdout'a yazılsaydı hata mesajı sonuç dosyasının içine karışır, siz de bir sorun olduğunu ancak dosyayı açınca görürdünüz. Sonuçla tanı mesajını ayırmak Unix araçlarının ortak sözleşmesidir; minigrep'in çıktısı bu sayede boruyla başka bir programa da verilebilir.

## Çıkış kodu

Kabuk ve betikler "başarılı mı?" sorusunu çıkış koduyla yanıtlar: 0 başarı, sıfırdan farklı bir değer başarısızlıktır. Kabukta son komutun kodu `$?` ile okunur:

```console
$ cargo run -q -p minigrep -- hiç
Argüman hatası: dosya yolu verilmedi
Kullanım: minigrep <sorgu> <dosya_yolu>   (büyük/küçük harf: IGNORE_CASE=1)
$ echo $?
1
$ cargo run -q -p minigrep
Argüman hatası: sorgu verilmedi
Kullanım: minigrep <sorgu> <dosya_yolu>   (büyük/küçük harf: IGNORE_CASE=1)
$ cargo run -q -p minigrep -- NE learn/minigrep/siir.txt
$ echo $?
0
$ IGNORE_CASE=1 cargo run -q -p minigrep -- NE learn/minigrep/siir.txt
Ne sıkıcı olurdu bir şey olmak!
Ne bayağı, bir kurbağa gibi
```

Üçüncü komutta hiçbir satır uymadı, ama bu bir hata değildir; program 0 ile biter. `IGNORE_CASE` açıldığında aynı sorgu iki satır bulur.

`process::exit(1)`, panik mesajı ve yığın çözülmesi olmadan, seçtiğiniz kodla çıkmanızı sağlar. Ama programı *hemen* bitirir: Stack'teki değerlerin yıkıcıları çalışmaz.[^exit]

```rust
struct Gozcu;

impl Drop for Gozcu {
    fn drop(&mut self) {
        println!("temizlik yapıldı");
    }
}

fn main() {
    let _g = Gozcu;
    println!("çıkılıyor");
    std::process::exit(0);
}
```

Çıktı:

```text
çıkılıyor
```

Minigrep'te bu sorun değildir, çünkü o noktada temizlenmesi gereken bir şey yoktur. Yıkıcıların çalışması gerekiyorsa `main`'den bir `ExitCode` (ya da `Result`) döndürün; standart kütüphane belgesi de bunu önerir:

```rust,no_run
use std::process::ExitCode;

fn main() -> ExitCode {
    match calistir() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Uygulama hatası: {e}");
            ExitCode::FAILURE
        }
    }
}
# fn calistir() -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
```

## Testler

Kütüphanenin üç birim testi vardır: harf duyarlı arama, harf duyarsız arama ve eksik argüman. `build` iterator aldığı için sonuncusunda komut satırına hiç dokunmadan sahte argümanlar verebiliyoruz:[^testler]

```rust
# pub struct Config {
#     pub sorgu: String,
#     pub dosya_yolu: String,
#     pub harf_duyarsiz: bool,
# }
# impl Config {
#     pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
#         args.next();
#         let Some(sorgu) = args.next() else { return Err("sorgu verilmedi") };
#         let Some(dosya_yolu) = args.next() else { return Err("dosya yolu verilmedi") };
#         Ok(Config { sorgu, dosya_yolu, harf_duyarsiz: false })
#     }
# }
# pub fn ara<'a>(sorgu: &str, icerik: &'a str) -> Vec<&'a str> {
#     icerik.lines().filter(|satir| satir.contains(sorgu)).collect()
# }
# pub fn ara_harf_duyarsiz<'a>(sorgu: &str, icerik: &'a str) -> Vec<&'a str> {
#     let sorgu = sorgu.to_lowercase();
#     icerik.lines().filter(|satir| satir.to_lowercase().contains(&sorgu)).collect()
# }
#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn harf_duyarli_tek_sonuc() {
        let sorgu = "hiç";
        let icerik = "Ben bir hiçim\nSen kimsin\nHiç misin";
        assert_eq!(vec!["Ben bir hiçim"], ara(sorgu, icerik));
    }

    #[test]
    fn harf_duyarsiz_iki_sonuc() {
        let sorgu = "hiç";
        let icerik = "Ben bir hiçim\nSen kimsin\nHiç misin";
        assert_eq!(vec!["Ben bir hiçim", "Hiç misin"], ara_harf_duyarsiz(sorgu, icerik));
    }

    #[test]
    fn eksik_arguman_hata_doner() {
        // Program adı ve sorgu var, dosya yolu yok.
        let args = vec![String::from("prog"), String::from("sorgu")].into_iter();
        assert!(Config::build(args).is_err());
    }
}
# fn main() {}
```

Testlerin tamamını çalıştırabilir ya da ada göre süzebilirsiniz:

```console
$ cargo test -p minigrep
running 3 tests
test testler::harf_duyarsiz_iki_sonuc ... ok
test testler::eksik_arguman_hata_doner ... ok
test testler::harf_duyarli_tek_sonuc ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
$ cargo test -p minigrep duyarsiz
running 1 test
test testler::harf_duyarsiz_iki_sonuc ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s
```

(Cargo, ikili hedef ve belge testleri için de boş birer bölüm yazdırır; burada kısaltıldı. Testlerin sırası çalıştırmadan çalıştırmaya değişebilir.)

## Sık yapılan hatalar

| Ne yapılıyor? | Ne olur? | Doğrusu |
| --- | --- | --- |
| Depo kökünden `cargo run -p minigrep -- hiç siir.txt` yazmak | `Uygulama hatası: No such file or directory (os error 2)` | Göreli yol, komutun çalıştırıldığı dizine göre çözülür. Kökten `learn/minigrep/siir.txt` yazın; çıplak `siir.txt` yalnızca `learn/minigrep` içinden çalışır. |
| Arama mantığını `main.rs`'e yazmak | Entegrasyon testine içe aktarılamaz | `Config::build`, `run` ve `ara` kütüphanede kalsın. |
| Hataları `println!` ile yazmak | Yönlendirmede sonuçla karışır | `eprintln!` kullanın. |
| Eksik argümanda `panic!` etmek | Kullanıcı yardım yerine çökme raporu görür | `Err` döndürün; kararı `main` versin. |
| `'a`'yı `sorgu`'ya da vermek | Sonuç gereksiz yere `sorgu`'ya bağlanır (`E0597`) | Ömrü yalnızca `icerik`'e verin. |

## Hata kodu başvurusu

| Kod | Ne zaman görülür? | Ne yapmalı? |
| --- | --- | --- |
| `E0106` | Dönüş türünde referans var, ama hangi girdiye bağlı olduğu belirtilmemişse | Ömrü, sonucun gerçekten baktığı girdiye yazın. |
| `E0597` | Dönen dilim, gösterdiği metinden uzun yaşarsa | Metni daha geniş bir kapsamda tutun ya da sahipli `String` döndürün. |

```console
rustc --explain E0106
```

## Perde arkası: Derleyici ne yapıyor?

> [!NOTE]
> Bu başlık ilk okuyuşta atlanabilir; programı yazmak için gerekli değildir.

**Argümanlar sahipli gelir.** `env::args` bir `Args` iterator'ı döndürür; her `next` çağrısı işletim sisteminden alınmış argümanı sahipli bir `String`'e çevirip verir. Çevirme, geçersiz Unicode'da panikleyen satırla yapılır.[^args-kod] Değer iterator'dan çıktıktan sonra `Config`'e taşınır; araya ikinci bir tahsis girmez.

**`?` ve `Box<dyn Error>`.** `?`, hata durumunda değeri `From::from` ile fonksiyonun hata türüne çevirip geri döner.[^soru] Standart kütüphanedeki `From<E> for Box<dyn Error + 'a>` genel impl'i, `Error` uygulayan her türü kutuya koyar; `io::Error`'ın tek satırda `Box<dyn Error>` olmasını sağlayan budur.[^from-kod]

**Ömürler yalnızca derleme zamanında vardır.** `ara<'a>` imzasındaki ilişki ödünç denetleyicisi tarafından denetlenir. Kod üretimine gelindiğinde ömürler silinir (region erasure);[^silme] çalışan programda `&'a str` yalnızca bir işaretçi ve bir uzunluktur, denetim için ek bir iş yapılmaz.

**Tasarımın özeti.** Çözümleme (`Config::build`) ve iş (`run`) kütüphanededir ve doğrudan sınanabilir. `main` yalnızca dış dünyayla konuşur. Sonuç ile hata ayrı kanallardan, başarı ile başarısızlık çıkış koduyla bildirilir. Bu iskelet, daha büyük bir komut satırı aracında da aynen işe yarar.

## Özet

- `Config::build` argümanları bir iterator'dan alır, `String`'leri klonlamadan taşır; eksik argümanda paniklemez, `Err` döner.
- `ara<'a>` sonuçları `icerik`'e bağlar. Harf duyarsız arama `to_lowercase` kullanır ve Türkçe `İ`/`ı` harflerinde beklediğiniz sonucu vermeyebilir.
- `run` dosyayı okur ve `Box<dyn Error>` ile her türlü hatayı tek imzada taşır. Sonuçlar stdout'a, hatalar stderr'e gider.
- Mantık `lib.rs`'te, vitrin `main.rs`'tedir. Çıkış kodu kabuğa başarıyı ya da başarısızlığı bildirir.

Sıradaki bölüm [12. Closure ve Iterator'lar](12-closures-iterators.md), burada hızla geçtiğimiz `lines().filter(..).collect()` zincirini, `|satir| ...` yazımını ve `unwrap_or_else`'in istediği `FnOnce` sınırını ayrıntılı olarak anlatıyor.

## Kaynaklar

[^entegrasyon]: The Rust Programming Language, [Integration Tests](https://doc.rust-lang.org/book/ch11-03-test-organization.html#integration-tests).
[^ayrim]: The Rust Programming Language, [Splitting Code into a Library Crate](https://doc.rust-lang.org/book/ch12-03-improving-error-handling-and-modularity.html#splitting-code-into-a-library-crate) ve [Refactoring to Improve Modularity and Error Handling](https://doc.rust-lang.org/book/ch12-03-improving-error-handling-and-modularity.html#refactoring-to-improve-modularity-and-error-handling).
[^config]: The Rust Programming Language, [12.3 Refactoring to Improve Modularity and Error Handling](https://doc.rust-lang.org/book/ch12-03-improving-error-handling-and-modularity.html).
[^args]: The Rust Programming Language, [Reading the Argument Values](https://doc.rust-lang.org/book/ch12-01-accepting-command-line-arguments.html#reading-the-argument-values); std, [`env::args`](https://doc.rust-lang.org/std/env/fn.args.html).
[^result]: The Rust Programming Language, [Fixing the Error Handling](https://doc.rust-lang.org/book/ch12-03-improving-error-handling-and-modularity.html#fixing-the-error-handling).
[^klon]: The Rust Programming Language, [Removing a `clone` Using an Iterator](https://doc.rust-lang.org/book/ch13-03-improving-our-io-project.html#removing-a-clone-using-an-iterator) ve [The Trade-Offs of Using `clone`](https://doc.rust-lang.org/book/ch12-03-improving-error-handling-and-modularity.html#the-trade-offs-of-using-clone).
[^args-os]: The Rust Programming Language, [The `args` Function and Invalid Unicode](https://doc.rust-lang.org/book/ch12-01-accepting-command-line-arguments.html#the-args-function-and-invalid-unicode); std, [`env::args_os`](https://doc.rust-lang.org/std/env/fn.args_os.html).
[^ortam]: The Rust Programming Language, [12.5 Working with Environment Variables](https://doc.rust-lang.org/book/ch12-05-working-with-environment-variables.html); std, [`env::var`](https://doc.rust-lang.org/std/env/fn.var.html).
[^tdd]: The Rust Programming Language, [Writing a Failing Test](https://doc.rust-lang.org/book/ch12-04-testing-the-librarys-functionality.html#writing-a-failing-test).
[^lines]: The Rust Programming Language, [Clarifying Code with Iterator Adapters](https://doc.rust-lang.org/book/ch13-03-improving-our-io-project.html#clarifying-code-with-iterator-adapters); std, [`str::lines`](https://doc.rust-lang.org/std/primitive.str.html#method.lines) ve [`str::contains`](https://doc.rust-lang.org/std/primitive.str.html#method.contains).
[^omur]: The Rust Programming Language, [Writing a Failing Test](https://doc.rust-lang.org/book/ch12-04-testing-the-librarys-functionality.html#writing-a-failing-test) ve [Relationships](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html#relationships); The Rust Reference, [Lifetime elision in functions](https://doc.rust-lang.org/reference/lifetime-elision.html#lifetime-elision-in-functions).
[^kucuk]: The Rust Programming Language, [Implementing the `search_case_insensitive` Function](https://doc.rust-lang.org/book/ch12-05-working-with-environment-variables.html#implementing-the-search_case_insensitive-function); std, [`str::to_lowercase`](https://doc.rust-lang.org/std/primitive.str.html#method.to_lowercase) ve [`char::to_lowercase`](https://doc.rust-lang.org/std/primitive.char.html#method.to_lowercase).
[^box-error]: The Rust Programming Language, [Extracting Logic from `main`](https://doc.rust-lang.org/book/ch12-03-improving-error-handling-and-modularity.html#extracting-logic-from-main); std, [`Error`](https://doc.rust-lang.org/std/error/trait.Error.html).
[^main]: The Rust Programming Language, [Fixing the Error Handling](https://doc.rust-lang.org/book/ch12-03-improving-error-handling-and-modularity.html#fixing-the-error-handling); std, [`Result::unwrap_or_else`](https://doc.rust-lang.org/std/result/enum.Result.html#method.unwrap_or_else).
[^stderr]: The Rust Programming Language, [Printing Errors to Standard Error](https://doc.rust-lang.org/book/ch12-06-writing-to-stderr-instead-of-stdout.html#printing-errors-to-standard-error); std, [`eprintln!`](https://doc.rust-lang.org/std/macro.eprintln.html).
[^exit]: std, [`process::exit`](https://doc.rust-lang.org/std/process/fn.exit.html) ve [`process::ExitCode`](https://doc.rust-lang.org/std/process/struct.ExitCode.html).
[^testler]: The Rust Programming Language, [Running a Subset of Tests by Name](https://doc.rust-lang.org/book/ch11-02-running-tests.html#running-a-subset-of-tests-by-name) ve [Unit Tests](https://doc.rust-lang.org/book/ch11-03-test-organization.html#unit-tests).
[^args-kod]: std 1.99.0 kaynak kodu, [`env.rs`, `impl Iterator for Args`](https://github.com/rust-lang/rust/blob/1.99.0/library/std/src/env.rs#L874).
[^soru]: The Rust Reference, [The try propagation expression](https://doc.rust-lang.org/reference/expressions/operator-expr.html#the-try-propagation-expression).
[^from-kod]: std 1.99.0 kaynak kodu, [`convert.rs`, `From<E> for Box<dyn Error + 'a>`](https://github.com/rust-lang/rust/blob/1.99.0/library/alloc/src/boxed/convert.rs#L493).
[^silme]: rustc 1.99.0 kaynak kodu, [`erase_regions.rs`, `erase_and_anonymize_regions`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_middle/src/ty/erase_regions.rs#L22).
