# Generic, Trait ve Lifetime

## Aptala anlatır gibi

Üç araç aynı işe yarar: tekrarı azaltmak. Generic, aynı tarifi farklı türler (type) için bir kez yazmandır; trait, o türlerden beklediğin ortak davranışı (behavior) söyler; lifetime ise ödünç verilen verinin ne kadar süre geçerli olduğunu bildirir. Okuldaki tek form (generic), her öğrencinin atabildiği imza (trait) ve formun bugün mü yarın mı geçerli olduğu (lifetime) gibidir.

## 5 yaşındaki çocuğa anlatır gibi

Bir kâğıt formun var; adını yazınca herkes kullanabilir. Formun üstünde "imza atabilen" yazıyor; yani kalemi olan herkes doldurabilir. Bir de not var: "bu form yalnız bugün geçerli". Süre bitince form işe yaramaz.

> Not: Kod mantığı değiştirilmedi. Değişiklikleri `cargo check` ile doğrula.
