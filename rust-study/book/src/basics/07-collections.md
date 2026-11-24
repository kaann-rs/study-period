# 7. Koleksiyonlar

Bir dizinin (`[i32; 3]`) uzunluğu derleme zamanında bellidir ve sonradan değişmez. Oysa programların çoğu kaç değerle karşılaşacağını önceden bilmez: Bir dosyada kaç satır olduğu, bir toplantıya kaç kişinin geleceği, bir metinde hangi sözcüklerin geçtiği ancak program çalışırken ortaya çıkar. Birden çok değeri bir arada tutan ve program çalışırken büyüyüp küçülebilen türlere *koleksiyon* (collection) denir.

Standart kütüphanede birçok koleksiyon vardır; bu bölümde en sık kullanılan üçünü göreceğiz:[^koleksiyonlar]

- `Vec<T>`, aynı türden değerleri yan yana dizer.
- `String`, UTF-8 ile kodlanmış metin tutar.
- `HashMap<K, V>`, anahtarları değerlerle eşler.

Üçü de içeriğinin sahibidir, içeriğini heap'te tutar ve standart kütüphanede Rust ile yazılmıştır; derleyiciye gömülü özel türler değildir. Yine de güvenliklerinin önemli bir bölümü kütüphaneden değil, dilin kendisinden gelir. Örneğin bir `Vec`'in elemanına referans tutarken `Vec`'i büyütmenizi, [3. bölümde](03-ownership.md) tanıdığımız ödünç denetleyicisi engeller. Bunun nedenini birazdan göreceğiz.

> [!NOTE]
> Bu bölümün kodu `learn/collections` dizinindedir. Depo kökünden `cargo run -p collections` komutuyla çalıştırabilirsiniz.

## Günlük hayattan bir benzetme

Bir çocuk odası düşünün. Odada oyuncak bir tren, boncuklardan yapılmış bir kolye ve üzerlerinde etiket olan çekmeceler var.

Trenin vagonları sıra sıra dizilir; sona yeni vagon takabilir, gerektiğinde treni kısaltabilirsiniz. Her vagon aynı cins yük taşır. Vagonlar 0'dan numaralanır. Trende olmayan bir vagonu zorla almaya kalkarsanız oyun durur; "Öyle bir vagon var mı?" diye sorarsanız yalnızca "yok" yanıtını alırsınız. `Vec` böyle bir trendir.

Kolyenin boncukları aynı boyda değildir: `a` kısa, `ş` uzun bir boncuktur. Kolyeyi bir boncuğun ortasından keserseniz elinizde yarım boncuk kalır ve kolye bozulur. `String` böyle bir kolyedir: Boyu *bayt* (byte) ile ölçülür, boncukların her biri bir *karakterdir* (`char`).

Çekmecelerin üzerinde adlar yazar. "Çorap" yazan çekmeceye çorap koyar, sonra aynı adla bulursunuz; kaçıncı çekmece olduğunu bilmeniz gerekmez. `HashMap`'te etikete *anahtar* (key), çekmecenin içindekine *değer* (value) denir.

<details>
<summary>Bu bölümde geçen terimler</summary>

- **koleksiyon (collection):** Birden çok değeri bir arada tutan tür.
- **vektör (`Vec<T>`):** Hepsi `T` türünden elemanların büyüyüp küçülebilen sırası.
- **`vec!`:** İçi dolu bir `Vec`'i kısa yoldan kuran makro: `vec![1, 2, 3]`.
- **`push`:** Sona bir eleman ekleyen metot.
- **indeks (index):** Elemanın sıra numarası. Saymaya 0'dan başlanır: `v[0]`.
- **`get`:** İndeksle erişimin panik yaratmayan yolu. Eleman varsa `Some(..)`, yoksa `None` verir.
- **`Option`, `Some`, `None`:** "Var, işte bu" ya da "yok" bilgisini taşıyan enum. [5. bölümde](05-enums.md) anlatılmıştı.
- **panik (panic):** Programın düzeltilemeyecek bir yanlışla karşılaşıp işi yarıda kesmesi.
- **`&` ve `&mut`:** Bakmak için ve değiştirmek için ödünç alma.
- **başvuruyu çözmek (dereference):** `*x` yazarak `x` referansının gösterdiği değerin kendisine ulaşmak.
- **heap:** Boyu çalışma zamanında belli olan verilerin durduğu bellek bölgesi.
- **işaretçi, uzunluk, kapasite (pointer, length, capacity):** Bir `Vec`'i oluşturan üç bilgi: verinin yeri, eleman sayısı, ayrılmış yer.
- **yeniden tahsis (reallocation):** Yer yetmeyince daha büyük bir bellek bloğuna geçmek.
- **sarkan referans (dangling reference):** Gösterdiği veri taşınmış ya da silinmiş olan referans.
- **iterator geçersizleşmesi (iterator invalidation):** Bir koleksiyonu gezerken değiştirip gezmeyi bozmak.
- **ödünç denetleyicisi (borrow checker):** Derleyicinin ödünç kurallarını denetleyen bölümü.
- **`String` ve `&str`:** Sahip olduğunuz, büyüyebilen metin ve bir metne (ya da parçasına) bakış.
- **`char`:** Tek bir Unicode skaler değeri; kabaca bir harf ya da işaret.
- **bayt (byte):** Belleğin adreslenebilen en küçük birimi. UTF-8'de bir karakter 1 ile 4 bayt arasında yer tutar.
- **UTF-8:** Karakterlerin baytlara nasıl yazılacağını belirleyen yaygın kodlama.
- **dilim (slice):** Bir metnin ya da listenin bir parçasına bakış: `&t[0..2]`.
- **klonlama (clone):** Değerin bağımsız bir eşini üretmek.
- **`format!`:** `println!` gibi çalışan, ama metni ekrana yazmak yerine `String` olarak döndüren makro.
- **`HashMap<K, V>`:** `K` türünden anahtarları `V` türünden değerlerle eşleyen tür.
- **`insert`:** Bir anahtarı değeriyle birlikte koyar; anahtar zaten varsa değerini değiştirir.
- **`entry` ve `or_insert`:** "Bu anahtar varsa değerini ver, yoksa şu değerle oluştur" demenin yolu.
- **özet (hash):** Anahtardan, değerin saklanacağı yeri hesaplayan işlem ve bu işlemin sonucu.
- **tohum (seed):** Özet hesabına katılan rastgele sayı.
- **`Copy` ve taşıma (move):** `Copy` olan türler kopyalanır; `String` ve `Vec` gibi heap sahipleri taşınır.
- **deyimsel (idiomatic):** Dili bilenlerin alıştığı, yerleşik yazım biçimi.
- **belirlenimci (deterministic):** Her çalıştırmada aynı sonucu veren.
- **standart kütüphane (`std`):** Rust ile birlikte gelen hazır kod.
- **derleyici (`rustc`):** Kodu programa çeviren ve hataları önceden bulan araç.

</details>

## `Vec<T>`: Büyüyebilen sıra

### Oluşturmak ve eleman eklemek

Boş bir vektör `Vec::new` ile kurulur, elemanlar `push` ile sona eklenir. Başlangıç değerleri belliyse `vec!` makrosu daha kısadır:[^vec-yeni]

```rust
fn main() {
    let mut v: Vec<i32> = Vec::new();
    v.push(10);
    v.push(20);
    v.push(30);

    let v2 = vec![1, 2, 3];

    println!("{v:?} {v2:?}");
}
```

Çıktı:

```text
[10, 20, 30] [1, 2, 3]
```

`push` vektörü değiştirdiği için `v`, `mut` olarak tanımlanmıştır. `Vec::new` satırındaki tür bilgisi burada zorunlu değildir: `push(10)` çağrısı elemanların türünü zaten belirler. Hiç eleman eklemeseydiniz derleyici `T`'nin ne olduğunu bilemezdi ve türü yazmanız gerekirdi.

Bir vektör kapsam dışına çıktığında elemanları da onunla birlikte düşürülür.[^vec-drop]

### Elemanlara erişmek: indeks ve `get`

Bir elemana iki yolla ulaşılır. Köşeli parantezle indeks vermek doğrudan elemanı verir. `get` metodu ise bir `Option<&T>` döndürür:[^vec-erisim]

```rust
fn main() {
    let v = vec![10, 20, 30];

    println!("v[0] = {}", v[0]);
    println!("{:?}", v.get(1));

    match v.get(99) {
        Some(x) => println!("{x}"),
        None => println!("99 indeksli eleman yok"),
    }
}
```

Çıktı:

```text
v[0] = 10
Some(20)
99 indeksli eleman yok
```

İki yol, olmayan bir eleman istendiğinde ayrılır. `get` size `None` verir ve program sürer. İndeks ise panik yaratır:

```rust,should_panic
fn main() {
    let v = vec![10, 20, 30];
    let x = v[99];
    println!("{x}");
}
```

Programın standart hata akışındaki çıktısı:

```text
index out of bounds: the len is 3 but the index is 99
```

Seçim, eksik elemanın sizin için ne anlama geldiğine bağlıdır. İndeksin geçersiz olması programdaki bir hataya işaret ediyorsa `v[i]` ile hemen durmak doğrudur. Dışarıdan gelen bir sayıyla erişiyorsanız geçersiz indeks olağan bir durumdur; `get` kullanıp `None` kolunu ele alın. Panik konusuna [8. bölümde](08-error-handling.md) ayrıntılı bakacağız.

### Elemanları gezmek

Bir vektörü `for` ile gezerken `&v` yazarsanız elemanlara paylaşımlı referans alırsınız; `&mut v` yazarsanız her elemanı yerinde değiştirebilirsiniz:[^vec-gezme]

```rust
fn main() {
    let mut v = vec![10, 20, 30];

    let mut toplam = 0;
    for x in &v {
        toplam += x;
    }
    println!("toplam = {toplam}");

    for x in &mut v {
        *x += 1;
    }
    println!("{v:?}");
}
```

Çıktı:

```text
toplam = 60
[11, 21, 31]
```

İkinci döngüde `x`'in türü `&mut i32`'dir. `*x`, referansın gösterdiği sayının kendisine ulaşır; `+=` işlemi bu sayede vektörün içindeki değeri değiştirir. `for x in v` yazsaydık vektörün sahipliği döngüye geçerdi ve `v` sonrasında kullanılamazdı.

### Bir elemana referans varken `push`

Rust'a yeni başlayanları sık sık şaşırtan bir hatayla devam edelim:

```rust,compile_fail,E0502
fn main() {
    let mut v = vec![10, 20, 30];
    let ilk = &v[0];
    v.push(40);
    println!("{ilk}");
}
```

Derleyicinin yanıtı:

```text
error[E0502]: cannot borrow `v` as mutable because it is also borrowed as immutable
  |
3 |     let ilk = &v[0];
  |                - immutable borrow occurs here
4 |     v.push(40);
  |     ^^^^^^^^^^ mutable borrow occurs here
5 |     println!("{ilk}");
  |                --- immutable borrow later used here
```

İlk bakışta kural aşırı sıkı görünür: Sona eleman eklemenin ilk elemanla ne ilgisi olabilir? Yanıt, vektörün bellekteki düzenindedir. Elemanlar heap'te tek bir blokta yan yana durur. Blok dolmuşken yeni eleman eklerseniz vektör daha büyük bir blok ister; yeni blok başka bir adreste olabilir. Bu durumda elemanlar oraya taşınır ve eski blok geri verilir.[^vec-push] Tren benzetmesiyle söylersek vagonlar raya sığmayınca bütün tren daha uzun bir raya aktarılır ve eski ray sökülür. Eski raydaki ilk vagonu gösteren parmağınız artık boşluğu gösterir.

`ilk`, böyle bir aktarmadan sonra geri verilmiş belleği gösteren bir sarkan referans olurdu; onu okumak tanımsız davranıştır. Derleyici o anda yer olup olmadığına bakmaz. `push` vektörü `&mut` ile ister, `ilk` yaşadığı sürece vektör `&` ile ödünçtedir ve ikisi bir arada bulunamaz.[^vec-e0502]

Çözüm çoğu zaman sırayı değiştirmektir. Referansla işinizi bitirin, sonra ekleyin:

```rust
fn main() {
    let mut v = vec![10, 20, 30];

    let ilk = &v[0];
    println!("{ilk}"); // ilk'in son kullanımı

    v.push(40);
    println!("{v:?}");
}
```

Çıktı:

```text
10
[10, 20, 30, 40]
```

Eleman `Copy` ise referans yerine değerin kendisini almak da yeterlidir: `let ilk = v[0];`.

Aynı kural, gezilen bir vektöre eleman eklemeyi de engeller. Pek çok dilde *iterator geçersizleşmesi* (iterator invalidation) denen ve ancak program çalışırken ortaya çıkan bu hata, Rust'ta derlenmez:

```rust,compile_fail,E0502
fn main() {
    let mut v = vec![1, 2, 3];
    for x in &v {
        if *x == 2 {
            v.push(4);
        }
    }
}
```

Derleyicinin yanıtı:

```text
error[E0502]: cannot borrow `v` as mutable because it is also borrowed as immutable
  |
3 |       for x in &v {
  |       -        --
  |       |        |
  |       |        immutable borrow occurs here
  |  _____|        immutable borrow later used here
  | |
4 | |         if *x == 2 {
5 | |             v.push(4);
  | |             ^^^^^^^^^ mutable borrow occurs here
6 | |         }
7 | |     }
  | |_____- this for loop borrows `v` immutably, preventing mutation within its body
  |
  = help: consider using an index-based loop instead, or collecting modifications into a separate collection
```

### Farklı türleri aynı vektörde tutmak

Bir vektörün bütün elemanları aynı türdendir. Bir sayı ile bir metni yan yana koyamazsınız:

```rust,compile_fail,E0308
fn main() {
    let karisik = vec![3, "top"];
}
```

Derleyicinin yanıtı:

```text
error[E0308]: mismatched types
  |
2 |     let karisik = vec![3, "top"];
  |                           ^^^^^ expected integer, found `&str`
```

Hangi türlerin gelebileceği baştan belliyse bir enum tanımlanır. Enum'un her varyantı başka bir tür taşıyabilir, ama hepsi aynı enum'un değeridir. Trenin her vagonuna aynı cins kutu koymak gibidir: Kutunun üzerinde içinde ne olduğu yazar.[^vec-enum]

```rust
#[derive(Debug)]
enum Hucre {
    Sayi(i32),
    Metin(String),
    Bos,
}

fn main() {
    let satir = vec![
        Hucre::Sayi(3),
        Hucre::Metin(String::from("merhaba")),
        Hucre::Bos,
    ];
    println!("satır = {satir:?}");

    for h in &satir {
        match h {
            Hucre::Sayi(n) => println!("  sayı: {n}"),
            Hucre::Metin(m) => println!("  metin: {m}"),
            Hucre::Bos => println!("  boş"),
        }
    }
}
```

Çıktı:

```text
satır = [Sayi(3), Metin("merhaba"), Bos]
  sayı: 3
  metin: merhaba
  boş
```

`match` burada bütün varyantları ele almak zorundadır; böylece hücre türlerinden birini unutma ihtimali kalmaz.

> [!NOTE]
> Varyantların içindeki değerleri yalnızca `{:?}` ile yazdırıp hiç okumasaydınız derleyici bir uyarı verirdi: ``field `0` is never read``. Türetilen `Debug` uygulaması, kullanılmayan kod çözümlemesinde bilerek yok sayılır. [5. bölümde](05-enums.md) gördüğümüz bu durumda çözüm, alanları yukarıdaki gibi gerçekten okumaktır.[^dead-code]

## `String`: Büyüyebilen metin

Rust'ın çekirdek dilinde tek bir metin türü vardır: çoğunlukla `&str` biçiminde karşımıza çıkan dilim `str`. `&str`, başka bir yerde duran metne bakan bir ödünçtür ve büyütülemez. `String` ise standart kütüphanenin sağladığı, sahip olunan ve büyüyebilen türdür. İkisi de UTF-8 ile kodlanmıştır.[^string-nedir]

### Oluşturmak ve büyütmek

`push_str` bir metin dilimini, `push` tek bir karakteri sona ekler:[^string-push]

```rust
fn main() {
    let mut s = String::from("mer");
    s.push_str("haba");
    s.push('!');

    println!("{s}");
}
```

Çıktı:

```text
merhaba!
```

`push_str` bir `&str` ister; eklenen metnin sahipliğini almaz, yalnızca içeriğini kopyalar.

### Birleştirmek: `+` ve `format!`

İki metni birleştirmenin iki yolu vardır. `+` operatörü soldaki `String`'i tüketir, sağında `&str` bekler:[^string-arti]

```rust
fn main() {
    let s = String::from("merhaba!");
    let d = String::from("dünya");

    let birlesik = s.clone() + " " + &d;
    let f = format!("{s} {d}");

    println!("{birlesik} / {f}");
}
```

Çıktı:

```text
merhaba! dünya / merhaba! dünya
```

`clone` çağrısı gereksiz görünebilir, ama onsuz `s` sonraki satırda kullanılamazdı:

```rust,compile_fail,E0382
fn main() {
    let s = String::from("merhaba!");
    let d = String::from("dünya");

    let birlesik = s + " " + &d;
    println!("{s} / {birlesik}");
}
```

Derleyicinin yanıtı:

```text
error[E0382]: borrow of moved value: `s`
  |
2 |     let s = String::from("merhaba!");
  |         - move occurs because `s` has type `String`, which does not implement the `Copy` trait
...
5 |     let birlesik = s + " " + &d;
  |                    ------- `s` moved due to usage in operator
6 |     println!("{s} / {birlesik}");
  |                ^ value borrowed here after move
```

`+` operatörü `Add::add(self, &str)` metoduna karşılık gelir.[^string-add] Soldaki `String` taşınır, tamponu gerekirse büyütülerek yeniden kullanılır ve sağdaki metin onun sonuna eklenir. Sağ tarafa `String`'in kendisini yazarsanız `E0308` alırsınız; `&d` yazdığınızda ise derleyici `&String`'i kendiliğinden `&str`'ye dönüştürür (deref coercion).

`format!` ise argümanlarının hiçbirini tüketmez ve yeni bir `String` üretir.[^string-format] İki parçadan fazlası söz konusu olduğunda okunaklı olan da odur; gereksiz `clone` çağrılarından da kurtarır.

### Bayt ve karakter: `t[0]` neden yok?

UTF-8'de bir karakter 1 ile 4 bayt arasında yer tutar.[^char-utf8] `len` metodu karakterleri değil, baytları sayar:[^str-len]

```rust
fn main() {
    let t = "şema";

    println!("'{t}': bayt = {}, karakter = {}", t.len(), t.chars().count());
    println!("{:?}", t.chars().next());
}
```

Çıktı:

```text
'şema': bayt = 5, karakter = 4
Some('ş')
```

`ş` iki bayt tuttuğu için dört karakterlik metin beş bayttır. Bu yüzden bir metne tam sayı indeksle erişmek yoktur:

```rust,compile_fail,E0277
fn main() {
    let t = "şema";
    let c = t[0];
    println!("{c}");
}
```

Derleyicinin yanıtı:

```text
error[E0277]: the type `str` cannot be indexed by `{integer}`
  |
3 |     let c = t[0];
  |               ^ string indices are ranges of `usize`
  |
  = help: the trait `SliceIndex<str>` is not implemented for `{integer}`
  = note: you can use `.chars().nth()` or `.bytes().nth()`
```

Bunun iki nedeni vardır.[^string-indeks] Birincisi anlam belirsizliğidir: `t[0]` ilk baytı mı, ilk karakteri mi vermelidir? İlk bayt tek başına geçerli bir harf değildir; kolyeden yarım boncuk koparmak gibidir. İkincisi maliyettir: İndeksle erişimin, metin ne kadar uzun olursa olsun sabit sürede, yani O(1)'de bitmesi beklenir. Karakterlerin boyu değiştiği için *n*'inci karaktere ulaşmak baştan saymayı gerektirir. Rust bu belirsizliği ve maliyeti gizlemek yerine tam sayı indeksi hiç sunmaz.

### Metinden dilim almak

Bayt aralığı vererek bir metnin dilimini alabilirsiniz; ancak aralığın iki ucu da bir karakter sınırına denk gelmelidir. `&t[0..2]` geçerlidir ve `ş` harfini verir. `&t[0..1]` ise `ş`'nin ortasından keser ve çalışma zamanında panik yaratır:[^string-dilim]

```rust,should_panic
fn main() {
    let t = "şema";

    println!("{}", &t[0..2]);
    println!("{}", &t[0..1]);
}
```

Programın çıktısı (panik iletisi standart hata akışına yazılır):

```text
ş
end byte index 1 is not a char boundary; it is inside 'ş' (bytes 0..2 of string)
```

Derleyici bu hatayı yakalayamaz, çünkü karakter sınırlarının nerede olduğu metnin içeriğine bağlıdır. Sınırın tuttuğundan emin değilseniz panik yerine `Option` döndüren `get` kullanılabilir: `t.get(0..1)` sonucu `None`, `t.get(0..2)` sonucu `Some("ş")` olur.[^str-get]

### Metni gezmek: baytlar, karakterler ve harfler

Metnin parçalarıyla çalışmanın güvenli yolu, hangi parçayı istediğinizi açıkça söylemektir. `chars` karakterleri, `bytes` ham baytları gezer:[^string-gezme]

```rust
fn main() {
    let t = "şema";

    let ilk_char: String = t.chars().take(1).collect();
    println!("ilk karakter: {ilk_char}");

    print!("'ab' baytları: ");
    for b in "ab".bytes() {
        print!("{b} ");
    }
    println!();
}
```

Çıktı:

```text
ilk karakter: ş
'ab' baytları: 97 98
```

`chars().take(1)` metnin ilk karakterini alır, `collect` onu bir `String`'de toplar. Tek karakter yetiyorsa yukarıdaki gibi `chars().next()` yazmak daha kısadır. (`take` ve `collect` gibi iterator metotlarını [12. bölümde](../intermediate/12-closures-iterators.md) ele alacağız.)

Üçüncü bir bakış daha vardır: insanların "harf" diye algıladığı *grafem kümeleri* (grapheme cluster). Örneğin bir harf ile onun üzerine binen ayrı bir aksan işareti iki `char` sayılır, ama okur tek harf görür. Grafem kümelerini ayırmak standart kütüphanenin işi değildir; bunun için crates.io'daki crate'ler kullanılır.[^grafem]

## `HashMap<K, V>`: Anahtardan değere

### Eklemek ve okumak

Bir vektörde aradığınızı bulmak için çoğu zaman baştan sona bakarsınız. `HashMap`'te ise anahtarı verir, değere doğrudan ulaşırsınız.[^maliyet] Tür, başlangıçta kendiliğinden gelen adlar (prelude) arasında yer almadığından `use` ile kapsama alınır:[^hashmap-yeni]

```rust
use std::collections::HashMap;

fn main() {
    let mut puan: HashMap<String, i32> = HashMap::new();
    puan.insert(String::from("mavi"), 10);
    puan.insert(String::from("kirmizi"), 50);

    match puan.get("mavi") {
        Some(v) => println!("mavi = {v}"),
        None => println!("yok"),
    }
    println!("{:?}", puan.get("mor"));
}
```

Çıktı:

```text
mavi = 10
None
```

`get`, `Option<&V>` döndürür: Anahtar varsa değerine bir referans, yoksa `None`.[^hashmap-get] Haritanın anahtarları `String` olduğu hâlde `get`'e `&str` verebildiğimize dikkat edin. Metot, anahtar türünün ödünç alınmış biçimini de kabul eder; yalnızca aramak için bir `String` kurmanız gerekmez.

### Haritanın sahipliği

`HashMap` anahtarlarının ve değerlerinin sahibidir. `i32` gibi `Copy` türler haritaya kopyalanır; `String` gibi türler ise taşınır:[^hashmap-sahip]

```rust,compile_fail,E0382
use std::collections::HashMap;

fn main() {
    let mut puan: HashMap<String, i32> = HashMap::new();

    let k = String::from("yesil");
    puan.insert(k, 5);
    println!("{k}");
}
```

Derleyicinin yanıtı:

```text
error[E0382]: borrow of moved value: `k`
  |
6 |     let k = String::from("yesil");
  |         - move occurs because `k` has type `String`, which does not implement the `Copy` trait
7 |     puan.insert(k, 5);
  |                 - value moved here
8 |     println!("{k}");
  |                ^ value borrowed here after move
```

`insert`'ten sonra anahtarın sahibi haritadır. Metne sonra da ihtiyacınız varsa önce kullanıp sonra ekleyebilir, ekledikten sonra haritadan okuyabilir ya da iki bağımsız metin gerçekten gerekiyorsa `k.clone()` verebilirsiniz.

### Değeri güncellemek

Aynı etiketten iki çekmece olmaz. Var olan bir anahtarla `insert` çağırırsanız yeni değer eskisinin yerine geçer ve eski değer size geri verilir:[^hashmap-insert]

```rust
use std::collections::HashMap;

fn main() {
    let mut puan: HashMap<String, i32> = HashMap::new();

    println!("{:?}", puan.insert(String::from("mavi"), 10));
    println!("{:?}", puan.insert(String::from("mavi"), 25));
    println!("mavi = {:?}, eleman sayısı = {}", puan.get("mavi"), puan.len());
}
```

Çıktı:

```text
None
Some(10)
mavi = Some(25), eleman sayısı = 1
```

Bazen istediğiniz, değeri ezmek değil "varsa ona dokun, yoksa şununla başlat" demektir. Bunun deyimsel yolu `entry` metodudur. `entry`, anahtarın yerini gösteren bir `Entry` değeri verir. `or_insert(x)` ise anahtar varsa onun değerine, yoksa `x` ile oluşturduğu yeni değere `&mut V` döndürür:[^hashmap-entry]

```rust
use std::collections::HashMap;

fn main() {
    let mut puan = HashMap::from([
        (String::from("mavi"), 10),
        (String::from("kirmizi"), 50),
    ]);

    *puan.entry(String::from("mavi")).or_insert(0) += 1;
    puan.entry(String::from("sari")).or_insert(7);

    println!("mavi = {:?}", puan.get("mavi"));
    println!("sari = {:?}", puan.get("sari"));
    println!("eleman sayısı = {}", puan.len());
}
```

Çıktı:

```text
mavi = Some(11)
sari = Some(7)
eleman sayısı = 3
```

Çekmece diliyle söylersek `entry(..).or_insert(0)` şunu der: "Bu etiketli çekmece varsa onu aç; yoksa içine 0 koyup yeni bir çekmece yap ve onu aç." Sonuç bir referans olduğundan çekmecenin içini değiştirmek için başına `*` yazılır. "mavi" anahtarı haritada olduğundan `or_insert(0)` yeni değer koymadı, var olan 10'un referansını verdi. "sari" yoktu; 7 ile oluşturuldu.

### Örnek: kelime saymak

`entry` ile kelime sayan bir program birkaç satırdır. Kelime ilk kez görülüyorsa sayacı 0 ile başlatılır; her durumda sayaç bir artırılır:[^hashmap-sayma]

```rust
use std::collections::HashMap;

fn main() {
    let metin = "a b a c b a";

    let mut say: HashMap<&str, i32> = HashMap::new();
    for kelime in metin.split_whitespace() {
        *say.entry(kelime).or_insert(0) += 1;
    }

    let mut ciftler: Vec<_> = say.iter().collect();
    ciftler.sort();
    println!("kelime sayımı: {ciftler:?}");
}
```

Çıktı:

```text
kelime sayımı: [("a", 3), ("b", 2), ("c", 1)]
```

Anahtarlar `&str` olduğu için harita `metin`'in parçalarına bakar; `metin`, en az harita kullanıldığı sürece geçerli kalmak zorundadır. `Vec<_>` yazımındaki alt çizgi, eleman türünü derleyicinin bulmasını ister.

### Gezinme sırası belli değildir

Son örnekte çiftleri yazdırmadan önce bir vektöre alıp sıraladık. Bunun nedeni, bir `HashMap`'in elemanlarını gezerken hangi sırayla geleceğinin belirtilmemiş olmasıdır.[^hashmap-sira] Şu program her çalıştırmada başka bir sıra yazabilir:

```rust
use std::collections::HashMap;

fn main() {
    let h: HashMap<i32, ()> = (0..8).map(|i| (i, ())).collect();
    println!("{:?}", h.keys().collect::<Vec<_>>());
}
```

Art arda iki örnek çalıştırma (sizde farklı olacaktır):

```text
[4, 0, 1, 2, 5, 3, 6, 7]
[0, 6, 3, 5, 7, 4, 1, 2]
```

Sıra önemliyse iki seçenek vardır: Ya yukarıdaki gibi elemanları toplayıp sıralarsınız ya da anahtarlarını sıralı tutan `BTreeMap`'i kullanırsınız.[^btreemap] `BTreeMap` de `entry` sunar; kelime sayma örneğinde sıralama adımı gereksizleşir:

```rust
use std::collections::BTreeMap;

fn main() {
    let mut say: BTreeMap<&str, i32> = BTreeMap::new();
    for kelime in "a b a c b a".split_whitespace() {
        *say.entry(kelime).or_insert(0) += 1;
    }
    println!("{say:?}");
}
```

Çıktı:

```text
{"a": 3, "b": 2, "c": 1}
```

## Hata kodu başvurusu

| Kod | Ne zaman görülür? | Ne yapmalı? |
| --- | --- | --- |
| `E0502` | Bir elemana referans yaşarken `push` çağrıldığında ya da gezilen vektör değiştirildiğinde | Referansın son kullanımını öne alın, değeri kopyalayın ya da değişiklikleri ayrı bir vektörde toplayın. |
| `E0308` | Bir vektöre farklı türden elemanlar konduğunda; `s + d` yazılıp sağa `String` verildiğinde | Türleri bir enum'da toplayın; `+`'nın sağına `&d` yazın. |
| `E0277` | `str` ya da `String` tam sayıyla indekslendiğinde | `chars`, `bytes` ya da bayt aralığıyla dilim kullanın. |
| `E0382` | `+` ile tüketilen `String` ya da haritaya verilen anahtar yeniden kullanıldığında | `format!` kullanın, sırayı değiştirin ya da gerçekten iki değer gerekiyorsa klonlayın. |

Derleyicinin yakalayamadığı, çalışma zamanında paniğe yol açan iki durum daha vardır: sınır dışı indeks (`v[99]`) ve karakter sınırına denk gelmeyen dilim (`&t[0..1]`). İkisinin de panik yaratmayan karşılığı `get`'tir.

## Perde arkası: Derleyici ne yapıyor?

> [!NOTE]
> Bu başlık ilk okuyuşta atlanabilir; program yazarken gereken kurallar yukarıdadır. Burada trenin neden başka raya taşındığına, kolyenin aslında baytlardan oluştuğuna ve çekmecelerin neden her seferinde başka sırayla gezildiğine bakıyoruz.

**`Vec` üç alandan oluşur.** Belgede verilen güvence, bir `Vec`'in her zaman bir işaretçi, bir kapasite ve bir uzunluktan oluşacağıdır; alanların sırası belirtilmemiştir.[^vec-garanti] Kapasite elemanlar için ayrılmış yeri, uzunluk gerçekten var olan elemanları sayar:

```rust
fn main() {
    let mut v: Vec<i32> = Vec::new();
    println!("uzunluk = {}, kapasite = {}", v.len(), v.capacity());

    for i in 1..=5 {
        v.push(i);
        println!("uzunluk = {}, kapasite = {}", v.len(), v.capacity());
    }
}
```

Rust 1.99.0'daki çıktı:

```text
uzunluk = 0, kapasite = 0
uzunluk = 1, kapasite = 4
uzunluk = 2, kapasite = 4
uzunluk = 3, kapasite = 4
uzunluk = 4, kapasite = 4
uzunluk = 5, kapasite = 8
```

Uzunluk kapasiteye ulaştığında `push` daha büyük bir blok ister. Şu anki uygulama kapasiteyi en az iki katına çıkarır,[^vec-buyume] ama belge belirli bir büyüme stratejisi için söz vermez; yalnızca `push`'un *amortize* O(1) olmasını güvence altına alır. Amortize O(1), arada bir pahalı bir aktarma olsa da eklemelerin ortalamada sabit maliyetli kalması demektir. Aktarma sırasında tahsis edici bloğu yerinde büyütebilir ya da yeni bir bloğa geçebilir.[^vec-grow] Elemanlar bit düzeyinde taşınır: `clone` çağrılmaz, elemanların `Copy` olması da gerekmez.

**`push` yasağını derleyici koyar.** Ödünç denetleyicisi `Vec`'in yeniden tahsis yaptığını bilmez; yalnızca imzalara bakar. `v[0]` ifadesi `Index::index(&self, ..)` çağrısıdır[^vec-index] ve dönen referans `v`'nin paylaşımlı ödüncüne bağlıdır. `push(&mut self, ..)` ise değiştirilebilir ödünç ister.[^vec-push-kod] Çakışma `E0502` olarak bildirilir.[^e0502-kod] Kısacası güvenlik kütüphanenin içindeki bir denetimden değil, dilin ödünç kurallarından gelir.

**`String`, içeriğinin geçerli UTF-8 olduğu güvence altındaki bir `Vec<u8>`'dir.** Türün tek alanı budur.[^string-kod] `String` ve `str` için `Index` uygulaması, indeks türünün `SliceIndex<str>` trait'ini uygulamasını ister;[^string-index-kod] bu trait bayt aralıkları için uygulanmıştır, `usize` için uygulanmamıştır; yani metinlerde `Index<usize>` yoktur. Bu yüzden `t[0]` çalışma zamanındaki bir denetime takılmaz, bir tür hatası olarak `E0277` verir. Hata mesajındaki öneri de trait'in tanımına eklenmiş özel bir açıklamadır.[^sliceindex] Aralık geçerli bir türdür; ama uçları bir karakterin içine düşerse dilim alma kodu panik yaratır.[^str-panik]

**`HashMap`, SwissTable tasarımına dayanır.** Standart kütüphane, Google'ın SwissTable tablosunun Rust uyarlaması olan `hashbrown`'ı kullanır; varsayılan özet algoritması şimdilik SipHash 1-3'tür ve ileride değişebilir.[^hashmap-kod] Gezinme sırasının değişmesi bilinçli bir savunmadır. Özet algoritması sabit olsaydı bir saldırgan hep aynı yere düşen anahtarlar üretip haritayı yavaşlatabilirdi; buna HashDoS saldırısı denir. Bu yüzden algoritma rastgele bir *tohumla* (seed) başlatılır.

Tohumun nasıl üretildiği sık yanlış anlatılır: Her harita işletim sisteminden yeni rastgele sayı istemez. Anahtar çifti thread başına bir kez alınır ve saklanır; aynı thread'de oluşturulan her yeni `RandomState` bu çiftin bir yarısını bir artırarak kullanır.[^randomstate] Böylece hem haritaların sıraları birbirinden farklı olur hem de çok sayıda harita kurmak yavaşlamaz.

> [!NOTE]
> `Vec<T>`'nin tam tanımında bellek tahsis edicisini belirten ikinci bir tür parametresi vardır. Rust 1.99.0'da bu parametre kararlı değildir; `Allocator` trait'i ve ilişkili API'ler 1.100.0'da kararlı oldu.[^allocator] Günlük kullanımda `Vec<T>` yazmaya devam edersiniz.

## Özet

- `Vec<T>` aynı türden elemanları heap'te yan yana tutar. `v[i]` sınır dışında panikler, `v.get(i)` `None` verir.
- Bir elemana referans yaşarken vektör değiştirilemez (`E0502`), çünkü `push` bütün elemanları başka bir bloğa taşıyabilir.
- `String` büyüyebilen UTF-8 metindir; `len` bayt sayar. Tam sayı indeks yoktur (`E0277`), karakter sınırını tutturmayan dilim panikler. Parçalara `chars` ya da `bytes` ile ulaşılır.
- `+` soldaki `String`'i tüketir, `format!` hiçbir şeyi tüketmez.
- `HashMap` anahtarlarının ve değerlerinin sahibidir (`E0382`). Deyimsel güncelleme `*harita.entry(k).or_insert(0) += 1` biçimindedir. Gezinme sırası belirsizdir; sıra gerekiyorsa sıralayın ya da `BTreeMap` kullanın.

Sıradaki bölüm [8. Hata Yönetimi](08-error-handling.md), bu bölümde birkaç kez karşımıza çıkan paniğin ne olduğunu ve ondan kaçınmanın yolunu, yani `Result` türünü anlatıyor.

## Kaynaklar

[^koleksiyonlar]: The Rust Programming Language, [8. Common Collections](https://doc.rust-lang.org/book/ch08-00-common-collections.html); std, [Koleksiyon seçimi](https://doc.rust-lang.org/std/collections/index.html#when-should-you-use-which-collection).
[^vec-yeni]: The Rust Programming Language, [Creating a New Vector](https://doc.rust-lang.org/book/ch08-01-vectors.html#creating-a-new-vector) ve [Updating a Vector](https://doc.rust-lang.org/book/ch08-01-vectors.html#updating-a-vector).
[^vec-drop]: The Rust Programming Language, [Dropping a Vector Drops Its Elements](https://doc.rust-lang.org/book/ch08-01-vectors.html#dropping-a-vector-drops-its-elements).
[^vec-erisim]: The Rust Programming Language, [Reading Elements of Vectors](https://doc.rust-lang.org/book/ch08-01-vectors.html#reading-elements-of-vectors); std, [`Vec`: Indexing](https://doc.rust-lang.org/std/vec/struct.Vec.html#indexing) ve [`get`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.get).
[^vec-gezme]: The Rust Programming Language, [Iterating Over the Values in a Vector](https://doc.rust-lang.org/book/ch08-01-vectors.html#iterating-over-the-values-in-a-vector).
[^vec-push]: std, [`Vec`: Capacity and reallocation](https://doc.rust-lang.org/std/vec/struct.Vec.html#capacity-and-reallocation) ve [`Vec::push`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.push).
[^vec-e0502]: The Rust Programming Language, [Reading Elements of Vectors](https://doc.rust-lang.org/book/ch08-01-vectors.html#reading-elements-of-vectors); hata dizini, [`E0502`](https://doc.rust-lang.org/error_codes/E0502.html).
[^vec-enum]: The Rust Programming Language, [Using an Enum to Store Multiple Types](https://doc.rust-lang.org/book/ch08-01-vectors.html#using-an-enum-to-store-multiple-types); hata dizini, [`E0308`](https://doc.rust-lang.org/error_codes/E0308.html).
[^dead-code]: The rustc book, [`dead_code`](https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#dead-code); rustc 1.99.0 kaynak kodu, [`dead.rs`, `should_ignore_impl_item`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_passes/src/dead.rs#L438).
[^string-nedir]: The Rust Programming Language, [8.2 Storing UTF-8 Encoded Text with Strings](https://doc.rust-lang.org/book/ch08-02-strings.html); std, [`String`: UTF-8](https://doc.rust-lang.org/std/string/struct.String.html#utf-8).
[^string-push]: The Rust Programming Language, [Appending with `push_str` or `push`](https://doc.rust-lang.org/book/ch08-02-strings.html#appending-with-push_str-or-push).
[^string-arti]: The Rust Programming Language, [Concatenating with `+` or `format!`](https://doc.rust-lang.org/book/ch08-02-strings.html#concatenating-with--or-format); hata dizini, [`E0382`](https://doc.rust-lang.org/error_codes/E0382.html).
[^string-add]: rustc 1.99.0 kaynak kodu, [`string.rs`, `impl Add<&str> for String`](https://github.com/rust-lang/rust/blob/1.99.0/library/alloc/src/string.rs#L2808).
[^string-format]: std, [`format!`](https://doc.rust-lang.org/std/macro.format.html); The Rust Programming Language, [Concatenating with `+` or `format!`](https://doc.rust-lang.org/book/ch08-02-strings.html#concatenating-with--or-format).
[^char-utf8]: std, [`char::len_utf8`](https://doc.rust-lang.org/std/primitive.char.html#method.len_utf8).
[^str-len]: std, [`str::len`](https://doc.rust-lang.org/std/primitive.str.html#method.len); The Rust Programming Language, [Internal Representation](https://doc.rust-lang.org/book/ch08-02-strings.html#internal-representation).
[^string-indeks]: The Rust Programming Language, [Indexing into Strings](https://doc.rust-lang.org/book/ch08-02-strings.html#indexing-into-strings) ve [Internal Representation](https://doc.rust-lang.org/book/ch08-02-strings.html#internal-representation); hata dizini, [`E0277`](https://doc.rust-lang.org/error_codes/E0277.html).
[^string-dilim]: The Rust Programming Language, [Slicing Strings](https://doc.rust-lang.org/book/ch08-02-strings.html#slicing-strings).
[^str-get]: std, [`str::get`](https://doc.rust-lang.org/std/primitive.str.html#method.get).
[^string-gezme]: The Rust Programming Language, [Iterating Over Strings](https://doc.rust-lang.org/book/ch08-02-strings.html#iterating-over-strings); std, [`str::chars`](https://doc.rust-lang.org/std/primitive.str.html#method.chars) ve [`str::bytes`](https://doc.rust-lang.org/std/primitive.str.html#method.bytes).
[^grafem]: The Rust Programming Language, [Bytes, Scalar Values, and Grapheme Clusters](https://doc.rust-lang.org/book/ch08-02-strings.html#bytes-scalar-values-and-grapheme-clusters).
[^maliyet]: std, [Koleksiyon işlemlerinin maliyeti](https://doc.rust-lang.org/std/collections/index.html#cost-of-collection-operations).
[^hashmap-yeni]: The Rust Programming Language, [Creating a New Hash Map](https://doc.rust-lang.org/book/ch08-03-hash-maps.html#creating-a-new-hash-map).
[^hashmap-get]: The Rust Programming Language, [Accessing Values in a Hash Map](https://doc.rust-lang.org/book/ch08-03-hash-maps.html#accessing-values-in-a-hash-map); std, [`HashMap::get`](https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.get).
[^hashmap-sahip]: The Rust Programming Language, [Managing Ownership in Hash Maps](https://doc.rust-lang.org/book/ch08-03-hash-maps.html#managing-ownership-in-hash-maps).
[^hashmap-insert]: The Rust Programming Language, [Overwriting a Value](https://doc.rust-lang.org/book/ch08-03-hash-maps.html#overwriting-a-value); std, [`HashMap::insert`](https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.insert).
[^hashmap-entry]: The Rust Programming Language, [Adding a Key and Value Only If a Key Isn't Present](https://doc.rust-lang.org/book/ch08-03-hash-maps.html#adding-a-key-and-value-only-if-a-key-isnt-present); std, [`HashMap::entry`](https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.entry) ve [`Entry::or_insert`](https://doc.rust-lang.org/std/collections/hash_map/enum.Entry.html#method.or_insert).
[^hashmap-sayma]: The Rust Programming Language, [Updating a Value Based on the Old Value](https://doc.rust-lang.org/book/ch08-03-hash-maps.html#updating-a-value-based-on-the-old-value).
[^hashmap-sira]: std, [`HashMap::iter`](https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.iter).
[^btreemap]: std, [`BTreeMap`](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html) ve [Koleksiyon seçimi: `BTreeMap`](https://doc.rust-lang.org/std/collections/index.html#use-a-btreemap-when).
[^vec-garanti]: std, [`Vec`: Guarantees](https://doc.rust-lang.org/std/vec/struct.Vec.html#guarantees).
[^vec-buyume]: rustc 1.99.0 kaynak kodu, [`raw_vec/mod.rs`, `grow_amortized`](https://github.com/rust-lang/rust/blob/1.99.0/library/alloc/src/raw_vec/mod.rs#L503).
[^vec-grow]: rustc 1.99.0 kaynak kodu, [`raw_vec/mod.rs`, `finish_grow`](https://github.com/rust-lang/rust/blob/1.99.0/library/alloc/src/raw_vec/mod.rs#L544); std, [`GlobalAlloc::realloc`](https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html#method.realloc).
[^vec-index]: rustc 1.99.0 kaynak kodu, [`vec/mod.rs`, `Index<I> for Vec`](https://github.com/rust-lang/rust/blob/1.99.0/library/alloc/src/vec/mod.rs#L3929).
[^vec-push-kod]: rustc 1.99.0 kaynak kodu, [`vec/mod.rs`, `pub fn push(&mut self`](https://github.com/rust-lang/rust/blob/1.99.0/library/alloc/src/vec/mod.rs#L995).
[^e0502-kod]: rustc 1.99.0 kaynak kodu, [`borrowck_errors.rs`, `cannot_reborrow_already_borrowed`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_borrowck/src/borrowck_errors.rs#L189).
[^string-kod]: rustc 1.99.0 kaynak kodu, [`string.rs`, `pub struct String`](https://github.com/rust-lang/rust/blob/1.99.0/library/alloc/src/string.rs#L353); std, [`String`: Representation](https://doc.rust-lang.org/std/string/struct.String.html#representation).
[^string-index-kod]: rustc 1.99.0 kaynak kodu, [`string.rs`, `ops::Index<I> for String`](https://github.com/rust-lang/rust/blob/1.99.0/library/alloc/src/string.rs#L2831).
[^sliceindex]: rustc 1.99.0 kaynak kodu, [`slice/index.rs`, `rustc_on_unimplemented`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/slice/index.rs#L112).
[^str-panik]: rustc 1.99.0 kaynak kodu, [`str/mod.rs`, `slice_error_fail_rt`](https://github.com/rust-lang/rust/blob/1.99.0/library/core/src/str/mod.rs#L84).
[^hashmap-kod]: rustc 1.99.0 kaynak kodu, [`map.rs`, `SipHash 1-3`](https://github.com/rust-lang/rust/blob/1.99.0/library/std/src/collections/hash/map.rs#L26); std, [`HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html).
[^randomstate]: rustc 1.99.0 kaynak kodu, [`random.rs`, `pub struct RandomState`](https://github.com/rust-lang/rust/blob/1.99.0/library/std/src/hash/random.rs#L36).
[^allocator]: Rust 1.100.0 sürüm notları, [Libraries](https://github.com/rust-lang/rust/releases/tag/1.100.0); std 1.99.0, [`Vec`](https://doc.rust-lang.org/std/vec/struct.Vec.html).
