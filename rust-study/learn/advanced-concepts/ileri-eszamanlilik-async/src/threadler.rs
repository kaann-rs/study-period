// Açıklama: learn/advanced-concepts/ileri-eszamanlilik-async/ACIKLAMA.md
// BÖLÜM A — İleri eşzamanlılık: RwLock, Condvar, kilit sıralaması, atomikler, Send/Sync
//
// Böl. 16 (concurrency) Mutex + kanal + scope ile "veri yarışı olmayan" kod yazmayı öğretti.
// Burada bir adım ötesi var: kilidin TÜRÜNÜ yüke göre seçmek, bir koşul gerçekleşene dek
// işlemciyi yakmadan beklemek ve derleyicinin yakalayamadığı deadlock'u TASARIMLA önlemek.

// ============================================================================
// SADE ANLATIM — önce buradan başla
// ============================================================================
// Ana sözlük main.rs'in başındadır; önce onu oku. Bu dosyanın tek cümlelik özeti: ortak bir
// eşyayı birçok thread kullanacaksa doğru kapıyı (kilit türünü) seçmek, beklerken işlemciyi
// yormamak ve kilitleri hep aynı sırayla almak gerekir.
//
// MİNİ SÖZLÜK (bu dosyaya özgü yeni kelimeler):
//   RwLock               → okuma-yazma kilidi: çok okuyucu birlikte ya da tek yazar tek başına.
//   read() / write()     → okuma anahtarı / yazma anahtarı istemek (bekletebilir).
//   try_read / try_write → beklemeden denemek: alınamıyorsa hemen "olmadı" der.
//   kilit yükseltme      → okuma anahtarını bırakmadan yazma anahtarına çevirmek. RwLock'ta yok.
//   Condvar              → koşul değişkeni: "şu durum olunca beni uyandır" zili.
//   wait / notify        → zili bekleyerek uyumak / zili çalmak (birine ya da herkese).
//   lost wakeup          → kayıp uyandırma: zil, sen daha uyumadan çalar ve boşa gider.
//   spurious wakeup      → sebepsiz uyanma: zil çalmadığı hâlde uyanmak.
//   busy-wait            → uyumak yerine durmadan "oldu mu?" diye bakmak.
//   sınırlı kuyruk       → en çok N öğe alan sıra (bounded queue).
//   üretici / tüketici   → kuyruğa öğe koyan / kuyruktan öğe alan taraf.
//   geri basınç          → backpressure: kuyruk dolunca üreticinin de beklemesi.
//   kilit sıralaması     → bütün kodun kilitleri aynı sırayla alması kuralı.
//   compare_exchange     → "değer hâlâ beklediğimse değiştir, değilse dokunma" hamlesi.
//   Ordering             → atomik işlemin öteki thread'lere ne kadar sıkı bir sırayla
//                          görüneceğini seçen ayar (Relaxed, Acquire/Release, SeqCst).
//   thread::scope        → içinde açılan thread'lerin blok bitmeden bitmesini garanti eden alan.

use std::cell::Cell;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread;

pub fn calistir() {
    println!("== A1) RwLock: çok okuyucu YA DA tek yazar ==");
    rwlock_ornegi();
    println!("\n== A2) Condvar: sınırlı kuyruk (üretici/tüketici, geri basınç) ==");
    condvar_ornegi();
    println!("\n== A3) Deadlock ve kilit sıralaması: banka transferi ==");
    kilit_siralamasi_ornegi();
    println!("\n== A4) Atomikler: compare_exchange ve durdurma bayrağı ==");
    atomik_ornegi();
    println!("\n== A5) Send / Sync: hangi tür neden geçer, neden geçmez ==");
    send_sync_tablosu();
}

// ============================================================================
// A1) RwLock<T> — okuma ağırlıklı paylaşımlı durum
// ============================================================================
// Hangi problemi çözer? Mutex okuyanla yazanı AYIRMAZ: on thread yalnız okumak istese bile
// sıraya girer. Çoğu paylaşılan veri (yapılandırma, yönlendirme tablosu, önbellek) çok okunur,
// seyrek yazılır. RwLock, borrow kuralının ("çok & YA DA tek &mut") çalışma anındaki, thread'ler
// arası karşılığıdır:
//     read()  → RwLockReadGuard  (&T gibi: aynı anda çok sayıda olabilir)
//     write() → RwLockWriteGuard (&mut T gibi: tek başına)
//
// Ne zaman Mutex daha iyi? Kritik bölge çok kısaysa (bir sayıyı artırmak), yazma sık ise ya da
// T: Sync değilse. RwLock'un kendi yönetim maliyeti Mutex'ten yüksektir; "okuma var" diye
// refleksle seçme, okuyucular gerçekten birbirini bekletiyorsa seç.
// SADE ANLATIM: Kütüphanenin okuma salonuna aynı anda istediği kadar kişi girip okuyabilir;
// kimse kimseyi rahatsız etmez. Ama rafları yeniden dizecek görevli, salon boşalınca tek
// başına girer; o içerideyken okuyucu da alınmaz. `RwLock` bu kuralı uygular: `read()` okuyucu
// girişidir, `write()` görevli girişidir. Bu neden var? `Mutex` okuyanları da tek tek sıraya
// sokar; veri çok okunup seyrek yazılıyorsa okuyucular boşuna bekler.
//   Mini örnek:
//     let kilit = RwLock::new(5);
//     let o1 = kilit.read().unwrap();
//     let o2 = kilit.read().unwrap();           // ✓ iki okuyucu birlikte içeride
//     assert!(kilit.try_write().is_err());      // yazar giremez: içeride okuyucu var
//     drop(o1);
//     drop(o2);                                 // okuyucular çıktı
//     *kilit.write().unwrap() += 1;             // ✓ yazar tek başına girdi
//     // o1 elindeyken kilit.write() demek: ✗ derlenir ama thread kendi kendini bekler
pub struct AyarDeposu {
    ayarlar: RwLock<HashMap<String, String>>,
    surum: AtomicU64, // kaç kez yazıldı — tek sayı için kilide gerek yok
}

impl AyarDeposu {
    pub fn yeni() -> Self {
        AyarDeposu {
            ayarlar: RwLock::new(HashMap::new()),
            surum: AtomicU64::new(0),
        }
    }

    // Guard'ı DIŞARI vermiyoruz: değeri klonlayıp kilidi hemen bırakıyoruz. `&str` döndürmek
    // guard'ı çağırana taşımayı gerektirirdi; kilidin ne kadar tutulacağı o zaman çağırana
    // kalırdı (ve unutulan bir okuma guard'ı tüm yazarları bekletirdi).
    pub fn oku(&self, anahtar: &str) -> Option<String> {
        self.ayarlar.read().unwrap().get(anahtar).cloned()
    }

    pub fn yaz(&self, anahtar: &str, deger: &str) {
        self.ayarlar
            .write()
            .unwrap()
            .insert(anahtar.to_string(), deger.to_string());
        self.surum.fetch_add(1, Ordering::Relaxed);
    }

    pub fn surum(&self) -> u64 {
        self.surum.load(Ordering::Relaxed)
    }
}

fn rwlock_ornegi() {
    let depo = AyarDeposu::yeni();
    depo.yaz("tema", "açık");

    // Kuralı gözle görelim (try_* bloklamaz, o anki durumu söyler):
    {
        let okuyucu1 = depo.ayarlar.read().unwrap();
        let ikinci_okuyucu = depo.ayarlar.try_read().is_ok();
        let yazar = depo.ayarlar.try_write().is_ok();
        println!(
            "   bir okuyucu varken ({} ayar): ikinci okuyucu = {ikinci_okuyucu}, yazar = {yazar}",
            okuyucu1.len()
        ); // true, false
    }

    // 4 okuyucu + 1 yazar. Okuyucular ya eski ya yeni değeri görür; YARIM yazılmış bir değeri
    // asla göremezler (yazar tek başınadır).
    let gecersiz_okuma = AtomicU32::new(0);
    thread::scope(|k| {
        for _ in 0..4 {
            k.spawn(|| {
                for _ in 0..1_000 {
                    match depo.oku("tema").as_deref() {
                        Some("açık") | Some("koyu") => {}
                        _ => {
                            gecersiz_okuma.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
            });
        }
        k.spawn(|| {
            for i in 0..100 {
                depo.yaz("tema", if i % 2 == 0 { "koyu" } else { "açık" });
            }
        });
    });
    println!(
        "   son değer = {:?}, sürüm = {}, geçersiz okuma = {}",
        depo.oku("tema"),
        depo.surum(),
        gecersiz_okuma.into_inner()
    ); // Some("açık"), 101, 0

    // SIK YAPILAN HATA (derlenir, çalışma anında kilitlenebilir): okuma guard'ı elindeyken aynı
    // thread'de write() istemek. "Okuyup, yoksa ekleyeyim" derken olur:
    //     let g = depo.ayarlar.read().unwrap();
    //     if !g.contains_key("x") { depo.ayarlar.write().unwrap().insert(..); }   // ✗ kendi kendini bekler
    // RwLock okuma kilidini yazmaya YÜKSELTMEZ. ✓ Okuma guard'ını bırak, write() al ve koşulu
    // kilit altında YENİDEN denetle (arada başka bir yazar eklemiş olabilir).
}

// ============================================================================
// A2) Condvar — "koşul gerçekleşene dek uyu"
// ============================================================================
// Sorun: tüketici, kuyruk boşken ne yapsın? Döngüde kilitleyip bakmak (busy-wait) bir çekirdeği
// boşuna yakar; araya sleep koymak hem gecikme ekler hem yine boşa uyanır. İstediğimiz:
// "kuyruğa bir şey gelince BENİ UYANDIR."
//
// Condvar (koşul değişkeni) tam budur ve her zaman bir Mutex ile birlikte kullanılır:
//     wait(guard)   → kilidi BIRAKIR + uyur (ikisi atomik); uyanınca kilidi GERİ ALIR.
//     notify_one()  → bekleyenlerden birini uyandırır.
// "Bırak + uyu"nun atomik olması şarttır: arada boşluk olsaydı, bildirim tam o boşlukta gelip
// kaybolabilirdi (lost wakeup) ve tüketici sonsuza dek uyurdu.
//
// İki kural:
//  1) Koşulu HER ZAMAN döngüde denetle. wait sebepsiz yere de dönebilir (spurious wakeup) ve
//     sen kilidi geri alana dek başka bir thread öğeyi kapmış olabilir. `wait_while` bu
//     döngüyü senin yerine yazar.
//  2) Koşulu değiştiren taraf bunu KİLİT ALTINDA yapar, sonra bildirir.
//
// Senaryo: sınırlı (bounded) kuyruk. Kapasite dolunca ÜRETİCİ de bekler — buna geri basınç
// (backpressure) denir: hızlı üretici, yavaş tüketicinin belleği şişirmesine izin vermez.
// SADE ANLATIM: Pastanenin tezgâhında dört börek alan bir tepsi var. Tepsi boşsa müşteri
// kapıda dikilip durmadan "geldi mi?" diye sormaz; oturur, börek gelince görevli "sıran geldi"
// diye zili çalar. Tepsi doluysa bu kez fırıncı bekler; yer açılınca onun zili çalar.
// `Condvar` bu zildir ve hep bir `Mutex` ile birlikte çalışır: kilit tepsiyi korur, zil
// bekleyeni uyandırır. Bu neden var? Beklerken işlemciyi boşa çalıştırmamak ve hızlı
// üreticinin belleği doldurmasını önlemek için.
//   Mini örnek:
//     let kuyruk = SinirliKuyruk::yeni(1);      // tepside tek yer var
//     kuyruk.koy(1).unwrap();                   // ✓ yer vardı, hemen kondu
//     // kuyruk.koy(2);                         // hata değil: biri al() diyene kadar BEKLER
//     assert_eq!(kuyruk.al(), Some(1));         // ✓ aldı; bekleyen üretici varsa zili çalındı
//     kuyruk.kapat();
//     assert_eq!(kuyruk.al(), None);            // kapalı ve boş: beklemez, None döner
//     assert_eq!(kuyruk.koy(3), Err(3));        // kapalı kuyruk öğeyi geri verir
pub struct SinirliKuyruk<T> {
    durum: Mutex<KuyrukDurumu<T>>,
    dolu_degil: Condvar, // üreticiler bunda bekler
    bos_degil: Condvar,  // tüketiciler bunda bekler
    kapasite: usize,
}

struct KuyrukDurumu<T> {
    ogeler: VecDeque<T>,
    kapali: bool,
    en_yuksek_doluluk: usize,
}

impl<T> SinirliKuyruk<T> {
    pub fn yeni(kapasite: usize) -> Self {
        SinirliKuyruk {
            durum: Mutex::new(KuyrukDurumu {
                ogeler: VecDeque::new(),
                kapali: false,
                en_yuksek_doluluk: 0,
            }),
            dolu_degil: Condvar::new(),
            bos_degil: Condvar::new(),
            kapasite: kapasite.max(1),
        }
    }

    // Yer açılana dek bekler. Kuyruk kapatılmışsa öğeyi geri verir (kaybetmez).
    pub fn koy(&self, oge: T) -> Result<(), T> {
        let kilit = self.durum.lock().unwrap();
        // wait_while: koşul DOĞRU oldukça uyu. Kapanışı da koşula katıyoruz; yoksa kapalı ve
        // dolu bir kuyrukta üretici sonsuza dek beklerdi.
        let mut durum = self
            .dolu_degil
            .wait_while(kilit, |d| d.ogeler.len() >= self.kapasite && !d.kapali)
            .unwrap();
        if durum.kapali {
            return Err(oge);
        }
        durum.ogeler.push_back(oge);
        durum.en_yuksek_doluluk = durum.en_yuksek_doluluk.max(durum.ogeler.len());
        // Önce kilidi bırak, SONRA bildir. Kilit elimizdeyken bildirseydik uyanan tüketici
        // hemen kilidi ister, alamaz ve yeniden uyurdu (boşa bir uyanma). Doğruluk açısından
        // iki sıra da geçerlidir — koşul kilit altında değişti; bu yalnız bir verim inceliği.
        drop(durum);
        self.bos_degil.notify_one(); // bekleyen bir tüketici varsa: "artık boş değil"
        Ok(())
    }

    // Öğe gelene dek bekler. None = kuyruk kapalı VE boş (kalan öğeler önce tüketilir).
    pub fn al(&self) -> Option<T> {
        let kilit = self.durum.lock().unwrap();
        let mut durum = self
            .bos_degil
            .wait_while(kilit, |d| d.ogeler.is_empty() && !d.kapali)
            .unwrap();
        let oge = durum.ogeler.pop_front();
        drop(durum); // bildirimden önce kilidi bırak (koy'daki notla aynı gerekçe)
        if oge.is_some() {
            self.dolu_degil.notify_one(); // bekleyen bir üretici varsa: "yer açıldı"
        }
        oge
    }

    pub fn kapat(&self) {
        self.durum.lock().unwrap().kapali = true;
        // notify_ALL: kapanış, bekleyen HERKESİ ilgilendirir; notify_one yalnız birini
        // uyandırır, ötekiler sonsuza dek uyurdu.
        self.bos_degil.notify_all();
        self.dolu_degil.notify_all();
    }

    pub fn en_yuksek_doluluk(&self) -> usize {
        self.durum.lock().unwrap().en_yuksek_doluluk
    }
}

// 2 üretici (her biri 1..=n), 2 tüketici. Dönen: tüketilenlerin toplamı.
pub fn uret_tuket(kuyruk: &SinirliKuyruk<u64>, uretici_basina: u64) -> u64 {
    thread::scope(|k| {
        let ureticiler: Vec<_> = (0..2)
            .map(|_| {
                k.spawn(|| {
                    for i in 1..=uretici_basina {
                        kuyruk.koy(i).expect("kuyruk üretim bitmeden kapatılmamalı");
                    }
                })
            })
            .collect();
        let tuketiciler: Vec<_> = (0..2)
            .map(|_| {
                k.spawn(|| {
                    let mut toplam = 0;
                    while let Some(x) = kuyruk.al() {
                        toplam += x;
                    }
                    toplam
                })
            })
            .collect();
        for u in ureticiler {
            u.join().unwrap();
        }
        kuyruk.kapat(); // üretim bitti → tüketiciler kalanları alıp None görünce çıkar
        tuketiciler.into_iter().map(|t| t.join().unwrap()).sum()
    })
}

fn condvar_ornegi() {
    let kuyruk = SinirliKuyruk::yeni(4);
    let toplam = uret_tuket(&kuyruk, 500);
    println!(
        "   tüketilen toplam = {toplam}, doluluk hiç kapasiteyi (4) aşmadı = {}",
        kuyruk.en_yuksek_doluluk() <= 4
    ); // 250500, true
    println!("   kapalı kuyruğa koy → {:?}", kuyruk.koy(7)); // Err(7)

    // Not: std'nin mpsc::sync_channel(n) kanalı aynı davranışı hazır verir. Burada elle yazdık
    // çünkü Condvar'ın asıl kullanım kalıbı budur: "bir Mutex'in koruduğu durum üzerinde bir
    // koşulu bekle" — bağlantı havuzu, iş kuyruğu, bariyer, semafor hep bu iskelettir.
}

// ============================================================================
// A3) Deadlock ve kilit sıralaması
// ============================================================================
// Derleyici veri yarışını yakalar, deadlock'u YAKALAMAZ. Klasik biçimi:
//     Thread 1: A'yı kilitle → B'yi bekle
//     Thread 2: B'yi kilitle → A'yı bekle          → ikisi de sonsuza dek bekler.
// Para transferi tam bu tuzaktır: `transfer(a, b)` ile `transfer(b, a)` aynı anda çağrılırsa,
// "önce kaynağı, sonra hedefi kilitle" yazan saf gerçekleştirim kilitlenir.
//
// Çözüm çalışma anı hilesi değil, bir KURALDIR: tüm kod, kilitleri aynı GLOBAL SIRAYLA alır.
// Burada sıra hesap numarasıdır: küçük numara önce. İki thread de aynı ilk kilidi isteyeceği
// için biri bekler, döngüsel bekleme oluşamaz.
// SADE ANLATIM: İki çocuk iki oyuncakla oynamak istiyor. Ali kırmızı arabayı kapıp mavi
// arabayı bekliyor; Ayşe mavi arabayı kapıp kırmızıyı bekliyor. İkisi de elindekini
// bırakmadığı için sonsuza dek beklerler. Buna deadlock denir. Rust veri yarışını derlemede
// yakalar, ama bu bekleyişi yakalayamaz; kod derlenir ve çalışırken takılır. Çözüm bir ev
// kuralıdır: herkes oyuncakları hep AYNI sırayla alır, önce küçük numaralıyı. O zaman ikisi de
// önce aynı oyuncağa uzanır; biri alır, öteki bekler, kimse elinde yarım işle kalmaz.
//   Mini örnek:
//     let a = Hesap::yeni(1, 100);
//     let b = Hesap::yeni(2, 100);
//     transfer(&a, &b, 30);     // ✓ önce 1 numaralı, sonra 2 numaralı hesap kilitlenir
//     transfer(&b, &a, 10);     // ✓ yön ters ama kilit sırası AYNI: yine önce 1, sonra 2
//     transfer(&a, &a, 5);      // false: aynı kilidi iki kez istemek kendini bekletirdi
//     // ✗ "önce kaynağı, sonra hedefi kilitle" yazsaydık: derlenirdi, ama iki thread
//     //   karşılıklı transfer yaparken program sonsuza dek bekleyebilirdi
pub struct Hesap {
    pub no: u32,
    bakiye: Mutex<i64>,
}

impl Hesap {
    pub fn yeni(no: u32, bakiye: i64) -> Self {
        Hesap {
            no,
            bakiye: Mutex::new(bakiye),
        }
    }

    pub fn bakiye(&self) -> i64 {
        *self.bakiye.lock().unwrap()
    }
}

pub fn transfer(kimden: &Hesap, kime: &Hesap, tutar: i64) -> bool {
    if kimden.no == kime.no {
        return false; // aynı Mutex'i iki kez kilitlemek = kendi kendine deadlock
    }
    if tutar <= 0 {
        return false; // negatif tutar transferi ters çevirir ve bakiye denetimini atlatırdı
    }
    // 1) Sırayı YÖNE göre değil, hesap numarasına göre belirle.
    let kaynak_once = kimden.no < kime.no;
    let (ilk, ikinci) = if kaynak_once {
        (kimden, kime)
    } else {
        (kime, kimden)
    };
    // 2) İki kilidi de al. İkisi de elimizde olmadan hiçbir bakiyeye dokunmuyoruz: böylece
    //    "kaynaktan düştü ama hedefe eklenmedi" ara durumu dışarıdan GÖRÜLEMEZ.
    let mut ilk_kilit = ilk.bakiye.lock().unwrap();
    let mut ikinci_kilit = ikinci.bakiye.lock().unwrap();
    // 3) Guard'ları yeniden kaynak/hedef rollerine eşle.
    let (kaynak, hedef) = if kaynak_once {
        (&mut *ilk_kilit, &mut *ikinci_kilit)
    } else {
        (&mut *ikinci_kilit, &mut *ilk_kilit)
    };
    if *kaynak < tutar {
        return false;
    }
    *kaynak -= tutar;
    *hedef += tutar;
    true
} // guard'lar tanımın tersi sırayla düşer; bırakma sırası deadlock açısından önemsizdir

fn kilit_siralamasi_ornegi() {
    let a = Hesap::yeni(1, 1_000);
    let b = Hesap::yeni(2, 1_000);
    // Karşılıklı transferler: saf "kaynak → hedef" sırası burada kilitlenirdi.
    thread::scope(|k| {
        k.spawn(|| (0..1_000).for_each(|_| assert!(transfer(&a, &b, 1))));
        k.spawn(|| (0..500).for_each(|_| assert!(transfer(&b, &a, 1))));
    });
    println!(
        "   A = {}, B = {}, toplam = {} (para kaybolmadı, kilitlenme yok)",
        a.bakiye(),
        b.bakiye(),
        a.bakiye() + b.bakiye()
    ); // A = 500, B = 1500, toplam = 2000
    println!("   yetersiz bakiye → {}", transfer(&a, &b, 10_000)); // false

    // Diğer savunmalar:
    //  • Tek kilit: iki hesabı aynı Mutex<Banka> korusun. En basit ve çoğu zaman yeterli.
    //  • try_lock + geri çekil: ikinci kilit alınamazsa birinciyi BIRAK ve yeniden dene.
    //  • Kilit tutarken yabancı kod çağırma (closure, trait metodu): içeride hangi kilidi
    //    alacağını bilemezsin.
    //  • Mesajlaşmaya geç: hesapların tek sahibi bir thread olsun, transferler kanaldan gelsin.
}

// ============================================================================
// A4) Atomikler — kilitsiz tek adımlık işlemler
// ============================================================================
// compare_exchange(beklenen, yeni): "değer hâlâ `beklenen` ise `yeni` yap; değilse dokunma ve
// gördüğün değeri söyle." Kilitsiz algoritmaların temel taşı: "oku → karar ver → yaz" üçlüsünü
// araya kimse girmeden tek adımda yapar.
//
// Ordering (kısa rehber):
//   Relaxed → yalnız bu işlemin atomikliği garanti. Bağımsız sayaçlar için.
//   Release (yazarken) / Acquire (okurken) → "bu yazmadan ÖNCE yaptığım her şey, bunu Acquire
//             ile okuyan thread'e görünür." Bayrak/işaretçi ile BAŞKA veriyi yayımlarken.
//   SeqCst  → tüm SeqCst işlemler için tek bir global sıra. En güçlüsü ve en kolay akıl
//             yürütüleni; emin değilsen bunu kullan, ölçmeden zayıflatma.
// SADE ANLATIM: Bayrak kapmaca oynuyorsunuz. Bayrağı kapmak tek hamledir: iki çocuk aynı
// bayrağı yarı yarıya kapamaz, biri kapar, öteki boş yere uzanmış olur. Atomik değerler böyle
// çalışır: okuma ve yazma yarıda kesilemeyen tek hamledir, bu yüzden kilit gerekmez.
// `compare_exchange(0, kimlik)` şunu der: "bayrak hâlâ yerdeyse (0) onu ben aldım diye yaz;
// değilse dokunma ve kimde olduğunu söyle". Bu neden var? Tek bir sayı ya da bayrak için
// kilit kurmak gereksiz yere pahalıdır.
//   Mini örnek:
//     let lider = AtomicU32::new(0);            // 0 = bayrak yerde
//     lider.compare_exchange(0, 7, Ordering::SeqCst, Ordering::SeqCst);  // Ok(0): 7 kaptı
//     lider.compare_exchange(0, 9, Ordering::SeqCst, Ordering::SeqCst);  // Err(7): geç kaldı
//     let sayac = AtomicU32::new(0);
//     sayac.fetch_add(1, Ordering::Relaxed);    // ✓ kilitsiz, tek hamlede +1
//     // let mut n = 0; iki thread'den n += 1;  // ✗ derlenmez: kilitsiz ortak yazma yasak
pub fn lider_sec(aday_sayisi: u32) -> (u32, u32) {
    let lider = AtomicU32::new(0); // 0 = henüz lider yok
    let kazanan_sayisi = AtomicU32::new(0);
    thread::scope(|k| {
        for kimlik in 1..=aday_sayisi {
            let (lider, kazanan_sayisi) = (&lider, &kazanan_sayisi);
            k.spawn(move || {
                // Yalnız İLK gelen 0'ı görür; ötekiler Err(mevcut lider) alır.
                if lider
                    .compare_exchange(0, kimlik, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                {
                    kazanan_sayisi.fetch_add(1, Ordering::Relaxed); // bağımsız sayaç → Relaxed
                }
            });
        }
    });
    (lider.into_inner(), kazanan_sayisi.into_inner())
}

fn atomik_ornegi() {
    let (lider, kazanan_sayisi) = lider_sec(8);
    println!(
        "   8 aday yarıştı: kazanan sayısı = {kazanan_sayisi}, lider geçerli bir aday = {}",
        (1..=8).contains(&lider)
    ); // 1, true  (hangi adayın kazandığı çalıştırmadan çalıştırmaya değişir)

    // Durdurma bayrağı: işçiye "artık dur" demenin en hafif yolu. Bayrak başka bir veriyi
    // yayımlamıyor, yalnız kendi değeri önemli → yukarıdaki rehbere göre Relaxed yeter.
    let dur = Arc::new(AtomicBool::new(false));
    let isci = {
        let dur = Arc::clone(&dur);
        thread::spawn(move || {
            let mut tur: u64 = 0;
            while !dur.load(Ordering::Relaxed) {
                tur += 1;
                thread::yield_now();
            }
            tur
        })
    };
    dur.store(true, Ordering::Relaxed);
    let tur = isci.join().unwrap();
    // Tur sayısı zamanlamaya bağlıdır (0 bile olabilir); deterministik olan, işçinin DURMASIDIR.
    println!(
        "   işçi bayrağı görüp durdu (sayaç geçerli: {})",
        tur < u64::MAX
    );
}

// ============================================================================
// A5) Send / Sync — pratik tablo
// ============================================================================
// ileri-send-sync mekanizmayı (auto-trait, UnsafeCell, negatif impl) açtı. Burada gündelik
// soruyu cevaplıyoruz: "bu türü thread'e verebilir miyim, paylaşabilir miyim?"
//
//   Tür                Send   Sync   Neden
//   -----------------  -----  -----  ------------------------------------------------------
//   i32, String, Vec   ✓      ✓      iç değişkenlik yok; &T yalnız okur
//   Cell<T>, RefCell   T:Send ✗      taşınabilir, ama &Cell üzerinden kilitsiz yazılır
//   Rc<T>              ✗      ✗      sayaç atomik değil; klonlar aynı sayaca dokunur
//   Arc<T>             T:Send+Sync   paylaşılan T'ye her thread'den &T ile erişilir
//   Mutex<T>           T:Send  → ✓   kilit erişimi TEKE indirir: T'nin Sync olması gerekmez
//   RwLock<T>          Sync için T: Send + Sync — çünkü aynı anda ÇOK okuyucu &T görür
//   MutexGuard<T>      ✗      (T:Sync) kilidi alan thread bırakmalı
//   mpsc::Sender<T>    T:Send → ✓ ✓  gönderen klonlanıp thread'lere dağıtılabilir
//
// Mutex ile RwLock arasındaki fark tabloda en öğretici satırdır: Mutex<Cell<u8>> Sync'tir,
// RwLock<Cell<u8>> DEĞİLDİR.
// SADE ANLATIM: Bir kargo şirketi düşün. `Send`, "bu paket başka şubeye yollanabilir"
// damgasıdır. `Sync`, "bu vitrine birçok şubeden aynı anda bakılabilir" damgasıdır. Tablodaki
// en önemli satır iki kilidin farkıdır: `Mutex` odasına her seferinde tek kişi girer, bu
// yüzden içerideki eşyanın yalnız taşınabilir (`Send`) olması yeter. `RwLock` odasına birçok
// okuyucu birlikte girer; içerideki eşyanın birlikte bakılmaya da uygun (`Sync`) olması gerekir.
//   Mini örnek:
//     sync_ister::<Mutex<Cell<u8>>>();          // ✓ tek kişi girer: Cell'in Send olması yetti
//     // sync_ister::<RwLock<Cell<u8>>>();      // ✗ derlenmez: iki okuyucu aynı Cell'e yazardı
//     send_ister::<Arc<Mutex<Vec<u8>>>>();      // ✓ thread'ler arası paylaşımın klasik kalıbı
//     // send_ister::<Arc<RefCell<u8>>>();      // ✗ derlenmez: RefCell birlikte bakılmaya uymaz
fn send_ister<T: Send>() {}
fn sync_ister<T: Sync>() {}

fn send_sync_tablosu() {
    send_ister::<Arc<Mutex<Vec<u8>>>>();
    sync_ister::<Arc<RwLock<String>>>();
    send_ister::<Cell<u8>>(); // taşınabilir
    sync_ister::<Mutex<Cell<u8>>>(); // Mutex, !Sync bir türü Sync yapar
    println!("   derleme anı üyelik testleri geçti (kod çalıştırmadan, yalnız derlenerek)");
    // ✗ Aşağıdakiler DERLENMEZ (hepsi doğrulandı):
    //   sync_ister::<Cell<u8>>();            // E0277: `Cell<u8>` cannot be shared between threads safely
    //   sync_ister::<RwLock<Cell<u8>>>();    // E0277: aynı — iki okuyucu aynı Cell'e yazabilirdi
    //   send_ister::<MutexGuard<'static, u8>>(); // E0277: `MutexGuard<'static, u8>` cannot be sent ...
    //   send_ister::<Arc<RefCell<u8>>>();    // E0277: `RefCell<u8>` cannot be shared ... (Arc, T: Sync ister)
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn rwlock_okuyucular_birlikte_yazar_tek_basina() {
        let depo = AyarDeposu::yeni();
        depo.yaz("a", "1");
        let o1 = depo.ayarlar.read().unwrap();
        assert!(depo.ayarlar.try_read().is_ok());
        assert!(depo.ayarlar.try_write().is_err());
        drop(o1);
        assert!(depo.ayarlar.try_write().is_ok());
        assert_eq!(depo.surum(), 1);
    }

    #[test]
    fn sinirli_kuyruk_hicbir_ogeyi_kaybetmez() {
        let kuyruk = SinirliKuyruk::yeni(2);
        assert_eq!(uret_tuket(&kuyruk, 200), 2 * (200 * 201 / 2));
        assert!(kuyruk.en_yuksek_doluluk() <= 2);
    }

    #[test]
    fn kapali_kuyruk_kalanlari_verir_sonra_none() {
        let kuyruk = SinirliKuyruk::yeni(3);
        kuyruk.koy(1).unwrap();
        kuyruk.koy(2).unwrap();
        kuyruk.kapat();
        assert_eq!(kuyruk.koy(3), Err(3));
        assert_eq!(
            (kuyruk.al(), kuyruk.al(), kuyruk.al()),
            (Some(1), Some(2), None)
        );
    }

    #[test]
    fn dolu_kuyruk_ureticiyi_bekletir() {
        let kuyruk = SinirliKuyruk::yeni(1);
        kuyruk.koy(1).unwrap();
        thread::scope(|k| {
            let uretici = k.spawn(|| kuyruk.koy(2)); // yer açılana dek bloklanır
            thread::sleep(std::time::Duration::from_millis(20));
            assert!(!uretici.is_finished()); // dolu kuyrukta koy() dönmemiş olmalı
            assert_eq!(kuyruk.al(), Some(1)); // yer aç → üretici uyanır
            assert_eq!(uretici.join().unwrap(), Ok(()));
        });
        assert_eq!(kuyruk.al(), Some(2));
        assert_eq!(kuyruk.en_yuksek_doluluk(), 1); // kapasite hiç aşılmadı
    }

    #[test]
    fn karsilikli_transferler_kilitlenmez_ve_toplami_korur() {
        let a = Hesap::yeni(1, 500);
        let b = Hesap::yeni(2, 500);
        thread::scope(|k| {
            for _ in 0..4 {
                k.spawn(|| (0..200).filter(|_| transfer(&a, &b, 1)).count());
                k.spawn(|| (0..200).filter(|_| transfer(&b, &a, 1)).count());
            }
        });
        assert_eq!(a.bakiye() + b.bakiye(), 1_000);
        assert!(!transfer(&a, &a, 1));
        assert!(!transfer(&a, &b, -5)); // negatif tutar reddedilir
        assert!(!transfer(&a, &b, 0));
    }

    #[test]
    fn compare_exchange_tek_kazanan_secer() {
        for _ in 0..20 {
            let (lider, kazanan_sayisi) = lider_sec(6);
            assert_eq!(kazanan_sayisi, 1);
            assert!((1..=6).contains(&lider));
        }
    }
}
