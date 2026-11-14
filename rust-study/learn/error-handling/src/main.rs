// Rust Book — Bölüm 9: Error Handling (hata yönetimi)
// Çalıştır: cargo run -p error-handling
//
// İki tür hata: KURTARILAMAZ (panic!) ve KURTARILABİLİR (Result<T, E>). Rust'ta exception
// (try/catch) YOKTUR; hatalar türle (Result) taşınır, böylece ele almayı derleyici hatırlatır.
//
// Doğrulanmış (rustc ile): kullanılmayan Result → "unused `Result` that must be used" uyarısı;
// `?` operatörü Result/Option dönmeyen fn'de → E0277. Ayrıntı: DERLEYİCİ PERSPEKTİFİ (en altta).

use std::fmt;
use std::num::ParseIntError;

fn main() {
    panik_vs_result();

    println!("ikiye_bol(10, 2) = {:?}", ikiye_bol(10, 2));
    println!("ikiye_bol(10, 0) = {:?}", ikiye_bol(10, 0));

    match propagasyon() {
        Ok(n) => println!("propagasyon OK: {n}"),
        Err(e) => println!("propagasyon HATA: {e}"),
    }

    from_donusumu();
}

// 1. panic! — kurtarılamaz hata
fn panik_vs_result() {
    // panic! o thread'i sonlandırır. VARSAYILAN davranış "unwind": stack geri sarılır,
    // yoldaki değerlerin Drop'ları çalışır (kaynaklar temizlenir), sonra thread biter.
    //   panic!("durum kötü");           // kasıtlı panik
    //   let v = vec![1, 2, 3]; v[99];   // sınır dışı erişim de panic eder
    // RUST_BACKTRACE=1 ortam değişkeni ile çağrı yığını (backtrace) basılır.
    // Cargo.toml'de  [profile.release] panic = "abort"  dersen unwind yerine anında ABORT
    // olur: Drop çalışmaz, ikili dosya küçülür. Ne zaman panic? Program devam edemeyecekse
    // (bozulmuş değişmez/invariant, imkânsız durum) — yani "hata ele alınamaz" dediğinde.
    println!("panic bölümü: yalnızca yorumda (programı bilerek çökertmiyoruz)");
}

// 2. Result<T, E> = enum Result { Ok(T), Err(E) } — kurtarılabilir hata
fn ikiye_bol(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err(String::from("sıfıra bölme"))
    } else {
        Ok(a / b)
    }
    // unwrap()/expect("...") Result'ı açar ama Err ise PANİK eder. expect mesaj verdirir,
    // bu yüzden hata ayıklamada unwrap'tan iyidir. İkisi de "burada Err olamaz" dediğin
    // yerler için; aksi halde match/`?` ile ele al.
    //
    // Not (doğrulanmış): Result #[must_use]'tır. `ikiye_bol(10, 2);` deyip sonucu yok sayarsan
    // "unused `Result` that must be used" UYARISI alırsın. Bilerek yok saymak için: `let _ = ...;`
}

// 3. ? operatörü — hata yayma (propagation)
// x.parse()? : parse Ok ise içteki değeri verir; Err ise FONKSİYONDAN hemen o Err ile döner.
fn metni_sayiya(x: &str) -> Result<i32, ParseIntError> {
    let n: i32 = x.parse()?; // Err olursa buradan erken Err dönülür
    Ok(n * 2)
}
fn propagasyon() -> Result<i32, ParseIntError> {
    let a = metni_sayiya("21")?; // 42
    // let _b = metni_sayiya("xx")?; // açık olsaydı: Err → propagasyon da hemen Err dönerdi
    Ok(a)
    // ? ancak Result/Option (veya FromResidual uygulayan bir tür) DÖNEN fonksiyonda kullanılabilir.
    // `fn main()` varsayılan olarak () döndüğünden, main içinde doğrudan `?` kullanmak E0277 verir
    // (DOĞRULANDI). Çözüm: ya match ile ele al ya da `fn main() -> Result<(), E>` yap. ? ayrıca
    // Option üzerinde de çalışır: None → fonksiyondan hemen None döner.
}

// 4. Farklı hata türlerini TEK türde toplamak + ?'in From dönüşümü
#[derive(Debug)]
enum UygulamaHatasi {
    Ayristirma(ParseIntError),
    Mantik(String),
}
impl fmt::Display for UygulamaHatasi {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UygulamaHatasi::Ayristirma(e) => write!(f, "ayrıştırma hatası: {e}"),
            UygulamaHatasi::Mantik(m) => write!(f, "mantık hatası: {m}"),
        }
    }
}
// From<ParseIntError> tanımlayınca, ? operatörü ParseIntError'ı otomatik UygulamaHatasi'na çevirir.
impl From<ParseIntError> for UygulamaHatasi {
    fn from(e: ParseIntError) -> Self {
        UygulamaHatasi::Ayristirma(e)
    }
}
fn from_donusumu() {
    fn hesapla(x: &str) -> Result<i32, UygulamaHatasi> {
        let n: i32 = x.parse()?; // ParseIntError → UygulamaHatasi (From ile OTOMATİK), sonra erken dönüş
        if n < 0 {
            return Err(UygulamaHatasi::Mantik(String::from("negatif olamaz")));
        }
        Ok(n)
    }
    println!("hesapla(\"7\")  = {:?}", hesapla("7")); // Ok(7)
    println!("hesapla(\"xx\") = {:?}", hesapla("xx")); // Err(Ayristirma(..)) — ? From ile dönüştürdü
    println!("hesapla(\"-3\") = {:?}", hesapla("-3")); // Err(Mantik(..))
    // Display üzerinden okunaklı mesaj:
    if let Err(e) = hesapla("xx") {
        println!("Display: {e}");
    }
    // Neden From? Sorun: alt fonksiyonlar FARKLI hata türleri döndürür (ParseIntError, IO hatası...).
    // ? bunları, fonksiyonun ilan ettiği TEK hata türüne From ile köprüler; böylece her katmanda
    // elle dönüştürme yazmazsın ama tür güvenliği korunur.
}

// ============================================================================
// DERLEYİCİ PERSPEKTİFİ — Result ve ? nasıl çalışıyor?
// ============================================================================
//
// • Result<T, E> ve Option<T> SIRADAN enum'lardır; özel çalışma-anı mekanizması yoktur.
//   Rust'ta exception YOK — "hata olabilir" bilgisi DÖNÜŞ TÜRÜNE kodlanır. Böylece çağıran,
//   Err ihtimalini match/`?`/unwrap ile ele almak zorunda kalır (sessizce atlanamaz).
//
// • #[must_use]: Result (ve Option gibi bazı türler) bu öznitelikle işaretlidir. Değerini
//   kullanmadan bırakırsan derleyici uyarır ("unused `Result` that must be used"). Böylece
//   "hatayı kontrol etmeyi unutma" derleyici tarafından hatırlatılır. (Bilerek yok say: `let _ =`.)
//
// • ? operatörünün indirgenmesi (lowering): AST → HIR aşamasında (rustc_ast_lowering/src/expr.rs,
//   fn lower_expr_try, DesugaringKind::QuestionMark) kabaca şuna açılır:
//       match Try::branch(<expr>) {
//           ControlFlow::Continue(v)      => v,                               // Ok/Some → değer
//           ControlFlow::Break(residual)  => return FromResidual::from_residual(residual), // Err/None → erken dönüş
//       }
//   Result için Break yolu, hatayı From ile dönüştürür (yukarıdaki From<ParseIntError> bu yüzden iş görür).
//   ? bu nedenle fonksiyonun FromResidual uygulayan bir tür döndürmesini ister; döndürmezse E0277.
//
// • panic davranışı: "unwind" (varsayılan) stack'i geri sarıp Drop'ları çalıştırır; "abort"
//   (panic = "abort") hemen sonlandırır, Drop çalışmaz. Seçim derleme profilinde yapılır.
