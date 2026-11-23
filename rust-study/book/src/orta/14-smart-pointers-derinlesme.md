# 14.1 Akıllı İşaretçiler: Derinleşme

Önceki sayfa her işaretçiyi tek başına tanıttı. Gerçek kodda ise çoğu zaman birlikte kullanılırlar ve asıl zor olan, hangisinin ne zaman gerektiğine karar vermektir. Bu sayfada şu konulara bakacağız:

- Doğru işaretçiyi seçmek için hangi üç soru sorulmalı?
- Ebeveynini bilen bir ağaç `Rc`, `Weak` ve `RefCell` ile nasıl kurulur?
- Referans döngüsü nasıl sızdırır ve bunu nasıl ölçeriz?
- Dışarıdan değişmez görünen bir tür içeride nasıl kayıt tutar?
- `RefCell` ve `Rc` ile en sık yapılan dört hata nedir?
- "Paylaşılıyorsa kopyala" yöntemi nasıl işler?

> [!NOTE]
> Bu sayfanın kodu `learn/smart-pointers/src/derinlesme.rs` dosyasındadır. Depo kökünden `cargo run -p smart-pointers` ile çalıştırabilir, testlerini `cargo test -p smart-pointers` ile sınayabilirsiniz.

<details>
<summary>Bu sayfada geçen yeni terimler</summary>

- **`Cell<T>`:** Değeri içinden alıp yerine yenisini koyarak güncelleyen hücre.
- **`Arc<T>`:** `Rc`'nin birden çok thread'de güvenle çalışan karşılığı.
- **`Mutex<T>` ve `RwLock<T>`:** `RefCell`'in çok thread'li karşılıkları: kilitle, kullan, bırak.
- **güçlü ve zayıf sayaç (strong / weak count):** Sırasıyla sahiplerin ve yalnızca yerini bilenlerin sayısı.
- **`Rc::downgrade`:** Bir `Rc`'den, güçlü sayacı artırmayan bir `Weak` üretmek.
- **`upgrade()`:** `Weak`'e "gösterdiğin şey hâlâ duruyor mu?" diye sormak. Yanıt `Some` ya da `None` olur.
- **koruyucu (guard):** `borrow()` ve `borrow_mut()` ile alınan izin kâğıdı (`Ref` ve `RefMut`). Elinizde durdukça izin açık kalır.
- **`try_borrow_mut()`:** İzni panik yerine `Result` yanıtıyla istemek.
- **mock (test ikizi):** Testte gerçeğinin yerine geçen ve olanları kaydeden sahte parça.
- **gözlemci (observer):** "Bir şey olunca bana haber ver" diye listeye yazılan parça.
- **önbellek (cache):** Bir daha hesaplamamak için sonucu saklanan değer.
- **clone-on-write:** "Paylaşılıyorsa önce kopyala, sonra yaz" yöntemi.
- **`saturating_add`:** Taşacaksa en büyük değerde duran, başa sarmayan toplama.
- **atomik (atomic):** İki thread aynı anda dokunsa bile bozulmayan, tek hamlede gerçekleşen işlem.

</details>

## Hangi işaretçi? Üç soru

Okulda bir eşyanın nerede duracağına üç soruyla karar verirsiniz: Onu kaç kişi kullanacak? Kullananlar onu değiştirecek mi? Biri yalnızca nerede durduğunu mu bilmek istiyor? Rust'ta da aynı üç soru sırayla sorulur:[^secim]

| Soru | Yanıt | Araç | Bedeli |
| --- | --- | --- | --- |
| Kaç sahip var? | Tek sahip | `Box<T>` | Bir heap tahsisi |
| | Çok sahip, tek thread | `Rc<T>` | Sıradan tam sayı olan bir sayaç |
| | Çok sahip, birden çok thread | `Arc<T>` | Atomik bir sayaç |
| Paylaşılan değer değişecek mi? | Hayır | `Rc<T>` ya da `Arc<T>` yeter | Ek bedel yok |
| | Evet, küçük bir `Copy` değer | `Cell<T>` | İçine referans alınamaz; panik yoktur |
| | Evet, genel | `RefCell<T>` | Ödünç sayacı; ihlalde panik |
| | Evet, birden çok thread | `Mutex<T>` ya da `RwLock<T>` | Kilit |
| Sahip olmadan bilmek mi gerekiyor? | Evet (geri işaret, önbellek, gözlemci listesi) | `Weak<T>` | Her kullanımdan önce `upgrade()` |

Her kutunun bir bedeli olduğu için sorular bu sırayla sorulur: Gereksiz bedeli ödemeyin. Dört temel seçim kodda şöyle görünür:

```rust
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let tek = Box::new(1); // tek sahip
    let ortak = Rc::new(1); // çok sahip, yalnızca okunur
    let degisen = Rc::new(RefCell::new(1)); // çok sahip, içi değişir
    let bilen = Rc::downgrade(&ortak); // sahip değil, yalnızca yerini biliyor

    *degisen.borrow_mut() += 1;
    println!("tek = {tek}, ortak = {ortak}");
    println!("değişen = {}", degisen.borrow());
    println!("bilen = {:?}", bilen.upgrade());
}
```

Çıktı:

```text
tek = 1, ortak = 1
değişen = 2
bilen = Some(1)
```

Sık görülen birleşimler bu yanıtların çarpımıdır. `Rc<RefCell<T>>` tek thread'de çok sahipli ve değişen veri, `Arc<Mutex<T>>` (çok okunan veride `Arc<RwLock<T>>`) onun çok thread'li karşılığıdır. `Rc<T>` ile `Weak<T>` birlikte ağaç ve graf kurar.

> [!IMPORTANT]
> Bunlar ilk tercih değil, kaçış kapılarıdır. Önce şunu sorun: Veriyi tek bir sahipte toplayıp ötekilere `&` veremez miyim? `Rc<RefCell<T>>` programın her yerine yayılırsa derleyicinin derleme zamanında yakalayacağı hatalar çalışma zamanı paniklerine dönüşür.

## Ebeveynini bilen ağaç

Bir dosya sistemi ağacında her dizin çocuklarının sahibidir: Dizin silinirse içindekiler de silinir. Öte yandan her çocuk, tam yolunu üretebilmek için ebeveynini de *bilmelidir*. İki yön de `Rc` olsaydı ebeveyn ile çocuk birbirini canlı tutar, hiçbiri düşürülmezdi.

Bir sınıf da öğrenci listesinin sahibidir: Sınıf kapanırsa liste de kalkar. Öğrenci hangi sınıfta olduğunu bilir, ama o biliyor diye sınıf açık tutulmaz. Ağaçtaki iki yön de böyle ayrılır:

- Ebeveynden çocuğa: `Rc` ("sahibiyim").
- Çocuktan ebeveyne: `Weak` ("yerini biliyorum, ama onu canlı tutmuyorum").

```rust
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub struct Dizin {
    ad: String,
    ebeveyn: RefCell<Weak<Dizin>>,
    cocuklar: RefCell<Vec<Rc<Dizin>>>,
}

impl Dizin {
    pub fn yeni(ad: &str) -> Rc<Dizin> {
        Rc::new(Dizin {
            ad: ad.to_string(),
            ebeveyn: RefCell::new(Weak::new()), // hiçbir şeye bakmayan boş Weak
            cocuklar: RefCell::new(Vec::new()),
        })
    }

    pub fn cocuk_ekle(ebeveyn: &Rc<Dizin>, cocuk: Rc<Dizin>) {
        *cocuk.ebeveyn.borrow_mut() = Rc::downgrade(ebeveyn);
        ebeveyn.cocuklar.borrow_mut().push(cocuk);
    }

    pub fn tam_yol(&self) -> String {
        match self.ebeveyn.borrow().upgrade() {
            Some(ebeveyn) => format!("{}/{}", ebeveyn.tam_yol(), self.ad),
            None => self.ad.clone(), // kök ya da ebeveyni düşmüş düğüm
        }
    }

    pub fn cocuk_sayisi(&self) -> usize {
        self.cocuklar.borrow().len()
    }
}

fn main() {
    let kok = Dizin::yeni("");
    let home = Dizin::yeni("home");
    let kaan = Dizin::yeni("kaan");
    Dizin::cocuk_ekle(&kok, Rc::clone(&home));
    Dizin::cocuk_ekle(&home, Rc::clone(&kaan));

    println!("tam yol = {}", kaan.tam_yol());
    println!("home: güçlü = {}", Rc::strong_count(&home));
    println!("home: zayıf = {}", Rc::weak_count(&home));
    println!("home: çocuk = {}", home.cocuk_sayisi());

    drop(kok);
    drop(home); // güçlü sayaç 0 oldu; kaan'daki Weak onu kurtarmaz
    println!("ebeveyn düştükten sonra tam yol = {}", kaan.tam_yol());
}
```

Çıktı:

```text
tam yol = /home/kaan
home: güçlü = 2
home: zayıf = 1
home: çocuk = 1
ebeveyn düştükten sonra tam yol = kaan
```

Alan türlerini içten dışa okuyun. `RefCell<Weak<Dizin>>`, sonradan değiştirilebilen (`RefCell`) ve sahip olmayan (`Weak`) bir `Dizin` işaretidir. İki alanda da `RefCell` bulunmasının nedeni, düğümlere yalnızca `Rc` üzerinden, yani paylaşımlı referansla ulaşılmasıdır; ebeveyn de çocuk listesi de sonradan değişir.

`cocuk_ekle` üç adımda çalışır:

1. `Rc::downgrade(ebeveyn)`, güçlü sayaca dokunmadan bir `Weak` üretir; yalnızca zayıf sayaç bir artar.
2. Bu `Weak`, çocuğun `ebeveyn` hücresine yazılır.
3. Çocuğun `Rc`'si ebeveynin listesine taşınır. Çağıran elinde başka bir `Rc` tutmuyorsa çocuğun tek güçlü sahibi artık ebeveyndir.

`home`'un güçlü sayacı 2'dir: Biri yerel `home` değişkeni, öteki kökün listesidir. Zayıf sayacı 1'dir: `kaan`'ın ebeveyn işareti. `tam_yol` içindeki `upgrade`, ebeveyn yaşıyorsa geçici bir güçlü sahiplik verir; o sürede ebeveyn düşürülemez. Yaşamıyorsa `None` döner. `Weak`'i güvenli kodda kullanmanın yolu bu soruyu sormaktan geçtiği için sarkan bir işaretçiye bakmak mümkün değildir.[^agac]

## Referans döngüsü sızdırır

İki öğrenci birbirinin kalemini tutuyor ve ikisi de "önce sen benimkini bırak" diyor. Zil çalar, herkes gider; onlar öylece kalır. Birbirini `Rc` ile tutan iki değer de böyledir. Derleyici bu durumu yakalamaz: Sızıntı Rust'ta güvenli sayılır; bellek bozulmaz, yalnızca geri verilmez.[^dongu] Uzun süre çalışan bir sunucuda bu, yavaş yavaş büyüyen bir bellek tüketimi demektir.

Sızıntıyı ölçmek için düşürülen her düğüm ortak bir sayacı artırsın. Fonksiyonun dönüş değeri, kapsam bittiğinde gerçekten düşürülen düğüm sayısıdır; doğrusu 2'dir:

```rust
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

struct Dugum {
    dusme_sayaci: Rc<Cell<u32>>,
    guclu_komsu: RefCell<Option<Rc<Dugum>>>,
    zayif_komsu: RefCell<Weak<Dugum>>,
}

impl Dugum {
    fn yeni(sayac: &Rc<Cell<u32>>) -> Rc<Dugum> {
        Rc::new(Dugum {
            dusme_sayaci: Rc::clone(sayac),
            guclu_komsu: RefCell::new(None),
            zayif_komsu: RefCell::new(Weak::new()),
        })
    }
}

impl Drop for Dugum {
    fn drop(&mut self) {
        self.dusme_sayaci.set(self.dusme_sayaci.get() + 1);
    }
}

fn cift_olustur(geri_isaret_zayif: bool) -> u32 {
    let sayac = Rc::new(Cell::new(0));
    {
        let a = Dugum::yeni(&sayac);
        let b = Dugum::yeni(&sayac);
        *a.guclu_komsu.borrow_mut() = Some(Rc::clone(&b)); // a, b'nin sahibi
        if geri_isaret_zayif {
            *b.zayif_komsu.borrow_mut() = Rc::downgrade(&a); // b, a'yı yalnızca bilir
        } else {
            *b.guclu_komsu.borrow_mut() = Some(Rc::clone(&a)); // döngü
        }
    } // a ve b burada kapsam dışına çıkar
    sayac.get()
}

fn main() {
    println!("iki yön de Rc: düşen düğüm = {}", cift_olustur(false));
    println!("geri yön Weak: düşen düğüm = {}", cift_olustur(true));
}
```

Çıktı:

```text
iki yön de Rc: düşen düğüm = 0
geri yön Weak: düşen düğüm = 2
```

Döngülü sürümde olanları adım adım izleyelim. Kapsam sonunda önce `b` adı düşürülür; `b`'nin güçlü sayacı 2'den 1'e iner, çünkü `a` onu hâlâ tutmaktadır. Ardından `a` adı düşürülür; onun sayacı da 2'den 1'e iner, çünkü `b` onu tutmaktadır. Artık ikisine de ulaşan kimse yoktur, ama ikisinin sayacı da 1'de takılıdır: Düğümler sızmıştır.

`Weak` kullanan sürümde `b` adının düşmesi yine `b`'nin sayacını 1'e indirir. `a` adı düştüğünde ise `a`'nın güçlü sayacı 1'den 0'a iner, `a` düşürülür ve elindeki `b` sahipliğini bırakır. `b`'nin sayacı da 0 olur; zincir çözülür.

Kural şudur: Döngü kurabilecek işaretlerden en az biri `Weak` olmalıdır; bu da genellikle sahiplik taşımayan yöndür (geri işaret, yan işaret, gözlemci).

## İç değiştirilebilirlik: içeride kayıt tutan tür

Kütüphaneden bir kitabı yalnızca okumak için alırsınız; sayfalarına dokunmazsınız. Yine de arka kapaktaki fişe "bir kez daha okundu" diye bir çentik atılır. Kitap sizin açınızdan değişmemiştir, ama içindeki küçük bir kayıt güncellenmiştir.

Programlarda da bir işlem *mantıksal olarak* salt okunur olabilir: Çağıran açısından hiçbir şey değişmez, ama gerçekleştirim içeride bir çağrı sayacını, önbelleği ya da denetim kaydını güncellemek ister. Metodu `&mut self` yapmak bu iç ayrıntıyı imzaya sızdırır ve bütün çağıranları özel erişim almaya zorlar; aynı anda iki okuyucu bile olamaz. İç değiştirilebilirlik bu ayrımı korur: İmza `&self` kalır, değişen alan bir hücrenin içinde durur. İki hücre türü vardır:[^hucre]

- **`Cell<T>`** değeri içeri koyar ve dışarı çıkarır (`get`, `set`, `replace`, `take`), içine referans vermez. Referans olmadığı için iki ödüncün çakışması diye bir durum oluşamaz; panik de olmaz. Sayaç ve bayrak gibi küçük `Copy` değerler için ilk tercihtir.
- **`RefCell<T>`** içine referans verir (`borrow`, `borrow_mut`) ve kuralı çalışma zamanında denetler. `Vec`, `String`, `HashMap` gibi yerinde değiştirilen değerler içindir.

```rust
use std::cell::{Cell, RefCell};

fn main() {
    let sayac = Cell::new(0); // mut yok
    sayac.set(sayac.get() + 1);

    let liste = RefCell::new(vec![1]);
    liste.borrow_mut().push(2); // izin, deyim bitince geri verilir

    println!("sayaç = {}, liste = {:?}", sayac.get(), liste.borrow());
}
```

Çıktı:

```text
sayaç = 1, liste = [1, 2]
```

Hücresiz ve `mut` olmadan tanımlanmış bir değer ise değiştirilemez:

```rust,compile_fail,E0384
fn main() {
    let duz = 0;
    duz += 1;
    println!("{duz}");
}
```

Derleyicinin yanıtı:

```text
error[E0384]: cannot assign twice to immutable variable `duz`
```

> [!NOTE]
> `Cell`'in yalnızca `get` metodu `T: Copy` ister. `set`, `replace`, `swap` ve `take` metotları (`take` için `T: Default` gerekir) `Copy` olmayan türlerle de çalışır, çünkü değeri kopyalamaz, taşırlar: `Cell<String>` yazabilir, içindeki metni `replace` ile yenisiyle değiştirip eskisini alabilirsiniz.[^cell]

### Test ikizi: `&self` arkasında kayıt tutmak

Bildirim gönderen bir trait düşünelim. Gönderen taraf "bildirim göndermek beni değiştirmez" varsayar; bu yüzden metot `&self` alır. Testte gerçek bir bildirim göndermek yerine mesajları kaydeden bir *test ikizi* (mock) kullanmak isteriz. Kaydı düz bir `Vec` alanında tutmaya kalkarsak derleyici izin vermez:

```rust,compile_fail,E0596
pub trait Bildirimci {
    fn gonder(&self, mesaj: &str);
}

struct Kayit {
    mesajlar: Vec<String>,
}

impl Bildirimci for Kayit {
    fn gonder(&self, mesaj: &str) {
        self.mesajlar.push(mesaj.to_string());
    }
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0596]: cannot borrow `self.mesajlar` as mutable, as it is behind a `&` reference
   |
11 |         self.mesajlar.push(mesaj.to_string());
   |         ^^^^^^^^^^^^^ `self` is a `&` reference, so it cannot be borrowed as mutable
```

Derleyici trait'teki imzayı `&mut self` yapmayı önerir. Bu öneri burada yanlış yöne götürür: Trait'i test uğruna değiştirmek, üretim kodunun tasarımını teste kurban etmek olurdu. Doğru çözüm, değişen alanları hücrelere koymaktır:[^mock]

```rust
use std::cell::{Cell, RefCell};

pub trait Bildirimci {
    fn gonder(&self, mesaj: &str);
}

#[derive(Default)]
pub struct KayitliBildirimci {
    mesajlar: RefCell<Vec<String>>, // yerinde büyüyen koleksiyon
    cagri: Cell<u32>,               // küçük Copy sayaç
}

impl Bildirimci for KayitliBildirimci {
    fn gonder(&self, mesaj: &str) {
        self.cagri.set(self.cagri.get() + 1);
        self.mesajlar.borrow_mut().push(mesaj.to_string());
    }
}

impl KayitliBildirimci {
    pub fn cagri_sayisi(&self) -> u32 {
        self.cagri.get()
    }

    pub fn mesajlar(&self) -> Vec<String> {
        self.mesajlar.borrow().clone()
    }
}

fn main() {
    let bildirimci = KayitliBildirimci::default(); // mut yok
    bildirimci.gonder("deneme");
    println!("{} çağrı: {:?}", bildirimci.cagri_sayisi(), bildirimci.mesajlar());
}
```

Çıktı:

```text
1 çağrı: ["deneme"]
```

`gonder` içindeki `borrow_mut()` geçici bir koruyucu döndürür; deyim bitince bırakılır. Şimdi bu ikizi kullanan üretim koduna bakalım. `KotaTakibi` bildirimcinin hangi tür olduğunu bilmez, yalnızca trait'e bağlıdır:

```rust
# use std::cell::{Cell, RefCell};
# pub trait Bildirimci { fn gonder(&self, mesaj: &str); }
# #[derive(Default)]
# pub struct KayitliBildirimci { mesajlar: RefCell<Vec<String>>, cagri: Cell<u32> }
# impl Bildirimci for KayitliBildirimci {
#     fn gonder(&self, mesaj: &str) {
#         self.cagri.set(self.cagri.get() + 1);
#         self.mesajlar.borrow_mut().push(mesaj.to_string());
#     }
# }
# impl KayitliBildirimci {
#     pub fn cagri_sayisi(&self) -> u32 { self.cagri.get() }
#     pub fn mesajlar(&self) -> Vec<String> { self.mesajlar.borrow().clone() }
# }
pub struct KotaTakibi<'a, B: Bildirimci> {
    bildirimci: &'a B,
    sinir: u32,
    kullanilan: u32,
}

impl<'a, B: Bildirimci> KotaTakibi<'a, B> {
    pub fn yeni(bildirimci: &'a B, sinir: u32) -> Self {
        KotaTakibi {
            bildirimci,
            sinir: sinir.max(1), // 0 sınır, aşağıda sıfıra bölme paniği olurdu
            kullanilan: 0,
        }
    }

    pub fn kullan(&mut self, miktar: u32) {
        self.kullanilan = self.kullanilan.saturating_add(miktar);
        let yuzde = u64::from(self.kullanilan) * 100 / u64::from(self.sinir);
        if yuzde >= 100 {
            self.bildirimci.gonder("Hata: kota aşıldı");
        } else if yuzde >= 75 {
            self.bildirimci.gonder("Uyarı: kotanın %75'i doldu");
        }
    }
}

fn main() {
    let bildirimci = KayitliBildirimci::default();
    let mut takip = KotaTakibi::yeni(&bildirimci, 100);
    takip.kullan(50); // %50: bildirim yok
    takip.kullan(30); // %80: uyarı
    takip.kullan(25); // %105: hata

    println!("çağrı = {}", bildirimci.cagri_sayisi());
    println!("{:?}", bildirimci.mesajlar());
}
```

Çıktı:

```text
çağrı = 2
["Uyarı: kotanın %75'i doldu", "Hata: kota aşıldı"]
```

`kullan` içindeki iki ayrıntı taşma (overflow) yüzündendir. `u32` toplamı taşarsa taşma denetiminin açık olduğu derlemelerde (varsayılan `debug`) panik, kapalı olduğu derlemelerde (varsayılan `release`) sessiz sarma görülür.[^tasma] Bu yüzden toplam `saturating_add` ile en büyük değerde durdurulur, yüzde çarpımı da `u64` ile yapılır.

## Sık yapılan hatalar

### Ödünç açıkken ikincisini istemek (panik)

Nöbetçiden "bakma izni" alıp kâğıdı cebinizde unutursanız yazmak isteyen hiç kimse içeri giremez. `borrow()` böyle bir izin kâğıdı verir; kâğıt yaşadıkça `borrow_mut()` geri çevrilir ve program panikler. Biri bakarken başkası yazsaydı bakan kişi yarım değişmiş veri görürdü. Derleme zamanında hata çıkmaz; denetimi çalışma zamanına taşımanın bedeli budur. `try_borrow_mut` aynı durumu panik yerine `Result` olarak gösterir:[^try]

```rust
use std::cell::RefCell;

fn main() {
    let d = RefCell::new(vec![1]);

    let bakan = d.borrow(); // bakma izni açık
    println!("{} öğe", bakan.len());
    println!("yazılabilir mi? {}", d.try_borrow_mut().is_ok());

    drop(bakan); // izin geri verildi
    println!("şimdi? {}", d.try_borrow_mut().is_ok());
    d.borrow_mut().push(2);
    println!("{:?}", d.borrow());
}
```

Çıktı:

```text
1 öğe
yazılabilir mi? false
şimdi? true
[1, 2]
```

Pratikte bu panik en çok, bir ödünç açıkken aynı `RefCell`'e dokunan başka bir metot çağrıldığında doğar; özellikle geri çağrılarda ve gözlemci listelerinde. En kısa örneği bir döngüdür:

```rust,should_panic
use std::cell::RefCell;

fn main() {
    let liste = RefCell::new(vec![1, 2, 3]);

    for x in liste.borrow().iter() {
        liste.borrow_mut().push(*x);
    }
}
```

Çıktı:

```text
RefCell already borrowed
```

`for` başlığındaki `liste.borrow()` geçici bir koruyucudur ve döngü bitene dek yaşar. Çözüm ödüncü dar tutmaktır: Önce hesaplayın, ödüncü kapatın, sonra yazın.

```rust
use std::cell::RefCell;

fn main() {
    let liste = RefCell::new(vec![1, 2, 3]);

    let eklenecek: Vec<i32> = liste.borrow().iter().map(|x| x * 10).collect();
    liste.borrow_mut().extend(eklenecek); // okuma ödüncü önceki deyimde bitti

    println!("liste = {:?}", liste.borrow());
}
```

Çıktı:

```text
liste = [1, 2, 3, 10, 20, 30]
```

### Geçici koruyucudan referans kaçırmak (`E0716`)

```rust,compile_fail,E0716
use std::cell::RefCell;

fn main() {
    let liste = RefCell::new(vec![1, 2, 3]);
    let ilk = liste.borrow().first().unwrap();
    println!("{ilk}");
}
```

Derleyicinin yanıtı:

```text
error[E0716]: temporary value dropped while borrowed
  |
5 |     let ilk = liste.borrow().first().unwrap();
  |               ^^^^^^^^^^^^^^                 - temporary value is freed at the end of this statement
  |               |
  |               creates a temporary value which is freed while still in use
6 |     println!("{ilk}");
  |                --- borrow later used here
  |
help: consider using a `let` binding to create a longer lived value
```

`borrow()` bir `Ref` koruyucusu döndürür. İçeriye giden referans koruyucuya bağlıdır; koruyucu ise deyimin sonunda düşürülen bir geçicidir.[^gecici] İki çözüm vardır: Değeri kopyalayıp çıkarmak ya da koruyucuyu bir değişkene bağlamak.

```rust
use std::cell::RefCell;

fn main() {
    let liste = RefCell::new(vec![1, 2, 3]);

    // i32 Copy olduğu için değer, koruyucu düşmeden kopyalanır.
    let ilk = *liste.borrow().first().expect("liste üç öğeyle kuruldu");
    println!("ilk = {ilk}");

    // Ya da koruyucu bir ada bağlanır; kapsamı kısa tutulur.
    {
        let koruyucu = liste.borrow();
        let son = koruyucu.last().expect("liste üç öğeyle kuruldu");
        println!("son = {son}");
    }
}
```

Çıktı:

```text
ilk = 1
son = 3
```

### `Rc`'nin içini doğrudan değiştirmek (`E0594`, `E0596`)

```rust,compile_fail,E0594
use std::rc::Rc;

fn main() {
    let mut r = Rc::new(5);
    *r += 1;
}
```

Derleyicinin yanıtı:

```text
error[E0594]: cannot assign to data in an `Rc`
  |
5 |     *r += 1;
  |     ^^^^^^^ cannot assign
  |
  = help: trait `DerefMut` is required to modify through a dereference, but it is not implemented for `Rc<i32>`
```

Aynı engel metot çağrısında `E0596` olarak görünür:

```rust,compile_fail,E0596
use std::rc::Rc;

fn main() {
    let v = Rc::new(vec![1]);
    v.push(2);
}
```

Derleyicinin yanıtı:

```text
error[E0596]: cannot borrow data in an `Rc` as mutable
```

`Rc` yalnızca `Deref` uygular, `DerefMut` uygulamaz. Başka sahipler de aynı veriye bakıyor olabilir; `&mut` vermek *aliasing XOR mutability* kuralını çiğnerdi.[^rc-deref] Herkesin değişikliği görmesi gerekiyorsa `Rc<RefCell<T>>`, yalnızca kendi kopyanızın değişmesi gerekiyorsa aşağıdaki `Rc::make_mut` kullanılır:

```rust
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let v = Rc::new(RefCell::new(vec![1]));
    let oteki_sahip = Rc::clone(&v);

    v.borrow_mut().push(2);
    println!("{:?}", oteki_sahip.borrow());
}
```

Çıktı:

```text
[1, 2]
```

### `Rc<RefCell<T>>`'yi thread'e taşımak (`E0277`)

```rust,compile_fail,E0277
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let r = Rc::new(RefCell::new(1));
    std::thread::spawn(move || {
        *r.borrow_mut() += 1;
    });
}
```

Derleyicinin yanıtı:

```text
error[E0277]: `Rc<RefCell<i32>>` cannot be sent between threads safely
```

`Rc` yerine `Arc` koymak da yetmez; bu kez hata `RefCell`'den gelir:

```rust,compile_fail,E0277
use std::cell::RefCell;
use std::sync::Arc;

fn main() {
    let r = Arc::new(RefCell::new(1));
    std::thread::spawn(move || {
        *r.borrow_mut() += 1;
    });
}
```

Derleyicinin yanıtı:

```text
error[E0277]: `RefCell<i32>` cannot be shared between threads safely
```

Ne `Rc`'nin sayacı ne de `RefCell`'in ödünç sayacı atomiktir: İki thread aynı anda değiştirse sayı bozulabilirdi. Derleyici bu yüzden `Rc`'nin başka thread'e gönderilmesini, `RefCell`'in de thread'ler arasında paylaşılmasını daha derlerken engeller. İkisi aynı şey değildir: Tek sahipli bir `RefCell` başka thread'e taşınabilir, çünkü o zaman ona yalnızca bir thread dokunur; yasak olan, ona aynı anda iki thread'den ulaşmaktır. Çok thread'li karşılık `Arc<Mutex<T>>`'dir:[^thread]

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let ortak = Arc::new(Mutex::new(1));
    let kopya = Arc::clone(&ortak);

    let is = thread::spawn(move || {
        *kopya.lock().expect("kilit zehirlenmedi") += 1;
    });
    is.join().expect("thread panik etmedi");

    println!("{}", ortak.lock().expect("kilit zehirlenmedi"));
}
```

Çıktı:

```text
2
```

Ayrıntısı [15. bölümde](15-concurrency.md), kök nedeni [25. bölümdedir](../ileri/25-send-sync.md).

## `Rc::make_mut`: Paylaşılıyorsa kopyala

Sınıfın ders programı panoda asılı ve herkes ona bakıyor. Siz kendi programınıza bir not eklemek istiyorsunuz. Panodakini karalarsanız herkes etkilenir; bunun yerine fotokopisini çeker, notu kendi kopyanıza yazarsınız. Panoya sizden başka bakan yoksa fotokopiye gerek kalmaz, doğrudan yazarsınız. Bu yönteme *clone-on-write* denir.

Programlarda karşılığı, birçok bileşenin paylaştığı bir yapılandırmadır. Çoğu yalnızca okur; biri kendi kopyasını değiştirmek isterse ötekiler etkilenmemelidir. `RefCell` burada yanlış araçtır, çünkü değişikliği herkes görürdü. `Rc::make_mut(&mut rc)` şöyle davranır:[^make-mut]

- Tek sahip sizseniz doğrudan `&mut T` verir; kopya yoktur.
- Başka güçlü sahipler varsa veriyi klonlar, sizin `Rc`'nizi yeni kopyaya çevirir ve ona `&mut` verir.
- Başka güçlü sahip yok ama `Weak`'ler varsa veriyi klonlamaz, yeni bir bloğa taşır; eski `Weak`'ler artık `None` verir.

```rust
use std::rc::Rc;

fn main() {
    let varsayilan = Rc::new(vec![String::from("tema=açık"), String::from("dil=tr")]);
    let mut kullanici_ayari = Rc::clone(&varsayilan); // şimdilik aynı veri
    println!("önce aynı veri mi? {}", Rc::ptr_eq(&varsayilan, &kullanici_ayari));

    Rc::make_mut(&mut kullanici_ayari)[0] = String::from("tema=koyu"); // klonlandı
    println!("sonra aynı veri mi? {}", Rc::ptr_eq(&varsayilan, &kullanici_ayari));
    println!("varsayılan = {varsayilan:?}");
    println!("kullanıcı = {kullanici_ayari:?}");

    let onceki_adres = Rc::as_ptr(&kullanici_ayari);
    Rc::make_mut(&mut kullanici_ayari).push(String::from("yazı=büyük")); // tek sahip
    println!("kullanıcı = {} ayar", kullanici_ayari.len());
    println!("yerinde mi? {}", Rc::as_ptr(&kullanici_ayari) == onceki_adres);
}
```

Çıktı:

```text
önce aynı veri mi? true
sonra aynı veri mi? false
varsayılan = ["tema=açık", "dil=tr"]
kullanıcı = ["tema=koyu", "dil=tr"]
kullanıcı = 3 ayar
yerinde mi? true
```

İlk çağrıda veri paylaşıldığı için klonlanır ve varsayılan ayarlara dokunulmaz. İkinci çağrıda `kullanici_ayari` artık kendi verisinin tek sahibidir; `make_mut` kopya çıkarmadan yerinde değiştirir. Üçüncü durum, yani yalnızca `Weak` kalması da sınanabilir:

```rust
use std::rc::Rc;

fn main() {
    let mut veri = Rc::new(String::from("a"));
    let zayif = Rc::downgrade(&veri);

    Rc::make_mut(&mut veri).push('b');
    println!("veri = {veri}, zayıf = {:?}", zayif.upgrade());
}
```

Çıktı:

```text
veri = ab, zayıf = None
```

## Akılda kalması gerekenler

- Her işaretçi güvenliğin bedelini başka bir yerde öder: `Box` yalnızca bir tahsisle, `Rc` çalışma zamanında bir sayaçla, `RefCell` ödünç sayacı ve olası bir panikle, `Weak` her kullanımda bir "yaşıyor mu?" sorusuyla. Bedelsiz olan (düz referans ya da `Box`) yetiyorsa bedelli olana geçmeyin.
- `Rc::clone(&x)` yazımı bilinçlidir. `x.clone()` da derlenir, ama okuyana "pahalı bir derin kopya mı?" diye düşündürür. `Rc::clone` yalnızca sayacın arttığını söyler.
- `Cell` panik etmez, `RefCell` edebilir. Küçük `Copy` değerler için `Cell`'i yeğleyin.
- `RefCell` ödünçlerini tek deyime sığdırın. Bir `Ref` ya da `RefMut`'u değişkende tutup araya başka çağrılar koymak, paniğin en sık kaynağıdır.
- `Rc` ile `Rc<RefCell<T>>` tek thread içindir; derleyici `Rc`'yi başka thread'e göndermeyi, `RefCell`'i thread'ler arasında paylaşmayı reddeder.
- Döngü kurabilecek işaretlerden en az biri `Weak` olmalıdır.

## Alıştırmalar

**1.** `Dizin`'e `fn derinlik(&self) -> usize` metodunu ekleyin: Kök 0, çocuğu 1 versin. Özyineleme yerine döngü kullanın. `/home/kaan` ağacında `kaan.derinlik()` 2 olmalıdır. Döngüyü `while let Some(d) = &su_an` diye kurarsanız neden derlenmez?

<details>
<summary>İpucu ve çözüm</summary>

`&su_an` yazarsanız `d`, `su_an`'dan ödünç alınmış olur. Bir sonraki adımda `su_an`'a yazarken bu ödünç hâlâ kullanılmaktadır ve derleyici `E0506` verir:

```rust,compile_fail,E0506
# use std::cell::RefCell;
# use std::rc::Weak;
# pub struct Dizin { ebeveyn: RefCell<Weak<Dizin>> }
# impl Dizin {
fn derinlik(&self) -> usize {
    let mut adim = 0;
    let mut su_an = self.ebeveyn.borrow().upgrade();
    while let Some(d) = &su_an {
        adim += 1;
        su_an = d.ebeveyn.borrow().upgrade();
    }
    adim
}
# }
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0506]: cannot assign to `su_an` because it is borrowed
```

Çözüm `&` işaretini kaldırmaktır. `Some(d)` deseni değeri `su_an`'dan taşır; boşalan `su_an`'a yeniden yazılabilir, `d` de turun sonunda düşürülür. (Referansı korumak isterseniz `let sonraki = d.ebeveyn.borrow().upgrade();` diye bir ara değişken kullanıp atamayı ondan yapabilirsiniz.)

```rust
# use std::cell::RefCell;
# use std::rc::{Rc, Weak};
# pub struct Dizin { ebeveyn: RefCell<Weak<Dizin>>, cocuklar: RefCell<Vec<Rc<Dizin>>> }
# impl Dizin {
#     pub fn yeni() -> Rc<Dizin> {
#         Rc::new(Dizin { ebeveyn: RefCell::new(Weak::new()), cocuklar: RefCell::new(Vec::new()) })
#     }
#     pub fn cocuk_ekle(ebeveyn: &Rc<Dizin>, cocuk: Rc<Dizin>) {
#         *cocuk.ebeveyn.borrow_mut() = Rc::downgrade(ebeveyn);
#         ebeveyn.cocuklar.borrow_mut().push(cocuk);
#     }
# }
impl Dizin {
    pub fn derinlik(&self) -> usize {
        let mut adim = 0;
        let mut su_an = self.ebeveyn.borrow().upgrade();
        while let Some(d) = su_an {
            adim += 1;
            su_an = d.ebeveyn.borrow().upgrade();
        }
        adim
    }
}
# fn main() {
#     let (kok, home, kaan) = (Dizin::yeni(), Dizin::yeni(), Dizin::yeni());
#     Dizin::cocuk_ekle(&kok, Rc::clone(&home));
#     Dizin::cocuk_ekle(&home, Rc::clone(&kaan));
#     assert_eq!((kok.derinlik(), home.derinlik(), kaan.derinlik()), (0, 1, 2));
# }
```

</details>

**2.** Bir olay yayıncısı yazın: `Yayinci { aboneler: RefCell<Vec<Weak<dyn Bildirimci>>> }`. `abone_ol(&self, abone: &Rc<dyn Bildirimci>)` bir `Weak` saklasın; `yayinla(&self, mesaj: &str)` yaşayan abonelere göndersin ve düşmüş olanları listeden temizlesin. Aboneler neden `Rc` değil `Weak`?

<details>
<summary>İpucu ve çözüm</summary>

Yayıncı aboneyi canlı tutmamalıdır; `Rc` saklasaydı abonelikten çıkmayı unutan nesne yayıncı yaşadıkça düşürülmezdi. Önce `retain` ile ölü kayıtlar silinir, sonra kalanlar `upgrade` edilir:

```rust
# use std::cell::{Cell, RefCell};
# use std::rc::{Rc, Weak};
# pub trait Bildirimci { fn gonder(&self, mesaj: &str); }
# #[derive(Default)]
# pub struct KayitliBildirimci { mesajlar: RefCell<Vec<String>>, cagri: Cell<u32> }
# impl Bildirimci for KayitliBildirimci {
#     fn gonder(&self, mesaj: &str) {
#         self.cagri.set(self.cagri.get() + 1);
#         self.mesajlar.borrow_mut().push(mesaj.to_string());
#     }
# }
#[derive(Default)]
pub struct Yayinci {
    aboneler: RefCell<Vec<Weak<dyn Bildirimci>>>,
}

impl Yayinci {
    pub fn abone_ol(&self, abone: &Rc<dyn Bildirimci>) {
        self.aboneler.borrow_mut().push(Rc::downgrade(abone));
    }

    pub fn yayinla(&self, mesaj: &str) {
        self.aboneler.borrow_mut().retain(|w| w.strong_count() > 0);

        // Ödünç bu deyimde biter: abone, gonder içinde abone_ol çağırsa bile panik olmaz.
        let yasayanlar: Vec<Rc<dyn Bildirimci>> =
            self.aboneler.borrow().iter().filter_map(Weak::upgrade).collect();

        for abone in yasayanlar {
            abone.gonder(mesaj);
        }
    }
}

fn main() {
    let yayinci = Yayinci::default();
    let kayit = Rc::new(KayitliBildirimci::default());
    let kalan: Rc<dyn Bildirimci> = Rc::clone(&kayit) as Rc<dyn Bildirimci>;
    let gecici: Rc<dyn Bildirimci> = Rc::new(KayitliBildirimci::default());

    yayinci.abone_ol(&kalan);
    yayinci.abone_ol(&gecici);
    yayinci.yayinla("ilk");
    drop(gecici);
    yayinci.yayinla("ikinci");

    println!("kalan abone = {}", yayinci.aboneler.borrow().len());
    println!("{:?}", kayit.mesajlar.borrow());
}
```

Çıktı:

```text
kalan abone = 1
["ilk", "ikinci"]
```

Yaşayanları önce ayrı bir `Vec`'e toplamak, ilk hatadaki tuzağı önler: `gonder` çağrıları sırasında `aboneler` üzerinde açık bir ödünç kalmaz.

</details>

**3.** `cift_olustur(false)` neden 0 döndürüyor? `a` ve `b`'nin güçlü sayaçlarını adım adım yazın. Sonra döngüyü `Weak` kullanmadan, kapsam bitmeden elle kırın: Döngü kurulduktan sonra `a.guclu_komsu.borrow_mut().take();` satırını ekleyince sonuç kaç olur?

<details>
<summary>İpucu ve çözüm</summary>

| Adım | `a`'nın sayacı | `b`'nin sayacı |
| --- | --- | --- |
| İki düğüm oluştu | 1 | 1 |
| `a`, `b`'yi sahiplendi | 1 | 2 |
| `b`, `a`'yı sahiplendi | 2 | 2 |
| `b` adı düşürüldü | 2 | 1 |
| `a` adı düşürüldü | 1 | 1 |

İki sayaç da 1'de kalır, hiçbir düğüm düşürülmez. `take()` ise `a`'nın tuttuğu `Rc`'yi alıp hemen düşürür; `b`'nin sayacı 1'e iner ve döngü kırılır. Kapsam sonunda iki düğüm de düşürülür, sonuç 2 olur:

```rust
# use std::cell::{Cell, RefCell};
# use std::rc::Rc;
# struct Dugum { dusme_sayaci: Rc<Cell<u32>>, guclu_komsu: RefCell<Option<Rc<Dugum>>> }
# impl Dugum {
#     fn yeni(sayac: &Rc<Cell<u32>>) -> Rc<Dugum> {
#         Rc::new(Dugum { dusme_sayaci: Rc::clone(sayac), guclu_komsu: RefCell::new(None) })
#     }
# }
# impl Drop for Dugum {
#     fn drop(&mut self) { self.dusme_sayaci.set(self.dusme_sayaci.get() + 1); }
# }
fn main() {
    let sayac = Rc::new(Cell::new(0));
    {
        let a = Dugum::yeni(&sayac);
        let b = Dugum::yeni(&sayac);
        *a.guclu_komsu.borrow_mut() = Some(Rc::clone(&b));
        *b.guclu_komsu.borrow_mut() = Some(Rc::clone(&a));

        a.guclu_komsu.borrow_mut().take(); // döngü elle kırıldı
        println!("a = {}, b = {}", Rc::strong_count(&a), Rc::strong_count(&b));
    }
    println!("düşen düğüm = {}", sayac.get());
}
```

Çıktı:

```text
a = 2, b = 1
düşen düğüm = 2
```

Elle kırmak işe yarar, ama unutulabilir; `Weak` ise doğruluğu türün kendisine yazar.

</details>

## Özet

- İşaretçiyi üç soruyla seçin: Kaç sahip var, değer değişecek mi, yalnızca yerini bilmek yetiyor mu?
- Ağaçlarda ebeveynden çocuğa `Rc`, çocuktan ebeveyne `Weak` kullanılır; düğümler sonradan değişecekse alanlar `RefCell` içine alınır.
- Birbirini `Rc` ile tutan değerler sızar; derleyici bunu yakalamaz.
- İç değiştirilebilirlik, `&self` arkasında kayıt tutmayı sağlar: küçük değerler için `Cell`, yerinde değişenler için `RefCell`.
- `RefCell` panikleri geniş tutulan ödünçlerden doğar; ödüncü tek deyime sığdırın.
- `Rc::make_mut`, paylaşılan veriyi yalnızca gerektiğinde kopyalar.

Aynı üç sorunun çok thread'li yanıtları `Arc`, `Mutex` ve `RwLock`'tur; sıradaki bölüm [15. Eşzamanlılık](15-concurrency.md) onları anlatıyor. Hücrelerin altındaki ortak temel olan `UnsafeCell` ile `Rc`'nin neden `Send` olmadığı [25. bölümde](../ileri/25-send-sync.md), referans tutan türlerde düşürme sırası [26. bölümde](../ileri/26-dropck.md) ele alınacak.

## Kaynaklar

[^secim]: The Rust Programming Language, [Enforcing Borrowing Rules at Runtime](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html#enforcing-borrowing-rules-at-runtime) (`Box`, `Rc` ve `RefCell` karşılaştırması); std, [When to choose interior mutability](https://doc.rust-lang.org/std/cell/index.html#when-to-choose-interior-mutability); The Rust Programming Language, [Comparing `RefCell<T>`/`Rc<T>` and `Mutex<T>`/`Arc<T>`](https://doc.rust-lang.org/book/ch16-03-shared-state.html#comparing-refcelltrct-and-mutextarct).
[^agac]: The Rust Programming Language, [Adding a Reference from a Child to Its Parent](https://doc.rust-lang.org/book/ch15-06-reference-cycles.html#adding-a-reference-from-a-child-to-its-parent) ve [Visualizing Changes to `strong_count` and `weak_count`](https://doc.rust-lang.org/book/ch15-06-reference-cycles.html#visualizing-changes-to-strong_count-and-weak_count); std, [`Weak::upgrade`](https://doc.rust-lang.org/std/rc/struct.Weak.html#method.upgrade).
[^dongu]: The Rust Programming Language, [Creating a Reference Cycle](https://doc.rust-lang.org/book/ch15-06-reference-cycles.html#creating-a-reference-cycle); The Rustonomicon, [Leaking](https://doc.rust-lang.org/nomicon/leaking.html).
[^hucre]: std, [`std::cell`: `Cell<T>`](https://doc.rust-lang.org/std/cell/index.html#cellt) ve [`RefCell<T>`](https://doc.rust-lang.org/std/cell/index.html#refcellt).
[^cell]: std, [`Cell::get`](https://doc.rust-lang.org/std/cell/struct.Cell.html#method.get), [`Cell::replace`](https://doc.rust-lang.org/std/cell/struct.Cell.html#method.replace) ve [`Cell::take`](https://doc.rust-lang.org/std/cell/struct.Cell.html#method.take).
[^mock]: The Rust Programming Language, [Testing with Mock Objects](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html#testing-with-mock-objects).
[^tasma]: The Rust Reference, [Operator expressions: Overflow](https://doc.rust-lang.org/reference/expressions/operator-expr.html#overflow); std, [`u32::saturating_add`](https://doc.rust-lang.org/std/primitive.u32.html#method.saturating_add).
[^try]: std, [`RefCell::try_borrow_mut`](https://doc.rust-lang.org/std/cell/struct.RefCell.html#method.try_borrow_mut) ve [`RefCell::borrow_mut`](https://doc.rust-lang.org/std/cell/struct.RefCell.html#method.borrow_mut).
[^gecici]: The Rust Reference, [Destructors: Temporary scopes](https://doc.rust-lang.org/reference/destructors.html#temporary-scopes); hata dizini, [`E0716`](https://doc.rust-lang.org/error_codes/E0716.html).
[^rc-deref]: std 1.99.0 kaynak kodu, [`rc.rs`, `Deref for Rc`](https://github.com/rust-lang/rust/blob/1.99.0/library/alloc/src/rc.rs#L2436); hata dizini, [`E0594`](https://doc.rust-lang.org/error_codes/E0594.html) ve [`E0596`](https://doc.rust-lang.org/error_codes/E0596.html).
[^thread]: The Rust Programming Language, [Multiple Ownership with Multiple Threads](https://doc.rust-lang.org/book/ch16-03-shared-state.html#multiple-ownership-with-multiple-threads); std 1.99.0 kaynak kodu, [`rc.rs`, `!Send for Rc`](https://github.com/rust-lang/rust/blob/1.99.0/library/alloc/src/rc.rs#L334) ve [`cell.rs`, `!Sync for RefCell`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/cell.rs#L1442).
[^make-mut]: std, [`Rc::make_mut`](https://doc.rust-lang.org/std/rc/struct.Rc.html#method.make_mut), [`Rc::ptr_eq`](https://doc.rust-lang.org/std/rc/struct.Rc.html#method.ptr_eq) ve [`Rc::as_ptr`](https://doc.rust-lang.org/std/rc/struct.Rc.html#method.as_ptr); std 1.99.0 kaynak kodu, [`rc.rs`, `make_mut`](https://github.com/rust-lang/rust/blob/1.99.0/library/alloc/src/rc.rs#L2084).
