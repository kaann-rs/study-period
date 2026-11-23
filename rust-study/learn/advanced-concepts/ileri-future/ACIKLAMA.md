# Future'ın İç Yüzü

## Aptala anlatır gibi

Gelecek (Future), henüz bitmemiş bir hesaptır. Yürütücü (executor) onu yoklar (poll); cevap ya `Ready(değer)` ya `Pending`'dir. `Pending` dönerken future bir uyardırıcı (Waker) saklar; iş hazır olunca `wake()` çağrılır ve yürütücü yeniden yoklar. Boşa dönen bekleme (busy-wait) yoktur. `async fn` derleyici tarafından her `.await`'te durup devam eden bir durum makinesine (state machine) çevrilir; o makine future'ın ta kendisidir.

## 5 yaşındaki çocuğa anlatır gibi

Çamaşır makinesine "bittiğinde beni ara" diye not bırakıyorsun. Sen başka iş yapıyorsun; telefon çalınca gidip çamaşırı alıyorsun. Durmadan "bitti mi?" diye bakmıyorsun. Uzun tarifin de bölünmüş sayfaları var: her `.await` bir sayfa sonu, orada durup devam edersin.

> Not: Kod mantığı değiştirilmedi. Değişiklikleri `cargo check` ile doğrula.
