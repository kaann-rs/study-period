// Kitap: 29.3. bölüm, Asenkron API İstemcisi

use std::future::Future;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::runtime::{Runner, block_on, sleep_ms, timeout};

pub fn run() {
    println!("== C1) API istemcisi: eşzamanlı istek + zaman aşımı + yeniden deneme ==");
    show_board();
    println!("\n== C2) Yaygın hata: async içinde thread'i bloklamak ==");
    show_blocking();
    println!("\n== C3) Yaygın hata: kilidi await boyunca tutmak ==");
    show_lock();
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestError {
    Temporary(u16),
    NotFound,
    Timeout,
}

impl RequestError {
    fn is_temporary(&self) -> bool {
        matches!(self, RequestError::Temporary(_))
    }
}

pub struct FakeServer {
    unstable_calls: AtomicU32,
}

impl FakeServer {
    pub fn new() -> Arc<Self> {
        Arc::new(FakeServer {
            unstable_calls: AtomicU32::new(0),
        })
    }
}

pub async fn send_request(
    server: Arc<FakeServer>,
    path: &'static str,
) -> Result<String, RequestError> {
    match path {
        "/hizli" => {
            sleep_ms(20).await;
            Ok(String::from("hızlı cevap"))
        }
        "/yavas" => {
            sleep_ms(400).await;
            Ok(String::from("yavaş cevap"))
        }
        "/kararsiz" => {
            sleep_ms(10).await;
            let call_number = server.unstable_calls.fetch_add(1, Ordering::SeqCst) + 1;
            if call_number <= 2 {
                Err(RequestError::Temporary(503))
            } else {
                Ok(format!("{call_number}. denemede cevap"))
            }
        }
        _ => {
            sleep_ms(5).await;
            Err(RequestError::NotFound)
        }
    }
}

pub async fn retry<F, Fut, T>(max_tries: u32, mut try_once: F) -> Result<T, RequestError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, RequestError>>,
{
    let mut attempt = 1;
    loop {
        match try_once().await {
            Err(error) if error.is_temporary() && attempt < max_tries => {
                sleep_ms(5 * u64::from(attempt)).await;
                attempt += 1;
            }
            result => return result,
        }
    }
}

pub fn load_board() -> ([Result<String, RequestError>; 3], Duration) {
    let server = FakeServer::new();
    let mut runner = Runner::new();
    let spawner = runner.spawner();
    let start = Instant::now();

    let fast = spawner.spawn(send_request(Arc::clone(&server), "/hizli"));

    let unstable = {
        let server = Arc::clone(&server);
        spawner.spawn(retry(4, move || {
            send_request(Arc::clone(&server), "/kararsiz")
        }))
    };

    let slow = spawner.spawn(async move {
        match timeout(Duration::from_millis(60), send_request(server, "/yavas")).await {
            Ok(response) => response,
            Err(_) => Err(RequestError::Timeout),
        }
    });

    runner.run();
    let results =
        [fast, unstable, slow].map(|handle| handle.take_result().expect("görev bitmeliydi"));
    (results, start.elapsed())
}

fn show_board() {
    let (results, duration) = load_board();
    for (name, result) in ["hızlı", "kararsız", "yavaş"].iter().zip(&results) {
        println!("   {name:<8} → {result:?}");
    }
    println!(
        "   toplam süre sıralı toplamdan (125 ms) kısa mı? {}",
        duration < Duration::from_millis(125)
    );
}

async fn wait_badly() {
    thread::sleep(Duration::from_millis(30));
}

async fn wait_well() {
    sleep_ms(30).await;
}

pub async fn run_on_thread<T, F>(job: F) -> T
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    use std::task::{Poll, Waker};
    let slot: Arc<Mutex<(Option<T>, Option<Waker>)>> = Arc::new(Mutex::new((None, None)));
    let worker_slot = Arc::clone(&slot);
    thread::spawn(move || {
        let result = job();
        let waker = {
            let mut lock = worker_slot.lock().unwrap();
            lock.0 = Some(result);
            lock.1.take()
        };
        if let Some(waker) = waker {
            waker.wake();
        }
    });
    std::future::poll_fn(move |cx| {
        let mut lock = slot.lock().unwrap();
        match lock.0.take() {
            Some(result) => Poll::Ready(result),
            None => {
                lock.1 = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    })
    .await
}

fn time_three_tasks<F, Fut>(make_task: F) -> Duration
where
    F: Fn() -> Fut,
    Fut: Future<Output = ()> + Send + 'static,
{
    let mut runner = Runner::new();
    let spawner = runner.spawner();
    let start = Instant::now();
    for _ in 0..3 {
        spawner.spawn(make_task());
    }
    runner.run();
    start.elapsed()
}

fn show_blocking() {
    let limit = Duration::from_millis(90);
    let bad = time_three_tasks(wait_badly);
    let good = time_three_tasks(wait_well);
    let separate = time_three_tasks(|| run_on_thread(|| thread::sleep(Duration::from_millis(30))));
    println!(
        "   thread::sleep ile 3 görev ≥ 90 ms (sıraya girdiler)      : {}",
        bad >= limit
    );
    println!(
        "   async uyku ile 3 görev  < 90 ms (birlikte beklediler)    : {}",
        good < limit
    );
    println!(
        "   bloklayan iş ayrı thread'de < 90 ms (kuyruk tıkanmadı)   : {}",
        separate < limit
    );
}

pub async fn increase_counter(counter: Arc<Mutex<u32>>) -> u32 {
    let new_value = {
        let mut lock = counter.lock().unwrap();
        *lock += 1;
        *lock
    };
    sleep_ms(5).await;
    new_value
}

pub fn run_shared_counter(task_count: u32) -> u32 {
    let counter = Arc::new(Mutex::new(0));
    let mut runner = Runner::new();
    let spawner = runner.spawner();
    for _ in 0..task_count {
        spawner.spawn(increase_counter(Arc::clone(&counter)));
    }
    runner.run();
    *counter.lock().unwrap()
}

fn show_lock() {
    println!("   5 görev sayacı artırdı → {}", run_shared_counter(5));
    println!(
        "   görev içinden sonuç → {}",
        block_on(increase_counter(Arc::new(Mutex::new(41))))
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn board_keeps_each_result_separate() {
        let (results, _) = load_board();
        assert_eq!(results[0], Ok(String::from("hızlı cevap")));
        assert_eq!(results[1], Ok(String::from("3. denemede cevap")));
        assert_eq!(results[2], Err(RequestError::Timeout));
    }

    #[test]
    fn retry_gives_last_error_after_max_tries() {
        let server = FakeServer::new();
        let result = block_on(retry(2, move || {
            send_request(Arc::clone(&server), "/kararsiz")
        }));
        assert_eq!(result, Err(RequestError::Temporary(503)));
    }

    #[test]
    fn permanent_error_is_not_retried() {
        let calls = Arc::new(AtomicU32::new(0));
        let counter = Arc::clone(&calls);
        let server = FakeServer::new();
        let result = block_on(retry(5, move || {
            counter.fetch_add(1, Ordering::SeqCst);
            send_request(Arc::clone(&server), "/yok")
        }));
        assert_eq!(result, Err(RequestError::NotFound));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn blocking_job_returns_result_from_thread() {
        let thread_id = thread::current().id();
        let other_thread = block_on(run_on_thread(move || thread::current().id() != thread_id));
        assert!(other_thread);
    }

    #[test]
    fn all_tasks_go_on_when_lock_is_dropped_before_await() {
        assert_eq!(run_shared_counter(20), 20);
    }
}

// Learn more
//
// You can learn concurrent requests from these sources:
//   - Async Book, "join!": https://rust-lang.github.io/async-book/06_multiple_futures/02_join.html
//   - Comprehensive Rust, "Join": https://google.github.io/comprehensive-rust/concurrency/async-control-flow/join.html
//   - docs.rs, "tokio::task::JoinSet": https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html
//   - Easy: DEV Community, "Async/Await in Rust: A Beginner's Guide": https://dev.to/leapcell/asyncawait-in-rust-a-beginners-guide-237h
//
// You can learn retry with backoff from these sources:
//   - AWS, "Exponential Backoff and Jitter": https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter/
//   - docs.rs, "backon": https://docs.rs/backon/latest/backon/
//   - docs.rs, "tokio_retry": https://docs.rs/tokio-retry/latest/tokio_retry/
//
// You can learn blocking code in async from these sources:
//   - Alice Ryhl, "Async: What is blocking?": https://ryhl.io/blog/async-what-is-blocking/
//   - docs.rs, "tokio::task::spawn_blocking": https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html
//   - Comprehensive Rust, "Blocking the Executor": https://google.github.io/comprehensive-rust/concurrency/async-pitfalls/blocking-executor.html
//   - Easy: Learn by Building, "Sync / Async Bridge": https://its-saeed.github.io/learn-by-building/async/25-sync-async-bridge.html
//
// You can learn locks across await from these sources:
//   - Tokio tutorial, "Holding a MutexGuard across an .await": https://tokio.rs/tokio/tutorial/shared-state#holding-a-mutexguard-across-an-await
//   - docs.rs, "tokio::sync::Mutex": https://docs.rs/tokio/latest/tokio/sync/struct.Mutex.html
//   - sunshowers, "Cancelling async Rust": https://sunshowers.io/posts/cancelling-async-rust/
//   - Easy: Learn by Building, "tokio::sync Internals": https://its-saeed.github.io/learn-by-building/async/18-tokio-sync.html
