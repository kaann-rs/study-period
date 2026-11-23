# İleri Tip Sistemi Dersleri — Teknik Doğruluk Denetimi (5 klasör)

Kapsam: `learn/advanced-concepts/` altındaki `ileri-lifetime-hrtb-gat`, `ileri-variance`,
`ileri-sized-dst`, `ileri-coherence`, `ileri-deref-metot` klasörlerinin `src/main.rs` ve
`ACIKLAMA.md` dosyaları. `learn/` altında **hiçbir değişiklik yapılmadı**.

Doğrulama ortamı: `rustc 1.99.0 (b940084d7 2026-09-28)`, `cargo 1.99.0`,
`x86_64-unknown-linux-gnu`. `cargo check` (5 crate) ve `cargo test` (7 test) temiz;
`cargo run` çıktıları ve `/tmp/opencode/rustrepro/*.rs` mini reproları hata kodlarını/mesajlarını
doğruladı. Yerel rustc kaynak ağacı: `guide/rust`.

---

## 1. Rust for Rustaceans bölüm / altbaşlık / listing atıfları doğru mu?

### Takeaway
Dört dosyadaki "DÜZELTME NOTU"nun **bölüm numaraları doğru** (yayıncının ayrıntılı içindekiler
listesiyle doğrulandı); ancak notların hemen üstündeki orijinal `// Kaynak:` satırları **yanlış
numarayı taşımaya devam ediyor** — okuyucu aynı dosyada hem yanlış atıf hem düzeltmesini görüyor.
Listing numaraları ve "Deref and Inherent Methods" kutusu içindekiler listesinde yer almadığı için
o iddialar DOĞRULANAMADI.

### Cited Findings

- **`learn/advanced-concepts/ileri-variance/src/main.rs:5`** — Alıntı: `// Kaynak: Rust for Rustaceans (Jon Gjengset), Böl. 2 "Lifetime Variance" (Listing 2-11).` — **Hükm: YANLIŞ** — **Kaynak:** yayıncı ayrıntılı İçindekiler PDF'si (`/tmp/opencode/rfr_toc.txt:13-25`): Böl. 1 = FOUNDATIONS, "Borrowing and Lifetimes → Lifetimes" s.12; Böl. 2 = TYPES (`:27-28`) — **Öneri:** `Kaynak: RfR Böl. 1 (Foundations → Borrowing and Lifetimes → Lifetimes)`; listing numarasını kaldır veya "doğrulanamadı" de — **Önem: ORTA**.
- **`ileri-variance/src/main.rs:6-8`** — Alıntı: `// ↳ DÜZELTME NOTU (son gözden geçirme): kitapta bu konu Böl. 1'dedir (Foundations → //   Borrowing and Lifetimes → Lifetimes). Yayıncının ayrıntılı içindekiler listesiyle doğrulandı. //   Listeler bölüme göre numaralandığından buradaki "Listing 2-11" aslında Listing 1-11'dir.` — **Hükm: KISMEN DOĞRU / KISMEN DOĞRULANAMADI** — bölüm 1 ve "Borrowing and Lifetimes → Lifetimes" yolu TOC'ta var (`rfr_toc.txt:20-24`); fakat TOC'ta "Lifetime Variance" adlı bir altbaşlık **yok** ve listing numaraları hiç listelenmiyor. "Listing 1-11'dir" çıkarımı TOC ile doğrulanamaz. — **Kaynak:** `/tmp/opencode/rfr_toc.txt:13-25` — **Öneri:** "Listing 1-11" iddiasını çıkar; "kitabın Lifetime'lar bölümündeki variance tartışması (kesin listing no. TOC'ta yer almıyor)" de — **Önem: ORTA**.
- **`ileri-variance/src/main.rs:130`** — Alıntı: `// 2. INVARIANCE + senin örneğin:  &'a mut &'b str  (RfR Listing 2-11)` — **Hükm: YANLIŞ / DOĞRULANAMADI** (bayat listing numarası; TOC doğrulayamıyor) — **Kaynak:** aynı TOC — **Öneri:** `(RfR Böl. 1, Lifetimes)` — **Önem: DÜŞÜK**.
- **`ileri-sized-dst/src/main.rs:5`** — Alıntı: `// Kaynak: Rust for Rustaceans Böl. 3 "Dynamically Sized Types and Fat Pointers" + std.` — **Hükm: YANLIŞ** — hem bölüm (3 yerine 2) hem başlık ("Fat" yerine "Wide") yanlış — **Kaynak:** `rfr_toc.txt:33`: "Dynamically Sized Types and Wide Pointers . . 23", Böl. 2 = TYPES — **Öneri:** `Kaynak: RfR Böl. 2 "Dynamically Sized Types and Wide Pointers" + std.` — **Önem: ORTA**.
- **`ileri-sized-dst/src/main.rs:6-8`** — Alıntı: `// ↳ DÜZELTME NOTU (son gözden geçirme): kitapta bu başlık Böl. 2'dedir (Types) ve adı //   "Dynamically Sized Types and Wide Pointers"tır; "fat pointer" ile "wide pointer" aynı //   şeyin iki adıdır. Yayıncının ayrıntılı içindekiler listesiyle doğrulandı.` — **Hükm: DOĞRU** — **Kaynak:** `rfr_toc.txt:33`; terim eşitliği için `guide/rust/library/core/src/ops/unsize.rs:10` ("fat pointer") ve `guide/rust/library/core/src/byte_str/mod.rs:27` ("wide pointer") — **Öneri:** yok — **Önem: DÜŞÜK**.
- **`ileri-coherence/src/main.rs:5`** — Alıntı: `// Kaynak: Rust for Rustaceans Böl. 3 ("Coherence and the Orphan Rule") + std.` — **Hükm: YANLIŞ** (Böl. 2 olmalı) — **Kaynak:** `rfr_toc.txt:34,37`: "Traits and Trait Bounds → Coherence and the Orphan Rule . . 28", Böl. 2 — **Öneri:** `Kaynak: RfR Böl. 2 ("Coherence and the Orphan Rule") + std.` — **Önem: ORTA**.
- **`ileri-coherence/src/main.rs:6-7`** — Alıntı: `// ↳ DÜZELTME NOTU (son gözden geçirme): kitapta bu başlık Böl. 2'dedir (Types → Traits and //   Trait Bounds). Yayıncının ayrıntılı içindekiler listesiyle doğrulandı.` — **Hükm: DOĞRU** — **Kaynak:** `rfr_toc.txt:34,37` — **Öneri:** yok — **Önem: DÜŞÜK**.
- **`ileri-deref-metot/src/main.rs:5`** — Alıntı: `// Kaynak: Rust for Rustaceans Böl. 4 ("Deref and Inherent Methods" kutusu) + The Rust Reference` — **Hükm: YANLIŞ** — kitabın 4. bölümü ERROR HANDLING'tir — **Kaynak:** `rfr_toc.txt:64-65` — **Öneri:** Böl. 3 (Designing Interfaces) veya kutu adını doğrulanamadı olarak işaretle — **Önem: ORTA**.
- **`ileri-deref-metot/src/main.rs:8-10`** — Alıntı: `// ↳ DÜZELTME NOTU (son gözden geçirme): kitabın 4. bölümü Error Handling'dir; "Deref and //   Inherent Methods" kutusu Böl. 3'tedir (Designing Interfaces → Wrapper Types). Bölüm listesi //   yayıncının içindekiler sayfasıyla doğrulandı; kutu orada ayrıca listelenmiyor.` — **Hükm: KISMEN DOĞRU** — 4. bölüm = Error Handling ✓, Böl. 3 = Designing Interfaces → Wrapper Types ✓ (`rfr_toc.txt:43-49`); "kutu orada ayrıca listelenmiyor" cümlesi doğru ve dürüst — dolayısıyla kutunun adı/yeri TOC'tan DOĞRULANAMADI. — **Kaynak:** `rfr_toc.txt:43-65` — **Öneri:** "Böl. 3'teki Wrapper Types bölümünde ele alınır (kutunun tam adı TOC'ta geçmiyor)" de — **Önem: DÜŞÜK**.
- **`ileri-deref-metot/src/main.rs:5-7`** — Alıntı: `Böl. 5 (alıcılar) ve Böl. 15 (Deref) burada birleşiyor` — **Hükm: DOĞRU ama YANILTICI SUNUM** — bunlar kurs içi bölüm numaraları (README:34 = `learn/structs`, README:43 = `learn/smart-pointers`); aynı "Kaynak" satırında RfR bölüm numaralarıyla yan yana durduğu için karışıklık riski var. — **Kaynak:** `README.md:34,43` — **Öneri:** "(kurs bölümleri)" etiketi ekle — **Önem: DÜŞÜK**.
- **`ileri-lifetime-hrtb-gat/src/main.rs:7`** — Alıntı: `// Rust for Rustaceans Böl. 1–2 (Foundations, Types).` — **Hükm: DOĞRU** — **Kaynak:** `rfr_toc.txt:13-14,27-28` ve `README.md:75` ("Son iki satır ... doğrudur") — **Öneri:** yok — **Önem: DÜŞÜK**.
- **`README.md:62-67`** (kapsam dışı ama rapora değer) — tabloda hâlâ Böl. 2/3/4 yazıyor; `README.md:71-75` düzeltme notu bunu prose'da düzeltiyor. Aynı "not ekle, eskisini bırak" deseni. — **Hükm: TUTARSIZ** — **Öneri:** tabloyu da güncelle — **Önem: ORTA**.

### Inferences
- Düzeltme notları "son gözden geçirme"de eklenmiş; orijinal `Kaynak:` satırları düzenlenmemiş. Bu, dersi okuyan için çelişkili iki bilgi üretiyor; en temiz çözüm eski satırları da düzeltmektir.
- TOC erişilebilir durumda (`https://nostarch.com/download/samples/Rust_CID.pdf` → `pdftotext`); O'Reilly önizlemesi 403 döndüğü için kitap metninin kendisi okunamadı.

### Gaps
- Listing 1-11 / 2-11 numaraları: TOC listing listelemiyor → DOĞRULANAMADI.
- "Lifetime Variance" altbaşlığı ve "Deref and Inherent Methods" kutusunun tam adı: TOC'ta yok → DOĞRULANAMADI.
- "Listeler bölüme göre numaralandığından..." gerekçesi genel olarak doğru bir yayınevi kuralı olsa da bu başlık özelinde kanıtlanamadı.

---

## 2. HRTB (`for<'a>`) "kim seçer" anlatımı ve GAT iddiaları teknik olarak doğru mu?

### Takeaway
HRTB/GAT ile ilgili teknik iddialar ve **bütün ✗ hata kodu/mesaj iddiaları rustc 1.99 ile birebir doğrulandı**. Tek anlam sorunu, bir bölüm başlığının "süreyi çağrılan seçer" demesi (gövdede doğru anlatılmasına rağmen) ve "aynı makine koduna iner" ifadesinin garanti edilemez olması.

### Cited Findings

- **`ileri-lifetime-hrtb-gat/src/main.rs:16-18`** — Alıntı: `//   Bir generic parametreyi KİM seçer? fn f<'a>(..) → 'a'yı ÇAĞIRAN seçer; ... F: for<'a> Fn(&'a T) → 'a'yı F'yi //   ÇAĞIRAN (yani biz, gövdenin içinden) seçeriz; F her süreye razı olmak zorundadır. HRTB bu "kim seçer" //   sorusunu tersine çevirmenin sözdizimidir.` — **Hükm: DOĞRU** — **Kaynak:** https://doc.rust-lang.org/reference/trait-bounds.html#higher-ranked-trait-bounds ("bounds ... true *for all* lifetimes"; `call_on_ref_zero` örneği tam bu durumu gösterir) — **Önem: DÜŞÜK**.
- **`ileri-lifetime-hrtb-gat/ACIKLAMA.md:5`** — Alıntı: `for<'a>` (HRTB) tersini söyler: süreyi çağrılanın içinden biz seçeriz, karşı taraf her süreye razı olur. — **Hükm: DOĞRU** (F'nin kendisi süreyi seçmez; F her süreye razıdır) — **Kaynak:** aynı Reference bölümü — **Önem: DÜŞÜK**.
- **`ileri-lifetime-hrtb-gat/src/main.rs:78`** — Alıntı: `println!("\n== 2) HRTB: for<'a> — süreyi çağrılan seçer ==");` — **Hükm: YANILTICI (hafif)** — dosyanın 16-18. satırları ve ACIKLAMA.md:5 "biz seçeriz, F razıdır" derken bu başlık "çağrılan (F) seçer" okunabilir. — **Kaynak:** Reference HRTB; dosya içi :16-18 — **Öneri:** `HRTB: for<'a> — süreyi F'yi çağıran seçer, F her süreye razıdır` — **Önem: ORTA**.
- **`src/main.rs:266-271`** — Alıntı: `F: Fn(&str) -> &str` / `F: for<'a> Fn(&'a str) -> &'a str` "iki imza birebir aynıdır" — **Hükm: DOĞRU** — **Kaynak:** Reference `bound.higher-ranked.trait` (`where F: for<'a> Fn(&'a i32)` eşdeğerliği) — **Önem: DÜŞÜK**.
- **`src/main.rs:298-299`** — Alıntı: `// Fonksiyon işaretçisi türlerinde de aynı: fn(&str) -> &str = for<'a> fn(&'a str) -> &'a str.` — **Hükm: DOĞRU** — **Kaynak:** Reference HRTB + `guide/rust/library/core/src/ops/function.rs` (Fn ailesi) — **Önem: DÜŞÜK**.
- **`src/main.rs:257-259`** — Alıntı: `f(&yerel).len()  // ✗ E0597: `yerel` does not live long enough` — **Hükm: DOĞRU** — **Kaynak:** `/tmp/opencode/rustrepro/uygula.rs` → E0597, tam mesaj — **Önem: DÜŞÜK**.
- **`src/main.rs:302-303`** — Alıntı: `// let kirp = |s: &str| s.trim();  // ✗ error: lifetime may not live long enough` — **Hükm: DOĞRU** — **Kaynak:** `rustrepro/closure.rs` → kodsuz "lifetime may not live long enough" — **Önem: DÜŞÜK**.
- **`src/main.rs:304`** — Alıntı: `// Closure'larda dönüş için elision kuralı İŞLEMEZ: derleyici girdi ile çıktıya iki ayrı Süre atar ve aralarında bağ kuramaz.` — **Hükm: DOĞRU** — **Kaynak:** `rustrepro/closure.rs` çıktısı — **Önem: DÜŞÜK**.
- **`src/main.rs:333-338`** — Alıntı: `serde bunun için DeserializeOwned = for<'de> Deserialize<'de> der` — **Hükm: DOĞRU** — **Kaynak:** https://docs.rs/serde/latest/serde/de/trait.DeserializeOwned.html (`pub trait DeserializeOwned: for<'de> Deserialize<'de> {}`) — **Önem: DÜŞÜK**.
- **`src/main.rs:397-399`** — Alıntı: `✗ error: implementation of `Coz` is not general enough` + iki `note:` satırı — **Hükm: DOĞRU** (mesaj metni birebir) — **Kaynak:** `rustrepro/coz.rs` — **Önem: DÜŞÜK**.
- **`src/main.rs:411-412`** — Alıntı: `(std'de `windows` var ama `windows_mut` YOK)` — **Hükm: DOĞRU** — **Kaynak:** `guide/rust/library/core/src/slice/mod.rs:1092-1095` ("there is no `windows_mut` analog to `windows`") — **Önem: DÜŞÜK**.
- **`src/main.rs:442-443`** — Alıntı: `` `where Self: 'a` zorunludur (yazmazsan: "error: missing required bound on `Oge`") `` — **Hükm: DOĞRU** — **Kaynak:** `rustrepro/gatbound.rs` → "error: missing required bound on `O`" + `where Self: 'a` help + issue #87479 — **Önem: DÜŞÜK**.
- **`src/main.rs:513-515`** — Alıntı: `✗ E0038: the trait `OduncVerenIterator` is not dyn compatible` — **Hükm: DOĞRU** — **Kaynak:** `rustrepro/gatdyn.rs` → E0038 "...because it contains generic associated type `O`"; ayrıca https://blog.rust-lang.org/2022/10/28/gats-stabilization/ ("Traits with GATs are not object safe") — **Önem: DÜŞÜK**.
- **`src/main.rs:670-673`** — Alıntı: `GAT (1.65'ten beri kararlı) ... where Self: 'a şartının elle yazdırılması, ileride varsayılan hâle getirilebilmesi için konmuş geçici bir zorunluluktur (hata mesajındaki issue #87479).` — **Hükm: DOĞRU** — **Kaynak:** GAT stabilizing blog (1.65, "requiring these bounds now to leave room in the future to potentially imply these automatically") + derleyici notu — **Önem: DÜŞÜK**.
- **`src/main.rs:691-698`** (alıştırma 2) — Alıntı: `Trait derlenir ama çağrı E0597 verir; notu oku: "due to a current limitation of the type system, this implies a `'static` lifetime" ... for<'a> HER 'a'yı kapsar, where Self: 'a ise her 'a için Self: 'a ister → Self: 'static` — **Hükm: DOĞRU** — **Kaynak:** `rustrepro/ex2.rs` → E0597 + söz konusu not; GAT blog ("Implied `'static` requirement from higher-ranked trait bounds") — **Önem: DÜŞÜK**.
- **`src/main.rs:632-634`** — Alıntı: `E0521  "borrowed data escapes outside of function" ... thread::spawn, global, Box<dyn Trait>` — **Hükm: DOĞRU** — **Kaynak:** `rustrepro/e0521.rs` → E0521 tam mesaj — **Önem: DÜŞÜK**.
- **`src/main.rs:607`** — Alıntı: `✗ E0277: `Rc<String>` cannot be sent between threads safely` — **Hükm: DOĞRU** — **Kaynak:** `rustrepro/rcsend.rs` — **Önem: DÜŞÜK**.
- **`src/main.rs:689`** — Alıntı: `` `metin: &str` bırakırsan E0621 alırsın `` — **Hükm: DOĞRU** — **Kaynak:** `rustrepro/e0621.rs` → E0621 "explicit lifetime required in the type of `metin`" — **Önem: DÜŞÜK**.
- **`src/main.rs:666-668`** — Alıntı: `` `for<'a>` yalnız LIFETIME üzerinde niceler; "her T türü için" diyen `for<T>` kararlı Rust'ta yoktur. `` — **Hükm: DOĞRU** — **Kaynak:** Reference "Trait bounds may be *higher ranked* over lifetimes" — **Önem: DÜŞÜK**.
- **`src/main.rs:659-663`** — Alıntı: "`fn f<'a>(x: &'a str)` içindeki 'a ... serbest kalır ('late-bound') — bu yüzden `f`, tek bir değerken `for<'a> Fn(&'a str)` bound'unu sağlar" — **Hükm: DOĞRU** — **Kaynak:** Reference HRTB + Rustonomicon HRTB; `rustrepro/uygula.rs` ile dolaylı doğrulama — **Önem: DÜŞÜK**.
- **`src/main.rs:676-678`** — Alıntı: `PencereMut::sonraki, elle yazılmış pointer aritmetiğiyle aynı makine koduna iner; kazandığımız şey hız değil, yanlış kullanımın derlenmemesidir.` — **Hükm: YANILTICI** — erasure doğru; "aynı makine koduna iner" garanti edilmez (kod üretimi optimizasyona/LLVM'e bağlı; Rust dil garantisi değildir). — **Kaynak:** erasure: Reference/general Rust semantics; "aynı kod" iddiası için kaynak yok — **Öneri:** `"lifetime denetimi derleme öncesinde kalkar; çalışma zamanında ek bir kontrol veya maliyet eklenmez"` de — **Önem: ORTA**.
- **`src/main.rs:22`** — Alıntı: `// ✗ işaretli derlenmeyen örneklerin hata kodları/mesajları rustc 1.99 ile doğrulandı.` — **Hükm: DOĞRU** — yukarıdaki 12 ✗ örneğinin tamamı reprolandı (E0597 ×2, E0309, E0499, E0038, E0277, E0521, E0621, E0596, E0117, E0119 + kodsuz mesajlar). — **Önem: DÜŞÜK**.

### Inferences
- "Kim seçer" çerçevesi pedagojik olarak sağlam ve Reference ile uyumlu; tek risk kısa başlıktaki ölçülü bozulma.
- GAT bölümü, derleyici mesajlarını neredeyse birebir aktardığı için en güvenilir kısımlardan biri.

### Gaps
- Yok (bu kapsamda teknik bir doğrulama boşluğu kalmadı).

---

## 3. Variance ve `&'s mut &'a str` (E0502/E0597) anlatımı doğru mu?

### Takeaway
Covariance/invariance/contravariance tanımları ve iki-lifetime vs tek-lifetime gerekçesi **doğru**; reprolar tek lifetime ile E0502, iki lifetime ile derleme başarısını gösteriyor. Sadece "Rust Book'ta neredeyse yok" ifadesi öznel/kaynaksız.

### Cited Findings

- **`ileri-variance/src/main.rs:12-14`** — Alıntı: `// Doğrulanmış (rustc ile): MutStr iki lifetime ile DERLENİR, tek lifetime ile E0502; &mut // ardındaki tür invariant olduğu için uzun-ömürlü değer kısaltılamaz (E0597); paylaşımlı & ise // covariant olduğu için kabul edilir.` — **Hükm: DOĞRU** — **Kaynak:** `rustrepro/iki_lifetime.rs` (derlenir), `tek_lifetime.rs` (E0502), `mutvec2.rs` (E0597), `oku(&depo)` örneği crate içinde derlenir — not: imza biçimi `fn sakla(liste: &mut Vec<&str>, ...)` kodsuz "lifetime may not live long enough" verir; dosya bunu :196-198'de doğru aktarıyor, :12-14'teki E0597 çağıran taraf senaryosuna aittir — **Önem: DÜŞÜK**.
- **`src/main.rs:62-65`** — Alıntı: `'b: 'a ('b, 'a'yı KAPSAR / daha uzun yaşar) ise 'b, 'a'nın ALT-TİPİdir ... Uç örnek: 'static her 'a'nın alt-tipidir.` — **Hükm: DOĞRU** — **Kaynak:** https://doc.rust-lang.org/nomicon/subtyping.html; ayrıca Reference `'a: 'b` tanımı (`bound.lifetime.outlive-lifetime`) — **Önem: DÜŞÜK**.
- **`src/main.rs:97-98`** — Alıntı: `&'a T, hem 'a'da hem T'de COVARIANT'tır ... Bu yüzden &Vec<&'static str>, &Vec<&'a str> beklenen yere geçer.` — **Hükm: DOĞRU** — **Kaynak:** Nomicon subtyping; `oku(&depo)` çağrısı `cargo check` ile derlendi — **Önem: DÜŞÜK**.
- **`src/main.rs:132-135`** — Alıntı: `&mut T, T'de INVARIANT'tır ... mutability sağlayan her tür (&mut T, Cell<T>, RefCell<T>...) invariant'tır.` — **Hükm: DOĞRU** — **Kaynak:** Nomicon subtyping + `rustrepro/mutvec.rs` ("mutable references are invariant over their type parameter") — **Önem: DÜŞÜK**.
- **`src/main.rs:177-190`** — Alıntı: `İki lifetime varsa: iç 'b invariant → 'static'e çivili kalır. &'a mut &'b str'in örtük well-formedness kısıtı 'b: 'a'dır; 'b = 'static olunca bu 'static: 'a olur → HER 'a için doğru. Yani kısıt 'a'ya alttan hiç baskı yapmaz; dış 'a serbest kalıp println'den önce bitecek kadar KISALIR ... Tek lifetime varsa ... 'a = 'static'e çivilenir ... &mut borrow 'static yaşar → println'e taşar.` — **Hükm: DOĞRU** — **Kaynak:** `rustrepro/iki_lifetime.rs` vs `tek_lifetime.rs` (E0502) çıktıları bu gerekçeyi doğruluyor; WF kısıtı Reference `bound.implied.def` ile uyumlu — **Önem: DÜŞÜK**.
- **`src/main.rs:212-214`** — Alıntı: `CONTRAVARIANT: ÜST-tip kabul edilir; fonksiyon ARGÜMANLARINDA görülür. fn(T) argümanı T'de contravariant. RfR örneği: fn(&'a str) ... bu yüzden fn(&'static str) ... beklenen yere geçer.` — **Hükm: DOĞRU** — **Kaynak:** Nomicon subtyping (fn parameters contravariant) — **Önem: DÜŞÜK**.
- **`src/main.rs:215-217`** — Alıntı: `Variance ELLE yazılmaz; derleyici bir tipin variance'ını alanlarının konumlarından YAPISAL olarak çıkarır. Kütüphane yazarları PhantomData<...> ile ... istedikleri variance'ı belirtebilir.` — **Hükm: DOĞRU** — **Kaynak:** Nomicon subtyping (PhantomData ile variance kontrolü) — **Önem: DÜŞÜK**.
- **`src/main.rs:9`** — Alıntı: `Bu konu Rust Book'ta neredeyse yok ama lifetime hatalarının görünmez sebebi çoğu zaman budur.` — **Hükm: DOĞRULANAMADI (öznel)** — Rust Book'ta variance başlığı yok, Rustonomicon'da var; "neredeyse" ifadesi subjektif — **Kaynak:** https://doc.rust-lang.org/book/ (ilgili bölüm yok), https://doc.rust-lang.org/nomicon/subtyping.html — **Öneri:** `"Rust Book'ta variance ayrıntılı ele alınmaz; Rustonomicon'da ele alınır"` de — **Önem: DÜŞÜK**.
- **`ileri-variance/ACIKLAMA.md:5,9`** — bilet/defter analojileri — **Hükm: DOĞRU** (uzun referans kısa yere sığar; yazılabilir kutuda esneme yok) — **Önem: DÜŞÜK**.

### Inferences
- Tek-lifetime `&'a mut &'a str`'in E0502 vermesi, iç konumdaki invariantlığın dış ödünç ömrünü de sabitlediği iddiasının doğrudan kanıtı; analoji ("kitabı beş dakika tutmak" vs "yıllarca elinde tutmuş sayılmak") teknik gerekçeyle uyumlu.

### Gaps
- Yok.

---

## 4. Sized / DST / fat pointer (8 vs 16 bayt, `?Sized`, vtable) iddiaları doğru mu?

### Takeaway
Bütün boyut ve DST iddiaları çalışma zamanında doğrulandı. Tek önemli sorun: `std::ptr::metadata` / `Pointee` API'sinin kararlı gibi sunulması — rustc 1.99'da **unstable** (E0658).

### Cited Findings

- **`ileri-sized-dst/src/main.rs:12-13`** — Alıntı: `// Doğrulanmış (rustc ile): ince pointer 8 bayt; &str/&[T]/&dyn T/Box<str>/Box<dyn T> = 16 bayt // (fat pointer); unsized değeri doğrudan kullanmak → E0277.` — **Hükm: DOĞRU** — **Kaynak:** `cargo run -q -p ileri-sized-dst` çıktısı: `usize=8, &i32=8, &str=16, &[i32]=16, &dyn Ciz=16, Box<str>=16, Box<dyn Ciz>=16`; `rustrepro/pointee.rs` → E0277 — **Önem: DÜŞÜK**.
- **`src/main.rs:81-104`** — Alıntı: `size_of::<&i32>() // 8 ... size_of::<&str>() // 16: ptr + uzunluk ... size_of::<&dyn Ciz>() // 16: ptr + vtable` — **Hükm: DOĞRU** — **Kaynak:** yukarıdaki çalışma zamanı çıktısı — **Önem: DÜŞÜK**.
- **`src/main.rs:103-104`** — Alıntı: `(RfR: "twice the size of a usize")` — **Hükm: DOĞRULANAMADI (kesik alıntı)** — 2×usize olgusu doğru ama RfR'den birebir alıntı olduğu TOC/erişilebilir kaynakla teyit edilemedi — **Öneri:** tırnakları kaldır, olguyu kendi cümlesiyle de — **Önem: DÜŞÜK**.
- **`src/main.rs:115-120`** — Alıntı: `fn al(_x: str) {}   // ✗ E0277: the size for values of type `str` cannot be known... "the trait `Sized` is not implemented for `str"` ... Çözüm: DST'leri bir POINTER ARDINA koy` — **Hükm: DOĞRU** — **Kaynak:** `rustrepro/pointee.rs` E0277 mesajı; Reference `bound.sized` — **Önem: DÜŞÜK**.
- **`src/main.rs:122-124,138-146`** — Alıntı: `?Sized ile GENEL fonksiyon: &T alır, T Sized OLMAYABİLİR → str, [i32], dyn Ciz hepsi geçer ... Varsayılan T: Sized olsaydı str geçmezdi; ?Sized bunu gevşetir.` — **Hükm: DOĞRU** — **Kaynak:** https://doc.rust-lang.org/reference/trait-bounds.html#sized ("`?` is only used to relax the implicit `Sized` trait bound ..."); crate çalışıyor — **Önem: DÜŞÜK**.
- **`src/main.rs:168-184`** — Alıntı: `SLICE/STR: ikinci word = UZUNLUK ... TRAIT OBJECT: ikinci word = VTABLE adresi. vtable, somut türün metot adreslerini + layout/align + drop'unu tutar` — **Hükm: DOĞRU** — **Kaynak:** `guide/rust/library/core/src/ops/unsize.rs:10,81-83` ve `guide/rust/library/core/src/fmt/mod.rs:2972` (wide pointer = address + type-erased metadata); çalışma zamanı çıktısı — **Önem: DÜŞÜK**.
- **`src/main.rs:206-210`** — Alıntı: `Fat pointer'ın KENDİSİ Sized'dır (2×usize) ... Box ve Arc de fat pointer saklayabildiği için `T: ?Sized` destekler` — **Hükm: DOĞRU** — **Kaynak:** boyut reproları + Reference `?Sized` — **Önem: DÜŞÜK**.
- **`src/main.rs:215-216`** — Alıntı: `(std ayrıca std::ptr::Pointee/metadata ile bu metadata'yı programatik erişilebilir yapar.)` — **Hükm: YANILTICI** — rustc 1.99'da `std::ptr::metadata` ve `std::ptr::Pointee` **kararlı değil**: `ptr_metadata` özelliği gerekiyor (issue #81513). — **Kaynak:** `rustrepro/metadata.rs` ve `pointee.rs` → `error[E0658]: use of unstable library feature 'ptr_metadata'`, `see issue #81513` — **Öneri:** `"nightly'de, ptr_metadata özellik bayrağı altında (issue #81513) erişilebilir; kararlı Rust'ta değil"` de — **Önem: ORTA**.
- **`src/main.rs:212-214`** — Alıntı: `VTABLE ... Metot çağrısı bu tablodan adresi okuyup atlar (dinamik dispatch, küçük bir dolaylama).` — **Hükm: DOĞRU** — **Kaynak:** `guide/rust/library/core/src/ops/unsize.rs:81-83` — **Önem: DÜŞÜK**.
- **`ileri-sized-dst/ACIKLAMA.md:5,9`** — "etiket iki bilgi taşır" analojisi — **Hükm: DOĞRU** — **Önem: DÜŞÜK**.

### Inferences
- Fat pointer'ın 16 bayt olması doğrudan ölçülmüş; tek yanlış anlam riski API kararlılığı notu.

### Gaps
- `RawWakerVTable` ile vtable arasındaki "elle yazılmış hâli" benzetmesi (`src/main.rs:176,214`) bu kapsamda ayrıca doğrulanmadı (ileri-future konusu).

---

## 5. Coherence / orphan / newtype iddiaları doğru mu?

### Takeaway
Bütün kural ve hata kodu iddiaları doğru; E0117/E0119 reproları ve covered impl'ın derlenmesi teyit edildi.

### Cited Findings

- **`ileri-coherence/src/main.rs:10-11`** — Alıntı: `Doğrulanmış (rustc ile): çakışan impl'ler → E0119 "conflicting implementations"; hem trait hem tip yabancıysa → E0117.` — **Hükm: DOĞRU** — **Kaynak:** `rustrepro/conflict.rs` → E0119, `orph.rs` → E0117 — **Önem: DÜŞÜK**.
- **`src/main.rs:58-63`** — Alıntı: `COHERENCE özelliği: herhangi bir (tip, trait) çifti için EN FAZLA BİR impl olabilir ... Yalnız trait'i TANIMLAYAN crate blanket yazabilir` — **Hükm: DOĞRU** — **Kaynak:** https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules; `rustrepro/conflict.rs` — **Önem: DÜŞÜK**.
- **`src/main.rs:94-96`** — Alıntı: `E0119 "conflicting implementations of trait `Etiketle` for type `i32`" (DOĞRULANDI)` — **Hükm: DOĞRU** — **Kaynak:** `rustrepro/conflict.rs` — **Önem: DÜŞÜK**.
- **`src/main.rs:134-136`** — Alıntı: `✗ YASAK: hem trait hem tip YABANCI → E0117 (DOĞRULANDI, Böl. 20) ... Çözüm: NEWTYPE ile sar` — **Hükm: DOĞRU** — **Kaynak:** `rustrepro/orph.rs`; `README.md:48` (Böl. 20 = newtype) — **Önem: DÜŞÜK**.
- **`src/main.rs:142-161`** — Alıntı: `impl From<Benim> for String`: From de String de YABANCI ... Yine de İZİNLİ! ... generic konumdaki Benim YEREL ve "kapsanmış" (covered)` — **Hükm: DOĞRU** — **Kaynak:** bu impl dosyada ve `cargo check -p ileri-coherence` başarılı; Reference orphan rules — **Önem: DÜŞÜK**.
- **`src/main.rs:187-194`** — Alıntı: `• BLANKET IMPL: yalnız trait'i TANIMLAYAN crate yazabilir; var olan bir trait'e blanket EKLEMEK KIRICI değişikliktir ... • FUNDAMENTAL TİPLER (#[fundamental]: &, &mut, Box): orphan rule açısından "yokmuş" sayılır ... Bu yüzden impl IntoIterator for &MyType yazabilirsin` — **Hükm: DOĞRU** — **Kaynak:** https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules; `rustrepro/fund.rs` (`impl IntoIterator for &MyType` derlendi) — **Önem: DÜŞÜK**.
- **`src/main.rs:196-200`** — Alıntı: `COVERED IMPL: dar istisna. impl<P..> ForeignTrait<T1..Tn> for T0 ancak en az bir Ti YEREL ve ilk yerel Ti'den ÖNCE bir generic P gelmiyorsa izinli ... T0=String yabancı ama T1=Benim yerel ve kapsanmış.` — **Hükm: DOĞRU** — **Kaynak:** Reference orphan rules (uncovered type parameter koşulu) — **Önem: DÜŞÜK**.
- **`ileri-coherence/ACIKLAMA.md:5,9`** — okul/not analojisi — **Hükm: DOĞRU** — **Önem: DÜŞÜK**.

### Inferences
- Coherence dosyası, "kaynak bölüm numarası" dışında teknik olarak sorunsuz; RfR atıf düzeltmesi burada da eski satırı bırakmış (bkz. Bölüm 1).

### Gaps
- Yok.

---

## 6. Deref / metot çözümü (autoderef, autoref, inherent gölgeleme, E0596) iddiaları doğru mu?

### Takeaway
Metot çözümü algoritması ve düzeltilmiş `len()` zinciri notu **doğru**; bütün E0596/gölgeleme iddiaları doğrulandı.

### Cited Findings

- **`ileri-deref-metot/src/main.rs:12-13`** — Alıntı: `Doğrulanmış (rustc ile): Box<Vec>.len() autoderef'le çalışır; immutable `v.push()` → E0596 (autoref &mut gerektirir); inherent metot, Deref-hedefindeki aynı adlı metodu GÖLGELER.` — **Hükm: DOĞRU** — **Kaynak:** `cargo run -q -p ileri-deref-metot` + `rustrepro/pushimmut.rs` (E0596) — **Önem: DÜŞÜK**.
- **`src/main.rs:96-102`** — Alıntı: `METOT ÇÖZÜMÜ ALGORİTMASI (The Rust Reference): ... 1) Alıcının türünden başla, tekrar tekrar Deref uygulayarak zinciri çıkar ... 2) Bu listedeki HER aday tür U için şu SIRAYLA metot ara: U, &U, &mut U (AUTOREF). 3) Metodu bulan İLK eşleşmeyi seç.` — **Hükm: DOĞRU** — **Kaynak:** https://doc.rust-lang.org/reference/expressions/method-call-expr.html#method-call-targets — **Önem: DÜŞÜK**.
- **`src/main.rs:118-122`** — Alıntı: `↳ DÜZELTME NOTU (son gözden geçirme): `len` için zincir Vec'te DURUR — Vec'in kendi (inherent) `len` metodu vardır, slice'a inilmez (Box → Vec, iki seviye). Slice'a gerçekten inen çağrılar, yalnız `[T]`de tanımlı metotlardır: `kutulu.first()`, `kutulu.iter()`, `kutulu.contains(&1)`.` — **Hükm: DOĞRU** — **Kaynak:** `guide/rust/library/alloc/src/vec/mod.rs:3159` (Vec'in inherent `len`'i); `grep -rn "fn first\|fn iter(\|fn contains" library/alloc/src/vec/` → bu metotlar Vec'te yok, slice'tan geliyor — **Önem: DÜŞÜK**.
- **`src/main.rs:130-131`** — Alıntı: `Eğer v `mut` olmasaydı: autoref &mut almaya çalışır ama alamaz → E0596 "cannot borrow `v` as mutable"... (DOĞRULANDI)` — **Hükm: DOĞRU** — **Kaynak:** `rustrepro/pushimmut.rs` — **Önem: DÜŞÜK**.
- **`src/main.rs:166-181`** — Alıntı: `s.ad() = SARMAL (inherent kazanır, Deref'e inilmez) ... (*s).ad() = IC ... Ic::ad(&s) = IC` — **Hükm: DOĞRU** — **Kaynak:** `cargo run -q -p ileri-deref-metot` çıktısı; Reference method-call (ilk eşleşme) — **Önem: DÜŞÜK**.
- **`src/main.rs:193-195`** — Alıntı: `Deref (core::ops::Deref): `*x` operatörünü belirler ... Yalnız TEK bir Target tanımlanabilir ...` — **Hükm: DOĞRU** — **Kaynak:** https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules (aynı trait için ikinci impl çakışır) — **Önem: DÜŞÜK**.
- **`src/main.rs:197-199`** — Alıntı: `DEREF COERCION: ... ZİNCİRLEME, otomatik, DERLEME ANINDA uygular. Runtime maliyeti yok (sadece pointer'ı "aynı yere" yeniden yorumlar).` — **Hükm: DOĞRU** — **Kaynak:** https://doc.rust-lang.org/reference/type-coercions.html#deref-coercions — **Önem: DÜŞÜK**.
- **`src/main.rs:213`** — Alıntı: `Deref "kalıtım DEĞİLDİR" (Böl. 18): yalnız metot çağrısı/coercion ergonomisi verir; alt-tip ilişkisi kurmaz.` — **Hükm: DOĞRU** — Böl. 18 = `learn/oop` ve bu klasör "Rust'ta kalıtım YOKTUR" konusunu işliyor (`learn/oop/src/main.rs:5-6,23,34,272-278`); Deref mekaniğinin kendisi Böl. 15'te (`learn/smart-pointers`), ama parantez "kalıtım" tartışmasına gönderme yapıyor. — **Kaynak:** `README.md:43,46` + ilgili dosyalar — **Önem: DÜŞÜK**.
- **`ileri-deref-metot/ACIKLAMA.md:5,9`** — kutu/oyuncak analojisi — **Hükm: DOĞRU** — **Önem: DÜŞÜK**.

### Inferences
- Bu dosyadaki `↳ DÜZELTME NOTU` (satır 120-122), önceki bir yanlışın (`len` → slice zinciri) gerçekten düzeltildiği nadir bir örnek; diğer dosyalarda eski kaynak satırları bırakılmış.

### Gaps
- "Deref and Inherent Methods" kutusunun RfR içindeki tam konumu TOC'ta geçmediği için doğrulanamadı (bkz. Bölüm 1).

---

## 7. Sorunsuz alanlar (ek doğrulama gerektirmeyenler)

### Takeaway
Kapsam genelinde teknik içerik yüksek doğrulukta; sorunlar yoğun olarak kaynak atıf/tutararlığı ve iki ifade biçiminde.

### Cited Findings
- `cargo check` 5 crate için temiz, `cargo test` 7 test geçti, `cargo run` çıktıları yorumlarla uyumlu. — **Kaynak:** yerel komut çıktıları — **Hükm: DOĞRU**.
- 5 `ACIKLAMA.md` dosyasındaki benzetmelerin tamamı ("bilet", "defter/çanta", "etiket", "okul notu", "kutu/oyuncak") ilgili teknik kurala uygun; yanıltıcı analoji yok. — **Hükm: DOĞRU**.
- Dosyalarda `guide/rust/...` veya rustc kaynak dosya yolu/idiası **bulunmuyor** (`grep -rn "guide/rust\|compiler/\|library/\|src/\|rustc/"` → boş); dolayısıyla "rustc kaynak yolu var mı?" kontrolü bu kapsamda geçersiz/uygulanamaz. — **Hükm: SORUNSUZ (N/A)**.
- `T: 'a`, `'a: 'b`, implied bounds, elision, late-bound, erasure, monomorphization, trait object/dispatch, Send/Sync auto-trait çıkarımı gibi genel dil açıklamaları Reference ile uyumlu. — **Kaynak:** https://doc.rust-lang.org/reference/trait-bounds.html#lifetime-bounds, #implied-bounds — **Hükm: DOĞRU**.
- `impl Display for Benim`, `impl From<Benim> for String`, `IntoIterator for &MyType` örneklerinin kendileri derleniyor. — **Hükm: DOĞRU**.

### Inferences
- Önerilen müdahale sırası: (1) yanlış `Kaynak:` satırlarını düzelt, (2) `Pointee/metadata` notunu unstable olarak işaretle, (3) `HRTB ... çağrılan seçer` başlığını düzelt, (4) "aynı makine kodu" ifadesini yumuşat, (5) listing numaralarını/kutu adını ya doğrula ya kaldır.

### Gaps
- Kapsam dışı: `ileri-future`, `ileri-send-sync`, `ileri-dropck`, `ileri-eszamanlilik-async` klasörleri denetlenmedi (README düzeltme notu onları da kapsıyor; ayrı görev).
