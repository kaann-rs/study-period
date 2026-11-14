// Sağlamlaştırma — İleri konu: UnsafeCell ve Send/Sync'in İÇ YÜZÜ
// Çalıştır: cargo run -p ileri-send-sync
//
// Kaynak: Rust for Rustaceans Böl. 4 (Auto-Traits) & Böl. 10 (UnsafeCell) + std (core::cell).
// Böl. 15 (RefCell) + Böl. 16 (Send/Sync) + ileri-future burada tek kökte birleşir:
//   - İÇ DEĞİŞKENLİĞİN tek gerçek temeli UnsafeCell'dir (Cell/RefCell/Mutex/atomik hepsi onun üstünde).
//   - Send/Sync birer AUTO-TRAIT'tir: derleyici tipin İÇERİĞİNE bakıp otomatik ekler/eklemez.
//
// Doğrulanmış (rustc ile): UnsafeCell !Sync (std'de açık negatif impl); RefCell !Sync →
// Arc<RefCell> thread'e geçmez; *const () içeren struct !Send. Ayrıntı: DERLEYİCİ PERSPEKTİFİ.

use std::cell::UnsafeCell;

fn main() {
    unsafecell_temel();
    auto_trait_testi();
}

// ============================================================================
// 1. UnsafeCell — iç değişkenliğin TEK yasal temeli
// ============================================================================
// Normalde `&T` ÜZERİNDEN veriyi değiştiremezsin; dahası derleyici "& ardındaki veri değişmez"
// varsayımıyla OPTİMİZASYON yapar (no-alias). UnsafeCell, derleyicinin TANIDIĞI tek istisnadır:
// içindeki veriyi bu garantiden muaf tutar. get(&self) -> *mut T verir; yani PAYLAŞIMLI bir
// referanstan yazılabilir ham pointer. Cell, RefCell, Mutex, atomikler... hepsi bunun üstüne kurulu.
//
// Aşağıda Cell'in minik bir kopyasını UnsafeCell ile kuruyoruz: &self ile (paylaşımlı referansla)
// değer değiştirmek. Tek-thread'de güvenli — çünkü UnsafeCell !Sync, paylaşımı en baştan engeller.
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
// 2. Send / Sync AUTO-TRAIT olarak — derleme-zamanı üyelik testi (RfR Listing 4-8)
// ============================================================================
// Send: değer başka bir thread'e GÜVENLE TAŞINABİLİR mi.
// Sync: &T başka thread'lerce GÜVENLE PAYLAŞILABİLİR mi — tanımı gereği:  T: Sync  ⇔  &T: Send.
// İkisi de AUTO-TRAIT: derleyici, tipin TÜM alanları Send/Sync ise otomatik ekler; değilse eklemez.
// Elle `impl Send` yazılmaz (gerekirse `unsafe impl`); tek bir !Send alan tüm tipi !Send yapar.

// Bu fonksiyonlar KOD ÇALIŞTIRMAZ; yalnız T'nin ilgili trait'i uygulamasını DERLEME ANINDA sınar.
// Çağrı derleniyorsa üyelik vardır (RfR'nin is_normal hilesi).
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

    // Neden !Sync iyi? Tek-thread iç değişkenliği güvenli; ama iki thread aynı anda yaz()'sa VERİ
    // YARIŞI olurdu. UnsafeCell'in !Sync olması (std'de açık negatif impl), MiniCell'i/RefCell'i
    // thread'ler ARASINDA paylaşmayı DERLEME ANINDA yasaklar → yarış en baştan imkânsız.
    //
    // KARŞI ÖRNEKLER (hepsi rustc ile doğrulandı, kod derlenmez diye yorumda):
    //   - RefCell<T>: !Sync → Arc<RefCell<T>> thread'e geçmez (E0277 "cannot be shared / Sync yok").
    //   - *const T / *mut T: hem !Send hem !Sync → bu alanı taşıyan struct da !Send olur.
    //   - Rc<T>: !Send + !Sync (sayacı atomik değil) → thread'e taşınamaz (Böl. 16).
    //   - Mutex<T>: İÇİNDE UnsafeCell olduğu hâlde Sync'TİR, çünkü std `unsafe impl Sync for Mutex`
    //     ile "erişimi kilit serileştirir, yarış olmaz" sözünü verir. Arc<Mutex<T>> bu yüzden çalışır.
}

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
//   (cell.rs ~2315). Auto-trait "her şey Sync olurdu" diyecekken bu negatif impl araya girer:
//   işte Cell/RefCell'in !Sync olmasının KÖK sebebi. Böylece iç-değişken bir değer, ek bir
//   eşzamanlama (Mutex gibi) olmadan thread'ler arası PAYLAŞILAMAZ — veri yarışı derleme hatası olur.
//
// • AUTO-TRAIT ve API (RfR Böl. 4): Send/Sync "gizli bir söz"dür; neredeyse her tipin parçasıdır.
//   Public tipin private bir alanı !Send olursa, tipin KENDİSİ sessizce !Send olur — bu bir KIRICI
//   değişikliktir (semver). Yakalamak için RfR'nin hilesi: `fn is_normal<T: Send + Sync + Unpin>(){}`
//   + bir #[test] içinde `is_normal::<Tipim>();`. Test kod çalıştırmaz; yalnız DERLENİR — üyelik
//   bozulursa derleme kırılır, erkenden haberin olur. (Yukarıdaki send_ister/sync_ister aynı fikir.)
