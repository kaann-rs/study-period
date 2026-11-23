// Açıklama: learn/patterns/ACIKLAMA.md
// Derinleşme — Bölüm 19: Patterns (pekiştirme: gerçekçi senaryo, sık hatalar, alıştırmalar)
// Çalıştır: cargo run -p patterns   |   Test: cargo test -p patterns
//
// main.rs desen sözdiziminin araçlarını tek tek gösterdi (literal, `|`, aralık, `..`, guard,
// `@`, let-else). Bu dosya onları birlikte çalıştırır ve main.rs'in değinmediği üç konuyu ekler:
//   • dilim desenleri (`[ilk, .., son]`, `kalan @ ..`) → küçük bir komut ayrıştırıcının omurgası
//   • bağlama modları (move / `ref` / `ref mut`, match ergonomics) → desen sahipliği NASIL etkiler
//   • iç içe yapı çözme, `matches!` ve bütünlük denetiminin refactoring'i nasıl güvenli kıldığı

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Bu dosya main.rs'teki desen araçlarını gerçek bir işte birlikte kullanır: kullanıcının yazdığı
// komutları anlayan küçük bir depo programı. Benzetmeler yine ŞEKİL EŞLEŞTİRME OYUNCAĞI ile sürer.
//
// Bu dosyanın tek cümlelik özeti: Desenler "kaç parça var, ilki ne, içinde ne var?" sorularını
// tek ifadede sorar; unuttuğun bir durum olursa derleyici onu sana gösterir.
//
// MİNİ SÖZLÜK (yalnız bu dosyada yeni geçenler; öteki terimler main.rs başındaki sözlükte):
//   dilim (slice) deseni     → bir dizinin uzunluğuna ve elemanlarına birlikte bakan desen.
//   `kalan @ ..`             → "geri kalan elemanların hepsi" (hiç eleman kalmamış da olabilir).
//   bağlama modu (binding mode) → desenin parçayı ALMASI mı yoksa yalnız ödünç BAKMASI mı.
//   ref / ref mut            → desende "alma, ödünç bak" / "alma, değiştirmek için ödünç al".
//   match ergonomics         → ödünç bir değeri açınca içindekilerin de ödünç gelmesi.
//   matches!                 → "bu değer şu desene uyuyor mu?" sorusunun evet/hayır cevabı.
//   ayrıştırıcı (parser)     → düz yazıyı okuyup programın anlayacağı yapıya çeviren kod.
//   argüman                  → komutun yanında verilen ek bilgi ("ekle vida 40" → vida, 40).
//   Result, Ok, Err          → "ya sonuç ya hata" kutusu ve iki hâli.
//   özyineleme (recursion)   → bir fonksiyonun işi küçültüp kendini yeniden çağırması.
//   refactoring              → davranışı değiştirmeden kodu yeniden düzenlemek.
//   saturating_add           → taşacaksa en büyük değerde duran, başa sarmayan toplama.
//   unsigned_abs             → mutlak değeri, taşma olmayacak biçimde işaretsiz sayı olarak almak.
//   never türü (`!`)         → "buradan geri dönülmez" demek (return, break, panic gibi).

pub fn calistir() {
    println!("\n== Derinleşme: Patterns — envanter komut ayrıştırıcısı ==");
    komut_ayristirici();
    baglama_modlari();
    ic_ice_at_ve_guard();
    dilim_desenleri();
    butunluk_ve_matches();
}

// 1. Gerçek hayattan örnek: komut ayrıştırıcı
// Kullanıcı satırını işlemek "kaç parça, ilki ne, gerisi ne?" sorularıdır. Desensiz `len()` ve
// `parcalar[1]` dağılır; biri güncellenip diğeri unutulunca "index out of bounds" paniği olur.
// Dilim deseni uzunluk testiyle erişimi TEK ifadede birleştirir.
// SADE ANLATIM: Kullanıcının yazdığı satırı kelimelere bölünce elinde bir sıra parça olur. Her
// komut bu sıra için bir deliktir: "tam iki parça var ve ilki `sil` mi?" Dilim deseni iki soruyu
// birden sorar: kaç parça olduğunu ve belli parçaların ne olduğunu. Uyarsa geri kalan parçalara
// ad da verir.
// Elle yaparsan uzunluğa bakıp `parcalar[1]` alırsın; iki satır olduğu için biri unutulup panik olur.
// Desende uzunluk uymuyorsa o kola hiç girilmez.
//   Mini örnek:
//     match ["sil", "vida"].as_slice() {
//         [] => println!("boş"),                            // hiç parça yok
//         ["listele"] => println!("listele"),               // tek parça ve o "listele"
//         ["sil", ad] => println!("{ad} silinecek"),        // ✓ iki parça, ilki "sil": ad = "vida"
//         [ilk, kalan @ ..] => println!("{ilk} + {} parça", kalan.len()),   // en az bir parça
//     }
#[derive(Debug, PartialEq)]
enum Komut {
    Ekle { ad: String, adet: u32 },
    Sil { ad: String },
    Listele,
    Not(String),
}

#[derive(Debug, PartialEq)]
enum AyristirmaHatasi {
    Bos,
    GecersizAdet(String),
    YanlisArguman { komut: String, verilen: usize },
    Bilinmeyen(String),
}

fn ayristir(satir: &str) -> Result<Komut, AyristirmaHatasi> {
    let parcalar: Vec<&str> = satir.split_whitespace().collect();
    // Adım adım kod analizi:
    // (1) `as_slice()`: dilim deseni Vec'e değil dilime/diziye uygulanır.
    // (2) Kollar yukarıdan aşağı denenir → özel biçimler önce, genel "yakalayıcılar" sonra.
    // (3) `["ekle", ad]` hem uzunluğu (2) hem ilk elemanı ("ekle") test eder, ikinciyi bağlar.
    // (4) `kalan @ ..` geri kalan elemanları alt dilim olarak bağlar (boş da olabilir).
    // (5) `komut @ ("ekle" | "sil" | ...)`: alternatiflerden biri tuttuysa HANGİSİ olduğunu sakla.
    match parcalar.as_slice() {
        [] => Err(AyristirmaHatasi::Bos),
        ["listele"] => Ok(Komut::Listele),
        ["sil", ad] => Ok(Komut::Sil { ad: ad.to_string() }),
        ["ekle", ad] => Ok(Komut::Ekle {
            ad: ad.to_string(),
            adet: 1,
        }),
        ["ekle", ad, adet] => {
            // let-else: sayı değilse burada dön; tutarsa `adet` girintisiz devam eder.
            let Ok(adet @ 1..) = adet.parse::<u32>() else {
                return Err(AyristirmaHatasi::GecersizAdet(adet.to_string()));
            };
            Ok(Komut::Ekle {
                ad: ad.to_string(),
                adet,
            })
        }
        ["not", ilk, kalan @ ..] => {
            let mut metin = ilk.to_string();
            for kelime in kalan {
                metin.push(' ');
                metin.push_str(kelime);
            }
            Ok(Komut::Not(metin))
        }
        [
            komut @ ("ekle" | "sil" | "listele" | "not"),
            argumanlar @ ..,
        ] => Err(AyristirmaHatasi::YanlisArguman {
            komut: komut.to_string(),
            verilen: argumanlar.len(),
        }),
        [bilinmeyen, ..] => Err(AyristirmaHatasi::Bilinmeyen(bilinmeyen.to_string())),
    }
}

#[derive(Debug, PartialEq)]
struct Kayit {
    ad: String,
    adet: u32,
}

fn uygula(envanter: &mut Vec<Kayit>, komut: Komut) -> String {
    match komut {
        Komut::Ekle { ad, adet } => {
            // iter_mut().find(..) → Option<&mut Kayit>. Desen &mut'un İÇİNE iner: `mevcut`
            // kendiliğinden &mut u32 olur (bölüm 2'deki match ergonomics).
            if let Some(Kayit { adet: mevcut, .. }) = envanter.iter_mut().find(|k| k.ad == ad) {
                // Adet kullanıcı girdisinden geliyor: `+=` taşmada panikler, doyurarak topla.
                *mevcut = mevcut.saturating_add(adet);
                format!("{ad}: {mevcut} adet oldu")
            } else {
                let mesaj = format!("{ad} eklendi ({adet} adet)");
                envanter.push(Kayit { ad, adet });
                mesaj
            }
        }
        Komut::Sil { ad } => match envanter.iter().position(|k| k.ad == ad) {
            // remove() Kayit'ın sahipliğini verir; desen onu parçalarına TAŞIYARAK ayırır.
            Some(sira) => {
                let Kayit { ad, adet } = envanter.remove(sira);
                format!("{ad} silindi ({adet} adet vardı)")
            }
            None => format!("{ad} bulunamadı"),
        },
        Komut::Listele => match envanter.as_slice() {
            [] => String::from("envanter boş"),
            [tek] => format!("1 kalem: {}", tek.ad),
            [ilk, .., son] => format!("{} kalem: {} … {}", envanter.len(), ilk.ad, son.ad),
        },
        Komut::Not(metin) => format!("not alındı: {metin}"),
    }
}

fn komut_ayristirici() {
    println!("-- 1. Komut ayrıştırıcı");
    let mut envanter = Vec::new();
    for satir in [
        "listele",
        "ekle vida 40",
        "ekle somun",
        "ekle vida 2",
        "not raf  3 dolu",
        "listele",
        "sil somun",
        "listele",
        "ekle vida çok",
        "sil",
        "boya kırmızı",
        "   ",
    ] {
        match ayristir(satir) {
            Ok(komut) => println!("{satir:?} → {}", uygula(&mut envanter, komut)),
            Err(hata) => println!("{satir:?} → hata: {hata:?}"),
        }
    }
    // Beklenen çıktı:
    //   "listele" → envanter boş
    //   "ekle vida 40" → vida eklendi (40 adet)
    //   "ekle somun" → somun eklendi (1 adet)
    //   "ekle vida 2" → vida: 42 adet oldu
    //   "not raf  3 dolu" → not alındı: raf 3 dolu
    //   "listele" → 2 kalem: vida … somun
    //   "sil somun" → somun silindi (1 adet vardı)
    //   "listele" → 1 kalem: vida
    //   "ekle vida çok" → hata: GecersizAdet("çok")
    //   "sil" → hata: YanlisArguman { komut: "sil", verilen: 0 }
    //   "boya kırmızı" → hata: Bilinmeyen("boya")
    //   "   " → hata: Bos
}

// 2. Bağlama modları (binding modes): desen değeri taşır mı, ödünç mü alır?
// SADE ANLATIM: Bir paketi açtığında içindekileri ya ALIRSIN (paket boşalır) ya da onlara yalnız
// BAKARSIN. Rust'ta kural basittir: paketin kendisi üzerinde eşleştirirsen parçalar dışarı
// taşınır; pakete `&` ile bakarak eşleştirirsen parçalar da ödünç gelir, paket yerinde kalır.
// `&mut` ile bakarsan parçaları yerinde değiştirebilirsin.
// Bakarken parçayı taşırsan paketin o gözü boş kalır, paketi bütün olarak kullanamazsın.
//   Mini örnek:
//     let k = Kayit { ad: String::from("vida"), adet: 10 };
//     let Kayit { ad, adet } = &k;       // ✓ & üzerinden aç: ad ve adet ödünç, k yerinde
//     println!("{ad} {adet} {k:?}");     // ✓ k hâlâ kullanılabilir
//     let Kayit { ad, adet } = k;        // & yok: ad dışarı TAŞINDI
//     // println!("{k:?}");              // ✗ derlenmez: k'nin bir parçası artık yerinde değil
fn baglama_modlari() {
    println!("-- 2. Bağlama modları");
    // Nasıl çalışır? Bir desendeki her isim üç biçimden biriyle bağlanır:
    //   move (ya da tür Copy ise kopya)  → varsayılan, değerin KENDİSİ üzerinde eşleştirirken
    //   ref      → isme değerin &'i bağlanır
    //   ref mut  → isme değerin &mut'u bağlanır
    // Match ergonomics: eşleştirilen şey bir REFERANS, desen ise referans deseni değilse
    // (`&..` ile başlamıyorsa) derleyici referansın içine kendisi iner ve alt bağlamaları
    // otomatik `ref` / `ref mut` yapar. Bu yüzden günlük kodda `ref` yazmak nadiren gerekir.
    let mut kayit = Kayit {
        ad: String::from("vida"),
        adet: 10,
    };

    // Sık yapılan hata (a) — bakmak isterken taşımak:
    //   let Kayit { ad, adet } = kayit;   // ad: String → kayit.ad dışarı TAŞINDI
    //   println!("{kayit:?}");            // ✗ E0382: borrow of partially moved value: `kayit`
    // ✓ Referans üzerinde eşleştir: ad: &String, adet: &u32 olur, kayit'a dokunulmaz.
    let Kayit { ad, adet } = &kayit;
    println!("ödünç: {ad} × {adet}"); // ödünç: vida × 10

    // Aynı şeyin açık yazımı: `ref` desenin SOL tarafında "taşıma, ödünç al" der. Alanları
    // FARKLI modlarla bağlamak gerektiğinde işe yarar: burada ad ödünç, adet (u32) kopya.
    let Kayit { ref ad, adet } = kayit;
    println!("ref: {ad} × {adet}, kayıt yerinde: {}", kayit.ad); // ref: vida × 10, kayıt yerinde: vida

    // Sık yapılan hata (b) — kopyayı değiştirmeye çalışmak:
    //   match kayit { Kayit { adet, .. } => adet += 1 }
    //                      // ✗ E0384: cannot assign twice to immutable variable `adet`
    // `adet` burada kayit.adet'in KOPYASI olan yeni bir değişkendir; `mut adet` yazıp derletsen
    // bile kayıt değişmezdi. ✓ &mut üzerinde eşleştir → adet: &mut u32, asıl alanı gösterir.
    let Kayit { adet, .. } = &mut kayit;
    *adet += 5;
    // Açık yazımı `ref mut`: sahip olunan değerin içinden değiştirilebilir ödünç alır.
    let Kayit {
        adet: ref mut sayac,
        ..
    } = kayit;
    *sayac *= 2;
    println!("güncel: {} × {}", kayit.ad, kayit.adet); // güncel: vida × 30

    // Sık yapılan hata (c) — referans desenini elle yazıp içini taşımak:
    //   let secili: &Option<String> = ..;
    //   match secili { &Some(s) => .., &None => .. }
    //         // ✗ E0507: cannot move out of `secili` as enum variant `Some` which is behind
    //         //   a shared reference
    // `&Some(s)` "referansı aç, sonra s'yi TAŞI" demektir; ödünç alınmış şeyin içi taşınamaz.
    // ✓ `&`'i yazma, ergonomics s'yi &String yapsın:
    let secili: &Option<String> = &Some(String::from("somun"));
    match secili {
        Some(s) => println!("seçili: {s} ({} bayt)", s.len()), // seçili: somun (5 bayt)
        None => println!("seçim yok"),
    }
}

// 3. İç içe yapı çözme, `@` ve guard birlikte
// SADE ANLATIM: İç içe geçen matruşka bebekleri düşün. Desen, hepsini tek bakışta açabilir: "dış
// bebek Tikla olsun, içindeki Nokta'nın x'i 0 olsun, y'sine de ad ver". Aynı desende `@` ile
// bütüne ad verip (`nokta @ ...`) parçalara da ayrı ad verebilir, sonuna bir ek soru (guard)
// ekleyebilirsin. Kollar yukarıdan aşağı denenir; bu yüzden en özel durum en üstte durur.
// İç içe `if`lerle her katman için ayrı kontrol/girinti gerekir, unutulan durum görünmezdi.
//   Mini örnek:
//     match &olay {
//         Olay::Tikla { konum: Nokta { x: 0, y }, .. } => println!("sol kenar, satır {y}"),
//         Olay::Tikla { konum: n @ Nokta { x, y }, .. } if x == y => println!("köşegen {n:?}"),
//         Olay::Kaydir(m @ 1..) => println!("aşağı {m}"),       // 1 ve üstü; değere m adı verildi
//         _ => println!("başka"),                               // geri kalan her şey
//     }
#[derive(Debug, Clone, Copy, PartialEq)]
enum FareTusu {
    Sol,
    Sag,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Nokta {
    x: i32,
    y: i32,
}

#[derive(Debug, PartialEq)]
enum Olay {
    Tus { karakter: char, ctrl: bool },
    Tikla { konum: Nokta, tus: FareTusu },
    Kaydir(i32),
}

fn yorumla(olay: &Olay) -> String {
    match olay {
        // Literal alanlarla tam eşleşme: enum → struct-like varyant → alan değerleri.
        Olay::Tus {
            karakter: 'q',
            ctrl: true,
        } => String::from("çıkış"),
        // `@` iç içe desende: aralığı TEST ET ve tutan değeri `harf` olarak sakla. Parantez
        // gerekli: `harf @ 'a'..='z' | 'A'..='Z'` yazılsaydı `@` yalnız ilk alternatife ait
        // olurdu ve derleyici E0408 ile reddederdi (aşağıda).
        Olay::Tus {
            karakter: harf @ ('a'..='z' | 'A'..='Z'),
            ctrl: false,
        } => format!("harf: {harf}"),
        Olay::Tus { karakter, ctrl } => format!("tuş: {karakter:?} ctrl={ctrl}"),
        // Üç katman tek desende: Olay → Nokta → x literal'i; ayrıca tus varyantı.
        Olay::Tikla {
            konum: Nokta { x: 0, y },
            tus: FareTusu::Sol,
        } => format!("sol kenar çubuğu, satır {y}"),
        // `nokta @ Nokta { x, y }`: hem bütünü hem parçaları bağla. Guard iki bağlamayı
        // karşılaştırıyor; bunu desen tek başına ifade edemez (desen değerleri sabitlerle
        // karşılaştırır, birbirleriyle değil).
        Olay::Tikla {
            konum: nokta @ Nokta { x, y },
            ..
        } if x == y => format!("köşegende tık: {nokta:?}"),
        Olay::Tikla {
            konum: Nokta { x, y },
            tus,
        } => format!("tık ({x},{y}) {tus:?}"),
        // Guard yerine aralık: üç kol tüm i32'leri kapsıyor ve derleyici bunu KANITLAYABİLİYOR.
        // unsigned_abs: `abs()` i32::MIN'de taşar (karşılığı i32'ye sığmaz) ve panikler.
        Olay::Kaydir(miktar @ ..=-1) => format!("yukarı {}", miktar.unsigned_abs()),
        Olay::Kaydir(0) => String::from("kaydırma yok"),
        Olay::Kaydir(miktar @ 1..) => format!("aşağı {miktar}"),
    }
}

fn ic_ice_at_ve_guard() {
    println!("-- 3. İç içe yapı çözme, @ ve guard");
    let olaylar = [
        Olay::Tus {
            karakter: 'q',
            ctrl: true,
        },
        Olay::Tus {
            karakter: 'q',
            ctrl: false,
        },
        Olay::Tus {
            karakter: '7',
            ctrl: false,
        },
        Olay::Tikla {
            konum: Nokta { x: 0, y: 12 },
            tus: FareTusu::Sol,
        },
        Olay::Tikla {
            konum: Nokta { x: 5, y: 5 },
            tus: FareTusu::Sag,
        },
        Olay::Tikla {
            konum: Nokta { x: 3, y: 8 },
            tus: FareTusu::Sag,
        },
        Olay::Kaydir(-3),
        Olay::Kaydir(2),
    ];
    for olay in &olaylar {
        println!("{}", yorumla(olay));
    }
    // Beklenen çıktı:
    //   çıkış
    //   harf: q
    //   tuş: '7' ctrl=false
    //   sol kenar çubuğu, satır 12
    //   köşegende tık: Nokta { x: 5, y: 5 }
    //   tık (3,8) Sag
    //   yukarı 3
    //   aşağı 2

    // Sık yapılan hatalar:
    //   Komut::Ekle { ad, .. } | Komut::Listele => println!("{ad}"),
    //                        // ✗ E0408: variable `ad` is not bound in all patterns
    //   `|` ile birleşen her alternatif AYNI isimleri bağlamalı; yoksa Listele tuttuğunda
    //   `ad` neye eşit olurdu? ✓ Kolları ayır ya da ortak olmayan ismi bağlama.
    //
    //   match n { x if x < 0 => "negatif", x if x >= 0 => "sıfır ya da pozitif" }
    //                        // ✗ E0004: non-exhaustive patterns: `i32::MIN..=i32::MAX` not covered
    //   İki guard birlikte her sayıyı kapsıyor ama derleyici guard'ın İÇİNE bakmaz (main.rs,
    //   derleyici perspektifi). ✓ Son kolu guard'sız bırak ya da Kaydir'daki gibi aralık yaz.
    //
    // Önemli nokta — guard, `|` zincirinin TAMAMINA uygulanır: `1 | 2 if kosul` deseni
    // `(1 | 2) if kosul` demektir, `1 | (2 if kosul)` değil.
    let izinli = false;
    let sonuc = match 1 {
        1 | 2 if izinli => "bir ya da iki, izinli",
        _ => "diğer",
    };
    println!("guard önceliği: {sonuc}"); // guard önceliği: diğer
}

// 4. Dilim desenleri
// SADE ANLATIM: Bir tren düşün: en önde lokomotif, en sonda son vagon, arada bilinmeyen sayıda
// vagon. `[ilk, .., son]` "baştakini ve sondakini ver, aradakilere bakmıyorum" demektir.
// `[ilk, kalan @ ..]` ise "baştakini ayır, geri kalan vagonları da bana `kalan` adıyla ver"
// demektir; geriye hiç vagon kalmamış da olabilir.
// Boş dilim de dilimdir; onu unutursan derleyici "boş dilim açıkta kaldı" diyerek geri çevirir.
//   Mini örnek:
//     match [3, 1, 4].as_slice() {
//         [] => println!("boş"),
//         [tek] => println!("tek: {tek}"),
//         [ilk, .., son] => println!("{ilk} … {son}"),    // ✓ 3 … 4
//     }
//     // `[] =>` kolunu silersen                          // ✗ derlenmez: boş dilim açıkta kalır
fn ozetle(sayilar: &[i32]) -> String {
    match sayilar {
        [] => String::from("boş"),
        [tek] => format!("tek eleman: {tek}"),
        [ilk, son] => format!("iki eleman: {ilk} ve {son}"),
        // `orta @ ..`: aradakiler alt dilim (&[i32]) olarak bağlanır. Aynı dilim deseninde `..`
        // yalnız BİR kez geçebilir; iki tane olsaydı nerede bölüneceği belirsiz kalırdı.
        [ilk, orta @ .., son] => format!("{ilk} … {son} (arada {} eleman)", orta.len()),
    }
}

// Özyineleme + dilim deseni: "boş" ve "baş + kuyruk" iki durumu tüm dilimleri kapsar.
fn topla(sayilar: &[i32]) -> i32 {
    match sayilar {
        [] => 0,
        [ilk, kalan @ ..] => ilk + topla(kalan),
    }
}

fn dilim_desenleri() {
    println!("-- 4. Dilim desenleri");
    for dilim in [&[][..], &[7], &[1, 2], &[3, 1, 4, 1, 5]] {
        println!("{dilim:?} → {} (toplam {})", ozetle(dilim), topla(dilim));
    }
    // Beklenen çıktı:
    //   [] → boş (toplam 0)
    //   [7] → tek eleman: 7 (toplam 7)
    //   [1, 2] → iki eleman: 1 ve 2 (toplam 3)
    //   [3, 1, 4, 1, 5] → 3 … 5 (arada 3 eleman) (toplam 14)

    // Dizi (uzunluğu türde yazılı) ile dilim (uzunluğu çalışma anında belli) farkı:
    // [u8; 3] için `[r, g, b]` REDDEDİLEMEZ bir desendir, düz `let` ile kullanılır.
    let renk: [u8; 3] = [255, 128, 0];
    let [r, g, b] = renk;
    // Dilimde aynı desen reddedilebilirdir (uzunluk 3 olmayabilir) → let-else gerekir.
    let parcalar = ["tasi", "vida", "raf-2"];
    let [komut, argumanlar @ ..] = parcalar.as_slice() else {
        return;
    };
    println!(
        "rgb = {r},{g},{b} | komut = {komut}, {} argüman",
        argumanlar.len()
    ); // rgb = 255,128,0 | komut = tasi, 2 argüman

    // Sık yapılan hatalar:
    //   match sayilar { [ilk, kalan @ ..] => ilk + topla(kalan) }
    //                               // ✗ E0004: non-exhaustive patterns: `&[]` not covered
    //   Boş dilim de bir dilimdir. İndeksle yazsaydın bu unutkanlık çalışma anında panic olurdu.
    //   let v = vec![1, 2, 3];
    //   match v { [ilk, ..] => .., _ => .. }
    //                               // ✗ E0529: expected an array or slice, found `Vec<{integer}>`
    //   ✓ `match v.as_slice()` (ayristir'daki gibi) ya da `match &v[..]`.
    //   let [r, g] = renk;          // ✗ E0527: pattern requires 2 elements but array has 3
    //   ✓ `let [r, g, _] = renk;` ya da `let [r, g, ..] = renk;`
}

// 5. Bütünlük denetimi refactoring'i nasıl güvenli kılar? + matches!
// SADE ANLATIM: Oyuncağa yeni bir şekil (beşgen) eklendiğini düşün. Kapakta beşgen delik yoksa
// derleyici "bu parça nereye girecek?" diye seni durdurur; verdiği hata listesi, güncellemen
// gereken yerlerin listesidir. Kapakta "ne gelirse yutan" bir delik (`_`) varsa beşgen sessizce
// oraya düşer ve kimse fark etmez. `matches!` ise yalnız bir soru sorar: "bu parça şu delikten
// geçer mi?" Cevabı evet ya da hayırdır.
// Kendi enum'unda `_` bu korumadan vazgeçmektir; "yeni gelen her şeye aynısını yaparım" dersen yaz.
//   Mini örnek:
//     let dolu = matches!(Some(3), Some(1..));         // ✓ true — "içi dolu ve en az 1 mi?"
//     let bos = matches!(None::<i32>, Some(_));        // ✓ false
//     // match komut { Komut::Listele => .., _ => .. } // derlenir; yeni varyant `_`a düşer
fn butunluk_ve_matches() {
    println!("-- 5. Bütünlük ve matches!");
    // Yanlış kullanım / Doğru kullanım: uygula() içindeki `match komut` dört varyantı da
    // ADIYLA sayıyor, `_ =>` kolu yok. Komut'a `Tasi { .. }` varyantı eklendiği gün uygula()
    // E0004 ile derlenmez; derleyicinin hata listesi "güncellemen gereken yerler" listesidir.
    //   _ => String::from("desteklenmiyor")   // bu kol olsaydı yeni komut SESSİZCE buraya düşerdi
    // Kural: kendi enum'unda `_` yazmadan önce "yeni varyant gelirse burada gerçekten aynı şeyi
    // mi yapmak isterim?" diye sor. Evet ise (ayristir'daki bilinmeyen komut gibi açık uçlu
    // girdilerde) `_` doğrudur; hayır ise varyantları tek tek yaz.
    // (closure parametresi de bir desendir: tuple burada `(ad, adet)` diye açılıyor.)
    let envanter: Vec<Kayit> = [("vida", 42), ("pul", 0), ("somun", 3)]
        .into_iter()
        .map(|(ad, adet)| Kayit {
            ad: ad.to_string(),
            adet,
        })
        .collect();

    // matches!: desen tutuyor mu? → bool (guard da alır); filter/assert gibi bool beklenen yerlerde.
    let tukenen = envanter
        .iter()
        .filter(|k| matches!(k, Kayit { adet: 0, .. }))
        .count();
    let azalan: Vec<&str> = envanter
        .iter()
        .filter(|k| matches!(k.adet, 1..=5))
        .map(|k| k.ad.as_str())
        .collect();
    let kisa_adli_dolu = envanter
        .iter()
        .any(|k| matches!(k, Kayit { ad, adet: 1.. } if ad.len() <= 4));
    println!("tükenen: {tukenen}, azalan: {azalan:?}, kısa adlı dolu var mı: {kisa_adli_dolu}"); // tükenen: 1, azalan: ["somun"], kısa adlı dolu var mı: true
    // Dikkat: matches! içeride `_ => false` kolu olan bir match'e açılır; yani bütünlük
    // korumasından bilerek vazgeçersin. Yeni varyant eklendiğinde matches! sessizce false der.

    // let-else + iç içe desen: "ilk kayıt var VE adedi sıfırdan büyük" tek satırda.
    let Some(Kayit {
        ad,
        adet: adet @ 1..,
    }) = envanter.first()
    else {
        println!("gösterilecek dolu kayıt yok");
        return;
    };
    println!("ilk dolu kayıt: {ad} ({adet})"); // ilk dolu kayıt: vida (42)
}

// SADE ANLATIM (aşağıdaki bölüm için): Aşağıda üç alıştırma ve bu konunun öteki bölümlerle
// bağlantısı var. İlk alıştırma en öğreticisidir: enum'a yeni bir komut ekleyip derleyicinin
// seni hangi satırlara götürdüğünü izlersin. İlk okuyuşta bu bölümü atlayabilirsin.
// ============================================================================
// ALIŞTIRMALAR (çözümler verilmedi)
// ============================================================================
//
// 1. Ayrıştırıcıya `tasi <ad> <kaynak> <hedef>` komutunu ekle (ör. "tasi vida raf-1 raf-2").
//    İpucu: önce yalnız `Komut::Tasi { .. }` varyantını ekleyip `cargo check -p patterns`
//    çalıştır; derleyicinin seni nereye götürdüğünü ve nereye GÖTÜRMEDİĞİNİ not et. ayristir()
//    içindeki `komut @ (..)` listesini de güncellemeyi unutma: orası bütünlük denetiminin
//    göremediği bir yer (string literal'leri açık uçludur).
//    Beklenen: ayristir("tasi vida raf-1 raf-2") → Ok(Tasi { .. }); "tasi vida" →
//    Err(YanlisArguman { komut: "tasi", verilen: 1 }).
//
// 2. `fn uclar(sayilar: &[i32]) -> Option<(i32, i32)>` yaz: en az iki eleman varsa ilk ve
//    sonu, tek eleman varsa onu iki kez, boşsa None döndürsün. İndeksleme ve len() kullanma.
//    İpucu: ozetle()'deki kolların sırasına bak; `&[i32]` üzerinde bağlanan isimler &i32'dir.
//    Beklenen: uclar(&[3, 1, 4]) == Some((3, 4)), uclar(&[7]) == Some((7, 7)), uclar(&[]) == None.
//
// 3. yorumla()'ya "ctrl ile basılan rakam" durumunu ekle: `Tus { karakter: '0'..='9', ctrl:
//    true }` için "sekme N" döndürsün.
//    İpucu: rakamı `@` ile bağla; yeni kolu NEREYE koyduğun sonucu değiştirir (kollar sırayla
//    denenir). Yanlış yere koyarsan derleyicinin "unreachable pattern" uyarısını ara.
//    Beklenen: ctrl+3 → "sekme 3"; ctrl'siz '7' hâlâ "tuş: '7' ctrl=false".
//
// Diğer konularla bağlantısı:
// Bir önceki `learn/oop` (Bölüm 18) "türe göre davranış" problemini trait object ile çözer (GUI örneği);
// buradaki enum + match çözümüyle karşılaştır: kapalı bir varyant kümesine yeni DAVRANIŞ
// eklemek enum'da kolaydır (yeni bir fn + match), açık bir kümeye yeni TÜR eklemek trait
// object'te. Aynı crate'teki type-state bölümü, enums/derinlesme.rs'teki "geçersiz durumu
// temsil edilemez kıl" fikrinin tür düzeyindeki hâlidir. `learn/advanced-features` (Bölüm 20)
// never türünü (`!`) anlatır: let-else'in `else` bloğunun return/break/panic ile bitmek
// ZORUNDA oluşunun ve bir match kolunda panic!() yazabilmenin açıklaması odur.

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn sinir_degerler_panik_ettirmez() {
        // abs() burada taşardı; unsigned_abs i32::MIN'in mutlak değerini u32 olarak verir.
        assert_eq!(yorumla(&Olay::Kaydir(i32::MIN)), "yukarı 2147483648");
        // İki büyük ekleme u32'yi aşar; doyurarak toplama u32::MAX'te kalır.
        let mut envanter = Vec::new();
        for _ in 0..2 {
            uygula(&mut envanter, ayristir("ekle vida 4000000000").unwrap());
        }
        assert_eq!(envanter[0].adet, u32::MAX);
    }

    #[test]
    fn gecerli_komutlar_ayristirilir() {
        assert_eq!(ayristir("listele"), Ok(Komut::Listele));
        assert_eq!(
            ayristir("  ekle   vida  40 "),
            Ok(Komut::Ekle {
                ad: String::from("vida"),
                adet: 40
            })
        );
        assert_eq!(
            ayristir("ekle somun"),
            Ok(Komut::Ekle {
                ad: String::from("somun"),
                adet: 1
            })
        );
        assert_eq!(
            ayristir("not raf 3 dolu"),
            Ok(Komut::Not(String::from("raf 3 dolu")))
        );
    }

    #[test]
    fn hatali_satirlar_dogru_hatayi_verir() {
        assert_eq!(ayristir("   "), Err(AyristirmaHatasi::Bos));
        assert_eq!(
            ayristir("ekle vida sıfır"),
            Err(AyristirmaHatasi::GecersizAdet(String::from("sıfır")))
        );
        assert_eq!(
            ayristir("ekle vida 0"),
            Err(AyristirmaHatasi::GecersizAdet(String::from("0")))
        );
        assert_eq!(
            ayristir("sil"),
            Err(AyristirmaHatasi::YanlisArguman {
                komut: String::from("sil"),
                verilen: 0
            })
        );
        assert_eq!(
            ayristir("boya kırmızı"),
            Err(AyristirmaHatasi::Bilinmeyen(String::from("boya")))
        );
    }

    #[test]
    fn ekleme_var_olan_kaydi_yerinde_artirir() {
        let mut envanter = Vec::new();
        for satir in ["ekle vida 40", "ekle vida 2", "ekle somun"] {
            uygula(&mut envanter, ayristir(satir).unwrap());
        }
        assert_eq!(envanter.len(), 2);
        assert_eq!(envanter[0].adet, 42);
        assert_eq!(
            uygula(&mut envanter, Komut::Listele),
            "2 kalem: vida … somun"
        );
        assert_eq!(
            uygula(&mut envanter, ayristir("sil vida").unwrap()),
            "vida silindi (42 adet vardı)"
        );
        assert_eq!(uygula(&mut envanter, Komut::Listele), "1 kalem: somun");
    }

    #[test]
    fn dilim_desenleri_her_uzunlugu_kapsar() {
        assert_eq!(ozetle(&[]), "boş");
        assert_eq!(ozetle(&[7]), "tek eleman: 7");
        assert_eq!(ozetle(&[1, 2]), "iki eleman: 1 ve 2");
        assert_eq!(ozetle(&[3, 1, 4, 1, 5]), "3 … 5 (arada 3 eleman)");
        assert_eq!(topla(&[]), 0);
        assert_eq!(topla(&[3, 1, 4, 1, 5]), 14);
    }

    #[test]
    fn olaylar_en_ozel_kola_duser() {
        let tik = |x, y, tus| Olay::Tikla {
            konum: Nokta { x, y },
            tus,
        };
        assert_eq!(
            yorumla(&tik(0, 4, FareTusu::Sol)),
            "sol kenar çubuğu, satır 4"
        );
        assert_eq!(
            yorumla(&tik(0, 0, FareTusu::Sag)),
            "köşegende tık: Nokta { x: 0, y: 0 }"
        );
        assert_eq!(yorumla(&tik(0, 4, FareTusu::Sag)), "tık (0,4) Sag");
        assert_eq!(yorumla(&Olay::Kaydir(0)), "kaydırma yok");
        assert_eq!(yorumla(&Olay::Kaydir(-3)), "yukarı 3");
        let harf = Olay::Tus {
            karakter: 'Z',
            ctrl: false,
        };
        assert_eq!(yorumla(&harf), "harf: Z");
    }
}
