# İleri Lifetime, HRTB ve GAT

## Aptala anlatır gibi

Bir generic parametreyi kim seçer? `fn f<'a>(..)` dersen süreyi çağıran seçer; gövde içinde o süre sabittir. `for<'a>` (HRTB) tersini söyler: süreyi çağrılanın içinden biz seçeriz, karşı taraf her süreye razı olur. GAT (generic associated type) aynı esnekliği ilişkili türe taşır: `type Oge<'a>` her kullanımda farklı süreyle örneklenir.

## 5 yaşındaki çocuğa anlatır gibi

Bir kural düşün: "bu bilet bugün geçerli". Normalde bileti veren kişi tarihi yazar. Bazen de sen girişte tarihi seçersin ve görevli "hangi tarih olursa olsun kabulüm" der. Bazen de biletin içindeki koltuk numarası her seferinde farklı olur; ona ayrı bir kutu gerekir.

> Not: Kod mantığı değiştirilmedi. Değişiklikleri `cargo check` ile doğrula.
