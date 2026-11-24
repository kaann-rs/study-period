# 14. Akıllı İşaretçiler

*İşaretçi* (pointer), verinin kendisini değil bellekte nerede durduğunu tutan bir değerdir. Rust'ta en sık karşılaştığınız işaretçi referanstır: Veriyi ödünç alır, başka bir yeteneği yoktur. *Akıllı işaretçi* (smart pointer) ise işaretçi gibi davranan, yanında ek bilgi (metadata) ve ek yetenek taşıyan bir veri yapısıdır. Çoğu zaman gösterdiği verinin sahibidir ve genellikle iki trait uygular: `Deref` sayesinde referans gibi kullanılır, `Drop` sayesinde kapsam dışına çıkarken kendi temizliğini yapar.[^giris]

[Ownership bölümünde](../basics/03-ownership.md) her değerin tek bir sahibi olduğunu ve ödünç kurallarının derleme zamanında denetlendiğini gördük. Programların büyük kısmına bu düzen yeter; bazı durumlarda ise dar gelir: Kendini içeren bir türün boyutu hesaplanamaz, bir verinin kaç sahibi olacağı ancak çalışma zamanında belli olur, dışarıdan değişmez görünen bir değerin içinde küçük bir kaydın güncellenmesi gerekir. Akıllı işaretçiler bu durumlar için hazırlanmış *kaçış kapılarıdır*. Hiçbiri güvenlikten vazgeçmez; her biri güvenliği başka bir yolla korur ve karşılığında küçük bir bedel alır.

Bu bölümde sırayla şu türlere ve trait'lere bakacağız:

- `Box<T>`: Değeri heap'e koyar; tek sahibi vardır.
- `Deref`: Kendi türünüze `*` işaretini ve referans gibi davranmayı öğretir.
- `Drop`: Değer düşürülürken çalışacak kodu belirler.
- `Rc<T>`: Bir değere birden çok sahip tanır; sahipleri sayar.
- `RefCell<T>`: Ödünç kurallarını derleme zamanında değil, çalışma zamanında denetler.
- `Weak<T>`: Sahip olmadan "nerede durduğunu bilen" işaretçidir.

> [!NOTE]
> Bu bölümün kodu `learn/smart-pointers` dizinindedir. Depo kökünden `cargo run -p smart-pointers` komutuyla çalıştırabilirsiniz.

## Günlük hayattan bir benzetme

Bir okul düşünün. Okulda eşyaların konduğu farklı kutular var ve her kutunun kuralı ayrı:

- Bir kutu yalnızca sizindir: Büyük maketinizi okulun deposuna koyar, sıranızda adresinin yazdığı küçük bir kâğıt tutarsınız. Bu `Box`'tır.
- Bir kutu paylaşılır: Sınıfın tek topunu birkaç takım birlikte kullanır; tahtada o an kaç takımın topu kullandığı yazar. Bu `Rc`'dir.
- Bir kutunun içi, dışarıdan bakınca değişmez göründüğü hâlde düzeltilebilir; kapısındaki nöbetçi aynı anda iki kişinin yazmasına izin vermez. Bu `RefCell`'dir.
- Son kutu ise bir kutu bile sayılmaz: Yalnızca eşyanın nerede durduğunu yazan bir nottur. Bu da `Weak`'tir.

İki defter birbirine "beni oku" diye not bırakırsa ikisi de çöpe atılamaz, çünkü her biri ötekini gerekli gösterir. Çaresi, notlardan birini yalnızca "nerede olduğunu" hatırlatan bir nota çevirmektir. Bölümün sonunda bu düğümü `Weak` ile çözeceğiz.

<details>
<summary>Bu bölümde geçen terimler</summary>

- **işaretçi (pointer):** Verinin kendisini değil, nerede durduğunu tutan değer; bir adres kâğıdı.
- **akıllı işaretçi (smart pointer):** İşaretçi gibi davranan, ek bilgi ve yetenek taşıyan tür. Sayaç tutabilir, çıkarken temizlik yapabilir.
- **metadata (üst veri):** Verinin yanında tutulan ek bilgi; örneğin "kaç sahibi var" sayısı.
- **stack ve heap:** Belleğin iki bölgesi. Stack küçük ve hızlı bir raf, heap büyük bir depo gibidir.
- **`Box<T>`:** Değeri heap'e koyan, kendisi yalnızca adresini tutan kutu.
- **özyineli tür (recursive type):** İçinde kendi türünden bir parça taşıyan tür.
- **cons list:** "Bir değer ve listenin geri kalanı" biçiminde kurulan zincir liste.
- **`Deref` ve `*`:** Adresten verinin kendisine gitmek.
- **deref coercion:** Derleyicinin bu "adrese git" adımını sizin yerinize atması.
- **`Drop` ve RAII:** Bir değer düşürülürken kendiliğinden çalışan temizlik. Kaynağın ömrünü bir değerin ömrüne bağlayan düzenin adı RAII'dir: İşi biten, arkasını kendi toplar.
- **`Rc<T>`:** Bir veriyi birden çok sahibin paylaşmasını sağlayan, referans sayan (reference counted) işaretçi.
- **güçlü sayaç (strong count):** Bir veriyi o anda kaç `Rc`'nin sahiplendiğini gösteren sayı.
- **`RefCell<T>`:** Ödünç kuralını program çalışırken denetleyen hücre.
- **iç değiştirilebilirlik (interior mutability):** Paylaşımlı bir referans üzerinden içerideki değeri değiştirebilmek.
- **`borrow()` ve `borrow_mut()`:** `RefCell`'den okuma ve yazma izni istemek.
- **aliasing XOR mutability:** "Aynı anda ya çok okuyan ya tek yazan" kuralının adı.
- **panik (panic):** Programın "bu hatayla devam edemem" deyip işi bırakması.
- **`Weak<T>`:** Verinin yerini bilen ama onu canlı tutmayan işaretçi.
- **referans döngüsü (reference cycle):** İki verinin birbirini sahiplenmesi ve bu yüzden hiçbirinin düşürülememesi.
- **bellek sızıntısı (memory leak):** İşi biten verinin geri verilmeyip yer kaplamayı sürdürmesi.
- **thread:** Programın içinde aynı anda yürüyen ayrı bir iş kolu.
- **enum, `match`, trait:** Sırasıyla "şunlardan biri" türü, hangisi olduğuna bakıp ona göre davranan ifade ve bir türün yapabildiği işlerin listesi.

</details>

## `Box<T>`: Değeri heap'te tutmak

Büyük bir maket yaptınız ve sıranıza sığmıyor. Onu okulun deposuna koyar, sıranıza da "3. raf, 2. göz" yazan küçük bir kâğıt bırakırsınız. Maket ne kadar büyük olursa olsun kâğıdın boyu değişmez. `Box::new(x)` de böyledir: `x` heap'e taşınır, elinizde sabit boyutlu bir adres kalır. Kâğıdın sahibi gidince depodaki maket de kaldırılır; yani `Box` düşürülürken hem değeri düşürür hem de heap'teki yeri geri verir.

```rust
fn main() {
    let b = Box::new(5); // 5 heap'te, b stack'te bir adres

    println!("b = {b}, *b + 1 = {}", *b + 1);
} // b düşürülür, heap'teki yer geri verilir
```

Çıktı:

```text
b = 5, *b + 1 = 6
```

`*b`, "adrese git ve oradaki değeri ver" demektir. Veriyi heap'te tutmanın kendi bedeli (yer ayırmak, geri vermek, adresi izlemek) dışında `Box`'ın çalışma zamanı yükü yoktur.[^box] Başlıca üç kullanım yeri vardır: boyutu derleme zamanında bilinemeyen türler, kopyalanmadan el değiştirmesi istenen büyük veriler ve [trait nesneleri](17-oop.md). Burada birincisine bakacağız.

### Özyineli türler

*Cons list*, Lisp dillerinden gelen basit bir liste biçimidir: Her düğüm bir değer ile listenin geri kalanını tutar, son düğüm ise listenin bittiğini söyler. Bunu bir enum ile doğrudan yazmayı deneyelim:

```rust,compile_fail,E0072
enum Kotu {
    Halka(i32, Kotu),
    Son,
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0072]: recursive type `Kotu` has infinite size
  |
1 | enum Kotu {
  | ^^^^^^^^^
2 |     Halka(i32, Kotu),
  |                ---- recursive without indirection
  |
help: insert some indirection (e.g., a `Box`, `Rc`, or `&`) to break the cycle
```

Derleyici bir türün boyutunu hesaplarken alanlarının boyutunu toplar. `Kotu`'nun boyutu bir `i32` ile bir `Kotu`'nun toplamıdır; o `Kotu`'nun boyutu yine bir `i32` ile bir `Kotu`'nun toplamıdır ve bu hesap hiç bitmez.[^ozyineli] Yardım satırı çareyi söyler: Araya bir *dolaylılık* (indirection), yani bir işaretçi koyun. İşaretçinin boyutu gösterdiği şeyin boyutuna bağlı olmadığı için döngü kırılır:

```rust
#[derive(Debug)]
enum Liste {
    Dugum(i32, Box<Liste>),
    Son,
}
use Liste::{Dugum, Son};

fn topla(l: &Liste) -> i32 {
    match l {
        Dugum(deger, sonraki) => deger + topla(sonraki),
        Son => 0,
    }
}

fn main() {
    let liste = Dugum(1, Box::new(Dugum(2, Box::new(Son))));

    println!("{liste:?}");
    println!("toplam = {}", topla(&liste));
    println!("bir Box {} bayt", size_of::<Box<Liste>>());
    println!("bir Liste {} bayt", size_of::<Liste>());
}
```

Çıktı:

```text
Dugum(1, Dugum(2, Son))
toplam = 3
bir Box 8 bayt
bir Liste 16 bayt
```

Liste ne kadar uzarsa uzasın bir `Liste` değeri 16 bayttır (64 bitlik makinede): Bir `i32`, bir adres ve hizalama boşluğu. Listenin geri kalanı heap'teki ayrı bloklarda durur.

> [!NOTE]
> `Liste`'yi yalnızca `{:?}` ile yazdırıp alanlarını hiçbir yerde okumasaydık derleyici `dead_code` uyarısı verirdi. Uyarının notu nedenini açıkça söyler: Türetilmiş `Debug`, kullanılmayan kod çözümlemesinde bilerek hesaba katılmaz.[^dead] `topla` alanları `match` ile gerçekten okuduğu için uyarı çıkmaz.

## `Deref`: Adresten değere gitmek

Elinizdeki adres kâğıdına bakarak maketi boyayamazsınız; önce depoya gidip maketin yanına varmanız gerekir. `*` işareti bu yürüyüştür. `Deref` trait'i, kendi yaptığınız kutuya bu yürüyüşü öğretmenin yoludur.

`Box`'a benzeyen küçük bir tür yazalım; heap kullanmasın, yalnızca sarmalasın. `Deref` uygulamadan `*` kullanırsak derleyici türün nasıl açılacağını bilemez:

```rust,compile_fail,E0614
struct Kutum<T>(T);

fn main() {
    let x = Kutum(5);
    println!("{}", *x + 1);
}
```

Derleyicinin yanıtı:

```text
error[E0614]: type `Kutum<{integer}>` cannot be dereferenced
  |
5 |     println!("{}", *x + 1);
  |                    ^^ can't be dereferenced
```

`Deref` tek bir metot ister: `deref`, `self`'i ödünç alır ve içerideki değere bir referans döndürür.[^deref-impl]

```rust
use std::ops::Deref;

struct Kutum<T>(T);

impl<T> Kutum<T> {
    fn yeni(x: T) -> Kutum<T> {
        Kutum(x)
    }
}

impl<T> Deref for Kutum<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

fn main() {
    let x = Kutum::yeni(5);

    println!("*x = {}", *x);
    println!("*x + 1 = {}", *x + 1);
    println!("*(x.deref()) = {}", *(x.deref()));
}
```

Çıktı:

```text
*x = 5
*x + 1 = 6
*(x.deref()) = 5
```

Derleyici `*x` ifadesini `*(x.deref())` olarak çözer. `deref`'in değerin kendisini değil referansını döndürmesi sahiplik yüzündendir: Değeri döndürseydi `*x` yazmak, değeri `Kutum`'un içinden taşırdı.[^deref-impl]

### Deref coercion

`Deref` uygulayan bir türün referansı, gerektiğinde hedef türün referansına kendiliğinden dönüşür. Buna *deref coercion* denir ve birden çok adım art arda işleyebilir:[^coercion]

```rust
# use std::ops::Deref;
# struct Kutum<T>(T);
# impl<T> Kutum<T> { fn yeni(x: T) -> Kutum<T> { Kutum(x) } }
# impl<T> Deref for Kutum<T> { type Target = T; fn deref(&self) -> &T { &self.0 } }
fn selamla(ad: &str) {
    println!("Selam, {ad}!");
}

fn main() {
    let isim = Kutum::yeni(String::from("Kaan"));
    selamla(&isim);
    selamla(&(*isim)[..]); // coercion olmasaydı yazılması gereken
    println!("uzunluk = {}", isim.len()); // String'in metodu

    let k = Box::new(String::from("Ada"));
    selamla(&k);
    println!("uzunluk = {}", k.len());
}
```

Çıktı:

```text
Selam, Kaan!
Selam, Kaan!
uzunluk = 4
Selam, Ada!
uzunluk = 3
```

`selamla` bir `&str` ister, biz `&Kutum<String>` verdik. Derleyici önce `Kutum`'un `Deref`'iyle `&String`'e, ardından `String`'in `Deref`'iyle `&str`'ye ulaşır. Aynı adımlar metot çağrısında da işler: `isim.len()` çağrısında `isim` bir kutudur, ama `len` içerideki `String`'in metodudur. Bu kolaylık olmasaydı her kutu için açma adımlarını ikinci `selamla` satırındaki gibi elle yazmanız gerekirdi.

Kaç adım gerektiği derleme zamanında belirlenir; dönüşüme karar vermenin çalışma zamanında bir bedeli yoktur.[^coercion] Değiştirilebilir referanslar için aynı işi `DerefMut` trait'i yapar. `&mut T`'den `&U`'ya dönüşüm serbesttir, tersi yasaktır: Paylaşımlı bir referanstan değiştirilebilir referans üretmek ödünç kurallarını bozardı.[^derefmut] Metot çözümlemenin tam kurallarını [24. bölümde](../advanced/24-deref-methods.md) göreceğiz.

## `Drop`: Düşürülürken çalışan kod

Resim dersinden çıkarken fırçanızı yıkayıp yerine koyarsınız. Rust her değer için toplama işini kendiliğinden yapar; `Drop` ile "çıkarken şunu da yap" diye kendi işinizi eklersiniz. Toplama sırası çıkarma sırasının tersidir: En son çıkardığınız malzeme ilk önce kaldırılır.

```rust
struct Kaynak {
    ad: String,
}

impl Drop for Kaynak {
    fn drop(&mut self) {
        println!("Drop: '{}' serbest bırakıldı", self.ad);
    }
}

fn main() {
    let _a = Kaynak { ad: String::from("A") };
    let b = Kaynak { ad: String::from("B") };
    let _c = Kaynak { ad: String::from("C") };

    drop(b); // B, kapsamın sonu beklenmeden düşürülür
    println!("main bitiyor");
}
```

Çıktı:

```text
Drop: 'B' serbest bırakıldı
main bitiyor
Drop: 'C' serbest bırakıldı
Drop: 'A' serbest bırakıldı
```

Gerçek programlarda `drop` metodu bir dosyayı kapatır, bir soketi bırakır ya da bir kilidi açar; burada yalnızca iz basıyoruz. `C`, `A`'dan sonra tanımlandığı için ondan önce düşürülür.[^sira] Temizlik size bırakılsaydı unutabilirdiniz (açık kalan bir dosya gibi) ya da aynı şeyi iki kez toplamaya kalkardınız. Kaynağın ömrünü bir değerin ömrüne bağlayan bu düzene RAII denir.

İkinci tehlike yüzünden metodu elle çağırmanıza izin verilmez:

```rust,compile_fail,E0040
# struct Kaynak { ad: String }
# impl Drop for Kaynak {
#     fn drop(&mut self) { println!("Drop: {}", self.ad); }
# }
fn main() {
    let b = Kaynak { ad: String::from("B") };
    b.drop();
}
```

Derleyicinin yanıtı:

```text
error[E0040]: explicit use of destructor method
   |
11 |     b.drop();
   |       ^^^^ explicit destructor calls not allowed
   |
help: consider using `drop` function
```

Bu çağrıya izin verilseydi `b` geçersizleşmezdi; kapsamın sonunda yıkıcı bir kez daha çalışır ve aynı kaynak iki kez geri verilirdi. Standart kütüphanedeki `std::mem::drop` fonksiyonu ise değeri *taşıyarak* alır: Değer fonksiyonun içinde düşürülür, çağırandaki değişken de artık kullanılamaz.[^drop] Fonksiyonun gövdesi boştur; bütün işi parametresinin kapsamı yapar.

> [!NOTE]
> Rust 1.100.0 ile gelen `std::mem::DropGuard`, yeni bir tür tanımlamadan "düşürülürken şu closure'ı çalıştır" demenizi sağlar.[^dropguard] Bu kitabın örnekleri 1.99.0 ile derlendiği için burada kullanılmıyor.

## `Rc<T>`: Birden çok sahip

Sınıfın tek bir topu var ve onu birkaç takım birlikte kullanıyor. Tahtada topu kullanan takım sayısı yazar: Katılan takım sayıyı bir artırır, ayrılan bir azaltır; sayı sıfır olunca top dolaba kaldırılır. Sayaç olmasaydı topun ne zaman kaldırılacağını kimse bilemezdi. Erken kaldırılsa oynayan takım topsuz kalır, hiç kaldırılmasa top sonsuza dek ortada dururdu.

`Rc<T>` (*reference counted*, referans sayılan) tam bu işi yapar. Önce neden gerektiğini görelim: Bir listeyi iki listenin ortak kuyruğu yapmak istiyoruz. `Box` ile bu mümkün değildir, çünkü `Box`'ın tek sahibi olur:

```rust,compile_fail,E0382
# #![allow(dead_code)]
enum Liste {
    Dugum(i32, Box<Liste>),
    Son,
}
use Liste::{Dugum, Son};

fn main() {
    let a = Dugum(5, Box::new(Dugum(10, Box::new(Son))));
    let b = Dugum(3, Box::new(a));
    let c = Dugum(4, Box::new(a));
}
```

Derleyicinin yanıtı:

```text
error[E0382]: use of moved value: `a`
   |
 8 |     let a = Dugum(5, Box::new(Dugum(10, Box::new(Son))));
   |         - move occurs because `a` has type `Liste`, which does not implement the `Copy` trait
 9 |     let b = Dugum(3, Box::new(a));
   |                               - value moved here
10 |     let c = Dugum(4, Box::new(a));
   |                               ^ value used here after move
```

Kuyruğu `Rc` ile tutunca `a`'yı iki liste birden sahiplenebilir:

```rust
use std::rc::Rc;

enum RcListe {
    Dugum(i32, Rc<RcListe>),
    Son,
}
use RcListe::{Dugum, Son};

fn rc_topla(l: &RcListe) -> i32 {
    match l {
        Dugum(deger, sonraki) => deger + rc_topla(sonraki),
        Son => 0,
    }
}

fn main() {
    let a = Rc::new(Dugum(5, Rc::new(Dugum(10, Rc::new(Son)))));
    println!("a oluştu, sayaç = {}", Rc::strong_count(&a));

    let b = Dugum(3, Rc::clone(&a));
    println!("b paylaştı, sayaç = {}", Rc::strong_count(&a));
    println!("b toplamı = {}", rc_topla(&b));

    {
        let c = Dugum(4, Rc::clone(&a));
        println!("c paylaştı, sayaç = {}", Rc::strong_count(&a));
        println!("c toplamı = {}", rc_topla(&c));
    } // c düşürülür

    println!("c düştü, sayaç = {}", Rc::strong_count(&a));
}
```

Çıktı:

```text
a oluştu, sayaç = 1
b paylaştı, sayaç = 2
b toplamı = 18
c paylaştı, sayaç = 3
c toplamı = 19
c düştü, sayaç = 2
```

`Rc::clone` yeni bir top yapmaz: Veriyi kopyalamaz, yalnızca sayacı bir artırır. Bir `Rc` düşürüldüğünde sayaç bir azalır; sıfıra inince içindeki değer düşürülür.[^rc] `a.clone()` yazımı da aynı işi yapar, ama `Rc::clone(&a)` yazımı okuyana "burada derin kopya yok" der; yerleşik yazım budur.[^rc-clone]

`Rc` üzerinden veriye yalnızca bakılabilir:

```rust,compile_fail,E0596
use std::rc::Rc;

fn main() {
    let top = Rc::new(String::from("top"));
    top.push('!');
}
```

Derleyicinin yanıtı:

```text
error[E0596]: cannot borrow data in an `Rc` as mutable
  |
4 |     top.push('!');
  |     ^^^ cannot borrow as mutable
  |
  = help: trait `DerefMut` is required to modify through a dereference, but it is not implemented for `Rc<String>`
```

Nedeni *aliasing XOR mutability* kuralıdır: Aynı veriye başka sahipler de bakıyor olabilir. Paylaşılan veriyi herkesin göreceği biçimde değiştirmenin yolu, bir sonraki başlıktaki `RefCell`'dir.

> [!WARNING]
> `Rc` yalnızca tek thread içindir. Sayacı atomik değildir; iki thread aynı anda artırsa sayı bozulabilirdi. Derleyici `Rc`'nin başka bir thread'e gönderilmesini reddeder. Aynı işi birden çok thread'de yapan tür `Arc<T>`'dir; onu [15. bölümde](15-concurrency.md) göreceğiz.[^rc-thread]

## `RefCell<T>`: Ödünç kuralını çalışma zamanına taşımak

Sınıf defterine herkes aynı anda bakabilir, ama yazarken tek kişi olmalıdır. Bu kuralı normalde derleyici, daha ders başlamadan denetler. `RefCell`'de ise kapıda bir nöbetçi durur ve kuralı ders sırasında denetler: `borrow()` "bakacağım", `borrow_mut()` "yazacağım" demektir. Biri yazarken ikinci kişi de yazmak isterse nöbetçi dersi durdurur, yani program panikler. Kural kalkmaz; yalnızca denetimin **zamanı** değişir.[^refcell-kural]

```rust
use std::cell::RefCell;

fn main() {
    let defter = RefCell::new(0); // mut yok

    *defter.borrow_mut() += 1; // yazdı, deyim bitince izni geri verdi
    println!("defter = {}", defter.borrow());
}
```

Çıktı:

```text
defter = 1
```

`defter` değiştirilebilir olarak tanımlanmadı, yine de içindeki değer değişti. Paylaşımlı bir referans üzerinden değişiklik yapabilmeye *iç değiştirilebilirlik* (interior mutability) denir.[^ic] `borrow()` bir `Ref<T>`, `borrow_mut()` bir `RefMut<T>` döndürür. Asıl akıllı işaretçiler bunlardır: `Deref` uygularlar, düşürüldüklerinde de aldıkları izni geri verirler.

Kuralı çiğneyen program derlenir, ama çalışırken durur:

```rust,should_panic
use std::cell::RefCell;

fn main() {
    let defter = RefCell::new(0);

    let _yazan1 = defter.borrow_mut(); // kalem _yazan1'de
    let _yazan2 = defter.borrow_mut(); // panik
}
```

Çıktı:

```text
RefCell already borrowed
```

Panik mesajı budur; öncesinde paniğin gerçekleştiği dosya ve satır da yazılır.[^panik] Biri yazarken `borrow()` istenirse mesaj `RefCell already mutably borrowed` olur.

Peki neden böyle bir araç var? Derleyici temkinlidir: Doğruluğunu kanıtlayamadığı bazı programları, aslında doğru olsalar bile geri çevirir. `RefCell` o programları yazmanıza izin verir. İki bedeli vardır: Hata derleme zamanında değil çalışma zamanında ortaya çıkar ve her ödünçte küçük bir sayaç denetimi yapılır. Hata yapıldığında da sonuç bellek bozulması değil, denetimli bir paniktir.[^refcell-kural]

### `Rc<RefCell<T>>`: Çok sahipli ve değiştirilebilir veri

`Rc` paylaşımı, `RefCell` değişikliği sağlar; ikisi birleşince birden çok sahibin değiştirebildiği bir değer elde edilir:[^rc-refcell]

```rust
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let ortak = Rc::new(RefCell::new(0_i32));
    let a = Rc::clone(&ortak);
    let b = Rc::clone(&ortak);

    *a.borrow_mut() += 10; // RefMut bu deyimin sonunda bırakılır
    *b.borrow_mut() += 5;

    println!("ortak sayaç = {}", ortak.borrow());
}
```

Çıktı:

```text
ortak sayaç = 15
```

İki `borrow_mut` çağrısı çakışmaz, çünkü her birinin verdiği izin kendi deyiminin sonunda biter.

## `Weak<T>`: Sahip olmadan bilmek

`Rc` "sayaç sıfıra inince düşür" der. İki değer birbirini `Rc` ile tutarsa sayaçlar hiç sıfıra inmez: Dışarıdaki bütün sahipler gitse bile her biri ötekini canlı tutar. Buna *referans döngüsü* (reference cycle) denir ve sonucu bellek sızıntısıdır. Derleyici bunu yakalamaz; Rust'ta sızıntı bellek güvenliğini bozan bir hata, yani tanımsız davranış sayılmaz.[^dongu]

Döngüyü kırmanın yolu, taraflardan birinin sahip olmaktan vazgeçmesidir. `Rc::downgrade` bir `Rc`'den `Weak<T>` üretir. `Weak` güçlü sayacı artırmaz; gösterdiği değeri kullanmadan önce `upgrade()` ile "hâlâ duruyor mu?" diye sormanız gerekir. Yanıt bir `Option<Rc<T>>`'dir:[^weak]

```rust
use std::rc::Rc;

fn main() {
    let sinif = Rc::new(String::from("4-A"));
    let bilen = Rc::downgrade(&sinif); // yerini biliyor, sahibi değil

    println!("güçlü = {}", Rc::strong_count(&sinif));
    println!("zayıf = {}", Rc::weak_count(&sinif));
    println!("{:?}", bilen.upgrade());

    drop(sinif); // tek sahip gitti
    println!("{:?}", bilen.upgrade());
}
```

Çıktı:

```text
güçlü = 1
zayıf = 1
Some("4-A")
None
```

Bir öğrenci hangi sınıfta olduğunu bilir, ama o biliyor diye sınıf açık tutulmaz. Veri düşürülmüşse `upgrade` `None` döndürür; geçersiz belleğe bakmanız mümkün değildir. Ağaçlarda çocuktan ebeveyne giden kenarı `Weak` yapmak bu yüzden yerleşik çözümdür: Ebeveyn çocuğun sahibidir, çocuk ebeveynini yalnızca bilir. Döngünün nasıl sızdırdığını ve `Weak` ile nasıl onarıldığını [sonraki sayfada](14-smart-pointers-deep-dive.md) ölçerek göreceğiz.

## Hata kodu başvurusu

| Kod | Ne zaman görülür? | Ne yapmalı? |
| --- | --- | --- |
| `E0072` | Bir tür kendini araya işaretçi koymadan içerdiğinde | Özyineli alanı `Box`, `Rc` ya da referans arkasına alın. |
| `E0614` | `Deref` uygulamayan bir türe `*` uygulandığında | `Deref` uygulayın ya da içteki alana doğrudan ulaşın. |
| `E0040` | `Drop::drop` elle çağrıldığında | `std::mem::drop(x)` kullanın. |
| `E0382` | Taşınmış bir değer yeniden kullanıldığında | Birden çok sahip gerekiyorsa `Rc` kullanın. |
| `E0596` | `Rc` içindeki veri `&mut` olarak istendiğinde | `Rc<RefCell<T>>` ya da `Rc::make_mut` kullanın. |

`RefCell` ihlalleri derleme hatası vermez; panikle sonuçlanır.

## Perde arkası: Derleyici ne yapıyor?

> [!NOTE]
> Bu başlık ilk okuyuşta atlanabilir. Beş aracın her biri güvenliği başka bir yerde sağlar; aşağıda bunların derleyicideki ve standart kütüphanedeki karşılıkları var.

**`Box` ve boyut hesabı.** `Box` standart kütüphanede bir struct'tır; ikinci tür parametresi belleği nereden alacağını söyler ve varsayılanı genel ayırıcıdır.[^box-kod] (Bu parametreye kendi ayırıcınızı vermenizi sağlayan `Allocator` API'sinin çekirdeği Rust 1.100.0'da kararlı oldu.[^dropguard]) `E0072` hatası, bir türün sonlu boyutta gösterilip gösterilemeyeceğini denetleyen `check_representability` sorgusundan gelir: Sorgu alanları izlerken başladığı türe geri dönerse bir döngü oluşur ve `recursive_type_error` bu döngüyü hataya çevirir.[^e0072] Araya giren işaretçinin boyutu, gösterdiği türü bilmeden hesaplanabildiği için döngü oluşmaz.

**`Deref`.** Referansların, ham işaretçilerin ve `Box`'ın `*` ile açılması dilin yerleşik işlemidir. `x` bunlardan biri değilse derleyici `Deref` uygulamasına bakar; iki adım da `lookup_derefing` içindedir.[^deref-kod] Hangi `deref` çağrılarının kaç kez ekleneceği derleme sırasında kararlaştırılır.

**`Drop`.** `E0040`, metot çağrısının `Drop::drop` olup olmadığına bakan `check_legal_trait_for_method_call` fonksiyonundan gelir.[^e0040] Kapsam sonundaki çağrıları derleyici ekler: MIR üzerinde çalışan *drop elaboration* geçişi, yıkıcının yalnızca değer gerçekten başlatılmışsa çalışmasını sağlar.[^elaborate]

**`Rc`.** Değer heap'te `RcInner` adlı bir yapının içinde durur; yanında `strong` ve `weak` adlı iki sayaç vardır.[^rcinner] Sayaçların türü `Cell<usize>`'dır, yani `Rc` kendi içinde iç değiştirilebilirlik kullanır: `&Rc<T>` elindeyken sayacı artırabilmesi gerekir. Güçlü sayaç sıfıra inince değer düşürülür; heap'teki blok ise ortada hiç `Weak` kalmayınca geri verilir.[^rc-weak-blok] Atomik olmayan bu sayaçlar yüzünden `Rc` için `!Send` ve `!Sync` açıkça yazılmıştır.[^rc-send]

**`RefCell`.** İçinde bir `BorrowCounter` tutar: Pozitif sayı açık `Ref`'lerin, negatif sayı açık `RefMut`'ların sayısıdır, sıfır ise kimsenin ödünç almadığını gösterir.[^borrowcounter] İhlal `panic_already_borrowed` fonksiyonuna gider.[^panik] Değerin kendisi, iç değiştirilebilirliğin dildeki tek temeli olan `UnsafeCell` içinde durur;[^ic] ayrıntısı [25. bölümdedir](../advanced/25-send-sync.md).

## Özet

- Akıllı işaretçiler tek sahip kuralının dar geldiği yerler için kaçış kapılarıdır; her biri güvenliği başka bir yolla korur.
- `Box<T>` değeri heap'e koyar, boyutu sabit bir adres tutar. Özyineli türler araya böyle bir işaretçi ister.
- `Deref` kendi türünüze `*` işaretini öğretir; deref coercion `&Kutum<String>` gibi referansları `&str`'ye kadar kendiliğinden açar.
- `Drop` temizliği otomatikleştirir. Değerler tanımlanma sırasının tersine düşürülür; erken düşürmek için `drop(x)` yazılır, `x.drop()` yazılamaz.
- `Rc<T>` sahipleri sayar, veriyi kopyalamaz ve yalnızca okutur. `RefCell<T>` ödünç kuralını çalışma zamanında denetler; ihlal panikle sonuçlanır.
- Birbirini `Rc` ile tutan değerler sızar. Döngünün bir yönünü `Weak<T>` yapmak sorunu çözer.

Sıradaki sayfa [14.1 Akıllı İşaretçiler: Derinleşme](14-smart-pointers-deep-dive.md), bu araçları birlikte kullanıyor: Hangisini ne zaman seçmeli, ebeveynini bilen bir ağaç nasıl kurulur, döngü nasıl sızdırır ve `RefCell` ile en sık hangi hatalar yapılır?

## Kaynaklar

[^giris]: The Rust Programming Language, [15. Smart Pointers](https://doc.rust-lang.org/book/ch15-00-smart-pointers.html).
[^box]: The Rust Programming Language, [15.1 Using `Box<T>` to Point to Data on the Heap](https://doc.rust-lang.org/book/ch15-01-box.html); std, [`Box<T>`](https://doc.rust-lang.org/std/boxed/struct.Box.html).
[^ozyineli]: The Rust Programming Language, [Enabling Recursive Types with Boxes](https://doc.rust-lang.org/book/ch15-01-box.html#enabling-recursive-types-with-boxes); hata dizini, [`E0072`](https://doc.rust-lang.org/error_codes/E0072.html).
[^dead]: rustc 1.99.0 kaynak kodu, [`dead.rs`, `should_ignore_impl_item`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_passes/src/dead.rs#L438).
[^deref-impl]: The Rust Programming Language, [Implementing the `Deref` Trait](https://doc.rust-lang.org/book/ch15-02-deref.html#implementing-the-deref-trait); std, [`Deref`](https://doc.rust-lang.org/std/ops/trait.Deref.html); hata dizini, [`E0614`](https://doc.rust-lang.org/error_codes/E0614.html).
[^coercion]: The Rust Programming Language, [Using Deref Coercion in Functions and Methods](https://doc.rust-lang.org/book/ch15-02-deref.html#using-deref-coercion-in-functions-and-methods); std, [Deref coercion](https://doc.rust-lang.org/std/ops/trait.Deref.html#deref-coercion).
[^derefmut]: The Rust Programming Language, [Handling Deref Coercion with Mutable References](https://doc.rust-lang.org/book/ch15-02-deref.html#handling-deref-coercion-with-mutable-references).
[^sira]: The Rust Reference, [Destructors: Drop scopes](https://doc.rust-lang.org/reference/destructors.html#drop-scopes).
[^drop]: The Rust Programming Language, [15.3 Running Code on Cleanup with the `Drop` Trait](https://doc.rust-lang.org/book/ch15-03-drop.html); std, [You cannot call `Drop::drop` yourself](https://doc.rust-lang.org/std/ops/trait.Drop.html#you-cannot-call-dropdrop-yourself) ve [`mem::drop`](https://doc.rust-lang.org/std/mem/fn.drop.html).
[^dropguard]: Rust blogu, [Announcing Rust 1.100.0](https://blog.rust-lang.org/2026/11/12/Rust-1.100.0/) (`DropGuard` ve `Allocator` başlıkları); std 1.100.0, [`DropGuard`](https://doc.rust-lang.org/1.100.0/std/mem/struct.DropGuard.html).
[^rc]: The Rust Programming Language, [15.4 `Rc<T>`, the Reference-Counted Smart Pointer](https://doc.rust-lang.org/book/ch15-04-rc.html); std, [`std::rc`](https://doc.rust-lang.org/std/rc/index.html).
[^rc-clone]: The Rust Programming Language, [Sharing Data](https://doc.rust-lang.org/book/ch15-04-rc.html#sharing-data) ve [Cloning to Increase the Reference Count](https://doc.rust-lang.org/book/ch15-04-rc.html#cloning-to-increase-the-reference-count).
[^rc-thread]: The Rust Programming Language, [Atomic Reference Counting with `Arc<T>`](https://doc.rust-lang.org/book/ch16-03-shared-state.html#atomic-reference-counting-with-arct).
[^refcell-kural]: The Rust Programming Language, [Enforcing Borrowing Rules at Runtime](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html#enforcing-borrowing-rules-at-runtime).
[^ic]: The Rust Reference, [Interior mutability](https://doc.rust-lang.org/reference/interior-mutability.html); std, [`std::cell`](https://doc.rust-lang.org/std/cell/index.html).
[^panik]: std 1.99.0 kaynak kodu, [`cell.rs`, `panic_already_borrowed`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/cell.rs#L915); std, [`RefCell::borrow_mut`](https://doc.rust-lang.org/std/cell/struct.RefCell.html#method.borrow_mut).
[^rc-refcell]: The Rust Programming Language, [15.5 `RefCell<T>` and the Interior Mutability Pattern](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html).
[^dongu]: The Rust Programming Language, [15.6 Reference Cycles Can Leak Memory](https://doc.rust-lang.org/book/ch15-06-reference-cycles.html); The Rustonomicon, [Leaking](https://doc.rust-lang.org/nomicon/leaking.html).
[^weak]: The Rust Programming Language, [Preventing Reference Cycles Using `Weak<T>`](https://doc.rust-lang.org/book/ch15-06-reference-cycles.html#preventing-reference-cycles-using-weakt); std, [`Weak::upgrade`](https://doc.rust-lang.org/std/rc/struct.Weak.html#method.upgrade).
[^box-kod]: std 1.99.0 kaynak kodu, [`boxed.rs`, `Box`](https://github.com/rust-lang/rust/blob/1.99.0/library/alloc/src/boxed.rs#L236).
[^e0072]: rustc 1.99.0 kaynak kodu, [`handle_cycle_error.rs`, `recursive_type_error`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_query_impl/src/handle_cycle_error.rs#L216).
[^deref-kod]: rustc 1.99.0 kaynak kodu, [`place_op.rs`, `lookup_derefing`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_typeck/src/place_op.rs#L21) ve [`sty.rs`, `builtin_deref`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_middle/src/ty/sty.rs#L1618); std 1.99.0 kaynak kodu, [`deref.rs`, `Deref`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/ops/deref.rs#L139).
[^e0040]: rustc 1.99.0 kaynak kodu, [`callee.rs`, `check_legal_trait_for_method_call`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_typeck/src/callee.rs#L50); hata dizini, [`E0040`](https://doc.rust-lang.org/error_codes/E0040.html).
[^elaborate]: rustc 1.99.0 kaynak kodu, [`elaborate_drops.rs`, `ElaborateDrops`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_mir_transform/src/elaborate_drops.rs#L49); rustc geliştirme kılavuzu, [Drop elaboration](https://rustc-dev-guide.rust-lang.org/mir/drop-elaboration.html).
[^rcinner]: std 1.99.0 kaynak kodu, [`rc.rs`, `RcInner`](https://github.com/rust-lang/rust/blob/1.99.0/library/alloc/src/rc.rs#L285).
[^rc-weak-blok]: std 1.99.0 kaynak kodu, [`rc.rs`, `Drop for Rc`](https://github.com/rust-lang/rust/blob/1.99.0/library/alloc/src/rc.rs#L2470) ve [`drop_slow`](https://github.com/rust-lang/rust/blob/1.99.0/library/alloc/src/rc.rs#L397); std, [`Weak<T>`](https://doc.rust-lang.org/std/rc/struct.Weak.html).
[^rc-send]: std 1.99.0 kaynak kodu, [`rc.rs`, `!Send for Rc`](https://github.com/rust-lang/rust/blob/1.99.0/library/alloc/src/rc.rs#L334).
[^borrowcounter]: std 1.99.0 kaynak kodu, [`cell.rs`, `BorrowCounter`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/cell.rs#L948).
