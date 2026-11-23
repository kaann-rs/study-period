# Yaygın Kavramlar (Common Concepts)

## Aptala anlatır gibi

Program; isimli değerlerden (değişken / binding), o değerlerin cinsinden (tür / type), iş yapan küçük tariflerden (fonksiyon / function) ve yol tabelalarından (kontrol akışı / control flow) oluşur. Rust varsayılan olarak tükenmez kalem kullanır: değer değişmez (immutable); `mut` dersen kurşun kaleme geçersin. Değişmezlik, iki kişinin aynı defteri aynı anda boyamasını engelleyen kuralın (Aliasing XOR Mutability) yüzeye yansımasıdır; kuralın kendisi referansların paylaşım/değiştirme kurallarıdır.

## 5 yaşındaki çocuğa anlatır gibi

Okulda defterin, kalemin, çantan var. Deftere bir sayı yazarsın; üstünü çizmek istersen önce "silebilir" demen gerekir. Öğretmen "şu kapı kapalı, şu kapı açık" der; sen de tabelaya bakıp yürürsün.

> Not: Kod mantığı değiştirilmedi. Değişiklikleri `cargo check` ile doğrula.
