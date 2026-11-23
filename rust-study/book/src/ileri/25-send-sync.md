# 25. `Send` ve `Sync`'in İç Yüzü

Bir işi iki thread arasında paylaştırmak istediğinizde ilk soru "bu veriyi öteki thread'e verebilir miyim?" olur. Rust bu soruyu çalışma zamanına bırakmaz; türün içindeki parçalara bakarak derleme zamanında yanıtlar. Yanıtı iki trait taşır: `Send` ve `Sync`. Onları siz yazmazsınız, ama kullandığınız hemen her tür için bir kararları vardır.

[15. bölümde](../orta/15-concurrency.md) bu iki trait'i dışarıdan tanımıştınız. Burada içlerine bakacağız: Karar nasıl veriliyor, `Cell` neden paylaşılamazken `Mutex<Cell<i32>>` paylaşılabiliyor, kendi türünüz bu üyeliği nasıl kazanıyor ya da sessizce nasıl kaybediyor?

Konunun kökünde tek bir tür vardır: `UnsafeCell`. `Cell`, `RefCell`, `Mutex` ve atomikler onun üstüne kuruludur. Bu yüzden önce ona, sonra thread güvenliğinin nasıl belirlendiğine bakıyoruz.

> [!NOTE]
> Bu bölümün notları ve çalıştırılabilir kodu `learn/advanced-concepts/ileri-send-sync` dizinindedir. Depo kökünden `cargo run -p ileri-send-sync` komutuyla çalıştırabilirsiniz. Auto trait ve negatif impl notları `learn/basic-concepts` dizinindedir. `&` ve `&mut` kurallarını ([3. bölüm](../temel/03-ownership.md)), `Rc` ile `RefCell`'i ([14. bölüm](../orta/14-smart-pointers.md)) ve `Arc` ile `Mutex`'i ([15. bölüm](../orta/15-concurrency.md)) bildiğiniz varsayılır.

## Günlük hayattan bir benzetme

Müzede eşyaların önünde "Dokunmayın, yalnızca bakın" yazar; `&` referansı böyledir. Derleyici de buna güvenir: "Kimse dokunamayacağına göre bir kez bakıp aklımda tutabilirim." `UnsafeCell` ise üstünde "Dikkat: siz bakarken bu değişebilir" yazan özel bir etikettir. Derleyici bu etiketi görünce ezberine güvenmez ve `&` üzerinden yazmaya izin veren kapıyı açar. O kapıdan doğru geçmek ise size kalmıştır.

`Send` ve `Sync` için evde iki oda ve her odada bir çocuk düşünün; odalar thread'lerdir. `Send`, "bu oyuncağı öteki odaya götürebilirsiniz" etiketidir. `Sync` ise "bu oyuncağa iki odadan aynı anda bakılabilir" demektir. Etiketi siz yapıştırmazsınız: Öğretmen kutunun içine bakıp karar verir. İçinde ıslak boya olan bir kutu öteki odaya taşınabilir, ama iki çocuk boyaya aynı anda uzanmasın diye ikinci etiket verilmez.

İki etiket birbirine bağlıdır. Bir şeye iki odadan bakılabiliyorsa, onu gösteren ok öteki odaya götürülebiliyor demektir.

<details>
<summary>Bu bölümde geçen terimler</summary>

- **thread (iş parçacığı):** Program içinde ötekilerle aynı anda çalışabilen ayrı bir yürütme dizisi.
- **veri yarışı (data race):** İki thread'in aynı belleğe eşzamanlama olmadan erişmesi ve en az birinin yazması. Tanımsız davranıştır; güvenli Rust'ta derlenmez.
- **iç değiştirilebilirlik (interior mutability):** `&` üzerinden bile içeriği değiştirebilmek.
- **`UnsafeCell`:** İç değiştirilebilirliğin temel taşı.
- **atomik (atomic):** Yarıda kesilemeyen tek hamleyle okunup yazılan küçük değer.
- **`Send`:** Değerin sahipliği başka bir thread'e verilebilir.
- **`Sync`:** Değere birçok thread aynı anda `&` ile bakabilir.
- **işaretleyici trait (marker trait):** Metodu olmayan, türün bir özelliği taşıdığını bildiren trait. `Copy`, `Send` ve `Sync` böyledir.
- **auto trait:** Derleyicinin türün alanlarına bakarak kendiliğinden verdiği işaretleyici trait.
- **negatif impl (negative impl):** "Bu tür bu trait'i uygulamaz" diye açıkça yazılmış kayıt: `impl !Sync for ...`.
- **ham işaretçi (raw pointer):** `*const T` ve `*mut T`; geçerliliğini derleyicinin denetlemediği adres.
- **`unsafe impl`:** "Bu trait'in gerektirdiği koşulu derleyici değil, ben garanti ediyorum" sözü.
- **çakışmama varsayımı (no-alias):** Derleyicinin, `&` ardındaki verinin değişmediğine ve `&mut` ardındakine başkasının erişmediğine güvenmesi.
- **optimizasyon:** Derleyicinin, sonucu değiştirmeden kodu hızlandırması.
- **anlamsal sürümleme (semver):** Sürüm numaralarının neyi bozabileceğini belirleyen kural.

</details>

## `UnsafeCell`: `&` üzerinden yazmanın tek kapısı

Sıradan bir `&T` üzerinden değer değiştirilemez:

```rust,compile_fail,E0594
fn main() {
    let x = 5;
    let r = &x;
    *r = 10;
    println!("{r}");
}
```

Derleyicinin yanıtı:

```text
error[E0594]: cannot assign to `*r`, which is behind a `&` reference
```

Derleyici bu kurala yalnızca uydurmakla kalmaz, ona dayanarak optimizasyon da yapar: `&T` ardındaki değeri bir kez okuyup saklayabilir. Sayaç, önbellek ve kilit gibi yapılar ise "yalnızca bakan" birçok yerden güncellenmek zorundadır. Bunun için dilde tek bir kapı vardır: `UnsafeCell<T>`. `get` metodu `&self` alır ve `*mut T` döndürür. Bu türün dışında, `&T`'den elde edilen bir işaretçiyle yazmak tanımsız davranıştır.[^unsafecell]

Bu kapıyla `Cell`'in küçük bir kopyasını kuralım:

```rust
use std::cell::UnsafeCell;

struct MiniCell<T> {
    v: UnsafeCell<T>,
}

impl<T: Copy> MiniCell<T> {
    fn yeni(v: T) -> MiniCell<T> {
        MiniCell { v: UnsafeCell::new(v) }
    }

    fn al(&self) -> T {
        // SAFETY: MiniCell Sync değildir ve içeriye referans vermez; bu okuma
        // sırasında başka bir erişim olamaz.
        unsafe { *self.v.get() }
    }

    fn yaz(&self, yeni: T) {
        // SAFETY: `al` ile aynı gerekçe; içeriye verilmiş hiçbir referans yoktur.
        unsafe { *self.v.get() = yeni }
    }
}

fn main() {
    let c = MiniCell::yeni(5);
    let r1 = &c;
    let r2 = &c; // iki paylaşımlı referans
    r1.yaz(10); // & üzerinden yazma
    println!("r2.al() = {}", r2.al());
}
```

Çıktı:

```text
r2.al() = 10
```

Alan `UnsafeCell<T>` yerine düz `T` olsaydı `yaz` derlenmezdi; `&self` ardındaki alana atama yine `E0594` verirdi.

`UnsafeCell`'in neyi gevşettiğini tam bilmek önemlidir. Kaldırdığı tek varsayım, "`&T` ardındaki veri değişmez" varsayımıdır. `&mut T`'nin tekil olması kuralına dokunmaz: İçeriye aynı anda iki `&mut` üretmek tanımsız davranıştır. Veri yarışı yasağına da dokunmaz: İki thread eşzamanlama olmadan yazarsa sonuç yine tanımsız davranıştır.[^kurallar] Kısacası `UnsafeCell` ödünç kurallarını büsbütün kaldırmaz; çakışan erişim olmamasını sağlama sorumluluğunu derleyiciden alıp size verir.

`MiniCell`'in güvenli olması bu yüzden iki şeye dayanır. Birincisi, `al` ve `yaz` dışarıya içeriyi gösteren bir referans vermez; değeri kopyalar. İkincisi, `MiniCell`'in `Sync` olmamasıdır; nedenini birazdan göreceğiz.

`UnsafeCell<T>`, içinden hem okunup hem yazılabildiği için `T`'ye göre invaryanttır ([21. bölüm](21-variance.md)). `Cell` ve `RefCell` bu özelliği ondan alır.

### Aynı temel, farklı disiplin

Standart kütüphanedeki iç değiştirilebilirlik türlerinin hepsi `UnsafeCell`'i sarar ve üstüne kendi güvenlik disiplinini koyar:[^icdeg]

| Tür | Güvenliği nasıl sağlar? | İhlalde ne olur? |
| --- | --- | --- |
| `Cell<T>` | İçeriye referans vermez; değeri bütün olarak alır, koyar ya da değiştirir. | İhlal edilemez. |
| `RefCell<T>` | Ödünçleri çalışma zamanında sayar. | `borrow`/`borrow_mut` panikler. |
| `Mutex<T>` | Kilitle aynı anda tek erişime izin verir. | İkinci thread bekler. |
| `RwLock<T>` | Kilitle ya birçok okuyucuya ya bir yazara izin verir. | Bekler. |
| `AtomicI32` ve benzerleri | İşlemcinin bölünmez işlemlerini kullanır. | İhlal edilemez. |

`Cell`'in yalnızca `Copy` türlerle çalıştığı sık söylenir; doğru değildir. `Copy` isteyen yalnızca `get` metodudur. Öteki metotlar değeri taşır:

```rust
use std::cell::Cell;

fn main() {
    let c = Cell::new(String::from("a"));
    c.set(String::from("b"));
    let eski = c.replace(String::from("c"));
    let son = c.take(); // yerine String::default() konur
    println!("{eski} {son}");
}
```

Çıktı:

```text
b c
```

## `Send` ve `Sync` ne söyler?

İkisi de metodu olmayan işaretleyici trait'lerdir:[^tanim]

- `T: Send`, bir `T` değerinin sahipliğini başka bir thread'e vermenin güvenli olduğunu söyler.
- `T: Sync`, bir `T` değerine birçok thread'den aynı anda `&T` ile erişmenin güvenli olduğunu söyler.

İkincinin kesin tanımı birincisine dayanır: `T`, ancak ve ancak `&T` `Send` ise `Sync`'tir. Bir şeyi paylaşmak, onu gösteren referansı öteki thread'e göndermek demektir. Her thread'in erişebildiği `static` değişkenlerin türü de aynı nedenle `Sync` olmalıdır.

Bir türün üyeliğini sınamak için hiçbir şey yapmayan iki jenerik fonksiyon yeter. Çağrı derleniyorsa üyelik vardır; çalışma zamanında hiçbir iş yapılmaz:

```rust
use std::cell::{Cell, RefCell};
use std::sync::atomic::AtomicI32;
use std::sync::{Arc, Mutex};

fn send_ister<T: Send>() {}
fn sync_ister<T: Sync>() {}

fn main() {
    send_ister::<i32>();
    sync_ister::<i32>();

    send_ister::<Cell<i32>>(); // taşınabilir, paylaşılamaz
    send_ister::<RefCell<i32>>();

    sync_ister::<Mutex<Cell<i32>>>(); // kilit içinde paylaşılabilir
    send_ister::<Arc<Mutex<Cell<i32>>>>();
    sync_ister::<AtomicI32>();

    println!("bütün üyelikler doğrulandı");
}
```

Çıktı:

```text
bütün üyelikler doğrulandı
```

En sık karşılaşacağınız türlerin durumu şöyledir:

| Tür | `Send` | `Sync` | Neden? |
| --- | --- | --- | --- |
| `i32`, `String`, `Vec<i32>` | evet | evet | Sıradan sahip olunan veri. |
| `Cell<i32>`, `RefCell<i32>` | evet | hayır | `&` ile eşzamanlamasız yazılabilir. |
| `Rc<i32>` | hayır | hayır | Referans sayacı atomik değildir. |
| `*const T`, `*mut T` | hayır | hayır | Derleyici gösterdiği yer hakkında bir şey bilmez. |
| `Arc<T>` | `T: Send + Sync` ise | `T: Send + Sync` ise | Aynı `T`'yi birçok thread'e paylaştırır. |
| `Mutex<T>` | `T: Send` ise | `T: Send` ise | Aynı anda tek thread erişir. |
| `RwLock<T>` | `T: Send` ise | `T: Send + Sync` ise | Birçok okuyucu aynı anda `&T` alır. |
| `MutexGuard<'_, T>` | hayır | `T: Sync` ise | Kilidi alan thread bırakmalıdır. |
| `mpsc::Sender<T>` | `T: Send` ise | `T: Send` ise | Gönderen uç paylaşılabilir. |
| `mpsc::Receiver<T>` | `T: Send` ise | hayır | Alıcı uç tek tüketicilidir. |

## Auto trait: etiketi derleyici verir

Her yeni struct'a "ben de `Send`'im" yazmak zorunda olsaydınız iki tür yanlış yapılırdı. Unutulan her tür, thread'lerle kullanılamaz hâle gelirdi. Daha kötüsü, parçalarından biri `Send` olmayan bir türe yanlışlıkla `Send` denirse veri yarışı doğardı. Oysa bu özellik *kompozisyoneldir*: Bir türün bütün parçaları `Send` ise kendisi de `Send`'dir. Derleyici bunu parçalardan kendisi çıkarabilir.

Böyle çalışan trait'lere *auto trait* denir. Kural şudur: Bir tür için açıkça yazılmış bir impl yoksa, derleyici bütün alanları trait'i sağlayan türe trait'i kendiliğinden verir.[^auto] Tek bir uygunsuz alan bütün türü dışarıda bırakır. Struct, enum, demet, dizi ve closure için kural aynıdır; closure'ın alanları yakaladığı değerlerdir. Standart kütüphanedeki auto trait'lerin başlıcaları `Send`, `Sync`, `Unpin`, `UnwindSafe` ve `RefUnwindSafe`'dir.

Uymayan bir tür `E0277` verir:

```rust,compile_fail,E0277
use std::rc::Rc;

fn send_ister<T: Send>() {}

fn main() {
    send_ister::<Rc<i32>>();
}
```

Derleyicinin yanıtı:

```text
error[E0277]: `Rc<i32>` cannot be sent between threads safely
  |
6 |     send_ister::<Rc<i32>>();
  |                  ^^^^^^^ `Rc<i32>` cannot be sent between threads safely
  |
  = help: the trait `Send` is not implemented for `Rc<i32>`
```

Uygunsuzluk derinde bir alandaysa derleyici yolu da gösterir. Aşağıdaki `MiniCell` hiçbir şey yazmadığı hâlde `Sync` değildir, çünkü içindeki `UnsafeCell` değildir:

```rust,compile_fail,E0277
use std::cell::UnsafeCell;

struct MiniCell<T> {
    v: UnsafeCell<T>,
}

fn sync_ister<T: Sync>() {}

fn main() {
    sync_ister::<MiniCell<i32>>();
}
```

Derleyicinin yanıtı:

```text
error[E0277]: `UnsafeCell<i32>` cannot be shared between threads safely
   |
10 |     sync_ister::<MiniCell<i32>>();
   |                  ^^^^^^^^^^^^^ `UnsafeCell<i32>` cannot be shared between threads safely
   |
   = help: within `MiniCell<i32>`, the trait `Sync` is not implemented for `UnsafeCell<i32>`
note: required because it appears within the type `MiniCell<i32>`
```

Aynı hata ham işaretçi alanında da çıkar; bu kez eksik olan `Send`'dir:

```rust,compile_fail,E0277
struct Tut {
    p: *const u8,
}

fn send_ister<T: Send>() {}

fn main() {
    send_ister::<Tut>();
}
```

Derleyicinin yanıtı:

```text
error[E0277]: `*const u8` cannot be sent between threads safely
  |
  = help: within `Tut`, the trait `Send` is not implemented for `*const u8`
```

## Negatif impl: istisnayı açıkça yazmak

Auto trait kuralı tek başına kalsaydı `UnsafeCell<i32>` de `Sync` olurdu; tek alanı bir `i32`'dir. Standart kütüphane bu yüzden istisnayı açıkça yazar:

```rust,ignore
// library/core/src/cell.rs
impl<T: ?Sized> !Sync for UnsafeCell<T> {}
```

Bu satır kararlı Rust'ta derlenmediği için çalıştırılabilir bir örnek değildir; standart kütüphanenin kaynağından alınmıştır.[^negatif] `!Sync`, "bu tür `Sync` değildir" diye yazılmış bir kayıttır ve otomatik çıkarımı engeller. `Cell` ile `RefCell`'in paylaşılamamasının kökü bu satırdır. Ham işaretçiler, `Rc` ve `MutexGuard` için de benzer satırlar vardır.

Sonucu en çok şu kodda görürsünüz. `RefCell` sayacını atomik tutmaz; iki thread aynı anda `borrow_mut` çağırsa sayaç bozulurdu. Derleyici buna izin vermez:

```rust,compile_fail,E0277
use std::cell::RefCell;
use std::sync::Arc;
use std::thread;

fn main() {
    let ortak = Arc::new(RefCell::new(0));
    let kopya = Arc::clone(&ortak);
    let t = thread::spawn(move || *kopya.borrow_mut() += 1);
    t.join().expect("thread panik yapmadı");
    println!("{}", ortak.borrow());
}
```

Derleyicinin yanıtı:

```text
error[E0277]: `RefCell<i32>` cannot be shared between threads safely
  |
  = help: the trait `Sync` is not implemented for `RefCell<i32>`
  = note: if you want to do aliasing and mutation between multiple threads, use `std::sync::RwLock` instead
  = note: required for `Arc<RefCell<i32>>` to implement `Send`
```

Çözüm `RefCell` yerine `Mutex` ya da `RwLock` kullanmaktır. `RwLock` seçerken içeriğe dikkat edin: Birçok okuyucu aynı anda `&T` alabildiği için `RwLock<T>`'nin `Sync` olması, `T`'nin de `Sync` olmasını gerektirir. `Mutex<Cell<i32>>` paylaşılabilir, `RwLock<Cell<i32>>` paylaşılamaz:

```rust,compile_fail,E0277
use std::cell::Cell;
use std::sync::RwLock;

fn sync_ister<T: Sync>() {}

fn main() {
    sync_ister::<RwLock<Cell<i32>>>();
}
```

Derleyicinin yanıtı:

```text
error[E0277]: `Cell<i32>` cannot be shared between threads safely
  |
  = help: the trait `Sync` is not implemented for `Cell<i32>`
  = note: if you want to do aliasing and mutation between multiple threads, use `std::sync::RwLock` or `std::sync::atomic::AtomicI32` instead
  = note: required for `std::sync::RwLock<Cell<i32>>` to implement `Sync`
```

### Kendi auto trait'iniz: yalnızca nightly'de

Auto trait tanımlamak ve negatif impl yazmak kararsız özelliklerdir:[^kararsiz]

```rust,compile_fail,E0658
auto trait Valid {}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0658]: auto traits are experimental and possibly buggy
```

Nightly derleyicide iki özelliği açarak deneyebilirsiniz. Aşağıdaki örnek kararlı derleyicide derlenmez:

```rust,ignore
#![feature(auto_traits, negative_impls)]

auto trait Valid {}

struct Color<F>
where
    F: Fn(i8) -> i8,
{
    f: F,
}

// F, Valid'i sağlasa bile hiçbir Color<F> sağlamaz.
impl<F> !Valid for Color<F> where F: Fn(i8) -> i8 {}

struct Tablo {
    c: Color<fn(i8) -> i8>,
}

fn valid_ister<T: Valid>() {}

fn main() {
    valid_ister::<i32>();
    valid_ister::<(String, Vec<u8>)>();
    valid_ister::<fn(i8) -> i8>();
    // valid_ister::<Tablo>(); // E0277: içindeki Color, Valid değil
}
```

`i32` için hiçbir şey yazmadığımız hâlde `Valid` geçerlidir; auto trait'i geçersiz kılan bir kayıt yoktur. `Color<F>` için yazılan negatif impl ise onu ve içinde taşıyan `Tablo`'yu dışarıda bırakır. `where` koşulunun yinelenmesinin nedeni, `Color`'ın kendi tanımındaki sınırın impl'de de sağlanması gerektiğidir.

Kararlı Rust'ta bir türü bilerek `Send` ve `Sync` dışında tutmak isterseniz aynı etkiyi bir işaret alanıyla elde edersiniz. `PhantomData<*const ()>` yer kaplamaz, ama ham işaretçi gibi davranır:

```rust,compile_fail,E0277
use std::marker::PhantomData;

struct Jeton {
    _tek_thread: PhantomData<*const ()>,
}

fn send_ister<T: Send>() {}

fn main() {
    send_ister::<Jeton>();
}
```

Derleyicinin yanıtı:

```text
error[E0277]: `*const ()` cannot be sent between threads safely
   |
   = help: within `Jeton`, the trait `Send` is not implemented for `*const ()`
```

## `unsafe impl`: sözü siz verirsiniz

Bazen derleyicinin kararı gereğinden katıdır. `Mutex<T>` bunun en iyi örneğidir: İçinde `UnsafeCell<T>` taşır, dolayısıyla kendiliğinden `Sync` olamaz. Ama erişim kilitle sıraya sokulduğundan veri yarışı çıkamaz. Standart kütüphane bu sözü elle verir:[^mutex]

```rust,ignore
// library/std/src/sync/poison/mutex.rs
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}
```

Bu satır da kütüphane kaynağındandır; kendi crate'inizde yazsanız yetim kuralına takılır. Koşula dikkat edin: `T: Send` yeterlidir, `T: Sync` gerekmez. Kilit aynı anda tek thread'i içeri aldığı için içerik hiç paylaşılmaz, yalnızca thread'den thread'e devredilir. `Arc<Mutex<Cell<i32>>>`'in çalışmasının nedeni budur. Demek ki `UnsafeCell` içermek paylaşımı büsbütün yasaklamaz; yalnızca ek bir disiplin gerektirir.

Kendi türünüz için de aynı yolu kullanabilirsiniz, ama düz bir `impl` kabul edilmez:

```rust,compile_fail,E0200
struct Tut {
    p: *mut u8,
}

impl Send for Tut {}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0200]: the trait `Send` requires an `unsafe impl` declaration
  |
  = note: the trait `Send` enforces invariants that the compiler can't check. Review the trait documentation and make sure this implementation upholds those invariants before adding the `unsafe` keyword
```

`Send` ve `Sync` *unsafe trait*'tir: Yanlış bir söz, güvenli koddan veri yarışına yol açar. Bu yüzden sözün gerekçesi yazılır:

```rust
use std::thread;

struct Tut {
    p: *mut u8,
}

impl Tut {
    fn yeni(deger: u8) -> Tut {
        Tut { p: Box::into_raw(Box::new(deger)) }
    }

    fn oku(&self) -> u8 {
        // SAFETY: `p`, Box::into_raw'dan gelir ve drop'a kadar geçerlidir.
        unsafe { *self.p }
    }
}

impl Drop for Tut {
    fn drop(&mut self) {
        // SAFETY: `p`, Box::into_raw'dan gelir ve yalnızca burada geri alınır.
        drop(unsafe { Box::from_raw(self.p) });
    }
}

// SAFETY: `Tut`, gösterdiği baytın tek sahibidir ve işaretçiyi dışarı vermez;
// sahiplik başka bir thread'e geçince eski thread'de erişim kalmaz.
unsafe impl Send for Tut {}

fn main() {
    let t = Tut::yeni(7);
    let is = thread::spawn(move || t.oku());
    println!("{}", is.join().expect("thread panik yapmadı"));
}
```

Çıktı:

```text
7
```

## Üyelik gizli bir sözdür

`Send` ve `Sync` imzalarda görünmez, ama kütüphanenizin sözünün parçasıdır. Herkese açık bir türün özel bir alanını `Rc`'ye çevirirseniz türün kendisi sessizce `Send` olmaktan çıkar. Onu thread'e taşıyan kullanıcıların kodu artık derlenmez; bu kırıcı bir değişikliktir.[^rfr]

Bunu erkenden yakalamanın yolu, üyeliği derlenen bir teste yazmaktır. *Rust for Rustaceans* bu fonksiyona `is_normal` adını verir:[^rfr]

```rust
struct Rapor {
    baslik: String,
    satirlar: Vec<String>,
}

fn is_normal<T: Send + Sync + Unpin>() {}

#[test]
fn rapor_olagan_bir_turdur() {
    is_normal::<Rapor>();
}
# fn main() {
#     is_normal::<Rapor>();
#     let r = Rapor { baslik: String::new(), satirlar: Vec::new() };
#     let _ = (r.baslik, r.satirlar);
# }
```

Testin gövdesi çalışma zamanında bir iş yapmaz; değeri derlenmesindedir. Alanlardan biri `Rc<str>` olduğu gün test derlenmez:

```rust,compile_fail,E0277
use std::rc::Rc;

struct Rapor {
    baslik: Rc<str>,
    satirlar: Vec<String>,
}

fn is_normal<T: Send + Sync + Unpin>() {}

fn main() {
    is_normal::<Rapor>();
}
```

Derleyicinin yanıtı:

```text
error[E0277]: `Rc<str>` cannot be sent between threads safely
   |
   = help: within `Rapor`, the trait `Send` is not implemented for `Rc<str>`
```

### `impl Trait` üyeliği sızdırır, `dyn` sızdırmaz

Dönüş türü `impl Fn() -> i32` olan bir fonksiyon, somut türü gizler; ama auto trait üyeliği bu perdeden geçer. Derleyici arkadaki gerçek türe bakar.[^sizma] `Rc` yakalayan bir closure bu yüzden `Send` sayılmaz, oysa imzada hiçbir şey yazmaz:

```rust,compile_fail,E0277
use std::rc::Rc;

fn sayac() -> impl Fn() -> i32 {
    let n = Rc::new(1);
    move || *n
}

fn send_ister<T: Send>(_: T) {}

fn main() {
    send_ister(sayac());
}
```

Derleyicinin yanıtı:

```text
error[E0277]: `Rc<i32>` cannot be sent between threads safely
   |
   = help: within `impl Fn() -> i32`, the trait `Send` is not implemented for `Rc<i32>`
```

Trait nesnelerinde ise tür gerçekten silinir ve derleyici arkasını göremez. `Box<dyn Fn()>`, içinde `Send` bir closure taşısa bile `Send` değildir. Gerekiyorsa sınırı türe yazarsınız: `Box<dyn Fn() + Send>`.

## Hata kodu başvurusu

| Kod | Ne zaman görülür? | Ne yapmalı? |
| --- | --- | --- |
| `E0277` | `Send` ya da `Sync` istenen yerde üye olmayan tür verildiğinde | `Rc` yerine `Arc`, `RefCell` yerine `Mutex` kullanın; `dyn` türlerine sınırı yazın. |
| `E0200` | `impl Send`/`impl Sync`, `unsafe` olmadan yazıldığında | Gerekçesi varsa `// SAFETY:` ile `unsafe impl` yazın. |
| `E0594` | `&` ardındaki değere doğrudan atama yapıldığında | `&mut` alın ya da iç değiştirilebilirlik türü kullanın. |
| `E0658` | Kararlıda `auto trait` ya da negatif impl yazıldığında | `PhantomData<*const ()>` gibi bir işaret alanı kullanın. |

## Perde arkası: Derleyici ne yapıyor?

> [!NOTE]
> Bu başlık ilk okuyuşta atlanabilir; yukarıdaki kuralları anlamış olmanız yeterlidir.

**Tanımlar.** `Send` ve `Sync`, `core` içindeki `marker.rs` dosyasında `pub unsafe auto trait` olarak tanımlanır; ham işaretçilerin negatif impl'leri de oradadır.[^marker] Üyelik kararında derleyicinin bu iki trait'e özel bir mantığı yoktur; ne yapılacağını `auto` sözcüğü söyler.

**Üyelik kararı.** Trait çözücü, bir tür için açık impl bulamazsa `assemble_candidates_from_auto_impls` ile otomatik adayı ekler. Ardından `constituent_types_for_auto_trait` türün parçalarını çıkarır ve aynı soru her parça için yeniden sorulur.[^aday] Hata notlarındaki `required because it appears within the type` zinciri bu özyinelemenin izidir. Opak `impl Trait` türünde parça, arkadaki gerçek türdür; sızıntının nedeni budur.

**`UnsafeCell`'in izi.** Derleyici türleri, içinde dolaylama olmadan `UnsafeCell` bulunup bulunmadığına göre ayırır; bulunmayanlar iç kullanımdaki `Freeze` auto trait'ini taşır. Fonksiyon imzaları LLVM'e aktarılırken `&T` parametresi, `T` `Freeze` ise `noalias` ve `readonly` öznitelikleriyle işaretlenir; değilse işaretlenmez.[^freeze] "Müze etiketi" makine koduna böyle yansır.

**`unsafe impl` denetimi.** Unsafe bir trait'in `unsafe` olmadan uygulanması, tutarlılık denetimi sırasında `coherence/unsafety.rs` içinde yakalanır ve `E0200` üretir.[^unsafety]

## Özet

- `UnsafeCell`, `&` üzerinden yazmanın tek yasal yoludur; `&mut`'un tekilliğini ve veri yarışı yasağını değiştirmez.
- `Send` sahipliğin, `Sync` paylaşımın thread'ler arasında güvenli olduğunu söyler; `T: Sync`, `&T: Send` demektir.
- İkisi de auto trait'tir: Bütün alanlar uygunsa tür uygundur, tek bir uygunsuz alan bütün türü dışarıda bırakır.
- `UnsafeCell`, ham işaretçiler, `Rc` ve `MutexGuard` için negatif impl'ler vardır; `Mutex` ise `unsafe impl` ile `Sync` olur.
- Üyelik kütüphanenizin gizli sözüdür; `is_normal` gibi derlenen bir testle koruyun.

Sıradaki bölüm [26. Düşürme Denetimi (Drop Check)](26-dropck.md), bir değer yok edilirken içindeki referansların hâlâ geçerli olmasını derleyicinin nasıl sağladığını anlatıyor.

## Kaynaklar

[^unsafecell]: std, [`UnsafeCell`](https://doc.rust-lang.org/std/cell/struct.UnsafeCell.html); The Rust Reference, [`UnsafeCell<T>`](https://doc.rust-lang.org/reference/special-types-and-traits.html#unsafecellt); hata dizini, [`E0594`](https://doc.rust-lang.org/error_codes/E0594.html).
[^kurallar]: std, [`UnsafeCell`, Aliasing rules](https://doc.rust-lang.org/std/cell/struct.UnsafeCell.html#aliasing-rules).
[^icdeg]: The Rust Programming Language, [`RefCell<T>` and the Interior Mutability Pattern](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html) ve [Shared-State Concurrency](https://doc.rust-lang.org/book/ch16-03-shared-state.html); std, [`Cell`](https://doc.rust-lang.org/std/cell/struct.Cell.html); Jon Gjengset, *Rust for Rustaceans*, 9. bölüm ("Unsafe Code").
[^tanim]: std, [`Send`](https://doc.rust-lang.org/std/marker/trait.Send.html) ve [`Sync`](https://doc.rust-lang.org/std/marker/trait.Sync.html); The Rustonomicon, [Send and Sync](https://doc.rust-lang.org/nomicon/send-and-sync.html).
[^auto]: The Rust Reference, [Auto traits](https://doc.rust-lang.org/reference/special-types-and-traits.html#auto-traits); The Rust Programming Language, [Extensible Concurrency with `Send` and `Sync`](https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html); hata dizini, [`E0277`](https://doc.rust-lang.org/error_codes/E0277.html).
[^negatif]: rustc 1.99.0 kaynak kodu, [`cell.rs`, `!Sync for UnsafeCell`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/cell.rs#L2316).
[^kararsiz]: The Rust Unstable Book, [`auto_traits`](https://doc.rust-lang.org/unstable-book/language-features/auto-traits.html) ve [`negative_impls`](https://doc.rust-lang.org/unstable-book/language-features/negative-impls.html); rustc 1.99.0 kaynak kodu, [`unstable.rs`, `auto_traits`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_feature/src/unstable.rs#L315).
[^mutex]: std, [`Mutex`](https://doc.rust-lang.org/std/sync/struct.Mutex.html); rustc 1.99.0 kaynak kodu, [`mutex.rs`, `Sync for Mutex`](https://github.com/rust-lang/rust/blob/1.99.0/library/std/src/sync/poison/mutex.rs#L257); hata dizini, [`E0200`](https://doc.rust-lang.org/error_codes/E0200.html).
[^rfr]: Jon Gjengset, *Rust for Rustaceans*, 3. bölüm ("Designing Interfaces").
[^sizma]: The Rust Reference, [Auto traits](https://doc.rust-lang.org/reference/special-types-and-traits.html#auto-traits) ve [Impl trait](https://doc.rust-lang.org/reference/types/impl-trait.html).
[^marker]: rustc 1.99.0 kaynak kodu, [`marker.rs`, `auto trait Send`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/marker.rs#L92) ve [`auto trait Sync`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/marker.rs#L657).
[^aday]: rustc 1.99.0 kaynak kodu, [`candidate_assembly.rs`, `assemble_candidates_from_auto_impls`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_trait_selection/src/traits/select/candidate_assembly.rs#L742) ve [`select/mod.rs`, `constituent_types_for_auto_trait`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_trait_selection/src/traits/select/mod.rs#L2305).
[^freeze]: rustc 1.99.0 kaynak kodu, [`marker.rs`, `Freeze`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/marker.rs#L906) ve [`rustc_ty_utils/src/abi.rs`, `SharedRef`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_ty_utils/src/abi.rs#L367).
[^unsafety]: rustc 1.99.0 kaynak kodu, [`unsafety.rs`, `E0200`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_analysis/src/coherence/unsafety.rs#L60).
