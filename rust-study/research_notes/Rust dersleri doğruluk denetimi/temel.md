# Rust dersleri doğruluk denetimi — temel kavramlar (common-concepts / basic-concepts / ownership)

Kapsam: `learn/common-concepts`, `learn/basic-concepts`, `learn/ownership` altındaki 4 `.rs` dosyasının yorum satırları ve 3 `ACIKLAMA.md`. Yalnızca araştırma yapıldı; `learn/` altında hiçbir dosya değiştirilmedi. Yeniden üretim dosyaları `/tmp/opencode/verify/` altına yazıldı ve `rustc 1.99.0 (b940084d7 2026-09-28)` ile derlendi. Yerel kaynak ağacı: `guide/rust` (RELEASES.md’e göre 1.99.1).

Genel sonuç: Denetlenen iddiaların büyük çoğunluğu doğru; özellikle hata kodları, rustc kaynak yolu/satır numaraları ve “derlendi/DOĞRULANDI” iddiaları birebir doğrulandı. Bulunan sorunlar çoğunlukla (a) birkaç yanlış/eksik kaynak dosya yolu, (b) “yalnızca stack’teki türler Copy’dir” ve “drop’un sıfır çalışma-anı maliyeti” gibi aşırı genellemeler, (c) aynı yorum bloğunda önce yanlış yazılıp sonra düzeltilmiş ama eski cümlenin bırakılmış olması, (d) `ACIKLAMA.md` benzetmesindeki “değişmezlik = kuralın ta kendisi” ifadesi.

---

## 1. `learn/common-concepts/src/main.rs`

### 1.1. Rust Book atfı — DOĞRU

- **Yol/satır:** `learn/common-concepts/src/main.rs:2`
- **Alıntı:** `// Rust Book — Bölüm 3: Yaygın Programlama Kavramları (Common Programming Concepts)`
- **Hüküm:** DOĞRU
- **Kaynak:** https://doc.rust-lang.org/book/ch03-00-common-programming-concepts.html — sayfa başlığı “Common Programming Concepts”, kapsamı “variables, basic types, functions, comments, and control flow”.
- **Düzeltme:** Yok.
- **Önem:** küçük.

### 1.2. E0384’ün üretildiği aşama — ilk cümle YANLIŞ, düzeltme notu DOĞRU

- **Yol/satır:** `learn/common-concepts/src/main.rs:103`
- **Alıntı:** `// rustc: immutable bir binding'e atama hatası (E0384) tür/HIR denetiminde yakalanır;`
- **Hüküm:** YANLIŞ (hemen sonraki satırlarda düzeltilmiş)
- **Kaynak:** `guide/rust/compiler/rustc_borrowck/src/borrowck_errors.rs:252-260` — `cannot_reassign_immutable()` içinde `struct_span_code_err!(..., E0384, "cannot assign {} {}", ...)`. Ayrıca `guide/rust/compiler/rustc_hir_typeck/src/fn_ctxt/checks.rs:2632` E0308 içindir; E0384 typeck’te üretilmez. Yeniden üretim: `/tmp/opencode/verify/e0384_alone.rs` → `error[E0384]: cannot assign twice to immutable variable 'x'`.
- **Düzeltme:** İlk cümleyi tümüyle sil; 104-106. satırlardaki düzeltme notunu tek doğru metin olarak bırak.
- **Önem:** küçük (yanlış bilgi aynı blokta düzeltilmiş olsa da okuyucu ilk cümleyi alıntılayabilir).

- **Yol/satır:** `learn/common-concepts/src/main.rs:104-106`
- **Alıntı:** `// ↳ DÜZELTME NOTU (son gözden geçirme): E0384 de MIR üzerinde borrow checker (rustc_borrowck, borrowck_errors.rs) tarafından üretilir. Kanıtı: aynı fonksiyonda bir tür hatası (E0308) varsa E0384 hiç raporlanmaz, çünkü derleme borrowck aşamasına ulaşmaz.`
- **Hüküm:** DOĞRU
- **Kaynak:** `guide/rust/compiler/rustc_borrowck/src/borrowck_errors.rs:252-260`; yeniden üretim `/tmp/opencode/verify/e0384_and_e0308.rs` → yalnız `error[E0308]: mismatched types`, E0384 raporlanmadı; `/tmp/opencode/verify/e0384_alone.rs` → E0384. Ayrıca E0308’in typeck kaynağı: `guide/rust/compiler/rustc_hir_typeck/src/fn_ctxt/checks.rs:2632`.
- **Düzeltme:** Yok.
- **Önem:** küçük.

### 1.3. const evaluation crate’i — DOĞRU

- **Yol/satır:** `learn/common-concepts/src/main.rs:126-127`
- **Alıntı:** `// rustc: const ifadeleri derleme anında "const evaluation" ile hesaplanır // (rustc_const_eval crate'i; aynı motor dizi uzunluğu [u8; N] gibi yerlerde çalışır).`
- **Hüküm:** DOĞRU
- **Kaynak:** `guide/rust/compiler/rustc_const_eval/` dizini mevcut; ayrıca `compiler/rustc_const_eval/src/transform/promote_consts` (Rustdoc: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_const_eval/transform/promote_consts/index.html).
- **Düzeltme:** Yok.
- **Önem:** küçük.

### 1.4. Taşma denetiminin kaynak konumu — DOĞRU

- **Yol/satır:** `learn/common-concepts/src/main.rs:191-194`
- **Alıntı:** `// rustc: taşma denetimi MIR KURULUMUNDA eklenir. check_overflow bayrağı // compiler/rustc_mir_build/src/builder/mod.rs:872 civarında // (check_overflow |= tcx.sess.overflow_checks();) belirlenir; açıksa toplama/ // çıkarma bir CheckedBinaryOp + Assert'e dönüşür, kapalıysa (release) sade işleme.`
- **Hüküm:** DOĞRU
- **Kaynak:** `guide/rust/compiler/rustc_mir_build/src/builder/mod.rs:871-872` — birebir `check_overflow |= tcx.sess.overflow_checks();` satırı 872’de. AssertKind::Overflow tanımı: `guide/rust/compiler/rustc_middle/src/mir/syntax.rs:1042`.
- **Düzeltme:** Yok.
- **Önem:** küçük.

### 1.5. Sabit dizi indeksi iddiası — YANLIŞ (düzeltilmiş)

- **Yol/satır:** `learn/common-concepts/src/main.rs:220`
- **Alıntı:** `// dizi[5] yazsan çalışma anında PANİK: "index out of bounds".`
- **Hüküm:** YANLIŞ (221-223. satırlarda düzeltilmiş)
- **Kaynak:** Yeniden üretim `/tmp/opencode/verify/const_index.rs` → `error: this operation will panic at runtime ... index out of bounds: the length is 3 but the index is 5` ve `note: #[deny(unconditional_panic)] on by default`. Yani sabit indeks derleme zamanında reddedilir; çalışma-anı paniği yalnız `dizi[i]` gibi çalışırken belli olan indeks için söz konusudur (219. satır doğru).
- **Düzeltme:** 220. satırı sil veya şu hâle getir: `// dizi[i] çalışma anında PANİK; dizi[5] ise derlemede deny(unconditional_panic) ile reddedilir (aşağıda).`
- **Önem:** küçük.

### 1.6. Sabit indeks düzeltme notu — DOĞRU

- **Yol/satır:** `learn/common-concepts/src/main.rs:221-223`
- **Alıntı:** `// ↳ DÜZELTME NOTU (son gözden geçirme): indeks SABİT yazılırsa (dizi[5]) derleyici bunu // önceden görür ve hiç derlemez ("this operation will panic at runtime"). Çalışma anı // paniği, indeks ancak program çalışırken belli oluyorsa (dizi[i]) görülür.`
- **Hüküm:** DOĞRU
- **Kaynak:** Yeniden üretim `/tmp/opencode/verify/const_index.rs` (yukarıdaki çıktı); mesajın birebir metni “this operation will panic at runtime”.
- **Düzeltme:** Yok.
- **Önem:** küçük.

### 1.7. Bounds-check Assert kaynak konumları — DOĞRU

- **Yol/satır:** `learn/common-concepts/src/main.rs:226-230`
- **Alıntı:** `// AssertKind::BoundsCheck tanımı compiler/rustc_middle/src/mir/syntax.rs:1041; // indeksleme öncesi bu Assert'i ekleyen yer // compiler/rustc_mir_build/src/builder/expr/as_place.rs:752. Optimizer indeksin // güvenli olduğunu kanıtlayabilirse (ör. sabit) denetimi kaldırır — güvenlik yine durur.`
- **Hüküm:** DOĞRU
- **Kaynak:** `guide/rust/compiler/rustc_middle/src/mir/syntax.rs:1041` → `BoundsCheck { len: O, index: O },`; `guide/rust/compiler/rustc_mir_build/src/builder/expr/as_place.rs:751-754` → `let msg = BoundsCheck {...}` ve `self.assert(..., msg, expr_span)`.
- **Düzeltme:** Yok.
- **Önem:** küçük.

### 1.8. `for` döngüsü desugaring iddiası — DOĞRU

- **Yol/satır:** `learn/common-concepts/src/main.rs:338-343`
- **Alıntı:** `// rustc: 'for x in it {...}' sözdizimsel şekerdir (syntactic sugar). AST→HIR indirgemesi // (rustc_ast_lowering) bunu kabaca şuna açar: // let mut it = IntoIterator::into_iter(it); // loop { match it.next() { Some(x) => {...}, None => break } }`
- **Hüküm:** DOĞRU
- **Kaynak:** `guide/rust/compiler/rustc_ast_lowering/src/expr.rs:1758-1772` (desugaring şeması: `match IntoIterator::into_iter(<head>) { mut iter => loop { match Iterator::next(&mut iter) { None => break, Some(<pat>) => <body> } } }`) ve `:1877-1888` (`LangItem::IntoIterIntoIter` çağrısı).
- **Düzeltme:** Yok. Kaynakta desugaring `match + loop + match` biçimindedir; yorumdaki “kabaca” nitelemesi bunu kapsar.
- **Önem:** küçük.

### 1.9. Derleyici borusu hattındaki E0384 konumu — ilk cümle YANLIŞ, düzeltme DOĞRU

- **Yol/satır:** `learn/common-concepts/src/main.rs:365-367`
- **Alıntı:** `// 4. rustc_hir_typeck + : tür denetimi, tür çıkarımı (inference), trait çözümü. // rustc_hir_analysis Immutable binding'e atama (E0384), tür uyuşmazlığı (E0308) burada. // ↳ DÜZELTME NOTU (son gözden geçirme): E0308 bu adımda; E0384 ise 6. adımda (rustc_borrowck) üretilir.`
- **Hüküm:** YANLIŞ (366. satır) / DOĞRU (367. satır düzeltmesi)
- **Kaynak:** 1.2’deki kaynaklar; ayrıca 4. adımdaki crate adlarının tümü `guide/rust/compiler/` altında mevcut (`rustc_hir_typeck`, `rustc_hir_analysis`). 6. adım `rustc_borrowck` doğru.
- **Düzeltme:** 366. satırdan “Immutable binding'e atama (E0384),” ifadesini çıkar.
- **Önem:** küçük.

### 1.10. Aliasing XOR Mutability / noalias — DOĞRU (küçük nüans)

- **Yol/satır:** `learn/common-concepts/src/main.rs:380-405` (özellikle 389-395 ve 401-403)
- **Alıntı:** `// Rust'ın tek kuralı bunların HEPSİNİ baştan keser: herhangi bir anda ya // • N tane paylaşımlı &T (yalnız okuma, istediğin kadar), YA DA // • tam 1 tane &mut T (okuma+yazma, tek sahip) ... // + '&mut T'nin tekliği LLVM'e noalias der → daha agresif optimizasyon.`
- **Hüküm:** DOĞRU
- **Kaynak:** https://doc.rust-lang.org/nomicon/races.html — veri yarışı tanımı “two or more threads concurrently accessing a location of memory / one or more of them is a write / one or more of them is unsynchronized” ve “impossible to alias a mutable reference”. `noalias` için kesin kaynak: `guide/rust/compiler/rustc_ty_utils/src/abi.rs:358-373` — “`&mut T` and `Box<T>` where `T: Unpin` are unique and hence `noalias`”, `if no_alias && !is_return { attrs.set(ArgAttribute::NoAlias); }`; LLVM’e uygulama: `guide/rust/compiler/rustc_codegen_llvm/src/abi.rs:43`.
- **Düzeltme:** İsteğe bağlı nüans ekle: `noalias` yalnız **argüman** konumunda ve `T: Unpin` için verilir (dönüş değerinde verilmez). İddia yanlış değil; eksik ayrıntı.
- **Önem:** küçük.

---

## 2. `learn/basic-concepts/src/main.rs`

### 2.1. const generic varsayılanı — DOĞRU

- **Yol/satır:** `learn/basic-concepts/src/main.rs:54-56`
- **Alıntı:** `// struct Buffer<const N: usize = 0> { data: [u8; N] } // N bir const generic parametresidir; Buffer<4>, dört baytlık bir diziye sahiptir. // Varsayılan = 0, const argüman verilmediğinde bir uzunluk sağlar.`
- **Hüküm:** DOĞRU
- **Kaynak:** Yeniden üretim `/tmp/opencode/verify/const_generic.rs` — aynı tanım `rustc 1.99.0` ile **stable** derlendi (yalnız `dead_code` uyarısı); `Buffer::<4>` kullanımı geçerli.
- **Düzeltme:** Yok.
- **Önem:** küçük.

### 2.2. Fn/FnMut/FnOnce gerekçesinde “data race” ifadesi — YANILTICI

- **Yol/satır:** `learn/basic-concepts/src/main.rs:87-90`
- **Alıntı:** `// Bu ayrım olmasaydı: bir kez tüketen closure'ı iki kez çağırmak, taşınmış/serbest // bırakılmış değeri tekrar kullanırdı (use-after-move / double-free); değiştiren bir // closure'a aynı anda iki yerden erişmek veri yarışı (data race) olurdu.`
- **Hüküm:** YANILTICI
- **Kaynak:** https://doc.rust-lang.org/nomicon/races.html — data race tanımı **iki veya daha fazla thread** eşzamanlı erişimini gerektirir. Tek thread’de iki `&mut` alias’ı aliasing kuralını ihlal eder (UB), fakat “data race” değildir. Aynı öğrenme setindeki `learn/common-concepts/src/main.rs:387` bunu doğru söylüyor: `// • İki thread aynı veriyi biri yazarken biri okursa → data race.`
- **Düzeltme:** Şu cümleyle değiştir: “değiştiren bir closure’a aynı anda iki yerden erişmek `&mut` tekliğini (aliasing kuralını) ihlal eder; bu tür eşzamanlı erişim **thread’ler arasında** olursa sonucu veri yarışıdır.”
- **Önem:** küçük (kavramsal karışıklık; aynı dosyada tutarlı olmayan terminoloji).

### 2.3. HRTB / constant promotion / E0597 doğrulama iddiası — DOĞRU

- **Yol/satır:** `learn/basic-concepts/src/main.rs:182-183`, `208-212`, `240-248`
- **Alıntı:** `// Doğrulama: Nomicon (HRTB) + Rust Reference (constant promotion); davranış // rustc ile de test edildi (aşağıdaki yerel değişken örneği hex_v3'te E0597 verir).` ve `// Tuzak: aşağıdaki gövdeler f(&1) kullanıyor ve İKİSİ DE derlenir! Çünkü &1 // literali 'static'e TERFİ eder (constant promotion = ... , Rust Reference → &'static i8),`
- **Hüküm:** DOĞRU
- **Kaynak:** https://doc.rust-lang.org/nomicon/hrtb.html (`for<'a>` = “for all choices of 'a”); https://doc.rust-lang.org/reference/destructors.html#constant-promotion (“Promotion of a value expression to a `'static` slot occurs when the expression could be written in a constant and borrowed...”); yeniden üretim `/tmp/opencode/verify/hrtb_local.rs` → yalnızca `hex_v3_yerel` için `error[E0597]: 'n' does not live long enough`, `hex_v2_yerel` hatasız; `/tmp/opencode/verify/hrtb_literal.rs` → iki fonksiyon da hatasız (yalnız `dead_code` uyarısı).
- **Düzeltme:** Yok.
- **Önem:** küçük.

### 2.4. RFC 3323 atfı — DOĞRU (nüans: nightly)

- **Yol/satır:** `learn/basic-concepts/src/main.rs:339-342`
- **Alıntı:** `// RFC 3323, public bir trait'i kimin implemente edebileceğini sınırlamak için // ayrı bir özellik öneriyor. Örneğin pub impl(crate) trait Example {}, diğer // crate'lerin Example'ı kullanmasına izin verirken implementasyonları kendi // crate'ine saklardı.`
- **Hüküm:** DOĞRU
- **Kaynak:** https://rust-lang.github.io/rfcs/3323-restrictions.html — Summary: “You can write `pub impl(crate) trait Foo {}`, which limits the ability to implement the trait to the crate it is defined in.”; tracking issue https://github.com/rust-lang/rust/issues/105077 (`impl_restriction` feature gate).
- **Düzeltme:** İsteğe bağlı güncelleme: özellik 2026 itibarıyla nightly’de `impl_restriction` feature gate’i altında kullanılabiliyor (https://blog.rust-lang.org/inside-rust/2026/08/10/call-for-testing-impl-and-mut-restrictions); “öneriyor” ifadesi RFC için doğru olmaya devam eder.
- **Önem:** küçük.

### 2.5. Auto trait / negative impl “nightly” iddiası — DOĞRU

- **Yol/satır:** `learn/basic-concepts/src/main.rs:271-298` (özellikle 271 ve 289)
- **Alıntı:** `// 7. Auto trait ve negative impl (nightly Rust)` / `// #![feature(auto_traits, negative_impls)]`
- **Hüküm:** DOĞRU
- **Kaynak:** `guide/rust/compiler/rustc_ast_passes/src/feature_gate.rs:166,179,568,570` — `negative_impls` ve `auto_traits` hâlâ feature gate altında (“auto traits are experimental and possibly buggy”, “negative impls are experimental”).
- **Düzeltme:** Yok.
- **Önem:** küçük.

---

## 3. `learn/ownership/src/main.rs`

### 3.1. “rustc ile doğrulandı” iddiası (E0382/E0502/E0106/E0515) — DOĞRU

- **Yol/satır:** `learn/ownership/src/main.rs:5-7`
- **Alıntı:** `// Bu crate, common-concepts'teki KÖK KURALIN — "Aliasing XOR Mutability" — somut // olarak yaşadığı yerdir. Aşağıdaki ✗ örneklerin hata kodları (E0382, E0502, E0106, // E0515) rustc ile DERLENEREK doğrulandı; uydurma değil.`
- **Hüküm:** DOĞRU
- **Kaynak:** Yeniden üretimler (rustc 1.99.0): `/tmp/opencode/verify/e0382.rs` → `error[E0382]: borrow of moved value: 's1'`; `/tmp/opencode/verify/e0502.rs` → `error[E0502]: cannot borrow 's' as mutable because it is also borrowed as immutable`; `/tmp/opencode/verify/e0106.rs` → `error[E0106]: missing lifetime specifier`; `/tmp/opencode/verify/e0515.rs` → `error[E0515]: cannot return reference to local variable 's'`.
- **Düzeltme:** Yok.
- **Önem:** küçük.

### 3.2. Double free gerekçesi — DOĞRU

- **Yol/satır:** `learn/ownership/src/main.rs:53-54`, `66-70`, `95-99`
- **Alıntı:** `// İki kişi aynı arabanın sahibi olsaydı, ikisi de "işim bitti" deyip onu çöpe // atmaya çalışırdı. Aynı şeyi iki kez çöpe atmak programı bozar. Tek sahip = tek kez silme.` ve `// s1 ve s2 aynı heap bloğunu işaret edip İKİSİ DE geçerli olsaydı, ikisi de scope // sonunda o bloğu serbest bırakmaya çalışır → DOUBLE FREE ...`
- **Hüküm:** DOĞRU
- **Kaynak:** https://doc.rust-lang.org/std/marker/trait.Copy.html — “copying `String` would duplicate responsibility for managing the String's buffer, leading to a double free down the line”; “any type implementing Drop can't be Copy, because it's managing some resource besides its own size_of::<T> bytes.”
- **Düzeltme:** Yok.
- **Önem:** küçük.

### 3.3. Move/initialization dataflow kaynak yolları — DOĞRU

- **Yol/satır:** `learn/ownership/src/main.rs:71-74`
- **Alıntı:** `// rustc: move, MIR'de Operand::Move(place) olarak modellenir; taşınan yer artık // "uninitialized" sayılır. Hangi yerlerin başlatılı olduğunu dataflow analizi // (rustc_mir_dataflow: MaybeInitializedPlaces + MoveData) izler;`
- **Hüküm:** DOĞRU
- **Kaynak:** `guide/rust/compiler/rustc_mir_dataflow/src/move_paths/mod.rs:176` → `pub struct MoveData<'tcx>`; `guide/rust/compiler/rustc_mir_dataflow/src/impls/initialized.rs:103,139` → `MaybeInitializedPlaces` tanımı ve doc’u (“tracks all places that might be... initialized”). Drop elaboration’ın da aynı analizi kullandığı: `guide/rust/compiler/rustc_mir_transform/src/elaborate_drops.rs:8,64`.
- **Düzeltme:** Yok.
- **Önem:** küçük.

### 3.4. E0382’nin üretildiği dosya yolu — YANLIŞ

- **Yol/satır:** `learn/ownership/src/main.rs:75`
- **Alıntı:** `//        (rustc_borrowck/src/diagnostics/mod.rs, "use of moved value").`
- **Hüküm:** YANLIŞ
- **Kaynak:** Gerçek üretici `guide/rust/compiler/rustc_borrowck/src/borrowck_errors.rs:320-340` — `cannot_act_on_moved_value()`, `E0382` burada `struct_span_code_err!` ile oluşturuluyor; çağrı yeri `guide/rust/compiler/rustc_borrowck/src/diagnostics/conflict_errors.rs:170`. `guide/rust/compiler/rustc_borrowck/src/diagnostics/mod.rs:1261` yalnızca bir **doc comment içindeki örnek çıktıdır** (`// error[E0382]: use of moved value: 'blk'`), tanı/üretim yeri değildir.
- **Düzeltme:** `(rustc_borrowck/src/borrowck_errors.rs, "use of moved value — cannot_act_on_moved_value")`.
- **Önem:** orta (“rustc:” notu olarak verilmiş kesin bir kaynak yanlış; aynı dosya 245. satırda E0515 için **doğru** yolu kullanıyor).

### 3.5. “Yalnızca stack’teki türler Copy’dir” — YANILTICI

- **Yol/satır:** `learn/ownership/src/main.rs:79-80`
- **Alıntı:** `// Yalnızca STACK'te duran sabit boyutlu türler (i32, bool, char, f64, ve tamamı Copy // olan tuple'lar) Copy trait'ini uygular: atama bit kopyası yapar, orijinal GEÇERLİ kalır.`
- **Hüküm:** YANILTICI
- **Kaynak:** https://doc.rust-lang.org/std/marker/trait.Copy.html — “Shared references (`&T`) are also Copy, so a type can be Copy, even when it holds shared references of types T that are not Copy”; “Function pointer types (e.g., `fn() -> i32`)”; “Closure types, if they capture no value from the environment or if all such captured values implement Copy themselves”; `impl<T> Copy for &T`, `impl Copy for *const T`, `impl<T: Copy> Copy for Option<T>`, `impl<T: Copy, const N: usize> Copy for [T; N]`. Yani Copy, “stack’te durma” değil **bit kopyasının güvenli olması** ile ilgilidir; heap’i işaret eden `&String` bile Copy’dir (sahiplik taşımadığı için).
- **Düzeltme:** “Copy, bit kopyasıyla güvenle çoğaltılabilen türler için uygulanır: sayılar, bool, char, `&T`, fn işaretçileri, tüm alanları Copy olan struct/tuple/dizi/Option gibi türler. Heap **sahibi** olan türler (String, Vec) Drop gerektirdiğinden Copy olamaz.”
- **Önem:** orta ( Yanlış genelleme, dersin ilerleyen bölümlerinde referanslarla çelişir).

### 3.6. Copy’nin heap sahipliği gerektirmemesi — DOĞRU

- **Yol/satır:** `learn/ownership/src/main.rs:95-99`
- **Alıntı:** `// Neden i32 Copy ama String değil? Sorun/mantık: Copy'nin güvenli olması için // değerin heap sahipliği OLMAMASI gerekir — yoksa kopyalamak iki sahip yaratır ve // yine double free riski doğar. ... Bu yüzden Copy ile move // birbirini dışlar: bir tür ya Copy'dir ya move edilir, ikisi birden olmaz.`
- **Hüküm:** DOĞRU
- **Kaynak:** https://doc.rust-lang.org/std/marker/trait.Copy.html (“copying `String` would duplicate responsibility... double free”; “any type implementing Drop can't be Copy”). Not: “heap sahipliği” doğru ifade; `&String` heap’i **gösterir** ama sahip değildir ve Copy’dir — 3.5’teki düzeltme ile birlikte okunmalı.
- **Düzeltme:** Yok.
- **Önem:** küçük.

### 3.7. Drop elaboration / drop flag / ters sıra — DOĞRU

- **Yol/satır:** `learn/ownership/src/main.rs:143-146`
- **Alıntı:** `// rustc: drop çağrılarını SEN yazmazsın; "drop elaboration" geçişi MIR'e ekler // (rustc_mir_transform/src/elaborate_drops.rs). Bir değer koşullu taşınmış // olabileceği için "drop flag"ler kullanılır: yalnız hâlâ başlatılı değerler düşürülür. // (Değişkenler scope sonunda ters tanım sırasında düşürülür.)`
- **Hüküm:** DOĞRU
- **Kaynak:** `guide/rust/compiler/rustc_mir_transform/src/elaborate_drops.rs:8,27,64` (`use ... MaybeInitializedPlaces`, “achieved by inserting drop flags for every variable...”); ters sıra için https://doc.rust-lang.org/reference/destructors.html#destructors-scope-drop-order — “all variables associated to that scope are dropped in reverse order of declaration (for variables)”. (Ayrı bir ayrıntı: struct **alanları** ters sırada değil, bildirim sırasında düşürülür; yorum yalnızca değişkenlerden bahsettiği için doğrudur.)
- **Düzeltme:** Yok.
- **Önem:** küçük.

### 3.8. “SIFIR çalışma-anı maliyetiyle düşürülür” — YANILTICI

- **Yol/satır:** `learn/ownership/src/main.rs:140-142`
- **Alıntı:** `// anında duraklamalar ve ek yük. Rust'ın yolu RAII: değer, sahibinin scope'u bitince // DETERMİNİSTİK biçimde ve SIFIR çalışma-anı maliyetiyle düşürülür.`
- **Hüküm:** YANILTICI
- **Kaynak:** https://doc.rust-lang.org/reference/destructors.html — destructor, `T: Drop` ise `<T as Drop>::drop` çağrısını ve alanların yıkımını içerir; String/Vec için bu heap serbest bırakma (dealloc/free) anlamına gelir ve çalışma anında gerçekleşir. Aynı dosyanın **299. satırı** bunu doğru söylüyor: `// programda referans sadece bir pointer, drop sadece bir free çağrısıdır`. “Sıfır maliyet” yalnız borrow/ownership **kontrollerinin** çalışma anında yapılmaması için geçerlidir.
- **Düzeltme:** “deterministik biçimde, derleyicinin eklediği kodla ve GC duraklaması/ek yükü olmadan düşürülür (heap serbest bırakma elbette çalışma anında olur)”.
- **Önem:** orta (aynı dosyayla iç çelişki).

### 3.9. Borrowck kaynak yolları (borrow_set / places_conflict / nll / region_infer) — DOĞRU

- **Yol/satır:** `learn/ownership/src/main.rs:204-207` ve `306-309`
- **Alıntı:** `// rustc: borrow checker MIR üzerinde çalışır; borç kümelerini (rustc_borrowck/ // borrow_set.rs) ve çakışmayı (places_conflict.rs) hesaplar, ömürleri NLL ile // çıkarır (nll.rs, region_infer/). "Last use"tan sonra borç biter — r1/r2 sonrası // r3'ün serbest olmasının nedeni budur (NLL = non-lexical lifetimes).`
- **Hüküm:** DOĞRU
- **Kaynak:** `guide/rust/compiler/rustc_borrowck/src/borrow_set.rs` (587 satır), `places_conflict.rs` (544 satır), `nll.rs` (372 satır), `region_infer/` dizini (mod.rs, region_context.rs, vb.) — tümü mevcut. NLL “son kullanım” davranışı yeniden üretimle uyumlu: `/tmp/opencode/verify/e0502.rs` (r1/r2 hâlâ kullanılırken &mut → E0502) derleniyor; main.rs’teki sıralama (println! Sonrası r3) derlenir.
- **Düzeltme:** Yok.
- **Önem:** küçük.

### 3.10. E0106 / E0515 kodları ve E0515 dosya yolu — DOĞRU

- **Yol/satır:** `learn/ownership/src/main.rs:228`, `233`, `243-245`
- **Alıntı:** `// fn sarkan() -> &String {          // E0106: missing lifetime specifier` / `// fn sarkan<'a>() -> &'a String {   // E0515: cannot return reference to local variable 's'` / `// rustc: borrowck, döndürülen referansın ömrünün fonksiyondan uzun olması gerektiğini // ama yerelin ömrünün fonksiyonla bittiğini görür → E0515 // (rustc_borrowck/src/borrowck_errors.rs, "return reference to local").`
- **Hüküm:** DOĞRU
- **Kaynak:** Yeniden üretim `/tmp/opencode/verify/e0106.rs` ve `/tmp/opencode/verify/e0515.rs` (yukarıdaki birebir mesajlar); `guide/rust/compiler/rustc_borrowck/src/borrowck_errors.rs:432-452` — `path_does_not_live_long_enough()` E0597, `cannot_return_reference_to_local()` E0515.
- **Düzeltme:** Yok. Not: mesajın gerçek metni “cannot return reference to local variable `s`” olduğu için alıntı doğru.
- **Önem:** küçük.

### 3.11. Dilim + `s.clear()` → E0502 “DOĞRULANDI” — DOĞRU

- **Yol/satır:** `learn/ownership/src/main.rs:274-278`
- **Alıntı:** `//   let w = ilk_kelime(&s); // w hâlâ &s borç'u tutuyor //   s.clear();            // ✗ E0502 (DOĞRULANDI): w hâlâ &s borç'u tutuyor`
- **Hüküm:** DOĞRU
- **Kaynak:** Yeniden üretim `/tmp/opencode/verify/slice_clear.rs` → `error[E0502]: cannot borrow 's' as mutable because it is also borrowed as immutable`. `&String` → `&str` dönüşümü için deref coercion: https://doc.rust-lang.org/reference/coercion.html (yorum 281. satır).
- **Düzeltme:** Yok.
- **Önem:** küçük.

### 3.12. NLL “son kullanımda biter” genel anlatımı — DOĞRU

- **Yol/satır:** `learn/ownership/src/main.rs:315-316`
- **Alıntı:** `// • NLL (non-lexical lifetimes): bir borç, SÖZDİZİMSEL blok sonunda değil, SON KULLANIMINDA // biter. borrow_mut()'taki r1/r2'den sonra r3'ün (&mut) serbest olması bu yüzdendir.`
- **Hüküm:** DOĞRU
- **Kaynak:** 3.9’daki kaynaklar; https://doc.rust-lang.org/nomicon/borrow-splitting.html ve NLL açıklaması RFC 2094 (https://rust-lang.github.io/rfcs/2094-nll.html) — loan’lar region inference ile son kullanıma kadar uzatılır. Verilen örnekte r1/r2’nin son kullanımı `println!` satırı, r3 ondan sonra gelir; derlenir.
- **Düzeltme:** Yok.
- **Önem:** küçük.

---

## 4. `learn/ownership/src/derinlesme.rs`

### 4.1. Beş hata kodunun “rustc 1.99 ile doğrulandığı” iddiası — DOĞRU

- **Yol/satır:** `learn/ownership/src/derinlesme.rs:9-11`
- **Alıntı:** `// - borrow checker'ın en sık verdiği beş hata (E0382, E0499, E0502, E0505, E0507) hangi // alışkanlıktan doğar ve her birinin deyimsel (idiomatic) çözümü nedir. // Buradaki ✗ örneklerin hata kodları da rustc 1.99 ile derlenerek doğrulandı.`
- **Hüküm:** DOĞRU
- **Kaynak:** Yeniden üretimler (rustc 1.99.0): `/tmp/opencode/verify/e0382_partial.rs` → `E0382: use of partially moved value: 'p'`; `/tmp/opencode/verify/e0499.rs` → `E0499: cannot borrow 'puanlar[_]' as mutable more than once at a time`; `/tmp/opencode/verify/e0502.rs` → E0502; `/tmp/opencode/verify/e0505.rs` → `E0505: cannot move out of 'baslik' because it is borrowed`; `/tmp/opencode/verify/e0507.rs` → `E0507: cannot move out of index of 'Vec<String>'`; `/tmp/opencode/verify/e0507_field.rs` → `E0507: cannot move out of 'k.ad' which is behind a shared reference`.
- **Düzeltme:** Yok.
- **Önem:** küçük.

### 4.2. Alıştırma 1 — `max_by_key` eşitlikte sonuncuyu verir — DOĞRU

- **Yol/satır:** `learn/ownership/src/derinlesme.rs:461-463`
- **Alıntı:** `//    Beklenen: Ada/Linus/Grace (hepsi aktif) için Some("Linus") ya da Some("Grace") — beşer //    harf; max_by_key eşitlikte SONUNCUYU verir, yani Some("Grace").`
- **Hüküm:** DOĞRU
- **Kaynak:** `guide/rust/library/core/src/iter/traits/iterator.rs:3340-3342` — `/// If several elements are equally maximum, the last element is returned.` (Ada=3, Linus=5, Grace=5 harf.)
- **Düzeltme:** Yok.
- **Önem:** küçük.

### 4.3. Alıştırma 3 — `liste[0]` için E0507 — DOĞRU

- **Yol/satır:** `learn/ownership/src/derinlesme.rs:471-472`
- **Alıntı:** `//        fn ilk_iki(v: &Vec<String>) -> (String, String) { (v[0], v[1]) } //    İpucu: E0507. Seçenekler: (&str, &str) döndürmek ya da clone'lamak.`
- **Hüküm:** DOĞRU
- **Kaynak:** Yeniden üretim `/tmp/opencode/verify/e0507.rs` → `error[E0507]: cannot move out of index of 'Vec<String>'`.
- **Düzeltme:** Yok.
- **Önem:** küçük.

---

## 5. `ACIKLAMA.md` dosyaları

### 5.1. common-concepts benzetmesi: “değişmezlik = kuralın ta kendisi” — YANILTICI

- **Yol/satır:** `learn/common-concepts/ACIKLAMA.md:5`
- **Alıntı:** `Değişmezlik, iki kişinin aynı defteri aynı anda boyamasını engelleyen kuralın (Aliasing XOR Mutability) ta kendisidir.`
- **Hüküm:** YANILTICI
- **Kaynak:** Aynı dersin kaynak dosyası `learn/common-concepts/src/main.rs:395` bunu doğru biçimde şöyle diyor: `// "Varsayılan immutable" işte bu kuralın yüzeye yansımasıdır.` Yani değişmezlik kuralın kendisi değil, yüzeye yansımasıdır. Kuralın özü referanslardır: https://doc.rust-lang.org/nomicon/races.html — “it's impossible to alias a mutable reference, so it's impossible to perform a data race.” Ayrıca bir `let` değişkenini `mut` yapmak tek başına aliasing’i engellemez; birden çok `&T` aynı veriyi ödünç alabilir.
- **Düzeltme:** “Değişmezlik, bu kuralın (Aliasing XOR Mutability) yüzeye yansımasıdır; kuralın kendisi referansların paylaşım/değiştirme kurallarıdır.”
- **Önem:** orta (kavramsal olarak kuralın yerini yanlış gösteriyor; kaynak dosyayla çelişiyor).

### 5.2. ownership benzetmeleri — DOĞRU

- **Yol/satır:** `learn/ownership/ACIKLAMA.md:5`, `:9`
- **Alıntı:** `Her verinin tek bir sahibi (owner) vardır... ya birçok bakan (&) olur, ya da tek bir boyayan (&mut); ikisi aynı anda olmaz... Dilim (slice) pastanın bir parçasını gösterir, kopyası değildir; pasta gidince dilim de gider.`
- **Hüküm:** DOĞRU
- **Kaynak:** https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html (sahiplik, borrowing, slices); https://doc.rust-lang.org/nomicon/races.html (& alias / &mut uniqueness); `learn/ownership/src/main.rs:252` (slice = pointer + uzunluk, kopya değil) ve `:274-278` (E0502 ile doğrulandı).
- **Düzeltme:** Yok. Benzetmeler teknik olarak yanlış bir şey ima etmiyor.
- **Önem:** küçük.

### 5.3. basic-concepts benzetmesi — DOĞRU

- **Yol/satır:** `learn/basic-concepts/ACIKLAMA.md:5`, `:9`
- **Alıntı:** `Varsayılan kalem tükenmez gibidir: yazdığını silemezsin; mut dersen kurşun kaleme geçersin.`
- **Hüküm:** DOĞRU
- **Kaynak:** https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html — variables are immutable by default, `mut` makes them mutable.
- **Düzeltme:** Yok.
- **Önem:** küçük.

### 5.4. Üç dosyadaki ortak “Not” satırı — teknik iddia içermiyor

- **Yol/satır:** `learn/{common-concepts,basic-concepts,ownership}/ACIKLAMA.md:11`
- **Alıntı:** `> Not: Kod mantığı değiştirilmedi. Değişiklikleri 'cargo check ile doğrula'.`
- **Hüküm:** DOĞRULANAMADI (meta-talimat; doğrulanabilir teknik bir iddia değil. Bu denetimde “Çalıştır:” komutları çalıştırılmadı.)
- **Kaynak:** —
- **Düzeltme:** Yok.
- **Önem:** küçük.

---

## 6. Sorunsuz görünen alanlar (kısa liste)

Aşağıdaki iddialar ya doğrudan yeniden üretimle ya da birincil kaynakla doğrulandı; ayrı bulgu yazılmadı:

- **common-concepts/main.rs:** varsayılan immutable + `mut` (E0384), `const` tür zorunluluğu/derleme-anı hesap, shadowing’in her `let` için ayrı MIR local üretmesi (`-Zunpretty=mir` çıktısında `_2: &str` ve `_1: usize` ayrı local’ler), i32/f64/char (Unicode scalar, 4 byte) açıklamaları, debug’da panik / release’de wrap, ifade-vs-deyim ve `;` ile `()` dönüşü (E0308), if dallarının aynı türde olması, `loop` + `break value`, `1..4` = 1,2,3, rustc borusu hattındaki crate adlarının tümünün `guide/rust/compiler/` altında gerçekten bulunması.
- **basic-concepts/main.rs:** raw pointer/unsafe açıklaması (`*const i32`, geçerli+hizalı+başlatılmış garantisi), `ptr::read`’in i32 için sorunsuz kullanımı, associated type vs generic parametre gerekçesi, inherent impl / fn pointer ayrımı, orphan rule örnekleri ve E0119 gerekçesi, `!`-negative impl açıklaması.
- **ownership/main.rs:** üç ownership kuralı, `clone` açıklaması, RAII’nin GC’ye/elle `free()`’a üstünlüğü (3.8’deki “sıfır maliyet” düzeltmesi hariç), `&T`/`&mut T` kuralı ve E0502 örneği, dangling reference önizleme gerekçesi, dilim = pointer+uzunluk ve `deref coercion` notu, derleyici perspektifindeki tüm dosya/crate adları.
- **ownership/derinlesme.rs:** imza seçimi (`String`/`&str`/`&mut String`), kısmi move ve alan düzeyinde izleme, `KullaniciDeposu` imzaları ve lifetime elision notu, beş klasik hata + deyimsel çözümleri, `split_at_mut`, `mem::take`/`replace`/`Option::take`, çalışma zamanı çıktı yorumları (`[2, 3, 20]`, `[20, 10, 30]`, `["üç", "iki"]` — koddan elle doğrulandı), C++ iterator invalidation = UB benzetmesi.
- **“2024 edition” iddiası:** Bu üç klasörde hiçbir 2024 edition atfı yok (tarama: yalnız `learn/async-await`, `learn/closures-iterators`, `learn/generics-traits-lifetimes` altında geçiyor; onlar bu denetimin kapsamı dışında).

## 7. Kesin kanıt bulunamayan / kapsam dışı kalan noktalar

- `ACIKLAMA.md` dosyalarındaki “Kod mantığı değiştirilmedi” ifadesi, git geçmişi/diff incelenmediği için doğrulanamadı (yalnızca mevcut dosyalar okundu).
- `basic-concepts/src/main.rs:170-173`’teki `#[proc_macro]` / `.parser()` satırları **kod** olduğu için bu denetimin kapsamı dışında bırakıldı; yorumlarda bunların derlendiğine dair bir iddia yok. (Not: bir binary crate içinde `proc_macro` attribute’u ve `syn` import’u olmadan bu satırların derlenmeyeceği gözlemlenmiştir; ancak “Çalıştır:” komutlarını çalıştırma kısıtı gereği `cargo check` yapılmadı.)
- Rust sürümü: yerel `guide/rust` ağacı 1.99.1, yüklü derleyici 1.99.0. Kaynak satır numaraları 1.99.1 için geçerlidir; 1.99.0 ile davranış (hata kodları) yeniden üretimle doğrulandı.
