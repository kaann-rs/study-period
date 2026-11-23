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
dogrula.py           doğrulama aracı
YAZAR-KILAVUZU.md    yazım ve kaynak kuralları (kitaba dâhil değildir)
tema/kitap.css       ek stiller
yedek/               kitabın ilk hâlinin arşivi
src/
    SUMMARY.md       içindekiler
    index.md         önsöz
    giris.md         kitabı kullanma rehberi
    kaynaklar.md     kaynaklar ve doğrulama yöntemi
    temel/           1–8. bölümler
    orta/            9–20. bölümler
    ileri/           21–29. bölümler
```

## Doğrulama

```bash
python3 dogrula.py
```

Araç bütün Rust bloklarını derleyip çalıştırır, yazılan çıktıları ve hata mesajlarını gerçekleriyle karşılaştırır, bağlantıları denetler. Python 3 ile `rustup` üzerinden kurulmuş Rust 1.99.0 ve `rust-docs` bileşeni gerekir. Tek bir sayfayı denetlemek için dosya yolunu yazın:

```bash
python3 dogrula.py src/temel/03-ownership.md
```

Bir bölümde notlardan eksik kalan terimleri görmek için:

```bash
python3 dogrula.py --kapsam src/temel/03-ownership.md src/temel/03-ownership-derinlesme.md
```
