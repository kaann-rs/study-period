// Sağlamlaştırma — İleri konu: SUBTYPING ve VARIANCE (alt-tip ve değişkenlik)
// Çalıştır: cargo run -p ileri-variance
//
// Kaynak: Rust for Rustaceans (Jon Gjengset), Böl. 2 "Lifetime Variance" (Listing 2-11).
// Bu konu Rust Book'ta neredeyse yok ama lifetime hatalarının görünmez sebebi çoğu zaman budur.
// Senin sorduğun `&'s mut &'a v` kalıbı tam buraya düşüyor.
//
// Doğrulanmış (rustc ile): MutStr iki lifetime ile DERLENİR, tek lifetime ile E0502; &mut
// ardındaki tür invariant olduğu için uzun-ömürlü değer kısaltılamaz (E0597); paylaşımlı & ise
// covariant olduğu için kabul edilir. Ayrıntı: DERLEYİCİ PERSPEKTİFİ (altta).

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
fn subtyping() {
    fn kisa_bekler(x: &str) -> usize {
        x.len()
    }
    let sabit: &'static str = "program boyu yaşarım"; // &'static str
    // &'static str, &'a str beklenen yere GEÇER (alt-tip): daha uzun ömürlü, daha "işe yarar".
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
fn covariance() {
    fn oku(v: &Vec<&str>) -> usize {
        v.len()
    }
    let depo: Vec<&'static str> = vec!["a", "b", "c"];
    // &Vec<&'static str> --covariant--> &Vec<&'a str>: sorun yok.
    println!(
        "covariance: paylaşımlı & esnek, eleman sayısı = {}",
        oku(&depo)
    );
    // Neden güvenli? Paylaşımlı & yalnız OKUR. Daha uzun ömürlü veriyi daha kısa süre okumak
    // hiçbir şeyi bozmaz → derleyici uzun'u kısa'ya zahmetsiz "kısaltır" (covariance).
}

// ============================================================================
// 2. INVARIANCE + senin örneğin:  &'a mut &'b str  (RfR Listing 2-11)
// ============================================================================
// &mut T, T'de INVARIANT'tır: ne alt-tipi ne üst-tipi kabul eder, TAM O TİP gerekir. Sebebi:
// &mut hem okur HEM yazar. Uzun-ömürlüyü kısa yerine koymaya izin verilse, fonksiyon içeriye
// KISA ömürlü bir şey yazar; çağıran hâlâ "uzun ömürlü" sanıp kullanır → dangling. Bu yüzden
// mutability sağlayan her tür (&mut T, Cell<T>, RefCell<T>...) invariant'tır.
struct MutStr<'a, 'b> {
    s: &'a mut &'b str, // dış &mut'un ömrü 'a, içteki &str'nin ömrü 'b — AYRI
}

fn invariance_mutstr() {
    // İKİ lifetime ile sorunsuz çalışır (DOĞRULANDI):
    let mut s = "hello";
    *MutStr { s: &mut s }.s = "world"; // içteki &str'yi "world" yap
    println!("invariance: MutStr<'a,'b> ile s = {s}"); // world

    // NEDEN iki lifetime? Tek lifetime olsaydı:  struct MutStr<'a> { s: &'a mut &'a str }
    // ...aynı kod DERLENMEZDİ → E0502 "cannot borrow `s` as immutable because it is also
    // borrowed as mutable". Mantık:
    //   - İçteki &str başta &'static ("hello" literal'i).
    //   - println! için derleyici s'in &mut borrow'unu KISALTMAK ister (ki paylaşımlı okuma sığsın).
    //   - İki lifetime varsa: iç 'b invariant → 'static'e çivili kalır. `&'a mut &'b str`'in örtük
    //     well-formedness kısıtı 'b: 'a'dır; 'b = 'static olunca bu  'static: 'a  olur → HER 'a için
    //     doğru. Yani kısıt 'a'ya alttan hiç baskı yapmaz; dış 'a serbest kalıp println'den önce
    //     bitecek kadar KISALIR (çünkü 'a covariant). Çakışma yok → OK.
    //   - Tek lifetime varsa: dış ve iç AYNI 'a. İç konum invariant olduğundan 'a = 'static'e çivilenir
    //     (tek parametre hem invariant hem covariant konumda → invariant kazanır). Artık 'a kısalamaz;
    //     &mut borrow 'static yaşar → println'e taşar. Derleyici: "hâlâ mutably borrowed" (E0502).
    // Ders: &mut ardındaki ömür invariant olduğu için, DIŞ borrow ömrünü İÇ veri ömründen AYIRMAK
    // (ayrı lifetime) esnekliği geri kazandırır. "Gereksiz gibi duran ikinci lifetime"ın sebebi bu.
    println!("invariance: tek lifetime'da aynı kod E0502 verir (yorumda açıklandı)");
}

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
