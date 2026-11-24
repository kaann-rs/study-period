# Yazım kılavuzu (iç kullanım)

Bu dosya `book/src/` altındaki bölümlerin ortak kurallarını içerir. Kitabın parçası değildir, `SUMMARY.md`'ye eklenmez.

**Örnek sayfalar:** Tonu, düzeni ve ayrıntı düzeyini şu iki sayfadan alın:

- `src/basics/03-ownership.md` (ana bölüm)
- `src/basics/03-ownership-deep-dive.md` (derinleşme sayfası)

## 1. Amaç

Kitap, `learn/` altındaki notların okunabilir hâlidir. İki ölçüt vardır:

1. **Okur aramaz.** Her kavram anlatıldığı yerde kodunu, çıktısını ve hatasını gösterir. "Maddeler yukarıda, kod en altta" düzeni yasaktır.
2. **Not kaybolmaz.** Notlardaki her bilgi (benzetme, gerekçe, "olmasaydı ne bozulurdu", hata örneği, alıştırma, derleyici notu) yeni sayfada anlatım içinde yer alır. Tekrarlar birleştirilir, yanlışlar düzeltilir, bilgi silinmez.

## 2. Kaynak malzeme

Her sayfa için üç yer okunur:

- Asıl notlar: `22a3cca` commit'indeki `learn/<konu>/src/*.rs` yorumları, `tests/` ve `ACIKLAMA.md` (`git show 22a3cca:rust-study/learn/...`). Çalışma ağacındaki `learn/` yalnızca kodu ve dosya sonlarındaki `Learn more` kaynak listesini tutar; adlar kitapta Türkçe, `learn/` kodunda İngilizcedir.
- Eski sayfa: `backup/book-first-version-2026-11-23.tar.gz` içinde `src/...` (notlarda olmayan bilgi içerebilir). Şu an `src/` altında duran hâli de aynıdır.
- Denetim raporu: `22a3cca` commit'indeki `reports/Rust dersleri doğruluk denetimi.md` ve `research_notes/` (`git show 22a3cca:rust-study/reports/...`) → bilinen yanlışlar ve düzeltmeleri. Bulgulara uyulur, ama yeniden doğrulanır: o rapor derleyici kaynağı için yanlışlıkla nightly ağacını (1.102) "1.99.1" sanmıştır; dosya yolları ve satır numaraları güvenilir değildir.

## 3. Sayfa düzeni

### Ana bölüm

1. `# N. Başlık` (SUMMARY ile birebir aynı).
2. Başlıksız giriş: 2–4 paragraf. Hangi sorun çözülüyor, neden önemli, bölümde ne var.
3. `> [!NOTE]` kutusu: kodun dizini ve çalıştırma komutu.
4. `## Günlük hayattan bir benzetme` (özgün `ACIKLAMA.md`'deki iki anlatım birleştirilir, "siz" diliyle).
5. `<details><summary>Bu bölümde geçen terimler</summary>` içinde madde listesi: `- **terim (english):** açıklama.` (notlardaki mini sözlük).
6. Kavram başlıkları (`##`, gerekirse `###`). Her biri: kısa açıklama → kod → çıktı → açıklama → varsa derlenmeyen örnek → derleyicinin yanıtı → neden böyle → çözüm.
7. `## Hata kodu başvurusu` (bölümde hata kodu geçiyorsa): Kod | Ne zaman görülür? | Ne yapmalı?
8. `## Perde arkası: Derleyici ne yapıyor?` (notlarda derleyici bilgisi varsa; "ilk okuyuşta atlanabilir" notuyla).
9. `## Özet` ve sonraki sayfaya bağlantı.
10. `## Kaynaklar` (yalnızca dipnot tanımları).

### Derinleşme sayfası

Giriş, not kutusu, terimler, senaryolar ve örnekler, sık hatalar (her biri kendi `###` başlığında, hatalı kod ve düzeltilmiş kod yan yana), `## Akılda kalması gerekenler`, `## Alıştırmalar` (çözümler `<details>` içinde ve **derlenir**), `## Özet`, `## Kaynaklar`.

Notlarda alıştırma varsa hepsi alınır; çözümü siz yazar ve doğrularsınız. Notlarda yoksa uydurmayın.

## 4. Kod kuralları

- Her Rust bloğu tek başına derlenir (`rustdoc --test`, edition 2024, yalnızca `std`). Görünmesi gerekmeyen iskele satırları `# ` ile gizlenir.
- Derlenmemesi gereken örnek: ```` ```rust,compile_fail,E0382 ````. Kodu olmayan hatalarda yalnızca `compile_fail`.
- Paniklemesi gereken örnek: ```` ```rust,should_panic ````. Sonsuza dek süren ya da ağa çıkan: `no_run`.
- `ignore` yalnızca gerçekten derlenemeyecekse (proc-macro crate'i, nightly özellik, birden çok dosya) kullanılır ve nedeni metinde söylenir. Nightly örneği `ignore` yapılır ve nightly gerektiği yazılır.
- Çıktısı olan her örnekten sonra `Çıktı:` satırı ve ```` ```text ```` bloğu gelir. Çıktı gerçek çalıştırmadan alınır; `verify.py` karşılaştırır. Çıktı her çalıştırmada değişiyorsa (süre, thread sırası, adres) `Çıktı` sözcüğünü kullanmayın: "Örnek bir çalıştırma:" yazın ve değişebileceğini söyleyin. Mümkünse örneği belirlenimci yapın.
- Derlenmeyen örnekten sonra `Derleyicinin yanıtı:` ve gerçek rustc 1.99.0 mesajının gerekli kısmı gelir (`-->` satırı atlanabilir; `error[...]` satırı birebir). Mesajı uydurmayın, derleyip kopyalayın.
- Görünen kod 60 satırı geçmesin; uzunsa parçalara bölün, önceki parçayı gizli satırlarla taşıyın. Satır genişliği 100 sütunu geçmesin.
- Kod içi yorum kısa olsun; açıklama düzyazıya gider. Yorumda büyük harfle bağırma, ✓/✗ yok.
- Notlardaki tanımlayıcı adları korunur.
- Dışarıdan crate gereken örnek yoktur. `unsafe` bloklarında `// SAFETY:` yorumu zorunludur. Deyimsel kod yazın: `unwrap` yerine gerekçeli `expect` ya da `?`; gereksiz `clone` yok.
- Komutlar ```` ```console ```` bloğunda verilir.

## 5. Dil kuralları

- Okura **siz** diye seslenilir; anlatımda birinci çoğul ("göreceğiz") ve edilgen kullanılabilir. "Sen" dili kullanılmaz.
- Tam cümle kurulur. Telgraf dili, ok işareti (→), ✓/✗, BÜYÜK HARFLE vurgu yoktur. Vurgu için *italik* (terimin ilk geçişi) ya da **kalın** (az).
- İngilizce terim ilk geçişte parantez içinde verilir: "taşıma (move)". Sonra Türkçesi kullanılır.
- Ekler kod biçiminin dışında kalır ve kesme işaretiyle ayrılır: `` `String`'in ``, `` `Vec<T>`'ye ``, Rust'ın.
- İki noktadan sonra tam cümle geliyorsa büyük harfle başlar; sıralama geliyorsa küçük.
- TDK yazımı: hâlâ, hâl, kâğıt, imkân; "çalışma zamanı", "derleme zamanı" (an değil); "herhangi bir", "ya da", "pek çok". "ki" bağlacı ve soru eki "mi" ayrı.
- "Aptala anlatır gibi", "5 yaşındaki çocuğa" başlıkları kullanılmaz; içeriği benzetme başlığında korunur.
- Uzun tireyi (—) cümle kurmamak için kullanmayın; iki nokta, virgül ya da yeni cümle kullanın.
- Abartılı kesinlik yok ("her zaman", "asla" yalnızca gerçekten öyleyse).

### Terimler

| English | Kitapta |
| --- | --- |
| ownership, owner | sahiplik, sahip |
| move / copy / clone | taşıma / kopyalama / klonlama (derin kopya) |
| partial move | kısmi taşıma |
| borrow, borrowing, borrow checker | ödünç, ödünç alma, ödünç denetleyicisi (borrow checker) |
| reference; shared / mutable reference | referans; paylaşımlı / değiştirilebilir referans |
| dangling reference | sarkan referans |
| lifetime, lifetime elision | ömür (lifetime), ömür çıkarımı (elision) |
| scope, shadowing | kapsam, gölgeleme |
| drop, destructor, drop check | düşürme, yıkıcı, düşürme denetimi |
| slice | dilim |
| binding, variable | bağlama, değişken |
| mutable / immutable | değiştirilebilir / değiştirilemez |
| interior mutability | iç değiştirilebilirlik |
| statement / expression | deyim / ifade |
| struct, enum, field, variant | struct (yapı), enum, alan, varyant |
| method, receiver, associated function/type | metot, alıcı, ilişkili fonksiyon/tür |
| trait, trait bound, trait object | trait, trait sınırı, trait nesnesi |
| generic, monomorphization | jenerik, monomorfizasyon |
| static / dynamic dispatch | statik / dinamik gönderim |
| dyn compatible (object safe) | dyn uyumlu |
| closure, capture | closure, yakalama |
| iterator, lazy, adapter, consumer | iterator, tembel, dönüştürücü, tüketici |
| pattern, match, match guard, exhaustive | desen, `match`, koruyucu koşul, eksiksiz |
| refutable / irrefutable | çürütülebilir / çürütülemez |
| smart pointer, raw pointer, wide (fat) pointer | akıllı işaretçi, ham işaretçi, geniş işaretçi (wide/fat pointer) |
| reference counting, reference cycle | referans sayımı, referans döngüsü |
| allocation, leak | tahsis, sızıntı |
| double free / use-after-free | çifte serbest bırakma / serbest bırakıldıktan sonra kullanım |
| undefined behavior, invariant | tanımsız davranış, değişmez |
| thread, channel, lock, deadlock, poisoning | thread, kanal, kilit, kilitlenme, zehirlenme |
| data race, race condition | veri yarışı, yarış durumu |
| concurrency / parallelism | eşzamanlılık / paralellik |
| async runtime, executor, task, waker, poll | çalışma zamanı, yürütücü, görev, uyandırıcı (waker), yoklama (poll) |
| backpressure, cancellation, timeout | geri basınç, iptal, zaman aşımı |
| variance; covariant / contravariant / invariant | varyans; kovaryant / kontravaryant / invaryant |
| subtyping, supertype | alt tür ilişkisi, üst tür |
| coherence, orphan rule, blanket impl, newtype | tutarlılık, yetim kuralı, genel impl (blanket), newtype |
| coercion, cast, deref coercion, autoref | örtük dönüşüm, tür dönüştürme, deref coercion, autoref |
| DST, ZST, unsized | dinamik boyutlu tür, sıfır boyutlu tür, boyutu bilinmeyen |
| macro, declarative / procedural macro | makro, bildirimsel / yordamsal makro |
| overflow, panic, unwinding | taşma, panik, yığın çözülmesi |
| compile time / runtime | derleme zamanı / çalışma zamanı |
| crate, package, module, path, visibility | crate, paket, modül, yol, görünürlük |
| unit test / integration test / doc test | birim testi / entegrasyon testi / belge testi |
| operator, attribute, argument | operatör, öznitelik, argüman |
| element (koleksiyonda) / item | öğe ya da eleman (bir sayfada tek biçim) / öğe |
| destructuring, dereference | parçalama, başvuruyu çözmek |
| inlining, re-export | satır içine alma, yeniden dışa aktarma |
| busy-wait, pin projection | meşgul bekleme, pin projeksiyonu |
| brackets: `()` `[]` `{}` `<>` | yuvarlak, köşeli, süslü, açılı parantez |
| terminal | terminal |

`stack`, `heap`, `thread`, `trait`, `struct`, `enum`, `closure`, `iterator`, `crate`, `future` Türkçeleştirilmez.

## 6. Doğrulama ve kaynak kuralları

Her teknik iddia yazılmadan önce **resmî kaynaktan okunarak** doğrulanır:

1. Yerel resmî belge (1.99.0): `$(rustc --print sysroot)/share/doc/rust/html/` altında `book`, `reference`, `nomicon`, `std`, `core`, `alloc`, `cargo`, `error_codes`, `rust-by-example`, `edition-guide`, `rustc`, `rustdoc`. HTML'i metne çevirip okuyun: `sed -e 's/<[^>]*>//g' dosya.html`.
2. Standart kütüphane kaynağı (1.99.0): `$(rustc --print sysroot)/lib/rustlib/src/rust/library/`.
3. Derleyici kaynağı: `https://raw.githubusercontent.com/rust-lang/rust/1.99.0/compiler/...` (`guide/rust` ağacı nightly'dir, **kullanılmaz**).
4. İnternet: güncel sürüm, RFC, blog.rust-lang.org, rustc-dev-guide, crate belgeleri. Güncel kararlı sürüm 1.100.0'dır (12 Kasım 2026); never type (`!`), çekirdek `Allocator` API'si ve `DropGuard` orada kararlı oldu. Konu bunlardan birine değiyorsa belirtin. Örnekler 1.99.0 ile derlenir.
5. Davranış iddiası (boyut, düşürme sırası, hata kodu, panik mesajı) küçük bir programla sınanır.

Doğrulanamayan iddia metne kesin bilgi gibi yazılmaz. Önemliyse `> [!NOTE]` içinde "birincil kaynakta doğrulanamadı" denir, değilse çıkarılır ve raporda bildirilir.

Kaynaklar **dipnot** ile verilir: cümlenin sonunda `[^ad]`, tanım dosyanın sonunda `## Kaynaklar` altında:

```markdown
[^ad]: The Rust Programming Language, [4.1 What Is Ownership?](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html).
[^kod]: rustc 1.99.0 kaynak kodu, [`borrowck_errors.rs`, `cannot_act_on_moved_value`](https://github.com/rust-lang/rust/blob/1.99.0/compiler/rustc_borrowck/src/borrowck_errors.rs#L321).
```

- Her kavram başlığında en az bir dipnot bulunur. Genel ana sayfa (`/reference/`, `/std/`) kaynak sayılmaz; ilgili alt sayfa ve çapa verilir.
- Kaynak kodu linki `1.99.0` etiketini gösterir; satır numarası o etikete göre bulunur ve bağlantı metninde o satırın yakınında geçen bir sembol ters tırnak içinde yazılır (araç bunu denetler).
- Basılı kitaplar (*Rust for Rustaceans*, *Programming Rust*, *Rust Atomics and Locks*) ad ve bölümle anılabilir.

## 7. Bitirme denetimi

```console
python3 verify.py src/basics/05-enums.md src/basics/05-enums-deep-dive.md
python3 verify.py --kapsam src/basics/05-enums.md src/basics/05-enums-deep-dive.md
```

İlk komut **0 hata** vermelidir; uyarılar gerekçesiz kalmamalıdır. İkincisi eksik terimleri listeler: Hata kodlarının tamamı, kavram taşıyan her ad ve her satır içi kod ya sayfada yer almalı ya da neden gerekmediği bilinmelidir (eski yazım artıkları, `fn` sarmalayıcı adları ve test adları sayılmaz).
