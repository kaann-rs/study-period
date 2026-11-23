// Sağlamlaştırma — İleri konu: SIZED, DST ve FAT POINTER
// Çalıştır: cargo run -p ileri-sized-dst
//
// Kaynak: Rust for Rustaceans Böl. 3 "Dynamically Sized Types and Fat Pointers" + std.
// ↳ DÜZELTME NOTU (son gözden geçirme): kitapta bu başlık Böl. 2'dedir (Types) ve adı
//   "Dynamically Sized Types and Wide Pointers"tır; "fat pointer" ile "wide pointer" aynı
//   şeyin iki adıdır. Yayıncının ayrıntılı içindekiler listesiyle doğrulandı.
// Bu, daha önceki birçok parçayı tek köke bağlar: `str`/`[T]` neden hep & ya da Box ardında,
// `dyn Trait` nasıl çalışır (ileri-future'daki vtable buraya bağlanır), `?Sized` ne demek.
//
// Doğrulanmış (rustc ile): ince pointer 8 bayt; &str/&[T]/&dyn T/Box<str>/Box<dyn T> = 16 bayt
// (fat pointer); unsized değeri doğrudan kullanmak → E0277. Ayrıntı: DERLEYİCİ PERSPEKTİFİ.

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Bu dosyadaki her bölümün üstünde "SADE ANLATIM" diye bir kutu var. Önce onu oku, sonra
// altındaki teknik açıklamaya geç. Teknik kısım zor gelirse sorun değil: sade kutu ana fikri verir.
//
// Bu dosyayı okumadan önce bilmen gerekenler: `learn/ownership` (referans ve dilim),
// `learn/generics-traits-lifetimes` (generic ve trait), `learn/smart-pointers` (Box) ve
// `learn/oop` (`dyn Trait`, yani trait object).
//
// Bu bölümün tek cümlelik özeti: Rust bir değeri saklamak için boyunu önceden bilmek ister;
// boyu önceden bilinmeyen şeyleri (yazı, dilim, `dyn Trait`) ancak bir işaretçinin ardında
// tutar ve o işaretçi adresin yanında eksik bilgiyi de taşır.
//
// MİNİ SÖZLÜK (bu dosyada geçen kelimeler):
//   bayt (byte)          → belleğin en küçük kutusu. Boyutlar bayt ile ölçülür.
//   word / usize         → bilgisayarın bir seferde taşıdığı parça. 64-bit makinede 8 bayt.
//   pointer (işaretçi)   → bir verinin bellekteki adresini tutan ok. `&T` ve `Box<T>` böyledir.
//   Sized                → boyu derleme sırasında bilinen türlerin işareti (i32, String, Dugme).
//   işaret trait'i       → metodu olmayan, yalnız "bu türde şu özellik var" diyen trait.
//   DST                  → dynamically sized type: boyu ancak program çalışırken belli olan tür.
//   slice `[T]` / `str`  → art arda dizilmiş elemanlar / yazı. Kaç tane olduğu türde yazmaz.
//   trait object (dyn)   → gerçek türü gizlenmiş, yalnız trait'in metotları bilinen değer.
//   ince (thin) pointer  → yalnız adres taşıyan işaretçi. Tek word.
//   fat / wide pointer   → adres + bir bilgi daha taşıyan işaretçi. İki word.
//   metadata             → o ek bilgi: dilimde uzunluk, `dyn Trait`'te vtable'ın adresi.
//   vtable               → bir türün "hangi metot hangi adreste" listesini tutan tablo.
//   dispatch (dinamik)   → çağrılacak fonksiyonu program çalışırken vtable'dan bulmak.
//   ?Sized               → "bu türün boyu bilinmeyebilir, yine de kabul ediyorum" izni.
//   generic              → türü sonradan doldurulan kalıp: `fn f<T>(x: T)`.
//   monomorphization     → derleyicinin her kullanılan tür için generic kodun ayrı kopyasını
//                          üretmesi.
//   layout / alignment   → bir türün bellekte nasıl dizildiği / hangi sınıra hizalandığı.
//   offset               → bir alanın, yapının başından kaç bayt ileride durduğu.
//   E0277                → derleyicinin "bu tür gereken trait'i sağlamıyor" hata numarası.

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
// SADE ANLATIM: Birine evini tarif etmek için tek kâğıt yeter: üstüne adresi yazarsın. Ama
// "şu sokaktaki evler" dersen ikinci bir kâğıt gerekir: "kaç ev?" Bir de "ne tür bir bina
// olduğunu söylemeyeceğim" dersen ikinci kâğıda "kullanma kılavuzu şurada" yazman gerekir.
// Rust'ta sıradan referans tek kâğıttır (ince işaretçi, 8 bayt). Dilim, yazı ve `dyn Trait`
// referansları çift kâğıttır (fat pointer, 16 bayt): adres + uzunluk ya da adres + vtable.
// Bu neden var? Bu türlerin boyu türün adında yazmaz; eksik bilgi bir yerde durmak zorunda,
// o yer de işaretçinin kendisidir.
//   Mini örnek (64-bit makinede):
//     size_of::<&i32>()       // 8  → tek kâğıt: yalnız adres
//     size_of::<&[i32]>()     // 16 → adres + "kaç eleman var"
//     size_of::<&str>()       // 16 → adres + "kaç bayt var"
//     size_of::<&dyn Ciz>()   // 16 → adres + "metot listesi (vtable) nerede"
//     size_of::<Box<i32>>()   // 8  → Box da bir işaretçidir; aynı kural geçerli
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
// SADE ANLATIM: Kargo şirketi rafına koyacağı her kutunun boyunu ÖNCEDEN bilmek ister, yoksa
// rafta ne kadar yer ayıracağını bilemez. `Sized` bu demektir: "boyum belli". Bir ip yumağı
// ise ne kadar uzun olduğunu söylemez; onu rafa doğrudan koyamazsın. Ama ipin ucuna bağlı
// küçük bir etiket hep aynı boydadır ve rafa sığar. Rust'ta `str`, `[T]` ve `dyn Trait` o ip
// yumağıdır; `&str` ya da `Box<str>` ise etikettir. Rust her generic `T` için sormadan "boyu
// belli" varsayar; `?Sized` yazınca "belli olmasa da olur" demiş olursun.
//   Mini örnek:
//     fn al(x: str) {}                      // ✗ derlenmez: str'nin boyu önceden bilinmiyor
//     fn al_ok(x: &str) {}                  // ✓ referans hep aynı boyda
//     fn yaz<T: ?Sized + Debug>(x: &T) {}   // "T'nin boyu bilinmeyebilir" izni
//     yaz("yazı");                          // ✓ burada T = str
//     fn yaz2<T: Debug>(x: &T) {}           // izin yok: T'nin boyu belli olmalı
//     // yaz2("yazı");                      // ✗ derlenmez: T = str olurdu, str Sized değil
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
// (Sade: "backing array" = dilimin baktığı asıl dizi demek.)
// (Sade: "dispatch" = çağrılacak fonksiyonu seçip ona atlamak demek.)
// SADE ANLATIM: Bir kitabın tamamını değil, "40. sayfadan başla, 3 sayfa oku" diye bir
// aralığını tarif edebilirsin. Dilimin ikinci bilgisi budur: kaç tane. `dyn Trait`'te ise
// ikinci bilgi bir telefon rehberi gibidir: değerin gerçek türünü bilmezsin, ama rehberde
// "çizmek için şu numarayı ara" yazar. Program `n.ciz()` görünce rehbere (vtable) bakar ve
// doğru fonksiyona gider. Bu neden var? Aynı listede farklı türleri tutabilmek için: liste
// "çizilebilen şeyler" der, her elemanın rehberi kendi türüne göre doludur.
//   Mini örnek:
//     let dizi = [10, 20, 30, 40, 50];
//     let d: &[i32] = &dizi[1..4];          // adres: dizi[1], uzunluk: 3
//     println!("{}", d.len());              // ✓ 3 → uzunluk işaretçinin içinden okunur
//     let n: Box<dyn Ciz> = Box::new(Dugme { ad: String::from("Tamam") });
//     println!("{}", n.ciz());              // ✓ [Tamam] → ciz'in adresi vtable'dan bulunur
//     // println!("{}", n.ad);              // ✗ derlenmez: dyn Ciz yalnız Ciz'in metotlarını
//     //                                    //   bilir, Dugme'nin alanlarını göremez
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

// SADE ANLATIM (aşağıdaki bölüm için): Yukarıda üç şey gördün: bazı işaretçiler çift kâğıttır,
// Rust her yerde "boyu belli" tür ister, ikinci kâğıtta uzunluk ya da metot listesi durur.
// Aşağıdaki bölüm aynı konuyu derleyicinin gözünden anlatır: boyu bilmeden neden kod
// üretemediğini, vtable'ın içinde tam olarak ne durduğunu ve generic ile `dyn` arasındaki
// farkı (her tür için ayrı kopya üretmek ya da tek kod + tablo) açıklar. İlk okuyuşta
// atlayabilirsin; yukarıdaki üç bölümü anlamış olman yeter.
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
