// Rust Book — Bölüm 19: Patterns and Matching (desenler ve eşleştirme)
// Çalıştır: cargo run -p patterns
//
// Desen (pattern) = bir değerin YAPISINA göre eşleştirme + içindeki parçaları bağlama
// (destructuring). match'ten ibaret değil; dilin birçok yerinde aynı dilbilgisi çalışır.
//
// Doğrulanmış (rustc ile): `let` REDDEDİLEMEZ (irrefutable) desen ister; Some(x) gibi
// reddedilebilir desen koyunca → E0005, derleyici let-else önerir. Ayrıntı: en altta.

fn main() {
    desen_yerleri();
    reddedilebilirlik();
    desen_sozdizimi();
}

// ============================================================================
// 1. DESENLERİN KULLANILDIĞI YERLER
// ============================================================================

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
    let cift = (2, -2);
    match cift {
        (x, y) if x + y == 0 => println!("guard: toplamı sıfır"),
        (x, _) if x % 2 == 0 => println!("guard: ilk çift"),
        _ => println!("guard: diğer"),
    }

    // (g) `@` bağlama: hem ARALIĞI test et HEM değeri bir değişkene BAĞLA
    let id = 5;
    match id {
        n @ 1..=5 => println!("@ bağlama: {n} küçük aralıkta"),
        n @ 6..=10 => println!("@ bağlama: {n} büyük aralıkta"),
        _ => println!("@ bağlama: aralık dışı"),
    }

    // Shadowing tuzağı: match kolundaki isim DIŞ değişkeni DEĞİL, yeni bir bağlamadır.
    let x = Some(10);
    let y = 20;
    match x {
        Some(y) => println!("shadowing: içteki y={y} (dıştaki 20 DEĞİL)"), // y burada 10
        None => {}
    }
    println!("shadowing: dıştaki y hâlâ {y}");
}

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
