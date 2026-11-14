// Rust Book — Bölüm 8: Common Collections (Vec, String, HashMap)
// Çalıştır: cargo run -p collections
//
// Bunlar standart kütüphane türleridir (derleyici "sihri" değil); ama Vec'in güvenliği
// gibi bazı kurallar yine borrow checker tarafından zorlanır.
//
// Doğrulanmış (rustc ile): Vec borrow+push → E0502; String integer index → E0277;
// HashMap insert key taşıma → E0382; char sınırı dışı dilim → çalışma anı panik;
// "şema" = 5 byte / 4 karakter. Ayrıntı: DERLEYİCİ PERSPEKTİFİ (en altta).

use std::collections::HashMap;

fn main() {
    vektorler();
    stringler();
    hashmapler();
}

// 1. Vec<T> — büyüyebilen, sahipli, aynı türden eleman dizisi (heap'te)
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

    // ✗ BORROW + PUSH tuzağı (Rust'ın en ünlü "neden olmuyor"u):
    //   let ilk = &v[0];
    //   v.push(40);          // E0502: push &mut ister, ama `ilk` hâlâ & borç tutuyor
    //   println!("{ilk}");
    // Neden? Vec = (pointer, len, cap). push kapasiteyi aşarsa HEAP'i YENİDEN TAHSİS eder
    // (realloc): yeni blok ayrılır, eski serbest bırakılır. `ilk` eski bloğu gösterdiği için
    // dangling olurdu. Borrow checker bunu DERLEME ANINDA yasaklar → iterator invalidation
    // (koleksiyonu gezerken değiştirip bozma) daha kaynakta önlenir. (Bölüm 4'ün somut kazancı.)

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
fn stringler() {
    // String: büyüyebilen, sahipli, heap'te, UTF-8 metin. &str: ödünç alınan, sabit bir dilim.
    let mut s = String::from("mer");
    s.push_str("haba"); // &str ekler
    s.push('!'); // tek char ekler
    let d = String::from("dünya");

    let birlesik = s.clone() + " " + &d; // `+`: sol String'i TÜKETİR, sağ tarafta &str ister
    let f = format!("{s} {d}"); // format!: hiçbir şeyi tüketmez — tercih edilen birleştirme
    println!("{birlesik} / {f}");

    // UTF-8 gerçeği: "şema" 5 BYTE ama 4 KARAKTER ('ş' 2 byte tutar).
    let t = "şema";
    println!("'{t}': bytes={} chars={}", t.len(), t.chars().count()); // 5, 4

    // Integer ile indexleme YOKTUR:
    //   let c = t[0]; // ✗ E0277: the type `str` cannot be indexed by `{integer}`
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

    // entry API: "varsa dokunma, yoksa ekle" — idiomatic güncelleme
    *puan.entry(String::from("mavi")).or_insert(0) += 1; // mavi var → artır
    puan.entry(String::from("sari")).or_insert(7); // sari yok → 7 ekle

    // Kelime sayma — entry'nin klasik kullanımı
    let metin = "a b a c b a";
    let mut say: HashMap<&str, i32> = HashMap::new();
    for kelime in metin.split_whitespace() {
        *say.entry(kelime).or_insert(0) += 1;
    }
    // Gezinme SIRASI garantili DEĞİL; deterministik çıktı için sıralayıp basalım.
    let mut ciftler: Vec<_> = say.iter().collect();
    ciftler.sort();
    println!("kelime sayımı: {ciftler:?}");
    println!("puan tablosu eleman sayısı: {}", puan.len());
}

// ============================================================================
// DERLEYİCİ / KÜTÜPHANE PERSPEKTİFİ
// ============================================================================
//
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
// • HashMap<K, V>: anahtar ve değerlerin SAHİBİdir; non-Copy anahtarlar insert'te taşınır
//   (E0382). Gezinme sırası bilinçli olarak belirsizdir çünkü varsayılan hash'leyici SipHash'tir
//   ve her HashMap rastgele bir tohumla (seed) başlar — bu, kötü niyetli girdilerin kasıtlı
//   çakışma üretip performansı düşürmesini (HashDoS) engellemek içindir. Sıra gerekiyorsa
//   BTreeMap (sıralı) ya da çıktı öncesi sort kullanılır.
