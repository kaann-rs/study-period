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

// Learn more
//
// You can learn how an async runtime works from these sources:
//   - Comprehensive Rust, "Runtimes": https://google.github.io/comprehensive-rust/concurrency/async/runtimes.html
//   - Ibraheem Ahmed, "Learning Async Rust With Entirely Too Many Web Servers": https://ibraheem.ca/posts/too-many-web-servers/
//   - Tokio tutorial, "Async in depth": https://tokio.rs/tokio/tutorial/async
//   - LogRocket, "A practical guide to async in Rust": https://blog.logrocket.com/a-practical-guide-to-async-in-rust/
//   - Easy: Learn by Building, "A Minimal Executor": https://its-saeed.github.io/learn-by-building/async/05-executor.html
