// Rust Book — Bölüm 8: Common Collections (Vec, String, HashMap)
// Çalıştır: cargo run -p collections
//
// Bunlar standart kütüphane türleridir (derleyici "sihri" değil); ama Vec'in güvenliği
// gibi bazı kurallar yine borrow checker tarafından zorlanır.
//
// Doğrulanmış (rustc ile): Vec borrow+push → E0502; String integer index → E0277;
// HashMap insert key taşıma → E0382; char sınırı dışı dilim → çalışma anı panik;
// "şema" = 5 byte / 4 karakter. Ayrıntı: DERLEYİCİ PERSPEKTİFİ (en altta).

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Bu dosyadaki her bölümün üstünde "SADE ANLATIM" diye bir kutu var. Önce onu oku, sonra
// altındaki teknik açıklamaya geç. Teknik kısım zor gelirse sorun değil: sade kutu ana fikri verir.
// Benzetmelerin hepsi bir ÇOCUK ODASINDAN: oyuncak tren, boncuk kolye, etiketli çekmeceler.
//
// Bu bölümün tek cümlelik özeti: Birden çok değeri bir arada tutmanın üç yaygın yolu vardır:
// sıralı liste (Vec), yazı (String) ve "etiket → değer" eşlemesi (HashMap).
//
// MİNİ SÖZLÜK (bu dosyada geçen kelimeler):
//   koleksiyon             → birden çok değeri bir arada tutan tür.
//   Vec<T>                 → boyu büyüyüp küçülebilen, hepsi aynı türden (T) elemanların sırası.
//   vec![1, 2, 3]          → içi dolu bir Vec'i kısa yoldan kuran makro.
//   push                   → sona bir eleman eklemek.
//   index, v[0]            → sıra numarasıyla erişim. Saymaya 0'dan başlanır.
//   get                    → erişimin güvenli yolu: eleman varsa Some(..), yoksa None verir.
//   Option, Some, None     → "var, işte bu" ya da "yok" diyen kutu (enums bölümünde anlatıldı).
//   panik (panic)          → programın "burada bir yanlışlık var" deyip kendini durdurması.
//   & / &mut               → bakmak için ödünç / değiştirmek için ödünç (ownership bölümü).
//   dereference, *x        → okun gösterdiği değerin kendisine ulaşmak.
//   heap                   → boyu çalışırken değişebilen verilerin durduğu büyük bellek deposu.
//   pointer, len, cap      → Vec'in künyesi: verinin yeri, eleman sayısı, ayrılmış yer (kapasite).
//   realloc                → yer yetmeyince veriyi daha büyük bir yere taşımak (yeniden tahsis).
//   dangling               → gösterdiği veri taşınmış ya da silinmiş, boşu gösteren ok.
//   iterator invalidation  → bir listeyi gezerken onu değiştirip gezmeyi bozmak.
//   borrow checker         → derleyicinin, ödünç kurallarına uyulduğunu denetleyen bölümü.
//   String / &str          → sahibi olduğun, büyüyebilen yazı / bir yazıya (ya da parçasına) bakış.
//   char                   → tek bir harf ya da işaret.
//   byte                   → belleğin en küçük kutucuğu. Bir harf 1 ile 4 byte arası yer tutar.
//   UTF-8                  → harflerin byte'lara nasıl yazılacağını söyleyen yaygın kural.
//   dilim (slice)          → bir yazının ya da listenin bir parçasına bakış: &t[0..2].
//   clone                  → bire bir aynısını yeniden üretmek.
//   format!                → `println!` gibi, ama ekrana basmak yerine String üretir.
//   HashMap<K, V>          → "anahtar → değer" eşlemesi. K anahtarın, V değerin türüdür.
//   anahtar (key) / değer (value) → çekmecenin etiketi / çekmecenin içindeki şey.
//   insert                 → bir anahtarla birlikte değer koymak; anahtar varsa değeri değişir.
//   entry, or_insert       → "bu anahtar varsa onu ver, yoksa şu değerle oluştur" kısayolu.
//   hash                   → anahtardan, değerin saklanacağı yeri hesaplayan işlem.
//   Copy / move            → küçük değerler kopyalanır; String gibi büyükler taşınır.
//   idiomatic (deyimsel)   → o dili bilenlerin alıştığı, yerleşik yazım biçimi.
//   deterministik          → her çalıştırmada aynı sonucu veren.
//   standart kütüphane     → Rust ile birlikte gelen hazır kod; `std::` ile başlar.
//   derleyici (rustc)      → kodu programa çeviren ve hataları önceden bulan araç.

use std::collections::HashMap;

fn main() {
    vektorler();
    stringler();
    hashmapler();
}

// 1. Vec<T> — büyüyebilen, sahipli, aynı türden eleman dizisi (heap'te)
// SADE ANLATIM: Oyuncak trenini düşün: vagonlar sıra sıra dizilidir, sona yeni vagon takabilirsin
// ve her vagon aynı cins yük taşır. `Vec` böyle bir trendir. Vagonlar 0'dan numaralanır: ilk vagon
// `v[0]`dır. Trende olmayan bir vagonu `v[99]` diye istersen program durur (panikler); `v.get(99)`
// diye sorarsan "öyle bir vagon yok" cevabını (None) alırsın ve program sürer.
// Neden böyle? Kaç eleman olacağını çoğu zaman önceden bilemezsin (kaç kişi gelecek, kaç satır
// okunacak); boyu sabit bir dizi yetmez, uzayabilen bir tren gerekir.
//   Mini örnek:
//     let mut tren = vec![10, 20];        // iki vagonlu tren
//     tren.push(30);                      // sona bir vagon takıldı
//     println!("{}", tren[0]);            // ✓ 10 → ilk vagon
//     println!("{:?}", tren.get(1));      // ✓ Some(20) → o vagon var
//     println!("{:?}", tren.get(99));     // None → o vagon yok; program durmaz
fn vektorler() {
    let mut v: Vec<i32> = Vec::new();
    v.push(10);
    v.push(20);
    v.push(30);
    let v2 = vec![1, 2, 3]; // vec! makrosu ile kısa oluşturma

    // İki erişim biçimi:
    println!("v[0] = {}", v[0]); // index: sınır dışıysa PANİK
    match v.get(99) {
        Some(x) => println!("{x}"),
        None => println!("99. eleman yok (get → Option, güvenli)"),
    }

    // (Sade: `for x in &v` her vagona sırayla BAKAR. `for x in &mut v` her vagonun içini
    //  değiştirerek geçer; `*x` "okun gösterdiği vagonun içi" demektir.)
    // Gezinme
    let mut toplam = 0;
    for x in &v {
        toplam += x; // paylaşımlı ödünç
    }
    println!("toplam = {toplam}");
    for x in &mut v {
        *x += 1; // değiştirerek gezinme: *x ile dereference edip yaz
    }
    println!("{v:?} {v2:?}");

    // SADE ANLATIM: Tren uzayıp rayına sığmayınca bütün vagonlar daha uzun bir raya taşınır, eski
    // ray kaldırılır. Parmağınla eski raydaki ilk vagonu gösteriyorsan parmağın boşu gösterir.
    // Rust bu yüzden şunu der: bir vagonu gösteren parmak dururken trene vagon takamazsın.
    // Neden böyle? Boşu gösteren parmakla okunan şey rastgele bir veridir; program sessizce
    // yanlış çalışır. Kural bunu program daha derlenirken engeller.
    //   Mini örnek:
    //     let mut tren = vec![10, 20];
    //     let ilk = &tren[0];      // parmak ilk vagonda
    //     tren.push(30);           // ✗ derlenmez: parmak dururken vagon takılamaz
    //     println!("{ilk}");
    //     // ✓ Çözüm: önce `println!("{ilk}")` ile parmağın işini bitir, sonra `tren.push(30)` yaz.
    // ✗ BORROW + PUSH tuzağı (Rust'ın en ünlü "neden olmuyor"u):
    //   let ilk = &v[0];
    //   v.push(40);          // E0502: push &mut ister, ama `ilk` hâlâ & borç tutuyor
    //   println!("{ilk}");
    // Neden? Vec = (pointer, len, cap). push kapasiteyi aşarsa HEAP'i YENİDEN TAHSİS eder
    // (realloc): yeni blok ayrılır, eski serbest bırakılır. `ilk` eski bloğu gösterdiği için
    // dangling olurdu. Borrow checker bunu DERLEME ANINDA yasaklar → iterator invalidation
    // (koleksiyonu gezerken değiştirip bozma) daha kaynakta önlenir. (Bölüm 4'ün somut kazancı.)

    // SADE ANLATIM: Trenin vagonları aynı cins yük taşır. Bir vagona sayı, ötekine yazı koymak
    // istersen ikisini de aynı cins KUTUYA koyarsın: kutunun üstünde "içimde sayı var" ya da
    // "içimde yazı var" yazar. Bu kutu bir enum'dur; tren artık "kutu" taşıyan bir trendir.
    //   Mini örnek:
    //     enum Kutu { Sayi(i32), Yazi(String) }
    //     let tren = vec![Kutu::Sayi(3), Kutu::Yazi(String::from("top"))];   // ✓ hepsi Kutu
    //     let karisik = vec![3, "top"];        // ✗ derlenmez: biri sayı, öteki yazı
    // Vec'te "farklı türler" tutmak: hepsini saran bir enum kullan.
    #[derive(Debug)]
    enum Hucre {
        Sayi(i32),
        Metin(String),
        Bos,
    }
    let satir = vec![Hucre::Sayi(3), Hucre::Metin(String::from("merhaba")), Hucre::Bos];
    println!("satır = {satir:?}");
    // Payload'ları GERÇEKTEN okuyalım: yoksa (Bölüm 5'teki doğrulanmış tuzak) derive(Debug)
    // dead_code analizinde yok sayıldığı için "field never read" uyarısı çıkar.
    for h in &satir {
        match h {
            Hucre::Sayi(n) => println!("  sayı: {n}"),
            Hucre::Metin(m) => println!("  metin: {m}"),
            Hucre::Bos => println!("  boş"),
        }
    }
}

// 2. String ve &str — UTF-8, neden integer index yok
// SADE ANLATIM: Harf boncuklarından bir kolye düşün. Boncukların boyu aynı değildir: 'a' kısa bir
// boncuktur (1 birim), 'ş' uzun bir boncuktur (2 birim). "şema" kolyesinde 4 boncuk vardır ama
// kolyenin boyu 5 birimdir. "Kolyenin 0. birimini ver" dersen uzun boncuğun yarısı gelirdi; yarım
// boncuk bir harf değildir. Rust bu yüzden yazıda `t[0]` yazdırmaz; harfleri `chars()` ile tek tek
// gezersin. Buradaki "birim" byte'tır, boncuk ise char.
// Neden böyle? Dünyadaki bütün harfler tek byte'a sığmaz; bazı harfler 2, 3 ya da 4 byte tutar.
//   Mini örnek:
//     let t = "şema";
//     println!("{}", t.len());               // ✓ 5 → kolyenin boyu (byte)
//     println!("{}", t.chars().count());     // ✓ 4 → boncuk sayısı (harf)
//     println!("{:?}", t.chars().next());    // ✓ Some('ş') → ilk harf
//     let ilk = t[0];                        // ✗ derlenmez: yazıya sıra numarasıyla erişilmez
fn stringler() {
    // String: büyüyebilen, sahipli, heap'te, UTF-8 metin. &str: ödünç alınan, sabit bir dilim.
    let mut s = String::from("mer");
    s.push_str("haba"); // &str ekler
    s.push('!'); // tek char ekler
    let d = String::from("dünya");

    // (Sade: `+` soldaki yazıyı alır ve sağdakini onun sonuna ekler; soldaki değişken artık
    //  kullanılamaz, bu yüzden burada önce `clone` ile kopyası çıkarıldı. `format!` ise hiçbirine
    //  dokunmadan yepyeni bir yazı üretir.)
    let birlesik = s.clone() + " " + &d; // `+`: sol String'i TÜKETİR, sağ tarafta &str ister
    let f = format!("{s} {d}"); // format!: hiçbir şeyi tüketmez — tercih edilen birleştirme
    println!("{birlesik} / {f}");

    // UTF-8 gerçeği: "şema" 5 BYTE ama 4 KARAKTER ('ş' 2 byte tutar).
    let t = "şema";
    println!("'{t}': bytes={} chars={}", t.len(), t.chars().count()); // 5, 4

    // Integer ile indexleme YOKTUR:
    //   let c = t[0]; // ✗ E0277: the type `str` cannot be indexed by `{integer}`
    // (Sade: O(1) = "yazı ne kadar uzun olursa olsun tek adımda" demek.)
    // Neden? char'lar DEĞİŞKEN genişlikte (1–4 byte). t[0] "ilk byte mı yoksa ilk karakter mi?"
    // belirsiz; ayrıca "n. karakter"e O(1) erişim mümkün değil (baştan saymak gerekir). Rust
    // bu maliyeti ve belirsizliği gizlemek yerine integer index'i hiç sunmaz.

    // Byte-aralığı dilim ALINABİLİR ama bir char SINIRINA denk gelmeli:
    //   &t[0..1]  // çalışma anı PANİK: "byte index 1 is not a char boundary" ('ş' 0..2 arası)
    // Doğru yol: karakter bazlı işle.
    let ilk_char: String = t.chars().take(1).collect();
    println!("ilk karakter: {ilk_char}");
    print!("'ab' byte'ları: ");
    for b in "ab".bytes() {
        print!("{b} ");
    }
    println!();
}

// 3. HashMap<K, V> — anahtar → değer eşlemesi
// SADE ANLATIM: Üstünde etiket olan çekmeceler düşün: "mavi" yazan çekmecede 10, "kirmizi" yazan
// çekmecede 50 duruyor. Bir şeyi sıra numarasıyla değil ETİKETİYLE bulursun. `HashMap` budur:
// etiket anahtardır, çekmecenin içindeki de değerdir. Aynı etiketten iki çekmece olmaz; aynı
// etiketle yeniden koyarsan eskisinin yerine geçer. Olmayan bir etiketi sorarsan None alırsın.
// Neden böyle? Listede (Vec) bir şeyi bulmak için baştan sona tek tek bakarsın; etiketli
// çekmecede doğrudan doğru çekmeceye gidersin.
//   Mini örnek:
//     let mut puan = HashMap::new();
//     puan.insert("mavi", 10);                  // "mavi" çekmecesine 10 kondu
//     println!("{:?}", puan.get("mavi"));       // ✓ Some(10)
//     println!("{:?}", puan.get("mor"));        // None → öyle bir çekmece yok
//     *puan.entry("mavi").or_insert(0) += 1;    // ✓ çekmece vardı; içindeki 11 oldu
//     *puan.entry("mor").or_insert(0) += 1;     // ✓ çekmece yoktu; 0 ile açıldı, 1 oldu
fn hashmapler() {
    let mut puan: HashMap<String, i32> = HashMap::new();
    puan.insert(String::from("mavi"), 10);
    puan.insert(String::from("kirmizi"), 50);

    // insert, non-Copy KEY'i taşır:
    //   let k = String::from("yesil");
    //   puan.insert(k, 5);
    //   println!("{k}"); // ✗ E0382: borrow of moved value: `k` (HashMap artık sahibi)

    match puan.get("mavi") {
        Some(v) => println!("mavi = {v}"), // get → Option<&V>
        None => println!("yok"),
    }

    // (Sade: `entry(..).or_insert(0)` şunu der: "bu etiketli çekmece varsa onu aç; yoksa içine 0
    //  koyup yeni bir çekmece yap ve onu aç". Baştaki `*`, açılan çekmecenin içine ulaşır.)
    // entry API: "varsa dokunma, yoksa ekle" — idiomatic güncelleme
    *puan.entry(String::from("mavi")).or_insert(0) += 1; // mavi var → artır
    puan.entry(String::from("sari")).or_insert(7); // sari yok → 7 ekle

    // Kelime sayma — entry'nin klasik kullanımı
    let metin = "a b a c b a";
    let mut say: HashMap<&str, i32> = HashMap::new();
    for kelime in metin.split_whitespace() {
        *say.entry(kelime).or_insert(0) += 1;
    }
    // (Sade: çekmeceler odada etiket sırasıyla dizili durmaz; hepsini gezdiğinde hangisinin önce
    //  geleceği belli değildir. Sıralı bir çıktı istiyorsan önce kendin sıralarsın.)
    // Gezinme SIRASI garantili DEĞİL; deterministik çıktı için sıralayıp basalım.
    let mut ciftler: Vec<_> = say.iter().collect();
    ciftler.sort();
    println!("kelime sayımı: {ciftler:?}");
    println!("puan tablosu eleman sayısı: {}", puan.len());
}

// SADE ANLATIM (aşağıdaki bölüm için): Bu bölüm üç koleksiyonun perde arkasını anlatır. Trenin
// neden bazen başka raya taşındığını (Vec), kolyenin aslında byte'lardan oluşan bir tren olduğunu
// (String) ve çekmecelerin neden her çalıştırmada başka sırayla gezildiğini (HashMap) söyler.
// Program yazarken bilmen gereken kurallar yukarıdaki üç bölümdedir. İlk okuyuşta bu bölümü
// atlayabilirsin.
// ============================================================================
// DERLEYİCİ / KÜTÜPHANE PERSPEKTİFİ
// ============================================================================
//
// (Sade: amortize O(1) = arada bir pahalı taşınma olsa da ortalamada her ekleme ucuzdur.)
// • Vec<T> bellekte üç alandır: (heap'e pointer, len = eleman sayısı, cap = ayrılmış yer).
//   push, cap dolmuşsa daha büyük yeni bir heap bloğu ayırır, eski elemanları kopyalar, eskiyi
//   serbest bırakır (amortize O(1)). İşte bu YENİDEN TAHSİS yüzünden, bir elemana & borç
//   tutarken push (&mut) yasaktır — eski bloğa işaret eden referans dangling olurdu. Bunu
//   zorlayan derleyicidir: borrow checker (rustc_borrowck), E0502. Yani güvenlik kütüphanede
//   değil, dilin ödünç kurallarında.
//
// • String = içinde geçerli UTF-8 olduğu GARANTİ edilen bir Vec<u8>. `Index<usize>` IMPL
//   EDİLMEMİŞTİR; bu yüzden s[0] bir tür hatasıdır (E0277), çalışma anı kontrolü değil. Byte
//   aralığıyla dilim alınır ama sınır bir char ortasına düşerse çalışma anında panik olur.
//   Üç bakış açısı: .bytes() (ham byte), .chars() (Unicode scalar), ayrıca grapheme (harici crate).
//
// (Sade: tohum (seed) = hash hesabına katılan rastgele sayı. Her çalıştırmada değiştiği için
//  çekmecelerin gezilme sırası da değişebilir.)
// • HashMap<K, V>: anahtar ve değerlerin SAHİBİdir; non-Copy anahtarlar insert'te taşınır
//   (E0382). Gezinme sırası bilinçli olarak belirsizdir çünkü varsayılan hash'leyici SipHash'tir
//   ve her HashMap rastgele bir tohumla (seed) başlar — bu, kötü niyetli girdilerin kasıtlı
//   çakışma üretip performansı düşürmesini (HashDoS) engellemek içindir. Sıra gerekiyorsa
//   BTreeMap (sıralı) ya da çıktı öncesi sort kullanılır.
