// Kitap: 16.1. bölüm, Async ve Await: Derinleşme

use std::cell::RefCell;
use std::fmt;
use std::future::{Future, poll_fn};
use std::pin::{Pin, pin};
use std::task::{Context, Poll, Waker};

use crate::block_on;

pub fn run() {
    println!("\n== Derinleşme: async/await pratikte ==");
    show_lazy_future();
    show_serial_and_concurrent();
    show_api_client();
    show_daily_pin_use();
}

type Log = RefCell<Vec<String>>;

async fn write_log(log: &Log, message: &str) {
    log.borrow_mut().push(message.to_string());
}

fn show_lazy_future() {
    let log = Log::default();

    let waiting = write_log(&log, "çalıştım");
    println!("çağrıdan sonra kayıt sayısı = {}", log.borrow().len());
    block_on(waiting);
    println!("block_on'dan sonra kayıt sayısı = {}", log.borrow().len());
}

struct Delay {
    remaining: u32,
}

impl Future for Delay {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.remaining == 0 {
            Poll::Ready(())
        } else {
            self.remaining -= 1;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

pub fn run_and_count_rounds<F: Future>(fut: F) -> (F::Output, u32) {
    let mut fut = pin!(fut);
    let mut cx = Context::from_waker(Waker::noop());
    let mut round = 0;
    loop {
        round += 1;
        if let Poll::Ready(value) = fut.as_mut().poll(&mut cx) {
            return (value, round);
        }
    }
}

async fn run_step(log: &Log, name: &str, delay: u32) -> u32 {
    log.borrow_mut().push(format!("{name} başladı"));
    Delay { remaining: delay }.await;
    log.borrow_mut().push(format!("{name} bitti"));
    delay
}

pub async fn wait_both<A: Future, B: Future>(a: A, b: B) -> (A::Output, B::Output) {
    let mut a = pin!(a);
    let mut b = pin!(b);
    let mut a_result = None;
    let mut b_result = None;
    poll_fn(|cx| {
        if a_result.is_none()
            && let Poll::Ready(v) = a.as_mut().poll(cx)
        {
            a_result = Some(v);
        }
        if b_result.is_none()
            && let Poll::Ready(v) = b.as_mut().poll(cx)
        {
            b_result = Some(v);
        }
        match (a_result.take(), b_result.take()) {
            (Some(x), Some(y)) => Poll::Ready((x, y)),
            (x, y) => {
                a_result = x;
                b_result = y;
                Poll::Pending
            }
        }
    })
    .await
}

fn show_serial_and_concurrent() {
    let log = Log::default();
    let (total, round) = run_and_count_rounds(async {
        let a = run_step(&log, "A", 3).await;
        let b = run_step(&log, "B", 2).await;
        a + b
    });
    println!(
        "sıralı    : {:?} → {round} tur (gecikme toplamı {total})",
        log.borrow()
    );

    let log = Log::default();
    let ((a, b), round) =
        run_and_count_rounds(wait_both(run_step(&log, "A", 3), run_step(&log, "B", 2)));
    println!(
        "eşzamanlı : {:?} → {round} tur (gecikme toplamı {})",
        log.borrow(),
        a + b
    );
}

#[derive(Debug, PartialEq, Eq)]
pub enum ApiError {
    NotFound(String),
    ServerError(u16),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::NotFound(path) => write!(f, "bulunamadı: {path}"),
            ApiError::ServerError(code) => write!(f, "sunucu hatası: {code}"),
        }
    }
}

pub async fn fetch(path: &str) -> Result<String, ApiError> {
    let (delay, response) = match path {
        "/kullanici/1" => (2, Ok("Ada")),
        "/kullanici/1/siparisler" => (4, Ok("3 sipariş")),
        "/kullanici/1/oneriler" => (3, Ok("klavye, fare")),
        "/bakim" => (1, Err(ApiError::ServerError(503))),
        _ => (1, Err(ApiError::NotFound(path.to_string()))),
    };
    Delay { remaining: delay }.await;
    response.map(String::from)
}

pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

pub async fn wait_all<T>(mut tasks: Vec<BoxedFuture<'_, T>>) -> Vec<T> {
    let mut results: Vec<Option<T>> = tasks.iter().map(|_| None).collect();
    poll_fn(|cx| {
        for (task, result) in tasks.iter_mut().zip(results.iter_mut()) {
            if result.is_none()
                && let Poll::Ready(v) = task.as_mut().poll(cx)
            {
                *result = Some(v);
            }
        }
        if results.iter().all(Option::is_some) {
            Poll::Ready(
                results
                    .iter_mut()
                    .map(|result| result.take().unwrap())
                    .collect(),
            )
        } else {
            Poll::Pending
        }
    })
    .await
}

pub async fn load_profile_serial(id: u32) -> Result<String, ApiError> {
    let name = fetch(&format!("/kullanici/{id}")).await?;
    let order = fetch(&format!("/kullanici/{id}/siparisler")).await?;
    let suggestion = fetch(&format!("/kullanici/{id}/oneriler")).await?;
    Ok(format!("{name} | {order} | {suggestion}"))
}

pub async fn load_profile_concurrent(id: u32) -> Result<String, ApiError> {
    let paths = [
        format!("/kullanici/{id}"),
        format!("/kullanici/{id}/siparisler"),
        format!("/kullanici/{id}/oneriler"),
    ];
    let tasks: Vec<BoxedFuture<_>> = paths
        .iter()
        .map(|path| Box::pin(fetch(path)) as BoxedFuture<_>)
        .collect();
    let parts = wait_all(tasks)
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;
    Ok(parts.join(" | "))
}

fn show_api_client() {
    let (result, round) = run_and_count_rounds(load_profile_serial(1));
    println!("sıralı istemci    : {result:?} ({round} tur)");
    let (result, round) = run_and_count_rounds(load_profile_concurrent(1));
    println!("eşzamanlı istemci : {result:?} ({round} tur)");
    let (result, _) = run_and_count_rounds(load_profile_concurrent(7));
    match result {
        Ok(profile) => println!("profil: {profile}"),
        Err(e) => println!("hata: {e}"),
    }
    let (result, _) = run_and_count_rounds(fetch("/bakim"));
    println!("bakım: {}", result.unwrap_err());
}

fn show_daily_pin_use() {
    let mut cx = Context::from_waker(Waker::noop());

    let mut on_stack = pin!(fetch("/kullanici/1"));
    let is_first_poll_pending = on_stack.as_mut().poll(&mut cx).is_pending();
    println!("pin! ile ilk poll Pending mi? {is_first_poll_pending}");

    let boxed: BoxedFuture<_> = Box::pin(fetch("/kullanici/1"));
    let mut list = vec![boxed];
    let (results, _) = run_and_count_rounds(wait_all(std::mem::take(&mut list)));
    println!("Box::pin ile listeden: {results:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn future_does_not_run_without_poll() {
        let log = Log::default();
        let fut = write_log(&log, "x");
        assert!(log.borrow().is_empty());
        block_on(fut);
        assert_eq!(*log.borrow(), ["x"]);
    }

    #[test]
    fn dropped_future_never_runs() {
        let log = Log::default();
        drop(write_log(&log, "asla"));
        assert!(log.borrow().is_empty());
    }

    #[test]
    fn serial_wait_adds_times() {
        let log = Log::default();
        let (_, round) = run_and_count_rounds(async {
            run_step(&log, "A", 3).await;
            run_step(&log, "B", 2).await;
        });
        assert_eq!(round, 6);
        assert_eq!(
            *log.borrow(),
            ["A başladı", "A bitti", "B başladı", "B bitti"]
        );
    }

    #[test]
    fn concurrent_wait_takes_as_long_as_the_slowest() {
        let log = Log::default();
        let (result, round) =
            run_and_count_rounds(wait_both(run_step(&log, "A", 3), run_step(&log, "B", 2)));
        assert_eq!(result, (3, 2));
        assert_eq!(round, 4);
        assert_eq!(log.borrow()[..2], ["A başladı", "B başladı"]);
    }

    #[test]
    fn serial_and_concurrent_give_same_result() {
        let (serial, serial_rounds) = run_and_count_rounds(load_profile_serial(1));
        let (concurrent, concurrent_rounds) = run_and_count_rounds(load_profile_concurrent(1));
        assert_eq!(serial, concurrent);
        assert_eq!((serial_rounds, concurrent_rounds), (10, 5));
    }

    #[test]
    fn question_mark_passes_error_up() {
        let (result, _) = run_and_count_rounds(load_profile_serial(9));
        assert_eq!(
            result,
            Err(ApiError::NotFound(String::from("/kullanici/9")))
        );
        assert_eq!(block_on(fetch("/bakim")), Err(ApiError::ServerError(503)));
    }

    #[test]
    fn wait_all_keeps_task_order() {
        let tasks: Vec<BoxedFuture<u32>> = vec![
            Box::pin(async {
                Delay { remaining: 5 }.await;
                1
            }),
            Box::pin(async { 2 }),
        ];
        assert_eq!(block_on(wait_all(tasks)), [1, 2]);
    }
}
