// Rust Book — Bölüm 10: Generic Types, Traits, and Lifetimes
// Çalıştır: cargo run -p generics-traits-lifetimes
//
// Üç konu tek bölümde çünkü üçü de AYNI amaca hizmet eder: tekrarı azaltmak ve DERLEME
// ANINDA güvenliği korumak. Generics "farklı türler için tek kod", traits "farklı türlerin
// ortak davranışı", lifetimes "referansların ne kadar geçerli kaldığı".
//
// Doğrulanmış (rustc ile): trait bound sağlanmazsa E0277; olmayan metot E0599; elision
// başarısız → E0106; `dyn` için uygun olmayan trait → E0038; orphan rule → E0117.
// (Lifetime/HRTB ve orphan rule'u daha önce ayrıca derinlemesine konuşmuştuk; burada
// kitabın çerçevesine oturtuyorum.)

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Bu dosyadaki her bölümün üstünde "SADE ANLATIM" diye bir kutu var. Önce onu oku, sonra
// altındaki teknik açıklamaya geç. Teknik kısım zor gelirse sorun değil: sade kutu ana fikri verir.
// Bu dosyadaki benzetmelerin hepsi aynı yerde geçer: bir okulda (sınıf, belge, kütüphane).
//
// Bu bölümün tek cümlelik özeti: Aynı kodu farklı türler için bir kez yazarsın (generics), o
// türlerden ne beklediğini söylersin (traits) ve ödünç verilen verinin ne zamana kadar geçerli
// olduğunu derleyiciye bildirirsin (lifetimes).
//
// MİNİ SÖZLÜK (bu dosyada geçen kelimeler):
//   generic (jenerik)        → türü boş bırakılmış kod: "hangi tür gelirse onunla çalış".
//   tür parametresi <T>      → o boş bırakılan yerin adı. T yerine i32 de gelir, char da.
//   somut tür                → boşluğa gerçekten konan tür: i32, String, Haber gibi.
//   trait (özellik)          → bir türün "şunu yapabilirim" diye verdiği söz; metot listesi.
//   interface                → başka dillerde trait'e benzeyen şeyin adı.
//   imza                     → fonksiyonun ilk satırı: adı, ne aldığı, ne döndürdüğü.
//   impl                     → "bu tür için şu metotları yazıyorum" bloğu.
//   varsayılan gövde         → trait'in içinde hazır gelen metot; tür isterse kendi yazar.
//   ezmek (override)         → hazır gelen metodun yerine kendininkini yazmak.
//   trait bound (kısıt)      → "T her tür olamaz, şu sözü vermiş olmalı" şartı: `T: PartialOrd`.
//   `impl Trait`             → "şu sözü veren herhangi bir tür" demenin kısa yazımı.
//   dispatch                 → "hangi türün metodu çağrılacak?" sorusuna cevap bulma işi.
//   statik / dinamik         → cevabın derlerken bulunması / program çalışırken bulunması.
//   monomorphization         → derleyicinin generic kodu, kullanılan her tür için ayrı kopyalaması.
//   trait object (dyn Trait) → asıl türü unutulmuş, yalnız verdiği söz bilinen değer.
//   vtable                   → trait object'in yanındaki "ben bu işi şöyle yaparım" çizelgesi.
//   Box                      → değeri heap'e koyup ona tek bir okla ulaşan kutu.
//   pointer dolaylaması      → veriye doğrudan değil, önce bir oku izleyerek ulaşmak.
//   inline (satır içi)       → fonksiyon gövdesinin çağrıldığı yere yapıştırılması; çağrı kalkar.
//   homojen / heterojen      → hepsi aynı türden / farklı türlerden oluşan.
//   blanket impl             → "şu sözü veren HER türe bu trait'i de ver" diyen tek kural.
//   orphan rule              → "trait de tür de başkasınınsa o ikisi için impl yazamazsın" kuralı.
//   lifetime ('a)            → bir referansın geçerli kaldığı kod bölgesi; `'a` o bölgenin adı.
//   elision                  → derleyicinin, belli olan lifetime'ı senin yerine doldurması.
//   'static                  → program boyunca geçerli kalan lifetime.
//   literal                  → koda elle yazılmış sabit değer: "merhaba", 42 gibi.
//   dangling (sarkan)        → gösterdiği veri silinmiş referans (bkz. ownership).
//   zero-cost                → kolaylığın çalışan programı yavaşlatmaması.
//   derleme anı / runtime    → kodun derlendiği an / programın çalıştığı an.

use std::fmt::Display;

mod derinlesme;

fn main() {
    generikler();
    traitler();
    lifetimelar();
    derinlesme::calistir();
}

// ============================================================================
// 1. GENERICS (jenerikler) — tek kod gövdesi, çok tür
// ============================================================================
// SADE ANLATIM: Sınıfta "en uzununu bul" oyunu oynanıyor. Öğrencilerle de oynanır, kalemlerle de,
// iplerle de; kural hep aynıdır: ikisini yan yana koy, uzun olanı tut. Oyunu her eşya için
// yeniden anlatmazsın. Generic fonksiyon budur: `<T>` "hangi tür gelirse" demektir. Tek şart
// vardır: yan yana konup karşılaştırılabilmeleri (`T: PartialOrd`); kokuları "hangisi büyük"
// diye sıralayamazsın. Neden böyle? Generics olmasaydı aynı kodu her tür için kopyalardın; bir
// kopyadaki hatayı düzeltip ötekini unutmak kolaylaşırdı.
//   Mini örnek:
//     fn ilki<T>(liste: &[T]) -> &T { &liste[0] }   // T: hangi tür gelirse
//     ilki(&[3, 7]);                                // ✓ T = i32
//     ilki(&['a', 'z']);                            // ✓ T = char; aynı fonksiyon
//     fn buyuk<T>(a: T, b: T) -> bool { a > b }     // ✗ derlenmez: her T karşılaştırılamaz
//     fn buyuk<T: PartialOrd>(a: T, b: T) -> bool { a > b }   // ✓ şartı yazdık
fn generikler() {
    // Generic FONKSİYON: <T> bir TÜR PARAMETRESİdir. T: PartialOrd "T karşılaştırılabilmeli"
    // kısıtını (trait bound) koyar; yoksa `>` kullanamayız (bu bir trait metodudur).
    fn en_buyuk<T: PartialOrd>(liste: &[T]) -> &T {
        let mut en = &liste[0];
        for x in liste {
            if x > en {
                en = x;
            }
        }
        en
    }
    println!("en büyük (i32)  = {}", en_buyuk(&[3, 7, 2, 9, 4]));
    println!("en büyük (char) = {}", en_buyuk(&['a', 'z', 'm']));

    // SADE ANLATIM: Etiketi boş bırakılmış, iki gözlü bir kalem kutusu düşün. İçine ne koyacağına
    // sen karar verirsin; ama iki göze de aynı türden şey girer. `Nokta<T>` böyle bir kalıptır:
    // T'yi i32 seçersen iki alan da i32, f64 seçersen ikisi de f64 olur. Neden böyle? Yoksa
    // NoktaI32, NoktaF64 diye aynı struct'ı her tür için yeniden yazman gerekirdi.
    //   Mini örnek:
    //     struct Cift<T> { sol: T, sag: T }
    //     let a = Cift { sol: 1, sag: 2 };       // ✓ T = i32
    //     let b = Cift { sol: 1.5, sag: 2.5 };   // ✓ T = f64
    //     let c = Cift { sol: 1, sag: 2.5 };     // ✗ derlenmez: iki alan da aynı T olmalı
    // Generic STRUCT: aynı tanımla hem int hem float noktalar.
    #[derive(Debug)]
    struct Nokta<T> {
        x: T,
        y: T,
    }
    // impl bloğunun da <T>'yi bildirmesi gerekir: "her T için bu metot".
    impl<T> Nokta<T> {
        fn x(&self) -> &T {
            &self.x
        }
    }
    // (Sade: aşağıdaki blok "bu metot yalnız içinde f64 olan noktalarda var" der; karekök almak
    // tam sayılı noktada tanımlı olmadığı için `ni.uzaklik()` derlenmez, `nf.uzaklik()` derlenir.)
    // SADECE belirli bir tür için metot: impl Nokta<f64> { ... } (koşullu metot).
    impl Nokta<f64> {
        fn uzaklik(&self) -> f64 {
            (self.x * self.x + self.y * self.y).sqrt() // sqrt yalnız f64'te var
        }
    }
    let ni = Nokta { x: 3, y: 4 };
    let nf = Nokta { x: 3.0, y: 4.0 };
    println!("ni.x = {}, nf.uzaklık = {}", ni.x(), nf.uzaklik());
    // self.y'yi de okuyalım ki derive(Debug) dead_code tuzağına düşmeyelim (Bölüm 5 nugget'ı).
    println!("ni = {ni:?} (y = {})", ni.y);

    // Neden generics? Sorun: generics olmasaydı her tür için AYNI mantığı kopyalamak gerekirdi
    // (en_buyuk_i32, en_buyuk_char ...). Hem tekrar, hem bir yerde düzeltip ötekini unutma riski.
    // Neden "zero-cost"? Çünkü runtime'da tür bilgisi taşınmaz — bkz. DERLEYİCİ PERSPEKTİFİ.
}

// ============================================================================
// 2. TRAITS (özellikler) — paylaşılan davranışın arayüzü
// ============================================================================

// SADE ANLATIM: Okulda "sunum yapabilir" belgesi var. Belgeyi alan herkes "konunu söyle"
// denince söyleyebilmek zorundadır; nasıl söyleyeceği kendine kalmıştır. Belgede bir de hazır
// kapanış cümlesi yazar: kendi cümlesini yazmayan onu kullanır. Trait bu belgedir: metotların
// adını ve biçimini söyler, her tür gövdesini kendi yazar; hazır gövdeye "varsayılan" denir.
// Neden böyle? Belge olmasaydı farklı türlerle aynı işi yapan kod yazamazdın; her tür için
// ayrı fonksiyon gerekirdi.
//   Mini örnek:
//     trait Sunum {
//         fn konu(&self) -> String;                                    // zorunlu
//         fn kapanis(&self) -> String { String::from("Teşekkürler") }  // hazır (varsayılan)
//     }
//     struct Ali;
//     impl Sunum for Ali { fn konu(&self) -> String { String::from("uzay") } }
//     Ali.konu();      // ✓ "uzay" — Ali'nin kendi yazdığı
//     Ali.kapanis();   // ✓ "Teşekkürler" — Ali yazmadı, hazır olan çalıştı
// Trait = bir türün uygulaması gereken metot imzaları kümesi (başka dillerdeki "interface"e
// yakın). VARSAYILAN gövde verilebilir; tür isterse ezer (override).
trait Ozet {
    fn yazar(&self) -> String; // zorunlu: her tür kendi verir
    fn ozetle(&self) -> String {
        // varsayılan: zorunlu metodu kullanabilir
        format!("(devamı...) — {}", self.yazar())
    }
}

struct Haber {
    baslik: String,
    yazar: String,
}
struct Tweet {
    kullanici: String,
}
impl Ozet for Haber {
    fn yazar(&self) -> String {
        self.yazar.clone()
    }
    fn ozetle(&self) -> String {
        // varsayılanı EZER
        format!("{} — {}", self.baslik, self.yazar)
    }
}
impl Ozet for Tweet {
    fn yazar(&self) -> String {
        format!("@{}", self.kullanici) // ozetle'yi EZMEZ → varsayılan gövde çalışır
    }
}

// SADE ANLATIM: Öğretmen "sunum belgesi olan herkes tahtaya gelebilir" der. Kimin geldiğine
// bakmaz, belgeye bakar. `&impl Ozet` tam olarak bunu söyler: "türün ne olduğu önemli değil,
// Ozet sözünü vermiş olsun." Neden böyle? Fonksiyon içeride `ozetle()` çağıracak; derleyici bu
// metodun gelen her türde bulunduğundan emin olmak ister, belgesiz olanı kapıda geri çevirir.
//   Mini örnek (Sunum ve Ali, bir üstteki kutudan):
//     fn tahtaya_cagir(x: &impl Sunum) { println!("{}", x.konu()); }
//     tahtaya_cagir(&Ali);   // ✓ Ali'nin belgesi var
//     tahtaya_cagir(&5);     // ✗ derlenmez: i32 için Sunum yazılmamış
// Trait BOUND fonksiyon: "Ozet uygulayan HER tür". İki eşdeğer yazım:
fn duyur(x: &impl Ozet) {
    // `impl Trait` = kısa yazım (anonim generic)
    println!("Yeni: {}", x.ozetle());
}
fn _duyur_uzun<T: Ozet>(x: &T) {
    println!("Yeni: {}", x.ozetle());
}

fn traitler() {
    let h = Haber {
        baslik: String::from("Rust 2024"),
        yazar: String::from("Kaan"),
    };
    let t = Tweet {
        kullanici: String::from("kaan"),
    };
    duyur(&h); // ezilmiş ozetle
    duyur(&t); // varsayılan ozetle

    // SADE ANLATIM: Sunum günü için iki düzen var. (a) "Bu saatte yalnız Ali sunacak": kim
    // geleceği önceden bellidir, sorulacak bir şey kalmaz. Generic kod böyledir: derleyici her
    // tür için ayrı bir kopya hazırlar. (b) "Listede kim varsa sırayla gelsin": gelen herkes
    // yanında "ben şöyle sunarım" yazan küçük bir kart taşır, öğretmen o karta bakar. `dyn Ozet`
    // böyledir; kartın adı vtable'dır. Karta bakmak çok az zaman alır; karşılığında farklı
    // türleri aynı listeye koyabilirsin. Neden iki yol? Biri en hızlısıdır, öteki en esneğidir.
    //   Mini örnek (h bir Haber, t bir Tweet):
    //     let ayni: Vec<Haber> = vec![h];                    // tek tür: dyn gerekmez
    //     let olmaz = vec![h, t];                            // ✗ derlenmez: Vec tek tür tutar
    //     let karisik: Vec<Box<dyn Ozet>> = vec![Box::new(h), Box::new(t)];   // ✓ ikisi birlikte
    // STATİK vs DİNAMİK dispatch — Rust'ın iki çağırma biçimi:
    //
    // (a) GENERIC (statik dispatch): her somut tür için AYRI kod üretilir (monomorphization),
    //     çağrı derleme anında çözülür → hızlı, satır içine (inline) alınabilir.
    //
    // (b) TRAIT OBJECT `dyn Ozet` (dinamik dispatch): tek kod; hangi metot çağrılacağı
    //     ÇALIŞMA ANINDA vtable (sanal metot tablosu) üzerinden bulunur. Esnek: farklı türleri
    //     TEK bir Vec'te tutabilirsin; bedeli bir pointer dolaylaması.
    let karisik: Vec<Box<dyn Ozet>> = vec![
        Box::new(Haber {
            baslik: String::from("Hava"),
            yazar: String::from("Ajans"),
        }),
        Box::new(Tweet {
            kullanici: String::from("bot"),
        }),
    ];
    for oge in &karisik {
        println!("liste: {}", oge.ozetle()); // dinamik dispatch (vtable)
    }
    // Neden iki yol? Sorun: tek yol olsaydı ya hep tür başına kod şişmesi (generic'in bedeli)
    // ya hep dolaylama (dyn'in bedeli) yaşardık. Rust seçimi SANA bırakır: homojen + sıcak yol →
    // generic; heterojen koleksiyon/eklenti → dyn. (dyn olabilmesi için trait "dyn compatible"
    // olmalı; neden, DERLEYİCİ PERSPEKTİFİ'nde.)
}

// SADE ANLATIM: Okul kuralı: "Okumayı bilen herkes kütüphane kartı alır." Kartı her öğrenci
// için ayrı ayrı yazmazsın; tek kural hepsini kapsar, okumayı sonradan öğrenen de kendiliğinden
// kart sahibi olur. Blanket impl böyle bir kuraldır: "Display sözünü veren her türe Etiket'i de
// ver." Neden böyle? Yoksa i32 için, String için, char için aynı impl'i tek tek yazardın.
//   Mini örnek:
//     42.etiket();         // ✓ "[42]"  — i32 Display'dir, kural onu kapsar
//     "ada".etiket();      // ✓ "[ada]" — &str de Display'dir
//     vec![1].etiket();    // ✗ derlenmez: Vec Display değildir, kural onu kapsamaz
// Blanket impl örneği: "Display olan HER türe benim trait'imi uygula" — stdlib'in
// `impl<T: Display> ToString for T` kalıbı. Burada küçük bir örnek:
trait Etiket {
    fn etiket(&self) -> String;
}
impl<T: Display> Etiket for T {
    fn etiket(&self) -> String {
        format!("[{self}]")
    }
}

// ============================================================================
// 3. LIFETIMES (yaşam süreleri) — referanslar ne kadar geçerli?
// ============================================================================

// SADE ANLATIM: Okul kütüphanesinden iki kitap ödünç aldın; birinin iadesi cuma, ötekinin
// pazartesi. Arkadaşına "kalın olanı sana vereceğim" dedin, ama hangisinin kalın olduğu henüz
// belli değil. Arkadaşın kitabı güvenle ne zamana kadar tutabilir? İkisinden erken olana, yani
// cumaya kadar. `'a` etiketi bunu yazar: "dönen referans, iki girdinin de geçerli olduğu süre
// boyunca geçerlidir." Etiket iade tarihini UZATMAZ; yalnız hangi tarihin geçerli olduğunu
// bildirir. Neden böyle? Derleyici fonksiyonun içine bakmadan, yalnız imzadan, sonucun hangi
// girdiye bağlı olduğunu bilmelidir; yoksa silinmiş veriye bakan bir referans kullanılabilirdi.
//   Mini örnek:
//     fn en_uzun(x: &str, y: &str) -> &str { .. }               // ✗ derlenmez: hangisine bağlı?
//     fn en_uzun<'a>(x: &'a str, y: &'a str) -> &'a str { .. }  // ✓ "ikisine de bağlı"
//     let sonuc;
//     {
//         let kisa = String::from("ab");
//         sonuc = en_uzun("abc", &kisa);
//     }                        // kisa burada silindi
//     println!("{sonuc}");     // ✗ derlenmez: sonuç kisa'ya da bağlı sayılır, kisa artık yok
// Lifetime, bir referansın İŞARET ETTİĞİ verinin en az ne kadar yaşayacağını söyleyen bir
// GENERIC parametredir (tür değil, SÜRE/bölge parametresi). Yeni bir süre yaratmaz; var olan
// sürelerin İLİŞKİSİNİ derleyiciye bildirir.
//
// 'a burada "dönen referans, x ve y'nin İKİSİNİN de yaşadığı ortak bölge kadar geçerli" der.
fn en_uzun<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

// SADE ANLATIM: Ödünç kitabın bir sayfasına ayraç koydun. Ayraç tek başına bir şey anlatmaz;
// kitap iade edilince gösterdiği sayfa da gider. İçinde referans tutan struct bu ayraçtır:
// gösterdiği veri durdukça kullanılabilir. `<'a>` yazarak "bu struct bir şeye bakıyor, ondan uzun
// yaşayamaz" notunu düşersin. Neden böyle? Not olmasaydı ayraç, kitaptan sonra da kullanılabilir
// ve silinmiş bir veriyi gösterebilirdi.
//   Mini örnek:
//     struct Ayrac<'a> { satir: &'a str }
//     let a;
//     {
//         let kitap = String::from("Bir varmış");
//         a = Ayrac { satir: &kitap };
//         println!("{}", a.satir);    // ✓ kitap hâlâ burada
//     }
//     println!("{}", a.satir);        // ✗ derlenmez: kitap silindi, ayraç boşu gösterirdi
// Referans TUTAN struct: alan bir referanssa, struct kendi verisini aşamaz; bu yüzden
// lifetime ZORUNLU. "Bu Alinti, içindeki &str yaşadığı sürece geçerli."
struct Alinti<'a> {
    parca: &'a str,
}
impl<'a> Alinti<'a> {
    fn parca(&self) -> &str {
        self.parca
    }
}

fn lifetimelar() {
    let s1 = String::from("uzun bir cümle");
    let sonuc;
    {
        let s2 = String::from("kısa");
        sonuc = en_uzun(s1.as_str(), s2.as_str());
        println!("en uzun = {sonuc}"); // s2 HÂLÂ yaşıyorken kullan → OK
    }
    // println!("{sonuc}"); // ✗ burada s2 düştü; 'a ikisinin ortak bölgesi olduğu için E0597 olurdu

    let metin = String::from("İlk cümle. İkincisi.");
    let ilk = metin.split('.').next().unwrap();
    let a = Alinti { parca: ilk };
    println!("alıntı: {}", a.parca());

    // SADE ANLATIM: Kütüphane görevlisi, tek kitap getirip içinden bir sayfa gösteren birine
    // "bu sayfa hangi kitaptan?" diye sormaz; cevap bellidir. İki kitap getirene ise sorar.
    // Derleyici de böyledir: sonucun hangi girdiye bağlı olduğu belliyse `'a` yazdırmaz (buna
    // elision denir); belli değilse tahmin yürütmez, senden yazmanı ister.
    //   Mini örnek:
    //     fn ilk_kelime(s: &str) -> &str { .. }      // ✓ tek girdi var: sonuç ona bağlıdır
    //     fn sec(x: &str, y: &str) -> &str { .. }    // ✗ derlenmez: iki girdi var, hangisi?
    // ELISION (lifetime çıkarımı): çoğu referanslı imzada 'a yazmayız çünkü derleyici 3 kuralla
    // kendi doldurur (bkz. DERLEYİCİ PERSPEKTİFİ). `fn parca(&self) -> &str` yukarıda o yüzden
    // 'a'sız yazılabildi: Kural 3, &self'in süresini çıktıya verir.
    //
    // SADE ANLATIM: Okulun duvarına boyanmış yazıyı kimse ödünç alıp iade etmez; okul durdukça
    // orada durur. Koda tırnak içinde yazdığın yazılar da böyledir: program dosyasının içine
    // gömülür ve program çalıştığı sürece yerinde kalır. Bu süreye `'static` denir. Sonradan
    // oluşturup sileceğin bir veriye `'static` yazmak onu duvar yazısına çevirmez.
    //   Mini örnek:
    //     let s: &'static str = "duvardaki yazı";   // ✓ programa gömülü, hep geçerli
    //     let yerel = String::from("defter");
    //     let r: &'static str = &yerel;             // ✗ derlenmez: yerel fonksiyon bitince silinir
    // 'static: tüm program boyu yaşayan özel süre. String LİTERALLERİ 'static'tir (ikiliye
    // gömülü). Her yere 'static yazmak çözüm değildir — çoğu zaman gerçek sorun dangling
    // referanstır, süre etiketini uzatmak değil.
    let s: &'static str = "program boyu yaşar";
    println!("{s} / etiket örneği: {}", 42.etiket()); // blanket impl (Display→Etiket)
}

// SADE ANLATIM (aşağıdaki bölüm için): Yukarıdaki üç konunun ortak yanı şudur: işin tamamına
// yakını sen kodu DERLERKEN yapılır. Okul açılmadan önce hazırlanan ders programı gibi: kim hangi
// sınıfa girecek, hangi belge kimde var, hangi kitap ne zaman iade edilecek, hepsi önceden
// belirlenir; ders sırasında kimse durup bunları sormaz. Aşağıdaki bölüm bu hazırlığın derleyici
// içinde nerede ve nasıl yapıldığını anlatır. İlk okuyuşta atlayabilirsin; yukarıdaki üç bölümü
// anlamış olman yeter.
// ============================================================================
// DERLEYİCİ PERSPEKTİFİ — üçü de derleme anında, runtime bedeli (çoğunlukla) sıfır
// ============================================================================
//
// (Sade: "monomorphization" = derleyicinin `en_buyuk<T>`yi, kullanıldığı her tür için ayrı bir
// fonksiyon olarak yeniden yazması demek. Sen bir kez yazarsın, kopyaları derleyici çıkarır.)
// • MONOMORPHIZATION (jeneriklerin "zero-cost" sırrı): generic kod runtime'da tür taşımaz.
//   Derleyici, kodun KULLANDIĞI her somut tür için ayrı bir kopya üretir — en_buyuk::<i32> ve
//   en_buyuk::<char> ayrı iki fonksiyon olur. Bu toplama rustc_monomorphize/src/collector.rs
//   yapar: monomorphic örnekleri "ihtiyaç oldukça" (on demand) toplar. Dosyanın baş yorumu
//   (satır ~12) şunu da söyler: X crate'indeki bir generic, Y crate'inde derlenen bir
//   monomorphization üretebilir — ayrı derlemeyle (separate compilation) uyumlu. Sonuç:
//   C++ template'leri gibi hızlı (dolaylama yok), ama ikili dosya büyür (kod kopyalanır).
//   dyn Trait ise tersi: tek kod + vtable, çalışma anı dolaylaması.
//
// (Sade: "ördek tipi" (duck typing), bazı dillerdeki "metodu varsa çağır, yoksa çalışırken
// hata ver" yaklaşımıdır. Rust bunu yapmaz; metodun varlığını derlerken ister.)
// • TRAIT BOUND çözümü tür denetleyicide (rustc_hir_typeck/rustc_trait_selection) yapılır:
//   `T: Ozet` sağlanmazsa daha derlemede E0277. Metot bir trait'ten gelip de bound yoksa
//   "no method named ..." E0599. Yani "ördek tipi" (duck typing) YOK — bir metodu çağırmak
//   için o türün ilgili trait'i uyguladığı KANITLANMALI.
//
// (Sade: "dyn compatible" = bu trait'ten `dyn Trait` yapılabilir demek. Şartı: her metodun,
// asıl tür bilinmeden de çizelgeden (vtable) bulunup çağrılabilmesi.)
// • dyn COMPATIBILITY (eski adıyla "object safety"): her trait `dyn` olamaz. Kontrol
//   rustc_trait_selection/src/traits/dyn_compatibility.rs'tedir. Kabaca: trait bir VTABLE
//   kurabilmeli. Bu yüzden `fn uret() -> Self` gibi Self DÖNEN ya da generic metotlu trait
//   dyn olamaz (E0038): çünkü vtable'da hangi somut türü kuracağı/hangi monomorphization'ı
//   koyacağı belli değildir. Sorun: izin verilse, çalışma anında var olmayan bir tür/kopya
//   çağrılmaya kalkılırdı.
//
// (Sade: aşağıdaki üç kural, derleyicinin `'a`yı senin yerine ne zaman yazabildiğini sayar.
// "Erasure" (silme) ise bu etiketlerin çalışan programda hiç yer tutmaması demek.)
// • LIFETIME ELISION (3 kural, çıktı referansının süresini DOLDURUR; isim çözümlemede,
//   rustc_resolve/src/late.rs, `elided`/`current_elision_failures` ~399/~840):
//     1) Her referans parametre kendi ayrı süresini alır.
//     2) TEK girdi süresi varsa, o süre tüm çıktılara verilir.
//     3) Metotta &self/&mut self varsa, self'in süresi tüm çıktılara verilir.
//   Kurallar çıktıyı tek bir süreye bağlayamazsa derleyici PES eder ve senden etiket ister
//   (E0106). Yani elision bir TAHMİN değil, deterministik 3 kuraldır; belirsizlikte sessiz
//   kalmaz, hata verir. Lifetime'lar yalnız derleme anında denetlenir — üretilen makine
//   kodunda lifetime diye bir şey YOKTUR (tür silme / "erasure").
