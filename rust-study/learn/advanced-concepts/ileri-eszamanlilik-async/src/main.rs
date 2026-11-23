// Sağlamlaştırma — İleri konu: İLERİ EŞZAMANLILIK ve ASYNC RUST
// Çalıştır: cargo run -p ileri-eszamanlilik-async   |   Test: cargo test -p ileri-eszamanlilik-async
//
// Kaynaklar: std (std::sync, std::sync::atomic, std::task, std::pin) + Rust Atomics and Locks
// (Mara Bos) + Rust for Rustaceans Böl. 8 (Asynchronous Programming) ve Böl. 10 (Concurrency).
//
// Önceki crate'ler parçaları tek tek kurdu:
//   concurrency (Böl. 16)   → thread, kanal, Arc<Mutex>, scope, işçi havuzu
//   async-await (Böl. 17)   → Future, elle executor, join, Pin'in gündelik yüzü
//   ileri-future            → poll/wake sözleşmesi, Waker'ın iç yapısı, self-referential + Pin
//   ileri-send-sync         → UnsafeCell, Send/Sync auto-trait mekanizması
// Bu crate onları "üretimde karşılaşılan" sorulara bağlar. Diğer ileri-* crate'lerinden farklı
// olarak üç dosyaya bölündü, çünkü üç ayrı katman var ve her biri kendi başına okunabilir:
//
//   threadler.rs       A) Thread tarafı — doğru kilidi seçmek ve deadlock'u tasarımla önlemek:
//                         RwLock, Condvar (sınırlı kuyruk), kilit sıralaması, atomikler,
//                         Send/Sync tablosu.
//   calisma_zamani.rs  B) Async tarafı — bir runtime GERÇEKTE ne yapar: görev kuyruğu, spawn,
//                         JoinHandle, zamanlayıcı, zaman aşımı; Pin ve Unpin pratikte.
//   istemci.rs         C) Uygulama — asenkron API istemcisi (eşzamanlı istek, zaman aşımı,
//                         yeniden deneme) ve async kodda yaygın hatalar.
//
// ÖZET ZİHİNSEL MODEL:
//   Thread'ler ve async görevler AYNI soruya iki cevaptır: "beklerken ne yapalım?"
//     Thread: işletim sistemi thread'i uyutur, başkasını koşturur. Basit, ama her bekleyen iş
//             bir thread (yığın belleği + bağlam değişimi) demektir.
//     Async : görev "şimdilik bu kadar" (Pending) deyip thread'i GERİ VERİR; çalıştırıcı aynı
//             thread'de başka göreve geçer. On binlerce bekleyen iş birkaç thread'le yürür,
//             ama görevlerin iş birliği şarttır: bloklayan tek görev herkesi durdurur.
//   Paylaşılan duruma erişim kuralları ikisinde de aynıdır (Send, Sync, kilit, atomik); async
//   yalnızca "kilidi await boyunca tutma" kuralını ekler.
//
// Not: süreye bağlı çıktılar (kaç ms sürdü) makineden makineye değiştiği için yalnız
// KARŞILAŞTIRMA sonuçları (true/false) basılır; bunlar her çalıştırmada aynıdır.
// ✗ işaretli derlenmeyen örneklerin hata kodları/mesajları rustc 1.99 ile doğrulandı.

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Bu crate'in dört dosyasında da her bölümün üstünde "SADE ANLATIM" diye bir kutu var. Önce
// onu oku, sonra altındaki teknik açıklamaya geç. Teknik kısım zor gelirse sorun değil: sade
// kutu ana fikri verir. Okuma sırası: bu dosya → threadler.rs → calisma_zamani.rs → istemci.rs.
// Ana sözlük aşağıdadır; öteki üç dosya yalnız kendi yeni kelimelerini ekler.
//
// Bu dosyayı okumadan önce bilmen gerekenler: `learn/ownership`, `learn/smart-pointers` (Arc),
// `learn/concurrency` (thread, Mutex, kanal), `learn/async-await` (async/await kullanımı) ve
// bu klasördeki `ileri-future` ile `ileri-send-sync`.
//
// Bu bölümün tek cümlelik özeti: aynı anda birçok iş yürütmenin iki yolu vardır (her işe bir
// işçi vermek ya da az sayıda işçinin bekleyen işi bırakıp sıradakine geçmesi) ve ikisinde de
// ortak eşyaya kimin, ne zaman dokunacağını kurala bağlamak gerekir.
//
// MİNİ SÖZLÜK (bu crate'te geçen ortak kelimeler):
//   eşzamanlılık         → birden çok işin aynı zaman diliminde ilerlemesi (concurrency).
//   thread (iş parçacığı)→ işletim sisteminin yönettiği ayrı bir işçi. Kendi belleği vardır.
//   bağlam değişimi      → işlemcinin bir thread'i bırakıp ötekine geçmesi; bedava değildir.
//   yığın (stack)        → her thread'in kendi çalışma masası; yerel değişkenler orada durur.
//   heap                 → ortak büyük depo; Box ve Arc değerleri oraya koyar.
//   veri yarışı          → iki thread'in aynı veriye aynı anda dokunup en az birinin yazması.
//   kilit (lock) / Mutex → içeriğe aynı anda yalnız bir thread'in girmesini sağlayan kapı.
//   guard                → kilidi aldığında eline verilen anahtar; düşünce kilit açılır.
//   RAII                 → "değer silinince işini de kendisi kapatır" düzeni (guard böyledir).
//   kritik bölge         → kilidin tutulduğu kod parçası.
//   deadlock             → iki tarafın sonsuza dek birbirini beklemesi. Rust bunu ÖNLEMEZ.
//   atomik               → yarıda kesilemeyen tek hamleyle okunup yazılan küçük değer.
//   Arc                  → thread'ler arasında paylaşılan sayaçlı sahiplik işaretçisi.
//   kanal (channel)      → bir ucundan atılan değerin öteki ucundan alındığı boru.
//   Send / Sync          → "başka thread'e verilebilir" / "thread'ler birlikte bakabilir".
//   auto-trait           → derleyicinin türün alanlarına bakıp kendiliğinden verdiği trait.
//   async / .await       → "bu iş bekleyebilir" diye yazılan fonksiyon / bekleme noktası.
//   future               → henüz bitmemiş iş. Poll edilince ilerler, kendi kendine çalışmaz.
//   poll / Pending / Ready → "biraz ilerle" demek / "daha bitmedi" / "bitti, sonuç bu".
//   görev (task)         → çalıştırıcıya teslim edilmiş, bağımsız yürüyen future.
//   çalıştırıcı / runtime→ görevleri poll eden program parçası (executor).
//   waker                → "ben hazırım, bana yine bak" zili.
//   spawn (başlatmak)    → yeni bir thread ya da görev başlatmak.
//   JoinHandle (tutamaç) → başlattığın işin sonucunu sonradan almaya yarayan fiş.
//   zamanlayıcı          → süre dolunca haber veren parça (timer).
//   zaman aşımı          → "şu süreden uzun sürerse bırak" kuralı (timeout).
//   bloklamak            → bir thread'i iş bitene kadar başka hiçbir şey yapamaz hâlde tutmak.
//   iptal                → bir future'ı bitmeden düşürmek; async'te iptal budur.
//   state machine        → "hangi adımdayım?" bilgisini saklayıp kaldığı yerden süren yapı.
//   Pin / Unpin          → "gösterdiğim değer yerinden oynamayacak" kilidi / kilide gerek
//                          duymayan türlerin işareti.
//   G/Ç (I/O)            → dış dünyayla alışveriş: dosya, ağ, klavye.

mod calisma_zamani;
mod istemci;
mod threadler;

fn main() {
    println!("######## A) İLERİ EŞZAMANLILIK (thread'ler) ########");
    threadler::calistir();

    println!("\n######## B) MİNİ ASYNC ÇALIŞMA ZAMANI ########");
    calisma_zamani_tanitimi();

    println!("\n######## C) UYGULAMA: API İSTEMCİSİ ve YAYGIN HATALAR ########");
    istemci::calistir();
}

// calisma_zamani.rs'teki parçaların küçük bir turu: görev başlat, uyut, sonucunu bekle.
// SADE ANLATIM: İki arkadaşına aynı anda iş verdin: biri 30 dakikada, öteki 10 dakikada
// bitirecek. İkisi de hemen işe koyulur. Sen önce yavaş olanın kapısında beklersin; o bittiğinde
// hızlı olan çoktan bitmiştir, ona sorar sormaz cevabı alırsın. Toplam bekleme 40 değil,
// yaklaşık 30 dakikadır. Aşağıdaki kod bunu yapar: `baslat` işi hemen sıraya koyar, `.await`
// yalnız sonucu bekler. Bu neden var? Bekleyen işler birbirinin süresini uzatmasın diye.
//   Mini örnek:
//     let yavas = baslatici.baslat(async { uyu_ms(30).await; "yavaş" });  // hemen sıraya girdi
//     let hizli = baslatici.baslat(async { uyu_ms(10).await; "hızlı" });  // o da
//     let a = yavas.await;     // ~30 ms bekler
//     let b = hizli.await;     // ✓ beklemez: hızlı görev çoktan bitti
//     // toplam ≈ 30 ms; art arda çalışsalardı 40 ms sürerdi
fn calisma_zamani_tanitimi() {
    use calisma_zamani::{Calistirici, uyu_ms};

    let mut calistirici = Calistirici::yeni();
    let baslatici = calistirici.baslatici();
    let ic_baslatici = baslatici.clone();

    // Kök görev iki alt görev başlatır ve sonuçlarını bekler (yapılandırılmış bir "join").
    let kok = baslatici.baslat(async move {
        let yavas = ic_baslatici.baslat(async {
            uyu_ms(30).await;
            "yavaş görev (30 ms)"
        });
        let hizli = ic_baslatici.baslat(async {
            uyu_ms(10).await;
            "hızlı görev (10 ms)"
        });
        // Bekleme sırası sonuç sırasını belirler; görevler ise zaten eşzamanlı ilerliyor.
        // `yavas.await` beklerken `hizli` çoktan bitmiş olur — ikinci await anında döner.
        [yavas.await, hizli.await]
    });

    let poll_sayisi = calistirici.calistir();
    println!("   sonuçlar = {:?}", kok.sonucu_al().unwrap());
    println!("   toplam poll = {poll_sayisi} (3 görev; yalnız uyandırılan poll edildi)");
    // Beklenen çıktı:
    //   sonuçlar = ["yavaş görev (30 ms)", "hızlı görev (10 ms)"]
    //   toplam poll = 6 (3 görev; yalnız uyandırılan poll edildi)
    // Sayım: kök 2 (ilk poll + yavaş görev bitince), yavaş 2, hızlı 2. `hizli` bittiğinde kökü
    // UYANDIRMAZ: kök o sırada `hizli`yı değil `yavas`ı bekliyor, tutamaca waker kaydetmedi.
    // 40 ms boyunca çalıştırıcı thread'i neredeyse hep uykudaydı (kanal recv'inde).
}

// SADE ANLATIM (aşağıdaki bölüm için): Üç dosyada kilitleri, atomikleri, küçük bir çalışma
// zamanını ve bir istemciyi gördün. Aşağıdaki bölüm perdenin arkasını özetler: derleyici
// kilitleri tanımaz, yalnız "bu değer başka thread'e geçebilir mi?" sorusunu denetler;
// `async fn` bir durum makinesine dönüşür; gerçek çalışma zamanları aynı iskelete daha çok
// işçi ve işletim sistemi desteği ekler. İlk okuyuşta atlayabilirsin; üç dosyadaki sade
// kutuları anlamış olman yeter.
// ============================================================================
// DERLEYİCİ / std PERSPEKTİFİ
// ============================================================================
//
// • Mutex/RwLock/Condvar birer KÜTÜPHANE türüdür; derleyicinin kilitlerden haberi yoktur.
//   Güvenliği sağlayan iki genel mekanizmadır: (1) veriye yalnız guard üzerinden erişilir ve
//   guard'ın ömrü kilidin süresidir (RAII + borrow checker), (2) Send/Sync auto-trait'leri hangi
//   türün thread sınırını geçebileceğini belirler. "Kilidi almadan veriye dokundum" hatası bu
//   yüzden yazılamaz; "kilitleri yanlış sırayla aldım" hatası ise yazılabilir.
//
// • Atomiklerin Ordering'i işlemciye/derleyiciye verilen bir İZİNDİR: daha zayıf sıralama, daha
//   fazla yeniden sıralama özgürlüğü demektir. Yanlış seçilmiş Relaxed çoğu x86 makinesinde
//   "çalışıyor" görünür, ARM'da bozulur; bu sınıf hatalar testle bulunmaz. Ölçmeden zayıflatma.
//
// • `async fn` → state machine. Makine, bir await noktasında CANLI olan her yereli alan olarak
//   saklar; auto-trait'ler (Send, Sync, Unpin) bu alanlardan türetilir. "future is not Send"
//   hatasının kaynağı budur ve çözüm her zaman aynıdır: sorunlu değeri await'ten önce düşür.
//
// • Waker Send + Sync olmak zorundadır (her thread'den uyandırılabilmeli); bu yüzden
//   std::task::Wake, Arc<W: Send + Sync> ister ve görevimiz future'ı bir Mutex'in ardında tutar.
//   Tek thread'li runtime'lar (tokio'nun LocalSet'i gibi) !Send future'ları da koşturabilir,
//   bedeli görevlerin o thread'den hiç ayrılamamasıdır.
//
// • Pin bir İŞARETÇİ SARMALAYICISIDIR, veri özelliği değil: `Pin<&mut T>` "bu T bir daha
//   taşınmayacak" sözüdür. T: Unpin ise söz boştur (get_mut serbest). async blokların future'ı
//   !Unpin'dir; onları sabitlemenin iki güvenli yolu pin! (yığın) ve Box::pin (heap).
//
// • Gerçek runtime'lar bu iskeletin üstüne şunları ekler: çok thread'li iş çalma (work
//   stealing) kuyruğu, işletim sistemi olay bildirimiyle (epoll/kqueue/IOCP) bütünleşik G/Ç,
//   tek bir zamanlayıcı yapısı, adil zamanlama ve iptal güvenliği yardımcıları. Kavramlar
//   (görev, waker, kuyruk, yaprak future) birebir aynıdır.
// (Sade: "iş çalma / work stealing" = işi biten işçinin, kuyruğu dolu olan işçiden iş alması
//  demek. "epoll/kqueue/IOCP" = işletim sisteminin "şu bağlantıda veri var" diye haber verme
//  yolları demek.)
//
// ============================================================================
// ALIŞTIRMALAR
// ============================================================================
// 1) (threadler.rs) `SinirliKuyruk`a `fn al_zaman_asimli(&self, sure: Duration) -> Option<T>`
//    ekle: süre içinde öğe gelmezse None dönsün.
//    İpucu: Condvar::wait_timeout_while; dönen ikilinin ikinci elemanı `.timed_out()` verir.
//    Test: boş kuyrukta 20 ms'lik çağrı en az 20 ms sürmeli ve None dönmeli.
//
// 2) (threadler.rs) `transfer`i üç hesaplı bir işleme genelle: `fn dagit(kaynak: &Hesap,
//    hedefler: [&Hesap; 2], tutar: i64) -> bool`. Kilit sırası yine hesap numarası olsun.
//    İpucu: üç referansı `no`ya göre sırala, sırayla kilitle, guard'ları bir Vec'te tut ve
//    numaradan guard'a eşle. Dört thread'le rastgele yönlerde çağırıp toplamın korunduğunu
//    ve programın bitebildiğini test et.
//
// 3) (calisma_zamani.rs) `pub async fn hepsini_bekle<T>(tutamaclar: Vec<Tutamac<T>>) -> Vec<T>`
//    yaz. En basit çözüm sırayla await etmektir; bu neden yine de EŞZAMANLIDIR?
//    (İpucu: görevler `baslat` anında kuyruğa girdi; await yalnız SONUCU bekler. async-await
//    crate'indeki `hepsini_bekle` ise başlatılmamış future'ları kendi poll ediyordu — farkı
//    bir cümleyle yaz.)
//
// 4) (istemci.rs) `yeniden_dene`ye toplam süre sınırı ekle: `zaman_asimi` ile sararak
//    "en fazla 4 deneme VE toplam 50 ms". Zaman aşımı denemenin ortasında dolarsa ne olur?
//    (İpucu: iç future düşürülür = iptal. `yeniden_dene`nin future'ı Send + 'static olmalı;
//    hangi parametreler bunu belirliyor?)
//
// Bağlantılar: for<'a> ve lifetime'ların async imzalarda neden zorlaştığı →
// ileri-lifetime-hrtb-gat; Drop sırası ve guard'lar → ileri-dropck.
