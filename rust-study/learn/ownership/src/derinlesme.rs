// Açıklama: learn/ownership/ACIKLAMA.md
// Derinleşme — Bölüm 4: Ownership ve Borrowing (pekiştirme: gerçekçi senaryo, sık hatalar, alıştırmalar)
// Çalıştır: cargo run -p ownership   |   Test: cargo test -p ownership
//
// main.rs kuralları TEK TEK tanıttı (move, Copy, clone, drop, &T, &mut T, dilim). Bu dosya aynı
// kuralları "kod yazarken karşına nasıl çıkar?" sorusuyla yeniden ele alır:
//   - bir fonksiyon imzası yazarken String mi, &str mi, &mut String mi istemelisin,
//   - gerçek bir veri yapısında (kullanıcı deposu) sahiplik kimde durur, kim ödünç alır,
//   - borrow checker'ın en sık verdiği beş hata (E0382, E0499, E0502, E0505, E0507) hangi
//     alışkanlıktan doğar ve her birinin deyimsel (idiomatic) çözümü nedir.
// Buradaki ✗ örneklerin hata kodları da rustc 1.99 ile derlenerek doğrulandı.

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Önce main.rs'i oku; bu dosya onun devamıdır. Her bölümde önce SADE ANLATIM kutusu,
// sonra teknik açıklama var. Benzetmelerin hepsi bir KÜTÜPHANEDE geçer.
//
// Bu dosyanın tek cümlelik özeti: main.rs'te öğrendiğin "ver, bak, değiştir" kurallarını gerçek
// kod yazarken nerede, nasıl seçeceğini ve derleyici itiraz ettiğinde ne yapacağını gösterir.
//
// MİNİ SÖZLÜK (yalnız bu dosyada yeni geçenler; öteki terimler main.rs başındaki sözlükte):
//   imza (signature)       → fonksiyonun ilk satırı: adı, ne aldığı, ne verdiği.
//   parametre              → fonksiyonun dışarıdan aldığı değer.
//   struct, alan (field)   → birkaç bilgiyi tek pakette toplayan tür; paketin her gözü bir alandır.
//   metot, self            → bir struct'a bağlı fonksiyon; `self` o struct'ın kendisidir.
//   kısmi move             → paketin tamamını değil, yalnız bir gözünü başkasına vermek.
//   dilim (&str, &[T])     → bir yazının ya da listenin bir parçasına (ya da tamamına) bakış.
//   Vec                    → boyu büyüyüp küçülebilen liste.
//   Option, Some, None     → "bir şey var" (Some) ya da "yok" (None) diyen kutu.
//   iterator               → elemanları sırayla, tek tek veren şey.
//   |k| k.id == id         → closure: yerinde yazılmış isimsiz küçük fonksiyon (k girdidir).
//   tahsis (allocation)    → heap'ten (büyük depodan) yeni yer istemek; emek ister.
//   double free            → aynı yeri iki kez silmeye çalışma hatası.
//   tanımsız davranış      → programın ne yapacağının belli olmadığı, kural dışı durum.
//   deyimsel (idiomatic)   → o dili bilenlerin alıştığı, yerleşik yazım biçimi.
//   NLL                    → ödünç, parantez kapanınca değil, son kullanıldığı satırda biter.
//   elision                → derleyicinin, yazmadığın ömür bilgisini kendisinin tamamlaması.
//   Default                → bir türün "boş hâli": boş liste, boş yazı, 0.
//   değişmez (invariant)   → hiçbir an bozulmaması gereken söz: "her rafta geçerli bir şey durur".
//   E0382, E0502 ...       → derleyicinin hata numaraları; `rustc --explain E0382` açıklar.
//   test, assert           → kodun doğru çalıştığını denetleyen küçük sınav ve o sınavın sorusu.

pub fn calistir() {
    println!("\n== Derinleşme: ownership pratikte ==");
    imza_secimi();
    kismi_move();
    kullanici_deposu_senaryosu();
    sik_hatalar_ve_cozumleri();
    sahipligi_yerinden_almak();
}

// ============================================================================
// 1. BU KONUYU NEDEN ÖĞRENİYORUZ? — İmza, sahiplik sözleşmesidir
// ============================================================================
// Rust'ta bir fonksiyonun parametre türü yalnız "ne alıyorum" demez; "aldığım şeyle NE
// YAPACAĞIM"ı da söyler. Çağıran, gövdeyi okumadan şunu bilir:
//
//   fn f(x: String)       → "Bunu senden ALIYORUM." Çağıran artık kullanamaz (move).
//   fn f(x: &str)         → "Yalnız BAKACAĞIM." Çağıranın değeri olduğu gibi kalır.
//   fn f(x: &mut String)  → "Yerinde DEĞİŞTİRECEĞİM." Çağıran sonucu kendi değişkeninde görür.
//
// Benzetme: kitabı HEDİYE etmek (String), kütüphanede OKUMAK (&str), not için ÖDÜNÇ vermek
// (&mut String). Yanlış olan, yalnız bakacakken kitabı elinden almaktır.
//
// Karar kuralı (çoğu durumda yeter):
//   1) Değeri saklayacak mısın (struct alanına koymak, başka thread'e göndermek)? → sahipliği al.
//   2) Yalnız okuyacak mısın? → & ile ödünç al; metin için &String değil &str, dizi için
//      &Vec<T> değil &[T] iste (dilim daha geneldir: String, literal ve alt dilim hepsi uyar).
//   3) Yerinde değiştirecek misin? → &mut ile ödünç al.
// SADE ANLATIM: Kütüphane bankosunda üç sepet durur. "BAĞIŞ" sepetine bıraktığın kitap artık
// kütüphanenindir. "SALONDA OKUNUR" sepetindeki kitaba görevli yalnız bakar; kitap senindir.
// "ONARILACAK" sepetindeki kitabın yırtığını görevli yapıştırır, kitap onarılmış hâlde yine sende
// kalır. Rust'ta fonksiyonun parametre türü, sepetin üstündeki bu yazıdır: `String` bağış, `&str`
// bakış, `&mut String` onarımdır.
// Neden? İmza olmasaydı kitabın geri alınıp alınmayacağını görevlinin yaptıklarından çıkarırdın.
//   Mini örnek:
//     fn bagisla(kitap: String) {}             // alır; çağırana geri dönmez
//     fn bak(kitap: &str) -> usize { kitap.len() }
//     let k = String::from("masal");
//     bak(&k);              // ✓ k hâlâ bizde
//     bagisla(k);           // ✓ k kütüphaneye geçti
//     bak(&k);              // ✗ derlenmez: k artık bizde değil
fn imza_secimi() {
    // (2) Yalnız okur: &str. &String de, literal de, bir alt dilim de verilebilir.
    fn kelime_say(metin: &str) -> usize {
        metin.split_whitespace().count()
    }
    // (3) Yerinde değiştirir: &mut String. Yeni tahsis yapmadan mevcut tamponu düzenler.
    fn bosluklari_kirp(metin: &mut String) {
        let kirpilmis = metin.trim().len();
        let bas = metin.len() - metin.trim_start().len();
        metin.drain(..bas); // baştaki boşlukları at
        metin.truncate(kirpilmis); // sondaki boşlukları at
    }
    // (1) Saklar/dönüştürür: String. Aldığı tamponu YENİDEN KULLANIP geri verir — kopya yok.
    fn selamlamaya_cevir(mut ad: String) -> String {
        ad.insert_str(0, "Merhaba, ");
        ad.push('!');
        ad
    }

    let mut girdi = String::from("  Ada Lovelace  ");
    println!("kelime sayısı = {}", kelime_say(&girdi)); // 2 — girdi hâlâ bizim
    println!("literal de olur = {}", kelime_say("bir iki üç")); // 3
    bosluklari_kirp(&mut girdi);
    println!("kırpılmış = '{girdi}'"); // 'Ada Lovelace'
    let selam = selamlamaya_cevir(girdi); // girdi buraya TAŞINDI
    println!("{selam}"); // Merhaba, Ada Lovelace!
    // println!("{girdi}"); // ✗ E0382: borrow of moved value: `girdi`

    // Sık yapılan hata: yalnız okuyacak fonksiyona `String` parametresi yazmak. Derlenir, ama
    // çağıranı her seferinde `.clone()` yazmaya zorlar — gereksiz tahsis. İmza ne kadar az şey
    // isterse (sahiplik yerine ödünç, &String yerine &str) fonksiyon o kadar çok yerde kullanılır.
}

// ============================================================================
// 2. KISMİ MOVE (partial move) — bir struct'ın tek alanını taşımak
// ============================================================================
// Move yalnız değişkenin tamamına değil, ALANLARINA da uygulanır. Bir alanı taşırsan struct
// "kısmen taşınmış" olur: kalan alanları tek tek kullanabilirsin ama struct'ı BÜTÜN olarak
// artık kullanamazsın. Derleyici bunu alan alan izler (main.rs'teki "başlatılmış mı" analizi,
// değişken düzeyinde değil yer/alan düzeyinde çalışır).
// SADE ANLATIM: Üç ciltlik bir masal seti düşün. İkinci cildi kütüphaneye bağışlarsan birinci ve
// üçüncü cildi okumaya devam edersin; ama elindekini artık "tam set" diye kimseye veremezsin,
// çünkü bir cildi eksik. Rust'ta struct'ın bir alanını taşımak da böyledir: kalan alanlar
// kullanılır, struct'ın bütünü kullanılamaz.
// Neden böyle? Bütüne izin verilseydi, giden cilt hem yeni sahibinde hem "tam set"in içinde
// sayılırdı; aynı veri iki kez silinmeye çalışılırdı.
//   Mini örnek:
//     struct Set { cilt1: String, cilt2: String }
//     let s = Set { cilt1: String::from("bir"), cilt2: String::from("iki") };
//     let giden = s.cilt2;        // yalnız 2. cilt taşındı
//     println!("{}", s.cilt1);    // ✓ 1. cilt hâlâ yerinde
//     let tam_set = s;            // ✗ derlenmez: set eksik, bütün olarak verilemez
struct Profil {
    ad: String,
    eposta: String,
    yas: u32,
}

fn kismi_move() {
    let p = Profil {
        ad: String::from("Kaan"),
        eposta: String::from("kaan@ornek.dev"),
        yas: 30,
    };
    let ad = p.ad; // yalnız `ad` alanı taşındı
    println!("ad = {ad}, e-posta = {}, yaş = {}", p.eposta, p.yas); // öteki alanlar hâlâ okunur
    // let tamami = p; // ✗ E0382: use of partially moved value: `p`

    // ✓ Yapıyı bozmadan okumak istiyorsan alanı ödünç al (&p.ad); gerçekten parçalayacaksan
    //   hepsini BİRDEN desenle çöz — böylece hiçbir alan "unutulmuş yarım struct" içinde kalmaz:
    let p2 = Profil {
        ad: String::from("Ada"),
        eposta: String::from("ada@ornek.dev"),
        yas: 36,
    };
    let Profil { ad, eposta, yas } = p2; // üç alanın sahipliği üç ayrı değişkene geçti
    println!("çözüldü: {ad} <{eposta}> ({yas})");
}

// ============================================================================
// 3. GERÇEK HAYATTAN ÖRNEK — kullanıcı deposu: sahip kim, ödünç alan kim?
// ============================================================================
// Tasarım sorusu her zaman aynıdır: "Bu verinin TEK sahibi kim olmalı?" Burada cevap depo:
// kullanıcılar deponun Vec'inde yaşar, dışarıya yalnız REFERANS verilir. Metot imzaları
// sahiplik akışını belgeler:
//
//   ekle(&mut self, Kullanici)            → kullanıcının sahipliği DEPOYA geçer
//   bul(&self, id) -> Option<&Kullanici>  → depo sahip kalır, çağıran yalnız bakar
//   bul_mut(&mut self, id) -> Option<&mut Kullanici> → yerinde güncelleme için özel erişim
//   cikar(&mut self, id) -> Option<Kullanici> → sahiplik depodan ÇAĞIRANA geri taşınır
// SADE ANLATIM: Kütüphanedeki kitapların sahibi RAFTIR. Kitap bağışlarsan rafa geçer (`ekle`).
// Okumak istersen raftaki kitaba bakarsın, kitap yerinde durur (`bul`). Görevli bir kitabın yanlış
// etiketini yerinde düzeltebilir (`bul_mut`). Kitabı raftan tamamen çıkarırsan kitap senin olur
// (`cikar`). Rust'ta bir veri yapısı tasarlarken ilk soru hep budur: "Bu verinin rafı neresi?"
// Neden böyle? Sahibi belli olmayan bir kitabı kimin kaldıracağı da belli olmaz; ya rafta unutulur
// ya da iki kişi birden kaldırmaya kalkar.
//   Mini örnek:
//     let mut raf = vec![String::from("masal"), String::from("atlas")];
//     let bakis = &raf[0];            // ✓ bak: kitap rafta kalır
//     println!("{bakis}");
//     let alinan = raf.remove(0);     // ✓ çıkar: "masal" artık bizim, rafta yalnız "atlas" var
//     let kacak = raf[0];             // ✗ derlenmez: rafa haber vermeden kitap alınmaz
#[derive(Debug, PartialEq, Eq)]
pub struct Kullanici {
    pub id: u32,
    pub ad: String,
    pub aktif: bool,
}

#[derive(Default)]
pub struct KullaniciDeposu {
    kayitlar: Vec<Kullanici>,
}

impl KullaniciDeposu {
    pub fn ekle(&mut self, k: Kullanici) {
        self.kayitlar.push(k); // k buraya taşındı; çağıran artık k'yi kullanamaz
    }

    pub fn bul(&self, id: u32) -> Option<&Kullanici> {
        self.kayitlar.iter().find(|k| k.id == id)
    }

    pub fn bul_mut(&mut self, id: u32) -> Option<&mut Kullanici> {
        self.kayitlar.iter_mut().find(|k| k.id == id)
    }

    pub fn cikar(&mut self, id: u32) -> Option<Kullanici> {
        let sira = self.kayitlar.iter().position(|k| k.id == id)?;
        Some(self.kayitlar.remove(sira)) // değer Vec'ten çıkarılıp sahipliğiyle döner
    }

    // (Sade: aşağıdaki liste adları kopyalamaz, raftaki adlara bakar. Liste elindeyken raf
    //  değiştirilemez; yoksa yerinden oynamış adları göstermeye devam ederdi.)
    // İmzadaki elision "dönen &str'ler &self kadar yaşar" der; depo değişirse liste
    // geçersizleşeceği için derleyici, liste kullanılırken depoyu değiştirmeni engeller (aşağıda ✗).
    pub fn aktif_adlar(&self) -> Vec<&str> {
        self.kayitlar
            .iter()
            .filter(|k| k.aktif)
            .map(|k| k.ad.as_str())
            .collect()
    }
}

fn kullanici_deposu_senaryosu() {
    let mut depo = KullaniciDeposu::default();
    let ada = Kullanici {
        id: 1,
        ad: String::from("Ada"),
        aktif: true,
    };
    depo.ekle(ada);
    // println!("{ada:?}"); // ✗ E0382: `ada` depoya taşındı; tek sahip artık depo
    depo.ekle(Kullanici {
        id: 2,
        ad: String::from("Linus"),
        aktif: false,
    });
    depo.ekle(Kullanici {
        id: 3,
        ad: String::from("Grace"),
        aktif: true,
    });

    // Paylaşımlı ödünç: okumak için.
    if let Some(k) = depo.bul(1) {
        println!("bulundu: {} (aktif: {})", k.ad, k.aktif); // bulundu: Ada (aktif: true)
    }

    // Değiştirilebilir ödünç: yerinde güncelleme. `k` son kullanıldığı satırda biter (NLL),
    // bu yüzden hemen ardından depoyu yeniden okuyabiliyoruz.
    if let Some(k) = depo.bul_mut(2) {
        k.aktif = true;
    }
    println!("aktifler: {:?}", depo.aktif_adlar()); // ["Ada", "Linus", "Grace"]

    // ✗ Ödünç verilmiş veriye bakan bir liste dururken depoyu değiştirmek:
    //     let adlar = depo.aktif_adlar();      // &depo ödüncü `adlar` yaşadıkça sürer
    //     depo.cikar(1);                       // E0502: cannot borrow `depo` as mutable ...
    //     println!("{adlar:?}");
    // Neden iyi ki derlenmiyor? `cikar` Vec'i kaydırır; `adlar[0]` artık silinmiş bir String'in
    // tamponuna bakardı. C++'ta bu "iterator invalidation" hatasıdır: tanımsız davranış — bazen çöker,
    // bazen sessizce yanlış veri okur.
    // ✓ Çözüm: önce ödünç alınan sonucu kullanıp bitir, SONRA değiştir (aşağıdaki sıra).

    // Sahipliği geri almak: değer depodan çıkar, bize taşınır.
    let cikan = depo.cikar(1);
    println!("çıkarılan: {:?}", cikan.map(|k| k.ad)); // Some("Ada")
    println!("kalan aktifler: {:?}", depo.aktif_adlar()); // ["Linus", "Grace"]
}

// ============================================================================
// 4. SIK YAPILAN HATALAR — beş klasik, beş deyimsel çözüm
// ============================================================================
// SADE ANLATIM: Görevlinin en sık yaptığı beş uyarı ve her birinin kolay çözümü:
//   1) Kitapları tek tek dağıttıysan raf boşalmıştır; sonra "rafta kaç kitap var" diye soramazsın.
//      Dağıtma, yalnız göster (`for k in &raf`).
//   2) Rafı sayarken rafa kitap eklenmez; sayım şaşar. Önce say, sonra ekle.
//   3) Aynı rafın iki gözünü aynı anda düzenlemek için rafı ikiye böl (`split_at_mut`); derleyici
//      iki elinin aynı kitaba uzanmadığını ancak o zaman görebilir.
//   4) Biri kitabına bakarken o kitabı başkasına veremezsin. Önce bakan işini bitirsin.
//   5) Salonda bakmak için aldığın kitabı çantana atamazsın. Ya bak, ya kopyasını çıkar (`clone`),
//      ya da rafa haber verip kitabı raftan çıkar.
// Neden böyle? Beş uyarının da derdi aynıdır: kimse, başkası kullanırken altından çekilen ya da
// değişen bir kitapla baş başa kalmasın.
//   Mini örnek (1. uyarı):
//     let raf = vec![String::from("masal"), String::from("atlas")];
//     for k in &raf { println!("{k}"); }    // ✓ & ile yalnız gösterdik
//     println!("{}", raf.len());            // ✓ 2 → raf hâlâ dolu
//     for k in raf { println!("{k}"); }     // kitaplar tek tek dağıtıldı, raf tüketildi
//     println!("{}", raf.len());            // ✗ derlenmez: raf artık yok
fn sik_hatalar_ve_cozumleri() {
    // --- Hata 1: döngüde koleksiyonu tüketmek (E0382) --------------------------------------
    //   ✗  let adlar = vec![String::from("a")];
    //      for ad in adlar { ... }        // `for x in v` = v.into_iter(): Vec'i TÜKETİR
    //      println!("{}", adlar.len());   // E0382: borrow of moved value: `adlar`
    // ✓ Yalnız okuyacaksan referans üzerinden dolaş: `for ad in &adlar`.
    let adlar = vec![String::from("Ada"), String::from("Grace")];
    for ad in &adlar {
        print!("{ad} ");
    }
    println!("→ {} ad (Vec hâlâ bizim)", adlar.len()); // Ada Grace → 2 ad (Vec hâlâ bizim)

    // --- Hata 2: üzerinde dolaşırken koleksiyonu değiştirmek (E0502) ------------------------
    //   ✗  for x in &sayilar { if *x == 2 { sayilar.push(4); } }
    //      // E0502: cannot borrow `sayilar` as mutable because it is also borrowed as immutable
    // Sorun gerçek: push yeniden tahsis ederse döngünün elindeki iterator serbest bırakılmış
    // belleği gösterir. ✓ İki adıma böl: önce ne yapılacağına karar ver, sonra uygula.
    let mut sayilar = vec![1, 2, 3];
    let eklenecek: Vec<i32> = sayilar
        .iter()
        .filter(|x| **x % 2 == 0)
        .map(|x| x * 10)
        .collect();
    sayilar.extend(eklenecek); // okuma bitti; artık güvenle yazabiliriz
    sayilar.retain(|x| *x != 1); // "dolaşırken sil" ihtiyacının hazır, güvenli karşılığı
    println!("sayilar = {sayilar:?}"); // [2, 3, 20]

    // --- Hata 3: aynı anda iki &mut (E0499) -------------------------------------------------
    //   ✗  let a = &mut puanlar[0];
    //      let b = &mut puanlar[1];       // E0499: cannot borrow `puanlar[_]` as mutable more than once
    //      std::mem::swap(a, b);
    // Derleyici indekslerin farklı olduğunu KANITLAYAMAZ (0 ve 1 yerine i ve j olabilirdi).
    // ✓ "Bu iki parça çakışmıyor" bilgisini türle ifade eden API'leri kullan: split_at_mut,
    //   swap, iter_mut, chunks_mut.
    let mut puanlar = [10, 20, 30];
    let (sol, sag) = puanlar.split_at_mut(1); // iki AYRIK &mut dilim — çakışma imkânsız
    std::mem::swap(&mut sol[0], &mut sag[0]);
    println!("puanlar = {puanlar:?}"); // [20, 10, 30]

    // --- Hata 4: ödünç verilmişken taşımak (E0505) ------------------------------------------
    //   ✗  let r = &baslik;
    //      let yeni_sahip = baslik;       // E0505: cannot move out of `baslik` because it is borrowed
    //      println!("{r}");
    // ✓ Referansın son kullanımı taşımadan ÖNCE olsun; ya da iki bağımsız sahip gerekiyorsa clone.
    let baslik = String::from("rapor");
    let r = &baslik;
    println!("önce referansı kullan: {r}");
    let yeni_sahip = baslik; // r artık kullanılmıyor → ödünç bitti → taşımak serbest
    println!("sonra taşı: {yeni_sahip}");

    // --- Hata 5: referansın ardındaki değeri taşımaya çalışmak (E0507) ----------------------
    //   ✗  let ilk = liste[0];            // E0507: cannot move out of index of `Vec<String>`
    //   ✗  fn ad_al(k: &Kullanici) -> String { k.ad }
    //                                     // E0507: cannot move out of `k.ad` which is behind a shared reference
    // Mantık: & yalnız BAKMA hakkıdır; içindekini alırsan asıl sahip "delik" değerle kalır (double free).
    // ✓ Üç seçenek, niyete göre:
    let mut liste = vec![String::from("bir"), String::from("iki"), String::from("üç")];
    let bakis: &String = &liste[0]; // (a) yalnız bak
    let kopya: String = liste[1].clone(); // (b) bağımsız bir kopya üret (bedeli: tahsis)
    println!("bakış = {bakis}, kopya = {kopya}");
    let alinan: String = liste.swap_remove(0); // (c) gerçekten AL — ama koleksiyondan çıkararak
    println!("alınan = {alinan}, kalan = {liste:?}"); // alınan = bir, kalan = ["üç", "iki"]
}

// ============================================================================
// 5. YANLIŞ KULLANIM / DOĞRU KULLANIM — &mut ardındaki değeri "yerinden almak"
// ============================================================================
// Elinde yalnız `&mut T` varken içindeki değeri taşıyamazsın (E0507), çünkü yeri boş bırakmak
// yasaktır. Çözüm: yerine geçerli BAŞKA bir değer koyarak al. std bunun için üç araç verir:
//   std::mem::take(&mut x)        → x'i alır, yerine Default::default() koyar
//   std::mem::replace(&mut x, y)  → x'i alır, yerine y koyar
//   Option::take(&mut opt)        → Some(v)'yi alır, yerine None koyar
// Böylece "her yer her an geçerli bir değer tutar" değişmezi hiç bozulmaz.
// SADE ANLATIM: Raftan kitap alınca yerine "bu kitap alındı" yazan boş bir karton konur ki raf
// hiçbir an boşluklu kalmasın. `std::mem::take` bunu yapar: değeri alır, yerine o türün boş
// hâlini koyar. `Option`ın `take` metodu da aynısıdır: içindekini alır, yerine None bırakır.
// Neden? Rafa sonradan bakan biri orada geçerli bir şey bulmak zorundadır.
//   Mini örnek:
//     let mut raf = vec![String::from("masal")];
//     let alinan = std::mem::take(&mut raf);   // kitaplar bizde, rafta boş bir liste kaldı
//     println!("{} {}", alinan.len(), raf.len());   // ✓ 1 0
//     let mut kupon = Some("FOTOKOPI");
//     println!("{:?}", kupon.take());          // ✓ Some("FOTOKOPI") → kupon alındı
//     println!("{:?}", kupon.take());          // None → ikinci kez alınacak bir şey kalmadı
#[derive(Default)]
pub struct Sepet {
    urunler: Vec<String>,
    kupon: Option<String>,
}

impl Sepet {
    pub fn ekle(&mut self, urun: &str) {
        self.urunler.push(urun.to_string());
    }

    pub fn kupon_tanimla(&mut self, kod: &str) {
        self.kupon = Some(kod.to_string());
    }

    // (Sade: "üç kelime" Vec'in küçük künyesidir: pointer, uzunluk, kapasite; elemanlar heap'te
    //  durur, el değiştiren yalnız künyedir, tek bayt kopyalanmaz.)
    // ✗ `let hepsi = self.urunler;` → E0507: &mut self'in ardından alan taşınamaz.
    // ✗ clone + clear boşuna kopyalar. ✓ mem::take yerine boş Vec koyar.
    pub fn siparise_cevir(&mut self) -> Vec<String> {
        std::mem::take(&mut self.urunler)
    }

    // Kupon tek kullanımlıktır: take() onu alır ve alanı None yapar — "ikinci kez kullanma"
    // hatası mantık düzeyinde imkânsızlaşır.
    pub fn kuponu_kullan(&mut self) -> Option<String> {
        self.kupon.take()
    }

    pub fn urun_sayisi(&self) -> usize {
        self.urunler.len()
    }
}

fn sahipligi_yerinden_almak() {
    let mut sepet = Sepet::default();
    sepet.ekle("klavye");
    sepet.ekle("fare");
    sepet.kupon_tanimla("HOSGELDIN");

    let siparis = sepet.siparise_cevir();
    println!(
        "sipariş = {siparis:?}, sepette kalan = {}",
        sepet.urun_sayisi()
    ); // ["klavye", "fare"], 0
    println!("kupon 1. kullanım = {:?}", sepet.kuponu_kullan()); // Some("HOSGELDIN")
    println!("kupon 2. kullanım = {:?}", sepet.kuponu_kullan()); // None
}

// SADE ANLATIM (aşağıdaki iki bölüm için): "ÖNEMLİ NOKTALAR" bu dosyanın dört cümlelik özetidir:
// derleyicinin itirazı bir ipucudur; kopya çıkarmak (`clone`) gerektiğinde yanlış değildir; ödünç
// son kullanımda biter; bir verinin gerçekten birçok sahibi olacaksa başka bir araç gerekir.
// "ALIŞTIRMALAR" ise kendini denemen içindir. Takılırsan ipuçlarına bak; ilk okuyuşta çözemezsen
// sonraki bölümlere geçip sonra geri dönebilirsin.
// ============================================================================
// ÖNEMLİ NOKTALAR
// ============================================================================
// • Borrow checker bir "engel" değil, tasarım geri bildirimidir: E0502/E0499 çoğu zaman "bu
//   veriyi iki yer aynı anda yönetmeye çalışıyor" der. Çözüm genellikle clone serpiştirmek
//   değil, sahipliği netleştirmek (kim sahip, kim ne kadar süre ödünç alıyor) ya da işi
//   "oku → karar ver → yaz" diye sıralamaktır.
// • clone bir günah değildir: iki bağımsız sahibe gerçekten ihtiyaç varsa doğru araçtır. Koku
//   olan, hatayı susturmak için refleksle yazılan clone'dur. Kendine sor: "Bu kopyaya gerçekten
//   ihtiyacım var mı, yoksa bir & yeter miydi?"
// • Ödünç, sözdizimsel bloğun sonunda değil SON KULLANIMDA biter (NLL). Bir E0502 aldığında ilk
//   bakacağın şey: "eski referansı bu satırdan sonra hâlâ kullanıyor muyum?"
// • Bir değerin birden çok sahibi GERÇEKTEN gerekiyorsa (graf, paylaşılan yapılandırma) cevap
//   referans değil Rc/Arc'tır → smart-pointers crate'i.
//
// ============================================================================
// ALIŞTIRMALAR
// ============================================================================
// 1) `fn en_uzun_ad(depo: &KullaniciDeposu) -> Option<&str>` yaz: en uzun ada sahip aktif
//    kullanıcının adını KOPYALAMADAN döndürsün.
//    İpucu: aktif_adlar() + iter().max_by_key(|ad| ad.chars().count()); dönüşte .copied().
//    Beklenen: Ada/Linus/Grace (hepsi aktif) için Some("Linus") ya da Some("Grace") — beşer
//    harf; max_by_key eşitlikte SONUNCUYU verir, yani Some("Grace").
//
// 2) `Sepet`e `fn urunu_degistir(&mut self, sira: usize, yeni: String) -> Option<String>` ekle:
//    verilen sıradaki ürünü `yeni` ile değiştirip ESKİSİNİ döndürsün; sıra geçersizse None.
//    İpucu: self.urunler.get_mut(sira) + std::mem::replace. clone kullanma.
//
// 3) Aşağıdaki fonksiyon derlenmez. Hata kodunu tahmin et, sonra iki farklı biçimde düzelt
//    (biri imzayı, öteki gövdeyi değiştirsin):
//        fn ilk_iki(v: &Vec<String>) -> (String, String) { (v[0], v[1]) }
//    İpucu: E0507. Seçenekler: (&str, &str) döndürmek ya da clone'lamak. Hangisi çağırana
//    daha fazla seçenek bırakır?
//
// Bir sonraki konuyla bağlantısı: struct'lar (learn/structs) sahipliği ALANLARA taşır —
// `&self` / `&mut self` / `self` alıcıları bu dosyadaki üç imza biçiminin metot hâlidir.
// Referans TUTAN struct'lar ise lifetime ister → generics-traits-lifetimes ve
// ileri-lifetime-hrtb-gat.

// SADE ANLATIM: Aşağıdaki `testler` bölümü programın bir parçası olarak çalışmaz; kodun doğru
// davrandığını denetleyen küçük sınavlardır. `cargo test -p ownership` yazınca her biri çalışır.
// `assert_eq!(a, b)` "a ile b eşit olmalı" der; eşit değilse o sınav kalır ve sana haber verilir.
#[cfg(test)]
mod testler {
    use super::*;

    fn ornek_depo() -> KullaniciDeposu {
        let mut depo = KullaniciDeposu::default();
        for (id, ad, aktif) in [(1, "Ada", true), (2, "Linus", false), (3, "Grace", true)] {
            depo.ekle(Kullanici {
                id,
                ad: ad.to_string(),
                aktif,
            });
        }
        depo
    }

    #[test]
    fn bul_odunc_verir_depo_sahip_kalir() {
        let depo = ornek_depo();
        assert_eq!(depo.bul(3).map(|k| k.ad.as_str()), Some("Grace"));
        assert!(depo.bul(99).is_none());
    }

    #[test]
    fn bul_mut_yerinde_gunceller() {
        let mut depo = ornek_depo();
        depo.bul_mut(2).unwrap().aktif = true;
        assert_eq!(depo.aktif_adlar(), ["Ada", "Linus", "Grace"]);
    }

    #[test]
    fn cikar_sahipligi_geri_verir() {
        let mut depo = ornek_depo();
        let cikan = depo.cikar(1).unwrap();
        assert_eq!(cikan.ad, "Ada");
        assert!(depo.bul(1).is_none());
        assert!(depo.cikar(1).is_none()); // ikinci kez çıkarılamaz
    }

    #[test]
    fn mem_take_yeri_bos_degerle_doldurur() {
        let mut sepet = Sepet::default();
        sepet.ekle("klavye");
        assert_eq!(sepet.siparise_cevir(), ["klavye"]);
        assert_eq!(sepet.urun_sayisi(), 0);
        assert!(sepet.siparise_cevir().is_empty());
    }

    #[test]
    fn kupon_yalniz_bir_kez_kullanilir() {
        let mut sepet = Sepet::default();
        sepet.kupon_tanimla("X");
        assert_eq!(sepet.kuponu_kullan().as_deref(), Some("X"));
        assert_eq!(sepet.kuponu_kullan(), None);
    }
}
