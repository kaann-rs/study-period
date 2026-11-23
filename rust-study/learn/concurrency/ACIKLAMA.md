# Eşzamanlılık (Concurrency)

## Aptala anlatır gibi

Bir program işini birkaç kola (iş parçacığı / thread) bölebilir; mutfakta birden fazla aşçı gibi. İki aşçı aynı tencereye aynı anda el atarsa yemek bozulur (veri yarışı / data race), Rust buna izin vermez. Açılan iş parçacığını beklemek `join`, haber göndermek kanal (channel), ortak durumu korumak kilit (Mutex), paylaşılan sahiplik ise Arc ile olur.

## 5 yaşındaki çocuğa anlatır gibi

Mutfağa üç aşçı girdi. Biri tuz atarken diğeri aynı tencereyi karıştırırsa yemek bozulur. Bu yüzden tencereyi alan önce "ben kullanıyorum" der, bitirince bırakır. Aşçılar birbirine not da geçebilir; notu kutuya koyar, öteki alır.

> Not: Kod mantığı değiştirilmedi. Değişiklikleri `cargo check` ile doğrula.
