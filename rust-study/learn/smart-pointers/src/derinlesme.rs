// Açıklama: learn/smart-pointers/ACIKLAMA.md
// Derinleşme — Bölüm 15: Smart Pointers ve Interior Mutability (pekiştirme: gerçekçi senaryo, sık hatalar, alıştırmalar)
// Çalıştır: cargo run -p smart-pointers   |   Test: cargo test -p smart-pointers
//
// main.rs her işaretçiyi tek başına tanıttı; Weak ve referans döngüsü yalnız yorumda geçti.
// Bu dosya onları BİRLİKTE, gerçek bir veri yapısında kullanır ve "hangisini ne zaman?"
// sorusunu yanıtlar:
//   - seçim tablosu: sahip sayısı × değişiklik ihtiyacı × thread,
//   - ebeveynini bilen ağaç: Rc (aşağı) + Weak (yukarı) + RefCell (değişen çocuk listesi),
//   - referans döngüsünün sızdırdığını GÖZLE görmek ve Weak ile onarmak,
//   - Cell / RefCell ile "dışarıdan değişmez, içeriden kayıt tutan" tür (iç değişkenlik),
//   - Rc::make_mut ile "paylaşılıyorsa kopyala" (clone-on-write).
// ✗ örneklerin hata kodları rustc 1.99 ile derlenerek doğrulandı.

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Bu dosya main.rs'teki araçları birlikte kullanır. Her bölümün "SADE ANLATIM" kutusunu önce oku;
// benzetmeler yine aynı OKULDA geçer.
//
// Bu dosyanın tek cümlelik özeti: Doğru kutuyu üç soruyla seçersin (kaç sahip, değişecek mi,
// yalnız yerini bilmek yetiyor mu) ve birbirini tutan iki verinin sızmaması için bir yönü Weak
// yaparsın.
//
// MİNİ SÖZLÜK (yalnız bu dosyada yeni geçenler; öteki terimler main.rs başındaki sözlükte):
//   Cell<T>                  → küçük bir değeri "al, değiştir, geri koy" diye güncelleyen hücre.
//   Arc<T>                   → Rc'nin birden çok thread'de güvenle çalışan hâli.
//   Mutex<T> / RwLock<T>     → RefCell'in çok thread'li karşılıkları: kilitle, kullan, bırak.
//   güçlü / zayıf sayaç (strong / weak count) → sahiplerin / yalnız yerini bilenlerin sayısı.
//   Rc::downgrade            → bir Rc'den, sayacı artırmayan bir Weak üretmek.
//   upgrade()                → Weak'e "gösterdiğin şey hâlâ duruyor mu?" diye sormak (Some / None).
//   guard (Ref / RefMut)     → borrow() ile aldığın izin kâğıdı; elinde durdukça izin açık kalır.
//   try_borrow_mut()         → izni panik yerine "olur / olmaz" cevabıyla (Result) istemek.
//   mock (test ikizi)        → testte gerçeğinin yerine geçen, olanları kaydeden sahte parça.
//   gözlemci (observer)      → "bir şey olunca bana haber ver" diye listeye yazılan parça.
//   önbellek (cache)         → bir daha hesaplamamak için sonucu saklanan değer.
//   clone-on-write           → "paylaşılıyorsa önce kopyala, sonra yaz" yöntemi (Rc::make_mut).
//   saturating_add           → taşacaksa en büyük değerde duran, başa sarmayan toplama.
//   atomik (atomic)          → iki thread aynı anda dokunsa bile bozulmayan, tek hamlelik işlem.

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

pub fn calistir() {
    println!("\n== Derinleşme: akıllı işaretçiler birlikte ==");
    dizin_agaci();
    dongu_ve_sizinti();
    ic_degiskenlik_senaryosu();
    refcell_tuzaklari();
    paylasiliyorsa_kopyala();
}

// ============================================================================
// 1. BU KONUYU NEDEN ÖĞRENİYORUZ? — Seçim tablosu
// ============================================================================
// SADE ANLATIM: Okulda bir eşyanın nerede duracağına üç soruyla karar verirsin: Onu kaç kişi
// kullanacak? Kullananlar onu değiştirecek mi? Biri yalnız "nerede durduğunu" mu bilmek istiyor?
// Aşağıdaki tablo aynı üç soruyu Rust için sorar ve her cevabın karşısına bir kutu yazar.
// Neden sırayla? Her kutunun bedeli var (sayaç, çalışırken denetim); gereksiz bedeli ödeme.
//   Mini örnek:
//     let tek = Box::new(1);                     // tek sahip
//     let ortak = Rc::new(1);                    // çok sahip, yalnız okunur
//     let degisen = Rc::new(RefCell::new(1));    // çok sahip ve içi değişir
//     let bilen = Rc::downgrade(&ortak);         // sahip değil, yalnız yerini biliyor (Weak)
//
// Sahiplik modelinin varsayılanı "tek sahip, derleme anında denetim"dir ve programların büyük
// kısmına yeter. Akıllı işaretçiler bu varsayılanın yetmediği ÜÇ ayrı soruya cevap verir;
// doğru işaretçiyi seçmek için bu üç soruyu sırayla sor:
//
//   Soru 1 — Kaç sahip var?
//       tek            → Box<T>            (heap + tek sahip; maliyeti bir tahsis)
//       çok, tek thread→ Rc<T>             (sayaçlı paylaşım; sayaç sıradan tam sayı)
//       çok, çok thread→ Arc<T>            (sayaç atomik → concurrency crate'i)
//   Soru 2 — Paylaşılan değer DEĞİŞECEK mi?
//       hayır          → Rc<T> / Arc<T> yeter (salt okunur)
//       evet, Copy/küçük değer → Cell<T>   (değeri kopyalayıp koyar; referans vermez, panik yok)
//       evet, genel    → RefCell<T>        (çalışma anında borrow sayacı; ihlalde panik)
//       evet, çok thread → Mutex<T> / RwLock<T>
//   Soru 3 — "Sahip olmadan bilmek" gerekiyor mu (geri işaret, önbellek, gözlemci listesi)?
//       evet           → Weak<T>           (sayacı artırmaz; kullanmadan önce upgrade() ile sorar)
//
// Sık görülen birleşimler bu cevapların çarpımıdır: Rc<RefCell<T>> (çok sahip + değişir, tek
// thread), Arc<Mutex<T>> (aynısının çok thread'li hâli), Rc<T> + Weak<T> (ağaç/graf).
//
// Önemli: bunlar ilk tercih değil, KAÇIŞ KAPILARIDIR. Önce "veriyi tek bir sahipte toplayıp
// ötekilere & veremez miyim?" diye sor. Rc<RefCell<...>> her yere yayılırsa derleyicinin
// derleme anında yakalayacağı hatalar çalışma anı paniklerine dönüşür.

// ============================================================================
// 2. GERÇEK HAYATTAN ÖRNEK — ebeveynini bilen dizin ağacı
// ============================================================================
// Bir dosya sistemi ağacı: her dizin çocuklarına SAHİPTİR (dizin silinirse içindekiler de
// silinir), ama her çocuk ebeveynini de BİLMELİDİR (tam yolu üretmek için). İki yön de Rc
// olsaydı ebeveyn ↔ çocuk birbirini canlı tutar, hiçbiri düşmezdi (bölüm 3).
//
// Sahiplik yönünü belirle, öteki yönü Weak yap:
//   ebeveyn → çocuk : Rc   ("sahibim")
//   çocuk → ebeveyn : Weak ("kim olduğunu biliyorum ama seni canlı tutmuyorum")
// SADE ANLATIM: Bir sınıf, içindeki öğrenci listesinin sahibidir: sınıf kapanırsa liste de
// kalkar. Öğrenci ise hangi sınıfta olduğunu BİLİR, ama o biliyor diye sınıf açık tutulmaz.
// Aşağıdaki ağaçta da iki yön böyle ayrılır: yukarıdan aşağıya Rc ("sahibim"), aşağıdan yukarıya
// Weak ("yerini biliyorum"). Weak'i kullanmadan önce `upgrade()` ile "hâlâ duruyor mu?" diye
// sorarsın; cevap ya Some(...) ya None olur, yani boş bir yere bakman mümkün değildir.
// Alan türlerini içten dışa oku: `RefCell<Weak<Dizin>>` = "sonradan değiştirilebilen (RefCell),
// sahip olmayan (Weak) bir Dizin işareti".
//   Mini örnek:
//     let sinif = Rc::new(String::from("4-A"));
//     let bilen = Rc::downgrade(&sinif);     // yerini biliyor; sahip sayacı hâlâ 1
//     println!("{:?}", bilen.upgrade());     // ✓ Some("4-A") — hâlâ açık
//     drop(sinif);                           // tek sahip gitti, sınıf kapandı
//     println!("{:?}", bilen.upgrade());     // ✓ None — "artık yok" cevabı geldi
pub struct Dizin {
    ad: String,
    ebeveyn: RefCell<Weak<Dizin>>, // sonradan atanacağı için RefCell
    cocuklar: RefCell<Vec<Rc<Dizin>>>, // &self ile çocuk ekleyebilmek için RefCell
}

impl Dizin {
    pub fn yeni(ad: &str) -> Rc<Dizin> {
        Rc::new(Dizin {
            ad: ad.to_string(),
            ebeveyn: RefCell::new(Weak::new()), // Weak::new(): hiçbir şeye bakmayan boş Weak
            cocuklar: RefCell::new(Vec::new()),
        })
    }

    // Adım adım:
    //  1) Rc::downgrade(ebeveyn): güçlü sayaca DOKUNMADAN bir Weak üretir (weak sayaç +1).
    //  2) O Weak, çocuğun `ebeveyn` hücresine yazılır (RefCell: &self üzerinden mutasyon).
    //  3) Çocuğun Rc'si ebeveynin listesine TAŞINIR: çocuğun tek güçlü sahibi artık ebeveyn.
    pub fn cocuk_ekle(ebeveyn: &Rc<Dizin>, cocuk: Rc<Dizin>) {
        *cocuk.ebeveyn.borrow_mut() = Rc::downgrade(ebeveyn);
        ebeveyn.cocuklar.borrow_mut().push(cocuk);
    }

    // upgrade(): "işaret ettiğim değer hâlâ yaşıyor mu?" → Option<Rc<Dizin>>. Yaşıyorsa geçici
    // bir güçlü sahiplik alırız (bu sırada düşemez), yoksa None. Sarkan işaretçi İMKÂNSIZ:
    // Weak'i kullanmanın tek yolu bu soruyu sormaktır.
    pub fn tam_yol(&self) -> String {
        match self.ebeveyn.borrow().upgrade() {
            Some(ebeveyn) => format!("{}/{}", ebeveyn.tam_yol(), self.ad),
            None => self.ad.clone(), // kök (ya da ebeveyni düşmüş, sahipsiz düğüm)
        }
    }

    pub fn cocuk_sayisi(&self) -> usize {
        self.cocuklar.borrow().len()
    }
}

fn dizin_agaci() {
    let kok = Dizin::yeni("");
    let home = Dizin::yeni("home");
    let kaan = Dizin::yeni("kaan");
    Dizin::cocuk_ekle(&kok, Rc::clone(&home));
    Dizin::cocuk_ekle(&home, Rc::clone(&kaan));

    println!("tam yol = {}", kaan.tam_yol()); // /home/kaan
    println!(
        "home: güçlü = {}, zayıf = {}, çocuk = {}",
        Rc::strong_count(&home), // 2: yerel `home` + kök'ün listesi
        Rc::weak_count(&home),   // 1: kaan'ın ebeveyn işareti
        home.cocuk_sayisi()      // 1
    );

    // Ebeveyni düşür: çocuk elimizde kalsa da ebeveyni canlı TUTMAZ.
    drop(kok);
    drop(home); // güçlü sayaç 0 → home düştü (kaan'daki Weak onu kurtarmadı — istenen bu)
    println!("ebeveyn düştükten sonra tam yol = {}", kaan.tam_yol()); // kaan
    // Beklenen çıktı:
    //   tam yol = /home/kaan
    //   home: güçlü = 2, zayıf = 1, çocuk = 1
    //   ebeveyn düştükten sonra tam yol = kaan
}

// ============================================================================
// 3. SIK YAPILAN HATA — referans döngüsü sızdırır (ve bunu gözle görmek)
// ============================================================================
// Rc "sayaç 0 olunca serbest bırak" der. İki değer birbirini Rc ile tutuyorsa sayaçlar asla
// 0'a inmez: dışarıdaki tüm sahipler gitse bile her biri ötekini canlı tutar. Derleyici bunu
// YAKALAMAZ; sızıntı Rust'ta "güvenli" sayılır (bellek bozulmaz, yalnız geri verilmez).
// Uzun yaşayan bir sunucuda bu, yavaş yavaş büyüyen bellek tüketimi demektir.
//
// Aşağıda düşen her düğüm ortak bir sayacı artırır; böylece "drop çağrıldı mı?"yı ölçebiliriz.
// SADE ANLATIM: İki öğrenci birbirinin kalemini tutuyor ve ikisi de "önce sen benimkini bırak"
// diyor. Zil çalar, herkes gider; onlar öylece kalır. İki veri birbirini Rc ile tutarsa aynısı
// olur: ikisinin de sayacı 1'de takılır, 0'a inemez ve kapladıkları yer geri verilmez. Çare,
// yönlerden birini Weak yapmaktır; Weak sayaca yazılmadığı için düğüm çözülür.
//   Mini örnek (kısaltılmış; gerçek kod alanlara borrow_mut() ile yazıyor):
//     a.komsu = Some(Rc::clone(&b));      // a, b'yi tutuyor
//     b.komsu = Some(Rc::clone(&a));      // ✗ b de a'yı tutuyor: döngü, ikisi de sızar
//     b.zayif_komsu = Rc::downgrade(&a);  // ✓ b, a'nın yalnız yerini biliyor: döngü yok
struct Dugum {
    dusme_sayaci: Rc<Cell<u32>>,
    guclu_komsu: RefCell<Option<Rc<Dugum>>>,
    zayif_komsu: RefCell<Weak<Dugum>>,
}

impl Dugum {
    fn yeni(sayac: &Rc<Cell<u32>>) -> Rc<Dugum> {
        Rc::new(Dugum {
            dusme_sayaci: Rc::clone(sayac),
            guclu_komsu: RefCell::new(None),
            zayif_komsu: RefCell::new(Weak::new()),
        })
    }
}

impl Drop for Dugum {
    fn drop(&mut self) {
        self.dusme_sayaci.set(self.dusme_sayaci.get() + 1);
    }
}

// Dönen değer: kapsam bittiğinde kaç düğümün GERÇEKTEN düştüğü (ideali 2).
fn cift_olustur(geri_isaret_zayif: bool) -> u32 {
    let sayac = Rc::new(Cell::new(0));
    {
        let a = Dugum::yeni(&sayac);
        let b = Dugum::yeni(&sayac);
        *a.guclu_komsu.borrow_mut() = Some(Rc::clone(&b)); // a → b : güçlü
        if geri_isaret_zayif {
            *b.zayif_komsu.borrow_mut() = Rc::downgrade(&a); // b → a : zayıf ✓
        } else {
            *b.guclu_komsu.borrow_mut() = Some(Rc::clone(&a)); // b → a : güçlü ✗ DÖNGÜ
        }
    } // a ve b yerel adları burada kapsam dışına çıkar
    sayac.get()
}

fn dongu_ve_sizinti() {
    println!(
        "iki yön de Rc  → düşen düğüm = {} (sızıntı!)",
        cift_olustur(false)
    ); // 0
    println!("geri yön Weak  → düşen düğüm = {}", cift_olustur(true)); // 2
    // Adım adım (✗ sürüm): kapsam sonunda `b` adı düşer → b'nin sayacı 2'den 1'e iner (a hâlâ
    // tutuyor), düşmez. Sonra `a` adı düşer → a'nın sayacı 2'den 1'e iner (b hâlâ tutuyor),
    // düşmez. İkisine de ulaşan kimse kalmadı ama ikisi de 1'de takılı: sızdı.
    // (✓ sürüm): `a` adı düşer → a'nın güçlü sayacı 1'den 0'a → a düşer → elindeki b Rc'sini
    // bırakır; `b` adı da düşmüştü → b'nin sayacı 0 → b düşer. Zincir çözülür.
    //
    // Kural: döngü kurabilen işaretlerden en az biri Weak olmalı — sahiplik taşımayan (geri/yan/gözlemci) kenar.
}

// ============================================================================
// 4. INTERIOR MUTABILITY — dışarıdan değişmez, içeriden kayıt tutan tür
// ============================================================================
// Hangi problemi çözer? Bazen bir işlem MANTIKSAL olarak salt okunurdur (çağıran açısından
// hiçbir şey değişmez) ama gerçekleştirim içeride bir şey güncellemek ister: çağrı sayacı,
// önbellek, denetim kaydı. Metodu `&mut self` yaparsak bu "iç ayrıntı" imzaya sızar ve tüm
// çağıranları özel erişim almaya zorlar — aynı anda iki okuyucu bile olamaz.
//
// İç değişkenlik bu ayrımı korur: imza `&self` kalır, değişen alan bir hücrenin içindedir.
//   Cell<T>    → değeri KOPYALAYARAK okur/yazar (get/set/replace/take). İçine referans vermez;
//                dolayısıyla "aynı anda iki borrow" diye bir durum oluşamaz, panik de olmaz.
//                Sayaç, bayrak gibi küçük Copy değerler için ilk tercih.
//   RefCell<T> → içine referans VERİR (borrow/borrow_mut); kuralı çalışma anında sayaçla
//                denetler. Vec, String, HashMap gibi yerinde değiştirilen değerler için.
// SADE ANLATIM: Kütüphaneden bir kitabı yalnız OKUMAK için alırsın; sayfalarına dokunmazsın. Yine
// de arka kapaktaki fişe "bir kez daha okundu" diye bir çentik atılır. Kitap senin açından
// değişmemiştir, ama içindeki küçük bir kayıt güncellenmiştir. Rust'ta da metot `&self` ("yalnız
// bakacağım") alır, değişen küçük kayıt ise bir hücrenin içinde durur. Cell küçük değerler
// içindir: al, değiştir, geri koy; panikleyemez. RefCell içine referans verir, nöbetçi denetler.
//   Mini örnek:
//     let sayac = Cell::new(0);            // `mut` yok
//     sayac.set(sayac.get() + 1);          // ✓ al, bir artır, geri koy
//     let liste = RefCell::new(vec![1]);
//     liste.borrow_mut().push(2);          // ✓ nöbetçi izin verdi; satır bitince izin de bitti
//     let duz = 0;
//     // duz += 1;                         // ✗ derlenmez: hücresiz ve `mut`suz değer değişmez
pub trait Bildirimci {
    // Arayüz &self ister: gönderen taraf "bildirim göndermek beni değiştirmez" varsayar.
    fn gonder(&self, mesaj: &str);
}

// Test ikizi (mock): gerçekten göndermek yerine mesajları KAYDEDER. Trait &self verdiği için
// kaydı ancak iç değişkenlikle tutabiliriz — trait'i test uğruna &mut self yapmak, üretim
// kodunun tasarımını teste kurban etmek olurdu.
#[derive(Default)]
pub struct KayitliBildirimci {
    mesajlar: RefCell<Vec<String>>, // yerinde büyüyen koleksiyon → RefCell
    cagri: Cell<u32>,               // küçük Copy sayaç → Cell
}

impl Bildirimci for KayitliBildirimci {
    fn gonder(&self, mesaj: &str) {
        self.cagri.set(self.cagri.get() + 1);
        self.mesajlar.borrow_mut().push(mesaj.to_string());
    } // borrow_mut()'un döndürdüğü RefMut geçiciydi; ifade bitince bırakıldı
}

impl KayitliBildirimci {
    pub fn cagri_sayisi(&self) -> u32 {
        self.cagri.get()
    }
    pub fn mesajlar(&self) -> Vec<String> {
        self.mesajlar.borrow().clone()
    }
}

// Üretim kodu: bildirimcinin hangi tür olduğunu bilmez, yalnız trait'e bağlıdır.
pub struct KotaTakibi<'a, B: Bildirimci> {
    bildirimci: &'a B,
    sinir: u32,
    kullanilan: u32,
}

impl<'a, B: Bildirimci> KotaTakibi<'a, B> {
    pub fn yeni(bildirimci: &'a B, sinir: u32) -> Self {
        KotaTakibi {
            bildirimci,
            sinir: sinir.max(1), // 0 sınır, aşağıdaki yüzde hesabında sıfıra bölme paniği olurdu

            kullanilan: 0,
        }
    }

    pub fn kullan(&mut self, miktar: u32) {
        // Aritmetik güvenliği: debug derlemede u32 taşması panik, release'de sessiz sarma
        // demektir. Toplamı doyurarak (saturating) artırıyor, çarpımı u64'te yapıyoruz.
        self.kullanilan = self.kullanilan.saturating_add(miktar);
        let yuzde = u64::from(self.kullanilan) * 100 / u64::from(self.sinir);
        if yuzde >= 100 {
            self.bildirimci.gonder("Hata: kota aşıldı");
        } else if yuzde >= 75 {
            self.bildirimci.gonder("Uyarı: kotanın %75'i doldu");
        }
    }
}

fn ic_degiskenlik_senaryosu() {
    let bildirimci = KayitliBildirimci::default(); // `mut` YOK — yine de içine kayıt düşecek
    let mut takip = KotaTakibi::yeni(&bildirimci, 100);
    takip.kullan(50); // %50 → bildirim yok
    takip.kullan(30); // %80 → uyarı
    takip.kullan(25); // %105 → hata
    println!(
        "çağrı = {}, mesajlar = {:?}",
        bildirimci.cagri_sayisi(),
        bildirimci.mesajlar()
    );
    // Beklenen çıktı:
    //   çağrı = 2, mesajlar = ["Uyarı: kotanın %75'i doldu", "Hata: kota aşıldı"]
}

// ============================================================================
// 5. YANLIŞ KULLANIM / DOĞRU KULLANIM — RefCell tuzakları
// ============================================================================
// SADE ANLATIM: Nöbetçiden "bakma izni" alıp kâğıdı cebinde unutursan, yazmak isteyen hiç kimse
// içeri giremez. RefCell'de `borrow()` böyle bir izin kâğıdı (guard) verir; kâğıt yaşadıkça
// `borrow_mut()` geri çevrilir ve program panikler. Çare: izni kısa tut — bak, kopyala, bırak,
// sonra yaz. Emin değilsen `try_borrow_mut()` ile sor: panik yerine "olur"/"olmaz" cevabı alırsın.
// Neden böyle? Biri bakarken başkası yazsaydı, bakan kişi yarım değişmiş veri görürdü.
//   Mini örnek:
//     let d = RefCell::new(vec![1]);
//     let bakan = d.borrow();                        // bakma izni açık kaldı
//     println!("{}", d.try_borrow_mut().is_ok());    // ✓ false — sorduk, "olmaz" dedi, panik yok
//     drop(bakan);                                   // izni geri verdik
//     d.borrow_mut().push(2);                        // ✓ artık yazılabilir
fn refcell_tuzaklari() {
    let liste = RefCell::new(vec![1, 2, 3]);

    // --- Tuzak 1: bir borrow'u canlı tutarken ikincisini istemek → ÇALIŞMA ANI paniği --------
    // Derleme anında hata YOK; RefCell denetimi çalışma anına taşıdığı için bedeli budur.
    // try_borrow_mut ile aynı durumu panik yerine Result olarak görebiliriz:
    {
        let okuyucu = liste.borrow(); // paylaşımlı borrow açık
        let deneme = liste.try_borrow_mut(); // aynı anda özel borrow istiyoruz
        println!(
            "okuyucu açıkken (uzunluk {}) borrow_mut alınabildi mi? {}",
            okuyucu.len(),
            deneme.is_ok()
        ); // false — borrow_mut() burada panik ederdi: "RefCell already borrowed"
    } // okuyucu burada düştü
    println!(
        "okuyucu kapandıktan sonra? {}",
        liste.try_borrow_mut().is_ok()
    ); // true

    // Bu panik pratikte en çok şöyle doğar: bir borrow açıkken, içeride yine aynı RefCell'e
    // dokunan başka bir metot çağrılır (özellikle geri çağrılarda / gözlemci listelerinde).
    //   ✗  for x in liste.borrow().iter() { liste.borrow_mut().push(*x); }   // panik
    // ✓ Çözüm: borrow'ları DAR tut — değeri kopyalayıp/hesaplayıp borrow'u kapat, sonra yaz.
    let eklenecek: Vec<i32> = liste.borrow().iter().map(|x| x * 10).collect(); // borrow burada biter
    liste.borrow_mut().extend(eklenecek);
    println!("liste = {:?}", liste.borrow()); // [1, 2, 3, 10, 20, 30]

    // --- Tuzak 2: borrow()'un döndürdüğü geçici guard'dan referans kaçırmak (derleme hatası) --
    //   ✗  let ilk = liste.borrow().first().unwrap();
    //      println!("{ilk}");             // E0716: temporary value dropped while borrowed
    // borrow() bir Ref<T> GUARD'ı döndürür; referans guard'a bağlıdır, guard ise satır sonunda
    // ölen bir geçicidir. ✓ Ya guard'ı bir değişkene bağla, ya da değeri kopyalayıp çıkar:
    let ilk = *liste.borrow().first().unwrap(); // i32 Copy → guard ölmeden değer kopyalandı
    println!("ilk = {ilk}"); // 1

    // --- Tuzak 3: Rc'nin içini doğrudan değiştirmeye çalışmak --------------------------------
    //   ✗  let r = Rc::new(5);        *r += 1;     // E0594: cannot assign to data in an `Rc`
    //   ✗  let v = Rc::new(vec![1]);  v.push(2);   // E0596: cannot borrow data in an `Rc` as mutable
    // Rc yalnız Deref uygular (DerefMut değil): başka sahipler de aynı veriye baktığı için &mut
    // vermesi Aliasing XOR Mutability'yi çiğnerdi. ✓ Rc<RefCell<T>> ya da (aşağıda) Rc::make_mut.

    // --- Tuzak 4: Rc<RefCell<T>>'yi thread'e taşımak ------------------------------------------
    //   ✗  let r = Rc::new(RefCell::new(1));
    //      std::thread::spawn(move || { *r.borrow_mut() += 1; });
    //         // E0277: `Rc<RefCell<i32>>` cannot be sent between threads safely
    //   ✗  Arc<RefCell<i32>> de olmaz:
    //         // E0277: `RefCell<i32>` cannot be shared between threads safely
    // RefCell'in borrow sayacı da Rc'nin sayacı da atomik değildir. ✓ Çok thread'li karşılığı
    // Arc<Mutex<T>> → concurrency crate'i. (Kök neden: ileri-send-sync.)
    // (Sade: "atomik değil" = iki thread sayacı aynı anda değiştirirse sayı bozulabilir demek;
    // Rust bu yüzden Rc ve RefCell'i başka thread'e göndermeni daha derlerken engeller.)
}

// ============================================================================
// 6. Rc::make_mut — "paylaşılıyorsa kopyala" (clone-on-write)
// ============================================================================
// Senaryo: birçok bileşenin paylaştığı bir yapılandırma. Çoğu yalnız okur; biri kendi
// kopyasını değiştirmek isterse ötekiler ETKİLENMEMELİ. RefCell burada yanlış araçtır (herkes
// değişikliği görürdü). Rc::make_mut(&mut rc) şunu yapar:
//   - tek sahip bizsek → doğrudan &mut T verir (kopya yok),
//   - başka sahipler varsa → veriyi klonlar, bizim Rc'yi yeni kopyaya çevirir, ona &mut verir.
// SADE ANLATIM: Sınıfın ders programı panoda asılı ve herkes ona bakıyor. Sen kendi programına
// bir not eklemek istiyorsun. Panodakini karalarsan herkes etkilenir; bunun yerine fotokopisini
// çeker, notu kendi kopyana yazarsın. Panoya senden başka bakan yoksa fotokopiye gerek kalmaz,
// doğrudan yazarsın. `Rc::make_mut` tam bunu yapar: başka sahip varsa önce kopyalar.
//   Mini örnek:
//     let mut benim = Rc::new(vec![1]);
//     let pano = Rc::clone(&benim);           // ikimiz aynı listeye bakıyoruz
//     Rc::make_mut(&mut benim).push(2);       // paylaşılıyordu → önce kopya, sonra yazı
//     println!("{pano:?} {benim:?}");         // ✓ [1] [1, 2] — panodaki liste etkilenmedi
fn paylasiliyorsa_kopyala() {
    let varsayilan = Rc::new(vec![String::from("tema=açık"), String::from("dil=tr")]);
    let mut kullanici_ayari = Rc::clone(&varsayilan); // şimdilik AYNI veriyi paylaşıyor
    println!(
        "değişiklik öncesi aynı veri mi? {}",
        Rc::ptr_eq(&varsayilan, &kullanici_ayari)
    ); // true

    Rc::make_mut(&mut kullanici_ayari)[0] = String::from("tema=koyu"); // paylaşılıyordu → klonlandı
    println!(
        "değişiklik sonrası aynı veri mi? {}",
        Rc::ptr_eq(&varsayilan, &kullanici_ayari)
    ); // false
    println!("varsayılan = {varsayilan:?}"); // ["tema=açık", "dil=tr"] — dokunulmadı
    println!("kullanıcı  = {kullanici_ayari:?}"); // ["tema=koyu", "dil=tr"]

    Rc::make_mut(&mut kullanici_ayari).push(String::from("yazı=büyük")); // artık tek sahip → kopya YOK
    println!("kullanıcı  = {} ayar", kullanici_ayari.len()); // 3
}

// SADE ANLATIM (aşağıdaki bölüm için): Kısa özet ve üç alıştırma var; ana fikir, bedelsiz araç
// (Box, düz referans) yetiyorsa bedelli olana (Rc, RefCell) geçmemek. Alıştırmaları atlayabilirsin.
// ============================================================================
// ÖNEMLİ NOKTALAR
// ============================================================================
// • Her işaretçi güvenliği BAŞKA bir yerde öder: Box derleme anında (bedelsiz), Rc çalışma
//   anında bir sayaçla, RefCell çalışma anında borrow sayacı + olası panikle, Weak her
//   erişimde bir "yaşıyor mu?" sorusuyla. Bedelsiz olan yetiyorsa bedelli olana geçme.
// • Rc::clone(&x) yazımı bilinçlidir: `x.clone()` da derlenir ama okuyana "pahalı derin kopya
//   mı?" diye düşündürür. Rc::clone "yalnız sayaç artıyor" der.
// • Cell panik etmez, RefCell edebilir. Küçük Copy değer için Cell'i tercih et.
// • RefCell borrow'larını tek ifadeye sığdır; bir Ref/RefMut'u değişkende tutup araya başka
//   çağrılar koymak, paniğin en sık kaynağıdır.
// • Rc + RefCell'in tümü TEK THREAD içindir; derleyici thread'e taşımayı reddeder.
//
// ============================================================================
// ALIŞTIRMALAR
// ============================================================================
// 1) `Dizin`e `fn derinlik(&self) -> usize` ekle (kök 0, çocuğu 1, ...). Özyineleme yerine
//    döngüyle yaz.
//    İpucu: `let mut su_an = self.ebeveyn.borrow().upgrade();` ile başla; her adımda
//    `su_an = d.ebeveyn.borrow().upgrade()` — ama bu atama derlenmezse nedenini düşün: `d`
//    hâlâ ödünçteyken `su_an`a yazıyorsun. Ara değişken kullan.
//    Beklenen: /home/kaan ağacında kaan.derinlik() == 2.
//
// 2) Bir "olay yayıncısı" yaz: `Yayinci { aboneler: RefCell<Vec<Weak<dyn Bildirimci>>> }`.
//    `abone_ol(&self, &Rc<dyn Bildirimci>)` Weak saklasın; `yayinla(&self, mesaj)` yaşayan
//    abonelere göndersin ve ölmüş Weak'leri listeden temizlesin.
//    İpucu: önce `retain(|w| w.strong_count() > 0)`, sonra upgrade + gonder. Aboneler neden
//    Rc değil Weak? (Yayıncı aboneyi canlı tutmamalı — abonelikten çıkmayı unutan nesne
//    sonsuza dek yaşardı.)
//
// 3) `cift_olustur(false)` neden 0 döndürüyor? Kâğıt üzerinde a ve b'nin güçlü sayaçlarını
//    adım adım yaz. Sonra döngüyü Weak kullanmadan, kapsam bitmeden ELLE kır:
//    `a.guclu_komsu.borrow_mut().take();` satırını ekleyince sonuç kaç olur? (Beklenen: 2.)
//
// Bir sonraki konuyla bağlantısı: aynı üç sorunun çok thread'li cevapları Arc / Mutex / RwLock
// → concurrency. Hücrelerin hepsinin altındaki tek ilkel UnsafeCell ve Rc'nin neden !Send
// olduğu → ileri-send-sync. Drop sırası ve referans tutan Drop türleri → ileri-dropck.

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn tam_yol_ebeveyn_zincirini_izler() {
        let kok = Dizin::yeni("");
        let etc = Dizin::yeni("etc");
        Dizin::cocuk_ekle(&kok, Rc::clone(&etc));
        assert_eq!(etc.tam_yol(), "/etc");
        assert_eq!(kok.cocuk_sayisi(), 1);
    }

    #[test]
    fn weak_ebeveyni_canli_tutmaz() {
        let cocuk = Dizin::yeni("yetim");
        {
            let ebeveyn = Dizin::yeni("gecici");
            Dizin::cocuk_ekle(&ebeveyn, Rc::clone(&cocuk));
            assert_eq!(cocuk.tam_yol(), "gecici/yetim");
            assert_eq!(Rc::strong_count(&ebeveyn), 1); // çocuktaki Weak güçlü sayacı artırmadı
        }
        assert_eq!(cocuk.tam_yol(), "yetim"); // ebeveyn düştü → upgrade None
    }

    #[test]
    fn dongu_sizdirir_weak_onarir() {
        assert_eq!(cift_olustur(false), 0);
        assert_eq!(cift_olustur(true), 2);
    }

    #[test]
    fn mock_bildirimci_paylasimli_referansla_kayit_tutar() {
        let b = KayitliBildirimci::default();
        let mut takip = KotaTakibi::yeni(&b, 10);
        takip.kullan(5);
        assert_eq!(b.cagri_sayisi(), 0);
        takip.kullan(3);
        takip.kullan(2);
        assert_eq!(
            b.mesajlar(),
            ["Uyarı: kotanın %75'i doldu", "Hata: kota aşıldı"]
        );
    }

    #[test]
    fn refcell_ihlali_hata_olarak_gorulebilir() {
        let hucre = RefCell::new(0);
        let _okuyucu = hucre.borrow();
        assert!(hucre.try_borrow().is_ok()); // çok okuyucu serbest
        assert!(hucre.try_borrow_mut().is_err()); // okuyucu varken yazıcı yok
    }

    #[test]
    #[should_panic(expected = "already borrowed")]
    fn refcell_cift_borrow_mut_panik_eder() {
        let hucre = RefCell::new(0);
        let _a = hucre.borrow_mut();
        let _b = hucre.borrow_mut();
    }

    #[test]
    fn make_mut_yalniz_paylasiliyorsa_kopyalar() {
        let mut a = Rc::new(vec![1]);
        let onceki_adres = Rc::as_ptr(&a);
        Rc::make_mut(&mut a).push(2); // tek sahip → yerinde
        assert_eq!(Rc::as_ptr(&a), onceki_adres);

        let b = Rc::clone(&a);
        Rc::make_mut(&mut a).push(3); // paylaşılıyor → kopya
        assert_eq!(*a, [1, 2, 3]);
        assert_eq!(*b, [1, 2]);
    }
}
