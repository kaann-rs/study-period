# Hata Yönetimi (Error Handling)

## Aptala anlatır gibi

İki hata var: kurtarılamaz (panik / panic) ve kurtarılabilir. Kurtarılabilir hata bir kutunun içinde taşınır: `Result<T, E>` ya `Ok` ya `Err` döner. Rust'ta try/catch yoktur; hata dönüş değerinin parçasıdır, böylece ona bakmayı unutamazsın. `?` operatörü "burada hata olursa yukarı gönder" der. İyi hata ne olduğunu, nerede olduğunu ve neden olduğunu söyler.

## 5 yaşındaki çocuğa anlatır gibi

Tarif kartını okuyorsun. "Un 2 bardak" yazan satır yoksa yemek yapamazsın; ya durursun ya da annene sorarsın. Annene sormak `?` gibidir: "bunu ben çözemediğim, sen devam et". Kart tamamen boşsa zorunlu anahtarlar eksik sayılır, biri mutlaka haber verir; tek bir boş satır sorun değildir.

> Not: Kod mantığı değiştirilmedi. Değişiklikleri `cargo check` ile doğrula.
