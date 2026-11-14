// Küçük örneklerle Rust temelleri. Önce main'deki binding'ler ve raw pointer ile
// başla, sonra closure'lar ile trait'lerin nasıl oturduğunu görmek için test'i
// takip et. (Terimler aşağıda ilk geçtikleri yerde açıklanıyor.)

use std::ptr;

fn main() {
    // 1. Mutable binding (tam anlatım + "Neden immutable?" kök nedeni — Aliasing XOR
    // Mutability — learn/common-concepts, Bölüm 3'te). Kısaca: let bir ismi bir değere
    // bağlar, varsayılan immutable; mut ile değiştirilebilir olur. Tür açıklaması +
    // i32 literal son eki türü belirtir.
    let mut first_var: i32 = 10_i32;

    first_var = 101;

    println!("First Variables {}", first_var);

    // 2. Raw pointer (ham işaretçi) üzerinden okuma
    // raw pointer = belleğe doğrudan işaret eden adres; "ham" çünkü normal
    // referansların güvenlik ve ömür garantileri YOK. *const i32, bir i32'ye
    // işaret eden raw pointer'dır; &s'den oluşturmak güvenlidir ama üzerinden
    // okumak unsafe (güvensiz = derleyicinin denetleyemediği, doğruluğunu
    // programcının garanti ettiği) gerektirir: okumanın geçerli, hizası doğru ve
    // başlatılmış bir i32'ye işaret ettiğini bizim garanti etmemiz gerekir.
    // Neden ayrı bir `unsafe`? Sorun: C gibi her yerde serbest işaretçi/bellek işlemine
    // izin veren dillerde bir bellek hatası KODUN HER YERİNDE olabilir; hata ararken her
    // satırı şüpheli saymak zorundasın. Çözüm: Rust kodun çoğunu derleyici denetler,
    // kanıtlayamadığı işleri yalnız `unsafe` bloklara hapseder. Böylece bir bellek hatası
    // çıkınca sadece o bloklara bakman yeter — güven yüzeyi küçük kalır. (FFI ve düşük
    // seviye işler derleyicinin kanıtlayamadığı şeylerdir.)
    // SAFETY: s1, &s'den geliyor ve her iki okuma da s yaşarken oluyor.
    unsafe {
        let s: i32 = 10;

        let s1: *const i32 = &s as *const i32;

        // ptr::read, belleği değiştirmeden bit'lerin bir kopyasını döndürür.
        // Burada i32 Copy olduğundan, orijinal değeri tekrar okumak sorun değil.
        // Aşağıdaki dereference (içeriğe erişme = işaretçinin gösterdiği değere *s1
        // ile ulaşmak) s'yi tekrar okur.
        ptr::read(s1);

        println!("{:?}", *s1);
    }

    test();
}

// 3. const (sabit) ve const generic
// const = derleme zamanında bilinen, değişmeyen sabit. const generic ise tür
// parametresi gibi ama TÜR değil DEĞER (sayı) alan bir parametre — ör. dizi
// uzunluğu. Şöyle bir tanımda:
// struct Buffer<const N: usize = 0> { data: [u8; N] }
// N bir const generic parametresidir; Buffer<4>, dört baytlık bir diziye sahiptir.
// Varsayılan = 0, const argüman verilmediğinde bir uzunluk sağlar.
// Neden const generic? Sorun: [u8; 4] ile [u8; 8] ayrı türlerdir; eskiden tek bir
// fonksiyon/tür ile FARKLI uzunlukları kapsayamazdın — ya her boyut için ayrı kod (ya
// da makro) yazardın, ya da heap'teki Vec'e kaçıp çalışma anında maliyet + derleme
// zamanı uzunluk bilgisini kaybederdin. Çözüm: const generic ile DEĞERİ (uzunluğu)
// türün parçası yaparsın; tek kod tüm boyutları, sıfır ek maliyetle kapsar.
// Aşağıdaki N ise başlı başına sıradan bir const.
const N: usize = 0;

// 4. Trait bound (trait sınırı) olan çağrılabilir bir değeri saklamak
// trait bound = bir generic türün HANGİ trait'leri implemente etmesi gerektiğini
// şart koşan kısıt. Color<F>, F türünde bir değer saklar; where'deki
// F: Fn(i8) -> i8 bound'u, F'nin bir i8 ile çağrılıp i8 döndürebilmesini şart
// koşar. Fn, paylaşımlı bir referans üzerinden çağrıya izin verir.
// closure (kapanış) = çevresindeki değişkenleri "yakalayıp" (capture) içine
// alabilen isimsiz fonksiyon; çevreyi içine "kapattığı" için kapanış denir.
// Örn: let k = 3; let ekle = |x| x + k;  → buradaki closure dışarıdaki k'yı
// yakalar. Böyle bir closure, yakaladıklarını uygun kullandığı sürece Fn'i de
// karşılar.
// Neden trait bound? Sorun: sınırsız bir generic F ile derleyici F hakkında hiçbir
// şey bilmez, dolayısıyla x'i taşımak dışında onunla HİÇBİR ŞEY yapamazsın (metot
// bile çağıramazsın). Bound'suz şablonların olduğu dillerde (eski C++) hata, türü
// kullandığın derin bir yerde anlaşılmaz mesajlarla patlar. Çözüm: bound, F'nin
// sözleşmesini ("i8 ile çağrılır, i8 döndürür") baştan belirtir — derleyici çağrı
// yerinde net denetler ve her somut tür için ayrı, hızlı kod üretir.
//
// Fn/FnMut/FnOnce neden üçe ayrıldı? Sorun: bir closure çevresinden değer YAKALAR ve
// onu NASIL kullandığı güvenliği etkiler:
//   • sadece okuyorsa             → defalarca, paylaşarak çağrılabilir  (Fn)
//   • yakaladığını DEĞİŞTİRİYORSA → çağırmak özel (&mut) erişim ister    (FnMut)
//   • yakaladığını TÜKETİYORSA    → yalnızca BİR KEZ çağrılabilir        (FnOnce)
// Bu ayrım olmasaydı: bir kez tüketen closure'ı iki kez çağırmak, taşınmış/serbest
// bırakılmış değeri tekrar kullanırdı (use-after-move / double-free); değiştiren bir
// closure'a aynı anda iki yerden erişmek veri yarışı (data race) olurdu. Üçlü ayrım
// bu tehlikeli çağrıları derleyicinin DERLEME ZAMANINDA reddetmesini sağlar:
//   let s = String::from("x");
//   let tuket = move || drop(s);  // s'yi sahiplenir → FnOnce
//   tuket(); tuket();             // ✗ 2. çağrı derlenmez — s zaten taşındı
struct Color<F>
where
    F: Fn(i8) -> i8,
{
    x: F,
}

// 5. Trait parametreleri, associated type ve associated function
// associated type (ilişkili tür) = bir trait'e bağlı, her implementasyonun kendi
// somut türünü doldurduğu bir "tür yuvası". Örn: std'deki Iterator trait'inde
// `type Item` vardır; Vec<i32>'yi gezen iterator için Item = i32 olur, satır satır
// okuyan bir iterator için String olur. associated function (ilişkili fonksiyon) =
// self almayan, bir örnek üzerinden değil doğrudan tür üzerinden çağrılan fonksiyon
// (ör. String::new() — elindeki bir String'e değil, String türüne çağrı).
// T, Animal'ın bir parametresidir; Animal<i32>, T = i32 belirtir. Her
// implementasyon associated type Your için somut bir tür sağlar.
// Neden bazen associated type bazen generic parametre? Sorun: Iterator `Iterator<Item>`
// gibi generic parametreyle tasarlansaydı, bir tür hem Iterator<i32> hem
// Iterator<String> implemente edebilirdi; o zaman `for x in it` ifadesinde hangi Item
// olduğu belirsiz kalır ve her kullanımda türü elle yazman gerekirdi. Çözüm: associated
// type "her implementasyon için TEK Item" der → belirsizlik yok, çağıran yazmak zorunda
// değil. Gerçekten birden çok varyant anlamlıysa (Animal<i32>, Animal<String>) o zaman
// generic parametre kullanılır.
trait Animal<T> {
    type Your;

    // self parametresi olmadığından voice bir associated function'dır: tür
    // üzerinden, instance (örnek = bir türden üretilmiş somut değer) oluşturmadan
    // çağrılır.
    fn voice() -> T;
}

// Bu trait impl'i, F üzerindeki bound'u karşılayan her Color<F> için Animal<i32>
// sağlar. Your'ı String yapar ve voice bir i32 döndürür.
impl<F> Animal<i32> for Color<F>
where
    F: Fn(i8) -> i8,
{
    type Your = String;

    fn voice() -> i32 {
        1
    }
}

impl<T: Future> Animal<i32> for T {
    // OR &mut T OR &T
    type Your = i32;

    fn voice() -> i32 {
        1
    }
}

// 6. Inherent impl ve function pointer
// inherent impl (kendine ait impl) = bir türe, bir trait üzerinden DEĞİL doğrudan
// kendi metotlarını ekleyen impl bloğu. function pointer (fonksiyon işaretçisi) =
// bir fonksiyonun adresini tutan, fn(i8) -> i8 türündeki değer. Bu impl, hex'i
// doğrudan Color<F>'e ekler: ilk argüman generic çağrılabilir tür F, ikincisi
// somut function pointer türü fn(i8) -> i8. Örnek her iki biçimi de kabul eder
// ama yalnızca f'yi çağırır.
// Neden hem generic F hem somut fn(i8)->i8? Sorun: her closure AYRI bir türdür; sadece
// generic F olsaydı farklı fonksiyonları tek bir değişkende/koleksiyonda tutamaz
// (Vec<F> tek closure türü alır) ve C'ye (FFI) geçiremezdin. Sadece fn pointer olsaydı
// da çevre YAKALAYAMAZ (closure olamaz) ve inline edilemediği için yavaşlardın. Çözüm:
// ikisi birlikte — generic F yakalar + inline hız verir; fn pointer tek somut tür olarak
// depolanır + FFI'da kullanılır. Duruma göre seçersin.
impl<F> Color<F>
where
    F: Fn(i8) -> i8,
{
    fn hex(f: F, b: fn(i8) -> i8) -> i8 {
        f(1)
    }
}

#[proc_macro]
pub fn my_macro(_item: TokenStream) -> TokenStream {
    "fn answer() -> u32 { 42 }".parser().unwrap()
}

// unsafe extern "C" fonksiyon (başka bir dilden, ör. C'den çağrılabilen fonksiyon)
unsafe extern "C" fn noise() -> i8 {
    1
}

// Higher-Ranked (yüksek dereceli) Fonksiyonlar — HRTB
//
// Doğrulama: Nomicon (HRTB) + Rust Reference (constant promotion); davranış
// rustc ile de test edildi (aşağıdaki yerel değişken örneği hex_v3'te E0597 verir).
//
// İki imza ömür (lifetime = bir referansın geçerli kaldığı süre) bakımından farklı:
//   hex_v2: `for<'a> Fn(&'a i8)`    → HRTB. "for<'a>" = "HER 'a seçimi için"
//           (Nomicon). f tüm ömürler için çalışmak zorunda; ömür kararı İÇERİDE.
//   hex_v3: `<'a, F> ... Fn(&'a i8)` → 'a'yı ÇAĞIRAN dıştan seçer: tek, sabit bir
//           ömür. f yalnız o 'a için çalışır.
//
// Neden biri patlar? Ömürleri iç içe kutu gibi düşün:
//
//   {                 // DIŞ bölge = 'a (çağıran; çağrıyı kapsar, uzun yaşar)
//       hex_v3(f) →
//           {         // İÇ bölge = n'nin ömrü (çağrının stack frame'i)
//               let n = 1; f(&n)   // &n yalnız İÇ bölge kadar yaşar
//           }         // ← n burada ÖLÜR
//   }                 // ← 'a burada biter
//
//   hex_v3: f, DIŞ ('a) kadar yaşayan referans ister; &n İÇ'te kalır → yetişemez
//           → E0597 "borrowed value does not live long enough".
//   hex_v2: for<'a> "her ömre uyarım" dediğinden derleyici 'a = İÇ bölge seçer
//           → &n uyar → derlenir.
//
// Sezgi: for'suzken ömür DIŞTAN sabitlenir; for<'a> ile o sabitlemeyi bırakıp
// f'in HER ömre uymasını şart koşarız ("dıştan beklemeyi kapatmak").
//
// Tuzak: aşağıdaki gövdeler `f(&1)` kullanıyor ve İKİSİ DE derlenir! Çünkü `&1`
// literali 'static'e TERFİ eder (constant promotion = derleyicinin sabit bir
// literali kalıcı/'static ömre yükseltmesi, Rust Reference → &'static i8),
// 'static de her ömrü karşılar. Fark ancak static'e terfi etmeyen yerelde (`&n`)
// ortaya çıkar — bunu aşağıdaki hex_v3_yerel örneğinde görebilirsin.
fn hex_v2<F>(f: F) -> i8
where
    F: for<'a> Fn(&'a i8) -> i8,
{
    f(&1)
}

fn hex_v3<'a, F>(f: F) -> i8
where
    F: Fn(&'a i8) -> i8,
{
    f(&1)
}

// ÖRNEK — farkı yerel bir değişkenle canlı gör. Gövdeler `f(&1)` yerine
// `let n = 1; f(&n)` kullanıyor; `&n` static'e terfi etmez, iç bölgede ölür.
// Yorumdan çıkarıp `cargo check -p basic-concepts` ile deneyebilirsin:
//
//   // ✓ DERLENİR — for<'a> her ömre uyar, derleyici 'a = n'nin iç bölgesini seçer.
//   fn hex_v2_yerel<F>(f: F) -> i8
//   where
//       F: for<'a> Fn(&'a i8) -> i8,
//   {
//       let n: i8 = 1;
//       f(&n)
//   }
//
//   // ✗ DERLENMEZ — E0597 "`n` does not live long enough".
//   // f, dıştaki tek 'a kadar yaşayan referans ister; n fonksiyon scope'unda ölür.
//   fn hex_v3_yerel<'a, F>(f: F) -> i8
//   where
//       F: Fn(&'a i8) -> i8,
//   {
//       let n: i8 = 1;
//       f(&n)
//   }

fn test() {
    // Bu closure argümanını döndürür ve scope'undan (kapsam = bir ismin geçerli
    // olduğu bölge) hiçbir şey capture etmez (yakalamaz).
    let x = |y: i8| -> i8 { y };

    // İlk x kendi closure türünü korur ve F'yi belirler. İkinci x hiçbir şey
    // capture etmediği için function pointer'a coerce olur (örtük dönüşüm =
    // derleyicinin sen yazmadan yaptığı otomatik tür dönüşümü).
    let a: i8 = Color::hex(x, x);

    // Derleyici, i8'i hex'in dönüş türünden de infer eder (çıkarsama = türü sen
    // yazmadan bağlamdan bulmak).
    let b = Color::hex(x, x);

    // voice argüman almaz, bu yüzden ::<...> ile somut bir F belirtiriz.
    // Bu imzaya sahip bir function pointer, Color üzerindeki bound'u karşılar.
    Color::<fn(i8) -> i8>::voice();

    println!("{:?}", a);
}

// 7. Auto trait ve negative impl (nightly Rust)
// marker trait (işaretleyici trait) = metodu olmayan, bir türü sadece "şu
// özelliğe sahiptir" diye işaretleyen trait. Örn: Copy = "bu değer atanırken bit
// bit kopyalanabilir" işareti; Send = "bu tür thread'ler arasında güvenle
// taşınabilir" işareti. auto trait = böyle bir marker'ı, bir struct'ın tüm
// field'ları (alan) onu implemente ediyorsa ve bunu geçersiz kılan açık bir impl
// yoksa, derleyicinin OTOMATİK verdiği trait. negative impl (negatif impl) = bir
// türün bir trait'i implemente ETMEDİĞİNİ açıkça belirten impl.
// Neden auto trait? Sorun: Send/Sync gibi özellikler KOMPOZİSYONEL'dir — bir tür, tüm
// parçaları Send ise Send'dir. Elle yazsaydın her yeni struct'a "ben de Send'im" eklemen
// gerekir; birini unutmak kütüphaneleri kullanılamaz yapar, daha kötüsü parçalarından
// biri Send DEĞİLKEN yanlışlıkla "Send'im" dersen güvenlik çöker (veri yarışı). Çözüm:
// derleyici bunu parçalardan OTOMATİK ve doğru çıkarır; negative impl ise istisnayı
// ("bu tür özellikle karşılamıyor") açıkça belirtir.
//
// Bu örneği denemek için feature attribute'unu (öznitelik) dosyanın en üstüne koy
// ve aşağıdaki trait tanımı ile impl'i yorumdan çıkar.
//
// #![feature(auto_traits, negative_impls)]
// auto trait Valid {}
//
// ! işareti negative bir impl'i belirtir: F, Valid'i implemente etse bile her
// Color<F>'i Valid'den çıkarır. where ifadesi Color'daki bound'u tekrar eder.
//
// impl<F> !Valid for Color<F>
// where
//     F: Fn(i8) -> i8,
// {}

// 8. Bir trait'i bir tür için nerede implemente edebilirsin? (orphan rule)
//
// Kural: bir trait'i bir tür için implemente edebilmen için, İKİSİNDEN EN AZ
// BİRİ senin crate'inde tanımlı olmalı — ya trait senin ya da tür senin.
//
// Neden böyle? Asıl amaç: tüm program boyunca her (trait, tür) ikilisi için EN
// FAZLA BİR impl olsun (buna coherence denir). Yoksa derleyici "bu tür bu trait'i
// nasıl yapıyor?" sorusuna tek ve kesin cevap veremezdi.
// Peki crate'ler AYRI AYRI derlenirken bu nasıl garanti edilir? İşte orphan kuralı
// bunun için: herhangi bir trait/tür tam olarak TEK bir crate'e ait olduğundan, o
// impl'i yazabilecek tek aday o crate olur → iki farklı crate aynı impl'i yazamaz,
// çakışma matematiksel olarak imkânsızlaşır. Dahası, şu an ortada hiç impl olmasa
// bile yasaktır: std ileride ekleyebilir ya da başka bir crate ekleyebilir;
// derleyici öbür crate'leri göremediği için "yabancı + yabancı" kategorisini
// tedbiren komple kapatır. (Bu mekanizma olmasaydı A ve B crate'leri ayrı ayrı aynı
// impl'i yazar, bir program ikisini birden kullanınca çakışırdı — ve aynı-crate
// çakışma denetimi E0119 bunu yakalayamazdı, çünkü impl'ler ayrı derlenmiş
// crate'lerde.) Adı "yetim (orphan)": ne trait'in ne de türün sahibi sen olduğunda
// o impl ortada "sahipsiz" kalır — yasak olan bu.
//
// Örnek (Display std'nin trait'i, Vec de std'nin türü; ikisi de sana yabancı):
//   impl std::fmt::Display for Vec<i32> {}   // ✗ YASAK — ikisi de yabancı (yetim)
//   impl std::fmt::Display for Color<F> {}   // ✓ OLUR — tür (Color) senin
//   impl KendiTraitin        for Vec<i32> {} // ✓ OLUR — trait senin
//
// Yabancı trait'i yabancı tür için gerçekten istiyorsan "newtype" hilesi: yabancı
// türü kendi struct'ının içine sar, böylece tür artık senin olur:
//   struct BenimVec(Vec<i32>);
//   impl std::fmt::Display for BenimVec {}    // ✓ OLUR — BenimVec senin türün
//
// Yukarıdaki Animal impl'i bu kurala uyar: hem Animal trait'i hem Color türü bu
// crate'te tanımlı.
//
// (İleri ayrıntı — şimdilik yukarıdaki "ya trait ya tür senin" kuralı yeter.)
// Yabancı bir trait'i generic bir türle implemente ederken, generic parametrenin
// yabancı bir türün İÇİNE sarılmış (covered) olması istenir; tek başına, sarılmamış
// (uncovered) bir parametre yine iki crate'in çakışmasına yol açabileceği için ek
// kurallarla kısıtlanır.
//
// RFC 3323, public bir trait'i kimin implemente edebileceğini sınırlamak için
// ayrı bir özellik öneriyor. Örneğin pub impl(crate) trait Example {}, diğer
// crate'lerin Example'ı kullanmasına izin verirken implementasyonları kendi
// crate'ine saklardı.
