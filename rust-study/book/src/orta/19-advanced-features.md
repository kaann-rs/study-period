# 19. Gelişmiş Özellikler

Rust'ın her gün kullanılmayan ama gerektiğinde yerine başka bir şey konamayan araçları vardır. Bu bölüm onları beş öbekte topluyor: `unsafe` Rust, gelişmiş trait'ler, gelişmiş türler, fonksiyon işaretçileri ile closure döndürmek ve makrolar.[^giris]

Ortak tema hep aynıdır: Güçlü aracı küçük bir yerde tutmak ve dışarıya **güvenli bir soyutlama** sunmak. Standart kütüphane de böyle kuruludur. `Vec` ya da `split_at_mut` gibi parçaların içinde `unsafe` bloklar bulunur, ama bunları kullanırken `unsafe` yazmanız gerekmez.

> [!NOTE]
> Bu bölümün notları ve çalıştırılabilir kodu `learn/advanced-features` dizinindedir; depo kökünden `cargo run -p advanced-features` komutuyla çalıştırabilirsiniz. Ham işaretçi okuma örneği ile `noise` fonksiyonu `learn/basic-concepts` notlarından alınmıştır; o crate başka bölümlerindeki hatalar yüzünden şu an derlenmez.

## Günlük hayattan bir benzetme

Bir atölyede çalıştığınızı düşünün. Makinelerin üstünde koruyucu kapak vardır; kapak kapalıyken elinizi kesemezsiniz. Bazı ince işler için usta kapağı açar. Kapak açıkken makine ustayı korumaz, dikkat etmek ustanın işidir. Rust'taki `unsafe` bloğu o açık kapaktır: "Buranın doğruluğunu derleyici göremiyor, ben garanti ediyorum." İyi usta kapağı kısa süre açar, işi bitince kapatır. Makası ya da usturayı küçük bir kutunun içinde kullanır, size yalnızca güvenli sapını uzatır; dışarıdan bakan kişi kapağın açıldığını bilmek zorunda kalmaz.

Atölyedeki öteki araçlar da bu bölümün konularıdır:

- **Terazi** elma tartmayı bilir; kendi yaptığınız bir şeyi tartabilmesi için nasıl tartılacağını öğretmeniz gerekir. Bu, `+` gibi işaretlere kendi türünüzü öğretmektir.
- **Etiket makinesi:** Ortak aletlere kendi etiketinizi yapıştıramazsınız, çünkü alet de etiket makinesi de başkasınındır. Ama aleti kendi kutunuza koyup etiketi kutuya yapıştırabilirsiniz. Bu, *newtype* desenidir.
- **Lakap, tarif kartı, ayarlı alet:** Uzun adlı bir alete kısa bir lakap takarsınız (tür takma adı), bir tarif kartını başkasına verirsiniz (fonksiyon işaretçisi), ayarı önceden yapılmış bir alet üretirsiniz (closure döndürmek).
- **Lastik damga:** Bir kez hazırlanır, her basışta aynı yazıyı sizin yerinize yazar. Makro, kod yazan bir damgadır.

<details>
<summary>Bu bölümde geçen terimler</summary>

- **`unsafe`:** "Buranın doğruluğunu derleyici değil, ben garanti ediyorum" diyen blok, fonksiyon ya da trait işareti.
- **ham işaretçi (raw pointer, `*const T`, `*mut T`):** Referansların güvenlik ve ömür garantilerini taşımayan çıplak adres.
- **başvuruyu çözmek (dereference, `*r`):** Adrese gidip orada duran değere ulaşmak.
- **null ve sarkan (dangling) işaretçi:** Sırasıyla hiçbir yeri göstermeyen ve artık geçerli olmayan bir yeri gösteren adres.
- **tanımsız davranış (undefined behavior, UB):** Kural çiğnendiğinde programın ne yapacağının artık bilinmemesi.
- **soyutlama (abstraction):** Karmaşık içi saklayıp dışarıya basit bir kullanım sunmak.
- **ödünç denetleyicisi (borrow checker):** Ödünç kurallarını derleme zamanında denetleyen derleyici parçası.
- **`assert!`:** "Şu koşul doğru olmalı, değilse panikle" denetimi.
- **FFI (foreign function interface):** Rust ile başka dillerin birbirinin fonksiyonlarını çağırması.
- **ABI (application binary interface):** Bir fonksiyonun makine düzeyinde nasıl çağrılacağını belirleyen kurallar.
- **operatör aşırı yükleme:** `+` gibi işaretlerin kendi türünüzde ne yapacağını tanımlamak.
- **varsayılan tür parametresi:** Yazılmadığında kullanılacak türü baştan belirlenmiş jenerik parametre.
- **ilişkili tür (associated type):** Bir trait'in içinde "sonucun türü şudur" diye adı konan tür.
- **newtype:** Var olan bir türü tek alanlı kendi türünüzle sarmak.
- **yetim kuralı (orphan rule):** "Trait de tür de başkasınınsa ikisini siz bağlayamazsınız" kuralı.
- **`Display`:** Bir değerin `{}` ile nasıl yazdırılacağını söyleyen trait.
- **tür takma adı (type alias):** Var olan bir türe verilen ikinci ad; yeni tür oluşturmaz.
- **fonksiyon işaretçisi (`fn`):** Bir fonksiyonu değer gibi taşımayı sağlayan tür.
- **closure, `Fn`, `FnMut`, `FnOnce`:** Yerinde yazılan adsız fonksiyon ve onun üç çağrılma biçimi.
- **`impl Fn`, `Box<dyn Fn>`:** Closure döndürmenin iki yolu: türü derleme zamanında belli olan ve kutu içinde taşınan.
- **makro, `macro_rules!`:** Kod üreten kod ve onu desenlerle tanımlamanın yolu.
- **metaprogramlama:** Program yazan program yazmak.
- **genişleme (expansion):** Makro çağrısının, derleme ilerlemeden önce gerçek koda çevrilmesi.
- **bildirimsel / yordamsal makro (declarative / procedural):** Sırasıyla desenle tanımlanan makro ve kod parçalarını işleyen fonksiyon biçimindeki makro.
- **`TokenStream`:** Kaynak kodun sözcük parçalarına ayrılmış hâli.
- **never type (`!`):** Hiçbir zaman bir değer üretmeyen ifadelerin türü.
- **dinamik boyutlu tür (DST) ve `Sized`:** Boyutu derleme zamanında bilinmeyen tür ve boyutu bilinen türlerin trait'i.
- **edition:** Dilin, eski kodu bozmadan yeni kurallar getirebilmek için crate başına seçilen sürüm kümesi. Bu kitabın örnekleri edition 2024 ile derlenir.
- **`static`, `union`, DSL:** Program boyunca yaşayan değişken, alanları aynı belleği paylaşan tür ve belli bir iş için tasarlanmış küçük dil.

</details>

## `unsafe`: Derleyicinin kanıtlayamadığı yer

Derleyici yalnızca doğruluğunu *kanıtlayabildiği* kodu kabul eder; kanıtlayamadığını, aslında doğru olsa bile geri çevirir. Öte yandan donanıma dokunmak, başka dilde yazılmış bir fonksiyonu çağırmak ya da `Vec` gibi bir yapının içini kurmak, derleyicinin kanıtlayamayacağı işlerdir. Bu işler için Rust'ın ikinci bir yüzü vardır: `unsafe` Rust.[^unsafe]

C gibi işaretçi ve bellek işlemlerinin her yerde serbest olduğu dillerde bir bellek hatası kodun herhangi bir yerinde olabilir; hata ararken her satırı şüpheli saymak zorundasınız. Rust'ta kodun çoğunu derleyici denetler, kanıtlayamadığı işlemleri ise yalnızca `unsafe` işaretli yerlerde kabul eder. Böylece şüpheli alan küçülür.

`unsafe` blok şu işlemlerin kapısını açar:[^bes-guc]

1. Ham işaretçinin başvurusunu çözmek (dereference).
2. `unsafe` bir fonksiyonu ya da metodu çağırmak.
3. Değiştirilebilir bir `static` değişkene erişmek ya da onu değiştirmek.
4. `unsafe` bir trait'i uygulamak (`unsafe impl`).
5. Bir `union`'ın alanına erişmek (alana yalnızca atama yapmak güvenlidir).

> [!NOTE]
> Bu beşli liste *The Rust Programming Language* kitabından alınmıştır. The Rust Reference listeye üç madde daha ekler: gereken işlemci özelliği etkin olmayan bir yerden `target_feature` işaretli güvenli bir fonksiyonu çağırmak, `extern` blok bildirmek ve `unsafe` bir öznitelik uygulamak.[^reference-liste] Satır içi assembly de `unsafe` ister.

`unsafe` ödünç denetleyicisini kapatmaz. Yalnızca yukarıdaki işlemleri açar; geri kalan her şey alışıldık kurallara bağlıdır:

```rust,compile_fail,E0499
fn main() {
    let mut s = String::from("a");
    unsafe {
        let a = &mut s;
        let b = &mut s;
        a.push('x');
        b.push('y');
    }
}
```

Derleyicinin yanıtı:

```text
error[E0499]: cannot borrow `s` as mutable more than once at a time
  |
4 |         let a = &mut s;
  |                 ------ first mutable borrow occurs here
5 |         let b = &mut s;
  |                 ^^^^^^ second mutable borrow occurs here
6 |         a.push('x');
  |         - first borrow later used here
```

### Ham işaretçiler

`*const T` ve `*mut T` türleri *ham işaretçidir*. "Ham" denmesinin nedeni, referansların güvenlik ve ömür garantilerini taşımamalarıdır: Derleyici onları ödünç kurallarına göre denetlemez, geçerli belleği göstermeleri garanti değildir, null olabilirler ve kapsam sonunda kendiliğinden temizlik yapmazlar.[^ham] Ham işaretçi **oluşturmak** güvenlidir, çünkü bir adresi elde tutmak kimseye zarar vermez. Tehlikeli olan, adrese gidip orada duranı kullanmaktır:

```rust
fn main() {
    let x = 42;
    let r = &raw const x; // güvenli: yalnızca adres alındı

    // SAFETY: r, hâlâ yaşayan ve başlatılmış x'i gösteriyor.
    unsafe {
        println!("*r = {}", *r);
    }
}
```

Çıktı:

```text
*r = 42
```

`unsafe` blok olmadan başvuruyu çözmek derlenmez:

```rust,compile_fail,E0133
fn main() {
    let x = 42;
    let r = &raw const x;
    println!("{}", *r);
}
```

Derleyicinin yanıtı:

```text
error[E0133]: dereference of raw pointer is unsafe and requires unsafe block
  |
4 |     println!("{}", *r);
  |                    ^^ dereference of raw pointer
  |
  = note: raw pointers may be null, dangling or unaligned; they can violate aliasing rules and cause data races: all of these are undefined behavior
```

Not, derleyicinin neyi bilemediğini sayar: İşaretçi null olabilir, sarkabilir ya da hizasız olabilir; ödünç kurallarını çiğneyip veri yarışına yol açabilir. Böyle bir erişim *tanımsız davranıştır*; programın ne yapacağı artık bilinmez.[^ub] `unsafe` blok yazarak bu koşulların hiçbirinin gerçekleşmediğini siz garanti etmiş olursunuz. `// SAFETY:` yorumu, o garantinin gerekçesini okuyana bırakmanın yerleşik yoludur.

İşaretçinin gösterdiği değeri okumanın bir yolu da `ptr::read` fonksiyonudur. Belleği değiştirmeden bitlerin bir kopyasını döndürür:

```rust
use std::ptr;

fn main() {
    let s: i32 = 10;
    let s1: *const i32 = &raw const s;

    // SAFETY: s1, s'den geliyor ve iki okuma da s yaşarken yapılıyor.
    unsafe {
        println!("ptr::read = {}", ptr::read(s1));
        println!("*s1 = {}", *s1);
    }
}
```

Çıktı:

```text
ptr::read = 10
*s1 = 10
```

Okumanın doğru olması için işaretçinin geçerli, doğru hizalanmış ve başlatılmış bir `i32` göstermesi gerekir; bunu derleyici denetlemez. `i32` `Copy` olduğu için aynı değeri iki kez okumak sorun çıkarmaz. `ptr::read` ise `Copy` olmayan türleri de bit bit kopyalar: Bir `String`'i böyle okuyup iki değeri de düşürürseniz aynı heap bloğu iki kez geri verilir.[^ptr-read]

### `&raw const` ve eski yazım

Eski kodda ham işaretçi çoğunlukla `&s as *const i32` diye, yani önce referans oluşturup sonra tür dönüştürerek alınır. Bu yazım sıradan değişkenlerde hâlâ geçerlidir. Sorun, referans oluşturmanın başlı başına tanımsız davranış sayıldığı yerlerde çıkar: hizasız alanlar, başlatılmamış bellek, sarkan işaretçiler. Rust 1.82'de gelen `&raw const` ve `&raw mut`, arada referans oluşturmadan doğrudan ham işaretçi üretir.[^raw]

`packed` bir yapıda alanlar arasında hizalama boşluğu bırakılmaz; bu yüzden `uzunluk` alanı hizasız bir adrestedir ve ona referans alınamaz:

```rust,compile_fail,E0793
#[repr(C, packed)]
struct Paket {
    tur: u8,
    uzunluk: u32,
}

fn main() {
    let p = Paket { tur: 1, uzunluk: 258 };
    let isaretci = &p.uzunluk as *const u32;
}
```

Derleyicinin yanıtı:

```text
error[E0793]: reference to field of packed struct is unaligned
  |
9 |     let isaretci = &p.uzunluk as *const u32;
  |                    ^^^^^^^^^^
  |
  = note: this struct is 1-byte aligned, but the type of this field may require higher alignment
  = note: creating a misaligned reference is undefined behavior (even if that reference is never dereferenced)
```

`&raw const` ile işaretçi alınabilir; hizasız olduğu için de `read_unaligned` ile okunur:

```rust
#[repr(C, packed)]
struct Paket {
    tur: u8,
    uzunluk: u32,
}

fn main() {
    let p = Paket { tur: 1, uzunluk: 258 };
    let isaretci = &raw const p.uzunluk;

    // SAFETY: isaretci, yaşayan p'nin alanını gösteriyor; hizasız okuma kullanılıyor.
    let uzunluk = unsafe { isaretci.read_unaligned() };
    println!("tür = {}, uzunluk = {uzunluk}", p.tur);
}
```

Çıktı:

```text
tür = 1, uzunluk = 258
```

### `unsafe` fonksiyonlar

Bir fonksiyonun doğru çalışması için çağıranın uyması gereken, türle ifade edilemeyen bir koşul varsa fonksiyon `unsafe fn` olarak bildirilir. Koşul, belge yorumunda `# Safety` başlığı altında yazılır:[^unsafe-fn]

```rust
/// # Safety
///
/// `p` geçerli, hizalı ve başlatılmış bir `i32` göstermelidir.
unsafe fn oku(p: *const i32) -> i32 {
    // SAFETY: Koşulu çağıran garanti eder.
    unsafe { *p }
}

fn main() {
    let x = 7;

    // SAFETY: İşaretçi, yaşayan x'ten oluşturuldu.
    let okunan = unsafe { oku(&raw const x) };
    println!("okunan = {okunan}");
}
```

Çıktı:

```text
okunan = 7
```

Çağrıyı `unsafe` blok dışında yapmak derlenmez:

```rust,compile_fail,E0133
# /// # Safety
# /// `p` geçerli olmalıdır.
# unsafe fn oku(p: *const i32) -> i32 { unsafe { *p } }
fn main() {
    let x = 7;
    let okunan = oku(&raw const x);
}
```

Derleyicinin yanıtı:

```text
error[E0133]: call to unsafe function `oku` is unsafe and requires unsafe block
```

Edition 2024'te `unsafe fn` gövdesi kendiliğinden `unsafe` sayılmaz: Gövdedeki ham işaretçi işlemi ayrı bir `unsafe` bloğa alınmazsa `unsafe_op_in_unsafe_fn` uyarısı çıkar. Böylece "çağırana yüklenen koşul" ile "içeride alınan risk" birbirinden ayrılır.[^op-in-fn]

### Güvenli soyutlama: `ikiye_bol`

Bir dilimi ortasından ikiye bölüp iki ayrı `&mut` dilim döndürmek istiyoruz. Güvenli Rust'la yazılan ilk deneme derlenmez:

```rust,compile_fail,E0499
fn ikiye_bol(dilim: &mut [i32], orta: usize) -> (&mut [i32], &mut [i32]) {
    let uzunluk = dilim.len();
    assert!(orta <= uzunluk);
    (&mut dilim[..orta], &mut dilim[orta..])
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0499]: cannot borrow `*dilim` as mutable more than once at a time
  |
1 | fn ikiye_bol(dilim: &mut [i32], orta: usize) -> (&mut [i32], &mut [i32]) {
  |                     - let's call the lifetime of this reference `'1`
...
4 |     (&mut dilim[..orta], &mut dilim[orta..])
  |     --------------------------^^^^^---------
  |     |     |                   |
  |     |     |                   second mutable borrow occurs here
  |     |     first mutable borrow occurs here
  |     returning this value requires that `*dilim` is borrowed for `'1`
  |
  = help: use `.split_at_mut(position)` to obtain two mutable non-overlapping sub-slices
```

Biz iki parçanın çakışmadığını biliyoruz; ödünç denetleyicisi ise yalnızca aynı dilimden iki kez `&mut` istendiğini görür. `unsafe` ile iki ayrı dilimi kendimiz kurabiliriz:[^guvenli-soyutlama]

```rust
use std::slice;

fn ikiye_bol(dilim: &mut [i32], orta: usize) -> (&mut [i32], &mut [i32]) {
    let uzunluk = dilim.len();
    let isaretci = dilim.as_mut_ptr();
    assert!(orta <= uzunluk); // güvenlik koşulunu burada sağlıyoruz

    // SAFETY: isaretci, dilim'den geliyor. orta <= uzunluk olduğu için iki bölge de
    // dilimin içindedir ve birbiriyle çakışmaz.
    unsafe {
        (
            slice::from_raw_parts_mut(isaretci, orta),
            slice::from_raw_parts_mut(isaretci.add(orta), uzunluk - orta),
        )
    }
}

fn main() {
    let mut sayilar = [1, 2, 3, 4, 5, 6];
    let (sol, sag) = ikiye_bol(&mut sayilar, 3);

    sol[0] = 10;
    sag[0] = 40;
    println!("sol = {sol:?}, sağ = {sag:?}");
}
```

Çıktı:

```text
sol = [10, 2, 3], sağ = [40, 5, 6]
```

`ikiye_bol` `unsafe fn` değildir; çağıran kapağın açıldığını bilmez. Yanlış bir `orta` değeri verse bile `assert!` programı panikle durdurur, bellek bozulmaz. Derleyicinin yardım satırındaki `split_at_mut` standart kütüphanedeki karşılığıdır ve gerçek kodda onu kullanmanız gerekir; buradaki sürüm, böyle bir fonksiyonun içinin nasıl kurulduğunu gösterir.

### Güven yüzeyi nerede biter?

Bir bellek hatası çıktığında yalnızca `unsafe` bloklara bakmanın yeteceği sık söylenir. Yararlı bir kısaltmadır, ama tam doğru değildir. `ikiye_bol`'daki `assert!` satırı güvenli koddur; onu silerseniz `unsafe` bloğa hiç dokunmadığınız hâlde fonksiyon bozulur. `unsafe` blok, çevresindeki güvenli kodun koruduğu koşullara yaslanır. Bu yüzden gözden geçirilmesi gereken yer bloğun kendisi değil, o koşulları bozabilecek kodun tamamıdır. Koşullar gizli (private) alanlarda duruyorsa sınırı *modül* çizer: Alanlara modülün dışından ulaşılamaz.[^guven]

`unsafe` blokları küçük tutmanın, gerekçesini yazmanın ve güvenli bir API arkasına saklamanın nedeni budur. Blokların doğruluğunu sınamak için Miri kullanılabilir. Miri, programı yorumlayarak çalıştırır ve pek çok tanımsız davranışı yakalar; yalnızca çalıştırılan yolları gördüğü için hata bulamaması kanıt sayılmaz. Nightly araç zinciri ister:[^miri]

```console
rustup +nightly component add miri
cargo +nightly miri test
```

### Başka dillerle konuşmak

Başka dilde yazılmış bir fonksiyonu çağırmak için imzası bir `extern` blokta bildirilir. Derleyici o imzanın doğru olup olmadığını bilemez; bu yüzden edition 2024'te bloğun kendisi `unsafe extern` diye yazılır. Her öğe `safe` ya da `unsafe` olarak işaretlenebilir; işaretsiz öğe `unsafe` sayılır:[^extern]

```rust
use std::ffi::c_int;

// SAFETY: İmza, C standart kütüphanesindeki abs ile aynıdır.
unsafe extern "C" {
    safe fn abs(girdi: c_int) -> c_int;
}

fn main() {
    println!("abs(-3) = {}", abs(-3));
}
```

Çıktı:

```text
abs(-3) = 3
```

`safe` yazdığımız için çağrıda `unsafe` gerekmedi: `abs`'ın çağıranın uyması gereken bir bellek koşulu yoktur ve imzanın doğruluğunu bloğu yazan kişi üstlenmiştir. Edition 2024'te blok `unsafe` olmadan yazılırsa derleyici `extern blocks must be unsafe` hatası verir.

Ters yönde, Rust'ta yazılmış bir fonksiyonu başka dilden çağrılabilir yapmak için `extern "C"` kullanılır. Notlardaki `noise` örneği şöyledir:

```rust
/// # Safety
///
/// Çağıranın uyması gereken bir koşul yoktur.
unsafe extern "C" fn noise() -> i8 {
    1
}

fn main() {
    // SAFETY: noise hiçbir koşul istemez.
    let n = unsafe { noise() };
    println!("noise = {n}");
}
```

Çıktı:

```text
noise = 1
```

İmzadaki her sözcüğün ayrı bir görevi vardır:[^extern-fn]

- `extern "C"` yalnızca çağrı kurallarını, yani ABI'yi belirler: Fonksiyon C'nin beklediği biçimde çağrılabilir.
- Fonksiyonun C'den **adıyla** bulunabilmesi için sembol adının da korunması gerekir. Bunu `#[unsafe(no_mangle)]` sağlar. `unsafe` yazılmasının nedeni, aynı adı taşıyan iki sembolün tanımsız davranışa yol açabilmesidir. Edition 2024'te eski `#[no_mangle]` yazımı derlenmez.
- `unsafe` ise C'den çağrılabilmek için gerekli değildir. "Çağıranın uyması gereken bir koşul var" der. `noise`'un böyle bir koşulu olmadığı için gerçek kodda güvenli bir `extern "C" fn` olarak yazılması daha doğrudur:

```rust
#[unsafe(no_mangle)]
pub extern "C" fn noise() -> i8 {
    1
}

fn main() {
    println!("noise = {}", noise());
}
```

Çıktı:

```text
noise = 1
```

### `static mut`, `unsafe` trait ve `union`

Beş işlemin kalan üçü gündelik kodda daha seyrek karşınıza çıkar.

**Değiştirilebilir `static`.** Program boyunca yaşayan bir değişkeni birden çok thread aynı anda değiştirebilirse veri yarışı doğar. Bu yüzden `static mut`'a erişmek `unsafe`'tir. Çoğu durumda atomik bir tür aynı işi güvenle yapar:[^static-mut]

```rust
use std::sync::atomic::{AtomicU32, Ordering};

static mut HAM_SAYAC: u32 = 0;
static SAYAC: AtomicU32 = AtomicU32::new(0);

/// # Safety
///
/// Aynı anda yalnızca bir thread'den çağrılmalıdır.
unsafe fn ham_artir(n: u32) {
    // SAFETY: Koşulu çağıran garanti eder.
    unsafe { HAM_SAYAC += n }
}

fn main() {
    // SAFETY: Programda tek thread var.
    let ham = unsafe {
        ham_artir(3);
        HAM_SAYAC
    };

    SAYAC.fetch_add(3, Ordering::Relaxed); // unsafe gerekmez
    println!("ham = {ham}, atomik = {}", SAYAC.load(Ordering::Relaxed));
}
```

Çıktı:

```text
ham = 3, atomik = 3
```

`static mut`'a *referans* almak edition 2024'te varsayılan olarak derlenmez (`static_mut_refs`); işaretçi gerekiyorsa `&raw const` ya da `&raw mut` kullanılır.

**`unsafe` trait.** Bir trait'in uyulması gereken ama derleyicinin denetleyemediği bir koşulu varsa trait `unsafe trait` olarak bildirilir. Uygulayan kişi `unsafe impl` yazarak koşulu üstlenir:[^unsafe-trait]

```rust,compile_fail,E0200
unsafe trait Guvenceli {}

struct Tur;

impl Guvenceli for Tur {}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0200]: the trait `Guvenceli` requires an `unsafe impl` declaration
```

Standart kütüphanedeki en bilinen örnekler `Send` ve `Sync`'tir; ayrıntıları [25. bölümdedir](../ileri/25-send-sync.md).

**`union`.** Alanları aynı belleği paylaşan bir türdür; çoğunlukla C ile veri alışverişinde kullanılır. Derleyici hangi alanın yazılmış olduğunu izlemediği için okumak `unsafe`'tir:[^union]

```rust
#[repr(C)]
union Bitler {
    sayi: u32,
    kayan: f32,
}

fn main() {
    let b = Bitler { kayan: 1.0 };

    // SAFETY: Dört baytın tamamı yazıldı ve her bit dizisi geçerli bir u32'dir.
    let sayi = unsafe { b.sayi };
    println!("{sayi:#x}, {:#x}", 1.0_f32.to_bits());
}
```

Çıktı:

```text
0x3f800000, 0x3f800000
```

Bu örnekteki iş için `union` gerekmez; `f32::to_bits` aynı sonucu `unsafe` olmadan verir.

### Güncel yazım

Notlarda ve eski kodlarda rastlayabileceğiniz biçimlerin bugünkü karşılıkları şunlardır:[^guncel]

| Eski biçim | Bugünkü biçim | Durum |
| --- | --- | --- |
| `&s as *const i32` | `&raw const s` | Eskisi geçerli; hizasız ya da başlatılmamış yerlerde yenisi gerekir (1.82'den beri) |
| `extern "C" { … }` | `unsafe extern "C" { … }` | Edition 2024'te zorunlu (1.82'den beri yazılabilir) |
| `#[no_mangle]` | `#[unsafe(no_mangle)]` | Edition 2024'te zorunlu (1.82'den beri yazılabilir) |
| `unsafe fn` gövdesinde doğrudan işlem | Gövdede ayrı `unsafe` blok | Edition 2024'te uyarı |
| `&SAYAC` (`static mut`) | `&raw const SAYAC` ya da atomik tür | Edition 2024'te hata |

## Operatör aşırı yükleme

Rust'ta operatörlerin karşılığı trait'lerdir: `a + b` ifadesi `Add::add(a, b)` çağrısına karşılık gelir. Kendi türünüze `+` öğretmek için `Add` trait'ini uygularsınız:[^operator]

```rust
use std::ops::Add;

#[derive(Debug, PartialEq)]
struct Metre(u32);

impl Add for Metre {
    type Output = Metre; // ilişkili tür: sonucun türü

    fn add(self, baska: Metre) -> Metre {
        Metre(self.0 + baska.0)
    }
}

fn main() {
    let toplam = Metre(3) + Metre(4);
    println!("{toplam:?}");
    assert_eq!(toplam, Metre(7));
}
```

Çıktı:

```text
Metre(7)
```

Trait'in standart kütüphanedeki tanımı, sadeleştirilmiş hâliyle şöyledir: `trait Add<Rhs = Self> { type Output; fn add(self, rhs: Rhs) -> Self::Output; }`. `Rhs = Self` bir *varsayılan tür parametresidir*: `impl Add for Metre` yazdığınızda `Rhs` kendiliğinden `Metre` olur. `Output` ise *ilişkili türdür*; toplamın hangi türü vereceğini uygulayan kişi belirler.

Öğretmediğiniz işlemler derlenmez. `Metre` ile düz bir sayı toplanamaz:

```rust,compile_fail,E0308
# use std::ops::Add;
# struct Metre(u32);
# impl Add for Metre { type Output = Metre; fn add(self, b: Metre) -> Metre { Metre(self.0 + b.0) } }
fn main() {
    let k = Metre(3) + 4;
}
```

Derleyicinin yanıtı:

```text
error[E0308]: mismatched types
```

Hiç öğretmediğimiz çıkarma ise `E0369` verir:

```rust,compile_fail,E0369
# use std::ops::Add;
# struct Metre(u32);
# impl Add for Metre { type Output = Metre; fn add(self, b: Metre) -> Metre { Metre(self.0 + b.0) } }
fn main() {
    let f = Metre(3) - Metre(1);
}
```

Derleyicinin yanıtı:

```text
error[E0369]: cannot subtract `Metre` from `Metre`
```

Farklı bir türle toplamak isterseniz varsayılanı açıkça değiştirirsiniz:

```rust
use std::ops::Add;

#[derive(Debug)]
struct Metre(u32);
struct Kilometre(u32);

impl Add<Kilometre> for Metre {
    type Output = Metre;

    fn add(self, baska: Kilometre) -> Metre {
        Metre(self.0 + baska.0 * 1000)
    }
}

fn main() {
    println!("{:?}", Metre(3) + Kilometre(2));
}
```

Çıktı:

```text
Metre(2003)
```

## Newtype

*Newtype*, var olan bir türü saran tek alanlı tuple struct'tır. İki işe yarar.

### Yetim kuralını aşmak

Ne trait ne de tür sizin crate'inizdeyse ikisini bağlayamazsınız. `Vec` de `Display` de standart kütüphanenindir:

```rust,compile_fail,E0117
use std::fmt;

impl fmt::Display for Vec<String> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.join(", "))
    }
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0117]: only traits defined in the current crate can be implemented for types defined outside of the crate
```

Bu kurala *yetim kuralı* (orphan rule) denir. Olmasaydı iki ayrı kütüphane aynı tür için aynı trait'i uygulayabilir, hangisinin geçerli olduğu bilinemezdi. Türü kendi kutunuza koyarsanız engel kalkar:[^newtype]

```rust
use std::fmt;

struct Sarmal(Vec<String>);

impl fmt::Display for Sarmal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.0.join(", "))
    }
}

fn main() {
    let w = Sarmal(vec![
        String::from("a"),
        String::from("b"),
        String::from("c"),
    ]);

    println!("{w}, {} öğe", w.0.len());
}
```

Çıktı:

```text
[a, b, c], 3 öğe
```

`Sarmal`, `Vec`'in metotlarını kendiliğinden almaz; içerideki değere `.0` ile ulaşılır. Bütün metotları açmak isterseniz `Deref` uygulayabilirsiniz. Yetim kuralının tam hâli [23. bölümdedir](../ileri/23-coherence.md).

### Karışmaması gereken değerleri ayırmak

`Metre` ile `Saniye`'nin içinde aynı `u32` bulunsa da türleri farklıdır. Birini ötekinin yerine veremezsiniz:[^newtype-tur]

```rust,compile_fail,E0308
struct Metre(u32);
struct Saniye(u32);

fn kos(mesafe: Metre) -> u32 {
    mesafe.0
}

fn main() {
    kos(Saniye(5));
}
```

Derleyicinin yanıtı:

```text
error[E0308]: mismatched types
  |
9 |     kos(Saniye(5));
  |     --- ^^^^^^^^^ expected `Metre`, found `Saniye`
  |     |
  |     arguments to this function are incorrect
```

Bu ayrımın çalışma zamanı maliyeti yoktur; sarmalayıcı derleme sırasında ortadan kalkar:

```rust
struct Metre(u32);

fn main() {
    println!("{} {}", size_of::<Metre>(), size_of::<u32>());
    println!("{}", Metre(5).0);
}
```

Çıktı:

```text
4 4
5
```

Bellek düzeninin içteki türle aynı olduğuna güvenmeniz gerekiyorsa (örneğin FFI'de) türe `#[repr(transparent)]` eklenir; dil bu eşliği ancak o zaman garanti eder.[^transparent]

## Tür takma adı

`type` ile var olan bir türe ikinci bir ad verilir. Yeni bir tür oluşmaz; newtype'ın tersine takma ad ile asıl tür birbirinin yerine kullanılabilir:[^takma]

```rust
type Metin = String;
type Islem = Box<dyn Fn(i32) -> i32>;

fn main() {
    let s: Metin = String::from("takma ad");
    let t: String = s; // dönüşüm gerekmez: aynı tür
    println!("{t}");

    let islem: Islem = Box::new(|x| x + 1);
    println!("{}", islem(1));
}
```

Çıktı:

```text
takma ad
2
```

Takma adlar uzun imzaları kısaltmak için kullanılır. Standart kütüphanedeki `std::io::Result<T>`, `Result<T, std::io::Error>`'ın takma adıdır.

## Never type: `!`

Bazı ifadeler hiçbir zaman bir değer üretmez: `panic!()`, `return`, `break`, `continue` ve içinde `break` olmayan `loop {}`. Bunların türü `!`'dir ve adı *never type*'tır. Böyle bir ifadeden sonraki kod hiç çalışmaz. Bu yüzden `!` her türe örtük olarak dönüşebilir: Ortada dönüştürülecek bir değer yoktur.[^never]

```rust
fn sec(b: bool) -> i32 {
    let x = match b {
        true => 5,
        false => panic!("olmamalıydı"), // türü !, i32'ye dönüşür
    };
    x
}

fn main() {
    println!("{}", sec(true));

    let mut toplam = 0;
    for girdi in ["1", "iki", "3"] {
        let sayi: i32 = match girdi.parse() {
            Ok(n) => n,
            Err(_) => continue, // türü !
        };
        toplam += sayi;
    }
    println!("toplam = {toplam}");
}
```

Çıktı:

```text
5
toplam = 4
```

Geri dönmeyen bir fonksiyonun dönüş türü de `!` yazılır. Derleyici böyle bir çağrıdan sonra değer beklemez:

```rust
fn vazgec(neden: &str) -> ! {
    panic!("vazgeçildi: {neden}")
}

fn yarisi(n: u32) -> u32 {
    if n % 2 == 0 { n / 2 } else { vazgec("tek sayı") }
}

fn main() {
    println!("{}", yarisi(8));
}
```

Çıktı:

```text
4
```

### Rust 1.100.0 ile ne değişti?

1.99.0'a kadar `!` yalnızca fonksiyonların dönüş türünde yazılabiliyordu; başka bir yerde yazılırsa derleyici ``error[E0658]: the `!` type is experimental`` hatası verir. 12 Kasım 2026'da çıkan 1.100.0 ile never type her yerde yazılabilir oldu.[^yuz] Üç sonucu vardır:

- `Result<T, !>` gibi türler yazılabilir: "Bu işlem hiç başarısız olmaz" bilgisi türün kendisinde durur.
- Aynı amaçla kullanılan ve eskiden varyantı olmayan ayrı bir enum olan `std::convert::Infallible`, artık `!`'in takma adıdır. Bu sayede `Infallible` kullanan eski API'ler `!` ile birlikte çalışır.
- Geri dönmeyen bir ifadenin hangi türe dönüşeceği belirlenemediğinde derleyicinin seçtiği yedek tür (*never type fallback*), edition 2015, 2018 ve 2021'de de `()` yerine `!` oldu. Edition 2024'te zaten böyleydi.[^fallback]

Bu kitabın örnekleri 1.99.0 ile derlendiği için aynı fikri `Infallible` ile gösterelim. 1.100.0'da imza `Result<T, !>` diye de yazılabilir; gövde aynı kalır:

```rust
use std::convert::Infallible;

fn kesin_al<T>(sonuc: Result<T, Infallible>) -> T {
    let Ok(deger) = sonuc; // Err kolu hiç oluşamaz
    deger
}

fn main() {
    let sayi: u64 = kesin_al(u64::try_from(7_u32));
    let ad: String = kesin_al("ada".parse());
    println!("{sayi} {ad}");
}
```

Çıktı:

```text
7 ada
```

`let Ok(deger) = sonuc;` deseninin çürütülebilir sayılmaması, hata türünün hiçbir değeri olmamasındandır; böyle kolların atlanabilmesi 1.82'den beri geçerlidir.[^bos-desen]

## Dinamik boyutlu türler ve `Sized`

`str`, `[T]` ve `dyn Trait` gibi türlerin boyutu derleme zamanında bilinmez; bunlara *dinamik boyutlu tür* (dynamically sized type, DST) denir. Değişkende doğrudan tutulamazlar:[^dst]

```rust,compile_fail,E0277
fn main() {
    let metin: str = *"merhaba";
}
```

Derleyicinin yanıtı:

```text
error[E0277]: the size for values of type `str` cannot be known at compilation time
  |
2 |     let metin: str = *"merhaba";
  |         ^^^^^ doesn't have a size known at compile-time
  |
  = help: the trait `Sized` is not implemented for `str`
  = note: all local variables must have a statically known size
```

Bu türler her zaman bir işaretçinin arkasında kullanılır: `&str`, `Box<dyn Trait>`, `Rc<[T]>`. İşaretçi, adresin yanında eksik bilgiyi (uzunluk ya da vtable adresi) de taşıyan bir *geniş işaretçidir*.

Aynı nedenle her jenerik parametrenin üstünde görünmeyen bir `T: Sized` sınırı vardır. Aşağıdaki fonksiyon değeri referansla aldığı hâlde bu yüzden `str` kabul etmez:

```rust,compile_fail,E0277
use std::fmt::Debug;

fn goster<T: Debug>(deger: &T) {
    println!("{deger:?}");
}

fn main() {
    goster("merhaba"); // T = str, oysa T: Sized
}
```

Derleyicinin yanıtı:

```text
error[E0277]: the size for values of type `str` cannot be known at compilation time
  |
8 |     goster("merhaba"); // T = str, oysa T: Sized
  |     ------ ^^^^^^^^^ doesn't have a size known at compile-time
  |     |
  |     required by a bound introduced by this call
  |
  = help: the trait `Sized` is not implemented for `str`
```

Sınır `?Sized` ile gevşetilir. Karşılığında değeri yalnızca bir işaretçi üzerinden alabilirsiniz:

```rust
use std::fmt::Debug;

fn goster<T: Debug + ?Sized>(deger: &T) {
    println!("{deger:?}");
}

fn main() {
    goster("merhaba");
    goster(&[1, 2, 3][..]);
    println!("&str {} bayt, &i32 {} bayt", size_of::<&str>(), size_of::<&i32>());
}
```

Çıktı:

```text
"merhaba"
[1, 2, 3]
&str 16 bayt, &i32 8 bayt
```

Boyutlar 64 bitlik makine içindir. Konunun ayrıntıları [22. bölümdedir](../ileri/22-sized-dst.md).

## Fonksiyon işaretçileri

Küçük harfli `fn`, bir fonksiyonu değer olarak taşımayı sağlayan türdür. "Şu sayıya **şu** işlemi uygula" derken işlemin kendisini de vermiş olursunuz:[^fn]

```rust
fn uygula(f: fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}

fn carp_iki(x: i32) -> i32 {
    x * 2
}

fn main() {
    println!("{}", uygula(carp_iki, 10)); // fonksiyonun adı değer gibi verildi
    println!("{}", uygula(|x| x + 1, 10)); // yakalamayan closure da olur

    let yazilar: Vec<String> = [1, 2].iter().map(ToString::to_string).collect();
    let sarili: Vec<Option<i32>> = [1, 2].into_iter().map(Some).collect();
    println!("{yazilar:?} {sarili:?}");
}
```

Çıktı:

```text
20
11
["1", "2"] [Some(1), Some(2)]
```

Fonksiyon işaretçileri `Fn`, `FnMut` ve `FnOnce` trait'lerinin üçünü de uygular. Bu yüzden closure bekleyen her yere, örneğin `map`'e verilebilirler; enum varyantlarının adları da birer fonksiyondur. Tersi yönde yalnızca dışarıdan değişken **yakalamayan** closure'lar `fn` türüne dönüşür:

```rust,compile_fail,E0308
fn uygula(f: fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}

fn main() {
    let yirmi = 20;
    uygula(|x| x + yirmi, 1);
}
```

Derleyicinin yanıtı:

```text
error[E0308]: mismatched types
  |
7 |     uygula(|x| x + yirmi, 1);
  |     ------ ^^^^^^^^^^^^^ expected fn pointer, found closure
  |     |
  |     arguments to this function are incorrect
  |
note: closures can only be coerced to `fn` types if they do not capture any variables
```

Fonksiyon işaretçisi yalnızca bir kod adresidir; `yirmi`'yi koyacak yeri yoktur. Kendi API'nizde iki türü de kabul etmek istiyorsanız parametreyi `fn` yerine trait sınırıyla yazın:

```rust
fn uygula_genel(f: impl Fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}

fn carp_iki(x: i32) -> i32 {
    x * 2
}

fn main() {
    let yirmi = 20;
    println!("{}", uygula_genel(|x| x + yirmi, 1));
    println!("{}", uygula_genel(carp_iki, 10));
}
```

Çıktı:

```text
21
20
```

Parametreyi özellikle `fn` yazmanın bir nedeni, closure tanımayan dillerle (örneğin C ile) konuşmaktır.

## Closure döndürmek

Her closure'ın türü ayrı ve adsızdır. Türün boyutu derleme zamanında bilinir; eksik olan yalnızca imzaya yazılabilecek bir addır. `impl Fn` bu boşluğu doldurur: "Tek bir somut tür döndürüyorum, adını bilmeniz gerekmez."[^closure]

```rust
fn ekleyici_impl(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}

fn ekleyici_kutulu(n: i32) -> Box<dyn Fn(i32) -> i32> {
    Box::new(move |x| x + n)
}

fn main() {
    let yuz_ekle = ekleyici_impl(100); // 100'ü içinde saklayan bir fonksiyon
    let bir_ekle = ekleyici_kutulu(1);

    println!("{} {}", yuz_ekle(5), bir_ekle(5));
}
```

Çıktı:

```text
105 6
```

`move`, `n`'nin closure'ın içine taşınmasını sağlar; fonksiyon bittiğinde `n` yok olacağı için gereklidir. `impl Fn` en ucuz yoldur, ama tek bir türle sınırlıdır. Değişken yakalayan iki farklı closure'ı aynı fonksiyondan döndüremezsiniz:

```rust,compile_fail,E0308
fn ekleyici(n: i32, katla: bool) -> impl Fn(i32) -> i32 {
    if katla {
        move |x| x * n
    } else {
        move |x| x + n
    }
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0308]: `if` and `else` have incompatible types
  |
2 | /     if katla {
3 | |         move |x| x * n
  | |         --------------
  | |         |
  | |         the expected closure
  | |         expected because of this
4 | |     } else {
5 | |         move |x| x + n
  | |         ^^^^^^^^^^^^^^ expected closure, found a different closure
6 | |     }
  | |_____- `if` and `else` have incompatible types
  |
  = note: no two closures, even if identical, have the same type
  = help: consider boxing your closure and/or using it as a trait object
```

`Box<dyn Fn>` closure'ları heap'te bir trait nesnesi olarak taşır; farklı closure'lar aynı imzadan dönebilir, aynı `Vec`'e konabilir. Bedeli bir heap tahsisi ve dolaylı çağrıdır:

```rust
fn ekleyici(n: i32, katla: bool) -> Box<dyn Fn(i32) -> i32> {
    if katla {
        Box::new(move |x| x * n)
    } else {
        Box::new(move |x| x + n)
    }
}

fn main() {
    let islemler = vec![ekleyici(3, true), ekleyici(3, false)];
    for islem in &islemler {
        println!("{}", islem(10));
    }
}
```

Çıktı:

```text
30
13
```

> [!NOTE]
> Hiç değişken yakalamayan iki closure `if` ve `else` kollarından `impl Fn` olarak dönebilir: İkisi de aynı `fn` türüne örtük olarak dönüşür.[^lub]

## Makrolar

Makro, kod üreten koddur; buna *metaprogramlama* denir. Çağrıda adın sonuna yazılan `!`, "bu bir fonksiyon değil, makro" der. Bir fonksiyonun kaç argüman alacağı baştan bellidir; makro ise istediği kadar argüman alabilir, çünkü çalışan bir şey değil, kod üreten bir kalıptır.[^makro]

```rust
macro_rules! benim_vec {
    ( $( $x:expr ),* ) => {{
        let mut v = Vec::new();
        $( v.push($x); )*
        v
    }};
}

fn main() {
    let v: Vec<i32> = benim_vec![10, 20, 30];
    println!("{v:?}");

    // Yukarıdaki çağrının genişlemiş hâli:
    let ayni: Vec<i32> = {
        let mut v = Vec::new();
        v.push(10);
        v.push(20);
        v.push(30);
        v
    };
    println!("{ayni:?}");
}
```

Çıktı:

```text
[10, 20, 30]
[10, 20, 30]
```

Desendeki `$( $x:expr ),*`, "virgülle ayrılmış sıfır ya da daha fazla ifade" demektir; `$x:expr` bir ifadeyi yakalar. Gövdedeki `$( v.push($x); )*` yakalanan her ifade için bir `push` satırı üretir. Desen sıfır argümanı da kabul ettiği için boş çağrı da geçerlidir:

```rust
# #[allow(unused_mut)]
# fn main() {
macro_rules! benim_vec {
    ( $( $x:expr ),* ) => {{
        let mut v = Vec::new();
        $( v.push($x); )*
        v
    }};
}

let bos: Vec<i32> = benim_vec![];
println!("{}", bos.len());
# }
```

Çıktı:

```text
0
```

`macro_rules!` makroları, metin sırasında **kullanılmadan önce** tanımlanmalıdır. Tanım çağrıdan sonra gelirse derleyici ``cannot find macro `benim_vec` in this scope`` hatasını verir ve tanımı öne almanızı önerir. Başka bir modüldeki makroyu kullanmanın bir yolu `#[macro_use]` özniteliğidir.[^metin-sirasi]

Makroların iki ailesi vardır:

- **Bildirimsel (declarative):** `macro_rules!` ile yazılır; desen ve kod şablonundan oluşur. Dilbilgisi üzerinde çalışır.
- **Yordamsal (procedural):** `TokenStream` alıp `TokenStream` üreten fonksiyonlardır. `proc-macro = true` ayarlı ayrı bir crate'te yazılırlar ve üç biçimleri vardır: türetme (`#[derive(Debug)]`), öznitelik (`#[route(...)]`) ve fonksiyon benzeri (`sql!(...)`).

Fonksiyon çalışma zamanında çağrılır ve imzası sabittir; makro derleme zamanında kod üretir. "Her tür için kod yaz" türünden işler (`derive`) ve değişken sayıda argüman alan küçük diller (DSL) bu yüzden makroyla yazılır. Ayrıntılar [20. bölümde](20-macros.md).

## Hata kodu başvurusu

| Kod | Ne zaman görülür? | Ne yapmalı? |
| --- | --- | --- |
| `E0133` | `unsafe` isteyen bir işlem `unsafe` blok dışında yapıldığında | Koşulları doğrulayıp `unsafe` bloğa alın ve `// SAFETY:` gerekçesini yazın. |
| `E0793` | `packed` yapının hizasız alanına referans alındığında | `&raw const` ile işaretçi alıp `read_unaligned` kullanın. |
| `E0499` | Aynı dilimden iki `&mut` istendiğinde | `split_at_mut` gibi hazır soyutlamaları kullanın. |
| `E0200` | `unsafe` bir trait düz `impl` ile uygulandığında | Koşulu doğrulayıp `unsafe impl` yazın. |
| `E0308` | Farklı newtype'lar, `Metre + 4` ya da yakalayan closure `fn` beklenen yere verildiğinde | Doğru türü verin; closure için `impl Fn` ya da `Box<dyn Fn>` kullanın. |
| `E0369` | Operatörün trait'i tür için uygulanmadığında | İlgili trait'i (`Sub`, `Mul` gibi) uygulayın. |
| `E0117` | Yabancı trait yabancı türe uygulandığında | Türü bir newtype ile sarın. |
| `E0277` | Boyutu bilinmeyen tür `Sized` beklenen yerde kullanıldığında | İşaretçi arkasında tutun; jenerikte `?Sized` ekleyin. |
| `E0658` | 1.99.0 ve öncesinde `!` dönüş türü dışında yazıldığında | `Infallible` kullanın ya da 1.100.0'a geçin. |

## Perde arkası: Derleyici ne yapıyor?

> [!NOTE]
> Bu başlık ilk okuyuşta atlanabilir. Aklınızda kalması gereken tek cümle şudur: `unsafe` ödünç kurallarını kapatmaz; yalnızca birkaç tehlikeli işlem için "sorumluluk bende" demenizi sağlar.

**`unsafe` denetimi.** İşlemlerin listesi derleyicide `UnsafeOpKind` enum'ıdır; denetim `check_unsafety` fonksiyonunda, THIR adlı ara biçim üzerinde yapılır.[^check-unsafety] THIR, MIR'den bir adım önceki, türleri belirlenmiş biçimdir. Ödünç denetimi ise MIR üzerinde çalışır; iki denetim birbirinden ayrı olduğu için `unsafe` yazmak ödünç hatalarını susturmaz.

**Operatörler.** Tür denetleyicisi `&&` ve `||` dışındaki ikili işlemlerin hepsine önce aşırı yüklenmiş gözüyle bakar: Operatörün trait'ini (`+` için `Add`) bulur ve ifadeyi metot çağrısı olarak çözer; bu iş `lookup_op_method` içindedir. İki yerleşik sayı söz konusuysa işlem ayrıca yerleşik kurallara göre ele alınır.[^op-kod] `Metre - Metre` için aranan `Sub` uygulaması bulunamayınca `E0369` doğar.

**Newtype.** Sarmalayıcı, üretilen kodda ek bir işlem gerektirmez; alan erişimi derleme sırasında ortadan kalkar. Dil, bellek düzeninin içteki türle aynı olmasını yalnızca `#[repr(transparent)]` yazıldığında garanti eder.[^transparent]

**`!` dönüşümü.** Tür denetleyicisi `!` türünden bir ifadeyi başka bir tür beklenen yerde görünce `NeverToAny` uyarlaması ekler.[^never-kod] 1.99.0'da `!`'in dönüş türü dışında yazılmasını, ayrıştırılan koda bakan özellik kapısı denetimi engeller.[^never-kapi]

**Makroların açılması.** Makro çağrıları, ad çözümleme ile iç içe yürüyen genişleme aşamasında açılır; tür denetimi ve ödünç denetimi yalnızca açılmış kodu görür.[^genisleme] `macro_rules!` makrolarını `MacroRulesMacroExpander`, yordamsal makroları ise biçimlerine göre `BangProcMacro`, `AttrProcMacro` ve `DeriveProcMacro` işler.[^makro-kod]

## Özet

- `unsafe` beş tehlikeli işlemin kapısını açar (Reference birkaç madde daha sayar), ödünç denetleyicisini kapatmaz. Bloğu küçük tutun, gerekçesini yazın, güvenli bir API arkasına saklayın.
- Ham işaretçi oluşturmak güvenli, başvurusunu çözmek `unsafe`'tir. Bugünkü yazımı `&raw const` ve `&raw mut`'tur.
- Edition 2024'te `unsafe extern`, `#[unsafe(no_mangle)]` ve `unsafe fn` içinde ayrı `unsafe` blok beklenir.
- Operatörlerin karşılığı trait'lerdir; `Add<Rhs = Self>` varsayılan tür parametresi sayesinde kısa yazılır.
- Newtype yetim kuralını aşar ve türleri birbirinden ayırır; tür takma adı yalnızca ikinci bir addır.
- `!` hiçbir zaman değer üretmeyen ifadelerin türüdür ve her türe dönüşür. 1.100.0'dan beri her yerde yazılabilir.
- Yakalamayan closure `fn` olur; closure döndürmek için `impl Fn`, farklı closure'lar için `Box<dyn Fn>` kullanılır.
- Makro derleme zamanında kod üretir; fonksiyonun yapamadığını yapar.

Sıradaki bölüm [20. Makrolar](20-macros.md), bildirimsel ve yordamsal makroları ayrıntısıyla ele alıyor.

## Kaynaklar

[^giris]: The Rust Programming Language, [20. Advanced Features](https://doc.rust-lang.org/book/ch20-00-advanced-features.html).
[^unsafe]: The Rust Programming Language, [20.1 Unsafe Rust](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html); The Rustonomicon, [What Unsafe Can Do](https://doc.rust-lang.org/nomicon/what-unsafe-does.html).
[^bes-guc]: The Rust Programming Language, [Performing Unsafe Superpowers](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html#performing-unsafe-superpowers); hata dizini, [`E0133`](https://doc.rust-lang.org/error_codes/E0133.html).
[^reference-liste]: The Rust Reference, [Unsafety](https://doc.rust-lang.org/reference/unsafety.html#unsafety).
[^ham]: The Rust Programming Language, [Dereferencing a Raw Pointer](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html#dereferencing-a-raw-pointer); std, [`pointer` temel türü](https://doc.rust-lang.org/std/primitive.pointer.html).
[^ub]: The Rust Reference, [Behavior considered undefined](https://doc.rust-lang.org/reference/behavior-considered-undefined.html).
[^ptr-read]: std, [`ptr::read`](https://doc.rust-lang.org/std/ptr/fn.read.html).
[^raw]: The Rust Reference, [Raw borrow operators](https://doc.rust-lang.org/reference/expressions/operator-expr.html#raw-borrow-operators); Rust blogu, [Announcing Rust 1.82.0](https://blog.rust-lang.org/2024/10/17/Rust-1.82.0/); hata dizini, [`E0793`](https://doc.rust-lang.org/error_codes/E0793.html).
[^unsafe-fn]: The Rust Programming Language, [Calling an Unsafe Function or Method](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html#calling-an-unsafe-function-or-method); The Rust Reference, [Unsafe functions](https://doc.rust-lang.org/reference/unsafe-keyword.html#unsafe-functions-unsafe-fn).
[^op-in-fn]: Edition Guide, [`unsafe_op_in_unsafe_fn` warning](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html).
[^guvenli-soyutlama]: The Rust Programming Language, [Creating a Safe Abstraction over Unsafe Code](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html#creating-a-safe-abstraction-over-unsafe-code); std, [`slice::from_raw_parts_mut`](https://doc.rust-lang.org/std/slice/fn.from_raw_parts_mut.html) ve [`slice::split_at_mut`](https://doc.rust-lang.org/std/primitive.slice.html#method.split_at_mut).
[^guven]: The Rustonomicon, [Working with Unsafe](https://doc.rust-lang.org/nomicon/working-with-unsafe.html).
[^miri]: The Rust Programming Language, [Using Miri to Check Unsafe Code](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html#using-miri-to-check-unsafe-code).
[^extern]: The Rust Reference, [External blocks](https://doc.rust-lang.org/reference/items/external-blocks.html); Edition Guide, [Unsafe `extern` blocks](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-extern.html).
[^extern-fn]: The Rust Reference, [Extern function qualifier](https://doc.rust-lang.org/reference/items/functions.html#extern-function-qualifier) ve [The `no_mangle` attribute](https://doc.rust-lang.org/reference/abi.html#the-no_mangle-attribute); The Rust Programming Language, [Calling Rust Functions from Other Languages](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html#calling-rust-functions-from-other-languages); Edition Guide, [Unsafe attributes](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-attributes.html).
[^static-mut]: The Rust Programming Language, [Accessing or Modifying a Mutable Static Variable](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html#accessing-or-modifying-a-mutable-static-variable); Edition Guide, [Disallow references to `static mut`](https://doc.rust-lang.org/edition-guide/rust-2024/static-mut-references.html).
[^unsafe-trait]: The Rust Programming Language, [Implementing an Unsafe Trait](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html#implementing-an-unsafe-trait); hata dizini, [`E0200`](https://doc.rust-lang.org/error_codes/E0200.html).
[^union]: The Rust Reference, [Unions](https://doc.rust-lang.org/reference/items/unions.html); The Rust Programming Language, [Accessing Fields of a Union](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html#accessing-fields-of-a-union).
[^guncel]: Rust blogu, [Announcing Rust 1.82.0](https://blog.rust-lang.org/2024/10/17/Rust-1.82.0/); Edition Guide, [Unsafe `extern` blocks](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-extern.html), [Unsafe attributes](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-attributes.html), [`unsafe_op_in_unsafe_fn` warning](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html) ve [Disallow references to `static mut`](https://doc.rust-lang.org/edition-guide/rust-2024/static-mut-references.html).
[^operator]: The Rust Programming Language, [Using Default Generic Parameters and Operator Overloading](https://doc.rust-lang.org/book/ch20-02-advanced-traits.html#using-default-generic-parameters-and-operator-overloading); std, [`Add`](https://doc.rust-lang.org/std/ops/trait.Add.html); hata dizini, [`E0369`](https://doc.rust-lang.org/error_codes/E0369.html).
[^newtype]: The Rust Programming Language, [Implementing External Traits with the Newtype Pattern](https://doc.rust-lang.org/book/ch20-02-advanced-traits.html#implementing-external-traits-with-the-newtype-pattern); The Rust Reference, [Orphan rules](https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules); hata dizini, [`E0117`](https://doc.rust-lang.org/error_codes/E0117.html).
[^newtype-tur]: The Rust Programming Language, [Type Safety and Abstraction with the Newtype Pattern](https://doc.rust-lang.org/book/ch20-03-advanced-types.html#type-safety-and-abstraction-with-the-newtype-pattern).
[^transparent]: The Rust Reference, [The `transparent` representation](https://doc.rust-lang.org/reference/type-layout.html#the-transparent-representation); The Rust Programming Language, [Implementing External Traits with the Newtype Pattern](https://doc.rust-lang.org/book/ch20-02-advanced-traits.html#implementing-external-traits-with-the-newtype-pattern).
[^takma]: The Rust Programming Language, [Type Synonyms and Type Aliases](https://doc.rust-lang.org/book/ch20-03-advanced-types.html#type-synonyms-and-type-aliases); The Rust Reference, [Type aliases](https://doc.rust-lang.org/reference/items/type-aliases.html).
[^never]: The Rust Programming Language, [The Never Type That Never Returns](https://doc.rust-lang.org/book/ch20-03-advanced-types.html#the-never-type-that-never-returns); The Rust Reference, [Never type](https://doc.rust-lang.org/reference/types/never.html#never-type) ve [Coercion types](https://doc.rust-lang.org/reference/type-coercions.html#coercion-types).
[^yuz]: Rust blogu, [Announcing Rust 1.100.0](https://blog.rust-lang.org/2026/11/12/Rust-1.100.0/); The Rust Reference 1.100.0, [Never type](https://doc.rust-lang.org/1.100.0/reference/types/never.html); std 1.100.0, [`Infallible`](https://doc.rust-lang.org/1.100.0/std/convert/type.Infallible.html).
[^fallback]: The Rust Reference, [Divergence: Fallback](https://doc.rust-lang.org/reference/divergence.html#fallback); Edition Guide, [Never type fallback change](https://doc.rust-lang.org/edition-guide/rust-2024/never-type-fallback.html).
[^bos-desen]: Rust blogu, [Announcing Rust 1.82.0](https://blog.rust-lang.org/2024/10/17/Rust-1.82.0/) (*Omitting empty types in pattern matching*); std 1.99.0, [`Infallible`](https://doc.rust-lang.org/1.99.0/std/convert/enum.Infallible.html).
[^dst]: The Rust Programming Language, [Dynamically Sized Types and the `Sized` Trait](https://doc.rust-lang.org/book/ch20-03-advanced-types.html#dynamically-sized-types-and-the-sized-trait); The Rust Reference, [Dynamically sized types](https://doc.rust-lang.org/reference/dynamically-sized-types.html) ve [`Sized`](https://doc.rust-lang.org/reference/special-types-and-traits.html#sized).
[^fn]: The Rust Programming Language, [Function Pointers](https://doc.rust-lang.org/book/ch20-04-advanced-functions-and-closures.html#function-pointers); The Rust Reference, [Function pointer types](https://doc.rust-lang.org/reference/types/function-pointer.html).
[^closure]: The Rust Programming Language, [Returning Closures](https://doc.rust-lang.org/book/ch20-04-advanced-functions-and-closures.html#returning-closures); The Rust Reference, [Closure types](https://doc.rust-lang.org/reference/types/closure.html).
[^lub]: The Rust Reference, [Least upper bound coercions](https://doc.rust-lang.org/reference/type-coercions.html#least-upper-bound-coercions) ve [Coercion types](https://doc.rust-lang.org/reference/type-coercions.html#coercion-types).
[^makro]: The Rust Programming Language, [The Difference Between Macros and Functions](https://doc.rust-lang.org/book/ch20-05-macros.html#the-difference-between-macros-and-functions) ve [Declarative Macros for General Metaprogramming](https://doc.rust-lang.org/book/ch20-05-macros.html#declarative-macros-for-general-metaprogramming).
[^metin-sirasi]: The Rust Reference, [Macros by example: Textual scope](https://doc.rust-lang.org/reference/macros-by-example.html#textual-scope).
[^check-unsafety]: rustc 1.99.0 kaynak kodu, [`check_unsafety.rs`, `UnsafeOpKind`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_mir_build/src/check_unsafety.rs#L643) ve [`check_unsafety`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_mir_build/src/check_unsafety.rs#L1053); rustc geliştirme kılavuzu, [Unsafety checking](https://rustc-dev-guide.rust-lang.org/unsafety-checking.html).
[^op-kod]: rustc 1.99.0 kaynak kodu, [`op.rs`, `check_expr_binop`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_typeck/src/op.rs#L89) ve [`lookup_op_method`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_typeck/src/op.rs#L1071).
[^never-kod]: rustc 1.99.0 kaynak kodu, [`coercion.rs`, `NeverToAny`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_typeck/src/coercion.rs#L253).
[^never-kapi]: rustc 1.99.0 kaynak kodu, [`feature_gate.rs`, `never_type`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_ast_passes/src/feature_gate.rs#L265).
[^genisleme]: rustc geliştirme kılavuzu, [Macro expansion](https://rustc-dev-guide.rust-lang.org/macro-expansion.html); rustc 1.99.0 kaynak kodu, [`expand.rs`, `expand_crate`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_expand/src/expand.rs#L450).
[^makro-kod]: rustc 1.99.0 kaynak kodu, [`macro_rules.rs`, `MacroRulesMacroExpander`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_expand/src/mbe/macro_rules.rs#L194) ve [`proc_macro.rs`, `BangProcMacro`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_expand/src/proc_macro.rs#L33); The Rust Reference, [Procedural macros](https://doc.rust-lang.org/reference/procedural-macros.html).
