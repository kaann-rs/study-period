# Closure ve Iterator'lar

## Aptala anlatır gibi

Closure, çevresindeki değişkenleri hatırlayan adsız küçük bir fonksiyondur (closure); iterator ise elemanları tek tek, istendikçe veren bir sıradır (iterator). Kargo şubesindeki görev notları gibi: notu yazarsın, bant da paketleri önüne getirir. `map` paketi başka bir şeye çevirir, `filter` bazılarını ayıklar, `collect` sonunda hepsini kutuya koyar. Bant tembeldir (lazy): kimse istemedikçe paket üretmez.

## 5 yaşındaki çocuğa anlatır gibi

Önce bir not yaz: "kırmızı olanları ayır". Not cebinde durur, hatırlar. Sonra taşıma bandı paketleri tek tek önünden geçirir; sen kırmızıları alırsın, mavileri geçersin. En sonda hepsini bir kutuya koyarsın.

> Not: Kod mantığı değiştirilmedi. Değişiklikleri `cargo check` ile doğrula.
