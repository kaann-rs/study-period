# Nesne Yönelimli Özellikler (OOP)

## Aptala anlatır gibi

Rust'ta nesne = veri + davranış vardır ama kalıtım (inheritance) yoktur. Yerine ortak davranış için trait, kod paylaşımı için bileşim (composition) kullanılır. Trait object (`dyn Ciz`) ile farklı türler aynı kutuda durabilir; çekirdek kod yalnız "çizebilen bir şey" bilir. Yeni bir kanal eklemek çekirdeğe dokunmaz.

## 5 yaşındaki çocuğa anlatır gibi

Oyuncak kutusuna kırmızı araba, mavi tren, sarı top koyuyorsun. Hepsi farklı ama hepsi "yuvarlanabilir". Kutuya yeni bir oyuncak eklemek için kutuyu değiştirmen gerekmez; yeter ki o da yuvarlanabilsin.

> Not: Kod mantığı değiştirilmedi. Değişiklikleri `cargo check` ile doğrula.
