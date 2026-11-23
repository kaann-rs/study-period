# Tutarlılık ve Yetim Kuralı (Coherence / Orphan Rule)

## Aptala anlatır gibi

Bir tür ile bir trait'in eşleşmesi (impl) bütün programda yalnız bir kez yazılabilir; yoksa aynı soruya iki cevap olur. Bu yüzden eşleşmeyi ancak trait'in ya da türün sahibi yazabilir: buna yetim kuralı (orphan rule) denir. Yabancı bir türe yabancı bir trait eklemeye kalkınca kapı kapanır; kendi türünü ince bir kutuya (newtype) koyunca kapı açılır.

## 5 yaşındaki çocuğa anlatır gibi

Okulda her öğrencinin tek numarası olur. İki kişi aynı numarayı alırsa yoklama karışır. Numarayı ya okul verir ya da sen kendi defterine yeni bir isim yazarsın; başkasının defterine izinsiz numara yazamazsın.

> Not: Kod mantığı değiştirilmedi. Değişiklikleri `cargo check` ile doğrula.
