# Enum'lar ve Desen Eşleme

## Aptala anlatır gibi

Enum, "bu değer şu seçeneklerden yalnız biridir" der; her seçeneğe varyant (variant) denir. Postacının elindeki zarf ya teslim edilmiştir, ya yoldadır, ya da kaybolmuştur; üçü birden olamaz. `match` her varyant için ne yapılacağını yazdırır ve hiçbirini unutmana izin vermez (bütünlük / exhaustiveness). `Option` aynı fikrin küçük hâlidir: ya `Some` var ya `None` yok.

## 5 yaşındaki çocuğa anlatır gibi

Postacı kapıya gelir. Zarf ya kutuya girer, ya cebinde kalır, ya da geri götürülür. Bir zarfın aynı anda hem kutuda hem cebinde olması imkânsızdır. Postacı her durumda ne yapacağını önceden bilir; birini unutursa patronu hemen fark eder.

> Not: Kod mantığı değiştirilmedi. Değişiklikleri `cargo check` ile doğrula.
