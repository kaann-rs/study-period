# Akıllı İşaretçiler (Smart Pointers)

## Aptala anlatır gibi

Akıllı işaretçi (smart pointer), işaretçi gibi davranıp ek bilgi taşıyan kutudur: `Box` tek sahibi büyük depoya (heap) koyar, `Rc` sahipliği birden çok okuyucuya paylaştırır, `RefCell` içeriden değiştirmeye izin verir, `Weak` ise sahip olmadan bakar. Her biri güvenliği başka yolla korur. İki veri birbirini gösterirse (döngü) `Weak` olmasa hiçbiri silinemez.

## 5 yaşındaki çocuğa anlatır gibi

Okulda birçok kutu var. Biri sadece senin (Box), birini arkadaşlarınla paylaşıyorsun (Rc), birinin içini öğretmen izin vermeden düzeltebiliyorsun (RefCell). İki defter birbirine "beni oku" diye not bırakırsa ikisi de çöpe atılamaz; bu yüzden biri sadece "nerede olduğunu" hatırlatır (Weak).

> Not: Kod mantığı değiştirilmedi. Değişiklikleri `cargo check` ile doğrula.
