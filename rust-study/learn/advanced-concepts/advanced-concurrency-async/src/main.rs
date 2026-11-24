// Kitap: 29. bölüm, İleri Eşzamanlılık ve Async

mod client;
mod runtime;
mod threads;

fn main() {
    println!("######## A) İLERİ EŞZAMANLILIK (thread'ler) ########");
    threads::run();

    println!("\n######## B) MİNİ ASYNC ÇALIŞMA ZAMANI ########");
    show_runtime();

    println!("\n######## C) UYGULAMA: API İSTEMCİSİ ve YAYGIN HATALAR ########");
    client::run();
}

fn show_runtime() {
    use runtime::{Runner, sleep_ms};

    let mut runner = Runner::new();
    let spawner = runner.spawner();
    let inner_spawner = spawner.clone();

    let root = spawner.spawn(async move {
        let slow = inner_spawner.spawn(async {
            sleep_ms(30).await;
            "yavaş görev (30 ms)"
        });
        let fast = inner_spawner.spawn(async {
            sleep_ms(10).await;
            "hızlı görev (10 ms)"
        });
        [slow.await, fast.await]
    });

    let poll_count = runner.run();
    println!("   sonuçlar = {:?}", root.take_result().unwrap());
    println!("   toplam poll = {poll_count} (3 görev; yalnız uyandırılan poll edildi)");
}
