// Açıklama: learn/advanced-concepts/ileri-send-sync/ACIKLAMA.md
// Sağlamlaştırma — İleri konu: UnsafeCell ve Send/Sync'in İÇ YÜZÜ
// Çalıştır: cargo run -p ileri-send-sync
//
// Kaynak: Rust for Rustaceans Böl. 3 (Auto-Traits, Designing Interfaces) & Böl. 9 (UnsafeCell,
//   Unsafe Code) + std (core::cell). Bölüm listesi yayıncının içindekiler sayfasıyla doğrulandı.
// Müfredat Böl. 15 (RefCell) + Böl. 16 (Send/Sync) + ileri-future burada tek kökte birleşir:
//   - İÇ DEĞİŞKENLİĞİN tek gerçek temeli UnsafeCell'dir (Cell/RefCell/Mutex/atomik hepsi onun üstünde).
//   - Send/Sync birer AUTO-TRAIT'tir: derleyici tipin İÇERİĞİNE bakıp otomatik ekler/eklemez.
//
// Doğrulanmış (rustc ile): UnsafeCell !Sync (std'de açık negatif impl); RefCell !Sync →
// Arc<RefCell> thread'e geçmez; *const () içeren struct !Send. Ayrıntı: DERLEYİCİ PERSPEKTİFİ.

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Her bölümün üstündeki "SADE ANLATIM" kutusunu önce oku; teknik kısım zor gelirse sorun değil.
//
// Bu dosyayı okumadan önce bilmen gerekenler: `learn/ownership` (`&` ile bakılır, `&mut` ile
// değiştirilir), `learn/smart-pointers` (Rc, RefCell) ve `learn/concurrency` (thread, Arc, Mutex).
//
// Bu bölümün tek cümlelik özeti: Rust'ta "yalnız bakma" referansı üzerinden değer değiştirmenin
// tek yasal kapısı UnsafeCell'dir; bir türün thread'ler arasında taşınıp paylaşılabileceğine
// de derleyici, türün içindeki parçalara bakarak kendisi karar verir.
//
// MİNİ SÖZLÜK (bu dosyada geçen kelimeler):
//   thread (iş parçacığı) → programın içinde aynı anda çalışan ayrı bir işçi.
//   veri yarışı (data race) → iki thread'in aynı veriye aynı anda dokunması ve en az birinin
//                          yazması. Sonuç belirsizdir; Rust bunu derlemede engeller.
//   iç değişkenlik       → interior mutability: `&` üzerinden bile içeriği değiştirebilmek.
//   UnsafeCell           → iç değişkenliğin temel taşı; "bakarken değişebilir" etiketi.
//   Cell / RefCell       → UnsafeCell'in üstüne kurulu, tek thread'de güvenli kutular.
//   Mutex                → kilitli kutu: içeriğe aynı anda yalnız bir thread girer.
//   atomik               → yarıda kesilemeyen tek hamleyle okunup yazılan küçük değer.
//   Rc / Arc             → paylaşımlı sahiplik sayacı. Rc tek thread içindir, Arc çok thread.
//   Send                 → "bu değer başka bir thread'e verilebilir" işareti.
//   Sync                 → "bu değere birçok thread aynı anda `&` ile bakabilir" işareti.
//   auto-trait           → senin yazmadığın, derleyicinin türün alanlarına bakıp kendiliğinden
//                          verdiği trait.
//   negatif impl (!Sync) → "bu tür Sync DEĞİLDİR" diye açıkça yazılmış kayıt.
//   ham pointer (*mut T) → derleyicinin denetlemediği çıplak adres.
//   unsafe / unsafe impl → "bu kuralı derleyici değil, ben garanti ediyorum" sözü.
//   no-alias             → derleyicinin "`&` ardındaki veri değişmez" varsayımı.
//   optimizasyon         → derleyicinin kodu aynı işi daha hızlı yapacak hâle getirmesi.
//   Copy                 → atanınca fotokopisi çekilen küçük türler (sayılar gibi).
//   semver / kırıcı değişiklik → sürüm numarası kuralı / eski kodu derlenmez yapan değişiklik.
//   E0277                → "bu tür gereken trait'i sağlamıyor" hatasının numarası.

use std::cell::UnsafeCell;

fn main() {
    unsafecell_temel();
    auto_trait_testi();
}

// ============================================================================
// 1. UnsafeCell — iç değişkenliğin TEK yasal temeli
// ============================================================================
// Normalde `&T` ÜZERİNDEN veri değiştirilemez; derleyici "& ardındaki veri değişmez" varsayımıyla
// OPTİMİZASYON yapar (no-alias). UnsafeCell bu garantiden muaf tutulan TEK istisnadır:
// get(&self) -> *mut T verir; yani paylaşımlı referanstan yazılabilir ham pointer.
//
// Aşağıda Cell'in minik bir kopyasını UnsafeCell ile kuruyoruz: &self ile (paylaşımlı referansla)
// değer değiştirmek. Tek-thread'de güvenli — çünkü UnsafeCell !Sync, paylaşımı en baştan engeller.
// SADE ANLATIM: Müzedeki eşyaların önünde "dokunmayın, yalnız bakın" yazar; `&` referansı
// böyledir. Derleyici de buna güvenir: "kimse dokunamadığına göre bir kez bakıp aklımda
// tutabilirim" der. `UnsafeCell` ise üstünde "DİKKAT: sen bakarken bu değişebilir" yazan özel
// bir etikettir. Derleyici bu etiketi görünce ezberine güvenmez ve `&` üzerinden yazmaya izin
// veren kapıyı açar. Kapıdan doğru geçmek sana kalır; bu yüzden içi `unsafe`'tir. Bu neden
// var? Sayaç, önbellek, kilit gibi şeyler "yalnız bakan" birçok yerden güncellenmek zorundadır.
//   Mini örnek:
//     let c = MiniCell::yeni(5);
//     let r1 = &c;
//     let r2 = &c;                 // iki "yalnız bakan" referans
//     r1.yaz(10);                  // ✓ & üzerinden değişti: UnsafeCell izin verdi
//     println!("{}", r2.al());     // 10 → öteki referans da yeni değeri görür
//     let x = 5;
//     let r = &x;
//     // *r = 10;                  // ✗ derlenmez: sıradan & üzerinden değer değiştirilemez
struct MiniCell<T> {
    v: UnsafeCell<T>,
}
impl<T: Copy> MiniCell<T> {
    fn yeni(v: T) -> MiniCell<T> {
        MiniCell { v: UnsafeCell::new(v) }
    }
    fn al(&self) -> T {
        // get() paylaşımlı &self'ten *mut T döndürür; deref etmek unsafe (çakışma garantisi BİZDE).
        unsafe { *self.v.get() }
    }
    fn yaz(&self, yeni: T) {
        // &self (paylaşımlı!) ile mutasyon — UnsafeCell olmadan bu derlenmez bile.
        unsafe { *self.v.get() = yeni }
    }
}
fn unsafecell_temel() {
    let c = MiniCell::yeni(5);
    let r1 = &c;
    let r2 = &c; // iki paylaşımlı referans
    r1.yaz(10); // paylaşımlı referanstan YAZ (iç değişkenlik)
    println!("MiniCell: r1.yaz(10) sonrası r2.al() = {}", r2.al()); // 10
    // Bu, &mut olmadan mutasyon: aliasing XOR mutability'nin "güvenli kaçış kapısı". Güvenli çünkü
    // tek-thread (MiniCell !Sync: aşağıda) ve mantığı biz garantiliyoruz. RefCell aynı şeyi yapar
    // ama ihlali çalışma anında yakalar (borrow sayacı); Cell ise sadece Copy değerlerle değiştirir.
}

// ============================================================================
// 2. Send / Sync AUTO-TRAIT olarak — derleme-zamanı üyelik testi
// ============================================================================
// Send: değer başka bir thread'e GÜVENLE TAŞINABİLİR mi.
// Sync: &T başka thread'lerce GÜVENLE PAYLAŞILABİLİR mi — tanımı gereği:  T: Sync  ⇔  &T: Send.
// İkisi de AUTO-TRAIT: derleyici, tipin TÜM alanları Send/Sync ise otomatik ekler; değilse eklemez.
// Elle `impl Send` yazılmaz (gerekirse `unsafe impl`); tek bir !Send alan tüm tipi !Send yapar.

// Bu fonksiyonlar KOD ÇALIŞTIRMAZ; yalnız T'nin ilgili trait'i uygulamasını DERLEME ANINDA sınar.
// Çağrı derleniyorsa üyelik vardır (RfR'nin is_normal hilesi).
// SADE ANLATIM: Evde iki oda, her odada bir çocuk var; odalar thread'lerdir. `Send`, "bu
// oyuncağı öteki odaya götürebilirsin" demektir. `Sync`, "bu oyuncağa iki odadan aynı anda
// bakılabilir" demektir. İkisi birbirine bağlıdır: bir şeye iki odadan bakılabiliyorsa, onu
// gösteren ok (`&T`) öteki odaya götürülebilir; `T: Sync` ile `&T: Send` aynı sözdür. Bu
// etiketleri sen yapıştırmazsın: derleyici türün içindeki her parçaya bakar, tek bir parça
// uygun değilse bütün tür uygun sayılmaz. Bu neden var? İki thread'in aynı veriyi aynı anda
// bozması (veri yarışı) daha derleme sırasında yakalansın diye.
//   Mini örnek:
//     send_ister::<i32>();                    // ✓ sayı başka thread'e verilebilir
//     send_ister::<Cell<i32>>();              // ✓ Cell taşınabilir...
//     // sync_ister::<Cell<i32>>();           // ✗ ...ama iki thread aynı anda bakamaz
//     sync_ister::<Mutex<Cell<i32>>>();       // ✓ kilit içine koyunca paylaşılabilir oldu
//     // send_ister::<Rc<i32>>();             // ✗ derlenmez: Rc'nin sayacı thread'e uygun değil
//     // sync_ister::<RwLock<Cell<i32>>>();   // ✗ RwLock çok okuyucu alır: içerik de Sync olmalı
fn send_ister<T: Send>() {}
fn sync_ister<T: Sync>() {}

fn auto_trait_testi() {
    send_ister::<i32>();
    sync_ister::<i32>();
    println!("i32: Send + Sync ✓ (sıradan değer)");

    // MiniCell<i32> Send'DİR (UnsafeCell<T>, T: Send ise Send'dir) ama Sync DEĞİLDİR:
    send_ister::<MiniCell<i32>>();
    println!("MiniCell<i32>: Send ✓  (ama Sync değil — derleme-testi yorumda)");
    // sync_ister::<MiniCell<i32>>();  // ✗ DERLENMEZ: UnsafeCell !Sync → MiniCell !Sync
    //   "the trait `Sync` is not implemented for `UnsafeCell<i32>`"

    // Neden !Sync iyi? Tek-thread iç değişkenliği güvenli, ama iki thread yaz()'sa VERİ YARIŞI olurdu;
    // UnsafeCell'in !Sync olması (std'de açık negatif impl) paylaşımı DERLEME ANINDA yasaklar.
    //
    // KARŞI ÖRNEKLER (hepsi rustc ile doğrulandı, kod derlenmez diye yorumda):
    //   - RefCell<T>: !Sync → Arc<RefCell<T>> thread'e geçmez (E0277 "cannot be shared / Sync yok").
    //   - *const T / *mut T: hem !Send hem !Sync → bu alanı taşıyan struct da !Send olur.
    //   - Rc<T>: !Send + !Sync (sayacı atomik değil) → thread'e taşınamaz (Böl. 16).
    //   - Mutex<T>: İÇİNDE UnsafeCell olduğu hâlde Sync'TİR, çünkü std `unsafe impl Sync for Mutex`
    //     ile "erişimi kilit serileştirir, yarış olmaz" sözünü verir. Arc<Mutex<T>> bu yüzden çalışır.
}

// SADE ANLATIM (aşağıdaki bölüm için): Yukarıda iki şey gördün: "bakarken değişebilir"
// etiketi (UnsafeCell) ve derleyicinin kendiliğinden yapıştırdığı iki etiket (Send, Sync).
// Aşağıdaki bölüm bunların standart kütüphanede nerede ve nasıl yazıldığını gösterir, bir de
// kütüphane yazanlar için bir uyarı ekler: türüne eklediğin tek bir alan, o türü sessizce
// "thread'e verilemez" yapabilir; bunu bir testle erkenden yakalayabilirsin. İlk okuyuşta
// atlayabilirsin; yukarıdaki iki bölümü anlamış olman yeter.
// ============================================================================
// DERLEYİCİ / std / RfR PERSPEKTİFİ
// ============================================================================
//
// • UnsafeCell<T> (std core/src/cell.rs:2313): iç değişkenliğin TEK temel yapı taşı. get(&self)
//   -> *mut T verir; paylaşımlı &self'ten yazılabilir pointer elde etmenin TEK yasal yolu budur.
//   Derleyici "& ardındaki veri değişmez" no-alias varsayımını UnsafeCell içeriği için KALDIRIR;
//   başka hiçbir tür bunu yapamaz. Cell, RefCell, Mutex, RwLock, atomikler — hepsi UnsafeCell sarar
//   ve üstüne kendi güvenlik disiplinini (Copy-değişim / borrow sayacı / kilit) ekler. UnsafeCell
//   ayrıca T'de INVARIANT'tır (#[rustc_assert_variance(invariant)]) — mutasyon içerdiği için
//   (ileri-variance ile bağ).
//
// • Send: değeri başka thread'e taşımak güvenli mi. Sync: T: Sync ⇔ &T: Send (paylaşmak güvenli mi).
//   İkisi de AUTO-TRAIT (unsafe auto trait, core::marker): derleyici tipin alanlarına bakıp, hepsi
//   uygunsa OTOMATİK uygular; tek bir uygunsuz alan tüm tipi dışarıda bırakır. impl Trait / closure
//   gibi tür-silinmiş türlere bile PROPAGE olur.
//
// • UnsafeCell'in std'de AÇIK NEGATİF impl'i var: `impl<T: ?Sized> !Sync for UnsafeCell<T> {}`
//   (cell.rs:2318). Auto-trait "her şey Sync olurdu" diyecekken bu negatif impl araya girer:
//   işte Cell/RefCell'in !Sync olmasının KÖK sebebi. Böylece iç-değişken bir değer, ek bir
//   eşzamanlama (Mutex gibi) olmadan thread'ler arası PAYLAŞILAMAZ — veri yarışı derleme hatası olur.
//
// • AUTO-TRAIT ve API (RfR Böl. 3): Send/Sync "gizli bir söz"dür; neredeyse her tipin parçasıdır.
//   Public tipin private bir alanı !Send olursa, tipin KENDİSİ sessizce !Send olur — bu bir KIRICI
//   değişikliktir (semver). Yakalamak için RfR'nin hilesi: `fn is_normal<T: Send + Sync + Unpin>(){}`
//   + bir #[test] içinde `is_normal::<Tipim>();`. Test kod çalıştırmaz; yalnız DERLENİR — üyelik
//   bozulursa derleme kırılır, erkenden haberin olur. (Yukarıdaki send_ister/sync_ister aynı fikir.)
