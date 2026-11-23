// Açıklama: learn/structs/ACIKLAMA.md
// Rust Book — Bölüm 5: Structs (yapılar), metotlar, associated function'lar
// Çalıştır: cargo run -p structs
//
// Stil: terim (İngilizce) + Türkçe açıklama + örnek + Neden? + Sorun + rustc: notu.
// ✗ örneklerin hata kodları (E0106, E0382) rustc ile DERLENEREK doğrulandı.
// Bölüm 4'teki ownership kuralları burada struct alanlarına ve metot receiver'larına
// birebir uygulanır.

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Her bölümde önce "SADE ANLATIM" kutusunu oku, sonra teknik açıklamaya geç.
// Benzetmelerin hepsi aynı şeyle, bir ÖĞRENCİ KARTIYLA kurulur.
//
// Bu bölümün tek cümlelik özeti: struct, birbiriyle ilgili birkaç bilgiyi (ad, sınıf, numara)
// tek bir kartta toplar; metotlar da o kartla yapılabilecek işlerdir.
//
// MİNİ SÖZLÜK (bu dosyada geçen kelimeler):
//   struct (yapı)          → üstünde adlı gözler bulunan boş kart KALIBI. Kendi başına bir türdür.
//   alan (field)           → karttaki gözlerden biri: `ad`, `sinif` gibi.
//   instance (örnek)       → kalıptan çıkarılıp doldurulmuş tek bir kart.
//   field init shorthand   → değişkenin adı gözün adıyla aynıysa `ad: ad` yerine `ad` yazmak.
//   struct update syntax   → `..eski`: "yazmadığım gözleri eski karttan al".
//   partial move           → kartın tamamını değil, yalnız bazı gözlerini başka yere taşımak.
//   tuple struct           → gözlerinin adı olmayan, yalnız sırası olan kart: Renk(0, 0, 0).
//   unit-like struct       → hiç gözü olmayan kart; bilgi taşımaz, yalnız bir işarettir.
//   impl                   → "bu kartla şunlar yapılabilir" diye metotların yazıldığı blok.
//   metot                  → bir karta bağlı fonksiyon; `kart.is()` diye çağrılır.
//   self, Self             → self: metodun çağrıldığı kartın kendisi. Self: o kartın türü.
//   receiver (alıcı)       → metodun ilk parametresi: `self`, `&self` ya da `&mut self`.
//   associated function    → karta değil kalıba bağlı fonksiyon; `Tur::ad()` diye çağrılır.
//   constructor (yapıcı)   → yeni kart üreten fonksiyon; adı çoğunlukla `new` ya da `yeni` olur.
//   String, &str           → sahibi olunan yazı / başka yerdeki bir yazıya bakış.
//   u8, u32, u64           → eksi olamayan tam sayı türleri (8, 32, 64 bitlik).
//   Copy, move             → Copy trait'i olan türler kopyalanır (sayı, &T, Option<T>); String/Vec taşınır (move).
//   lifetime ('a)          → "bu ok, gösterdiği veri yaşadığı sürece geçerlidir" diyen etiket.
//   dangling               → gösterdiği veri silinmiş, boşu gösteren ok.
//   trait                  → türlerin ortak yeteneği. Debug: "ekrana yazdırılabilir" yeteneği.
//   derive                 → derleyiciye "bu yeteneğin kodunu benim yerime sen yaz" demek.
//   makro                  → kod üreten kod. `println!` ve `#[derive(...)]` birer makrodur.
//   `.into()`              → bir türü, beklenen türe çevirir: "Ada".into() burada String olur.
//   type safety            → derleyicinin, bir türün yerine yanlış türü vermeni engellemesi.
//   layout, padding        → kartın bellekteki yerleşimi ve gözler arasında kalan boşluk.
//   derleyici (rustc)      → kodu programa çeviren ve hataları önceden bulan araç.
//
// SADE ANLATIM (tanımlar için): Buradaki beş `struct` satırı kart değil, kart KALIPLARIDIR:
// "bir Kullanici kartında şu dört göz bulunur" der. Kalıp bir kez tanımlanır, ondan istediğin
// kadar kart doldurulur; işi aşağıdaki bölümlerde anlatılır, şimdilik adlarına bak.

#[derive(Debug)]
struct Kullanici {
    ad: String,
    eposta: String,
    aktif: bool,
    giris: u64,
}

// Referans tutan alan için LIFETIME zorunludur (aşağıda anlatılıyor).
struct Alinti<'a> {
    metin: &'a str,
}

// Tuple struct: alanların adı yok, sırası var. Kendi TÜRÜdür.
struct Renk(u8, u8, u8);

// Unit-like struct: hiç alanı yok.
struct Birim;

#[derive(Debug)]
struct Dikdortgen {
    en: u32,
    boy: u32,
}

// SADE ANLATIM: `impl` bloğu kartla yapılabilecek işlerin listesidir, her biri bir metottur.
// Hangi receiver'ın okuduğunu, değiştirdiğini ya da kartı aldığını 5. bölümdeki kutu anlatır.
impl Dikdortgen {
    // associated function (self YOK): tür üzerinden çağrılır, genelde yapıcı (constructor).
    fn yeni(en: u32, boy: u32) -> Self {
        Self { en, boy } // Self = Dikdortgen; field init shorthand (en: en → en)
    }
    // &self: receiver'ı ödünç alır, sahiplenmez, değiştirmez — en yaygın metot biçimi.
    fn alan(&self) -> u32 {
        self.en * self.boy
    }
    // &mut self: değiştirmek için ödünç alır.
    fn olcekle(&mut self, k: u32) {
        self.en *= k;
        self.boy *= k;
    }
    // self: receiver'ı TÜKETİR (sahipliğini alır); çağrıdan sonra orijinal kullanılamaz.
    fn yok_et(self) -> (u32, u32) {
        (self.en, self.boy)
    }
}

mod derinlesme;
fn main() {
    tanimla_ve_olustur();
    guncelleme_sozdizimi();
    tuple_ve_unit();
    alanlar_sahiplenir();
    metotlar();
    debug_cikti();
    derinlesme::calistir();
}

// 1. Tanımlama ve örnekleme (defining & instantiating)
// SADE ANLATIM: Okul, öğrenci kartları için bir KALIP hazırlar: "ad" gözü, "sınıf" gözü. Bu kalıp
// `struct` tanımıdır. Kalıptan bir kart çıkarıp gözlerini doldurunca elinde gerçek bir kart
// (instance) olur. Gözlere nokta ile ulaşırsın: `kart.ad`.
// Neden böyle? Kart olmasaydı ad, sınıf ve numarayı üç ayrı değişkende taşırdın; hangisinin kime
// ait olduğu kolayca karışırdı. Kart, ilgili bilgileri bir arada tutar.
//   Mini örnek:
//     struct Kart { ad: String, sinif: u8 }
//     let mut k = Kart { ad: String::from("Ada"), sinif: 3 };   // kalıptan bir kart doldurduk
//     println!("{}", k.ad);     // ✓ Ada
//     k.sinif = 4;              // ✓ k `mut` olduğu için göz değiştirilebilir
//     let sabit = Kart { ad: String::from("Can"), sinif: 3 };
//     sabit.sinif = 4;          // ✗ derlenmez: `mut` yok, kartın hiçbir gözü değişmez
fn kullanici_olustur(ad: String, eposta: String) -> Kullanici {
    Kullanici {
        ad,
        eposta,
        aktif: true,
        giris: 1,
    }
}

fn tanimla_ve_olustur() {
    let mut u = kullanici_olustur("Kaan".into(), "k@x.com".into());
    u.giris += 1; // değiştirmek için instance mut olmalı
    println!("{u:?}");
    println!("{} giriş: {} aktif: {}", u.ad, u.giris, u.aktif);
    // (Sade: dead_code = derleyicinin "bunu yazmışsın ama hiç kullanmamışsın" uyarısı.)
    // Not (doğrulanmış): bir alanı yalnızca {:?} (derive(Debug)) ile basmak, dead_code
    // analizinde "okundu" SAYILMAZ — rustc derive'ın ürettiği impl'i bu analizde kasıtlı
    // yok sayar. Bu yüzden aktif'i ayrıca gerçek kodda okuyoruz, yoksa "field never read" uyarısı çıkar.

    // Neden tek bir alanı "mut" işaretleyemiyorum? Rust'ta mutability bir ALANIN değil,
    // BINDING'in (sahipliğin) özelliğidir: ya tüm instance mut'tur ya hiçbiri. Çünkü
    // `&mut u` elde eden, ödünç kuralları gereği instance'ın TÜM alanlarına yazabilir
    // (standart kurallardan çıkarım; özel repro yok) — bu, ch4'teki "tek &mut = tek yazar"
    // kuralıyla tutarlıdır. Alan bazlı mut, içsel değişebilirlik
    // (Cell/RefCell) gibi ayrı araçlarla yapılır, o da ileride.
}

// 2. Struct update syntax (`..diger`) ve partial move (kısmi taşıma) TUZAĞI
// SADE ANLATIM: Yeni bir kart doldururken "sınıfı ben yazıyorum, gerisini eski karttan al"
// diyebilirsin: `..eski`. Dikkat: eski karttaki fotoğraf sökülüp yeni karta yapıştırılır, eski
// kartta kalmaz. Sayılar ise yeniden yazılır, eskisinde de durur. Rust'ta fotoğraf `String`
// gibi taşınan alanlara, sayı ise kopyalanan (Copy) alanlara karşılık gelir.
// Neden böyle? Aynı fotoğraf iki kartta birden sayılsaydı, iki kart da atılırken onu ayrı ayrı
// yok etmeye çalışırdı. Taşıyınca fotoğrafın tek sahibi kalır.
//   Mini örnek:
//     let eski = Kart { ad: String::from("Ada"), sinif: 3 };
//     let yeni = Kart { sinif: 4, ..eski };   // ad eski karttan söküldü, yeni karta geçti
//     println!("{}", yeni.ad);     // ✓ Ada
//     println!("{}", eski.sinif);  // ✓ 3 → sayı kopyalanmıştı, eskide de duruyor
//     println!("{}", eski.ad);     // ✗ derlenmez: ad artık yeni kartta
fn guncelleme_sozdizimi() {
    let u1 = kullanici_olustur("Ada".into(), "a@x.com".into());
    let u2 = Kullanici {
        ad: "Ada II".into(),
        ..u1
    };
    println!("{} / {}", u2.ad, u2.eposta);

    // DİKKAT: `..u1` bir KOPYA değildir. Kalan alanlardan non-Copy olanları (eposta: String)
    // u2'ye TAŞIR; u1.eposta artık geçersizdir (u1 kısmen taşınmış sayılır):
    //   println!("{}", u1.eposta); // ✗ E0382: borrow of moved value: `u1.eposta`
    // (u1.aktif ve u1.giris Copy olduğundan onlar hâlâ okunabilirdi.)
    // Neden? Çünkü ch4 aynen geçerli: non-Copy bir alanı başka bir yere koymak move'dur,
    // çift sahiplik (ve double free) olmasın diye kaynak geçersizleşir.
}

// 3. Tuple struct ve unit-like struct
// SADE ANLATIM: Bazı kartların gözlerine ad vermeye değmez: bir renk "kırmızı, yeşil, mavi" diye
// üç sayıdır, sırası yeter. Bu `tuple struct`tır. Yine de kartın bir ADI vardır: "Renk" kartı ile
// "Konum" kartının ikisi de üç sayı taşısa bile biri ötekinin yerine verilemez. Hiç gözü olmayan
// kart da olur (`unit-like struct`): üstünde yazı yoktur, yalnız "ben şu cins kartım" der.
// Neden böyle? Ad olmasaydı renk bekleyen yere yanlışlıkla konum verebilirdin; ikisi de üç sayı.
//   Mini örnek:
//     struct Renk(u8, u8, u8);
//     struct Konum(u8, u8, u8);
//     fn boya(r: Renk) { println!("{}", r.0); }   // .0 → ilk sayı
//     boya(Renk(255, 0, 0));       // ✓
//     boya(Konum(255, 0, 0));      // ✗ derlenmez: Konum bir Renk değildir
//     boya((255, 0, 0));           // ✗ derlenmez: adsız üçlü de Renk değildir
fn tuple_ve_unit() {
    let siyah = Renk(0, 0, 0);
    println!("R={} G={} B={}", siyah.0, siyah.1, siyah.2); // .indeks ile erişim

    let _b = Birim; // alanı yok; genelde bir trait'i implemente etmek için "marker" olarak kullanılır

    // Neden tuple struct, sıradan tuple yerine? Renk kendi TÜRÜdür: Renk isteyen bir fonksiyona
    // rastgele bir üçlü ya da Konum(u8,u8,u8) veremezsin → tür güvenliği (type safety).
}

// 4. Struct alanları sahipliği + referans alan için lifetime
// SADE ANLATIM: Karta bir bilgiyi iki yoldan koyabilirsin. Ya bilgiyi kartın ÜSTÜNE yazarsın
// (kart bilginin sahibidir), ya da karta "bu bilgi şu defterde yazıyor" diye bir not düşersin
// (referans). İkinci yolda defter atılırsa kart boşu gösterir. Rust bu yüzden senden bir söz
// ister: "defter, karttan önce atılmayacak". Bu sözün yazılışı `'a` (lifetime) etiketidir.
// Neden böyle? Söz olmasaydı kart, silinmiş bir yeri veri sanıp okuyabilirdi.
//   Mini örnek:
//     struct Sahipli { ad: String }             // ✓ bilgi kartın üstünde
//     struct Notlu<'a> { ad: &'a str }          // ✓ bilgi başka yerde; 'a sözü verildi
//     struct Eksik { ad: &str }                 // ✗ derlenmez: ok var ama söz ('a) yok
fn alanlar_sahiplenir() {
    // Alanlar varsayılan olarak veriyi SAHİPLENİR (ad: String gibi), struct yaşadığı sürece geçerli.
    let s = String::from("merhaba dünya");
    let a = Alinti { metin: &s[0..7] }; // referans tutan alan → 'a ömrü = s'in ömrü
    println!("alıntı: {}", a.metin);

    // Referans alan neden lifetime İSTER?
    //   struct S { r: &str }        // ✗ E0106: missing lifetime specifier
    //   struct S<'a> { r: &'a str } // ✓  'a: "r'nin işaret ettiği veri, S'ten uzun yaşamalı"
    // Sorun: referans veriyi sahiplenmez; veri (burada s) struct'tan önce ölürse dangling olur.
    // Lifetime bu sözü verdirtir; borrowck denetler. (Alinti<'a> tam bu yüzden var.)
    // rustc: lifetime'lar da tür denetiminin parçası; ihlali MIR üzerinde rustc_borrowck yakalar.
}

// 5. Metotlar ve associated function'lar — receiver ownership (ch4 ile bağ)
// SADE ANLATIM: Kart sahibinden üç türlü şey istenebilir. "Kartını göster": bakılır, kart sende
// kalır (`&self`). "Kartındaki sınıfı güncelleyelim": kart yerinde değişir, yine sende kalır
// (`&mut self`). "Kartını teslim et": kart elinden çıkar, bir daha kullanamazsın (`self`).
// Henüz ortada kart yokken yapılan iş de vardır: "yeni kart bas". Buna associated function denir
// ve kalıbın adıyla çağrılır: `Kart::yeni(...)`.
// Neden böyle? Her istek kartı elinden alsaydı, kartını bir kez gösterdikten sonra kartsız
// kalırdın. Metotların çoğu bu yüzden yalnız bakar (`&self`).
//   Mini örnek:
//     impl Kart {
//         fn yeni(ad: &str) -> Self { Self { ad: ad.to_string(), sinif: 1 } }
//         fn goster(&self) -> &str { &self.ad }           // yalnız bakar
//         fn sinif_atla(&mut self) { self.sinif += 1; }   // yerinde değiştirir
//         fn teslim_et(self) -> String { self.ad }        // kartı alır
//     }
//     let mut k = Kart::yeni("Ada");
//     k.sinif_atla();              // ✓ sınıf 2 oldu
//     let ad = k.teslim_et();      // ✓ kart teslim edildi
//     k.goster();                  // ✗ derlenmez: k artık elimizde değil
fn metotlar() {
    let mut d = Dikdortgen::yeni(3, 4); // associated function :: ile çağrılır (self yok)
    println!("alan = {}", d.alan());    // &self: `d.alan()` aslında `Dikdortgen::alan(&d)`
    d.olcekle(2);                       // &mut self: d mut olmalı
    println!("ölçekli alan = {} ({:?})", d.alan(), d);

    let (en, boy) = d.yok_et();         // self: d burada TÜKETİLİR
    println!("yok edildi: {en}x{boy}");
    // d.alan(); // ✗ E0382: use of moved value `d` — yok_et(self) d'nin sahipliğini aldı

    // Otomatik referanslama/deref (automatic referencing & dereferencing): `d.alan()` yazınca
    // Rust, metodun &self istediğini görüp çağrıyı `(&d).alan()` diye düzenler; & yazmana gerek
    // yok. &mut ve * için de aynı otomatik ayarı yapar.
    // Neden çoğu metot &self? Sorun: her metot self alsaydı ilki nesneyi yerdi; &self sahiplenmeden okur.
}

// 6. #[derive(Debug)] ve hata ayıklama çıktısı
// SADE ANLATIM: Kartın üstünde yazanları ekrana dökmek için birinin kartı gözünden gözüne okuması
// gerekir. Her kart türü için bu okuma kodunu elle yazmak sıkıcıdır. Kalıbın üstüne
// `#[derive(Debug)]` yazarsan derleyici bu kodu senin yerine üretir; sonra `{:?}` ile kartı
// yazdırırsın. Hata ararken "bu kartın içinde şu an ne var" diye bakmanın en kısa yolu budur.
//   Mini örnek:
//     #[derive(Debug)]
//     struct Kart { ad: String, sinif: u8 }
//     let k = Kart { ad: String::from("Ada"), sinif: 3 };
//     println!("{k:?}");    // ✓ Kart { ad: "Ada", sinif: 3 }
//     println!("{k}");      // ✗ derlenmez: {} için Display gerekir, o derive ile gelmez
fn debug_cikti() {
    let d = Dikdortgen::yeni(6, 7);
    println!("{d:?}");  // tek satır
    println!("{d:#?}"); // "pretty" çok satırlı
    // (Sade: proc-macro = derleme sırasında çalışıp senin yerine kod yazan küçük program.)
    // Neden #[derive(Debug)]? Sorun: Debug'ı elle yazmak (impl std::fmt::Debug for ...) her struct için
    // tekrar eden, hataya açık iştir. Debug/Clone/PartialEq gibi STANDART derive'lar derleyiciye gömülü
    // (built-in) makrolardır; kullanıcıların yazdığı derive'lar (serde::Serialize gibi) gerçek proc-macro'dur.
    // Derleme anında fmt::Debug implementasyonunu SENİN İÇİN üretir (makrolar bölümüne bağlanır).
}

// SADE ANLATIM (aşağıdaki bölüm için): Perde arkası: `&self` kısaltmadır, `kart.goster()` yazınca
// derleyici `&` koyar; alan sırası garanti değildir (yer kazanmak için değişir); derive kodu derleme
// sırasında üretilir. Program yazarken gerekmez; ilk okuyuşta atla, altı bölüm yeter.
// ============================================================================
// DERLEYİCİ PERSPEKTİFİ — struct ve metotlar nasıl çalışıyor?
// ============================================================================
//
// • Metot = ilk parametresi "receiver" olan bir fonksiyon. `self`/`&self`/`&mut self`
//   birer KISALTMADIR:
//       self        ==  self: Self
//       &self       ==  self: &Self
//       &mut self   ==  self: &mut Self
//   Yani ch4'teki ownership/borrow kuralları receiver'a aynen uygulanır: self tüketir,
//   &self paylaşımlı ödünç alır, &mut self özel (tek) ödünç alır.
//
// • Metot çağrısı çözümü (method resolution): `d.alan()` için derleyici bir aday zinciri
//   dener — önce receiver'ı olduğu gibi, sonra &d, &mut d, sonra Deref ile açılmış hâlleri...
//   İlk uyan metodu seçer. Kaynak: rustc_hir_typeck/src/method/probe.rs (adayları toplar) +
//   confirm.rs (seçimi kesinleştirir). "Otomatik referanslama" dediğimiz şey budur.
//
// (Sade: padding = sayıların bellekte düzgün hizalanması için gözler arasında bırakılan boşluk.)
// • Bellek düzeni (layout): repr(Rust)'ta (varsayılan) struct alanlarının sırası GARANTİ
//   DEĞİLDİR — derleyici, boşluğu (padding) azaltmak için alanları yeniden sıralayabilir.
//   Sabit, C-uyumlu sıra gerekiyorsa `#[repr(C)]` kullanılır. Hesaplama: rustc_abi/src/
//   layout.rs (satır ~35 yorumu: sıra yalnız repr(C) struct'larda korunur).
//
// • derive(Debug): derleyiciye gömülü (built-in) derive makrosu; kaynağa fmt::Debug impl'ini
//   makro genişletme aşamasında enjekte eder (sen yazmazsın). Kullanıcı derive'ları (serde vb.)
//   ise gerçek proc-macro crate'lerindendir.
