// Açıklama: learn/enums/ACIKLAMA.md
// Rust Book — Bölüm 6: Enums (numaralandırma) ve Pattern Matching (desen eşleme)
// Çalıştır: cargo run -p enums
//
// Stil: terim (İngilizce) + Türkçe açıklama + örnek + Neden? + Sorun + rustc: notu.
// Doğrulanmış (rustc ile derlenerek): non-exhaustive match → E0004; Option'ın niche
// optimization boyutları. Ayrıntılar dosyanın sonundaki DERLEYİCİ PERSPEKTİFİ'nde.

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Her bölümde önce "SADE ANLATIM" kutusunu oku, sonra teknik açıklamaya geç.
// Benzetmelerin hepsi aynı kişiyle, bir POSTACIYLA kurulur.
//
// Bu bölümün tek cümlelik özeti: enum "bu değer şu seçeneklerden yalnız biridir" der; match de
// her seçenek için ne yapılacağını yazdırır ve hiçbirini unutmana izin vermez.
//
// MİNİ SÖZLÜK (bu dosyada geçen kelimeler):
//   enum                   → seçenek listesi olan tür. Değer, seçeneklerden yalnız biridir.
//   varyant (variant)      → o seçeneklerden her biri: `Yon::Kuzey` gibi.
//   unit varyant           → yanında veri taşımayan seçenek: `Cik`.
//   tuple-like varyant     → yanında adsız veriler taşıyan seçenek: `V4(127, 0, 0, 1)`.
//   struct-like varyant    → yanında adlı veriler taşıyan seçenek: `Tasi { x: 1, y: 2 }`.
//   Option<T>              → "içinde bir T olabilir de olmayabilir de" diyen hazır enum.
//   Some(x) / None         → Option'ın iki seçeneği: "var, işte x" / "yok".
//   null                   → başka dillerdeki "boş" değer. Rust'ta yoktur; yerine None kullanılır.
//   match                  → bir değerin hangi seçenek olduğuna bakıp ona göre iş yapan yapı.
//   kol (arm)              → match içindeki her `desen => yapılacak iş` satırı.
//   desen (pattern)        → değerin uyması beklenen kalıp: `Some(n)`, `3`, `_`.
//   bağlamak (bind)        → desenin içindeki veriye isim vermek: `Some(n)` içindeki n.
//   catch-all, `_`         → "geri kalan her şey" kolu. `_` değeri kullanmayacağını söyler.
//   bütünlük denetimi      → (exhaustiveness) derleyicinin "her seçeneğe kol yazdın mı" bakışı.
//   if let / let-else      → tek bir seçenekle ilgilenirken match'in kısa yazımları.
//   unwrap_or(y)           → Option doluysa içindekini, boşsa y'yi verir.
//   format!                → `println!` gibi, ama ekrana basmak yerine String üretir.
//   derive(Debug)          → türü `{:?}` ile yazdırılabilir yapar (structs bölümünde anlatıldı).
//   impl, metot, &self     → türe bağlı fonksiyonlar (structs bölümünde anlatıldı).
//   <T> (generic)          → "herhangi bir tür" için yer tutucu: Option<i32>, Option<String>.
//   Copy                   → Copy trait'i olan türler kopyalanır (sayı, &T, Option<T>); eskisi de kullanılır.
//   Box<T>                 → heap'te duran bir değeri gösteren, onun sahibi olan ok.
//   discriminant (tag)     → değerin hangi varyant olduğunu söyleyen gizli küçük etiket.
//   refactor               → kodun davranışını değiştirmeden düzenini iyileştirmek.
//   sugar (şeker)          → aynı işi daha kısa yazdıran kolaylık.
//   E0004                  → "match'te eksik seçenek var" hatasının numarası.
//   derleyici (rustc)      → kodu programa çeviren ve hataları önceden bulan araç.
//
// SADE ANLATIM (tanımlar için): Üç `enum` birer seçenek listesi; `IpAdres` üç, `Mesaj` dört,
// `Yon` dört seçenek. Bazıları veri taşır (ör. `Yaz` bir yazı); `impl Mesaj::calistir` hangi seçenek
// olduğunu `match` ile bulur. Her varyant unit, tuple ya da struct gibi olabilir; ayrıntı aşağıda.
#[derive(Debug)]
enum IpAdres {
    V4(u8, u8, u8, u8), // tuple-like varyant (veri taşır)
    V6(String),         // tek alanlı varyant
    Bilinmeyen,         // unit varyant (veri taşımaz)
}

#[derive(Debug)]
enum Mesaj {
    Cik,                     // unit
    Tasi { x: i32, y: i32 }, // struct-like varyant
    Yaz(String),             // tuple-like
    RenkDegis(u8, u8, u8),
}

impl Mesaj {
    // enum'lar da metot alabilir; genelde içini match ile ayırırsın.
    fn calistir(&self) {
        match self {
            Mesaj::Cik => println!("çık"),
            Mesaj::Tasi { x, y } => println!("taşı {x},{y}"),
            Mesaj::Yaz(s) => println!("yaz: {s}"),
            Mesaj::RenkDegis(r, g, b) => println!("renk {r},{g},{b}"),
        }
    }
}

#[derive(Debug)]
enum Yon {
    Kuzey,
    Guney,
    Dogu,
    Bati,
}

mod derinlesme;
fn main() {
    enum_ve_varyantlar();
    option_neden();
    match_ornekleri();
    exhaustiveness();
    if_let_ornekleri();
    derinlesme::calistir();
}

// 1. Enum tanımı, veri taşıyan varyantlar, metotlar
// SADE ANLATIM: Postacının elindeki gönderi ya bir MEKTUPTUR, ya bir KOLİDİR, ya da bir
// KARTPOSTALDIR. Aynı anda ikisi olamaz; bunların dışında bir şey de olamaz. Her cins kendi
// bilgisini taşır: kolinin kilosu vardır, mektubun yoktur. Rust'ta `enum` böyle bir "şunlardan
// biri" listesidir; her seçeneğe varyant denir ve her varyant kendine özgü veri taşıyabilir.
// Neden böyle? Tek bir "gönderi" kartına hem kilo hem kimden gözü koysaydık, mektup için kilo
// gözüne ne yazılacağı belirsiz kalırdı; biri o anlamsız gözü okuyup yanlış iş yapabilirdi.
//   Mini örnek:
//     enum Gonderi {
//         Mektup,                      // veri taşımaz
//         Koli(u32),                   // kilosunu taşır
//         Kartpostal { kimden: String },   // adlı bir bilgi taşır
//     }
//     let g = Gonderi::Koli(3);        // ✓ g bir kolidir; kilosu 3
//     let h = Gonderi::Mektup(3);      // ✗ derlenmez: mektubun kilo bilgisi yoktur
fn enum_ve_varyantlar() {
    let adresler = [
        IpAdres::V4(127, 0, 0, 1),
        IpAdres::V6(String::from("::1")),
        IpAdres::Bilinmeyen,
    ];
    for a in adresler {
        // match bir İFADEDİR: değer üretir. Varyantın taşıdığı veriyi burada bağlarız.
        let aciklama = match a {
            IpAdres::V4(a, b, c, d) => format!("IPv4 {a}.{b}.{c}.{d}"),
            IpAdres::V6(s) => format!("IPv6 {s}"),
            IpAdres::Bilinmeyen => String::from("bilinmeyen"),
        };
        println!("{aciklama}");
    }

    for m in [
        Mesaj::Cik,
        Mesaj::Tasi { x: 1, y: 2 },
        Mesaj::Yaz(String::from("selam")),
        Mesaj::RenkDegis(255, 0, 0),
    ] {
        m.calistir();
    }

    // Neden enum? Ayrı alanlar + "hangisi geçerli?" bayrağı yanlış alanı okumaya açıktır; enum
    // olasılıkları TEK TÜRDE toplar, her varyant kendi verisini taşır, match hepsini zorunlu kılar.
}

// 2. Option<T> — null'ın olmayışı ve nedeni
// SADE ANLATIM: Kapının önündeki posta kutusunu düşün: içinde ya bir mektup vardır ya da kutu
// boştur. Kapağı açıp bakmadan mektubu okuyamazsın. `Option` bu kutudur: `Some(5)` "içinde 5 var",
// `None` "boş" demektir. Rust, kutuyu açıp iki ihtimale de bakmadan içindeki değeri kullandırmaz.
// Neden böyle? Birçok dilde "boş" (null) her yerde karşına çıkabilir ve bakmayı unutursan program
// çalışırken çöker. Rust'ta boş olabilecek değerler ayrı bir türdür; bakmayı unutursan derlenmez.
//   Mini örnek:
//     let kutu: Option<i32> = Some(5);
//     let bos: Option<i32> = None;
//     let a = kutu + 1;                // ✗ derlenmez: kutu sayı değildir, önce açılmalı
//     let b = kutu.unwrap_or(0) + 1;   // ✓ 6 → doluysa içindeki, boşsa 0
//     let c = bos.unwrap_or(0) + 1;    // ✓ 1 → kutu boştu, 0 kullanıldı
fn option_neden() {
    // Rust'ta null YOKTUR. "Değer olmayabilir" durumu Option<T> ile TÜRÜN İÇİNE kodlanır:
    //   enum Option<T> { Some(T), None }
    let bazi: Option<i32> = Some(5);
    let yok: Option<i32> = None;

    // i32 ile Option<i32> toplanamaz: None ihtimalini ELE ALMALISIN (unutulmuş null kontrolü yok).
    let sonuc = match bazi {
        Some(n) => n + 1,
        None => 0,
    };
    println!("sonuç = {sonuc}, yok = {yok:?}");
    println!("yok.unwrap_or(-1) = {}", yok.unwrap_or(-1)); // kısayol metotlardan biri

    // Neden null yerine Option? Sorun: null, dillerin "milyar dolarlık hatası" (Tony Hoare).
    // Her referans null olabildiğinde, kontrol etmeyi UNUTABİLİRSİN → çalışma anında null
    // pointer dereference / çökme. Çözüm: Rust'ta "yokluk" ayrı bir TÜR (Option); None'ı ele
    // almadan içindeki T'ye erişemezsin, yani derleyici kontrolü ZORUNLU kılar. Hatayı
    // çalışma anından derleme anına çeker.
}

// 3. match: değer bağlama, catch-all, ifade oluşu
// SADE ANLATIM: Postacının cebinde bir talimat kâğıdı vardır: "mektupsa kutuya at, koliyse zili
// çal, geri kalan her şeyi şubeye geri götür". Gönderiye bakar, uyan İLK satırı uygular. `match`
// bu kâğıttır: her satır bir koldur. En alttaki "geri kalan her şey" satırı `_` ile yazılır.
// `match` bir değer de verir; sonucu doğrudan bir değişkene koyabilirsin.
// Neden? Uzun if-else zincirinde ihtimal atlamak kolay; match'te derleyici eksik olanı söyler.
//   Mini örnek:
//     let zar = 6;
//     let sonuc = match zar {
//         6 => "kazandın",          // zar 6 ise
//         1 => "kaybettin",         // zar 1 ise
//         _ => "tekrar at",         // geri kalan her sayı
//     };
//     println!("{sonuc}");          // ✓ kazandın
fn match_ornekleri() {
    let zar = 9;
    match zar {
        3 => println!("üç!"),
        7 => println!("yedi!"),
        diger => println!("diğer: {diger}"), // isimli catch-all: değeri BAĞLAR
    }
    // `_` de catch-all'dur ama değeri umursamadığında kullanılır:
    let n = 4;
    let paritesi = match n % 2 {
        0 => "çift",
        _ => "tek", // 0 dışındaki her şey
    };
    println!("{n} → {paritesi}");
}

// 4. Exhaustiveness (bütünlük): derleyici TÜM olasılıkları ister
// SADE ANLATIM: Talimat kâğıdında her gönderi cinsi için bir satır bulunmalıdır. "Koli gelirse ne
// yapacağım" satırı eksikse postacı ilk kolide ne yapacağını bilemez. Rust'ın derleyicisi kâğıdı
// baştan okur; bir seçenek için satır yoksa programı derlemez. Buna bütünlük denetimi denir.
// Neden böyle? Yarın listeye "Dergi" diye yeni bir cins eklersen, derleyici o cinsi unutan bütün
// `match`leri tek tek gösterir; hiçbirini gözden kaçıramazsın.
//   Mini örnek:
//     enum Gonderi { Mektup, Koli }
//     let g = Gonderi::Koli;
//     let is = match g {
//         Gonderi::Mektup => "kutuya at",
//         Gonderi::Koli => "zili çal",     // ✓ iki seçenek de yazıldı
//     };
//     let eksik = match g {
//         Gonderi::Mektup => "kutuya at",  // ✗ derlenmez: Koli için satır yok
//     };
fn exhaustiveness() {
    for y in [Yon::Kuzey, Yon::Guney, Yon::Dogu, Yon::Bati] {
        // Bu match tüm varyantları kapsar → derlenir.
        let ok = match y {
            Yon::Kuzey => "↑",
            Yon::Guney => "↓",
            Yon::Dogu => "→",
            Yon::Bati => "←",
        };
        print!("{ok} ");
    }
    println!();

    // ✗ Bir varyantı atlarsan derlenmez:
    //   match y {
    //       Yon::Kuzey => "↑",
    //       Yon::Guney => "↓",
    //   } // E0004: non-exhaustive patterns: `Yon::Dogu` and `Yon::Bati` not covered
    // Neden derleyici bunu zorluyor? Sorun: C'de switch'te bir case'i unutmak SESSİZCE
    // atlanır (ya da yanlış default'a düşer) — fark edilmeyen mantık hatası. Dahası, enum'a
    // İLERİDE yeni varyant eklenirse onu ele almayan TÜM match'ler derlenmez → derleyici-güdümlü refactor.
}

// 5. if let ve let-else — tek desen için match'in kısası
// SADE ANLATIM: Bazen yalnız tek bir ihtimal seni ilgilendirir: "kutuda mektup varsa oku, yoksa
// bir şey yapma". Bunun için bütün bir talimat kâğıdı yazmak uzun kaçar. `if let` kısa yoludur:
// "eğer kutu doluysa, içindekine şu ismi ver ve şunu yap". `let ... else` ise tersinden söyler:
// "kutu doluysa içindekini al ve yola devam et; boşsa hemen buradan çık".
//   Mini örnek:
//     let kutu = Some(3);
//     if let Some(adet) = kutu {
//         println!("{adet} mektup var");   // ✓ yalnız kutu doluysa çalışır
//     }
//     let Some(adet) = kutu else {
//         return;                          // kutu boşsa fonksiyondan çık
//     };
//     println!("{adet}");                  // ✓ buraya geldiysek adet elimizde: 3
fn if_let_ornekleri() {
    let ayar: Option<u8> = Some(3); // Option<u8> Copy'dir (u8 Copy), tekrar kullanılabilir

    // if let: yalnız bir desenle ilgileniyorsan match'in kısa yazımı.
    if let Some(v) = ayar {
        println!("ayar = {v}");
    } else {
        println!("ayar yok");
    }

    // let ... else: desen tutmazsa bloktan ERKEN çıkarsın (burada return). Tutarsa, bağlanan
    // değer (v) sonraki satırlarda normal biçimde kullanılabilir.
    let Some(v) = ayar else {
        println!("değer yok, çık");
        return;
    };
    println!("let-else ile v = {v}");
    // Neden? Tam match + `_ => ()` gürültülü; bunlar aynı güvenliği koruyan kısa yazımlar (sugar).
}

// SADE ANLATIM (aşağıdaki bölüm için): Her gönderinin cinsini söyleyen küçük bir etiket vardır;
// postacı önce ona bakar. Derleyici bazen etikete hiç yer ayırmaz, "boş"u türün kullanmadığı bir
// değere saklar. Bölüm ayrıca bütünlük denetiminin hangi adımda yapıldığını söyler. Program yazarken
// gerekmez; ilk okuyuşta atla, beş bölüm yeter.
// ============================================================================
// DERLEYİCİ PERSPEKTİFİ — enum ve match nasıl çalışıyor?
// ============================================================================
//
// • Enum bellekte "tagged union" (etiketli birleşim): bir discriminant (hangi varyant
//   olduğunu söyleyen etiket) + en büyük varyantı tutacak kadar yer (payload). match,
//   bu discriminant'a bakıp doğru dalı seçer.
//
// (Sade: niche = bir türün hiç kullanmadığı değer. Örneğin bir referans hiçbir zaman 0 olamaz;
//  derleyici "yok" bilgisini 0 diye yazar ve fazladan yer harcamaz.)
// • NICHE OPTIMIZATION (doğrulanmış ölçümler): bir türün KULLANILMAYAN bir bit deseni
//   ("niche") varsa, derleyici None'ı o desende saklar — ayrı bir tag baytı EKLEMEZ:
//       i32        = 4,  Option<i32>        = 8   (i32 tüm desenleri kullanır → ayrı tag + hizalama)
//       &i32       = 8,  Option<&i32>       = 8   (referans asla null olamaz → o desen None olur)
//       Box<i32>   = 8,  Option<Box<i32>>   = 8   (aynı mantık)
//       bool       = 1,  Option<bool>       = 1   (bool 1 byte'ta yalnız 0/1 kullanır; 254 geçersiz desenden biri None olur)
//   Yani `Option<&T>` "null'lanabilir pointer" kadar ucuzdur ama güvenlidir.
//
// (Sade: THIR, derleyicinin kodu çevirirken kullandığı ara biçimlerden biridir.)
// • Exhaustiveness denetimi: match, tür denetiminden sonra THIR (typed HIR) üzerinde
//   kontrol edilir. Eksik varyant → E0004, kaynak
//   rustc_mir_build/src/thir/pattern/check_match.rs:1281 civarı. Altındaki "usefulness"
//   (bir desenin yeni bir durum kapatıp kapatmadığını hesaplayan) algoritma ayrı bir
//   crate'tedir: rustc_pattern_analysis.
//
// (Sade: lowering/indirgeme = kısa yazımı, daha temel bir yazıma çevirmek.)
// • if let / while let / let-else: sözdizimsel şekerdir; ANLAMCA match'e eşdeğerdirler ama güncel
//   rustc bunları HIR'da match'e açmaz, `Let` / `If` / let-else düğümleri olarak korur
//   (rustc_ast_lowering/src/expr.rs:324-325). `for` → loop + match ve `?` → match indirgemeleri
//   ise gerçekten yapılır.
