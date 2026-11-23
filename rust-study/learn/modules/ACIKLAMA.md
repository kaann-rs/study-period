# Modüller (Modules)

## Aptala anlatır gibi

Kod büyüyünce odalara (modül / module) bölünür. Her odanın kapısı kapalı gelir; dışarı göstermek istediğin şeylere `pub` yazarsın. Paket (package) birden çok crate içerebilir; crate ise tek derleme birimidir. Yol (path), "hangi odadan hangi eşya" sorusunun cevabıdır.

## 5 yaşındaki çocuğa anlatır gibi

Evde odalar var: mutfak, yatak odası, oyun odası. Her odanın kapısı kapalı; "girebilirsin" etiketi yapıştırdığın eşyayı alabilirsin. "Mutfak → bardak" demek, hangi odada ne arayacağını söylemektir.

> Not: Kod mantığı değiştirilmedi. Değişiklikleri `cargo check` ile doğrula.
