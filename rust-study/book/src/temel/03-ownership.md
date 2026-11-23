# 3. Ownership (Sahiplik)

Her program çalışırken bellek kullanır; işi bitince de o belleği geri vermek zorundadır. Diller bu görevi farklı yollarla çözer. C'de belleği programcı yönetir: `free` çağrısını unutursa bellek sızar, iki kez çağırırsa bellek bozulur, erken çağırırsa silinmiş veriyi kullanır. Java ve Go'da bu işi çöp toplayıcı (garbage collector) üstlenir; karşılığında çalışma zamanında ek yük ve duraklamalar ortaya çıkar.

Rust üçüncü bir yol izler: Belleğin ne zaman geri verileceğini, **derleme zamanında** denetlenen bir kurallar bütünü belirler.[^kurallar] Bu kurallara *sahiplik* (ownership) denir. Kurallardan biri çiğnenirse program derlenmez; çiğnenmediği sürece kuralların denetlenmesi çalışan programa yük bindirmez.

Bu bölümde önce sahipliğin üç kuralını, ardından taşıma, kopyalama ve klonlamayı göreceğiz. Sonra veriyi sahiplenmeden kullanmayı, yani ödünç almayı, son olarak da dilimleri ele alacağız.

> [!NOTE]
> Bu bölümün notları ve çalıştırılabilir kodu `learn/ownership` dizinindedir. Depo kökünden `cargo run -p ownership` komutuyla çalıştırabilirsiniz.

## Günlük hayattan bir benzetme

Elinizde tek bir oyuncak araba olduğunu düşünün. Arabayı arkadaşınıza verirseniz artık onundur; siz oynayamazsınız. Buna *taşıma* (move) denir. Bir kâğıda yazdığınız sayının fotokopisini çekmek ise bedavadır: Sayı ikinizde de kalır. Bu da *kopyalama* (copy) demektir. Büyük bir lego kutusunun aynısını yeniden yapmak mümkündür ama emek ister; bu yüzden kendiliğinden olmaz, açıkça istemeniz gerekir. Buna *klonlama* (clone) denir.

Oyun odasından çıkarken oyuncaklar kendiliğinden toplanır; "topla" demeniz gerekmez. Rust'ta odanın karşılığı *kapsam* (scope), toplamanın karşılığı *düşürme* (drop) işlemidir.

Kitabınızı birine vermeden yalnızca bakmasına izin verebilirsiniz. İsteyen herkes aynı anda bakabilir; ama biri kitabı boyarken başkası bakarsa yarım resim görür, iki kişi birden boyarsa resim bozulur. Bu yüzden aynı anda ya birçok bakan ya da tek bir boyayan olur. Buna *ödünç alma* (borrowing) denir. Defterinizi çöpe atarsanız arkadaşınızın bakacağı bir şey kalmaz; Rust böyle bir duruma baştan izin vermez. *Dilim* (slice) ise pastanın bir parçasını göstermeye benzer: Dilim ayrı bir pasta değildir, pasta gidince dilim de gider.

<details>
<summary>Bu bölümde geçen terimler</summary>

- **sahip (owner):** Veriyi elinde tutan ve onu düşürmekle yükümlü olan değişken. Her verinin tek bir sahibi vardır.
- **taşıma (move):** Verinin sahipliğini başka bir değişkene devretmek. Devreden değişken artık kullanılamaz.
- **kopyalama (copy):** `Copy` trait'ini uygulayan bir değerin bitlerini çoğaltmak. İki değişken de geçerli kalır.
- **klonlama (clone):** Bir değerin bağımsız bir eşini `.clone()` çağrısıyla açıkça üretmek.
- **düşürme (drop):** Değerin ömrünün sona ermesi ve tuttuğu kaynağın geri verilmesi.
- **ödünç alma (borrow):** Sahibi değiştirmeden veriyi bir referans üzerinden kullanmak.
- **referans (`&T`, `&mut T`):** Verinin kendisi değil, nerede durduğunu gösteren işaretçi. Geçerliliği derleyici tarafından denetlenir.
- **kapsam (scope):** Bir ismin geçerli olduğu program bölümü; çoğunlukla süslü parantezler arasıdır.
- **stack ve heap:** Belleğin iki bölgesi. Stack, boyutu derleme zamanında bilinen değerler için hızlı bir alandır. Heap, boyutu çalışma zamanında belli olan veriler içindir; yer ayırmak ve geri vermek gerekir.

</details>

## Sahipliğin üç kuralı

Bölümün geri kalanı şu üç kuralın sonuçlarından ibarettir:[^kurallar]

- Rust'ta her değerin bir *sahibi* vardır.
- Aynı anda yalnızca bir sahip olabilir.
- Sahip kapsam dışına çıktığında değer düşürülür.

Kuralların neden böyle olduğunu görmek için heap üzerinde veri tutan bir türe, `String`'e bakacağız. Bir `String` üç parçadan oluşur: heap'teki metni gösteren bir işaretçi, metnin uzunluğu ve ayrılmış kapasite. Bu üç parça stack'te, metnin kendisi heap'te durur.[^string]

## Taşıma (move)

Bir `String`'i başka bir değişkene atadığınızda sahiplik yeni değişkene geçer:

```rust
fn main() {
    let s1 = String::from("merhaba");
    let s2 = s1; // sahiplik s2'ye geçti

    println!("s2 = {s2}");
}
```

Çıktı:

```text
s2 = merhaba
```

Atama sırasında yalnızca stack'teki üç parça (işaretçi, uzunluk, kapasite) kopyalanır; heap'teki metin yerinde kalır. Bu yüzden taşıma ucuzdur. Ne var ki artık aynı heap bloğunu gösteren iki işaretçi vardır. İkisi de geçerli sayılsaydı kapsam sonunda ikisi de aynı bloğu geri vermeye çalışırdı. *Çifte serbest bırakma* (double free) denen bu hata belleği bozar ve güvenlik açıklarına yol açabilir. Rust sorunu `s1`'i geçersiz sayarak çözer: Bloğu artık yalnızca `s2` geri verecektir.[^tasima]

Geçersiz kalan değişkeni kullanmaya çalışırsanız program derlenmez:

```rust,compile_fail,E0382
fn main() {
    let s1 = String::from("merhaba");
    let s2 = s1;
    println!("{s1}, {s2}");
}
```

Derleyicinin yanıtı:

```text
error[E0382]: borrow of moved value: `s1`
  |
2 |     let s1 = String::from("merhaba");
  |         -- move occurs because `s1` has type `String`, which does not implement the `Copy` trait
3 |     let s2 = s1;
  |              -- value moved here
4 |     println!("{s1}, {s2}");
  |                ^^ value borrowed here after move
```

Mesaj üç şeyi birden söyler: Taşımanın nerede gerçekleştiğini (`value moved here`), değerin sonradan nerede kullanıldığını ve taşımanın nedenini. Neden, `String`'in `Copy` trait'ini uygulamamasıdır. Bu trait sonraki başlığın konusudur.

## Kopyalama: `Copy` trait'i

Tam sayılarla aynı kodu yazdığınızda sonuç değişir:

```rust
fn main() {
    let x = 5;
    let y = x; // bit bit kopyalanır

    println!("x = {x}, y = {y}");
}
```

Çıktı:

```text
x = 5, y = 5
```

Burada `x` geçerliliğini yitirmez, çünkü `i32` türü `Copy` trait'ini uygular. Ölçüt türün boyutu ya da stack'te durması değil, bu trait'tir: Tam ve kayan noktalı sayılar, `bool`, `char`, paylaşımlı referanslar (`&T`), fonksiyon işaretçileri ve yalnızca `Copy` öğelerden oluşan diziler, demetler ve `Option` değerleri kopyalanır.[^copy]

`String` ile `Vec<T>` ise heap'teki bir bloğun sahibidir ve bloğu geri vermek için `Drop`'a ihtiyaç duyar. Bitlerini kopyalamak aynı bloğa ikinci bir sahip yaratacağından bu türler `Copy` olamaz. Aynı engel onları içeren yapılar için de geçerlidir:

```rust,compile_fail,E0204
#[derive(Clone, Copy)]
struct Kutu {
    ad: String,
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0204]: the trait `Copy` cannot be implemented for this type
```

Böylece iki davranış birbirini dışlar: Bir türün değeri ya kopyalanır ya da taşınır.

## Klonlama: `clone`

İki bağımsız `String` gerçekten gerekiyorsa heap'teki verinin de çoğaltılması gerekir. Bu iş `.clone()` ile yapılır:

```rust
fn main() {
    let s1 = String::from("ada");
    let s2 = s1.clone(); // heap'teki metin de çoğaltıldı

    println!("s1 = {s1}, s2 = {s2}");
}
```

Çıktı:

```text
s1 = ada, s2 = ada
```

Derin kopya pahalı olabilir: Yeni bir heap bloğu ayırır ve içeriği oraya kopyalar. Otomatik yapılsaydı bedelini fark etmeden öderdiniz. Bu yüzden ucuz olan taşıma varsayılandır, pahalı olan klonlama ise kodda `.clone()` olarak görünür.[^clone]

## Kapsam ve `Drop`

Bir değişken kapsam dışına çıktığında Rust, `drop` fonksiyonunu kendiliğinden çağırır ve kaynak geri verilir.[^drop] Sizin "sil" demeniz gerekmez; unutma ihtimali de yoktur:

```rust
fn main() {
    {
        let s = String::from("geçici");
        println!("içeride: {s}");
    } // s burada düşürülür, heap'teki blok geri verilir

    println!("dışarıda s diye bir şey yok");
}
```

Çıktı:

```text
içeride: geçici
dışarıda s diye bir şey yok
```

Kaynağın ömrünü bir değerin ömrüne bağlayan bu yaklaşım C++ dünyasında RAII (*Resource Acquisition Is Initialization*) adıyla bilinir. Değişkenler tanımlandıkları sıranın **tersine** düşürülür;[^sira] bunu `Drop`'u elle uygulayan küçük bir türle gözlemleyebiliriz:

```rust
struct Gozcu(&'static str);

impl Drop for Gozcu {
    fn drop(&mut self) {
        println!("{} düşürüldü", self.0);
    }
}

fn main() {
    let _ilk = Gozcu("ilk");
    let _ikinci = Gozcu("ikinci");
    println!("main bitiyor");
}
```

Çıktı:

```text
main bitiyor
ikinci düşürüldü
ilk düşürüldü
```

> [!NOTE]
> "Sıfır maliyet" sözü yalnızca sahiplik ve ödünç **denetimleri** için geçerlidir: Bunlar derleme zamanında yapılır. Heap'teki bloğu geri vermek ise çalışma zamanında gerçekleşen gerçek bir iştir. Rust'ın kazancı, bu işin çöp toplayıcı duraklaması olmadan ve öngörülebilir bir anda yapılmasıdır.

## Paylaşımlı ödünç: `&T`

Bir fonksiyona `String` verdiğinizde sahipliği de devretmiş olursunuz. Değeri yalnızca okutmak istiyorsanız fonksiyonun onu geri döndürmesi gerekir; bu da her çağrıda gereksiz bir al-ver demektir. Referanslar bu zahmeti ortadan kaldırır. `&` işareti bir değeri sahiplenmeden kullanmayı sağlar ve bu işleme *ödünç alma* denir:[^odunc]

```rust
fn main() {
    let s = String::from("kitap");
    let n = uzunluk_hesapla(&s); // yalnızca ödünç verildi

    println!("'{s}' uzunluğu {n}");
}

fn uzunluk_hesapla(x: &String) -> usize {
    x.len()
}
```

Çıktı:

```text
'kitap' uzunluğu 5
```

`uzunluk_hesapla` çağrısından sonra `s` hâlâ kullanılabilir, çünkü sahiplik hiç el değiştirmemiştir. Fonksiyon bittiğinde `x` kapsam dışına çıkar; ama `x` verinin sahibi olmadığı için hiçbir şey düşürülmez.

> [!TIP]
> Gerçek kodda bu parametreyi `&String` yerine `&str` yazmak daha kullanışlıdır. Nedenini [bir sonraki sayfada](03-ownership-derinlesme.md) göreceğiz.

## Değiştirilebilir ödünç: `&mut T`

Paylaşımlı referans üzerinden `String`'e ekleme yapamazsınız. Değiştirmek için `&mut` ile ödünç almak, bunun için de değişkeni `mut` olarak tanımlamak gerekir:

```rust
fn main() {
    let mut s = String::from("merhaba");
    ekle_dunya(&mut s);

    println!("{s}");
}

fn ekle_dunya(x: &mut String) {
    x.push_str(" dünya");
}
```

Çıktı:

```text
merhaba dünya
```

### Ödünç almanın kuralı

Referanslar iki kurala uyar:[^referans]

- Belirli bir anda **ya** istediğiniz kadar paylaşımlı referans (`&T`) **ya da** tek bir değiştirilebilir referans (`&mut T`) olabilir.
- Referanslar her zaman geçerli olmalıdır.

Birinci kural kısaca *aliasing XOR mutability* diye anılır: Veri ya paylaşılır ya da değiştirilir, ikisi birden olmaz. Aşağıdaki kod kuralı çiğner:

```rust,compile_fail,E0502
fn main() {
    let mut s = String::from("merhaba");
    let a = &s;
    let b = &mut s;
    println!("{a} {b}");
}
```

Derleyicinin yanıtı:

```text
error[E0502]: cannot borrow `s` as mutable because it is also borrowed as immutable
  |
3 |     let a = &s;
  |             -- immutable borrow occurs here
4 |     let b = &mut s;
  |             ^^^^^^ mutable borrow occurs here
5 |     println!("{a} {b}");
  |                - immutable borrow later used here
```

Bu kod derlenseydi `a` üzerinden okuyan kişi, `b` üzerinden yapılan değişikliği ayağının altında bulurdu. Daha kötüsü, `b` ile yapılan bir ekleme metni heap'te başka bir yere taşıyabilir ve `a` artık var olmayan bir belleği gösterebilirdi. Kuralın tek thread'li programda da geçerli olmasının nedeni budur. Birden çok thread söz konusu olduğunda aynı kural *veri yarışlarını* (data race) da derleme zamanında engeller.[^yaris]

İki değiştirilebilir referans da aynı anda var olamaz:

```rust,compile_fail,E0499
fn main() {
    let mut s = String::from("merhaba");
    let a = &mut s;
    let b = &mut s;
    a.push('!');
    b.push('?');
}
```

Derleyicinin yanıtı:

```text
error[E0499]: cannot borrow `s` as mutable more than once at a time
```

### Ödünç ne zaman biter?

Bir ödünç, bloğun sonunda değil, referansın **son kullanıldığı** yerde biter.[^nll] Bu davranışa *non-lexical lifetimes* (NLL) denir. Aşağıdaki programda `r1` ile `r2` bir daha kullanılmadığı için `r3` sorunsuz oluşturulur:

```rust
fn main() {
    let mut s = String::from("merhaba dünya");

    let r1 = &s;
    let r2 = &s; // birden çok paylaşımlı referans serbesttir
    println!("{r1} / {r2}"); // son kullanım: iki ödünç burada biter

    let r3 = &mut s;
    r3.push('!');
    println!("{r3}");
}
```

Çıktı:

```text
merhaba dünya / merhaba dünya
merhaba dünya!
```

`E0502` aldığınızda ilk bakacağınız yer burasıdır: Eski referansı bu satırdan sonra hâlâ kullanıyor musunuz?

## Sarkan referanslar

Gösterdiği veri yok olduğu hâlde ortada duran referansa *sarkan referans* (dangling reference) denir. C ve C++'ta yerel bir değişkenin adresini döndürmek bu hatanın klasik örneğidir: Fonksiyon dönünce o yer geçersizleşir ve program orada bulduğu rastgele baytları veri sanır. Rust bu kodu derlemez:

```rust,compile_fail,E0106
fn sarkan() -> &String {
    let s = String::from("defter");
    &s
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0106]: missing lifetime specifier
  |
1 | fn sarkan() -> &String {
  |                ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but there is no value for it to be borrowed from
```

Yardım satırı sorunun özünü söyler: Dönüş türünde ödünç bir değer vardır, ama ödünç alınabileceği bir girdi yoktur. Ömrü (lifetime) elle yazmak da durumu kurtarmaz; bu kez derleyici yerel değişkene referans döndürüldüğünü bildirir:

```rust,compile_fail,E0515
fn sarkan<'a>() -> &'a String {
    let s = String::from("defter");
    &s
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error[E0515]: cannot return reference to local variable `s`
  |
3 |     &s
  |     ^^ returns a reference to data owned by the current function
```

Çözüm, referans yerine değerin kendisini döndürmektir. Böylece sahiplik çağırana taşınır ve ortada sarkacak bir şey kalmaz:[^sarkan]

```rust
fn main() {
    let s = sarkmayan();
    println!("{s}");
}

fn sarkmayan() -> String {
    String::from("defter")
}
```

Çıktı:

```text
defter
```

Aynı sorun tek bir fonksiyonun içinde de çıkabilir. Burada referans, gösterdiği değerden uzun yaşamaya çalışır ve derleyici `E0597` verir:

```rust,compile_fail,E0597
fn main() {
    let r;
    {
        let s = String::from("kısa ömürlü");
        r = &s;
    } // s burada düşürülür
    println!("{r}");
}
```

Derleyicinin yanıtı:

```text
error[E0597]: `s` does not live long enough
```

## Dilimler

*Dilim* (slice), bir verinin ardışık bir bölümüne verilen referanstır; verinin kopyası değildir. İçinde yalnızca bir başlangıç işaretçisi ve bir uzunluk taşır. Metin dilimlerinin türü `&str`, dizi dilimlerinin türü `&[T]`'dir:[^dilim]

```rust
fn main() {
    let s = String::from("merhaba dünya");
    let ilk = ilk_kelime(&s);
    println!("ilk kelime: {ilk}");

    let dizi = [1, 2, 3, 4, 5];
    let orta = &dizi[1..4];
    println!("orta = {orta:?}");
}

fn ilk_kelime(s: &str) -> &str {
    match s.find(' ') {
        Some(i) => &s[..i],
        None => s,
    }
}
```

Çıktı:

```text
ilk kelime: merhaba
orta = [2, 3, 4]
```

`ilk_kelime` bir `&str` beklediği hâlde ona `&String` verebildik. Derleyici `&String`'i kendiliğinden `&str`'ye dönüştürür; bu dönüşüme *deref coercion* denir ve [24. bölümde](../ileri/24-deref-metot.md) ayrıntılı olarak ele alınır.

Dilim yerine "başlangıç ve uzunluk" sayılarını ayrı değişkenlerde tutabilirdik. Ama asıl metin değiştiğinde bu sayılar sessizce geçersizleşir ve program yanlış sonuç üretirdi. Dilim ise sahibine bağlı bir referanstır: Dilim yaşadığı sürece sahibini değiştiremezsiniz.

```rust,compile_fail,E0502
fn main() {
    let mut s = String::from("merhaba dünya");
    let w = ilk_kelime(&s);
    s.clear();
    println!("{w}");
}
# fn ilk_kelime(s: &str) -> &str { s.split(' ').next().unwrap_or(s) }
```

Derleyicinin yanıtı:

```text
error[E0502]: cannot borrow `s` as mutable because it is also borrowed as immutable
```

`clear` metodu `s`'i değiştirmek için `&mut` ister; oysa `w` hâlâ `s`'den ödünç alınmış durumdadır. Derleyici böylece, sessizce yanlış sonuç üretecek bir programı çalışmadan önce durdurur.

## Hata kodu başvurusu

| Kod | Ne zaman görülür? | Ne yapmalı? |
| --- | --- | --- |
| `E0382` | Taşınmış bir değer kullanıldığında | Yeni sahibi kullanın, ödünç verin (`&`) ya da gerçekten iki değer gerekiyorsa klonlayın. |
| `E0204` | `Copy` olmayan alanı bulunan türe `Copy` türetildiğinde | `Copy`'yi kaldırın; `Clone` çoğu zaman yeterlidir. |
| `E0499` | Aynı anda iki `&mut` alındığında | İlk ödüncün son kullanımını öne alın ya da veriyi ayrık parçalara bölün. |
| `E0502` | `&` ile `&mut` çakıştığında | Paylaşımlı referansın işini bitirip sonra değiştirin. |
| `E0106` | İmzada referansın ömrü belirlenemediğinde | Girdi referansına bağlayın ya da sahipli değer döndürün. |
| `E0515` | Yerel değişkene referans döndürüldüğünde | Değerin kendisini döndürün. |
| `E0597` | Referans, gösterdiği değerden uzun yaşadığında | Değeri daha geniş bir kapsamda tanımlayın. |

Herhangi bir hata kodunun uzun açıklamasını terminalde görebilirsiniz:

```console
rustc --explain E0382
```

## Perde arkası: Derleyici ne yapıyor?

> [!NOTE]
> Bu başlık ilk okuyuşta atlanabilir; yukarıdaki kuralları anlamış olmanız yeterlidir.

Sahiplik kuralları, okula girişte çantalara bakan bir nöbetçi gibi yalnızca girişte, yani derleme zamanında denetlenir. İçeri girdikten sonra kimse sizi durdurmaz: Çalışan programda referans yalnızca bir işaretçidir. Denetim, kaynak kodun sadeleştirilmiş ara biçimlerinden biri olan MIR üzerinde yapılır.

**Taşımaların izlenmesi.** MIR'de her kullanım ya `Copy` ya da `Move` işlenenidir.[^mir-move] Taşınan yer "başlatılmamış" sayılır. Bir noktada hangi yerlerin başlatılmış olabileceğini `MaybeInitializedPlaces` veri akışı analizi hesaplar;[^mir-init] taşınmış bir yerin kullanılması `E0382` hatasına dönüşür.[^e0382]

**Ödünçlerin denetlenmesi.** Ödünç denetleyicisi (borrow checker) programdaki ödünçleri bir `BorrowSet` içinde toplar[^borrowset] ve her erişimde, erişilen yerin etkin bir ödünçle çakışıp çakışmadığına `places_conflict` ile bakar.[^conflict] Ödünçlerin ne kadar süreceği NLL bölge çözümlemesiyle hesaplanır.[^nll-kod] `E0499`, `E0502`, `E0515` ve `E0597`, aynı kuralın bu aşamada yakalanan farklı yüzleridir.

**Düşürme çağrılarının eklenmesi.** `drop` çağrılarını siz yazmazsınız; MIR kurulurken düşürme gerekebilecek her yere bir `Drop` sonlandırıcısı konur. *Drop elaboration* geçişi bunları, yalnızca değer gerçekten başlatılmışsa yıkıcıyı çalıştıracak biçimde düzenler.[^elaborate] Bir değerin taşınıp taşınmadığı koşula bağlıysa bu bilgi, stack'te tutulan bir *drop bayrağında* (drop flag) saklanır.[^bayrak]

## Özet

- Her değerin tek bir sahibi vardır ve sahip kapsam dışına çıkınca değer düşürülür. Tek sahip, tek serbest bırakma demektir.
- Atama, `Copy` olmayan türlerde sahipliği taşır; `Copy` türlerde bitleri kopyalar. Derin kopya yalnızca `.clone()` ile, açıkça yapılır.
- Referanslar sahiplik almadan kullanım sağlar: Aynı anda ya birçok `&T` ya da tek bir `&mut T`. Ödünç, son kullanımda biter.
- Referans ve dilimler asla sarkmaz; derleyici sahibin onlardan uzun yaşamasını zorunlu kılar.

Sıradaki sayfa [3.1 Ownership: Derinleşme](03-ownership-derinlesme.md), bu kuralların gerçek kodda nasıl kullanıldığını gösteriyor: Parametre türü seçmek, kısmi taşıma, bir kullanıcı deposu tasarımı ve en sık karşılaşılan beş hata.

## Kaynaklar

[^kurallar]: The Rust Programming Language, [4.1 What Is Ownership?](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html) ve [Ownership Rules](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html#ownership-rules).
[^string]: The Rust Programming Language, [Variables and Data Interacting with Move](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html#variables-and-data-interacting-with-move); std, [`String`: Representation](https://doc.rust-lang.org/std/string/struct.String.html#representation).
[^tasima]: The Rust Programming Language, [Variables and Data Interacting with Move](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html#variables-and-data-interacting-with-move); hata dizini, [`E0382`](https://doc.rust-lang.org/error_codes/E0382.html).
[^copy]: std, [`Copy` trait'i](https://doc.rust-lang.org/std/marker/trait.Copy.html); The Rust Reference, [Special types and traits: `Copy`](https://doc.rust-lang.org/reference/special-types-and-traits.html#copy); hata dizini, [`E0204`](https://doc.rust-lang.org/error_codes/E0204.html).
[^clone]: The Rust Programming Language, [Variables and Data Interacting with Clone](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html#variables-and-data-interacting-with-clone); std, [`Clone` trait'i](https://doc.rust-lang.org/std/clone/trait.Clone.html).
[^drop]: The Rust Programming Language, [Memory and Allocation](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html#memory-and-allocation); std, [`Drop` trait'i](https://doc.rust-lang.org/std/ops/trait.Drop.html).
[^sira]: The Rust Reference, [Destructors: Drop scopes](https://doc.rust-lang.org/reference/destructors.html#drop-scopes).
[^odunc]: The Rust Programming Language, [4.2 References and Borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html).
[^referans]: The Rust Programming Language, [The Rules of References](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html#the-rules-of-references); hata dizini, [`E0499`](https://doc.rust-lang.org/error_codes/E0499.html) ve [`E0502`](https://doc.rust-lang.org/error_codes/E0502.html).
[^yaris]: The Rust Programming Language, [Mutable References](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html#mutable-references); The Rustonomicon, [Data Races and Race Conditions](https://doc.rust-lang.org/nomicon/races.html).
[^nll]: The Rust Programming Language, [Mutable References](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html#mutable-references); [RFC 2094: Non-lexical lifetimes](https://rust-lang.github.io/rfcs/2094-nll.html).
[^sarkan]: The Rust Programming Language, [Dangling References](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html#dangling-references); hata dizini, [`E0106`](https://doc.rust-lang.org/error_codes/E0106.html), [`E0515`](https://doc.rust-lang.org/error_codes/E0515.html) ve [`E0597`](https://doc.rust-lang.org/error_codes/E0597.html).
[^dilim]: The Rust Programming Language, [4.3 The Slice Type](https://doc.rust-lang.org/book/ch04-03-slices.html); std, [`slice` temel türü](https://doc.rust-lang.org/std/primitive.slice.html).
[^mir-move]: rustc 1.99.0 kaynak kodu, [`syntax.rs`, `Move(Place<'tcx>)`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_middle/src/mir/syntax.rs#L1306).
[^mir-init]: rustc 1.99.0 kaynak kodu, [`initialized.rs`, `MaybeInitializedPlaces`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_mir_dataflow/src/impls/initialized.rs#L139) ve [`move_paths/mod.rs`, `MoveData`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_mir_dataflow/src/move_paths/mod.rs#L176).
[^e0382]: rustc 1.99.0 kaynak kodu, [`borrowck_errors.rs`, `cannot_act_on_moved_value`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_borrowck/src/borrowck_errors.rs#L321).
[^borrowset]: rustc 1.99.0 kaynak kodu, [`borrow_set.rs`, `BorrowSet`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_borrowck/src/borrow_set.rs#L21).
[^conflict]: rustc 1.99.0 kaynak kodu, [`places_conflict.rs`, `places_conflict`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_borrowck/src/places_conflict.rs#L81).
[^nll-kod]: rustc 1.99.0 kaynak kodu, [`nll.rs`, `compute_regions`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_borrowck/src/nll.rs#L113).
[^elaborate]: rustc 1.99.0 kaynak kodu, [`elaborate_drops.rs`, `ElaborateDrops`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_mir_transform/src/elaborate_drops.rs#L53).
[^bayrak]: The Rustonomicon, [Drop Flags](https://doc.rust-lang.org/nomicon/drop-flags.html).
