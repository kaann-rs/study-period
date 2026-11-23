# Rust dersleri doğruluk denetimi — generics / closures / modules / cargo

**Denetim tarihi:** 2026-11-23
**Kapsam:** `learn/generics-traits-lifetimes`, `learn/closures-iterators`, `learn/modules`, `learn/cargo-and-crates` altındaki tüm `.rs` yorumları ve `ACIKLAMA.md` dosyaları.
**Yöntem:** `guide/rust` yerel derleyici kaynağı (rustc 1.99.0), doc.rust-lang.org (Book / Reference / rustdoc / Cargo), `rustc --explain`, `/tmp/opencode/rust-audit` altında mini reprolar. `"Çalıştır:"` komutları çalıştırılmadı; `learn/` altında değişiklik yapılmadı.
**Genel sonuç:** Çoğu teknik iddia doğru ve kaynak yolları (collector.rs:11-13, iterator.rs:78, late.rs:399/842, upvar.rs) yerinde. 14 nokta yanlış/yanıltıcı; bunların 5 tanesi "DÜZELTME NOTU" başlıklı satırların kendisi veya ana metinle çelişmesi. Bölüm atıflarının tamamı güncel Book ile uyumlu (ch17 async eklendiği için trait object Bölüm 18, patterns Bölüm 19 olmuş; dosyalar 18'i doğru kullanıyor).

---

## 1. Yanlış ve yanıltıcı iddialar

### 1.1 `learn/modules/src/main.rs:228`
- **Alıntı:** `//   rustc_resolve. (İlginç ayrıntı: Rust'ta iki ayrı "namespace" vardır — türler ve`
- **Hüküm:** YANLIŞ
- **Kaynak:** https://doc.rust-lang.org/reference/names/namespaces.html — Type, Value, Macro, Lifetime, Label namespace'leri listelenir; https://rustc-dev-guide.rust-lang.org/name-resolution.html — "There are different namespaces (e.g. macros, values, types, lifetimes)".
- **Önerilen düzeltme:** "Rust'ta birden fazla namespace vardır; bu örnekte çakışmayı önleyenler TÜR ve DEĞER namespace'leridir."
- **Önem:** ORTA (hemen altındaki "DÜZELTME NOTU" bile eksik; bkz. 1.2)

### 1.2 `learn/modules/src/main.rs:231`
- **Alıntı:** `// ↳ DÜZELTME NOTU (son gözden geçirme): namespace sayısı üçtür: türler, değerler ve MAKROLAR.`
- **Hüküm:** YANLIŞ (düzeltme notunun kendisi eksik)
- **Kaynak:** https://doc.rust-lang.org/reference/names/namespaces.html — başlık altında 5 namespace: *Type Namespace*, *Value Namespace*, *Macro Namespace*, *Lifetime Namespace*, *Label Namespace*.
- **Önerilen düzeltme:** "Güncel Reference'a göre 5 namespace vardır: türler, değerler, makrolar, lifetime'lar ve label'lar. Makro namespace'i bu tartışma için önemlidir, ama sayı üç değildir."
- **Önem:** ORTA

### 1.3 `learn/modules/src/main.rs:233-237`
- **Alıntı:**
  ```
  // • Görünürlük (visibility/privacy) denetimi ayrı bir geçişte zorlanır (crate: rustc_privacy).
  //   Doğrulanmış hatalar: private öğeye erişim E0603; private alanla dışarıdan struct kurma
  //   E0451; private alana erişim E0616.
  // ↳ DÜZELTME NOTU (son gözden geçirme): üç hata üç ayrı yerde üretilir: E0603 isim çözümlemede
  //   (rustc_resolve), E0616 tür denetiminde (rustc_hir_typeck), yalnız E0451 rustc_privacy'de.
  ```
- **Hüküm:** YANILTICI (yayın yerleri "yalnız şurada" denir gibi sunulmuş, ama çoklu yayın noktaları var)
- **Kaynak:** `guide/rust/compiler/rustc_resolve/src/diagnostics/mod.rs:1199` (E0603), `guide/rust/compiler/rustc_hir_typeck/src/diagnostics.rs:975` (E0603, tuple struct constructor), `guide/rust/compiler/rustc_hir_typeck/src/expr.rs:3391` ve `guide/rust/compiler/rustc_hir_analysis/src/hir_ty_lowering/mod.rs:3587` (E0616), `guide/rust/compiler/rustc_privacy/src/diagnostics.rs:13` (E0451). Repro: `/tmp/opencode/rust-audit/priv0603.rs`, `priv0451.rs`, `priv0616.rs` → sırasıyla E0603, E0451, E0616.
- **Önerilen düzeltme:** "E0603 çoğunlukla isim çözümlemede (rustc_resolve) üretilir, ancak tuple struct constructor için rustc_hir_typeck'te de üretilir; E0616 tür denetiminde/lower'da üretilir; E0451 rustc_privacy'de üretilir. Görünürlük denetiminin kendisi rustc_privacy'de yapılır."
- **Önem:** ORTA

### 1.4 `learn/modules/src/main.rs:72-74`
- **Alıntı:** `// super:: → bir ÜST modül (burada lokanta). salon, lokanta'nın torunu olduğu için`
  `// lokanta'nın private alt modülü mutfak'a buradan erişebilir.`
- **Hüküm:** YANILTICI (sonuç doğru, gerekçe yanlış)
- **Kaynak:** https://doc.rust-lang.org/reference/visibility-and-privacy.html — "If an item is private, it may be accessed by the current module and its descendants." Kodda `salon`, `lokanta`'nın **doğrudan alt** modülü; `mutfak` onun kardeşi.
- **Önerilen düzeltme:** "salon, lokanta'nın alt modülüdür; private `mutfak`'a erişim 'torun' olduğu için değil, private öğelerin tanımlandığı modülün kendisi ve tüm alt modülleri tarafından görülebilmesi kuralı sayesinde çalışır."
- **Önem:** DÜŞÜK

### 1.5 `learn/closures-iterators/src/main.rs:127-129`
- **Alıntı:**
  ```
  //   FnOnce  — en az bir kez çağrılabilir; yakalananı DIŞARI taşıyabilir (yalnız bir kez).
  // ↳ DÜZELTME NOTU (son gözden geçirme): doğru ifade "EN FAZLA bir kez çağrılabilir"dir. "En az" olan şey
  //   şudur: her closure en az FnOnce'ı UYGULAR (Fn ⊂ FnMut ⊂ FnOnce).
  ```
- **Hüküm:** YANLIŞ (ana metin; düzeltme notunun kendisi DOĞRU)
- **Kaynak:** `guide/rust/library/core/src/ops/function.rs:76,163,242` (`Fn: FnMut: FnOnce` zinciri); repro `/tmp/opencode/rust-audit/two_calls.rs` → E0382; `rustc --explain E0382`.
- **Önerilen düzeltme:** Ana satırı **"FnOnce — en fazla bir kez çağrılabilir; yakalananı dışarı taşıyabilir."** yap; düzeltme notunu kaldır. "En az FnOnce'ı uygular" ayrı ve doğru bir cümle olarak kalabilir.
- **Önem:** ORTA (yanlış ifade düzeltme notuyla yan yana duruyor)

### 1.6 `learn/closures-iterators/src/main.rs:147-149`
- **Alıntı:**
  ```
  // FnOnce alır: en az bir kez çağırır (sonucu döndürebilir). En gevşek kısıt.
  // ↳ DÜZELTME NOTU (son gözden geçirme): `F: FnOnce` alan fonksiyon closure'ı en fazla bir kez çağırabilir; iki kez
  //   çağırmak E0382 verir, hiç çağırmamak ise derlenir (ikisi de rustc ile doğrulandı).
  ```
- **Hüküm:** YANLIŞ (ana metin; düzeltme notu DOĞRU)
- **Kaynak:** `guide/rust/library/core/src/ops/function.rs:242`; repro `two_calls.rs` → E0382; `fn f<F: FnOnce()>(_: F) {}` gövdesiz hali derlenir (repro: `traitsel.rs`).
- **Önerilen düzeltme:** "FnOnce alır: **en fazla** bir kez çağırır (hiç çağırmayabilir); sonucu döndürebilir. En gevşek kısıt."
- **Önem:** ORTA

### 1.7 `learn/closures-iterators/src/derinlesme.rs:628`
- **Alıntı:**
  ```
  //    olarak `let mut kurallar` gerekir ve §2'deki döngü hem `filter` içinde hem gövdede `ihlaller`'i
  //    çağırdığı için derlenmez (E0501) ve durumlu kuralı iki kez tetiklerdi; döngüyü sipariş başına
  ```
- **Hüküm:** YANLIŞ (hata kodu)
- **Kaynak:** Repro `/tmp/opencode/rust-audit/e0501_try.rs` (aynı desen: `filter` içinde `&mut self` metodu + gövdede ikinci çağrı) → rustc 1.99 **E0499**: "cannot borrow `kurallar` as mutable more than once at a time". `rustc --explain E0501` = "A mutable variable is used but it is already captured by a closure" — bu senaryo değil.
- **Önerilen düzeltme:** "(E0499: cannot borrow `kurallar` as mutable more than once at a time)" yazın; E0501'i alıştırmadan kaldırın veya ayrı bir örnekle gösterin.
- **Önem:** ORTA

### 1.8 `learn/cargo-and-crates/ACIKLAMA.md:5`
- **Alıntı:** "`pub use` ise iç rafları gizleyip önden tek kapı açar; kullanıcı hangi rafın nerede olduğunu bilmek zorunda kalmaz."
- **Hüküm:** YANILTICI
- **Kaynak:** https://doc.rust-lang.org/reference/items/use-declarations.html — "An example of re-exporting: `mod quux { pub use self::foo::{bar, baz}; pub mod foo { ... } }`" (re-export, gizleme değil); yerel karşı örnek `learn/cargo-and-crates/src/lib.rs:121-125` — `turler` ve `islem` hâlâ `pub`, uzun yol da çalışıyor.
- **Önerilen düzeltme:** "`pub use` iç rafları gizlemez; onlara **ek olarak** önden kısa bir kapı açar. Kullanıcı uzun yolu bilmek zorunda kalmaz, ama bilirse çalışır."
- **Önem:** DÜŞÜK

### 1.9 `learn/cargo-and-crates/src/lib.rs:44`
- **Alıntı:** `//   opt-level                → derleyicinin kodu ne kadar hızlandırmaya uğraşacağı (0–3).`
- **Hüküm:** YANILTICI (eksik)
- **Kaynak:** https://doc.rust-lang.org/cargo/reference/profiles.html#opt-level — düzeyler `0`, `1`, `2`, `3` ve `"s"`, `"z"` (boyut optimizasyonu).
- **Önerilen düzeltme:** "`opt-level → 0–3 (ayrıca boyut için \"s\" ve \"z\")`".
- **Önem:** DÜŞÜK

### 1.10 `learn/closures-iterators/src/main.rs:298-301`
- **Alıntı:** `// • ZERO-COST: iterator zinciri runtime'da ek yük getirmez. Her adaptör generiktir →`
  `//   monomorphization her closure'ı somutlaştırır, sonra LLVM inline + optimizasyonla tüm zinciri`
  `//   elle yazılmış bir döngü kadar sıkı makine koduna indirger. Soyutlama derlemede eriyip gider;`
- **Hüküm:** YANILTICI (mutlak ifade)
- **Kaynak:** https://doc.rust-lang.org/book/ch13-04-performance.html — "**In many cases**, Rust code using iterators compiles to the same assembly you'd write by hand"; https://doc.rust-lang.org/cargo/reference/profiles.html — dev profili varsayılan `opt-level = 0`. Makine kodu karşılaştırması yapılmadı (bkz. DOĞRULANAMADI).
- **Önerilen düzeltme:** "Optimize derlemelerde (ör. `--release`) LLVM zinciri çoğu zaman elle yazılmış döngüyle aynı koda indirger; `opt-level = 0` (dev) derlemesinde dolaylama kalabilir."
- **Önem:** DÜŞÜK

### 1.11 `learn/generics-traits-lifetimes/src/main.rs:413`
- **Alıntı:** `//   kodunda lifetime diye bir şey YOKTUR (tür silme / "erasure").`
- **Hüküm:** YANILTICI (terminoloji)
- **Kaynak:** https://doc.rust-lang.org/reference/lifetime-elision.html (ömürler derleme zamanı kavramı); `guide/rust/compiler/rustc_monomorphize/src/collector.rs:4-13` — generic **türler** monomorphization ile koda yazılır, silinmez. "Tür silme" terminolojisi `dyn Trait` içindir (ör. `learn/closures-iterators/src/derinlesme.rs:26` doğru kullanıyor).
- **Önerilen düzeltme:** "`... makine kodunda lifetime diye bir şey YOKTUR (ömür silme / lifetime erasure). Generics'te türler silinmez, monomorphization ile kopyalanır; 'tür silme' terimi trait object'ler için kullanılır.`"
- **Önem:** DÜŞÜK

### 1.12 `learn/generics-traits-lifetimes/src/main.rs:123`
- **Alıntı:** `            (self.x * self.x + self.y * self.y).sqrt() // sqrt yalnız f64'te var`
- **Hüküm:** YANLIŞ
- **Kaynak:** https://doc.rust-lang.org/std/primitive.f32.html#method.sqrt (`f32::sqrt` mevcut); https://doc.rust-lang.org/std/primitive.f64.html#method.sqrt. Tam sayı türlerinde `sqrt` yok.
- **Önerilen düzeltme:** "`// sqrt f32'de de var, tam sayılarda yok; burada Nokta<f64> olduğu için çalışır`".
- **Önem:** DÜŞÜK

### 1.13 `learn/generics-traits-lifetimes/src/derinlesme.rs:410-412`
- **Alıntı:** `//     E0106: missing lifetime specifier`
  `//   Alanlarda elision YOKTUR; referans tutan her struct lifetime parametresi bildirir.`
- **Hüküm:** YANILTICI (aşırı genelleme)
- **Kaynak:** https://doc.rust-lang.org/reference/lifetime-elision.html — "`lifetimes` cannot be elided in struct definitions"; `&'static str` alanı açıkça yazıldığında parametre gerekmez (ör. `learn/closures-iterators/src/derinlesme.rs:142-145` alan `&'static str` kullanıyor).
- **Önerilen düzeltme:** "Alanlarda elision YOKTUR; referans tutan her struct alanı ya bir lifetime parametresi ya da açık `'static` bildirir."
- **Önem:** DÜŞÜK

### 1.14 `learn/generics-traits-lifetimes/src/derinlesme.rs:85-86`
- **Alıntı:**
  ```
  // ✗ Gereğinden sıkı: iki parametre de 'a. Sonuç yalnız `metin`den gelse bile imza "ikisinden
  //   de gelebilir" der; derleyici imzaya güvenir, gövdeye bakmaz.
  ```
- **Hüküm:** YANILTICI (aynı dosyanın 66-68. satırlarıyla çelişiyor)
- **Kaynak:** Bu dosya `:66-68` — "Derleyici bu sözü iki yerde denetler: gövdede ... ve çağrı yerinde ..."; repro `/tmp/opencode/rust-audit/lt_err.rs` → "error: lifetime may not live long enough" (E-kodusuz), yani gövde de denetlenir.
- **Önerilen düzeltme:** "**Çağrı yerinde** derleyici yalnız imzaya bakar; gövde ayrıca 'lifetime may not live long enough' hatasıyla denetlenir (bkz. aşağıdaki 115-117. satırlar)."
- **Önem:** DÜŞÜK

---

## 2. Doğrulanan / sorunsuz alanlar

### 2.1 Hata kodları ve derleyici davranışı (DOĞRU)
- `generics-traits-lifetimes/src/main.rs:9-10` — E0277 (trait bound eksik), E0599 (olmayan metot), E0106 (elision başarısız), E0038 (dyn uyumsuzluk), E0117 (orphan rule): `rustc --explain` ile doğrulandı; E0106/E0038/E0382/E0499/E0597/E0310/E0525/E0502/E0283/E0284/E0599/E0603/E0451/E0616 reprolarla doğrulandı (`/tmp/opencode/rust-audit/`).
- `generics-traits-lifetimes/src/derinlesme.rs:115-117` — "'lifetime may not live long enough' hatasının E-kodu YOKTUR": repro `lt_err.rs` çıktısında kod yok, "consider adding the following bound: `'b: 'a`" önerisi var ✓.
- `generics-traits-lifetimes/src/derinlesme.rs:161-168` — dönüşte `'a` yerine `'_` kullanılırsa `&mut self`'e bağlanır ve ikinci çağrı E0499 olur: repro `parser.rs` → E0499 ✓.
- `generics-traits-lifetimes/src/derinlesme.rs:239-285` — `&'static T` vs `T: 'static`, E0597 ("argument requires that `yerel` is borrowed for `'static`") ve E0310 ("the parameter type `T` may not live long enough"): repro `e0597_static.rs`, `e0310.rs` ✓.
- `generics-traits-lifetimes/src/derinlesme.rs:400-412` — E0369 (`>`), E0284 (`parse`), E0106 (struct alanı): `rustc --explain` + repro `parse_err.rs` ✓.
- `closures-iterators/src/derinlesme.rs:546-590` — E0382 / E0525 / E0502 / E0499 / E0373 kodları ve mesajları: repro `two_calls.rs`, `fnmut.rs`, `e0502.rs`, `e0499_two.rs`, `e0373.rs` ✓ (E0501 dışında, bkz. 1.7).
- `closures-iterators/src/derinlesme.rs:519-525` — `collect()` hedef tür yoksa E0283 ✓ (repro `collect_err.rs`); `parse()` için E0284 ✓ (`parse_err.rs`).
- `closures-iterators/src/derinlesme.rs:191-193` — `self.kural(tutar)` → E0599 "field, not a method", çözüm `(self.kural)(..)`: repro `fieldcall.rs` ✓.
- `modules/src/main.rs:11-12, 143-144` — E0603 / E0451 / E0616 üretiliyor ✓ (repro `priv0603.rs`, `priv0451.rs`, `priv0616.rs`); yayın yerleri için 1.3'e bakın.

### 2.2 Kaynak yolu / satır atıfları (DOĞRU)
- `generics-traits-lifetimes/src/main.rs:380-384` — `rustc_monomorphize/src/collector.rs` baş yorumunun ~12. satırı gerçekten "A generic definition in crate X might produce monomorphizations that are compiled into crate Y" diyor: `guide/rust/compiler/rustc_monomorphize/src/collector.rs:11-13` ✓. ("On demand" ifadesi aynı dosyada lazy collection stratejisi için `:145-154` ve constants/vtables için `:23-29` geçiyor; yorum bağlamı kabaca doğru.)
- `generics-traits-lifetimes/src/main.rs:389-392` — trait bound çözümü `rustc_hir_typeck`/`rustc_trait_selection`: `guide/rust/compiler/rustc_trait_selection/src/error_reporting/traits/fulfillment_errors.rs:339,967` (E0277) ✓.
- `generics-traits-lifetimes/src/main.rs:396-401` — dyn uyumluluk kontrolü: `guide/rust/compiler/rustc_trait_selection/src/traits/dyn_compatibility.rs:38` (`hir_ty_lowering_dyn_compatibility_violations`); `fn uret() -> Self` ve generic metot reproları E0038 veriyor ✓.
- `generics-traits-lifetimes/src/main.rs:405-412` — elision 3 kuralı ve `rustc_resolve/src/late.rs` simgeleri: `guide/rust/compiler/rustc_resolve/src/late.rs:399` (`fn elided`), `:842` (`current_elision_failures`) ✓; kuralların kendisi https://doc.rust-lang.org/reference/lifetime-elision.html ✓.
- `closures-iterators/src/main.rs:277-284` — capture mode `rustc_hir_typeck/src/upvar.rs`: `guide/rust/compiler/rustc_hir_typeck/src/upvar.rs:12,758-766` (ByRef/ByValue, closure kind) ✓; Edition 2021 disjoint capture (RFC 2229) ✓.
- `closures-iterators/src/main.rs:290-292` — `library/core/src/iter/traits/iterator.rs:78` === `fn next(&mut self) -> Option<Self::Item>;` ✓ (tam satır). `map/filter/sum` varsayılan metotlar ✓.
- `modules/src/main.rs:226-230` — name resolution `rustc_resolve` ve "path'ler tür denetiminden önce çözülür": https://rustc-dev-guide.rust-lang.org/name-resolution.html ("Later, after we have the whole AST, we do full name resolution ... in `rustc_resolve::late`") ✓; sadece "iki namespace" kısmı yanlış (bkz. 1.1).

### 2.3 Kavramsal iddialar (DOĞRU)
- `generics-traits-lifetimes/src/main.rs:21-54, 82-135, 141-250, 296-364` — generic/trait/lifetime tanımları, statik vs dinamik dispatch, vtable, blanket impl (`impl<T: Display> ToString for T` kalıbı gerçek kaynakta `guide/rust/library/alloc/src/string.rs:3015` = `impl<T: fmt::Display + ?Sized> ToString for T`), `'static`, elision örnekleri ✓ (mini örneklerdeki `Ali.konu()` ve `42.etiket()` sözdizimleri derleniyor: repro `mini_syntax.rs`).
- `generics-traits-lifetimes/src/derinlesme.rs:294-371` — bound'u struct yerine impl'e koyma; std'de `HashMap<K,V>` tanımında bound yok (`guide/rust/library/std/src/collections/hash/map.rs:245-257`), `insert` `:1332`, `get` `:1034`, `entry` `:1012` ✓. `FnOnce yeter` iddiası: `uret` yalnızca `Entry::Vacant` dalında çağrılıyor ✓.
- `generics-traits-lifetimes/src/derinlesme.rs:418-438` — associated type vs generic parametre (`Iterator::Item`, `Deref::Target`, `Add::Output`; `From<&str>`/`From<char>`) ✓.
- `generics-traits-lifetimes/src/derinlesme.rs:501-507` — alıştırma: `-> Option<&str>` (elision) E0106 veriyor (repro `elision.rs`); `-> Option<&'s str>` fonksiyonu derleniyor ama Vec düşürülünce test E0597 veriyor (repro `s_lifetime.rs`, `s_lifetime_test.rs`) ✓.
- `closures-iterators/src/main.rs:126-139` — Fn ⊂ FnMut ⊂ FnOnce hiyerarşisi ve "her closure en az FnOnce uygular" ✓ (`guide/rust/library/core/src/ops/function.rs:76,163,242`); DÜZELTME NOTU'nun kendisi doğru (ana metin hâlâ yanlış: bkz. 1.5).
- `closures-iterators/src/main.rs:184-196, 277-301` — adaptör tembelliği, tüketici, `must_use` uyarısı: repro `unused_map.rs` → "unused `Map` that must be used" + "iterators are lazy and do nothing unless consumed" ✓ (zero-cost'un mutlak ifadesi için bkz. 1.10).
- `closures-iterators/src/derinlesme.rs:64-122` — **`move` trait seçimini DEĞİŞTİRMEZ**: repro `traitsel.rs` derlendi ve çalıştı → `move || ad.chars().count()` iki kez çağrılabiliyor (Fn), `move` olmayan ama değeri döndüren `|| not` yalnız FnOnce ✓. Dosyanın ana tezi doğru.
- `closures-iterators/src/derinlesme.rs:148-179` — farklı closure'lar `impl Fn` dönüşünde E0308 ("no two closures, even if identical, have the same type"), çözüm `Box<dyn Fn>`: repro `match_closures.rs` → E0308 ✓.
- `closures-iterators/src/derinlesme.rs:421-477` — Fibonacci sayacı: `checked_add` taşmadan önce **92 terim** üretiliyor (son terim F(91)=4660046610375530309), `take(10)` = `[0,1,1,2,3,5,8,13,21,34]`, çiftlerin toplamı 44, 1000'i aşan ilk `(17, 1597)` ✓ (python + dosyadaki testlerle uyumlu).
- `closures-iterators/src/derinlesme.rs:291-292, 332-350` — HashMap sırası rastgele/değişebilir; f64 `Ord` değil, `max_by` + `total_cmp` ✓.
- `modules/src/main.rs:6-9, 65-98, 186-209` — package/crate/module, `crate::`/`self::`/`super::`, `pub(crate)`, `use`/`as`, re-export tanımları ✓.
- `modules/src/main.rs:245-247` — `mod salon;` → `src/salon.rs` veya `src/salon/mod.rs`: https://doc.rust-lang.org/reference/items/modules.html ("Module filenames may also be the name of the module as a directory with the contents in a file named `mod.rs`") ✓.
- `cargo-and-crates/src/lib.rs:6, 91-92, 127-129, 157-160` — doc-test'ler `cargo test` ile koşar, `main` sarılır: https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html ("if the example does not contain `fn main`, the remainder of the text is wrapped in `fn main() { your_code }`") ve https://doc.rust-lang.org/cargo/commands/cargo-test.html ("It extracts code samples from documentation comments of the library target") ✓.
- `cargo-and-crates/src/lib.rs:162-164` — `//!` içinde bulunduğu öğeyi, `///` sonraki öğeyi belgeler, Markdown → HTML ✓ (rustdoc book).
- `cargo-and-crates/src/lib.rs:170-178` — dev/release varsayılan opt-level 0/3; profil ayarları workspace **kök** Cargo.toml'undan okunur, üye crate'teki yok sayılır: https://doc.rust-lang.org/cargo/reference/profiles.html ("Cargo only looks at the profile settings in the Cargo.toml manifest at the root of the workspace"); repro `ws/member/Cargo.toml` → "warning: profiles for the non root package will be ignored" ✓. Workspace tek `target/` + `Cargo.lock` paylaşır ✓; `cargo install` → `~/.cargo/bin` ✓.
- **Bölüm atıflarının tamamı DOĞRU:** https://raw.githubusercontent.com/rust-lang/book/main/src/SUMMARY.md — Bölüm 7 packages/crates/modules, 9 error handling, 10 generics/traits/lifetimes, 13 iterators & closures, 14 cargo, 15 smart pointers, 16 concurrency, 18 trait objects. Dosyalardaki "Bölüm 5", "Bölüm 18", "Bölüm 15", "Bölüm 16", "Bölüm 9" atıfları güncel numaralarla uyumlu (Eski numaralar 17/18 iken yeni Book'ta trait object 18, patterns 19; kaynaklar 18'i kullanıyor) ✓.
- **ACIKLAMA.md benzetmeleri:** `generics`, `closures-iterators`, `modules` dosyalarındaki benzetmeler teknik olarak tutarlı ✓ (bkz. 1.8 cargo ACIKLAMA.md istisnası).

### 2.4 Küçük notlar
- Tüm `ACIKLAMA.md` dosyalarının son satırındaki ``Değişiklikleri `cargo check ile doğrula` `` ifadesinde kapanmayan backtick var: `` `cargo check `` → `` `cargo check` `` (biçim hatası, teknik değil).
- `cargo-and-crates/src/lib.rs:172-173`'teki "üye crate'te yazılırsa yok sayılır/uyarır — o yüzden buraya koymadık" gerekçesi doğru olsa da, bu crate'in profil ayarı koymamasının asıl nedeni varsayılanların kullanılması olabilir; teknik iddia yanlış değil.

## 3. DOĞRULANAMADI
- "iterator zinciri **elle yazılmış döngü kadar sıkı makine koduna** indirgenir" (1.10) iddiası için bu projede assembly/LLVM IR karşılaştırması yapılmadı; yalnız Book'ın "in many cases" nitelendirmesi ve profil varsayılanları esas alındı.
- Closure capture için RFC 2229 atfı ve `rustc_hir_typeck/src/upvar.rs` yorumları doğrulandı; ancak "disjoint capture her durumda gereksiz borrow çakışmalarını azaltır" iddiasının ölçülen bir performans/kullanım verisi yok (kavramsal olarak doğru kabul edildi).
