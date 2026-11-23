# 2. Yaygın Kavramlar

Neredeyse her programlama dilinde aynı dört parçayı bulursunuz: isim verilmiş değerler (*değişkenler*), bu değerlerin cinsi (*türler*), iş yapan küçük tarifler (*fonksiyonlar*) ve "şu olursa bunu yap, şunu tekrarla" diyen yol tabelaları (*kontrol akışı*). Bu bölümde dört parçayı sırayla ele alacağız.

Kavramlar tanıdık gelse de Rust bazı kararları başka dillerden farklı verir. Değişkenler varsayılan olarak değiştirilemez. Aynı isim, başka türde bir değerle yeniden kullanılabilir. Sığmayan bir toplama ya da dizinin dışına taşan bir erişim sessizce geçiştirilmez. `if` ve `loop` birer değer verebilir. Her kararın bir gerekçesi vardır; bölüm boyunca "böyle olmasaydı ne bozulurdu?" sorusunu da yanıtlayacağız.

Bölümün sonunda, bu kararlardan birinin ardındaki kök kurala bakacağız: Bir veriye aynı anda ya birçok yerden bakılır ya da tek bir yerden yazılır. Bu kural [3. bölümün](03-ownership.md) ana konusudur; burada yalnızca neden gerektiğini göreceksiniz.

> [!NOTE]
> Bu bölümün notları ve çalıştırılabilir kodu `learn/common-concepts` dizinindedir. Depo kökünden `cargo run -p common-concepts` komutuyla çalıştırabilirsiniz. Kodu değiştirdiğinizde yalnızca derlenip derlenmediğini görmek için `cargo check -p common-concepts` yeterlidir.

## Günlük hayattan bir benzetme

Okulda olduğunuzu düşünün. Defteriniz, kaleminiz ve çantanız var. Defterinize tükenmez kalemle yazdığınızı silemezsiniz; kurşun kalemle yazdığınızı silip yenisini yazabilirsiniz. Rust size önce tükenmez kalemi uzatır: Yazdığınız değer değişmez. Kurşun kalem istiyorsanız bunu `mut` diyerek önceden söylersiniz.

Çantanızda her şeyin kendi yeri vardır; suluğa kalem koymazsınız. *Tür*, hangi kaba ne konabileceğini söyler. Koridorda yürürken "bu kapı açık, şu kapı kapalı" diyen tabelalara bakarsınız; *kontrol akışı* da programa hangi satıra gideceğini söyleyen tabelalardır.

Bir de sınıfın en önemli kuralı vardır: Bir deftere aynı anda ya birçok kişi bakar ya da tek kişi yazar. İki kişi aynı anda boyarsa resim bozulur, biri boyarken öteki bakarsa yarım resim görür. Varsayılan kalemin tükenmez olması bu kuralın kendisi değil, dışarıdan görünen yansımasıdır.

<details>
<summary>Bu bölümde geçen terimler</summary>

- **bağlama (binding), değişken (variable):** Bir değere verilen isim; defterdeki "yaş: 7" satırı gibi.
- **değiştirilemez (immutable):** İlk değeri verildikten sonra yeni bir değer alamayan.
- **değiştirilebilir (mutable):** Değeri sonradan değiştirilebilen. `mut` anahtar sözcüğüyle belirtilir.
- **sabit (constant):** `const` ile tanımlanan, derleme zamanında hesaplanan ve hiç değişmeyen değer.
- **gölgeleme (shadowing):** Aynı ismi yeni bir `let` ile başka bir değere bağlamak. Eski değer silinmez, o isimle erişilemez olur.
- **tür (type):** Değerin ne olduğunu ve onunla neler yapılabileceğini belirleyen bilgi.
- **skaler ve bileşik tür (scalar, compound):** Tek parça değer (bir sayı, bir karakter) ve birkaç değerin paketi (demet, dizi).
- **tam sayı ve kayan noktalı sayı (integer, float):** `7` gibi küsuratsız ve `3.15` gibi küsuratlı sayılar.
- **`i32`, `u32`, `u8`, `f64`:** Sayının türü ve kaç bit yer tuttuğu. `i` eksi değer de alabilen, `u` eksi değer almayan tam sayı, `f` küsuratlı sayı demektir.
- **`usize`:** "Kaç tane?" ve "kaçıncı?" sorularının cevabında kullanılan tam sayı türü.
- **`bool`:** Yalnızca `true` ya da `false` olabilen tür.
- **`char`:** Tek tırnak içinde yazılan tek bir karakter: `'a'`, `'ş'`, `'7'`.
- **`&str`:** Çift tırnak içindeki metnin türü: `"merhaba"`.
- **demet (tuple):** Farklı türde olabilen birkaç değerin paketi: `(1, 2.0, 'x')`.
- **dizi (array):** Aynı türde, sayısı baştan belli değerlerin sırası: `[10, 20, 30]`.
- **parçalama (destructuring):** Bir paketi açıp içindekilere ayrı ayrı isim vermek.
- **indeks (index):** Sıradaki yer numarası. Saymaya 0'dan başlanır; ilk öğe `dizi[0]` olur.
- **taşma (overflow):** Bir hesabın sonucunun, türün tutabileceği aralığın dışına çıkması.
- **panik (panic):** Programın "burada bir yanlışlık var" deyip kendini durdurması.
- **debug ve release:** İki derleme profili. Birincisi geliştirirken kullanılır ve daha çok denetim içerir; ikincisi en iyileştirilmiş programı üretir.
- **fonksiyon (function):** İsim verilmiş iş tarifi. Girdi alabilir, iş yapar, sonuç verebilir. `fn` ile tanımlanır.
- **ifade (expression):** Bir değer veren kod parçası: `2 + 3`.
- **deyim (statement):** Bir iş yapan ama kullanılabilir bir değer vermeyen parça: `let x = 5;`.
- **birim türü (unit), `()`:** "Anlamlı bir değer yok" demenin Rust'taki yazılışı.
- **kontrol akışı (control flow):** Programın hangi satıra gideceğine karar veren yapılar: `if`, `loop`, `while`, `for`.
- **aralık (range):** `1..4` gibi bir sayı aralığı; 1, 2 ve 3'ü kapsar, son sayı dâhil değildir.
- **iterator:** Öğeleri "sıradaki, sıradaki" diye tek tek veren değer.
- **derleyici (rustc):** Kodunuzu çalışan programa çeviren ve pek çok hatayı önceden bulan araç.
- **derleme zamanı ve çalışma zamanı:** Kodun programa çevrildiği ve programın çalıştığı evre.
- **hata kodu:** Derleyici hatalarının numarası. `rustc --explain E0384` komutu `E0384`'ü açıklar.
- **referans (`&`) ve ödünç (borrow):** Verinin kendisi değil, yerini gösteren değer ve bu yolla alınan kullanma izni.
- **alias:** Aynı veriye ulaşan ikinci bir yol; örneğin aynı değeri gösteren başka bir referans.
- **ödünç denetleyicisi (borrow checker):** Derleyicinin, referansların güvenli kullanıldığını denetleyen bölümü.
- **AST, HIR, MIR:** Derleyicinin kodu adım adım sadeleştirirken kullandığı ara biçimler.
- **stack:** Boyutu derleme zamanında bilinen yerel değerlerin durduğu hızlı bellek bölgesi.
- **thread:** Programın aynı anda yürüyebilen kollarından biri.
- **veri yarışı (data race):** En az biri yazan iki thread'in aynı belleğe eşgüdümsüz, aynı anda erişmesi.
- **serbest bırakıldıktan sonra kullanım (use-after-free):** Geri verilmiş bir belleği hâlâ geçerliymiş gibi kullanma hatası.
- **jenerik ve trait (generic, trait):** İlerideki bölümlerin konusu; burada yalnızca adları geçer.

</details>

## Değişkenler ve değişebilirlik

Bir ismi bir değere `let` ile bağlarsınız. "Değişken" sözcüğü yaygın olsa da Rust'ta daha kesin terim *bağlamadır* (binding), çünkü `let` ile tanımlanan isim varsayılan olarak **değiştirilemez** (immutable):[^degisken]

```rust
fn main() {
    let x = 5;
    println!("x = {x}");

    let mut y = 5;
    println!("ilk y = {y}");
    y = 6; // mut yazıldığı için serbest
    println!("yeni y = {y}");
}
```

Çıktı:

```text
x = 5
ilk y = 5
yeni y = 6
```

`x` tükenmez kalemle yazılmıştır: Değeri bu fonksiyon boyunca 5 kalır. `y` ise `mut` (*mutable*, değiştirilebilir) sözcüğü sayesinde kurşun kalemle yazılmıştır. Tükenmez kalemle yazılanı değiştirmeye kalkarsanız program derlenmez:

```rust,compile_fail,E0384
fn main() {
    let x = 5;
    x = 6;
    println!("x = {x}");
}
```

Derleyicinin yanıtı:

```text
error[E0384]: cannot assign twice to immutable variable `x`
  |
2 |     let x = 5;
  |         - first assignment to `x`
3 |     x = 6;
  |     ^^^^^ cannot assign twice to immutable variable
  |
help: consider making this binding mutable
  |
2 |     let mut x = 5;
  |         +++
```

Derleyici çözümü de önerir: Değer gerçekten değişecekse `let mut x` yazın. Değişmeyecekse hata size bir yanlışı göstermiş olur.

### Neden varsayılan değiştirilemez?

Her değer her yerden değiştirilebilseydi, bir değeri kimin, nerede değiştirdiğini bulmak zorlaşırdı. Rust'ta `mut` sözcüğü "burası değişecek" diyen bir işarettir. Böylece değer yanlışlıkla değiştirilemez ve niyetiniz kodda görünür. `mut` olmayan bir ismi okurken değerinin sonraki satırlarda aynı kaldığını bilirsiniz.

> [!WARNING]
> Sık duyulan "değiştirilemez değişkenler daha hızlıdır" açıklaması yanıltıcıdır. Yerel bir tam sayıyı yalnızca `let` yerine `let mut` ile tanımlamak, üretilen makine kodunu değiştirmez; derleyici bir değerin gerçekten değişip değişmediğini zaten görür. Hız kazancı doğuran bilgi bağlamanın `mut` olup olmaması değil, bir *referansın* tek olduğunun bilinmesidir.[^alias] Bunu [bölümün sonunda](#kuralın-kazancı-ve-bedeli) ele alacağız.

`mut`'u asıl gerekli kılan, bölümün sonunda göreceğimiz kök kuraldır. Şimdilik şunu bilmeniz yeter: Yazma yetkisi istenmeden verilmez.

### Sabitler: `const`

Sınıfın duvarındaki "Bu sınıfta 30 sıra var" levhasını düşünün. Levha okul açılmadan önce basılmıştır ve dersin ortasında değişmez. *Sabit* (constant) böyle bir levhadır:[^sabit]

```rust
const SIRA_SAYISI: usize = 3;

fn main() {
    const MAX_PUAN: u32 = 100_000; // alt çizgi yalnızca okumayı kolaylaştırır
    println!("MAX_PUAN = {MAX_PUAN}");

    let siralar = [0; SIRA_SAYISI]; // 3 tane 0
    println!("{siralar:?}");
}
```

Çıktı:

```text
MAX_PUAN = 100000
[0, 0, 0]
```

İkinci satırdaki `{siralar:?}`, dizi gibi değerleri hata ayıklama biçiminde yazdırır; süslü parantezin içindeki `:?` bunu ister. Ayrıntısını [4. bölümde](04-structs.md) göreceğiz.

`const`, `let`'ten dört yönden ayrılır:

- Hiç değişmez; `const mut` yazılamaz.
- Türü yazılmak zorundadır.
- Değeri çalışma zamanında değil, **derleme zamanında** hesaplanır. Bu yüzden yalnızca derleme zamanında değeri bulunabilen *sabit ifadelerle* kurulabilir.[^sabit-ifade]
- Fonksiyonların dışında da tanımlanabilir. Adı, geleneğe göre büyük harfle ve alt çizgiyle yazılır.

Tür yazılmadığında ya da `mut` eklendiğinde derleyici sabiti reddeder:

```rust,compile_fail
const TURSUZ = 3;
const mut DEGISEN: u32 = 3;
# fn main() {}
```

Derleyicinin yanıtı:

```text
error: const globals cannot be mutable
error: missing type for `const` item
```

Peki `let` zaten değiştirilemezken neden ayrı bir `const` var? Çünkü bazı yerlerde değerin derleme zamanında bilinmesi şarttır. Dizi uzunluğu bunların başında gelir. `let` ile bağlanan bir isim, değeri kâğıt üzerinde belli olsa bile orada kullanılamaz:

```rust,compile_fail,E0435
fn main() {
    let n = 3;
    let baska = [0; n];
    println!("{baska:?}");
}
```

Derleyicinin yanıtı:

```text
error[E0435]: attempt to use a non-constant value in a constant
  |
3 |     let baska = [0; n];
  |                     ^ non-constant value
  |
help: consider using `const` instead of `let`
```

`let` ile kurulan bağlamaya kullanıcı girdisi, dosya içeriği ya da başka bir hesabın sonucu da bağlanabilir; derleyici bu yüzden onu sabit saymaz. `n`'yi `const N: usize = 3;` biçiminde tanımlamak sorunu çözer.

### Gölgeleme (shadowing)

Defterinize "cevap" başlıklı bir satır yazdınız. Sonra alt satıra yine "cevap" başlığıyla başka bir şey yazdınız. Artık "cevap" denince alttaki satıra bakılır; üstteki silinmemiştir, ama ona bakan kalmamıştır. Rust'ta aynı ismi ikinci kez `let` ile bağlamaya *gölgeleme* (shadowing) denir:[^golge]

```rust
fn main() {
    let bosluk = "   "; // türü &str
    let bosluk = bosluk.len(); // türü usize
    println!("boşluk sayısı = {bosluk}");

    let cevap = "42"; // metin
    let cevap: i32 = cevap.parse().expect("metin bir tam sayı olmalı"); // sayı
    println!("cevap + 1 = {}", cevap + 1);
}
```

Çıktı:

```text
boşluk sayısı = 3
cevap + 1 = 43
```

`parse` metodu bir metni istenen türe çevirir; çeviremezse `expect` programı verdiğiniz iletiyle durdurur. Hata yönetimini [8. bölümde](08-error-handling.md) göreceğiz.

Gölgelemenin `mut`'tan iki farkı vardır. Birincisi, yeni bağlama yeni bir değişkendir ve **türü farklı olabilir**. İkincisi, sonuç yine değiştirilemez. `mut` ise değeri değiştirir ama türü değiştiremez:

```rust,compile_fail,E0308
fn main() {
    let mut s = "   ";
    s = s.len();
    println!("{s}");
}
```

Derleyicinin yanıtı:

```text
error[E0308]: mismatched types
  |
2 |     let mut s = "   ";
  |                 ----- expected due to this value
3 |     s = s.len();
  |         ^^^^^^^ expected `&str`, found `usize`
```

`let mut puan = "beş"; puan = 5;` yazsaydınız da aynı hatayı alırdınız. Gölgeleme olmasaydı bir değeri adım adım dönüştürürken her ara adıma `cevap_metin`, `cevap_sayi` gibi yeni bir isim uydurmak gerekirdi.

Gölgelenen bağlama yok olmaz; yalnızca o isimle erişilemez olur. İç bir blokta yapılan gölgeleme bittiğinde dıştaki bağlamanın yerinde durduğunu görebilirsiniz:

```rust
fn main() {
    let cevap = 1;
    {
        let cevap = cevap * 10; // dıştaki cevap okunur, sonra gölgelenir
        println!("içeride: {cevap}");
    }
    println!("dışarıda: {cevap}");
}
```

Çıktı:

```text
içeride: 10
dışarıda: 1
```

## Veri türleri

Rust statik türlü bir dildir: Her değerin türü derleme zamanında bilinmek zorundadır.[^tur] Cinsi belli olmayan bir değerle ne yapılabileceği de belli olmaz; "elma" metni ile 3 sayısını toplamak ne demektir? Derleyici bu karışıklığı program çalışmadan yakalar:

```rust,compile_fail,E0308
fn main() {
    let hata: i32 = "yedi";
    println!("{hata}");
}
```

Derleyicinin yanıtı:

```text
error[E0308]: mismatched types
  |
2 |     let hata: i32 = "yedi";
  |               ---   ^^^^^^ expected `i32`, found `&str`
  |               |
  |               expected due to this
```

Türü çoğu zaman siz yazmazsınız; derleyici onu kullanımdan çıkarır. Birden çok tür mümkünse, örneğin `parse` çağrısında, hangisini istediğinizi yazmanız gerekir.

### Skaler türler

*Skaler* (scalar) tür tek bir değeri temsil eder. Rust'ta dört temel skaler tür vardır: tam sayılar, kayan noktalı sayılar, `bool` ve `char`.[^skaler]

```rust
fn main() {
    let tam: i32 = -42; // işaretli, 32 bit
    let ondalik: f64 = 3.15; // küsuratlı
    let dogru: bool = true;
    let harf: char = 'ş'; // tek tırnak
    println!("{tam} {ondalik} {dogru} {harf}");

    let yas = 7; // tür yazılmadı: i32
    let boy = 1.25; // tür yazılmadı: f64
    println!("{yas} yaşında, {boy} metre");
}
```

Çıktı:

```text
-42 3.15 true ş
7 yaşında, 1.25 metre
```

Tam sayı türünün adı iki bilgi taşır: `i` (*signed*) eksi değer de alabileceğini, `u` (*unsigned*) alamayacağını; sondaki sayı ise kaç bit yer tuttuğunu söyler:[^sayilar]

| Uzunluk | İşaretli | İşaretsiz | İşaretsizin en büyük değeri |
| --- | --- | --- | --- |
| 8 bit | `i8` | `u8` | 255 |
| 16 bit | `i16` | `u16` | 65 535 |
| 32 bit | `i32` | `u32` | 4 294 967 295 |
| 64 bit | `i64` | `u64` | yaklaşık 1,8 × 10¹⁹ |
| 128 bit | `i128` | `u128` | yaklaşık 3,4 × 10³⁸ |
| mimariye bağlı | `isize` | `usize` | 64 bitlik hedefte `u64` ile aynı |

İşaretli türler aralığı eksi ve artı tarafa böler; örneğin `i8` -128'den 127'ye kadar değer tutar. Başka bir bilgi yoksa tam sayılar `i32`, küsuratlı sayılar `f64` sayılır. `usize`, "kaç tane?" ve "kaçıncı?" sorularının türüdür: Uzunluklar ve indeksler `usize` ile verilir. Küsuratlı sayılar için `f64`'ün yanında 32 bitlik `f32` de vardır.

`char`, tek bir Unicode skaler değerini tutar ve bellekte 4 bayt yer kaplar.[^char] Metinler ise UTF-8 ile saklanır; aynı karakter bir metnin içinde daha az yer tutabilir:

```rust
fn main() {
    println!("char olarak {} bayt", size_of::<char>());
    println!("metin içinde {} bayt", 'ş'.len_utf8());
}
```

Çıktı:

```text
char olarak 4 bayt
metin içinde 2 bayt
```

### Taşma

Parmaklarınızla en çok 10'a kadar sayabilirsiniz; 11 için parmak kalmaz. Sayı türlerinin de bir sınırı vardır: `u8` en çok 255'i tutar. 255'e 1 eklemek *taşmaya* (overflow) yol açar. Taşmanın olabileceği yerde ne istediğinizi adıyla söyleyen metotlar vardır:[^tasma]

```rust
fn main() {
    let puan: u8 = 255; // u8'in tutabildiği en büyük sayı

    println!("{:?}", puan.checked_add(1)); // sığmadı: None
    println!("{}", puan.wrapping_add(1)); // bilerek başa sarar
    println!("{}", puan.saturating_add(1)); // sınırda kalır
    println!("{:?}", puan.overflowing_add(1)); // sonuç ve "taştı mı?" bilgisi
}
```

Çıktı:

```text
None
0
255
(0, true)
```

`checked_add` sonucu `Option` türüyle bildirir: Sığarsa `Some(sonuç)`, sığmazsa `None`. `Option`'ı [5. bölümde](05-enums.md) ayrıntılı göreceğiz.

Sıradan `+` işlemi ise taşmayı bir hata sayar. Derleyici taşmayı daha derlerken görebiliyorsa programı reddeder:

```rust,compile_fail
fn main() {
    let puan: u8 = 255;
    let yeni = puan + 1;
    println!("{yeni}");
}
```

Derleyicinin yanıtı:

```text
error: this arithmetic operation will overflow
  |
3 |     let yeni = puan + 1;
  |                ^^^^^^^^ attempt to compute `u8::MAX + 1_u8`, which would overflow
  |
  = note: `#[deny(arithmetic_overflow)]` on by default
```

Değer ancak çalışma zamanında belli oluyorsa derleyici bunu önceden bilemez. O zaman debug derlemesinde program panikler:

```rust,should_panic
fn main() {
    let puan: u8 = "255".parse().expect("u8 aralığında bir sayı olmalı");
    let yeni = puan + 1;
    println!("{yeni}");
}
```

Çıktı:

```text
attempt to add with overflow
```

Aynı program `cargo run --release` ile derlenirse taşma denetimi varsayılan olarak kapalıdır: Sonuç başa sarar, yani `0` yazılır. Denetim release derlemesinde de istenirse `Cargo.toml` içinde açılır:[^profil]

```toml
[profile.release]
overflow-checks = true
```

Taşma neden hata sayılıyor? C'de işaretsiz sayılar sessizce başa sarar; işaretli sayıların taşması ise tanımsız davranıştır, yani dil bu durumda programın ne yapacağını hiç söylemez.[^c-tasma] İki durumda da hata fark edilmeden ilerleyebilir. Rust'ta `+`, `-` ve `*` işlemlerinin taşması tanımsız davranış değildir; program ya panikler ya da başa sarılmış, önceden bilinen bir sonuçla devam eder.[^rfc560]

> [!WARNING]
> Release derlemesinin başa sarmasına güvenerek kod yazmayın; debug derlemesinde panik alırsınız. Sarma istiyorsanız `wrapping_add` ile bunu açıkça yazın.

### Demet (tuple)

Beslenme çantanıza farklı şeyler koyarsınız: bir elma, bir sandviç, bir su. *Demet* (tuple) böyle karışık bir pakettir; öğelerinin türü farklı olabilir, ama kaç öğesi olduğu baştan bellidir ve sonradan değişmez:[^demet]

```rust
fn main() {
    let demet: (i32, f64, char) = (1, 2.0, 'x');
    let (a, b, c) = demet; // parçalara ayırma
    println!("{a} {b} {c}, .1 = {}", demet.1);

    let canta = (1, 2.5, 'e');
    println!("ilk göz: {}", canta.0); // saymaya 0'dan başlanır
}
```

Çıktı:

```text
1 2 x, .1 = 2
ilk göz: 1
```

Öğelere iki yoldan ulaşılır. `let (a, b, c) = demet;` satırı paketi açar ve her öğeye isim verir; buna *parçalama* (destructuring) denir. Nokta ve sıra numarası (`demet.1`) ise tek bir öğeyi okur. `2.0` sayısının ekrana `2` olarak yazılması yalnızca gösterimdir; türü hâlâ `f64`'tür.

Hiç öğesi olmayan demete *birim* (unit) denir ve `()` yazılır. Anlamlı bir sonuç vermeyen işlemlerin sonucu budur; birazdan fonksiyonlarda karşımıza çıkacak.

### Dizi (array)

Boya kalemi kutusunda hep aynı cins şey durur ve göz sayısı bellidir. *Dizi* (array) de böyledir: Öğelerinin türü aynıdır, uzunluğu türünün parçasıdır ve sonradan değişmez:[^dizi]

```rust
fn main() {
    let kalemler: [i32; 3] = [3, 5, 8]; // üç tane i32
    println!("üçüncü göz: {}", kalemler[2]);

    let dizi = [10, 20, 30];
    println!("ilk = {}", dizi[0]);
    println!("uzunluk = {}", dizi.len());
}
```

Çıktı:

```text
üçüncü göz: 8
ilk = 10
uzunluk = 3
```

`[i32; 3]` "üç tane `i32`" diye okunur; `[0; 3]` ise "3 tane 0" demektir. Burada `dizi` gibi yerel bir dizinin öğeleri stack'te durur. Uzunluğu program çalışırken değişebilen bir liste gerekiyorsa `Vec<T>` kullanılır; onu [7. bölümde](07-collections.md) göreceğiz.

### Dizinin dışına uzanmak

Sınıfta 3 sıra varsa "6. sıraya otur" denen öğrenci oturacak yer bulamaz. Sıra numarasının geçerli olduğundan emin değilseniz `get` metodunu kullanın; olmayan bir yer için programı durdurmadan `None` verir:[^sinir]

```rust
fn main() {
    let siralar = [10, 20, 30];
    println!("{:?}", siralar.get(1)); // orada bir sıra var
    println!("{:?}", siralar.get(5)); // öyle bir sıra yok
}
```

Çıktı:

```text
Some(20)
None
```

Köşeli parantez ise her erişimde indeksi uzunlukla karşılaştırır. İndeks çalışma zamanında belli oluyorsa ve sınırın dışındaysa program panikler:

```rust,should_panic
fn main() {
    let siralar = [10, 20, 30];
    let i: usize = "5".parse().expect("geçerli bir sıra numarası olmalı");
    println!("{}", siralar[i]);
}
```

Çıktı:

```text
index out of bounds: the len is 3 but the index is 5
```

İndeks kodda sabit olarak yazılmışsa derleyici çalışmayı beklemez ve programı derleme zamanında reddeder:

```rust,compile_fail
fn main() {
    let dizi = [10, 20, 30];
    println!("{}", dizi[5]);
}
```

Derleyicinin yanıtı:

```text
error: this operation will panic at runtime
  |
3 |     println!("{}", dizi[5]);
  |                    ^^^^^^^ index out of bounds: the length is 3 but the index is 5
  |
  = note: `#[deny(unconditional_panic)]` on by default
```

Program durdurulmasaydı dizinin yanındaki, başka işe ayrılmış belleği okuyabilir ya da bozabilirdi. C gibi dillerde bu denetim yapılmaz; dizi sınırını aşmak güvenlik açıklarının klasik kaynaklarından biridir. Yanlış veriyle sessizce devam etmek, durmaktan çok daha tehlikelidir.

## Fonksiyonlar

Fonksiyon, isim verilmiş bir iş tarifidir. `fn` ile tanımlanır, parametrelerinin türü yazılmak zorundadır ve sonuç veriyorsa dönüş türü `->` işaretinden sonra belirtilir:[^fonksiyon]

```rust
fn main() {
    println!("toplam(2, 3) = {}", toplam(2, 3));
    println!("iki_kati(4) = {}", iki_kati(4));
}

fn toplam(x: i32, y: i32) -> i32 {
    x + y // noktalı virgül yok: dönen değer budur
}

fn iki_kati(x: i32) -> i32 {
    x * 2
}
```

Çıktı:

```text
toplam(2, 3) = 5
iki_kati(4) = 8
```

İmza, fonksiyonun ne alıp ne verdiğini gövdeye bakmadan söyler. Derleyicinin parametre türlerini istemesi sayesinde fonksiyonu çağıran yerlerde tür yazmanız çoğu zaman gerekmez.

### Deyim ve ifade

Öğretmen iki türlü cümle kurar. "Kapıyı kapat" bir iştir; karşılığında cevap beklenmez. "2 + 3 kaç eder?" ise bir sorudur; karşılığında bir değer gelir: 5. Rust'ta iş cümlelerine *deyim* (statement), değer veren parçalara *ifade* (expression) denir.[^deyim]

- `let x = 5;` bir deyimdir. Bir iş yapar, ama başka bir yere yazılabilecek bir değer vermez.
- `5 + 6`, bir fonksiyon çağrısı, bir blok (`{ ... }`) ve `if` birer ifadedir. Her biri bir değere karşılık gelir.

Rust *ifade temelli* bir dildir: Kodun büyük bölümü ifadelerden oluşur ve ifadeler birbirinin içine yerleşebilir. Süslü parantezlerle çevrili bir blok da ifadedir. Onu küçük bir karalama kâğıdı gibi düşünebilirsiniz: İçinde hesap yaparsınız, son satırdaki sonuç dışarı çıkar, karalamada kullandığınız isimler dışarıdan görünmez:

```rust
fn bes() -> i32 {
    let hesap = {
        let temel = 2; // yalnızca bu blokta geçerli
        temel + 3 // blok bu değere karşılık gelir
    };
    hesap
}

fn main() {
    println!("bes() = {}", bes());
}
```

Çıktı:

```text
bes() = 5
```

Bir blokta ya da fonksiyonda noktalı virgülle bitmeyen son ifadeye *kuyruk ifadesi* (tail expression) denir; bloğun değeri odur. Fonksiyondan daha erken çıkmak isterseniz `return` kullanırsınız, ama son satırda gerekmez.

### Noktalı virgülün etkisi

Bir ifadenin sonuna noktalı virgül koymak onu deyime çevirir; ifadenin değeri yok sayılır. Dönüş değeri vermesi gereken bir fonksiyonda bu, cevabın kaybolması demektir:

```rust,compile_fail,E0308
fn bozuk(x: i32) -> i32 {
    x * 2;
}
# fn main() { println!("{}", bozuk(3)); }
```

Derleyicinin yanıtı:

```text
error[E0308]: mismatched types
  |
1 | fn bozuk(x: i32) -> i32 {
  |    -----            ^^^ expected `i32`, found `()`
  |    |
  |    implicitly returns `()` as its body has no tail or `return` expression
2 |     x * 2;
  |          - help: remove this semicolon to return this value
```

Geriye kalan `()`, az önce gördüğümüz birim değeridir. Kuyruk ifadesi olmayan bir blok `()` verir; dönüş türü yazılmamış fonksiyonlar da `()` döndürür:[^fonksiyon]

```rust
fn selamla() {
    println!("merhaba");
}

fn main() {
    let sonuc = selamla();
    println!("{sonuc:?}");
}
```

Çıktı:

```text
merhaba
()
```

Hatayı düzeltmek için yapılacak tek şey, derleyicinin de önerdiği gibi noktalı virgülü silmektir: `x * 2`.

## Kontrol akışı

Program satırları yukarıdan aşağıya sırayla çalıştırır. Yol ayrımı ve tekrar olmasaydı her program hep aynı şeyi, yalnızca bir kez yapardı. Yol ayrımını `if`, tekrarı `loop`, `while` ve `for` sağlar.[^akis]

### `if` ifadesi

"Zil çaldıysa bahçeye çık, çalmadıysa yerinde otur" bir yol ayrımıdır. Rust'ta `if` bir ifadedir; değer verir ve bu değer bir isme bağlanabilir:[^if]

```rust
fn main() {
    let zil_caldi = true;
    let yer = if zil_caldi { "bahçe" } else { "sınıf" };
    println!("{yer}");

    let sayi = 7;
    let etiket = if sayi % 2 == 0 { "çift" } else { "tek" };
    println!("{sayi} sayısı {etiket}");
}
```

Çıktı:

```text
bahçe
7 sayısı tek
```

`%` işlemi bölmeden kalanı verir; 7'nin 2'ye bölümünden kalan 1 olduğu için sayı tektir. İkiden çok yol varsa araya `else if` eklenir.

Değer veren bir `if`'in dalları aynı türde olmalıdır:

```rust,compile_fail,E0308
fn main() {
    let zil_caldi = true;
    let x = if zil_caldi { 1 } else { "yok" };
    println!("{x}");
}
```

Derleyicinin yanıtı:

```text
error[E0308]: `if` and `else` have incompatible types
  |
3 |     let x = if zil_caldi { 1 } else { "yok" };
  |                            -          ^^^^^ expected integer, found `&str`
  |                            |
  |                            expected because of this
```

Derleyici `x`'in türünü derleme zamanında tek ve kesin olarak bilmek zorundadır: Bellekte ne kadar yer ayıracağı ve `x` ile hangi işlemlerin yapılabileceği buna bağlıdır. Bir dal `i32`, öteki `&str` verseydi tür belirsiz kalırdı. Bir dal hiç değer üretmeden akışı kesiyorsa (`return`, `break`, `panic!`) bu kural sorun çıkarmaz: Böyle ifadelerin türü `!` (never) diye yazılır ve her türe dönüşebilir.[^never]

Koşulun türü de `bool` olmak zorundadır. Bazı dillerde sıfırdan farklı sayılar "doğru" sayılır; Rust böyle bir dönüşüm yapmaz:

```rust,compile_fail,E0308
fn main() {
    let sayi = 7;
    if sayi {
        println!("sıfır değil");
    }
}
```

Derleyicinin yanıtı:

```text
error[E0308]: mismatched types
  |
3 |     if sayi {
  |        ^^^^ expected `bool`, found integer
```

Niyetinizi açıkça yazmanız gerekir: `if sayi != 0`.

### `loop` ve `break` ile değer döndürmek

`loop`, "ben dur diyene kadar devam et" demektir. `break` o "dur" sözüdür ve döngüye bir sonuç da verdirebilir:[^loop]

```rust
fn main() {
    let mut sayac = 0;
    let sonuc = loop {
        sayac += 1;
        if sayac == 5 {
            break sayac * 2; // döngünün değeri
        }
    };
    println!("loop sonucu = {sonuc}");

    let mut tur = 0;
    let son = loop {
        tur += 1;
        if tur == 3 {
            break tur * 10;
        }
    };
    println!("{tur}. turda durdu, değeri {son}");
}
```

Çıktı:

```text
loop sonucu = 10
3. turda durdu, değeri 30
```

`break` ile değer vermek yalnızca `loop`'ta (ve etiketli bloklarda) mümkündür. `while` ve `for` hiç dönmeden de bitebilir; o durumda verecek bir değerleri olmazdı.

### `while`

"Tahtada yazı kaldıkça sil": Koşul doğru oldukça dönen tekrar `while` ile yazılır. Örnekteki `print!`, `println!`'in satır sonu eklemeyen hâlidir:[^while]

```rust
fn main() {
    let mut geri = 3;
    while geri > 0 {
        print!("{geri} ");
        geri -= 1;
    }
    println!("kalk!");
}
```

Çıktı:

```text
3 2 1 kalk!
```

Koşul her turun başında denetlenir; baştan yanlışsa gövde hiç çalışmaz.

### `for`

Öğretmenin yoklama listesindeki her ismi sırayla okuması `for`'dur. Rust'ta en çok kullanılan döngüdür; bir aralığı da bir dizinin öğelerini de dolaşabilir:[^for]

```rust
fn main() {
    for sira in 1..4 {
        println!("{sira}. öğrenci burada");
    }

    let dizi = [10, 20, 30];
    for eleman in dizi {
        println!("eleman: {eleman}");
    }
}
```

Çıktı:

```text
1. öğrenci burada
2. öğrenci burada
3. öğrenci burada
eleman: 10
eleman: 20
eleman: 30
```

`1..4` bir *aralıktır* (range) ve üst sınırı kapsamaz: 1, 2 ve 3. Üst sınır da istenirse `1..=4` yazılır.[^aralik]

### `for` neden tercih edilir?

Aynı işi `while` ve bir indeksle de yapabilirsiniz. Ama sınırı siz yazarsınız ve yanlış yazabilirsiniz. Aşağıdaki programda sınır bir fazladır:

```rust,should_panic
fn main() {
    let dizi = [10, 20, 30];
    let mut i = 0;
    while i < 4 { // uzunluk 3
        println!("{}", dizi[i]);
        i += 1;
    }
}
```

Çıktı:

```text
10
20
30
index out of bounds: the len is 3 but the index is 3
```

Sınır bir eksik yazılsaydı program paniklemez, son öğeyi sessizce atlardı. `for` ise koleksiyonun kendisi üzerinde gezer; elle yazılacak bir sınır olmadığı için bu hatalar baştan ortadan kalkar.

Bunu nasıl yaptığını görmek için `for`'un açılımına bakabiliriz. `for x in it { ... }`, aynı işi kısa yazdıran bir *sözdizimsel şekerdir* (syntactic sugar). Derleyici onu kabaca şu koda çevirir:[^for-acilim]

```rust
fn main() {
    let dizi = [10, 20, 30];

    let mut it = IntoIterator::into_iter(dizi);
    loop {
        match it.next() {
            Some(eleman) => println!("eleman: {eleman}"),
            None => break,
        }
    }
}
```

Çıktı:

```text
eleman: 10
eleman: 20
eleman: 30
```

`into_iter` diziden bir *iterator* üretir; iterator, "sıradaki" dendikçe bir öğe veren değerdir. `next` öğeyi `Some` içinde verir, bitince `None` döndürür ve döngü durur. Demek ki `for`, aslında bir `loop` ile bir `match`'ten oluşur; hiçbir yerde indeks ve dolayısıyla sınır derdi yoktur. `match`'i [5. bölümde](05-enums.md), iterator'ları [12. bölümde](../orta/12-closures-iterators.md) göreceğiz.

## Kök kural: Aliasing XOR Mutability

Bölümün başında "neden varsayılan değiştirilemez?" sorusuna kısa bir yanıt verdik: Yanlışlıkla değiştirmeyi önler ve niyeti görünür kılar. Daha derindeki yanıt, Rust'ın bellek güvenliğini dayandırdığı tek bir kuraldır. Kuralın kendisini [3. bölümde](03-ownership.md) örneklerle işleyeceğiz; bu başlığın amacı, `mut`'un neden var olduğunu göstermektir. İlk okuyuşta ayrıntılar yerine oturmazsa [3. bölümden](03-ownership.md) sonra geri dönün.

> [!NOTE]
> *Alias*, aynı veriye ulaşan ikinci bir yoldur. *XOR* "ya o ya bu, ikisi birden değil" demektir. Kuralın adı "ya paylaşım ya değiştirme" diye çevrilebilir.

### Kural

Bir değeri `&` ile *ödünç almak*, onun yerini gösteren bir *referans* oluşturur. `&T` yalnızca okumaya, `&mut T` hem okumaya hem yazmaya izin verir. Bir veri için, herhangi bir anda:[^referans]

- ya istediğiniz kadar paylaşımlı referans (`&T`),
- ya da tam olarak bir değiştirilebilir referans (`&mut T`) bulunabilir.

İkisi aynı anda olamaz. `&mut T` veriye sahip olmaz; sahiplik başka bir konudur. Verdiği şey, ödünç sürdüğü sürece **tek ve dışlayıcı** erişimdir.

### Kural olmasaydı ne bozulurdu?

Bellek hatalarının ve veri yarışlarının önemli bir bölümü aynı durumdan doğar: Aynı veriye bir yoldan yazılırken başka bir yol da o veriyi görmektedir. Üç örnek verelim. Üçü de Rust'ta derlenmez.

**Büyüyen liste.** `Vec`, uzunluğu değişebilen bir listedir. Ayrılan yer dolduğunda yeni öğe eklemek, bütün listeyi bellekte daha geniş bir yere taşıyabilir.[^vec] İçindeki bir öğeye tuttuğunuz referans o zaman geri verilmiş belleği gösterirdi; buna *serbest bırakıldıktan sonra kullanım* (use-after-free) denir. Derleyici bu kodu reddeder:

```rust,compile_fail,E0502
fn main() {
    let mut v = vec![1, 2, 3];
    let ilk = &v[0];
    v.push(4);
    println!("{ilk}");
}
```

Derleyicinin yanıtı:

```text
error[E0502]: cannot borrow `v` as mutable because it is also borrowed as immutable
  |
3 |     let ilk = &v[0];
  |                - immutable borrow occurs here
4 |     v.push(4);
  |     ^^^^^^^^^ mutable borrow occurs here
5 |     println!("{ilk}");
  |                --- immutable borrow later used here
```

**Döngü sırasında değiştirmek.** Bir koleksiyonun üzerinde dönerken onu değiştirmek, döngünün elindeki konum bilgisini geçersiz kılabilir. C++'ta bu hata *iterator invalidation* adıyla bilinir:

```rust,compile_fail,E0502
fn main() {
    let mut v = vec![1, 2, 3];
    for x in &v {
        v.push(*x);
    }
}
```

Derleyicinin yanıtı:

```text
error[E0502]: cannot borrow `v` as mutable because it is also borrowed as immutable
```

**İki thread.** Programın aynı anda yürüyen iki kolundan biri yazarken öteki okursa *veri yarışı* (data race) oluşur; bu, tanımsız davranıştır. Thread'leri [15. bölümde](../orta/15-concurrency.md), örnekteki `||` ile başlayan closure'ları [12. bölümde](../orta/12-closures-iterators.md) göreceğiz. Burada yalnızca derleyicinin aynı kuralla bunu da yakaladığına dikkat edin:

```rust,compile_fail,E0502
fn main() {
    let mut sayac = 0;
    std::thread::scope(|s| {
        s.spawn(|| sayac += 1); // yazan kol
        s.spawn(|| println!("{sayac}")); // okuyan kol
    });
}
```

Derleyicinin yanıtı:

```text
error[E0502]: cannot borrow `sayac` as immutable because it is also borrowed as mutable
```

Üç hata birbirinden çok farklı görünür, ama kökleri aynıdır ve tek bir kural üçünü birden baştan keser.[^yarislar]

### `mut`'un bu kuraldaki yeri

Tehlikeli olan okumak değil, yazmaktır. Rust bu yüzden yazma yetkisini varsayılan olarak vermez: Bağlamalar değiştirilemez, referanslar paylaşımlıdır. Yazmak istiyorsanız bunu `mut` ve `&mut` ile **açıkça** istersiniz ve istek kodda izlenebilir. `mut` olmayan bir isimden değiştirilebilir referans alınamaması bunun sonucudur:

```rust,compile_fail,E0596
fn main() {
    let x = 5;
    let a = &mut x;
    *a += 1;
    println!("{x}");
}
```

Derleyicinin yanıtı:

```text
error[E0596]: cannot borrow `x` as mutable, as it is not declared as mutable
  |
help: consider changing this to be mutable
```

Demek ki "varsayılan değiştirilemez" kuralın kendisi değildir. Kural, referansların paylaşma ve değiştirme ilişkisidir; değiştirilemez varsayılan, onun yüzeye yansımasıdır. `mut` ise bu kuralın açık anahtarıdır.

> [!NOTE]
> Paylaşımlı bir referans üzerinden denetimli biçimde yazmayı sağlayan `Cell`, `RefCell` ve `Mutex` gibi türler de vardır. Buna *iç değiştirilebilirlik* denir ve [14. bölümün](../orta/14-smart-pointers.md) konusudur. Bu türler kuralı kaldırmaz, ona uymanın başka bir yolunu kurar; örneğin `RefCell` denetimi çalışma zamanında yapar.

### Kuralın kazancı ve bedeli

Kural derleme zamanında denetlenir; çalışan programa denetim yükü bindirmez. Karşılığında güvenli Rust kodunda veri yarışı ve serbest bırakıldıktan sonra kullanım olmaz.[^yarislar]

İkinci kazanç hızdır. Derleyici, `&mut T` türündeki bir parametrenin gösterdiği veriye o sırada başka yoldan erişilmediğini bilir ve bunu kod üreticisi LLVM'e `noalias` ("bu veriye ulaşan başka yol yok") bilgisiyle bildirir. LLVM de okumaları birleştirmek gibi iyileştirmeleri daha rahat yapar. Bu bilgi yalnızca parametre konumunda ve `T: Unpin` olduğunda verilir, dönüş değerine verilmez. İçinde iç değiştirilebilirlik bulunmayan `&T` parametreleri de aynı bilgiyi taşır.[^alias][^noalias]

Bedel şudur: Başka dillerde serbest olan bazı kodlar, örneğin aynı veriyi gösteren iki değiştirilebilir işaretçi, reddedilir:

```rust,compile_fail,E0499
fn main() {
    let mut x = 5;
    let a = &mut x;
    let b = &mut x;
    *a += 1;
    *b += 1;
}
```

Derleyicinin yanıtı:

```text
error[E0499]: cannot borrow `x` as mutable more than once at a time
```

Bu yüzden Rust öğrenirken ara sıra "ödünç denetleyicisiyle güreştiğinizi" hissedebilirsiniz. Bu bir kusur değil, kuralın bedelidir; karşılığında yukarıdaki güvenceleri alırsınız.

## Hata kodu başvurusu

| Kod | Ne zaman görülür? | Ne yapmalı? |
| --- | --- | --- |
| `E0384` | `mut` olmayan bir bağlamaya ikinci kez değer atandığında | Değer değişecekse `let mut` yazın; yeni bir değerse gölgeleyin. |
| `E0308` | Beklenen tür ile verilen tür uyuşmadığında | Türü düzeltin; fonksiyon sonundaki fazladan noktalı virgülü silin; `if` dallarını aynı türe getirin. |
| `E0435` | Sabit gereken yerde (ör. dizi uzunluğu) `let` ile bağlanan isim kullanıldığında | `const` tanımlayın. |
| `E0502` | Paylaşımlı ve değiştirilebilir ödünç çakıştığında | Okuyan referansın işini bitirip sonra değiştirin. |
| `E0499` | Aynı anda iki `&mut` alındığında | İlk ödüncün son kullanımını öne alın. |
| `E0596` | `mut` olmayan bir isimden `&mut` alındığında | Bağlamayı `let mut` ile tanımlayın. |

Kodu olmayan hatalar da bu bölümde geçti. Derleme zamanında görülen taşma (`arithmetic_overflow`) ve sabit indeksle dizi dışına uzanma (`unconditional_panic`), varsayılan düzeyi *deny* olan lint'lerdir; mesajlarının sonundaki not bunu söyler. `const mut` ve türü yazılmamış `const` ise kodu olmayan sıradan hatalardır. Kodlu bir hatanın uzun açıklamasını terminalde okuyabilirsiniz:

```console
rustc --explain E0384
```

## Perde arkası: Derleyici ne yapıyor?

> [!NOTE]
> Bu başlık ilk okuyuşta atlanabilir; yukarıdaki kavramları anlamış olmanız yeterlidir.

Ödevinizin öğretmene ulaşmadan önce birkaç kişiden geçtiğini düşünün: Biri yazım hatalarına, biri cümlelerin anlamlı olup olmadığına, biri de "aynı deftere iki kişi aynı anda mı yazıyor?" sorusuna bakar. Derleyici de kodunuzu kabaca dokuz duraktan geçirir. Duraklar kesin bir sıra değildir; derleyici sorgulara dayalı çalışır ve bir sonuca ihtiyaç duydukça onu hesaplar.[^genel]

1. **`rustc_lexer`** metni *token* denen parçalara böler.[^lexer]
2. **`rustc_parse`** token'lardan sözdizim ağacını (AST) kurar. `let mut y` burada bir bağlama ve bir değişebilirlik bilgisi olarak tutulur.[^ast] Makrolar bu aşamada açılır ve isim çözümlemesi (*name resolution*) bir ismin hangi bağlamayı gösterdiğini bulur.
3. **`rustc_ast_lowering`** AST'yi HIR'e indirger. Kısa yazımlar burada uzun hâllerine açılır (*desugaring*): `for` bir `loop` ile bir `match`'e, `?` bir `match`'e, `while` ise içinde `if` bulunan bir `loop`'a dönüşür.[^lowering]
4. **`rustc_hir_typeck`** ve **`rustc_hir_analysis`** tür denetimini, tür çıkarımını ve trait çözümünü yapar. Tür uyuşmazlığı (`E0308`) burada bulunur.
5. **`rustc_mir_build`**, HIR'den (THIR adlı bir ara biçim üzerinden) MIR'i kurar. MIR, *temel bloklardan* oluşan bir kontrol akışı grafiğidir (CFG); "bu adımdan sonra hangi adımlara gidilebilir?" haritası gibidir. Dizi sınırı ve taşma denetimleri bu grafiğe `Assert` düğümleri olarak eklenir. `Assert`, "şu doğru değilse dur" demektir.
6. **`rustc_borrowck`**, MIR üzerinde çalışan ödünç denetleyicisidir. Aliasing XOR mutability kuralını ve ödünçlerin ömrünü (NLL, *non-lexical lifetimes*) burada denetler. `E0384` de bu aşamada üretilir.[^e0384]
7. **`rustc_mir_transform`** MIR'i iyileştirir; gereksizliği kanıtlanan denetimler burada ya da LLVM'de kaldırılabilir.
8. **`rustc_monomorphize`**, jenerik kod için hangi somut kopyaların gerektiğini toplar. Jenerik, her türle çalışan bir kalıptır; derleyici kullandığınız her tür için kalıbın ayrı bir kopyasını üretir. Buna monomorfizasyon (monomorphization) denir.[^mono]
9. **`rustc_codegen_llvm`** MIR'i LLVM'in ara biçimine (LLVM IR) çevirir, LLVM de makine kodunu üretir.

**`E0384` neden `E0308`'den sonra gelir?** Bir fonksiyonda tür hatası varsa ödünç denetleyicisi o fonksiyonun gövdesini hiç incelemez.[^tainted] Aşağıdaki programda iki hata vardır, ama derleyici yalnızca tür hatasını bildirir. Tür hatasını düzelttiğinizde `E0384` ortaya çıkar. İki hata ayrı fonksiyonlarda olsaydı ikisini birden görürdünüz:

```rust,compile_fail,E0308
fn main() {
    let x = 5;
    x = 6; // E0384 adayı
    let y: i32 = "yedi"; // E0308
    println!("{x} {y}");
}
```

Derleyicinin yanıtı:

```text
error[E0308]: mismatched types
  |
4 |     let y: i32 = "yedi"; // E0308
  |            ---   ^^^^^^ expected `i32`, found `&str`
  |            |
  |            expected due to this
```

**Gölgeleme.** Her `let`, MIR'de ayrı bir yerel değişken (*local*) üretir. İsim çözümlemesi bir ismi görünce kapsamları içten dışa tarar ve en son yapılan bağlamayı bulur.[^rib] Gölgelenen değer bu yüzden yerinde durur ve kendi kapsamı bittiğinde düşürülür.

**Taşma denetimi.** Denetimin eklenip eklenmeyeceğine MIR kurulurken `check_overflow` bayrağıyla karar verilir.[^check] Bayrak açıksa toplama, sonucu ve taşma bilgisini birlikte veren `AddWithOverflow` işlemine ve bir `Assert`'e dönüşür;[^addoverflow] kapalıysa sade `Add` olur. Sabitlerin hesabında bayrak her zaman açıktır. `toplam` fonksiyonunun MIR'ini `rustc --emit=mir` ile görebilirsiniz. Debug derlemesinde gövdesi şöyle başlar:

```text
_3 = AddWithOverflow(copy _1, copy _2);
assert(!move (_3.1: bool), "attempt to compute `{} + {}`, which would overflow", copy _1, copy _2)
    -> [success: bb1, unwind continue];
```

**Sınır denetimi.** İndeksleme MIR'de önce bir `BoundsCheck` denetimine çevrilir;[^bounds] indeks uzunluktan küçükse erişim yapılır. İndeksin güvenli olduğu kanıtlanabiliyorsa (sabit indeks ya da uzunlukla sınırlanmış bir döngü gibi) iyileştirme aşamaları denetimi kaldırabilir; güvenlik yine durur. Kanıtlanabilen *yanlış* indeksleri ise `KnownPanicsLint` geçişi derleme hatasına çevirir.[^lint]

**Sabitler.** `const` değerleri `rustc_const_eval` crate'indeki *derleme zamanı değerlendiricisi* hesaplar; dizi uzunluğu gibi yerlerde de aynı motor çalışır.[^consteval]

**Noktalı virgül.** Bir bloğun deyimleri AST'de bir liste olarak tutulur. Noktalı virgülsüz ifade ile noktalı virgüllü ifade bu listede ayrı çeşitlerdir (`Expr` ve `Semi`).[^stmt] Listenin sonunda `Expr` varsa bloğun değeri odur; yoksa, akış bloktan daha önce çıkmıyorsa, blok `()` verir. Noktalı virgül eklemek, ifadeyi `Semi` çeşidine çevirip değerini yok saymak demektir.

## Özet

- `let` bir ismi bir değere bağlar; bağlama varsayılan olarak değiştirilemez, `mut` ile değiştirilebilir olur. `const` derleme zamanında hesaplanan, türü yazılan bir sabittir.
- Gölgeleme aynı isimle yeni bir bağlama kurar; türü farklı olabilir, eski bağlama yalnızca o isimle erişilemez olur.
- Skaler türler tam sayı, kayan noktalı sayı, `bool` ve `char`'dır. Demet farklı türleri, dizi aynı türden sabit sayıda değeri bir arada tutar.
- Taşma ve sınır dışı erişim sessizce geçmez: Derleyici görebiliyorsa derlemez, göremiyorsa program panikler (taşma denetimi release derlemesinde varsayılan olarak kapalıdır ve sonuç sarar).
- Rust ifade temellidir. Bloğun noktalı virgülsüz son ifadesi bloğun değeridir; noktalı virgül bu değeri yok sayar ve geriye `()` kalır.
- `if` ve `loop` değer verebilir. Bir koleksiyonu dolaşmak için `for` tercih edilir, çünkü elle yazılacak sınır yoktur.
- Bir veri ya paylaşılır ya değiştirilir. `mut`, yazma yetkisini açıkça istemenin yoludur.

Sıradaki bölüm olan [3. Ownership (Sahiplik)](03-ownership.md), bu kök kuralı ve verinin kime ait olduğunu baştan sona ele alır.

## Kaynaklar

[^degisken]: The Rust Programming Language, [3.1 Variables and Mutability](https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html#variables-and-mutability); The Rust Reference, [Variables](https://doc.rust-lang.org/reference/variables.html); hata dizini, [`E0384`](https://doc.rust-lang.org/error_codes/E0384.html).
[^alias]: The Rustonomicon, [Aliasing: Why Aliasing Matters](https://doc.rust-lang.org/nomicon/aliasing.html#why-aliasing-matters).
[^sabit]: The Rust Programming Language, [3.1 Declaring Constants](https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html#declaring-constants); The Rust Reference, [Constant items](https://doc.rust-lang.org/reference/items/constant-items.html#constant-items).
[^sabit-ifade]: The Rust Reference, [Constant expressions](https://doc.rust-lang.org/reference/const_eval.html#constant-expressions) ve [Const context](https://doc.rust-lang.org/reference/const_eval.html#const-context); hata dizini, [`E0435`](https://doc.rust-lang.org/error_codes/E0435.html).
[^golge]: The Rust Programming Language, [3.1 Shadowing](https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html#shadowing); The Rust Reference, [Pattern binding shadowing](https://doc.rust-lang.org/reference/names/scopes.html#pattern-binding-shadowing).
[^tur]: The Rust Programming Language, [3.2 Data Types](https://doc.rust-lang.org/book/ch03-02-data-types.html#data-types).
[^skaler]: The Rust Programming Language, [3.2 Scalar Types](https://doc.rust-lang.org/book/ch03-02-data-types.html#scalar-types); The Rust Reference, [Boolean type](https://doc.rust-lang.org/reference/types/boolean.html#boolean-type).
[^sayilar]: The Rust Programming Language, [3.2 Integer Types](https://doc.rust-lang.org/book/ch03-02-data-types.html#integer-types); The Rust Reference, [Integer types](https://doc.rust-lang.org/reference/types/numeric.html#integer-types).
[^char]: The Rust Reference, [Character type](https://doc.rust-lang.org/reference/types/char.html#character-type); std, [`char::len_utf8`](https://doc.rust-lang.org/std/primitive.char.html#method.len_utf8).
[^tasma]: The Rust Programming Language, [3.2 Integer Overflow](https://doc.rust-lang.org/book/ch03-02-data-types.html#integer-overflow); std, [`u8::checked_add`](https://doc.rust-lang.org/std/primitive.u8.html#method.checked_add), [`wrapping_add`](https://doc.rust-lang.org/std/primitive.u8.html#method.wrapping_add), [`saturating_add`](https://doc.rust-lang.org/std/primitive.u8.html#method.saturating_add) ve [`overflowing_add`](https://doc.rust-lang.org/std/primitive.u8.html#method.overflowing_add); rustc kitabı, [`arithmetic-overflow`](https://doc.rust-lang.org/rustc/lints/listing/deny-by-default.html#arithmetic-overflow).
[^profil]: The Cargo Book, [Profiles: `overflow-checks`](https://doc.rust-lang.org/cargo/reference/profiles.html#overflow-checks) ve [`release`](https://doc.rust-lang.org/cargo/reference/profiles.html#release).
[^c-tasma]: cppreference, [C: Arithmetic operators, Overflows](https://en.cppreference.com/c/language/operator_arithmetic#Overflows).
[^rfc560]: The Rust Reference, [Overflow](https://doc.rust-lang.org/reference/expressions/operator-expr.html#overflow) ve [Integer overflow](https://doc.rust-lang.org/reference/behavior-not-considered-unsafe.html#integer-overflow); [RFC 560: Integer Overflow](https://rust-lang.github.io/rfcs/0560-integer-overflow.html).
[^demet]: The Rust Programming Language, [3.2 The Tuple Type](https://doc.rust-lang.org/book/ch03-02-data-types.html#the-tuple-type); The Rust Reference, [Tuple types](https://doc.rust-lang.org/reference/types/tuple.html#tuple-types).
[^dizi]: The Rust Programming Language, [3.2 The Array Type](https://doc.rust-lang.org/book/ch03-02-data-types.html#the-array-type); The Rust Reference, [Array types](https://doc.rust-lang.org/reference/types/array.html#array-types).
[^sinir]: The Rust Programming Language, [3.2 Invalid Array Element Access](https://doc.rust-lang.org/book/ch03-02-data-types.html#invalid-array-element-access); The Rust Reference, [Array and slice indexing expressions](https://doc.rust-lang.org/reference/expressions/array-expr.html#array-and-slice-indexing-expressions); std, [`slice::get`](https://doc.rust-lang.org/std/primitive.slice.html#method.get); rustc kitabı, [`unconditional-panic`](https://doc.rust-lang.org/rustc/lints/listing/deny-by-default.html#unconditional-panic).
[^fonksiyon]: The Rust Programming Language, [3.3 Parameters](https://doc.rust-lang.org/book/ch03-03-how-functions-work.html#parameters) ve [Functions with Return Values](https://doc.rust-lang.org/book/ch03-03-how-functions-work.html#functions-with-return-values); The Rust Reference, [Functions](https://doc.rust-lang.org/reference/items/functions.html).
[^deyim]: The Rust Programming Language, [3.3 Statements and Expressions](https://doc.rust-lang.org/book/ch03-03-how-functions-work.html#statements-and-expressions); The Rust Reference, [Statements](https://doc.rust-lang.org/reference/statements.html#statements) ve [Block expressions](https://doc.rust-lang.org/reference/expressions/block-expr.html#block-expressions).
[^akis]: The Rust Programming Language, [3.5 Control Flow](https://doc.rust-lang.org/book/ch03-05-control-flow.html#control-flow).
[^if]: The Rust Programming Language, [3.5 `if` Expressions](https://doc.rust-lang.org/book/ch03-05-control-flow.html#if-expressions) ve [Using `if` in a `let` Statement](https://doc.rust-lang.org/book/ch03-05-control-flow.html#using-if-in-a-let-statement); The Rust Reference, [`if` expressions](https://doc.rust-lang.org/reference/expressions/if-expr.html#if-expressions).
[^never]: The Rust Reference, [Never type](https://doc.rust-lang.org/reference/types/never.html#never-type). `!` Rust 1.99'da yalnızca dönüş türü olarak yazılabilir; her yerde yazılabilmesi 1.100 ile kararlı oldu: Rust Blog, [Announcing Rust 1.100.0](https://blog.rust-lang.org/2026/11/12/Rust-1.100.0/#never-type).
[^loop]: The Rust Programming Language, [3.5 Returning Values from Loops](https://doc.rust-lang.org/book/ch03-05-control-flow.html#returning-values-from-loops); The Rust Reference, [`break` and loop values](https://doc.rust-lang.org/reference/expressions/loop-expr.html#break-and-loop-values).
[^while]: The Rust Programming Language, [3.5 Streamlining Conditional Loops with `while`](https://doc.rust-lang.org/book/ch03-05-control-flow.html#streamlining-conditional-loops-with-while); The Rust Reference, [Predicate loops](https://doc.rust-lang.org/reference/expressions/loop-expr.html#predicate-loops).
[^for]: The Rust Programming Language, [3.5 Looping Through a Collection with `for`](https://doc.rust-lang.org/book/ch03-05-control-flow.html#looping-through-a-collection-with-for).
[^aralik]: The Rust Reference, [Range expressions](https://doc.rust-lang.org/reference/expressions/range-expr.html).
[^for-acilim]: The Rust Reference, [Iterator loops](https://doc.rust-lang.org/reference/expressions/loop-expr.html#iterator-loops); std, [`IntoIterator`](https://doc.rust-lang.org/std/iter/trait.IntoIterator.html).
[^referans]: The Rust Programming Language, [4.2 The Rules of References](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html#the-rules-of-references).
[^vec]: std, [`Vec`: Capacity and reallocation](https://doc.rust-lang.org/std/vec/struct.Vec.html#capacity-and-reallocation); The Rust Programming Language, [8.1 Reading Elements of Vectors](https://doc.rust-lang.org/book/ch08-01-vectors.html#reading-elements-of-vectors).
[^yarislar]: The Rust Programming Language, [4.2 Mutable References](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html#mutable-references); The Rustonomicon, [Data Races and Race Conditions](https://doc.rust-lang.org/nomicon/races.html).
[^noalias]: rustc 1.99.0 kaynak kodu, [`rustc_ty_utils/src/abi.rs`, `no_alias`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_ty_utils/src/abi.rs#L373).
[^genel]: Rust Compiler Development Guide, [Overview of the compiler](https://rustc-dev-guide.rust-lang.org/overview.html).
[^lexer]: rustc 1.99.0 kaynak kodu, [`rustc_lexer/src/lib.rs`, `tokenize`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_lexer/src/lib.rs#L327).
[^ast]: rustc 1.99.0 kaynak kodu, [`ast.rs`, `BindingMode`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_ast/src/ast.rs#L817).
[^lowering]: rustc 1.99.0 kaynak kodu, [`rustc_ast_lowering/src/expr.rs`, `lower_expr_for`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_ast_lowering/src/expr.rs#L1707), [`lower_expr_while_in_loop_scope`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_ast_lowering/src/expr.rs#L641) ve [`lower_expr_try`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_ast_lowering/src/expr.rs#L1883).
[^e0384]: rustc 1.99.0 kaynak kodu, [`borrowck_errors.rs`, `cannot_reassign_immutable`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_borrowck/src/borrowck_errors.rs#L253).
[^mono]: rustc 1.99.0 kaynak kodu, [`collector.rs`, `collect_crate_mono_items`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_monomorphize/src/collector.rs#L1914).
[^tainted]: rustc 1.99.0 kaynak kodu, [`rustc_borrowck/src/lib.rs`, `tainted_by_errors`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_borrowck/src/lib.rs#L136).
[^rib]: rustc 1.99.0 kaynak kodu, [`ident.rs`, `ribs`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_resolve/src/ident.rs#L332).
[^check]: rustc 1.99.0 kaynak kodu, [`rustc_mir_build/src/builder/mod.rs`, `check_overflow`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_mir_build/src/builder/mod.rs#L766).
[^addoverflow]: rustc 1.99.0 kaynak kodu, [`syntax.rs`, `AddWithOverflow`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_middle/src/mir/syntax.rs#L1615) ve [`AssertKind`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_middle/src/mir/syntax.rs#L1038).
[^bounds]: rustc 1.99.0 kaynak kodu, [`as_place.rs`, `bounds_check`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_mir_build/src/builder/expr/as_place.rs#L726) ve [`syntax.rs`, `BoundsCheck`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_middle/src/mir/syntax.rs#L1039).
[^lint]: rustc 1.99.0 kaynak kodu, [`known_panics_lint.rs`, `KnownPanicsLint`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_mir_transform/src/known_panics_lint.rs#L29).
[^consteval]: The Rust Reference, [Constant evaluation](https://doc.rust-lang.org/reference/const_eval.html#constant-evaluation); rustc 1.99.0 kaynak kodu, [`eval_queries.rs`, `eval_to_allocation_raw_provider`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_const_eval/src/const_eval/eval_queries.rs#L409).
[^stmt]: rustc 1.99.0 kaynak kodu, [`ast.rs`, `StmtKind`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_ast/src/ast.rs#L1241) ve [`rustc_ast_lowering/src/block.rs`, `lower_stmts`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_ast_lowering/src/block.rs#L30).
