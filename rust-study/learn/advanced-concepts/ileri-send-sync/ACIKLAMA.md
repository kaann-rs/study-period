# Send / Sync'in İç Yüzü

## Aptala anlatır gibi

İç değişkenliğin (interior mutability) temel taşı `UnsafeCell`'dir; `Cell`, `RefCell`, `Mutex` ve atomikler onun üstüne kurulur. `Send` ve `Sync` otomatik trait'lerdir (auto trait): derleyici türün içeriğine bakıp bunları ekler ya da eklemez. `UnsafeCell` tek başına `Sync` değildir; `std`, `Mutex` gibi türlerde `unsafe impl Sync` ile güvenliği ayrıca garanti eder. Bu yüzden `Mutex`, içinde `UnsafeCell` taşımasına rağmen paylaşılabilir.

## 5 yaşındaki çocuğa anlatır gibi

Bir kutuya "başka odaya götürülebilir" etiketi (Send) ya da "birden çok kişi aynı anda bakabilir" etiketi (Sync) yapıştırılır. Etiketi sen yazmazsın; kutunun içine bakıp öğretmen karar verir. İçinde boya olan kutu taşınabilir ama iki kişi aynı anda boyaya dokunmasın diye ikinci etiket verilmez.

> Not: Kod mantığı değiştirilmedi. Değişiklikleri `cargo check` ile doğrula.
