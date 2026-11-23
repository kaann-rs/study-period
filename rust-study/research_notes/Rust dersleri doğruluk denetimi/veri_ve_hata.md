# Rust dersleri doğruluk denetimi — veri yapıları ve hata yönetimi

**Denetim tarihi:** 2026-11-23
**Derleyici:** `rustc 1.99.0 (b940084d7 2026-09-28)`
**Kapsam:** `learn/structs`, `learn/enums`, `learn/collections`, `learn/error-handling` altındaki 4 `ACIKLAMA.md` + 8 `.rs` dosyası.
**Yöntem:** Yerel Rust ağacı `guide/rust` (kaynak yol + satır), resmî `doc.rust-lang.org/book/print.html` (Rust 1.98 / edition 2024 metni), `rustc --explain` ve yalnızca `/tmp/opencode/rust_verify/` altında derlenen mini repro'lar. `learn/` altında hiçbir dosya değiştirilmedi; "Çalıştır:" satırları komut olarak çalıştırılmadı, veri olarak okundu.

## Özet

| Dosya | Doğru | Sorunlu | Not |
|---|---|---|---|
| `structs/src/main.rs` | 11 | 2 | `derive` proc-macro çelişkisi; "Copy = küçük değer" genellemesi |
| `structs/src/derinlesme.rs` | 8 | 0 | Hata kodlarının tamamı repro ile doğrulandı |
| `structs/ACIKLAMA.md` | 2 | 0 | Benzetmeler tutarlı |
| `enums/src/main.rs` | 8 | 2 | `if let` lowering metni yanlış (düzeltme notu doğru); niche açıklaması belirsiz |
| `enums/src/derinlesme.rs` | 9 | 0 | E-family kodlarının tamamı doğru |
| `enums/ACIKLAMA.md` | 2 | 0 | Postacı benzetmesi tutarlı |
| `collections/src/main.rs` | 9 | 1 | "Copy = küçük değer" genellemesi (structs ile aynı) |
| `collections/ACIKLAMA.md` | 1 | 1 | "bayt değil karakter sınırı" ifadesi yanıltıcı |
| `error-handling/src/main.rs` | 9 | 0 | `Option` must_use düzeltme notu doğrulandı |
| `error-handling/src/derinlesme.rs` | 10 | 0 | "Böl. 18" iddiası **doğru** (güncel kitapta trait object Böl. 18) |
| `error-handling/ACIKLAMA.md` | 2 | 0 | Benzetme, boş satır/boş kart ayrımıyla uyumlu |

---

## 1. "Derlenerek doğrulandı" iddialarının denetimi

### `learn/structs/src/main.rs:6`
- **Alıntı:** `// ✗ örneklerin hata kodları (E0106, E0382) rustc ile DERLENEREK doğrulandı.`
- **Hükm:** DOĞRU
- **Kaynak:** `/tmp/opencode/rust_verify/e0106.rs` → `error[E0106]: missing lifetime specifier`; `/tmp/opencode/rust_verify/self_receiver.rs` → `error[E0382]: borrow of moved value: 'd'`; `/tmp/opencode/rust_verify/partial_move.rs` → `error[E0382]: borrow of moved value: 'u1.eposta'`.
- **Düzeltme:** gerek yok.
- **Önem:** Düşük.

### `learn/enums/src/main.rs:6-7`
- **Alıntı:** `// Doğrulanmış (rustc ile derlenerek): non-exhaustive match → E0004; Option'ın niche optimization boyutları.`
- **Hükm:** DOĞRU
- **Kaynak:** `/tmp/opencode/rust_verify/e0004.rs` → `error[E0004]: non-exhaustive patterns: 'Yon::Dogu' and 'Yon::Bati' not covered`; `/tmp/opencode/rust_verify/sizes2.rs` → `i32=4 Option<i32>=8`, `&i32=8 Option<&i32>=8`, `Box<i32>=8 Option<Box<i32>>=8`, `bool=1 Option<bool>=1`.
- **Düzeltme:** gerek yok.
- **Önem:** Düşük.

### `learn/collections/src/main.rs:8-10`
- **Alıntı:** `// Doğrulanmış (rustc ile): Vec borrow+push → E0502; String integer index → E0277; HashMap insert key taşıma → E0382; char sınırı dışı dilim → çalışma anı panik; "şema" = 5 byte / 4 karakter.`
- **Hükm:** DOĞRU
- **Kaynak:** `/tmp/opencode/rust_verify/e0502.rs` → `error[E0502]: cannot borrow 'v' as mutable because it is also borrowed as immutable`; `e0277index.rs` → `error[E0277]: the type 'str' cannot be indexed by '{integer}'`; `partial_move.rs` (aynı mekanik) → E0382; `slicepanic.rs` → `byte index 1 is not a char boundary; it is inside 'ş' (bytes 0..2)`; `schema.rs` → `len=5 chars=4`.
- **Düzeltme:** gerek yok.
- **Önem:** Düşük.

### `learn/error-handling/src/main.rs:8-9`
- **Alıntı:** `// Doğrulanmış (rustc ile): kullanılmayan Result → "unused 'Result' that must be used" uyarısı; '?' operatörü Result/Option dönmeyen fn'de → E0277.`
- **Hükm:** DOĞRU
- **Kaynak:** `/tmp/opencode/rust_verify/result_must_use.rs` → `warning: unused 'Result' that must be used`; `e0277q.rs` → `error[E0277]: the '?' operator can only be used in a function that returns 'Result' or 'Option' (or another type that implements 'FromResidual')`.
- **Düzeltme:** gerek yok.
- **Önem:** Düşük.

### `learn/error-handling/src/derinlesme.rs:528`
- **Alıntı:** `// 7. SIK YAPILAN HATALAR — yanlış kullanım / doğru kullanım (kodlar rustc ile doğrulandı)`
- **Hükm:** DOĞRU
- **Kaynak:** `/tmp/opencode/rust_verify/e0277q.rs` (a) → E0277 yukarıdaki mesaj; `e0277conv.rs` (b, hem `parse::<i32>()` hem `let n: i32 = s.parse()` biçimi) → `error[E0277]: '?' couldn't convert the error to 'AyarHatasi'` + `the trait 'From<ParseIntError>' is not implemented`; `e0277mix.rs` (c) → sırasıyla `error[E0277]: the '?' operator can only be used on 'Option's, not 'Result's, in a function that returns 'Option'` ve tersi.
- **Düzeltme:** gerek yok. Metindeki üç hata mesajı da birebir doğru.
- **Önem:** Düşük.

### `learn/structs/src/main.rs:141-143`
- **Alıntı:** `// Not (doğrulanmış): bir alanı yalnızca {:?} (derive(Debug)) ile basmak, dead_code analizinde "okundu" SAYILMAZ — rustc derive'ın ürettiği impl'i bu analizde kasıtlı yok sayar.`
- **Hükm:** DOĞRU
- **Kaynak:** `/tmp/opencode/rust_verify/dead_code.rs` → `warning: fields 'ad' and 'gizli' are never read` + `note: 'Kart' has a derived impl for the trait 'Debug', but this is intentionally ignored during dead code analysis`.
- **Düzeltme:** gerek yok.
- **Önem:** Düşük.

### `learn/collections/src/main.rs:141-142`
- **Alıntı:** `// Payload'ları GERÇEKTEN okuyalım: yoksa (Bölüm 5'teki doğrulanmış tuzak) derive(Debug) dead_code analizinde yok sayıldığı için "field never read" uyarısı çıkar.`
- **Hükm:** DOĞRU
- **Kaynak:** aynı repro (`dead_code.rs`); yukarıdaki maddedeki rustc notu.
- **Düzeltme:** gerek yok.
- **Önem:** Düşük.

### `learn/error-handling/src/main.rs:113-114`
- **Alıntı:** `// Not (doğrulanmış): Result #[must_use]'tır. 'ikiye_bol(10, 2);' deyip sonucu yok sayarsan "unused 'Result' that must be used" UYARISI alırsın.`
- **Hükm:** DOĞRU
- **Kaynak:** `guide/rust/library/core/src/result.rs:555` → `#[must_use = "this 'Result' may be an 'Err' variant, which should be handled"]`; repro `result_must_use.rs`.
- **Düzeltme:** gerek yok.
- **Önem:** Düşük.

---

## 2. Derleyici / kaynak yolu iddiaları

### `learn/structs/src/main.rs:309-312`
- **Alıntı:** `// İlk uyan metodu seçer. Kaynak: rustc_hir_typeck/src/method/probe.rs (adayları toplar) + confirm.rs (seçimi kesinleştirir).`
- **Hükm:** DOĞRU
- **Kaynak:** `guide/rust/compiler/rustc_hir_typeck/src/method/mod.rs:3-7` (dev-guide method-lookup bağlantısı, `pub(crate) mod confirm; mod probe;`); `probe.rs:336` `probe_for_name`, `:885` `assemble_probe`; `confirm.rs:60` `confirm_method`, `:112` `confirm`.
- **Düzeltme:** gerek yok.
- **Önem:** Düşük.

### `learn/structs/src/main.rs:315-318`
- **Alıntı:** `// Sabit, C-uyumlu sıra gerekiyorsa '#[repr(C)]' kullanılır. Hesaplama: rustc_abi/src/layout.rs (satır ~35 yorumu: sıra yalnız repr(C) struct'larda korunur).`
- **Hükm:** DOĞRU
- **Kaynak:** `guide/rust/compiler/rustc_abi/src/layout.rs:34` → `/// Particularly for 'repr(Rust)' types, this may not be the same as *layout* order. (It is for 'repr(C)' 'struct's, however.)` — "~35" doğru.
- **Düzeltme:** gerek yok.
- **Önem:** Düşük.

### `learn/structs/src/main.rs:229` ve `learn/collections/src/main.rs:266`
- **Alıntı:** `// rustc: lifetime'lar da tür denetiminin parçası; ihlali MIR üzerinde rustc_borrowck yakalar.` / `// zorlayan derleyicidir: borrow checker (rustc_borrowck), E0502.`
- **Hükm:** DOĞRU
- **Kaynak:** `guide/rust/compiler/rustc_borrowck/src/lib.rs:1` → `//! This crate implemens MIR typeck and MIR borrowck.`; `:109` `mir_borrowck` provider'ı; `:115` `fn mir_borrowck`.
- **Düzeltme:** gerek yok (yazım hatası "implemens" kaynakta, derste değil).
- **Önem:** Düşük.

### `learn/enums/src/main.rs:310-314`
- **Alıntı:** `// • Exhaustiveness denetimi: match, tür denetiminden sonra THIR (typed HIR) üzerinde kontrol edilir. Eksik varyant → E0004, kaynak rustc_mir_build/src/thir/pattern/check_match.rs:1281 civarı. Altındaki "usefulness" ... ayrı bir crate'tedir: rustc_pattern_analysis.`
- **Hükm:** DOĞRU
- **Kaynak:** `guide/rust/compiler/rustc_mir_build/src/thir/pattern/check_match.rs:1278-1284` → `E0004, "non-exhaustive patterns: {joined_patterns} not covered"` (tam 1281 civarı); `:17-18, 403, 594` → `use rustc_pattern_analysis::...`, `rustc_pattern_analysis::rustc::analyze_match(...)`; çağrı `rustc_mir_build/src/builder/mod.rs:75` → `tcx.ensure_result().check_match(def)`.
- **Düzeltme:** gerek yok. ("1281 civarı" ifadesi isabetli; şüphe üzerine kontrol edildi, hata yok.)
- **Önem:** Düşük.

### `learn/error-handling/src/main.rs:225-232`
- **Alıntı:** `// • ? operatörünün indirgenmesi (lowering): AST → HIR aşamasında (rustc_ast_lowering/src/expr.rs, fn lower_expr_try, DesugaringKind::QuestionMark) kabaca şuna açılır: ... ControlFlow::Continue(v) => v, ... ControlFlow::Break(residual) => return FromResidual::from_residual(residual)`
- **Hükm:** DOĞRU
- **Kaynak:** `guide/rust/compiler/rustc_ast_lowering/src/expr.rs:1935-1953` → doc comment `Desugar 'ExprKind::Try' from: '<expr>?' into: match Try::branch(<expr>) { ControlFlow::Continue(val) => val, ControlFlow::Break(residual) => ... return FromResidual::from_residual(residual), }` ve `:1954` `fn lower_expr_try` + `DesugaringKind::QuestionMark`.
- **Düzeltme:** gerek yok.
- **Önem:** Düşük.

### `learn/structs/src/main.rs:286-288` ve `:320` — iç çelişki
- **Alıntı (doğru olan):** `// ↳ DÜZELTME NOTU (son gözden geçirme): Debug/Clone/PartialEq gibi STANDART derive'lar derleyiciye gömülü (built-in) makrolardır; ayrı bir proc-macro crate'i değildir.`
- **Alıntı (sorunlu):** `// • derive(Debug): bir procedural macro; kaynağa fmt::Debug impl'ini makro genişletme aşamasında enjekte eder (sen yazmazsın).`
- **Hükm:** YANILTICI (dosya içi çelişki). 286-288 notu DOĞRU; 320. satır onu geçersiz kılıyor.
- **Kaynak:** `guide/rust/compiler/rustc_builtin_macros/src/deriving/` (derleyici içi dizin, ayrı proc-macro crate'i değil); içsel kanıt: aynı dosyadaki düzeltme notu.
- **Düzeltme:** 320. satırı şu şekilde değiştirin: `// • derive(Debug): derleyiciye gömülü (built-in) bir derive makrosu; kaynağa fmt::Debug impl'ini makro genişletme aşamasında enjekte eder. Kullanıcı derive'ları (serde vb.) gerçek proc-macro crate'leridir.` Ayrıca 285. satırdaki `derive bir PROC-MACRO'dur:` ifadesi de notla çelişiyor; orayı da `derive bir makrodur:` yapın.
- **Önem:** Orta (öğrenen için kavramsal çelişki).

### `learn/enums/src/main.rs:317-321` — if-let lowering
- **Alıntı (sorunlu gövde):** `// • if let / while let / let-else: sözdizimsel şekerdir; derleyici bunları eşdeğer bir match'e indirger (lowering).`
- **Alıntı (doğru düzeltme notu):** `// ↳ DÜZELTME NOTU (son gözden geçirme): ANLAMCA match'e eşdeğerdirler; ancak güncel rustc bunları HIR'da match'e açmaz, 'Let' / 'If' / let-else düğümleri olarak korur. (for → loop + match ve '?' → match indirgemeleri ise gerçekten yapılır.)`
- **Hükm:** YANILTICI (gövde), DOĞRU (düzeltme notu). Notun kendisi tamamen isabetli.
- **Kaynak:** `guide/rust/compiler/rustc_ast_lowering/src/expr.rs:324-325` → `ExprKind::Let(pat, scrutinee, span, recovered) => hir::ExprKind::Let(self.arena.alloc(hir::LetExpr {...}))`; `guide/rust/compiler/rustc_hir/src/hir.rs:1895` `pub struct LetExpr`; aynı dosya `:1787-1864` → `DesugaringKind::ForLoop` + `hir::ExprKind::Loop(..., match_expr, ...)` (for gerçekten açılır); `:1935-1953` → `?` gerçekten match'e açılır.
- **Düzeltme:** 317-318. satırları doğrudan düzeltme notunun içeriğiyle değiştirip "DÜZELTME NOTU" satırlarını kaldırın; böylece tek bir doğru ifade kalır.
- **Önem:** Orta (derleyici perspektifi bölümü yanlış bilgi veriyor).

---

## 3. Dil ve kütüphane davranışı iddiaları

### `learn/structs/src/main.rs:174-177`
- **Alıntı:** `// DİKKAT: '..u1' bir KOPYA değildir. Kalan alanlardan non-Copy olanları (eposta: String) u2'ye TAŞIR; u1.eposta artık geçersizdir (u1 kısmen taşınmış sayılır)`
- **Hükm:** DOĞRU
- **Kaynak:** `/tmp/opencode/rust_verify/partial_move.rs` → `error[E0382]: borrow of moved value: 'u1.eposta'`, `move occurs because 'u1.eposta' has type 'String', which does not implement the 'Copy' trait`; aynı dosyada `u1.aktif`/`u1.giris` (Copy) okunabiliyor → 177. satır da doğru.
- **Düzeltme:** gerek yok.
- **Önem:** Düşük.

### `learn/structs/src/derinlesme.rs:422-424`
- **Alıntı:** `//   println!("{taban:?}"); // ✗ E0382: borrow of partially moved value: 'taban'`
- **Hükm:** DOĞRU
- **Kaynak:** `/tmp/opencode/rust_verify/partial_move_debug.rs` → `error[E0382]: borrow of partially moved value: 'taban'`, `partial move occurs because 'taban.dil' has type 'String'`.
- **Düzeltme:** gerek yok.
- **Önem:** Düşük.

### `learn/structs/src/derinlesme.rs:262-265`
- **Alıntı:** `//   let e = hesap::Eposta(String::from("asdf")); // ✗ E0603: tuple struct constructor 'Eposta' is private` / `//   println!("{}", e.0); // ✗ E0616: field '0' of struct 'Eposta' is private` / `// ✗ E0451: fields .. of struct 'Kullanici' are private`
- **Hükm:** DOĞRU (üçü de)
- **Kaynak:** `/tmp/opencode/rust_verify/privacy.rs` → `error[E0603]: tuple struct constructor 'Eposta' is private`, `error[E0616]: field '0' of struct 'Eposta' is private`; `/tmp/opencode/rust_verify/e0451.rs` → `error[E0451]: fields 'eposta', 'yonetici' and 'giris_sayisi' of struct 'Kullanici' are private`.
- **Düzeltme:** gerek yok.
- **Önem:** Düşük.

### `learn/structs/src/derinlesme.rs:308-317`
- **Alıntı:** `//                                // ✗ E0507: cannot move out of a mutable reference` / `//   let k2 = b.olustur();        // ✗ E0382: use of moved value: 'b'`
- **Hükm:** DOĞRU (ikisi de)
- **Kaynak:** `/tmp/opencode/rust_verify/e0507b.rs` → `error[E0507]: cannot move out of a mutable reference` (`olustur(self)` alındığı için); `self_receiver.rs` mantığıyla ikinci çağrı → E0382.
- **Düzeltme:** gerek yok.
- **Önem:** Düşük.

### `learn/structs/src/derinlesme.rs:344-345`, `:357`, `:366-367`
- **Alıntı:** `//   let ada = ..; ada.giris_kaydet(); // ✗ E0596: cannot borrow 'ada' as mutable, as it is not declared as mutable` / `//   ada.giris_kaydet(); // ✗ E0382: borrow of moved value: 'ada'` / `// ✗ E0507: cannot move out of '*k' which is behind a shared reference`
- **Hükm:** DOĞRU
- **Kaynak:** `rustc --explain E0596/E0382/E0507` ve `self_receiver.rs` (`self` receiver'ın taşıdığını gösteren repro); E0507 mesajı `e0507b.rs` ile aynı aile.
- **Düzeltme:** gerek yok.
- **Önem:** Düşük.

### `learn/structs/src/derinlesme.rs:455-462`
- **Alıntı:** `//   let a = Ayarlar { tema: String::from("koyu") }; // ✗ E0063: missing fields 'bildirim', 'dil' and 'sayfa_boyutu' in initializer` / `// dışında 'Kullanici { ad: .., ..k }' ile güncellenemez (✗ E0451: ...)`
- **Hükm:** DOĞRU
- **Kaynak:** `/tmp/opencode/rust_verify/e0063.rs` → mesaj birebir `error[E0063]: missing fields 'bildirim', 'dil' and 'sayfa_boyutu' in initializer of 'Ayar'`; `e0451update.rs` → E0451 ailesi; `rustc --explain E0451`.
- **Düzeltme:** gerek yok.
- **Önem:** Düşük.

### `learn/enums/src/main.rs:301-307` (niche optimization ölçümleri)
- **Alıntı:** `//       i32        = 4,  Option<i32>        = 8   ... //       bool       = 1,  Option<bool>       = 1`
- **Hükm:** DOĞRU (ölçüm değerleri); açıklama kısmı için aşağıya bakın.
- **Kaynak:** `/tmp/opencode/rust_verify/sizes2.rs` çıktısı: `i32=4 Option<i32>=8`, `&i32=8 Option<&i32>=8`, `Box<i32>=8 Option<Box<i32>>=8`, `bool=1 Option<bool>=1` (x86_64, rustc 1.99.0).
- **Düzeltme:** gerek yok.
- **Önem:** Düşük.

### `learn/enums/src/main.rs:306` — niche açıklaması
- **Alıntı:** `//       bool       = 1,  Option<bool>       = 1   (bool yalnız 0/1 → 2 deseni None olur)`
- **Hükm:** YANILTICI (belirsiz ifade; ölçümün kendisi DOĞRU)
- **Kaynak:** `/tmp/opencode/rust_verify/sizes2.rs` (`bool=1 Option<bool>=1`); `guide/rust/compiler/rustc_abi/src/lib.rs:2458` → `// 'is_bool' checks validity and is ABI-relevant...`. bool 1 byte'ta yalnız 0 ve 1 değerlerini kullanır; geri kalan **254** bit deseni geçersizdir ve bunlardan biri `None` için kullanılır. "2 deseni None olur" okunuşu "yalnız 2 desen None olur" anlamına geliyor.
- **Düzeltme:** `(bool 1 byte'ta yalnız 0/1 değerlerini kullanır; 254 geçersiz desenden biri None olur)`.
- **Önem:** Düşük.

### `learn/enums/src/derinlesme.rs:195-201`
- **Alıntı:** `// matches!: "şu varyantlardan biri mi?" sorusunu bool'a çevirir (içeride '_ => false').`
- **Hükm:** DOĞRU
- **Kaynak:** `guide/rust/library/core/src/macros/mod.rs:431-438` → `macro_rules! matches { ... match $expression { $pattern ... => true, _ => false } }`.
- **Düzeltme:** gerek yok.
- **Önem:** Düşük.

### `learn/enums/src/derinlesme.rs:314-317`
- **Alıntı:** `// Çarpma u64'te: büyük tutarlarda 'tutar * oran' u32'ye sığmaz (debug'da panik, release'de sessiz sarma).`
- **Hükm:** DOĞRU
- **Kaynak:** `doc.rust-lang.org/book/print.html` ("Overflow" bölümü: debug build'lerde overflow check panikler, release'de iki's complement wrapping yapılır); `guide/rust/library/core/src/...` taşma denetimi profil ayarıyla yönetilir. Ayrıca `enums/src/derinlesme.rs:488-492` testi bu davranışı u64'e çıkararak doğruluyor.
- **Düzeltme:** gerek yok.
- **Önem:** Düşük.

### `learn/enums/src/derinlesme.rs:388-447`
- **Alıntı:** `// ✗ E0308: mismatched types (expected 'u32', found 'Option<u32>')` ... `// ✗ E0369: cannot add '{integer}' to 'Option<u32>'` ... `// ✗ E0277: the '?' operator can only be used on 'Option's, not 'Result's...` ... `// ✗ E0507: cannot move out of 'p.takma_ad' which is behind a shared reference` ... `// ✗ E0382: borrow of partially moved value: 'durum'` ... `// ✗ E0023: this pattern has 1 field, but the corresponding tuple variant has 2 fields`
- **Hükm:** DOĞRU (altısının tamamı)
- **Kaynak:** `/tmp/opencode/rust_verify/e0369.rs` → mesaj birebir; `e0277mix.rs` → mesajlar birebir; `e0023.rs` → `error[E0023]: this pattern has 1 field, but the corresponding tuple variant has 2 fields`; `partial_move_debug.rs` → E0382 "partially moved"; E0308/E0507 için `rustc --explain E0308/E0507` ve yukarıdaki E0507 repro'ları.
- **Düzeltme:** gerek yok.
- **Önem:** Düşük.

### `learn/collections/src/main.rs:116-123`
- **Alıntı:** `//   v.push(40);          // E0502: push &mut ister, ama 'ilk' hâlâ & borç tutuyor` / `// (realloc): yeni blok ayrılır, eski serbest bırakılır. 'ilk' eski bloğu gösterdiği için dangling olurdu.`
- **Hükm:** DOĞRU
- **Kaynak:** `/tmp/opencode/rust_verify/e0502.rs` → `error[E0502]: cannot borrow 'v' as mutable because it is also borrowed as immutable`; `guide/rust/library/alloc/src/vec/mod.rs` (Vec = RawVec + len; büyüme yeniden tahsis eder) ve `guide/rust/compiler/rustc_borrowck/src/lib.rs:1`.
- **Düzeltme:** gerek yok.
- **Önem:** Düşük.

### `learn/collections/src/main.rs:179-191`
- **Alıntı:** `// UTF-8 gerçeği: "şema" 5 BYTE ama 4 KARAKTER ('ş' 2 byte tutar).` / `//   &t[0..1]  // çalışma anı PANİK: "byte index 1 is not a char boundary" ('ş' 0..2 arası)`
- **Hükm:** DOĞRU
- **Kaynak:** `/tmp/opencode/rust_verify/schema.rs` → `len=5 chars=4 first=Some('ş')`; `slicepanic.rs` → `byte index 1 is not a char boundary; it is inside 'ş' (bytes 0..2) at index 1 in 'şema'`.
- **Düzeltme:** gerek yok.
- **Önem:** Düşük.

### `learn/collections/src/main.rs:262-280`
- **Alıntı:** `// • String = içinde geçerli UTF-8 olduğu GARANTİ edilen bir Vec<u8>. 'Index<usize>' IMPL EDİLMEMİŞTİR; bu yüzden s[0] bir tür hatasıdır (E0277), çalışma anı kontrolü değil.` / `// • HashMap<K, V>: ... varsayılan hash'leyici SipHash'tir ve her HashMap rastgele bir tohumla (seed) başlar ... (HashDoS)`
- **Hükm:** DOĞRU
- **Kaynak:** `guide/rust/library/alloc/src/string.rs:2920-2922` → `impl<I> ops::Index<I> for String where I: slice::SliceIndex<str>` (`usize`, `SliceIndex<str>` uygulamadığı için E0277; `/tmp/opencode/.../e0277index.rs` ile doğrulandı); `guide/rust/library/std/src/hash/random.rs:10, 35, 84, 94` → `use super::{..., SipHasher13}`, `RandomState`, `DefaultHasher(SipHasher13)`, `SipHasher13::new_with_keys(self.k0, self.k1)`.
- **Düzeltme:** gerek yok.
- **Önem:** Düşük.

### `learn/error-handling/src/main.rs:77-84` ve `:234-235`
- **Alıntı:** `// panic! o thread'i sonlandırır. VARSAYILAN davranış "unwind": stack geri sarılır, yoldaki değerlerin Drop'ları çalışır ... Cargo.toml'de [profile.release] panic = "abort" dersen unwind yerine anında ABORT olur: Drop çalışmaz, ikili dosya küçülür.`
- **Hükm:** DOĞRU
- **Kaynak:** `doc.rust-lang.org/book/print.html`, Bölüm 9 ("To Abort or Not to Abort"): `By default, when a panic occurs, the program starts unwinding... Rust therefore allows you to choose the alternative of immediately aborting, which ends the program without cleaning up... you can switch from unwinding to aborting upon a panic by adding 'panic = "abort"'... If in your project you need to make the resultant binary as small as possible...`.
- **Düzeltme:** gerek yok.
- **Önem:** Düşük.

### `learn/error-handling/src/main.rs:216-221` — Option must_use düzeltmesi
- **Alıntı (gövde):** `// • #[must_use]: Result (ve Option gibi bazı türler) bu öznitelikle işaretlidir.`
- **Alıntı (düzeltme notu):** `// ↳ DÜZELTME NOTU (son gözden geçirme): TÜR olarak yalnız Result '#[must_use]'tur; Option türü değildir ('Some(3);' ya da Option dönen bir çağrıyı yok saymak uyarı vermez — rustc ile doğrulandı). Yalnız bazı Option METOTLARI tek tek #[must_use] taşır.`
- **Hükm:** DOĞRU (düzeltme notu); YANILTICI (gövde, notla geçersiz kılınmış)
- **Kaynak:** `guide/rust/library/core/src/result.rs:555` → `#[must_use = "..."]`; `guide/rust/library/core/src/option.rs:586-592` → `#[doc(search_unbox)] #[derive(Copy, Debug, Hash)]` — `#[must_use]` YOK. Repro: `option_must_use.rs` ve `#![deny(unused_must_use)]` içeren `option_deny.rs` hiç uyarı üretmedi; `result_must_use.rs` uyarı üretti.
- **Düzeltme:** Gövdeyi düzeltme notunun içeriğiyle değiştirip ayrı "DÜZELTME NOTU" satırlarını kaldırın: `// • #[must_use]: Result türü bu öznitelikle işaretlidir; Option türü değildir, yalnızca bazı Option metotlarıdır.`
- **Önem:** Orta (öğrenci gövdeyi okuyup yanlış öğrenebilir).

### `learn/error-handling/src/derinlesme.rs:138-149`
- **Alıntı:** `// Error trait'i 'Debug + Display' ister (ikisi de yukarıda var); gövdesi boş bile olabilir. Tek ilginç metodu source(): ... Kural: bir hatayı source() ile veriyorsan Display'inde TEKRAR yazma`
- **Hükm:** DOĞRU
- **Kaynak:** `guide/rust/library/core/src/error.rs:59` → `pub trait Error: Debug + Display {` ve `:60-61` → `/// Returns the lower-level source of this error, if any.`; `error.rs:70+` örnekte `source` zinciri.
- **Düzeltme:** gerek yok.
- **Önem:** Düşük.

### `learn/error-handling/src/derinlesme.rs:444-449`, `:463-472` — Box<dyn Error>
- **Alıntı:** `// Box<dyn Error>, Error uygulayan HER türü kabul eden trait object'tir (Böl. 18): '?' her hatayı kutulayıp geçirir, hatta '&str'/'String' bile '.into()' ile hataya dönüşür. Bedeli: tür SİLİNİR; çağıran match yapamaz, ancak downcast_ref ile "şu tür müydü?" diye sorabilir.`
- **Hükm:** DOĞRU
- **Kaynak:** `/tmp/opencode/rust_verify/boxdyn.rs` derlendi (exit 0): `"toplam sıfır olamaz".into()` (→ `Box<dyn Error>`), `String.into()`, `parse::<u32>()?` (ParseIntError → Box) üçü de çalışıyor. Bölüm referansı için: `doc.rust-lang.org/book/print.html` → `"The 'Box<dyn Error>' type is a trait object, which we'll talk about in 'Using Trait Objects to Abstract over Shared Behavior' in Chapter 18"` (kitabın güncel 2024 edisyonunda trait object'ler **Böl. 18**'de; eski edisyonlarda Böl. 17 idi — şüphe üzerine resmî kaynaktan doğrulandı, derste hata yok).
- **Düzeltme:** gerek yok. (İsterseniz "(Böl. 18, güncel kitap)" notu eklenebilir; eski baskı okuyanlar için.)
- **Önem:** Düşük.

### `learn/error-handling/src/derinlesme.rs:573-582`
- **Alıntı:** `//    İpucu: 'partition(Result::is_ok)'; bu, §6'daki "kanıtlanabilir unwrap" için iyi bir örnek.` / `//   • generics-traits-lifetimes (Böl. 10) ... //   • minigrep (Böl. 12): 'run() -> Result<(), Box<dyn Error>>' ve 'main''de 'process::exit(1)' ...`
- **Hükm:** DOĞRU
- **Kaynak:** `/tmp/opencode/rust_verify/partition.rs` derlendi (exit 0); `doc.rust-lang.org/book/print.html` → Bölüm 10 "Generic Data Types, Traits, and Lifetimes", Bölüm 12 "An I/O Project: Building a Command Line Program" (minigrep), Bölüm 9'da `Box<dyn Error>` örneği; Bölüm 11 "Automated Tests" (`:585` notu).
- **Düzeltme:** gerek yok.
- **Önem:** Düşük.

### `learn/error-handling/src/derinlesme.rs:391-393`
- **Alıntı:** `// silme satırına ulaşılır; çünkü Err bir DEĞERDİR, akışı kesmez. Panik etseydi ulaşılmazdı — o garanti için Drop'lu bir koruyucu (RAII guard) gerekir (Böl. 15).`
- **Hükm:** DOĞRU
- **Kaynak:** `doc.rust-lang.org/book/print.html` → Bölüm 15 "Smart Pointers" (`Drop` ve `RefCell` bu bölümde; metin içi referanslar "in Chapter 15" ile defalarca doğrulandı); kodda `gecici_dosya_ile` (`:405-412`) `Err` değerini döndürüp `fs::remove_file` satırına geçiyor.
- **Düzeltme:** gerek yok.
- **Önem:** Düşük.

---

## 4. Benzetmeler ve sade anlatım kutuları

### `learn/structs/ACIKLAMA.md:5`, `:9`
- **Alıntı:** `Struct, ilgili bilgileri tek kartta toplayan bir kalıptır (struct); ... 'self' kartı alıp götürür. Alanları kapalı tutup kartı yalnız doğrulayan yapıcıyla basarsan, eline geçen her kartın geçerli olduğuna güvenebilirsin.` / `Öğrenci kartında ad, sınıf, numara gözleri var. Kartı basan büro kontrol eder; sen sadece kullanırsın.`
- **Hükm:** DOĞRU
- **Kaynak:** `learn/structs/src/derinlesme.rs:51-58, 72-118` (private alan + doğrulayan `ayristir` + `olustur` zinciri) ile birebir örtüşüyor.
- **Düzeltme:** gerek yok.
- **Önem:** Düşük.

### `learn/enums/ACIKLAMA.md:5`, `:9`
- **Alıntı:** `Enum, "bu değer şu seçeneklerden yalnız biridir" der; ... 'match' her varyant için ne yapılacağını yazdırır ve hiçbirini unutmana izin vermez (bütünlük / exhaustiveness).` / ` Bir zarfın aynı anda hem kutuda hem cebinde olması imkânsızdır.`
- **Hükm:** DOĞRU
- **Kaynak:** `learn/enums/src/main.rs:16-17, 226-248`; E0004 repro'su (`e0004.rs`). Postacı benzetmesi `main.rs:100-103` ile aynı.
- **Düzeltme:** gerek yok.
- **Önem:** Düşük.

### `learn/collections/ACIKLAMA.md:5` — yanıltıcı UTF-8 ifadesi
- **Alıntı:** `String'in harfleri bayt değil karakter sınırıyla ilgilidir; yanlış yerde kesmek çalışma anında durur.`
- **Hükm:** YANILTICI
- **Kaynak:** `learn/collections/src/main.rs:190-191` → `// Byte-aralığı dilim ALINABİLİR ama bir char SINIRINA denk gelmeli: &t[0..1] // çalışma anı PANİK`; `/tmp/opencode/rust_verify/slicepanic.rs` → `byte index 1 is not a char boundary`. Yani dilim **bayt ofsetleriyle** alınır; şart, bu bayt ofsetlerinin UTF-8 karakter sınırına denk gelmesidir. "bayt değil" ifadesi öğrenciyi `t[0..1]`'in "ilk karakter" olduğu düşüncesine götürebilir ki bu yanlıştır.
- **Düzeltme:** `String dilimleri bayt ofsetleriyle alınır; bu ofsetler bir UTF-8 karakter sınırına denk gelmek zorundadır, aksi hâlde çalışma anında panik olur.`
- **Önem:** Orta (konunun en kritik noktası yanlış aktarılıyor).

### `learn/error-handling/ACIKLAMA.md:5`, `:9`
- **Alıntı:** `Rust'ta try/catch yoktur; hata dönüş değerinin parçasıdır, böylece ona bakmayı unutamazsın. ... Kâğıt boşsa sessizce geçip gidemezsin; biri mutlaka haber verir.`
- **Hükm:** DOĞRU (küçük bir açıklama önerisiyle)
- **Kaynak:** `doc.rust-lang.org/book/print.html` Bölüm 9 ("Rust doesn't have exceptions... errors are values"); `learn/error-handling/src/derinlesme.rs:205-209` → boş **satır** `Ok(None)` döner ve atlanır, bütün **kart** boşsa `sayi_al(&tablo, "port")?` → `AyarHatasi::Eksik("port")` döner. Benzetmedeki "kâğıt boşsa" tüm kart anlamında okunmalı.
- **Düzeltme:** Karışıklığı önlemek için `Kart tamamen boşsa zorunlu anahtarlar eksik sayılır ve hata verilir; karttaki tek bir boş satır ise sorun değildir, geçilir.` eklenebilir.
- **Önem:** Düşük.

---

## 5. Genelleme hatası: Copy ≠ küçük değer

### `learn/structs/src/main.rs:37` (aynı ifade `enums/src/main.rs:40` ve `collections/src/main.rs:51`)
- **Alıntı:** `//   Copy, move             → küçük değerler kopyalanır (Copy); String gibi büyükler taşınır (move).` / `//   Copy                   → sayı gibi küçük değerler atanınca kopyalanır; eskisi de kullanılır.` / `//   Copy / move            → küçük değerler kopyalanır; String gibi büyükler taşınır.`
- **Hükm:** YANILTICI (kural olarak; "sayı gibi" yumuşatması `enums` versiyonunda kabul edilebilir)
- **Kaynak:** `/tmp/opencode/rust_verify/copy_check.rs` → 1024 byte'lık `[u8; 1024]` ve 16 byte'lık `&str` **Copy** (üçer kez atanıp kullanıldı, derleme exit 0); buna karşılık `String` 24 byte olmasına rağmen Copy değil. Ölçüt türün boyutu değil, `Copy` trait'ini uygulayıp uygulamamasıdır. Aynı dosyanın 201-203. satırlarındaki tuple struct benzetmesi doğru olsa da sözlükteki genelleme yanlıştır.
- **Düzeltme:** `Copy, move → Copy trait'ini uygulayan türler (sayılar, bool, char, referanslar, tamamı Copy olan diziler/tuple'lar) atanınca kopyalanır; String gibi sahiplik taşıyan türler taşınır (move). Kural türün boyutu değil, Copy trait'idir.`
- **Önem:** Orta (ileride `[u8; N]` veya `&str` ile karşılaşınca kavram kırığı yaratır).

---

## 6. Sorunsuz alanlar (ayrıca doğrulandı)

- `learn/structs/src/main.rs:126-133, 195-204, 252-266` — field init shorthand, tuple struct/unit-like struct ayrımı, üç receiver'ın sahiplik anlamı ve otomatik referanslama: doğru. `self_receiver.rs` ile `self` receiver'ın taşıdığı doğrulandı.
- `learn/structs/src/main.rs:224-228` — `struct S { r: &str }` → E0106, `'a` ile çözümü ve dangling gerekçesi: doğru (repro `e0106.rs`).
- `learn/structs/src/derinlesme.rs:85-98, 148-160` — doğrulayan yapıcı + builder zinciri (`mut self -> Self`, `ok_or_else`, `?`) mantıksal olarak tutarlı; beklenen çıktılar kodla örtüşüyor.
- `learn/enums/src/derinlesme.rs:81-103` — "2 × 2 × 2 = 8 olasılık, anlamlı olan 3" sayımı doğru; `tutarli_mi()` mantığıyla örtüşüyor.
- `learn/enums/src/derinlesme.rs:158-192` — `(durum, olay)` geçiş tablosu, catch-all'ın "varsayılan reddet" olması ve `ozet()`'te `_ =>` bulunmaması: doğru. Yeni varyant eklenince `ozet()` E0004 verir, `gecis()` catch-all sayesinde derlenir — alıştırma notu (`:461`) doğru.
- `learn/collections/src/main.rs:85-103, 226-249` — `v[99]` panik / `get(99)` → None, `entry().or_insert`, HashMap gezinme sırasının garantisizliği ve sıralama: doğru.
- `learn/collections/src/main.rs:172-177` — `+` operatörünü soldaki String'i tüketmesi, `format!`'in tüketmemesi: doğru (dil kurallarıyla uyumlu).
- `learn/error-handling/src/main.rs:177-200` — `From<ParseIntError>` + `?` otomatik dönüşümü ve `Display` kullanımı: doğru (`e0277conv.rs`'te From eklenince derlendi).
- `learn/error-handling/src/derinlesme.rs:205-248` — `transpose()`, `collect()` ile ilk hatada durma, `map_err` ile bağlam ekleme, `FromStr<Err = ParseIntError>` bound'u: doğru.
- `learn/error-handling/src/derinlesme.rs:495-519` — `unwrap`/`expect`'in meşru olduğu yerler (sabit literal, kanıtlanabilir `max()`), "sorumlu kim?" ölçütü: doğru.
- Tüm `## 5 yaşındaki çocuğa anlatır gibi` kutuları ve `SADE ANLATIM` blokları, kendi dosyalarındaki kodla tutarlı; teknik bir yanlışlık bulunmadı.
- Tüm "Çalıştır:" satırları yalnızca yorum olarak duruyor; hiçbiri çalıştırılmadı ve herhangi bir `learn/` dosyasına repro eklenmedi.

## 7. Doğrulanamayan iddialar

Bu denetimde kesin kaynak bulunamayan veya doğrulanamayan bir iddia **kalmadı**. Şüpheli görülen üç nokta (Böl. 18 trait object, `check_match.rs:1281`, `Option` must_use) doğrudan kaynaktan doğrulandı ve derste hata olmadığı görüldü. Yalnızca aşağıdaki yorum niteliğindeki ifadeler ölçülebilir değildir, hata sayılmaz:

- `structs/src/main.rs:147` — "`&mut u` elde eden, instance'ın TÜM alanlarına yazma hakkı kazanır": dil kuralı olarak doğru, ancak özel bir mini repro ile ayrıca sınanmadı (standart borrow kurallarından çıkarıldı).
- `collections/src/main.rs:263-265` — "push, cap dolmuşsa ... eski elemanları kopyalar" (amortize O(1)): Vec'in büyüme stratejisi belgelenmiş olmakla birlikte, tam kopyalama vs. move ayrımı sürüm detayıdır; derste "kopyalar" denmesi pedagojik sadeleştirme olarak kabul edilebilir.

---

## Ek — kullanılan mini repro'lar

Tümü `/tmp/opencode/rust_verify/` altında, `--edition=2021` ile derlendi; `learn/` ile ilişkileri yoktur:

`sizes.rs`, `sizes2.rs`, `schema.rs`, `slicepanic.rs`, `e0106.rs`, `partial_move.rs`, `partial_move_debug.rs`, `e0382.rs`, `e0451.rs`, `e0451update.rs`, `privacy.rs`, `dead_code.rs`, `self_receiver.rs`, `e0507.rs`, `e0507b.rs`, `e0063.rs`, `option_must_use.rs`, `option_deny.rs`, `result_must_use.rs`, `copy_check.rs`, `e0004.rs`, `e0277index.rs`, `e0277q.rs`, `e0277from.rs`, `e0277conv.rs`, `e0277from2.rs`, `e0277from3.rs`, `e0277mix.rs`, `e0369.rs`, `e0023.rs`, `e0502.rs`, `e0596.rs`, `partition.rs`, `boxdyn.rs`.

**Kaynaklar:** `guide/rust` (yerel rust-lang/rust ağacı, belirtilen dosya:satır'lar), `https://doc.rust-lang.org/book/print.html` (resmî kitap, Rust 1.98 / edition 2024), `rustc 1.99.0 --explain E####`.
