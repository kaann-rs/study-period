# Koleksiyonlar (Common Collections)

## Aptala anlatır gibi

Üç kap vardır: uzayıp giden tren dizisi (Vec), boncuklardan yapılmış yazı (String) ve etiketli çekmeceler (HashMap). Vec'e sona eklemek `push`, sıra numarasıyla bakmak indeks (index); saymaya 0'dan başlanır. HashMap'de anahtar (key) sorarsın, değer (value) gelir. String dilimleri bayt ofsetleriyle alınır; bu ofsetler bir UTF-8 karakter sınırına denk gelmek zorundadır, aksi hâlde çalışma anında panik olur.

## 5 yaşındaki çocuğa anlatır gibi

Oyuncak treninin vagonlarını uzatıp kısaltabilirsin. Yazı boncuk kolye gibidir; boncuğun ortasından kesince kolyeyi bozarsın. Çekmecelere isim yapıştırırsın: "çorap" yazana çorap koyarsın, sonra aynı isimle bulursun.

> Not: Kod mantığı değiştirilmedi. Değişiklikleri `cargo check` ile doğrula.
