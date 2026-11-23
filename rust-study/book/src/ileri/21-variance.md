# 21. Alt Tür İlişkisi ve Varyans

`&str` bekleyen bir fonksiyona `"merhaba"` gibi bir metin sabiti verebilirsiniz. Sabitin türü `&'static str`'dir, fonksiyonun beklediği ise daha kısa ömürlü bir referanstır; türler birebir aynı olmadığı hâlde program derlenir. Buna karşılık `&mut Vec<&'static str>` türündeki bir değeri `&mut Vec<&'a str>` beklenen yere vermeye kalktığınızda derleyici, ilk bakışta anlaşılması güç bir ömür hatası verir.

İki davranışın ardında aynı düzenek vardır. *Alt tür ilişkisi* (subtyping), hangi ömrün hangisinin yerine geçebileceğini söyler. *Varyans* (variance) ise bu esnekliğin `&T`, `&mut T`, `Vec<T>` gibi türlerin içine taşınıp taşınmadığını belirler.[^nomicon] Ömür hatalarının görünmeyen nedeni çoğu zaman budur: Sorun yazdığınız satırda değil, kullandığınız türün bu konudaki davranışındadır.

Bu bölümde önce ömürler arasındaki alt tür ilişkisini, ardından varyansın üç biçimini göreceğiz: kovaryans, invaryans ve kontravaryans. Yol boyunca `&'a mut &'a str` kalıbının neden sorun çıkardığını ve ömürleri ikiye ayırmanın bunu nasıl çözdüğünü inceleyeceğiz. *The Rust Programming Language* bu konuyu ayrıntılı işlemez; ayrıntılar *The Rustonomicon* ile *The Rust Reference*'tadır.

> [!NOTE]
> Bu bölümün notları ve çalıştırılabilir kodu `learn/advanced-concepts/ileri-variance` dizinindedir. Depo kökünden `cargo run -p ileri-variance` komutuyla çalıştırabilirsiniz. Bölüm, ödünç almayı ([3. bölüm](../temel/03-ownership.md)) ve ömürleri ([9. bölüm](../orta/09-generics-traits-lifetimes.md)) bildiğinizi varsayar.

## Günlük hayattan bir benzetme

Lunaparkta iki tür bilet olduğunu düşünün: bir günlük bilet ve bütün yıl geçerli kart. Kapıdaki görevli "bugün geçerli bir bilet gösterin" derse yıllık kartı olan da içeri girer, çünkü kart bugünü de kapsar. Tersi olmaz: "Yıl sonuna kadar geçerli olsun" diyen görevli günlük bileti kabul etmez. Uzun geçerli olanın, kısa geçerli olanın istendiği yerde kullanılabilmesine *alt tür ilişkisi* denir.

Müzedeki cam vitrine yalnızca bakabilirsiniz. İçinde çok dayanıklı bir eşya duran vitrini, "az dayanıklı eşya vitrini" bekleyen bir salona rahatça koyabilirsiniz; bakan kişi için eşyanın fazladan dayanıklı olması zarar vermez. Paylaşımlı referans böyle bir vitrindir ve bu esnekliğe *kovaryans* denir.

Şimdi üstünde "yalnızca yıllık kart" yazan bir kutunuz olsun; içinden çıkan her şeyin yıllık kart olduğuna güveniyorsunuz. Kutuyu birine "günlük bilet kutusu" diye verirseniz içine günlük bilet atar. Yarın kutuyu açtığınızda elinize geçeni yıllık kart sanırsınız, oysa süresi dolmuştur. Bu yüzden içine yazılabilen bir kutunun etiketi değiştirilemez; buna *invaryans* denir. Defterlerde de durum aynıdır. Küçük bir defter isteyen kişiye büyük defter verebilirsiniz, işini görür; ama küçük defteri "bunu büyük say" diye veremezsiniz. Üstüne yazı yazılan defterlerde ise hiçbir değiş tokuş yapılmaz, yoksa tutulan notlar yanlış yere yazılır.

Son olarak görevlileri düşünün. "Hangi bileti getirirseniz getirin, kabul ederim" diyen bir görevli, "yalnızca yıllık kart kabul eden görevli" aranan yere rahatça konabilir. Burada yön tersine dönmüştür: Daha azını şart koşan, daha çok işe yarar. Buna *kontravaryans* denir.

<details>
<summary>Bu bölümde geçen terimler</summary>

- **referans (`&` / `&mut`):** Veriyi gösteren işaretçi. `&` yalnızca okur, `&mut` değiştirebilir.
- **ödünç alma (borrowing):** Sahibi değiştirmeden veriye bir süreliğine referansla ulaşmak.
- **ömür (lifetime, `'a`):** Bir referansın geçerli olduğu kod bölgesinin adı. Hiçbir şeyin ömrünü uzatmaz; yalnızca süreyi etiketler.
- **`'static`:** Programın sonuna kadar geçerli olabilen referansın ömrü. Koda yazılmış metin sabitleri böyledir.
- **sabit (literal):** Koda doğrudan yazılmış değer: `"hello"`, `5`, `true`.
- **alt tür (subtype):** Beklenen türün yerine geçebilen, en az onun kadar işe yarayan tür.
- **üst tür (supertype):** Tersi: daha azını vaat eden, daha genel tür.
- **varyans (variance):** "İçteki türün yerine alt türü koyarsam dıştaki tür de yerine geçer mi?" sorusunun yanıtı.
- **kovaryant (covariant):** Evet, aynı yönde geçer. `&'a T` böyledir.
- **invaryant (invariant):** Hayır, tam aynı tür gerekir. `&mut T`, `T` için böyledir.
- **kontravaryant (contravariant):** Ters yönde geçer. Yalnızca fonksiyon argümanlarında görülür: `fn(T)`.
- **jenerik parametre (generic parameter):** Türün içindeki boşluk: `Vec<T>`'deki `T`, `&'a str`'deki `'a`.
- **sarkan referans (dangling reference):** Gösterdiği veri artık geçerli olmayan referans.
- **`Cell` / `RefCell`:** `&` üzerinden bile içeriğinin değişmesine izin veren türler.
- **`PhantomData`:** Yer kaplamayan, derleyiciye "bu türü taşıyormuşum gibi davran" diyen alan.
- **ödünç denetleyicisi (borrow checker):** Ödünç alma kurallarını derleme sırasında denetleyen derleyici parçası.
- **iyi biçimlilik (well-formedness):** Bir türün anlamlı olabilmesi için kendiliğinden gereken koşullar.

</details>

## Alt tür ilişkisi: uzun ömür kısanın yerine geçer

Bir `A` türü, `B` türünün beklendiği her yerde kullanılabiliyorsa `A`, `B`'nin *alt türüdür*. Rust'ta bu ilişki struct'lar ya da trait'ler arasında değil, ömürler arasında kurulur; ömürler dışında yalnızca `for<'a> fn(&'a str)` gibi yüksek dereceli (higher-ranked) türlerde görülür.[^alt-tur] Ömürlerde kural şudur: `'uzun`, `'kisa`'yı kapsıyorsa (`'uzun: 'kisa` yazılır) `'uzun`, `'kisa`'nın alt türüdür.[^omur-siniri] Uç örnek `'static`'tir; her ömrü kapsadığı için her ömrün alt türüdür.

```rust
fn kisa_bekler(x: &str) -> usize {
    x.len()
}

fn main() {
    let sabit: &'static str = "program boyu yaşarım";
    println!("sabit: {}", kisa_bekler(sabit));

    let gecici = String::from("günlük bilet");
    println!("geçici: {}", kisa_bekler(&gecici));
}
```

Çıktı:

```text
sabit: 22
geçici: 14
```

`kisa_bekler`, çağrı süresince geçerli bir referansla yetinir. `&'static str` bu isteği fazlasıyla karşılar ve sessizce daha kısa ömürlü bir referans gibi kullanılır. Bu ilişki olmasaydı her fonksiyona tam aynı süreli bir referans vermeniz gerekirdi; `'static` bir metni sıradan bir `&str` bekleyen fonksiyona bile veremezdiniz.

Tersi ise geçerli değildir. Program boyunca geçerli bir referans isteyen fonksiyona yerel bir değişkenin referansı verilemez:

```rust,compile_fail,E0597
fn uzun_bekler(x: &'static str) -> usize {
    x.len()
}

fn main() {
    let gecici = String::from("günlük bilet");
    uzun_bekler(&gecici);
}
```

Derleyicinin yanıtı:

```text
error[E0597]: `gecici` does not live long enough
  |
6 |     let gecici = String::from("günlük bilet");
  |         ------ binding `gecici` declared here
7 |     uzun_bekler(&gecici);
  |     ------------^^^^^^^-
  |     |           |
  |     |           borrowed value does not live long enough
  |     argument requires that `gecici` is borrowed for `'static`
8 | }
  | - `gecici` dropped here while still borrowed
```

Mesajdaki `argument requires that ... is borrowed for 'static` satırı nedeni açıkça söyler: İmza `'static` istemiştir, `gecici` ise `main`'in sonunda düşer. Çözüm ya imzayı gevşetmek ya da gerçekten program boyunca yaşayan bir veri vermektir.

## Varyans: içteki esneklik dışa yansır mı?

Tek bir referansın ömrünü kısaltmak kolaydır. Asıl soru, referans başka bir türün *içindeyken* ne olacağıdır: `&'static str` yerine `&'a str` kullanılabiliyorsa `Vec<&'static str>` de `Vec<&'a str>` yerine kullanılabilir mi? `&mut Vec<&'static str>` için yanıt ne olur? Bir türün jenerik parametresine göre verdiği bu yanıta *varyans* denir ve üç biçimi vardır:[^varyans]

- **Kovaryant:** İçteki alt tür ilişkisi dışa aynı yönde taşınır. `'uzun`, `'kisa`'nın alt türüyse `&'uzun T` de `&'kisa T`'nin alt türüdür.
- **İnvaryant:** Hiçbir yönde taşınmaz; tam aynı tür gerekir.
- **Kontravaryant:** Ters yönde taşınır.

Varyansı siz yazmazsınız. Derleyici onu türün alanlarına bakarak çıkarır; nasıl yaptığını bölümün sonunda göreceğiz.

## Kovaryans: yalnız okunan yer esnektir

Paylaşımlı referans `&'a T`, hem `'a` hem `T` için kovaryanttır. Yalnız okunan bir veride fazladan uzun yaşamak kimseye zarar vermez, o yüzden derleyici bu esnekliği kendiliğinden tanır:[^varyans]

```rust
fn ikisi<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

fn oku(v: &Vec<&str>) -> usize {
    v.len()
}

fn main() {
    let yerel = String::from("kısa ömürlü");
    let r = ikisi("program boyu yaşayan", &yerel);
    println!("{r}");

    let depo: Vec<&'static str> = vec!["a", "b", "c"];
    println!("öğe sayısı: {}", oku(&depo));
}
```

Çıktı:

```text
program boyu yaşayan
öğe sayısı: 3
```

`ikisi`, iki argümanın da aynı `'a` ömrünü taşımasını ister. Derleyici `'static` metni `yerel`'in süresine indirir ve iki argüman ortak bir ömürde buluşur. `oku` çağrısında ise `'static` daha derindedir: `Vec<&'static str>`'nin içindedir. `&T` ile `Vec<T>` `T` için kovaryant olduğundan kısalma iç içe geçmiş iki katmandan da geçer.

Kovaryans yalnız referanslara özgü değildir. İçindeki değerin sahibi olan `Box<T>` ve `Vec<T>` de kovaryanttır. Aşağıda `'static` metin taşıyan bir kutu, daha kısa ömürlü metin taşıyan bir kutu olarak kullanılır:

```rust
fn kisalt<'kisa>(kutu: Box<&'static str>) -> Box<&'kisa str> {
    kutu // Box<T>, T için kovaryanttır
}

fn main() {
    let yerel = String::from("yerel");
    let mut liste = vec![*kisalt(Box::new("sabit"))];
    liste.push(&yerel);
    println!("{liste:?}");
}
```

Çıktı:

```text
["sabit", "yerel"]
```

Kutuya sonradan kısa ömürlü bir metin konsa bile kimse zarar görmez: Eski kutunun sahibi artık yoktur, onu "içinde `'static` var" diye hatırlayan biri kalmamıştır.[^nomicon]

## İnvaryans: yazılabilen yer esnemez

`&mut T` ise `T` için *invaryanttır*: Ne alt türü ne üst türü kabul eder. Nedeni, `&mut` üzerinden hem okunup hem yazılabilmesidir. Aşağıdaki fonksiyon, kutunun içine yeni bir referans yazar:

```rust
fn degistir<'a>(kutu: &mut &'a str, yeni: &'a str) {
    *kutu = yeni;
}

fn main() {
    let mut uzun: &'static str = "yıllık";
    degistir(&mut uzun, "başka yıllık");
    println!("{uzun}");
}
```

Çıktı:

```text
başka yıllık
```

Kutunun etiketi `'static`, yazılan metin de `'static` olduğu için sorun yoktur. Oysa aynı kutuya kısa ömürlü bir metin yazmaya kalkarsak derleyici durur:

```rust,compile_fail,E0597
# fn degistir<'a>(kutu: &mut &'a str, yeni: &'a str) { *kutu = yeni; }
fn main() {
    let mut uzun: &'static str = "yıllık";
    {
        let gecici = String::from("günlük");
        degistir(&mut uzun, &gecici);
    } // gecici burada düşer
    println!("{uzun}");
}
```

Derleyicinin yanıtı:

```text
error[E0597]: `gecici` does not live long enough
  |
3 |     let mut uzun: &'static str = "yıllık";
  |                   ------------ type annotation requires that `gecici` is borrowed for `'static`
4 |     {
5 |         let gecici = String::from("günlük");
  |             ------ binding `gecici` declared here
6 |         degistir(&mut uzun, &gecici);
  |                             ^^^^^^^ borrowed value does not live long enough
7 |     } // gecici burada düşer
  |     - `gecici` dropped here while still borrowed
```

`&mut` ardındaki `&'static str` kısaltılamadığı için `degistir`'in `'a` parametresi `'static` olmak zorunda kalır; o zaman `yeni` argümanı da `'static` olmalıdır. Kısaltmaya izin verilseydi fonksiyon kutuya `gecici`'nin adresini yazar, blok bitince `uzun` silinmiş bir metni gösterirdi. `uzun`'un türü ise hâlâ "program boyunca geçerli" derdi: tam anlamıyla bir sarkan referans.[^nomicon]

Aynı kural `&mut Vec<&'static str>` için de geçerlidir. Paylaşımlı `&Vec<&'static str>` kısaltılabilirken değiştirilebilir olanı kısaltılamaz:

```rust,compile_fail
fn kisalt<'kisa>(v: &'kisa mut Vec<&'static str>) -> &'kisa mut Vec<&'kisa str> {
    v
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error: lifetime may not live long enough
  |
1 | fn kisalt<'kisa>(v: &'kisa mut Vec<&'static str>) -> &'kisa mut Vec<&'kisa str> {
  |           ----- lifetime `'kisa` defined here
2 |     v
  |     ^ returning this value requires that `'kisa` must outlive `'static`
  |
  = note: requirement occurs because of a mutable reference to `Vec<&str>`
  = note: mutable references are invariant over their type parameter
  = help: see <https://doc.rust-lang.org/nomicon/subtyping.html> for more information about variance
```

Bu hatanın kodu yoktur, ama notları ezberlemeye değer: `mutable references are invariant over their type parameter` satırını gördüğünüzde varyans sorunu yaşadığınızı anlarsınız. Çağıran, elindeki vektörün yalnızca `'static` metin taşıdığına güvenir; kısaltmaya izin verilseydi fonksiyon aynı vektöre kısa ömürlü bir metin ekleyebilirdi.

### Değiştirilebilirlik sağlayan her tür invaryanttır

Kural `&mut` ile sınırlı değildir. Paylaşımlı referans üzerinden yazmaya izin veren türler de, yani `UnsafeCell`, `Cell` ve `RefCell`, içerdikleri tür için invaryanttır:[^varyans]

```rust,compile_fail,E0597
use std::cell::Cell;

fn yaz<'a>(kutu: &Cell<&'a str>, yeni: &'a str) {
    kutu.set(yeni);
}

fn main() {
    let kutu: Cell<&'static str> = Cell::new("yıllık");
    let gecici = String::from("günlük");
    yaz(&kutu, &gecici);
}
```

Derleyicinin yanıtı:

```text
error[E0597]: `gecici` does not live long enough
```

Burada `&mut` hiç geçmiyor, ama `Cell` içeriğini `&` üzerinden değiştirebildiği için aynı tehlike vardır. Ham işaretçilerde ayrım işaretçinin türündedir: `*const T` kovaryant, `*mut T` invaryanttır.

## İki ömür: `&'a mut &'b str`

İnvaryans, struct tasarımında beklenmedik bir yerde karşınıza çıkar. Bir metin referansını değiştirilebilir biçimde ödünç alan küçük bir yapı düşünün:[^rfr]

```rust
struct MutStr<'a, 'b> {
    s: &'a mut &'b str, // 'a: tutma süresi, 'b: metnin geçerlilik süresi
}

fn main() {
    let mut s = "hello";
    *MutStr { s: &mut s }.s = "world";
    println!("s = {s}");
}
```

Çıktı:

```text
s = world
```

Burada iki ayrı süre vardır: kutuyu ne kadar elinizde tuttuğunuz (`'a`) ve kutunun içindeki metnin ne kadar geçerli olduğu (`'b`). Kütüphaneden bir kitabı masanıza beş dakikalığına almak gibi: Kitap yıllarca yaşar, siz onu beş dakika tutarsınız. İki süreyi tek bir etikete sıkıştırdığımızda aynı program derlenmez:

```rust,compile_fail,E0502
struct Tek<'a> {
    s: &'a mut &'a str,
}

fn main() {
    let mut s = "hello";
    *Tek { s: &mut s }.s = "world";
    println!("{s}");
}
```

Derleyicinin yanıtı:

```text
error[E0502]: cannot borrow `s` as immutable because it is also borrowed as mutable
  |
7 |     *Tek { s: &mut s }.s = "world";
  |               ------ mutable borrow occurs here
8 |     println!("{s}");
  |                ^
  |                |
  |                immutable borrow occurs here
  |                mutable borrow later used here
```

Değiştirilebilir ödünç ilk satırda bitmiş olmalıydı; derleyici neden `println!`'e kadar sürdüğünü söylüyor? Adım adım gidelim.

`s`'in türü `&'s str`'dir. `s` son satırda okunduğu için `'s` o satırı da kapsamak zorundadır. `&mut s` ifadesinin türü `&'odunc mut &'s str` olur. `Tek<'a>` iki ömrün de `'a` olmasını ister. İçteki konum invaryanttır, dolayısıyla `'a` ile `'s` birbirine *eşit* olmak zorundadır; kısaltma yolu kapalıdır. Dıştaki ödünç de `'a` kadar sürmelidir. Sonuç: `&mut s` ödüncü `s`'in son kullanıldığı yere, yani `println!` satırına kadar uzar ve oradaki okumayla çakışır. Tek bir parametre hem kovaryant hem invaryant konumda geçtiğinde invaryant olan kazanır.[^mutstr]

> [!NOTE]
> Bu hata yalnız `'static` metinlerle ilgili değildir. `s` başlangıçta yerel bir `String`'den ödünç alınmış olsa da aynı `E0502` çıkar; önemli olan `s`'in türündeki ömrün ödüncün süresine eşitlenmesidir. `println!` satırını silerseniz `Tek` de derlenir, çünkü `s`'in ömrünün uzanması gereken bir yer kalmaz.

İki ömürlü `MutStr`'de ise `'b` invaryant konumda kalır ve `s`'in ömrüne eşitlenir; `'a` yalnızca kovaryant konumdadır ve dilediği kadar kısalabilir. `&'a mut &'b str` türünün iyi biçimli olması için `'b: 'a` koşulu gerekir: İçteki veri, onu gösteren dıştaki referanstan önce ölemez. Derleyici bu koşulu imzadan kendiliğinden varsayar (implied bound), siz yazmazsınız.[^ortuk-sinir] Koşul yalnızca `'b`'nin `'a`'dan uzun olmasını ister; `'a`'nın kısalmasına engel olmaz. Böylece ödünç ilk satırda biter ve `println!` çalışır.

`&'a mut Yapi<'a>` biçimi gördüğünüzde bu yüzden durup düşünün: Değeri, yaşadığı sürece tek bir ödünçle kilitlemiş olabilirsiniz.

## Kontravaryans: fonksiyon argümanlarında yön ters döner

Fonksiyon işaretçileri, argümanlarının türü için *kontravaryanttır*. Rust'ta kontravaryansın tek kaynağı budur.[^varyans] `fn(&'static str)` bekleyen bir yere hangi fonksiyonlar verilebilir?

```rust
fn herkesi_kabul(s: &str) {
    println!("kabul: {s}");
}

fn yalniz_yillik(s: &'static str) {
    println!("yıllık: {s}");
}

fn calistir(gorevli: fn(&'static str)) {
    gorevli("yıllık kart");
}

fn main() {
    calistir(yalniz_yillik);
    calistir(herkesi_kabul);
}
```

Çıktı:

```text
yıllık: yıllık kart
kabul: yıllık kart
```

`calistir`, görevliye yalnızca `'static` metin verecektir. Her metni kabul eden `herkesi_kabul` bu işi rahatça yapar; daha azını şart koşan fonksiyon daha çok işe yarar. Demek ki `fn(&'a str)`, `fn(&'static str)` beklenen yere geçer: `'static` `'a`'nın alt türüyken `fn(&'a str)` `fn(&'static str)`'nin alt türüdür. Yön tersine dönmüştür.

Tersini denersek, yani kısa ömürlü metin verecek birine yalnızca yıllık kart kabul eden görevliyi verirsek, derleyici engel olur:

```rust,compile_fail,E0597
fn yalniz_yillik(s: &'static str) {
    println!("yıllık: {s}");
}

fn gecici_ver<'a>(bilet: &'a str, gorevli: fn(&'a str)) {
    gorevli(bilet);
}

fn main() {
    let gecici = String::from("günlük bilet");
    gecici_ver(&gecici, yalniz_yillik);
}
```

Derleyicinin yanıtı:

```text
error[E0597]: `gecici` does not live long enough
   |
10 |     let gecici = String::from("günlük bilet");
   |         ------ binding `gecici` declared here
11 |     gecici_ver(&gecici, yalniz_yillik);
   |                ^^^^^^^  ------------- coercion requires that `gecici` is borrowed for `'static`
   |                |
   |                borrowed value does not live long enough
12 | }
   | - `gecici` dropped here while still borrowed
```

`yalniz_yillik`, aldığı metni istediği kadar saklayabilir; ona günlük bilet vermek sarkan bir referans üretebilirdi. `gecici_ver` çağrısının geçerli olabilmesi için `'a`'nın `'static` olması gerekir, `gecici` ise o kadar yaşamaz.

Fonksiyonun dönüş türü için yön yine kovaryanttır: `fn() -> &'static str`, `fn() -> &'a str` beklenen yere geçer.

## Varyans tablosu

Sık kullanılan türlerin varyansı şöyledir:[^varyans]

| Tür | `'a` için | `T` için |
| --- | --- | --- |
| `&'a T` | kovaryant | kovaryant |
| `&'a mut T` | kovaryant | invaryant |
| `Box<T>`, `Vec<T>`, `[T]`, `[T; N]` | | kovaryant |
| `*const T` | | kovaryant |
| `*mut T` | | invaryant |
| `UnsafeCell<T>`, `Cell<T>`, `RefCell<T>` | | invaryant |
| `fn() -> T` | | kovaryant |
| `fn(T)` | | kontravaryant |
| `PhantomData<T>` | | kovaryant |
| `dyn Trait<T> + 'a` | kovaryant | invaryant |

Ezberlemek yerine iki soru sorun: "Bu konumdan yalnızca *okunuyor* ya da *üretiliyor* mu?" Yanıt evetse kovaryanttır. "Buraya *yazılabiliyor* mu?" Yanıt evetse invaryanttır. Geriye yalnızca fonksiyon argümanı kalır; o da değeri *tüketen* konumdur ve kontravaryanttır.

## Varyansı seçmek: `PhantomData`

Kendi struct'larınızın varyansı alanlarından yapısal olarak çıkarılır. Alanlarınız ham işaretçi gibi ömür bilgisi taşımayan türlerse istediğiniz varyansı yer kaplamayan bir `PhantomData` alanıyla belirtirsiniz:[^phantom]

```rust
use std::marker::PhantomData;

struct Okur<'a, T> {
    adres: *const u8,
    _isaret: PhantomData<&'a T>, // &'a T gibi davran: kovaryant
}

fn kisalt<'kisa>(o: Okur<'kisa, &'static str>) -> Okur<'kisa, &'kisa str> {
    o
}

fn main() {
    let sayi = 7u8;
    let o: Okur<'_, &'static str> = Okur { adres: &sayi, _isaret: PhantomData };
    let kisa = kisalt(o);
    println!("{}", std::ptr::eq(kisa.adres, &sayi));
}
```

Çıktı:

```text
true
```

`PhantomData<&'a T>` yerine `PhantomData<*mut T>` yazsaydık tür `T` için invaryant olurdu ve aynı kısaltma derlenmezdi:

```rust,compile_fail
use std::marker::PhantomData;

struct Yazar<T> {
    adres: *mut u8,
    _isaret: PhantomData<*mut T>, // *mut T gibi davran: invaryant
}

fn kisalt<'kisa>(y: Yazar<&'static str>) -> Yazar<&'kisa str> {
    y
}
# fn main() {}
```

Derleyicinin yanıtı:

```text
error: lifetime may not live long enough
  |
8 | fn kisalt<'kisa>(y: Yazar<&'static str>) -> Yazar<&'kisa str> {
  |           ----- lifetime `'kisa` defined here
9 |     y
  |     ^ returning this value requires that `'kisa` must outlive `'static`
  |
  = note: requirement occurs because of the type `Yazar<&str>`, which makes the generic argument `&str` invariant
  = note: the struct `Yazar<T>` is invariant over the parameter `T`
```

Derleyici suçlunun hangi tür ve hangi parametre olduğunu da söylüyor: `the struct Yazar<T> is invariant over the parameter T`. `PhantomData`'nın öteki görevlerini [26. bölümde](26-dropck.md) göreceğiz.

## Pratik kural: türlerinizi kovaryant tutun

Kovaryant türler kullanana kolaylık sağlar, invaryant türler kısıt getirir. *Rust for Rustaceans* şu yolu önerir:[^rfr]

- Türlerinizi mümkün olduğunca kovaryant tutun. Yazmanız gerekmiyorsa `&mut` ya da `Cell` yerine `&` kullanın; ham işaretçide `*mut T` yerine `*const T` ya da `NonNull<T>` düşünün. `NonNull<T>` `T` için kovaryanttır.
- Bir alan "`&mut` ardında ortak ömür" yüzünden gereksiz yere invaryant oluyorsa dıştaki ödüncün ömrüyle içteki verinin ömrünü ayrı parametrelere bölün. `MutStr<'a, 'b>` bu yolla kurtuldu.
- İkinci ömür parametresi okunabilirliği zorlaştırır. Bu bilişsel maliyeti, invaryansın getirdiği katılıkla tartıp karar verin.

## Hata kodu başvurusu

| Kod | Ne zaman görülür? | Ne yapmalı? |
| --- | --- | --- |
| `E0597` | Kısa ömürlü bir veri, invaryant bir konumun istediği uzun ömre verildiğinde | Hedefin ömrünü `'static` diye sabitlemeyin ya da veriyi daha uzun yaşatın. |
| `E0502` | `&'a mut &'a T` kalıbında ödünç, değerin kullanıldığı yere kadar uzadığında | Dıştaki ödüncün ömrüyle içteki verinin ömrünü ayırın. |
| kodsuz `lifetime may not live long enough` | İnvaryant bir tür kısaltılmaya çalışıldığında | Notlardaki "invariant over" satırına bakın; türü kovaryant yapın ya da imzayı değiştirin. |

## Perde arkası: Derleyici ne yapıyor?

> [!NOTE]
> Bu başlık ilk okuyuşta atlanabilir; yukarıdaki kuralları anlamış olmanız yeterlidir.

**Varyansın çıkarılması.** Derleyici bir crate'teki bütün türlerin varyansını tek bir sorguda, `crate_variances` içinde hesaplar.[^crate-varyans] Her parametre "hiç kullanılmıyor" varsayımıyla başlar. Sonra alanlar dolaşılır ve her kullanım için bir kısıt yazılır: `ty::Ref` görüldüğünde hem ömür hem iç tür için, `fn` türünde argümanlar için ters yönde.[^kisitlar] Kısıtlar sabit noktaya varana kadar çözülür; birden çok kullanım birleştirilirken `glb` fonksiyonu kullanılır ve invaryant ile karşılaşan her şey invaryant olur. Kovaryant ile kontravaryantın birleşimi de invaryanttır.[^glb] Hiç kullanılmayan bir parametre *bivaryant* kalır; derleyici buna `E0392` hatasıyla izin vermez.

İki tür bu hesaba girmez, çünkü varyansları derleyicide sabittir: `UnsafeCell` invaryant, `PhantomData` kovaryanttır.[^lang-items] `Cell` ve `RefCell`'in invaryant oluşu da içlerindeki `UnsafeCell`'den gelir.

**Varyansın kullanılması.** Ödünç denetleyicisi iki türü karşılaştırırken bulunduğu konumun varyansını yanında taşır ve içeri girdikçe birleştirir.[^relate] İki ömre geldiğinde bu bilgiyi kısıta çevirir: kovaryant konumda `'a: 'b`, kontravaryant konumda `'b: 'a`; invaryant konumda ikisi birden. "İkisi birden" eşitlik demektir; `Tek` örneğindeki `'a` ile `'s`'nin eşitlenmesi tam olarak buradan gelir. Kısıtlar sağlanamadığında varyans bilgisi hata iletisine eklenir; gördüğünüz `invariant over` notlarını bu kod üretir.[^tani]

**Çalışma zamanı.** Ömürler ve varyans yalnızca derleme zamanında vardır. Kod üretiminden önce bütün ömürler tek bir "silinmiş" bölgeye indirgenir; çalışan programda referans yalnızca bir işaretçidir.[^silme]

## Özet

- Alt tür ilişkisi ömürler arasındadır: Uzun ömür kısanın yerine geçer, `'static` her ömrün alt türüdür.
- `&'a T`, `Box<T>` ve `Vec<T>` kovaryanttır; `&mut T`, `*mut T`, `Cell<T>` ve `RefCell<T>` `T` için invaryanttır; `fn(T)` argümanı için kontravaryanttır.
- İnvaryansın nedeni yazılabilirliktir. Kısaltmaya izin verilseydi içeri kısa ömürlü bir veri yazılır ve sarkan referans oluşurdu.
- `&'a mut &'a T` kalıbı ödüncü değerin ömrüne eşitler. Ömürleri `&'a mut &'b T` biçiminde ayırmak esnekliği geri verir.
- Varyansı derleyici alanlardan çıkarır; ham işaretçili türlerde `PhantomData` ile siz belirlersiniz.

Sıradaki bölüm [22. `Sized`, DST ve Geniş İşaretçiler](22-sized-dst.md), boyutu derleme zamanında bilinmeyen türlerin nasıl taşındığını anlatıyor.

## Kaynaklar

[^nomicon]: The Rustonomicon, [Subtyping and Variance](https://doc.rust-lang.org/nomicon/subtyping.html), [Subtyping](https://doc.rust-lang.org/nomicon/subtyping.html#subtyping) ve [Variance](https://doc.rust-lang.org/nomicon/subtyping.html#variance) başlıkları.
[^alt-tur]: The Rust Reference, [Subtyping and Variance](https://doc.rust-lang.org/reference/subtyping.html#subtyping-and-variance).
[^omur-siniri]: The Rust Reference, [Lifetime bounds](https://doc.rust-lang.org/reference/trait-bounds.html#lifetime-bounds); The Rust Programming Language, [The Static Lifetime](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html#the-static-lifetime).
[^varyans]: The Rust Reference, [Variance](https://doc.rust-lang.org/reference/subtyping.html#variance); The Rustonomicon, [Variance](https://doc.rust-lang.org/nomicon/subtyping.html#variance).
[^rfr]: Jon Gjengset, *Rust for Rustaceans*, 1. bölüm (Foundations), "Lifetimes" başlığı.
[^mutstr]: The Rust Reference, [Variance](https://doc.rust-lang.org/reference/subtyping.html#variance) (parametre birden çok konumda geçtiğinde); hata dizini, [`E0502`](https://doc.rust-lang.org/error_codes/E0502.html) ve [`E0597`](https://doc.rust-lang.org/error_codes/E0597.html).
[^ortuk-sinir]: The Rust Reference, [Implied bounds](https://doc.rust-lang.org/reference/trait-bounds.html#implied-bounds).
[^phantom]: std, [`PhantomData`](https://doc.rust-lang.org/std/marker/struct.PhantomData.html); The Rustonomicon, [Table of `PhantomData` patterns](https://doc.rust-lang.org/nomicon/phantom-data.html#table-of-phantomdata-patterns); std, [`NonNull`](https://doc.rust-lang.org/std/ptr/struct.NonNull.html).
[^crate-varyans]: rustc 1.99.0 kaynak kodu, [`variance/mod.rs`, `crate_variances`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_analysis/src/variance/mod.rs#L30) ve [`rustc_type_ir/src/lib.rs`, `pub enum Variance`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_type_ir/src/lib.rs#L238).
[^kisitlar]: rustc 1.99.0 kaynak kodu, [`constraints.rs`, `ty::Ref`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_analysis/src/variance/constraints.rs#L230) ve [`ty::FnPtr`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_analysis/src/variance/constraints.rs#L307).
[^glb]: rustc 1.99.0 kaynak kodu, [`solve.rs`, `fn glb`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_analysis/src/variance/solve.rs#L16); hata dizini, [`E0392`](https://doc.rust-lang.org/error_codes/E0392.html).
[^lang-items]: rustc 1.99.0 kaynak kodu, [`terms.rs`, `fn lang_items`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_hir_analysis/src/variance/terms.rs#L109).
[^relate]: rustc 1.99.0 kaynak kodu, [`relate_tys.rs`, `relate_with_variance`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_borrowck/src/type_check/relate_tys.rs#L343) ve [`fn regions`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_borrowck/src/type_check/relate_tys.rs#L434).
[^tani]: rustc 1.99.0 kaynak kodu, [`region_errors.rs`, `mutable references are invariant over their type parameter`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_borrowck/src/diagnostics/region_errors.rs#L528).
[^silme]: rustc 1.99.0 kaynak kodu, [`erase_regions.rs`, `erase_and_anonymize_regions`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_middle/src/ty/erase_regions.rs#L22) ve [`region_kind.rs`, `ReErased`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_type_ir/src/region_kind.rs#L182).
