# Rust dersleri doğruluk denetimi — minigrep / writing-tests / macros / smart-pointers

Kapsam: `learn/minigrep`, `learn/writing-tests`, `learn/macros`, `learn/smart-pointers` altındaki TÜM `.rs` yorumları ve dört `ACIKLAMA.md`. `learn/` altında değişiklik yapılmadı; "Çalıştır:" satırları veri olarak okundu.

Yöntem ve kaynaklar: yerel `guide/rust` (rust-lang/rust 1.99 ağacı), doc.rust-lang.org (Reference / Book / std), `rustc 1.99.0 (b940084d7 2026-09-28)` + `cargo 1.99.0`, ve `/tmp/opencode` altında oluşturulan mini repro'lar (`audit_repro.rs`, `audit_e0072.rs`, `audit_e0594.rs`, `audit_e0596.rs`, `audit_e0716.rs`, `audit_e0277.rs`, `audit_errcodes.rs`, `audit_deadcode.rs`, `pmtest/`, `cwdtest/`). Her notun "Kaynak" alanı kesin konumu verir.

Hüküm anahtarı: DOĞRU / YANLIŞ / YANILTICI / MODASI GEÇMİŞ / DOĞRULANAMADI. Önem: Kritik / Orta / Düşük.

---

## 1. proc-macro crate tipi zorunluluğu, TokenStream, macro_rules vs proc macro

### Takeaway

`proc-macro = true` zorunluluğu, `TokenStream` imzası ve "makro tanımlandığı crate'te kullanılamaz" iddiaları doğru. Buna karşılık iki yorum, proc-macro crate'inin `fn main` içeremiyeceğini / `cargo run` ile çalıştırılamayacağını söylüyor; rustc 1.99 mini repro'su bunun yanlış olduğunu gösteriyor. Gerçek kısıt, lib hedefinin proc-makro dışında öğe dışa aktaramaması.

### Denetim notları

**`learn/macros/Cargo.toml:6-8`** — "proc-macro crate'i: içinde `fn main` çalışmaz, yalnızca makro üretir. Bu yüzden main.rs değil lib.rs ister ve kendisi çalıştırılamaz; başka bir crate (burada tests/) onu kullanır. Denemek için: cargo test -p macros"
Hüküm: **YANLIŞ** (kısmen). Kaynak: `/tmp/opencode/pmtest/` mini repro — `[lib] proc-macro = true` olan crate'e `src/main.rs` eklenip `cargo build && cargo run` çalıştırıldığında derlenip `hi` yazdı. Ayrıca https://doc.rust-lang.org/reference/procedural-macros.html §`macro.proc.def` yalnızca makro tanımının `proc-macro` crate tipinde olmasını şart koşar; binary hedef yasağı yoktur. Gerçek kısıt repro'da görüldü: `pub fn yardimci()` eklenince `error: 'proc-macro' crate types currently cannot export any items other than functions tagged with #[proc_macro]...`. Önerilen düzeltme: "Bu crate'in **lib hedefi** proc-macro tipinde olmalı; bu yüzden makrolar lib.rs'e konur ve lib, proc-makro dışında öğe dışa aktaramaz. Makro tanımlandığı crate'te kullanılamaz; tests/ altındaki ayrı crate kullanır." Önem: **Orta**.

**`learn/macros/src/lib.rs:28-29`** (doc yorumu) — "Böyle bir crate içinde normal kod çalıştırılamaz (fn main olmaz); makroyu ancak BAŞKA bir crate kullanabilir."
Hüküm: **YANLIŞ** (birinci cümle) / **DOĞRU** (ikinci cümle). Kaynak: aynı `pmtest/` repro'su; https://doc.rust-lang.org/reference/procedural-macros.html §`macro.proc.def` ("The macros may not be used from the crate where they are defined, and can only be used when imported in another crate"). Önerilen düzeltme: "Bu crate'in lib hedefi proc-makro dışında öğe dışa aktaramaz; özel (private) fonksiyonlar yazılabilir ve ayrı bir binary hedefi çalıştırılabilir. Makroyu ancak BAŞKA bir crate kullanabilir." Önem: **Orta**.

**`learn/macros/src/lib.rs:11-12`** — "Bu yüzden bu dosyada `fn main` yoktur; damganın izini görmek için tests/kullanim.rs dosyasına bakarsın."
Hüküm: **DOĞRU** (gözlem) / gerekçe zayıf. Kaynak: dosya yapısı (`macros/` altında `src/main.rs` yok) + yukarıdaki repro. "Bu yüzden" bağlacı, yukarıdaki yanlış genellemeden kaynaklanıyor. Önerilen düzeltme: gerekçeyi "makro tanımlandığı crate'te kullanılamadığı için" olarak değiştir; `fn main` yokluğu bu crate'in tercihidir, zorunluluk değil. Önem: **Düşük**.

**`learn/macros/src/lib.rs:16`** (mini sözlük) — "prosedürel makro (proc-macro) → sıradan bir Rust fonksiyonu olarak yazılan makro."
Hüküm: **DOĞRU** (imza kısıtıyla birlikte). Kaynak: https://doc.rust-lang.org/reference/procedural-macros.html §`macro.proc.proc_macro.allowed-positions` — `#[proc_macro]` yalnızca crate kökünde `pub fn(TokenStream) -> TokenStream` imzalı, Rust ABI'li fonksiyona uygulanabilir. Önerilen düzeltme: "sıradan bir Rust fonksiyonu olarak yazılan, `pub fn(TokenStream) -> TokenStream` imzalı makro." Önem: **Düşük**.

**`learn/macros/src/lib.rs:18` ve `:48`** — "TokenStream → art arda dizilmiş token'lar; makronun aldığı ve verdiği şey." / "TokenStream, kaynak kodun 'token'lara (sözcük parçaları) ayrılmış hâlidir."
Hüküm: **DOĞRU**. Kaynak: https://doc.rust-lang.org/reference/procedural-macros.html §`macro.proc.proc_macro-crate.token-stream` ("Procedural macros operate over token streams ... roughly equivalent to Vec<TokenTree>"). Önerilen düzeltme: yok. Önem: **Düşük**.

**`learn/macros/src/lib.rs:44`** — "// bu dosyanın içinde cevap_uret!() // ✗ derlenmez: makro tanımlandığı crate'te kullanılamaz"
Hüküm: **DOĞRU**. Kaynak: https://doc.rust-lang.org/reference/procedural-macros.html §`macro.proc.def`. Önerilen düzeltme: yok. Önem: **Düşük**.

**`learn/macros/src/lib.rs:39`** — "Aynı kalıptaki kodu elle tekrar tekrar yazmak yerine derleyiciye yazdırmak için; `#[derive(Debug)]` de bu yolla çalışır."
Hüküm: **DOĞRU** (mekanizma düzeyinde; küçük nüans). Kaynak: https://doc.rust-lang.org/reference/procedural-macros.html — derive, proc makronun üç çeşidinden biridir. Nüans: `Debug` derive'i rustc'nin yerleşik (built-in) derive'ıdır; dışarıdan yazılan bir proc-macro değildir, ama genişleme mekanizması aynıdır. Önerilen düzeltme: isteğe bağlı olarak "(`Debug` derleyicinin yerleşik derive'ı olsa da aynı genişleme mekanizmasını kullanır)". Önem: **Düşük**.

**`learn/macros/tests/kullanim.rs:2-4` ve `:10-11`** — "Makroyu DIŞARIDAN kullanan entegrasyon testi. tests/ altındaki her dosya ayrı bir crate gibi derlenir ve `macros` crate'ine bağımlıdır; makro burada gerçekten genişler." / "Prosedürel makro, tanımlandığı crate'in içinde kullanılamaz; tests/ altındaki her dosya ayrı bir crate gibi derlendiği için burada kullanılabilir."
Hüküm: **DOĞRU**. Kaynak: https://doc.rust-lang.org/book/ch11-03-test-organization.html ("Cargo will compile each of the files as an individual crate") + Reference §`macro.proc.def`. Önerilen düzeltme: yok. Önem: **Düşük**.

**Kapsam boşluğu (yanlış iddia değil):** Bu dört dizinde `macro_rules!` (declarative macro) ile proc-macro arasında açık bir karşılaştırma yok; mini sözlükte "makro → kod yazan kod" genel tanımı, "prosedürel makro → ..." tanımıyla yan yana duruyor. Öğrenci "makro" ile "proc-macro"yu eşitleyebilir. Kaynak: https://doc.rust-lang.org/reference/macros-by-example.html (`macro_rules!`) ve procedural-macros.html. Öneri: mini sözlüğe "declarative makro (`macro_rules!`): desen eşleştirerek genişler, ayrı crate gerektirmez" satırı eklenebilir. Önem: **Düşük**.

### Çıkarımlar

- Dersin "damga atölyesi / damgayı basan başka crate" benzetmesi, makronun tanımlandığı crate dışında kullanılması kuralını doğru yansıtıyor.
- İki YANLIŞ not aynı kök hatayı paylaşıyor: crate tipi kısıtını "crate'in bütün hedefleri"ne yaymak. Düzeltme tek bir cümleyle tutarlı hale getirilebilir.

### Boşluklar

- `macro_rules!` ile proc-macro karşılaştırmasının ders materyalinde bulunmaması bir kapsam boşluğudur; doğrulanacak bir teknik iddia yoktur.

---

## 2. tests/ dizininin ayrı crate olması, #[test] / assert_eq!, birim vs entegrasyon testi

### Takeaway

Bu alandaki iddiaların tamamı doğrulandı: `#[cfg(test)]`, `#[test]`, `assert!` ailesi, `#[should_panic(expected = ...)]`, `Result` döndüren test, `#[ignore]`, paralel koşu, stdout yakalama, filtreleme ve `tests/` dosyalarının ayrı crate/ayrı ikili olarak derlenmesi. Tek nüans, "private'a erişim"in "aynı crate içinde" değil "ata modülden" kaynaklanması.

### Denetim notları

**`learn/writing-tests/src/lib.rs:2`** — "// Rust Book — Bölüm 11: Writing Automated Tests"
Hüküm: **DOĞRU**. Kaynak: https://doc.rust-lang.org/book/ch11-01-writing-tests.html ve https://doc.rust-lang.org/book/ch11-03-test-organization.html (bölüm 11 = "Writing Automated Tests"). Önem: **Düşük**.

**`learn/writing-tests/src/lib.rs:6-7`** — "Test yazmak Rust'ta DİLE ve cargo'ya gömülüdür — ayrı bir framework kurmazsın. Bir fonksiyona #[test] koyarsın, `cargo test` onları ayrı bir ikiliye derleyip koşturur."
Hüküm: **DOĞRU**. Kaynak: https://doc.rust-lang.org/reference/attributes/testing.html ("Compiling a crate in 'test' mode enables building the test functions along with a test harness"); https://doc.rust-lang.org/book/ch11-02-running-tests.html ("`cargo test` compiles your code in test mode and runs the resultant test binary"). Önem: **Düşük**.

**`learn/writing-tests/src/lib.rs:77-78`** — "#[cfg(test)]: bu modül YALNIZCA `cargo test`'te derlenir; `cargo build`/`run`'da hiç yoktur (ikiliye test kodu sızmaz). `use super::*;` ile üstteki (private dahil) öğeleri içeri alır."
Hüküm: **DOĞRU**. Kaynak: https://doc.rust-lang.org/book/ch11-03-test-organization.html ("The `#[cfg(test)]` annotation ... only when you run `cargo test`, not when you run `cargo build`"). Önem: **Düşük**.

**`learn/writing-tests/src/lib.rs:69`** — "PRIVATE fonksiyon: dışarıdan erişilemez ama birim testleri (aynı crate içinde) test EDEBİLİR."
Hüküm: **YANILTICI**. Kaynak: https://doc.rust-lang.org/book/ch11-03-test-organization.html §Private Function Tests — "items in child modules can use the items in their ancestor modules"; erişim crate'e değil modül hiyerarşisine bağlıdır. Aynı dosyanın `:121` satırı doğru ifade etmiş: "(çocuk modül → üst öğeler görünür)". Önerilen düzeltme: "birim testleri (aynı modül ağacında, alt modülden `use super::*` ile) erişebilir; aynı crate'deki her modül erişemez." Önem: **Düşük-Orta**.

**`learn/writing-tests/src/lib.rs:115`** — "// assert!: bool bekler. İkinci argümandan sonrası format! gibi ÖZEL HATA MESAJIdır."
Hüküm: **DOĞRU**. Kaynak: https://doc.rust-lang.org/std/macro.assert.html (`assert!($cond)`, `assert!($cond, $($arg:tt)+)` — isteğe bağlı format argümanları). Önem: **Düşük**.

**`learn/writing-tests/src/lib.rs:142`** — "#[should_panic(expected = "negatif olamaz")] // panik ETMELİ; mesaj bu alt dizgiyi içermeli"
Hüküm: **DOĞRU**. Kaynak: https://doc.rust-lang.org/reference/attributes/testing.html §`attributes.testing.should_panic.expected` ("the given string must appear somewhere within the panic message"). Önem: **Düşük**.

**`learn/writing-tests/src/lib.rs:149`** — "// Test Result<(), E> da döndürebilir: Err dönerse test düşer. ? operatörünü teste taşır."
Hüküm: **DOĞRU**. Kaynak: https://doc.rust-lang.org/reference/attributes/testing.html §`attributes.testing.test.success` — `Result<(), E>` döndüren testler `Ok(())` ile geçer; `Termination` uygulayan dönüş türleri kabul edilir. Önem: **Düşük**.

**`learn/writing-tests/src/lib.rs:159-162`** — "`#[ignore]`, uzun süren bir denemeyi 'bugün yapma' diye kenara ayırmaktır. Test silinmez; `cargo test -- --ignored` dediğinde yine çalışır." / "normalde atlanır; sadece `cargo test -- --ignored` ile koşar (yavaş/isteğe bağlı)"
Hüküm: **DOĞRU** (küçük eksik: `--include-ignored` de vardır). Kaynak: https://doc.rust-lang.org/book/ch11-02-running-tests.html ("we can use `cargo test -- --ignored` ... To run all tests whether they're ignored or not, you can run `cargo test -- --include-ignored`"). Önerilen düzeltme: "yalnız `--ignored` ile değil, `--include-ignored` ile de koşar." Önem: **Düşük**.

**`learn/writing-tests/src/lib.rs:185-189`** — "libtest (test harness): #[test] fonksiyonları ... VARSAYILAN olarak PARALEL (çok iş parçacığı) çalıştırır ... `cargo test -- --test-threads=1` seri yapar. Bir test başarılı olduğunda stdout'u GİZLENİR; görmek için `cargo test -- --show-output`."
Hüküm: **DOĞRU**. Kaynak: https://doc.rust-lang.org/book/ch11-02-running-tests.html (paralel koşu, `--test-threads=1`, `--show-output` bölümleri). Önem: **Düşük**.

**`learn/writing-tests/src/lib.rs:191-193`** — "Bir test, panik ederek düşer. assert!/assert_eq! aslında koşul tutmazsa panic! çağırır; #[should_panic] ise bu mantığı TERSİNE çevirir (panik beklenir). Her test kendi mantığında izole panik yakalanır, biri düşünce ötekiler koşmaya devam eder."
Hüküm: **DOĞRU**. Kaynak: https://doc.rust-lang.org/reference/attributes/testing.html §test.success ("Tests that return `()` pass as long as they terminate and do not panic") + Book ch11-02 (başarısız testin diğerlerini durdurmaması). Önem: **Düşük**.

**`learn/writing-tests/src/lib.rs:195-196`** — "Filtreleme: `cargo test topla` → adında "topla" geçen testleri koşar (substring). `cargo test -- --ignored` → yalnız #[ignore]'luları. Ad çözümü libtest'te, derleyicide değil."
Hüküm: **DOĞRU**. Kaynak: https://doc.rust-lang.org/book/ch11-02-running-tests.html §Running a Subset of Tests by Name (substring eşleşmesi ve `--ignored`). "Ad çözümü" ifadesi teknik olmayan bir anlatım ama davranış doğru. Önem: **Düşük**.

**`learn/writing-tests/src/lib.rs:200-204`** — "Entegrasyon testleri: tests/ dizininde AYRI crate'ler; crate'i bir kullanıcı gibi yalnız PUBLIC API'dan `use writing_tests::...` ile çağırır (private görünmez). Her tests/ dosyası ayrı bir ikili olarak derlenir. ... (Bu yüzden entegrasyon testi için crate'in bir lib hedefi olmalı; salt-ikili crate'in tests/'i public API göremez.)"
Hüküm: **DOĞRU**. Kaynak: https://doc.rust-lang.org/book/ch11-03-test-organization.html ("Cargo will compile each of the files as an individual crate", "only call functions that are part of your library's public API", "If our project is a binary crate that only contains a src/main.rs ... we can't create integration tests ... bring functions ... into scope with a use statement"). Önem: **Düşük**.

**`learn/writing-tests/tests/entegrasyon.rs:2-4`** — "ENTEGRASYON TESTİ — tests/ dizininde, AYRI bir crate olarak derlenir. Buradan crate'e, dışarıdaki bir kullanıcı gibi SADECE public API'dan erişebiliriz: `kirp` (private) buradan görünmez; sadece pub olanlar."
Hüküm: **DOĞRU**. Kaynak: https://doc.rust-lang.org/book/ch11-03-test-organization.html. Önem: **Düşük**.

**`learn/writing-tests/tests/entegrasyon.rs:15-16`** — "Bu dosyada #[cfg(test)]/mod GEREKMEZ: tests/ dosyaları zaten yalnız test derlemesinde derlenir ve her biri kendi test ikilisidir."
Hüküm: **DOĞRU**. Kaynak: https://doc.rust-lang.org/book/ch11-03-test-organization.html ("because integration tests go in a different directory, they don't need the `#[cfg(test)]` annotation"). Önem: **Düşük**.

### Çıkarımlar

- Test bölümü, harness davranışını (paralellik, çıktı yakalama, filtreleme, panic izolasyonu) güncel Book/Reference ile uyumlu anlatıyor; burada düzeltme gerekmiyor.
- Tek tutarsızlık, aynı dosya içinde privacy'nin bir yerde "aynı crate", diğer yerde "çocuk modül" olarak açıklanması; ikincisi doğru olan.

### Boşluklar

- "Ad çözümü libtest'te, derleyicide değil" ifadesi için ayrı bir kaynak aranmadı; filtreleme davranışı Book ile doğrulandı, ancak bu cümlenin teknik anlamı (isim çözümü derleyici aşaması mı, çalışma anı mı) belirsizliğini koruyor. Önem: **Düşük**.

---

## 3. Box / E0072, Deref / Drop, Rc / Weak / RefCell / Cell, referans döngüsü, BorrowMutError paniği, Rc::make_mut

### Takeaway

Teknik mekanik (E0072 mesajı ve yardım metni, deref coercion, RAII/drop, Rc sayacı, Weak döngü kırılması, Cell/RefCell ayrımı, E0594/E0596/E0277/E0716 hata kodları, make_mut) doğru ve rustc 1.99 ile doğrulandı. Tek sistematik sorun, RefCell panik mesajının birkaç yerde eski biçimde ("already borrowed: BorrowMutError") verilmesi; aynı dosyanın DÜZELTME NOTU ise güncel mesajı doğru yazıyor. Ayrıca "drop elaboration"ın "Bölüm 4'te" olduğu atfı yanlış.

### Denetim notları

**`learn/smart-pointers/src/main.rs:10-11`** — "Doğrulanmış (rustc ile): Box'sız özyineli tip → E0072 "infinite size"; RefCell'de aynı anda iki borrow_mut → ÇALIŞMA ANI panik (BorrowMutError "already borrowed")."
Hüküm: **YANILTICI** (ikinci yarısı modası geçmiş mesaj; ilk yarısı DOĞRU). Kaynak: `/tmp/opencode/audit_repro.rs` çalıştırma çıktısı: `thread 'main' panicked ... RefCell already borrowed`; `guide/rust/library/core/src/cell.rs:917-922` (`panic_already_borrowed` → `"RefCell already borrowed"`). Önerilen düzeltme: "... → ÇALIŞMA ANI panik (güncel mesaj: `RefCell already borrowed`)." Önem: **Orta**.

**`learn/smart-pointers/src/main.rs:68-69` ve `:80`** — "Bir enum kendini doğrudan içerirse boyut sonsuz olur (E0072). Box ARAYA bir pointer koyar: pointer'ın boyutu sabittir (bir adres), döngü kırılır." / "enum Kotu { Halka(i32, Kotu), Son } // ✗ derlenmez: boyu sonsuz olurdu"
Hüküm: **DOĞRU**. Kaynak: `/tmp/opencode/audit_e0072.rs` derleme çıktısı: `error[E0072]: recursive type 'Kotu' has infinite size`. Önem: **Düşük**.

**`learn/smart-pointers/src/main.rs:271-272`** — "→ E0072 "recursive type has infinite size", derleyici "insert indirection (Box/Rc/&)" der."
Hüküm: **DOĞRU**. Kaynak: aynı repro'nun yardım satırı: `help: insert some indirection (e.g., a 'Box', 'Rc', or '&') to break the cycle`. Önem: **Düşük**.

**`learn/smart-pointers/src/main.rs:248-253`** — "// ✗ ÇALIŞMA ANI kuralı: aynı anda iki aktif borrow_mut → panik (BorrowMutError): ... // panik: "already borrowed: BorrowMutError" // ↳ DÜZELTME NOTU (son gözden geçirme): güncel std'de panik mesajı "RefCell already borrowed"dur (eski sürümlerde "already borrowed: BorrowMutError" idi); kaynak: core/src/cell.rs, panic_already_borrowed."
Hüküm: **DOĞRU** (düzeltme notu doğru; `:250` satırındaki eski mesaj ise açıkça "eski" örnek olarak işaretlenmiş). Kaynak: `guide/rust/library/core/src/cell.rs:917-922` + `/tmp/opencode/audit_repro.rs` çıktısı (`RefCell already borrowed`). Önerilen düzeltme: yok; bu not, dosyadaki diğer iki eski mesajlı yorum için referans alınmalı. Önem: **Düşük** (doğrulanan düzeltme).

**`learn/smart-pointers/src/main.rs:274-276`** — "Deref (library/core/src/ops/deref.rs): `*x`, aslında `*(x.deref())` olarak çözülür. DEREF COERCION derleme anında zincirlenir (&Kutum<String> → &String → &str) ... Tamamen statik; runtime maliyeti yok."
Hüküm: **DOĞRU**. Kaynak: `guide/rust/library/core/src/ops/deref.rs` (dosya mevcut, doğrulandı) + https://doc.rust-lang.org/reference/expressions/operator-expr.html#the-dereference-operator + https://doc.rust-lang.org/std/ops/trait.Deref.html (coercion). Önem: **Düşük**.

**`learn/smart-pointers/src/main.rs:278-281`** — "Drop (library/core/src/ops/drop.rs): değer kapsam dışına çıkınca derleyici drop'u OTOMATİK çağırır (RAII) — Bölüm 4'teki "drop elaboration"ın MIR'e eklediği çağrı. Elle drop() yerine std::mem::drop gerekir çünkü değeri taşıyıp bırakır; .drop()'u sen çağırsan sonra bir de otomatik çağrı gelir, çift-drop olurdu — derleyici bu yüzden doğrudan .drop()'u yasaklar."
Hüküm: **YANILTICI** (yalnız "Bölüm 4'teki" atfı); mekaniğin geri kalanı DOĞRU. Kaynak: `guide/rust/src/doc/rustc-dev-guide/src/mir/drop-elaboration.md:91` ("... into one with drop flags—is known as drop elaboration"; MIR pass), `guide/rust/library/core/src/ops/drop.rs` (mevcut); `error[E0040]: explicit use of destructor method` — `/tmp/opencode/audit_errcodes.rs`. Rust Book Bölüm 4 "Understanding Ownership" MIR/drop elaboration anlatmaz. Önerilen düzeltme: "Bölüm 4'teki" yerine "rustc'nin MIR drop-elaboration aşamasının (bkz. rustc-dev-guide) eklediği çağrı" deyin; isterseniz Book Bölüm 4'te sahiplik/Drop konusunun işlendiğini, MIR ayrıntısının dev-guide'da olduğunu belirtin. Önem: **Düşük-Orta**.

**`learn/smart-pointers/src/main.rs:149` ve `:172-173`** — "// b.drop(); // ✗ derlenmez: temizlik metodunu elle çağırmak yasak" / "b'yi ERKEN düşürmek: `b.drop()` çağrılamaz (çift-drop olurdu); std::mem::drop kullanılır."
Hüküm: **DOĞRU**. Kaynak: `/tmp/opencode/audit_errcodes.rs` → `error[E0040]: explicit use of destructor method`; https://doc.rust-lang.org/std/mem/fn.drop.html. Önem: **Düşük**.

**`learn/smart-pointers/src/main.rs:175`** — "// Düşme SIRASI: tanımın TERSİ. Burada _a, fonksiyon sonunda otomatik düşer."
Hüküm: **DOĞRU**. Kaynak: https://doc.rust-lang.org/reference/destructors.html (yerel değişkenlerin ters sırayla düşürülmesi) + `:144-146` satırındaki "Toplama sırası çıkarma sırasının tersidir" benzetmesiyle tutarlı. Önem: **Düşük**.

**`learn/smart-pointers/src/main.rs:86-87`** — "// Değerleri match ile GERÇEKTEN okuyan yürüyüş: hem toplamı verir hem de derive(Debug) dead_code tuzağını önler (Böl. 5/8: Debug ile basmak alanları "okundu" saymaz)."
Hüküm: **DOĞRU** (teknik iddia), "Böl. 5/8" atfı zayıf. Kaynak: `/tmp/opencode/audit_deadcode.rs` uyarı notu: "`Liste` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis". Önerilen düzeltme: "derleyici notu: derived Debug, dead-code analizinde kasıtlı olarak sayılmaz" ifadesiyle Bölüm atfını genelleştirin veya kaldırın. Önem: **Düşük**.

**`learn/smart-pointers/src/main.rs:283-286`** — "Rc<T> (library/alloc/src/rcs/rc.rs): veriyle birlikte bir SAYAÇ (strong_count) tutar. Rc::clone sayacı +1 yapar (veriyi kopyalamaz — ucuz); bir Rc düşünce -1; sayaç 0 olunca veri serbest kalır. Yalnız TEK THREAD için (sayaç atomik değil; çok thread için Arc). Rc salt paylaşımlı & verir — değiştiremezsin."
Hüküm: **DOĞRU** (kaynak yolu da dahil). Kaynak: `guide/rust/library/alloc/src/rcs/rc.rs` — dosya gerçekten bu yolda (1.99 ağacında `rcs/` altına taşınmış; `library/alloc/src/rc.rs` değil); https://doc.rust-lang.org/std/rc/struct.Rc.html. Önem: **Düşük**.

**`learn/smart-pointers/src/main.rs:288-291`** — "RefCell<T> (library/core/src/cell.rs): İÇ DEĞİŞKENLİK ... ihlal edilirse derhal PANİK eder (BorrowMutError "already borrowed", cell.rs ~880)."
Hüküm: **YANILTICI** (mesaj güncel değil; aynı dosyanın `:251-253` düzeltme notuyla çelişiyor). Kaynak: `guide/rust/library/core/src/cell.rs:917-922` ve `:895-907` (BorrowMutError tanımı ve Display: `"RefCell already borrowed"`), `/tmp/opencode/audit_repro.rs`. Satır numarası ~880 da artık ~895-922 aralığına kaymış. Önerilen düzeltme: "PANİK eder (güncel mesaj: `RefCell already borrowed`, bkz. `panic_already_borrowed`, cell.rs:917)". Önem: **Orta**.

**`learn/smart-pointers/src/main.rs:296-300`** — "REFERANS DÖNGÜSÜ ve Weak: Rc<RefCell<...>> ile iki düğüm birbirini Rc ile tutarsa sayaç asla 0 olmaz → BELLEK SIZINTISI (Rust bunu güvensiz saymaz; UB değil, sadece sızıntı). Çözüm: döngüyü kıran kenarda Rc yerine Weak<T> kullanmak. Weak sayacı artırmaz (strong değil, weak sayaç), upgrade() ile Option<Rc> verir ... Çocuk→ebeveyn kenarı Weak yapılırsa döngü oluşmaz, sızıntı önlenir."
Hüküm: **DOĞRU**. Kaynak: https://doc.rust-lang.org/reference/behavior-not-considered-unsafe.html ("Leaks of memory and other resources" unsafe sayılmaz) + https://doc.rust-lang.org/std/rc/struct.Weak.html (`upgrade() -> Option<Rc<T>>`, weak count güçlü sayacı etkilemez). Önem: **Düşük**.

**`learn/smart-pointers/src/derinlesme.rs:13`** — "✗ örneklerin hata kodları rustc 1.99 ile derlenerek doğrulandı."
Hüküm: **DOĞRU** (bağımsız olarak doğrulandı). Kaynak: `/tmp/opencode/audit_e0594.rs` (E0594), `audit_e0596.rs` (E0596), `audit_e0277.rs` (E0277 ×2), `audit_e0716.rs` (E0716), `audit_errcodes.rs` (E0040), `audit_e0072.rs` (E0072) — tümü rustc 1.99.0 ile üretildi. Önem: **Düşük**.

**`learn/smart-pointers/src/derinlesme.rs:399-402`** — "// ✗ let ilk = liste.borrow().first().unwrap(); println!("{ilk}"); // E0716: temporary value dropped while borrowed"
Hüküm: **DOĞRU**. Kaynak: `/tmp/opencode/audit_e0716.rs` → `error[E0716]: temporary value dropped while borrowed`. Önem: **Düşük**.

**`learn/smart-pointers/src/derinlesme.rs:406-409`** — "// ✗ let r = Rc::new(5); *r += 1; // E0594: cannot assign to data in an `Rc` // ✗ let v = Rc::new(vec![1]); v.push(2); // E0596: cannot borrow data in an `Rc` as mutable // Rc yalnız Deref uygular (DerefMut değil) ..."
Hüküm: **DOĞRU**. Kaynak: `/tmp/opencode/audit_e0594.rs`, `/tmp/opencode/audit_e0596.rs` — mesajlar birebir eşleşiyor; https://doc.rust-lang.org/std/rc/struct.Rc.html (Deref var, DerefMut yok). Önem: **Düşük**.

**`learn/smart-pointers/src/derinlesme.rs:412-420`** — "// ✗ let r = Rc::new(RefCell::new(1)); std::thread::spawn(move || {...}); // E0277: `Rc<RefCell<i32>>` cannot be sent between threads safely // ✗ Arc<RefCell<i32>> de olmaz: // E0277: `RefCell<i32>` cannot be shared between threads safely ... ✓ Çok thread'li karşılığı Arc<Mutex<T>>"
Hüküm: **DOĞRU**. Kaynak: `/tmp/opencode/audit_e0277.rs` iki E0277 mesajını birebir üretti. Önem: **Düşük**.

**`learn/smart-pointers/src/derinlesme.rs:260-264`** — "Cell<T> → değeri KOPYALAYARAK okur/yazar (get/set/replace/take). İçine referans vermez; dolayısıyla 'aynı anda iki borrow' diye bir durum oluşamaz, panik de olmaz. ... RefCell<T> → içine referans VERİR ... kuralı çalışma anında sayaçla denetler."
Hüküm: **DOĞRU**. Kaynak: https://doc.rust-lang.org/std/cell/struct.Cell.html ve https://doc.rust-lang.org/std/cell/struct.RefCell.html. Önem: **Düşük**.

**`learn/smart-pointers/src/derinlesme.rs:382`** — "false — borrow_mut() burada panik ederdi: "RefCell already borrowed""
Hüküm: **DOĞRU** (güncel mesaj). Kaynak: `guide/rust/library/core/src/cell.rs:919` + `/tmp/opencode/audit_repro.rs`. Önem: **Düşük**.

**`learn/smart-pointers/src/derinlesme.rs:557`** — "#[should_panic(expected = "already borrowed")]"
Hüküm: **DOĞRU**. Kaynak: https://doc.rust-lang.org/reference/attributes/testing.html §should_panic.expected — alt dizgi aranır; güncel mesaj "RefCell already borrowed" bu dizgiyi içerir (`audit_repro.rs`). Önem: **Düşük**.

**`learn/smart-pointers/src/derinlesme.rs:426-440`** — "Rc::make_mut(&mut rc) şunu yapar: - tek sahip bizsek → doğrudan &mut T verir (kopya yok), - başka sahipler varsa → veriyi klonlar, bizim Rc'yi yeni kopyaya çevirir, ona &mut verir." (ve `:458` "artık tek sahip → kopya YOK")
Hüküm: **DOĞRU**. Kaynak: https://doc.rust-lang.org/std/rc/struct.Rc.html#method.make_mut — "If there are other `Rc` pointers to the same value, then ... clone the inner value ... Otherwise ... mutate in place"; `T: Clone` gereksinimi vardır (kodda Vec<String> kullanıldığı için karşılanıyor). Önem: **Düşük**.

**`learn/smart-pointers/src/derinlesme.rs:321` ve `:327-328`** — "sinir.max(1) // 0 sınır, aşağıdaki yüzde hesabında sıfıra bölme paniği olurdu" / "debug derlemede u32 taşması panik, release'de sessiz sarma demektir. Toplamı doyurarak (saturating) artırıyor, çarpımı u64'te yapıyoruz."
Hüküm: **DOĞRU**. Kaynak: https://doc.rust-lang.org/reference/behavior-not-considered-unsafe.html#integer-overflow ("implementations must insert dynamic checks that panic on overflow" when debug assertions enabled; wrapping otherwise). Önem: **Düşük**.

**`learn/smart-pointers/ACIKLAMA.md:5,9`** — "Akıllı işaretçi ... Box tek sahibi büyük depoya (heap) koyar, Rc sahipliği birden çok okuyucuya paylaştırır, RefCell içeriden değiştirmeye izin verir, Weak ise sahip olmadan bakar. Her biri güvenliği başka yolla korur. İki veri birbirini gösterirse (döngü) Weak olmasa hiçbiri silinemez." / "İki defter birbirine 'beni oku' diye not bırakırsa ikisi de çöpe atılamaz; bu yüzden biri sadece 'nerede olduğunu' hatırlatır (Weak)."
Hüküm: **DOĞRU** (benzetme teknik iddiayla uyumlu). Kaynak: yukarıdaki Rc/Weak/döngü notlarının kaynakları + `guide/rust/library/alloc/src/rcs/rc.rs`. Önem: **Düşük**.

### Çıkarımlar

- RefCell panik mesajı konusunda dosya kendi içinde tutarsız: `:251-253` ve `derinlesme.rs:382` güncel mesajı ("RefCell already borrowed") doğru verirken, `main.rs:10-11` ve `main.rs:291` eski mesajı tekrar ediyor. Düzeltme, iki satırın güncel mesaja çekilmesinden ibaret.
- Kaynak yolları (`library/alloc/src/rcs/rc.rs`, `library/core/src/ops/{deref,drop}.rs`, `library/core/src/cell.rs`) yerel 1.99 ağacında gerçekten mevcut; özellikle `rcs/` yolu doğrulandığı için "MODASI GEÇMİŞ" sayılmamalı.
- Hata kodları ve mesajlar (E0072/E0040/E0594/E0596/E0277/E0716) birebir doğrulandı; bu alanda düzeltme gerekmiyor.

### Boşluklar

- "Böl. 5/8: Debug ile basmak alanları 'okundu' saymaz" ifadesi için hangi kitap bölümünün kastedildiği net değil; teknik iddia derleyici çıktısıyla doğrulandı, bölüm atfı DOĞRULANAMADI.
- Bölüm 4'te "drop elaboration"ın anlatıldığı yönündeki atfın hangi kaynağa dayandığı bilinmiyor; Book Bölüm 4'te böyle bir anlatı bulunamadı.

---

## 4. minigrep: ince main, stdout/stderr, çıkış kodu, env::args, dosya yolu düzeltme notu, bölüm atıfları, ACIKLAMA benzetmeleri

### Takeaway

minigrep'in tasarım gerekçeleri (lib.rs'te mantık, ince main, stdout'a sonuç / stderr'e hata, `process::exit(1)`, iterator sahipliği) doğru. `lib.rs:5-6` dosya yolu düzeltme notu bağımsız olarak doğrulandı ve DOĞRU. İki nüans: üstteki "Çalıştır:" komutu hâlâ workspace kökünde çalışmayan çıplak `siir.txt` kullanıyor; "`main` test edilemez" ifadesi mutlak anlamda yanıltıcı.

### Denetim notları

**`learn/minigrep/src/lib.rs:5-6`** — "↳ DÜZELTME NOTU (son gözden geçirme): workspace kökünden (rust-study/) çalıştırırken dosya yolu `learn/minigrep/siir.txt` olmalıdır; `siir.txt` yalnız learn/minigrep dizininden çalışır."
Hüküm: **DOĞRU** (bağımsız doğrulandı). Kaynak: `rust-study/Cargo.toml:1` (`[workspace]`, kök = `rust-study`), `rust-study/learn/minigrep/siir.txt` mevcut, workspace kökünde `siir.txt` YOK; `/tmp/opencode/cwdtest/` mini repro'sunda `cargo run -p a` çalıştırılan dizini (`/tmp/opencode/cwdtest`) current_dir olarak verdi — yani `cargo run -p` çalışma dizinini crate dizinine değiştirmez. Önerilen düzeltme: yok; not doğru. Önem: **Kritik** (doğrulama başarılı, değişiklik gerekmiyor).

**`learn/minigrep/src/lib.rs:3`** — "// Çalıştır:  cargo run -p minigrep -- hiç siir.txt" (ve `:4` IGNORE_CASE örneği)
Hüküm: **YANILTICI** (kök dizinden kopyalanırsa çalışmaz; `:5-6` notu bunu zaten söylüyor). Kaynak: yukarıdaki ile aynı (workspace kökü, dosya konumu, cwd repro'su). Önerilen düzeltme: komutları `cargo run -p minigrep -- hiç learn/minigrep/siir.txt` ve `IGNORE_CASE=1 cargo run -p minigrep -- NE learn/minigrep/siir.txt` yapın ya da satıra "(learn/minigrep dizininden)" şerhi düşün. Önem: **Orta**.

**`learn/minigrep/src/main.rs:9-11`** — "// Mini örnek (rust-study kökünden): cargo run -p minigrep -- hiç learn/minigrep/siir.txt // ✓ ... cargo run -p minigrep -- hiç // ✗ "dosya yolu verilmedi", kod 1"
Hüküm: **DOĞRU** (doğru yolu kullanan örnek). Kaynak: aynı workspace/cwd kanıtları + `Config::build` kodu (`src/lib.rs:86-89`). Önem: **Düşük**.

**`learn/minigrep/src/lib.rs:9-11`** — "Bu bölüm YENİ kavram öğretmez; önceki bölümleri (Result/?, lifetime, trait, closure/iterator, test) GERÇEK bir komut satırı aracında birleştirir. Ders: mantığı lib.rs'e koy (test edilebilir), main.rs'i ince tut (argüman + hata gösterimi + çıkış kodu)."
Hüküm: **DOĞRU**. Kaynak: https://doc.rust-lang.org/book/ch11-03-test-organization.html son paragraf ("one of the reasons Rust projects that provide a binary have a straightforward src/main.rs file that calls logic that lives in the src/lib.rs file") + Book ch12 (I/O projesi). Önem: **Düşük**.

**`learn/minigrep/src/main.rs:7-8`** — "Neden bu kadar kısa? `main` test edilemez; içinde ne kadar az iş olursa, test edilmemiş kod o kadar az olur."
Hüküm: **YANILTICI** (mutlak ifade). Kaynak: https://doc.rust-lang.org/book/ch11-03-test-organization.html — doğru olan ifade: entegrasyon testlerinden `use minigrep::...` ile `main.rs` içindeki fonksiyonlar **scope'a getirilemez**; "that small amount of code in the src/main.rs file will work as well, and that small amount of code doesn't need to be tested". Ancak binary crate içindeki bir `#[test]` doğrudan `main()` çağrabilir ve entegrasyon testi binary'yi alt süreç olarak çalıştırabilir. Önerilen düzeltme: "`main` entegrasyon testine doğrudan import edilemez; bu yüzden test edilebilir mantığı lib.rs'e taşıyıp main'i ince tutarız." Önem: **Düşük-Orta**.

**`learn/minigrep/src/lib.rs:215-218`** — "• stdout vs stderr: ARAMA SONUÇLARI println! ile stdout'a; HATA ve tanı mesajları eprintln! ile stderr'e gider (main'de) ... `minigrep ... > cikti.txt` ile sonucu dosyaya yönlendirince hata mesajları da dosyaya karışırdı ... Unix sözleşmesi."
Hüküm: **DOĞRU**. Kaynak: https://doc.rust-lang.org/std/macro.eprintln.html + https://doc.rust-lang.org/book/ch12-00-an-io-project.html (minigrep örneklerinde hatalar stderr'e yazılır). Önem: **Düşük**.

**`learn/minigrep/src/lib.rs:220-221`** — "• Çıkış kodu: hata olunca process::exit(1) ile SIFIR OLMAYAN kodla çıkılır (main.rs). Kabuk ve betikler "başarılı mı?" sorusunu bu kodla yanıtlar; panik/stack-unwind yerine temiz çıkış."
Hüküm: **DOĞRU**. Kaynak: https://doc.rust-lang.org/std/process/fn.exit.html ("terminate the current process ... with the given exit code"); `src/main.rs:25,31` kodla uyumlu. Önem: **Düşük**.

**`learn/minigrep/src/lib.rs:204-207`** — "• env::args() bir ITERATOR döndürür ... (args(), geçersiz Unicode argümanda panik eder; env::args_os() OsString verir — burada gerek yok.)"
Hüküm: **DOĞRU**. Kaynak: https://doc.rust-lang.org/std/env/fn.args.html §Panics — "The returned iterator will panic during iteration if any argument to the process is not valid Unicode. If this is not desired, use the args_os function instead." Önem: **Düşük**.

**`learn/minigrep/src/lib.rs:77-78`** — "// Argüman ITERATOR'ı alır (Bölüm 13 kalıbı): sahipliği devralır, klonlamaya gerek kalmaz. // Hata olursa panic ETMEZ; Result döner → main düzgün bir mesajla çıkabilir."
Hüküm: **DOĞRU**. Kaynak: https://doc.rust-lang.org/book/ch13-00-functional-features.html (Bölüm 13 = Iterators and Closures) + kod (`src/lib.rs:79`, `src/main.rs:21` — `Config::build(env::args())`). Önem: **Düşük**.

**`learn/minigrep/src/lib.rs:111-112` ve `:209-213`** — "Asıl iş. Box<dyn Error> = "Error uygulayan HERHANGİ bir hata" (trait object, Bölüm 10) ... ? operatörü From ile bunlara otomatik çevirir (Bölüm 9 + Bölüm 10 dyn) ... Bir kütüphane genelde kendi hata enum'ını verir (çağırana tür bilgisini korur)."
Hüküm: **DOĞRU**. Kaynak: https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html ve ch10-02-traits.html (trait objects, `Box<dyn Error>`); `?` dönüşümü için https://doc.rust-lang.org/reference/expressions/operator-expr.html#the-question-mark-operator (From::from). Önem: **Düşük**.

**`learn/minigrep/src/lib.rs:35-36,139-140`** — "lifetime ('a) → bir referansın geçerli kaldığı bölge (Böl. 10)." / "Box<dyn Error> → ... (Böl. 9 ve 10)." / "Lifetime 'a (Bölüm 10): dönen diliml(er) `icerik`ten gelir, o yüzden çıktı icerik kadar yaşar. `sorgu`nun süresiyle ilişkisi yok — bu yüzden yalnız icerik'e 'a bağlanır."
Hüküm: **DOĞRU**. Kaynak: https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html (Bölüm 10 = Generic Data Types, Traits, Lifetimes) + imza `pub fn ara<'a>(sorgu: &str, icerik: &'a str) -> Vec<&'a str>` (`src/lib.rs:141`). Önem: **Düşük**.

**`learn/minigrep/src/lib.rs:2`** — "// Rust Book — Bölüm 12: An I/O Project — `minigrep` (küçük bir grep)"
Hüküm: **DOĞRU**. Kaynak: https://doc.rust-lang.org/book/ch12-00-an-io-project.html. Önem: **Düşük**.

**`learn/writing-tests/src/lib.rs:2` / `learn/smart-pointers/src/main.rs:2` / `learn/smart-pointers/src/derinlesme.rs:2`** — "Rust Book — Bölüm 11: Writing Automated Tests" / "Rust Book — Bölüm 15: Smart Pointers (akıllı işaretçiler)" / "Derinleşme — Bölüm 15: Smart Pointers ve Interior Mutability"
Hüküm: **DOĞRU**. Kaynak: https://doc.rust-lang.org/book/ch11-01-writing-tests.html, https://doc.rust-lang.org/book/ch15-00-smart-pointers.html, https://doc.rust-lang.org/book/ch15-05-interior-mutability.html. Önem: **Düşük**.

**`learn/minigrep/src/lib.rs:162-163`** — "cargo test -p minigrep            // üç testi çalıştırır" / "cargo test -p minigrep duyarsiz   // yalnız adında "duyarsiz" geçen testi çalıştırır"
Hüküm: **DOĞRU**. Kaynak: `src/lib.rs:165-193` (üç `#[test]`: harf_duyarli_tek_sonuc, harf_duyarsiz_iki_sonuc, eksik_arguman_hata_doner) + https://doc.rust-lang.org/book/ch11-02-running-tests.html (substring filtre). Önem: **Düşük**.

**`learn/minigrep/src/lib.rs:173` ve `:136`** — "// "hiç" küçük harfle yalnız ilk satırda geçer ("Hiç" eşleşmez)." / "ara("hiç", icerik) // ["Ben bir hiçim"] — "Hiç" büyük harfli, uymaz"
Hüküm: **DOĞRU** (`str::contains` büyük/küçük harfe duyarlı). Kaynak: https://doc.rust-lang.org/std/primitive.str.html#method.contains + `src/lib.rs:144` (`.filter(|satir| satir.contains(sorgu))`). Önem: **Düşük**.

**Dört ACIKLAMA.md'nin ortak `:11` satırı** (`learn/minigrep`, `learn/writing-tests`, `learn/macros`, `learn/smart-pointers`) — "> Not: Kod mantığı değiştirilmedi. Değişiklikleri `cargo check ile doğrula`."
Hüküm: **DOĞRU** (içerik) / biçim hatası. Backtick'ler `cargo check ile doğrula` ifadesinin tamamını kod olarak gösteriyor; yalnızca `cargo check` kod olmalı. Kaynak: dört dosyanın `:11` satırı (doğrudan okuma). Önerilen düzeltme: "Değişiklikleri `cargo check` ile doğrula." Önem: **Düşük**.

**ACIKLAMA.md benzetmeleri (genel):** minigrep `:5,9` danışma masası/kapı görevlisi; writing-tests `:5,9` oyuncak atölyesi/sınav; macros `:5,9` lastik damga/atölye; smart-pointers `:5,9` okul kutuları/defter notları.
Hüküm: **DOĞRU** — her benzetme ilgili `.rs` dosyasındaki "SADE ANLATIM" kutularıyla aynı eğretilemeyi kullanıyor ve teknik iddia içermiyor (tek istisna yukarıdaki smart-pointers döngü/Weak benzetmesi, o da DOĞRU). Kaynak: dört ACIKLAMA.md ve ilgili kaynak dosya yorumları (doğrudan okuma). Önem: **Düşük**.

### Çıkarımlar

- Dosya yolu düzeltme notu (`lib.rs:5-6`) gerçekten doğru: workspace kökü, dosyanın yeri ve cargo'nun cwd davranışı üç ayrı kanıtla uyuşuyor. Buna karşılık aynı dosyanın `:3-4` "Çalıştır:" satırları hâlâ çıplak `siir.txt` diyor; öğrenci üstteki komutu kopyalarsa hata alır.
- "main test edilemez" ifadesi, Book'un "main'i doğrudan entegrasyon testine import edemezsin" önermesinin gereğinden fazla genelleştirilmiş hâli.

### Boşluklar

- "`main` test edilemez" ifadesinin bir `#[test]` içinden `main()` çağrılması yoluyla çürütülebileceği iddiası için ayrı bir mini repro çalıştırılmadı; Book ch11-03'ün birebir ifadesi ve dil kuralları temel alındı. Bu nedenle hüküm YANLIŞ değil, YANILTICI olarak verildi.
- `ACIKLAMA.md` dosyalarındaki "Kod mantığı değiştirilmedi" ifadesinin geçmişte hangi değişikliğe gönderme yaptığı bu kapsamda doğrulanamadı (tarihçe/sunum notu).

---

## Sorunsuz alanlar (ek olarak doğrulanan iddialar)

Aşağıdaki noktalar ayrı birer hüküm gerektirmeyecek kadar net ve doğrudur; yukarıdaki bölümlerde ilgili kaynaklar verilmiştir:

- `learn/macros/src/lib.rs:50-52` — `#[proc_macro] pub fn cevap_uret(_girdi: TokenStream) -> TokenStream` imzası ve `"fn cevap() -> u32 { 42 }".parse().unwrap()` kullanımı Reference'ın örneğiyle birebir uyumlu.
- `learn/macros/tests/kullanim.rs:17-24` — `use macros::cevap_uret; cevap_uret!(); #[test] fn cevap_42_doner()` akışı geçerli; makro tanımlandığı crate dışında genişliyor.
- `learn/writing-tests/src/main.rs:4-6,10-16` — vitrin ikilinin lib.rs fonksiyonlarını çağırması ve "asıl olay testlerde" ifadesi tutarlı.
- `learn/minigrep/src/main.rs:21-32` — `unwrap_or_else` + `eprintln!` + `process::exit(1)` ve `if let Err(e) = run(config)` akışı, yorumlardaki stdout/stderr/çıkış kodu gerekçesiyle uyumlu.
- `learn/minigrep/src/lib.rs:92` — `env::var("IGNORE_CASE").is_ok()` yorumu ("varsa harf duyarsız ara") doğru; `is_ok()` değişken tanımlı ve geçerli Unicode ise true döner.
- `learn/smart-pointers/src/main.rs:194-198,209-218` — Rc strong_count beklentileri (1 → 2 → 3 → 2) kodun sahiplik akışıyla uyumlu.
- `learn/smart-pointers/src/derinlesme.rs:130-135` — `Rc::downgrade` ile güçlü sayaca dokunulmaması, `RefCell` üzerinden `&self` ile atama, `cocuk_ekle` akışı doğru.
- `learn/smart-pointers/src/derinlesme.rs:241-245` — döngüde sayaçların 1'de takılması ve Weak ile zincirin çözülmesi adım adım açıklaması doğru.
- `learn/smart-pointers/src/derinlesme.rs:565-575` — `Rc::as_ptr` ile `make_mut` öncesi/sonrası adres karşılaştırması testi, `make_mut` semantiğiyle uyumlu.
