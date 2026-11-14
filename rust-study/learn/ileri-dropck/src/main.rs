// Sağlamlaştırma — İleri konu: DROP CHECK (dropck), PhantomData, #[may_dangle]
// Çalıştır: cargo run -p ileri-dropck
//
// Kaynak: Rust for Rustaceans Böl. 10 ("The Drop Check") + std. ileri-variance + Böl. 15 (Drop)
// burada birleşir. Temel soru: "bir değer DÜŞERKEN, ödünç aldığı veriye hâlâ erişebilir mi?"
//
// Doğrulanmış (rustc ile): lifetime'lı bir tip Drop uygularsa, ödünç aldığı veri o tipi AŞMALIDIR
// (yoksa E0597 "dropped here while still borrowed ... when `x` is dropped and runs Drop code");
// Drop YOKSA NLL borrow'u erken bitirir ve aynı kod derlenir. Ayrıntı: DERLEYİCİ PERSPEKTİFİ.

use std::marker::PhantomData;
use std::mem::size_of;

fn main() {
    dropck_temel();
    phantomdata_zst();
}

// ============================================================================
// 1. DROP CHECK — Drop impl'i, lifetime kısıtını SIKILAŞTIRIR
// ============================================================================
struct Denetci<'a>(&'a u8);
impl<'a> Drop for Denetci<'a> {
    fn drop(&mut self) {
        // drop ÖDÜNÇ ALINAN veriye erişiyor → bu referans drop anında GEÇERLİ olmalı.
        println!("  Denetci düşerken okuyor: {}", self.0);
    }
}

fn dropck_temel() {
    // ÇALIŞAN sıra: `gun` ÖNCE bildirildi → EN SON düşer. `denetci` sonra bildirildi → ÖNCE düşer.
    // denetci.drop() çalıştığında gun hâlâ yaşıyor → OK.
    let gun = 5u8;
    let denetci = Denetci(&gun);
    println!("dropck: değer = {}", denetci.0);
    // Kapsam sonunda: denetci düşer (gun'u okur, gun yaşıyor), sonra gun düşer.

    // ✗ TERS sıra DERLENMEZ (rustc ile doğrulandı):
    //     let denetci;            // önce bildir → EN SON düşer
    //     let gun = 5u8;          // sonra → ÖNCE düşer
    //     denetci = Denetci(&gun);
    //   → E0597: "`gun` does not live long enough / dropped here while still borrowed /
    //     borrow might be used here, when `denetci` is dropped and runs the Drop code".
    //   Çünkü gun önce düşer, ama denetci.drop() ondan SONRA çalışıp &gun'u okumaya kalkar → dangling.
    //
    // KRİTİK FARK: Denetci'nin Drop impl'i OLMASAYDI, aynı ters-sıra kod DERLENİRDİ. Çünkü Drop yoksa
    // derleyici "düşerken referansa dokunulmayacak" bilir → NLL borrow'u son kullanımda bitirir.
    // Yani lifetime + Drop birleşimi şunu zorunlu kılar: ödünç ALINAN veri (gun), ödünç ALAN
    // değerden (denetci) DAHA UZUN yaşamalı — çünkü denetci'nin &gun'a SON erişimi drop() içinde,
    // yani denetci ölürken olur; borrow o ana kadar uzamak zorundadır, NLL onu erken bitiremez.
}

// ============================================================================
// 2. PhantomData<T> — sıfır boyutlu, ama tipe "sanki T içerir/sahiptir" dedirten işaretçi
// ============================================================================
// PhantomData<T>: hiç yer kaplamayan (ZST) bir alan. Hiçbir veri TUTMAZ ama derleyiciye tipin
// T açısından nasıl davranacağını söyler: VARIANCE (ileri-variance), DROP CHECK ve AUTO-TRAIT
// (Send/Sync, ileri-send-sync) hesaplarında "sanki bir T varmış gibi" davranılır.
struct Belirtec<T> {
    id: u32,
    _sahip: PhantomData<T>, // "mantıken bir T'ye sahibim" — ama bellekte T YOK
}
fn phantomdata_zst() {
    let b: Belirtec<String> = Belirtec {
        id: 7,
        _sahip: PhantomData,
    };
    println!("PhantomData ZST: Belirtec.id = {}", b.id);
    // PhantomData yer kaplamaz: Belirtec<String> boyutu = sadece u32 kadar.
    println!(
        "  size_of::<Belirtec<String>>() = {} (== u32 = {})",
        size_of::<Belirtec<String>>(),
        size_of::<u32>()
    );
    // Neden var? Üç yaygın kullanım (hangi PhantomData → hangi anlam):
    //   PhantomData<T>          → "T'ye SAHİBİM": dropck T::drop'u çağırdığımı varsayar (owned ptr fix).
    //   PhantomData<&'a T>      → "T'yi 'a boyunca ÖDÜNÇ aldım": 'a covariant, dropck T'yi okumaz.
    //   PhantomData<*const T>   → !Send + !Sync işaretçisi (ham pointer gibi davran).
    //   PhantomData<fn() -> T>  → T'de covariant AMA sahiplenmez (dropck tetiklenmez).
    // Bir tip ham pointer (*mut T) tutup mantıken T'ye sahipse (kendi Box/Vec'ini yazıyorsan),
    // *mut T dropck'e "T'm yok" dedirtir; PhantomData<T> eklemek bu yanlışı düzeltir (RfR Böl. 10).
}

// ============================================================================
// DERLEYİCİ / RfR PERSPEKTİFİ
// ============================================================================
//
// • DROP CHECK (dropck): borrow checker'ın bir parçası. Soru: bir değer düşerken, lifetime'ı
//   üzerinden ödünç aldığı veriye erişebilir mi? Erişebiliyorsa (yani tip Drop uyguluyorsa ve o
//   lifetime'ı Drop'ta kullanabiliyorsa), ödünç alınan veri o değeri AŞMALIDIR. RfR'nin örneği:
//     let mut x = true; let foo = Foo(&mut x); x = false;
//   Bu kod, Foo'nun Drop'u OLUP OLMAMASINA göre derlenir/derlenmez. Drop yoksa foo'nun &mut'una son
//   kullanımdan sonra dokunulmaz → x değiştirilebilir (NLL). Drop varsa foo::drop &mut x'e
//   erişebilir → x hâlâ ödünçte sayılır, mutasyon yasak. (Bizim Denetci örneği bunun paylaşımlı hâli.)
//
// • Neden Drop fark yaratır: Drop YOKSA, derleyici "düşerken bu referansa dokunulmaz" bilir ve
//   borrow'u son kullanımda sonlandırır (NLL). Drop VARSA, drop() gövdesi referansı kullanabilir;
//   bu yüzden borrow, değerin düşmesine (drop çağrısına) kadar UZATILIR. Sorun: izin verilse,
//   drop() çoktan serbest bırakılmış/değişmiş veriyi okur/yazar → use-after-free.
//
// • PhantomData<T>: sıfır boyutlu işaret alanı. Bir tip, bir GENERIC parametreyi doğrudan ALAN
//   olarak tutmuyorsa (ör. ham pointer *mut T ardında ya da hiç) ama variance/dropck/auto-trait
//   açısından "sanki T içeriyormuş" gibi davranmalıysa PhantomData eklenir. En kritik kullanım
//   (RfR): kendi Box<T>/Vec<T>'ini yazarken içeride *mut T tutarsın; dropck *mut T'yi görüp "T'ye
//   sahip değil, T::drop'u çağırmaz" sanır. Bu YANLIŞ (sen T::drop'u elle çağırırsın). PhantomData<T>
//   eklemek dropck'e gerçeği söyler: "T'ye sahibim, düşerken T::drop çalışır" → lifetime'lar doğru
//   kısıtlanır.
//
// • #[may_dangle] (unstable `dropck_eyepatch`): TERS yön. Kendi Box<&mut T> veya Box<T>'ini yazıyorsan
//   ve Drop'un T'ye SADECE düşürmek dışında DOKUNMUYORSA, dropck'in aşırı-katı kısıtını gevşetmek
//   için Drop impl'inde generic'i #[may_dangle] ile işaretlersin:
//       unsafe impl<#[may_dangle] T> Drop for KendiBox<T> { ... }
//   Bu "söz veriyorum, bu T'ye düşürmek dışında erişmeyeceğim" der → ödünç, değerden önce bitebilir.
//   `unsafe`tir (yanlış söz = use-after-free) ve nightly'dir; bu yüzden burada yalnız anlatıldı,
//   derlenmedi. std'de Vec/Box tam da bunu + PhantomData<T>'yi birlikte kullanır.
//
// • BAĞLANTILAR: dropck + PhantomData, lifetime (ileri-variance: PhantomData variance'ı da seçer)
//   ve Send/Sync (ileri-send-sync: PhantomData<*const T> !Send yapar) ile iç içedir. PhantomData,
//   "derleyiciye tipin anlamını anlatan" tek bir ZST araç; hangi Phantom<...> yazdığın anlamı belirler.
