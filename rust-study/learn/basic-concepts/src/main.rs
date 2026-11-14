// Küçük örneklerle Rust temelleri. Önce main'deki bağlamalar (binding) ve ham
// işaretçiyle başla, sonra closure'lar ile trait'lerin nasıl oturduğunu görmek
// için test'i takip et.

use std::ptr;

fn main() {
    // 1. Değiştirilebilir bağlamalar (mutable bindings)
    // let ile tanımlanan bir bağlama varsayılan olarak değişmezdir. Yeni bir
    // değer atamak istiyorsan mut ekle; first_var'da yaptığımız gibi.
    // Hem tür açıklaması hem de literal son eki i32 belirtir.
    let mut first_var: i32 = 10_i32;

    first_var = 101;

    println!("First Variables {}", first_var);

    // 2. Ham işaretçi (raw pointer) üzerinden okuma
    // *const i32, bir i32'ye işaret eden ham işaretçidir. &s'den oluşturmak güvenlidir.
    // Üzerinden okumak unsafe gerektirir; çünkü okumanın geçerli, hizasının doğru ve
    // başlatılmış bir i32'ye işaret ettiğini bizim garanti etmemiz gerekir.
    // GÜVENLİK (SAFETY): s1, &s'den geliyor ve her iki okuma da s yaşarken oluyor.
    unsafe {
        let s: i32 = 10;

        let s1: *const i32 = &s as *const i32;

        // ptr::read, belleği değiştirmeden bitlerin bir kopyasını döndürür.
        // Burada i32 Copy olduğundan, orijinal değeri tekrar okumak sorun değil.
        // Bu çağrı sonucunu atar. Aşağıdaki dereference s'yi tekrar okur.
        ptr::read(s1);

        println!("{:?}", *s1);
    }

    test();
}

// 3. Sabitler ve const generic'ler
// Aşağıdaki N başlı başına bir sabittir. Şöyle bir tanımda:
// struct Buffer<const N: usize = 0> { data: [u8; N] }
// N ise bir const generic parametresidir. Buffer<4>, dört baytlık bir diziye sahiptir.
// Varsayılan = 0, const argüman verilmediğinde bir uzunluk sağlar.
const N: usize = 0;

// 4. Trait sınırı olan çağrılabilir bir değeri saklamak
// Color<F>, F türünde bir değer saklar. where ifadesi F'nin Fn(i8) -> i8
// implemente etmesini şart koşar; böylece bir i8 ile çağrılıp bir i8 döndürebilir.
// Fn, paylaşımlı bir referans üzerinden çağrıya izin verir. Değer yakalayan bir
// closure, gövdesi bu yakalananları uygun kullanıyorsa yine Fn'i karşılayabilir.
struct Color<F>
where
    F: Fn(i8) -> i8,
{
    x: F,
}

// 5. Trait parametreleri, ilişkili türler (associated types) ve ilişkili fonksiyonlar
// T, Animal'ın bir parametresidir; yani Animal<i32>, T = i32 belirtir.
// Her implementasyon, ilişkili tür Your için somut bir tür sağlar.
trait Animal<T> {
    type Your;

    // self parametresi olmadığından voice, tür üzerinde ilişkili bir fonksiyon
    // olarak çağrılır. Bir örnek (instance) oluşturmadan çağırabilirsin.
    fn voice() -> T;
}

// Bu trait impl'i, F üzerindeki sınırı karşılayan her Color<F> için Animal<i32>
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

// 6. Kendine ait (inherent) implementasyonlar ve fonksiyon işaretçileri
// Bu inherent impl, hex'i doğrudan Color<F>'e ekler. İlk argüman generic
// çağrılabilir tür F'ye sahiptir. İkincisi ise somut fonksiyon işaretçisi türü
// fn(i8) -> i8'dir. Bu örnek her iki biçimi de kabul eder ama yalnızca f'yi çağırır.
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

// Güvensiz (unsafe)
unsafe extern "C" fn noise() -> i8 {
    1
}

// Yüksek Dereceli (Higher-Ranked) Fonksiyonlar — HRTB
//
// Doğrulama: Nomicon (HRTB) + Rust Reference (constant promotion); davranış
// rustc ile de test edildi (aşağıdaki yerel-değişken örneği hex_v3'te E0597 verir).
//
// İki imza ömür (lifetime) bakımından farklı:
//   hex_v2: `for<'a> Fn(&'a i8)`    → HRTB. "for<'a>" = "HER 'a seçimi için"
//           (Nomicon). f tüm ömürler için çalışmak zorunda; ömür kararı İÇERİDE.
//   hex_v3: `<'a, F> ... Fn(&'a i8)` → 'a'yı ÇAĞIRAN dıştan seçer: tek, sabit bir
//           ömür. f yalnız o 'a için çalışır.
//
// Neden biri patlar? Ömürleri iç-içe kutu gibi düşün:
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
// literali 'static'e TERFİ eder (constant promotion, Rust Reference → &'static i8),
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

// ÖRNEK —farkı yerel bir değişkenle canlı gör. Gövdeler `f(&1)` yerine
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
    // Bu closure argümanını döndürür ve kapsamından (scope) hiçbir şey yakalamaz.
    let x = |y: i8| -> i8 { y };

    // İlk x kendi closure türünü korur ve F'yi belirler. İkinci x hiçbir şey
    // yakalamadığı için fonksiyon işaretçisine dönüşür (coerce).
    let a: i8 = Color::hex(x, x);

    // Derleyici, i8'i hex'in dönüş türünden de çıkarsar.
    let b = Color::hex(x, x);

    // voice argüman almaz, bu yüzden ::<...> ile somut bir F belirtiriz.
    // Bu imzaya sahip bir fonksiyon işaretçisi, Color üzerindeki sınırı karşılar.
    Color::<fn(i8) -> i8>::voice();

    println!("{:?}", a);
}

// 7. Auto trait'ler ve negatif implementasyonlar (nightly Rust)
// Auto trait, bir işaretçi (marker) trait'tir. Varsayılan olarak bir struct,
// tüm alan türleri onu implemente ediyorsa ve bunu geçersiz kılan açık bir impl
// yoksa onu implemente eder.
//
// Bu örneği denemek için feature özniteliğini dosyanın en üstüne koy ve
// aşağıdaki trait tanımı ile impl'i yorumdan çıkar.
//
// #![feature(auto_traits, negative_impls)]
// auto trait Valid {}
//
// ! işareti negatif bir impl'i belirtir. F, Valid'i implemente etse bile her
// Color<F>'i Valid'den çıkarır. where ifadesi Color'daki sınırı tekrar eder.
//
// impl<F> !Valid for Color<F>
// where
//     F: Fn(i8) -> i8,
// {}

// 8. Trait implementasyonlarına nerede izin verilir
// Rust'ın orphan (yetim) kurallarına göre, bir crate kendi tanımladığı bir
// trait'i implemente edebilir. Yabancı bir trait için, impl'deki bir türün yerel
// olması gerekir ve örtülmemiş (uncovered) generic parametrelerin nerede
// görünebileceğine dair ek kurallar işler.
// Yukarıdaki Animal impl'i, bu crate'te tanımlı bir trait ve bir tür kullanır.
//
// RFC 3323, public bir trait'i kimin implemente edebileceğini sınırlamak için
// ayrı bir özellik öneriyor. Örneğin pub impl(crate) trait Example {}, diğer
// crate'lerin Example'ı kullanmasına izin verirken implementasyonları kendi
// crate'ine saklardı.
