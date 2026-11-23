// Açıklama: learn/advanced-concepts/ileri-dropck/ACIKLAMA.md
// Sağlamlaştırma — İleri konu: DROP CHECK (dropck), PhantomData, #[may_dangle]
// Çalıştır: cargo run -p ileri-dropck
//
// Kaynak: Rust for Rustaceans Böl. 9 ("The Drop Check", Unsafe Code) + std. ileri-variance + Böl. 15
// (Drop, kurs bölümü) burada birleşir. Yayıncının ayrıntılı içindekiler listesiyle doğrulandı.
// Temel soru: "bir değer DÜŞERKEN, ödünç aldığı veriye hâlâ erişebilir mi?"
//
// Doğrulanmış (rustc ile): lifetime'lı bir tip Drop uygularsa, ödünç aldığı veri o tipi AŞMALIDIR
// (yoksa E0597 "dropped here while still borrowed ... when `x` is dropped and runs Drop code");
// Drop YOKSA NLL borrow'u erken bitirir ve aynı kod derlenir. Ayrıntı: DERLEYİCİ PERSPEKTİFİ.

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Her bölümün üstündeki "SADE ANLATIM" kutusunu önce oku; teknik kısım zor gelirse sorun değil.
//
// Bu dosyayı okumadan önce bilmen gerekenler: `learn/ownership` (kapsam, drop, ödünç alma),
// `learn/generics-traits-lifetimes` (lifetime), `learn/smart-pointers` (Drop trait'i) ve bu
// klasördeki `ileri-variance` ile `ileri-send-sync`.
//
// Bu bölümün tek cümlelik özeti: bir değer silinirken ödünç aldığı veriye son bir kez
// bakabiliyorsa (yani Drop yazılmışsa), o veri bu değerden önce silinemez; derleyici bunu
// denetler.
//
// MİNİ SÖZLÜK (bu dosyada geçen kelimeler):
//   drop (düşürme)       → bir değerin silinmesi, kapladığı yerin geri verilmesi.
//   Drop trait'i         → "silinirken şunu da yap" diye yazdığın son iş (`fn drop`).
//   kapsam (scope)       → bir değişkenin yaşadığı süslü parantez { ... } arası.
//   düşme sırası         → kapsam bitince değişkenler tanımlandıkları sıranın TERSİYLE silinir.
//   borrow (ödünç)       → sahibi değiştirmeden veriye referansla ulaşmak.
//   lifetime ('a)        → bir referansın geçerli olduğu kod aralığının adı.
//   NLL                  → non-lexical lifetimes: ödünç, parantezin sonunda değil SON
//                          kullanıldığı satırda biter.
//   drop check (dropck)  → "bu değer silinirken baktığı veri hâlâ duruyor mu?" denetimi.
//   dangling (sarkan)    → gösterdiği veri silinmiş referans.
//   use-after-free       → silinmiş belleği kullanmak. Programı bozar; Rust engeller.
//   generic parametre    → türün içindeki boşluk: `Belirtec<T>`'deki T.
//   PhantomData<T>       → yer kaplamayan sahte alan: "T ile ilgim var" diye not düşer.
//   ZST                  → zero-sized type: bellekte sıfır bayt kaplayan tür.
//   ham pointer (*mut T) → derleyicinin denetlemediği çıplak adres.
//   variance             → bir türün içindeki ömrün ne kadar esneyebildiği (ileri-variance).
//   auto-trait           → derleyicinin kendiliğinden verdiği trait (Send, Sync).
//   #[may_dangle]        → "silinirken bu T'ye bakmayacağım" diye verilen unsafe söz.
//   nightly / unstable   → Rust'ın henüz kararlı sürüme girmemiş deneme özellikleri.
//   E0597                → "ödünç alınan değer yeterince uzun yaşamıyor" hatasının numarası.

use std::marker::PhantomData;
use std::mem::size_of;

fn main() {
    dropck_temel();
    phantomdata_zst();
}

// ============================================================================
// 1. DROP CHECK — Drop impl'i, lifetime kısıtını SIKILAŞTIRIR
// ============================================================================
// SADE ANLATIM: Arkadaşının defterini ödünç aldın ve "odadan çıkarken son bir kez bakacağım"
// dedin. O zaman defter, sen çıkana kadar yerinde durmalı. Arkadaşın defteri senden önce
// toplayıp götürürse çıkarken boş masaya bakarsın. `Drop` yazmak, o "çıkarken son bir kez
// bakacağım" sözüdür. Derleyici bu sözü görünce şunu denetler: ödünç verilen şey, ödünç
// alandan DAHA SONRA silinecek mi? Rust'ta değişkenler tanımlandıkları sıranın tersiyle
// silinir; son gelen ilk çıkar. Drop yoksa "çıkarken bakma" da yoktur; ödünç son kullanımda
// biter ve sıra önemini yitirir.
//   Mini örnek:
//     let gun = 5u8;               // önce tanımlandı → en son silinir
//     let d = Denetci(&gun);       // sonra tanımlandı → önce silinir
//     // kapsam sonu: d çıkarken gun'a bakar, gun hâlâ duruyor  ✓
//
//     let d2;                      // önce tanımlandı → en son silinir
//     let gun2 = 5u8;              // sonra tanımlandı → önce silinir
//     // d2 = Denetci(&gun2);      // ✗ derlenmez: d2 çıkarken gun2 çoktan silinmiş olurdu
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
    // KRİTİK FARK: Denetci'nin Drop impl'i OLMASAYDI, aynı ters-sıra kod DERLENİRDİ. Drop yoksa derleyici
    // "düşerken referansa dokunulmayacak" bilir → NLL borrow'u son kullanımda bitirir. Drop varsa
    // denetci'nin &gun'a SON erişimi drop() içinde olduğundan borrow o ana kadar uzamak zorundadır:
    // ödünç ALINAN veri (gun), ödünç ALAN değerden (denetci) DAHA UZUN yaşamalı.
}

// ============================================================================
// 2. PhantomData<T> — sıfır boyutlu, ama tipe "sanki T içerir/sahiptir" dedirten işaretçi
// ============================================================================
// PhantomData<T>: hiç yer kaplamayan (ZST) bir alan. Hiçbir veri TUTMAZ ama derleyiciye tipin
// T açısından nasıl davranacağını söyler: VARIANCE (ileri-variance), DROP CHECK ve AUTO-TRAIT
// (Send/Sync, ileri-send-sync) hesaplarında "sanki bir T varmış gibi" davranılır.
// SADE ANLATIM: Boş bir odanın kapısına "Zeynep'in odası" yazan bir levha astığını düşün.
// Odada kimse yok, levha da yer kaplamıyor; ama binadaki görevliler o odaya Zeynep'inmiş gibi
// davranır. `PhantomData<T>` bu levhadır: bellekte T yoktur, ama derleyici türünü incelerken
// "burada bir T varmış" diye hesap yapar. Bu neden var? Rust, kullanılmayan bir generic
// parametreye izin vermez; ayrıca ham pointer tutan türlerde derleyiciye "bu T ile ilişkim
// şudur" demenin yolu budur (ömür esnekliği ve Send/Sync hesabı buna göre yapılır).
//   Mini örnek:
//     struct Bos<T> { id: u32 }                    // ✗ derlenmez: T hiç kullanılmıyor
//     struct Belirtec<T> { id: u32, _sahip: PhantomData<T> }   // ✓ levha asıldı
//     let b: Belirtec<String> = Belirtec { id: 7, _sahip: PhantomData };
//     size_of::<Belirtec<String>>();               // 4 → yalnız u32; String için yer yok
//     struct Ham { p: PhantomData<*const u8> }     // ✓ derlenir, ama ham pointer taşıyormuş
//     //                                           //   gibi sayılır: artık Send değildir
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
    // Güncel kural: Drop impl'i olan bir tipin TÜM generic parametreleri zaten canlı sayılır
    // (RFC 1238); bu yüzden *mut T tutan bir tipte PhantomData<T> eklemek dropck açısından gereksizdir.
    // E0597 için T'nin gerçekten bir ödünç taşıması gerekir:
    //     struct K<T>(T); impl<T> Drop for K<T> { fn drop(&mut self) {} }
    //     let k; let x = 5u8; k = K(&x); // k, x'ten sonra düşer → E0597
    // PhantomData<T>'nin bağımsız dropck etkisi yalnızca Drop impl'i olmayan (ama drop glue'ı olan)
    // tiplerde ve `#[may_dangle]` ile görünür (std'nin Vec/Box'ı ikisini bu yüzden birlikte kullanır).
    // Variance ve auto-trait (Send/Sync) etkisi için yukarıda yazılanlar geçerlidir.
}

// SADE ANLATIM (aşağıdaki bölüm için): Yukarıda iki şey gördün: Drop yazınca derleyici
// "çıkarken bakacağın şey hâlâ duruyor mu?" diye sorar, PhantomData ise yer kaplamayan bir
// nottur. Aşağıdaki bölüm bunların derleyicideki tam kurallarını anlatır ve kendi Box'ını ya
// da Vec'ini yazanların kullandığı ince bir ayarı ekler: `#[may_dangle]`, "silinirken bu T'ye
// dokunmayacağım, bana güven" diye verilen sözdür. Günlük kod yazarken buna ihtiyacın olmaz.
// İlk okuyuşta atlayabilirsin; yukarıdaki iki bölümü anlamış olman yeter.
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
// • Neden Drop fark yaratır: Drop YOKSA borrow son kullanımda biter (NLL); Drop VARSA drop() gövdesi
//   referansı kullanabilir → borrow, değerin düşmesine (drop çağrısına) kadar UZATILIR. İzin verilse
//   drop() serbest bırakılmış/değişmiş veriyi okur/yazar → use-after-free.
//
// • PhantomData<T>: sıfır boyutlu işaret alanı. Bir tip, bir GENERIC parametreyi doğrudan ALAN
//   olarak tutmuyorsa (ör. ham pointer *mut T ardında ya da hiç) ama variance/auto-trait açısından
//   "sanki T içeriyormuş" gibi davranmalıysa PhantomData eklenir. dropck açısından belirleyici olan
//   Drop impl'inin varlığıdır: Drop yazılmış tipte T zaten canlı sayılır (RFC 1238). PhantomData<T>'nin
//   bağımsız dropck etkisi yalnızca Drop impl'i olmayan (ama drop glue'ı olan) tiplerde ve
//   `#[may_dangle]` ile görünür.
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
