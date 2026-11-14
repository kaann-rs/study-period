// Rust Book — Bölüm 3: Yaygın Programlama Kavramları (Common Programming Concepts)
// Değişkenler ve değişebilirlik, veri türleri, fonksiyonlar, kontrol akışı.
// Çalıştır: cargo run -p common-concepts
//
// Stil: teknik terim İngilizce kalır, ilk geçtiği yerde kısa Türkçe açıklaması
// verilir; önemli kararlarda "Neden?" + o tasarım olmasaydı çıkacak "Sorun" +
// gerektiğinde "rustc:" notu (derleyici bunu hangi AŞAMADA, hangi kaynak dosyada
// uyguluyor) yazılır. Dosyanın SONUNDA "DERLEYİCİ PERSPEKTİFİ" bölümü, rustc'nin
// tüm boru hattını ve mut'un KÖK NEDENİNİ (Aliasing XOR Mutability) anlatır.

fn main() {
    degiskenler_ve_mut();
    golgeleme();
    veri_turleri();
    kontrol_akisi();

    println!("toplam(2, 3) = {}", toplam(2, 3));
    println!("bes() = {}", bes());
}

// 1. Değişkenler ve değişebilirlik (variables & mutability)
// binding = bir ismi bir değere bağlamak. `let x = 5;` x ismini 5 değerine BAĞLAR;
// bu yüzden "değişken" yerine daha hassas terim "binding" (bağlama). Varsayılan
// olarak immutable'dır (değiştirilemez); değiştirmek için `mut` eklersin.
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
    // rustc: immutable bir binding'e atama hatası (E0384) tür/HIR denetiminde yakalanır;
    //        ödünç (borrow) ihlalleri ise MIR üzerinde rustc_borrowck'ta yakalanır.

    // const = derleme zamanı sabiti. let'ten farkı: HER ZAMAN değişmez (mut olamaz),
    // türü ZORUNLU yazılır ve değeri DERLEME anında hesaplanır (çalışma anında değil).
    const MAX_PUAN: u32 = 100_000;
    println!("MAX_PUAN = {MAX_PUAN}");
    // Neden const ayrı bir şey? Sorun: sıradan bir `let` çalışma anında hesaplanır ve
    // yalnızca o an immutable'dır; oysa dizi uzunluğu gibi DERLEME ZAMANINDA bilinmesi
    // gereken, asla değişmeyecek bir sabit gerekir. const tam bunu garanti eder.
    // rustc: const ifadeleri derleme anında "const evaluation" ile hesaplanır
    //        (rustc_const_eval crate'i; aynı motor dizi uzunluğu [u8; N] gibi yerlerde çalışır).
}

// 2. Gölgeleme (shadowing)
// Aynı ismi `let` ile yeniden bağlamak eskisini "gölgeler": YENİ bir binding oluşur.
// mut'tan farkı: (a) yeni değerin TÜRÜ farklı olabilir, (b) sonuç yine immutable kalır.
fn golgeleme() {
    let bosluk = "   ";          // tür: &str
    let bosluk = bosluk.len();   // tür: usize — shadowing sayesinde tür değişti
    println!("boşluk sayısı = {bosluk}");

    // Neden shadowing? Sorun: `mut` ile türü değiştiremezsin — `let mut s = "   "; s = s.len();`
    // "beklenen &str, bulunan usize" (E0308) verir. O zaman bir dönüşümün ara adımları için
    // her seferinde yeni isim (bosluk_str, bosluk_sayi...) uydurman gerekirdi. Çözüm: aynı
    // ismi yeniden bağlayıp hem türü değiştirir hem değeri immutable tutarsın.
    // rustc: her `let`, HIR/MIR seviyesinde AYRI bir yerel değişken (local) üretir; isim
    //        çözümlemesi (name resolution) sonraki kullanımı en son bağlanana yönlendirir.
    //        Yani eski binding kaybolmaz, sadece o isimden erişilemez olur.
}

// 3. Veri türleri (data types)
fn veri_turleri() {
    // Scalar türler: integer, float, bool, char
    let tam: i32 = -42;      // işaretli 32-bit (i); işaretsiz için u32
    let ondalik: f64 = 3.14; // varsayılan float f64 (f32 de var)
    let dogru: bool = true;
    let harf: char = 'ş';    // char = tek bir Unicode scalar değeri (4 byte), 'tek tırnak'
    println!("{tam} {ondalik} {dogru} {harf}");

    // Integer overflow (taşma): debug derlemesinde PANİK eder, release'de sarar (wrap).
    // Neden panik? Sorun: C'de taşma sessizce sarar ve fark edilmeyen hatalara yol açar;
    // Rust debug'da bunu yakalatır. Kasıtlı sarma gerekiyorsa wrapping_add gibi metotlar var.
    // rustc: taşma denetimi MIR KURULUMUNDA eklenir. `check_overflow` bayrağı
    //        compiler/rustc_mir_build/src/builder/mod.rs:872 civarında
    //        (`check_overflow |= tcx.sess.overflow_checks();`) belirlenir; açıksa toplama/
    //        çıkarma bir CheckedBinaryOp + Assert'e dönüşür, kapalıysa (release) sade işleme.

    // Compound (bileşik) türler: tuple ve array
    let demet: (i32, f64, char) = (1, 2.0, 'x');
    let (a, b, c) = demet;              // destructuring (parçalara ayırma)
    println!("{a} {b} {c}, .1 = {}", demet.1); // .indeks ile de erişilir

    let dizi = [10, 20, 30];            // [tür; uzunluk] — SABİT uzunluk, stack'te durur
    println!("ilk = {}", dizi[0]);
    // dizi[5] yazsan çalışma anında PANİK: "index out of bounds".
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
fn toplam(x: i32, y: i32) -> i32 {
    x + y // son ifade; NOKTALI VİRGÜL YOK → dönen değer budur
}

fn bes() -> i32 {
    let hesap = {
        let temel = 2;
        temel + 3 // blok bir ifadedir, bu değere çözülür → 5
    };
    hesap
    // Neden noktalı virgül yok? Sorun: `x + y;` yazarsan bu bir STATEMENT olur, `()` döndürür
    // ve "beklenen i32, bulunan ()" (E0308) hatası alırsın. Yani ; olup olmaması dönüşü değiştirir.
    // rustc: AST'te bir blok, bir dizi statement + İSTEĞE BAĞLI son bir expression'dır. Son
    //        expression varsa bloğun değeri odur; yoksa blok `()` (unit) üretir. `;` eklemek,
    //        expression'ı statement'a çevirip o son değeri atar.
}

// 5. Kontrol akışı (control flow)
fn kontrol_akisi() {
    let sayi = 7;

    // if bir İFADEDİR: değer üretir, let'e atanabilir. Dalların türü AYNI olmalı.
    let etiket = if sayi % 2 == 0 { "çift" } else { "tek" };
    println!("{sayi} → {etiket}");
    // Neden dallar aynı türde olmalı? Sorun: derleyici değişkenin türünü derleme zamanında
    // tek ve kesin bilmek zorunda (bellekte ne kadar yer, hangi işlemler geçerli); bir dal
    // i32, öbürü &str dönseydi tür belirsiz kalırdı.

    // loop + break ile değer döndürme
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
    // rustc: `for x in it {...}` sözdizimsel şekerdir (syntactic sugar). AST→HIR indirgemesi
    //        (rustc_ast_lowering) bunu kabaca şuna açar:
    //            let mut it = IntoIterator::into_iter(it);
    //            loop { match it.next() { Some(x) => {...}, None => break } }
    //        Yani for aslında bir loop + match; sınır denetimi derdi kalmaz çünkü biten
    //        iterator None döndürür.
}

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
//    3. rustc_ast_lowering  : AST → HIR. "desugaring": for→loop+match, `?`→match, while→loop...
//    4. rustc_hir_typeck +  : tür denetimi, tür çıkarımı (inference), trait çözümü.
//       rustc_hir_analysis    Immutable binding'e atama (E0384), tür uyuşmazlığı (E0308) burada.
//    5. rustc_mir_build     : HIR → MIR. MIR = temel blokların (basic block) KONTROL-AKIŞ
//                             GRAFİĞİ (CFG). DİZİ SINIR denetimi ve TAŞMA denetimi tam burada,
//                             Assert düğümleri olarak EKLENİR (yukarıdaki kaynak satırları).
//    6. rustc_borrowck      : MIR üzerinde ÖDÜNÇ DENETLEYİCİ (borrow checker). Aliasing XOR
//                             Mutability kuralını ve ömürleri (NLL = non-lexical lifetimes) burada zorlar.
//    7. rustc_mir_transform : MIR optimizasyonları (gereksiz bounds check kaldırma vb.).
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
//      + `&mut T`'nin tekliği LLVM'e `noalias` der → daha agresif optimizasyon.
//      − Bazen "borrow checker'la güreşirsin": C'de serbest olan (iki mutable alias) reddedilir.
//        Bu bir kusur değil, kuralın bedeli; karşılığında yukarıdaki garantileri alırsın.
//
//    Yani "neden mut?" sorusunun dibi şu: Rust, güvenliği bir KURALA indirgemiş (Aliasing XOR
//    Mutability) ve mut'u o kuralın açık anahtarı yapmış. mut'u anlamak = bu kuralı anlamak.
