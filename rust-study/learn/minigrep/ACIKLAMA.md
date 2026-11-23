# minigrep

## Aptala anlatır gibi

Programa bir kelime ve dosya adı verirsin; dosyada o kelimenin geçtiği satırları bulur. Asıl iş kütüphane kısmındadır (lib); `main` yalnız fişi doldurur: argümanları toplar, hatayı hata kanalına (stderr) yazar, çıkış koduyla biter. Mantığı test edilebilir yere koymak, test edilmeyen kısmı küçültmektir.

## 5 yaşındaki çocuğa anlatır gibi

Kütüphanede danışma masası var. Sen "masal" kelimesini soruyorsun, görevli kitabı tarayıp o kelimenin olduğu sayfaları gösteriyor. Kapıdaki kişi sadece soruyu alıp görevliye verir; aramayı o yapmaz.

> Not: Kod mantığı değiştirilmedi. Değişiklikleri `cargo check` ile doğrula.
