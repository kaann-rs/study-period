// Rust Book — Bölüm 13: Functional Language Features — Closures & Iterators
// Çalıştır: cargo run -p closures-iterators
//
// İki "fonksiyonel" araç: CLOSURE (ortamını yakalayan isimsiz fonksiyon) ve ITERATOR (öğeleri
// tek tek, tembel üreten soyutlama). İkisi de borrow kurallarına (daha önce konuştuğumuz
// Aliasing XOR Mutability) tam uyar ve çoğu zaman zero-cost'tur.
//
// Doğrulanmış (rustc ile): dışarı taşıyan closure iki kez çağrılamaz (FnOnce) → E0382;
// borrow eden closure fonksiyonu aşarsa → E0373 (move gerek); tüketilmeyen adaptör → "iterators
// are lazy" must_use uyarısı. Ayrıntı: DERLEYİCİ PERSPEKTİFİ (altta).

fn main() {
    closurelar();
    iteratorlar();
    kendi_iteratorun();
}

// ============================================================================
// 1. CLOSURES — ortamını yakalayan isimsiz fonksiyonlar
// ============================================================================
fn closurelar() {
    // Söz dizimi: |parametreler| gövde. Türler çoğunlukla çıkarılır (fonksiyonun aksine).
    let kare = |x: i32| x * x;
    println!("kare(5) = {}", kare(5));

    // Fonksiyondan farkı: ÇEVRESİNDEKİ değişkenleri YAKALAR (capture). Üç yakalama biçimi,
    // tıpkı borrow'lar gibi — closure ne yaparsa derleyici o kadarını alır:

    // (a) PAYLAŞIMLI ödünç (&): yalnız okuyorsa.
    let liste = vec![1, 2, 3];
    let yazdir = || println!("yakaladı (&): {liste:?}");
    yazdir();
    yazdir(); // & yakaladığı için birden çok kez çağrılabilir; liste hâlâ kullanılabilir
    println!("liste hâlâ burada: {liste:?}");

    // (b) DEĞİŞTİREN ödünç (&mut): değiştiriyorsa. Closure'ın kendisi `mut` olmalı.
    let mut sayac = vec![1, 2, 3];
    let mut ekle = || sayac.push(4);
    ekle();
    println!("ekle sonrası: {sayac:?}");

    // (c) move: değeri İÇERİ TAŞIR (sahiplik). Thread'e veya fonksiyon dışına taşırken şart.
    let sahip = String::from("sahiplendim");
    let tasi = move || println!("move ile: {sahip}");
    tasi();
    // println!("{sahip}"); // ✗ sahip artık closure'da; burada E0382 olurdu

    // Fn / FnMut / FnOnce: bir closure'ı GEÇERKEN hangi trait'i uyguladığı, ne yaptığına bağlı:
    //   FnOnce  — en az bir kez çağrılabilir; yakalananı DIŞARI taşıyabilir (yalnız bir kez).
    //   FnMut   — çağrılınca yakalananı DEĞİŞTİRİR (birden çok kez, ama &mut gerektirir).
    //   Fn      — yakalananı yalnız okur/ödünç alır (serbestçe, paylaşımlı).
    // Hiyerarşi: her Fn aynı zamanda FnMut ve FnOnce'tur; her FnMut aynı zamanda FnOnce'tur.
    uygula_fn(|| println!("Fn: sadece okur"));
    let n = hesapla_sonra_calistir(|| 7 * 6);
    println!("FnOnce döndürdü: {n}");
    // Neden üç ayrı trait? Sorun: closure'lar borrow kurallarını BOZMAMALI. Bir closure
    // yakaladığını değiştiriyorsa &mut tutar; o yüzden onu Fn (paylaşımlı) kabul eden bir API'ye
    // veremezsin — tür sistemi aliasing+mutation'ı derleme anında engeller. Trait seçimi,
    // closure'ın ne kadar "yetki" aldığını imzaya taşır.
}

// Fn alır: closure'ı çok kez, paylaşımlı çağırabilir.
fn uygula_fn<F: Fn()>(f: F) {
    f();
    f();
}
// FnOnce alır: en az bir kez çağırır (sonucu döndürebilir). En gevşek kısıt.
fn hesapla_sonra_calistir<F: FnOnce() -> i32>(f: F) -> i32 {
    f()
}

// ============================================================================
// 2. ITERATORS — tembel, tek tek öğe üreten soyutlama
// ============================================================================
fn iteratorlar() {
    let v = vec![1, 2, 3, 4, 5];

    // Iterator trait'inin TEK zorunlu metodu next(): bir sonraki Some(öğe), bitince None.
    let mut it = v.iter();
    println!("next: {:?} {:?}", it.next(), it.next()); // Some(1) Some(2)

    // TEMBELLİK (lazy): adaptörler (map, filter) hiçbir şey YAPMAZ ta ki bir TÜKETİCİ gelene
    // kadar. Aşağıdaki zincir, collect çağrılana dek çalışmaz.
    let sonuc: Vec<i32> = v
        .iter() // &i32 verir
        .map(|x| x * 2) // adaptör: her öğeyi dönüştür (tembel)
        .filter(|x| x % 3 != 0) // adaptör: ele (tembel)
        .collect(); // TÜKETİCİ: şimdi gerçekten çalışır
    println!("map+filter+collect: {sonuc:?}"); // [2, 4, 8, 10]

    // Tüketici adaptörler zinciri bitirir ve tek değer üretir:
    let toplam: i32 = v.iter().sum();
    let kareler_toplami: i32 = v.iter().map(|x| x * x).sum();
    println!("toplam = {toplam}, kareler toplamı = {kareler_toplami}");

    // üç gezinme biçimi — sahiplik açısından farklılar:
    //   iter()      → &T    (ödünç; koleksiyon kalır)
    //   iter_mut()  → &mut T (değiştirerek gez)
    //   into_iter() → T      (sahipliği devralır; koleksiyon TÜKENİR)
    let mut m = vec![10, 20, 30];
    for x in m.iter_mut() {
        *x += 1;
    }
    println!("iter_mut sonrası: {m:?}"); // [11, 21, 31]
    let sahiplenen: Vec<String> = vec![String::from("a"), String::from("b")]
        .into_iter()
        .map(|s| s + "!")
        .collect();
    println!("into_iter: {sahiplenen:?}");
}

// ============================================================================
// 3. KENDİ ITERATOR'UNU YAZMAK — sadece next() uygula, gerisi bedava gelir
// ============================================================================
struct Sayac {
    sayi: u32,
}
impl Iterator for Sayac {
    type Item = u32; // associated type: ürettiği öğenin türü
    fn next(&mut self) -> Option<u32> {
        if self.sayi < 5 {
            self.sayi += 1;
            Some(self.sayi)
        } else {
            None
        }
    }
}
fn kendi_iteratorun() {
    // next() yazdık; map/filter/sum/zip... HEPSİ Iterator'ın VARSAYILAN metotları olarak
    // doğrudan çalışır — tek satır bile yazmadan.
    let toplam: u32 = Sayac { sayi: 0 }
        .zip(Sayac { sayi: 0 }.skip(1)) // (1,2),(2,3),(3,4),(4,5)
        .map(|(a, b)| a * b)
        .filter(|x| x % 3 == 0)
        .sum();
    println!("kendi iterator zinciri = {toplam}"); // 2+6+12+20=40 → %3==0 olan: 6+12=18
}

// ============================================================================
// DERLEYİCİ PERSPEKTİFİ — closure'lar nasıl yakalıyor, iterator neden zero-cost?
// ============================================================================
//
// • CLOSURE = veri + kod. Derleyici her closure için, yakaladığı değişkenleri ALAN olarak
//   tutan isimsiz bir struct üretir ve ona Fn/FnMut/FnOnce trait'lerini uygular. "& mi, &mut mı,
//   move mı?" kararını (capture mode) tür denetleyici verir: rustc_hir_typeck/src/upvar.rs.
//   Edition 2021+ (RFC 2229) ile yakalama DISJOINT'tir: closure `s.a`'yı kullanıyorsa tüm `s`'i
//   değil yalnız `s.a` alanını yakalar — bu da gereksiz borrow çakışmalarını azaltır. Hangi
//   trait? Closure yakaladığını dışarı TAŞIYORSA yalnızca FnOnce (bu yüzden iki kez çağrılamaz,
//   E0382); DEĞİŞTİRİYORSA FnMut; sadece OKUYORSA Fn. Yani trait seçimi borrow davranışının
//   doğrudan yansımasıdır — aliasing+mutation tür düzeyinde yasaklı kalır.
//
// • move yakaladığını içeri taşır. Bir closure borrow ettiği değeri AŞACAKSA (fonksiyondan
//   döndürme, thread'e verme), derleyici move ister: E0373 "closure may outlive borrowed value".
//   Sorun: aksi halde closure, düşmüş bir yereli ödünç tutan dangling bir yapı olurdu.
//
// • ITERATOR trait'inin TEK zorunlu metodu next (library/core/src/iter/traits/iterator.rs:78,
//   `fn next(&mut self) -> Option<Self::Item>`). map, filter, sum, zip... hepsi bu next üstüne
//   kurulu VARSAYILAN metotlardır. Bu yüzden kendi türüne next yazınca tüm zincir bedava gelir.
//
// • TEMBELLİK: adaptörler (map/filter) yeni bir iterator struct'ı döndürür, iş yapmaz. Zincir
//   ancak bir TÜKETİCİ (collect/sum/for) next'i çağırmaya başlayınca akar. Tüketmezsen iş olmaz
//   — derleyici "unused `Map` that must be used / iterators are lazy" uyarısı verir (DOĞRULANDI).
//
// • ZERO-COST: iterator zinciri runtime'da ek yük getirmez. Her adaptör generiktir →
//   monomorphization her closure'ı somutlaştırır, sonra LLVM inline + optimizasyonla tüm zinciri
//   elle yazılmış bir döngü kadar sıkı makine koduna indirger. Soyutlama derlemede eriyip gider;
//   "yüksek seviye yazım, düşük seviye performans" bu yüzden çelişmez.
