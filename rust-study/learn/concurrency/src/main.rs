// Açıklama: learn/concurrency/ACIKLAMA.md
// Rust Book — Bölüm 16: Fearless Concurrency (korkusuz eşzamanlılık)
// Çalıştır: cargo run -p concurrency
//
// "Korkusuz" çünkü sahiplik + tür sistemi, VERİ YARIŞINI (data race: en az iki thread aynı
// veriye erişir, biri yazar, eşzamanlama yoktur) DERLEME ANINDA engeller. C/C++'ta bu bir
// çalışma-anı bilinmeyeniyken Rust'ta derlenmeyen bir programdır.
//
// Doğrulanmış (rustc ile): Rc bir thread'e taşınamaz → E0277 "`Rc<i32>` cannot be sent between
// threads safely" (Rc, Send değil). Ayrıntı: DERLEYİCİ PERSPEKTİFİ (altta).

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Her bölümün "SADE ANLATIM" kutusunu önce oku; teknik kısım zorsa ana fikir orada.
// Benzetmeler bir lokanta MUTFAĞINDA geçer; derinlesme.rs de orada sürer.
//
// Bu bölümün tek cümlelik özeti: Bir program işini birkaç kola (thread) bölebilir; Rust, iki
// kolun aynı veriyi sırasız değiştirmesine yol açacak kodu daha derlerken geri çevirir.
//
// MİNİ SÖZLÜK (bu dosyada geçen kelimeler):
//   thread (iş parçacığı)    → programın içinde kendi işini yürüten ayrı kol: mutfakta bir aşçı.
//   eşzamanlılık (concurrency) → birden çok işin aynı dönemde İLERLEMESİ (sırayla da olabilir).
//   paralellik (parallelism) → birden çok işin gerçekten AYNI ANDA yapılması. İkisi aynı şey değil.
//   veri yarışı (data race)  → iki thread'in aynı veriye sırasız uzanması ve en az birinin yazması.
//   thread::spawn            → yeni bir thread başlatmak (mutfağa bir aşçı daha çağırmak).
//   closure (`|| ...`)       → adı olmayan, yerinde yazılan küçük fonksiyon; thread'e verilen iş.
//   move                     → closure'ın kullandığı verinin sahipliğini closure'a vermek.
//   tutamaç (handle) / join  → başlattığın thread'i gösteren fiş / "bitmesini bekliyorum" demek.
//   dangling (sarkan)        → gösterdiği veri silinmiş olan referans. Rust buna izin vermez.
//   kanal (channel)          → thread'ler arası tek yönlü ileti hattı. Servis penceresi gibi.
//   tx / rx                  → kanalın gönderen ucu (transmitter) / alan ucu (receiver).
//   mpsc                     → "çok gönderen, tek alan" kanal türünün kısa adı.
//   Mutex<T>                 → içindeki veriye aynı anda tek thread'in girmesini sağlayan kilit.
//   lock() / guard (MutexGuard) → kilidi almak / elindeki anahtar: yok olunca kilit açılır.
//   Arc<T>                   → bir veriyi birkaç thread'in paylaşmasını sağlayan sayaçlı kutu.
//   atomik (atomic)          → iki thread aynı anda yapsa bile bozulmayan, tek hamlelik işlem.
//   Send / Sync              → "başka thread'e verilebilir" / "birkaç thread birlikte bakabilir".
//   marker trait (işaret trait'i) → metodu olmayan, türe yalnız bir etiket yapıştıran trait.
//   poison (zehirlenme)      → kilidi tutan thread çökünce kilide konan "dikkat" işareti.
//   deadlock (kilitlenme)    → iki thread'in birbirini sonsuza dek beklemesi. Rust bunu ÖNLEMEZ.
//   deterministik            → her çalıştırmada aynı sonucu veren.
//   lost update              → iki yazmadan birinin ötekini ezmesi, bir artışın kaybolması.

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

mod derinlesme;

fn main() {
    threadler();
    kanallar();
    paylasimli_durum();
    derinlesme::calistir();
}

// ============================================================================
// 1. THREAD'LER — thread::spawn + join + move
// ============================================================================
// SADE ANLATIM: Mutfakta tek aşçı varken yemekler sırayla yapılır. İkinci bir aşçı çağırırsan
// (`thread::spawn`) o çorbayı karıştırırken sen salatayı hazırlarsın. Ona verdiğin malzeme artık
// onundur (`move`): sen mutfaktan çıksan bile elindeki malzeme ortadan kalkmaz. `join()` ise
// "işini bitirmeni bekliyorum, sonucu bana ver" demektir.
//   Mini örnek:
//     let malzeme = vec![1, 2, 3];
//     let asci = thread::spawn(move || malzeme.len());   // malzeme aşçıya geçti
//     // println!("{malzeme:?}");                        // ✗ derlenmez: malzeme artık bizde değil
//     let sonuc = asci.join().unwrap();                  // ✓ aşçıyı bekle, sonucu al: 3
fn threadler() {
    let veri = vec![1, 2, 3];
    // `move`: sahiplik thread'e geçer — thread bu fonksiyondan uzun yaşayabilir, ödünç olsa dangling olurdu.
    let tutamac = thread::spawn(move || {
        let toplam: i32 = veri.iter().sum();
        toplam // thread'in dönüş değeri
    });
    // join(): thread'in BİTMESİNİ bekler; Result döner (thread panikleşmişse Err).
    let sonuc = tutamac.join().unwrap();
    println!("thread topladı = {sonuc}");
}

// ============================================================================
// 2. KANALLAR (channels) — "paylaşma, mesajlaş": mpsc = multi-producer, single-consumer
// ============================================================================
// SADE ANLATIM: Aşçılar birbirinin tezgâhına uzanmaz; hazır olan tabağı servis penceresine
// koyar, garson oradan alır. Kanal bu penceredir: `tx` tabağı koyan el, `rx` alan eldir. Pencereye
// birçok aşçı tabak koyabilir (`tx.clone()`), alan ise tektir. Tabağı koyan aşçı ona bir daha
// dokunamaz, çünkü `send` tabağın sahipliğini karşıya geçirir. Bütün aşçılar gidince (bütün
// `tx`'ler silinince) garson "başka tabak gelmeyecek" diye anlar ve beklemeyi bırakır.
//   Mini örnek:
//     let (tx, rx) = mpsc::channel();
//     let tabak = String::from("çorba");
//     thread::spawn(move || {
//         tx.send(tabak).unwrap();           // tabak pencereye kondu, sahipliği karşıya geçti
//         // println!("{tabak}");            // ✗ derlenmez: gönderdiğin tabak artık sende değil
//     });
//     println!("{}", rx.recv().unwrap());    // ✓ çorba
fn kanallar() {
    let (tx, rx) = mpsc::channel(); // tx: gönderen (transmitter), rx: alan (receiver)

    // Çok ÜRETİCİ: tx klonlanabilir, her thread kendi kopyasıyla gönderir. Tek TÜKETİCİ: rx.
    for id in 0..3 {
        let tx = tx.clone();
        thread::spawn(move || {
            // send sahipliği TAŞIR: gönderen artık o veriye erişemez — iki yerden erişim baştan imkânsız.
            tx.send(format!("thread {id} selam")).unwrap();
        });
    }
    drop(tx); // orijinal gönderen kapanmazsa rx döngüsü asla bitmez (tüm tx'ler düşmeli)

    // rx bir iterator gibi: tüm gönderenler kapanınca döngü biter. Varış SIRASI belirsiz → sırala.
    let mut mesajlar: Vec<String> = rx.iter().collect();
    mesajlar.sort();
    println!("kanaldan gelenler (sıralı): {mesajlar:?}");
}

// ============================================================================
// 3. PAYLAŞIMLI DURUM — Arc<Mutex<T>>: "mesajlaş" yerine "güvenle paylaş"
// ============================================================================
// SADE ANLATIM: Bazı şeyler elden ele gezmez, yerinde durur: mutfağın kileri gibi. Kilerin tek
// anahtarı vardır (Mutex). Anahtarı alan (`lock()`) içeri girer, ötekiler kapıda bekler; çıkan
// kişi anahtarı unutamaz, çünkü işi bitince anahtar kendiliğinden yerine asılır. Arc ise kilerin
// yerini gösteren karttır: her aşçıya bir kart verilir, son kart da atılınca kiler kapatılır.
// Dikkat: kilit veri yarışını önler, ama deadlock'u (iki aşçı birbirinin anahtarını bekler) Rust önlemez.
//   Mini örnek:
//     let kiler = Arc::new(Mutex::new(0));
//     let kiler2 = Arc::clone(&kiler);            // ikinci aşçıya da bir kart verildi
//     thread::spawn(move || {
//         *kiler2.lock().unwrap() += 1;           // ✓ anahtarı al, değiştir, satır bitince bırak
//     })
//     .join()
//     .unwrap();
//     println!("{}", *kiler.lock().unwrap());     // ✓ 1
//     // Arc yerine Rc yazsaydık                  // ✗ derlenmez: Rc başka thread'e verilemez
fn paylasimli_durum() {
    // Mutex<T> (mutual exclusion): veriye aynı anda TEK thread erişsin. lock() bir kilit alır,
    // MutexGuard döner; guard kapsam dışına çıkınca kilit OTOMATİK bırakılır (RAII, Böl. 15 Drop).
    // Arc<T> (atomic Rc): Rc'nin thread-güvenli sürümü — sayaç atomik olarak artar/azalır.
    let sayac = Arc::new(Mutex::new(0));
    let mut tutamaclar = vec![];

    for _ in 0..10 {
        let sayac = Arc::clone(&sayac); // her thread'e paylaşımlı bir sahip (sayaç +1)
        tutamaclar.push(thread::spawn(move || {
            let mut n = sayac.lock().unwrap(); // kilidi al; poison (thread çökerse "yarım kalmış" işareti) olabilir → unwrap durdurur
            *n += 1; // kilit bende → güvenle değiştir
        })); // n burada düşer → kilit bırakılır
    }
    for t in tutamaclar {
        t.join().unwrap();
    }
    // 10 thread, her biri +1 → tam 10. Veri yarışı OLMADAN deterministik sonuç.
    println!("Arc<Mutex> sayaç = {}", *sayac.lock().unwrap());
    // Arc "çok sahip"liği, Mutex "güvenli değişim"i çözer; Mutex'siz Arc salt-okunur olurdu (Rc DERLENMEZ).
}

// SADE ANLATIM (aşağıdaki bölüm için): Derleyici her türün üstünde iki görünmez etiket arar:
// Send ("başka bir aşçıya verilebilir") ve Sync ("birkaç aşçı aynı anda bakabilir"). Etiketi
// olmayan bir şeyi başka thread'e vermeye kalkarsan kod derlenmez; Rc bu yüzden mutfak kapısından
// geçemez, Arc geçer. Aşağıdaki bölüm bu etiketlerin nerede tanımlandığını ve Mutex'in veriyi
// neden kilidin ARKASINDA tuttuğunu anlatır. İlk okuyuşta atlayabilirsin. Aklında kalması gereken
// tek şey: derlenen kodda veri yarışı olmaz, ama deadlock ve mantık hataları hâlâ olabilir.
// ============================================================================
// DERLEYİCİ PERSPEKTİFİ — veri yarışı neden derlenmiyor?
// ============================================================================
//
// • İki İŞARET TRAİTİ (marker trait) her şeyin temeli (library/core/src/marker.rs):
//     - Send (satır 92): "bu tür başka bir thread'e GÜVENLE TAŞINABİLİR". thread::spawn'a
//       verilen closure'ın yakaladığı her şey Send olmalı.
//     - Sync (satır 657): "bu türe &T ile birden çok thread'den GÜVENLE ERİŞİLEBİLİR"
//       (yani &T Send ise T Sync'tir).
//   İkisi de `unsafe auto trait`: derleyici, alanları uygunsa bir türe OTOMATİK uygular;
//   programcı elle yazmaz. Çoğu tür Send+Sync'tir.
//
// • Rc NEDEN Send DEĞİL? Rc'nin sayacı SIRADAN (atomik olmayan) bir tam sayıdır. İki thread
//   aynı anda Rc::clone/düşürme yapsa sayaç bozulur (lost update) → ya erken serbest bırakma ya
//   sızıntı → bellek güvensizliği. Bu yüzden Rc bilinçli olarak !Send'dir; thread'e taşımaya
//   kalkınca E0277 "cannot be sent between threads safely" (DOĞRULANDI). Çok-thread için atomik
//   sayaçlı Arc (library/alloc/src/rcs/arc.rs; sync.rs yeniden dışa aktarır) kullanılır — aynı API,
//   atomik sayaç (biraz daha pahalı).
//
// • Mutex<T> (library/std/src/sync/poison/mutex.rs:227) kilit + veriyi BİRLİKTE tutar: veriye
//   ancak lock() ile MutexGuard alarak erişirsin, guard düşünce kilit bırakılır (Drop). Böylece
//   "önce kilitle" KURALI unutulamaz — tür sistemi veriyi kilidin ARKASINA koyar. "poison":
//   kilidi tutan thread panik ederse Mutex zehirlenir; sonraki lock() Err döner (veri tutarsız
//   olabilir diye uyarı) — bu yüzden .unwrap() görürsün.
//
// • SONUÇ: veri yarışı = "paylaşımlı + değiştirilebilir + eşzamanlamasız erişim". Rust'ın
//   Aliasing XOR Mutability kuralı thread'ler arasında da geçerlidir; Send/Sync bunu thread SINIRINA
//   taşır → ihlal DERLEME hatasıdır ("fearless"). Deadlock/mantık hatası ayrı mesele.
