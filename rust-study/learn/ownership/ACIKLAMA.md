# Ownership (Sahiplik)

## Aptala anlatır gibi

Her verinin **tek bir sahibi (owner)** vardır. Sahibi oyuncağı arkadaşına verirse (taşıma / move), araba artık onundur; sen oynayamazsın. Sayılar gibi küçük şeylerin fotokopisi bedavadır (kopya / copy); büyük kutunun aynısını yeniden yaptırmak (klon / clone) emek ister, o yüzden kendiliğinden yapılmaz. Sahip odadan (kapsam / scope) çıkınca oyuncağını da götürür (düşürme / drop) — "sil" demene gerek yok. Başkası oyuncağı alıp götürmeden sadece bakabilir (ödünç alma / borrowing): ya birçok bakan (&) olur, ya da tek bir boyayan (&mut); ikisi aynı anda olmaz. Ok işareti (referans / reference), gösterdiği defter yoksa boşa bakar; Rust buna izin vermez. Dilim (slice) pastanın bir parçasını gösterir, kopyası değildir; pasta gidince dilim de gider.

## 5 yaşındaki çocuğa anlatır gibi

Oyuncağın var, değil mi? Onu arkadaşına verdiğinde artık onun olur, sen tutamazsın. Ama "sadece bakabilir misin?" dersen oyuncak yine senindir. Herkes birlikte bakabilir; ama boyayan tek kişi olmalı, yoksa resim bozulur. Oyun odasından çıkınca oyuncaklar kendiliğinden toplanır, sen "topla" demek zorunda değilsin. Defteri çöpe atarsan arkadaşının bakacağı şey kalmaz — Rust "hayır" der. Pasta dilimi gibi: bütün pastayı almadan bir parçaya bakarsın; pasta gidince dilim de gider.

> Not: Kod mantığı değiştirilmedi. Değişiklikleri `cargo check` ile doğrula.
