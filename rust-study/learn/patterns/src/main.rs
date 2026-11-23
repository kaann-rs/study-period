// Rust Book — Bölüm 19: Patterns and Matching (desenler ve eşleştirme)
// Çalıştır: cargo run -p patterns
//
// Desen (pattern) = bir değerin YAPISINA göre eşleştirme + içindeki parçaları bağlama
// (destructuring). match'ten ibaret değil; dilin birçok yerinde aynı dilbilgisi çalışır.
//
// Doğrulanmış (rustc ile): `let` REDDEDİLEMEZ (irrefutable) desen ister; Some(x) gibi
// reddedilebilir desen koyunca → E0005, derleyici let-else önerir. Ayrıntı: en altta.

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Bu dosyadaki her bölümün üstünde "SADE ANLATIM" diye bir kutu var. Önce onu oku, sonra
// altındaki teknik açıklamaya geç. Teknik kısım zor gelirse sorun değil: sade kutu ana fikri verir.
// Bu crate'teki benzetmeler ŞEKİL EŞLEŞTİRME OYUNCAĞI çevresinde geçer: kapağında yıldız, kare,
// daire biçiminde delikler olan kutu. derinlesme.rs de aynı oyuncakla sürer.
//
// Bu bölümün tek cümlelik özeti: Desen, bir değere "şu şekilde misin?" diye sormanın ve
// uyuyorsa içindeki parçalara ad vermenin yoludur.
//
// MİNİ SÖZLÜK (bu dosyada geçen kelimeler):
//   desen (pattern)          → değerin şeklini tarif eden kalıp. Oyuncaktaki delik.
//   eşleştirme (matching)    → değeri desene tutup "uyuyor mu?" diye bakmak.
//   destructuring (yapı çözme) → bir paketi açıp içindeki parçalara ayrı ayrı ad vermek.
//   bağlama (binding)        → eşleşen parçaya bir değişken adı vermek.
//   match ve kol (arm)       → değeri sırayla desenlere tutan yapı / o desenlerin her biri.
//   if let / while let       → tek desene bakan kısa match / desen uydukça dönen döngü.
//   let-else                 → "uyarsa devam et, uymazsa şuradan çık" yazımı.
//   reddedilemez (irrefutable) → her değere uyan desen. Eşleşmeme ihtimali yoktur.
//   reddedilebilir (refutable) → bazı değerlere uymayabilen desen (`Some(x)`, None'a uymaz).
//   literal                  → koda doğrudan yazılmış değer: 1, 'a', "selam".
//   `|`, `..=`               → "şu ya da bu" / "şundan şuna kadar" (aralık).
//   `_` ve `..`              → "bu parçaya bakmıyorum" / "geri kalanların hiçbirine bakmıyorum".
//   guard                    → desenden sonra gelen ek soru: `if x > 0`.
//   `@`                      → "hem şu desene uyup uymadığına bak, hem de değere ad ver".
//   shadowing (gölgeleme)    → aynı adın yeni bir değişkene verilmesi; eskisi arkada kalır.
//   tuple                    → birkaç değerin parantez içinde yan yana durduğu demet: (1, 2).
//   varyant                  → bir enum'un alabileceği biçimlerden her biri.
//   Option, Some, None       → "ya bir değer var ya da yok" kutusu ve iki hâli.
//   bütünlük denetimi (exhaustiveness) → derleyicinin "açıkta kalan durum var mı?" kontrolü.
//   erişilemez kol (unreachable pattern) → üstteki kollar her şeyi aldığı için hiç çalışmayan kol.

mod derinlesme;
fn main() {
    desen_yerleri();
    reddedilebilirlik();
    desen_sozdizimi();
    derinlesme::calistir();
}

// ============================================================================
// 1. DESENLERİN KULLANILDIĞI YERLER
// ============================================================================

// SADE ANLATIM: Şekil eşleştirme oyuncağında parçayı deliğe tutarsın; şekli uyarsa geçer. Rust'ta
// desen o deliktir, değer de parçadır. Üstelik parça geçerken açılır ve içindekilere ad
// verirsin: `let (a, b) = (1, 2);` ikiliyi açıp a ile b'yi doldurur. Bu oyun yalnız `match`te
// oynanmaz; `let`, `if let`, `while let`, `for` ve fonksiyon parametreleri de birer deliktir.
// Neden böyle? Desen olmasaydı her parçayı tek tek "0. eleman, 1. eleman" diye elle çıkarır,
// şeklin uyup uymadığını da ayrıca sorardın; ikisini ayrı yazınca biri kolayca unutulur.
//   Mini örnek:
//     let (a, b) = (1, 2);                   // ✓ ikiliyi aç: a = 1, b = 2
//     if let Some(r) = Some("mavi") {        // ✓ "içi dolu mu?" Doluysa içindekine r de
//         println!("{r}");
//     }
//     // let (c, d) = (1, 2, 3);             // ✗ derlenmez: iki gözlü desen, üç parçalı değer
// (e) Fonksiyon PARAMETRESİ de bir desendir: tuple'ı doğrudan parçala.
fn konum(&(x, y): &(i32, i32)) -> String {
    format!("({x}, {y})")
}

fn desen_yerleri() {
    // (a) let ile DESTRUCTURING (reddedilemez desen): tuple'ı parçala
    let (a, b, c) = (1, 2, 3);
    println!("let destructuring: {a} {b} {c}");

    // (b) match: her kol bir desen (Böl. 6)
    let sayi = 7;
    let parite = match sayi % 2 {
        0 => "çift",
        _ => "tek",
    };
    println!("match: {sayi} -> {parite}");

    // (c) if let / else if let / else: tek dala bakan esnek match
    let renk: Option<&str> = Some("mavi");
    if let Some(r) = renk {
        println!("if let: renk {r}");
    } else {
        println!("if let: renk yok");
    }

    // (d) while let: desen eşleştiği sürece dön
    let mut yigin = vec![1, 2, 3];
    print!("while let (pop): ");
    while let Some(ust) = yigin.pop() {
        print!("{ust} ");
    }
    println!();

    // for döngüsü de desen alır: enumerate'in verdiği tuple'ı parçala
    print!("for destructuring: ");
    for (i, harf) in ['a', 'b', 'c'].iter().enumerate() {
        print!("{i}:{harf} ");
    }
    println!();

    // (e) fonksiyon parametresi
    println!("fn param destructuring: {}", konum(&(3, 4)));
}

// ============================================================================
// 2. REDDEDİLEBİLİR (refutable) vs REDDEDİLEMEZ (irrefutable)
// ============================================================================
// SADE ANLATIM: Bazı delikler her parçayı geçirir (kocaman, yuvarlak bir ağız); bunlara
// reddedilemez desen denir. Bazıları yalnız tek bir şekli geçirir; bunlar reddedilebilir
// desenlerdir. Düz `let`, parçanın KESİN geçeceği delik ister, çünkü "geçmezse ne olacak?"
// sorusuna verecek cevabı yoktur. `if let` ve `let ... else` ise bu cevabı taşır: uymazsa bloğu
// atla ya da buradan çık.
// Neden böyle? `let Some(x) = None` derlenseydi, program çalışırken x'in içine konacak bir değer
// bulunamazdı.
//   Mini örnek:
//     let veri: Option<i32> = Some(5);
//     let kopya = veri;                               // ✓ her değere uyar (reddedilemez)
//     // let Some(n) = veri;                          // ✗ derlenmez: ya None gelirse?
//     let Some(n) = veri else { return };             // ✓ "uymazsa çık" yolu var; n = 5
//     if let Some(m) = kopya { println!("{m} {n}"); } // ✓ uymazsa blok atlanır
fn reddedilebilirlik() {
    // İrrefutable = HER değere uyar (ör. `let x`, `let (a,b)`). Eşleşmeme ihtimali yok.
    // Refutable  = bazı değerlere uymayabilir (ör. `Some(x)` → None'a uymaz).
    //
    // `let`, `for`, fn parametreleri yalnız İRREFUTABLE desen kabul eder (eşleşmeme olasılığı
    // olamaz). if let / while let REFUTABLE kabul eder (eşleşmezse dalı atlar).
    //
    //   let Some(x) = bir_option;   // ✗ E0005: refutable pattern in local binding
    //
    // Doğru yol 1 — let...else: eşleşmezse `else` KONTROLÜ DEVRALIR (return/break/panic ile çıkar),
    // eşleşirse bağlanan değişken dışarıda kalır (if let'in aksine iç içe girinti yok).
    let veri: Option<i32> = Some(99);
    let Some(x) = veri else {
        println!("veri yok, çık");
        return;
    };
    println!("let-else ile bağlandı: x = {x}");
}

// ============================================================================
// 3. DESEN SÖZ DİZİMİ — tüm araçlar
// ============================================================================
// SADE ANLATIM: Oyuncağın kapağında çeşit çeşit delik olabilir. "Tam şu değer" (literal), "şu ya
// da bu" (`|`), "şundan şuna kadar" (`..=`), "gerisine bakmıyorum" (`_` ve `..`), "şekli uydu ama
// bir sorum daha var" (guard: `if ...`) ve "geçen parçaya ad da ver" (`@`). Parça delikleri
// yukarıdan aşağı sırayla dener ve ilk uyduğundan geçer; bu yüzden dar delikler üstte, geniş
// delikler altta durur.
// Neden böyle? Derleyici bir de bütünlük denetimi yapar: hiçbir deliğe uymayan bir parça
// kalıyorsa kodu derlemez. Böylece "bu durumu unutmuşum" hatası program çalışmadan yakalanır.
//   Mini örnek:
//     let n = 42;
//     let etiket = match n {
//         1 | 2 => "bir ya da iki",                             // şu ya da bu
//         3..=9 => "üç ile dokuz arası",                        // aralık
//         k @ 10..=99 if k % 2 == 0 => "iki basamaklı çift",    // ✓ ad ver + ek soru: 42 buraya
//         _ => "başka",                                         // bu kol olmasa ✗ derlenmez
//     };
enum Mesaj {
    Cik,
    Tasi { x: i32, y: i32 },
    Renk(u8, u8, u8),
    Yaz(String),
}

struct Nokta {
    x: i32,
    y: i32,
}

fn desen_sozdizimi() {
    // (a) literal + (b) `|` (veya) + (c) aralık `..=`
    for n in [1, 5, 7, 42] {
        let etiket = match n {
            1 | 2 => "bir ya da iki",
            3..=9 => "üç ile dokuz arası",
            _ => "başka",
        };
        println!("match {n}: {etiket}");
    }

    // (d) struct destructuring + alan kısayolu + `..` (gerisini yok say)
    let p = Nokta { x: 0, y: 7 };
    let Nokta { x, y } = p; // x ve y değişkenlerine bağla (shorthand)
    println!("struct destructuring: x={x} y={y}");
    let p2 = Nokta { x: 10, y: 20 };
    match p2 {
        Nokta { x: 0, y } => println!("y ekseninde, y={y}"),
        Nokta { x, .. } => println!("x={x}, geri kalanı umursama"),
    }

    // (e) enum (iç içe) destructuring
    for m in [
        Mesaj::Cik,
        Mesaj::Tasi { x: 3, y: 4 },
        Mesaj::Renk(255, 0, 0),
        Mesaj::Yaz(String::from("selam")),
    ] {
        match m {
            Mesaj::Cik => println!("enum: çık"),
            Mesaj::Tasi { x, y } => println!("enum: taşı ({x},{y})"),
            Mesaj::Renk(r, _, _) => println!("enum: renk, kırmızı={r} (yeşil/maviyi _ ile yok say)"),
            Mesaj::Yaz(s) => println!("enum: yaz '{s}'"),
        }
    }

    // Üç kanalı da BAĞLAYAN tam destructuring (yukarıdaki arm `_` ile ikisini atlıyordu):
    if let Mesaj::Renk(r, g, b) = Mesaj::Renk(10, 20, 30) {
        println!("tam renk destructuring: r={r} g={g} b={b}");
    }

    // (f) match GUARD: desenin yanında ek bir koşul (if ...)
    // (Sade: guard, şekli uyan parçaya sorulan ek sorudur: "ikili olduğun tamam, peki toplamın
    // sıfır mı?" Cevap hayırsa parça bu delikten geçmez, sıradaki deliği dener.)
    let cift = (2, -2);
    match cift {
        (x, y) if x + y == 0 => println!("guard: toplamı sıfır"),
        (x, _) if x % 2 == 0 => println!("guard: ilk çift"),
        _ => println!("guard: diğer"),
    }

    // (g) `@` bağlama: hem ARALIĞI test et HEM değeri bir değişkene BAĞLA
    // (Sade: `n @ 1..=5` = "1 ile 5 arasında mısın diye bak; öyleyse bu değere n adını ver".
    // `@` olmasaydı aralığa bakabilir ama değerin kendisini kol içinde kullanamazdın.)
    let id = 5;
    match id {
        n @ 1..=5 => println!("@ bağlama: {n} küçük aralıkta"),
        n @ 6..=10 => println!("@ bağlama: {n} büyük aralıkta"),
        _ => println!("@ bağlama: aralık dışı"),
    }

    // Shadowing tuzağı: match kolundaki isim DIŞ değişkeni DEĞİL, yeni bir bağlamadır.
    // (Sade: desendeki ad yeni bir değişkendir; dışarıda aynı adlı bir değişken olsa bile ona
    // bakılmaz. Aşağıda `Some(y)` "içindeki 20 mi?" diye sormaz, içindekine y adını verir.)
    let x = Some(10);
    let y = 20;
    match x {
        Some(y) => println!("shadowing: içteki y={y} (dıştaki 20 DEĞİL)"), // y burada 10
        None => {}
    }
    println!("shadowing: dıştaki y hâlâ {y}");
}

// SADE ANLATIM (aşağıdaki bölüm için): Aşağıdaki bölüm derleyicinin desenleri nasıl denetlediğini
// anlatır: bir desenin her değere uyup uymadığını, kolların her durumu kapsayıp kapsamadığını ve
// hiç çalışmayacak bir kol olup olmadığını daha derlerken hesaplar. Akılda kalacak tek ayrıntı:
// derleyici guard'ın (ek sorunun) içine bakmaz, bu yüzden guard'lı kollar "her durumu kapsadım"
// demeye yetmez. İlk okuyuşta bu bölümü atlayabilirsin.
// ============================================================================
// DERLEYİCİ PERSPEKTİFİ — desenler nasıl denetlenir?
// ============================================================================
//
// • REDDEDİLEBİLİRLİK derleme anında belirlenir: bir desen değerin TÜM olası biçimlerine uyuyor
//   mu? Uyuyorsa irrefutable (let kabul eder), uymayan hâl varsa refutable. `let` bir dal
//   atlayamaz (else yoktur), bu yüzden refutable deseni reddeder → E0005 "refutable pattern in
//   local binding", derleyici let-else önerir. Kontrol rustc_mir_build/src/thir/pattern/
//   check_match.rs (check_binding_is_irrefutable, ~60). Sorun: izin verilseydi, `let Some(x) =
//   None` çalışma anında "x neye bağlanacak?" belirsizliği/çökmesi olurdu — tür sistemi bunu
//   baştan eler.
//
// • EXHAUSTIVENESS (match tüm halleri kapsıyor mu, Böl. 6) ve USEFULNESS (bir kol asla erişilmez
//   mi) aynı analiz ailesindendir (rustc_pattern_analysis). Erişilemez kol → "unreachable
//   pattern" uyarısı. `_` ve `..` bu analizi tatmin etmek için "geri kalan her şey" der.
//
// • GUARD'lar (if koşulu) desen analizinin DIŞINDADIR: derleyici bir guard'ın ne zaman doğru
//   olacağını bilemez, bu yüzden guard'lı kolu "her zaman eşleşir" saymaz — exhaustiveness'i
//   guard'a GÜVENEREK tamamlayamazsın (yine de bir `_` kolu gerekebilir).
//
// • `@` tek bir kolda iki işi birleştirir: aralık/alt-desen TESTİ + eşleşen değeri BİR
//   DEĞİŞKENE bağlama. Ayrı yazınca ya testi ya bağlamayı kaybederdin; @ ikisini verir.
//
// • Desenler tamamen DERLEME ZAMANI bir araçtır: eşleştirme/bağlama, en sonunda sıradan
//   dallanma + alan okuma makine koduna iner; "pattern" diye bir runtime nesnesi yoktur.
