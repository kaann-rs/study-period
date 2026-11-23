# 6. Modüller

Şimdiye kadarki örneklerin hepsi tek bir dosyaya sığdı. Gerçek programlar ise kısa sürede yüzlerce fonksiyona ve türe ulaşır. Hepsi yan yana durduğunda iki sorun çıkar: Aradığınızı bulamazsınız ve her parça öteki her parçaya dokunabildiği için birini değiştirmek hiç ummadığınız bir yeri bozabilir.

Rust bu iki soruna tek bir araçla cevap verir: *modül* (module). Modüller kodu adlandırılmış bölümlere ayırır ve bölümlerin dışarıya neyi göstereceğini belirler.[^moduller] Her bölümün kapısı kapalı gelir; açmak istediğiniz şeyleri `pub` yazarak tek tek seçersiniz.

Bu bölümde önce paket, crate ve modül kavramlarını ayıracağız. Sonra görünürlük kurallarını, öğelere adres vermeyi (yol), uzun adresleri `use` ile kısaltmayı, `pub use` ile derli toplu bir dış yüz sunmayı ve modülleri dosyalara bölmeyi göreceğiz.

> [!NOTE]
> Bu bölümün notları ve çalıştırılabilir kodu `learn/modules` dizinindedir. Depo kökünden `cargo run -p modules` komutuyla çalıştırabilirsiniz.

## Günlük hayattan bir benzetme

Bir evde odalar vardır: mutfak, yatak odası, oyun odası. Her odanın kapısı kapalıdır; yalnızca üstünde "girebilirsin" etiketi bulunan eşyaya dışarıdan ulaşılır. "Mutfaktaki bardak" dediğinizde ise hangi odada neyi arayacağınızı söylemiş olursunuz. Modül odadır, `pub` etikettir, "mutfaktaki bardak" da yoldur.

Bölüm boyunca bu evi bir lokantaya çevireceğiz, çünkü orada kapıların nedeni daha açıktır. Salon müşteriye açıktır, mutfağa yalnızca çalışanlar girer. Müşteri mutfağa girip tencereyi karıştıramaz; yemeğini salondan ister, salon mutfağa haber verir. Lokantanın öteki âdetleri de bölümün konularına karşılık gelir:

- **Kasa fişi.** Garson fişteki ürün adını düzeltebilir, ama tutarı yalnızca kasa yazar. Struct alanları da böyle tek tek açılır.
- **Adres tarifi.** Bir yeri "şehir, sokak, bina" diye de tarif edebilirsiniz, "bir üst kat, soldaki kapı" diye de. Yolların da iki türü vardır.
- **Telefon rehberi.** Arkadaşınızın numarasını her seferinde tuşlamak yerine rehbere "Ada" diye kaydedersiniz. `use` uzun bir adresi böyle kısaltır; numara yerinde durur.
- **Vitrin.** Tatlılar deponun dibindeki rafta durur, ama müşteri onları girişteki vitrinden ister. `pub use` bu vitrindir.

<details>
<summary>Bu bölümde geçen terimler</summary>

- **paket (package):** `Cargo.toml` dosyasıyla tanımlanan, bir ya da daha çok crate içeren proje.
- **`Cargo.toml`:** Paketin adını, sürümünü ve bağımlılıklarını tutan dosya.
- **crate:** Derleyicinin tek seferde derlediği bütün.
- **binary crate:** Çalıştırılabilir program; kökü çoğunlukla `src/main.rs`'dir.
- **library crate:** Başka kodların kullandığı kütüphane; kökü çoğunlukla `src/lib.rs`'dir.
- **crate kökü (crate root):** Derlemenin başladığı dosya ve modül ağacının tepesi.
- **modül (module):** Öğeleri gruplayan ve görünürlüğü belirleyen bölüm.
- **öğe (item):** Modülün içine konan her tanım: fonksiyon, struct, enum, sabit, başka bir modül.
- **modül ağacı:** Modüllerin kökten başlayan iç içe dizilişi.
- **yol (path):** Bir öğenin adresi. Parçaları `::` ile ayrılır: `lokanta::salon::masa_ayarla`.
- **mutlak yol (absolute path):** Crate kökünden başlayan adres: `crate::lokanta::ac`.
- **göreli yol (relative path):** Bulunulan modülden başlayan tarif: `salon::masa_ayarla`.
- **`crate::`, `self::`, `super::`:** "Kökten başla", "bu modülden başla" ve "bir üst modülden başla".
- **görünürlük (visibility), gizlilik (privacy):** Bir öğeyi kimin kullanabildiğini belirleyen kurallar.
- **private:** Yalnızca tanımlandığı modülden ve onun alt modüllerinden görülen. Varsayılan budur.
- **`pub`:** Modülün dışından da görülen.
- **`pub(crate)`:** Crate'in her yerinden görülen, crate'in dışından görülmeyen.
- **`use`:** Bir yola bulunulan kapsamda kısa ad veren bildirim.
- **`as`:** `use` ile getirilen ada başka bir ad takar.
- **yeniden dışa aktarma (re-export, `pub use`):** Başka modüldeki bir öğeyi kendi modülünüzün kapısından da sunmak.
- **API:** Bir modülün ya da crate'in dışarıya sunduğu, başkalarının kullandığı öğelerin tamamı.
- **yapıcı (constructor):** Yeni bir değer üreten fonksiyon; örneğin `Fis::yeni`.
- **getter (okuyucu metot):** Private bir alanı yalnızca okutan küçük metot.
- **kapsülleme (encapsulation):** İç ayrıntıyı saklayıp dışarıya yalnızca gerekeni açmak.
- **ad alanı (namespace):** Adların tutulduğu defter. Tür adları ile değer adları ayrı defterlerdedir.

</details>

## Paket, crate ve crate kökü

`cargo new` komutunu çalıştırdığınızda içinde `Cargo.toml` bulunan bir dizin oluşur. Bu dizin bir *pakettir*. Paket, birlikte geliştirilen bir ya da daha çok *crate*'i bir araya getirir; crate ise derleyicinin tek seferde derlediği birimdir.[^paket]

Crate iki türden biridir. *Binary crate* çalıştırılabilir bir programdır ve bir `main` fonksiyonu vardır. *Library crate* tek başına çalışmaz; başka crate'lere fonksiyon ve tür sunar. Cargo, aksi `Cargo.toml`'da yazmadıkça, hangi dosyanın hangi crate'in kökü olduğunu dosyanın yerinden anlar:[^yerlesim]

| Dosya | Ne anlama gelir? |
| --- | --- |
| `src/main.rs` | Paketle aynı adı taşıyan binary crate'in kökü |
| `src/lib.rs` | Paketle aynı adı taşıyan library crate'in kökü |
| `src/bin/ad.rs` | `ad` adında ek bir binary crate'in kökü |

Bir pakette en az bir crate bulunmalıdır. İstediğiniz kadar binary crate olabilir, ama library crate en çok bir tanedir. Bu bölümün `learn/modules` paketinde yalnızca `src/main.rs` vardır; paket tek bir binary crate'ten oluşur ve kökü bu dosyadır.

> [!NOTE]
> İçinde `Cargo.toml` bulunan her dizin paket değildir. Bu deponun kökündeki `Cargo.toml` dosyasında `[package]` bölümü yoktur; o dosya bir çalışma alanını (workspace) tanımlar.[^calisma-alani] Cargo'nun bu yönünü [13. bölümde](../orta/13-cargo-and-crates.md) göreceğiz.

## Modül tanımlamak

Modül `mod` anahtar sözcüğüyle tanımlanır. İçine fonksiyon, struct, enum ve başka modüller konabilir:[^modul-tanim]

```rust
mod lokanta {
    pub mod salon {
        pub fn masa_ayarla() {
            println!("salon: masa ayarlandı");
            super::mutfak::hazirla();
        }
    }

    mod mutfak {
        pub fn hazirla() {
            println!("mutfak: hazır");
        }
    }

    pub fn ac() {
        self::salon::masa_ayarla();
        mutfak::hazirla();
    }

    pub(crate) fn ic_rapor() {
        println!("lokanta: iç rapor (yalnız crate içi)");
    }
}

fn main() {
    crate::lokanta::ac();
    lokanta::ic_rapor();
}
```

Çıktı:

```text
salon: masa ayarlandı
mutfak: hazır
mutfak: hazır
lokanta: iç rapor (yalnız crate içi)
```

Modüller iç içe geçtiği için bir ağaç oluşur. Ağacın tepesinde, kök dosyanın kendisini temsil eden `crate` adlı örtük modül bulunur:

```text
crate
├── lokanta
│   ├── salon
│   │   └── masa_ayarla
│   ├── mutfak
│   │   └── hazirla
│   ├── ac
│   └── ic_rapor
└── main
```

Dizin ağacına benzemesi tesadüf değildir: Dosyaya yoluyla ulaştığınız gibi öğeye de yoluyla ulaşırsınız. Koddaki `pub`, `self::`, `super::` ve `crate::` parçalarının her birini aşağıdaki başlıklarda tek tek ele alacağız.

## Gizlilik: Kapılar kapalı gelir

Bir öğenin başına hiçbir şey yazmazsanız öğe *private*'tır. `pub` yazmak onu dışarıya açar:

```rust
mod lokanta {
    pub fn kapidan_gir() {
        println!("kapıdan girildi");
    }

    fn kasayi_say() {
        println!("kasa sayıldı");
    }

    pub fn gun_sonu() {
        kasayi_say(); // aynı modülün içinden çağrılabilir
    }
}

fn main() {
    lokanta::kapidan_gir();
    lokanta::gun_sonu();
}
```

Çıktı:

```text
kapıdan girildi
kasa sayıldı
```

Kasayı dışarıdan saymaya kalkarsanız program derlenmez:

```rust,compile_fail,E0603
mod lokanta {
    fn kasayi_say() {
        println!("kasa sayıldı");
    }
}

fn main() {
    lokanta::kasayi_say();
}
```

Derleyicinin yanıtı:

```text
error[E0603]: function `kasayi_say` is private
  |
8 |     lokanta::kasayi_say();
  |              ^^^^^^^^^^ private function
  |
note: the function `kasayi_say` is defined here
  |
2 |     fn kasayi_say() {
  |     ^^^^^^^^^^^^^^^
```

### Private öğeyi kim görür?

Kural tek cümledir: Private öğe, **tanımlandığı modülden ve o modülün bütün alt modüllerinden** görülür.[^gorunurluk] Alt modül üst modülün içini görür; üst modül alt modülün içini göremez.

Yukarıdaki ilk `lokanta` örneğinin tamamı bu kuralla açıklanır:

- `main`, `pub` olmayan `lokanta`'yı kullanabilir, çünkü ikisi de aynı yerde, crate kökünde tanımlıdır.
- `salon`, private olan `mutfak`'a ulaşabilir: `mutfak` `lokanta`'nın içinde tanımlıdır, `salon` da `lokanta`'nın alt modülüdür.
- `main`, `mutfak`'a ulaşamaz, çünkü `lokanta`'nın içinde değildir:

```rust,compile_fail,E0603
mod lokanta {
    mod mutfak {
        pub fn hazirla() {}
    }
}

fn main() {
    lokanta::mutfak::hazirla();
}
```

Derleyicinin yanıtı:

```text
error[E0603]: module `mutfak` is private
  |
8 |     lokanta::mutfak::hazirla();
  |              ^^^^^^  ------- function `hazirla` is not publicly re-exported
  |              |
  |              private module
```

`hazirla` fonksiyonunun `pub` olması yetmez. Bir öğeye dışarıdan ulaşabilmek için yol üstündeki modüllerin hepsine ulaşabilmek gerekir.

### `pub mod` içindekileri açmaz

Tersi de doğrudur: Bir modülü açmak, içindekileri açmaz. `lokanta`, kendi alt modülü `mutfak`'ı görür, ama onun private fonksiyonunu göremez:

```rust,compile_fail,E0603
mod lokanta {
    pub mod mutfak {
        fn hazirla() {}
    }

    pub fn ac() {
        mutfak::hazirla();
    }
}

fn main() {
    lokanta::ac();
}
```

Derleyicinin yanıtı:

```text
error[E0603]: function `hazirla` is private
```

İlk örnekte `mutfak` private, içindeki `hazirla` ise `pub` idi; böylece `lokanta` ve onun alt modülleri `hazirla`'yı çağırabiliyor, lokantanın dışındakiler çağıramıyordu.

### `pub(crate)` ve `pub(super)`

Çoğu zaman "ya herkes ya hiç kimse" fazla kabadır. `pub`'dan sonra parantez açarak görünürlüğü bir üst modülle sınırlayabilirsiniz:[^sinirli-pub]

| Yazım | Öğeyi kim görür? |
| --- | --- |
| hiçbir şey ya da `pub(self)` | Tanımlandığı modül ve alt modülleri |
| `pub(super)` | Bir üst modül ve onun alt modülleri |
| `pub(in crate::lokanta)` | Yolu verilen üst modül ve onun alt modülleri |
| `pub(crate)` | Crate'in her yeri |
| `pub` | Yoluna ulaşabilen her yer; öteki crate'ler de buna dâhildir |

```rust
mod lokanta {
    pub mod mutfak {
        pub(super) fn yalniz_lokanta() {
            println!("mutfak: günün tarifi");
        }
    }

    pub fn tarif_ver() {
        mutfak::yalniz_lokanta();
    }

    pub(crate) fn ic_rapor() {
        println!("lokanta: iç rapor (yalnız crate içi)");
    }
}

fn main() {
    lokanta::tarif_ver();
    lokanta::ic_rapor();
}
```

Çıktı:

```text
mutfak: günün tarifi
lokanta: iç rapor (yalnız crate içi)
```

`main` aynı crate'te olduğu için `ic_rapor`'u çağırabilir. Bu crate'i kütüphane olarak kullanan başka bir crate ise çağıramaz: `pub(crate)`, "lokantanın bütün çalışanlarına açık, müşteriye kapalı" demektir. Aynı paketteki `src/main.rs` ile `src/lib.rs` de iki ayrı crate'tir; binary, kütüphanenin `pub(crate)` öğelerini göremez.[^iki-crate]

`pub(super)` öğesi yalnızca `lokanta`'ya açıktır; kökten çağrılamaz:

```rust,compile_fail,E0603
mod lokanta {
    pub mod mutfak {
        pub(super) fn yalniz_lokanta() {}
    }
}

fn main() {
    lokanta::mutfak::yalniz_lokanta();
}
```

Derleyicinin yanıtı:

```text
error[E0603]: function `yalniz_lokanta` is private
```

### Neden varsayılan private?

Her şey kendiliğinden açık olsaydı modülün bütün iç ayrıntıları onun API'si sayılırdı. Başka kodlar mutfağın içine bağlanır, siz rafın yerini değiştirdiğinizde uzaktaki kodlar bozulurdu; üstelik hangi ayrıntının güvenle değiştirilebileceğini bilemezdiniz.

Varsayılan private olunca durum tersine döner. Dışarıya açtığınız her şey bilinçli bir seçimdir, geri kalanını gönül rahatlığıyla değiştirebilirsiniz. Sınırı da derleyici korur.[^neden-private]

> [!TIP]
> Alt modülün üst modülün içini görebilmesi testlerde işe yarar. Birim testleri `mod tests` adlı bir alt modüle yazılır ve `use super::*;` ile üst modülün private fonksiyonlarını da sınayabilir.[^testler] Ayrıntıları [10. bölümde](../orta/10-writing-tests.md) göreceğiz.

## Yollar: `crate`, `self` ve `super`

Bir öğenin adresine *yol* denir ve iki türü vardır.[^yollar] *Mutlak yol* her yerde aynıdır ve crate kökünden başlar. *Göreli yol* bulunduğunuz modülden başlar:

| Başlangıç | Nereden başlar? | Örnek |
| --- | --- | --- |
| `crate::` | Crate kökünden | `crate::lokanta::ac()` |
| `self::` | Bulunulan modülden | `self::salon::masa_ayarla()` |
| `super::` | Bir üst modülden | `super::mutfak::hazirla()` |
| düz ad | Bulunulan kapsamdan | `salon::masa_ayarla()` |

Aşağıdaki `duzenle` fonksiyonu aynı lokantaya dört farklı tarifle ulaşıyor:

```rust
mod lokanta {
    pub mod salon {
        pub fn masa_ayarla() {
            println!("salon: masa ayarlandı");
        }

        pub mod teras {
            pub fn duzenle() {
                super::masa_ayarla(); // bir üst: salon
                super::super::toparla(); // iki üst: lokanta
                crate::lokanta::toparla(); // kökten
                self::serin(); // bu modül; yalnızca serin() da olurdu
            }

            fn serin() {
                println!("teras: serin");
            }
        }
    }

    fn toparla() {
        println!("lokanta: toparlandı");
    }
}

fn main() {
    crate::lokanta::salon::teras::duzenle(); // mutlak yol
    lokanta::salon::masa_ayarla(); // göreli yol; main kökte durduğu için aynı yere varır
}
```

Çıktı:

```text
salon: masa ayarlandı
lokanta: toparlandı
lokanta: toparlandı
teras: serin
salon: masa ayarlandı
```

Düz ad yazdığınızda arama bulunduğunuz kapsamdan başlar. Kökte `ac` diye bir fonksiyon yoksa yalnızca adını yazmak yetmez:

```rust,compile_fail,E0425
mod lokanta {
    pub fn ac() {}
}

fn main() {
    ac();
}
```

Derleyicinin yanıtı:

```text
error[E0425]: cannot find function `ac` in this scope
  |
6 |     ac();
  |     ^^ not found in this scope
  |
help: consider importing this function
  |
1 + use crate::lokanta::ac;
```

`lokanta::ac()` yazmanız ya da derleyicinin önerdiği `use` satırını eklemeniz gerekir. Alt modüller de üst modülün adlarını kendiliğinden devralmaz; `salon` içinden kardeş modüle düz adla ulaşmak isterseniz şu yanıtı alırsınız:

```rust,compile_fail,E0433
mod lokanta {
    pub mod salon {
        pub fn masa_ayarla() {
            mutfak::hazirla();
        }
    }

    mod mutfak {
        pub fn hazirla() {}
    }
}

fn main() {
    lokanta::salon::masa_ayarla();
}
```

Derleyicinin yanıtı:

```text
error[E0433]: cannot find module or crate `mutfak` in this scope
  |
4 |             mutfak::hazirla();
  |             ^^^^^^ use of unresolved module or unlinked crate `mutfak`
  |
  = help: you might be missing a crate named `mutfak`
```

Çözüm `super::mutfak::hazirla()` yazmaktır. Mesajda crate'ten söz edilmesinin nedeni şudur: Yolun ilk parçası bir dış crate'in adı da olabilir. Standart kütüphane `std` adında bir dış crate'tir ve `std::collections` gibi yolları her modülden yazabilirsiniz.[^dis-crate]

İki yazımdan hangisi daha iyi? Tanım ile kullanım genellikle ayrı ayrı taşındığı için Rust'ın resmî kitabı mutlak yolu önerir. Birlikte taşınacak iki komşu modül arasında ise `super` daha uygundur.[^super]

## Struct alanları ve kapsülleme

Struct'ı `pub` yapmak alanlarını açmaz; her alan ayrı ayrı açılır.[^pub-struct] Lokantanın fişinde ürün adı herkese açık, tutar kapalı olsun:

```rust
mod siparis {
    #[derive(Debug)]
    pub struct Fis {
        pub urun: String, // dışarıdan okunur ve yazılır
        tutar: u32,       // yalnızca bu modülden erişilir
    }

    impl Fis {
        pub fn yeni(urun: &str, tutar: u32) -> Fis {
            Fis { urun: urun.to_string(), tutar }
        }

        pub fn tutar(&self) -> u32 {
            self.tutar
        }
    }
}

fn main() {
    let mut f = siparis::Fis::yeni("çay", 15);
    println!("fiş: {} = {}", f.urun, f.tutar());

    f.urun = String::from("kahve");
    println!("fiş: {} = {}", f.urun, f.tutar());
}
```

Çıktı:

```text
fiş: çay = 15
fiş: kahve = 15
```

Private alanı olan bir struct'ı modülün dışından elle kuramazsınız. Bu yüzden modül bir *yapıcı* (`yeni`) sunar. Tutarı okutan `tutar()` metodu ise değeri gösterir ama değiştirtmez; böyle metotlara *getter* denir. İkisinin birlikte sağladığı korumanın adı *kapsüllemedir*. Tutar herkese açık olsaydı programın herhangi bir yeri onu 0 yapabilirdi.

Kuralı üç biçimde çiğneyebilirsiniz ve her birinin ayrı bir hata kodu vardır. Fişi dışarıdan kurmak `E0451` verir:

```rust,compile_fail,E0451
mod siparis {
    pub struct Fis {
        pub urun: String,
        tutar: u32,
    }
}

fn main() {
    let f = siparis::Fis { urun: String::from("çay"), tutar: 15 };
}
```

Derleyicinin yanıtı:

```text
error[E0451]: field `tutar` of struct `Fis` is private
  |
9 |     let f = siparis::Fis { urun: String::from("çay"), tutar: 15 };
  |                                                       ^^^^^ private field
```

Var olan bir fişin kapalı alanına dokunmak `E0616` verir; okumak da yazmak da yasaktır:

```rust,compile_fail,E0616
# mod siparis {
#     pub struct Fis { pub urun: String, tutar: u32 }
#     impl Fis {
#         pub fn yeni(urun: &str, tutar: u32) -> Fis { Fis { urun: urun.to_string(), tutar } }
#     }
# }
#
fn main() {
    let mut f = siparis::Fis::yeni("çay", 15);
    f.tutar = 0;
}
```

Derleyicinin yanıtı:

```text
error[E0616]: field `tutar` of struct `Fis` is private
```

Private bir metodu dışarıdan çağırmak ise `E0624` verir:

```rust,compile_fail,E0624
mod siparis {
    pub struct Fis {
        tutar: u32,
    }

    impl Fis {
        pub fn yeni(tutar: u32) -> Fis {
            Fis { tutar }
        }

        fn yariya_indir(&mut self) {
            self.tutar /= 2;
        }
    }
}

fn main() {
    let mut f = siparis::Fis::yeni(15);
    f.yariya_indir();
}
```

Derleyicinin yanıtı:

```text
error[E0624]: method `yariya_indir` is private
```

> [!NOTE]
> Gizliliğin sınırı tür değil, modüldür. `siparis` modülünün içindeki her fonksiyon, `Fis`'in metodu olmasa bile `tutar` alanına erişebilir. Bir alanı gerçekten korumak istiyorsanız türü kendi modülüne koyun.

### Enum varyantları açık gelir

Varsayılan private kuralının iki istisnası vardır: `pub` bir enum'un varyantları ve `pub` bir trait'in ilişkili öğeleri kendiliğinden açıktır.[^gorunurluk] Varyantları kapalı bir enum'un dışarıda işe yarayacak bir yanı kalmazdı:

```rust
mod menu {
    #[derive(Debug)]
    pub enum Baslangic {
        Corba,
        Salata,
    }
}

fn main() {
    println!("{:?} ya da {:?}", menu::Baslangic::Corba, menu::Baslangic::Salata);
}
```

Çıktı:

```text
Corba ya da Salata
```

## `use` ile kısa ad vermek

`lokanta::salon::masa_ayarla` yolunu her çağrıda baştan yazmak yorucudur. `use`, yola bulunduğunuz kapsamda kısa bir ad verir:[^use]

```rust
mod lokanta {
    pub mod salon {
        pub fn masa_ayarla() {
            println!("salon: masa ayarlandı");
        }
    }
}

use lokanta::salon;
use lokanta::salon::masa_ayarla as hazirla_masa;

fn main() {
    salon::masa_ayarla();
    hazirla_masa();
}
```

Çıktı:

```text
salon: masa ayarlandı
salon: masa ayarlandı
```

`use` hiçbir şeyi kopyalamaz ya da taşımaz; yalnızca bir ad tanımlar. `as` ile bu ada istediğinizi verebilirsiniz: `hazirla_masa` ile `masa_ayarla` aynı fonksiyonun iki adıdır.

Getirmediğiniz bir adı kullanamazsınız:

```rust,compile_fail,E0425
# mod lokanta { pub mod salon { pub fn masa_ayarla() {} pub fn masa_topla() {} } }
use lokanta::salon::masa_ayarla;

fn main() {
    masa_ayarla();
    masa_topla(); // bu ad için use yok
}
```

Derleyicinin yanıtı:

```text
error[E0425]: cannot find function `masa_topla` in this scope
```

`use` yalnızca yazıldığı kapsamda geçerlidir. Alt modül onu devralmaz; kendi `use` satırını yazar ya da üst modüldeki ada `super::salon` diye ulaşır.[^use-kapsam]

Alışılmış kullanım şöyledir: Fonksiyonlarda üst modül getirilir (`use lokanta::salon;`), böylece çağrı yerinde fonksiyonun başka bir modülden geldiği görünür. Struct, enum ve öteki türlerde ise tam yol getirilir (`use std::collections::HashMap;`).[^use-deyim]

### Aynı adlı iki öğe

Bir modülde aynı ad iki kez getirilemez:

```rust,compile_fail,E0252
use std::fmt::Result;
use std::io::Result;
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0252]: the name `Result` is defined multiple times
  |
1 | use std::fmt::Result;
  |     ---------------- previous import of the type `Result` here
2 | use std::io::Result;
  |     ^^^^^^^^^^^^^^^ `Result` reimported here
  |
  = note: `Result` must be defined only once in the type namespace of this module
help: you can use `as` to change the binding name of the import
```

Çözüm, birine `as` ile başka ad takmak ya da yalnızca üst modülleri getirmektir:[^as]

```rust
use std::fmt::Result;
use std::io::Result as IoResult;

fn bicimle() -> Result {
    Ok(())
}

fn oku() -> IoResult<()> {
    Ok(())
}
# fn main() { let _ = (bicimle(), oku()); }
```

### Birden çok adı tek satırda getirmek

Ortak başlayan yollar süslü parantezle birleştirilir; `self`, yolun o ana kadarki kısmını da getirir. `*` ise modüldeki erişilebilir bütün öğeleri getirir:[^ic-ice]

```rust
use std::collections::*;
use std::fmt::{self, Display};

struct Masa(u32);

impl Display for Masa {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "masa {}", self.0)
    }
}

fn main() {
    let mut dolu = BTreeSet::new();
    dolu.insert(3);
    println!("{} dolu mu? {}", Masa(3), dolu.contains(&3));
}
```

Çıktı:

```text
masa 3 dolu mu? true
```

`*` hangi adın nereden geldiğini gizlediği için ölçülü kullanılır; en yaygın yeri test modüllerindeki `use super::*;` satırıdır.

## Yeniden dışa aktarmak: `pub use`

Bir öğe, düzen gereği derin bir modülde durabilir. Kullananların o derinliği bilmesi gerekmez. `pub use`, öğeyi bulunduğunuz modülün kapısından da sunar:[^yeniden-acma]

```rust
mod derin {
    pub mod cok {
        pub mod ic {
            pub fn eylem() {
                println!("derin::cok::ic::eylem çalıştı");
            }
        }
    }
}

mod cephe {
    pub use crate::derin::cok::ic::eylem;

    pub fn demo() {
        eylem(); // use sayesinde kısa ad
    }
}

fn main() {
    cephe::demo();
    cephe::eylem();
    derin::cok::ic::eylem(); // uzun yol da çalışır
}
```

Çıktı:

```text
derin::cok::ic::eylem çalıştı
derin::cok::ic::eylem çalıştı
derin::cok::ic::eylem çalıştı
```

`pub` yazmasaydınız `eylem` adı yalnızca `cephe`'nin ve alt modüllerinin içinde geçerli olurdu, çünkü `use` bildirimleri de private gelir:

```rust,compile_fail,E0603
# mod derin { pub mod cok { pub mod ic { pub fn eylem() {} } } }
mod cephe {
    use crate::derin::cok::ic::eylem;
}

fn main() {
    cephe::eylem();
}
```

Derleyicinin yanıtı:

```text
error[E0603]: function import `eylem` is private
```

`pub use`'un asıl gücü, iç düzeni saklamaktır. `raf` modülü kapalı kalır, tatlı ise vitrinden istenir:

```rust
mod depo {
    mod raf {
        pub fn tatli() -> &'static str {
            "tatlı hazır"
        }
    }

    pub use self::raf::tatli;
}

mod vitrin {
    pub use crate::depo::tatli;
}

fn main() {
    println!("{}", vitrin::tatli());
    println!("{}", depo::tatli());
}
```

Çıktı:

```text
tatlı hazır
tatlı hazır
```

`depo::raf::tatli()` yazılsaydı `raf` private olduğu için `E0603` alınırdı. Yarın tatlıyı başka bir rafa taşırsanız yalnızca `depo`'daki tek satırı düzeltirsiniz; vitrinden isteyenlerin kodu değişmez. Kütüphaneler, iç düzenlerinden bağımsız bir API'yi bu yolla sunar.[^api]

Private bir öğeyi `pub use` ile dışarıya kaçıramazsınız:

```rust,compile_fail,E0364
mod depo {
    fn gizli() {}

    pub mod vitrin {
        pub use super::gizli;
    }
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0364]: `gizli` is private, and cannot be re-exported
```

Çözüm `gizli`'nin kendisini yeterince açmaktır. Dışa aktarılmak istenen şey bir modül ya da adlı alanları olan bir struct gibi yalnızca tür ad alanında duruyorsa kod `E0365` olur; ad alanlarını perde arkasında göreceğiz.

## Modülleri dosyalara bölmek

Modüllerin içeriği büyüyünce ayrı dosyalara taşınır. Süslü parantezli gövde yerine noktalı virgül yazarsanız derleyici modülün içeriğini ayrı bir dosyada arar.[^dosyalar] Dosyanın yeri, modülün ağaçtaki yerini izler:

| Bildirim | Nerede yazılı? | Aranan dosya |
| --- | --- | --- |
| `mod lokanta;` | crate kökünde | `src/lokanta.rs` ya da `src/lokanta/mod.rs` |
| `mod salon;` | `lokanta` modülünde | `src/lokanta/salon.rs` ya da `src/lokanta/salon/mod.rs` |

Lokantayı dosyalara bölelim. Paketin dizini şöyle görünür:

```text
src
├── main.rs
├── lokanta.rs
└── lokanta
    ├── mutfak.rs
    └── salon.rs
```

Aşağıdaki bloklar ayrı dosyalara ait olduğu için tek başlarına derlenmez. Kök dosya `lokanta`'yı bildirir ve kullanır:

```rust,ignore
// src/main.rs
mod lokanta;

fn main() {
    lokanta::salon::masa_ayarla();
}
```

`lokanta` modülü kendi alt modüllerini bildirir:

```rust,ignore
// src/lokanta.rs
pub mod salon;
mod mutfak;
```

Alt modüllerin dosyalarında yeniden `mod salon { ... }` yazılmaz; dosyanın kendisi modülün gövdesidir:

```rust,ignore
// src/lokanta/salon.rs
pub fn masa_ayarla() {
    println!("salon: masa ayarlandı");
    super::mutfak::hazirla();
}
```

```rust,ignore
// src/lokanta/mutfak.rs
pub fn hazirla() {
    println!("mutfak: hazır");
}
```

Program çalıştırıldığında şunu yazar:

```text
salon: masa ayarlandı
mutfak: hazır
```

Modül ağacı hiç değişmedi; yollar ve görünürlük kuralları da aynı kaldı. Yalnızca kaynak birden çok dosyaya dağıldı.

İki noktaya dikkat edin. Birincisi, `mod` ile `use` farklı işler yapar: `mod` modülü ağaca **ekler** ve ağacın bir yerinde bir kez yazılır; `use` ise var olan bir yola ad verir, dosya yüklemez. İkincisi, dosya bulunamazsa program derlenmez:

```rust,compile_fail,E0583
mod mutfak; // dosyası yok

fn main() {
    mutfak::hazirla();
}
```

Derleyicinin yanıtı:

```text
error[E0583]: file not found for module `mutfak`
  |
1 | mod mutfak; // dosyası yok
  | ^^^^^^^^^^^
```

Mesajın devamındaki `help` satırı, hangi dosyanın oluşturulması gerektiğini de söyler. Bu bildirim `src/main.rs`'de olsaydı aranan dosya `src/mutfak.rs` ya da `src/mutfak/mod.rs` olurdu.

`mutfak.rs` ile `mutfak/mod.rs` aynı anda bulunursa bu kez `E0761` alırsınız, çünkü derleyici hangisini seçeceğini bilemez. `mod.rs` eski usuldür; aynı adlı pek çok dosya birikmesin diye `mutfak.rs` biçimi önerilir.

## Hata kodu başvurusu

| Kod | Ne zaman görülür? | Ne yapmalı? |
| --- | --- | --- |
| `E0252` | Bir modüle aynı ad iki kez `use` ile getirildiğinde | Birine `as` ile başka ad takın ya da üst modülleri getirin. |
| `E0364`, `E0365` | Private bir öğe `pub use` ile dışa aktarılmak istendiğinde | Öğenin kendisini yeterince açın. |
| `E0425` | Bir fonksiyon ya da değişken adı bulunamadığında | Yolu tam yazın ya da `use` ekleyin. |
| `E0428` | Bir modülde aynı ad aynı ad alanında iki kez tanımlandığında | Birinin adını değiştirin. |
| `E0433` | Yolun başındaki ya da ortasındaki bir parça bulunamadığında | `super::`, `crate::` ya da `use` ile doğru yolu yazın. |
| `E0451` | Private alanı olan struct dışarıdan kurulduğunda ya da desende o alan anıldığında | Yapıcı fonksiyonu kullanın. |
| `E0583` | `mod ad;` için dosya bulunamadığında | Derleyicinin gösterdiği dosyayı oluşturun. |
| `E0603` | Private bir öğeye, modüle ya da `use` adına dışarıdan ulaşıldığında | Öğeyi `pub` yapın ya da açık bir yol kullanın. |
| `E0616` | Private alana nokta ile erişildiğinde | Getter kullanın ya da alanı açın. |
| `E0624` | Private metot çağrıldığında | Açık bir metot kullanın ya da metodu açın. |
| `E0761` | Modül için hem `ad.rs` hem `ad/mod.rs` bulunduğunda | İkisinden birini silin. |

## Perde arkası: Derleyici ne yapıyor?

> [!NOTE]
> Bu başlık ilk okuyuşta atlanabilir.

### Modüllerin çalışma zamanında karşılığı yoktur

Modüller öğedir ve öğeler bütünüyle derleme zamanında belirlenir.[^ogeler] Çalışan programda `salon` diye bir nesne dolaşmaz; `lokanta::salon::masa_ayarla()` ile `hazirla_masa()` aynı fonksiyonun çağrısıdır. Modüller adları düzenler ve neyin dışarı açık olduğunu belirler.

Dolaylı bir etkileri vardır: Derleyici, artımlı derlemede kodu üretim birimlerine (codegen unit) bölerken kaynak modüllerini ölçü alır. Böylece bir modüldeki değişiklikten sonra ilgisiz modüllerin üretilmiş kodu yeniden kullanılabilir.[^birimler] Bu, programın anlamını değil, derleme süresini ilgilendirir.

### Adlar nasıl çözülür?

`rustc_resolve`, yollardaki adları tanımlarıyla eşler. Reference, ad çözümlemeyi üç aşamaya ayırır: Makrolar ve `use` bildirimleri makro açılımı sırasında çözülür. Türe bağlı olmayan öteki adlar bunun ardından gelir. Metotlar, alanlar ve ilişkili öğeler gibi türe bağlı adlar ise en sona, tür bilgisinin hazır olduğu tür denetimine kalır.[^cozumleme] Yani `lokanta::ac` yolunun nereyi gösterdiği türlere bakılmadan bilinir, `f.tutar()` çağrısının hangi metot olduğu ise `f`'nin türü bilinince anlaşılır.

Adlar tek bir deftere yazılmaz. Rust'ta beş ad alanı vardır: türler, değerler, makrolar, ömürler ve etiketler.[^ad-alanlari] Derleyicinin içinde bunların ilk üçü `Namespace` enum'uyla temsil edilir.[^namespace] Bu ayrım sayesinde aynı ad farklı alanlarda birlikte durabilir:

```rust
struct S; // hem tür hem de değer ad alanında S

mod masa {} // tür ad alanı
fn masa() -> u32 { // değer ad alanı
    4
}

fn main() {
    let s: S = S; // ilk S tür, ikinci S değer
    let _ = s;
    println!("masa: {}", masa());
}
```

Çıktı:

```text
masa: 4
```

Adlı alanları olan bir struct yalnızca tür ad alanında durur. Alansız (unit) ve demet benzeri struct'lar ise aynı adda bir de yapıcı, yani değer tanımlar. Bu yüzden aynı adlı bir fonksiyonla çakışırlar:

```rust,compile_fail,E0428
struct S;
fn S() {}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0428]: the name `S` is defined multiple times
  |
  = note: `S` must be defined only once in the value namespace of this module
```

### Gizlilik nerede denetlenir?

Görünürlük tek bir yerde denetlenmez; denetim çoğunlukla adın çözüldüğü yerde yapılır:

- Yol ile anılan öğeler `rustc_resolve`'da denetlenir. `E0603` hatasının ve dışa aktarma hataları `E0364` ile `E0365`'in kaynağı burasıdır.[^resolve]
- Alan erişimi ve metot çağrısı türe bağlı olduğu için tür denetiminde, `rustc_hir_typeck`'te denetlenir: `E0616` ve `E0624`.[^typeck] Demet benzeri bir struct'ın private yapıcısı `Self(...)` yazımıyla çağrılırsa `E0603` de buradan gelir.[^ctor]
- `rustc_privacy`, önceki iki aşamada denetlenmeyenleri üstlenir: struct ifadelerinde ve desenlerinde adı geçen alanlar (`E0451`) ile ifadelerin ve desenlerin türlerine erişim izni.[^privacy]

## Özet

- Paket bir ya da daha çok crate içerir; crate derlemenin birimidir ve modül ağacı onun kök dosyasından başlar.
- Her öğe private gelir ve tanımlandığı modül ile alt modüllerinden görülür. `pub`, `pub(crate)` ve `pub(super)` ile bilinçli olarak açarsınız. `pub` enum'un varyantları ve `pub` trait'in öğeleri istisnadır.
- Yol bir adrestir: `crate::` kökten, `self::` bu modülden, `super::` bir üst modülden başlar.
- Struct alanları tek tek açılır. Private alan, yapıcı ve getter ile birlikte kapsüllemeyi sağlar.
- `use` kısa ad verir; `pub use` o adı dışarıya da sunar ve iç düzeni saklar.
- `mod ad;` modülü ayrı bir dosyadan yükler; ağaç ve kurallar değişmez.
- Modüllerin çalışma zamanında bedeli yoktur; gizlilik derleme zamanında denetlenir.

Sıradaki bölüm [7. Koleksiyonlar](07-collections.md), standart kütüphanenin `std::collections` ve çevresinde sunduğu `Vec`, `String` ve `HashMap` türlerini ele alıyor.

## Kaynaklar

[^moduller]: The Rust Programming Language, [7.2 Control Scope and Privacy with Modules](https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html#control-scope-and-privacy-with-modules).
[^paket]: The Rust Programming Language, [7.1 Packages and Crates](https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html#packages-and-crates); The Rust Reference, [Crates and source files](https://doc.rust-lang.org/reference/crates-and-source-files.html#crates-and-source-files).
[^yerlesim]: Cargo Book, [Package Layout](https://doc.rust-lang.org/cargo/guide/project-layout.html).
[^calisma-alani]: Cargo Book, [Virtual workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html#virtual-workspace) ve [The `[package]` section](https://doc.rust-lang.org/cargo/reference/manifest.html#the-package-section).
[^modul-tanim]: The Rust Reference, [Modules](https://doc.rust-lang.org/reference/items/modules.html#modules); The Rust Programming Language, [Grouping Related Code in Modules](https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html#grouping-related-code-in-modules).
[^gorunurluk]: The Rust Reference, [Visibility and privacy](https://doc.rust-lang.org/reference/visibility-and-privacy.html#visibility-and-privacy); hata dizini, [`E0603`](https://doc.rust-lang.org/error_codes/E0603.html).
[^sinirli-pub]: The Rust Reference, [`pub(in path)`, `pub(crate)`, `pub(super)`, and `pub(self)`](https://doc.rust-lang.org/reference/visibility-and-privacy.html#pubin-path-pubcrate-pubsuper-and-pubself).
[^iki-crate]: The Rust Programming Language, [Best Practices for Packages with a Binary and a Library](https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html#best-practices-for-packages-with-a-binary-and-a-library).
[^neden-private]: The Rust Programming Language, [Exposing Paths with the `pub` Keyword](https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html#exposing-paths-with-the-pub-keyword).
[^testler]: The Rust Programming Language, [Private Function Tests](https://doc.rust-lang.org/book/ch11-03-test-organization.html#private-function-tests).
[^yollar]: The Rust Programming Language, [7.3 Paths for Referring to an Item in the Module Tree](https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html#paths-for-referring-to-an-item-in-the-module-tree); The Rust Reference, [Path qualifiers](https://doc.rust-lang.org/reference/paths.html#path-qualifiers).
[^dis-crate]: The Rust Reference, [Extern prelude](https://doc.rust-lang.org/reference/names/preludes.html#extern-prelude); hata dizini, [`E0433`](https://doc.rust-lang.org/error_codes/E0433.html) ve [`E0425`](https://doc.rust-lang.org/error_codes/E0425.html).
[^super]: The Rust Programming Language, [Starting Relative Paths with `super`](https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html#starting-relative-paths-with-super).
[^pub-struct]: The Rust Programming Language, [Making Structs and Enums Public](https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html#making-structs-and-enums-public); hata dizini, [`E0451`](https://doc.rust-lang.org/error_codes/E0451.html), [`E0616`](https://doc.rust-lang.org/error_codes/E0616.html) ve [`E0624`](https://doc.rust-lang.org/error_codes/E0624.html).
[^use]: The Rust Reference, [Use declarations](https://doc.rust-lang.org/reference/items/use-declarations.html#use-declarations).
[^use-kapsam]: The Rust Programming Language, [7.4 Bringing Paths into Scope with the `use` Keyword](https://doc.rust-lang.org/book/ch07-04-bringing-paths-into-scope-with-the-use-keyword.html#bringing-paths-into-scope-with-the-use-keyword).
[^use-deyim]: The Rust Programming Language, [Creating Idiomatic `use` Paths](https://doc.rust-lang.org/book/ch07-04-bringing-paths-into-scope-with-the-use-keyword.html#creating-idiomatic-use-paths).
[^as]: The Rust Programming Language, [Providing New Names with the `as` Keyword](https://doc.rust-lang.org/book/ch07-04-bringing-paths-into-scope-with-the-use-keyword.html#providing-new-names-with-the-as-keyword); hata dizini, [`E0252`](https://doc.rust-lang.org/error_codes/E0252.html).
[^ic-ice]: The Rust Programming Language, [Using Nested Paths to Clean Up `use` Lists](https://doc.rust-lang.org/book/ch07-04-bringing-paths-into-scope-with-the-use-keyword.html#using-nested-paths-to-clean-up-use-lists) ve [Importing Items with the Glob Operator](https://doc.rust-lang.org/book/ch07-04-bringing-paths-into-scope-with-the-use-keyword.html#importing-items-with-the-glob-operator).
[^yeniden-acma]: The Rust Programming Language, [Re-exporting Names with `pub use`](https://doc.rust-lang.org/book/ch07-04-bringing-paths-into-scope-with-the-use-keyword.html#re-exporting-names-with-pub-use); The Rust Reference, [`use` visibility](https://doc.rust-lang.org/reference/items/use-declarations.html#use-visibility) ve [Re-exporting and visibility](https://doc.rust-lang.org/reference/visibility-and-privacy.html#re-exporting-and-visibility).
[^api]: The Rust Programming Language, [Exporting a Convenient Public API](https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#exporting-a-convenient-public-api); hata dizini, [`E0364`](https://doc.rust-lang.org/error_codes/E0364.html) ve [`E0365`](https://doc.rust-lang.org/error_codes/E0365.html).
[^dosyalar]: The Rust Reference, [Module source filenames](https://doc.rust-lang.org/reference/items/modules.html#module-source-filenames); The Rust Programming Language, [7.5 Separating Modules into Different Files](https://doc.rust-lang.org/book/ch07-05-separating-modules-into-different-files.html#separating-modules-into-different-files); hata dizini, [`E0583`](https://doc.rust-lang.org/error_codes/E0583.html) ve [`E0761`](https://doc.rust-lang.org/error_codes/E0761.html).
[^ogeler]: The Rust Reference, [Items](https://doc.rust-lang.org/reference/items.html#items).
[^birimler]: rustc 1.99.0 kaynak kodu, [`partitioning.rs`, "two `codegen units` for every source-level module"](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_monomorphize/src/partitioning.rs#L42).
[^cozumleme]: The Rust Reference, [Name resolution](https://doc.rust-lang.org/reference/names/name-resolution.html#name-resolution) ve [Type-relative resolution](https://doc.rust-lang.org/reference/names/name-resolution.html#type-relative-resolution); rustc geliştirme kılavuzu, [Name resolution](https://rustc-dev-guide.rust-lang.org/name-resolution.html).
[^ad-alanlari]: The Rust Reference, [Namespaces](https://doc.rust-lang.org/reference/names/namespaces.html#namespaces).
[^namespace]: rustc 1.99.0 kaynak kodu, [`def.rs`, `Namespace`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir/src/def.rs#L644).
[^resolve]: rustc 1.99.0 kaynak kodu, [rustc_resolve tanılamaları, `IsPrivate`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_resolve/src/diagnostics/mod.rs#L1163).
[^typeck]: rustc 1.99.0 kaynak kodu, [`expr.rs`, `private_field_err`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_typeck/src/expr.rs#L3325) ve [`method/suggest.rs`, `PrivateMatch`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_typeck/src/method/suggest.rs#L361).
[^ctor]: rustc 1.99.0 kaynak kodu, [rustc_hir_typeck tanılamaları, `CtorIsPrivate`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_typeck/src/diagnostics.rs#L955).
[^privacy]: rustc 1.99.0 kaynak kodu, [`rustc_privacy/src/lib.rs`, `NamePrivacyVisitor`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_privacy/src/lib.rs#L918) ve [`TypePrivacyVisitor`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_privacy/src/lib.rs#L1127).
