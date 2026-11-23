// Rust Book — Bölüm 4: Ownership (Sahiplik), Borrowing (Ödünç alma), Slices (Dilimler)
// Çalıştır: cargo run -p ownership
//
// Bu crate, common-concepts'teki KÖK KURALIN — "Aliasing XOR Mutability" — somut
// olarak yaşadığı yerdir. Aşağıdaki ✗ örneklerin hata kodları (E0382, E0502, E0106,
// E0515) rustc ile DERLENEREK doğrulandı; uydurma değil.
//
// Stil: terim (İngilizce) + Türkçe açıklama + örnek + Neden? + Sorun (olmasaydı ne
// bozulurdu) + rustc: (derleyici hangi aşamada/dosyada zorluyor). Derleyici
// perspektifi dosyanın sonunda.

// ============================================================================
// SADE ANLATIM — önce buradan başla (hiç bilmeyen biri için)
// ============================================================================
// Bu dosyadaki her bölümün üstünde "SADE ANLATIM" diye bir kutu var. Önce onu oku, sonra
// altındaki teknik açıklamaya geç. Teknik kısım zor gelirse sorun değil: sade kutu ana fikri verir.
//
// Bu bölümün tek cümlelik özeti: Rust'ta her verinin TEK bir sahibi vardır; sahibi gidince
// veri de silinir. Başkaları o veriyi ancak "ödünç" alabilir.
//
// MİNİ SÖZLÜK (bu dosyada geçen kelimeler):
//   sahip (owner)        → veriyi elinde tutan değişken. Oyuncağın sahibi gibi: tek kişidir.
//   taşıma (move)        → veriyi başka bir değişkene VERMEK. Verdikten sonra sende kalmaz.
//   kopya (copy)         → sayılar gibi küçük şeylerin fotokopisini çekmek. İkisi de kalır.
//   klon (clone)         → büyük bir şeyin bire bir aynısını YENİDEN yapmak (emek ister).
//   drop                 → verinin silinmesi, kapladığı yerin geri verilmesi.
//   ödünç alma (borrow)  → sahibi değiştirmeden veriye bakmak ya da onu kullanmak.
//   referans (&)         → verinin kendisi değil, "nerede durduğunu" gösteren ok işareti.
//   scope (kapsam)       → bir değişkenin yaşadığı süslü parantez { ... } arası.
//   heap / stack         → bellekteki iki bölge. Stack: küçük ve hızlı raf. Heap: büyük depo.
//   derleyici (rustc)    → yazdığın kodu çalışan programa çeviren ve hataları önceden bulan araç.

mod derinlesme;

fn main() {
    sahiplik_ve_move();
    copy_vs_move();
    klonlama();
    kapsam_ve_drop();
    borrow_paylasimli();
    borrow_mut();
    sarkma_onleme();
    dilimler();
    derinlesme::calistir();
}

// 1. Ownership kuralları ve move (taşıma)
// Üç kural: (a) her değerin bir SAHİBİ (owner) vardır; (b) aynı anda tek sahip;
// (c) sahip scope dışına çıkınca değer DÜŞÜRÜLÜR (drop → bellek serbest).
// SADE ANLATIM: Elinde TEK bir oyuncak araba var. Onu arkadaşına verirsen araba artık onundur;
// sen oynayamazsın. Rust'ta veri de böyledir: `let s2 = s1;` yazınca veri s1'den s2'ye GEÇER.
// Neden böyle? İki kişi aynı arabanın sahibi olsaydı, ikisi de "işim bitti" deyip onu çöpe
// atmaya çalışırdı. Aynı şeyi iki kez çöpe atmak programı bozar. Tek sahip = tek kez silme.
//   Mini örnek:
//     let s1 = String::from("araba");
//     let s2 = s1;          // araba artık s2'de
//     println!("{s2}");     // ✓ çalışır
//     println!("{s1}");     // ✗ derlenmez: s1'in elinde artık bir şey yok
fn sahiplik_ve_move() {
    let s1 = String::from("merhaba");
    let s2 = s1; // s1'in heap verisi s2'ye TAŞINIR (move); s1 artık GEÇERSİZ
    println!("s2 = {s2}");
    // println!("{s1}"); // ✗ E0382: borrow of moved value — s1 artık sahibi değil

    // Neden kopyalayıp ikisini de geçerli bırakmıyor? Sorun: String heap'te veri tutar.
    // s1 ve s2 aynı heap bloğunu işaret edip İKİSİ DE geçerli olsaydı, ikisi de scope
    // sonunda o bloğu serbest bırakmaya çalışır → DOUBLE FREE (aynı belleği iki kez
    // free etmek → bellek bozulması / güvenlik açığı). Çözüm: atama bir MOVE'dur;
    // s1 geçersizleşir, bloğu yalnızca s2 serbest bırakır. Tek sahip = tek free.
    // rustc: move, MIR'de `Operand::Move(place)` olarak modellenir; taşınan yer artık
    //        "uninitialized" sayılır. Hangi yerlerin başlatılı olduğunu dataflow analizi
    //        (rustc_mir_dataflow: MaybeInitializedPlaces + MoveData) izler; taşınmış bir
    //        değeri kullanınca borrowck E0382 üretir
    //        (rustc_borrowck/src/diagnostics/mod.rs, "use of moved value").
}

// 2. Copy vs move
// Yalnızca STACK'te duran sabit boyutlu türler (i32, bool, char, f64, ve tamamı Copy
// olan tuple'lar) `Copy` trait'ini uygular: atama bit kopyası yapar, orijinal GEÇERLİ kalır.
// SADE ANLATIM: Bazı şeyler o kadar küçüktür ki vermek yerine fotokopisini çekmek daha kolaydır.
// "5" sayısını bir kâğıda yazıp arkadaşına verirsen, aynı sayıyı kendi kâğıdına da yazabilirsin;
// kimse bir şey kaybetmez. Sayılar, doğru/yanlış değerleri ve tek harfler böyledir: kopyalanır.
// Uzun bir yazı (String) ise büyük bir kutudur; her atamada fotokopisini çekmek pahalı olurdu,
// o yüzden kopyalanmaz, TAŞINIR.
//   Mini örnek:
//     let x = 5;
//     let y = x;            // fotokopi: ikisi de 5
//     println!("{x} {y}");  // ✓ 5 5 — x hâlâ duruyor
fn copy_vs_move() {
    let x = 5;
    let y = x; // Copy: x bit bit kopyalanır; x hâlâ geçerli
    println!("x = {x}, y = {y}"); // ikisi de kullanılabilir

    // Neden i32 Copy ama String değil? Sorun/mantık: Copy'nin güvenli olması için
    // değerin heap sahipliği OLMAMASI gerekir — yoksa kopyalamak iki sahip yaratır ve
    // yine double free riski doğar. i32 tamamen stack'te, heap sahibi yok → kopyalamak
    // zararsız. String heap sahibi → Copy olamaz, taşınır. Bu yüzden Copy ile move
    // birbirini dışlar: bir tür ya Copy'dir ya move edilir, ikisi birden olmaz.
}

// 3. clone (derin kopya)
// SADE ANLATIM: Arkadaşın da aynı legodan istiyorsa ona kendininkini vermek zorunda değilsin:
// aynısından bir tane daha YAPABİLİRSİN. Buna klonlama denir. Ama yeni bir lego yapmak zaman
// ve parça ister. Rust bu yüzden klonlamayı kendiliğinden yapmaz; sen `.clone()` yazarak
// "evet, bu emeği harcamak istiyorum" dersin.
//   Mini örnek:
//     let s1 = String::from("lego");
//     let s2 = s1.clone();      // ikinci bir lego yapıldı
//     println!("{s1} {s2}");    // ✓ ikisi de duruyor
fn klonlama() {
    let s1 = String::from("ada");
    let s2 = s1.clone(); // heap verisi DERİN kopyalanır (ayrı blok); s1 hâlâ geçerli
    println!("s1 = {s1}, s2 = {s2}");

    // Neden clone AÇIKÇA çağrılır (otomatik değil)? Sorun: heap'i derin kopyalamak
    // pahalıdır (tahsis + byte kopyalama). Diller bunu sessizce/otomatik yapsaydı,
    // farkında olmadan ağır kopyalar maliyet çıkarırdı. Rust move'u varsayılan (ucuz:
    // sadece pointer taşınır), derin kopyayı `.clone()` ile görünür/istemli yapar.
}

// 4. Scope ve Drop (RAII — kaynak yönetiminin kalbi)
// SADE ANLATIM: Oyun odasından çıkarken oyuncaklarını toplarsın. Rust'ta süslü parantez { }
// bir odadır; odanın sonuna gelince içeride tanımlanan her şey KENDİLİĞİNDEN toplanır (silinir).
// Senin "sil" demene gerek yoktur, unutma ihtimalin de yoktur.
//   Mini örnek:
//     {
//         let s = String::from("oyuncak");
//         println!("{s}");      // ✓ oda içindeyiz
//     }                         // oda bitti: s burada silindi
//     // println!("{s}");       // ✗ derlenmez: s artık yok
fn kapsam_ve_drop() {
    {
        let s = String::from("geçici");
        println!("içeride: {s}");
    } // ← s burada scope dışına çıkar: Drop::drop çağrılır, heap OTOMATİK serbest kalır
    // Buradan sonra s yok.

    // Neden ne GC ne de elle free()? Sorun: elle free (C) → unutursan sızıntı, iki kez
    // çağırırsan double free, erken çağırırsan use-after-free. GC (Java/Go) → çalışma
    // anında duraklamalar ve ek yük. Rust'ın yolu RAII: değer, sahibinin scope'u bitince
    // DETERMİNİSTİK biçimde ve SIFIR çalışma-anı maliyetiyle düşürülür.
    // rustc: drop çağrılarını SEN yazmazsın; "drop elaboration" geçişi MIR'e ekler
    //        (rustc_mir_transform/src/elaborate_drops.rs). Bir değer koşullu taşınmış
    //        olabileceği için "drop flag"ler kullanılır: yalnız hâlâ başlatılı değerler düşürülür.
    //        (Değişkenler scope sonunda ters tanım sırasında düşürülür.)
}

// 5. Shared borrow — &T (paylaşımlı ödünç)
// & ile bir değeri "ödünç verirsin": fonksiyon onu KULLANIR ama sahipliği DEVRALMAZ.
// SADE ANLATIM: Kitabını arkadaşına VERMEK istemiyorsun, sadece BAKSIN istiyorsun. Kitabı
// masada tutarsın, o da gelip okur. Kitap hâlâ senindir. `&` işareti tam olarak bunu söyler:
// "al değil, bak". Buna ödünç verme denir.
//   Mini örnek:
//     let kitap = String::from("masal");
//     let uzunluk = uzunluk_hesapla(&kitap);  // & → sadece bakması için veriyoruz
//     println!("{kitap} {uzunluk}");           // ✓ kitap hâlâ bizde
fn borrow_paylasimli() {
    let s = String::from("kitap");
    let n = uzunluk_hesapla(&s); // &s: ödünç; sahiplik s'de kalır
    println!("'{s}' uzunluğu {n}"); // s hâlâ bizim, kullanmaya devam
}
fn uzunluk_hesapla(x: &String) -> usize {
    x.len()
    // Neden borrow? Sorun: referans olmasaydı, değeri kullanmak için sahipliği
    // fonksiyona MOVE edip sonra geri döndürmen gerekirdi (her çağrıda değeri alıp
    // iade etmek) — zahmetli ve hataya açık. & bunu çözer: sahiplenmeden kullanırsın.
}

// 6. Mutable borrow — &mut T ve BORÇ KURALLARI (Aliasing XOR Mutability'nin somut hali)
// SADE ANLATIM: Bir boyama kitabı düşün. İsteyen herkes aynı anda BAKABİLİR; bunda sakınca yok.
// Ama biri BOYARKEN başkası bakarsa yarım resim görür, iki kişi aynı anda boyarsa resim bozulur.
// Rust'ın kuralı bu yüzden şudur: aynı anda ya istediğin kadar "bakan" (&) olur, ya da TEK bir
// "boyayan" (&mut). İkisi birlikte olmaz. Bu kural sayesinde kimse ayağının altından değişen
// bir veriyle karşılaşmaz.
//   Mini örnek:
//     let mut s = String::from("merhaba");
//     let bakan1 = &s;
//     let bakan2 = &s;              // ✓ iki kişi birlikte bakabilir
//     println!("{bakan1} {bakan2}");
//     let boyayan = &mut s;         // ✓ bakanların işi bitti, şimdi tek kişi değiştirebilir
//     boyayan.push('!');
fn borrow_mut() {
    let mut s = String::from("merhaba");
    ekle_dunya(&mut s); // &mut: değiştirmek için ödünç; s mut olmalı
    println!("{s}");

    // Kural: aynı anda ya N tane &T (okuma) YA DA tam 1 tane &mut T (yazma). İkisi birden OLMAZ.
    let r1 = &s;
    let r2 = &s; // birden çok paylaşımlı borrow aynı anda SERBEST (hepsi yalnız okur)
    println!("{r1} {r2}");

    let r3 = &mut s; // r1/r2 artık kullanılmadığı için (NLL) buradan itibaren &mut serbest
    r3.push('!');
    println!("{r3}");

    // ✗ Paylaşımlı ve mutable borrow'u ÇAKIŞTIRMAK derlenmez:
    //   let a = &s;
    //   let b = &mut s;        // E0502: cannot borrow `s` as mutable ... also borrowed as immutable
    //   println!("{a} {b}");
    // Neden? Sorun: a ile okurken b ile yazarsan, a ayağının altından değişen veriyi
    // görür (tutarsız okuma); veri heap'te yeniden tahsis edilirse a geçersiz belleği
    // gösterir. Bu kural tam da bunu imkânsızlaştırır.
    // rustc: borrow checker MIR üzerinde çalışır; borç kümelerini (rustc_borrowck/
    //        borrow_set.rs) ve çakışmayı (places_conflict.rs) hesaplar, ömürleri NLL ile
    //        çıkarır (nll.rs, region_infer/). "Last use"tan sonra borç biter — r1/r2 sonrası
    //        r3'ün serbest olmasının nedeni budur (NLL = non-lexical lifetimes).
}
fn ekle_dunya(x: &mut String) {
    x.push_str(" dünya");
}

// 7. Dangling reference (sarkan referans) ÖNLENİR
// SADE ANLATIM: Arkadaşına "defterim masamda duruyor, git bak" dedin. Ama o gidene kadar sen
// defteri çöpe attın. Arkadaşın boş masaya bakar. Programlarda buna "sarkan referans" denir:
// ok işareti duruyor ama gösterdiği şey artık yok. Çok tehlikelidir, çünkü program orada
// duran rastgele şeyi veri sanar. Rust böyle bir kodu hiç derlemez.
//   Mini örnek:
//     fn kotu() -> &String {
//         let s = String::from("defter");
//         &s                // ✗ derlenmez: s fonksiyon bitince silinecek, ok boşu gösterecekti
//     }
//     fn iyi() -> String {
//         String::from("defter")   // ✓ oku değil, defterin KENDİSİNİ ver
//     }
fn sarkma_onleme() {
    // ✗ Yerel bir değere referans döndürmek derlenmez:
    //   fn sarkan() -> &String {          // E0106: missing lifetime specifier
    //       let s = String::from("x");
    //       &s
    //   }
    //   // Lifetime yazıp zorlasan bile:
    //   fn sarkan<'a>() -> &'a String {   // E0515: cannot return reference to local variable `s`
    //       let s = String::from("x");
    //       &s                             // s fonksiyon bitince ölür → referans sarkardı
    //   }
    // Neden? Sorun: C/C++'ta yerel değişkenin adresini döndürmek klasik hatadır —
    // fonksiyon dönünce o stack/heap yeri geçersizleşir, referans "sarkar" (dangling),
    // onu kullanmak use-after-free olur. Rust bunu DERLEME ZAMANINDA reddeder.
    // Çözüm: referans değil, DEĞERİ döndür (sahipliği çağırana taşı):
    let s = sarkmayan();
    println!("sarkmayan: {s}");
    // rustc: borrowck, döndürülen referansın ömrünün fonksiyondan uzun olması gerektiğini
    //        ama yerelin ömrünün fonksiyonla bittiğini görür → E0515
    //        (rustc_borrowck/src/borrowck_errors.rs, "return reference to local").
}
fn sarkmayan() -> String {
    String::from("x") // sahiplik çağırana TAŞINIR; sarkan referans yok
}

// 8. Slice (dilim) — &str ve &[T]
// Dilim = mevcut verinin bir bölümüne (pointer + uzunluk) referans; KOPYA DEĞİL.
// SADE ANLATIM: Bir pastanın tamamını almadan "şu dilime bak" diyebilirsin. Dilim, pastadan
// ayrı yeni bir pasta değildir; aynı pastanın bir parçasını gösterir. Rust'ta `&s[0..7]` de
// böyledir: yazının bir parçasını gösterir, kopyasını çıkarmaz. Pasta ortadan kalkarsa dilim
// de kalmaz; Rust bu yüzden dilim dururken pastayı değiştirmene izin vermez.
//   Mini örnek:
//     let s = String::from("merhaba dünya");
//     let ilk = &s[0..7];       // "merhaba" — s'nin ilk 7 baytını gösterir, kopya değil
//     println!("{ilk}");
fn dilimler() {
    let s = String::from("merhaba dünya");
    let ilk = ilk_kelime(&s); // &str: s'in bir bölümüne bakar
    println!("ilk kelime: {ilk}");

    let dizi = [1, 2, 3, 4, 5];
    let orta = &dizi[1..4]; // &[i32] dilimi: [2, 3, 4]
    println!("orta = {orta:?}");

    // Neden dilim? Sorun: bir metnin/dizinin bir parçasını "indeks + uzunluk" diye ayrı
    // değişkenlerde taşımak, asıl veri değişince (ör. String temizlenince) o indeksleri
    // geçersiz kılar — senkronizasyon derdi ve hata kaynağı. Dilim, parçayı asıl veriye
    // BAĞLI bir referans olarak tutar; borrow checker dilimin ömrünü sahibe bağladığı için
    // sahibi, dilim dururken değiştiremezsin:
    //   let w = ilk_kelime(&s);
    //   s.clear();            // ✗ E0502 (DOĞRULANDI): w hâlâ &s borç'u tutuyor
    //   println!("{w}");
    // Böylece dilim asla sarkmaz — doğruluk derleme zamanında garanti.
}
fn ilk_kelime(s: &str) -> &str {
    // Not: &String argümanı &str parametresine "deref coercion" ile otomatik uyar.
    match s.find(' ') {
        Some(i) => &s[..i],
        None => s,
    }
}

// SADE ANLATIM (aşağıdaki bölüm için): Buraya kadar anlatılan kuralların hepsini program
// ÇALIŞIRKEN değil, sen kodu DERLERKEN bir "kontrolcü" denetler. Okula girmeden önce çantana
// bakan nöbetçi gibi: yanlış bir şey varsa içeri almaz. İçeri girdikten sonra kimse seni bir
// daha durdurmaz; yani çalışan program bu kontroller yüzünden yavaşlamaz. Aşağıdaki bölüm bu
// kontrolcünün iç işleyişini anlatır. İlk okuyuşta atlayabilirsin; yukarıdaki sekiz bölümü
// anlamış olman yeter.
// ============================================================================
// DERLEYİCİ PERSPEKTİFİ — ownership nerede, nasıl zorlanıyor?
// ============================================================================
//
// Ownership/borrow kuralları ÇALIŞMA ANINDA değil, DERLEME ANINDA denetlenir; çalışan
// programda referans sadece bir pointer, drop sadece bir free çağrısıdır — GC yoktur,
// ek yük yoktur. Aşamalar:
//
//   • Move takibi: HIR → MIR indirgemesinde (rustc_mir_build) kullanımlar move/copy diye
//     ayrışır. Move edilen yer "uninitialized" olur. Başlatılı/taşınmış durumunu dataflow
//     (rustc_mir_dataflow: MoveData, MaybeInitializedPlaces) izler.
//
//   • Borrow checker (rustc_borrowck): MIR üzerinde çalışır. Borçları toplar (borrow_set.rs),
//     çakışmaları bulur (places_conflict.rs), ömürleri/bölgeleri NLL ile çıkarır
//     (nll.rs, region_infer/). Ürettiği hatalar: E0382 (taşınmış değeri kullanma),
//     E0502 (& ile &mut çakışması), E0515/E0597 (yerel/kısa ömürlü referans sarkması).
//     Hepsi "Aliasing XOR Mutability" kuralının farklı yüzleridir.
//
//   • Drop elaboration (rustc_mir_transform/src/elaborate_drops.rs): değerlerin scope
//     bitince düşürülmesini MIR'e EKLER; koşullu taşınmaları drop flag'lerle yönetir.
//
//   • NLL (non-lexical lifetimes): bir borç, SÖZDİZİMSEL blok sonunda değil, SON KULLANIMINDA
//     biter. borrow_mut()'taki r1/r2'den sonra r3'ün (&mut) serbest olması bu yüzdendir.
//
// Kısacası: common-concepts'te "neden mut?" diye sorduğumuz kuralın (Aliasing XOR
// Mutability) uygulayıcısı bu bölümdeki ownership + borrow sistemidir, zorlayan da borrowck.
