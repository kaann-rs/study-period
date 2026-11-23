// Açıklama: learn/advanced-concepts/ileri-lifetime-hrtb-gat/ACIKLAMA.md
// Sağlamlaştırma — İleri konu: İLERİ LIFETIME'LAR, HIGHER-RANKED TRAIT BOUNDS (for<'a>) ve GAT
// Çalıştır: cargo run -p ileri-lifetime-hrtb-gat   |   Test: cargo test -p ileri-lifetime-hrtb-gat
//
// Kaynaklar: The Rust Reference (Lifetime bounds, Higher-ranked trait bounds, Associated
// types), Rustonomicon (Higher-Rank Trait Bounds), GAT'ı kararlı yapan 1.65 duyurusu ve
// Rust for Rustaceans Böl. 1–2 (Foundations, Types).
//
// Böl. 10 (generics-traits-lifetimes) lifetime'ı "bu çıktı şu girdiden türedi" bildirimi olarak
// öğretti; ileri-variance alt tiplemeyi açtı. Burada üç soruya cevap arıyoruz:
//   1) İki lifetime arasındaki ilişkiyi ('a: 'b, T: 'a) NE ZAMAN elle yazmak gerekir?
//   2) Lifetime'ı çağıranın değil, ÇAĞRILANIN seçtiği durumları nasıl ifade ederiz? → for<'a>
//   3) Associated type'ın KENDİSİ bir lifetime'a (ya da türe) bağlıysa ne olur? → GAT
//
// ÖZET ZİHİNSEL MODEL (gerisi bunun ayrıntısı):
//   Bir generic parametreyi KİM seçer? `fn f<'a>(..)` → 'a'yı ÇAĞIRAN seçer; f her seçime
//   razıdır ama gövdesi içinde 'a sabit, tek bir süredir. `F: for<'a> Fn(&'a T)` → 'a'yı F'yi
//   ÇAĞIRAN (yani biz, gövdenin içinden) seçeriz; F her süreye razı olmak zorundadır.
//   HRTB bu "kim seçer" sorusunu tersine çevirmenin sözdizimidir. GAT ise aynı esnekliği
//   associated type'a taşır: `type Oge<'a>` her kullanımda farklı bir süreyle örneklenebilir.
//
// ✗ işaretli derlenmeyen örneklerin hata kodları/mesajları rustc 1.99 ile doğrulandı.

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Her bölümde önce "SADE ANLATIM" kutusunu oku; teknik kısım zor gelse de ana fikir orada.
//
// Bu dosyayı okumadan önce bilmen gerekenler: `learn/ownership` (ödünç alma),
// `learn/generics-traits-lifetimes` (lifetime ve associated type), `learn/closures-iterators`
// (closure ve Iterator) ve bu klasördeki `ileri-variance`.
//
// Bu bölümün tek cümlelik özeti: lifetime etiketleri "hangi referans hangisine bağlı" bilgisini
// taşır; `for<'a>` "hangi süreyi verirsen ver çalışırım" sözüdür; GAT ise bir trait'in içindeki
// türe, her kullanımda yeniden doldurulan boş bir hane ekler.
//
// MİNİ SÖZLÜK (bu dosyada geçen kelimeler):
//   lifetime (ömür, 'a)  → bir referansın geçerli olduğu kod aralığının adı. Hiçbir şeyin
//                          ömrünü uzatmaz; yalnız ilişkiyi etiketler.
//   'static              → "geçici hiçbir şeye borçlu değil" etiketi. Sonsuza dek yaşar demek
//                          değildir: bir String de 'static sayılır ve hemen silinebilir.
//   elision              → derleyicinin, yazmadığın lifetime etiketlerini kurala göre tamamlaması.
//   bound (sınır)        → bir generic parametreye konan şart: `T: Debug`, `'a: 'b`.
//   outlives ('a: 'b)    → "'a, 'b bitmeden bitmez", yani en az onun kadar sürer.
//   T: 'a                → "T'nin içindeki bütün referanslar en az 'a kadar geçerli".
//   implied bounds       → derleyicinin tür yazımından kendiliğinden çıkardığı şartlar.
//   covariant / invariant→ sürenin kısaltılabildiği / hiç esnetilemediği yer (ileri-variance).
//   closure              → yerinde yazılan adsız fonksiyon: `|s| s.trim()`.
//   Fn / FnMut / FnOnce  → "çağrılabilir şey" trait'leri (closure'lar ve fonksiyonlar).
//   fn işaretçisi        → bir fonksiyonun adresini tutan değer: `fn(&str) -> &str`.
//   HRTB / for<'a>       → higher-ranked trait bound: "HER 'a süresi için geçerli" şartı.
//   evrensel niceleme    → "her ... için" demenin matematikteki adı.
//   associated type      → trait'in içinde adı konmuş, impl'de doldurulan tür: `type Item`.
//   GAT                  → generic associated type: kendi parametresi olan associated type,
//                          ör. `type Oge<'a>` ya da `type Isaretci<T>`.
//   ödünç veren iterator → verdiği öğe kendisinden ödünç olan gezici (lending iterator).
//   dilim / pencere      → art arda elemanlar / dilimin sabit boylu, kayan bir parçası.
//   tür kurucusu / ailesi→ içi henüz doldurulmamış tür kalıbı: `Rc<_>`, `Arc<_>`.
//   trait object (dyn)   → gerçek türü gizlenmiş değer; dyn-uyumlu = `dyn Trait` yapılabilen.
//   tampon (buffer)      → verinin geçici olarak durduğu bellek alanı.
//   sıfır kopya          → veriyi kopyalamadan, olduğu yere bakarak kullanmak.
//   sahipli (owned)      → kimseden ödünç almayan, verisini kendi tutan değer (String, u32).
//   serde / Deserialize  → Rust'ın yaygın veri çözümleme kütüphanesi ve onun trait'i.
//   late-bound           → süresi tanımda değil, her çağrıda ayrı ayrı seçilen lifetime.
//   projeksiyon          → `<I as Trait>::Oge` yazımı: "I'nın bu trait'teki Oge türü".
//   erasure (silinme)    → lifetime'ların derleme bitince koddan tümüyle kaybolması.

use std::fmt::Debug;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

fn main() {
    println!("== 1) Lifetime ilişkileri: iki ayrı süre, 'a: 'b, T: 'a ==");
    lifetime_iliskileri();

    println!("\n== 2) HRTB: for<'a> — süreyi F'yi çağıran seçer, F her süreye razıdır ==");
    hrtb_closure();
    hrtb_trait();

    println!("\n== 3) GAT: ödünç veren iterator ==");
    gat_odunc_veren_iterator();

    println!("\n== 4) GAT: tür ailesi (Rc mi, Arc mı — sonradan seç) ==");
    gat_tur_ailesi();
}

// ============================================================================
// 1) LIFETIME İLİŞKİLERİ — tek 'a ne zaman yetmez?
// ============================================================================

// --- 1a. Struct'ta iki ayrı süre -------------------------------------------------------------
// Sorun: iki referans alanına AYNI 'a'yı verirsen derleyici ikisini ortak (yani KISA olan)
// süreye indirger (covariance — ileri-variance). Struct'tan çıkan her şey o kısa süreye bağlanır.
//
//   struct Arayici<'a> { metin: &'a str, kelime: &'a str }
//   impl<'a> Arayici<'a> { fn ilk(&self) -> Option<&'a str> { ... } }
//   let bulunan;
//   { let kelime = String::from("b");
//     bulunan = Arayici { metin: &metin, kelime: &kelime }.ilk(); }
//   println!("{bulunan:?}");      // ✗ E0597: `kelime` does not live long enough
//
// Çözüm: bağımsız yaşayan veriler için bağımsız parametreler.
// SADE ANLATIM: Bir kitapta kurşun kalemle bazı cümlelerin altını çizdin. Çizgiler kitabın
// içindedir; kalem bitse de kırılsa da çizgiler yerinde durur. `metin` kitaptır, `kelime`
// kalemdir, sonuç da çizilen yerlerdir. İkisine aynı etiketi (`'a`) verirsen derleyiciye
// "çizgiler kalem kadar yaşar" demiş olursun; kalem gidince sonucu da kullandırmaz. İki ayrı
// etiket (`'m`, `'k`) doğruyu söyler: sonuç yalnız kitaba bağlıdır.
//   Mini örnek:
//     struct Arayici<'m, 'k> { metin: &'m str, kelime: &'k str }   // iki etiket
//     let bulunan;
//     {
//         let kelime = String::from("kar");
//         bulunan = Arayici { metin: &metin, kelime: &kelime }.eslesmeler();
//     }                              // kelime burada silindi
//     println!("{bulunan:?}");       // ✓ sonuç yalnız metin'e bağlı, hâlâ geçerli
//     // Tek etiketli `struct Tek<'a> { metin: &'a str, kelime: &'a str }` ile aynı kod:
//     //                             // ✗ derlenmez: sonuç kelime'ye de bağlanmış sayılır
struct Arayici<'m, 'k> {
    metin: &'m str,
    kelime: &'k str,
}

impl<'m, 'k> Arayici<'m, 'k> {
    // Dönüş 'm: "sonuç metnin içindedir; kelimeyle ve Arayici'nin kendisiyle işi yoktur."
    fn eslesmeler(&self) -> Vec<&'m str> {
        self.metin
            .match_indices(self.kelime)
            .map(|(i, _)| &self.metin[i..i + self.kelime.len()])
            .collect()
    }
}

// --- 1b. Outlives bound: 'uzun: 'kisa ---------------------------------------------------------
// `'a: 'b` "a, b'yi KAPSAR" (en az onun kadar yaşar) diye okunur. İki farklı süreli
// referanstan birini döndürürken gerekir: 'b süreli bir dönüşe 'a süreli referans koymak
// ancak 'a daha uzunsa güvenlidir.
//
//   fn sec<'a, 'b>(x: &'a str, y: &'b str) -> &'b str { if y.is_empty() { x } else { y } }
//       ✗ error: lifetime may not live long enough
//         help: consider adding the following bound: `'a: 'b`
//
// Bound'u yazınca derleyici çağrı yerinde de aynı şartı arar: varsayılanın, sonucun
// kullanıldığı süreyi kapsaması gerekir.
// SADE ANLATIM: Maçta sakatlanan oyuncunun yerine yedek girer. Yedeğin en az maçın sonuna
// kadar sahada kalabilmesi gerekir; ilk yarıda eve gidecek biri yedek olamaz. `'uzun: 'kisa`
// yazımı bu şarttır: "`'uzun` süresi, `'kisa` bitmeden bitmez". Fonksiyon bazen `girdi`yi,
// bazen `varsayilan`ı döndürdüğü için, varsayılanın sonucun kullanılacağı süre boyunca
// geçerli kalacağına söz vermen gerekir. Bu söz olmadan derleyici, erken silinen bir yedeği
// döndürmediğinden emin olamaz.
//   Mini örnek:
//     fn sec<'a, 'b>(x: &'a str, y: &'b str) -> &'b str { if y.is_empty() { x } else { y } }
//     // ✗ derlenmez: x'in, y'nin süresi boyunca geçerli kalacağını kimse söylemedi
//     fn sec2<'a: 'b, 'b>(x: &'a str, y: &'b str) -> &'b str { if y.is_empty() { x } else { y } }
//     // ✓ `'a: 'b` → "x en az y kadar geçerli" sözü verildi
fn bos_ise_varsayilan<'kisa, 'uzun: 'kisa>(
    girdi: &'kisa str,
    varsayilan: &'uzun str,
) -> &'kisa str {
    if girdi.trim().is_empty() {
        varsayilan
    } else {
        girdi
    }
}

// --- 1c. Tür üzerinde outlives: T: 'a ---------------------------------------------------------
// `T: 'a` "T'nin içindeki BÜTÜN referanslar en az 'a kadar yaşar" demektir (T referans
// içermiyorsa her 'a için doğrudur). Bir T'yi 'a süreli bir kutuya/trait object'e koyarken
// gerekir; yoksa kutu, T'nin içindeki bir referanstan uzun yaşayabilirdi.
//
//   fn kutula<'a, T: Debug>(t: T) -> Box<dyn Debug + 'a> { Box::new(t) }
//       ✗ E0309: the parameter type `T` may not live long enough
//
// (`&'a T` alanı tutan struct'larda `T: 'a` yazmana gerek yoktur: derleyici bunu alan
//  türünden çıkarır — "implied bounds".)
// SADE ANLATIM: Çantana koyduğun şeylerin arasında kütüphaneden ödünç bir kitap varsa, o
// çantayı iade tarihinden sonra taşıyamazsın. `T: 'a` bu kuralın yazılışıdır: "T'nin içinde
// ödünç bir şey varsa, onun süresi en az `'a` kadardır". İçinde hiç ödünç olmayan türler
// (i32, String) bu şartı her süre için sağlar. `T: 'static` de aynı ailedendir: "sonsuza dek
// yaşar" demek DEĞİLDİR; "geçici hiçbir şeye borçlu değil" demektir.
//   Mini örnek:
//     let k1 = kutula(42);                // ✓ i32'nin içinde ödünç yok: her süreye uyar
//     let yerel = String::from("x");
//     let k2 = kutula(&yerel);            // ✓ kutu, yerel yaşadığı sürece kullanılabilir
//     fn statik_ister<T: 'static>(_: T) {}
//     statik_ister(String::from("x"));    // ✓ String kimseye borçlu değil; hemen silinse de olur
//     // statik_ister(&yerel);            // ✗ derlenmez: &yerel geçici bir değere borçlu
fn kutula<'a, T: Debug + 'a>(t: T) -> Box<dyn Debug + 'a> {
    Box::new(t)
}

// --- 1d. &mut'un arkasındaki lifetime: invariance pratikte -----------------------------------
//   fn sakla(liste: &mut Vec<&str>, s: &str) { liste.push(s); }
//       ✗ error: lifetime may not live long enough
//         note: mutable references are invariant over their type parameter
// Elision iki &str'e FARKLI süre verdi; &mut ardındaki tür "kısaltılıp uzatılamadığı" için
// derleyici `s`in, listenin beklediği süre kadar yaşadığını kanıtlayamaz. İlişkiyi biz yazarız:
// (Sade: "invariant" = süre ne uzatılabilir ne kısaltılabilir; tam tutması gerekir demek.)
// SADE ANLATIM: Sınıf gezisi listesinin başında "bu listedeki herkes akşama kadar bizimle"
// yazıyor. Öğlen ayrılacak birini bu listeye yazamazsın; yazarsan akşam yoklamada olmayan
// birini ararsın. `&mut Vec<&'a str>` böyle bir listedir: içine yazılabildiği için, eklenen
// her referans listenin süresine tam uymak zorundadır. Bu yüzden imzada iki yere de AYNI
// `'a` yazılır: "eklenen şey, listedekiler kadar geçerli".
//   Mini örnek:
//     let mut liste = Vec::new();
//     sakla(&mut liste, "sabit");             // ✓ sabit yazı her süreye uyar
//     {
//         let gecici = String::from("geçici");
//         // sakla(&mut liste, &gecici);      // ✗ derlenmez: gecici, listeden önce siliniyor
//     }
//     println!("{liste:?}");                  // liste burada hâlâ kullanılıyor
fn sakla<'a>(liste: &mut Vec<&'a str>, s: &'a str) {
    liste.push(s);
}

fn lifetime_iliskileri() {
    let metin = String::from("kar yağar, kar erir, kar kalmaz");
    let bulunan;
    {
        let kelime = String::from("kar"); // kısa ömürlü
        let arayici = Arayici {
            metin: &metin,
            kelime: &kelime,
        };
        bulunan = arayici.eslesmeler();
    } // kelime ve arayici öldü; sonuç 'm'ye bağlı olduğu için yaşıyor
    println!("   eşleşme sayısı = {}", bulunan.len()); // 3

    let varsayilan = String::from("anonim");
    let ad = {
        let girdi = String::from("   ");
        bos_ise_varsayilan(&girdi, &varsayilan).to_string()
    };
    println!("   boş girdi → {ad}"); // anonim

    let yerel = String::from("yerel veri");
    let kutu = kutula(&yerel); // T = &String; 'a = yerel'in ödünç süresi
    println!("   kutuda: {kutu:?}");

    let mut liste = Vec::new();
    sakla(&mut liste, &metin[..3]);
    sakla(&mut liste, "sabit");
    println!("   liste = {liste:?}"); // ["kar", "sabit"]
}

// ============================================================================
// 2) HIGHER-RANKED TRAIT BOUNDS — for<'a>
// ============================================================================

// --- 2a. Sorun: referans alan closure parametresi --------------------------------------------
// Bir fonksiyon, KENDİ İÇİNDE ürettiği veriye bakan bir closure çağıracak. Closure'ın
// parametre süresini fonksiyonun generic parametresi yaparsak:
//
//   fn uygula<'a, F: Fn(&'a str) -> &'a str>(f: F) -> usize {
//       let yerel = String::from(" a ");
//       f(&yerel).len()           // ✗ E0597: `yerel` does not live long enough
//   }
//
// Neden? 'a'yı ÇAĞIRAN seçer; `yerel` ise fonksiyonun içinde doğar. Çağıranın seçebileceği
// hiçbir 'a, henüz var olmayan bir yereli kapsayamaz. Gerekli söz: "F, vereceğim HER süreye razıdır."
//
// İşte bu `for<'a>`: bound'un içine gömülü, evrensel nicelenmiş ("her 'a için") bir süre.
// İyi haber: Fn sözdiziminde referans yazınca derleyici bunu KENDİ ekler; aşağıdaki iki
// imza birebir aynıdır:
//     F: Fn(&str) -> &str
//     F: for<'a> Fn(&'a str) -> &'a str
// SADE ANLATIM: İki terzi düşün. Biri "yalnız senin şimdi elinde tuttuğun kumaşı dikerim"
// der; öteki "hangi kumaşı getirirsen getir dikerim" der. Bir fonksiyon closure'a KENDİ
// içinde ürettiği veriyi verecekse ikinci terziye ihtiyacı vardır, çünkü o veri fonksiyon
// çağrılmadan önce ortada yoktur; çağıran onun süresini önceden seçemez. `for<'a>` ikinci
// terzinin sözüdür: "hangi süreyi verirsen ver çalışırım". `Fn(&str) -> &str` yazdığında
// derleyici bu sözü senin yerine ekler.
//   Mini örnek:
//     fn uygula<F: for<'a> Fn(&'a str) -> &'a str>(f: F) -> usize {
//         let yerel = String::from(" a ");    // fonksiyonun içinde doğdu
//         f(&yerel).len()                     // ✓ f her süreye razı, bu kısa süreye de
//     }
//     uygula(|s| s.trim());                   // ✓ 1
//     // fn uygula2<'a, F: Fn(&'a str) -> &'a str>(f: F) -> usize { /* aynı gövde */ }
//     // ✗ derlenmez: 'a'yı çağıran seçer; yerel o süre boyunca var olamaz
fn satirlari_donustur<F>(ham: &str, donustur: F) -> Vec<String>
where
    F: for<'a> Fn(&'a str) -> &'a str,
{
    ham.lines()
        .map(|satir| {
            let buyuk = satir.to_uppercase(); // her turda YENİ, kısa ömürlü bir String
            donustur(&buyuk).to_string() // closure her turda başka bir süreyle çağrılır
        })
        .collect()
}

// Fonksiyon işaretçisi türlerinde de aynı: `fn(&str) -> &str` = `for<'a> fn(&'a str) -> &'a str`.
type Kirpici = for<'a> fn(&'a str) -> &'a str;

// --- 2b. Sık görülen tuzak: `let` ile tanımlanan closure -------------------------------------
//   let kirp = |s: &str| s.trim();
//       ✗ error: lifetime may not live long enough
// Closure'larda dönüş için elision kuralı İŞLEMEZ: derleyici girdi ile çıktıya iki ayrı süre
// atar ve aralarında bağ kuramaz. Closure doğrudan HRTB bekleyen bir parametreye yazılırsa
// (yukarıdaki gibi) beklenen imzadan öğrenir. Önce değişkene bağlamak istiyorsan, imzayı bir
// "kimlik fonksiyonu" ile dayat:
// SADE ANLATIM: Bir formda "çıktı hangi girdiye bağlı?" hanesini boş bırakırsan görevli
// tahmin yürütür. Fonksiyonlarda bu tahmin kurala bağlıdır ve tutar; `let` ile yazılan
// closure'larda ise görevli girdiye ve çıktıya iki ayrı süre yazar, aralarında bağ kurmaz.
// `imza_dayat` hazır doldurulmuş bir şablondur: closure'ı içinden geçirince derleyici "çıktı
// girdiyle aynı süreye sahip" bilgisini şablondan okur. Kendisi hiçbir iş yapmaz, aldığını
// geri verir; yalnız türü öğretir.
//   Mini örnek:
//     // let kirp = |s: &str| s.trim();        // ✗ derlenmez: girdi ile çıktı bağlanamadı
//     let kirp = imza_dayat(|s| s.trim());     // ✓ bağ şablondan öğrenildi
//     println!("{}", kirp("  a  "));           // a
fn imza_dayat<F: for<'a> Fn(&'a str) -> &'a str>(f: F) -> F {
    f
}

fn hrtb_closure() {
    let cikti = satirlari_donustur("  ada  \n grace ", |s| s.trim());
    println!("   {cikti:?}"); // ["ADA", "GRACE"]

    let ilk_kelime = imza_dayat(|s| s.split_whitespace().next().unwrap_or(""));
    println!("   {:?}", satirlari_donustur("merhaba dünya", ilk_kelime)); // ["MERHABA"]

    let isaretci: Kirpici = str::trim_start;
    println!("   {:?}", satirlari_donustur("  girintili", isaretci)); // ["GIRINTILI"]
}

// --- 2c. Fn dışındaki trait'lerde HRTB: "sahipli çözümleme" ----------------------------------
// Gerçek dünyadan: serde'nin `Deserialize<'de>` trait'i bir lifetime taşır, çünkü bazı türler
// (&'de str) girdi tamponunun İÇİNE bakarak sıfır kopyayla çözülür. Peki girdiyi fonksiyonun
// içinde okuyup SONUCU döndürmek istersek? Tampon fonksiyonla birlikte ölecek; dönen değer
// ona bakamaz. serde bunun için `DeserializeOwned` = `for<'de> Deserialize<'de>` der:
// "HER girdi süresi için çözülebilen", yani girdiden ödünç almayan türler.
// Aynı tasarımın küçük modeli:
// SADE ANLATIM: Bir duvardaki resmi iki yolla "alabilirsin": pencereden bakarsın ya da
// fotoğrafını çekersin. Pencereden bakıyorsan duvar yıkılınca görüntü de gider (`&str`:
// girdinin içine bakar). Fotoğraf ise duvar yıkılsa da sende kalır (`u32`, `String`: kendi
// verisini tutar). Girdi fonksiyonun içinde üretilip fonksiyonla birlikte silinecekse yalnız
// fotoğraf çekebilen türler işe yarar. `T: for<'g> Coz<'g>` bunu söyler: "T, hangi duvardan
// olursa olsun çözülebilir; hiçbirine bağlı kalmaz".
//   Mini örnek:
//     let sayi: Option<u32> = kaynaktan_coz(|| String::from(" 42 "));    // ✓ Some(42)
//     // let o: Option<&str> = kaynaktan_coz(|| String::from("x"));      // ✗ derlenmez:
//     //                        // &str, fonksiyonla birlikte silinen tampona bakmak isterdi
//     let tampon = String::from(" x ");
//     let o: Option<&str> = tampondan_coz(&tampon);   // ✓ tampon bizde, ona bakılabilir
trait Coz<'g>: Sized {
    fn coz(girdi: &'g str) -> Option<Self>;
}

// Ödünç alan çözümleme: sonuç girdinin içine bakar → yalnız O girdi süresi için uygulanır.
impl<'g> Coz<'g> for &'g str {
    fn coz(girdi: &'g str) -> Option<Self> {
        Some(girdi.trim())
    }
}

// Sahipli çözümlemeler: sonuç girdiden bağımsız → HER 'g için uygulanır.
impl<'g> Coz<'g> for u32 {
    fn coz(girdi: &'g str) -> Option<Self> {
        girdi.trim().parse().ok()
    }
}

impl<'g> Coz<'g> for String {
    fn coz(girdi: &'g str) -> Option<Self> {
        Some(girdi.trim().to_string())
    }
}

// Girdi burada, fonksiyonun İÇİNDE üretiliyor. T'nin bu yerel tampondan çözülebilmesi için
// "her süre için Coz" olması gerekir; ödünç alan türler (&str) bu bound'u SAĞLAMAZ.
fn kaynaktan_coz<T>(kaynak: impl FnOnce() -> String) -> Option<T>
where
    T: for<'g> Coz<'g>,
{
    let tampon = kaynak(); // fonksiyon bitince ölecek
    T::coz(&tampon)
}

// Tampon çağırandaysa HRTB gerekmez; sıradan lifetime parametresi yeter ve &str de çalışır.
fn tampondan_coz<'g, T: Coz<'g>>(tampon: &'g str) -> Option<T> {
    T::coz(tampon)
}

fn hrtb_trait() {
    let sayi: Option<u32> = kaynaktan_coz(|| String::from(" 42 "));
    let metin: Option<String> = kaynaktan_coz(|| String::from("  sahipli  "));
    println!("   kaynaktan: {sayi:?}, {metin:?}"); // Some(42), Some("sahipli")

    //   let odunc: Option<&str> = kaynaktan_coz(|| String::from("x"));
    //       ✗ error: implementation of `Coz` is not general enough
    //         note: `Coz<'0>` would have to be implemented for the type `&str`, for any lifetime `'0`...
    //         note: ...but `Coz<'1>` is actually implemented for the type `&'1 str`, for some specific lifetime `'1`
    // Mesajı okumayı öğren: "her süre için" istendi (HRTB), elindeki impl "belirli bir süre
    // için". Bu hata E-kodsuzdur ve neredeyse her zaman bir for<'a> bound'una işaret eder.

    let tampon = String::from("  ödünç  ");
    let odunc: Option<&str> = tampondan_coz(&tampon); // tampon bizde → ödünç alan tür de olur
    println!("   tampondan: {odunc:?}"); // Some("ödünç")
}

// ============================================================================
// 3) GAT — Generic Associated Types: ödünç veren iterator
// ============================================================================
// Sorun: bir dilim üzerinde ÖRTÜŞEN, değiştirilebilir pencereler gezmek istiyoruz
// (std'de `windows` var ama `windows_mut` YOK). Iterator ile yazmayı deneyelim:
//
//   impl<'v, T> Iterator for PencereMut<'v, T> {
//       type Item = &'v mut [T];
//       fn next(&mut self) -> Option<&'v mut [T]> { self.veri.get_mut(..) ... }
//   }       ✗ error: lifetime may not live long enough
//
// Bu hata bir eksiklik değil, bir KORUMADIR. Iterator::next'in imzasında Item, `&mut self`in
// süresine bağlı DEĞİLDİR; yani çağıran, önceki öğeyi elinde tutarken sonrakini isteyebilir
// (`collect()` tam bunu yapar). Pencereler örtüştüğü için bu, aynı elemana iki canlı &mut
// demek olurdu. Iterator trait'i "öğe, iterator'ın kendisinden ödünçtür; bir sonrakini
// istemeden öncekini bırak" sözleşmesini İFADE EDEMEZ.
//
// GAT ile edebiliriz: associated type'a kendi lifetime parametresini veririz ve onu
// `sonraki`nin &mut self ödüncüne bağlarız.
// SADE ANLATIM: Kütüphanede tek bir büyüteç var ve ödünç fişiyle veriliyor. Sıradan
// Iterator'ın fişinde "ne zaman geri gelecek" hanesi yoktur; bu yüzden eskisini geri vermeden
// yenisini isteyebilirsin. Büyüteç tekse bu kabul edilemez. GAT, fişe her seferinde yeniden
// doldurulan boş bir hane ekler: `type Oge<'a>`. Her `sonraki()` çağrısında o haneye "bu
// çağrının ödünç süresi" yazılır. Sonuç: elindeki öğeyi bırakmadan bir sonrakini isteyemezsin.
// Burada bu gerekli, çünkü pencereler üst üste biner; ikisi birden elinde olsaydı aynı
// elemana iki `&mut` olurdu.
//   Mini örnek:
//     let mut veri = [1, 2, 3];
//     let mut it = PencereMut::yeni(&mut veri, 2);
//     while let Some(p) = it.sonraki() { p[1] += p[0]; }   // ✓ p her turun sonunda geri verilir
//     // let a = it.sonraki();
//     // let b = it.sonraki();     // ✗ derlenmez: a geri verilmeden ikincisi istenemez
//     // kullan(a, b);
trait OduncVerenIterator {
    // `where Self: 'a` zorunludur (yazmazsan: "error: missing required bound on `Oge`").
    // Anlamı: Oge<'a>, Self'ten ödünç aldığı için Self en az 'a kadar yaşamalıdır.
    type Oge<'a>
    where
        Self: 'a;

    // Öğe, BU çağrının &mut self ödüncü kadar yaşar. Öğe elindeyken iterator kilitlidir.
    fn sonraki<'a>(&'a mut self) -> Option<Self::Oge<'a>>;
}

struct PencereMut<'v, T> {
    veri: &'v mut [T],
    bas: usize,
    boy: usize,
}

impl<'v, T> PencereMut<'v, T> {
    fn yeni(veri: &'v mut [T], boy: usize) -> Self {
        PencereMut { veri, bas: 0, boy }
    }
}

impl<'v, T> OduncVerenIterator for PencereMut<'v, T> {
    type Oge<'a>
        = &'a mut [T]
    where
        Self: 'a;

    fn sonraki<'a>(&'a mut self) -> Option<Self::Oge<'a>> {
        let son = self.bas.checked_add(self.boy)?;
        let pencere = self.veri.get_mut(self.bas..son)?; // sığmıyorsa None → bitti
        self.bas += 1;
        Some(pencere)
    }
}

// Trait'e bağlı generic kod: her ödünç veren iterator'ı sayar. Öğe her turda düşürülür.
fn say<I: OduncVerenIterator>(mut it: I) -> usize {
    let mut n = 0;
    while it.sonraki().is_some() {
        n += 1;
    }
    n
}

fn gat_odunc_veren_iterator() {
    // Yerinde birikimli toplam: her pencere [önceki, şimdiki]; şimdiki += önceki.
    let mut veri = [1, 2, 3, 4, 5];
    let mut pencereler = PencereMut::yeni(&mut veri, 2);
    // `for` döngüsü Iterator ister; ödünç veren iterator `while let` ile gezilir.
    while let Some(p) = pencereler.sonraki() {
        p[1] += p[0];
    } // p her turun sonunda düşer → bir sonraki `sonraki()` çağrısı serbest
    println!("   birikimli toplam = {veri:?}"); // [1, 3, 6, 10, 15]

    //   let a = pencereler.sonraki();
    //   let b = pencereler.sonraki();     // ✗ E0499: cannot borrow `pencereler` as mutable more than once
    //   kullan(a, b);

    let mut harfler = ['a', 'b', 'c', 'd'];
    println!(
        "   3'lü pencere sayısı = {}",
        say(PencereMut::yeni(&mut harfler, 3))
    ); // 2

    // Sınırlar (dürüst olalım):
    //  • Ödünç veren iterator `collect`, `for`, `map` zincirleri gibi Iterator altyapısını
    //    kullanamaz; std'de henüz karşılığı yoktur. Öğeler örtüşmüyorsa (chunks_mut) sıradan
    //    Iterator yeter — GAT'a yalnız gerçekten "öğe iterator'dan ödünç" ise başvur.
    //  • GAT'lı trait'ler dyn-uyumlu DEĞİLDİR:
    //        fn f(_: &dyn OduncVerenIterator) {}
    //            ✗ E0038: the trait `OduncVerenIterator` is not dyn compatible
}

// ============================================================================
// 4) GAT — tür ailesi: "hangi işaretçi?" kararını parametreye çevirmek
// ============================================================================
// GAT yalnız lifetime almaz; TÜR parametresi de alabilir: `type Isaretci<T>`. Bu, "Rc<_>" ya da
// "Arc<_>" gibi henüz doldurulmamış bir tür kurucusunu (bir tür AİLESİNİ) soyutlamayı sağlar.
//
// Senaryo: sürüm geçmişi tutan bir belge. Sürümler paylaşılır (aynı metin birçok anlık
// görüntüde durur) → sayaçlı işaretçi. Tek thread'li kullanımda Rc yeter ve daha ucuzdur; çok
// thread'li kullanımda Arc şart. İki ayrı Belge yazmak yerine işaretçi ailesini parametre
// yaparız. GAT'sız da yazılabilir (`trait Aile<T> { type Isaretci; }` — T'yi trait parametresine
// taşıyarak), ama o zaman kullanılan her T için ayrı bound gerekir (`A: Aile<String> + Aile<u32>`).
// GAT "her T için" sözünü tek bound'da verir.
// SADE ANLATIM: Taşınırken eşyaları koliye koyarsın. Koli türünü baştan seçersin: karton
// (hafif, ucuz) ya da çelik sandık (ağır, dayanıklı). İçine ne koyacağın sonra belli olur:
// kitap, tabak, oyuncak. `type Isaretci<T>` böyle bir "koli türü"dür; `T` içine konacak
// şeydir ve sonradan doldurulur. `RcAilesi` karton kolidir (tek thread, ucuz), `ArcAilesi`
// çelik sandıktır (thread'ler arasında güvenli). Bu neden var? `Belge` kodunu bir kez yazıp
// koli türünü kullanırken seçebilmek için.
//   Mini örnek:
//     let a: Belge<RcAilesi> = Belge::yeni("taslak");    // içeride Vec<Rc<String>>
//     let b: Belge<ArcAilesi> = Belge::yeni("rapor");    // içeride Vec<Arc<String>>
//     send_ister(&b);          // ✓ Arc başka thread'e verilebilir
//     // send_ister(&a);       // ✗ derlenmez: Rc başka thread'e verilemez
trait IsaretciAilesi {
    type Isaretci<T>: Deref<Target = T> + Clone;
    fn yeni<T>(deger: T) -> Self::Isaretci<T>;
}

struct RcAilesi;
struct ArcAilesi;

impl IsaretciAilesi for RcAilesi {
    type Isaretci<T> = Rc<T>;
    fn yeni<T>(deger: T) -> Rc<T> {
        Rc::new(deger)
    }
}

impl IsaretciAilesi for ArcAilesi {
    type Isaretci<T> = Arc<T>;
    fn yeni<T>(deger: T) -> Arc<T> {
        Arc::new(deger)
    }
}

struct Belge<A: IsaretciAilesi> {
    surumler: Vec<A::Isaretci<String>>,
}

impl<A: IsaretciAilesi> Belge<A> {
    fn yeni(ilk: &str) -> Self {
        Belge {
            surumler: vec![A::yeni(ilk.to_string())],
        }
    }

    fn duzenle(&mut self, yeni_metin: &str) {
        self.surumler.push(A::yeni(yeni_metin.to_string()));
    }

    // Metni kopyalamadan paylaşır: yalnız işaretçi klonlanır (sayaç +1).
    fn guncel(&self) -> A::Isaretci<String> {
        self.surumler
            .last()
            .expect("Belge en az bir sürümle kurulur (bkz. yeni)")
            .clone()
    }

    fn surum_sayisi(&self) -> usize {
        self.surumler.len()
    }
}

// Derleme anı üyelik testi (ileri-send-sync'teki hile): çağrı derleniyorsa T: Send.
fn send_ister<T: Send>(_: &T) {}

fn gat_tur_ailesi() {
    let mut yerel: Belge<RcAilesi> = Belge::yeni("taslak");
    yerel.duzenle("taslak v2");
    println!(
        "   Rc belgesi : {} ({} sürüm)",
        *yerel.guncel(),
        yerel.surum_sayisi()
    );

    let mut paylasilan: Belge<ArcAilesi> = Belge::yeni("rapor");
    paylasilan.duzenle("rapor (son)");
    send_ister(&paylasilan); // ✓ Belge<ArcAilesi>: Send — alan türü Vec<Arc<String>>
    //   send_ister(&yerel);
    //       ✗ E0277: `Rc<String>` cannot be sent between threads safely
    // Aynı kaynak kod, seçilen aileye göre thread-güvenli ya da değil; fark tür parametresinde
    // ve derleyici auto-trait'leri alan türünden kendisi çıkarıyor.
    let guncel = paylasilan.guncel();
    let uzunluk = std::thread::spawn(move || guncel.chars().count())
        .join()
        .unwrap();
    println!("   Arc belgesi: başka thread'de okunan uzunluk = {uzunluk}"); // 11
}

// SADE ANLATIM (aşağıdaki bölüm için): Arabanın gösterge panelinde yanan her lambanın bir
// anlamı vardır; kullanma kılavuzundaki tablo "bu lamba yanarsa şuraya bak" der. Aşağıdaki
// liste lifetime hataları için o tablodur: derleyicinin mesajı solda, ne anlama geldiği ve
// ilk bakılacak yer sağda. Ezberlemen gerekmez; hata aldığında dönüp bakarsın.
// ============================================================================
// SIK KARŞILAŞILAN DERLEYİCİ HATALARI — mesajı okuma rehberi
// ============================================================================
//
//  E0597  "`x` does not live long enough"
//         Bir referans, baktığı değerden uzun yaşamaya çalışıyor. Önce sor: sonucun bu
//         değere bağlı olması GEREKİYOR mu? Gerekmiyorsa imzadaki etiketler fazla sıkıdır
//         (bölüm 1a); gerekiyorsa sahipli değere geç.
//  E0499 / E0502  aynı değere çakışan ödünçler
//         Dönüş türü &mut self'e bağlandıysa değer "kilitli" kalır. Struct'ın kendi 'a'sını
//         döndürebilir misin (Böl. 10 derinleşmesi)? `&'a mut self` yazdıysan sil.
//  E0521  "borrowed data escapes outside of function"
//         Fonksiyona gelen kısa ömürlü bir referans, 'static isteyen bir yere (thread::spawn,
//         global, Box<dyn Trait>) kaçıyor. Sahipliği ver ya da thread::scope kullan.
//  E0309 / E0310  "the parameter type `T` may not live long enough"
//         T'yi 'a (E0309) ya da 'static (E0310) süreli bir yere koyuyorsun; `T: 'a` /
//         `T: 'static` bound'u eksik (bölüm 1c).
//  E0261  "use of undeclared lifetime name `'a`"
//         'a kullanılmış ama bildirilmemiş: `fn f<'a>` / `impl<'a>` ekle.
//  (kodsuz) "lifetime may not live long enough"
//         Gövde, imzanın söz verdiğinden fazlasını istiyor. Mesajın altındaki `help`
//         çoğunlukla eksik ilişkiyi söyler ("consider adding the following bound: `'a: 'b`").
//         Notta "invariant" geçiyorsa sorun &mut / Cell ardındaki bir lifetime'dır (bölüm 1d).
//  (kodsuz) "implementation of `X` is not general enough"
//         Bir yerde for<'a> bound'u var ve verdiğin tür/closure yalnız BELİRLİ bir süre için
//         çalışıyor (bölüm 2b, 2c).
//  (kodsuz) "missing required bound on `Oge`"
//         GAT bildiriminde `where Self: 'a` eksik (bölüm 3).
//
// SADE ANLATIM (aşağıdaki bölüm için): Yukarıda dört araç gördün: ayrı etiketler, `'a: 'b`
// ve `T: 'a` şartları, `for<'a>` sözü ve GAT. Aşağıdaki bölüm bunların dilin içinde nasıl
// işlediğini anlatır: fonksiyonların "her süre için" sözünü zaten kendiliğinden verdiğini,
// `for<...>` yazımının yalnız süreler için geçerli olduğunu ve bütün bu etiketlerin derleme
// bitince silindiğini. İlk okuyuşta atlayabilirsin; yukarıdaki dört bölümü anlamış olman yeter.
// ============================================================================
// DERLEYİCİ / DİL PERSPEKTİFİ
// ============================================================================
//
// • ERKEN ve GEÇ bağlanan lifetime'lar: `fn f<'a>(x: &'a str)` içindeki 'a, fonksiyon bir
//   DEĞER olarak kullanıldığında bile serbest kalır ("late-bound") — bu yüzden `f`, tek bir
//   değerken `for<'a> Fn(&'a str)` bound'unu sağlar ve her çağrıda başka süreyle çağrılabilir.
//   HRTB, fonksiyon öğelerinin zaten sahip olduğu bu özelliği closure'lar ve trait bound'ları
//   için yazılabilir kılar.
// (Sade: "late-bound" = süre fonksiyon tanımlanırken değil, her çağrıda yeniden seçilir demek.)
//
// • `for<'a>` yalnız LIFETIME üzerinde niceler; "her T türü için" diyen `for<T>` kararlı
//   Rust'ta yoktur. Bu yüzden generic bir closure parametresi (her T için çalışan closure)
//   yazamazsın; onun yerine generic metotlu bir trait tanımlanır.
//
// • GAT (1.65'ten beri kararlı): associated type'a generic parametre ekler. Trait çözümleyici
//   `<I as OduncVerenIterator>::Oge<'a>`yı her 'a için ayrı bir projeksiyon olarak ele alır.
//   `where Self: 'a` şartının elle yazdırılması, ileride varsayılan hâle getirilebilmesi için
//   konmuş geçici bir zorunluluktur (hata mesajındaki issue #87479).
// (Sade: "projeksiyon" = "I türünün bu trait'teki Oge'si hangi tür?" sorusunun yazılışı demek.)
//
// • Hepsi DERLEME ANINDA: lifetime'lar, for<'a> ve GAT'ın süre parametreleri kod üretiminden
//   önce silinir (erasure). Lifetime denetimi derleme öncesinde kalkar; çalışma zamanında ek
//   bir kontrol veya maliyet eklenmez. Kazandığımız şey hız değil, yanlış kullanımın derlenmemesidir.
// (Sade: "erasure" = lifetime'lar yalnız denetim içindir; çalışan programda izleri kalmaz demek.)
//
// ============================================================================
// ALIŞTIRMALAR
// ============================================================================
// 1) `fn en_uzun_satir<F>(metin: &str, anahtar: F) -> Option<&str> where F: Fn(&str) -> usize`
//    yaz. F'nin bound'unu açık `for<'a>` ile yeniden yaz; iki imzanın aynı olduğunu, ikisini
//    de aynı closure ile çağırarak doğrula. Sonra imzayı bilerek
//    `fn f<'a, F: Fn(&'a str) -> usize>(metin: &'a str, anahtar: F) -> Option<&'a str>` yap:
//    gövde hâlâ derleniyor mu? Neden? (İpucu: satırlar `metin`den geliyor, yerelden değil; bu
//    yüzden çağıranın seçtiği 'a yetiyor. `metin: &str` bırakırsan E0621 alırsın — nedenini açıkla.)
//
// 2) `OduncVerenIterator`a varsayılan gövdeli `fn her_biri(mut self, mut f: impl for<'a> FnMut(Self::Oge<'a>))
//    where Self: Sized` metodunu ekle ve birikimli toplam örneğini onunla ÇAĞIRMAYI DENE. Trait
//    derlenir ama çağrı E0597 verir; notu oku: "due to a current limitation of the type system,
//    this implies a `'static` lifetime". Neden? (İpucu: `for<'a>` HER 'a'yı kapsar, `where Self: 'a`
//    ise her 'a için `Self: 'a` ister → `Self: 'static`; yerel diziden ödünç alan PencereMut bunu
//    sağlamaz. GAT + HRTB birleşiminin bilinen sınırıdır.) Sonra aynı metodu trait yerine
//    `impl<'v, T> PencereMut<'v, T>` içine `f: impl FnMut(&mut [T])` ile yaz.
//    Beklenen: [1, 3, 6, 10, 15].
//
// 3) `IsaretciAilesi`ne `KutuAilesi` (Box) eklemeyi dene. Hangi bound engel olur, neden?
//    (İpucu: `Clone` — Box<T> yalnız T: Clone ise klonlanır; Rc/Arc her T için.) Bound'u
//    kaldırırsan `Belge::guncel` hangi imzaya dönmek zorunda kalır?
//
// Bağlantılar: alt tipleme ve variance → ileri-variance; Drop ile lifetime etkileşimi →
// ileri-dropck; for<'a> bound'larının async'te (Send future'lar) çıkardığı dertler →
// ileri-eszamanlilik-async.

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn eslesmeler_kelimeden_uzun_yasar() {
        let metin = String::from("ab ab");
        let sonuc = {
            let kelime = String::from("ab");
            Arayici {
                metin: &metin,
                kelime: &kelime,
            }
            .eslesmeler()
        };
        assert_eq!(sonuc, ["ab", "ab"]);
    }

    #[test]
    fn outlives_bound_ile_secim() {
        let varsayilan = String::from("v");
        assert_eq!(bos_ise_varsayilan("", &varsayilan), "v");
        assert_eq!(bos_ise_varsayilan("dolu", &varsayilan), "dolu");
    }

    #[test]
    fn hrtb_closure_her_sureyle_cagrilir() {
        let sonuc = satirlari_donustur("a b\nc d", |s| &s[..1]);
        assert_eq!(sonuc, ["A", "C"]);
    }

    #[test]
    fn sahipli_turler_her_sure_icin_cozulur() {
        assert_eq!(kaynaktan_coz::<u32>(|| String::from("7")), Some(7));
        assert_eq!(kaynaktan_coz::<u32>(|| String::from("yedi")), None);
        let tampon = String::from(" x ");
        assert_eq!(tampondan_coz::<&str>(&tampon), Some("x"));
    }

    #[test]
    fn pencere_mut_ortusen_pencereleri_sirayla_verir() {
        let mut veri = [1, 1, 1, 1];
        let mut it = PencereMut::yeni(&mut veri, 2);
        while let Some(p) = it.sonraki() {
            p[1] += p[0];
        }
        assert_eq!(veri, [1, 2, 3, 4]);
    }

    #[test]
    fn pencere_dilimden_buyukse_hic_oge_yok() {
        let mut veri = [1, 2];
        assert_eq!(say(PencereMut::yeni(&mut veri, 3)), 0);
        assert_eq!(say(PencereMut::yeni(&mut veri, 2)), 1);
        assert_eq!(say(PencereMut::yeni(&mut veri, usize::MAX)), 0); // taşma yok: checked_add
    }

    #[test]
    fn belge_her_iki_aileyle_calisir() {
        let mut rc: Belge<RcAilesi> = Belge::yeni("a");
        rc.duzenle("b");
        let onceki = rc.guncel();
        rc.duzenle("c");
        assert_eq!((onceki.as_str(), rc.guncel().as_str()), ("b", "c"));
        assert_eq!(Rc::strong_count(&onceki), 2); // belge + elimizdeki kopya; metin kopyalanmadı

        let arc: Belge<ArcAilesi> = Belge::yeni("x");
        send_ister(&arc);
        assert_eq!(arc.surum_sayisi(), 1);
    }
}
