# Kitabı nasıl kullanmalı?

## Bir bölümün düzeni

Bütün bölümler aynı sırayı izler:

1. **Giriş.** Konunun hangi sorunu çözdüğü ve bölümde neler bulunduğu.
2. **Günlük hayattan bir benzetme.** Teknik terim kullanmadan ilk fikir.
3. **Bu bölümde geçen terimler.** Kapalı bir kutudur; başlığına tıklayınca açılır. Terimleri baştan ezberlemeniz gerekmez, takıldıkça dönüp bakın.
4. **Kavramlar.** Bölümün gövdesi. Her başlık tek bir kavramı anlatır; kod, çıktı ve hata mesajı kavramın hemen yanındadır.
5. **Hata kodu başvurusu.** Bölümde geçen derleyici hataları ve çözümleri.
6. **Perde arkası.** Derleyicinin konuyu nasıl uyguladığı. İlk okuyuşta atlanabilir.
7. **Özet** ve **Kaynaklar.**

Derinleşme sayfalarında bunlara senaryolar, sık yapılan hatalar ve alıştırmalar eklenir. Alıştırmaların çözümleri kapalı kutulardadır; önce kendiniz deneyin.

## Kod örneklerini okumak

Çalışan bir örneğin ardından `Çıktı:` satırı ve programın gerçek çıktısı gelir:

```rust
fn main() {
    let s1 = String::from("merhaba");
    let s2 = s1;
    println!("{s2}");
}
```

Çıktı:

```text
merhaba
```

Bazı örnekler bilerek hatalıdır; bir kuralın neden var olduğunu en iyi onlar gösterir. Bunların köşesinde **"Bu kod derlenmez"** ya da **"Bu kod çalışırken panikler"** etiketi bulunur. Altlarında derleyicinin gerçek mesajı yer alır:

```rust,compile_fail,E0382
fn main() {
    let s1 = String::from("merhaba");
    let s2 = s1;
    println!("{s1} {s2}");
}
```

Derleyicinin yanıtı:

```text
error[E0382]: borrow of moved value: `s1`
```

Mesajlar kısaltılmış olabilir: Dosya yolu satırı ve konuyla ilgisiz notlar yazılmaz. Köşeli parantez içindeki kodun (`E0382`) uzun açıklamasını `rustc --explain E0382` komutuyla okuyabilirsiniz.

Çıktısı her çalıştırmada değişebilen örneklerde (thread sırası, süre, bellek adresi) "Örnek bir çalıştırma:" yazar; sizdeki sonuç farklı olabilir.

Kod bloklarının sağ üstündeki düğmelerle örneği kopyalayabilir, gizli satırları (çoğunlukla `fn main` sarmalayıcısı) gösterebilir ya da örneği Rust Playground'da çalıştırabilirsiniz. Çalıştırma düğmesi internet bağlantısı ister.

## Kutular

> [!NOTE]
> Ek bilgi, sürüm farkı ya da notların yeri.

> [!TIP]
> İşinizi kolaylaştıracak bir öneri.

> [!WARNING]
> Gözden kaçırılması hataya yol açan bir ayrıntı.

## Kaynakları izlemek

Cümlelerin sonundaki küçük sayılar dipnottur. Tıkladığınızda sayfanın sonundaki kaynağa gidersiniz; oradaki bağlantı resmî belgenin ilgili başlığını ya da derleyicinin kaynak kodundaki ilgili satırı açar. Hangi kaynağın ne olduğu [Kaynaklar ve doğrulama](kaynaklar.md) sayfasında yazar.

## Üç okuma yolu

**Rust'ı ilk kez öğreniyorsanız** 1'den 8'e kadar sırayla okuyun. "Perde arkası" başlıklarını ve derinleşme sayfalarını ikinci okumaya bırakın.

**Bir derleme hatasını çözmeye çalışıyorsanız** sayfanın üstündeki arama kutusuna hata kodunu yazın (örneğin `E0502`). Arama kutusunu klavyede `s` tuşu da açar.

**Bir mekanizmanın içini merak ediyorsanız** derinleşme sayfalarına, "Perde arkası" başlıklarına ve 21'den sonraki bölümlere bakın.

## Notları çalıştırmak

Her bölümün başında ilgili crate'in adı yazar. Depo kökünde şu komutlar işinizi görür:

```console
cargo run -p ownership
cargo test -p ownership
```

Derinleşme sayfaları aynı crate'in `src/derinlesme.rs` dosyasına karşılık gelir. Örnekleri kendi projenizde denemek isterseniz `cargo new deneme` ile yeni bir proje açıp kodu `src/main.rs` dosyasına yapıştırmanız yeterlidir. Yalnızca 13. ve 20. bölümlerdeki gibi birden çok dosya ya da crate gerektiren örnekler ek hazırlık ister; bu, ilgili yerde belirtilir.
