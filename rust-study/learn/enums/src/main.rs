// Rust Book — Bölüm 6: Enums (numaralandırma) ve Pattern Matching (desen eşleme)
// Çalıştır: cargo run -p enums
//
// Stil: terim (İngilizce) + Türkçe açıklama + örnek + Neden? + Sorun + rustc: notu.
// Doğrulanmış (rustc ile derlenerek): non-exhaustive match → E0004; Option'ın niche
// optimization boyutları. Ayrıntılar dosyanın sonundaki DERLEYİCİ PERSPEKTİFİ'nde.

// Bir enum, bir değerin "şu OLASILIKLARDAN biri" olduğunu söyler. Her varyant veri
// taşıyabilir — hiç (unit), tuple gibi, ya da struct gibi.
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

fn main() {
    enum_ve_varyantlar();
    option_neden();
    match_ornekleri();
    exhaustiveness();
    if_let_ornekleri();
}

// 1. Enum tanımı, veri taşıyan varyantlar, metotlar
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

    // Neden enum? Sorun: bir değer "birkaç farklı biçimden biri" olabiliyorsa (IPv4 ya da
    // IPv6), bunu ayrı alanlar + "hangisi geçerli?" bayrağıyla tutmak hataya açıktır
    // (yanlış alanı okuma). enum, olasılıkları TEK TÜRDE toplar ve her varyantın kendi
    // verisini taşır; match seni her olasılığı ele almaya zorlar.
}

// 2. Option<T> — null'ın olmayışı ve nedeni
fn option_neden() {
    // Rust'ta null YOKTUR. "Değer olmayabilir" durumu Option<T> ile TÜRÜN İÇİNE kodlanır:
    //   enum Option<T> { Some(T), None }
    let bazi: Option<i32> = Some(5);
    let yok: Option<i32> = None;

    // i32 ile Option<i32>'yi doğrudan toplayamazsın; önce None ihtimalini ELE ALMAK
    // zorundasın. Bu, "unutulmuş null kontrolü"nü imkânsız kılar.
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
    // İLERİDE yeni bir varyant eklersen, onu ele almayan TÜM match'ler derlenmez olur; yani
    // derleyici seni her kullanımı güncellemeye zorlar → güvenli, derleyici-güdümlü refactor.
}

// 5. if let ve let-else — tek desen için match'in kısası
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
    // Neden if let / let-else? Sorun: tek bir varyantla ilgilenirken tam bir match + `_ => ()`
    // yazmak gürültülü olur. Bunlar aynı güvenliği koruyan kısa yazımlardır (sugar).
}

// ============================================================================
// DERLEYİCİ PERSPEKTİFİ — enum ve match nasıl çalışıyor?
// ============================================================================
//
// • Enum bellekte "tagged union" (etiketli birleşim): bir discriminant (hangi varyant
//   olduğunu söyleyen etiket) + en büyük varyantı tutacak kadar yer (payload). match,
//   bu discriminant'a bakıp doğru dalı seçer.
//
// • NICHE OPTIMIZATION (doğrulanmış ölçümler): bir türün KULLANILMAYAN bir bit deseni
//   ("niche") varsa, derleyici None'ı o desende saklar — ayrı bir tag baytı EKLEMEZ:
//       i32        = 4,  Option<i32>        = 8   (i32 tüm desenleri kullanır → ayrı tag + hizalama)
//       &i32       = 8,  Option<&i32>       = 8   (referans asla null olamaz → o desen None olur)
//       Box<i32>   = 8,  Option<Box<i32>>   = 8   (aynı mantık)
//       bool       = 1,  Option<bool>       = 1   (bool yalnız 0/1 → 2 deseni None olur)
//   Yani `Option<&T>` "null'lanabilir pointer" kadar ucuzdur ama güvenlidir.
//
// • Exhaustiveness denetimi: match, tür denetiminden sonra THIR (typed HIR) üzerinde
//   kontrol edilir. Eksik varyant → E0004, kaynak
//   rustc_mir_build/src/thir/pattern/check_match.rs:1281 civarı. Altındaki "usefulness"
//   (bir desenin yeni bir durum kapatıp kapatmadığını hesaplayan) algoritma ayrı bir
//   crate'tedir: rustc_pattern_analysis.
//
// • if let / while let / let-else: sözdizimsel şekerdir; derleyici bunları eşdeğer bir
//   match'e indirger (lowering).
