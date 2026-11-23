// Açıklama: learn/advanced-concepts/ileri-future/ACIKLAMA.md
// Sağlamlaştırma — İleri konu: FUTURE GERÇEKTE NEDİR, NASIL ÇALIŞIR?
// Çalıştır: cargo run -p ileri-future
//
// Kaynaklar: std (core::future, core::task) + Rust for Rustaceans (Jon Gjengset), Böl. 8
//   (Asynchronous Programming: Pin/Unpin, poll sözleşmesi, waker). Yayıncının ayrıntılı
//   içindekiler listesiyle doğrulandı.
// Böl. 17'de async'i KULLANDIK; burada MEKANİZMAYI açıyoruz: poll nedir, wake ne yapar,
// Waker gerçekte nedir, Pin neden var. Her şey std ile, bağımlılıksız; döngüyü GÖZLE göreceğiz.
//
// ÖZET ZİHİNSEL MODEL (gerisi bunun ayrıntısı):
//   Future = "henüz bitmemiş bir hesap". Onu bir EXECUTOR poll eder. poll ya Ready(değer) ya
//   Pending döner. Pending dönerken future, "ilerleyebilir hâle gelince beni uyandır" diye bir
//   WAKER saklar. İş hazır olunca waker.wake() çağrılır → executor future'ı YENİDEN poll eder.
//   Boşa dönen bekleme (busy-wait) YOK: haber verilince devam. async fn, derleyici tarafından
//   her .await'te duraklayıp devam eden bir STATE MACHINE'e çevrilir; bu state machine = Future.

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Her bölümün üstündeki "SADE ANLATIM" kutusunu önce oku; teknik kısım zor gelirse sorun değil.
//
// Bu dosyayı okumadan önce bilmen gerekenler: `learn/ownership` (taşıma ve ödünç alma),
// `learn/generics-traits-lifetimes` (trait nedir), `learn/smart-pointers` (Box, Arc) ve
// `learn/async-await` (async/await'i KULLANMAK). Burada kullanmayı değil, içini öğreniyoruz.
//
// Bu bölümün tek cümlelik özeti: future, "henüz bitmemiş bir iş"tir; kendi kendine çalışmaz,
// biri ona "biraz daha ilerle" dedikçe (poll) ilerler ve bekleyeceği zaman "hazır olunca beni
// haberdar et" diye bir zil (waker) bırakır.
//
// MİNİ SÖZLÜK (bu dosyada geçen kelimeler):
//   future               → henüz bitmemiş iş. Sonucu sonra verecek olan bir "yapılacak iş" kâğıdı.
//   poll                 → future'a "biraz daha ilerle, bitti mi?" diye sormak.
//   Poll::Ready(değer)   → "bitti, sonuç bu" cevabı.
//   Poll::Pending        → "daha bitmedi, sonra yine sor" cevabı.
//   executor (yürütücü)  → future'ları poll eden program parçası. İşi yürüten odur.
//   block_on             → "bu future bitene kadar burada bekle" diyen en basit executor.
//   waker (uyandırıcı)   → "ben hazırım, bana yine bak" demeye yarayan zil.
//   wake / wake_by_ref   → o zili çalmak. wake zili tüketir, wake_by_ref elde bırakır.
//   Context (cx)         → poll'a verilen küçük çanta; içinde o anki waker durur.
//   async fn / .await    → future üreten fonksiyon / "bu iş bitene kadar burada durakla" işareti.
//   state machine        → durum makinesi: "hangi adımdayım?" bilgisini saklayıp kaldığı yerden
//                          süren yapı. Kitap ayracı gibi düşün.
//   generator            → derleyicinin async fn'den ürettiği o durum makinesinin iç adı.
//   busy-wait            → boş yere durmadan "oldu mu, oldu mu?" diye sormak. İşlemciyi yorar.
//   park / unpark        → bir thread'i uyutmak / uyandırmak.
//   thread (iş parçacığı)→ programın içinde aynı anda çalışabilen ayrı bir işçi.
//   I/O                  → dış dünyayla alışveriş: dosya okumak, ağdan veri beklemek.
//   sözleşme (contract)  → "bunu yaparsan şunu garanti ederim" diye verilen söz, uyulacak kural.
//   trait object (dyn)   → gerçek türü gizlenmiş, yalnız "ne yapabildiği" bilinen değer.
//   vtable               → "bu değer için hangi fonksiyon çağrılacak?" listesinin tutulduğu tablo.
//   type-erased          → tür silinmiş: gerçek tür saklanır, yalnız yapabildikleri görünür.
//   pointer (işaretçi)   → bir verinin bellekteki adresini tutan ok işareti.
//   raw pointer (*const) → derleyicinin denetlemediği ham adres. Doğruluğu sana emanettir.
//   unsafe               → "buradaki kuralı derleyici değil, ben garanti ediyorum" dediğin blok.
//   heap                 → bellekteki büyük depo. Box bir değeri oraya koyar.
//   Arc                  → birçok sahibin (thread'ler arasında da) paylaştığı sayaçlı işaretçi.
//   move (taşıma)        → bir değeri bellekte başka bir yere geçirmek; adresi değişir.
//   dangling (sarkan)    → gösterdiği yerde artık doğru veri olmayan ok işareti.
//   self-referential     → kendine referanslı: içindeki bir ok, yine kendi içindeki bir alanı
//                          gösterir.
//   Pin                  → bir işaretçinin üstüne takılan kilit: "gösterdiğim değer yerinden
//                          oynatılmayacak".
//   Unpin                → "taşınsam da bozulmam" diyen türlerin işareti. Çoğu tür böyledir.
//   PhantomPinned        → bir türü bilerek Unpin OLMAYAN (!Unpin) yapan, yer kaplamayan alan.
//   Send + Sync          → başka thread'e verilebilir + thread'ler arasında paylaşılabilir.

use std::future::Future;
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::ptr;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use std::thread::{self, Thread};

fn main() {
    println!("== 1) poll / Pending / wake / yeniden-poll döngüsü (izlenebilir) ==");
    let sonuc = izle_block_on(GecikmeliDeger { kalan: 3 });
    println!("block_on döndü: {sonuc:?}\n");

    println!("== 2) Waker gerçekte nedir? ==");
    waker_nedir();

    println!("\n== 3) Pin ve self-referential (RfR Böl. 8) ==");
    pin_ve_self_referential();
}

// ============================================================================
// 1) poll NEDİR — bir hesabı "biraz ilerlet"
// ============================================================================
// poll'un SÖZLEŞMESİ (contract):
//   - İlerleyebildiğin kadar ilerle.
//   - Bitirdiysen Poll::Ready(değer) dön.
//   - Bitiremediysen (I/O hazır değil vb.): ÖNCE cx içindeki Waker'ı sakla/kaydet, SONRA
//     Poll::Pending dön. Waker'ı kaydetmeden Pending dönmek HATADIR: executor seni bir daha
//     asla poll etmez (kimse "artık hazırım" diyemez) → future sonsuza dek asılı kalır.
//
// Aşağıdaki future, "I/O 3 tur sonra hazır olacak"ı taklit eder: her poll'da bir tur azaltır,
// kendini uyandırıp Pending döner; sayaç 0'da Ready olur. Her adımı basıyoruz ki döngü görünsün.
// SADE ANLATIM: Fırına kek koydun. Kek kendi kendine "ben piştim" diye yanına gelmez; birinin
// fırının camından bakması gerekir. Bu bakışa `poll` denir. İki cevap vardır: "pişti, al"
// (`Poll::Ready`) ya da "daha olmadı" (`Poll::Pending`). "Daha olmadı" derken fırının zilini
// de kurman gerekir (waker), yoksa kimse bir daha bakmaz ve kek fırında unutulur.
// Bu neden var? Bekleyen binlerce iş için binlerce işçi tutmamak için: iş bekliyorsa kenara
// konur, zili çalınca kaldığı yerden sürer. Önemli ayrıntı: future'ı oluşturmak işi
// başlatmaz; iş ancak poll edilince ilerler.
//   Mini örnek:
//     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<u32> {
//         if self.hazir {
//             Poll::Ready(42)              // ✓ bitti: sonucu ver
//         } else {
//             cx.waker().wake_by_ref();    // ✓ önce zili kur: "bana yine bak"
//             Poll::Pending                // sonra "daha olmadı" de
//         }
//     }
//     // ✗ zili kurmadan Pending dönmek: derlenir ama future sonsuza dek bekler
struct GecikmeliDeger {
    kalan: u32,
}
impl Future for GecikmeliDeger {
    type Output = &'static str;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<&'static str> {
        println!("   [future.poll] çağrıldı — kalan = {}", self.kalan);
        if self.kalan == 0 {
            println!("   [future.poll] hazır → Poll::Ready");
            Poll::Ready("İŞ BİTTİ")
        } else {
            self.kalan -= 1;
            // Gerçekte: "beni rx hazır olunca uyandır" diye waker bir yere kaydedilir. Burada
            // hemen kendimizi uyandırıyoruz (sanki I/O anında hazır olmuş gibi) → tekrar poll.
            cx.waker().wake_by_ref();
            println!("   [future.poll] hazır değil → waker kaydedildi, Poll::Pending");
            Poll::Pending
        }
    }
}

// İzlenebilir executor: poll et; Ready ise bitir, Pending ise park ol (uyku) ve wake'i bekle.
// park/unpark burada "executor uyur / uyandırılır"ın en küçük canlandırmasıdır.
// SADE ANLATIM: Executor, göl kıyısındaki balıkçıya benzer. Oltasına bakar (poll); balık
// yoksa durmadan suya bakıp yorulmaz, uzanıp uyur (`thread::park`). Oltanın ucundaki çıngırak
// çalınca (`wake` → `unpark`) uyanır ve oltaya yeniden bakar. Balık gelene kadar bu döngü
// sürer. Bu neden var? Future'ı poll edecek biri olmazsa hiçbir iş ilerlemez; executor o
// "biri"dir. Uyuyarak beklediği için işlemciyi boşa çalıştırmaz.
//   Mini örnek:
//     let f = GecikmeliDeger { kalan: 3 };   // yalnız oluşturuldu: henüz hiçbir şey çalışmadı
//     let s = izle_block_on(f);              // ✓ şimdi poll edilir: 3 kez Pending, sonra Ready
//     println!("{s}");                       // İŞ BİTTİ
//     // izle_block_on(f);                   // ✗ derlenmez: f yukarıda fonksiyona taşındı
struct ThreadWaker(Thread);
impl Wake for ThreadWaker {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}
fn izle_block_on<F: Future>(fut: F) -> F::Output
where
    F::Output: std::fmt::Debug,
{
    let mut fut = std::pin::pin!(fut);
    let waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
    let mut cx = Context::from_waker(&waker);
    let mut tur = 0;
    loop {
        tur += 1;
        println!("  [executor] tur {tur}: poll ediyorum");
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(v) => {
                println!("  [executor] Ready geldi → döngüden çık");
                return v;
            }
            Poll::Pending => {
                println!("  [executor] Pending → uyu (park), wake bekliyorum");
                thread::park(); // wake unpark ettiği için hemen döner
            }
        }
    }
}

// ============================================================================
// 2) WAKER GERÇEKTE NEDİR?
// ============================================================================
// Waker, "bu görevi yeniden poll et" düğmesidir. İçten bakınca bir TRAIT OBJECT gibidir:
// (veri-pointer, &'static RawWakerVTable). Vtable DÖRT fonksiyon pointer'ı tutar (std,
// core/src/task/wake.rs:113): clone, wake, wake_by_ref, drop. Yani Waker tür-silinmiş (type-
// erased) bir "uyandırma kapasitesi"dir — executor ister thread park/unpark'la, ister bir görev
// kuyruğuna itmeyle gerçekleştirsin, future bilmez; sadece waker.wake() der.
//
// Ham RawWaker/RawWakerVTable yazmak unsafe'tir; std::task::Wake trait'i (Arc<impl Wake> →
// Waker) bunu güvenli sarar — biz de onu kullandık. Waker'ın iki garantisi: Clone (çoğaltılıp
// farklı yerlere dağıtılabilir) ve Send+Sync (başka thread'den uyandırılabilir).
// SADE ANLATIM: Kalabalık bir kafede sipariş verince eline titreyen bir çağrı cihazı verirler.
// Mutfağın içini bilmezsin; bilmen gereken tek şey şudur: cihaz titrerse siparişin hazırdır.
// Waker işte bu cihazdır. Future onu "iş hazır olunca çal" diye saklar; çalınca executor
// future'a yeniden bakar. Cihazın kopyası çıkarılabilir (`clone`) ve başka bir odadan, yani
// başka bir thread'den de çalınabilir. Bu neden var? Executor'ın "oldu mu?" diye durmadan
// sorması yerine, hazır olan işin kendisi haber versin diye.
//   Mini örnek:
//     let w: Waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
//     let w2 = w.clone();      // ✓ ikinci cihaz: aynı görevi uyandırır
//     w.wake_by_ref();         // ✓ zili çal, cihaz elde kalsın
//     w2.wake();               // ✓ zili çal ve cihazı geri ver (w2 tüketildi)
//     // w2.wake_by_ref();     // ✗ derlenmez: w2 az önce wake() ile tüketildi
fn waker_nedir() {
    let w: Waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
    let w2 = w.clone(); // vtable.clone → ucuz çoğaltma
    println!("   Waker = (veri ptr, vtable{{clone,wake,wake_by_ref,drop}}); klonlanır: {}", w.will_wake(&w2));
    // wake_by_ref: vtable.wake_by_ref'i çağırır (kendi thread'imizi unpark eder — zararsız).
    w.wake_by_ref();
    println!("   wake_by_ref() çağrıldı (bu executor'ı 'yeniden poll et' diye dürter)");
}

// ============================================================================
// 3) PIN NEDEN VAR? — self-referential state machine sorunu (RfR Böl. 8)
// ============================================================================
// RfR'nin anlattığı sorun: async fn bir state machine'e derlenir ve bu makine, bir .await
// noktasında KENDİ yerel değişkenine REFERANS tutabilir (ör. bir alt-future, argümanına işaret
// eder). Makine bir alanı + o alana işaret eden bir pointer'ı birlikte saklıyorsa buna SELF-
// REFERENTIAL (kendine referanslı) denir. Böyle bir değer bellekte TAŞINIRSA (move), iç pointer
// eski adrese bakmaya devam eder → dangling. Normal Rust değerleri serbestçe taşınabildiği için
// bu tehlikelidir.
//
// Çözüm: Pin<P>. "Pinli" bir değer artık (güvenle) TAŞINAMAZ; adresi sabit kalır, iç referanslar
// geçerli kalır. Unpin ise "beni pinlemek bir şey ifade etmez, taşınsam da sorun yok" diyen
// işaret trait'idir (çoğu sıradan tür Unpin'dir; asıl self-referential generator'lar DEĞİLDİR).
// poll'un imzası bu yüzden `Pin<&mut Self>`: iç referans kurulduktan sonra taşınmayacağı garanti altında.
// SADE ANLATIM: Bir karavanın içine "anahtar şu adresteki karavanın çekmecesinde" yazan bir
// not astığını düşün. Karavanı başka bir yere çekersen not hâlâ eski adresi gösterir; oraya
// giden boş araziyi bulur. Bazı future'lar böyledir: içlerindeki bir ok, kendi içlerindeki
// bir alanı gösterir. `Pin`, karavanın tekerine takılan kilittir: "bu değer artık yerinden
// oynatılmayacak" der. Pin bir değerin kendisi değil, onu gösteren işaretçinin sarmalayıcısıdır
// (`Pin<Box<T>>`, `Pin<&mut T>`). Taşınınca bozulmayan sıradan türlerde (`Unpin`: sayılar,
// String) bu kilit hiçbir şeyi kısıtlamaz.
//   Mini örnek:
//     let mut a = Box::pin(5);                // i32 Unpin'dir: Pin burada bir şey kısıtlamaz
//     *a = 6;                                 // ✓ Unpin türde içerik serbestçe değişir
//     let mut k = KendineRef::yeni("selam");  // Pin<Box<KendineRef>>: adresi sabit
//     println!("{}", k.bana_uzerinden());     // ✓ iç ok hâlâ doğru yeri gösteriyor
//     // let r: &mut KendineRef = &mut *k;    // ✗ derlenmez: !Unpin değere güvenli &mut yok;
//     //                                      //   &mut verilseydi değer yerinden oynatılabilirdi
struct KendineRef {
    veri: String,
    bana: *const String, // kendi `veri` alanına işaret edecek (self-referential)
    _pin: PhantomPinned,  // bu alan tipi !Unpin yapar → Pin gerçekten "taşınamaz" demek olur
}
impl KendineRef {
    fn yeni(s: &str) -> Pin<Box<Self>> {
        // Önce heap'te oluştur ve PINLE (Box::pin). Adres artık sabit.
        let mut kutu = Box::pin(KendineRef {
            veri: String::from(s),
            bana: ptr::null(),
            _pin: PhantomPinned,
        });
        // Adres sabitken iç pointer'ı kur: `bana`, `veri`yi göstersin.
        let adres: *const String = &kutu.veri;
        unsafe {
            // !Unpin olduğu için &mut almak unsafe (get_unchecked_mut): "taşımayacağıma söz veriyorum".
            let mref: &mut KendineRef = Pin::get_unchecked_mut(kutu.as_mut());
            mref.bana = adres;
        }
        kutu
    }
    fn veri(&self) -> &str {
        &self.veri
    }
    fn bana_uzerinden(&self) -> &String {
        // bana, veri'yi gösterir; Pin sayesinde adres hâlâ geçerli (değer taşınmadı).
        unsafe { &*self.bana }
    }
}
fn pin_ve_self_referential() {
    let p = KendineRef::yeni("merhaba");
    println!("   veri()           = {}", p.veri());
    println!("   bana_uzerinden() = {}", p.bana_uzerinden()); // aynı veriyi iç pointer'dan okur
    println!("   iki yol da aynı adresi görüyor (Pin değeri yerinden oynatmadığı için geçerli)");
    // Eğer bu değeri `bana`yı kurduktan SONRA taşısaydık, `bana` eski adrese bakardı → dangling.
    // Pin tam da bu taşımayı (güvenli yoldan) ENGELLER; bu yüzden async state machine'ler güvenli.
}

// SADE ANLATIM (aşağıdaki bölüm için): Buraya kadar bir saatin nasıl kullanıldığını ve içindeki
// üç ana parçayı (poll, waker, Pin) gördün. Aşağıdaki bölüm saatin arka kapağını açar: bu
// parçalar Rust'ın standart kütüphanesinde hangi dosyada, hangi satırda duruyor ve derleyici
// `async fn`'i nasıl bir durum makinesine çeviriyor, onu anlatır. İlk okuyuşta atlayabilirsin;
// yukarıdaki üç bölümü anlamış olman yeter.
// ============================================================================
// DERLEYİCİ / std PERSPEKTİFİ — kaynak referansları
// ============================================================================
//
// • Future trait'i: core/src/future/future.rs:37; tek zorunlu metot
//     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>   (:113)
//   Poll enum: core/src/task/poll.rs:13 → Ready(T) | Pending.
//
// • async fn/async {} → STATE MACHINE (generator). RfR Böl. 8: makine, her yield/await noktasındaki
//   TÜM yerel durumu (awaitlediği alt-future'lar dahil) içinde tutar; bu yüzden async fn'ler görünmez
//   şekilde büyüyebilir (örneğin büyük bir dizi generator'da saklanır) ve büyük future'ları Box::pin ile
//   heap'e alıp taşımayı ucuzlatmak gerekebilir. Bu makine Future'ı uygular; onu executor poll eder.
//
// • Waker: core/src/task/wake.rs. Waker (:411) = RawWaker (:22) = (data: *const (), vtable:
//   &'static RawWakerVTable). Vtable (:113) dört fn pointer'ı: clone, wake, wake_by_ref, drop
//   (bir nevi elle yazılmış trait object). wake() (:443) vtable.wake'i çağırır. std::task::Wake
//   trait'i bu ham yapıyı Arc'tan güvenle üretmemizi sağlar.
//
// • Pin: future bir kez poll'lanınca iç referans kurulabildiğinden, sonrasında taşınmaması
//   GEREKİR. poll'un Pin<&mut Self> alması bu garantiyi tür düzeyinde dayatır. Unpin türler
//   (self-referential olmayanlar) için Pin bir kısıt getirmez; PhantomPinned bir tipi !Unpin
//   yapıp Pin'in anlamını "gerçekten taşınamaz"a çevirir.
//
// • NEDEN executor std'de yok (Böl. 17 tekrarı): std mekanizmayı (Future/poll/Waker/Pin) tanımlar
//   ama poll döngüsü + I/O entegrasyonu + zamanlayıcı tek bir "doğru" biçimde olmadığı için
//   tokio/async-std'a bırakılmıştır. Yukarıdaki izle_block_on, o döngünün en yalın çekirdeğidir.
