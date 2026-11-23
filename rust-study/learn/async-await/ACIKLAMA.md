# Async / Await

## Aptala anlatır gibi

Async, tek aşçının beklerken boş durmamasıdır: çamaşır makinesi dönerken bulaşığı yıkarsın. Gelecek (Future) bir "iş kartı"dır; kartı yazmak işi başlatmaz, `await` ile o iş hazır olana kadar sıraya girersin. Eşzamanlılık (concurrency) işleri iç içe yürütmektir; çalışma zamanı (runtime) olmasa kartları kimse çevirmez. İki bağımsız işi arka arkaya değil birlikte beklersen toplam süre en uzun işin süresine iner.

## 5 yaşındaki çocuğa anlatır gibi

Çamaşırı makineye koydun. Oturup bekler misin? Hayır, o sırada bulaşıkları yıkarsın. Makine bitince zil çalar, sen de çamaşırı alırsın. İki makine varsa ikisini birden çalıştırırsın; ikisi de aynı anda biter.

> Not: Kod mantığı değiştirilmedi. Değişiklikleri `cargo check` ile doğrula.
