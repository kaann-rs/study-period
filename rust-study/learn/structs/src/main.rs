// Rust Book — Bölüm 5: Structs (yapılar), metotlar, associated function'lar
// Çalıştır: cargo run -p structs
//
// Stil: terim (İngilizce) + Türkçe açıklama + örnek + Neden? + Sorun + rustc: notu.
// ✗ örneklerin hata kodları (E0106, E0382) rustc ile DERLENEREK doğrulandı.
// Bölüm 4'teki ownership kuralları burada struct alanlarına ve metot receiver'larına
// birebir uygulanır.

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

fn main() {
    tanimla_ve_olustur();
    guncelleme_sozdizimi();
    tuple_ve_unit();
    alanlar_sahiplenir();
    metotlar();
    debug_cikti();
}

// 1. Tanımlama ve örnekleme (defining & instantiating)
// field init shorthand: parametre adı alan adıyla aynıysa `ad: ad` yerine yalnız `ad` yazılır.
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
    // Not (doğrulanmış): bir alanı yalnızca {:?} (derive(Debug)) ile basmak, dead_code
    // analizinde "okundu" SAYILMAZ — rustc derive'ın ürettiği impl'i bu analizde kasıtlı
    // yok sayar. Bu yüzden aktif'i ayrıca gerçek kodda okuyoruz, yoksa "field never read" uyarısı çıkar.

    // Neden tek bir alanı "mut" işaretleyemiyorum? Rust'ta mutability bir ALANIN değil,
    // BINDING'in (sahipliğin) özelliğidir: ya tüm instance mut'tur ya hiçbiri. Çünkü
    // `&mut u` elde eden, instance'ın TÜM alanlarına yazma hakkı kazanır — bu, ch4'teki
    // "tek &mut = tek yazar" kuralıyla tutarlıdır. Alan bazlı mut, içsel değişebilirlik
    // (Cell/RefCell) gibi ayrı araçlarla yapılır, o da ileride.
}

// 2. Struct update syntax (`..diger`) ve partial move (kısmi taşıma) TUZAĞI
fn guncelleme_sozdizimi() {
    let u1 = kullanici_olustur("Ada".into(), "a@x.com".into());
    // Kalan alanları u1'den al: struct update syntax.
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
fn tuple_ve_unit() {
    let siyah = Renk(0, 0, 0);
    println!("R={} G={} B={}", siyah.0, siyah.1, siyah.2); // .indeks ile erişim

    let _b = Birim; // alanı yok; genelde bir trait'i implemente etmek için "marker" olarak kullanılır

    // Neden tuple struct, sıradan tuple yerine? (u8, u8, u8) ile Renk(u8, u8, u8) farkı:
    // Renk kendi TÜRÜdür. Renk isteyen bir fonksiyona yanlışlıkla rastgele bir üçlü tuple
    // (ya da bir Konum(u8,u8,u8)) veremezsin → tür güvenliği (type safety).
}

// 4. Struct alanları sahipliği + referans alan için lifetime
fn alanlar_sahiplenir() {
    // Alanlar varsayılan olarak veriyi SAHİPLENİR (ad: String gibi). Böylece struct yaşadığı
    // sürece alanları da geçerli kalır — struct, verisinin sahibidir.
    let s = String::from("merhaba dünya");
    let a = Alinti { metin: &s[0..7] }; // referans tutan alan → 'a ömrü = s'in ömrü
    println!("alıntı: {}", a.metin);

    // Referans alan neden lifetime İSTER?
    //   struct S { r: &str }        // ✗ E0106: missing lifetime specifier
    //   struct S<'a> { r: &'a str } // ✓  'a: "r'nin işaret ettiği veri, S'ten uzun yaşamalı"
    // Sorun: referans veriyi sahiplenmez; işaret ettiği veri (burada s) struct'tan önce
    // ölürse referans dangling olur. Lifetime, derleyiciye "bu referans en az struct kadar
    // yaşayacak" sözünü verdirtir; borrowck bunu denetler. (Alinti<'a> burada tam bu yüzden var.)
    // rustc: lifetime'lar da tür denetiminin parçası; ihlali MIR üzerinde rustc_borrowck yakalar.
}

// 5. Metotlar ve associated function'lar — receiver ownership (ch4 ile bağ)
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
    // Neden metotların çoğu &self? Sorun: her metot self alsaydı (tüketseydi), bir nesne
    // üzerinde iki metot çağıramazdın (ilki onu yerdi). &self sahiplenmeden okur → nesne yaşar.
}

// 6. #[derive(Debug)] ve hata ayıklama çıktısı
fn debug_cikti() {
    let d = Dikdortgen::yeni(6, 7);
    println!("{d:?}");  // tek satır
    println!("{d:#?}"); // "pretty" çok satırlı
    // Neden #[derive(Debug)]? Sorun: Debug'ı elle yazmak (impl std::fmt::Debug for ...) her
    // struct için tekrar eden, hataya açık bir iştir. derive bir PROC-MACRO'dur: derleme
    // anında fmt::Debug implementasyonunu SENİN İÇİN üretir (makrolar bölümüne bağlanır).
}

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
// • Bellek düzeni (layout): repr(Rust)'ta (varsayılan) struct alanlarının sırası GARANTİ
//   DEĞİLDİR — derleyici, boşluğu (padding) azaltmak için alanları yeniden sıralayabilir.
//   Sabit, C-uyumlu sıra gerekiyorsa `#[repr(C)]` kullanılır. Hesaplama: rustc_abi/src/
//   layout.rs (satır ~35 yorumu: sıra yalnız repr(C) struct'larda korunur).
//
// • derive(Debug): bir procedural macro; kaynağa fmt::Debug impl'ini makro genişletme
//   aşamasında enjekte eder (sen yazmazsın).
