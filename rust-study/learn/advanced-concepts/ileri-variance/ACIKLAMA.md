# Alt-tip ve Değişkenlik (Subtyping / Variance)

## Aptala anlatır gibi

Uzun süre geçerli bir referans, kısa süre geçerli bir yere sığabilir; tersi olmaz. Buna değişkenlik (variance) denir. Paylaşımlı referans (&) esnek davranır (covariant); değiştirilebilir referans (&mut) sıkıdır (invariant), çünkü içine kısa ömürlü bir şey yazarsan uzun ömürlü taraf bozulur. Bazı lifetime hatalarının sebebi yazdığın kod değil, kutunun (türün) davranışıdır.

## 5 yaşındaki çocuğa anlatır gibi

Büyük bir defteri küçük bir çantaya koyabilirsin; ama küçük defteri büyük çantaya koyup "büyük defter san" diyemezsin. Üstüne yazı yazılabilen defterlerde ise hiç karıştırma yapamazsın; yoksa yanındaki not yanlış olur.

> Not: Kod mantığı değiştirilmedi. Değişiklikleri `cargo check` ile doğrula.
