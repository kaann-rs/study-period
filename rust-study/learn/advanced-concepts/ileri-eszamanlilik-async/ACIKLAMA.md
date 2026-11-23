# İleri Eşzamanlılık ve Async

## Aptala anlatır gibi

Thread'ler ve async görevler aynı soruya iki cevaptır: "beklerken ne yapalım?" Thread tarafında doğru kilidi seçmek (RwLock), koşul için zil beklemek (Condvar), kilitleri hep aynı sırayla almak ve tek bir sayı için kilit yerine atomik (atomic) kullanmak vardır. Async tarafında çalışma zamanı (runtime) yalnız uyandırılan görevleri yoklar; kuyruk, spawn, zamanlayıcı ve zaman aşımı bu döngünün parçalarıdır.

## 5 yaşındaki çocuğa anlatır gibi

Mutfafta aşçılar (thread) ve tek bir kasiyer (async) var. Kasiyer sipariş beklerken boş durmaz; zil çalan masaya gider. Herkes aynı tencereye el atmasın diye kilit koyar ve hep aynı sırayla kilitlersin; yoksa iki aşçı birbirini bekler, yemek hiç pişmez.

> Not: Kod mantığı değiştirilmedi. Değişiklikleri `cargo check` ile doğrula.
