// Derinleşme — Bölüm 18: OOP / Trait Object'ler (pekiştirme: gerçekçi senaryo, sık hatalar, alıştırmalar)
// Çalıştır: cargo run -p oop   |   Test: cargo test -p oop
//
// main.rs `Vec<Box<dyn Ciz>>` ile heterojen listeyi gösterdi; statik/dinamik dispatch ve dyn
// uyumluluğunun NEDENİ Böl. 10'da, fat pointer/vtable ileri-sized-dst'de anlatıldı. Burada onları
// tekrar etmiyoruz; TASARIM kararlarına iniyoruz: genişletilebilir bir bildirim sistemi, aynı
// problemde dyn / generic / enum karşılaştırması, hangi işaretçi (&dyn, Box, Rc, Arc), `+ Send` ve
// gizli `+ 'static`, supertrait + upcasting, `dyn Any`, şablon metot, bileşim ve `Box<dyn>` klonlama.

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Bu dosya main.rs'teki oyuncak kutusunu (trait object) gerçek bir tasarımda kullanır: e-posta,
// SMS gibi bildirim kanalları. Her bölümün üstündeki "SADE ANLATIM" kutusunu önce oku.
//
// Bu dosyanın tek cümlelik özeti: Çekirdek kod yalnız "bildirim gönderebilen bir şey" sözünü
// bilir; yeni bir kanal türü eklemek için çekirdeğe dokunmazsın.
//
// MİNİ SÖZLÜK (yalnız bu dosyada yeni geçenler; öteki terimler main.rs başındaki sözlükte):
//   supertrait               → bir trait'in ön şartı: "Kanal olan her şey Debug da olmalı".
//   şablon metot (template method) → adımları sabit, bazı adımları türlere bırakılmış tarif.
//   kanca (hook)             → hazır cevabı olan, isteyen türün değiştirdiği küçük adım.
//   sözleşme                 → bir trait'in "bunu uygulayan şunları yapar" diye verdiği söz.
//   eklenti (plugin)         → programa sonradan, çekirdeği değiştirmeden eklenen parça.
//   `impl Trait` (parametrede) → "bu trait'i uygulayan herhangi bir tür" demenin kısa yazımı.
//   Self: Sized              → "bu metot yalnız türü belli değerlerde çağrılır, dyn'de değil".
//   upcasting                → daha özel bir trait object'e daha genel gözle bakmak.
//   Any ve downcast          → "sen aslında şu tür müsün?" diye çalışırken sormak.
//   koku (code smell)        → çalışan ama tasarımda bir şeyin ters gittiğini gösteren işaret.
//   devretme (delegation)    → işi kendin yapmak yerine içindeki parçaya yaptırmak.
//   sarmalayıcı (wrapper)    → başka bir değeri içine alıp ona bir şey ekleyen tür.
//   blanket impl             → "şu şartı sağlayan HER tür için" bir kerede yazılan impl.
//   Send / Sync              → "başka thread'e verilebilir" / "birkaç thread birlikte bakabilir".
//   'static                  → "içinde kısa ömürlü bir ödünç yok" demek.
//   DST                      → boyu derlerken bilinmeyen tür (`dyn Trait` gibi).
//   unsize coercion          → `&Sms` gibi belli bir türün kendiliğinden `&dyn Kanal`a dönüşmesi.
//   HashMap                  → anahtar → değer tablosu; sırası belli değildir.

use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;

pub fn calistir() {
    println!("\n== Derinleşme: Trait Object'ler ==");
    genisletilebilir_sistem();
    dyn_generic_enum();
    isaretci_secimi();
    gizli_static_siniri();
    supertrait_upcasting_any();
    bilesim_ve_klonlama();
}

// ============================================================================
// 1. GERÇEK HAYATTAN ÖRNEK — bildirim kanalları (şablon metot + kanca + supertrait)
// ============================================================================
// Bu konuyu neden öğreniyoruz? Bir sistemin "hangi türlerle çalışacağı" yazıldığı gün belli
// değilse (eklentiler, sürücüler, bildirim kanalları, komutlar), çekirdek kodun o türleri
// BİLMEDEN çalışması gerekir. Trait object bunun aracıdır: çekirdek yalnız sözleşmeyi (trait) bilir.
//
// Supertrait'ler (`Kanal: Debug + Any + KanalKlonla`) "Kanal olan her şey bunları da sağlar" der:
//   • Debug       → `Box<dyn Kanal>` doğrudan {:?} ile basılabilir (derive(Debug) çalışır).
//   • Any         → §5'te downcast için; yan etkisi: Kanal uygulayan tür 'static olmak zorunda.
//   • KanalKlonla → §6'da `Box<dyn Kanal>`'ı klonlayabilmek için.
// SADE ANLATIM: Kurabiye tarifi herkes için aynıdır: hamuru yoğur, ŞEKİL VER, pişir. Yalnız
// "şekil ver" adımını her çocuk kendince yapar: biri yıldız, biri kalp. Aşağıdaki `gonder`
// metodu bu tariftir (şablon metot); `bicimle` her kanalın kendince yaptığı adımdır.
// `azami_uzunluk` ise "istersen süsle" gibi isteğe bağlı bir adımdır (kanca): hazır cevabı
// "sınır yok"tur, SMS onu değiştirir. Baştaki `Kanal: Debug + ...` da ön şarttır: "Kanal olmak
// isteyen, önce bunları da yapabilmeli".
// Neden böyle? Tarif tek yerde durmasaydı her kanal onu kopyalar, biri bir adımı unuturdu.
//   Mini örnek:
//     let e = Eposta { adres: String::from("a@b.c") };
//     let s = Sms { numara: String::from("111") };
//     println!("{}", e.gonder("çok uzun bir bildirim metni"));   // ✓ tamamı gider
//     println!("{}", s.gonder("çok uzun bir bildirim metni"));   // ✓ aynı tarif, 12 harfe kırpar
pub trait Kanal: Debug + Any + KanalKlonla {
    fn ad(&self) -> &'static str;
    fn bicimle(&self, mesaj: &str) -> String; // her kanalın FARKLI yaptığı tek adım

    // KANCA (hook): varsayılanı "sınır yok"; ihtiyacı olan tür ezer.
    fn azami_uzunluk(&self) -> Option<usize> {
        None
    }

    // ŞABLON METOT (template method): algoritmanın İSKELETİ burada, tek yerde. Türler iskeleti
    // değil yalnız adımları (bicimle, azami_uzunluk) sağlar. Kalıtımlı dillerde bu, taban sınıfın
    // metodudur; Rust'ta varsayılan trait metodu. Fark: paylaşılan şey DAVRANIŞTIR, alan değil —
    // trait'in verisi yoktur, ihtiyaç duyduğu her şeyi zorunlu metotlardan ister.
    fn gonder(&self, mesaj: &str) -> String {
        let govde: String = match self.azami_uzunluk() {
            Some(sinir) => mesaj.chars().take(sinir).collect(),
            None => mesaj.to_string(),
        };
        format!("[{}] {}", self.ad(), self.bicimle(&govde))
    }

    // dyn-UYUMSUZ bir metot (dönüş türünde Self geçiyor: Etiketli<Self>). `where Self:
    // Sized` onu vtable'ın DIŞINDA bırakır: trait dyn-uyumlu kalır, metot yalnız somut türlerde
    // çağrılabilir. std'deki Iterator::map/filter aynı numarayla `dyn Iterator`'ı mümkün kılar.
    fn etiketli(self, etiket: &'static str) -> Etiketli<Self>
    where
        Self: Sized,
    {
        Etiketli { ic: self, etiket }
    }
}

#[derive(Debug, Clone)]
struct Eposta {
    adres: String,
}
impl Kanal for Eposta {
    fn ad(&self) -> &'static str {
        "eposta"
    }
    fn bicimle(&self, mesaj: &str) -> String {
        format!("{} adresine: {mesaj}", self.adres)
    }
}

#[derive(Debug, Clone)]
struct Sms {
    numara: String,
}
impl Kanal for Sms {
    fn ad(&self) -> &'static str {
        "sms"
    }
    fn bicimle(&self, mesaj: &str) -> String {
        format!("{} numarasına: {mesaj}", self.numara)
    }
    fn azami_uzunluk(&self) -> Option<usize> {
        Some(12) // kancayı ezer: iskelet mesajı kırpar
    }
}

// Çekirdek: somut tür adı GEÇMİYOR. `impl Kanal` alıp içeride kutuluyoruz: çağıran Box::new
// yazmak zorunda kalmaz, generic → dyn geçişi tek noktada olur.
#[derive(Default, Clone)]
struct Dagitici {
    kanallar: HashMap<String, Box<dyn Kanal>>,
}
impl Dagitici {
    fn kaydet(&mut self, kanal: impl Kanal) {
        self.kaydet_kutulu(Box::new(kanal));
    }
    fn kaydet_kutulu(&mut self, kanal: Box<dyn Kanal>) {
        self.kanallar.insert(kanal.ad().to_string(), kanal); // aynı ad → üzerine yazar
    }
    fn gonder(&self, kanal_adi: &str, mesaj: &str) -> Option<String> {
        self.kanallar.get(kanal_adi).map(|k| k.gonder(mesaj))
    }
    fn adlar(&self) -> Vec<&str> {
        let mut adlar: Vec<&str> = self.kanallar.keys().map(String::as_str).collect();
        adlar.sort_unstable(); // HashMap sırası rastgele → çıktı için sırala
        adlar
    }
}

// "Sonradan eklenen" tür: Dagitici'de, Kanal'da ya da öteki türlerde TEK SATIR değişmedi.
// Başka bir crate'te de yazılabilirdi (trait pub ise) — eklenti mimarisinin özü budur.
#[derive(Debug, Clone)]
struct Webhook {
    url: String,
}
impl Kanal for Webhook {
    fn ad(&self) -> &'static str {
        "webhook"
    }
    fn bicimle(&self, mesaj: &str) -> String {
        format!("POST {} gövde={mesaj:?}", self.url)
    }
}

fn ornek_dagitici() -> Dagitici {
    let mut dagitici = Dagitici::default();
    dagitici.kaydet(Eposta {
        adres: String::from("ece@ornek.com"),
    });
    dagitici.kaydet(Sms {
        numara: String::from("555-0101"),
    });
    dagitici.kaydet(Webhook {
        url: String::from("/kanca"),
    });
    dagitici
}

fn genisletilebilir_sistem() {
    println!("-- 1. genişletilebilir sistem --");
    let dagitici = ornek_dagitici();
    println!("kayıtlı: {:?}", dagitici.adlar());
    for ad in ["eposta", "sms", "webhook", "faks"] {
        match dagitici.gonder(ad, "Kargonuz yola çıktı") {
            Some(cikti) => println!("  {cikti}"),
            None => println!("  ({ad}: böyle bir kanal yok)"),
        }
    }
    // Beklenen çıktı:
    //   kayıtlı: ["eposta", "sms", "webhook"]
    //     [eposta] ece@ornek.com adresine: Kargonuz yola çıktı
    //     [sms] 555-0101 numarasına: Kargonuz yol
    //     [webhook] POST /kanca gövde="Kargonuz yola çıktı"
    //     (faks: böyle bir kanal yok)
}

// ============================================================================
// 2. AYNI PROBLEM, ÜÇ ÇÖZÜM — dyn Trait / generic / enum
// ============================================================================
// SADE ANLATIM: Aynı oyuncakları üç ayrı kutuda saklayabilirsin. Birincinin üstünde "ses çıkaran
// her şey" yazar: yarın alınan yeni oyuncak da girer (dyn). İkincisi yalnız düdüklerin kutusudur:
// hepsi aynı türdendir, bu yüzden en hızlısıdır (generic). Üçüncünün bölmeleri sabittir ve
// üstünde "düdük ya da davul" yazar: listeyi yalnız kutuyu yapan değiştirebilir, ama içinde ne
// olabileceği baştan bellidir (enum).
// Neden üç yol? Asıl soru "yeni türleri kim ekleyecek?" sorusudur. Başkaları ekleyecekse dyn,
// yalnız sen ekleyeceksen ve türler azsa enum, tek tür varsa generic uygundur.
//   Mini örnek:
//     let a: Vec<Box<dyn Kanal>> = vec![Box::new(eposta), Box::new(sms)];      // ✓ karışık, açık
//     let b = [SabitKanal::Eposta(eposta2), SabitKanal::Sms(sms2)];            // ✓ karışık, kapalı
//     yayinla_generic(&[sms3, sms4], "m");                                     // ✓ hepsi Sms
//     // yayinla_generic(&[eposta3, sms5], "m");   // ✗ derlenmez: generic dilim tek tür ister
fn yayinla_dyn(kanallar: &[Box<dyn Kanal>], mesaj: &str) -> Vec<String> {
    kanallar.iter().map(|k| k.gonder(mesaj)).collect()
}
// Generic: K çağrı yerinde TEK bir somut türe sabitlenir → dilim HOMOJEN olmak zorunda.
fn yayinla_generic<K: Kanal>(kanallar: &[K], mesaj: &str) -> Vec<String> {
    kanallar.iter().map(|k| k.gonder(mesaj)).collect()
}
// Enum: türler kümesi KAPALI ve bu dosyada sabit. Heterojen ama kutusuz; `match` bütünlük
// (exhaustiveness) denetimi yapar: yeni varyant eklenince derleyici güncellenmesi gereken her yeri gösterir.
enum SabitKanal {
    Eposta(Eposta),
    Sms(Sms),
}
impl SabitKanal {
    fn gonder(&self, mesaj: &str) -> String {
        match self {
            SabitKanal::Eposta(e) => e.gonder(mesaj),
            SabitKanal::Sms(s) => s.gonder(mesaj),
        }
    }
}
//                 | dyn Trait                 | generic <K: Kanal>         | enum
//   tür kümesi    | AÇIK (dışarıdan eklenir)  | açık, ama çağrıda TEK tür  | KAPALI (yazar belirler)
//   heterojen liste| evet                     | hayır                      | evet
//   çağrı         | vtable, inline edilemez   | statik, inline edilebilir  | match (dallanma), inline
//   kullanan kod  | tek kopya                 | tür başına kopya           | tek kopya
//   yeni TÜR      | kolay (yeni impl)         | kolay (yeni impl)          | her match'e dokunulur
//   yeni İŞLEM    | trait'e + her impl'e      | trait'e + her impl'e       | kolay (yeni fn + match)
// Pratik ölçüt: türleri KİM ekleyecek? Kullanıcı/eklenti → dyn. Sen ekleyeceksen ve küme küçükse → enum.
// Tek tür ve sıcak döngü → generic.
fn dyn_generic_enum() {
    println!("-- 2. dyn / generic / enum --");
    let eposta = || Eposta {
        adres: String::from("a@b.c"),
    };
    let sms = || Sms {
        numara: String::from("111"),
    };
    let kutulu: Vec<Box<dyn Kanal>> = vec![Box::new(eposta()), Box::new(sms())];
    let sabit = [SabitKanal::Eposta(eposta()), SabitKanal::Sms(sms())];
    let enum_ciktisi: Vec<String> = sabit.iter().map(|k| k.gonder("Merhaba")).collect();
    println!("dyn     : {:?}", yayinla_dyn(&kutulu, "Merhaba"));
    println!(
        "enum    : aynı çıktı mı? {}",
        enum_ciktisi == yayinla_dyn(&kutulu, "Merhaba")
    );
    println!(
        "generic : {:?}",
        yayinla_generic(&[sms(), sms()], "Merhaba")
    );
    // Beklenen çıktı:
    //   dyn     : ["[eposta] a@b.c adresine: Merhaba", "[sms] 111 numarasına: Merhaba"]
    //   enum    : aynı çıktı mı? true
    //   generic : ["[sms] 111 numarasına: Merhaba", "[sms] 111 numarasına: Merhaba"]
}

// ============================================================================
// 3. HANGİ İŞARETÇİ? — &dyn, Box<dyn>, Rc<dyn>, Arc<dyn ... + Send + Sync>
// ============================================================================
//   &dyn Kanal  → yalnız KULLANACAKSAN (parametre). Tahsis yok, sahiplik yok. Varsayılan tercih.
//   Box<dyn>    → SAHİP olacaksan (alan, koleksiyon, dönüş değeri). Tek sahip.
//   Rc<dyn>     → aynı nesne birden çok sahipte duracaksa, tek thread (Böl. 15).
//   Arc<dyn>    → aynısı, thread'ler arası (Böl. 16).
// SADE ANLATIM: Kutudaki oyuncağa nasıl uzandığın da bir seçimdir. Arkadaşının oyuncağıyla bir
// kez oynayıp geri vereceksen ödünç alırsın (`&dyn`). Oyuncak seninse kendi kutunda durur
// (`Box<dyn>`). Birkaç çocuk aynı oyuncağın ortak sahibiyse sayaçlı kutu gerekir (`Rc<dyn>`);
// çocuklar ayrı thread'lerdeyse onun thread'ler arası güvenli olanı (`Arc<dyn>`).
// Neden böyle? `dyn` türünün boyu bilinmez, bu yüzden hep bir işaretçinin arkasında durur;
// hangi işaretçiyi seçtiğin "sahibi kim?" sorusunun cevabıdır.
//   Mini örnek:
//     let sms = Sms { numara: String::from("1") };
//     bir_kez_bildir(&sms, "selam");                  // ✓ ödünç: yalnız kullan, geri ver
//     let benim: Box<dyn Kanal> = Box::new(sms);      // ✓ sahip: sms kutuya taşındı
//     // let v: Vec<dyn Kanal> = Vec::new();          // ✗ derlenmez: dyn işaretçisiz durmaz
fn bir_kez_bildir(kanal: &dyn Kanal, mesaj: &str) -> String {
    kanal.gonder(mesaj)
}

fn isaretci_secimi() {
    println!("-- 3. işaretçi seçimi --");
    let sms = Sms {
        numara: String::from("222"),
    };
    println!("&dyn : {}", bir_kez_bildir(&sms, "kısa")); // &Sms → &dyn Kanal (unsize coercion)

    let ortak: Rc<dyn Kanal> = Rc::new(sms);
    let acil_listesi = [Rc::clone(&ortak)];
    let gunluk_listesi = [Rc::clone(&ortak)];
    let adlar = (acil_listesi[0].ad(), gunluk_listesi[0].ad());
    println!(
        "Rc   : adlar = {adlar:?}, sahip sayısı = {}",
        Rc::strong_count(&ortak)
    );

    // (Sade: kutunun kapağı kapalı olduğu için derleyici içindekinin başka thread'e verilip
    // verilemeyeceğini göremez. `+ Send + Sync` yazarak "bu kutuya yalnız thread'ler arası güvenli
    // şeyler koyacağım" sözünü sen verirsin; derleyici de o sözü tutturur.)
    // `dyn Kanal` tek başına Send/Sync DEĞİLDİR: tür silindiği için derleyici içindeki somut türün
    // thread-güvenli olup olmadığını bilemez (auto trait'ler dyn'den "sızmaz"). Söz vermek SENİN
    // işin: `dyn Kanal + Send + Sync`. Bu yazım, yalnız Send+Sync türlerin kutulanmasına izin verir.
    //   ✗ let k: Box<dyn Kanal> = Box::new(..); thread::spawn(move || k.gonder(".."));
    //     E0277: `dyn Kanal` cannot be sent between threads safely
    let paylasilan: Arc<dyn Kanal + Send + Sync> = Arc::new(Webhook {
        url: String::from("/olay"),
    });
    let isler: Vec<_> = (1..=2)
        .map(|no| {
            let kanal = Arc::clone(&paylasilan);
            thread::spawn(move || kanal.gonder(&format!("iş {no}")))
        })
        .collect();
    for is in isler {
        println!("Arc  : {}", is.join().expect("iş parçacığı panik etmez"));
    }
    // Beklenen çıktı:
    //   &dyn : [sms] 222 numarasına: kısa
    //   Rc   : adlar = ("sms", "sms"), sahip sayısı = 3
    //   Arc  : [webhook] POST /olay gövde="iş 1"
    //   Arc  : [webhook] POST /olay gövde="iş 2"
}

// ============================================================================
// 4. TRAIT OBJECT'İN GİZLİ LIFETIME SINIRI
// ============================================================================
// Nasıl çalışır? Her `dyn Trait` aslında `dyn Trait + 'ömür` biçimindedir; yazmazsan derleyici doldurur:
//   Box<dyn Sablon>   =  Box<dyn Sablon + 'static>   → içine REFERANS tutan tür konamaz
//   &'a dyn Sablon    =  &'a (dyn Sablon + 'a)
// Neden var? Tür silinince derleyici "bu kutunun içindeki şey bir yere ödünç bağlı mı?" sorusunu
// somut türden okuyamaz; bilgi dyn'in üstündeki bu sınırda taşınır.
// SADE ANLATIM: Kapalı bir kutunun içindeki oyuncak kütüphaneden ÖDÜNÇ alınmış olabilir; kapağı
// açmadan bunu göremezsin. Bu yüzden kutunun üstüne bir etiket yapışır. Bir şey yazmazsan etiket
// "içinde ödünç şey yok" der (`'static`), ve böyle etiketli kutuya ödünç oyuncak koyamazsın.
// Ödünç bir şey koyacaksan etikete "şu ödünç geçerli olduğu sürece kullanılabilir" yazarsın:
// aşağıdaki `+ '_` budur.
// Neden böyle? Etiket olmasaydı, ödünç alınan şey geri verildikten sonra kutu hâlâ onu
// gösteriyor olabilirdi (sarkan referans).
//   Mini örnek:
//     fn kutula(o: &str) -> Box<dyn Sablon + '_> { Box::new(Onek(o)) }   // ✓ etiket açıkça yazıldı
//     fn kotu(o: &str) -> Box<dyn Sablon> { Box::new(Onek(o)) }          // ✗ derlenmez: etiket
//                                                                        //   "ödünç yok" diyor
trait Sablon {
    fn uygula(&self, mesaj: &str) -> String;
}
struct Onek<'a>(&'a str); // referans tutan tür
impl Sablon for Onek<'_> {
    fn uygula(&self, mesaj: &str) -> String {
        format!("{}{mesaj}", self.0)
    }
}
// ✗ fn sablon_kutula(onek: &str) -> Box<dyn Sablon> { Box::new(Onek(onek)) }
//     error: lifetime may not live long enough   (bu hatanın E-kodu yok; "returning this value
//     requires that `'1` must outlive `'static`" der)
// ✗ Aynı kök neden, kodlu hâli: yereli ödünç alan Onek'i `Vec<Box<dyn Sablon>>`'a itip yerel
//     düştükten sonra kullanmak → E0597: `s` does not live long enough
// ✓ Sınırı AÇIKÇA gevşet: kutu, `onek` yaşadığı sürece geçerli. (`'_` = girdideki ömür.)
fn sablon_kutula(onek: &str) -> Box<dyn Sablon + '_> {
    Box::new(Onek(onek))
}

fn gizli_static_siniri() {
    println!("-- 4. gizli 'static sınırı --");
    let onek = String::from(">> ");
    let sablon = sablon_kutula(&onek);
    println!("{}", sablon.uygula("ödünç tutan trait object"));
    // Beklenen çıktı:
    //   >> ödünç tutan trait object
}

// ============================================================================
// 5. UPCASTING ve `dyn Any` — downcast ne zaman kokudur?
// ============================================================================
// Upcasting: `&dyn Kanal` → `&dyn Debug` / `&dyn Any` (alt trait object'ten supertrait object'e).
// Rust 1.86'dan beri kararlı; öncesinde her trait'e elle `fn as_any(&self) -> &dyn Any` yazılırdı.
//
// `dyn Any` çalışma anında "sen aslında şu tür müsün?" sorusunu sordurur (downcast_ref).
// KOKU olduğu durum: genel koda `if Sms ise ...` yazmak. Açık küme vaadini bozar — aşağıdaki
// `kirpar_mi_kokulu`, Sms'i saran Etiketli<Sms>'i ve yarın eklenecek her kısa-mesaj kanalını kaçırır.
// MEŞRU olduğu durum: türü gerçekten bilinmeyen kaplar (tür → değer haritaları, eklentinin kendi
// durumunu geri alması) ve `Box<dyn Error>`'dan belirli bir hatayı ayıklamak (Böl. 9).
// SADE ANLATIM: Kutudan çıkan oyuncağa "sen düdük müsün?" diye sorabilirsin (`dyn Any` ile
// downcast). Ama düdük renkli bir kılıfa sarılmışsa "hayır" cevabını alırsın, oysa hâlâ öter.
// Doğru soru "ötebilir misin?" sorusudur; yani soruyu türe değil trait'in bir metoduna sorarsın.
// Upcasting ise bunun tersidir ve zararsızdır: "ses çıkaran oyuncak"a yalnızca "bir oyuncak"
// gözüyle bakmak.
// Neden böyle? Tür adına bakan kod, yarın eklenecek her yeni türü ve her sarmalayıcıyı kaçırır.
//   Mini örnek:
//     let sms = Sms { numara: String::from("1") };
//     let sarili = sms.clone().etiketli("[ACİL]");
//     kirpar_mi_kokulu(&sarili);    // false — "Sms misin?" diye sordu, kılıfı tanımadı
//     kirpar_mi(&sarili);           // ✓ true — "sınırın var mı?" diye sordu, kılıf içine iletti
fn kirpar_mi_kokulu(kanal: &dyn Kanal) -> bool {
    let herhangi: &dyn Any = kanal; // upcast
    herhangi.is::<Sms>()
}
fn kirpar_mi(kanal: &dyn Kanal) -> bool {
    kanal.azami_uzunluk().is_some() // ✓ soruyu TRAIT'e sor: her tür kendi cevabını verir
}

fn supertrait_upcasting_any() {
    println!("-- 5. upcasting ve Any --");
    let sms = Sms {
        numara: String::from("333"),
    };
    let kanal: &dyn Kanal = &sms;
    let hata_ayiklama: &dyn Debug = kanal; // upcast: artık yalnız Debug yüzü görünür
    println!("Debug yüzü: {hata_ayiklama:?}");
    let herhangi: &dyn Any = kanal;
    println!(
        "downcast: {:?}",
        herhangi.downcast_ref::<Sms>().map(|s| s.numara.as_str())
    );
    println!(
        "Eposta'ya downcast: {}",
        herhangi.downcast_ref::<Eposta>().is_some()
    );

    let sarili = sms.clone().etiketli("[ACİL]");
    println!(
        "kokulu: sms={} sarılı={}",
        kirpar_mi_kokulu(&sms),
        kirpar_mi_kokulu(&sarili)
    );
    println!(
        "doğru : sms={} sarılı={}",
        kirpar_mi(&sms),
        kirpar_mi(&sarili)
    );
    // Beklenen çıktı:
    //   Debug yüzü: Sms { numara: "333" }
    //   downcast: Some("333")
    //   Eposta'ya downcast: false
    //   kokulu: sms=true sarılı=false
    //   doğru : sms=true sarılı=true
}

// ============================================================================
// 6. KALITIM YERİNE BİLEŞİM + `Box<dyn Kanal>` KLONLAMA
// ============================================================================
// Kalıtımlı dilde "AcilSms extends Sms" yazardın. Rust'ta SARMALARSIN: Etiketli, başka bir kanalı
// ALAN olarak tutar, işin çoğunu ona DEVREDER (delegation), yalnız bir adımı değiştirir. Kazanç:
// Etiketli tek bir taban türe bağlı değil — her Kanal'ı sarar ve sarmalayıcılar iç içe geçebilir.
// Bedel: devredilen her metodu elle yazarsın (aşağıda üç satır); Rust bunu otomatik yapmaz.
// SADE ANLATIM: Düdüğü daha gür öttürmek için yeni bir düdük türü icat etmezsin; eldeki düdüğü
// bir huninin içine yerleştirirsin. Huni yine "ses çıkaran bir şey"dir: sesi içindeki düdüğe
// çıkarttırır, üstüne yalnız kendi katkısını ekler. Aşağıdaki `Etiketli` böyle bir sarmalayıcıdır
// ve her Kanal'ı sarabilir; sarmalayıcılar iç içe de geçer. Dosyanın devamı bir de şunu çözer:
// kapalı kutudaki oyuncağın türünü bilmeden aynısından bir tane daha isteyebilmek (klonlama).
// Neden kalıtım değil? Kalıtım tek bir "ata" türe bağlar; sarmalayıcı her türle çalışır.
//   Mini örnek:
//     let acil = Sms { numara: String::from("1") }.etiketli("[ACİL]");
//     println!("{}", acil.gonder("selam"));     // ✓ [sms] [ACİL] 1 numarasına: selam
//     let kutu: Box<dyn Kanal> = Box::new(acil);
//     let kopya = kutu.clone();                 // ✓ türünü bilmeden aynısı yapıldı
#[derive(Debug, Clone)]
pub struct Etiketli<K> {
    ic: K,
    etiket: &'static str,
}
impl<K: Kanal + Clone> Kanal for Etiketli<K> {
    fn ad(&self) -> &'static str {
        self.ic.ad()
    }
    fn bicimle(&self, mesaj: &str) -> String {
        format!("{} {}", self.etiket, self.ic.bicimle(mesaj))
    }
    fn azami_uzunluk(&self) -> Option<usize> {
        self.ic.azami_uzunluk()
    }
}

// Klonlama sorunu: Clone::clone `Self` döndürür → `trait Kanal: Clone` yazarsan trait dyn-uyumsuz
// olur (§7-c). Çözüm: Self yerine KUTU döndüren, dyn-uyumlu bir yardımcı metot. Onu her türe elle
// yazmamak için blanket impl (Böl. 10'daki Etiket kalıbı): "Kanal + Clone olan her T için hazır".
// (Sade: `clone` "kendi türümden bir tane daha ver" der; kapalı kutuda tür bilinmediği için bu
// söz tutulamaz. Çare, "aynımı yap ve KUTUYA koyup ver" diyen ayrı bir metottur. Altındaki
// blanket impl onu "Kanal + Clone olan her tür için" bir kerede yazar.)
pub trait KanalKlonla {
    fn kutu_klonu(&self) -> Box<dyn Kanal>;
}
impl<T: Kanal + Clone> KanalKlonla for T {
    fn kutu_klonu(&self) -> Box<dyn Kanal> {
        Box::new(self.clone()) // burada T SOMUT: clone() statik çağrı, sonra kutula
    }
}
// Artık Box<dyn Kanal> Clone'dur; onu alan olarak tutan Dagitici de derive(Clone) yapabildi.
// `(**self)`: &Box<dyn Kanal> → dyn Kanal; çağrı vtable üzerinden İÇTEKİ somut türe gider. Açık
// yazmak bir sigortadır: ileride `impl Kanal for Box<dyn Kanal>` eklenirse `self.kutu_klonu()`
// kutunun KENDİ blanket impl'ini bulur, o da yine Box::clone'u çağırır → sonsuz özyineleme.
impl Clone for Box<dyn Kanal> {
    fn clone(&self) -> Self {
        (**self).kutu_klonu()
    }
}

fn bilesim_ve_klonlama() {
    println!("-- 6. bileşim ve klonlama --");
    let acil = Sms {
        numara: String::from("444"),
    }
    .etiketli("[ACİL]")
    .etiketli("[TEST]");
    println!("{}", acil.gonder("Sunucu yanıt vermiyor"));

    let asil: Box<dyn Kanal> = Box::new(acil);
    let kopya = asil.clone(); // somut türü bilmeden derin kopya
    println!("kopya: {kopya:?}");

    let dagitici = ornek_dagitici();
    let mut yedek = dagitici.clone(); // HashMap<String, Box<dyn Kanal>> artık klonlanabiliyor
    yedek.kaydet_kutulu(kopya); // yalnız YEDEKTEKİ sms kaydının üzerine yazar
    println!("yedek: {:?}", yedek.gonder("sms", "Deneme"));
    println!("asıl : {:?}", dagitici.gonder("sms", "Deneme"));
    // Beklenen çıktı:
    //   [sms] [TEST] [ACİL] 444 numarasına: Sunucu yanıt
    //   kopya: Etiketli { ic: Etiketli { ic: Sms { numara: "444" }, etiket: "[ACİL]" }, etiket: "[TEST]" }
    //   yedek: Some("[sms] [TEST] [ACİL] 444 numarasına: Deneme")
    //   asıl : Some("[sms] 555-0101 numarasına: Deneme")
}

// SADE ANLATIM (aşağıdaki bölüm için): Aşağıda üç yaygın derleme hatası, bu dosyanın özeti ve üç
// alıştırma var. Hataların ortak kökü şudur: kapalı kutunun (dyn) içindeki türün ne adı ne boyu
// bilinir; bu yüzden "kendi türümü döndür" diyen metotlar, işaretçisiz kullanım ve doğrudan
// klonlama çalışmaz. İlk okuyuşta bu bölümü atlayabilirsin.
// ============================================================================
// 7. SIK YAPILAN HATALAR — yanlış kullanım / doğru kullanım (kodlar rustc ile doğrulandı)
// ============================================================================
//
// (a) Self dönen ya da generic metotlu trait'i dyn yapmak (neden: Böl. 10, DERLEYİCİ PERSPEKTİFİ)
//     ✗ trait Kanal { fn kopya(&self) -> Self; }            fn f(_: &dyn Kanal) {}
//     ✗ trait Kanal { fn gonder<T: Display>(&self, m: T); }  fn f(_: &dyn Kanal) {}
//       E0038: the trait `Kanal` is not dyn compatible
//     ✓ O metodu `where Self: Sized` ile dyn'in dışında bırak (yukarıdaki `etiketli`). Bedeli:
//       metot trait object üzerinde çağrılamaz — `kutu.etiketli("x")` →
//       "error: the `etiketli` method cannot be invoked on a trait object" (E-kodu yok).
//
// (b) Trait object'i işaretçisiz kullanmak
//     ✗ let v: Vec<dyn Kanal> = Vec::new();
//       E0277: the size for values of type `dyn Kanal` cannot be known at compilation time
//     ✓ Vec<Box<dyn Kanal>> — dyn bir DST'dir, yalnız işaretçi ardında durur (ileri-sized-dst).
//
// (c) Box<dyn Trait>'i klonlamaya çalışmak
//     ✗ let k2 = kutu.clone();        // trait'te klon desteği yokken
//       E0599: the method `clone` exists for struct `Box<(dyn Kanal + 'static)>`, but its trait bounds were
//              not satisfied   (`dyn Kanal: Clone` sağlanmıyor)
//     ✗ "O zaman `trait Kanal: Clone`" → E0038: Clone, Self döndürdüğü için dyn-uyumsuz.
//     ✓ §6'daki kutu_klonu + `impl Clone for Box<dyn Kanal>`.
//
// ÖNEMLİ NOKTALAR
//   • Soru "dyn mi generic mi" değil, "tür kümesini KİM genişletecek ve liste heterojen mi?".
//   • dyn; Send/Sync ve ömür bilgisini kendiliğinden taşımaz: `+ Send`, `+ 'a` ile sen yazarsın.
//   • Downcast yerine trait'e metot ekle; kalıtım yerine sar ve devret.
//
// ============================================================================
// ALIŞTIRMALAR (çözümler verilmedi; `cargo test -p oop` ile kendin sına)
// ============================================================================
// 1) `Konsol` adında yeni bir kanal ekle (alanı olmayan unit struct) ve `ornek_dagitici`'ya kaydet.
//    İpucu: derive(Debug, Clone) + `ad` ve `bicimle` yeterli; Dagitici'ye DOKUNMA.
//    Beklenen: `dagitici.adlar()` → ["eposta", "konsol", "sms", "webhook"]. Dikkat:
//    `kutulu_kanal_ve_dagitici_bagimsiz_klonlanir` testi ve §1'in beklenen çıktısı `ornek_dagitici`ya
//    bağlıdır; ikisini de yeni listeye göre güncellemen gerekir (Dagitici ve Kanal ise değişmez).
// 2) Kanal'a `fn oncelik(&self) -> u8 { 0 }` kancası ekle, Sms'te 2 yap; Dagitici'ye bir mesajı
//    tüm kanallara öncelik sırasıyla (büyükten küçüğe, eşitlikte ada göre) gönderen `herkese` yaz.
//    İpucu: `values()` → Vec<&Box<dyn Kanal>> → `sort_by_key(|k| (Reverse(k.oncelik()), k.ad()))`.
//    Beklenen: ilk çıktı "[sms] ..." ile başlar; sıra her çalıştırmada aynıdır.
// 3) `SabitKanal`'a `Webhook` varyantı ekle ve derleyicinin seni nereye götürdüğünü not et. Sonra
//    aynı eklemenin dyn çözümünde kaç yere dokunduğunu karşılaştır.
//    İpucu: §2'deki tablonun "yeni TÜR" satırını kendi gözünle doğrulamış olacaksın.
//    Beklenen: enum'da `gonder` içindeki match derlenmez (E0004) ve bir kol istenir; dyn'de sıfır yer.
//
// BİR SONRAKİ KONUYLA BAĞLANTISI
//   • patterns (Böl. 19): §2'deki enum çözümünün gücü `match`'ten gelir — bütünlük denetimi,
//     iç içe desenler, guard'lar; "kapalı küme" tasarımlarının asıl aracı orada.
//   • ileri-sized-dst: `Box<dyn Kanal>`'ın neden iki word olduğu, `where Self: Sized`'ın neye göre
//     "dışarıda bıraktığı" ve `Vec<dyn Kanal>`'ın neden imkânsız olduğu (Sized/DST) orada.

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn yeni_kanal_cekirdege_dokunmadan_eklenir() {
        let mut dagitici = Dagitici::default();
        assert_eq!(dagitici.gonder("webhook", "x"), None);
        dagitici.kaydet(Webhook {
            url: String::from("/t"),
        });
        assert_eq!(
            dagitici.gonder("webhook", "x").as_deref(),
            Some("[webhook] POST /t gövde=\"x\"")
        );
    }

    #[test]
    fn sablon_metot_kancaya_gore_kirpar() {
        let sms = Sms {
            numara: String::from("1"),
        };
        let eposta = Eposta {
            adres: String::from("a@b"),
        };
        assert_eq!(
            sms.gonder("on iki karakterden uzun"),
            "[sms] 1 numarasına: on iki karak"
        );
        assert!(eposta.gonder("on iki karakterden uzun").ends_with("uzun"));
    }

    #[test]
    fn dyn_generic_ve_enum_ayni_sonucu_verir() {
        let sms = Sms {
            numara: String::from("9"),
        };
        let dyn_sonuc = yayinla_dyn(&[Box::new(sms.clone())], "m");
        let generic_sonuc = yayinla_generic(std::slice::from_ref(&sms), "m");
        assert_eq!(dyn_sonuc, generic_sonuc);
        assert_eq!(dyn_sonuc[0], SabitKanal::Sms(sms).gonder("m"));
    }

    #[test]
    fn sarmalayici_ic_kanala_devreder() {
        let sarili = Sms {
            numara: String::from("7"),
        }
        .etiketli("!");
        assert_eq!(sarili.ad(), "sms");
        assert_eq!(sarili.azami_uzunluk(), Some(12));
        assert_eq!(sarili.gonder("selam"), "[sms] ! 7 numarasına: selam");
    }

    #[test]
    fn downcast_sarmalayiciyi_kacirir_trait_metodu_kacirmaz() {
        let sarili = Sms {
            numara: String::from("7"),
        }
        .etiketli("!");
        assert!(!kirpar_mi_kokulu(&sarili));
        assert!(kirpar_mi(&sarili));
        let herhangi: &dyn Any = &sarili as &dyn Kanal;
        assert!(herhangi.is::<Etiketli<Sms>>());
    }

    #[test]
    fn kutulu_kanal_ve_dagitici_bagimsiz_klonlanir() {
        let asil = ornek_dagitici();
        let mut kopya = asil.clone();
        kopya.kanallar.remove("sms");
        assert_eq!(asil.adlar(), vec!["eposta", "sms", "webhook"]);
        assert_eq!(kopya.adlar(), vec!["eposta", "webhook"]);
    }

    #[test]
    fn odunc_tutan_trait_object_omru_kadar_gecerlidir() {
        let onek = String::from("# ");
        assert_eq!(sablon_kutula(&onek).uygula("başlık"), "# başlık");
    }
}
