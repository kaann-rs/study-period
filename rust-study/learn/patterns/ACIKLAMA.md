# Desenler (Patterns)

## Aptala anlatır gibi

Desen (pattern), bir değere "şu şekilde misin?" diye sormaktır; uyuyorsa içindeki parçalara ad verirsin (ayrıştırma / destructuring). `match` dışında `if let`, `while let`, `let` ve closure parametrelerinde de aynı dilbilgisi çalışır. `Some(x)` gibi reddedilebilir (refutable) desen `let` içinde doğrudan yazılamaz; derleyici `let else` önerir.

## 5 yaşındaki çocuğa anlatır gibi

Kapağında yıldız, kare, daire delikleri olan oyuncak kutusu var. Elindeki parçayı hangi deliğe sokabileceğine bakarsın; girerse içindeki boncuğu alıp "buna Ali diyelim" dersin. Uymayan deliğe zorla sokamazsın.

> Not: Kod mantığı değiştirilmedi. Değişiklikleri `cargo check` ile doğrula.
