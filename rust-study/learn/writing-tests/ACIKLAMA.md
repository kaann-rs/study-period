# Test Yazma (Writing Tests)

## Aptala anlatır gibi

Test, "şunu verirsem şunu almalıyım" diyen küçük bir denemedir; bir kez yazarsın, bilgisayar her seferinde senin yerine dener. `#[test]` koyduğun fonksiyon ayrı bir sınavdır, `assert_eq!` sınavın sorusudur. Birim testleri (unit test) kodun yanında, entegrasyon testleri (integration test) `tests/` dizinindedir. Özel bir framework kurmana gerek yoktur; dil ve cargo bunu içerir.

## 5 yaşındaki çocuğa anlatır gibi

Oyuncak atölyesinde küçük sınavlar var: "kutuya 2 ve 3 koyarsan 5 çıkmalı". Bunları bir kez yazarsın, öğretmen her sabah senin yerine kontrol eder. Yanlış olursa hangi sınavın kaldığını söyler.

> Not: Kod mantığı değiştirilmedi. Değişiklikleri `cargo check` ile doğrula.
