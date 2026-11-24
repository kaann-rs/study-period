# 12. Closure ve Iterator'lar

Bir listedeki fiyatlara vergi eklemek, pahalı siparişleri ayıklamak ya da tutarları toplamak istediğinizi düşünün. Her iş için ayrı bir fonksiyon ve ayrı bir döngü yazabilirsiniz; ama fonksiyona ihtiyaç duyduğu her değeri parametreyle taşımanız, döngünün içinde de süzmeyi, dönüştürmeyi ve toplamayı birbirine karıştırmanız gerekir. Kod uzadıkça hangi satırın ne için yazıldığı zor seçilir.

Rust bu iş için iki araç sunar. *Closure*, yazıldığı yerdeki değişkenleri hatırlayan adsız, küçük bir fonksiyondur. *Iterator* ise öğeleri tek tek ve istendikçe veren bir sıradır. İkisi birlikte kullanıldığında uzun döngüler, yukarıdan aşağıya okunan kısa adımlara dönüşür.

Bu bölümde önce closure'ların ortamlarını nasıl yakaladığını ve `Fn`, `FnMut`, `FnOnce` trait'lerini göreceğiz. Ardından fonksiyon işaretçilerini, iterator'ların çalışma biçimini ve kendi iterator'ınızı yazmayı ele alacağız. Her iki araç da sahiplik ve ödünç kurallarına tam olarak uyar; hataların çoğu [3. bölümden](../basics/03-ownership.md) tanıdık gelecektir.

> [!NOTE]
> Bu bölümün kodu `learn/closures-iterators` dizinindedir. Depo kökünden `cargo run -p closures-iterators` komutuyla çalıştırabilirsiniz.

## Günlük hayattan bir benzetme

Bir kargo şubesinde çalıştığınızı düşünün. İş arkadaşınıza bir görev notu bırakıyorsunuz: "Gelen her paketin fiyatına masadaki listede yazan vergiyi ekle." Not kısadır, adı yoktur ve en önemlisi, yazıldığı yerdeki bir şeyi, yani masadaki listeyi bilir. Cebinizde dururken de bunu unutmaz. Closure bu nottur.

Paketler ise bir taşıma bandından gelir. Bandın ucunda durup "sıradaki" dersiniz, bant bir paket verir; paket kalmayınca "bitti" der. Iterator bu banttır. Bandın üzerine istasyonlar kurabilirsiniz: Biri paketi başka bir şeye çevirir (`map`), biri bazılarını ayıklar (`filter`), sonuncusu kalanları bir kutuya koyar (`collect`).

Bant tembeldir (lazy): Ucunda kimse "sıradaki" demedikçe tek bir paket bile üretmez. "Kırmızı olanları ayır" notunu yazıp bandı çalıştırdığınızda paketler tek tek önünüzden geçer; kırmızıları alır, mavileri geçer, en sonunda aldıklarınızı bir kutuya koyarsınız.

<details>
<summary>Bu bölümde geçen terimler</summary>

- **closure:** `|x| x + 1` gibi yazılan, adı olmayan küçük fonksiyon.
- **ortam ve yakalama (environment, capture):** Closure'ın yazıldığı yerdeki değişkenler ve closure'ın onları içine alması.
- **ödünç (`&` ve `&mut`):** Veriye sahibi olmadan bakmak ya da onu değiştirmek.
- **`move`:** Closure'a "yakaladıklarını ödünç alma, içine taşı" diyen anahtar sözcük.
- **trait:** Bir türün "şunu yapabilirim" diye verdiği söz ([9. bölüm](09-generics-traits-lifetimes.md)).
- **`Fn`, `FnMut`, `FnOnce`:** Closure'ların uyguladığı üç trait. Sırasıyla, yakaladığını yalnızca okuyan, değiştiren ve en fazla bir kez çağrılabilen closure'ları anlatır.
- **fonksiyon işaretçisi (function pointer):** Bir fonksiyonun adresini tutan, `fn(i8) -> i8` gibi bir türü olan değer.
- **iterator:** Öğeleri sırayla, tek tek veren değer.
- **`next()`:** "Sıradakini ver" isteği. Öğe varsa `Some(öğe)`, bittiyse `None` döner.
- **tembel (lazy):** İstenmedikçe iş yapmayan.
- **dönüştürücü (adapter):** Bir iterator'ı alıp başka bir iterator veren ara adım; `map` ve `filter` gibi.
- **tüketici (consumer):** Sırayı çalıştırıp sonuç üreten adım; `collect` ve `sum` gibi.
- **`map`, `filter`:** Her öğeyi dönüştürmek; koşula uyanları bırakmak.
- **`collect`, `sum`:** Öğeleri bir koleksiyonda toplamak; öğelerin aritmetik toplamını almak.
- **`zip`, `skip`:** İki sırayı öğe öğe eşlemek; baştan belirli sayıda öğeyi atlamak.
- **`iter`, `iter_mut`, `into_iter`:** Öğelere bakarak, onları değiştirerek ya da sahiplenerek gezmek.
- **`Item` (ilişkili tür):** Bir iterator'ın verdiği öğenin türü.
- **aliasing XOR mutability:** "Ya birçok okuyan ya tek bir yazan" kuralı ([3. bölüm](../basics/03-ownership.md)).
- **thread:** Programın içinde aynı anda çalışabilen ayrı bir iş kolu ([15. bölüm](15-concurrency.md)).
- **API:** Bir fonksiyonun ya da kütüphanenin dışarıya açtığı kullanım biçimi.
- **sıfır maliyet (zero-cost):** Sağlanan kolaylığın çalışan programı yavaşlatmaması.
- **monomorfizasyon ve satır içine alma (inline):** Derleyicinin jenerik kodu her somut tür için ayrı üretmesi; çağrı yerine fonksiyonun gövdesini koyması.
- **LLVM:** rustc'nin makine kodu üretirken kullandığı altyapı.

</details>

## Closure: Ortamını tanıyan fonksiyon

Closure, parametreleri dik çizgiler arasına yazılan bir fonksiyondur: `|parametreler| gövde`. Sıradan bir fonksiyondan asıl farkı, yazıldığı yerdeki değişkenleri kullanabilmesidir. Buna *yakalama* (capture) denir:[^yakalama]

```rust
fn main() {
    let vergi = 20;
    let fiyat = |x: u32| x + x * vergi / 100; // vergi çevreden alınır

    println!("{}", fiyat(100));
}
```

Çıktı:

```text
120
```

`vergi`, `fiyat`'ın parametresi değildir; closure onu çevresinden almıştır. Aynı işi içeride tanımlanan bir `fn` ile yapmaya çalışırsanız program derlenmez, çünkü `fn` yalnızca kendisine parametre olarak verileni görür:

```rust,compile_fail,E0434
fn main() {
    let vergi = 20;
    fn fiyat2(x: u32) -> u32 {
        x + x * vergi / 100
    }
    println!("{}", fiyat2(100));
}
```

Derleyicinin yanıtı:

```text
error[E0434]: can't capture dynamic environment in a fn item
  |
4 |         x + x * vergi / 100
  |                 ^^^^^
  |
  = help: use the `|| { ... }` closure form instead
```

Closure olmasaydı, böyle küçük bir iş için bile adlı bir fonksiyon yazmanız ve ihtiyaç duyduğu her değeri ona tek tek parametreyle taşımanız gerekirdi.

### Türler çoğunlukla çıkarılır

Fonksiyonların parametre ve dönüş türleri açıkça yazılır, çünkü imza başkalarının kullanacağı bir sözleşmedir. Closure'lar ise çoğunlukla kısa bir bağlamda, hemen orada kullanılır; derleyici türleri kullanımdan çıkarabilir.[^cikarim] Aşağıdaki dört tanım aynı işi yapar:

```rust
fn kare_fn(x: i32) -> i32 { x * x }

fn main() {
    let kare_a = |x: i32| -> i32 { x * x };
    let kare_b = |x: i32| { x * x };
    let kare = |x: i32| x * x;

    println!("kare(5) = {}", kare(5));
    println!("{} {} {}", kare_fn(5), kare_a(5), kare_b(5));
}
```

Çıktı:

```text
kare(5) = 25
25 25 25
```

Tür çıkarımı closure'ı jenerik yapmaz. Her parametre için *tek bir* somut tür seçilir ve bu tür, closure'ın ilk kullanımıyla kesinleşir. `aynen` aşağıda önce `String` ile çağrıldığı için artık sayı kabul etmez:

```rust,compile_fail,E0308
fn main() {
    let aynen = |x| x;
    let s = aynen(String::from("merhaba"));
    let n = aynen(5);
    println!("{s} {n}");
}
```

Derleyicinin yanıtı:

```text
error[E0308]: mismatched types
  |
4 |     let n = aynen(5);
  |             ----- ^ expected `String`, found integer
  |
note: expected because the closure was earlier called with an argument of type `String`
```

## Ortamı yakalamanın üç yolu

Görev notu masadaki listeyi üç biçimde kullanabilir. "Listeye bak" derseniz liste masada kalır, siz de bakabilirsiniz. "Listeye bir satır ekle" derseniz not işini bitirene kadar listeye başkası dokunamaz. "Listeyi yanına al" derseniz liste artık notu taşıyanındır ve masa boşalır.

Closure'lar da ortamlarını bu üç yoldan biriyle yakalar: paylaşımlı ödünç (`&`), değiştirilebilir ödünç (`&mut`) ya da taşıma. Hangisinin gerektiğine derleyici, gövdenin değişkenle ne yaptığına bakarak karar verir ve yeterli olan ilk biçimi seçer; gerekenden fazlasını almaz.[^bicimler]

### Paylaşımlı ödünç

Gövde değişkeni yalnızca okuyorsa yakalama `&` ile yapılır:

```rust
fn main() {
    let liste = vec![1, 2, 3];
    let yazdir = || println!("yakaladı (&): {liste:?}");

    yazdir();
    yazdir(); // istenildiği kadar çağrılabilir
    println!("liste hâlâ burada: {liste:?}");
}
```

Çıktı:

```text
yakaladı (&): [1, 2, 3]
yakaladı (&): [1, 2, 3]
liste hâlâ burada: [1, 2, 3]
```

Aynı anda birden çok paylaşımlı referans bulunabildiği için `liste`'yi closure'ın yanında siz de okuyabilirsiniz.

### Değiştirilebilir ödünç

Gövde değişkeni değiştiriyorsa yakalama `&mut` ile yapılır. Closure'ı çağırmak artık onun tuttuğu durumu değiştirmek demektir; bu yüzden closure'ı tutan değişken de `mut` olmalıdır:

```rust
fn main() {
    let mut sayac = vec![1, 2, 3];
    let mut ekle = || sayac.push(4);

    ekle();
    println!("ekle sonrası: {sayac:?}");
}
```

Çıktı:

```text
ekle sonrası: [1, 2, 3, 4]
```

`mut` unutulursa derleyici closure'ın çağrılamayacağını söyler:

```rust,compile_fail,E0596
fn main() {
    let mut sayac = vec![1, 2, 3];
    let ekle = || sayac.push(4);
    ekle();
    println!("{sayac:?}");
}
```

Derleyicinin yanıtı:

```text
error[E0596]: cannot borrow `ekle` as mutable, as it is not declared as mutable
  |
3 |     let ekle = || sayac.push(4);
  |                   ----- calling `ekle` requires mutable binding due to mutable borrow of `sayac`
4 |     ekle();
  |     ^^^^ cannot borrow as mutable
```

Closure, yakaladığı ödüncü son kullanıldığı yere kadar elinde tutar. Tanım ile son çağrı arasında `sayac`'ı başka bir yerden okumaya çalışırsanız bildik ödünç kuralı devreye girer:

```rust,compile_fail,E0502
fn main() {
    let mut sayac = vec![1, 2, 3];
    let mut ekle = || sayac.push(4);
    println!("{sayac:?}");
    ekle();
}
```

Derleyicinin yanıtı:

```text
error[E0502]: cannot borrow `sayac` as immutable because it is also borrowed as mutable
  |
3 |     let mut ekle = || sayac.push(4);
  |                    -- ----- first borrow occurs due to use of `sayac` in closure
  |                    |
  |                    mutable borrow occurs here
4 |     println!("{sayac:?}");
  |                ^^^^^ immutable borrow occurs here
5 |     ekle();
  |     ---- mutable borrow later used here
```

Çözüm, okumayı closure'ın son çağrısından sonraya bırakmaktır. Çalışan örnekte `println!` tam da bu yüzden `ekle()` çağrısından sonra gelir.

### `move` ile taşıma

Gövde yalnızca okusa bile, yakalananların sahipliğini closure'a vermek isteyebilirsiniz. Bunun için dik çizgilerin önüne `move` yazılır:[^move]

```rust
fn main() {
    let sahip = String::from("sahiplendim");
    let tasi = move || println!("move ile: {sahip}");

    tasi();
}
```

Çıktı:

```text
move ile: sahiplendim
```

`sahip` artık closure'ın içindedir; dışarıdan kullanılamaz:

```rust,compile_fail,E0382
fn main() {
    let sahip = String::from("sahiplendim");
    let tasi = move || println!("move ile: {sahip}");
    tasi();
    println!("{sahip}");
}
```

Derleyicinin yanıtı:

```text
error[E0382]: borrow of moved value: `sahip`
  |
2 |     let sahip = String::from("sahiplendim");
  |         ----- move occurs because `sahip` has type `String`, which does not implement the `Copy` trait
3 |     let tasi = move || println!("move ile: {sahip}");
  |                -------                      ----- variable moved due to use in closure
  |                |
  |                value moved into closure here
4 |     tasi();
5 |     println!("{sahip}");
  |                ^^^^^ value borrowed here after move
```

`move` yazmasaydınız derleyici yeterli olan en dar biçimi seçer, `sahip` yalnızca ödünç alınır ve son satır sorunsuz derlenirdi. Değer dışarıda da gerekliyse `move`'u kaldırın ya da closure'a değerin bir klonunu verin.

`Copy` türlerde `move`, değeri içeri *kopyalar*. Closure bundan sonra kendi kopyasıyla çalışır ve dışarıdaki değişkeni etkilemez:

```rust
fn main() {
    let mut n = 1;
    let mut artir = move || {
        n += 1;
        n
    };

    println!("{} {}", artir(), artir());
    println!("dışarıdaki n = {n}");
}
```

Çıktı:

```text
2 3
dışarıdaki n = 1
```

### Closure, ödünç aldığı değerden uzun yaşayacaksa

`move`'un en sık gerektiği yer, closure'ın yazıldığı fonksiyonun ömrünü aşabildiği durumlardır: closure'ı başka bir thread'e vermek ya da fonksiyondan döndürmek. Yeni bir thread, onu başlatan fonksiyondan uzun yaşayabilir. Closure yerel bir değişkeni yalnızca ödünç almış olsaydı, fonksiyon bittiğinde silinmiş belleğe bakan sarkan bir referans tutardı. Derleyici buna izin vermez:

```rust,compile_fail,E0373
use std::thread;

fn main() {
    let liste = vec![1, 2, 3];
    let is = thread::spawn(|| println!("thread: {liste:?}"));
    is.join().expect("thread panikledi");
}
```

Derleyicinin yanıtı:

```text
error[E0373]: closure may outlive the current function, but it borrows `liste`, which is owned by the current function
  |
5 |     let is = thread::spawn(|| println!("thread: {liste:?}"));
  |                            ^^                    ----- `liste` is borrowed here
  |                            |
  |                            may outlive borrowed value `liste`
  |
note: function requires argument type to outlive `'static`
help: to force the closure to take ownership of `liste` (and any other referenced variables), use the `move` keyword
```

Buradaki `join` çağrısı thread'i bekletse de derleyici bunu hesaba katmaz: `thread::spawn`'ın imzası, closure'ın programın sonuna kadar geçerli kalabilmesini (`'static`) ister.[^spawn] Yardım satırının önerdiği gibi `move` yazmak yeterlidir:

```rust
use std::thread;

fn main() {
    let liste = vec![1, 2, 3];
    let is = thread::spawn(move || println!("thread: {liste:?}"));
    is.join().expect("thread panikledi");
}
```

Çıktı:

```text
thread: [1, 2, 3]
```

Closure döndüren fonksiyonları [sonraki sayfada](12-closures-iterators-deep-dive.md), thread'leri [15. bölümde](15-concurrency.md) göreceğiz.

## `Fn`, `FnMut` ve `FnOnce`

Şubede üç çeşit görev notu vardır. "Panodaki saati oku" notunu istediğiniz kadar uygularsınız, hiçbir şey değişmez. "Sayaca bir ekle" notu da tekrar tekrar uygulanabilir, ama her seferinde sayaç değişir. "Bu paketi alıcısına teslim et" notunda ise paket verilince elinizde kalmaz; bu not en fazla bir kez uygulanabilir.

Rust'ta bu üç çeşidin adı `Fn`, `FnMut` ve `FnOnce`'tır. Bir closure'ın hangisini uyguladığı, gövdesinin yakaladıklarıyla ne yaptığına bağlıdır:[^traitler]

- **`FnOnce`:** Closure'ı çağırmak onu tüketir; bu yüzden en fazla bir kez çağrılabilir. Bütün closure'lar `FnOnce`'ı uygular. Yakaladığı bir değeri dışarı taşıyan closure ise *yalnızca* `FnOnce`'tır.
- **`FnMut`:** Yakaladığını dışarı taşımayan closure'lar uygular. Birden çok kez çağrılabilir ve her çağrı yakalananı değiştirebilir; çağırmak için `&mut` erişim gerekir.
- **`Fn`:** Yakaladığını ne dışarı taşıyan ne de değiştiren closure'lar ile hiçbir şey yakalamayan closure'lar uygular. Paylaşımlı bir referans üzerinden istenildiği kadar çağrılabilir.

```rust
fn main() {
    let oku = || println!("saat 9"); // Fn
    oku();
    oku();

    let mut sayac = 0;
    let mut artir = || sayac += 1; // FnMut
    artir();
    artir();
    println!("sayac = {sayac}");

    let paket = String::from("kitap");
    let teslim = move || paket; // FnOnce: paketi dışarı verir
    println!("teslim edilen: {}", teslim());
}
```

Çıktı:

```text
saat 9
saat 9
sayac = 2
teslim edilen: kitap
```

Üç trait iç içedir: `Fn`, `FnMut`'ın; `FnMut` da `FnOnce`'ın alt trait'idir. Dolayısıyla `Fn` uygulayan bir closure, `FnMut` ya da `FnOnce` beklenen yere de verilebilir; tersi geçerli değildir.

### Tüketen closure iki kez çağrılamaz

Yakaladığı değeri dışarı taşıyan bir closure'ın ilk çağrısı, closure'ın kendisini de tüketir:

```rust,compile_fail,E0382
fn main() {
    let s = String::from("x");
    let tuket = move || drop(s);
    tuket();
    tuket();
}
```

Derleyicinin yanıtı:

```text
error[E0382]: use of moved value: `tuket`
  |
4 |     tuket();
  |     ------- `tuket` moved due to this call
5 |     tuket();
  |     ^^^^^ value used here after move
  |
note: closure cannot be invoked more than once because it moves the variable `s` out of its environment
note: this value implements `FnOnce`, which causes it to be moved when called
```

`drop(s)`, `s`'nin sahipliğini alır ve onu düşürür. İkinci çağrıya izin verilseydi serbest bırakılmış bir `String` yeniden düşürülmeye çalışılır, yani serbest bırakıldıktan sonra kullanım ve çifte serbest bırakma ortaya çıkardı. Bu hatada taşınan, `s` değil `tuket`'in kendisidir; mesaj da bunu söyler.

### İmzada closure istemek

Closure alan bir fonksiyon, parametresine trait sınırı koyarak closure'a ne kadar yetki verdiğini söyler:

```rust
// Closure'ı birden çok kez, paylaşımlı olarak çağırır.
fn uygula_fn<F: Fn()>(f: F) {
    f();
    f();
}

// Closure'ı en fazla bir kez çağırır ve sonucunu döndürür.
fn hesapla_sonra_calistir<F: FnOnce() -> i32>(f: F) -> i32 {
    f()
}

fn main() {
    uygula_fn(|| println!("Fn: sadece okur"));

    let n = hesapla_sonra_calistir(|| 7 * 6);
    println!("FnOnce döndürdü: {n}");
}
```

Çıktı:

```text
Fn: sadece okur
Fn: sadece okur
FnOnce döndürdü: 42
```

`|| 7 * 6` closure'ı `Fn`'i de uygular, ama `FnOnce` bekleyen fonksiyon için bu fazlasıyla yeterlidir. `FnOnce` sınırı fonksiyona "çağır" demez, "birden çok çağırma" der; fonksiyon closure'ı hiç çağırmayabilir de. Standart kütüphaneden bir örnek `Option::unwrap_or_else`'tir: Değer varsa closure hiç çalışmaz.[^unwrap]

```rust
fn main() {
    let var: Option<i32> = Some(1);
    let yok: Option<i32> = None;

    let hesapla = || {
        println!("closure çalıştı");
        0
    };
    println!("{}", var.unwrap_or_else(hesapla));
    println!("{}", yok.unwrap_or_else(hesapla));
}
```

Çıktı:

```text
1
closure çalıştı
0
```

### Neden üç ayrı trait?

Tek bir "çağrılabilir" trait'i olsaydı iki şeyden biri bozulurdu. Ya her closure'ın istenildiği kadar çağrılmasına izin verilirdi ve tüketen bir closure'ı iki kez çağırmak çifte serbest bırakmaya yol açardı; ya da her closure yalnızca bir kez çağrılabilirdi ve sıradan bir `map` bile yazılamazdı. Aynı sorun değiştirme için de vardır: Değiştiren bir closure'a aynı anda iki yerden ulaşılabilseydi `&mut` referansın tekliği çiğnenirdi. Bu erişim farklı thread'lerden gelseydi sonuç veri yarışı olurdu. Üçlü ayrım, closure'ın yetkisini türün parçası hâline getirir ve bu çağrıların hepsini derleme zamanında durdurur.

Değiştiren bir closure'ı, onu paylaşımlı çağıracak bir fonksiyona vermeyi deneyelim:

```rust,compile_fail,E0525
fn uygula_fn<F: Fn()>(f: F) {
    f();
    f();
}

fn main() {
    let mut n = 0;
    let artir = || n += 1;
    uygula_fn(artir);
    println!("{n}");
}
```

Derleyicinin yanıtı:

```text
error[E0525]: expected a closure that implements the `Fn` trait, but this closure only implements `FnMut`
  |
8 |     let artir = || n += 1;
  |                 ^^ - closure is `FnMut` because it mutates the variable `n` here
  |                 |
  |                 this closure implements `FnMut`, not `Fn`
9 |     uygula_fn(artir);
  |     --------- ----- the requirement to implement `Fn` derives from here
```

`uygula_fn` closure'ı gerçekten paylaşımlı çağırmak zorunda değilse çözüm, imzayı `F: FnMut()` olarak gevşetmektir. Genel kural şudur: Parametre alırken işinizi gören *en gevşek* trait'i isteyin. Bir kez çağıracaksanız `FnOnce`, defalarca çağıracaksanız `FnMut`, aynı anda birden çok yerden ya da yalnızca `&` üzerinden çağıracaksanız `Fn`. Gevşek sınır, çağıranın verebileceği closure'ların kümesini genişletir.

> [!TIP]
> Güvenli (`unsafe` olmayan) fonksiyonlar ve fonksiyon işaretçileri üç trait'i de uygular. Closure beklenen her yere, imzası uyan bir fonksiyonun adını da verebilirsiniz: `.map(ToString::to_string)` ya da `.map(Some)` gibi.[^fn-isaretcisi]

## Fonksiyon işaretçileri

Her closure'ın, derleyicinin ürettiği ve sizin adını yazamadığınız kendine özgü bir türü vardır. Buna karşılık `fn(i8) -> i8` gibi bir *fonksiyon işaretçisi* tek ve adı olan bir türdür. Küçük harfli `fn` tür, büyük harfli `Fn` ise trait'tir. Hiçbir şey yakalamayan bir closure, imzası uyan bir fonksiyon işaretçisine örtük olarak dönüşür (coercion).[^donusum] Aşağıdaki `hex` her iki biçimi de kabul eder: İlk parametre jenerik bir çağrılabilir, ikincisi somut bir fonksiyon işaretçisidir.

```rust
struct Color<F>
where
    F: Fn(i8) -> i8,
{
    x: F,
}

impl<F> Color<F>
where
    F: Fn(i8) -> i8,
{
    // İki biçimi de kabul eder, ama yalnızca f'yi çağırır.
    fn hex(f: F, _b: fn(i8) -> i8) -> i8 {
        f(1)
    }
}

fn kimlik(y: i8) -> i8 {
    y
}

fn main() {
    let x = |y: i8| -> i8 { y }; // ortamdan hiçbir şey yakalamaz

    let a: i8 = Color::hex(x, x);
    let b = Color::hex(x, x); // i8, hex'in dönüş türünden çıkarılır
    println!("a = {a}, b = {b}");

    println!("c = {}", Color::<fn(i8) -> i8>::hex(kimlik, kimlik));

    let k = 10;
    let renk = Color { x: |y: i8| y + k }; // ortamı yakalayan closure
    println!("renk = {}", (renk.x)(1));
}
```

Çıktı:

```text
a = 1, b = 1
c = 1
renk = 11
```

`Color::hex(x, x)` çağrısında ilk `x` kendi closure türünü korur ve `F`'yi belirler; ikinci `x` ise `fn(i8) -> i8` türüne dönüşür. Aynı closure'ı iki kez verebilmemiz, hiçbir şey yakalamadığı için `Copy` olmasındandır. Üçüncü çağrıda `F` açıkça `fn(i8) -> i8` olarak seçilmiştir: Fonksiyon işaretçileri `Fn` trait'ini uyguladığı için `Color`'ın koyduğu sınırı karşılar. Son satırdaki parantezlerin nedenini [sonraki sayfada](12-closures-iterators-deep-dive.md) göreceğiz.

Ortamı yakalayan bir closure fonksiyon işaretçisine dönüşemez. İşaretçi yalnızca kodun adresidir; yakalanan değerleri koyacak yeri yoktur:

```rust,compile_fail,E0308
fn uygula(f: fn(i8) -> i8) -> i8 {
    f(1)
}

fn main() {
    let k = 10;
    println!("{}", uygula(|y| y + k));
}
```

Derleyicinin yanıtı:

```text
error[E0308]: mismatched types
  |
7 |     println!("{}", uygula(|y| y + k));
  |                    ------ ^^^^^^^^^ expected fn pointer, found closure
  |
note: closures can only be coerced to `fn` types if they do not capture any variables
  |
7 |     println!("{}", uygula(|y| y + k));
  |                                   ^ `k` captured here
```

### Neden hem jenerik `F` hem `fn`?

İki biçimin güçlü yanları farklıdır. Yalnızca jenerik `F` olsaydı farklı fonksiyonları tek bir değişkende ya da koleksiyonda tutamazdınız: `Vec<F>` tek bir closure türü alır. Fonksiyon işaretçisi ise tek bir somut tür olduğu için hepsini kabul eder:

```rust
fn iki_kat(x: i32) -> i32 {
    x * 2
}

fn main() {
    let islemler: Vec<fn(i32) -> i32> = vec![iki_kat, |x| x + 1, i32::abs];
    let sonuclar: Vec<i32> = islemler.iter().map(|f| f(-5)).collect();

    println!("{sonuclar:?}");
}
```

Çıktı:

```text
[-10, -4, 5]
```

Fonksiyon işaretçisinin gerekli olduğu ikinci yer, başka bir dille konuşmaktır (FFI). C fonksiyonları başka fonksiyonları parametre olarak alabilir, ama C'de closure yoktur.[^ffi] Bu yüzden C'ye verilecek geri çağrılar, C çağrı kuralını belirten `extern "C" fn` türündedir:

```rust
extern "C" fn ikiye_katla(x: i32) -> i32 {
    x * 2
}

fn main() {
    let geri_cagri: extern "C" fn(i32) -> i32 = ikiye_katla;
    println!("{}", geri_cagri(21));
}
```

Çıktı:

```text
42
```

Yalnızca fonksiyon işaretçisi olsaydı bu kez ortam yakalanamazdı. Ayrıca işaretçi üzerinden yapılan çağrıda hedef, genel olarak ancak çalışma zamanında bilinir; derleyici hedefi kanıtlayamadığı sürece gövdeyi çağrı yerine koyamaz (inline). Jenerik `F` ise her closure için ayrı kod üretir ve çağrı derleme zamanında çözülür. Kısacası jenerik `F` yakalama ve satır içine alma olanağı, `fn` ise saklanabilen ve FFI'da kullanılabilen tek bir tür sağlar; duruma göre seçersiniz. Yakalayan farklı closure'ları bir arada tutmanın yolu olan `Box<dyn Fn>` [sonraki sayfanın](12-closures-iterators-deep-dive.md) konusudur.

## Iterator: Sıradakini istemek

Taşıma bandının tek bir düğmesi vardır: "sıradaki". Rust'ta bu düğme `next` metodudur ve `Iterator` trait'inde yazmak zorunda olduğunuz tek metottur:[^iterator]

```rust
pub trait Iterator {
    type Item; // verilen öğenin türü

    fn next(&mut self) -> Option<Self::Item>;
    // ... next'in üstüne kurulu onlarca hazır metot
}
# fn main() {}
```

Öğe varsa `Some(öğe)`, sıra bittiyse `None` gelir:

```rust
fn main() {
    let v = vec![10, 20];
    let mut bant = v.iter();

    println!("{:?}", bant.next());
    println!("{:?}", bant.next());
    println!("{:?}", bant.next()); // bant boşaldı
}
```

Çıktı:

```text
Some(10)
Some(20)
None
```

`bant` değişkeni `mut` olmak zorundadır, çünkü `next` iterator'ın iç durumunu, yani nerede kaldığını değiştirir. `iter` öğelere referans verdiği için gelen değerler aslında `Some(&10)` ve `Some(&20)`'dir; `{:?}` biçimi referansın gösterdiği değeri yazar.

Sırayla gezmenin tek ve ortak bir yolu olması büyük kazançtır: Kaynak ister bir liste, ister bir dosyanın satırları, ister bir aralık olsun, aynı araçlar (`map`, `filter`, `sum`) hepsinde çalışır. `for` döngüsü de arka planda bu düğmeye basar; `None` gelene kadar `next` çağırır.[^for]

## Dönüştürücüler tembeldir

Bandın üzerine istasyonlar kuran metotlara *dönüştürücü* (adapter) denir. `map` ve `filter` birer dönüştürücüdür: Bir iterator alır, yeni bir iterator verirler. Bandı çalıştıran ise *tüketicidir* (consumer); `collect`, `sum` ve `for` gibi.[^donusturucu] Dönüştürücüleri dizmek hiçbir iş yaptırmaz, iş tüketici `next` çağırmaya başlayınca yapılır:

```rust
fn main() {
    let v = vec![1, 2, 3, 4, 5];

    let mut it = v.iter();
    println!("next: {:?} {:?}", it.next(), it.next());

    let sonuc: Vec<i32> = v
        .iter() // &i32 verir
        .map(|x| x * 2) // dönüştürücü: her öğeyi ikiyle çarp
        .filter(|x| x % 3 != 0) // dönüştürücü: 3'e bölünenleri ele
        .collect(); // tüketici: zincir burada çalışır
    println!("map+filter+collect: {sonuc:?}");

    let toplam: i32 = v.iter().sum();
    let kareler_toplami: i32 = v.iter().map(|x| x * x).sum();
    println!("toplam = {toplam}, kareler toplamı = {kareler_toplami}");
}
```

Çıktı:

```text
next: Some(1) Some(2)
map+filter+collect: [2, 4, 8, 10]
toplam = 15, kareler toplamı = 55
```

Bir zinciri tüketmezseniz hiçbir şey olmaz. Aşağıdaki program derlenir, ama ekrana hiçbir sayı yazmaz:

```rust
fn main() {
    let v = vec![1, 2, 3];
    v.iter().map(|x| println!("{x}")); // isteyen yok
    println!("bitti");
}
```

Çıktı:

```text
bitti
```

Derleyicinin yanıtı bir hata değil, iki uyarıdır:

```text
warning: `Iterator::map` call that discard the iterator's values
 ...
help: you might have meant to use `Iterator::for_each`
 ...
warning: unused `Map` that must be used
  |
3 |     v.iter().map(|x| println!("{x}"));
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: iterators are lazy and do nothing unless consumed
```

Amacınız yan etkiyse (ekrana yazmak gibi) `for` döngüsü ya da `for_each` kullanın; sonuç istiyorsanız zinciri `collect` ya da `sum` ile bitirin. Tembelliğin karşılığı, istenmeyen öğe için emek harcanmamasıdır: Yalnızca ilk iki öğe istenirse gerisi hiç işlenmez. Bunun gösterimi [sonraki sayfadadır](12-closures-iterators-deep-dive.md).

## Üç gezinme biçimi

Raftaki paketleri üç biçimde gezebilirsiniz: yalnızca etiketlerine bakarak, üstlerine not yazarak ya da hepsini alıp götürerek. İlk ikisinde paketler rafta kalır, üçüncüsünde raf boşalır. Bu, closure'ın üç yakalama biçimiyle aynı ayrımdır:[^gezinme]

| Metot | Verdiği öğe | `for` ile eşdeğeri | Koleksiyon |
| --- | --- | --- | --- |
| `iter()` | `&T` | `for x in &v` | yerinde kalır |
| `iter_mut()` | `&mut T` | `for x in &mut v` | yerinde kalır, öğeleri değişebilir |
| `into_iter()` | `T` | `for x in v` | tüketilir |

```rust
fn main() {
    let mut m = vec![10, 20, 30];
    for x in m.iter() {
        print!("{x} "); // yalnızca bakar
    }
    println!();

    for x in m.iter_mut() {
        *x += 1; // yerinde değiştirir
    }
    println!("iter_mut sonrası: {m:?}");

    let sahiplenen: Vec<String> = vec![String::from("a"), String::from("b")]
        .into_iter() // String'lerin sahipliğini verir
        .map(|s| s + "!")
        .collect();
    println!("into_iter: {sahiplenen:?}");
}
```

Çıktı:

```text
10 20 30 
iter_mut sonrası: [11, 21, 31]
into_iter: ["a!", "b!"]
```

`into_iter` koleksiyonu tükettiği için sonrasında ona erişemezsiniz:

```rust,compile_fail,E0382
fn main() {
    let v = vec![String::from("a"), String::from("b")];
    for x in v.into_iter() {
        println!("{x}");
    }
    println!("{v:?}");
}
```

Derleyicinin yanıtı:

```text
error[E0382]: borrow of moved value: `v`
...
note: `std::iter::IntoIterator::into_iter` takes ownership of the receiver `self`, which moves `v`
```

Koleksiyona sonra da ihtiyacınız varsa `iter` kullanın, yani `for x in &v` yazın.

## Kendi iterator'ınızı yazmak

Şubedeki sıra numarası makinesini düşünün: Düğmeye basınca sıradaki numarayı verir, kâğıdı bitince vermez. Makinenin bilmesi gereken tek şey "şu an kaçtayım" sorusunun cevabıdır. Kendi iterator'ınızı yazmak bu makineyi yapmaktır: `Item` türünü belirler ve yalnızca `next`'i yazarsınız.[^kendi]

```rust
struct GeriSay {
    n: u32,
}

impl Iterator for GeriSay {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.n == 0 {
            None
        } else {
            self.n -= 1;
            Some(self.n + 1)
        }
    }
}

fn main() {
    let hepsi: Vec<u32> = GeriSay { n: 3 }.collect();
    println!("{hepsi:?}");
    println!("{}", GeriSay { n: 3 }.sum::<u32>());
}
```

Çıktı:

```text
[3, 2, 1]
6
```

`sum`'ı biz yazmadık. `map`, `filter`, `zip`, `sum` gibi onlarca metot `Iterator` trait'inde `next`'in üstüne kurulu *varsayılan metotlar* olarak bulunur; `next` yazıldığı anda hepsi hazır gelir. Böylece her yeni sıra türü için aynı yardımcıları yeniden yazmak gerekmez.

Aşağıdaki `Sayac` 1'den 5'e kadar sayar. Zincirde iki sayaç, biri bir adım öndeyken eşlenir (`zip` ile `skip`), çiftler çarpılır ve 3'e bölünenler toplanır:

```rust
struct Sayac {
    sayi: u32,
}

impl Iterator for Sayac {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.sayi < 5 {
            self.sayi += 1;
            Some(self.sayi)
        } else {
            None
        }
    }
}

fn main() {
    let toplam: u32 = Sayac { sayi: 0 }
        .zip(Sayac { sayi: 0 }.skip(1)) // (1,2), (2,3), (3,4), (4,5)
        .map(|(a, b)| a * b) // 2, 6, 12, 20
        .filter(|x| x % 3 == 0) // 6, 12
        .sum();

    println!("kendi iterator zinciri = {toplam}");
}
```

Çıktı:

```text
kendi iterator zinciri = 18
```

`zip`, iki sıradan biri bitince durur. Bu yüzden ilk sayacın verdiği 5'in eşi bulunmaz ve beşinci çift oluşmaz.

## Hata kodu başvurusu

| Kod | Ne zaman görülür? | Ne yapmalı? |
| --- | --- | --- |
| `E0434` | Bir `fn`, dışındaki yerel değişkeni kullanmaya çalıştığında | Closure yazın ya da değeri parametre olarak verin. |
| `E0308` | Closure farklı türlerle çağrıldığında ya da yakalayan bir closure `fn` beklenen yere verildiğinde | Türü tutarlı kullanın; yakalama gerekiyorsa jenerik `F: Fn(..)` ya da `Box<dyn Fn(..)>` kullanın. |
| `E0596` | Değiştiren closure `mut` olmayan bir değişkende tutulduğunda | `let mut` yazın. |
| `E0502` | Closure `&mut` tutarken değişken okunduğunda | Okumayı closure'ın son çağrısından sonraya alın. |
| `E0382` | Taşınmış değer kullanıldığında, `FnOnce` closure iki kez çağrıldığında ya da `into_iter` sonrası koleksiyona erişildiğinde | Ödünç verin, klonlayın ya da `iter` kullanın. |
| `E0373` | Closure, ödünç aldığı yerel değişkenden uzun yaşayabilecekse | `move` yazın. |
| `E0525` | Closure, istenenden daha gevşek bir trait uyguladığında (`Fn` yerine `FnMut` gibi) | İmzayı gevşetin ya da gövdeyi değiştirin. |

Herhangi bir hata kodunun uzun açıklamasını terminalde görebilirsiniz:

```console
rustc --explain E0373
```

## Perde arkası: Derleyici ne yapıyor?

> [!NOTE]
> Bu başlık ilk okuyuşta atlanabilir; yukarıdaki kuralları anlamış olmanız yeterlidir.

İki soruya yanıt arıyoruz: Closure'ın hatırladığı değerler nerede durur ve bunca istasyon programı yavaşlatmaz mı?

**Closure, veri ile kodun birleşimidir.** Derleyici her closure için yakalanan değişkenleri alan olarak tutan isimsiz bir yapı üretir ve bu yapıya uygun `Fn` trait'lerini uygular.[^closure-turu] Bölümün başındaki `fiyat` closure'ı kabaca şu yapıya karşılık gelir:

```rust
struct FiyatClosure<'a> {
    vergi: &'a u32, // paylaşımlı ödünçle yakalandı
}

impl FiyatClosure<'_> {
    fn call(&self, x: u32) -> u32 {
        x + x * *self.vergi / 100
    }
}

fn main() {
    let vergi = 20;
    let fiyat = FiyatClosure { vergi: &vergi };
    println!("{}", fiyat.call(100));
}
```

Çıktı:

```text
120
```

Bunun bir sonucu, closure'ın boyutunun yakaladıklarına bağlı olmasıdır. 64 bitlik bir hedefte:

```rust
use std::mem::size_of_val;

fn main() {
    let k = 10_i64;
    let metin = String::from("merhaba");

    let yakalamaz = |x: i64| x + 1;
    let odunc = |x: i64| x + k; // &i64 tutar
    let tasir = move || metin.len(); // String'in üç sözcüğünü tutar
    let isaretci: fn(i64) -> i64 = yakalamaz;

    println!("{} {}", size_of_val(&yakalamaz), size_of_val(&odunc));
    println!("{} {}", size_of_val(&tasir), size_of_val(&isaretci));
}
```

Çıktı:

```text
0 8
24 8
```

**Yakalama biçimi ve trait ayrı ayrı hesaplanır.** "`&` mi, `&mut` mu, taşıma mı?" kararını tür denetleyicisinin `upvar.rs` dosyasındaki çözümleme verir.[^upvar] Closure'ın hangi trait'i uygulayacağı, gövdedeki kullanımdan hesaplanır: Dışarı taşıma varsa yalnızca `FnOnce`, değiştirme varsa `FnMut`, yalnızca okuma varsa `Fn`.[^upvar-tur] `move` sözcüğü sonradan, yalnızca yakalama biçimine uygulanır;[^upvar-move] bu yüzden `move` yazmak bir closure'ı `Fn` olmaktan çıkarmaz. Trait seçimi, ödünç davranışının doğrudan yansımasıdır. The Rust Reference, üç yakalama biçiminin yanında bir dördüncüsünü daha sayar: yakalanan bir `&mut` referansın gösterdiği değer değiştirilirken kullanılan benzersiz ödünç. Bunu elle yazmanın bir yolu yoktur; yalnızca derleyicinin kararında görünür.[^bicimler]

**Yakalama ayrıktır.** Edition 2021'den beri closure, bir yapının tamamını değil yalnızca kullandığı alanını yakalar (RFC 2229).[^ayrik] Aşağıda closure `adres`'i ödünç almışken `not` alanı değiştirilebilir:

```rust
struct Koli {
    adres: String,
    not: String,
}

fn main() {
    let mut k = Koli {
        adres: String::from("İzmir"),
        not: String::new(),
    };

    let adresi_yaz = || println!("adres: {}", k.adres); // yalnızca k.adres yakalanır
    k.not.push_str("kırılacak eşya");
    adresi_yaz();
    println!("{}", k.not);
}
```

Çıktı:

```text
adres: İzmir
kırılacak eşya
```

Edition 2018'de aynı kod `E0502` verir, çünkü `k`'nin tamamı yakalanırdı. Rust 1.100 ile `if let` içinde kullanılan değişkenlerin yakalanması da daha isabetli hâle geldi; sürüm notu bunun bazı kodlarda yeni hatalara yol açabileceğini belirtir.[^iflet]

**`move` ve sarkan closure.** Ödünç denetleyicisi, closure'ın ödünç aldığı bir yerel değişkenden uzun yaşayabileceğini görürse `E0373` üretir.[^e0373] Denetim olmasaydı closure, düşürülmüş bir yereli gösteren sarkan bir yapıya dönüşürdü.

**Iterator'ın tek zorunlu metodu `next`'tir.** `Iterator` trait'inde gövdesi yazılmamış tek metot `fn next(&mut self) -> Option<Self::Item>`'dir; `map`, `filter`, `sum` ve `zip` bunun üstüne kurulu varsayılan metotlardır.[^next] Tembellik uyarısı da kütüphaneden gelir: `Map` gibi dönüştürücü yapıları `#[must_use]` özniteliği taşır[^map] ve `unused_must_use` denetimi, kullanılmayan bu değerleri bildirir.[^must-use]

**Maliyet.** Dönüştürücüler jeneriktir; monomorfizasyon her closure için ayrı kod üretir,[^mono] iyileştirme (optimizasyon) açıkken de LLVM bu küçük fonksiyonları birbirinin içine yerleştirir. Sonuç çoğu zaman elle yazılmış döngüyle aynı hızdadır. The Rust Programming Language bunu "birçok durumda" diye niteler; aynı makine kodu çıkacağı güvencesi yoktur.[^maliyet] Bu bölüm hazırlanırken `map` ve `sum` içeren bir zincir ile aynı işi yapan döngü x86-64 üzerinde, rustc 1.99.0 ve `opt-level=3` ile derlenip karşılaştırıldı: İkisi de SIMD komutlarıyla vektörleştirildi; zincir 49, döngü 53 komut oldu. Araya `filter` eklendiğinde sayılar 80 ve 84'tü. Yani iki kod birbirine çok yakın, ama birebir aynı değildi. İyileştirmesiz derlemede (`opt-level=0`) ise zincir, iç içe fonksiyon çağrıları olarak kalır. Bu nedenle `cargo build` ile değil, `cargo build --release` ile ölçün.

## Özet

- Closure, yazıldığı yerdeki değişkenleri yakalayan adsız fonksiyondur. Derleyici yeterli olan en dar yakalamayı seçer: `&`, `&mut` ya da taşıma. `move` taşımayı zorunlu kılar.
- Gövde, closure'ın hangi trait'leri uygulayacağını belirler: Okuyan `Fn`, değiştiren `FnMut`, dışarı taşıyan yalnızca `FnOnce`. `FnOnce` en fazla bir kez çağrılabilir demektir. İmzada işinizi gören en gevşek trait'i isteyin.
- Yakalamayan closure, `fn` işaretçisine dönüşür. `fn` tek ve adı olan bir türdür; jenerik `F` ise yakalamaya ve satır içine almaya elverişlidir.
- Iterator'ın tek zorunlu metodu `next`'tir. Dönüştürücüler tembeldir, zinciri tüketici çalıştırır. `iter` bakar, `iter_mut` değiştirir, `into_iter` sahiplenir.

Sıradaki sayfa [12.1 Closure ve Iterator'lar: Derinleşme](12-closures-iterators-deep-dive.md), bu araçları bir sipariş veri hattında birlikte kullanıyor: closure döndürmek ve saklamak, yeni dönüştürücüler, sonsuz bir iterator ve en sık karşılaşılan beş ödünç hatası.

## Kaynaklar

[^yakalama]: The Rust Programming Language, [Capturing the Environment](https://doc.rust-lang.org/book/ch13-01-closures.html#capturing-the-environment); The Rust Reference, [Closure expressions](https://doc.rust-lang.org/reference/expressions/closure-expr.html#closure-expressions); hata dizini, [`E0434`](https://doc.rust-lang.org/error_codes/E0434.html).
[^cikarim]: The Rust Programming Language, [Inferring and Annotating Closure Types](https://doc.rust-lang.org/book/ch13-01-closures.html#inferring-and-annotating-closure-types).
[^bicimler]: The Rust Programming Language, [Capturing References or Moving Ownership](https://doc.rust-lang.org/book/ch13-01-closures.html#capturing-references-or-moving-ownership); The Rust Reference, [Closure types: Capture modes](https://doc.rust-lang.org/reference/types/closure.html#capture-modes); hata dizini, [`E0596`](https://doc.rust-lang.org/error_codes/E0596.html) ve [`E0502`](https://doc.rust-lang.org/error_codes/E0502.html).
[^move]: The Rust Programming Language, [Capturing References or Moving Ownership](https://doc.rust-lang.org/book/ch13-01-closures.html#capturing-references-or-moving-ownership); std, [`move` anahtar sözcüğü](https://doc.rust-lang.org/std/keyword.move.html); The Rust Reference, [Copy values](https://doc.rust-lang.org/reference/types/closure.html#copy-values).
[^spawn]: std, [`thread::spawn`](https://doc.rust-lang.org/std/thread/fn.spawn.html); hata dizini, [`E0373`](https://doc.rust-lang.org/error_codes/E0373.html).
[^traitler]: The Rust Programming Language, [Moving Captured Values Out of Closures](https://doc.rust-lang.org/book/ch13-01-closures.html#moving-captured-values-out-of-closures); The Rust Reference, [Call traits and coercions](https://doc.rust-lang.org/reference/types/closure.html#call-traits-and-coercions); std, [`FnOnce`](https://doc.rust-lang.org/std/ops/trait.FnOnce.html), [`FnMut`](https://doc.rust-lang.org/std/ops/trait.FnMut.html) ve [`Fn`](https://doc.rust-lang.org/std/ops/trait.Fn.html); hata dizini, [`E0525`](https://doc.rust-lang.org/error_codes/E0525.html).
[^unwrap]: std, [`Option::unwrap_or_else`](https://doc.rust-lang.org/std/option/enum.Option.html#method.unwrap_or_else); The Rust Programming Language, [Moving Captured Values Out of Closures](https://doc.rust-lang.org/book/ch13-01-closures.html#moving-captured-values-out-of-closures).
[^fn-isaretcisi]: The Rust Programming Language, [Function Pointers](https://doc.rust-lang.org/book/ch20-04-advanced-functions-and-closures.html#function-pointers).
[^donusum]: The Rust Reference, [Call traits and coercions](https://doc.rust-lang.org/reference/types/closure.html#call-traits-and-coercions) ve [Function pointer types](https://doc.rust-lang.org/reference/types/function-pointer.html#function-pointer-types); hata dizini, [`E0308`](https://doc.rust-lang.org/error_codes/E0308.html).
[^ffi]: The Rust Programming Language, [Function Pointers](https://doc.rust-lang.org/book/ch20-04-advanced-functions-and-closures.html#function-pointers).
[^iterator]: The Rust Programming Language, [The `Iterator` Trait and the `next` Method](https://doc.rust-lang.org/book/ch13-02-iterators.html#the-iterator-trait-and-the-next-method); std, [`Iterator`](https://doc.rust-lang.org/std/iter/trait.Iterator.html).
[^for]: The Rust Reference, [Iterator loops](https://doc.rust-lang.org/reference/expressions/loop-expr.html#iterator-loops).
[^donusturucu]: The Rust Programming Language, [Methods That Consume the Iterator](https://doc.rust-lang.org/book/ch13-02-iterators.html#methods-that-consume-the-iterator) ve [Methods That Produce Other Iterators](https://doc.rust-lang.org/book/ch13-02-iterators.html#methods-that-produce-other-iterators); std, [Laziness](https://doc.rust-lang.org/std/iter/index.html#laziness).
[^gezinme]: std, [The three forms of iteration](https://doc.rust-lang.org/std/iter/index.html#the-three-forms-of-iteration) ve [`for` loops and `IntoIterator`](https://doc.rust-lang.org/std/iter/index.html#for-loops-and-intoiterator).
[^kendi]: std, [Implementing Iterator](https://doc.rust-lang.org/std/iter/index.html#implementing-iterator); std, [`Iterator::zip`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.zip) ve [`Iterator::skip`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.skip).
[^closure-turu]: The Rust Reference, [Closure types](https://doc.rust-lang.org/reference/types/closure.html#closure-types).
[^upvar]: rustc 1.99.0 kaynak kodu, [`upvar.rs`, `analyze_closure`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_typeck/src/upvar.rs#L166).
[^upvar-tur]: rustc 1.99.0 kaynak kodu, [`upvar.rs`, `process_collected_capture_information`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_typeck/src/upvar.rs#L646).
[^upvar-move]: rustc 1.99.0 kaynak kodu, [`upvar.rs`, `adjust_for_move_closure`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_typeck/src/upvar.rs#L2309); The Rust Reference, [Call traits and coercions](https://doc.rust-lang.org/reference/types/closure.html#call-traits-and-coercions).
[^ayrik]: [RFC 2229: Capture disjoint fields](https://rust-lang.github.io/rfcs/2229-capture-disjoint-fields.html); Edition Guide, [Disjoint capture in closures](https://doc.rust-lang.org/edition-guide/rust-2021/disjoint-capture-in-closures.html); The Rust Reference, [Edition 2018 and before](https://doc.rust-lang.org/reference/types/closure.html#edition-2018-and-before).
[^iflet]: [Rust 1.100.0 sürüm notları](https://github.com/rust-lang/rust/releases/tag/1.100.0) ve [154210 numaralı değişiklik](https://github.com/rust-lang/rust/pull/154210).
[^e0373]: rustc 1.99.0 kaynak kodu, [`borrowck_errors.rs`, `cannot_capture_in_long_lived_closure`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_borrowck/src/borrowck_errors.rs#L459).
[^next]: std 1.99.0 kaynak kodu, [`iterator.rs`, `fn next`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/iter/traits/iterator.rs#L78).
[^map]: std 1.99.0 kaynak kodu, [`map.rs`, `must_use`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/iter/adapters/map.rs#L58).
[^must-use]: rustc 1.99.0 kaynak kodu, [`must_use.rs`, `UNUSED_MUST_USE`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_lint/src/unused/must_use.rs#L44).
[^mono]: rustc 1.99.0 kaynak kodu, [`collector.rs`, `collect_crate_mono_items`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_monomorphize/src/collector.rs#L1914); The Rust Programming Language, [Performance of Code Using Generics](https://doc.rust-lang.org/book/ch10-01-syntax.html#performance-of-code-using-generics).
[^maliyet]: The Rust Programming Language, [13.4 Performance in Loops vs. Iterators](https://doc.rust-lang.org/book/ch13-04-performance.html); The Cargo Book, [Default profiles](https://doc.rust-lang.org/cargo/reference/profiles.html#default-profiles).
