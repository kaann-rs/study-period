# Rust Yolculuğu

`learn/` dizinindeki Rust notlarının kitap hâli. [mdBook](https://rust-lang.github.io/mdBook/) ile derlenir.

## Okumak

```bash
mdbook serve --open
```

Yalnızca HTML üretmek için `mdbook build` yeterlidir; çıktı `book/` dizinine yazılır. mdBook kurulu değilse:

```bash
cargo install mdbook --locked
```

## Dizin yapısı

```text
book.toml            mdBook ayarları
verify.py            doğrulama aracı
WRITING-GUIDE.md     yazım ve kaynak kuralları (kitaba dâhil değildir)
styles/book.css      ek stiller
backup/              kitabın ilk hâlinin arşivi
src/
    SUMMARY.md       içindekiler
    index.md         önsöz
    how-to-use.md    kitabı kullanma rehberi
    sources.md       kaynaklar ve doğrulama yöntemi
    basics/          1–8. bölümler
    intermediate/    9–20. bölümler
    advanced/        21–29. bölümler
```

## Doğrulama

```bash
python3 verify.py
```

Araç bütün Rust bloklarını derleyip çalıştırır, yazılan çıktıları ve hata mesajlarını gerçekleriyle karşılaştırır, bağlantıları denetler. Python 3 ile `rustup` üzerinden kurulmuş Rust 1.99.0 ve `rust-docs` bileşeni gerekir. Tek bir sayfayı denetlemek için dosya yolunu yazın:

```bash
python3 verify.py src/basics/03-ownership.md
```

Bir bölümde notlardan eksik kalan terimleri görmek için:

```bash
python3 verify.py --kapsam src/basics/03-ownership.md src/basics/03-ownership-deep-dive.md
```
