# Silme Kontrolü (Drop Check)

## Aptala anlatır gibi

Bir değer silinirken (drop) ödünç aldığı veriye son bir kez erişebilir; çünkü silme kodu onu kullanabilir. Bu yüzden ödünç alınan veri, o değerden daha uzun yaşamak zorundadır; derleyici bunu silme anında da kontrol eder (dropck). Türün özel silme kodu yoksa borç daha erken bitebilir, varsa süre uzatılır. `PhantomData` "burada bir ödünç varmış gibi davran" demenin yoludur.

## 5 yaşındaki çocuğa anlatır gibi

Çöpe atmadan önce defterine son bir not yazacaksın. O notu yazmak için masadaki kitaba bakman gerekiyor. Demek ki kitap, defter çöpe atılmadan önce orada durmalı. Kitabı çok erken kaldırırsan yazacak yerin kalmaz.

> Not: Kod mantığı değiştirilmedi. Değişiklikleri `cargo check` ile doğrula.
