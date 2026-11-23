// Derinleşme — Bölüm 13: Closures & Iterators (pekiştirme: gerçekçi senaryo, sık hatalar, alıştırmalar)
// Çalıştır: cargo run -p closures-iterators   |   Test: cargo test -p closures-iterators
//
// main.rs üç yakalama biçimini, Fn/FnMut/FnOnce'u ve map/filter/collect'i tek tek tanıttı. Burada
// onları bir SİPARİŞ VERİ HATTINDA birlikte kullanıyoruz ve main.rs'te olmayanları ekliyoruz:
// "hangi gövde hangi trait'i doğurur" (ve `move`'un buna ETKİSİZ olduğu), closure döndürme ve
// struct alanında saklama, on kadar yeni adaptör, sonsuz bir iterator, `collect`'in tür çıkarımı
// ve en sık görülen beş borrow hatasının düzeltilmiş hâlleri.

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Önce main.rs'i oku; closure ve iterator'ın ne olduğu orada anlatıldı. Bu dosya ikisini gerçeğe
// yakın bir işte birlikte kullanır: bir şubenin sipariş kayıtlarını okur, süzer, toplar, gruplar.
// Benzetmelerde yine kargo şubesindeyiz: görev notları ve taşıma bandı.
//
// Bu dosyanın tek cümlelik özeti: Küçük notları (closure) bandın istasyonlarına (iterator
// adaptörleri) takarak, uzun döngüler yazmadan veriyi adım adım işlersin.
//
// MİNİ SÖZLÜK (yalnız bu dosyada yeni geçenler; öteki terimler main.rs'in başındaki sözlükte):
//   gövde                    → closure'ın `|..|` işaretinden sonraki kısmı: yaptığı iş.
//   veri hattı (pipeline)    → verinin art arda adımlardan geçerek işlendiği zincir.
//   `impl Fn(..)` (dönüşte)  → "bir closure döndürüyorum; türünün adını yazmıyorum".
//   Box<dyn Fn(..)>          → closure'ı heap'teki bir kutuya koyup türünü gizlemek.
//   tür silme                → asıl türü gizleyip yalnız "çağrılabilir" olduğunu bırakmak.
//   fn işaretçisi            → hiçbir şey yakalamayan fonksiyonu gösteren ok.
//   statik dispatch, vtable  → çağrının derlerken çözülmesi / çalışırken bakılan çizelge (Böl. 10).
//   geri çağrı (callback)    → "şu olunca bunu çalıştır" diye başkasına verilen closure.
//   filter_map               → dönüştür ve None çıkanları at: map ile filter tek adımda.
//   fold, birikeç            → elde bir ara sonuç tutup her elemanla güncellemek / o ara sonuç.
//   partition                → elemanları tek geçişte "uyanlar" ve "uymayanlar" diye ikiye ayırmak.
//   flat_map                 → her elemandan birkaç eleman üretip hepsini tek sıraya dizmek.
//   enumerate / zip          → sıra numarası eklemek / iki sırayı eleman eleman eşlemek.
//   take / take_while        → ilk N elemanı almak / şart bozulana kadar almak.
//   windows / chunks         → örtüşen komşu grupları / örtüşmeyen parçalar (dilimlerde).
//   peekable, peek           → sıradaki elemana, onu sıradan almadan bakabilmek.
//   find / count / inspect   → uyan ilk elemanı bul / say / geçerken göz at (değiştirmeden).
//   for_each                 → her eleman için bir iş yapan tüketici.
//   HashMap / BTreeSet       → anahtar → değer sözlüğü (sırasız) / tekrarsız ve sıralı küme.
//   IntoIterator             → "`for` ile gezilebilir" sözü.
//   FromIterator             → "`collect` ile kurulabilir" sözü.
//   turbofish (::<T>)        → türü çağrı sırasında elle söylemek: `collect::<Vec<_>>()`.
//   taşma (overflow)         → sayının türüne sığmaması. checked_add: taşarsa None veren toplama.
//   saturating_sub           → sıfırın altına inmeyen çıkarma: 30 - 50 → 0.
//   NaN                      → "sayı değil" anlamına gelen özel f64 değeri; sıralamayı bozar.
//   yan etki (side effect)   → değer üretmek dışında yapılan iş: ekrana yazmak, dosyaya yazmak.
//   NLL                      → ödüncün blok sonunda değil, son kullanıldığı yerde bitmesi.

use std::collections::{BTreeSet, HashMap};

pub fn calistir() {
    println!("\n== Derinleşme: Closures & Iterators ==");
    govde_traiti_belirler();
    closure_dondurme_ve_saklama();
    veri_hatti();
    dilim_ve_akis_adaptorleri();
    sonsuz_iterator();
    sahiplik_tembellik_collect();
    sik_hatalar_dogru_kullanim();
}

// ============================================================================
// 1. HANGİ GÖVDE HANGİ TRAIT'İ DOĞURUR? — `move` trait'i DEĞİL, yakalama biçimini değiştirir
// ============================================================================
// Nasıl çalışır? İki AYRI soru vardır ve sık karıştırılır:
//   (1) Değişken closure'a NASIL girer?  → &, &mut ya da değer olarak (move bunu "değer"e zorlar).
//   (2) Gövde yakaladığıyla NE YAPAR?     → okur (Fn), değiştirir (FnMut), dışarı taşır (FnOnce).
// Trait'i yalnız (2) belirler. Yani `move` ile içeri alınmış bir String'i sadece OKUYAN closure
// hâlâ Fn'dir; defalarca çağrılabilir. Tersine, `move` yazmasan bile gövde değeri dışarı
// taşıyorsa (drop, return, başka sahibe verme) closure yalnız FnOnce olur.
// SADE ANLATIM: Bir not hakkında iki ayrı soru sorulur ve ikisi sık karışır. Birincisi: not,
// kullandığı şeyi yanında mı taşıyor, yoksa masada bırakıp oradan mı bakıyor? (`move` bunu
// belirler.) İkincisi: not o şeyle ne YAPIYOR: okuyor mu, değiştiriyor mu, başkasına mı veriyor?
// Notun kaç kez uygulanabileceğini yalnız ikinci soru belirler. Kataloğu çantana koymuş olman
// (`move`) onu tek kullanımlık yapmaz; her gün açıp okuyabilirsin. Ama kataloğu müşteriye
// verirsen, çantada taşımış olsan da olmasan da, ikinci kez veremezsin.
//   Mini örnek:
//     let ad = String::from("Ayşe");
//     let oku = move || ad.len();    // move var, gövde yalnız okuyor → Fn
//     oku();
//     oku();                         // ✓ istediğin kadar çağrılır
//     let not = String::from("x");
//     let ver = || not;              // move yok, gövde dışarı veriyor → yalnız FnOnce
//     ver();
//     ver();                         // ✗ derlenmez: ikinci kez verilemez
fn iki_kez_cagir(f: impl Fn() -> usize) -> usize {
    f() + f()
}
fn uc_kez_cagir(mut f: impl FnMut() -> u32) -> u32 {
    f();
    f();
    f()
}
fn bir_kez_cagir(f: impl FnOnce() -> String) -> String {
    f()
}

fn govde_traiti_belirler() {
    println!("-- 1. gövde trait'i belirler --");
    let ad = String::from("Ayşe");
    // move VAR ama gövde yalnız okuyor → Fn. `ad` closure'ın içine taşındı, orada yaşıyor.
    let uzunluk = move || ad.chars().count();
    println!("move + okuma  → Fn     : {}", iki_kez_cagir(uzunluk));

    let mut sayac = 0;
    // Gövde değiştiriyor → FnMut. (Fn bekleyen iki_kez_cagir'a verilemez: bkz. §7-b.)
    let artir = || {
        sayac += 1;
        sayac
    };
    println!("değiştirme    → FnMut  : {}", uc_kez_cagir(artir));

    let not = String::from("teslim edildi");
    // move YOK ama gövde `not`'u DÖNDÜRÜYOR (dışarı taşıyor) → yalnız FnOnce.
    let ver = || not;
    println!("dışarı taşıma → FnOnce : {}", bir_kez_cagir(ver));
    // Beklenen çıktı:
    //   move + okuma  → Fn     : 8
    //   değiştirme    → FnMut  : 3
    //   dışarı taşıma → FnOnce : teslim edildi
}

// ============================================================================
// 2. CLOSURE DÖNDÜRMEK ve STRUCT ALANINDA SAKLAMAK
// ============================================================================
// SADE ANLATIM: Şube müdürü sabah bir kural kâğıdı yazar: "200 liradan pahalı paketler ayrı
// rafa." Müdür odasına döner, ama kâğıt görevlide kalır ve gün boyu kullanılır. Kâğıdın işe
// yaraması için üstündeki 200 sayısı da kâğıtla birlikte gitmelidir. Closure döndüren fonksiyon
// bu müdürdür: fonksiyon biter, closure yaşamaya devam eder; bu yüzden yakaladığını `move` ile
// içine alması gerekir. Neden? Almasaydı closure, fonksiyon bitince silinen bir değişkene bakıyor
// olurdu.
//   Mini örnek:
//     fn en_az(esik: u32) -> impl Fn(u32) -> bool { move |t| t >= esik }   // ✓ esik içeride
//     let pahali_mi = en_az(200);
//     pahali_mi(250);    // true
//     pahali_mi(90);     // false
//     fn bozuk(esik: u32) -> impl Fn(u32) -> bool { |t| t >= esik }   // ✗ derlenmez: move yok
#[derive(Debug, Clone, PartialEq)]
struct Siparis {
    no: u32,
    musteri: &'static str,
    sehir: &'static str,
    tutar: u32,
    teslim: bool,
}

// `impl Fn`: TEK bir somut closure türü döner; derleyici onu bilir, sen adını yazamazsın. Tahsis
// yok, çağrı statik dispatch. `move` şart: `esik` bu fonksiyonun yereli, closure ondan uzun yaşar.
fn en_az(esik: u32) -> impl Fn(&Siparis) -> bool {
    move |s| s.tutar >= esik
}

// (Sade: her closure'ın türü kendine özgüdür; iki closure aynı görünse bile türleri ayrıdır.
// Bir fonksiyon duruma göre farklı closure'lar döndürecekse, hepsini aynı boy bir zarfa koyar:
// `Box<dyn Fn>`. Zarfın içi farklıdır, ama dışarıdan hepsi "çağrılabilen bir şey"dir.)
// Kollar ortam YAKALAYAN farklı closure'lar döndürüyorsa `impl Fn` olmaz: her closure'ın türü
// benzersizdir (E0308; not: "no two closures, even if identical, have the same type").
// (Hiçbir şey yakalamayan closure'lar `fn` işaretçisine dönüşebildiği için o durumda derlenir;
// burada kollar `oran`/`miktar` yakalıyor.) Çözüm tür silme:
// Box<dyn Fn> — heap tahsisi + vtable üzerinden çağrı (Böl. 18'deki trait object'in aynısı).
fn indirim_sec(kod: &str) -> Box<dyn Fn(u32) -> u32> {
    match kod.split_once(':') {
        Some(("yuzde", n)) => {
            // Bozuk kod = indirim yok; %100'ü aşan oran kırpılır (yoksa çıkarma taşardı).
            let oran: u32 = n.parse().unwrap_or(0).min(100);
            // Çarpma u64'te: büyük tutarlarda `tutar * oran` u32'ye sığmayabilir.
            Box::new(move |tutar| {
                let indirim = u64::from(tutar) * u64::from(oran) / 100;
                tutar.saturating_sub(u32::try_from(indirim).unwrap_or(tutar))
            })
        }
        Some(("sabit", n)) => {
            let miktar: u32 = n.parse().unwrap_or(0);
            Box::new(move |tutar| tutar.saturating_sub(miktar))
        }
        _ => Box::new(|tutar| tutar),
    }
}

// (Sade: bir closure'ı struct alanında tutmak, kural kâğıdını panoya asmak gibidir. Yol 1'de
// pano tek bir kâğıt çeşidine göre yapılır: hızlıdır, ama başka çeşit asılmaz. Yol 2'de panoya
// aynı boy zarflar asılır: içine her çeşit kural girer, karşılığında küçük bir dolaylama vardır.)
// Alanda saklama, yol 1 — GENERIC `F`: strateji derleme anında sabit; sıfır maliyet. Bedeli:
// her farklı closure AYRI bir `Fiyatlayici<F>` türüdür, ikisini aynı Vec'e koyamazsın.
struct Fiyatlayici<F: Fn(u32) -> u32> {
    kural: F,
}
impl<F: Fn(u32) -> u32> Fiyatlayici<F> {
    fn uygula(&self, tutar: u32) -> u32 {
        // Parantez şart: `self.kural(tutar)` yazarsan derleyici `kural` adlı bir METOT arar
        // (E0599: no method named `kural`). Alandaki closure'ı çağırmak için (self.kural)(..).
        (self.kural)(tutar)
    }
}

// Yol 2 — Box<dyn Fn>: kurallar çalışma anında eklenir, hepsi AYNI türdedir → tek Vec'te durur.
// Geri çağrı (callback) listeleri ve eklenti noktaları bu yüzden neredeyse hep Box<dyn Fn..>'dir.
type Kural = Box<dyn Fn(&Siparis) -> bool>;
struct KuralSeti {
    kurallar: Vec<(&'static str, Kural)>,
}
impl KuralSeti {
    fn ekle(mut self, ad: &'static str, kural: impl Fn(&Siparis) -> bool + 'static) -> Self {
        self.kurallar.push((ad, Box::new(kural)));
        self
    }
    fn ihlaller(&self, s: &Siparis) -> Vec<&'static str> {
        let ihlal = |(ad, kural): &(&'static str, Kural)| (!kural(s)).then_some(*ad);
        self.kurallar.iter().filter_map(ihlal).collect()
    }
}

fn closure_dondurme_ve_saklama() {
    println!("-- 2. closure döndürme ve saklama --");
    let siparisler = siparisleri_oku(HAM_KAYITLAR);
    let buyukler: Vec<u32> = siparisler
        .iter()
        .filter(|s| en_az(200)(s))
        .map(|s| s.no)
        .collect();
    println!("en_az(200) → {buyukler:?}");
    let indirimli: Vec<u32> = ["yuzde:10", "sabit:300", "yok"]
        .iter()
        .map(|kod| indirim_sec(kod)(250))
        .collect();
    println!("indirim_sec → {indirimli:?}");

    let kdv = Fiyatlayici {
        kural: |t| t + t / 5,
    };
    let azami = 300;
    let kurallar = KuralSeti {
        kurallar: Vec::new(),
    }
    .ekle("teslim edilmemiş", |s| s.teslim)
    .ekle("limit aşımı", move |s| s.tutar <= azami); // ortamı yakalayan kural
    println!("kdv(250) = {}", kdv.uygula(250));
    for s in siparisler
        .iter()
        .filter(|s| !kurallar.ihlaller(s).is_empty())
    {
        println!("  #{} {:?}", s.no, kurallar.ihlaller(s));
    }
    // Beklenen çıktı:
    //   en_az(200) → [1, 4]
    //   indirim_sec → [225, 0, 250]
    //   kdv(250) = 300
    //     #2 ["teslim edilmemiş"]
    //     #4 ["limit aşımı"]
    //     #6 ["teslim edilmemiş"]
}

// ============================================================================
// 3. GERÇEK HAYATTAN ÖRNEK — sipariş kayıtları üzerinde veri işleme hattı
// ============================================================================
// SADE ANLATIM: Şubede siparişler banttan akar ve sırayla istasyonlardan geçer: okunamayan
// etiketler ayrılır, yalnız teslim edilenler bırakılır, her paketin yerine tutarı konur, sonda
// tutarlar toplanır. Her istasyon tek bir küçük iş yapar; zinciri yukarıdan aşağıya okuyunca ne
// olduğunu görürsün. Neden böyle? Aynı işi tek bir döngüde yazınca süzme, dönüştürme ve toplama
// iç içe geçer; hangi satırın ne için olduğu zor seçilir.
//   Mini örnek:
//     siparisler.iter()
//         .filter(|s| s.teslim)    // istasyon 1: yalnız teslim edilenler geçer
//         .map(|s| s.tutar)        // istasyon 2: siparişin yerine tutarı konur
//         .sum::<u32>()            // bant sonu: tutarlar toplanır → 720
const HAM_KAYITLAR: &str = "1;Ayşe;İzmir;250;evet\n2;Can;Ankara;90;hayır\nbozuk satır\n\
                            3;Ayşe;İzmir;40;evet\n4;Deniz;Bursa;310;evet\n\
                            5;Can;Ankara;120;evet\n6;Ece;İzmir;75;hayır";

// (Sade: aşağıdaki fonksiyon tek bir satırı siparişe çevirir. Satır bozuksa hata vermez, None
// döner; `filter_map` de None'ları banttan atar. Böylece "bozuk satır" sessizce elenir.)
// Adım adım: `?` burada Option üzerinde çalışır (Böl. 9) — eksik alan ya da sayı olmayan değer
// None döndürür. filter_map = "dönüştür + None'ları at": map ile filter'ın tek geçişte birleşimi.
fn satirdan(satir: &'static str) -> Option<Siparis> {
    let mut alanlar = satir.split(';');
    Some(Siparis {
        no: alanlar.next()?.parse().ok()?,
        musteri: alanlar.next()?,
        sehir: alanlar.next()?,
        tutar: alanlar.next()?.parse().ok()?,
        teslim: alanlar.next()? == "evet",
    })
}
fn siparisleri_oku(ham: &'static str) -> Vec<Siparis> {
    ham.lines().filter_map(satirdan).collect()
}

// (Sade: aşağıdaki fonksiyon paketleri şehir kutularına ayırır ve her kutunun üstüne toplam
// tutarı yazar. `entry(sehir).or_insert(0)`: "bu şehrin kutusu yoksa içine 0 yazıp aç, sonra ver.")
// Gruplama: iterator + HashMap::entry. HashMap'in gezinme sırası RASTGELEDİR (her çalıştırmada
// değişebilir); çıktı/test için mutlaka sırala ya da BTreeMap kullan.
fn sehir_cirolari(siparisler: &[Siparis]) -> Vec<(&'static str, u32)> {
    let mut gruplar: HashMap<&'static str, u32> = HashMap::new();
    for s in siparisler {
        *gruplar.entry(s.sehir).or_insert(0) += s.tutar;
    }
    let mut sirali: Vec<_> = gruplar.into_iter().collect();
    // Büyükten küçüğe; eşit cirolarda şehir adına göre — yoksa eşitler HashMap sırasıyla gelirdi.
    sirali.sort_by_key(|&(sehir, ciro)| (std::cmp::Reverse(ciro), sehir));
    sirali
}

fn veri_hatti() {
    println!("-- 3. veri hattı --");
    let siparisler = siparisleri_oku(HAM_KAYITLAR);
    println!(
        "okunan: {} (bozuk satır filter_map ile elendi)",
        siparisler.len()
    );

    let teslim_cirosu: u32 = siparisler
        .iter()
        .filter(|s| s.teslim)
        .map(|s| s.tutar)
        .sum();
    // (Sade: fold, elinde bir hesap pusulası tutup her pakette onu güncellemektir. Burada pusulada
    // iki sayı var: (adet, toplam). (0, 0) ile başlar; her siparişte adet 1, toplam da tutar
    // kadar artar.)
    // fold: sum/count/max'ın genel hâli — "birikeç + her öğe → yeni birikeç". Tek geçişte iki sonuç.
    let (adet, toplam) = siparisler
        .iter()
        .fold((0, 0), |(a, t), s| (a + 1, t + s.tutar));
    println!(
        "teslim cirosu = {teslim_cirosu}, ortalama = {}",
        toplam.checked_div(adet).unwrap_or(0) // boş listede `/` sıfıra bölme paniği olurdu
    );

    println!("şehir ciroları = {:?}", sehir_cirolari(&siparisler));

    let en_kucuk = siparisler.iter().min_by_key(|s| s.tutar).map(|s| s.no);
    // f64 `Ord` DEĞİLDİR (NaN yüzünden) → max() yok; max_by + total_cmp ile sırayı sen verirsin.
    let en_iyi_puan = [4.5, 3.8, 4.9]
        .into_iter()
        .max_by(|a: &f64, b| a.total_cmp(b));
    println!("en küçük sipariş = {en_kucuk:?}, en iyi puan = {en_iyi_puan:?}");

    // (Sade: partition bandın sonuna iki sepet koyar: şarta uyanlar birine, uymayanlar ötekine.
    // Aşağıdaki flat_map ise her paketi açıp içindekileri tek tek banda dizer.)
    // partition: tek geçişte ikiye ayır (filter'ı iki kez çalıştırmaktan ucuz).
    let (teslim, bekleyen): (Vec<&Siparis>, Vec<&Siparis>) =
        siparisler.iter().partition(|s| s.teslim);
    println!("teslim/bekleyen = {}/{}", teslim.len(), bekleyen.len());

    // flat_map: her öğe BİR DİZİ üretir, sonuç düzleştirilir. BTreeSet'e collect = tekil + sıralı.
    let harfler: BTreeSet<char> = siparisler
        .iter()
        .flat_map(|s| s.musteri.chars().take(1))
        .collect();
    println!("müşteri baş harfleri = {harfler:?}");
    // Beklenen çıktı:
    //   okunan: 6 (bozuk satır filter_map ile elendi)
    //   teslim cirosu = 720, ortalama = 147
    //   şehir ciroları = [("İzmir", 365), ("Bursa", 310), ("Ankara", 210)]
    //   en küçük sipariş = Some(3), en iyi puan = Some(4.9)
    //   teslim/bekleyen = 4/2
    //   müşteri baş harfleri = {'A', 'C', 'D', 'E'}
}

// ============================================================================
// 4. KONUMA ve KOMŞUYA BAKAN ADAPTÖRLER — enumerate, zip, take_while, windows, chunks, peekable
// ============================================================================
// SADE ANLATIM: Bazı sorular tek pakete bakarak cevaplanmaz: "Bu paket kaçıncı sırada?", "Bir
// öncekinden ağır mı?", "Sıradaki de aynı adrese mi gidiyor?" Bunlar için paketin sırasına ya da
// komşusuna bakan istasyonlar vardır. `peekable` ise bantta sıradaki pakete, onu almadan göz
// atmanı sağlar. Neden gerekli? Sıradan bantta bakmak için paketi alman gerekir; aldığın paketi
// geri koyamazsın, grubun sınırındaki paket kaybolur.
//   Mini örnek:
//     [10, 20, 30].iter().enumerate()        // (0, 10), (1, 20), (2, 30): sıra numarası eklendi
//     [10, 20, 30].iter().zip(["a", "b"])    // (10, "a"), (20, "b"): kısa olan bitince durur
//     [10, 20, 30].windows(2)                // [10, 20], [20, 30]: örtüşen komşu çiftler
//     [10, 20, 30].chunks(2)                 // [10, 20], [30]: örtüşmeyen ikişerli parçalar
// Ardışık aynı öğeleri (öğe, adet) çiftlerine sıkıştırır. peekable, bir sonraki öğeye TÜKETMEDEN
// bakmayı sağlar; next_if_eq "sıradaki buna eşitse al" demektir. Sıradan iterator'da bakmak =
// tüketmek olduğu için grup sınırındaki öğe kaybolurdu.
fn ardisik_grupla<'a>(ogeler: &[&'a str]) -> Vec<(&'a str, usize)> {
    let mut akis = ogeler.iter().copied().peekable();
    let mut gruplar = Vec::new();
    while let Some(oge) = akis.next() {
        let mut adet = 1;
        while akis.next_if_eq(&oge).is_some() {
            adet += 1;
        }
        gruplar.push((oge, adet));
    }
    gruplar
}

fn dilim_ve_akis_adaptorleri() {
    println!("-- 4. konum ve komşu adaptörleri --");
    let gunluk_ciro = [120, 90, 150, 150, 80, 200];
    // enumerate: (sıra, öğe). zip: iki akışı fermuar gibi eşler, KISA olan bitince durur.
    let hedefler = [100, 100, 160];
    let tutan: Vec<usize> = gunluk_ciro
        .iter()
        .zip(hedefler)
        .enumerate()
        .filter(|(_, (ciro, hedef))| *ciro >= hedef)
        .map(|(gun, _)| gun + 1)
        .collect();
    println!("hedefi tutan günler = {tutan:?}");
    // take_while: koşul İLK bozulduğunda durur (filter gibi sonuna kadar taramaz).
    let ilk_seri: Vec<_> = gunluk_ciro.iter().take_while(|c| **c >= 90).collect();
    println!("90 altına düşene kadar = {ilk_seri:?}");
    // windows(2): örtüşen komşu çiftler → günden güne değişim. chunks(4): örtüşmeyen paketler.
    let artis_gunleri = gunluk_ciro.windows(2).filter(|w| w[1] > w[0]).count();
    let paket_toplamlari: Vec<i32> = gunluk_ciro.chunks(4).map(|p| p.iter().sum()).collect();
    println!("artış olan gün = {artis_gunleri}, 4'lü paket toplamları = {paket_toplamlari:?}");
    let gunluk = ["BİLGİ", "BİLGİ", "HATA", "HATA", "HATA", "BİLGİ"];
    println!("ardışık gruplar = {:?}", ardisik_grupla(&gunluk));
    // Beklenen çıktı:
    //   hedefi tutan günler = [1]
    //   90 altına düşene kadar = [120, 90, 150, 150]
    //   artış olan gün = 2, 4'lü paket toplamları = [510, 280]
    //   ardışık gruplar = [("BİLGİ", 2), ("HATA", 3), ("BİLGİ", 1)]
}

// ============================================================================
// 5. KENDİ ITERATOR'UN — sonsuz bir dizi, tek `next` ile tüm adaptörler
// ============================================================================
// main.rs'teki Sayac 5'te biter. Fibonacci'nin doğal bir sonu yok: iterator TEMBEL olduğu için
// sonsuz bir diziyi temsil etmek sorun değildir — ne kadarını istediğini tüketici (take,
// take_while, find) belirler. Burada tek bitiş koşulu u64 taşması: checked_add None verince
// `?` next'ten None döndürür ve akış kendiliğinden kapanır (panik/sessiz taşma yok). Bu yalın
// sürüm bir sonraki toplam taşacağı anda durur; o an elde tutulan son iki terimi üretmez.
// SADE ANLATIM: Sıra numarası makinesinde bütün numaraların listesi durmaz; makine yalnız "şu an
// kaçtayım" bilgisini tutar ve düğmeye basıldıkça bir sonrakini üretir. Bu yüzden sonu olmayan bir
// diziyi de verebilir: ne kadar basarsan o kadar üretir. Tembellik burada işe yarar; "ilk 10'u
// ver" dersen makine 10 kez çalışır ve durur. Neden böyle? Sonsuz bir diziyi önceden hesaplayıp
// bir listeye koymak mümkün değildir; istendikçe üretmek ise bedavadır.
//   Mini örnek:
//     Fibonacci::yeni().take(5).collect::<Vec<_>>()    // ✓ [0, 1, 1, 2, 3]: beş kez basıldı
//     Fibonacci::yeni().find(|&x| x > 50)              // ✓ Some(55): bulunca durdu
//     Fibonacci::yeni().count()                        // 92: bu sürüm u64 taşmadan önce biter
struct Fibonacci {
    simdiki: u64,
    sonraki: u64,
}
impl Fibonacci {
    fn yeni() -> Self {
        Fibonacci {
            simdiki: 0,
            sonraki: 1,
        }
    }
}
// (Sade: aşağıdaki `next` üç iş yapar: bir sonraki toplamı hesaplar (taşacaksa None döner ve
// sıra biter), şimdiki sayıyı dışarı vermek üzere eline alır, iki alanı bir adım ilerletir.
// `mem::replace`, bir alana yeni değeri koyup eskisini sana vermek demek.)
impl Iterator for Fibonacci {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        let yeni_sonraki = self.simdiki.checked_add(self.sonraki)?;
        let cikan = std::mem::replace(&mut self.simdiki, self.sonraki);
        self.sonraki = yeni_sonraki;
        Some(cikan)
    }
}

fn sonsuz_iterator() {
    println!("-- 5. kendi iterator'un: Fibonacci --");
    let ilk_on: Vec<u64> = Fibonacci::yeni().take(10).collect();
    let cift_toplam: u64 = Fibonacci::yeni()
        .take_while(|&x| x < 100)
        .filter(|x| x % 2 == 0)
        .sum();
    let bini_asan = Fibonacci::yeni().enumerate().find(|&(_, x)| x > 1000);
    println!("ilk 10 = {ilk_on:?}");
    println!("100'den küçük çiftlerin toplamı = {cift_toplam}, 1000'i aşan ilk = {bini_asan:?}");
    println!(
        "taşmadan önce üretilen terim sayısı = {}",
        Fibonacci::yeni().count()
    );
    // Beklenen çıktı:
    //   ilk 10 = [0, 1, 1, 2, 3, 5, 8, 13, 21, 34]
    //   100'den küçük çiftlerin toplamı = 44, 1000'i aşan ilk = Some((17, 1597))
    //   taşmadan önce üretilen terim sayısı = 92
}

// ============================================================================
// 6. SAHİPLİK, TEMBELLİK TUZAĞI ve `collect`'in TÜR ÇIKARIMI
// ============================================================================
// SADE ANLATIM: `collect` bandın sonundaki kaptır. Ama hangi kap? Koli mi (Vec), tekrarsız ve
// sıralı bir raf mı (BTreeSet), tek parça uzun bir etiket mi (String)? Bant bunu kendisi bilemez;
// sen söylersin: ya değişkenin türünü yazarak ya da `collect::<...>()` biçiminde. Aşağıdaki
// fonksiyon ayrıca main.rs'teki iki konuyu yeniden gösterir: `into_iter` paketleri alıp götürür
// ve istasyon dizmek bandı çalıştırmaz. Neden tür istenir? Aynı elemanlardan birçok farklı kap
// kurulabilir; derleyici senin yerine tahmin yürütmez.
//   Mini örnek:
//     let a: Vec<u32> = (1..=3).collect();                 // ✓ kap: Vec → [1, 2, 3]
//     let b = (1..=3).collect::<Vec<u32>>();               // ✓ aynısı; türü çağrıda söyledik
//     let c: String = ["r", "s"].into_iter().collect();    // ✓ kap: String → "rs"
//     let d = (1..=3).collect();                           // ✗ derlenmez: hangi kap?
fn sahiplik_tembellik_collect() {
    println!("-- 6. sahiplik, tembellik, collect --");
    // `for x in &v` = v.iter(), `for x in &mut v` = v.iter_mut(), `for x in v` = v.into_iter().
    // for döngüsü IntoIterator::into_iter çağırır; hangi impl'in seçileceğini `v`'nin yazılışı
    // belirler. Sık şaşırtan ayrıntı: iter() &T üretir, filter'ın closure'ı ona bir & daha ekler.
    let adlar = vec![String::from("ece"), String::from("can")];
    let uzunlar = adlar.iter().filter(|ad: &&String| ad.len() >= 3).count();
    // into_iter String'lerin SAHİPLİĞİNİ verir: klonlamadan yeni Vec'e taşınırlar. Bundan sonra
    // `adlar` kullanılamaz (✗ E0382: borrow of moved value: `adlar`).
    let buyuk: Vec<String> = adlar.into_iter().map(|ad| ad.to_uppercase()).collect();
    println!("uzunlar = {uzunlar}, büyük = {buyuk:?}");

    // TEMBELLİK: zincir KURULDUĞUNDA hiçbir closure çalışmaz. Aşağıda "kuruldu" satırı önce
    // basılır; take(2) yüzünden de 3. ve 4. öğe HİÇ işlenmez (gereksiz iş yapılmaz).
    let tembel = [1, 2, 3, 4]
        .iter()
        .inspect(|x| println!("  işleniyor: {x}"))
        .map(|x| x * 10);
    println!("zincir kuruldu (henüz işlenen yok)");
    let ilk_iki: Vec<i32> = tembel.take(2).collect();
    println!("ilk_iki = {ilk_iki:?}");
    // Tuzak: `v.iter().map(|x| println!("{x}"));` — tüketici yok, HİÇBİR ŞEY basılmaz. Derleyici
    // uyarır: "unused `Map` that must be used" / "iterators are lazy and do nothing unless
    // consumed". Yan etki için map değil `for` ya da `for_each` kullan.

    // collect, FromIterator uygulayan HER türe toplayabilir; hangisine toplayacağını yalnız
    // hedef türden öğrenir. Tür yoksa: ✗ `let x = v.iter().map(|x| x * 2).collect();`
    //   E0283: type annotations needed.   Üç eşdeğer çözüm:
    let kareler: Vec<u32> = (1..=4).map(|x| x * x).collect(); // (a) değişkene tür
    let metin = (1..=4).map(|x| x.to_string()).collect::<Vec<_>>().join("-"); // (b) turbofish
    let harfler: String = ["r", "u", "s", "t"].into_iter().collect(); // (c) başka hedef: String
    println!("kareler = {kareler:?}, metin = {metin}, harfler = {harfler}");
    // Beklenen çıktı:
    //   uzunlar = 2, büyük = ["ECE", "CAN"]
    //   zincir kuruldu (henüz işlenen yok)
    //     işleniyor: 1
    //     işleniyor: 2
    //   ilk_iki = [10, 20]
    //   kareler = [1, 4, 9, 16], metin = 1-2-3-4, harfler = rust
}

// ============================================================================
// 7. SIK YAPILAN HATALAR — yanlış kullanım / doğru kullanım (kodlar rustc ile doğrulandı)
// ============================================================================
// SADE ANLATIM (aşağıdaki bölüm için): Buradan sonrası en sık görülen beş hatadır. Beşinin de
// özü tektir: bir not bir şeyi yakaladıysa onu elinde TUTUYOR demektir. Tek kullanımlık notu iki
// kez uygulayamazsın (a); değiştiren notu "yalnız okur" diye veremezsin (b); not bir listeye
// yazarken sen o listeyi okuyamazsın (c); iki not aynı listeye aynı anda yazamaz (d); not,
// silinecek bir değişkene bakarak fonksiyondan çıkamaz (e). Her maddede bozuk hâl yorumda (✗),
// düzeltilmiş hâl çalışan kodda (✓) durur. Hata numaraları ilk okuyuşta önemli değildir.
fn sik_hatalar_dogru_kullanim() {
    println!("-- 7. sık hatalar: doğru kullanım --");
    // (a) FnOnce closure'ı iki kez çağırmak
    //   ✗ let etiket = String::from("x"); let ver = move || etiket; ver(); ver();
    //     E0382: use of moved value: `ver`   (ilk çağrı closure'ın KENDİSİNİ tüketti)
    //   ✓ Değeri dışarı taşıma; kopyasını ya da referansını ver → closure Fn olur.
    let etiket = String::from("acil");
    let ver = || etiket.clone();
    println!("(a) {} {}", ver(), ver());

    // (b) FnMut closure'ı `Fn` bekleyen yere vermek
    //   ✗ let mut n = 0; let artir = || n += 1; uygula_fn(artir);   // uygula_fn<F: Fn()>
    //     E0525: expected a closure that implements the `Fn` trait, but this closure only
    //            implements `FnMut`
    //   ✓ Alan taraf gerçekten paylaşımlı çağırmak zorunda değilse imzayı FnMut'a GEVŞET.
    //     API yazarken kural: işini gören EN GEVŞEK trait'i iste (FnOnce > FnMut > Fn).
    let mut n = 0;
    uc_kez_cagir(|| {
        n += 10;
        n
    });
    println!("(b) n = {n}");

    // (c) Closure değişkeni &mut yakalamışken dışarıdan okumak
    //   ✗ let mut ekle = |x| v.push(x); ekle(4); println!("{}", v.len()); ekle(5);
    //     E0502: cannot borrow `v` as immutable because it is also borrowed as mutable
    //   ✓ Ödünç, closure'ın SON kullanımında biter (NLL): önce işini bitir, sonra oku.
    let mut v = vec![1, 2, 3];
    let mut ekle = |x| v.push(x);
    ekle(4);
    ekle(5);
    println!("(c) uzunluk = {}", v.len());

    // (d) Aynı değişkeni değiştiren iki closure'ı aynı anda canlı tutmak
    //   ✗ let mut a = || v.push(1); let mut b = || v.push(2); a(); b();
    //     E0499: cannot borrow `v` as mutable more than once at a time
    //   ✓ Değişkeni yakalama, PARAMETRE olarak al: closure hiçbir şey ödünç tutmaz.
    let basa_ekle = |hedef: &mut Vec<i32>| hedef.insert(0, 0);
    let sona_ekle = |hedef: &mut Vec<i32>| hedef.push(6);
    basa_ekle(&mut v);
    sona_ekle(&mut v);
    println!("(d) v = {v:?}");

    // (e) Yereli ödünç alan closure'ı döndürmek
    //   ✗ fn esik_ustu() -> impl Fn(u32) -> bool { let esik = 10; |x| x > esik }
    //     E0373: closure may outlive the current function, but it borrows `esik`
    //   ✓ `move` ile sahipliği closure'a ver — yukarıdaki en_az tam olarak bunu yapıyor.
    println!(
        "(e) en_az(100)(tutar=250) = {}",
        en_az(100)(&siparisleri_oku(HAM_KAYITLAR)[0])
    );
    // Beklenen çıktı:
    //   (a) acil acil
    //   (b) n = 30
    //   (c) uzunluk = 5
    //   (d) v = [0, 1, 2, 3, 4, 5, 6]
    //   (e) en_az(100)(tutar=250) = true
}

// SADE ANLATIM (aşağıdaki bölüm için): Aşağısı bu dosyanın dört maddelik özeti, kendi kendine
// deneyeceğin alıştırmalar ve testlerdir. Bir madde yabancı gelirse ilgili bölümün sade kutusuna
// dön. Alıştırmalar zor gelirse ilk okuyuşta atlayabilirsin.
// ÖNEMLİ NOKTALAR
//   • Trait'i GÖVDE belirler, `move` yalnız yakalama biçimini. `move` + okuma = hâlâ Fn.
//   • Parametre alırken en gevşek trait'i iste; döndürürken tek tür → `impl Fn`, çok tür → Box<dyn Fn>.
//   • Adaptör tembeldir, tüketici çalıştırır; yan etki için `for`/`for_each`.
//   • HashMap sırası rastgeledir; gösterilecek/sınanacak sonucu sırala.
//
// ============================================================================
// ALIŞTIRMALAR (çözümler verilmedi; `cargo test -p closures-iterators` ile kendin sına)
// ============================================================================
// 1) `musteri_ozeti(&[Siparis]) -> Vec<(&'static str, u32, u32)>` yaz: her müşteri için (ad, sipariş
//    adedi, toplam tutar), ada göre sıralı.
//    İpucu: `entry(..).or_insert((0, 0))` ile tek geçiş; sonra `sort` (tuple'lar alan sırasıyla kıyaslanır).
//    Beklenen: [("Ayşe", 2, 290), ("Can", 2, 210), ("Deniz", 1, 310), ("Ece", 1, 75)]
// 2) `Sayfalayici<'a, T>` adında, bir dilimi `boyut` uzunluğunda sayfalara bölen kendi iterator'ını
//    yaz (`type Item = &'a [T]`); `chunks` KULLANMA.
//    İpucu: alanlar `kalan: &'a [T]` ve `boyut`; `split_at(boyut.min(kalan.len()))` ile ön parçayı ayır
//    (`split_at` uzunluğu aşan indekste panikler), `kalan`'ı güncelle; `boyut == 0` durumunu da düşün.
//    Beklenen: 6 siparişte boyut 4 için uzunlukları [4, 2] olan iki sayfa; boş dilimde hiç sayfa.
// 3) `KuralSeti`'ni `Box<dyn FnMut(&Siparis) -> bool>` tutacak biçimde değiştir ve "aynı müşteriden
//    ikinci sipariş" kuralı ekle (closure içinde görülen müşterileri bir HashSet'te biriktir).
//    İpucu: `ihlaller` artık `&mut self` almak zorunda kalacak — nedenini §1 ile açıkla. Buna bağlı
//    olarak `let mut kurallar` gerekir ve §2'deki döngü hem `filter` içinde hem gövdede `ihlaller`'i
//    çağırdığı için derlenmez (E0501) ve durumlu kuralı iki kez tetiklerdi; döngüyü sipariş başına
//    TEK çağrıya indir: `for s in &siparisler { let ihlal = kurallar.ihlaller(s); ... }`.
//    Beklenen: HAM_KAYITLAR için bu kural yalnız #3 ve #5'te ihlal verir.
//
// BİR SONRAKİ KONUYLA BAĞLANTISI
//   • smart-pointers (Böl. 15): `Box<dyn Fn>` bir Box'tır; bir closure'ı birden çok sahibe vermek
//     için `Rc<dyn Fn>`, yakalanan durumu paylaşarak değiştirmek için `Rc<RefCell<..>>` gerekir.
//   • concurrency (Böl. 16): `thread::spawn(move || ..)` imzası `F: FnOnce() -> T + Send + 'static`
//     ister — §1'deki `move` ve §7-e'deki E0373 orada her satırda karşına çıkar.

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn indirim_sinir_durumlarda_panik_etmez() {
        assert_eq!(indirim_sec("yuzde:150")(250), 0); // %100'e kırpılır
        assert_eq!(indirim_sec("yuzde:abc")(250), 250); // bozuk kod = indirim yok
        assert_eq!(indirim_sec("yuzde:50")(u32::MAX), u32::MAX - u32::MAX / 2); // çarpma taşmaz
    }

    #[test]
    fn bozuk_satirlar_elenir_gecerliler_okunur() {
        let siparisler = siparisleri_oku(HAM_KAYITLAR);
        assert_eq!(siparisler.len(), 6);
        assert_eq!(siparisler[0].musteri, "Ayşe");
        assert_eq!(satirdan("7;Ali;Van;abc;evet"), None);
    }

    #[test]
    fn sehir_cirolari_buyukten_kucuge_siralidir() {
        let cirolar = sehir_cirolari(&siparisleri_oku(HAM_KAYITLAR));
        assert_eq!(
            cirolar,
            vec![("İzmir", 365), ("Bursa", 310), ("Ankara", 210)]
        );
    }

    #[test]
    fn indirim_kodlari_farkli_closure_dondurur() {
        assert_eq!(indirim_sec("yuzde:20")(200), 160);
        assert_eq!(indirim_sec("sabit:50")(30), 0);
        assert_eq!(indirim_sec("bilinmeyen")(30), 30);
    }

    #[test]
    fn kural_seti_ihlalleri_adlariyla_dondurur() {
        let kurallar = KuralSeti {
            kurallar: Vec::new(),
        }
        .ekle("teslim", |s| s.teslim)
        .ekle("limit", |s| s.tutar <= 100);
        let siparisler = siparisleri_oku(HAM_KAYITLAR);
        assert_eq!(kurallar.ihlaller(&siparisler[1]), vec!["teslim"]);
        assert_eq!(kurallar.ihlaller(&siparisler[2]), Vec::<&str>::new());
        assert_eq!(kurallar.ihlaller(&siparisler[3]), vec!["limit"]);
    }

    #[test]
    fn ardisik_gruplama_sinirdaki_ogeyi_kaybetmez() {
        assert_eq!(
            ardisik_grupla(&["a", "a", "b", "a"]),
            vec![("a", 2), ("b", 1), ("a", 1)]
        );
        assert!(ardisik_grupla(&[]).is_empty());
    }

    #[test]
    fn fibonacci_tasmada_panik_etmeden_biter() {
        assert_eq!(Fibonacci::yeni().nth(10), Some(55));
        // Sonsuz gibi görünen akış kendiliğinden biter; bitince de None vermeye devam eder.
        let mut akis = Fibonacci::yeni();
        assert_eq!(akis.by_ref().count(), 92);
        assert_eq!(akis.next(), None);
    }
}
