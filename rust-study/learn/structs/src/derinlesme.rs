// Derinleşme — Bölüm 5: Structs (pekiştirme: gerçekçi senaryo, sık hatalar, alıştırmalar)
// Çalıştır: cargo run -p structs   |   Test: cargo test -p structs
//
// main.rs struct'ın SÖZDİZİMİNİ anlattı: tanım, update syntax, üç receiver, derive(Debug).
// Bu dosya aynı araçlarla bir TASARIM yapar — küçük bir kullanıcı yönetimi modülü:
//   • private alan + doğrulayan yapıcı → değişmez (invariant) tek yerde denetlenir, hep geçerli kalır
//   • builder kalıbı → çok alanlı bir struct'ı okunur ve eksiksiz kurmak
//   • receiver seçimi → metodun çağırana verdiği sahiplik sözü (`self` ile durum değişimi)
//   • `..taban` → tam olarak hangi alan taşınır, kaynak ne zaman kullanılabilir kalır

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Önce main.rs'i oku; bu dosya onun devamıdır. Buradaki her bölümün üstünde de bir "SADE ANLATIM"
// kutusu var. main.rs'teki öğrenci kartı benzetmesi sürer; bu kez kartları basan KAYIT BÜROSUNA
// bakıyoruz.
//
// Bu dosyanın tek cümlelik özeti: Bir kartı yalnız büronun basmasına izin verirsen, eline geçen
// her kartın denetimden geçtiğine güvenebilirsin.
//
// MİNİ SÖZLÜK (yalnız bu dosyada yeni geçenler; öteki terimler main.rs başındaki sözlükte):
//   modül (mod)            → kodun, kapısı olan bir odası. İçindekiler dışarıdan görünmeyebilir.
//   pub / private          → pub: kapısı açık, herkes kullanır. private: yalnız oda içi kullanır.
//   gizlilik (privacy)     → neyin odanın dışından görülebileceğini belirleyen kural.
//   değişmez (invariant)   → hiçbir an bozulmaması gereken söz: "bu kartta geçerli e-posta var".
//   doğrulamak             → bir bilginin kurallara uyup uymadığına bakmak.
//   newtype                → tek bir değeri sarıp ona yeni tür adı veren struct: Eposta(String).
//   builder (kurucu) kalıbı→ çok gözlü bir kartı adım adım dolduran yardımcı tür.
//   zincirleme             → metotları art arda nokta ile çağırmak: a.b().c().d()
//   Result, Ok, Err        → "iş başarıldı, sonuç bu" (Ok) ya da "olmadı, sebep bu" (Err) kutusu.
//   Option, Some, None     → "bir şey var" (Some) ya da "yok" (None) diyen kutu.
//   `?`                    → "hata varsa hemen onu geri döndür, yoksa devam et" kısaltması.
//   match                  → değerin hâllerine tek tek bakıp her birinde ne yapılacağını yazmak.
//   tüketmek (consume)     → bir değeri alıp bitirmek; sonra o değer kullanılamaz.
//   Clone, PartialEq, Default → derive ile gelen yetenekler: kopyalanabilme, `==` ile
//                            karşılaştırılabilme, "varsayılan hâl" üretebilme.
//   test, assert           → kodun doğru çalıştığını denetleyen küçük sınav ve o sınavın sorusu.

pub fn calistir() {
    println!("\n== Derinleşme: Structs — kullanıcı yönetimi senaryosu ==");
    degismez_koruma();
    builder_kalibi();
    alici_secimi();
    guncelleme_tuzagi();
}

// Bu konuyu neden öğreniyoruz?
// main.rs'teki Kullanici'nın alanlarına aynı dosyadaki her kod serbestçe yazabiliyordu;
// `eposta: "asdf".into()` derlenirdi. Gerçek projede asıl soru şudur: bu struct'ın bir değeri
// elimdeyse NEYE güvenebilirim? Gizlilik (privacy) varsayılanı bunun aracıdır: bir alan `pub`
// değilse yalnızca tanımlandığı modül (ve alt modülleri) ona dokunabilir. Kuralı modülün
// içinde bir kez yazarsın; modülün dışındaki hiçbir kod onu çiğneyen bir değer üretemez.
// SADE ANLATIM: Öğrenci kartını kendin yazamazsın; yalnız kayıt bürosu basar. Büro önce bilgini
// denetler, uygunsa kartı verir. Bu yüzden elinde kart olan herkesin bilgisi denetlenmiştir;
// kartı gören kimsenin yeniden denetlemesi gerekmez. Rust'ta büro bir modüldür (`mod hesap`),
// kartın gözleri `pub` yazılmadığı için dışarıya kapalıdır ve kart yalnız büronun verdiği
// fonksiyonla (`ayristir`) üretilir.
// Neden böyle? Gözler herkese açık olsaydı, programın herhangi bir satırı karta geçersiz bir
// e-posta yazabilirdi; karta güvenen bütün kod sessizce yanlış çalışırdı.
//   Mini örnek:
//     mod buro {
//         pub struct Numara(u32);                  // içindeki sayı dışarıya kapalı
//         impl Numara {
//             pub fn yeni(n: u32) -> Option<Numara> {
//                 if n > 0 { Some(Numara(n)) } else { None }   // denetim tek yerde
//             }
//         }
//     }
//     let n = buro::Numara::yeni(7);    // ✓ Some(..) → büro denetledi ve bastı
//     let k = buro::Numara(0);          // ✗ derlenmez: kartı yalnız büro basar
mod hesap {
    // (Sade: aşağıdaki Eposta, içinde tek bir yazı bulunan karttır. Sıradan bir String'den farkı:
    //  ancak denetimden geçen yazı Eposta olabilir.)
    // Newtype: tek alanlı tuple struct. İçindeki String private → dışarıdan ne kurulabilir
    // ne değiştirilebilir. Değişmez: "boşluksuz, küçük harfli, tek '@' içeren, '@' öncesi boş olmayan ve
    // alan adında nokta bulunan adres". (Gerçek e-posta doğrulaması daha katıdır; amaç kalıbı göstermek.)
    #[derive(Debug, Clone, PartialEq)]
    pub struct Eposta(String);

    impl Eposta {
        // Bir Eposta üretmenin TEK yolu. Denetim burada geçilmeden değer oluşmadığı için
        // `Eposta` alan her fonksiyon doğrulamayı tekrarlamak zorunda kalmaz.
        // (Result'ı enums ve error-handling crate'lerinde ayrıntılı göreceğiz.)
        pub fn ayristir(ham: &str) -> Result<Eposta, String> {
            let ham = ham.trim();
            let gecerli = ham.split_once('@').is_some_and(|(yerel, alan)| {
                !yerel.is_empty()
                        && alan.contains('.')
                        && !alan.contains('@') // tek '@'
                        && !ham.contains(char::is_whitespace)
            });
            if gecerli {
                Ok(Eposta(ham.to_lowercase()))
            } else {
                Err(format!("geçersiz e-posta: '{ham}'"))
            }
        }

        // Salt okunur erişim: &str verir, String'in kendisini (ve &mut'unu) vermez.
        pub fn as_str(&self) -> &str {
            &self.0
        }

        // Değişmezin getirisi: '@' kesin var, o yüzden burada hata dalı tasarlamaya gerek yok.
        pub fn alan_adi(&self) -> &str {
            self.0.split_once('@').map_or("", |(_, alan)| alan)
        }
    }

    // Alanların hepsi private: dışarıdan `Kullanici { .. }` yazılamaz, yalnız kurucu ile kurulur.
    #[derive(Debug)]
    pub struct Kullanici {
        ad: String,
        eposta: Eposta,
        yonetici: bool,
        giris_sayisi: u32,
    }

    // (Sade: KullaniciKurucu, kart basılmadan önce doldurulan başvuru formudur. Henüz doldurulmamış
    //  göz None'dır; form teslim edilince (`olustur`) eksik göz var mı diye bakılır.)
    // Builder: zorunlu alanlar Option ile "henüz verilmedi" olarak başlar, olustur() denetler.
    #[derive(Debug, Default)]
    pub struct KullaniciKurucu {
        ad: Option<String>,
        eposta: Option<String>,
        yonetici: bool,
    }

    impl KullaniciKurucu {
        // `mut self` alıp `Self` döndürmek zincirlemeyi sağlar: her adım kurucuyu devralır,
        // günceller, bir sonrakine teslim eder. Arada kopya da ödünç de yok.
        pub fn ad(mut self, ad: &str) -> Self {
            self.ad = Some(ad.trim().to_string());
            self
        }
        pub fn eposta(mut self, eposta: &str) -> Self {
            self.eposta = Some(eposta.to_string());
            self
        }
        pub fn yonetici(mut self, yonetici: bool) -> Self {
            self.yonetici = yonetici;
            self
        }
        // (Sade: aşağıdaki satır sonlarındaki `?` şunu der: "hata çıktıysa onu hemen geri döndür,
        //  çıkmadıysa içindeki değeri ver ve devam et".)
        // `self` tüketilir: kurucunun içindeki String'ler Kullanici'ya TAŞINIR (clone yok).
        pub fn olustur(self) -> Result<Kullanici, String> {
            let ad = self
                .ad
                .filter(|ad| !ad.is_empty())
                .ok_or_else(|| String::from("ad zorunlu"))?;
            let ham = self.eposta.ok_or_else(|| String::from("e-posta zorunlu"))?;
            Ok(Kullanici {
                ad,
                eposta: Eposta::ayristir(&ham)?,
                yonetici: self.yonetici,
                giris_sayisi: 0,
            })
        }
    }

    impl Kullanici {
        pub fn kurucu() -> KullaniciKurucu {
            KullaniciKurucu::default()
        }
        // &self: yalnız okur. Dönen &str, self'in ödüncüne bağlıdır (kullanıcıdan uzun yaşayamaz).
        pub fn ad(&self) -> &str {
            &self.ad
        }
        pub fn eposta(&self) -> &Eposta {
            &self.eposta
        }
        pub fn giris_sayisi(&self) -> u32 {
            self.giris_sayisi
        }
        pub fn ozet(&self) -> String {
            let rol = if self.yonetici { "yönetici" } else { "üye" };
            let Kullanici {
                ad,
                eposta,
                giris_sayisi,
                ..
            } = self;
            format!("{ad} <{}> [{rol}], {giris_sayisi} giriş", eposta.as_str())
        }
        // &mut self: sayacı yalnız bu metot artırır → "aktif bir hesabın giriş sayısı azalmaz" da bir
        // değişmez (yeniden etkinleştirme bilinçli olarak sıfırdan YENİ bir hesap dönemi başlatır).
        pub fn giris_kaydet(&mut self) {
            self.giris_sayisi += 1;
        }
        // (Sade: pasifleştirmek, öğrenci kartını geri verip yerine başka CİNS bir kart almaktır.
        //  Yeni kartın üstünde "giriş kaydet" diye bir iş yazmadığı için onu isteyemezsin.)
        // self: aktif kullanıcıyı TÜKETİP başka bir tür döndürür. `aktif: bool` bayrağı yerine
        // ayrı tür: pasif kullanıcıda giris_kaydet diye bir metot YOK, yanlışlıkla çağrılamaz.
        pub fn pasiflestir(self) -> PasifKullanici {
            PasifKullanici {
                ad: self.ad,
                eposta: self.eposta,
                son_giris_sayisi: self.giris_sayisi,
            }
        }
    }

    #[derive(Debug)]
    pub struct PasifKullanici {
        ad: String,
        eposta: Eposta,
        son_giris_sayisi: u32,
    }

    impl PasifKullanici {
        pub fn ozet(&self) -> String {
            format!(
                "[pasif] {} <{}>, son giriş sayısı {}",
                self.ad,
                self.eposta.as_str(),
                self.son_giris_sayisi
            )
        }
        pub fn yeniden_etkinlestir(self) -> Kullanici {
            Kullanici {
                ad: self.ad,
                eposta: self.eposta,
                yonetici: false, // yetki geri gelmez: kural tek yerde, bu modülde yazılı
                giris_sayisi: 0,
            }
        }
    }
}

use hesap::{Eposta, Kullanici};

// 1. Alan gizliliği ile değişmez (invariant) koruma
// SADE ANLATIM: Burada büroyu sınıyoruz: üç başvuru veriyoruz. Birincisi kurallara uyar ve kart
// basılır; büro bu arada yazıyı düzeltir (boşlukları atar, harfleri küçültür). Öteki ikisi
// reddedilir ve büro sebebini söyler. Kabul `Ok(...)`, ret `Err(...)` olarak döner; `match` ikisine
// ayrı ayrı bakar.
//   Mini örnek:
//     match Eposta::ayristir("ada@ornek.com") {
//         Ok(e) => println!("kart basıldı: {}", e.as_str()),   // ✓ bu dal çalışır
//         Err(sebep) => println!("ret: {sebep}"),
//     }
//     match Eposta::ayristir("ada.ornek.com") {                // içinde @ yok
//         Ok(e) => println!("kart basıldı: {}", e.as_str()),
//         Err(sebep) => println!("ret: {sebep}"),              // bu dal çalışır
//     }
fn degismez_koruma() {
    println!("-- 1. Alan gizliliği ile değişmez koruma");
    for ham in ["  Kaan@Ornek.COM ", "kaan.ornek.com", "@ornek.com"] {
        match Eposta::ayristir(ham) {
            Ok(e) => println!("kabul: {} (alan adı: {})", e.as_str(), e.alan_adi()),
            Err(hata) => println!("ret: {hata}"),
        }
    }
    // Beklenen çıktı:
    //   kabul: kaan@ornek.com (alan adı: ornek.com)
    //   ret: geçersiz e-posta: 'kaan.ornek.com'
    //   ret: geçersiz e-posta: '@ornek.com'

    // Sık yapılan hatalar — denetimi atlamaya çalışmak (hepsi modül DIŞINDAN denenmiş hâliyle):
    //   let e = hesap::Eposta(String::from("asdf")); // ✗ E0603: tuple struct constructor `Eposta` is private
    //   println!("{}", e.0);                         // ✗ E0616: field `0` of struct `Eposta` is private
    //   let k = Kullanici { ad: .., eposta: .., yonetici: true, giris_sayisi: 0 };
    //                                                // ✗ E0451: fields .. of struct `Kullanici` are private
    // Olmasaydı ne bozulurdu? Tek bir `k.eposta = "asdf".into()` satırı, e-postayı geçerli
    // sayan bütün kodu (posta gönderimi, alan adına göre gruplama...) sessizce yanlışa düşürürdü.
    // ✓ Doğrusu yukarıdaki gibi: değer yalnız ayristir()'dan doğar, as_str() ile okunur.
}

// 2. Gerçek hayattan örnek: builder kalıbı
// SADE ANLATIM: Kart başvuru formunu göz göz doldurursun: "adım Ada", "e-postam şu". En sonda
// formu teslim edersin; büro eksik ya da hatalı göz var mı diye bakar, yoksa kartı basar. Builder
// kalıbı bu formdur: her adımın bir adı vardır, hangi bilginin nereye yazıldığı okunur.
// Neden böyle? Dört bilgiyi tek satırda sırayla verseydin (`yeni("Ada", "a@b.c", true, false)`)
// hangi `true`nun ne demek olduğunu kimse anlayamazdı; sırayı karıştırsan derleyici de uyarmazdı.
//   Mini örnek:
//     let tam = Kullanici::kurucu().ad("Ada").eposta("ada@ornek.com").olustur();
//     println!("{}", tam.is_ok());       // ✓ true → form tamdı, kart basıldı
//     let eksik = Kullanici::kurucu().ad("Ada").olustur();
//     println!("{}", eksik.is_ok());     // false → e-posta gözü boş kaldı, kart basılmadı
fn builder_kalibi() {
    println!("-- 2. Builder kalıbı");
    // Neden builder? Rust'ta varsayılan/isimli argüman yok. Dört beş parametreli bir
    // `yeni("Kaan", "k@x.com", true, false)` çağrısında hangi bool'un ne olduğu okunmaz;
    // sırayı karıştırmak da tür aynı olduğu için derlenir. Builder her değere bir ad verir.
    let kaan = Kullanici::kurucu()
        .ad("Kaan")
        .eposta("Kaan@Ornek.com")
        .yonetici(true)
        .olustur();
    let eksik = Kullanici::kurucu().ad("Ada").olustur();
    let bozuk = Kullanici::kurucu()
        .ad("Ada")
        .eposta("ada(at)ornek")
        .olustur();
    for sonuc in [kaan, eksik, bozuk] {
        match sonuc {
            Ok(k) => println!("kuruldu: {}", k.ozet()),
            Err(hata) => println!("kurulamadı: {hata}"),
        }
    }
    // Beklenen çıktı:
    //   kuruldu: Kaan <kaan@ornek.com> [yönetici], 0 giriş
    //   kurulamadı: e-posta zorunlu
    //   kurulamadı: geçersiz e-posta: 'ada(at)ornek'

    // Yanlış kullanım / Doğru kullanım — kurucunun receiver'ı neden `self`?
    //   fn ad(&mut self, ..) -> &mut Self { .. }          // ayarlayıcılar &mut Self döndürseydi
    //   let k = KullaniciKurucu::default().ad("Kaan").olustur();
    //                                // ✗ E0507: cannot move out of a mutable reference
    // Zincirin ucunda elde yalnızca bir &mut kalır; olustur(self) ise sahiplik ister. Ödünç
    // alınan şey tüketilemez. `self -> Self` zincirinde sahiplik elden ele geçtiği için sorun yok.
    // Bedeli: tüketilen kurucu ikinci kez kullanılamaz —
    //   let b = Kullanici::kurucu().ad("Kaan");
    //   let k1 = b.olustur();
    //   let k2 = b.olustur();        // ✗ E0382: use of moved value: `b`
    // ✓ İki kullanıcı gerekiyorsa iki kurucu zinciri yazılır (yukarıdaki gibi).
}

// 3. Receiver (`self` alıcısı) seçiminin sahiplikle ilişkisi
// SADE ANLATIM: Okulu bitiren öğrencinin öğrenci kartı geri alınır, yerine mezun kartı verilir.
// Mezun kartıyla derse giriş kaydı yapılamaz; kartın üstünde öyle bir bölüm yoktur. Rust'ta
// `pasiflestir(self)` bunu yapar: eski kartı ALIR (bir daha kullanamazsın) ve başka cins bir kart
// verir. Yanlış kartla yanlış iş yapmak isteyen kod daha derlenirken yakalanır.
// Neden böyle? Tek bir kartın üstüne "aktif mi: evet/hayır" yazsaydık, "hayır" yazan kartla giriş
// yapılmasını ancak program çalışırken, her seferinde elle denetleyerek engelleyebilirdik.
//   Mini örnek:
//     let mut ada = Kullanici::kurucu().ad("Ada").eposta("ada@ornek.com").olustur().unwrap();
//     ada.giris_kaydet();              // ✓ öğrenci kartıyla giriş yapılır
//     let mezun = ada.pasiflestir();   // öğrenci kartı teslim edildi
//     println!("{}", mezun.ozet());    // ✓ mezun kartı okunabilir
//     ada.giris_kaydet();              // ✗ derlenmez: öğrenci kartı artık elimizde değil
fn alici_secimi() {
    println!("-- 3. Receiver seçimi = sahiplik sözleşmesi");
    // (let-else: desen tutmazsa erken çık; enums crate'inde anlatılıyor.)
    let Ok(mut ada) = Kullanici::kurucu()
        .ad("Ada")
        .eposta("ada@ornek.com")
        .olustur()
    else {
        return;
    };
    //   let ada = ..; ada.giris_kaydet(); // ✗ E0596: cannot borrow `ada` as mutable, as it is
    //                                     //   not declared as mutable → ✓ `let mut ada`
    ada.giris_kaydet();
    ada.giris_kaydet();
    println!("{} | {} harf", ada.ozet(), ada.ad().chars().count()); // Ada <ada@ornek.com> [üye], 2 giriş | 3 harf
    println!(
        "{} girişin alan adı: {}",
        ada.giris_sayisi(),
        ada.eposta().alan_adi()
    ); // 2 girişin alan adı: ornek.com

    let pasif = ada.pasiflestir(); // `ada` burada tüketildi
    println!("{}", pasif.ozet()); // [pasif] Ada <ada@ornek.com>, son giriş sayısı 2
    //   ada.giris_kaydet(); // ✗ E0382: borrow of moved value: `ada`
    // Bu hata burada bir ÖZELLİKTİR: pasifleştirilmiş hesaba giriş yazmak mantık hatasıdır ve
    // derleyici onu yakalar. `aktif: bool` bayrağıyla aynı hata ancak çalışma anında görülürdü.

    let geri = pasif.yeniden_etkinlestir();
    println!("{}", geri.ozet()); // Ada <ada@ornek.com> [üye], 0 giriş

    // Sık yapılan hata — elinde yalnız referans varken tüketen metodu çağırmak:
    //   fn arsivle(k: &Kullanici) -> hesap::PasifKullanici { k.pasiflestir() }
    //                       // ✗ E0507: cannot move out of `*k` which is behind a shared reference
    // ✓ Tüketecek fonksiyon sahipliği imzasında ister: `fn arsivle(k: Kullanici)`.
    //
    // Önemli noktalar — receiver'ı seçerken sor:
    //   yalnız okuyorum                         → &self      (en az kısıt; varsayılan tercih)
    //   yerinde değiştiriyorum                  → &mut self  (çağıran `mut` binding tutmalı)
    //   değeri başka bir şeye dönüştürüyorum,
    //   eskisi artık anlamsız                   → self       (eskisinin kullanımı derlenmez)
}

// 4. Struct update sözdiziminde kısmi move — hangi alan gider, hangisi kalır?
// SADE ANLATIM: main.rs'te gördük: `..eski` yazınca eski karttaki fotoğraf sökülüp yeni karta
// geçer. Buradaki incelik şudur: yalnız `..`nın gerçekten aldığı gözler gider. Yazı gözlerini yeni
// kartta kendin doldurursan eski karttan geriye yalnız sayılar kalır; onlar kopyalanır ve eski
// kart bütün hâlde elinde durur.
// Neden böyle? Taşıma göz göz yapılır; dokunulmayan gözün eski karttan çıkması için sebep yoktur.
//   Mini örnek:
//     struct Ayar { tema: String, boyut: u32 }
//     let eski = Ayar { tema: String::from("açık"), boyut: 20 };
//     let a = Ayar { tema: String::from("koyu"), ..eski };   // `..` yalnız boyut'u (sayı) aldı
//     println!("{}", eski.tema);      // ✓ eski kart bütün
//     let b = Ayar { boyut: 50, ..eski };                    // `..` tema'yı aldı: tema taşındı
//     println!("{}", eski.tema);      // ✗ derlenmez: tema artık b'de
#[derive(Debug, Clone, PartialEq)]
struct Ayarlar {
    tema: String,
    dil: String,
    bildirim: bool,
    sayfa_boyutu: u32,
}

impl Default for Ayarlar {
    fn default() -> Self {
        Ayarlar {
            tema: String::from("açık"),
            dil: String::from("tr"),
            bildirim: true,
            sayfa_boyutu: 20,
        }
    }
}

fn guncelleme_tuzagi() {
    println!("-- 4. Struct update: kısmi move");
    // Nasıl çalışır? `..taban` "kalanları kopyala" DEĞİL, "açıkça yazmadığım alanları taban'dan
    // AL" demektir. main.rs taşınan alanın (eposta) öldüğünü gösterdi; buradaki ek bilgi:
    // taşıma ALAN ALAN olur — yalnız `..`'nın gerçekten aldığı non-Copy alanlar gider.
    let taban = Ayarlar::default();
    let koyu = Ayarlar {
        tema: String::from("koyu"),
        ..taban // dil (String) TAŞINDI; bildirim ve sayfa_boyutu (Copy) kopyalandı
    };
    println!(
        "{} / {} / taban.tema hâlâ: {}",
        koyu.tema, koyu.dil, taban.tema
    ); // koyu / tr / taban.tema hâlâ: açık
    //   println!("{taban:?}"); // ✗ E0382: borrow of partially moved value: `taban`
    // taban.tema'ya dokunulmadı (yerine yenisi yazıldı), o yüzden tek başına okunabiliyor;
    // ama taban BÜTÜN olarak artık kullanılamaz, çünkü `dil` alanı taşındı.

    // ✓ Doğru kullanım 1: non-Copy alanların hepsini kendin ver → `..` yalnız Copy alır.
    let taban = Ayarlar::default();
    let ingilizce = Ayarlar {
        tema: taban.tema.clone(),
        dil: String::from("en"),
        ..taban
    };
    println!(
        "{} / taban bütün: {}",
        ingilizce.dil,
        taban == Ayarlar::default()
    ); // en / taban bütün: true

    // ✓ Doğru kullanım 2: kaynağı korumak istiyorsan kopyayı AÇIKÇA iste (maliyet görünür olur).
    let genis = Ayarlar {
        sayfa_boyutu: 50,
        ..taban.clone()
    };
    // ✓ Doğru kullanım 3: "varsayılanlar + birkaç fark" deyimi. Geçici değer zaten sahipsiz,
    // taşınması kimseyi etkilemez. Alan eklenince bu satırlar değişmeden derlenmeye devam eder.
    let sessiz = Ayarlar {
        bildirim: false,
        ..Default::default()
    };
    println!(
        "{} / {} / taban: {}",
        genis.sayfa_boyutu, sessiz.bildirim, taban.sayfa_boyutu
    ); // 50 / false / taban: 20

    // Sık yapılan hata — alan unutmak:
    //   let a = Ayarlar { tema: String::from("koyu") }; // ✗ E0063: missing fields `bildirim`,
    //                                                   //   `dil` and `sayfa_boyutu` in initializer
    // Rust'ta "yarım kurulmuş" struct olmaz; ya hepsini verirsin ya `..` ile kaynağını söylersin.
    //
    // Gizlilikle ilişkisi: `..` de alanlara ADIYLA erişir. Private alanı olan bir struct modül
    // dışında `Kullanici { ad: .., ..k }` ile güncellenemez (✗ E0451: fields `ad`, `eposta`,
    // `yonetici` and `giris_sayisi` of struct `Kullanici` are private). Yani bölüm 1'deki koruma bu sözdizimiyle de delinemez.
}

// SADE ANLATIM (aşağıdaki bölüm için): Alıştırmalar kendini denemen içindir. Üçü de bu dosyadaki
// kalıpları yeniden kurdurur: denetleyen bir kart, forma yeni bir göz, kartı yerinde değiştiren
// bir metot. Takılırsan ipuçlarına bak; ilk okuyuşta çözemezsen sonraki bölüme geçip dönebilirsin.
// ============================================================================
// ALIŞTIRMALAR (çözümler verilmedi; `cargo test -p structs` ile kendi testini ekleyip dene)
// ============================================================================
//
// 1. `Parola` newtype'ı yaz: en az 8 karakter ve en az bir rakam içermeyeni reddetsin.
//    İpucu: Eposta::ayristir ile aynı kalıp; alanı private tut, `chars().any(..)` işini görür.
//    Beklenen: Parola::ayristir("kisa1") → Err, Parola::ayristir("uzunparola1") → Ok.
//    Ek soru: Parola'ya neden derive(Debug) eklemek istemeyebilirsin?
//
// 2. KullaniciKurucu'ya `dil(self, &str) -> Self` adımı ekle; verilmezse "tr" olsun ve
//    ozet() bunu göstersin.
//    İpucu: Option<String> + olustur() içinde `unwrap_or_else`. Kullanici'ya alan eklediğinde
//    derleyicinin seni hangi iki yapıcı ifadeye götürdüğüne dikkat et (E0063).
//    Beklenen: mevcut testler değişmeden geçer; yeni alan iki yerde de doldurulmuştur.
//
// 3. `Kullanici::ad_degistir` metodunu yaz: boş (yalnız boşluk) adı reddetsin.
//    İpucu: hangi receiver? Değer yerinde değişiyor ve eski hâli anlamsızlaşmıyor.
//    Beklenen: başarısız çağrıdan sonra `ad()` eski değeri döndürür.
//
// Bir sonraki konuyla bağlantısı (enums):
// PasifKullanici ile "durumu türle ifade etme"nin ilk adımını attık, ama aktif ve pasif
// kullanıcıyı aynı listede tutamıyoruz: iki ayrı tür. `enum Hesap { Aktif(..), Pasif(..) }`
// tam bunu çözer. KullaniciKurucu'daki Option ve olustur()'un döndürdüğü Result da birer
// enum; `learn/enums` onların içini açıyor.

// SADE ANLATIM: Aşağıdaki `testler` bölümü programla birlikte çalışmaz; kodun doğru davrandığını
// denetleyen küçük sınavlardır. `cargo test -p structs` yazınca her biri çalışır.
// `assert_eq!(a, b)` "a ile b eşit olmalı" der; eşit değilse o sınav kalır ve sana haber verilir.
#[cfg(test)]
mod testler {
    use super::Ayarlar;
    use super::hesap::{Eposta, Kullanici};

    #[test]
    fn eposta_normallestirilir_ve_gecersizi_reddedilir() {
        let e = Eposta::ayristir(" Kaan@Ornek.COM ").unwrap();
        assert_eq!(e.as_str(), "kaan@ornek.com");
        assert_eq!(e.alan_adi(), "ornek.com");
        assert!(Eposta::ayristir("kaan.ornek.com").is_err());
        assert!(Eposta::ayristir("@ornek.com").is_err());
        assert!(Eposta::ayristir("kaan@ornek").is_err());
        // İlan edilen değişmezin her maddesi: boşluksuz ve tek '@'.
        assert!(Eposta::ayristir("ka an@ornek.com").is_err());
        assert!(Eposta::ayristir("kaan@or@nek.com").is_err());
    }

    #[test]
    fn kurucu_zorunlu_alanlari_denetler() {
        let eksik = Kullanici::kurucu().ad("Ada").olustur();
        assert_eq!(eksik.unwrap_err(), "e-posta zorunlu");
        let adsiz = Kullanici::kurucu().ad("   ").eposta("a@b.c").olustur();
        assert_eq!(adsiz.unwrap_err(), "ad zorunlu");
    }

    #[test]
    fn kurucu_varsayilanlari_doldurur() {
        let k = Kullanici::kurucu()
            .ad(" Ada ")
            .eposta("ada@ornek.com")
            .olustur()
            .unwrap();
        assert_eq!(k.ad(), "Ada");
        assert_eq!(k.giris_sayisi(), 0);
        assert_eq!(k.ozet(), "Ada <ada@ornek.com> [üye], 0 giriş");
    }

    #[test]
    fn pasiflestirme_girisleri_tasir_yetkiyi_geri_vermez() {
        let mut k = Kullanici::kurucu()
            .ad("Kaan")
            .eposta("kaan@ornek.com")
            .yonetici(true)
            .olustur()
            .unwrap();
        k.giris_kaydet();
        let pasif = k.pasiflestir();
        assert_eq!(
            pasif.ozet(),
            "[pasif] Kaan <kaan@ornek.com>, son giriş sayısı 1"
        );
        let geri = pasif.yeniden_etkinlestir();
        assert_eq!(geri.ozet(), "Kaan <kaan@ornek.com> [üye], 0 giriş");
    }

    #[test]
    fn update_sozdizimi_yalniz_copy_alirsa_kaynak_butun_kalir() {
        let taban = Ayarlar::default();
        let ingilizce = Ayarlar {
            tema: taban.tema.clone(),
            dil: String::from("en"),
            ..taban
        };
        assert_eq!(taban, Ayarlar::default());
        assert_eq!(ingilizce.dil, "en");
        assert_eq!(ingilizce.sayfa_boyutu, taban.sayfa_boyutu);
    }
}
