// Açıklama: learn/common-concepts/ACIKLAMA.md
// Rust Book — Bölüm 3: Yaygın Programlama Kavramları (Common Programming Concepts)
// Değişkenler ve değişebilirlik, veri türleri, fonksiyonlar, kontrol akışı.
// Çalıştır: cargo run -p common-concepts
//
// Stil: teknik terim İngilizce kalır, ilk geçtiğinde kısa Türkçe açıklaması verilir;
// önemli kararlarda "Neden?" + o tasarım olmasaydı çıkacak "Sorun" + gerektiğinde
// "rustc:" notu (hangi AŞAMADA, hangi kaynak dosyada) yazılır. Sondaki "DERLEYİCİ
// PERSPEKTİFİ" boru hattını ve mut'un KÖK NEDENİNİ (Aliasing XOR Mutability) anlatır.

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Her bölümde önce SADE ANLATIM kutusu, sonra teknik ayrıntı var.
// Benzetmelerin hepsi aynı yerden, OKULDAN gelir: defter, kalem, çanta, sıra, yoklama.
//
// Bu bölümün tek cümlelik özeti: Bir program; isim verilmiş değerlerden (değişken), o değerlerin
// cinsinden (tür), iş yapan küçük tariflerden (fonksiyon) ve "şu olursa bunu yap, şunu tekrarla"
// diyen yol tabelalarından (kontrol akışı) oluşur.
//
// MİNİ SÖZLÜK (bu dosyada geçen kelimeler):
//   değişken / binding    → bir değere taktığın isim. Defterdeki "yaş: 7" satırı gibi.
//   immutable             → değiştirilemez. Tükenmez kalemle yazılmış gibidir; silinmez.
//   mutable, mut          → değiştirilebilir. Kurşun kalemle yazılmış gibidir; silip yenilersin.
//   const (sabit)         → program derlenirken belli olan, hiç değişmeyen değer.
//   shadowing (gölgeleme) → aynı ismi yeni bir `let` ile yeniden kullanmak; eskisi arkada kalır.
//   tür (type)            → değerin cinsi: sayı mı, harf mi, doğru/yanlış mı.
//   scalar / compound     → tek parça değer (bir sayı, bir harf) / birkaç değerin paketi.
//   integer / float       → tam sayı (7) / küsuratlı sayı (3.15).
//   i32, u32, u8, f64     → sayının kaç bit yer tuttuğu. i: eksi olabilir, u: olamaz, f: küsuratlı.
//   usize                 → "kaç tane", "kaçıncı" sorularının cevabında kullanılan tam sayı türü.
//   bool                  → yalnız iki değer alır: true (doğru) ya da false (yanlış).
//   char                  → tek bir harf ya da işaret: 'a', 'ş', '7'.
//   &str                  → çift tırnak içindeki yazı: "merhaba".
//   tuple (demet)         → farklı cinsten birkaç değeri yan yana tutan paket: (1, 2.0, 'x').
//   array (dizi)          → aynı cinsten, sayısı baştan belli değerlerin sırası: [10, 20, 30].
//   destructuring         → paketi açıp içindekilere ayrı ayrı isim vermek.
//   indeks                → sıradaki yer numarası. Saymaya 0'dan başlanır: ilk eleman dizi[0].
//   overflow (taşma)      → sayının, türünün tutabileceği en büyük değeri aşması.
//   panik (panic)         → programın "burada bir yanlışlık var" deyip kendini durdurması.
//   debug / release       → iki derleme biçimi. debug: denerken, bol denetimli. release: hızlı.
//   fonksiyon (fn)        → isim verilmiş iş tarifi. Girdi alır, iş yapar, sonuç verebilir.
//   expression (ifade)    → bir DEĞER veren kod parçası: 2 + 3.
//   statement (deyim)     → bir iş yapan ama değer vermeyen satır: let x = 5;
//   unit, ()              → "ortada bir değer yok" demenin Rust'taki yazılışı.
//   kontrol akışı         → programın hangi satıra gideceğine karar veren yapılar: if, loop, for.
//   range (aralık)        → 1..4 gibi bir sayı aralığı: 1, 2, 3 (son sayı dâhil değil).
//   iterator              → elemanları "sıradaki, sıradaki" diye tek tek veren şey.
//   derleyici (rustc)     → yazdığın kodu çalışan programa çeviren ve hataları önceden bulan araç.
//   derleme zamanı        → kodun programa çevrildiği an. Çalışma anı: programın çalıştığı an.
//   E0384 gibi kodlar     → derleyicinin hata numaraları; `rustc --explain E0384` açıklar.
//   referans (&), borrow  → verinin kendisi değil, yerini gösteren ok; veriye "bakma izni" (ödünç).
//   alias                 → aynı veriyi gösteren ikinci bir ok.
//   borrow checker        → derleyicinin, okların güvenli kullanıldığını denetleyen bölümü.
//   AST, HIR, MIR         → derleyicinin kodu adım adım sadeleştirirken tuttuğu üç ara biçim.
//   stack                 → küçük ve boyu belli değerlerin durduğu hızlı bellek bölgesi.
//   thread                → programın aynı anda yürüyen kollarından biri.
//   data race             → iki kolun aynı veriyi, biri yazarken öteki okuyarak bozması.
//   use-after-free        → silinmiş bir veriyi hâlâ varmış gibi kullanma hatası.
//   generic, trait        → sonraki bölümlerin konusu; burada yalnız adları geçer.

fn main() {
    degiskenler_ve_mut();
    golgeleme();
    veri_turleri();
    kontrol_akisi();

    println!("toplam(2, 3) = {}", toplam(2, 3));
    println!("bes() = {}", bes());
}

// 1. Değişkenler ve değişebilirlik (variables & mutability)
// binding = bir ismi bir değere bağlamak (`let x = 5;`); "değişken" yerine daha hassas
// terim budur. Varsayılan immutable'dır (değiştirilemez); değiştirmek için `mut` eklersin.
// SADE ANLATIM: Defterine tükenmez kalemle yazdığını silemezsin; kurşun kalemle yazdığını silip
// yenisini yazabilirsin. Rust'ta `let x = 5;` tükenmez kalemdir: x hep 5 kalır. `let mut y = 5;`
// kurşun kalemdir: sonradan `y = 6;` diyebilirsin.
// Neden? Her şey değiştirilebilir olsaydı bir değeri kimin, nerede değiştirdiğini bulmak
// zorlaşırdı; `mut` "burası değişecek" işaretidir.
//   Mini örnek:
//     let x = 5;
//     x = 6;            // ✗ derlenmez: x tükenmez kalemle yazıldı (mut yok)
//     let mut y = 5;
//     y = 6;            // ✓ y kurşun kalemle yazıldı; artık 6
fn degiskenler_ve_mut() {
    let x = 5;
    println!("x = {x}"); // immutable; `x = 6;` yazsan derlenmez (E0384)

    let mut y = 5;
    println!("ilk y = {y}");
    y = 6; // mut olduğu için serbest
    println!("yeni y = {y}");

    // Neden varsayılan immutable? KISA cevap: yanlışlıkla değiştirmeyi engeller,
    // niyeti `mut` ile açık kılar, derleyici sabit değerlere güvenip optimize eder.
    // KÖK cevap ise tek bir kurala dayanır — "Aliasing XOR Mutability" — ve bunun
    // tam anlatımı + derleyicinin nasıl zorladığı dosyanın sonundaki DERLEYİCİ
    // PERSPEKTİFİ bölümünde. (Çünkü mut'u gerçekten anlamak borrow checker'ı anlamak.)
    // (Sade: HIR ve MIR, derleyicinin kodu çevirirken kullandığı ara biçimlerdir; ayrıntı altta.)
    // rustc: immutable bir binding'e atama hatası (E0384), MIR üzerinde borrow checker
    //        (rustc_borrowck, borrowck_errors.rs) tarafından üretilir (6. adım). Kanıtı:
    //        aynı fonksiyonda bir tür hatası (E0308) varsa E0384 hiç raporlanmaz, çünkü
    //        derleme borrowck aşamasına ulaşmaz. Ödünç ihlalleri de burada yakalanır.

    // SADE ANLATIM: Sınıf duvarındaki "Bu sınıfta 30 sıra var" levhasını düşün. Levha okul
    // açılmadan önce basılmıştır, dersin ortasında değişmez. `const` böyle bir levhadır: değeri
    // program daha derlenirken bellidir ve hiç değişmez; türünü de sen yazarsın.
    //   Mini örnek:
    //     const SIRA_SAYISI: usize = 3;       // tür yazıldı, değer baştan belli
    //     let siralar = [0; SIRA_SAYISI];     // ✓ dizi uzunluğu olarak kullanılabilir
    //     let n = 3;
    //     let baska = [0; n];                 // ✗ derlenmez: `let` ile yazılan n böyle sabit değil
    // const = derleme zamanı sabiti. let'ten farkı: HER ZAMAN değişmez (mut olamaz),
    // türü ZORUNLU yazılır ve değeri DERLEME anında hesaplanır (çalışma anında değil).
    const MAX_PUAN: u32 = 100_000;
    println!("MAX_PUAN = {MAX_PUAN}");
    // Neden const ayrı? Sorun: `let` çalışma anında hesaplanır ve yalnız o an immutable'dır;
    // dizi uzunluğu gibi DERLEME ZAMANINDA bilinmesi gereken sabit için const gerekir.
    // rustc: const ifadeleri derleme anında "const evaluation" ile hesaplanır
    //        (rustc_const_eval crate'i; aynı motor dizi uzunluğu [u8; N] gibi yerlerde çalışır).
}

// 2. Gölgeleme (shadowing)
// Aynı ismi `let` ile yeniden bağlamak eskisini "gölgeler": YENİ bir binding oluşur.
// mut'tan farkı: (a) yeni değerin TÜRÜ farklı olabilir, (b) sonuç yine immutable kalır.
// SADE ANLATIM: Defterde "cevap" başlıklı bir satır yazdın. Sonra alt satıra yine "cevap"
// başlığıyla başka bir şey yazdın. Artık "cevap" denince alttaki, yeni satıra bakılır; üstteki
// silinmedi ama ona bakan kalmadı. Rust'ta aynı ismi ikinci kez `let` ile yazmak budur. Yeni satır
// eskisinden farklı cinsten olabilir: üstte yazı, altta sayı.
// Neden böyle? Bir değeri adım adım dönüştürürken her adıma yeni bir isim uydurmak gerekmez.
//   Mini örnek:
//     let cevap = "42";                          // yazı
//     let cevap: i32 = cevap.parse().unwrap();   // ✓ aynı isim, yeni satır: artık sayı
//     let mut puan = "beş";
//     puan = 5;             // ✗ derlenmez: mut değeri değiştirir, değerin cinsini değiştiremez
fn golgeleme() {
    let bosluk = "   "; // tür: &str
    let bosluk = bosluk.len(); // tür: usize — shadowing sayesinde tür değişti
    println!("boşluk sayısı = {bosluk}");

    // Neden shadowing? Sorun: `mut` ile türü değiştiremezsin — `let mut s = "   "; s = s.len();`
    // "beklenen &str, bulunan usize" (E0308) verir; her ara adım için yeni isim uydurmak gerekirdi.
    // Çözüm: aynı ismi yeniden bağlayıp türü değiştirir, değeri immutable tutarsın.
    // (Sade: "isim çözümlemesi" = derleyicinin, yazdığın ismin hangi değeri kastettiğini bulması.)
    // rustc: her `let`, HIR/MIR seviyesinde AYRI bir yerel değişken (local) üretir; isim
    //        çözümlemesi (name resolution) sonraki kullanımı en son bağlanana yönlendirir.
    //        Yani eski binding kaybolmaz, sadece o isimden erişilemez olur.
}

// 3. Veri türleri (data types)
// SADE ANLATIM: Okul çantanda her şeyin kendi yeri vardır: kalem kutusuna kalem, suluğa su konur.
// Suluğa kalem koymazsın. Rust'ta her değerin de bir TÜRÜ (cinsi) vardır ve derleyici yanlış kaba
// yanlış şey koymana izin vermez. Dört temel cins: tam sayı, küsuratlı sayı, doğru/yanlış, harf.
// Neden böyle? Cinsi belli olmayan bir değerle ne yapılabileceği de belli olmaz: "elma" yazısı ile
// 3 sayısını toplamak ne demek? Derleyici bu karışıklığı program çalışmadan yakalar.
//   Mini örnek:
//     let yas: i32 = 7;          // ✓ tam sayı
//     let boy: f64 = 1.25;       // ✓ küsuratlı sayı
//     let hazir: bool = true;    // ✓ doğru/yanlış
//     let harf: char = 'k';      // ✓ tek harf
//     let hata: i32 = "yedi";    // ✗ derlenmez: sayı kabına yazı konmaz
fn veri_turleri() {
    // Scalar türler: integer, float, bool, char
    let tam: i32 = -42; // işaretli 32-bit (i); işaretsiz için u32
    let ondalik: f64 = 3.15; // varsayılan float f64 (f32 de var)
    let dogru: bool = true;
    let harf: char = 'ş'; // char = tek bir Unicode scalar değeri (4 byte), 'tek tırnak'
    println!("{tam} {ondalik} {dogru} {harf}");

    // SADE ANLATIM: Parmaklarınla en çok 10'a kadar sayarsın; 11 için parmak kalmaz. Sayı türleri
    // de böyledir: `u8` en çok 255'i tutar. 255'e 1 eklemeye "taşma" denir. Rust, programı
    // denerken (debug) taşma olursa programı durdurur ki hatayı göresin.
    //   Mini örnek:
    //     let puan: u8 = 255;                      // u8'in tutabildiği en büyük sayı
    //     println!("{:?}", puan.checked_add(1));   // None → "sığmadı" diye haber verir
    //     println!("{}", puan.wrapping_add(1));    // 0 → bilerek başa sardık
    // Integer overflow (taşma): debug derlemesinde PANİK eder, release'de sarar (wrap).
    // Neden panik? Sorun: C'de taşma sessizce sarar ve fark edilmeyen hatalara yol açar;
    // Rust debug'da bunu yakalatır. Kasıtlı sarma gerekiyorsa wrapping_add gibi metotlar var.
    // (Sade: aşağıdaki not, bu denetimin derleyicinin kendi kaynak kodunda nerede durduğunu söyler.
    //  Assert = "şu doğru değilse dur" komutu.)
    // rustc: taşma denetimi MIR KURULUMUNDA eklenir. `check_overflow` bayrağı
    //        compiler/rustc_mir_build/src/builder/mod.rs:872 civarında
    //        (`check_overflow |= tcx.sess.overflow_checks();`) belirlenir; açıksa toplama/
    //        çıkarma bir CheckedBinaryOp + Assert'e dönüşür, kapalıysa (release) sade işleme.

    // SADE ANLATIM: Beslenme çantana farklı şeyler koyarsın: bir elma, bir sandviç, bir su. `tuple`
    // böyle karışık bir pakettir. Boya kalemi kutusunda ise hep aynı cins şey durur ve göz sayısı
    // bellidir; `array` (dizi) de budur. İkisinin de boyu sonradan değişmez.
    //   Mini örnek:
    //     let canta = (1, 2.5, 'e');     // tuple: üç farklı cins yan yana
    //     println!("{}", canta.0);       // ✓ 1 → ilk göz (sayma 0'dan başlar)
    //     let kalemler = [3, 5, 8];      // array: üçü de tam sayı
    //     println!("{}", kalemler[2]);   // ✓ 8 → üçüncü göz
    // Compound (bileşik) türler: tuple ve array
    let demet: (i32, f64, char) = (1, 2.0, 'x');
    let (a, b, c) = demet; // destructuring (parçalara ayırma)
    println!("{a} {b} {c}, .1 = {}", demet.1); // .indeks ile de erişilir

    let dizi = [10, 20, 30]; // [tür; uzunluk] — SABİT uzunluk, stack'te durur
    println!("ilk = {}", dizi[0]);
    // SADE ANLATIM: Sınıfta 3 sıra varsa "6. sıraya otur" denen öğrenci oturacak yer bulamaz.
    // Rust, dizide olmayan bir yere uzanmak istediğinde programı durdurur (panikler).
    // Neden böyle? Durdurmasaydı program dizinin yanındaki, başka işe ayrılmış belleği okurdu;
    // yanlış veriyle sessizce devam etmek, durmaktan çok daha tehlikelidir.
    //   Mini örnek:
    //     let siralar = [10, 20, 30];
    //     println!("{:?}", siralar.get(1));   // ✓ Some(20) → orada bir sıra var
    //     println!("{:?}", siralar.get(5));   // None → öyle bir sıra yok; program durmaz
    //     siralar[i]                          // i çalışırken 5 çıkarsa program panikler
    // dizi[i] çalışma anında PANİK: "index out of bounds" (i program çalışırken belli olur).
    // dizi[5] gibi SABİT indekse derleyici `deny(unconditional_panic)` ile izin vermez:
    // "this operation will panic at runtime" diyerek DERLEME ZAMANINDA reddeder.
    // Neden sınır denetlenir? Sorun: C'de dizi sınırını aşmak bellekte başka bir yeri
    // okur/yazar (klasik güvenlik açığı); Rust sınırı denetleyip programı güvenle durdurur.
    // rustc: indeksleme, MIR kurulumunda bir sınır denetimine (bounds check) çevrilir.
    //        AssertKind::BoundsCheck tanımı compiler/rustc_middle/src/mir/syntax.rs:1041;
    //        indeksleme öncesi bu Assert'i ekleyen yer
    //        compiler/rustc_mir_build/src/builder/expr/as_place.rs:752. Optimizer indeksin
    //        güvenli olduğunu kanıtlayabilirse (ör. sabit) denetimi kaldırır — güvenlik yine durur.
}

// 4. Fonksiyonlar — ifade (expression) vs deyim (statement)
// statement = bir iş yapar ama DEĞER döndürmez (`let x = 5;`). expression = bir değere
// çözülür (`5 + 6`, bir blok `{...}`, bir fonksiyon çağrısı). Rust ifade-temelli bir dildir.
// SADE ANLATIM: Öğretmen iki türlü cümle kurar. "Kapıyı kapat" bir iştir; karşılığında bir cevap
// beklenmez. "2 + 3 kaç eder?" ise bir sorudur; karşılığında bir DEĞER gelir: 5. Rust'ta iş
// cümlelerine statement, değer veren parçalara expression denir. Fonksiyonun son satırı noktalı
// virgülsüz bir expression ise fonksiyonun verdiği cevap odur.
// Neden böyle? `return` yazmadan "cevabım bu" diyebilirsin; ama sona `;` koyarsan cevap kaybolur.
//   Mini örnek:
//     fn iki_kati(x: i32) -> i32 {
//         x * 2         // ✓ noktalı virgül yok → cevap bu
//     }
//     fn bozuk(x: i32) -> i32 {
//         x * 2;        // ✗ derlenmez: `;` cevabı attı, geriye sayı kalmadı
//     }

fn toplam(x: i32, y: i32) -> i32 {
    x + y // son ifade; NOKTALI VİRGÜL YOK → dönen değer budur
}

// SADE ANLATIM: Süslü parantez { } içi küçük bir karalama kâğıdıdır: içinde hesap yaparsın, son
// satırdaki sonuç dışarı çıkar; karalamada kullandığın isimler dışarıdan görünmez.
//   Mini örnek:
//     let sonuc = {
//         let a = 2;    // yalnız bu parantezin içinde yaşar
//         a + 3         // ✓ noktalı virgül yok → blok 5 değerini verir
//     };
fn bes() -> i32 {
    let hesap = {
        let temel = 2;
        temel + 3 // blok bir ifadedir, bu değere çözülür → 5
    };
    hesap
    // Neden noktalı virgül yok? Sorun: `x + y;` yazarsan bu bir STATEMENT olur, `()` döndürür
    // ve "beklenen i32, bulunan ()" (E0308) hatası alırsın. Yani ; olup olmaması dönüşü değiştirir.
    // (Sade: AST = derleyicinin, kodu okuyunca çıkardığı ağaç biçimli taslak.)
    // rustc: AST'te bir blok, bir dizi statement + İSTEĞE BAĞLI son bir expression'dır. Son
    //        expression varsa bloğun değeri odur; yoksa blok `()` (unit) üretir. `;` eklemek,
    //        expression'ı statement'a çevirip o son değeri atar.
}

// 5. Kontrol akışı (control flow)
// SADE ANLATIM: Program satırları yukarıdan aşağıya sırayla okur. Bazen bir yol ayrımı gerekir:
// "zil çaldıysa bahçeye çık, çalmadıysa yerinde otur". Bu `if`tir. Bazen de aynı işi tekrarlamak
// gerekir: öğretmenin yoklama listesindeki her ismi sırayla okuması `for`dur; "tahtada yazı
// kaldıkça sil" `while`dır; "ben dur diyene kadar devam et" ise `loop`tur.
// Neden böyle? Yol ayrımı ve tekrar olmasaydı her program hep aynı şeyi, yalnız bir kez yapardı.
//   Mini örnek:
//     let zil_caldi = true;
//     let yer = if zil_caldi { "bahçe" } else { "sınıf" };   // ✓ if bir değer verir
//     for sira in 1..4 {
//         println!("{sira}. öğrenci burada");                // 1, 2 ve 3 için birer kez çalışır
//     }
//     let x = if zil_caldi { 1 } else { "yok" };   // ✗ derlenmez: iki dal aynı cinsten olmalı
fn kontrol_akisi() {
    let sayi = 7;

    // if bir İFADEDİR: değer üretir, let'e atanabilir. Dalların türü AYNI olmalı.
    let etiket = if sayi % 2 == 0 { "çift" } else { "tek" };
    println!("{sayi} → {etiket}");
    // Neden dallar aynı türde olmalı? Sorun: derleyici değişkenin türünü derleme zamanında
    // tek ve kesin bilmek zorunda (bellekte ne kadar yer, hangi işlemler geçerli); bir dal
    // i32, öbürü &str dönseydi tür belirsiz kalırdı.

    // SADE ANLATIM: `loop`, "dur" denene kadar dönen bir tekrardır. `break` o "dur" sözüdür;
    // dururken döngüye bir sonuç da verdirebilir: `break 30;` yazarsan döngünün değeri 30 olur.
    //   Mini örnek:
    //     let mut tur = 0;
    //     let son = loop {
    //         tur += 1;
    //         if tur == 3 { break tur * 10; }   // ✓ 3. turda dur, 30 değerini ver
    //     };
    let mut sayac = 0;
    let sonuc = loop {
        sayac += 1;
        if sayac == 5 {
            break sayac * 2; // break bir DEĞER döndürebilir
        }
    };
    println!("loop sonucu = {sonuc}");

    // while
    let mut geri = 3;
    while geri > 0 {
        print!("{geri} ");
        geri -= 1;
    }
    println!("kalk!");

    // for + range (aralık) — en çok kullanılan, en güvenli döngü
    for i in 1..4 {
        print!("{i} "); // 1, 2, 3 (4 hariç)
    }
    println!();

    let dizi = [10, 20, 30];
    for eleman in dizi {
        print!("{eleman} "); // doğrudan elemanlar üzerinde gezer
    }
    println!();
    // Neden for tercih edilir? Sorun: while + indeks ile döngüde `i < len` koşulunu elle
    // yazmak, sınırı yanlış yazıp dizi dışına taşmaya (panik) ya da bir elemanı atlamaya
    // açıktır; for koleksiyonun kendi üzerinde gezerek bu hataları baştan eler.
    // (Sade: "sözdizimsel şeker" = aynı işi kısa yazdıran kolaylık; derleyici onu uzun hâle açar.)
    // rustc: `for x in it {...}` sözdizimsel şekerdir (syntactic sugar). AST→HIR indirgemesi
    //        (rustc_ast_lowering) bunu kabaca şuna açar:
    //            let mut it = IntoIterator::into_iter(it);
    //            loop { match it.next() { Some(x) => {...}, None => break } }
    //        Yani for aslında bir loop + match; sınır denetimi derdi kalmaz çünkü biten
    //        iterator None döndürür.
}

// SADE ANLATIM (aşağıdaki bölüm için): Ödevin, öğretmene ulaşmadan önce sırayla birkaç kişiden
// geçtiğini düşün: biri yazım hatalarına bakar, biri cümlelerin anlamlı olup olmadığına, biri de
// "aynı deftere iki kişi aynı anda mı yazıyor" diye bakar. Derleyici (rustc) de kodunu böyle dokuz
// duraktan geçirir; A kısmı bu durakları sayar. B kısmı tek bir kuralı anlatır: bir deftere aynı
// anda ya birçok kişi BAKAR ya da tek kişi YAZAR. `mut` bu kuralın anahtarıdır; kuralın kendisi
// ownership bölümünde örneklerle işlenir. İlk okuyuşta bu bölümü atlayabilirsin; yukarıdaki beş
// bölümü anlamış olman yeter.
// ============================================================================
// DERLEYİCİ PERSPEKTİFİ — kökü gerçekten anlamak için
// ============================================================================
//
// A) rustc BORU HATTI: kaynağın makine koduna yolculuğu, ve her garantinin NEREDE
//    zorlandığı. (Crate adları compiler/ altında; senin guide/rust ağacında mevcut.)
//
//    1. rustc_lexer         : metin → token (kelime parçaları).
//    2. rustc_parse → AST   : token → sözdizim ağacı (rustc_ast). `let mut y` burada bir
//                             binding + "mutable" bayrağı olarak ayrışır.
//    (Sade: desugaring = kısa yazımı, derleyicinin anladığı uzun hâle açmak.)
//    3. rustc_ast_lowering  : AST → HIR. "desugaring": for→loop+match, `?`→match, while→loop...
//    4. rustc_hir_typeck +  : tür denetimi, tür çıkarımı (inference), trait çözümü.
//       rustc_hir_analysis    Tür uyuşmazlığı (E0308) burada; E0384 ise 6. adımda üretilir.
//    (Sade: kontrol-akış grafiği = "bu adımdan sonra hangi adımlara gidilebilir" haritası.)
//    5. rustc_mir_build     : HIR → MIR. MIR = temel blokların (basic block) KONTROL-AKIŞ
//                             GRAFİĞİ (CFG). DİZİ SINIR denetimi ve TAŞMA denetimi tam burada,
//                             Assert düğümleri olarak EKLENİR (yukarıdaki kaynak satırları).
//    6. rustc_borrowck      : MIR üzerinde ÖDÜNÇ DENETLEYİCİ (borrow checker). Aliasing XOR
//                             Mutability kuralını ve ömürleri (NLL = non-lexical lifetimes) burada zorlar;
//                             immutable binding'e atama (E0384) de bu adımda üretilir.
//    7. rustc_mir_transform : MIR optimizasyonları (gereksiz bounds check kaldırma vb.).
//    (Sade: generic = her cinsle çalışan kalıp; derleyici, kullandığın her cins için kalıbın ayrı
//     bir kopyasını üretir. Bu işe monomorphization denir.)
//    8. rustc_monomorphize  : generic'leri somut türlerle "kopyalayıp" özelleştirir.
//    9. rustc_codegen_llvm  : MIR → LLVM IR → makine kodu.
//
// B) mut'un KÖK NEDENİ: "Aliasing XOR Mutability" (paylaşım YA DA değiştirme, ikisi aynı anda DEĞİL)
//
//    Kök sorun: bellek hataları ve veri yarışları (data race) hep ŞUNDAN doğar — aynı veriye
//    bir işaretçiden YAZARKEN, başka bir işaretçinin (alias) de aynı veriyi görmesi. Örnekler:
//      • Bir Vec'e eleman eklerken içine tuttuğun referans: Vec yeniden tahsis (realloc) edince
//        o referans serbest bırakılmış belleği gösterir → use-after-free.
//      • Üzerinde döngü dönerken koleksiyonu değiştirmek → iterator invalidation.
//      • İki thread aynı veriyi biri yazarken biri okursa → data race.
//
//    Rust'ın tek kuralı bunların HEPSİNİ baştan keser: herhangi bir anda ya
//      • N tane paylaşımlı `&T`  (yalnız okuma, istediğin kadar), YA DA
//      • tam 1 tane `&mut T`     (okuma+yazma, tek sahip)
//    olabilir — ikisi ASLA aynı anda olamaz. Yazma yetkisi (mutation) tehlikeli olan şey
//    olduğundan, Rust onu VARSAYILAN DIŞI yapar: binding'ler immutable, referanslar `&`
//    (paylaşımlı); yazmak istiyorsan `mut` / `&mut` ile AÇIKÇA ve İZLENEBİLİR biçimde istersin.
//    "Varsayılan immutable" işte bu kuralın yüzeye yansımasıdır.
//
//    Nasıl zorlanıyor: borrow checker (rustc_borrowck) MIR üzerinde çalışır; her `&mut` ödüncün,
//    aynı yerin (place) başka hiçbir ödüncüyle zaman/bölge olarak çakışmadığını kanıtlar.
//
//    Sonuçları (tradeoff):
//      + Derleme zamanında, SIFIR çalışma maliyetiyle: data race yok, use-after-free yok.
//    (Sade: noalias = "bu veriye bakan başka ok yok" bilgisi; derleyici buna güvenip hızlanır.)
//      + `&mut T`'nin tekliği LLVM'e `noalias` der → daha agresif optimizasyon
//        (yalnızca argüman konumunda ve `T: Unpin` için; dönüşte verilmez).
//      − Bazen "borrow checker'la güreşirsin": C'de serbest olan (iki mutable alias) reddedilir.
//        Bu bir kusur değil, kuralın bedeli; karşılığında yukarıdaki garantileri alırsın.
//
//    Yani "neden mut?" sorusunun dibi şu: Rust, güvenliği bir KURALA indirgemiş (Aliasing XOR
//    Mutability) ve mut'u o kuralın açık anahtarı yapmış. mut'u anlamak = bu kuralı anlamak.
