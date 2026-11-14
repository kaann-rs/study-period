// Sağlamlaştırma — İleri konu: SIZED, DST ve FAT POINTER
// Çalıştır: cargo run -p ileri-sized-dst
//
// Kaynak: Rust for Rustaceans Böl. 3 "Dynamically Sized Types and Fat Pointers" + std.
// Bu, daha önceki birçok parçayı tek köke bağlar: `str`/`[T]` neden hep & ya da Box ardında,
// `dyn Trait` nasıl çalışır (ileri-future'daki vtable buraya bağlanır), `?Sized` ne demek.
//
// Doğrulanmış (rustc ile): ince pointer 8 bayt; &str/&[T]/&dyn T/Box<str>/Box<dyn T> = 16 bayt
// (fat pointer); unsized değeri doğrudan kullanmak → E0277. Ayrıntı: DERLEYİCİ PERSPEKTİFİ.

use std::mem::size_of;

trait Ciz {
    fn ciz(&self) -> String;
}
struct Dugme {
    ad: String,
}
impl Ciz for Dugme {
    fn ciz(&self) -> String {
        format!("[{}]", self.ad)
    }
}

fn main() {
    boyutlar();
    sized_siniri();
    fat_pointer_anlami();
}

// ============================================================================
// 1. POINTER BOYUTLARI — fat pointer'ı GÖZLE gör
// ============================================================================
fn boyutlar() {
    let usize_boyu = size_of::<usize>(); // bir "word" (64-bit platformda 8)
    println!("usize (bir word) = {usize_boyu} bayt\n");

    println!("İNCE (thin) pointer'lar — tek word:");
    println!("  &i32       = {}", size_of::<&i32>()); // 8
    println!("  &Dugme     = {}", size_of::<&Dugme>()); // 8 (Dugme Sized)
    println!("  Box<i32>   = {}", size_of::<Box<i32>>()); // 8

    println!("\nFAT (geniş) pointer'lar — iki word (pointer + metadata):");
    println!("  &str       = {}", size_of::<&str>()); // 16: ptr + uzunluk
    println!("  &[i32]     = {}", size_of::<&[i32]>()); // 16: ptr + uzunluk
    println!("  &dyn Ciz   = {}", size_of::<&dyn Ciz>()); // 16: ptr + vtable
    println!("  Box<str>   = {}", size_of::<Box<str>>()); // 16
    println!("  Box<dyn Ciz> = {}", size_of::<Box<dyn Ciz>>()); // 16
    // Neden ikisi? Bir word veriye işaret eder; ikinci word "tipi tamamlar": slice/str için
    // UZUNLUK, trait object için VTABLE adresi. İşte fat pointer budur (RfR: "twice the size of
    // a usize").
}

// ============================================================================
// 2. SIZED SINIRI — her yerde örtük T: Sized, DST'ler istisna
// ============================================================================
// Sized, derleme anında boyutu BİLİNEN tiplerin işaret trait'idir. Çoğu tür otomatik Sized'dır.
// İKİ yaygın istisna DST'tir (dynamically sized type): slice `[T]` ve trait object `dyn Trait`
// (ve `str`). Bunların boyutu çalışma anına kadar bilinmez: elindeki `dyn Ciz` 8 baytlık mı
// 200 baytlık bir tür mü — derleyici bilemez.
//
// Derleyici neredeyse HER YERDE Sized ister: struct alanı, fonksiyon argümanı/dönüşü, yerel
// değişken, dizi elemanı. Bu yüzden yazdığın HER tür sınırı aslında "T: Sized" içerir; bundan
// `T: ?Sized` ile çıkarsın ("? = olmayabilir"). Ama DST'yi DEĞER olarak kullanamazsın:
//     fn al(_x: str) {}   // ✗ E0277: the size for values of type `str` cannot be known...
//                         //    "the trait `Sized` is not implemented for `str`"
// Çözüm: DST'leri bir POINTER ARDINA koy (&str, Box<[T]>, &dyn Trait) — pointer Sized'dır.

// ?Sized ile GENEL fonksiyon: &T alır, T Sized OLMAYABİLİR → str, [i32], dyn Ciz hepsi geçer.
// Dikkat: &T (Sized pointer) alıyoruz, T'nin kendisini değil. Varsayılan T: Sized olsaydı str
// geçmezdi; ?Sized bunu gevşetir.
fn tek_satir<T: std::fmt::Debug + ?Sized>(x: &T) {
    println!("  ?Sized ile: {x:?}");
}
fn sized_siniri() {
    println!("?Sized gevşetmesiyle DST'leri &T olarak alabiliriz:");
    tek_satir("ben bir str'yim"); // &str
    tek_satir(&[1, 2, 3][..]); // &[i32]
    let n: i32 = 42;
    tek_satir(&n); // &i32 (Sized de geçer, ?Sized "en az bunlar" demek)
}

// ============================================================================
// 3. FAT POINTER'IN İKİNCİ WORD'Ü NE TUTAR?
// ============================================================================
fn fat_pointer_anlami() {
    // SLICE/STR: ikinci word = UZUNLUK. Aynı backing array'e farklı uzunlukta dilimler:
    let dizi = [10, 20, 30, 40, 50];
    let d1: &[i32] = &dizi[..3]; // ptr=dizi başı, len=3
    let d2: &[i32] = &dizi[2..]; // ptr=dizi[2], len=3
    println!("slice fat pointer: d1.len()={} d2.len()={} (ikinci word = uzunluk)", d1.len(), d2.len());

    // TRAIT OBJECT: ikinci word = VTABLE adresi. vtable, somut türün metot adreslerini + layout/
    // align + drop'unu tutar (ileri-future'daki RawWakerVTable da bunun elle yazılmış hâliydi).
    let nesneler: Vec<Box<dyn Ciz>> = vec![
        Box::new(Dugme { ad: String::from("Tamam") }),
        Box::new(Dugme { ad: String::from("İptal") }),
    ];
    for n in &nesneler {
        // n.ciz() çağrısı: veri-pointer'dan nesneyi, vtable-pointer'dan ciz()'in adresini bulur.
        println!("trait object (vtable ile dispatch): {}", n.ciz());
    }
}

// ============================================================================
// DERLEYİCİ / std / RfR PERSPEKTİFİ
// ============================================================================
//
// • Sized bir işaret trait'idir (core::marker::Sized); derleyici otomatik uygular. Boyutu
//   derleme anında belli olan her tür Sized'dır. DST'ler (slice [T], str, dyn Trait) değildir:
//   boyutları çalışma anı bilgisine bağlıdır. Sorun: derleyici geçerli kod üretmek için boyutu
//   bilmek zorundadır (bir tuple'a kaç bayt ayrılacak, 4. alanın offset'i ne) — DST'de bu yok.
//
// • Bu yüzden Sized neredeyse HER KONUMDA zorunludur ve her tür sınırı örtük T: Sized içerir.
//   `T: ?Sized` bu örtük sınırı kaldırır; o zaman T'yi yalnızca bir pointer ardında (&T, Box<T>)
//   kullanabilirsin — çünkü POINTER Sized'dır, DST'nin kendisi değil.
//
// • FAT POINTER (RfR): bir DST'ye referans aldığında derleyici OTOMATİK iki-word'lük bir pointer
//   kurar: bir word veriye, bir word "tipi tamamlayan" metadata'ya. Slice/str için metadata =
//   UZUNLUK; trait object için = VTABLE adresi. Fat pointer'ın KENDİSİ Sized'dır (2×usize), bu
//   yüzden değişkende/alanda tutulabilir. Box ve Arc de fat pointer saklayabildiği için `T: ?Sized`
//   destekler (Box<str>, Arc<dyn Trait>).
//
// • VTABLE (dyn Trait'in ikinci word'ü): somut tür için trait metotlarının adresleri + tipin
//   layout/alignment bilgisi + drop metodu. Metot çağrısı bu tablodan adresi okuyup atlar
//   (dinamik dispatch, küçük bir dolaylama). ileri-future'daki RawWakerVTable, bunun elle yazılmış
//   özel bir hâliydi — aynı fikir. (std ayrıca std::ptr::Pointee/metadata ile bu metadata'yı
//   programatik erişilebilir yapar.)
//
// • GENERIC vs dyn (RfR tekrarı): generic<T> → monomorphization, her somut T için kod kopyası
//   (yalnız KULLANILAN parçalar kopyalanır); dyn → tek kod + vtable. Biri kod boyutu+hız, öteki
//   esneklik+küçük dolaylama. Sized/DST, bu ikisinin neden böyle çalıştığının alt yapısıdır.
