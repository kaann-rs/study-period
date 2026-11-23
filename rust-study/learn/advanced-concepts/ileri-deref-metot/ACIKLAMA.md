# Deref ve Metot Çözümü

## Aptala anlatır gibi

`x.metot()` yazdığında sihir olmaz: derleyici metodu bulana kadar işaretçiyi adım adım açar (otomatik açma / autoderef), gerekiyorsa referans alır (otomatik referans / autoref). Kutu (Box) içindeki vektörün `len()` çağrısı böyle çalışır. Kendi türünde aynı adda bir metot varsa, açılan türdeki aynı adlı metodu gölgeler.

## 5 yaşındaki çocuğa anlatır gibi

Kapalı bir kutu içinde oyuncak var. "Oyuncağın rengi ne?" diye sorunca önce kutuyu açarsın, sonra oyuncağa bakarsın. Kutunun kendisinde de "renk" diye bir etiket varsa önce ona bakarsın; gerçek oyuncağın rengini sormak için kutuyu açman gerekir.

> Not: Kod mantığı değiştirilmedi. Değişiklikleri `cargo check` ile doğrula.
