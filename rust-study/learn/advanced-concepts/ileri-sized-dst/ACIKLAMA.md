# Boyut, DST ve Şişman İşaretçi

## Aptala anlatır gibi

Rust bir değeri saklamak için boyutunu önceden bilmek ister (Sized). Boyu bilinmeyen türler (DST: `str`, `[T]`, `dyn Trait`) hep bir işaretçinin ardında durur. Bu işaretçi yalnız adres değil, uzunluk ya da yöntem tablosu (vtable) da taşır; buna şişman işaretçi (fat pointer) denir. `?Sized` ise "boyutunu bilmek zorunda değilim" der.

## 5 yaşındaki çocuğa anlatır gibi

Bir kutuya ne koyacağını bilmiyorsan kutuyu sabit büyüklükte yapamazsın. Bu yüzden kutunun yerine üzerinde "şu adreste, şu kadar uzunlukta" yazan bir etiket verirsin. Etiket küçük ama iki bilgi taşır; tek adres yetmez.

> Not: Kod mantığı değiştirilmedi. Değişiklikleri `cargo check` ile doğrula.
