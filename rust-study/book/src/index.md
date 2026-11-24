# Rust Yolculuğu

Bu kitap, Rust öğrenirken tutulmuş notların baştan sona okunabilecek hâlidir. İlk programdan başlar, sahiplik ve ödünç alma gibi temel kuralları işler, eşzamanlılık ve asenkron programlamayı anlatır, sonunda da küçük bir asenkron çalışma zamanını sıfırdan kurar.

Örneklerin kaynak kodu depodaki `learn/` dizinindedir. Orada her konu ayrı bir Cargo projesidir (crate) ve her dosyanın ilk satırında konunun kitaptaki bölümü yazar. Kitap, her kavramı kısa bir açıklama, çalışan bir örnek, örneğin gerçek çıktısı ve gerektiğinde derleyicinin verdiği hata mesajıyla birlikte anlatır.

## Kitabın üç kısmı

| Kısım | Bölümler | Konular |
| --- | --- | --- |
| **Temeller** | 1–8 | Değişkenler, sahiplik, struct ve enum, modüller, koleksiyonlar, hata yönetimi |
| **Orta seviye** | 9–20 | Jenerikler, trait'ler, ömürler, testler, closure ve iterator, Cargo, akıllı işaretçiler, eşzamanlılık, async, desenler, `unsafe`, makrolar |
| **İleri konular** | 21–29 | Varyans, dinamik boyutlu türler, tutarlılık, metot çözümleme, `Send`/`Sync`, düşürme denetimi, HRTB ve GAT, `Future`'ın yapısı, mini çalışma zamanı |

Zor konularda ana bölümün ardından bir **derinleşme** sayfası gelir. Ana bölüm kavramı tanıtır; derinleşme sayfası aynı kavramı gerçekçi bir senaryoda, sık yapılan hatalarla ve alıştırmalarla pekiştirir.

## Bu kitap kimler için?

Rust'a yeni başlayanlar kitabı baştan sona okuyabilir. Önceden bir programlama dili bilmek işinizi kolaylaştırır; ama ilk bölümler buna dayanmaz.

Rust'ı bilen ama belirli bir konunun içini görmek isteyenler doğrudan ilgili bölüme geçebilir. Bölümler birbirine bağlantı verir; ön bilgi gereken yerde hangi bölüme bakacağınız yazar.

[*The Rust Programming Language*](https://doc.rust-lang.org/book/) ("Rust Book"), dilin resmî kitabıdır ve bu kitabın başlıca kaynağıdır. Konu sırası büyük ölçüde onu izler. İki kitabı yan yana okumak iyi bir yoldur.

## Doğruluk

Kitaptaki teknik bilgiler resmî kaynaklardan doğrulanmış, her kavramın kaynağı dipnotla gösterilmiştir. Rust örnekleri derlenmiş, yazılan çıktılar ve hata mesajları gerçek çalıştırmalardan alınmıştır. Bu denetimin nasıl yapıldığı, hangi sürümlere dayandığı ve sınırları [Kaynaklar ve doğrulama](sources.md) sayfasında anlatılır.

Kitabı ilk kez açıyorsanız [Kitabı nasıl kullanmalı?](how-to-use.md) sayfasıyla başlayın.
