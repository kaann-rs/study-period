// Açıklama: learn/advanced-concepts/ileri-variance/ACIKLAMA.md
// Sağlamlaştırma — İleri konu: SUBTYPING ve VARIANCE (alt-tip ve değişkenlik)
// Çalıştır: cargo run -p ileri-variance
//
// Kaynak: RfR Böl. 1 (Foundations → Borrowing and Lifetimes → Lifetimes). Kesin listing no.
// yayıncının ayrıntılı içindekiler listesinde yer almıyor (DOĞRULANAMADI).
// Bu konu Rust Book'ta ayrıntılı ele alınmaz; Rustonomicon'da ele alınır — lifetime hatalarının
// görünmez sebebi çoğu zaman budur.
// Senin sorduğun `&'s mut &'a v` kalıbı tam buraya düşüyor.
//
// Doğrulanmış (rustc ile): MutStr iki lifetime ile DERLENİR, tek lifetime ile E0502; &mut
// ardındaki tür invariant olduğu için uzun-ömürlü değer kısaltılamaz (E0597); paylaşımlı & ise
// covariant olduğu için kabul edilir. Ayrıntı: DERLEYİCİ PERSPEKTİFİ (altta).

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Her bölümde önce "SADE ANLATIM" kutusunu oku; teknik kısım zor gelse de ana fikir orada.
//
// Bu dosyayı okumadan önce bilmen gerekenler: `learn/ownership` (ödünç alma, `&` ve `&mut`)
// ve `learn/generics-traits-lifetimes` (lifetime, yani `'a` yazımı).
//
// Bu bölümün tek cümlelik özeti: uzun süre geçerli bir referans, kısa süre geçerli bir
// referansın istendiği yerde çoğu zaman kullanılabilir; ama içine YAZILABİLEN bir yerin (`&mut`)
// ardındaki tür için bu esneklik yoktur, orada süre tam tutmalıdır.
//
// MİNİ SÖZLÜK (bu dosyada geçen kelimeler):
//   referans (& / &mut)  → veriyi gösteren ok. `&` yalnız bakar, `&mut` değiştirebilir.
//   borrow (ödünç alma)  → sahibi değiştirmeden veriye bir süreliğine referans almak.
//   lifetime (ömür, 'a)  → bir referansın geçerli olduğu kod aralığının adı. Hiçbir şeyin
//                          ömrünü uzatmaz; yalnız "şu kadar süre geçerli" diye etiketler.
//   'static              → programın sonuna kadar geçerli referansın etiketi ("merhaba" gibi
//                          koda yazılmış sabit yazılar böyledir).
//   literal              → koda doğrudan yazılmış değer: "hello", 5, true.
//   alt-tip (subtype)    → bir türün "en az onun kadar işe yarayan" hâli; onun yerine geçebilir.
//   üst-tip (supertype)  → tersi: daha az şey vaat eden, daha genel tür.
//   variance (değişkenlik)→ "içteki tür yerine alt-tipi koyarsam, dıştaki tür de yerine geçer
//                          mi?" sorusunun cevabı. Üç cevap vardır:
//   covariant            → evet, aynı yönde geçer (esnek taraf). `&'a T` böyledir.
//   invariant            → hayır, tam aynı tür gerekir (katı taraf). `&mut T`, T için böyledir.
//   contravariant        → ters yönde geçer. Yalnız fonksiyon argümanlarında görülür: `fn(T)`.
//   generic parametre    → türün içindeki boşluk: `Vec<T>`'deki T, `&'a str`'deki 'a.
//   dangling (sarkan)    → gösterdiği veri silinmiş, boşu gösteren referans.
//   Cell / RefCell       → `&` üzerinden bile içini değiştirmeye izin veren kutular.
//   PhantomData          → yer kaplamayan, derleyiciye "bu türü taşıyormuşum gibi davran"
//                          diyen sahte alan.
//   borrow checker       → ödünç alma kurallarını derleme sırasında denetleyen derleyici parçası.
//   well-formedness      → bir türün anlamlı olabilmesi için kendiliğinden gereken şartlar.
//   E0502, E0597         → derleyicinin hata numaraları (çakışan ödünç / yeterince yaşamıyor).

fn main() {
    subtyping();
    covariance();
    invariance_mutstr();
}

// ============================================================================
// 0. SUBTYPING (alt-tip) — lifetime'larda
// ============================================================================
// Bir tip A, B'nin ALT-TİPİdir (subtype) eğer A "en az B kadar işe yarıyorsa" ve B'nin
// beklendiği her yerde A kullanılabiliyorsa. Lifetime'larda:  'b: 'a  ('b, 'a'yı KAPSAR / daha
// uzun yaşar) ise  'b, 'a'nın ALT-TİPİdir — çünkü daha uzun yaşayan referans her zaman daha
// kısa beklenen yerde kullanılabilir. Uç örnek: 'static her 'a'nın alt-tipidir.
// SADE ANLATIM: Lunaparkta iki tür bilet var: bir günlük bilet ve bütün yıl geçerli kart.
// Kapıdaki görevli "bugün geçerli bir bilet göster" derse yıllık kartı gösteren de içeri
// girer, çünkü yıllık kart bugünü de kapsar. Tersi olmaz: "yıl sonuna kadar geçerli olsun"
// diyen görevli günlük bileti kabul etmez. Rust'ta daha uzun geçerli bir referans, daha kısa
// geçerli bir referansın beklendiği her yere verilebilir. Buna alt-tip ilişkisi denir.
// Bu neden var? Olmasaydı her fonksiyona tam aynı süreli referans vermen gerekirdi; `'static`
// bir yazıyı sıradan bir `&str` bekleyen fonksiyona bile veremezdin.
//   Mini örnek:
//     fn kisa_bekler(x: &str) -> usize { x.len() }
//     fn uzun_bekler(x: &'static str) {}
//     let sabit: &'static str = "yıllık kart";
//     kisa_bekler(sabit);          // ✓ uzun süreli olan, kısa süreli beklenen yere geçer
//     let gecici = String::from("günlük bilet");
//     kisa_bekler(&gecici);        // ✓ kısa süre yeterli
//     // uzun_bekler(&gecici);     // ✗ derlenmez: gecici program sonuna kadar yaşamıyor
fn subtyping() {
    fn kisa_bekler(x: &str) -> usize {
        x.len()
    }

    let sabit: &'static str = "program boyu yaşarım"; // &'static str
    println!(
        "subtyping: &'static → &'a kabul edildi, len = {}",
        kisa_bekler(sabit)
    );
}

// ============================================================================
// 1. COVARIANCE — "alt-tipi, tipin yerine koyabilirsin"
// ============================================================================
// &'a T, hem 'a'da hem T'de COVARIANT'tır: daha uzun 'a veya T'nin alt-tipi serbestçe kabul edilir.
// Bu yüzden &Vec<&'static str>, &Vec<&'a str> beklenen yere geçer. Covariance "esnek/rahat" taraftır.
// (Sade: "covariant" = içteki tür esnerse dıştaki tür de onunla aynı yönde esner demek.)
// SADE ANLATIM: Müzedeki cam vitrine yalnız bakabilirsin; içine bir şey koyamazsın. İçinde
// "çok dayanıklı" bir eşya duran vitrin, "az dayanıklı eşya vitrini" bekleyen bir salona
// rahatça konabilir: bakan kişi için içerideki şeyin fazladan dayanıklı olması zarar vermez.
// `&T` böyle bir vitrindir: yalnız okur. Bu yüzden `&Vec<&'static str>`, daha kısa süreli
// `&Vec<&'a str>` beklenen yere verilebilir. Bu neden var? Yalnız okunan veride "fazladan
// uzun yaşamak" kimseye zarar vermez; derleyici de bu esnekliği kendiliğinden tanır.
//   Mini örnek:
//     fn ikisi<'a>(x: &'a str, y: &'a str) -> &'a str { if x.len() > y.len() { x } else { y } }
//     let yerel = String::from("kısa ömürlü");
//     let r = ikisi("uzun ömürlü", &yerel);   // ✓ 'static olan yazı, yerel'in süresine iner
//     println!("{r}");
//     fn oku(v: &Vec<&str>) -> usize { v.len() }
//     let depo: Vec<&'static str> = vec!["a", "b"];
//     oku(&depo);                             // ✓ içteki 'static de daha kısa süreye iner
fn covariance() {
    fn oku(v: &Vec<&str>) -> usize {
        v.len()
    }

    let depo: Vec<&'static str> = vec!["a", "b", "c"];
    println!(
        "covariance: paylaşımlı & esnek, eleman sayısı = {}",
        oku(&depo)
    );
}

// ============================================================================
// 2. INVARIANCE + senin örneğin:  &'a mut &'b str  (RfR Böl. 1, Lifetimes)
// ============================================================================
// &mut T, T'de INVARIANT'tır: ne alt-tipi ne üst-tipi kabul eder, TAM O TİP gerekir. Sebebi:
// &mut hem okur HEM yazar. Uzun-ömürlüyü kısa yerine koymaya izin verilse, fonksiyon içeriye
// KISA ömürlü bir şey yazar; çağıran hâlâ "uzun ömürlü" sanıp kullanır → dangling. Bu yüzden
// mutability sağlayan her tür (&mut T, Cell<T>, RefCell<T>...) invariant'tır.
// (Sade: "invariant" = esneme yok; ne daha uzun ne daha kısa, tam o süre gerekir demek.)
// SADE ANLATIM: Üstünde "yalnız yıllık kart" yazan bir kutun var; içinden çıkan her şeyin
// yıllık kart olduğuna güveniyorsun. Bu kutuyu birine "günlük bilet kutusu" diye verirsen
// içine günlük bilet atar. Sen yarın kutuyu açıp çıkanı yıllık kart sanırsın, oysa süresi
// dolmuştur. İçine YAZILABİLEN bir kutunun etiketi bu yüzden değiştirilemez. `&mut T` böyle bir
// kutudur: T'nin süresi ne uzatılır ne kısaltılır. Bu neden var? Yazmaya izin verip etiketi de
// esnetseydik, süresi bitmiş bir veriyi gösteren sarkan referans elde edilirdi.
//   Mini örnek:
//     fn degistir<'a>(kutu: &mut &'a str, yeni: &'a str) { *kutu = yeni; }
//     let mut uzun: &'static str = "yıllık";
//     degistir(&mut uzun, "başka yıllık");    // ✓ etiket aynı: 'static kutuya 'static yazı
//     {
//         let gecici = String::from("günlük");
//         // degistir(&mut uzun, &gecici);    // ✗ derlenmez: kutunun etiketi 'static,
//     }                                       //   gecici ise bu satırda siliniyor
//     println!("{uzun}");
struct MutStr<'a, 'b> {
    s: &'a mut &'b str, // dış &mut'un ömrü 'a, içteki &str'nin ömrü 'b — AYRI
}

fn invariance_mutstr() {
    // İKİ lifetime ile sorunsuz çalışır (DOĞRULANDI):
    let mut s = "hello";
    *MutStr { s: &mut s }.s = "world"; // içteki &str'yi "world" yap
    println!("invariance: MutStr<'a,'b> ile s = {s}"); // world

    // SADE ANLATIM: Burada iki ayrı süre var. Biri, kutuyu ne kadar süre ELİNDE tuttuğun
    // (dıştaki `&'a mut`); öteki, kutunun içindeki yazının ne kadar süre GEÇERLİ olduğu
    // (içteki `&'b str`). Kütüphaneden bir kitabı masana beş dakikalığına alman gibi: kitap
    // yıllarca yaşar, sen onu beş dakika tutarsın. İki süreyi tek bir etikete yazmak zorunda
    // kalırsan derleyici "beş dakika" ile "yıllarca"yı eşitler; içteki süre kısaltılamadığı
    // için kitabı yıllarca elinde tutmuş sayılırsın ve başka kimse ona bakamaz. İki ayrı
    // etiket bu derdi çözer: tutma süresi kısa kalır, yazının süresi uzun kalır.
    //   Mini örnek:
    //     struct Iki<'a, 'b> { s: &'a mut &'b str }   // iki etiket
    //     struct Tek<'a> { s: &'a mut &'a str }       // tek etiket
    //     let mut s = "hello";
    //     *Iki { s: &mut s }.s = "world";
    //     println!("{s}");                    // ✓ ödünç hemen bitti, s yine okunabiliyor
    //     // *Tek { s: &mut s }.s = "world";
    //     // println!("{s}");                 // ✗ derlenmez: ödünç 'static'e uzadı, s kilitli
    // NEDEN iki lifetime? Tek lifetime olsaydı:  struct MutStr<'a> { s: &'a mut &'a str }
    // ...aynı kod DERLENMEZDİ → E0502 "cannot borrow `s` as immutable because it is also
    // borrowed as mutable". Mantık:
    //   - İçteki &str başta &'static ("hello" literal'i).
    //   - println! için derleyici s'in &mut borrow'unu KISALTMAK ister (ki paylaşımlı okuma sığsın).
    // (Sade: "well-formedness kısıtı" = türün anlamlı olması için kendiliğinden gelen şart
    //  demek. Buradaki şart: içteki veri, onu gösteren dıştaki referanstan önce ölemez.)
    //   - İki lifetime varsa: iç 'b invariant → 'static'e çivili kalır. `&'a mut &'b str`'in örtük
    //     well-formedness kısıtı 'b: 'a'dır; 'b = 'static olunca bu  'static: 'a  olur → HER 'a için
    //     doğru. Yani kısıt 'a'ya alttan hiç baskı yapmaz; dış 'a serbest kalıp println'den önce
    //     bitecek kadar KISALIR (çünkü 'a covariant). Çakışma yok → OK.
    //   - Tek lifetime varsa: dış ve iç AYNI 'a. İç konum invariant olduğundan 'a = 'static'e çivilenir
    //     (tek parametre hem invariant hem covariant konumda → invariant kazanır). Artık 'a kısalamaz;
    //     &mut borrow 'static yaşar → println'e taşar. Derleyici: "hâlâ mutably borrowed" (E0502).
    println!("invariance: tek lifetime'da aynı kod E0502 verir (yorumda açıklandı)");
}

// SADE ANLATIM (aşağıdaki bölüm için): Yukarıda iki davranış gördün: yalnız bakılan yer esnek,
// içine yazılan yer katı. Aşağıdaki bölüm bunları bir tabloya döker ve üçüncü bir türü ekler:
// contravariant. O da şudur: "hangi bileti getirirsen getir kabul ederim" diyen bir görevli,
// "yalnız yıllık kart kabul eden görevli" aranan yere rahatça konabilir; yani fonksiyon
// argümanlarında yön ters döner (`fn(&'a str)`, `fn(&'static str)` beklenen yere geçer).
// Bölüm ayrıca derleyicinin bu kuralları senin yerine nasıl çıkardığını anlatır. İlk okuyuşta
// atlayabilirsin; yukarıdaki üç bölümü anlamış olman yeter.
// ============================================================================
// DERLEYİCİ PERSPEKTİFİ — variance nereden gelir, neden önemli?
// ============================================================================
//
// • ÜÇ VARIANCE (bir tipin, generic parametresi yerine hangi "benzer" tipleri kabul ettiği):
//     - COVARIANT:   alt-tip kabul edilir. &'a T ('a ve T'de), Box<T>, Vec<T> → covariant.
//                    "Sadece üret/oku" konumları covarianttır.
//     - INVARIANT:   tam o tip gerekir. &mut T (T'de), Cell<T>/RefCell<T>, *mut T → invariant.
//                    "Hem oku hem yaz" (mutability) konumları invarianttır.
//     - CONTRAVARIANT: ÜST-tip kabul edilir; fonksiyon ARGÜMANLARINDA görülür. fn(T) argümanı
//                    T'de contravariant. RfR örneği: fn(&'a str) (kısa'yı da kabul eder) daha işe
//                    yarar, bu yüzden fn(&'static str) (yalnız 'static kabul) beklenen yere geçer.
//   Variance ELLE yazılmaz; derleyici bir tipin variance'ını alanlarının konumlarından YAPISAL
//   olarak çıkarır. Kütüphane yazarları PhantomData<...> ile (ör. PhantomData<&'a T> vs
//   PhantomData<*mut T>) istedikleri variance'ı belirtebilir.
//
// • NEDEN invariance "esnekliği kısar": &'a T genelde kısaltılabilir (covariant), ama &mut T
//   ardındaki T kısaltılamaz (invariant). Bu yüzden &mut Vec<&'static str>, &mut Vec<&'a str>
//   beklenen yere GEÇMEZ — verince fonksiyon içeriye kısa-ömürlü yazabilir, çağıran 'static sanıp
//   kullanır (DOĞRULANDI: o senaryoda E0597 "does not live long enough"). Paylaşımlı &Vec<...> ise
//   covariant olduğu için geçer.
//
// • PRATİK KURAL (RfR): tiplerini mümkün olduğunca COVARIANT tut. Bir alan "&mut ardında ortak
//   lifetime" yüzünden gereksiz yere invariant oluyorsa, dış borrow ömrü ile iç veri ömrünü AYRI
//   lifetime parametrelerine böl (MutStr<'a,'b> gibi). İkinci lifetime'ın bilişsel maliyetiyle,
//   invariance'ın getirdiği katılığı tartıp karar ver.
//
// • Variance tamamen DERLEME ZAMANI bir kavramdır (runtime'da lifetime/variance yoktur). Borrow
//   checker, bir referansın ömrünü "kısaltmaya" çalışırken variance kurallarına uyar; uyamazsa
//   (invariant konum) ömrü olduğu gibi tutar ve çakışmayı hata olarak bildirir.
