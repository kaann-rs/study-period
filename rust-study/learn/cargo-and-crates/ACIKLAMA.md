# Cargo ve Crate'ler

## Aptala anlatır gibi

Kodunu bir kutuya koyup başkalarının kullanımına sunarsın: buna paket (package) denir, içindeki derleme birimine crate denir. Kutunun içine yazdığın kullanım kılavuzu (doküman yorumu / doc comment) aynı zamanda sınavdır: örnek kodlar test olarak koşar. `pub use` ise iç rafları gizlemez; onlara ek olarak önden kısa bir kapı açar. Kullanıcı uzun yolu bilmek zorunda kalmaz, ama bilirse çalışır.

## 5 yaşındaki çocuğa anlatır gibi

Boya seti alıyorsun. Kapağında nasıl kullanılacağı yazılı; yazan örnekleri denemek de mümkün. İçinde onlarca göz var ama sen sadece "karıştır" düğmesine basıyorsun. Kimin hangi gözü açtığını bilmen gerekmiyor.

> Not: Kod mantığı değiştirilmedi. Değişiklikleri `cargo check` ile doğrula.
