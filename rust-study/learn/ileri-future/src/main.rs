// Sağlamlaştırma — İleri konu: FUTURE GERÇEKTE NEDİR, NASIL ÇALIŞIR?
// Çalıştır: cargo run -p ileri-future
//
// Kaynaklar: std (core::future, core::task) + Rust for Rustaceans (Jon Gjengset), Böl. 9.
// Böl. 17'de async'i KULLANDIK; burada MEKANİZMAYI açıyoruz: poll nedir, wake ne yapar,
// Waker gerçekte nedir, Pin neden var. Her şey std ile, bağımlılıksız; döngüyü GÖZLE göreceğiz.
//
// ÖZET ZİHİNSEL MODEL (gerisi bunun ayrıntısı):
//   Future = "henüz bitmemiş bir hesap". Onu bir EXECUTOR poll eder. poll ya Ready(değer) ya
//   Pending döner. Pending dönerken future, "ilerleyebilir hale gelince beni uyandır" diye bir
//   WAKER saklar. İş hazır olunca waker.wake() çağrılır → executor future'ı YENİDEN poll eder.
//   Boşa dönen bekleme (busy-wait) YOK: haber verilince devam. async fn, derleyici tarafından
//   her .await'te duraklayıp devam eden bir STATE MACHINE'e çevrilir; bu state machine = Future.

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

    println!("\n== 3) Pin ve self-referential (RfR Böl. 9) ==");
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
fn waker_nedir() {
    let w: Waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
    let w2 = w.clone(); // vtable.clone → ucuz çoğaltma
    println!("   Waker = (veri ptr, vtable{{clone,wake,wake_by_ref,drop}}); klonlanır: {}", w.will_wake(&w2));
    // wake_by_ref: vtable.wake_by_ref'i çağırır (kendi thread'imizi unpark eder — zararsız).
    w.wake_by_ref();
    println!("   wake_by_ref() çağrıldı (bu executor'ı 'yeniden poll et' diye dürter)");
}

// ============================================================================
// 3) PIN NEDEN VAR? — self-referential state machine sorunu (RfR Böl. 9)
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
// poll'un imzası bu yüzden `self: Pin<&mut Self>`: future bir kez poll'lanıp iç referans kurulduktan
// sonra taşınmayacağının GARANTİSİ altında ilerler.
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

// ============================================================================
// DERLEYİCİ / std PERSPEKTİFİ — kaynak referansları
// ============================================================================
//
// • Future trait'i: core/src/future/future.rs:37; tek zorunlu metot
//     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>   (:113)
//   Poll enum: core/src/task/poll.rs:13 → Ready(T) | Pending.
//
// • async fn/async {} → STATE MACHINE (generator). RfR Böl. 9: makine, her yield/await noktasındaki
//   TÜM yerel durumu (awaitlediği alt-future'lar dahil) içinde tutar; bu yüzden async fn'ler görünmez
//   şekilde büyüyebilir (içindeki [u8; 8192] generator'da saklanır) ve büyük future'ları Box::pin ile
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
