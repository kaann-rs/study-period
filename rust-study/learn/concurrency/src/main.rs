// Rust Book — Bölüm 16: Fearless Concurrency (korkusuz eşzamanlılık)
// Çalıştır: cargo run -p concurrency
//
// "Korkusuz" çünkü sahiplik + tür sistemi, VERİ YARIŞINI (data race: en az iki thread aynı
// veriye erişir, biri yazar, eşzamanlama yoktur) DERLEME ANINDA engeller. C/C++'ta bu bir
// çalışma-anı bilinmeyeniyken Rust'ta derlenmeyen bir programdır.
//
// Doğrulanmış (rustc ile): Rc bir thread'e taşınamaz → E0277 "`Rc<i32>` cannot be sent between
// threads safely" (Rc, Send değil). Ayrıntı: DERLEYİCİ PERSPEKTİFİ (altta).

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    threadler();
    kanallar();
    paylasimli_durum();
}

// ============================================================================
// 1. THREAD'LER — thread::spawn + join + move
// ============================================================================
fn threadler() {
    let veri = vec![1, 2, 3];
    // `move`: `veri`nin SAHİPLİĞİ thread'e geçer. Neden şart? Thread, onu doğuran fonksiyondan
    // UZUN yaşayabilir; ödünç alsaydı dangling olurdu — derleyici move ister.
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
fn kanallar() {
    let (tx, rx) = mpsc::channel(); // tx: gönderen (transmitter), rx: alan (receiver)

    // Çok ÜRETİCİ: tx klonlanabilir, her thread kendi kopyasıyla gönderir. Tek TÜKETİCİ: rx.
    for id in 0..3 {
        let tx = tx.clone();
        thread::spawn(move || {
            // send, DEĞERİ TAŞIR: gönderdikten sonra o thread'de artık kullanılamaz (sahiplik
            // alana geçti) → aynı veriye iki yerden erişim en baştan imkânsız.
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
fn paylasimli_durum() {
    // Mutex<T> (mutual exclusion): veriye aynı anda TEK thread erişsin. lock() bir kilit alır,
    // MutexGuard döner; guard kapsam dışına çıkınca kilit OTOMATİK bırakılır (RAII, Böl. 15 Drop).
    // Arc<T> (atomic Rc): Rc'nin thread-güvenli sürümü — sayaç atomik olarak artar/azalır.
    let sayac = Arc::new(Mutex::new(0));
    let mut tutamaclar = vec![];

    for _ in 0..10 {
        let sayac = Arc::clone(&sayac); // her thread'e paylaşımlı bir sahip (sayaç +1)
        tutamaclar.push(thread::spawn(move || {
            let mut n = sayac.lock().unwrap(); // kilidi al; .unwrap() çünkü poison olabilir
            *n += 1; // kilit bende → güvenle değiştir
        })); // n burada düşer → kilit bırakılır
    }
    for t in tutamaclar {
        t.join().unwrap();
    }
    // 10 thread, her biri +1 → tam 10. Veri yarışı OLMADAN deterministik sonuç.
    println!("Arc<Mutex> sayaç = {}", *sayac.lock().unwrap());
    // Neden Arc + Mutex AYRI? Arc "çok sahip"liği, Mutex "güvenli değişim"i çözer. Rc burada
    // DERLENMEZ (Send değil); Mutex'siz Arc salt-okunur olurdu (içindekini değiştiremezdin).
}

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
//   sayaçlı Arc (library/alloc/src/sync.rs) kullanılır — aynı API, atomik sayaç (biraz daha pahalı).
//
// • Mutex<T> (library/std/src/sync/poison/mutex.rs:227) kilit + veriyi BİRLİKTE tutar: veriye
//   ancak lock() ile MutexGuard alarak erişirsin, guard düşünce kilit bırakılır (Drop). Böylece
//   "önce kilitle" KURALI unutulamaz — tür sistemi veriyi kilidin ARKASINA koyar. "poison":
//   kilidi tutan thread panik ederse Mutex zehirlenir; sonraki lock() Err döner (veri tutarsız
//   olabilir diye uyarı) — bu yüzden .unwrap() görürsün.
//
// • SONUÇ: veri yarışının tanımı "paylaşımlı + değiştirilebilir + eşzamanlamasız erişim".
//   Rust'ın tek-yazan-veya-çok-okuyan kuralı (Aliasing XOR Mutability) tek thread'de olduğu gibi
//   thread'ler arası da geçerlidir; Send/Sync bu kuralı thread SINIRINA taşır. İhlal, çalışma
//   anı kumarı değil, bir DERLEME hatasıdır — "fearless" tam da bu: derlenen eşzamanlı kod,
//   veri yarışından (tanım gereği) azadedir. (Deadlock/mantık hatası ayrı mesele; onları önlemez.)
