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

fn main() {
    sahiplik_ve_move();
    copy_vs_move();
    klonlama();
    kapsam_ve_drop();
    borrow_paylasimli();
    borrow_mut();
    sarkma_onleme();
    dilimler();
}

// 1. Ownership kuralları ve move (taşıma)
// Üç kural: (a) her değerin bir SAHİBİ (owner) vardır; (b) aynı anda tek sahip;
// (c) sahip scope dışına çıkınca değer DÜŞÜRÜLÜR (drop → bellek serbest).
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
