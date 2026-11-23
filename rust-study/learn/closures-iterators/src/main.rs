// Açıklama: learn/closures-iterators/ACIKLAMA.md
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

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Her bölümün başındaki "SADE ANLATIM" kutusunu önce oku, sonra teknik açıklamaya geç.
// Bu dosyadaki benzetmelerin hepsi aynı yerde geçer: bir kargo şubesinde (görev notları ve
// paketlerin aktığı taşıma bandı).
//
// Bu bölümün tek cümlelik özeti: Closure, çevresindeki değişkenleri hatırlayan küçük bir
// fonksiyondur; iterator ise elemanları tek tek, istendikçe veren bir sıradır.
//
// MİNİ SÖZLÜK (bu dosyada geçen kelimeler):
//   closure                  → `|x| x + 1` gibi yazılan adsız, küçük fonksiyon.
//   ortam / yakalama         → closure'ın yazıldığı yerdeki değişkenler / onları içine alması
//                              (capture).
//   ödünç (& ve &mut)        → veriye sahibi olmadan bakmak (&) ya da onu değiştirmek (&mut).
//   move                     → "yakaladıklarını ödünç alma, içine taşı" diyen anahtar kelime.
//   trait                    → bir türün "şunu yapabilirim" diye verdiği söz (Böl. 10).
//   Fn / FnMut / FnOnce      → closure çeşitleri: okur / değiştirir / en fazla bir kez çalışır.
//   iterator                 → elemanları sırayla, tek tek veren şey.
//   next()                   → "sıradakini ver": Some(eleman) ya da bittiyse None.
//   tembel (lazy)            → istenmedikçe iş yapmayan.
//   adaptör                  → bir iterator'ı alıp başka bir iterator veren ara adım: map, filter.
//   tüketici (consumer)      → sırayı sonuna kadar çalıştırıp sonuç üreten adım: collect, sum.
//   map / filter             → her elemanı dönüştür / şarta uyanları bırak.
//   collect / sum            → elemanları bir kapta topla / hepsini topla (aritmetik).
//   zip / skip               → iki sırayı eşleştir / baştan şu kadarını atla.
//   iter/iter_mut/into_iter  → elemanlara bakarak / değiştirerek / sahiplenerek gezmek.
//   Item (associated type)   → bir iterator'ın verdiği elemanın türü.
//   Aliasing XOR Mutability  → "ya birçok okuyan ya tek yazan" kuralı (bkz. ownership).
//   thread (iş parçacığı)    → programın içinde aynı anda çalışan ayrı bir iş kolu.
//   API                      → bir fonksiyonun ya da kütüphanenin dışarıya açtığı kullanım biçimi.
//   zero-cost                → kolaylığın çalışan programı yavaşlatmaması.
//   monomorphization, inline → derleyicinin generic kodu türe göre kopyalaması / çağrıyı yerine
//                              gövdeyi yapıştırması (Böl. 10).
//   LLVM                     → rustc'nin makine kodu üretirken kullandığı araç.

mod derinlesme;
fn main() {
    closurelar();
    iteratorlar();
    kendi_iteratorun();
    derinlesme::calistir();
}

// ============================================================================
// 1. CLOSURES — ortamını yakalayan isimsiz fonksiyonlar
// ============================================================================
// SADE ANLATIM: Kargo şubesinde iş arkadaşına bir görev notu bırakırsın: "Gelen her paketin
// fiyatına masadaki listede yazan vergiyi ekle." Not kısadır, adı yoktur ve en önemlisi yazıldığı
// yerdeki bir şeyi (masadaki listeyi) bilir. Closure bu nottur: `|x| ...` biçiminde yazılır ve
// çevresindeki değişkenleri kullanabilir. Sıradan fonksiyon (`fn`) ise yalnız kendisine
// parametre olarak verileni görür. Neden böyle? Closure olmasaydı, küçük bir iş için bile adlı
// bir fonksiyon yazıp ihtiyacı olan her değeri ona tek tek parametre diye taşırdın.
//   Mini örnek:
//     let vergi = 20;
//     let fiyat = |x: u32| x + x * vergi / 100;   // vergi'yi çevresinden aldı
//     fiyat(100);                                 // ✓ 120
//     fn fiyat2(x: u32) -> u32 { x + x * vergi / 100 }   // ✗ derlenmez: fn, vergi'yi göremez
fn closurelar() {
    // Söz dizimi: |parametreler| gövde. Türler çoğunlukla çıkarılır (fonksiyonun aksine).
    let kare = |x: i32| x * x;
    println!("kare(5) = {}", kare(5));

    // SADE ANLATIM: Not masadaki listeyi üç biçimde kullanabilir. "Listeye bak": liste masada
    // kalır, sen de bakabilirsin (&). "Listeye bir satır ekle": not işini bitirene kadar listeye
    // başkası dokunmaz (&mut). "Listeyi yanına al": liste artık notu taşıyanındır, masa boşalır
    // (move). Hangisinin gerektiğine derleyici, closure'ın gövdesine bakarak karar verir ve
    // gerekenden fazlasını almaz; `move` yazarsan üçüncüsüne sen zorlarsın.
    //   Mini örnek:
    //     let liste = vec![1, 2];
    //     let bak = || println!("{liste:?}");          // yalnız okuyor: & ile yakaladı
    //     bak();
    //     println!("{liste:?}");                       // ✓ liste hâlâ bizde
    //     let gotur = move || println!("{liste:?}");   // move: liste closure'ın içine taşındı
    //     gotur();
    //     println!("{liste:?}");                       // ✗ derlenmez: liste artık bizde değil
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

    // SADE ANLATIM: Üç çeşit görev notu vardır. "Panodaki saati oku": istediğin kadar
    // uygularsın, hiçbir şey değişmez (Fn). "Sayaca bir ekle": yine tekrar tekrar uygularsın, ama
    // her seferinde sayaç değişir (FnMut). "Bu paketi alıcısına teslim et": paket verilince elinde
    // kalmaz; bu not EN FAZLA bir kez uygulanabilir (FnOnce). Neden üç ayrı ad? Bir fonksiyon
    // parametre olarak closure alırken ona ne kadar izin verdiğini imzasında söylemelidir;
    // derleyici de "iki kez çağıracağım" diyen yere tek kullanımlık not verilmesini engeller.
    //   Mini örnek:
    //     let oku = || println!("saat 9");     // Fn: istediğin kadar çağır
    //     let mut sayac = 0;
    //     let mut artir = || sayac += 1;       // FnMut: her çağrıda sayac değişir
    //     let paket = String::from("kitap");
    //     let teslim = move || paket;          // FnOnce: paketi dışarı verir
    //     teslim();                            // ✓ ilk çağrı: paket teslim edildi
    //     teslim();                            // ✗ derlenmez: verilecek paket kalmadı
    // Fn / FnMut / FnOnce: bir closure'ı GEÇERKEN hangi trait'i uyguladığı, ne yaptığına bağlı:
    //   FnOnce  — en fazla bir kez çağrılabilir; yakalananı DIŞARI taşıyabilir (hiç çağrılmayabilir).
    //   Her closure en az FnOnce'ı UYGULAR (Fn ⊂ FnMut ⊂ FnOnce).
    //   FnMut   — çağrılınca yakalananı DEĞİŞTİRİR (birden çok kez, ama &mut gerektirir).
    //   Fn      — yakalananı yalnız okur/ödünç alır (serbestçe, paylaşımlı).
    uygula_fn(|| println!("Fn: sadece okur"));
    let n = hesapla_sonra_calistir(|| 7 * 6);
    println!("FnOnce döndürdü: {n}");
    // Neden üç trait? Değiştiren closure &mut tutar; onu Fn bekleyen API'ye veremezsin. Seçim,
    // closure'ın yetkisini imzaya taşır; aliasing+mutation tür düzeyinde yasaklı kalır.
}

// Fn alır: closure'ı çok kez, paylaşımlı çağırabilir.
fn uygula_fn<F: Fn()>(f: F) {
    f();
    f();
}
// FnOnce alır: en fazla bir kez çağırır (hiç çağırmayabilir); sonucu döndürebilir. En gevşek kısıt.
fn hesapla_sonra_calistir<F: FnOnce() -> i32>(f: F) -> i32 {
    f()
}

// ============================================================================
// 2. ITERATORS — tembel, tek tek öğe üreten soyutlama
// ============================================================================
// SADE ANLATIM: Şubede paketler bir taşıma bandından gelir. Bandın ucunda durur, "sıradaki"
// dersin; bant bir paket verir. Paket kalmayınca "bitti" der. Iterator bu banttır: tek işi
// `next()` ile sıradaki elemanı vermektir; eleman varsa `Some(eleman)`, yoksa `None` döner.
// Neden böyle? Sırayla gezmenin tek bir ortak yolu olunca, liste de olsa dosya satırları da olsa
// aynı araçlar (map, filter, sum) hepsinde çalışır.
//   Mini örnek:
//     let v = vec![10, 20];
//     let mut bant = v.iter();
//     bant.next();    // Some(&10)
//     bant.next();    // Some(&20)
//     bant.next();    // None: bant boşaldı
fn iteratorlar() {
    let v = vec![1, 2, 3, 4, 5];

    // Iterator trait'inin TEK zorunlu metodu next(): bir sonraki Some(öğe), bitince None.
    let mut it = v.iter();
    println!("next: {:?} {:?}", it.next(), it.next()); // Some(1) Some(2)

    // SADE ANLATIM: Banda istasyonlar eklersin: biri her paketin ağırlığını iki katına çevirir
    // (map), öteki bazı paketleri bantta bırakmaz (filter). İstasyonları dizmek bandı
    // ÇALIŞTIRMAZ; bant, ucunda biri "sıradaki" demeye başlayınca döner. Bunu yapan adıma
    // tüketici denir: `collect`, `sum`, `for`. Neden böyle? İstenmeyen paket için emek harcanmaz;
    // ilk iki paketi isteyen biri için bandın tamamı işlenmez.
    //   Mini örnek:
    //     let v = vec![1, 2, 3];
    //     v.iter().map(|x| println!("{x}"));    // ✗ hiçbir şey yazmaz: isteyen yok (uyarı verir)
    //     let iki_kat: Vec<i32> = v.iter().map(|x| x * 2).collect();   // ✓ [2, 4, 6]
    // TEMBELLİK: adaptörler (map, filter) tüketici gelene kadar iş yapmaz; zincir collect'te çalışır.
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

    // SADE ANLATIM: Raftaki paketleri üç biçimde gezebilirsin: yalnız etiketlerine bakarak
    // (`iter`), üstlerine not yazarak (`iter_mut`) ya da hepsini alıp götürerek (`into_iter`).
    // İlk ikisinde paketler rafta kalır; üçüncüsünde raf boşalır. Bu, closure'ın üç yakalama
    // biçimiyle aynı ayrımdır: bak, değiştir, sahiplen.
    //   Mini örnek:
    //     let mut v = vec![1, 2];
    //     for x in v.iter() { println!("{x}"); }        // bak: v yerinde duruyor
    //     for x in v.iter_mut() { *x += 1; }            // değiştir: v = [2, 3]
    //     for x in v.into_iter() { println!("{x}"); }   // sahiplen: v tükendi
    //     println!("{v:?}");                            // ✗ derlenmez: v artık yok
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
// SADE ANLATIM: Şubedeki sıra numarası makinesini düşün: düğmeye basınca sıradaki numarayı
// verir, kâğıdı bitince vermez. Makinenin bilmesi gereken tek şey "şu an kaçtayım"dır. Kendi
// iterator'ını yazmak bu makineyi yapmaktır: yalnız `next`i yazarsın. `map`, `filter`, `sum`
// gibi onlarca metot `next`in üstüne kurulu olduğu için kendiliğinden gelir. Neden böyle?
// Her yeni sıra türü için aynı yardımcıları yeniden yazmak gerekmesin diye.
//   Mini örnek:
//     struct GeriSay { n: u32 }
//     impl Iterator for GeriSay {
//         type Item = u32;                          // verdiği elemanın türü
//         fn next(&mut self) -> Option<u32> {
//             if self.n == 0 { None } else { self.n -= 1; Some(self.n + 1) }
//         }
//     }
//     GeriSay { n: 3 }.sum::<u32>();    // ✓ 3 + 2 + 1 = 6 — sum'ı biz yazmadık, hazır geldi
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

// SADE ANLATIM (aşağıdaki bölüm için): Aşağısı iki soruyu cevaplar. Birincisi: closure "çevresini
// hatırlıyor" derken bu bilgi nerede duruyor? Cevap: derleyici her closure için gizli, küçük bir
// struct üretir; yakalanan değişkenler onun alanlarıdır. İkincisi: bu kadar istasyon dizmek
// programı yavaşlatmaz mı? Cevap: hayır; derleyici zinciri, elle yazılmış bir döngüyle aynı
// makine koduna çevirir. İlk okuyuşta atlayabilirsin; yukarıdaki üç bölümü anlamış olman yeter.
// ============================================================================
// DERLEYİCİ PERSPEKTİFİ — closure'lar nasıl yakalıyor, iterator neden zero-cost?
// ============================================================================
//
// (Sade: aşağıdaki "DISJOINT yakalama", closure'ın bir struct'ın tamamını değil, yalnız kullandığı
// alanını yakalaması demek: `s.a`yı kullanan closure `s.b`ye dokunmanı engellemez.)
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
// • ITERATOR'ın TEK zorunlu metodu next (library/core/src/iter/traits/iterator.rs:78,
//   `fn next(&mut self) -> Option<Self::Item>`); map/filter/sum/zip bu next üstüne kurulu VARSAYILAN metotlar.
//
// • TEMBELLİK: adaptörler (map/filter) yeni bir iterator struct'ı döndürür, iş yapmaz. Zincir
//   ancak bir TÜKETİCİ (collect/sum/for) next'i çağırmaya başlayınca akar. Tüketmezsen iş olmaz
//   — derleyici "unused `Map` that must be used / iterators are lazy" uyarısı verir (DOĞRULANDI).
//
// • ZERO-COST (çoğu zaman): adaptörler generiktir → monomorphization her closure'ı somutlaştırır;
//   optimize derlemelerde (ör. `--release`) LLVM inline + optimizasyonla zincir çoğu zaman elle
//   yazılmış döngüyle aynı koda iner. `opt-level = 0` derlemesinde dolaylama kalabilir; "aynı
//   makine kodu" garanti değildir (Book ch13-04: "in many cases").
