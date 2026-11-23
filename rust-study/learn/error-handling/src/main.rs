// Açıklama: learn/error-handling/ACIKLAMA.md
// Rust Book — Bölüm 9: Error Handling (hata yönetimi)
// Çalıştır: cargo run -p error-handling
//
// İki tür hata: KURTARILAMAZ (panic!) ve KURTARILABİLİR (Result<T, E>). Rust'ta exception
// (try/catch) YOKTUR; hatalar türle (Result) taşınır, böylece ele almayı derleyici hatırlatır.
//
// Doğrulanmış (rustc ile): kullanılmayan Result → "unused `Result` that must be used" uyarısı;
// `?` operatörü Result/Option dönmeyen fn'de → E0277. Ayrıntı: DERLEYİCİ PERSPEKTİFİ (en altta).

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Her bölümde önce "SADE ANLATIM" kutusunu oku, sonra teknik açıklamaya geç.
// Bu dosyadaki benzetmelerin hepsi aynı yerde geçer: bir ev mutfağında.
//
// Bu bölümün tek cümlelik özeti: İşler ters gidebilir; Rust'ta bir fonksiyon "olmadı" diyecekse
// bunu dönüş değerinin içine yazar, derleyici de o değere bakmanı hatırlatır.
//
// MİNİ SÖZLÜK (bu dosyada geçen kelimeler):
//   hata (error)           → işin planlandığı gibi gitmemesi: dosya yok, yazı sayı değil gibi.
//   panic (panik)          → "devam edemem" deyip işi yarıda kesmek. O iş kolu orada durur.
//   Result                 → içinde ya sonuç ya da hata bulunan kutu. İki hâli var: Ok ve Err.
//   Ok(değer) / Err(hata)  → kutunun iki hâli: "oldu, işte sonuç" / "olmadı, işte nedeni".
//   Option, Some / None    → içinde ya bir değer (Some) bulunan ya da boş duran (None) kutu.
//   enum                   → "şunlardan biri" diyen tür. Result bir enum'dur: ya Ok ya Err.
//   match                  → kutunun hangi hâlde olduğuna bakıp her hâl için yapılacak işi yazmak.
//   unwrap / expect        → kutuyu sormadan açmak. İçinde Err varsa program panikler.
//   ? (soru işareti)       → "hata geldiyse ben de hemen o hatayla döneyim" demenin kısa yolu.
//   yayma (propagation)    → hatayı kendin çözmeyip seni çağırana iletmek.
//   exception (try/catch)  → başka dillerdeki "hata fırlatma" yöntemi. Rust'ta yoktur.
//   thread (iş parçacığı)  → programın içinde çalışan bir iş kolu. Küçük programda tek tanedir.
//   stack (yığın)          → çağrılan fonksiyonların üst üste dizildiği yer: "kim kimi çağırdı".
//   unwind / abort         → panikten sonraki iki yol: ortalığı toplayıp çıkmak / hemen çıkmak.
//   backtrace              → panik anında "buraya hangi fonksiyonlardan geçerek geldik" listesi.
//   Drop                   → bir değerin işi bitince yapılan temizlik (bkz. ownership).
//   invariant (değişmez)   → programın "hep doğru olmalı" diye güvendiği kural:"yaş eksi olmaz".
//   trait                  → bir türün "şunu yapabilirim" diye verdiği söz (Böl. 10'da anlatılır).
//   From                   → "A türünden B türü şöyle yapılır" diyen dönüşüm tarifi (bir trait).
//   Display / Debug        → bir değeri yazıya dökme biçimi: kullanıcıya {} / geliştiriciye {:?}.
//   #[must_use]            → "bu değeri kullanmadan bırakma" etiketi. Bırakırsan derleyici uyarır.
//   E0277 gibi kodlar      → derleyicinin hata numaraları. `rustc --explain E0277` açıklar.
//   derleyici (rustc)      → yazdığın kodu çalışan programa çeviren ve hataları önceden bulan araç.

use std::fmt;
use std::num::ParseIntError;

mod derinlesme;
fn main() {
    panik_vs_result();

    println!("ikiye_bol(10, 2) = {:?}", ikiye_bol(10, 2));
    println!("ikiye_bol(10, 0) = {:?}", ikiye_bol(10, 0));

    match propagasyon() {
        Ok(n) => println!("propagasyon OK: {n}"),
        Err(e) => println!("propagasyon HATA: {e}"),
    }

    from_donusumu();
    derinlesme::calistir();
}

// 1. panic! — kurtarılamaz hata
// SADE ANLATIM: Mutfakta kek yaparken ocak tutuşursa keke devam etmezsin; her şeyi bırakır, ocağı
// kapatır ve mutfaktan çıkarsın. `panic!` budur: program "bu durumda devam etmek tehlikeli" der ve
// o işi yarıda keser. Rust da çıkarken elindekileri toplar (Drop). Neden? Bozuk durumla devam eden
// program yanlış sonuç üretir; durmak daha iyidir.
//   Mini örnek:
//     let v = vec![1, 2, 3];
//     println!("{}", v[1]);         // ✓ 2 — ikinci eleman var
//     println!("{}", v[99]);        // ✗ derlenir ama çalışırken panikler: 100. eleman yok
//     println!("buraya gelinmez");  // panikten sonraki satırlar çalışmaz
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
// SADE ANLATIM: Kardeşini süt almaya markete gönderdin. Eve iki şekilde dönebilir: elinde sütle
// ya da "süt kalmamış" yazan bir notla. İkisi de felaket değildir; mutfak yanmadı, yalnız plan
// değişecek. Rust'ta `Result` bu dönüştür: `Ok(süt)` ya da `Err(not)`. Neden böyle? Fonksiyon
// "olmayabilir" bilgisini dönüş türüne yazınca, çağıran poşeti açıp içine bakmadan sütü
// kullanamaz; hata ihtimali gözden kaçmaz.
//   Mini örnek:
//     fn sut_al(rafta: u32) -> Result<u32, String> {
//         if rafta == 0 { Err(String::from("süt kalmamış")) } else { Ok(1) }
//     }
//     match sut_al(0) {
//         Ok(adet) => println!("{adet} kutu süt geldi"),  // rafta süt olsaydı bu kol çalışırdı
//         Err(not) => println!("olmadı: {not}"),          // ✓ raf boş: bu kol çalışır
//     }
//     let adet: u32 = sut_al(3);    // ✗ derlenmez: gelen şey u32 değil, Result; önce açmalısın
fn ikiye_bol(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err(String::from("sıfıra bölme"))
    } else {
        Ok(a / b)
    }
    // unwrap()/expect Result'ı açar ama Err ise PANİK eder; expect mesaj verdirir, hata ayıklamada iyidir.
    // İkisi de "Err olamaz" dediklerin içindir; aksi halde match/`?` kullan.
    //
    // Not (doğrulanmış): Result #[must_use]'tır. `ikiye_bol(10, 2);` deyip sonucu yok sayarsan
    // "unused `Result` that must be used" UYARISI alırsın. Bilerek yok saymak için: `let _ = ...;`
}

// 3. ? operatörü — hata yayma (propagation)
// SADE ANLATIM: Annen sana "kek yap" dedi, sen de kardeşini süte yolladın. Kardeşin "süt
// kalmamış" notuyla dönerse yapabileceğin en doğru şey, keki bırakıp aynı notu annene götürmektir.
// Süt geldiyse hiçbir şey olmamış gibi devam edersin. `?` işareti bu iki cümlenin kısaltmasıdır:
// "Ok ise içindekini ver, Err ise ben de hemen o Err ile döneyim." Neden böyle? `?` olmasaydı her
// adımdan sonra aynı `match`i elle yazman gerekirdi; kod uzar, asıl iş arada kaybolurdu.
//   Mini örnek (sut_al, bir üstteki kutudaki fonksiyon):
//     fn kek_yap() -> Result<String, String> {
//         let sut = sut_al(0)?;                // Err geldi: fonksiyon BURADA o Err ile döner
//         Ok(format!("{sut} kutu sütle kek"))  // süt gelseydi buraya ulaşılırdı
//     }
//     fn mutfak() {
//         let sut = sut_al(3)?;   // ✗ derlenmez: mutfak() Result dönmüyor, hatayı iletemez
//     }
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
    // (Sade: "FromResidual uygulayan tür" = `?`in erken dönüş yapabildiği tür demek. Günlük
    // kullanımda bu Result ve Option'dır; ötekileri şimdilik bilmen gerekmez.)
}

// 4. Farklı hata türlerini TEK türde toplamak + ?'in From dönüşümü
// SADE ANLATIM: Marketten "süt kalmamış" notu gelir, fırından "ekmek daha çıkmadı" notu gelir.
// Annen her dükkânın notunu ayrı biçimde okumak istemez; buzdolabının üstünde TEK bir "olmayanlar
// listesi" tutar ve her not oraya aynı biçimde geçirilir. Rust'ta bu liste, kendi yazdığın hata
// enum'udur. `From` ise "market notu listeye şöyle geçirilir" tarifidir; tarifi bir kez yazarsın,
// sonra `?` çeviriyi kendisi yapar. Neden böyle? Bir fonksiyon tek bir hata türü ilan edebilir;
// From olmasaydı her adımda gelen hatayı o türe elle çevirmen gerekirdi.
//   Mini örnek:
//     enum Olmadi { Market(String), Firin(String) }   // tek liste, iki çeşit satır
//     impl From<MarketNotu> for Olmadi {               // çeviri tarifi: bir kez yazılır
//         fn from(n: MarketNotu) -> Self { Olmadi::Market(n.yazi) }
//     }
//     fn kahvalti() -> Result<u32, Olmadi> {
//         let sut = marketten_al()?;   // ✓ MarketNotu gelirse `?` onu Olmadi'ye çevirip döner
//         Ok(sut)
//     }
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
    // Neden From? Alt fonksiyonlar farklı hata türleri döndürür (ParseIntError, IO hatası...); `?`
    // bunları ilan edilen TEK türe From ile köprüler, elle dönüştürme gerekmez.
}

// SADE ANLATIM (aşağıdaki bölüm için): `?` sıradan bir `match`e çevrilir, içinde sihir yoktur;
// ayrıca "sonucu kullanmadın" uyarısı ve panikte toplanan kaynaklar anlatılır. İlk okuyuşta atla,
// dört bölüm yeter.
// ============================================================================
// DERLEYİCİ PERSPEKTİFİ — Result ve ? nasıl çalışıyor?
// ============================================================================
//
// • Result<T, E> ve Option<T> SIRADAN enum'lardır; özel çalışma-anı mekanizması yoktur.
//   Rust'ta exception YOK — "hata olabilir" DÖNÜŞ TÜRÜNE kodlanır; çağıran Err'i ele almak zorundadır.
//
// • #[must_use]: TÜR olarak yalnız Result bu öznitelikle işaretlidir; Option türü değildir
//   (`Some(3);` ya da Option dönen bir çağrıyı yok saymak uyarı vermez). Yalnız bazı Option
//   METOTLARI tek tek #[must_use] taşır. Result'ın değerini kullanmadan bırakırsan derleyici
//   uyarır ("unused `Result` that must be used") — "hatayı kontrol etmeyi unutma" böylece
//   hatırlatılır. (Bilerek yok say: `let _ =`.)
//
// (Sade: "indirgeme" (lowering/desugaring), derleyicinin kısa yazımı aynı işi yapan uzun ve yalın
// koda çevirmesi demek. Aşağıdaki Continue = "devam et", Break = "dur ve dön" anlamına gelir.)
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
