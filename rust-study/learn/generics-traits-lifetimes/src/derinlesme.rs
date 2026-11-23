// Derinleşme — Bölüm 10: Generics, Trait Bounds, Lifetimes (pekiştirme: gerçekçi senaryo, sık hatalar, alıştırmalar)
// Çalıştır: cargo run -p generics-traits-lifetimes   |   Test: cargo test -p generics-traits-lifetimes
//
// main.rs üç kavramı tanıttı ve derleyicinin onları nasıl uyguladığını (monomorphization,
// elision'ın 3 kuralı) anlattı. Bu dosya "imzayı ben yazarken neye dikkat etmeliyim?" sorusuna
// odaklanır:
//   - lifetime etiketini GEREĞİNDEN SIKI yazmanın çağıranı nasıl kısıtladığı,
//   - referans tutan bir struct'ta `&'a str` ile `&self`'e bağlı dönüşün farkı (sıfır kopyalı
//     ayrıştırıcı örneği),
//   - `'static` ve `T: 'static`'in gerçekte ne dediği (en çok yanlış anlaşılan nokta),
//   - generic bir veri yapısında bound'u NEREYE koymalı; associated type mi generic parametre mi.
// HRTB (`for<'a>`), GAT ve lifetime alt tiplemesi burada yok → learn/advanced-concepts/ileri-lifetime-hrtb-gat.
// ✗ örneklerin hata kodları rustc 1.99 ile derlenerek doğrulandı.

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Önce main.rs'i oku; orada generic, trait ve lifetime'ın ne olduğu anlatıldı. Bu dosya bir adım
// sonrasını sorar: "Bir fonksiyonun ilk satırını (imzasını) BEN yazarken neye dikkat etmeliyim?"
// Benzetmelerde yine okuldayız; çoğu okul kütüphanesinde geçer.
//
// Bu dosyanın tek cümlelik özeti: İmzaya yazdığın her etiket ve her şart, seni çağırana verilmiş
// bir sözdür; gereğinden sıkı yazarsan kod derlenir ama çağıranın eli kolu bağlanır.
//
// MİNİ SÖZLÜK (yalnız bu dosyada yeni geçenler; öteki terimler main.rs'in başındaki sözlükte):
//   ödünç (borrow)           → veriye sahibini değiştirmeden bakmak ya da onu kullanmak.
//   ödünç süresi             → o bakışın sürdüğü kod bölgesi; bitince veri yeniden serbesttir.
//   alıcı (&self, &mut self) → metodun üzerinde çağrıldığı değer: `ayr.sonraki()`daki `ayr`.
//   `'_`                     → "buradaki lifetime'ı sen doldur" demek için yazılan adsız etiket.
//   sahipli tür (owned)      → verisini kendi taşıyan tür: String, Vec. Kimseye borçlu değildir.
//   sıfır kopya (zero-copy)  → veriyi çoğaltmadan, aslının içine bakan dilimlerle çalışmak.
//   tahsis (allocation)      → heap'ten yer istemek; her String::from bir tahsistir.
//   optimizasyon             → aynı işi daha hızlı ya da daha az bellekle yapma çabası.
//   Iterator, Item           → elemanları sırayla veren şey / verdiği her elemanın türü.
//   associated type          → trait'in içindeki tür boşluğu; trait'i uygulayan tür doldurur.
//   HashMap                  → anahtar → değer eşleştiren sözlük; sırası belli değildir.
//   Eq + Hash                → HashMap anahtarının şartı: eşitliği sınanır, özeti çıkarılır.
//   Entry                    → HashMap'te bir anahtarın yuvası: ya doludur ya boş.
//   önbellek (cache)         → hesaplanan sonucun, yeniden hesaplanmasın diye saklandığı yer.
//   isabet / ıska            → aranan sonucun önbellekte bulunması / bulunmaması.
//   closure, FnOnce          → `|x| ...` biçimli adsız fonksiyon / en fazla bir kez çağrılabileni.
//   turbofish (::<T>)        → türü çağrı sırasında elle söylemek: `parse::<u32>()`.
//   deyimsel (idiomatic)     → o dilde alışılmış, önerilen yazım biçimi.
//   polimorfizm              → aynı çağrının farklı türlerde farklı iş yapabilmesi.
//   HRTB, GAT, alt tipleme   → lifetime'ın ileri konuları; bu dosyada anlatılmaz.

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt::Display;
use std::hash::Hash;

pub fn calistir() {
    println!("\n== Derinleşme: generics, trait bound, lifetime pratikte ==");
    lifetime_bir_iliskidir();
    sifir_kopyali_ayristirici();
    static_ne_demek();
    generic_onbellek();
    associated_type_mi_generic_mi();
}

// ============================================================================
// 1. NASIL ÇALIŞIR? — Lifetime bir süre değil, bir İLİŞKİ bildirimidir
// ============================================================================
// `'a` yazmak hiçbir şeyin ömrünü UZATMAZ ya da kısaltmaz. Yalnızca derleyiciye şunu söylersin:
// "dönen referans, ŞU girdiden türetildi; onun kadar geçerli say." Derleyici bu sözü iki yerde
// denetler: gövdede (gerçekten o girdiden mi dönüyorsun?) ve çağrı yerinde (çağıran, sonucu o
// girdi ölmeden mi kullanıyor?).
//
// Bu yüzden imzadaki etiketler bir TASARIM KARARIDIR: hangi girdilerin çıktıyla bağlı olduğunu
// ne kadar doğru söylersen çağıran o kadar özgür olur.

// SADE ANLATIM: Kütüphaneden bir kitap, arkadaşından da bir cetvel ödünç aldın. Cetveli yalnız
// kitaptaki bir satırın altını çizmek için kullandın; işaretlediğin satır kitaba aittir, cetvelle
// bir ilgisi kalmamıştır. Ödünç fişine "bu satır hem kitaba hem cetvele bağlı" yazarsan, cetveli
// geri verdiğin an satırı da kullanamazsın. Lifetime etiketi bu fiştir: süreyi değiştirmez, yalnız
// "sonuç hangi girdiye bağlı" bilgisini yazar. Neden önemli? Derleyici gövdeye değil fişe
// (imzaya) güvenir; fişe fazladan yazdığın her bağ, çağıranı gereksiz yere kısıtlar.
//   Mini örnek:
//     fn siki<'a>(metin: &'a str, on_ek: &'a str) -> &'a str    // fiş: "ikisine de bağlı"
//     fn dogru<'a>(metin: &'a str, on_ek: &str) -> &'a str      // fiş: "yalnız metin'e bağlı"
//     // on_ek, sonuçtan önce silinirse:
//     //   dogru(&yol, &on_ek) sonucu   ✓ kullanılmaya devam eder
//     //   siki(&yol, &on_ek) sonucu    ✗ derlenmez: on_ek'e de bağlı sayılır
// ✗ Gereğinden sıkı: iki parametre de 'a. Sonuç yalnız `metin`den gelse bile imza "ikisinden
//   de gelebilir" der; derleyici imzaya güvenir, gövdeye bakmaz.
fn on_eki_at_siki<'a>(metin: &'a str, on_ek: &'a str) -> &'a str {
    metin.strip_prefix(on_ek).unwrap_or(metin)
}

// ✓ Doğru: `on_ek` yalnız karşılaştırmada kullanılıyor; çıktıyla bağı yok. Ona ayrı (burada
//   elision'ın verdiği anonim) bir lifetime bırakıyoruz.
fn on_eki_at<'a>(metin: &'a str, on_ek: &str) -> &'a str {
    metin.strip_prefix(on_ek).unwrap_or(metin)
}

fn lifetime_bir_iliskidir() {
    let yol = String::from("/api/kullanicilar");
    let temiz;
    {
        let on_ek = String::from("/api"); // kısa ömürlü: bu blokta ölecek
        temiz = on_eki_at(&yol, &on_ek); // ✓ sonuç yalnız `yol`a bağlı
        // temiz = on_eki_at_siki(&yol, &on_ek);
        //   ✗ E0597: `on_ek` does not live long enough — sıkı imza, sonucu on_ek'e de bağladı;
        //     on_ek blok sonunda ölünce `temiz` de (kâğıt üzerinde) geçersiz sayılır.
        println!(
            "blok içinde sıkı sürüm de çalışır: {}",
            on_eki_at_siki(&yol, &on_ek)
        );
    }
    println!("on_ek öldü, sonuç hâlâ geçerli: {temiz}"); // /kullanicilar

    // Kural: bir parametre çıktıya KATKI VERMİYORSA çıktıyla aynı lifetime'ı paylaşmasın.
    // Tersi (gövde `on_ek`i döndürmeye kalksa) da yakalanır:
    //   fn f<'a, 'b>(x: &'a str, y: &'b str) -> &'a str { if x.len() > 1 { x } else { y } }
    //   ✗ error: lifetime may not live long enough  (bu hatanın E-kodu YOKTUR; mesaj
    //     "consider adding the following bound: `'b: 'a`" önerisiyle gelir).
}

// ============================================================================
// 2. GERÇEK HAYATTAN ÖRNEK — sıfır kopyalı (zero-copy) ayar ayrıştırıcı
// ============================================================================
// Senaryo: "anahtar=deger;anahtar=deger" biçimli bir satırı ayrıştırıyoruz. Her anahtar/değer
// için yeni String tahsis etmek yerine, girdinin İÇİNE bakan dilimler döndürmek istiyoruz —
// büyük girdilerde tahsis sayısı sıfıra iner. Bedeli: sonuçlar girdiye bağlıdır, yani bir
// lifetime taşırlar.
//
// Burada İKİ ayrı ömür vardır ve karıştırmak en sık yapılan hatadır:
//   'a     → girdi metninin ömrü (uzun)
//   &mut self → ayrıştırıcının o anki ödünç süresi (çok kısa: tek bir `sonraki()` çağrısı)
// SADE ANLATIM: Bir kitaptan alıntı toplayacaksın. İki yol var: cümleleri deftere tek tek
// yeniden yazmak (kopya: yorucu) ya da kitabın sayfalarına yapışkan ayraç koymak (kopya yok:
// hızlı). Ayraçlar KİTABA bağlıdır; kitap iade edilince işe yaramazlar. Ama ayracı yapıştıran
// PARMAĞINA bağlı değildirler; parmağını çekince ayraç yerinde durur. Aşağıda girdi metni
// kitaptır, `Ayar` ayraçtır, `Ayristirici` ise sayfalarda gezen parmaktır. Neden ayırt etmeli?
// Sonuçları parmağa bağlarsan, bir ayraç elindeyken parmağını bir sonraki sayfaya götüremezsin.
//   Mini örnek:
//     let girdi = String::from("ad=Kaan;dil=tr");
//     let mut ayr = Ayristirici::yeni(&girdi);
//     let a = ayr.sonraki();     // Some(Ayar { anahtar: "ad", deger: "Kaan" })
//     let b = ayr.sonraki();     // ✓ a hâlâ elimizde: sonuçlar ayr'a değil girdi'ye bağlı
//     drop(ayr);                 // parmağı çektik
//     println!("{a:?} {b:?}");   // ✓ ayraçlar yerinde: girdi hâlâ yaşıyor
#[derive(Debug, PartialEq, Eq)]
pub struct Ayar<'a> {
    pub anahtar: &'a str,
    pub deger: &'a str,
}

pub struct Ayristirici<'a> {
    kalan: &'a str,
}

impl<'a> Ayristirici<'a> {
    pub fn yeni(girdi: &'a str) -> Self {
        Ayristirici { kalan: girdi }
    }

    // (Sade: aşağıdaki açıklama şunu söyler: dönüş türüne `'a` yazmasaydık derleyici sonucu
    // "parmağa" (`&mut self`e) bağlardı ve ilk sonuç elindeyken ikinciyi isteyemezdin.)
    // Dönüş türündeki 'a KRİTİK. Elision'a bıraksaydık (`-> Option<Ayar<'_>>`) Kural 3 devreye
    // girer ve sonucu `&mut self`in ödüncüne bağlardı: "bu Ayar yaşadıkça ayrıştırıcı
    // değiştirilebilir biçimde ödünçte". O zaman ikinci `sonraki()` çağrısı derlenmezdi:
    //     let a = ayr.sonraki();
    //     let b = ayr.sonraki();      // ✗ E0499: cannot borrow `ayr` as mutable more than once
    //     println!("{a:?} {b:?}");
    // 'a yazarak "sonuç ayrıştırıcıya değil, GİRDİ METNİNE bağlı" diyoruz; ayrıştırıcı çağrı
    // biter bitmez serbest kalır, hatta düşürülse bile sonuçlar geçerlidir.
    pub fn sonraki(&mut self) -> Option<Ayar<'a>> {
        while !self.kalan.is_empty() {
            let (parca, gerisi) = self.kalan.split_once(';').unwrap_or((self.kalan, ""));
            self.kalan = gerisi;
            if let Some((anahtar, deger)) = parca.split_once('=') {
                return Some(Ayar {
                    anahtar: anahtar.trim(),
                    deger: deger.trim(),
                });
            }
            // '=' içermeyen parça: atla ve devam et
        }
        None
    }
}

// (Sade: bir türe Iterator yazmak, onu `for` ile gezilebilir ve `collect` ile toplanabilir
// yapmak demek. Tek istenen `next`: "sıradaki elemanı ver, kalmadıysa None de.")
// Iterator uygulamak, tam olarak yukarıdaki imzayı ister: Item türü `&mut self`e bağlanamaz
// (Iterator::next'in imzası buna izin vermez). Sonuç girdiye bağlı olduğu için sorun yok —
// "ödünç veren iterator" gerektiğinde ne olacağı GAT konusudur (ileri-lifetime-hrtb-gat).
impl<'a> Iterator for Ayristirici<'a> {
    type Item = Ayar<'a>;
    fn next(&mut self) -> Option<Ayar<'a>> {
        self.sonraki()
    }
}

fn sifir_kopyali_ayristirici() {
    let girdi = String::from("ad = Kaan; bozuk; dil=tr;tema = koyu");
    let ayarlar: Vec<Ayar> = {
        let ayr = Ayristirici::yeni(&girdi);
        ayr.collect()
    }; // ayrıştırıcı burada düştü — sonuçlar `girdi`ye bağlı olduğu için hâlâ geçerli
    for a in &ayarlar {
        println!("  {} → {}", a.anahtar, a.deger);
    }
    // Beklenen çıktı:
    //   ad → Kaan
    //   dil → tr
    //   tema → koyu

    // Dilimlerin gerçekten girdinin içine baktığını (kopya olmadığını) adresle doğrulayalım:
    let girdi_araligi = girdi.as_bytes().as_ptr_range();
    let icinde = girdi_araligi.contains(&ayarlar[0].deger.as_ptr());
    println!("  değerler girdinin içine bakıyor (kopya yok): {icinde}"); // true

    // ✗ Girdi ölürse sonuçlar da kullanılamaz — derleyici bunu ilişkiden bilir:
    //     let ayarlar = { let gecici = String::from("a=1"); Ayristirici::yeni(&gecici).collect::<Vec<_>>() };
    //     // E0597: `gecici` does not live long enough
    // ✓ Sonuçların girdiden uzun yaşaması gerekiyorsa o noktada sahipli türe geç (String /
    //   to_owned). Sıfır kopya bir OPTİMİZASYONdur; ömürler karmaşıklaşıyorsa kopyalamak meşrudur.

    // (Sade: `&'a mut self` yazmak "kitap iade edilene kadar parmağımı bu sayfadan çekmeyeceğim"
    // demektir; bu yüzden ikinci çağrıya izin çıkmaz. Aşağısı bunu ve düzeltmesini anlatır.)
    // YANLIŞ KULLANIM: `&'a mut self`
    //     impl<'a> Sayac<'a> { fn artir(&'a mut self) { ... } }
    //     s.artir();
    //     s.artir();   // ✗ E0499: cannot borrow `s` as mutable more than once at a time
    // Struct'ın kendi 'a parametresini &mut self'e yazmak "beni, içimdeki verinin TÜM ömrü
    // boyunca özel olarak ödünç al" demektir; ilk çağrıdan sonra değer sonsuza dek kilitlenir.
    // DOĞRU KULLANIM: alıcıyı etiketsiz bırak (`&mut self`); derleyici her çağrıya kısa, taze
    // bir ödünç süresi verir.
}

// ============================================================================
// 3. 'static — "program boyu yaşar" DEĞİL, "geçici bir şeye borçlu değil"
// ============================================================================
// İki ayrı kullanım var ve ikincisi sürekli yanlış anlaşılır:
//
//   &'static T   → bu REFERANS program sonuna dek geçerli (string literalleri, static'ler).
//   T: 'static   → bu TÜR, 'static'ten kısa ömürlü bir referans İÇERMİYOR. Değerin kendisi
//                  her an düşürülebilir! String, Vec<u8>, i32 hepsi 'static'tir, çünkü kimseye
//                  borçlu değiller — sahip oldukları veriyle birlikte istedikleri kadar
//                  yaşayabilirler.
//
// Benzetme: `T: 'static` "bu çantada emanet eşya yok" demektir; çantanın sonsuza dek
// saklanacağı anlamına gelmez. Emanet yoksa çantayı istediğin yere, istediğin kadar
// götürebilirsin — thread::spawn ve Box<dyn Trait> tam da bunu ister.
// SADE ANLATIM: Kendi kitabın ile kütüphaneden ödünç aldığın kitabı karşılaştır. Kendi kitabını
// istediğin yere götürür, istediğin kadar tutarsın; iade günün yoktur. `T: 'static` bunu söyler:
// "bu değerin içinde, günü gelince geri verilecek ödünç bir şey yok." Kitabı sonsuza dek
// saklayacağın anlamına gelmez; yarın başkasına verebilir ya da atabilirsin. Neden gerekli? Bir
// değeri ne kadar tutacağı belli olmayan bir yere (bir kutuya, başka bir iş koluna) verirken
// içinde yakında geçersiz kalacak bir referans bulunmamalıdır.
//   Mini örnek:
//     uzun_sure_sakla(String::from("kendi kitabım"));   // ✓ String kimseye borçlu değil
//     uzun_sure_sakla(42);                              // ✓ i32 de değil
//     let yerel = String::from("x");
//     uzun_sure_sakla(&yerel);   // ✗ derlenmez: &yerel ödünçtür, yerel silinince geçersiz kalır
//     uzun_sure_sakla(yerel);    // ✓ ödüncünü değil, kendisini ver
fn uzun_sure_sakla<T: Display + 'static>(deger: T) -> Box<dyn Display> {
    // Box<dyn Display> = Box<dyn Display + 'static> (varsayılan). İçine konan türün geçici bir
    // referans taşımadığından emin olmalıyız; yoksa kutu, işaret ettiği veriden uzun yaşardı.
    Box::new(deger)
}

fn static_ne_demek() {
    let sahipli = String::from("sahipli String");
    let kutu = uzun_sure_sakla(sahipli); // ✓ String: 'static (kimseye borçlu değil)
    println!("{kutu}");
    drop(kutu); // 'static bir tür olması "hiç düşmez" demek değildir — işte düştü.

    let sayi_kutusu = uzun_sure_sakla(42); // ✓ i32: 'static
    let sabit_kutusu = uzun_sure_sakla("literal"); // ✓ &'static str
    println!("{sayi_kutusu}, {sabit_kutusu}");

    // ✗ Yerel bir değere referans 'static DEĞİLDİR:
    //     let yerel = String::from("x");
    //     uzun_sure_sakla(&yerel);   // E0597: `yerel` does not live long enough
    //                                //        ("argument requires that `yerel` is borrowed for `'static`")
    // ✗ Bound'u unutursan hata bu kez fonksiyonun İÇİNDE çıkar:
    //     fn kutula<T: Display>(t: T) -> Box<dyn Display> { Box::new(t) }
    //                                // E0310: the parameter type `T` may not live long enough
    // Sık yapılan hata: bu mesajları görünce referansı 'static yapmaya çalışmak (Box::leak,
    // global değişken). Asıl çözüm neredeyse her zaman SAHİPLİĞİ vermektir: &yerel yerine yerel
    // (ya da klonu). 'static bound'u "referans verme, değerin kendisini ver" diye oku.
}

// ============================================================================
// 4. GENERIC VERİ YAPISI — bound'u nereye koymalı?
// ============================================================================
// Senaryo: pahalı bir hesabı (veritabanı sorgusu, dosya okuma) anahtara göre önbelleğe alan
// yapı. Hangi anahtar/değer türüyle kullanılacağını bilmiyoruz → generic.
//
// Önemli nokta: bound'ları struct tanımına DEĞİL, onlara gerçekten ihtiyaç duyan impl bloğuna
// koy. Struct'a konan bound, o türün adının geçtiği HER yere (her fonksiyon imzası, her impl)
// kopyalanmak zorunda kalır; impl'e konan bound yalnız ilgili metotları kısıtlar. std de böyle
// yapar: `HashMap<K, V>` tanımında bound yoktur, `K: Eq + Hash` yalnız insert/get impl'indedir.
// SADE ANLATIM: Zor bir çarpma sorusunu çözünce cevabını defterinin arkasına yazarsın; aynı soru
// yine gelirse hesaplamaz, deftere bakarsın. Önbellek bu defterdir; sorunun (K) ve cevabın (V)
// türü önceden belli olmadığı için generic yazılır. Şartlara gelince: okula kayıt olurken kimse
// "yüzme biliyor musun?" diye sormaz; bu soru havuzun kapısında sorulur. Rust'ta da şart (bound),
// ona gerçekten ihtiyaç duyan metotların impl bloğuna yazılır, struct tanımına değil. Neden?
// Struct'a yazılan şartı, o türün adının geçtiği her imzada yeniden yazman gerekir.
//   Mini örnek:
//     struct Defter<K, V> { .. }                             // şart yok: kayıt herkese açık
//     impl<K, V> Defter<K, V> { fn yeni() -> Self { .. } }   // ✓ her K, V için var
//     impl<K: Eq + Hash, V> Defter<K, V> { fn bak(&self, k: &K) { .. } }   // şart kapıda
//     // K şartı sağlamıyorsa: Defter::yeni() ✓ çalışır, defter.bak(..) ✗ derlenmez
pub struct Onbellek<K, V> {
    kayitlar: HashMap<K, V>,
    isabet: u32,
    iska: u32,
}

// Bound gerektirmeyen metotlar: her K, V için var.
impl<K, V> Onbellek<K, V> {
    pub fn yeni() -> Self {
        Onbellek {
            kayitlar: HashMap::new(),
            isabet: 0,
            iska: 0,
        }
    }

    pub fn istatistik(&self) -> (u32, u32) {
        (self.isabet, self.iska)
    }
}

// HashMap'e erişen metotlar: K karşılaştırılabilir ve özetlenebilir olmalı.
impl<K: Eq + Hash, V> Onbellek<K, V> {
    // (Sade: aşağıdaki metot defterde soruyu arar. Cevap yazılıysa onu verir (isabet). Yazılı
    // değilse `uret`i çağırıp hesaplatır, cevabı deftere yazar ve onu verir (ıska). `Entry`,
    // defterde o soruya ayrılmış satırdır: ya dolu (Occupied) ya boş (Vacant).)
    // `uret` yalnız değer YOKSA çağrılır → FnOnce yeter (en az şey isteyen bound; closures-
    // iterators crate'i). Dönüş &V: değer önbellekte yaşar, çağırana ödünç verilir.
    //
    // Adım adım: entry() anahtarı BİR KEZ arar ve "dolu / boş" yuvayı döndürür. Klasik
    // `if let Some(v) = map.get(&k) { return v; } map.insert(k, ..)` yazımı borrow checker'a
    // takılır (get'in ödüncü dönüş yüzünden fonksiyon sonuna dek uzar); Entry API bu ihtiyacın
    // deyimsel çözümüdür ve iki arama yerine tek arama yapar.
    pub fn getir_ya_da_uret(&mut self, anahtar: K, uret: impl FnOnce(&K) -> V) -> &V {
        match self.kayitlar.entry(anahtar) {
            Entry::Occupied(dolu) => {
                self.isabet += 1;
                dolu.into_mut()
            }
            Entry::Vacant(bos) => {
                self.iska += 1;
                let deger = uret(bos.key());
                bos.insert(deger)
            }
        }
    }
}

// (Sade: havuz kapısındaki soru gibi: `dokum()` yalnız anahtarı sıralanabilen ve içeriği
// yazdırılabilen defterlerde bulunur; öteki defterler yine kullanılır, yalnız bu metot görünmez.)
// KOŞULLU metot: yalnız K sıralanabilir ve ikisi de yazdırılabilirse var. Bu bound'ları
// sağlamayan türler için Onbellek yine kullanılır, sadece `dokum()` görünmez.
impl<K: Ord + Display, V: Display> Onbellek<K, V> {
    pub fn dokum(&self) -> String {
        let mut satirlar: Vec<(&K, &V)> = self.kayitlar.iter().collect();
        satirlar.sort_by(|a, b| a.0.cmp(b.0)); // HashMap sırası belirsiz → sırala
        satirlar
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn generic_onbellek() {
    let mut onbellek: Onbellek<String, usize> = Onbellek::yeni();
    let pahali_hesap = |metin: &String| {
        println!("  (hesaplanıyor: {metin})");
        metin.chars().count()
    };
    let a = *onbellek.getir_ya_da_uret(String::from("merhaba"), pahali_hesap);
    let b = *onbellek.getir_ya_da_uret(String::from("merhaba"), pahali_hesap); // hesap YOK
    let c = *onbellek.getir_ya_da_uret(String::from("dünya"), pahali_hesap);
    println!(
        "  sonuçlar = {a}, {b}, {c}; (isabet, ıska) = {:?}",
        onbellek.istatistik()
    );
    println!("  döküm: {}", onbellek.dokum());
    // Beklenen çıktı:
    //   (hesaplanıyor: merhaba)
    //   (hesaplanıyor: dünya)
    //   sonuçlar = 7, 7, 5; (isabet, ıska) = (1, 2)
    //   döküm: dünya=5, merhaba=7

    // Aynı yapı bambaşka türlerle — tek satır kod yazmadan (monomorphization ayrı kopya üretir):
    let mut kareler: Onbellek<u32, u64> = Onbellek::yeni();
    println!(
        "  12² = {}",
        kareler.getir_ya_da_uret(12, |n| u64::from(*n) * u64::from(*n))
    ); // 144

    // SIK YAPILAN HATALAR
    //   ✗ fn buyuk<T>(a: T, b: T) -> T { if a > b { a } else { b } }
    //       E0369: binary operation `>` cannot be applied to type `T`
    //     Generic gövde, T hakkında yalnız BOUND'ların söylediğini bilir. C++ template'lerinden
    //     farkı budur: hata kullanım yerinde değil, tanım yerinde ve bir kez çıkar.
    //     ✓ `T: PartialOrd` ekle.
    //   ✗ let n = "5".parse().unwrap();
    //       E0284: type annotations needed
    //     Dönüş türü generic olan fonksiyonlarda (parse, collect, into) hedefi derleyici
    //     tahmin etmez. ✓ `let n: u32 = ...` ya da turbofish: `"5".parse::<u32>()`.
    //   ✗ struct Baslik { metin: &str }
    //       E0106: missing lifetime specifier
    //     Alanlarda elision YOKTUR; referans tutan her struct lifetime parametresi bildirir.
}

// ============================================================================
// 5. ASSOCIATED TYPE mi, GENERIC PARAMETRE mi?
// ============================================================================
// İkisi de "trait içinde bir tür değişkeni"dir; fark, o türü KİMİN seçtiğidir:
//
//   trait Ayristir { type Cikti; }    → çıktıyı UYGULAYAN tür belirler; bir tür için TEK cevap
//                                       vardır. (Iterator::Item, Deref::Target, Add::Output)
//   trait Donustur<T> { ... }         → T'yi ÇAĞIRAN seçer; aynı tür birçok T için ayrı ayrı
//                                       uygulayabilir. (From<T>: String hem From<&str> hem
//                                       From<char> uygular)
//
// Ölçüt: "Bu tür için birden fazla anlamlı cevap olabilir mi?" Hayırsa associated type —
// çağıranı her yerde tür yazmaktan kurtarır ve belirsizliği baştan kapatır.
// SADE ANLATIM: Her sınıfın TEK bir sınıf öğretmeni vardır. "3-A'nın öğretmeni kim?" sorusunun
// tek cevabı olur; cevabı soran kişi seçmez, sınıf belirler. Associated type budur. Öte yandan
// bir öğrenci hem satranç hem resim kulübüne üye olabilir; "şu kulübe üye mi?" sorusu her kulüp
// için ayrı ayrı sorulur. Generic parametreli trait de budur. Neden ayrım var? Cevap tekse onu
// her çağrıda yazdırmak gereksizdir; ayrıca derleyici "hangisini kastettin?" diye sormak zorunda
// kalmaz.
//   Mini örnek:
//     impl Ayristir for TamSayi { type Cikti = i64; .. }   // TamSayi'nın çıktısı: tek cevap
//     impl Ayristir for TamSayi { type Cikti = u8; .. }    // ✗ derlenmez: ikinci cevap olamaz
//     impl From<&str> for Ad { .. }    // ✓ generic trait: aynı tür için
//     impl From<char> for Ad { .. }    // ✓ her T ile ayrı bir impl yazılabilir
pub trait Ayristir {
    type Cikti;
    fn ayristir(&self, ham: &str) -> Option<Self::Cikti>;
}

pub struct TamSayi;
pub struct EvetHayir;

impl Ayristir for TamSayi {
    type Cikti = i64;
    fn ayristir(&self, ham: &str) -> Option<i64> {
        ham.trim().parse().ok()
    }
}

impl Ayristir for EvetHayir {
    type Cikti = bool;
    fn ayristir(&self, ham: &str) -> Option<bool> {
        match ham.trim().to_lowercase().as_str() {
            "evet" | "e" | "1" => Some(true),
            "hayır" | "h" | "0" => Some(false),
            _ => None,
        }
    }
}

// (Sade: `A::Cikti`, "A hangi ayrıştırıcıysa onun çıktı türü" diye okunur: TamSayi verirsen
// Vec<i64>, EvetHayir verirsen Vec<bool> gelir; ayrıca belirtmen gerekmez.)
// Tek generic fonksiyon, her ayrıştırıcıyla çalışır; dönüş türü `A::Cikti` ayrıştırıcıdan
// OTOMATİK gelir — çağıran `hepsini_ayristir::<TamSayi, i64>` gibi ikinci bir parametre yazmaz.
pub fn hepsini_ayristir<A: Ayristir>(a: &A, girdiler: &[&str]) -> Vec<A::Cikti> {
    girdiler.iter().filter_map(|g| a.ayristir(g)).collect()
}

fn associated_type_mi_generic_mi() {
    let sayilar = hepsini_ayristir(&TamSayi, &["10", " 20 ", "otuz", "-4"]);
    let cevaplar = hepsini_ayristir(&EvetHayir, &["Evet", "h", "belki", "1"]);
    println!("  sayılar = {sayilar:?}"); // [10, 20, -4]
    println!("  cevaplar = {cevaplar:?}"); // [true, false, true]
}

// SADE ANLATIM (aşağıdaki bölüm için): Buradan sonrası bu dosyanın özeti, kendi kendine
// deneyeceğin alıştırmalar ve testlerdir. Özet maddeleri yukarıdaki beş kutunun kısa hâlidir;
// bir madde yabancı gelirse ilgili bölümün sade kutusuna dön. Alıştırmalar zor gelirse ilk
// okuyuşta atlayabilirsin.
// ============================================================================
// ÖNEMLİ NOKTALAR
// ============================================================================
// • Lifetime etiketi yazarken sor: "Çıktı HANGİ girdiden türüyor?" Yalnız o girdi çıktıyla aynı
//   etiketi taşır. Hepsine 'a yazmak derlenir ama çağıranı gereksiz kısıtlar.
// • Referans tutan struct'ta metot dönüşleri için iki seçenek vardır: struct'ın 'a'sı (veri
//   kaynağına bağlı, uzun) ya da &self'in ömrü (struct'a bağlı, kısa). Mümkünse 'a döndür.
// • `T: 'static` "referans içermeyen ya da yalnız 'static referans içeren tür" demektir; sahipli
//   türlerin hepsi bunu sağlar.
// • Generic gövde yalnız bound'ların verdiği yetenekleri kullanabilir; bound'u struct'a değil,
//   ihtiyacı olan impl'e koy; en az şeyi isteyen bound'u seç.
// • Lifetime'lar çalışma anında YOKTUR: yukarıdaki sıfır kopyalı ayrıştırıcı, C'deki
//   pointer+uzunluk koduyla aynı makine koduna derlenir — fark, sarkan pointer'ın imkânsız olması.
//
// ============================================================================
// ALIŞTIRMALAR
// ============================================================================
// 1) `fn en_uzun_deger<'a>(ayarlar: &[Ayar<'a>]) -> Option<&'a str>` yaz: en uzun `deger`i
//    döndürsün. Dönüşün `ayarlar` dilimine değil GİRDİ METNİNE bağlı olduğunu şu testle kanıtla:
//    Vec<Ayar>'ı düşürdükten sonra sonucu yazdır.
//    İpucu: iter().map(|a| a.deger).max_by_key(|d| d.len()). Dönüşü `Option<&str>` (elision)
//    yaparsan fonksiyon neden hiç derlenmez (E0106)? Dönüşü dilimin ömrüne bağlarsan
//    (`fn f<'s, 'a>(ayarlar: &'s [Ayar<'a>]) -> Option<&'s str>`) fonksiyon derlenir ama test
//    derlenmez — neden?
//
// 2) `Onbellek`e `fn gecersiz_kil(&mut self, anahtar: &K) -> Option<V>` ekle. Hangi impl
//    bloğuna koymalısın ve neden? Ardından std'nin `HashMap::remove` imzasına bak: parametre
//    neden `&K` değil de `&Q where K: Borrow<Q>`? (İpucu: `Onbellek<String, _>`ten `&str` ile
//    silebilmek için.)
//
// 3) `Ayristir`ı `Vec<A>` için uygula: `Cikti = Vec<A::Cikti>` olsun ve girdiyi virgülden bölüp
//    i. parçayı i. ayrıştırıcıya versin; biri bile başarısızsa None dönsün.
//    İpucu: zip + `collect::<Option<Vec<_>>>()`. Beklenen: `vec![TamSayi, TamSayi]` ile
//    "3, 4" → Some([3, 4]); "3, x" → None.
//
// Bir sonraki konuyla bağlantısı: trait'ler closure'ların da temelidir (Fn/FnMut/FnOnce birer
// trait bound'dur → closures-iterators); `dyn Trait` ile çalışma anı polimorfizmi → oop;
// lifetime'ların alt tipleme, `for<'a>` ve GAT yüzü → ileri-variance ve ileri-lifetime-hrtb-gat.

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn on_ek_sonucu_yalniz_metne_baglidir() {
        let metin = String::from("ön-ek:gövde");
        let sonuc = {
            let gecici = String::from("ön-ek:");
            on_eki_at(&metin, &gecici)
        };
        assert_eq!(sonuc, "gövde");
        assert_eq!(on_eki_at_siki("abc", "x"), "abc");
    }

    #[test]
    fn ayristirici_bozuk_parcalari_atlar() {
        let ayarlar: Vec<_> = Ayristirici::yeni("a=1;;bozuk; b = 2 ;").collect();
        assert_eq!(
            ayarlar,
            [
                Ayar {
                    anahtar: "a",
                    deger: "1"
                },
                Ayar {
                    anahtar: "b",
                    deger: "2"
                }
            ]
        );
    }

    #[test]
    fn sonuclar_ayristiricidan_uzun_yasar() {
        let girdi = String::from("x=1;y=2");
        let (ilk, ikinci) = {
            let mut ayr = Ayristirici::yeni(&girdi);
            // dönüş 'a'ya bağlı olduğu için iki sonuç birlikte tutulabilir
            (ayr.sonraki(), ayr.sonraki())
        }; // ayrıştırıcı burada düştü; sonuçlar girdiye bağlı, hâlâ geçerli
        assert_eq!(ilk.unwrap().anahtar, "x");
        assert_eq!(ikinci.unwrap().deger, "2");
    }

    #[test]
    fn onbellek_ureticiyi_yalniz_iskada_cagirir() {
        let mut cagri = 0;
        let mut onbellek = Onbellek::yeni();
        for anahtar in ["a", "b", "a", "a"] {
            onbellek.getir_ya_da_uret(anahtar, |k| {
                cagri += 1;
                k.len()
            });
        }
        assert_eq!(cagri, 2);
        assert_eq!(onbellek.istatistik(), (2, 2));
        assert_eq!(onbellek.dokum(), "a=1, b=1");
    }

    #[test]
    fn associated_type_donus_turunu_belirler() {
        let sayilar: Vec<i64> = hepsini_ayristir(&TamSayi, &["1", "x", "3"]);
        let cevaplar: Vec<bool> = hepsini_ayristir(&EvetHayir, &["hayır", "evet"]);
        assert_eq!(sayilar, [1, 3]);
        assert_eq!(cevaplar, [false, true]);
    }

    #[test]
    fn static_bound_sahipli_degeri_kabul_eder() {
        let kutu = uzun_sure_sakla(String::from("ada"));
        assert_eq!(kutu.to_string(), "ada");
    }
}
